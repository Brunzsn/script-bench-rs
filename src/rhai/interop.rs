use std::cell::Cell;
use std::rc::Rc;

use anyhow::{anyhow, bail, Result};

use crate::harness::{read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{
    BulkMode, BulkValue, Direction, InteropCase, InteropEngine, InteropOp,
};

#[derive(Clone)]
pub struct InteropData(Cell<i64>);

impl InteropData {
    fn new(value: i64) -> Self {
        Self(Cell::new(value))
    }

    fn get(&mut self) -> i64 {
        self.0.get()
    }

    fn value(&mut self) -> i64 {
        self.0.get()
    }

    fn set_value(&mut self, value: i64) {
        self.0.set(value);
    }
}

#[derive(Clone, Copy)]
pub struct CopyData(i64);

impl CopyData {
    fn new(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct RcData(Rc<str>);

impl RcData {
    fn new(value: String) -> Self {
        Self(value.into())
    }
}

enum RhaiMode {
    Call,
    CallIn { iterations: usize },
    Callback {
        callback: rhai::FnPtr,
        iterations: usize,
    },
    Userdata,
    BulkListIn { value: BulkValue, size: usize },
    BulkListOut,
    SerdeToScript,
    SerdeFromScript,
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

fn serde_map() -> rhai::Map {
    let mut child = rhai::Map::new();
    child.insert("id".into(), rhai::Dynamic::from(host::SERDE_CHILD_ID));
    let mut values = rhai::Array::new();
    for value in host::SERDE_VALUES {
        values.push(rhai::Dynamic::from(value));
    }
    let mut map = rhai::Map::new();
    map.insert("id".into(), rhai::Dynamic::from(host::SERDE_ID));
    map.insert(
        "name".into(),
        rhai::Dynamic::from(host::SERDE_NAME.to_string()),
    );
    map.insert("values".into(), rhai::Dynamic::from(values));
    map.insert("child".into(), rhai::Dynamic::from(child));
    map
}

fn serde_checksum(value: rhai::Map) -> i64 {
    let id = value
        .get("id")
        .expect("id")
        .clone()
        .cast::<i64>();
    let name = value
        .get("name")
        .expect("name")
        .clone()
        .into_immutable_string()
        .expect("name string");
    let values = value
        .get("values")
        .expect("values")
        .clone()
        .cast::<rhai::Array>();
    let value_sum: f64 = values
        .iter()
        .map(|value| value.as_float().unwrap_or(0.0))
        .sum();
    let child = value
        .get("child")
        .expect("child")
        .clone()
        .cast::<rhai::Map>();
    let child_id = child
        .get("id")
        .expect("child id")
        .clone()
        .cast::<i64>();
    id + name.len() as i64 + value_sum as i64 + child_id
}

impl InteropEngine for RhaiInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        if !matches!(
            case.op,
            InteropOp::Call
                | InteropOp::CallIn { .. }
                | InteropOp::Callback { .. }
                | InteropOp::Userdata(_)
                | InteropOp::Bulk(_)
                | InteropOp::Serde(_)
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
        if matches!(case.op, InteropOp::Userdata(_)) {
            register_userdata(&mut engine);
        }

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
            InteropOp::Userdata(_) => RhaiMode::Userdata,
            InteropOp::Bulk(op) if op.mode == BulkMode::List => match op.direction {
                Direction::In => RhaiMode::BulkListIn {
                    value: op.value,
                    size: op.size,
                },
                Direction::Out => RhaiMode::BulkListOut,
            },
            InteropOp::Serde(op) => match op {
                crate::interop::SerdeOp::ToScript => RhaiMode::SerdeToScript,
                crate::interop::SerdeOp::FromScript => RhaiMode::SerdeFromScript,
            },
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
            RhaiMode::Userdata => {
                let value: i64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(value);
            }
            RhaiMode::BulkListIn { value, size } => {
                let mut values = rhai::Array::new();
                match value {
                    BulkValue::F64 => {
                        for value in host::bulk_f64_values(*size) {
                            values.push(rhai::Dynamic::from(value));
                        }
                    }
                    BulkValue::U8 => {
                        for value in host::bulk_u8_values(*size) {
                            values.push(rhai::Dynamic::from(i64::from(value)));
                        }
                    }
                }
                let result: f64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, (values,))
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(result);
            }
            RhaiMode::BulkListOut => {
                let result: f64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(result);
            }
            RhaiMode::SerdeToScript => {
                let map = serde_map();
                let result: i64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, (map,))
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(result);
            }
            RhaiMode::SerdeFromScript => {
                let map: rhai::Map = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(serde_checksum(map));
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
            RhaiMode::Userdata => {
                let value: i64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
                Ok(Value::Int(value))
            }
            RhaiMode::BulkListIn { value, size } => {
                let mut values = rhai::Array::new();
                match value {
                    BulkValue::F64 => {
                        for value in host::bulk_f64_values(*size) {
                            values.push(rhai::Dynamic::from(value));
                        }
                    }
                    BulkValue::U8 => {
                        for value in host::bulk_u8_values(*size) {
                            values.push(rhai::Dynamic::from(i64::from(value)));
                        }
                    }
                }
                let result: f64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, (values,))
                    .map_err(|error| anyhow!("{error}"))?;
                Ok(Value::Float(result))
            }
            RhaiMode::BulkListOut => {
                let result: f64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
                Ok(Value::Float(result))
            }
            RhaiMode::SerdeToScript => {
                let map = serde_map();
                let result: i64 = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, (map,))
                    .map_err(|error| anyhow!("{error}"))?;
                Ok(Value::Int(result))
            }
            RhaiMode::SerdeFromScript => {
                let map: rhai::Map = self
                    .engine
                    .call_fn(&mut self.scope, &self.ast, self.entry, ())
                    .map_err(|error| anyhow!("{error}"))?;
                Ok(Value::Int(serde_checksum(map)))
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

fn register_userdata(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<InteropData>("InteropData")
        .register_fn("InteropData_new", InteropData::new)
        .register_fn("get", InteropData::get)
        .register_get_set("value", InteropData::value, InteropData::set_value)
        .register_type_with_name::<CopyData>("CopyData")
        .register_fn("CopyData_new", CopyData::new)
        .register_type_with_name::<RcData>("RcData")
        .register_fn("RcData_new", RcData::new);
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
        .register_fn("interop_sum_f64_list", |values: rhai::Array| -> f64 {
            values
                .iter()
                .map(|value| value.as_float().unwrap_or(0.0))
                .sum()
        })
        .register_fn("interop_sum_u8_list", |values: rhai::Array| -> f64 {
            values
                .iter()
                .map(|value| value.as_int().unwrap_or(0) as f64)
                .sum()
        })
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
