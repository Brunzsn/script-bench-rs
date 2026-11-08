use std::rc::Rc;

use anyhow::{anyhow, bail, Result};
use koto::{derive::*, prelude::*, runtime};

use crate::harness::{number_value, read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{
    BulkMode, BulkValue, Direction, InteropCase, InteropEngine, InteropOp,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, KotoCopy, KotoType)]
pub struct InteropData(i64);

#[koto_impl]
impl InteropData {
    fn new_koto_object(value: i64) -> KObject {
        KObject::from(Self(value))
    }

    #[koto_method]
    fn get(&self) -> KValue {
        self.0.into()
    }
}

impl KotoObject for InteropData {
    fn display(&self, ctx: &mut DisplayContext) -> runtime::Result<()> {
        ctx.append(self.0.to_string());
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, KotoCopy, KotoType)]
pub struct CopyData(i64);

#[koto_impl]
impl CopyData {}

impl KotoObject for CopyData {
    fn display(&self, ctx: &mut DisplayContext) -> runtime::Result<()> {
        ctx.append(self.0.to_string());
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, KotoCopy, KotoType)]
pub struct RcData(Rc<str>);

#[koto_impl]
impl RcData {}

impl KotoObject for RcData {
    fn display(&self, ctx: &mut DisplayContext) -> runtime::Result<()> {
        ctx.append(self.0.to_string());
        Ok(())
    }
}

enum KotoMode {
    Call,
    CallIn { iterations: usize },
    Callback { iterations: usize },
    Userdata,
    BulkListIn { value: BulkValue, size: usize },
    BulkListOut,
    SerdeToScript,
    SerdeFromScript,
    Error { propagates: bool },
}

pub struct KotoInterop {
    engine: Koto,
    entry: KValue,
    mode: KotoMode,
    result: ResultKind,
}

fn serde_value() -> KValue {
    let mut child = KMap::new();
    child.insert("id", host::SERDE_CHILD_ID);
    let mut values = Vec::new();
    for value in host::SERDE_VALUES {
        values.push(KValue::from(value));
    }
    let mut map = KMap::new();
    map.insert("id", host::SERDE_ID);
    map.insert("name", host::SERDE_NAME);
    map.insert("values", KList::from_slice(&values));
    map.insert("child", child);
    KValue::from(map)
}

fn serde_checksum(value: &KValue) -> i64 {
    let KValue::Map(map) = value else {
        return 0;
    };
    let id = match map.get("id") {
        Some(KValue::Number(number)) => f64::from(number) as i64,
        _ => 0,
    };
    let name_len = match map.get("name") {
        Some(KValue::Str(name)) => name.as_str().chars().count() as i64,
        _ => 0,
    };
    let mut value_sum = 0.0f64;
    if let Some(KValue::List(values)) = map.get("values") {
        for value in values.data().iter() {
            if let KValue::Number(number) = value {
                value_sum += f64::from(number);
            }
        }
    }
    let child_id = match map.get("child") {
        Some(KValue::Map(child)) => match child.get("id") {
            Some(KValue::Number(number)) => f64::from(number) as i64,
            _ => 0,
        },
        _ => 0,
    };
    id + name_len + value_sum as i64 + child_id
}

