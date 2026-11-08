use std::rc::Rc;

use anyhow::{anyhow, bail, Result};
use boa_engine::class::{Class, ClassBuilder};
use boa_engine::object::builtins::JsArray;
use boa_engine::object::FunctionObjectBuilder;
use boa_engine::property::Attribute;
use boa_engine::{
    js_string, Finalize, JsData, JsNativeError, JsString, JsValue, NativeFunction, Source, Trace,
};

use crate::harness::{number_value, read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{
    BulkMode, BulkValue, Direction, InteropCase, InteropEngine, InteropOp,
};

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize, JsData)]
pub struct InteropData(i64);

impl Class for InteropData {
    const NAME: &'static str = "InteropData";
    const LENGTH: usize = 1;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut boa_engine::Context,
    ) -> boa_engine::JsResult<Self> {
        Ok(Self(args[0].to_number(context)? as i64))
    }

    fn init(class: &mut ClassBuilder) -> boa_engine::JsResult<()> {
        class.method(
            js_string!("get"),
            0,
            NativeFunction::from_fn_ptr(|this, _args, _ctx| {
                let this = this.as_object().unwrap();
                let this = this.downcast_ref::<Self>().unwrap();
                Ok(JsValue::from(this.0))
            }),
        );

        let getter = FunctionObjectBuilder::new(
            class.context().realm(),
            NativeFunction::from_fn_ptr(|this, _args, _ctx| {
                let this = this.as_object().unwrap();
                let this = this.downcast_ref::<Self>().unwrap();
                Ok(JsValue::from(this.0))
            }),
        )
        .build();
        let setter = FunctionObjectBuilder::new(
            class.context().realm(),
            NativeFunction::from_fn_ptr(|this, args, ctx| {
                let this = this.as_object().unwrap();
                let mut this = this.downcast_mut::<Self>().unwrap();
                this.0 = args[0].to_number(ctx)? as i64;
                Ok(JsValue::undefined())
            }),
        )
        .build();
        class.accessor(
            js_string!("value"),
            Some(getter),
            Some(setter),
            Attribute::all(),
        );

        Ok(())
    }
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize, JsData)]
pub struct CopyData(i64);

impl Class for CopyData {
    const NAME: &'static str = "CopyData";
    const LENGTH: usize = 1;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut boa_engine::Context,
    ) -> boa_engine::JsResult<Self> {
        Ok(Self(args[0].to_number(context)? as i64))
    }

    fn init(_class: &mut ClassBuilder) -> boa_engine::JsResult<()> {
        Ok(())
    }
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize, JsData)]
pub struct RcData(Rc<str>);

impl Class for RcData {
    const NAME: &'static str = "RcData";
    const LENGTH: usize = 1;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut boa_engine::Context,
    ) -> boa_engine::JsResult<Self> {
        let value = args[0].to_string(context)?;
        Ok(Self(value.to_std_string().unwrap().into()))
    }

    fn init(_class: &mut ClassBuilder) -> boa_engine::JsResult<()> {
        Ok(())
    }
}

enum BoaMode {
    Call,
    CallIn { iterations: usize },
    Callback { iterations: usize },
    Userdata,
    BulkListIn { value: BulkValue, size: usize },
    BulkListOut,
    Error { propagates: bool },
}

pub struct BoaInterop {
    context: boa_engine::Context,
    entry: JsValue,
    mode: BoaMode,
    result: ResultKind,
}

impl BoaInterop {
    fn call(&mut self, args: &[JsValue]) -> Result<JsValue> {
        let Some(func) = self.entry.as_callable() else {
            bail!("interop entry is not callable");
        };
        func.call(&self.entry, args, &mut self.context)
            .map_err(|error| anyhow!("{error}"))
    }
}

