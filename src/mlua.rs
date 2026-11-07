use std::rc::Rc;

use anyhow::{anyhow, Result};
use mlua::{
    Function, Lua, LuaString, MetaMethod, Table, UserData, UserDataMethods, UserDataRef,
    UserDataRegistry,
};
use rand::RngExt;

use crate::harness::{read_script, Case, Engine, Value};

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustData(Rc<str>);

impl UserData for RustData {
    fn register(registry: &mut UserDataRegistry<Self>) {
        registry.add_function("new", |_, s: LuaString| Ok(RustData((*s.to_str()?).into())));
        registry.add_meta_method(MetaMethod::Lt, |_, this, rhs: UserDataRef<Self>| {
            Ok(this < &rhs)
        });
        registry.add_meta_method(MetaMethod::ToString, |_, this, ()| Ok(this.0.to_string()));

        #[cfg(feature = "mlua_luau")]
        registry.enable_namecall();
    }
}

pub struct MluaEngine {
    // The function handle must be dropped before the Lua state.
    bench: Function,
    _lua: Lua,
}

impl Engine for MluaEngine {
    fn setup(case: &Case) -> Result<Self> {
        let lua = Lua::new();

        let globals = lua.globals();
        let rust_data = lua
            .create_proxy::<RustData>()
            .map_err(|error| anyhow!("{error}"))?;
        globals
            .set("RustData", rust_data)
            .map_err(|error| anyhow!("{error}"))?;
        globals
            .set(
                "rand",
                Function::wrap_raw(|n: u32| rand::rng().random_range(0..n)),
            )
            .map_err(|error| anyhow!("{error}"))?;

        #[cfg(feature = "mlua_luau")]
        {
            lua.sandbox(true).map_err(|error| anyhow!("{error}"))?;
            lua.set_jit_options(mlua::state::JitOptions::new().inliner(true));
            lua.set_compiler(mlua::chunk::Compiler::new().set_optimization_level(2));
        }

        #[cfg(feature = "mlua_lua55")]
        {
            let table = lua
                .globals()
                .get::<mlua::Table>("table")
                .map_err(|error| anyhow!("{error}"))?;
            table
                .set(
                    "create",
                    lua.create_function(|lua, narr: usize| {
                        lua.create_table_with_capacity(narr, 0)
                    })
                    .map_err(|error| anyhow!("{error}"))?,
                )
                .map_err(|error| anyhow!("{error}"))?;
        }

        let script = read_script(case.scripts.lua)?;
        lua.load(script.as_str())
            .exec()
            .map_err(|error| anyhow!("{error}"))?;

        let bench = lua
            .globals()
            .get::<Function>("bench")
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Self { bench, _lua: lua })
    }

    fn run(&mut self) -> Result<()> {
        self.bench
            .call::<()>(())
            .map_err(|error| anyhow!("{error}"))?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let table: Table = self
            .bench
            .call(())
            .map_err(|error| anyhow!("{error}"))?;

        let mut items = Vec::new();
        table
            .for_each_value(|next: UserDataRef<RustData>| {
                items.push(Value::Object(next.0.to_string()));
                Ok(())
            })
            .map_err(|error| anyhow!("{error}"))?;

        Ok(Value::List(items))
    }
}
