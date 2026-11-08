use std::rc::Rc;

use anyhow::{anyhow, bail, Result};

mod interop;
pub use interop::RhaiInterop;
use itertools::Itertools;
use rhai::{Array, Dynamic};

use crate::harness::{number_value, read_script, Case, Engine, ResultKind, Value};
use crate::rng;

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustData(Rc<str>);

pub struct RhaiEngine {
    engine: rhai::Engine,
    ast: rhai::AST,
    result: ResultKind,
}

impl Engine for RhaiEngine {
    fn setup(case: &Case) -> Result<Option<Self>> {
        let Some(script_path) = case.scripts.rhai else {
            return Ok(None);
        };
        let mut engine = rhai::Engine::new();
        engine.set_max_call_levels(1000);
        engine.set_max_expr_depths(0, 0);

        engine
            .register_type_with_name::<RustData>("RustData")
            .register_fn("RustData_new", |s: &str| RustData(s.into()))
            .register_fn("to_string", |this: &mut RustData| this.0.to_string())
            .register_fn("<", |l: &mut RustData, r: RustData| *l < r)
            .register_fn("rand", |n: i64| rng::rand_below(n as u32) as i64)
            .register_fn("concat", |items: Array| {
                items
                    .into_iter()
                    .map(|x| x.into_immutable_string().unwrap())
                    .join("")
            });

        let script = read_script(script_path)?;
        let ast = engine
            .compile(script.as_str())
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Some(Self {
            engine,
            ast,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        self.engine
            .eval_ast::<Dynamic>(&self.ast)
            .map_err(|error| anyhow!("{error}"))?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let value = self
            .engine
            .eval_ast::<Dynamic>(&self.ast)
            .map_err(|error| anyhow!("{error}"))?;

        match self.result {
            ResultKind::UserDataList => {
                let array = value.cast::<Array>();
                let mut items = Vec::with_capacity(array.len());
                for next in array {
                    let next = next.cast::<RustData>();
                    items.push(Value::Object(next.0.to_string()));
                }

                Ok(Value::List(items))
            }
            ResultKind::Int | ResultKind::Float => {
                if let Ok(number) = value.as_int() {
                    number_value(self.result, number as f64)
                } else if let Ok(number) = value.as_float() {
                    number_value(self.result, number)
                } else {
                    bail!("expected the script to return a number, got {}", value.type_name())
                }
            }
        }
    }
}