impl InteropEngine for KotoInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        let mode = match case.op {
            InteropOp::Call => KotoMode::Call,
            InteropOp::CallIn { iterations } => KotoMode::CallIn { iterations },
            InteropOp::Callback { iterations } => KotoMode::Callback { iterations },
            InteropOp::Userdata(_) => KotoMode::Userdata,
            InteropOp::Bulk(op) if op.mode == BulkMode::List => match op.direction {
                Direction::In => KotoMode::BulkListIn {
                    value: op.value,
                    size: op.size,
                },
                Direction::Out => KotoMode::BulkListOut,
            },
            InteropOp::Serde(op) => match op {
                crate::interop::SerdeOp::ToScript => KotoMode::SerdeToScript,
                crate::interop::SerdeOp::FromScript => KotoMode::SerdeFromScript,
            },
            InteropOp::Error(op) => KotoMode::Error {
                propagates: matches!(op, crate::interop::ErrorOp::PropagatesToHost),
            },
            _ => return Ok(None),
        };
        let Some(script_path) = case.scripts.koto else {
            return Ok(None);
        };

        let mut engine = Koto::default();
        let prelude = engine.prelude();
        register_host_functions(prelude);

        let script = read_script(script_path)?;
        engine
            .compile_and_run(script.as_str())
            .map_err(|error| anyhow!("{error}"))?;

        let entry = engine
            .exports()
            .get(case.entry)
            .ok_or_else(|| anyhow!("missing export `{}`", case.entry))?;

        let entry = if matches!(mode, KotoMode::Callback { .. }) {
            engine
                .call_function(entry, &[])
                .map_err(|error| anyhow!("{error}"))?
        } else {
            entry
        };

        Ok(Some(Self {
            engine,
            entry,
            mode,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        match self.mode {
            KotoMode::Call => {
                self.engine
                    .call_function(self.entry.clone(), &[])
                    .map_err(|error| anyhow!("{error}"))?;
            }
            KotoMode::CallIn { iterations } => {
                let arg = KValue::from(host::UPDATE_DT);
                let mut sum = 0.0f64;
                for _ in 0..iterations {
                    let value = self
                        .engine
                        .call_function(self.entry.clone(), &[arg.clone()])
                        .map_err(|error| anyhow!("{error}"))?;
                    let KValue::Number(number) = value else {
                        bail!("interop call-in did not return a number");
                    };
                    sum += f64::from(number);
                }
                std::hint::black_box(sum);
            }
            KotoMode::Callback { iterations } => {
                let mut sum = 0i64;
                for index in 0..iterations {
                    let arg = KValue::from(index as f64);
                    let value = self
                        .engine
                        .call_function(self.entry.clone(), &[arg])
                        .map_err(|error| anyhow!("{error}"))?;
                    let KValue::Number(number) = value else {
                        bail!("interop callback did not return a number");
                    };
                    sum = sum.wrapping_add(f64::from(number) as i64);
                }
                std::hint::black_box(sum);
            }
            KotoMode::Userdata => {
                let value = self
                    .engine
                    .call_function(self.entry.clone(), &[])
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(value);
            }
            KotoMode::BulkListIn { value, size } => {
                let data = match value {
                    BulkValue::F64 => host::bulk_f64_values(size)
                        .into_iter()
                        .map(KValue::from)
                        .collect(),
                    BulkValue::U8 => host::bulk_u8_values(size)
                        .into_iter()
                        .map(|value| KValue::from(i64::from(value)))
                        .collect(),
                };
                let list = KValue::from(KList::from_slice(&data));
                let result = self
                    .engine
                    .call_function(self.entry.clone(), &[list])
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(result);
            }
            KotoMode::BulkListOut => {
                let result = self
                    .engine
                    .call_function(self.entry.clone(), &[])
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(result);
            }
            KotoMode::SerdeToScript => {
                let value = self
                    .engine
                    .call_function(self.entry.clone(), &[serde_value()])
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(value);
            }
            KotoMode::SerdeFromScript => {
                let value = self
                    .engine
                    .call_function(self.entry.clone(), &[])
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(serde_checksum(&value));
            }
            KotoMode::Error { propagates } => {
                let result = self.engine.call_function(self.entry.clone(), &[]);
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
            KotoMode::Call => {
                let value = self
                    .engine
                    .call_function(self.entry.clone(), &[])
                    .map_err(|error| anyhow!("{error}"))?;
                let KValue::Number(number) = value else {
                    bail!("interop entry did not return a number");
                };
                number_value(self.result, f64::from(number))
            }
            KotoMode::CallIn { iterations } => {
                let arg = KValue::from(host::UPDATE_DT);
                let mut sum = 0.0f64;
                for _ in 0..iterations {
                    let value = self
                        .engine
                        .call_function(self.entry.clone(), &[arg.clone()])
                        .map_err(|error| anyhow!("{error}"))?;
                    let KValue::Number(number) = value else {
                        bail!("interop call-in did not return a number");
                    };
                    sum += f64::from(number);
                }
                Ok(Value::Float(sum))
            }
            KotoMode::Callback { iterations } => {
                let mut sum = 0i64;
                for index in 0..iterations {
                    let arg = KValue::from(index as f64);
                    let value = self
                        .engine
                        .call_function(self.entry.clone(), &[arg])
                        .map_err(|error| anyhow!("{error}"))?;
                    let KValue::Number(number) = value else {
                        bail!("interop callback did not return a number");
                    };
                    sum = sum.wrapping_add(f64::from(number) as i64);
                }
                Ok(Value::Int(sum))
            }
            KotoMode::Userdata => {
                let value = self
                    .engine
                    .call_function(self.entry.clone(), &[])
                    .map_err(|error| anyhow!("{error}"))?;
                let KValue::Number(number) = value else {
                    bail!("interop userdata case did not return a number");
                };
                number_value(self.result, f64::from(number))
            }
            KotoMode::BulkListIn { value, size } => {
                let data = match value {
                    BulkValue::F64 => host::bulk_f64_values(size)
                        .into_iter()
                        .map(KValue::from)
                        .collect(),
                    BulkValue::U8 => host::bulk_u8_values(size)
                        .into_iter()
                        .map(|value| KValue::from(i64::from(value)))
                        .collect(),
                };
                let list = KValue::from(KList::from_slice(&data));
                let result = self
                    .engine
                    .call_function(self.entry.clone(), &[list])
                    .map_err(|error| anyhow!("{error}"))?;
                let KValue::Number(number) = result else {
                    bail!("interop bulk case did not return a number");
                };
                number_value(self.result, f64::from(number))
            }
            KotoMode::BulkListOut => {
                let result = self
                    .engine
                    .call_function(self.entry.clone(), &[])
                    .map_err(|error| anyhow!("{error}"))?;
                let KValue::Number(number) = result else {
                    bail!("interop bulk case did not return a number");
                };
                number_value(self.result, f64::from(number))
            }
            KotoMode::SerdeToScript => {
                let value = self
                    .engine
                    .call_function(self.entry.clone(), &[serde_value()])
                    .map_err(|error| anyhow!("{error}"))?;
                let KValue::Number(number) = value else {
                    bail!("interop serde case did not return a number");
                };
                number_value(self.result, f64::from(number))
            }
            KotoMode::SerdeFromScript => {
                let value = self
                    .engine
                    .call_function(self.entry.clone(), &[])
                    .map_err(|error| anyhow!("{error}"))?;
                Ok(Value::Int(serde_checksum(&value)))
            }
            KotoMode::Error { propagates } => {
                let result = self.engine.call_function(self.entry.clone(), &[]);
                match result {
                    Ok(value) => {
                        if propagates {
                            bail!("expected the script to raise an error");
                        }
                        let KValue::Number(number) = value else {
                            bail!("interop error case did not return a number");
                        };
                        number_value(self.result, f64::from(number))
                    }
                    Err(_) if propagates => Ok(Value::Int(1)),
                    Err(error) => Err(anyhow!("{error}")),
                }
            }
        }
    }
}

fn register_host_functions(prelude: &koto::runtime::KMap) {
    prelude.add_fn("interop_ping", |ctx| match ctx.args() {
        [] => Ok(host::ping().into()),
        unexpected => unexpected_args("no arguments", unexpected),
    });

    prelude.add_fn("interop_id_i64", |ctx| match ctx.args() {
        [KValue::Number(value)] => Ok(host::id_i64(i64::from(value)).into()),
        unexpected => unexpected_args("one number", unexpected),
    });

    prelude.add_fn("interop_sum_i64_3", |ctx| match ctx.args() {
        [KValue::Number(a), KValue::Number(b), KValue::Number(c)] => Ok(host::sum_i64_3(
            i64::from(a),
            i64::from(b),
            i64::from(c),
        )
        .into()),
        unexpected => unexpected_args("three numbers", unexpected),
    });

    prelude.add_fn("interop_sum_i64_5", |ctx| match ctx.args() {
        [
            KValue::Number(a),
            KValue::Number(b),
            KValue::Number(c),
            KValue::Number(d),
            KValue::Number(e),
        ] => Ok(host::sum_i64_5(
            i64::from(a),
            i64::from(b),
            i64::from(c),
            i64::from(d),
            i64::from(e),
        )
        .into()),
        unexpected => unexpected_args("five numbers", unexpected),
    });

    prelude.add_fn("interop_id_f64", |ctx| match ctx.args() {
        [KValue::Number(value)] => Ok(host::id_f64(f64::from(value)).into()),
        unexpected => unexpected_args("one number", unexpected),
    });

    prelude.add_fn("interop_strlen", |ctx| match ctx.args() {
        [KValue::Str(value)] => Ok(host::strlen(value.as_str()).into()),
        unexpected => unexpected_args("one string", unexpected),
    });

    prelude.add_fn("interop_string_out", |ctx| match ctx.args() {
        [] => Ok(KValue::Str(host::string_out().into())),
        unexpected => unexpected_args("no arguments", unexpected),
    });

    prelude.add_fn("interop_callback", |ctx| match ctx.args() {
        [KValue::Number(value)] => Ok(host::callback_value(i64::from(value)).into()),
        unexpected => unexpected_args("one number", unexpected),
    });

    prelude.add_fn("interop_fail", |ctx| match ctx.args() {
        [] => runtime::runtime_error!("{}", host::HOST_ERROR),
        unexpected => unexpected_args("no arguments", unexpected),
    });

    prelude.add_fn("InteropData_new", |ctx| match ctx.args() {
        [KValue::Number(value)] => Ok(InteropData::new_koto_object(i64::from(value)).into()),
        unexpected => unexpected_args("one number", unexpected),
    });

    prelude.add_fn("CopyData_new", |ctx| match ctx.args() {
        [KValue::Number(value)] => Ok(KObject::from(CopyData(i64::from(value))).into()),
        unexpected => unexpected_args("one number", unexpected),
    });

    prelude.add_fn("RcData_new", |ctx| match ctx.args() {
        [KValue::Str(value)] => Ok(KObject::from(RcData(value.as_str().into())).into()),
        unexpected => unexpected_args("one string", unexpected),
    });

    prelude.add_fn("interop_sum_f64_list", |ctx| match ctx.args() {
        [KValue::List(list)] => {
            let mut sum = 0.0f64;
            for value in list.data().iter() {
                if let KValue::Number(number) = value {
                    sum += f64::from(number);
                }
            }
            Ok(sum.into())
        }
        unexpected => unexpected_args("one list", unexpected),
    });

    prelude.add_fn("interop_sum_u8_list", |ctx| match ctx.args() {
        [KValue::List(list)] => {
            let mut sum = 0.0f64;
            for value in list.data().iter() {
                if let KValue::Number(number) = value {
                    sum += f64::from(number);
                }
            }
            Ok(sum.into())
        }
        unexpected => unexpected_args("one list", unexpected),
    });
}
