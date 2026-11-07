use std::rc::Rc;

use anyhow::{anyhow, Result};
use rand::RngExt;
use rquickjs::class::{Class, JsClass, Readable, Trace, Tracer};
use rquickjs::function::{Constructor, This};
use rquickjs::{
    Array, Context, Ctx, FromJs, Function, IntoJs, JsLifetime, Object, Persistent,
    Result as JsResult, Runtime, Value as JsValue,
};

use crate::harness::{read_script, Case, Engine, Value};

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustData(Rc<str>);

impl<'js> Trace<'js> for RustData {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> FromJs<'js> for RustData {
    fn from_js(_: &Ctx<'js>, value: JsValue<'js>) -> JsResult<Self> {
        Ok(Class::<Self>::from_value(&value)?.try_borrow()?.clone())
    }
}

impl<'js> IntoJs<'js> for RustData {
    fn into_js(self, ctx: &Ctx<'js>) -> JsResult<JsValue<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

unsafe impl<'js> JsLifetime<'js> for RustData {
    type Changed<'to> = RustData;
}

impl<'js> JsClass<'js> for RustData {
    const NAME: &'static str = "RustData";

    type Mutable = Readable;

    fn prototype(ctx: &Ctx<'js>) -> JsResult<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;

        let to_string = Function::new(ctx.clone(), |this: This<Self>| this.0 .0.to_string())?
            .with_name("toString")?;
        proto.prop("toString", to_string)?;

        let lt = Function::new(ctx.clone(), |this: This<Self>, other: Self| this.0 < other)?
            .with_name("lt")?;
        proto.prop("lt", lt)?;

        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> JsResult<Option<Constructor<'js>>> {
        Ok(Some(Constructor::new_class::<Self, _, _>(
            ctx.clone(),
            |s: String| RustData(s.into()),
        )?))
    }
}

pub struct QuickJsEngine {
    // Persistent handles must be dropped before the context and runtime.
    bench: Persistent<Function<'static>>,
    context: Context,
    _runtime: Runtime,
}

impl Engine for QuickJsEngine {
    fn setup(case: &Case) -> Result<Self> {
        let script = read_script(case.scripts.js)?;

        let runtime = Runtime::new().map_err(|error| anyhow!("{error}"))?;
        let context = Context::full(&runtime).map_err(|error| anyhow!("{error}"))?;

        let bench = context
            .with(|ctx| -> JsResult<Persistent<Function<'static>>> {
                let globals = ctx.globals();

                let rand =
                    Function::new(ctx.clone(), |n: u32| rand::rng().random_range(0..n))?
                        .with_name("rand")?;
                globals.set("rand", rand)?;

                Class::<RustData>::define(&globals)?;

                ctx.eval::<(), _>(script.as_str())?;

                let func = globals.get::<_, Function>("bench")?;
                Ok(Persistent::save(&ctx, func))
            })
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Self {
            bench,
            context,
            _runtime: runtime,
        })
    }

    fn run(&mut self) -> Result<()> {
        let bench = self.bench.clone();
        self.context
            .with(|ctx| {
                let func = bench.restore(&ctx)?;
                func.call::<_, ()>(())
            })
            .map_err(|error| anyhow!("{error}"))?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let bench = self.bench.clone();
        self.context
            .with(|ctx| -> JsResult<Value> {
                let func = bench.restore(&ctx)?;
                let array = func.call::<_, Array>(())?;

                let mut items = Vec::new();
                for next in array.iter::<RustData>() {
                    let next = next?;
                    items.push(Value::Object(next.0.to_string()));
                }

                Ok(Value::List(items))
            })
            .map_err(|error| anyhow!("{error}"))
    }
}
