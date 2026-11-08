use anyhow::{anyhow, bail, Result};
use koto::prelude::*;

use crate::harness::{number_value, read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};

enum KotoMode {
    Call,
    CallIn { iterations: usize },
    Callback { iterations: usize },
}

pub struct KotoInterop {
    engine: Koto,
    entry: KValue,
    mode: KotoMode,
    result: ResultKind,
}

impl InteropEngine for KotoInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        let mode = match case.op {
            InteropOp::Call => KotoMode::Call,
            InteropOp::CallIn { iterations } => KotoMode::CallIn { iterations },
            InteropOp::Callback { iterations } => KotoMode::Callback { iterations },
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
}
