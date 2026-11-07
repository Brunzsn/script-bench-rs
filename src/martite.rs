use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{anyhow, bail, Result};
use martite::{Engine, HostSignature, HostType, Outcome};
use rand::RngExt;

pub use martite::HostValue as Value;

const COUNT: usize = 10_000;

const LIMITS: martite::Limits = martite::Limits {
    fuel: 1_000_000_000,
    memory: 1 << 30,
    depth: 1_000,
    deadline: None,
};

const INTERFACE: &str = include_str!("../scripts/sort_userdata.mti");

const MANIFEST: &str = r#"
[plugin]
name = "martite-sort"
version = "0.1.0"
lang = "1"
api = "1.0"
entry = "main.mt"

[permissions]
grant = ["bench.data"]
"#;

const SCRIPT: &str = include_str!("../scripts/sort_userdata.mt");

pub fn sort_userdata(
    run: impl FnOnce(&mut dyn FnMut()),
    validate: impl FnOnce(Value),
) -> Result<()> {
    let pool: Rc<RefCell<Vec<Value>>> = Rc::new(RefCell::new(Vec::with_capacity(COUNT)));

    let mut engine = Engine::new();
    engine.set_limits(LIMITS, LIMITS);
    engine.interface("bench.data", INTERFACE);

    engine.host_fn(
        "bench.data.rand",
        HostSignature::new(vec![HostType::Int], HostType::Int),
        |_cx, args| {
            let n = args[0].as_int().expect("rand expects an int");
            Ok(Value::Int(rand::rng().random_range(0..n)))
        },
    );

    engine.host_fn(
        "bench.data.rust_data_new",
        HostSignature::new(vec![HostType::Str], HostType::Handle("RustData".into())),
        {
            let pool = Rc::clone(&pool);
            move |_cx, args| {
                let Value::Str(key) = &args[0] else {
                    panic!("rust_data_new expects a string");
                };
                let mut value = pool.borrow_mut().pop().expect("RustData handle pool");
                let Value::Handle(handle) = &mut value else {
                    unreachable!("handle pool contains a non-handle");
                };
                Rc::get_mut(handle).expect("unique RustData handle").token = key.to_string();
                Ok(value)
            }
        },
    );

    engine.host_fn(
        "bench.data.rust_data_lt",
        HostSignature::new(
            vec![
                HostType::Handle("RustData".into()),
                HostType::Handle("RustData".into()),
            ],
            HostType::Bool,
        ),
        |_cx, args| {
            let (Value::Handle(left), Value::Handle(right)) = (&args[0], &args[1]) else {
                panic!("rust_data_lt expects two handles");
            };
            Ok(Value::Bool(left.token.as_str() < right.token.as_str()))
        },
    );

    let provider = martite::memory_provider(&[("main.mt", SCRIPT)]);
    let plugin = engine
        .load(MANIFEST, &provider)
        .map_err(|diagnostics| anyhow!("Martite load failed: {} diagnostics", diagnostics.len()))?;

    // Opaque handles are created by the embedding, not by host functions, so
    // pre-create one shell per script object and fill in its token on demand.
    let fill = {
        let pool = Rc::clone(&pool);
        let plugin = &plugin;
        move || {
            let mut pool = pool.borrow_mut();
            pool.clear();
            for _ in 0..COUNT {
                pool.push(
                    plugin
                        .make_handle("RustData", "")
                        .expect("RustData is an opaque interface handle"),
                );
            }
        }
    };

    fill();
    let mut instance = plugin
        .instantiate()
        .map_err(|trap| anyhow!("Martite instantiate failed: {}", trap.panic_message()))?;

    let first = match instance
        .call("run", Vec::new())
        .map_err(|trap| anyhow!("Martite run failed: {}", trap.panic_message()))?
    {
        Outcome::Handled(value) => value,
        Outcome::NoHandler => bail!("Martite plugin has no `run` handler"),
    };
    validate(first);

    run(&mut || {
        fill();
        let _ = instance.call("run", Vec::new()).expect("Martite run");
    });

    Ok(())
}