impl InteropEngine for BoaInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        let mode = match case.op {
            InteropOp::Call => BoaMode::Call,
            InteropOp::CallIn { iterations } => BoaMode::CallIn { iterations },
            InteropOp::Callback { iterations } => BoaMode::Callback { iterations },
            InteropOp::Userdata(_) => BoaMode::Userdata,
            InteropOp::Bulk(op) if op.mode == BulkMode::List => match op.direction {
                Direction::In => BoaMode::BulkListIn {
                    value: op.value,
                    size: op.size,
                },
                Direction::Out => BoaMode::BulkListOut,
            },
            InteropOp::Error(op) => BoaMode::Error {
                propagates: matches!(op, crate::interop::ErrorOp::PropagatesToHost),
            },
            _ => return Ok(None),
        };
        let Some(script_path) = case.scripts.js else {
            return Ok(None);
        };

        let mut context = boa_engine::Context::default();
        register_host_functions(&mut context)?;
        if matches!(mode, BoaMode::Userdata) {
            register_userdata(&mut context)?;
        }

        let script = read_script(script_path)?;
        context
            .eval(Source::from_bytes(&script))
            .map_err(|error| anyhow!("{error}"))?;

        let entry = context
            .global_object()
            .get(JsString::from(case.entry), &mut context)
            .map_err(|error| anyhow!("{error}"))?;

        let entry = if matches!(mode, BoaMode::Callback { .. }) {
            let Some(factory) = entry.as_callable() else {
                bail!("callback factory is not callable");
            };
            factory
                .call(&entry, &[], &mut context)
                .map_err(|error| anyhow!("{error}"))?
        } else {
            entry
        };

        Ok(Some(Self {
            context,
            entry,
            mode,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        match self.mode {
            BoaMode::Call => {
                self.call(&[])?;
            }
            BoaMode::CallIn { iterations } => {
                let mut sum = 0.0f64;
                for _ in 0..iterations {
                    let value = self.call(&[JsValue::from(host::UPDATE_DT)])?;
                    sum += value
                        .to_number(&mut self.context)
                        .map_err(|error| anyhow!("{error}"))?;
                }
                std::hint::black_box(sum);
            }
            BoaMode::Callback { iterations } => {
                let mut sum = 0i64;
                for index in 0..iterations {
                    let value = self.call(&[JsValue::from(index as f64)])?;
                    let number = value
                        .to_number(&mut self.context)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(number as i64);
                }
                std::hint::black_box(sum);
            }
            BoaMode::Userdata => {
                let value = self.call(&[])?;
                std::hint::black_box(value);
            }
            BoaMode::BulkListIn { value, size } => {
                let array = match value {
                    BulkValue::F64 => JsArray::from_iter(
                        host::bulk_f64_values(size)
                            .into_iter()
                            .map(JsValue::from),
                        &mut self.context,
                    ),
                    BulkValue::U8 => JsArray::from_iter(
                        host::bulk_u8_values(size)
                            .into_iter()
                            .map(|value| JsValue::from(f64::from(value))),
                        &mut self.context,
                    ),
                };
                let value = self.call(&[array.into()])?;
                std::hint::black_box(value);
            }
            BoaMode::BulkListOut => {
                let value = self.call(&[])?;
                std::hint::black_box(value);
            }
            BoaMode::Error { propagates } => match self.call(&[]) {
                Ok(value) => {
                    if propagates {
                        bail!("expected the script to raise an error");
                    }
                    std::hint::black_box(value);
                }
                Err(error) => {
                    if !propagates {
                        return Err(error);
                    }
                }
            },
        }
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        match self.mode {
            BoaMode::Call => {
                let value = self.call(&[])?;
                let number = value
                    .to_number(&mut self.context)
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, number)
            }
            BoaMode::CallIn { iterations } => {
                let mut sum = 0.0f64;
                for _ in 0..iterations {
                    let value = self.call(&[JsValue::from(host::UPDATE_DT)])?;
                    sum += value
                        .to_number(&mut self.context)
                        .map_err(|error| anyhow!("{error}"))?;
                }
                Ok(Value::Float(sum))
            }
            BoaMode::Callback { iterations } => {
                let mut sum = 0i64;
                for index in 0..iterations {
                    let value = self.call(&[JsValue::from(index as f64)])?;
                    let number = value
                        .to_number(&mut self.context)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(number as i64);
                }
                Ok(Value::Int(sum))
            }
            BoaMode::Userdata => {
                let value = self.call(&[])?;
                let number = value
                    .to_number(&mut self.context)
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, number)
            }
            BoaMode::BulkListIn { value, size } => {
                let array = match value {
                    BulkValue::F64 => JsArray::from_iter(
                        host::bulk_f64_values(size)
                            .into_iter()
                            .map(JsValue::from),
                        &mut self.context,
                    ),
                    BulkValue::U8 => JsArray::from_iter(
                        host::bulk_u8_values(size)
                            .into_iter()
                            .map(|value| JsValue::from(f64::from(value))),
                        &mut self.context,
                    ),
                };
                let value = self.call(&[array.into()])?;
                let number = value
                    .to_number(&mut self.context)
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, number)
            }
            BoaMode::BulkListOut => {
                let value = self.call(&[])?;
                let number = value
                    .to_number(&mut self.context)
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, number)
            }
            BoaMode::Error { propagates } => match self.call(&[]) {
                Ok(value) => {
                    if propagates {
                        bail!("expected the script to raise an error");
                    }
                    let number = value
                        .to_number(&mut self.context)
                        .map_err(|error| anyhow!("{error}"))?;
                    number_value(self.result, number)
                }
                Err(error) => {
                    if propagates {
                        Ok(Value::Int(1))
                    } else {
                        Err(error)
                    }
                }
            },
        }
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
    register(
        "interop_callback",
        1,
        NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let value = args[0].to_number(ctx)? as i64;
            Ok(JsValue::from(host::callback_value(value)))
        }),
    )?;
    register(
        "interop_fail",
        0,
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Err(JsNativeError::typ()
                .with_message(host::HOST_ERROR)
                .into())
        }),
    )?;
    register(
        "interop_sum_f64_list",
        1,
        NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let array = JsArray::from_object(args[0].as_object().unwrap().clone())?;
            let length = array.length(ctx)?;
            let mut sum = 0.0f64;
            for index in 0..length {
                sum += array.get(index, ctx)?.to_number(ctx)?;
            }
            Ok(JsValue::from(sum))
        }),
    )?;
    register(
        "interop_sum_u8_list",
        1,
        NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let array = JsArray::from_object(args[0].as_object().unwrap().clone())?;
            let length = array.length(ctx)?;
            let mut sum = 0.0f64;
            for index in 0..length {
                sum += array.get(index, ctx)?.to_number(ctx)?;
            }
            Ok(JsValue::from(sum))
        }),
    )?;

    Ok(())
}

fn register_userdata(context: &mut boa_engine::Context) -> Result<()> {
    context
        .register_global_class::<InteropData>()
        .map_err(|error| anyhow!("{error}"))?;
    context
        .register_global_class::<CopyData>()
        .map_err(|error| anyhow!("{error}"))?;
    context
        .register_global_class::<RcData>()
        .map_err(|error| anyhow!("{error}"))?;
    Ok(())
}
