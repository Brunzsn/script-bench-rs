use anyhow::{anyhow, bail, Result};
use roto::{library, RotoString, Runtime};

use crate::harness::{script_path, ResultKind, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};

type IntBenchFn = fn() -> u64;
type FloatBenchFn = fn() -> f64;
type UpdateFn = fn(f64) -> f64;

enum RotoInteropFn {
    Int(roto::TypedFunc<roto::NoCtx, IntBenchFn>),
    Float(roto::TypedFunc<roto::NoCtx, FloatBenchFn>),
    CallIn(roto::TypedFunc<roto::NoCtx, UpdateFn>, usize),
}

pub struct RotoInterop {
    func: RotoInteropFn,
}

impl InteropEngine for RotoInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        if !matches!(case.op, InteropOp::Call | InteropOp::CallIn { .. }) {
            return Ok(None);
        }
        let Some(script) = case.scripts.roto else {
            return Ok(None);
        };

        let lib = library! {
            fn interop_ping() -> u64 {
                host::ping().max(0) as u64
            }

            fn interop_id_i64(value: u64) -> u64 {
                value
            }

            fn interop_sum_i64_3(a: u64, b: u64, c: u64) -> u64 {
                a + b + c
            }

            fn interop_sum_i64_5(a: u64, b: u64, c: u64, d: u64, e: u64) -> u64 {
                a + b + c + d + e
            }

            fn interop_id_f64(value: f64) -> f64 {
                host::id_f64(value)
            }

            fn interop_strlen(value: RotoString) -> u64 {
                value.bytes().len() as u64
            }

            fn interop_string_out() -> RotoString {
                host::string_out().into()
            }

            impl RotoString {
                fn len(self) -> u64 {
                    self.bytes().len() as u64
                }
            }
        };

        let runtime = Runtime::from_lib(lib).map_err(|error| anyhow!("{error:?}"))?;
        let mut package = runtime
            .compile(script_path(script))
            .map_err(|error| anyhow!("{error:?}"))?;

        let func = match case.op {
            InteropOp::CallIn { iterations } => RotoInteropFn::CallIn(
                package
                    .get_function::<UpdateFn>(case.entry)
                    .map_err(|error| anyhow!("{error:?}"))?,
                iterations,
            ),
            InteropOp::Call => match case.result {
                ResultKind::Int => RotoInteropFn::Int(
                    package
                        .get_function::<IntBenchFn>(case.entry)
                        .map_err(|error| anyhow!("{error:?}"))?,
                ),
                ResultKind::Float => RotoInteropFn::Float(
                    package
                        .get_function::<FloatBenchFn>(case.entry)
                        .map_err(|error| anyhow!("{error:?}"))?,
                ),
                ResultKind::UserDataList => {
                    bail!("interop cases cannot return userdata lists")
                }
            },
            _ => return Ok(None),
        };

        Ok(Some(Self { func }))
    }

    fn run(&mut self) -> Result<()> {
        match &self.func {
            RotoInteropFn::Int(func) => {
                func.call();
            }
            RotoInteropFn::Float(func) => {
                func.call();
            }
            RotoInteropFn::CallIn(func, iterations) => {
                let mut sum = 0.0f64;
                for _ in 0..*iterations {
                    sum += func.call(host::UPDATE_DT);
                }
                std::hint::black_box(sum);
            }
        }
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        match &self.func {
            RotoInteropFn::Int(func) => Ok(Value::Int(func.call() as i64)),
            RotoInteropFn::Float(func) => Ok(Value::Float(func.call())),
            RotoInteropFn::CallIn(func, iterations) => {
                let mut sum = 0.0f64;
                for _ in 0..*iterations {
                    sum += func.call(host::UPDATE_DT);
                }
                Ok(Value::Float(sum))
            }
        }
    }
}
