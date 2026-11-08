use anyhow::{anyhow, Result};
use mlua::{Function, Lua};

use crate::harness::{number_value, read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};

pub struct MluaInterop {
    // The function handle must be dropped before the Lua state.
    entry: Function,
    _lua: Lua,
    result: ResultKind,
}

impl InteropEngine for MluaInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        if case.op != InteropOp::Call {
            return Ok(None);
        }
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

        Ok(Some(Self {
            entry,
            _lua: lua,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        self.entry
            .call::<mlua::Value>(())
            .map_err(|error| anyhow!("{error}"))?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let number: f64 = self
            .entry
            .call(())
            .map_err(|error| anyhow!("{error}"))?;
        number_value(self.result, number)
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

    Ok(())
}
