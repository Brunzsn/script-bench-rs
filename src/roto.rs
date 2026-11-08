use anyhow::{anyhow, Result};
use roto::{library, List, RotoString, Runtime, Val};

mod interop;
pub use interop::RotoInterop;

use crate::harness::{script_path, Case, Engine, ResultKind, Value};
use crate::rng;

#[derive(Clone, Default, PartialEq)]
pub struct RustData(pub RotoString);

type ListBenchFn = fn() -> List<Val<RustData>>;
type IntBenchFn = fn() -> u64;
type FloatBenchFn = fn() -> f64;

enum BenchFn {
    List(roto::TypedFunc<roto::NoCtx, ListBenchFn>),
    Int(roto::TypedFunc<roto::NoCtx, IntBenchFn>),
    Float(roto::TypedFunc<roto::NoCtx, FloatBenchFn>),
}

pub struct RotoEngine {
    func: BenchFn,
}

impl Engine for RotoEngine {
    fn setup(case: &Case) -> Result<Option<Self>> {
        let Some(script) = case.scripts.roto else {
            return Ok(None);
        };

        let lib = library! {
            fn rand(n: u64) -> u64 {
                rng::rand_below(n as u32) as u64
            }

            impl RotoString {
                fn get(self, idx: u64) -> RotoString {
                    self[idx as usize..idx as usize + 1].into()
                }

                fn len(self) -> u64 {
                    self.bytes().len() as u64
                }
            }

            #[clone] type RustData = Val<RustData>;

            impl Val<RustData> {
                fn new(s: RotoString) -> Val<RustData> {
                    Val(RustData(s))
                }

                fn lt(this: Val<RustData>, rhs: Val<RustData>) -> bool {
                    &*this.0.0 < &*rhs.0.0
                }
            }
        };

        let runtime = Runtime::from_lib(lib).map_err(|error| anyhow!("{error:?}"))?;
        let mut package = runtime
            .compile(script_path(script))
            .map_err(|error| anyhow!("{error:?}"))?;

        let func = match case.result {
            ResultKind::UserDataList => BenchFn::List(
                package
                    .get_function::<ListBenchFn>("bench")
                    .map_err(|error| anyhow!("{error:?}"))?,
            ),
            ResultKind::Int => BenchFn::Int(
                package
                    .get_function::<IntBenchFn>("bench")
                    .map_err(|error| anyhow!("{error:?}"))?,
            ),
            ResultKind::Float => BenchFn::Float(
                package
                    .get_function::<FloatBenchFn>("bench")
                    .map_err(|error| anyhow!("{error:?}"))?,
            ),
        };

        Ok(Some(Self { func }))
    }

    fn run(&mut self) -> Result<()> {
        match &self.func {
            BenchFn::List(func) => {
                func.call();
            }
            BenchFn::Int(func) => {
                func.call();
            }
            BenchFn::Float(func) => {
                func.call();
            }
        }
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        match &self.func {
            BenchFn::List(func) => {
                let list = func.call();
                let mut items = Vec::new();
                for next in list {
                    let data = next.0;
                    items.push(Value::Object((&*data.0).to_string()));
                }
                Ok(Value::List(items))
            }
            BenchFn::Int(func) => Ok(Value::Int(func.call() as i64)),
            BenchFn::Float(func) => Ok(Value::Float(func.call())),
        }
    }
}
