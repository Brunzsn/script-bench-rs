use anyhow::{anyhow, Result};
use roto::{library, List, RotoString, Runtime, Val};

use crate::harness::{script_path, Case, Engine, Value};
use crate::rng;

#[derive(Clone, Default, PartialEq)]
pub struct RustData(pub RotoString);

type BenchFn = fn() -> List<Val<RustData>>;

pub struct RotoEngine {
    func: roto::TypedFunc<roto::NoCtx, fn() -> List<Val<RustData>>>,
}

impl Engine for RotoEngine {
    fn setup(case: &Case) -> Result<Self> {
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
            .compile(script_path(case.scripts.roto))
            .map_err(|error| anyhow!("{error:?}"))?;
        let func = package
            .get_function::<BenchFn>("bench")
            .map_err(|error| anyhow!("{error:?}"))?;

        Ok(Self { func })
    }

    fn run(&mut self) -> Result<()> {
        self.func.call();
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let list = self.func.call();
        let mut items = Vec::new();
        for next in list {
            let data = next.0;
            items.push(Value::Object((&*data.0).to_string()));
        }
        Ok(Value::List(items))
    }
}
