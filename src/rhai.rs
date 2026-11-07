use std::rc::Rc;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use rand::RngExt;
use rhai::Array;

use crate::harness::{read_script, Case, Engine, Value};

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustData(Rc<str>);

pub struct RhaiEngine {
    engine: rhai::Engine,
    ast: rhai::AST,
}

impl Engine for RhaiEngine {
    fn setup(case: &Case) -> Result<Self> {
        let mut engine = rhai::Engine::new();
        engine.set_max_call_levels(1000);
        engine.set_max_expr_depths(0, 0);

        engine
            .register_type_with_name::<RustData>("RustData")
            .register_fn("RustData_new", |s: &str| RustData(s.into()))
            .register_fn("to_string", |this: &mut RustData| this.0.to_string())
            .register_fn("<", |l: &mut RustData, r: RustData| *l < r)
            .register_fn("rand", |n: i64| rand::rng().random_range(0..n))
            .register_fn("concat", |items: Array| {
                items
                    .into_iter()
                    .map(|x| x.into_immutable_string().unwrap())
                    .join("")
            });

        let script = read_script(case.scripts.rhai)?;
        let ast = engine
            .compile(script.as_str())
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Self { engine, ast })
    }

    fn run(&mut self) -> Result<()> {
        self.engine
            .eval_ast::<Array>(&self.ast)
            .map_err(|error| anyhow!("{error}"))?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let array = self
            .engine
            .eval_ast::<Array>(&self.ast)
            .map_err(|error| anyhow!("{error}"))?;

        let mut items = Vec::with_capacity(array.len());
        for next in array {
            let next = next.cast::<RustData>();
            items.push(Value::Object(next.0.to_string()));
        }

        Ok(Value::List(items))
    }
}
