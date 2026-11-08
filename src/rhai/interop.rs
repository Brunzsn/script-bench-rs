use anyhow::{anyhow, bail, Result};

use crate::harness::{read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};

enum RhaiMode {
    Call,
    CallIn { iterations: usize },
    Callback {
        callback: rhai::FnPtr,
        iterations: usize,
    },
    Error { propagates: bool },
}

pub struct RhaiInterop {
    engine: rhai::Engine,
    ast: rhai::AST,
    scope: rhai::Scope<'static>,
    entry: &'static str,
    mode: RhaiMode,
    result: ResultKind,
}

impl InteropEngine for RhaiInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        if !matches!(
            case.op,
            InteropOp::Call
                | InteropOp::CallIn { .. }
                | InteropOp::Callback { .. }
                | InteropOp::Error(_)
        ) {
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

        let mut scope = rhai::Scope::new();
        let mode = match case.op {
            InteropOp::Call => RhaiMode::Call,
            InteropOp::CallIn { iterations } => RhaiMode::CallIn { iterations },
            InteropOp::Callback { iterations } => {
                let callback: rhai::FnPtr = engine
                    .call_fn(&mut scope, &ast, case.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
                RhaiMode::Callback {
                    callback,
                    iterations,
                }
            }
            InteropOp::Error(op) => RhaiMode::Error {
                propagates: matches!(op, crate::interop::ErrorOp::PropagatesToHost),
            },
            _ => return Ok(None),
        };

        Ok(Some(Self {
            engine,
            ast,
            scope,
            entry: case.entry,
            mode,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        match &self.mode {
            RhaiMode::Call => match self.result {
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
            },
            RhaiMode::CallIn { iterations } => {
                let mut sum = 0.0f64;
                for _ in 0..*iterations {
                    let value: f64 = self
                        .engine
                        .call_fn(
                            &mut self.scope,
                            &self.ast,
                            self.entry,
                            (host::UPDATE_DT,),
                        )
                        .map_err(|error| anyhow!("{error}"))?;
                    sum += value;
                }
                std::hint::black_box(sum);
            }
            RhaiMode::Callback {
                callback,
                iterations,
            } => {
                let mut sum = 0i64;
                for index in 0..*iterations {
                    let value: i64 = callback
                        .call(&self.engine, &self.ast, (index as i64,))
                        .map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(value);
                }
                std::hint::black_box(sum);
            }
            RhaiMode::Error { propagates } => {
                let result: Result<i64, _> =
                    self.engine.call_fn(&mut self.scope, &self.ast, self.entry, ());
                match result {
                    Ok(value) => {
                        if *propagates {
                            bail!("expected the script to raise an error");
                        }
                        std::hint::black_box(value);
                    }
                    Err(_) if *propagates => {}
                    Err(error) => return Err(anyhow!("{error}")),
                }
            }
        }
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        match &self.mode {
            RhaiMode::Call => match self.result {
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
            },
            RhaiMode::CallIn { iterations } => {
                let mut sum = 0.0f64;
                for _ in 0..*iterations {
                    let value: f64 = self
                        .engine
                        .call_fn(
                            &mut self.scope,
                            &self.ast,
                            self.entry,
                            (host::UPDATE_DT,),
                        )
                        .map_err(|error| anyhow!("{error}"))?;
                    sum += value;
                }
                Ok(Value::Float(sum))
            }
            RhaiMode::Callback {
                callback,
                iterations,
            } => {
                let mut sum = 0i64;
                for index in 0..*iterations {
                    let value: i64 = callback
                        .call(&self.engine, &self.ast, (index as i64,))
                        .map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(value);
                }
                Ok(Value::Int(sum))
            }
            RhaiMode::Error { propagates } => {
                let result: Result<i64, _> =
                    self.engine.call_fn(&mut self.scope, &self.ast, self.entry, ());
                match result {
                    Ok(value) => {
                        if *propagates {
                            bail!("expected the script to raise an error");
                        }
                        Ok(Value::Int(value))
                    }
                    Err(_) if *propagates => Ok(Value::Int(1)),
                    Err(error) => Err(anyhow!("{error}")),
                }
            }
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
        .register_fn("interop_string_out", || host::string_out())
        .register_fn("interop_callback", |value: i64| host::callback_value(value))
        .register_fn(
            "interop_fail",
            || -> Result<i64, Box<rhai::EvalAltResult>> {
                Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    host::HOST_ERROR.into(),
                    rhai::Position::NONE,
                )))
            },
        );
}
