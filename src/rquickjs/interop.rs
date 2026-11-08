use std::cell::Cell;
use std::rc::Rc;

use anyhow::{anyhow, bail, Result};
use rquickjs::class::{Class, JsClass, Readable, Trace, Tracer};
use rquickjs::function::{Constructor, This};
use rquickjs::object::Accessor;
use rquickjs::{
    Array, Context, Ctx, FromJs, Function, IntoJs, JsLifetime, Persistent, Result as JsResult,
    Runtime, TypedArray, Value as JsValue,
};

use crate::interop::{BulkMode, BulkValue, Direction};

use crate::harness::{number_value, read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};

#[derive(Default, Clone)]
pub struct InteropData(Rc<Cell<i64>>);

impl<'js> Trace<'js> for InteropData {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

unsafe impl<'js> JsLifetime<'js> for InteropData {
    type Changed<'to> = InteropData;
}

impl<'js> JsClass<'js> for InteropData {
    const NAME: &'static str = "InteropData";
    type Mutable = Readable;

    fn prototype(ctx: &Ctx<'js>) -> JsResult<Option<rquickjs::Object<'js>>> {
        let proto = rquickjs::Object::new(ctx.clone())?;
        let get = Function::new(ctx.clone(), |this: This<Self>| this.0 .0.get() as f64)?
            .with_name("get")?;
        proto.prop("get", get)?;
        proto.prop(
            "value",
            Accessor::from(|this: This<Self>| this.0 .0.get() as f64)
                .set(|this: This<Self>, value: f64| this.0 .0.set(value as i64)),
        )?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> JsResult<Option<Constructor<'js>>> {
        Ok(Some(Constructor::new_class::<Self, _, _>(
            ctx.clone(),
            |value: f64| InteropData(Rc::new(Cell::new(value as i64))),
        )?))
    }
}

#[derive(Default, Clone, Copy)]
pub struct CopyData(i64);

impl<'js> Trace<'js> for CopyData {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

unsafe impl<'js> JsLifetime<'js> for CopyData {
    type Changed<'to> = CopyData;
}

impl<'js> JsClass<'js> for CopyData {
    const NAME: &'static str = "CopyData";
    type Mutable = Readable;

