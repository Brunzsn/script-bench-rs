use anyhow::{anyhow, bail, Result};

use crate::harness::{read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};

pub struct RhaiInterop {
    engine: rhai::Engine,
    ast: rhai::AST,
    scope: rhai::Scope<'static>,
    entry: &'static str,
    result: ResultKind,
}

impl InteropEngine for RhaiInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        if case.op != InteropOp::Call {
            return Ok(None);
        }
        let Some(script_path) = case.scripts.rhai else {
            return Ok(None);
        };

        let mut engine = rhai::Engine::new();
        engine.set_max_call_levels(1000);
        engine.set_max_expr_depths(0, 0);
        register_host_functions(&mut engine);

        let script = read_script(script_path)?;
        let ast = engine
            .compile(script.as_str())
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Some(Self {
            engine,
            ast,
            scope: rhai::Scope::new(),
            entry: case.entry,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        match self.result {
            ResultKind::Int => {
                let _: i64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
            }
            ResultKind::Float => {
                let _: f64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
            }
            ResultKind::UserDataList => bail!("interop cases cannot return userdata lists"),
        }
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        match self.result {
            ResultKind::Int => {
                let value: i64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
                Ok(Value::Int(value))
            }
            ResultKind::Float => {
                let value: f64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
                Ok(Value::Float(value))
            }
            ResultKind::UserDataList => bail!("interop cases cannot return userdata lists"),
        }
    }
}

fn register_host_functions(engine: &mut rhai::Engine) {
    engine
        .register_fn("interop_ping", || host::ping())
        .register_fn("interop_id_i64", |value: i64| host::id_i64(value))
        .register_fn("interop_sum_i64_3", |a: i64, b: i64, c: i64| {
            host::sum_i64_3(a, b, c)
        })
        .register_fn("interop_sum_i64_5", |a: i64, b: i64, c: i64, d: i64, e: i64| {
            host::sum_i64_5(a, b, c, d, e)
        })
        .register_fn("interop_id_f64", |value: f64| host::id_f64(value))
        .register_fn("interop_strlen", |value: String| host::strlen(&value))
        .register_fn("interop_string_out", || host::string_out());
}
