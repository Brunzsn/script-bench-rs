use anyhow::{anyhow, bail, Result};
use martite::{HostSignature, HostType, Instance, Outcome, Plugin};

use crate::harness::{read_script, ResultKind, Value as BenchValue};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};
use crate::martite::Value;

use super::{LIMITS, MANIFEST};

enum MartiteMode {
    Call,
    CallIn { iterations: usize },
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
        }
    }
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
}