    fn prototype(_ctx: &Ctx<'js>) -> JsResult<Option<rquickjs::Object<'js>>> {
        Ok(None)
    }

    fn constructor(ctx: &Ctx<'js>) -> JsResult<Option<Constructor<'js>>> {
        Ok(Some(Constructor::new_class::<Self, _, _>(
            ctx.clone(),
            |value: f64| CopyData(value as i64),
        )?))
    }
}

#[derive(Default, Clone)]
pub struct RcData(Rc<str>);

impl<'js> Trace<'js> for RcData {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

unsafe impl<'js> JsLifetime<'js> for RcData {
    type Changed<'to> = RcData;
}

impl<'js> JsClass<'js> for RcData {
    const NAME: &'static str = "RcData";
    type Mutable = Readable;

    fn prototype(_ctx: &Ctx<'js>) -> JsResult<Option<rquickjs::Object<'js>>> {
        Ok(None)
    }

    fn constructor(ctx: &Ctx<'js>) -> JsResult<Option<Constructor<'js>>> {
        Ok(Some(Constructor::new_class::<Self, _, _>(
            ctx.clone(),
            |value: String| RcData(value.into()),
        )?))
    }
}

impl<'js> FromJs<'js> for InteropData {
    fn from_js(_: &Ctx<'js>, value: JsValue<'js>) -> JsResult<Self> {
        Ok(Class::<Self>::from_value(&value)?.try_borrow()?.clone())
    }
}

impl<'js> IntoJs<'js> for InteropData {
    fn into_js(self, ctx: &Ctx<'js>) -> JsResult<JsValue<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> FromJs<'js> for CopyData {
    fn from_js(_: &Ctx<'js>, value: JsValue<'js>) -> JsResult<Self> {
        Ok(Class::<Self>::from_value(&value)?.try_borrow()?.clone())
    }
}

impl<'js> IntoJs<'js> for CopyData {
    fn into_js(self, ctx: &Ctx<'js>) -> JsResult<JsValue<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> FromJs<'js> for RcData {
    fn from_js(_: &Ctx<'js>, value: JsValue<'js>) -> JsResult<Self> {
        Ok(Class::<Self>::from_value(&value)?.try_borrow()?.clone())
    }
}

impl<'js> IntoJs<'js> for RcData {
    fn into_js(self, ctx: &Ctx<'js>) -> JsResult<JsValue<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

enum QuickJsMode {
    Call,
    CallIn { iterations: usize },
    Callback { iterations: usize },
    Userdata,
    BulkListIn { value: BulkValue, size: usize },
    BulkListOut,
    BulkBufferIn { value: BulkValue, size: usize },
    BulkBufferOut,
    Error { propagates: bool },
}

pub struct QuickJsInterop {
    // The function handle must be dropped before the context and runtime.
    entry: Persistent<Function<'static>>,
    context: Context,
    _runtime: Runtime,
    mode: QuickJsMode,
    result: ResultKind,
}

impl InteropEngine for QuickJsInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        let mode = match case.op {
            InteropOp::Call => QuickJsMode::Call,
            InteropOp::CallIn { iterations } => QuickJsMode::CallIn { iterations },
            InteropOp::Callback { iterations } => QuickJsMode::Callback { iterations },
            InteropOp::Userdata(_) => QuickJsMode::Userdata,
            InteropOp::Bulk(op) if op.mode == BulkMode::List => match op.direction {
                Direction::In => QuickJsMode::BulkListIn {
                    value: op.value,
                    size: op.size,
                },
                Direction::Out => QuickJsMode::BulkListOut,
            },
            InteropOp::Bulk(op) if op.mode == BulkMode::Buffer => match op.direction {
                Direction::In => QuickJsMode::BulkBufferIn {
                    value: op.value,
                    size: op.size,
                },
                Direction::Out => QuickJsMode::BulkBufferOut,
            },
            InteropOp::Error(op) => QuickJsMode::Error {
                propagates: matches!(op, crate::interop::ErrorOp::PropagatesToHost),
            },
            _ => return Ok(None),
        };
        let is_callback = matches!(mode, QuickJsMode::Callback { .. });
        let is_userdata = matches!(mode, QuickJsMode::Userdata);
        let Some(script_path) = case.scripts.js else {
            return Ok(None);
        };
        let script = read_script(script_path)?;

        let runtime = Runtime::new().map_err(|error| anyhow!("{error}"))?;
        let context = Context::full(&runtime).map_err(|error| anyhow!("{error}"))?;

        let entry = context
            .with(|ctx| -> JsResult<Persistent<Function<'static>>> {
                register_host_functions(&ctx)?;
                if is_userdata {
                    let globals = ctx.globals();
                    Class::<InteropData>::define(&globals)?;
                    Class::<CopyData>::define(&globals)?;
                    Class::<RcData>::define(&globals)?;
                }
                ctx.eval::<(), _>(script.as_str())?;
                let func = ctx.globals().get::<_, Function>(case.entry)?;
                let func = if is_callback {
                    func.call::<_, Function>(())?
                } else {
                    func
                };
                Ok(Persistent::save(&ctx, func))
            })
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Some(Self {
            entry,
            context,
            _runtime: runtime,
            mode,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        let entry = self.entry.clone();
        match self.mode {
            QuickJsMode::Call => {
                self.context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        func.call::<_, ()>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
            }
            QuickJsMode::CallIn { iterations } => {
                self.context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        let mut sum = 0.0f64;
                        for _ in 0..iterations {
                            let value: f64 = func.call((host::UPDATE_DT,))?;
                            sum += value;
                        }
                        std::hint::black_box(sum);
                        Ok::<(), rquickjs::Error>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
            }
            QuickJsMode::Callback { iterations } => {
                self.context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        let mut sum = 0i64;
                        for index in 0..iterations {
                            let value: f64 = func.call((index as f64,))?;
                            sum = sum.wrapping_add(value as i64);
                        }
                        std::hint::black_box(sum);
                        Ok::<(), rquickjs::Error>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
            }
            QuickJsMode::Userdata => {
                self.context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        func.call::<_, f64>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
            }
            QuickJsMode::BulkListIn { value, size } => {
                self.context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        let array = Array::new(ctx.clone())?;
                        match value {
                            BulkValue::F64 => {
                                for (index, value) in
                                    host::bulk_f64_values(size).into_iter().enumerate()
                                {
                                    array.set(index, value)?;
                                }
                            }
                            BulkValue::U8 => {
                                for (index, value) in
                                    host::bulk_u8_values(size).into_iter().enumerate()
                                {
                                    array.set(index, value)?;
                                }
                            }
                        }
                        func.call::<_, f64>((array,))?;
                        Ok::<(), rquickjs::Error>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
            }
            QuickJsMode::BulkListOut => {
                self.context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        func.call::<_, f64>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
            }
            QuickJsMode::BulkBufferIn { value, size } => {
                self.context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        match value {
                            BulkValue::F64 => {
                                let array = TypedArray::<f64>::new(
                                    ctx.clone(),
                                    host::bulk_f64_values(size),
                                )?;
                                func.call::<_, f64>((array,))?;
                            }
                            BulkValue::U8 => {
                                let array =
                                    TypedArray::<u8>::new(ctx.clone(), host::bulk_u8_values(size))?;
                                func.call::<_, f64>((array,))?;
                            }
                        }
                        Ok::<(), rquickjs::Error>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
            }
            QuickJsMode::BulkBufferOut => {
                self.context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        func.call::<_, f64>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
            }
            QuickJsMode::Error { propagates } => {
                let result = self.context.with(|ctx| {
                    let func = entry.restore(&ctx)?;
                    func.call::<_, f64>(())
                });
                match result {
                    Ok(value) => {
                        if propagates {
                            bail!("expected the script to throw");
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
        let entry = self.entry.clone();
        match self.mode {
            QuickJsMode::Call => {
                let number = self
                    .context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        func.call::<_, f64>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, number)
            }
            QuickJsMode::CallIn { iterations } => {
                let sum = self
                    .context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        let mut sum = 0.0f64;
                        for _ in 0..iterations {
                            let value: f64 = func.call((host::UPDATE_DT,))?;
                            sum += value;
                        }
                        Ok::<f64, rquickjs::Error>(sum)
                    })
                    .map_err(|error| anyhow!("{error}"))?;
                Ok(Value::Float(sum))
            }
            QuickJsMode::Callback { iterations } => {
                let sum = self
                    .context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        let mut sum = 0i64;
                        for index in 0..iterations {
                            let value: f64 = func.call((index as f64,))?;
                            sum = sum.wrapping_add(value as i64);
                        }
                        Ok::<i64, rquickjs::Error>(sum)
                    })
                    .map_err(|error| anyhow!("{error}"))?;
                Ok(Value::Int(sum))
            }
            QuickJsMode::Userdata => {
                let value = self
                    .context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        func.call::<_, f64>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, value)
            }
            QuickJsMode::BulkListIn { value, size } => {
                let sum = self
                    .context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        let array = Array::new(ctx.clone())?;
                        match value {
                            BulkValue::F64 => {
                                for (index, value) in
                                    host::bulk_f64_values(size).into_iter().enumerate()
                                {
                                    array.set(index, value)?;
                                }
                            }
                            BulkValue::U8 => {
                                for (index, value) in
                                    host::bulk_u8_values(size).into_iter().enumerate()
                                {
                                    array.set(index, value)?;
                                }
                            }
                        }
                        func.call::<_, f64>((array,))
                    })
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, sum)
            }
            QuickJsMode::BulkListOut => {
                let sum = self
                    .context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        func.call::<_, f64>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, sum)
            }
            QuickJsMode::BulkBufferIn { value, size } => {
                let sum = self
                    .context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        match value {
                            BulkValue::F64 => {
                                let array = TypedArray::<f64>::new(
                                    ctx.clone(),
                                    host::bulk_f64_values(size),
                                )?;
                                func.call::<_, f64>((array,))
                            }
                            BulkValue::U8 => {
                                let array =
                                    TypedArray::<u8>::new(ctx.clone(), host::bulk_u8_values(size))?;
                                func.call::<_, f64>((array,))
                            }
                        }
                    })
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, sum)
            }
            QuickJsMode::BulkBufferOut => {
                let sum = self
                    .context
                    .with(|ctx| {
                        let func = entry.restore(&ctx)?;
                        func.call::<_, f64>(())
                    })
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, sum)
            }
            QuickJsMode::Error { propagates } => {
                let result = self.context.with(|ctx| {
                    let func = entry.restore(&ctx)?;
                    func.call::<_, f64>(())
                });
                match result {
                    Ok(value) => {
                        if propagates {
                            bail!("expected the script to throw");
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

    let callback = Function::new(ctx.clone(), |value: f64| {
        host::callback_value(value as i64) as f64
    })?
    .with_name("interop_callback")?;
    globals.set("interop_callback", callback)?;

    let fail = Function::new(ctx.clone(), |ctx: Ctx<'_>| -> JsResult<f64> {
        Err(rquickjs::Exception::throw_message(&ctx, host::HOST_ERROR))
    })?
    .with_name("interop_fail")?;
    globals.set("interop_fail", fail)?;

    let sum_f64 = Function::new(ctx.clone(), |array: Array| -> f64 {
        array
            .iter::<f64>()
            .map(|value| value.unwrap_or(0.0))
            .sum()
    })?
    .with_name("interop_sum_f64_list")?;
    globals.set("interop_sum_f64_list", sum_f64)?;

    let sum_u8 = Function::new(ctx.clone(), |array: Array| -> f64 {
        array
            .iter::<f64>()
            .map(|value| value.unwrap_or(0.0))
            .sum()
    })?
    .with_name("interop_sum_u8_list")?;
    globals.set("interop_sum_u8_list", sum_u8)?;

    let sum_f64_buffer = Function::new(ctx.clone(), |values: TypedArray<f64>| -> f64 {
        unsafe { values.as_slice() }.iter().sum()
    })?
    .with_name("interop_sum_f64_buffer")?;
    globals.set("interop_sum_f64_buffer", sum_f64_buffer)?;

    let sum_u8_buffer = Function::new(ctx.clone(), |values: TypedArray<u8>| -> f64 {
        unsafe { values.as_slice() }
            .iter()
            .map(|value| f64::from(*value))
            .sum()
    })?
    .with_name("interop_sum_u8_buffer")?;
    globals.set("interop_sum_u8_buffer", sum_u8_buffer)?;

    Ok(())
}
