use std::rc::Rc;

use anyhow::{anyhow, bail, Result};
use koto::{derive::*, prelude::*, runtime};
use rand::RngExt;

use crate::harness::{read_script, Case, Engine, Value};

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, KotoCopy, KotoType)]
pub struct RustData(pub Rc<str>);

#[koto_impl]
impl RustData {
    fn new_koto_object(s: &str) -> KObject {
        let me = Self(s.into());
        KObject::from(me)
    }
}

impl KotoObject for RustData {
    fn display(&self, ctx: &mut DisplayContext) -> runtime::Result<()> {
        ctx.append(self.0.to_string());
        Ok(())
    }

    fn less(&self, rhs: &KValue) -> runtime::Result<bool> {
        if let KValue::Object(kobj) = rhs {
            let rhs_dc = kobj.cast::<RustData>()?;
            Ok(*self < *rhs_dc)
        } else {
            unexpected_type("RustData object", rhs)
        }
    }
}

pub struct KotoEngine {
    engine: Koto,
    bench: KValue,
}

impl Engine for KotoEngine {
    fn setup(case: &Case) -> Result<Self> {
        let mut engine = Koto::default();
        let prelude = engine.prelude();

        prelude.add_fn("RustData_new", |ctx| match ctx.args() {
            [KValue::Str(input)] => Ok(RustData::new_koto_object(input.as_str()).into()),
            unexpected => unexpected_args("a string", unexpected),
        });

        prelude.add_fn("rand", |ctx| match ctx.args() {
            [KValue::Number(n)] => {
                let res = rand::random::<u32>() as i64 % i64::from(n);
                Ok(res.into())
            }
            unexpected => unexpected_args("a number", unexpected),
        });

        let script = read_script(case.scripts.koto)?;
        engine
            .compile_and_run(script.as_str())
            .map_err(|error| anyhow!("{error}"))?;

        let bench = engine
            .exports()
            .get("bench")
            .ok_or_else(|| anyhow!("missing `bench` function"))?;

        Ok(Self { engine, bench })
    }

    fn run(&mut self) -> Result<()> {
        self.engine
            .call_function(self.bench.clone(), &[])
            .map_err(|error| anyhow!("{error}"))?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let value = self
            .engine
            .call_function(self.bench.clone(), &[])
            .map_err(|error| anyhow!("{error}"))?;

        let KValue::List(list) = value else {
            bail!("expected the script to return a list");
        };

        let mut items = Vec::new();
        for item in list.data().iter() {
            let KValue::Object(object) = item else {
                bail!("expected a RustData object");
            };
            let data = object
                .cast::<RustData>()
                .map_err(|error| anyhow!("{error}"))?;
            items.push(Value::Object(data.0.to_string()));
        }

        Ok(Value::List(items))
    }
}
