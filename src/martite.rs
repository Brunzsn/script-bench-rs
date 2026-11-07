use anyhow::{anyhow, bail, Result};
use martite::{Engine as Martite, HostSignature, HostType, Instance, Outcome, Plugin};

use crate::harness::{read_script, Case, Engine, Value as BenchValue};
use crate::rng;

pub use martite::HostValue as Value;

const LIMITS: martite::Limits = martite::Limits {
    fuel: 1_000_000_000,
    memory: 1 << 30,
    depth: 1_000,
    deadline: None,
};

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

pub struct MartiteEngine {
    instance: Instance<'static>,
}

impl Engine for MartiteEngine {
    fn setup(case: &Case) -> Result<Self> {
        let mut engine = Martite::new();
        engine.set_limits(LIMITS, LIMITS);
        let interface = read_script(case.scripts.martite_interface)?;
        engine.interface("bench.data", interface);

        engine.host_fn(
            "bench.data.rand",
            HostSignature::new(vec![HostType::Int], HostType::Int),
            |_cx, args| {
                let n = args[0].as_int().expect("rand expects an int");
                Ok(Value::Int(rng::rand_below(n as u32) as i64))
            },
        );

        engine.host_fn(
            "bench.data.rust_data_new",
            HostSignature::new(vec![HostType::Str], HostType::Handle("RustData".into())),
            |cx, args| {
                let Value::Str(key) = &args[0] else {
                    panic!("rust_data_new expects a string");
                };
                cx.make_handle(key.to_string())
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

        let script = read_script(case.scripts.martite_script)?;
        let provider = martite::memory_provider(&[("main.mt", script.as_str())]);
        let plugin = engine.load(MANIFEST, &provider).map_err(|diagnostics| {
            anyhow!("Martite load failed: {} diagnostics", diagnostics.len())
        })?;

        // `Instance` borrows its `Plugin`, and re-instantiating per iteration
        // would change what the benchmark measures. The process is short-lived,
        // so keep the plugin alive for its duration.
        let plugin: &'static Plugin = Box::leak(Box::new(plugin));

        let instance = plugin
            .instantiate()
            .map_err(|trap| anyhow!("Martite instantiate failed: {}", trap.panic_message()))?;
        Ok(Self { instance })
    }

    fn run(&mut self) -> Result<()> {
        self.instance
            .call("run", Vec::new())
            .map_err(|trap| anyhow!("Martite run failed: {}", trap.panic_message()))?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<BenchValue> {
        let outcome = self
            .instance
            .call("run", Vec::new())
            .map_err(|trap| anyhow!("Martite run failed: {}", trap.panic_message()))?;

        match outcome {
            Outcome::Handled(Value::List(items)) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items.iter() {
                    let Value::Handle(handle) = item else {
                        bail!("expected a RustData handle");
                    };
                    values.push(BenchValue::Object(handle.token.clone()));
                }
                Ok(BenchValue::List(values))
            }
            Outcome::Handled(_) => bail!("expected `run` to return a list"),
            Outcome::NoHandler => bail!("Martite plugin has no `run` handler"),
        }
    }
}
