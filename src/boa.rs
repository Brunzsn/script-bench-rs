use std::rc::Rc;

use anyhow::{anyhow, Result};
use boa_engine::class::{Class, ClassBuilder};
use boa_engine::object::builtins::JsArray;
use boa_engine::property::Attribute;
use boa_engine::value::TryFromJs;
use boa_engine::{
    js_string, Context, Finalize, JsData, JsResult, JsString, JsValue, NativeFunction, Source,
    Trace,
};
use boa_runtime::Console;

use crate::harness::{number_value, read_script, Case, Engine, ResultKind, Value};
use crate::rng;

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize, JsData)]
pub struct RustData(Rc<str>);

impl Class for RustData {
    const NAME: &'static str = "RustData";
    const LENGTH: usize = 1;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        let kind = args[0].to_string(context)?;
        let s = kind.to_std_string().unwrap();
        Ok(RustData(s.into()))
    }

    fn init(class: &mut ClassBuilder) -> JsResult<()> {
        class.method(
            js_string!("toString"),
            0,
            NativeFunction::from_fn_ptr(|this, _args, _ctx| {
                let this = this.as_object().unwrap();
                let this = this.downcast_ref::<Self>().unwrap();
                Ok(JsString::from(&*this.0).into())
            }),
        );

        class.method(
            js_string!("lt"),
            1,
            NativeFunction::from_fn_ptr(|this, args, _ctx| {
                let this = this.as_object().unwrap();
                let this = this.downcast_ref::<Self>().unwrap();
                let other = args[0].as_object().unwrap();
                let other = other.downcast_ref::<Self>().unwrap();
                Ok((*this < *other).into())
            }),
        );

        Ok(())
    }
}

pub struct BoaEngine {
    context: Context,
    bench: JsValue,
    result: ResultKind,
}

impl Engine for BoaEngine {
    fn setup(case: &Case) -> Result<Option<Self>> {
        let Some(script_path) = case.scripts.js else {
            return Ok(None);
        };
        let mut context = Context::default();

        let console = Console::init(&mut context);
        context
            .register_global_property(js_string!(Console::NAME), console, Attribute::all())
            .map_err(|error| anyhow!("{error}"))?;

        context
            .register_global_class::<RustData>()
            .map_err(|error| anyhow!("{error}"))?;
        context
            .register_global_builtin_callable(
                js_string!("rand"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, ctx| {
                    let n = args[0].to_u32(ctx)?;
                    Ok(rng::rand_below(n).into())
                }),
            )
            .map_err(|error| anyhow!("{error}"))?;

        let script = read_script(script_path)?;
        let source = Source::from_bytes(&script);
        context.eval(source).map_err(|error| anyhow!("{error}"))?;

        let bench = context
            .global_object()
            .get(js_string!("bench"), &mut context)
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Some(Self {
            context,
            bench,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        let Some(bench) = self.bench.as_callable() else {
            return Err(anyhow!("`bench` is not callable"));
        };
        bench
            .call(&self.bench, &[], &mut self.context)
            .map_err(|error| anyhow!("{error}"))?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let value = {
            let Some(bench) = self.bench.as_callable() else {
                return Err(anyhow!("`bench` is not callable"));
            };
            bench
                .call(&self.bench, &[], &mut self.context)
                .map_err(|error| anyhow!("{error}"))?
        };

        match self.result {
            ResultKind::UserDataList => {
                let array: JsArray = TryFromJs::try_from_js(&value, &mut self.context)
                    .map_err(|error| anyhow!("expected the script to return an array: {error}"))?;
                let length = array
                    .length(&mut self.context)
                    .map_err(|error| anyhow!("{error}"))?;

                let mut items = Vec::with_capacity(length as usize);
                for index in 0..length {
                    let item = array
                        .get(index, &mut self.context)
                        .map_err(|error| anyhow!("{error}"))?;
                    let object = item
                        .as_object()
                        .ok_or_else(|| anyhow!("expected a RustData object"))?;
                    let data = object
                        .downcast_ref::<RustData>()
                        .ok_or_else(|| anyhow!("expected a RustData object"))?;
                    items.push(Value::Object(data.0.to_string()));
                }

                Ok(Value::List(items))
            }
            ResultKind::Int | ResultKind::Float => {
                let number = value
                    .to_number(&mut self.context)
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, number)
            }
        }
    }
}
