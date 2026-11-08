use anyhow::{anyhow, bail, Result};
use martite::{HostSignature, HostType, Instance, Outcome, Plugin};

use crate::harness::{read_script, ResultKind, Value as BenchValue};
use crate::interop::host;
use crate::interop::{
    BulkMode, BulkValue, Direction, InteropCase, InteropEngine, InteropOp,
};
use crate::martite::Value;

use super::{LIMITS, MANIFEST};

enum MartiteMode {
    Call,
    CallIn { iterations: usize },
    Callback { iterations: usize },
    Userdata,
    BulkListIn { value: BulkValue, size: usize },
    BulkListOut,
    SerdeFromScript,
    Error { propagates: bool },
}

pub struct MartiteInterop {
    instance: Instance<'static>,
    entry: &'static str,
    mode: MartiteMode,
    result: ResultKind,
}

impl InteropEngine for MartiteInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        let mode = match case.op {
            InteropOp::Call => MartiteMode::Call,
            InteropOp::CallIn { iterations } => MartiteMode::CallIn { iterations },
            InteropOp::Callback { iterations } => MartiteMode::Callback { iterations },
            InteropOp::Userdata(_) => MartiteMode::Userdata,
            InteropOp::Bulk(op) if op.mode == BulkMode::List => match op.direction {
                Direction::In => MartiteMode::BulkListIn {
                    value: op.value,
                    size: op.size,
                },
                Direction::Out => MartiteMode::BulkListOut,
            },
            InteropOp::Serde(crate::interop::SerdeOp::FromScript) => {
                MartiteMode::SerdeFromScript
            }
            InteropOp::Error(op) => MartiteMode::Error {
                propagates: matches!(op, crate::interop::ErrorOp::PropagatesToHost),
            },
            _ => return Ok(None),
        };
        let (Some(script_path), Some(interface_path)) = (
            case.scripts.martite_script,
            case.scripts.martite_interface,
        ) else {
            return Ok(None);
        };

        let mut engine = martite::Engine::new();
        engine.set_limits(LIMITS, LIMITS);
        let interface = read_script(interface_path)?;
        engine.interface("bench.data", interface);
        register_host_functions(&mut engine);

        let script = read_script(script_path)?;
        let provider = martite::memory_provider(&[("main.mt", script.as_str())]);
        let plugin = engine.load(MANIFEST, &provider).map_err(|diagnostics| {
            let details = diagnostics
                .iter()
                .map(|diagnostic| {
                    format!(
                        "{:?}: {} at {:?}",
                        diagnostic.code, diagnostic.message, diagnostic.span
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            anyhow!(
                "Martite load failed: {} diagnostics\n{details}",
                diagnostics.len()
            )
        })?;

        // `Instance` borrows its `Plugin`, and re-instantiating per iteration
        // would change what the benchmark measures. The process is short-lived,
        // so keep the plugin alive for its duration.
        let plugin: &'static Plugin = Box::leak(Box::new(plugin));
        let instance = plugin
            .instantiate()
            .map_err(|trap| anyhow!("Martite instantiate failed: {}", trap.panic_message()))?;

        Ok(Some(Self {
            instance,
            entry: case.entry,
            mode,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        match self.mode {
            MartiteMode::Call => {
                self.instance
                    .call(self.entry, Vec::new())
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
            }
            MartiteMode::CallIn { iterations } => {
                let mut sum = 0.0f64;
                for _ in 0..iterations {
                    let outcome = self
                        .instance
                        .call(self.entry, vec![Value::Float(host::UPDATE_DT)])
                        .map_err(|trap| {
                            anyhow!("Martite interop failed: {}", trap.panic_message())
                        })?;
                    let Outcome::Handled(Value::Float(value)) = outcome else {
                        bail!("expected `{}` to return a float", self.entry);
                    };
                    sum += value;
                }
                std::hint::black_box(sum);
            }
            MartiteMode::Callback { iterations } => {
                let mut sum = 0i64;
                for index in 0..iterations {
                    let outcome = self
                        .instance
                        .call(self.entry, vec![Value::Int(index as i64)])
                        .map_err(|trap| {
                            anyhow!("Martite interop failed: {}", trap.panic_message())
                        })?;
                    let Outcome::Handled(Value::Int(value)) = outcome else {
                        bail!("expected `{}` to return an int", self.entry);
                    };
                    sum = sum.wrapping_add(value);
                }
                std::hint::black_box(sum);
            }
            MartiteMode::Userdata => {
                let outcome = self
                    .instance
                    .call(self.entry, Vec::new())
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
                let Outcome::Handled(Value::Int(value)) = outcome else {
                    bail!("expected `{}` to return an int", self.entry);
                };
                std::hint::black_box(value);
            }
            MartiteMode::BulkListIn { value, size } => {
                let list = match value {
                    BulkValue::F64 => Value::list(
                        host::bulk_f64_values(size)
                            .into_iter()
                            .map(Value::Float)
                            .collect(),
                    ),
                    BulkValue::U8 => Value::list(
                        host::bulk_u8_values(size)
                            .into_iter()
                            .map(|value| Value::Int(i64::from(value)))
                            .collect(),
                    ),
                };
                let outcome = self
                    .instance
                    .call(self.entry, vec![list])
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
                std::hint::black_box(outcome);
            }
            MartiteMode::BulkListOut => {
                let outcome = self
                    .instance
                    .call(self.entry, Vec::new())
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
                std::hint::black_box(outcome);
            }
            MartiteMode::SerdeFromScript => {
                let outcome = self
                    .instance
                    .call(self.entry, Vec::new())
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
                let checksum = martite_serde_checksum(outcome)?;
                std::hint::black_box(checksum);
            }
            MartiteMode::Error { propagates } => {
                let outcome = self
                    .instance
                    .call(self.entry, Vec::new())
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
                match outcome {
                    Outcome::Handled(Value::Result(Err(_))) if propagates => {}
                    Outcome::Handled(Value::Int(value)) if !propagates => {
                        std::hint::black_box(value);
                    }
                    Outcome::Handled(_) => bail!("unexpected error-case outcome"),
                    Outcome::NoHandler => bail!("Martite plugin has no `{}` handler", self.entry),
                }
            }
        }
        Ok(())
    }

    fn run_checked(&mut self) -> Result<BenchValue> {
        match self.mode {
            MartiteMode::Call => {
                let outcome = self.instance.call(self.entry, Vec::new()).map_err(|trap| {
                    anyhow!("Martite interop failed: {}", trap.panic_message())
                })?;

                match (self.result, outcome) {
                    (ResultKind::Int, Outcome::Handled(Value::Int(value))) => {
                        Ok(BenchValue::Int(value))
                    }
                    (ResultKind::Float, Outcome::Handled(Value::Float(value))) => {
                        Ok(BenchValue::Float(value))
                    }
                    (ResultKind::Int, Outcome::Handled(_)) => {
                        bail!("expected `{}` to return an int", self.entry)
                    }
                    (ResultKind::Float, Outcome::Handled(_)) => {
                        bail!("expected `{}` to return a float", self.entry)
                    }
                    (_, Outcome::NoHandler) => {
                        bail!("Martite plugin has no `{}` handler", self.entry)
                    }
                    (ResultKind::UserDataList, Outcome::Handled(_)) => {
                        bail!("interop cases cannot return userdata lists")
                    }
                }
            }
            MartiteMode::CallIn { iterations } => {
                let mut sum = 0.0f64;
                for _ in 0..iterations {
                    let outcome = self
                        .instance
                        .call(self.entry, vec![Value::Float(host::UPDATE_DT)])
                        .map_err(|trap| {
                            anyhow!("Martite interop failed: {}", trap.panic_message())
                        })?;
                    let Outcome::Handled(Value::Float(value)) = outcome else {
                        bail!("expected `{}` to return a float", self.entry);
                    };
                    sum += value;
                }
                Ok(BenchValue::Float(sum))
            }
            MartiteMode::Callback { iterations } => {
                let mut sum = 0i64;
                for index in 0..iterations {
                    let outcome = self
                        .instance
                        .call(self.entry, vec![Value::Int(index as i64)])
                        .map_err(|trap| {
                            anyhow!("Martite interop failed: {}", trap.panic_message())
                        })?;
                    let Outcome::Handled(Value::Int(value)) = outcome else {
                        bail!("expected `{}` to return an int", self.entry);
                    };
                    sum = sum.wrapping_add(value);
                }
                Ok(BenchValue::Int(sum))
            }
            MartiteMode::Userdata => {
                let outcome = self
                    .instance
                    .call(self.entry, Vec::new())
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
                let Outcome::Handled(Value::Int(value)) = outcome else {
                    bail!("expected `{}` to return an int", self.entry);
                };
                Ok(BenchValue::Int(value))
            }
            MartiteMode::BulkListIn { value, size } => {
                let list = match value {
                    BulkValue::F64 => Value::list(
                        host::bulk_f64_values(size)
                            .into_iter()
                            .map(Value::Float)
                            .collect(),
                    ),
                    BulkValue::U8 => Value::list(
                        host::bulk_u8_values(size)
                            .into_iter()
                            .map(|value| Value::Int(i64::from(value)))
                            .collect(),
                    ),
                };
                let outcome = self
                    .instance
                    .call(self.entry, vec![list])
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
                match outcome {
                    Outcome::Handled(Value::Float(value)) => Ok(BenchValue::Float(value)),
                    Outcome::Handled(Value::Int(value)) => Ok(BenchValue::Int(value)),
                    Outcome::Handled(_) => bail!("expected a numeric bulk result"),
                    Outcome::NoHandler => bail!("Martite plugin has no `{}` handler", self.entry),
                }
            }
            MartiteMode::BulkListOut => {
                let outcome = self
                    .instance
                    .call(self.entry, Vec::new())
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
                match outcome {
                    Outcome::Handled(Value::Float(value)) => Ok(BenchValue::Float(value)),
                    Outcome::Handled(Value::Int(value)) => Ok(BenchValue::Int(value)),
                    Outcome::Handled(_) => bail!("expected a numeric bulk result"),
                    Outcome::NoHandler => bail!("Martite plugin has no `{}` handler", self.entry),
                }
            }
            MartiteMode::SerdeFromScript => {
                let outcome = self
                    .instance
                    .call(self.entry, Vec::new())
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
                Ok(BenchValue::Int(martite_serde_checksum(outcome)?))
            }
            MartiteMode::Error { propagates } => {
                let outcome = self
                    .instance
                    .call(self.entry, Vec::new())
                    .map_err(|trap| {
                        anyhow!("Martite interop failed: {}", trap.panic_message())
                    })?;
                match outcome {
                    Outcome::Handled(Value::Result(Err(_))) if propagates => {
                        Ok(BenchValue::Int(1))
                    }
                    Outcome::Handled(Value::Int(value)) if !propagates => {
                        Ok(BenchValue::Int(value))
                    }
                    Outcome::Handled(_) => bail!("unexpected error-case outcome"),
                    Outcome::NoHandler => bail!("Martite plugin has no `{}` handler", self.entry),
                }
            }
        }
    }
}

fn martite_serde_checksum(outcome: Outcome) -> Result<i64> {
    let Outcome::Handled(Value::Struct(nested)) = outcome else {
        bail!("expected a nested struct result");
    };
    let id = nested.fields[0].as_int().unwrap_or_default();
    let name_len = nested.fields[1]
        .as_str()
        .map(|name| name.chars().count() as i64)
        .unwrap_or_default();
    let mut value_sum = 0.0f64;
    if let Value::List(values) = &nested.fields[2] {
        for value in values.iter() {
            if let Value::Float(value) = value {
                value_sum += value;
            }
        }
    }
    let child_id = match &nested.fields[3] {
        Value::Struct(child) => child.fields[0].as_int().unwrap_or_default(),
        _ => 0,
    };
    Ok(id + name_len + value_sum as i64 + child_id)
}

fn register_host_functions(engine: &mut martite::Engine) {
    engine.host_fn(
        "bench.data.interop_ping",
        HostSignature::new(vec![], HostType::Int),
        |_cx, _args| Ok(Value::Int(host::ping())),
    );

    engine.host_fn(
        "bench.data.interop_id_i64",
        HostSignature::new(vec![HostType::Int], HostType::Int),
        |_cx, args| Ok(Value::Int(host::id_i64(args[0].as_int().unwrap_or_default()))),
    );

    engine.host_fn(
        "bench.data.interop_sum_i64_3",
        HostSignature::new(
            vec![HostType::Int, HostType::Int, HostType::Int],
            HostType::Int,
        ),
        |_cx, args| {
            Ok(Value::Int(host::sum_i64_3(
                args[0].as_int().unwrap_or_default(),
                args[1].as_int().unwrap_or_default(),
                args[2].as_int().unwrap_or_default(),
            )))
        },
    );

    engine.host_fn(
        "bench.data.interop_sum_i64_5",
        HostSignature::new(
            vec![
                HostType::Int,
                HostType::Int,
                HostType::Int,
                HostType::Int,
                HostType::Int,
            ],
            HostType::Int,
        ),
        |_cx, args| {
            Ok(Value::Int(host::sum_i64_5(
                args[0].as_int().unwrap_or_default(),
                args[1].as_int().unwrap_or_default(),
                args[2].as_int().unwrap_or_default(),
                args[3].as_int().unwrap_or_default(),
                args[4].as_int().unwrap_or_default(),
            )))
        },
    );

    engine.host_fn(
        "bench.data.interop_id_f64",
        HostSignature::new(vec![HostType::Float], HostType::Float),
        |_cx, args| {
            let value = match &args[0] {
                Value::Float(value) => *value,
                _ => 0.0,
            };
            Ok(Value::Float(host::id_f64(value)))
        },
    );

    engine.host_fn(
        "bench.data.interop_strlen",
        HostSignature::new(vec![HostType::Str], HostType::Int),
        |_cx, args| {
            let value = args[0].as_str().unwrap_or_default();
            Ok(Value::Int(host::strlen(value)))
        },
    );

    engine.host_fn(
        "bench.data.interop_string_out",
        HostSignature::new(vec![], HostType::Str),
        |_cx, _args| Ok(Value::str(host::string_out())),
    );

    engine.host_fn(
        "bench.data.interop_callback",
        HostSignature::new(vec![HostType::Int], HostType::Int),
        |_cx, args| {
            Ok(Value::Int(host::callback_value(
                args[0].as_int().unwrap_or_default(),
            )))
        },
    );

    engine.host_fn(
        "bench.data.interop_sum_f64_list",
        HostSignature::new(
            vec![HostType::List(Box::new(HostType::Float))],
            HostType::Float,
        ),
        |_cx, args| {
            let mut sum = 0.0f64;
            if let Value::List(values) = &args[0] {
                for value in values.iter() {
                    if let Value::Float(value) = value {
                        sum += value;
                    }
                }
            }
            Ok(Value::Float(sum))
        },
    );

    engine.host_fn(
        "bench.data.interop_sum_int_list",
        HostSignature::new(
            vec![HostType::List(Box::new(HostType::Int))],
            HostType::Int,
        ),
        |_cx, args| {
            let mut sum = 0i64;
            if let Value::List(values) = &args[0] {
                for value in values.iter() {
                    if let Value::Int(value) = value {
                        sum += value;
                    }
                }
            }
            Ok(Value::Int(sum))
        },
    );

    engine.host_fn(
        "bench.data.interop_new_doc",
        HostSignature::new(vec![HostType::Int], HostType::Handle("Doc".into())),
        |cx, args| {
            let value = args[0].as_int().unwrap_or_default();
            cx.make_handle(value.to_string())
        },
    );

    engine.host_fn(
        "bench.data.interop_method",
        HostSignature::new(vec![HostType::Handle("Doc".into())], HostType::Int),
        |_cx, _args| Ok(Value::Int(host::USERDATA_VALUE)),
    );

    engine.host_fn(
        "bench.data.interop_fail",
        HostSignature::new(
            vec![],
            HostType::Result(
                Box::new(HostType::Int),
                Box::new(HostType::Str),
            ),
        ),
        |_cx, _args| {
            Ok(Value::Result(Err(Box::new(Value::str(host::HOST_ERROR)))))
        },
    );
}
