use anyhow::{anyhow, bail, Result};
use boa_engine::{
    js_string, JsString, JsValue, NativeFunction, Source,
};

use crate::harness::{number_value, read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};

pub struct BoaInterop {
    context: boa_engine::Context,
    entry: JsValue,
    result: ResultKind,
}

impl InteropEngine for BoaInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        if case.op != InteropOp::Call {
            return Ok(None);
        }
        let Some(script_path) = case.scripts.js else {
            return Ok(None);
        };

        let mut context = boa_engine::Context::default();
        register_host_functions(&mut context)?;

        let script = read_script(script_path)?;
        context
            .eval(Source::from_bytes(&script))
            .map_err(|error| anyhow!("{error}"))?;

        let entry = context
            .global_object()
            .get(JsString::from(case.entry), &mut context)
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Some(Self {
            context,
            entry,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        let Some(func) = self.entry.as_callable() else {
            bail!("`{}` is not callable", "interop entry");
        };
        func.call(&self.entry, &[], &mut self.context)
            .map_err(|error| anyhow!("{error}"))?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let Some(func) = self.entry.as_callable() else {
            bail!("interop entry is not callable");
        };
        let value = func
            .call(&self.entry, &[], &mut self.context)
            .map_err(|error| anyhow!("{error}"))?;
        let number = value
            .to_number(&mut self.context)
            .map_err(|error| anyhow!("{error}"))?;
        number_value(self.result, number)
    }
}

fn register_host_functions(context: &mut boa_engine::Context) -> Result<()> {
    let mut register = |name: &str, length: usize, function: NativeFunction| -> Result<()> {
        context
            .register_global_builtin_callable(JsString::from(name), length, function)
            .map_err(|error| anyhow!("{error}"))
    };

    register(
        "interop_ping",
        0,
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::from(host::ping()))),
    )?;
    register(
        "interop_id_i64",
        1,
        NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let value = args[0].to_number(ctx)? as i64;
            Ok(JsValue::from(host::id_i64(value)))
        }),
    )?;
    register(
        "interop_sum_i64_3",
        3,
        NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let a = args[0].to_number(ctx)? as i64;
            let b = args[1].to_number(ctx)? as i64;
            let c = args[2].to_number(ctx)? as i64;
            Ok(JsValue::from(host::sum_i64_3(a, b, c)))
        }),
    )?;
    register(
        "interop_sum_i64_5",
        5,
        NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let a = args[0].to_number(ctx)? as i64;
            let b = args[1].to_number(ctx)? as i64;
            let c = args[2].to_number(ctx)? as i64;
            let d = args[3].to_number(ctx)? as i64;
            let e = args[4].to_number(ctx)? as i64;
            Ok(JsValue::from(host::sum_i64_5(a, b, c, d, e)))
        }),
    )?;
    register(
        "interop_id_f64",
        1,
        NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let value = args[0].to_number(ctx)?;
            Ok(JsValue::from(host::id_f64(value)))
        }),
    )?;
    register(
        "interop_strlen",
        1,
        NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let value = args[0].to_string(ctx)?;
            let value = value.to_std_string().unwrap();
            Ok(JsValue::from(host::strlen(&value)))
        }),
    )?;
    register(
        "interop_string_out",
        0,
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsString::from(host::string_out()).into())
        }),
    )?;

    Ok(())
}
