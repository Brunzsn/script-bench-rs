use anyhow::{anyhow, bail, Result};
use mlua::{Function, Lua};

use crate::harness::{number_value, read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};

enum MluaMode {
    Call,
    CallIn { iterations: usize },
    Callback { iterations: usize },
    Error { propagates: bool },
}

pub struct MluaInterop {
    // The function handle must be dropped before the Lua state.
    entry: Function,
    _lua: Lua,
    mode: MluaMode,
    result: ResultKind,
}

impl InteropEngine for MluaInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        let mode = match case.op {
            InteropOp::Call => MluaMode::Call,
            InteropOp::CallIn { iterations } => MluaMode::CallIn { iterations },
            InteropOp::Callback { iterations } => MluaMode::Callback { iterations },
            InteropOp::Error(op) => MluaMode::Error {
                propagates: matches!(op, crate::interop::ErrorOp::PropagatesToHost),
            },
            _ => return Ok(None),
        };
        let is_callback = matches!(mode, MluaMode::Callback { .. });
        let Some(script_path) = case.scripts.lua else {
            return Ok(None);
        };
        let lua = Lua::new();
        register_host_functions(&lua)?;

        #[cfg(feature = "mlua_luau")]
        {
            lua.sandbox(true).map_err(|error| anyhow!("{error}"))?;
            lua.set_jit_options(mlua::state::JitOptions::new().inliner(true));
            lua.set_compiler(mlua::chunk::Compiler::new().set_optimization_level(2));
        }

        let script = read_script(script_path)?;
        lua.load(script.as_str())
            .exec()
            .map_err(|error| anyhow!("{error}"))?;

        let entry = lua
            .globals()
            .get::<Function>(case.entry)
            .map_err(|error| anyhow!("{error}"))?;
        let entry = if is_callback {
            entry.call::<Function>(()).map_err(|error| anyhow!("{error}"))?
        } else {
            entry
        };

        Ok(Some(Self {
            entry,
            _lua: lua,
            mode,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        match self.mode {
            MluaMode::Call => {
                self.entry
                    .call::<mlua::Value>(())
                    .map_err(|error| anyhow!("{error}"))?;
            }
            MluaMode::CallIn { iterations } => {
                let mut sum = 0.0f64;
                for _ in 0..iterations {
                    let value: f64 = self
                        .entry
                        .call(host::UPDATE_DT)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum += value;
                }
                std::hint::black_box(sum);
            }
            MluaMode::Callback { iterations } => {
                let mut sum = 0i64;
                for index in 0..iterations {
                    let value: i64 = self
                        .entry
                        .call(index as i64)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(value);
                }
                std::hint::black_box(sum);
            }
            MluaMode::Error { propagates } => {
                let result = self.entry.call::<f64>(());
                match result {
                    Ok(value) => {
                        if propagates {
                            bail!("expected the script to raise an error");
                        }
                        std::hint::black_box(value);
                    }
                    Err(_) if propagates => {}
                    Err(error) => return Err(anyhow!("{error}")),
                }
            }
        }
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        match self.mode {
            MluaMode::Call => {
                let number: f64 = self
                    .entry
                    .call(())
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, number)
            }
            MluaMode::CallIn { iterations } => {
                let mut sum = 0.0f64;
                for _ in 0..iterations {
                    let value: f64 = self
                        .entry
                        .call(host::UPDATE_DT)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum += value;
                }
                Ok(Value::Float(sum))
            }
            MluaMode::Callback { iterations } => {
                let mut sum = 0i64;
                for index in 0..iterations {
                    let value: i64 = self
                        .entry
                        .call(index as i64)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(value);
                }
                Ok(Value::Int(sum))
            }
            MluaMode::Error { propagates } => {
                let result = self.entry.call::<f64>(());
                match result {
                    Ok(value) => {
                        if propagates {
                            bail!("expected the script to raise an error");
                        }
                        number_value(self.result, value)
                    }
                    Err(_) if propagates => Ok(Value::Int(1)),
                    Err(error) => Err(anyhow!("{error}")),
                }
            }
        }
    }
}

fn register_host_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals
        .set("interop_ping", Function::wrap_raw(|| host::ping()))
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_id_i64",
            Function::wrap_raw(|value: i64| host::id_i64(value)),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_sum_i64_3",
            Function::wrap_raw(|a: i64, b: i64, c: i64| host::sum_i64_3(a, b, c)),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_sum_i64_5",
            Function::wrap_raw(|a: i64, b: i64, c: i64, d: i64, e: i64| {
                host::sum_i64_5(a, b, c, d, e)
            }),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_id_f64",
            Function::wrap_raw(|value: f64| host::id_f64(value)),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_strlen",
            Function::wrap_raw(|value: String| host::strlen(&value)),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_string_out",
            Function::wrap_raw(|| host::string_out()),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_callback",
            Function::wrap_raw(|value: i64| host::callback_value(value)),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_fail",
            Function::wrap(|| -> mlua::Result<i64> {
                Err(mlua::Error::RuntimeError(host::HOST_ERROR.into()))
            }),
        )
        .map_err(|error| anyhow!("{error}"))?;

    Ok(())
}
