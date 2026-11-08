use anyhow::{anyhow, Result};
use rquickjs::{Context, Ctx, Function, Persistent, Result as JsResult, Runtime};

use crate::harness::{number_value, read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};

pub struct QuickJsInterop {
    // The function handle must be dropped before the context and runtime.
    entry: Persistent<Function<'static>>,
    context: Context,
    _runtime: Runtime,
    result: ResultKind,
}

impl InteropEngine for QuickJsInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        if case.op != InteropOp::Call {
            return Ok(None);
        }
        let Some(script_path) = case.scripts.js else {
            return Ok(None);
        };
        let script = read_script(script_path)?;

        let runtime = Runtime::new().map_err(|error| anyhow!("{error}"))?;
        let context = Context::full(&runtime).map_err(|error| anyhow!("{error}"))?;

        let entry = context
            .with(|ctx| -> JsResult<Persistent<Function<'static>>> {
                register_host_functions(&ctx)?;
                ctx.eval::<(), _>(script.as_str())?;
                let func = ctx.globals().get::<_, Function>(case.entry)?;
                Ok(Persistent::save(&ctx, func))
            })
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Some(Self {
            entry,
            context,
            _runtime: runtime,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        let entry = self.entry.clone();
        self.context
            .with(|ctx| {
                let func = entry.restore(&ctx)?;
                func.call::<_, ()>(())
            })
            .map_err(|error| anyhow!("{error}"))?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let entry = self.entry.clone();
        let number = self
            .context
            .with(|ctx| {
                let func = entry.restore(&ctx)?;
                func.call::<_, f64>(())
            })
            .map_err(|error| anyhow!("{error}"))?;
        number_value(self.result, number)
    }
}

fn register_host_functions(ctx: &Ctx<'_>) -> JsResult<()> {
    let globals = ctx.globals();

    let ping = Function::new(ctx.clone(), || host::ping() as f64)?.with_name("interop_ping")?;
    globals.set("interop_ping", ping)?;

    let id_i64 = Function::new(ctx.clone(), |value: f64| host::id_i64(value as i64) as f64)?
        .with_name("interop_id_i64")?;
    globals.set("interop_id_i64", id_i64)?;

    let sum3 = Function::new(ctx.clone(), |a: f64, b: f64, c: f64| {
        host::sum_i64_3(a as i64, b as i64, c as i64) as f64
    })?
    .with_name("interop_sum_i64_3")?;
    globals.set("interop_sum_i64_3", sum3)?;

    let sum5 = Function::new(ctx.clone(), |a: f64, b: f64, c: f64, d: f64, e: f64| {
        host::sum_i64_5(a as i64, b as i64, c as i64, d as i64, e as i64) as f64
    })?
    .with_name("interop_sum_i64_5")?;
    globals.set("interop_sum_i64_5", sum5)?;

    let id_f64 =
        Function::new(ctx.clone(), |value: f64| host::id_f64(value))?.with_name("interop_id_f64")?;
    globals.set("interop_id_f64", id_f64)?;

    let strlen = Function::new(ctx.clone(), |value: String| host::strlen(&value) as f64)?
        .with_name("interop_strlen")?;
    globals.set("interop_strlen", strlen)?;

    let string_out = Function::new(ctx.clone(), || host::string_out())?
        .with_name("interop_string_out")?;
    globals.set("interop_string_out", string_out)?;

    Ok(())
}
