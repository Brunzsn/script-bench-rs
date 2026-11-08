use std::cell::Cell;
use std::rc::Rc;

use anyhow::{anyhow, bail, Result};
use mlua::{
    Function, Lua, Table, UserData, UserDataFields, UserDataMethods, UserDataRegistry,
};

use crate::interop::{BulkMode, BulkValue, Direction};

use crate::harness::{number_value, read_script, ResultKind, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropEngine, InteropOp};

#[derive(Default, Clone)]
pub struct InteropData(Cell<i64>);

impl UserData for InteropData {
    fn register(registry: &mut UserDataRegistry<Self>) {
        registry.add_function("new", |_, value: i64| Ok(InteropData(Cell::new(value))));
        registry.add_method("get", |_, this, ()| Ok(this.0.get()));
        registry.add_field_method_get("value", |_, this| Ok(this.0.get()));
        registry.add_field_method_set("value", |_, this, value: i64| {
            this.0.set(value);
            Ok(())
        });
    }
}

#[derive(Default, Clone, Copy)]
pub struct CopyData(i64);

impl UserData for CopyData {
    fn register(registry: &mut UserDataRegistry<Self>) {
        registry.add_function("new", |_, value: i64| Ok(CopyData(value)));
    }
}

#[derive(Default, Clone)]
pub struct RcData(Rc<str>);

impl UserData for RcData {
    fn register(registry: &mut UserDataRegistry<Self>) {
        registry.add_function("new", |_, value: String| Ok(RcData(value.into())));
    }
}

enum MluaMode {
    Call,
    CallIn { iterations: usize },
    Callback { iterations: usize },
    Userdata,
    BulkListIn { value: BulkValue, size: usize },
    BulkListOut,
    BulkBufferIn { value: BulkValue, size: usize },
    BulkBufferOut,
    SerdeToScript,
    SerdeFromScript,
    Error { propagates: bool },
}

pub struct MluaInterop {
    // The function handle must be dropped before the Lua state.
    entry: Function,
    _lua: Lua,
    mode: MluaMode,
    result: ResultKind,
}

impl MluaInterop {
    fn serde_table(&mut self) -> Result<Table> {
        let child = self
            ._lua
            .create_table()
            .map_err(|error| anyhow!("{error}"))?;
        child
            .set("id", host::SERDE_CHILD_ID)
            .map_err(|error| anyhow!("{error}"))?;
        let values = self
            ._lua
            .create_table()
            .map_err(|error| anyhow!("{error}"))?;
        for (index, value) in host::SERDE_VALUES.iter().enumerate() {
            values
                .set(index + 1, *value)
                .map_err(|error| anyhow!("{error}"))?;
        }
        let object = self
            ._lua
            .create_table()
            .map_err(|error| anyhow!("{error}"))?;
        object
            .set("id", host::SERDE_ID)
            .map_err(|error| anyhow!("{error}"))?;
        object
            .set("name", host::SERDE_NAME)
            .map_err(|error| anyhow!("{error}"))?;
        object
            .set("values", values)
            .map_err(|error| anyhow!("{error}"))?;
        object
            .set("child", child)
            .map_err(|error| anyhow!("{error}"))?;
        Ok(object)
    }

    fn serde_checksum(&mut self, object: Table) -> Result<i64> {
        let id: f64 = object.get("id").map_err(|error| anyhow!("{error}"))?;
        let name: String = object.get("name").map_err(|error| anyhow!("{error}"))?;
        let values: Table = object
            .get("values")
            .map_err(|error| anyhow!("{error}"))?;
        let mut value_sum = 0.0f64;
        for value in values.sequence_values::<f64>() {
            value_sum += value.map_err(|error| anyhow!("{error}"))?;
        }
        let child: Table = object.get("child").map_err(|error| anyhow!("{error}"))?;
        let child_id: f64 = child.get("id").map_err(|error| anyhow!("{error}"))?;
        Ok(id as i64 + name.len() as i64 + value_sum as i64 + child_id as i64)
    }

    #[cfg(feature = "mlua_luau")]
    fn buffer_in(&mut self, value: BulkValue, size: usize) -> Result<f64> {
        let bytes = match value {
            BulkValue::F64 => {
                let mut bytes = Vec::with_capacity(size * 8);
                for value in host::bulk_f64_values(size) {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
                bytes
            }
            BulkValue::U8 => host::bulk_u8_values(size),
        };
        let buffer = self
            ._lua
            .create_buffer(bytes)
            .map_err(|error| anyhow!("{error}"))?;
        self.entry
            .call(buffer)
            .map_err(|error| anyhow!("{error}"))
    }

    #[cfg(feature = "mlua_luau")]
    fn buffer_out(&mut self) -> Result<f64> {
        self.entry.call(()).map_err(|error| anyhow!("{error}"))
    }
}

impl InteropEngine for MluaInterop {
    fn setup(case: &InteropCase) -> Result<Option<Self>> {
        #[cfg(not(feature = "mlua_luau"))]
        if matches!(case.op, InteropOp::Bulk(op) if op.mode == BulkMode::Buffer) {
            return Ok(None);
        }
        let mode = match case.op {
            InteropOp::Call => MluaMode::Call,
            InteropOp::CallIn { iterations } => MluaMode::CallIn { iterations },
            InteropOp::Callback { iterations } => MluaMode::Callback { iterations },
            InteropOp::Userdata(_) => MluaMode::Userdata,
            InteropOp::Bulk(op) if op.mode == BulkMode::List => match op.direction {
                Direction::In => MluaMode::BulkListIn {
                    value: op.value,
                    size: op.size,
                },
                Direction::Out => MluaMode::BulkListOut,
            },
            InteropOp::Bulk(op) if op.mode == BulkMode::Buffer => match op.direction {
                Direction::In => MluaMode::BulkBufferIn {
                    value: op.value,
                    size: op.size,
                },
                Direction::Out => MluaMode::BulkBufferOut,
            },
            InteropOp::Serde(op) => match op {
                crate::interop::SerdeOp::ToScript => MluaMode::SerdeToScript,
                crate::interop::SerdeOp::FromScript => MluaMode::SerdeFromScript,
            },
            InteropOp::Error(op) => MluaMode::Error {
                propagates: matches!(op, crate::interop::ErrorOp::PropagatesToHost),
            },
            _ => return Ok(None),
        };
        let is_callback = matches!(mode, MluaMode::Callback { .. });
        let is_userdata = matches!(mode, MluaMode::Userdata);
        let Some(script_path) = case.scripts.lua else {
            return Ok(None);
        };
        let lua = Lua::new();
        register_host_functions(&lua)?;
        if is_userdata {
            let globals = lua.globals();
            let proxy = lua
                .create_proxy::<InteropData>()
                .map_err(|error| anyhow!("{error}"))?;
            globals
                .set("InteropData", proxy)
                .map_err(|error| anyhow!("{error}"))?;
            let proxy = lua
                .create_proxy::<CopyData>()
                .map_err(|error| anyhow!("{error}"))?;
            globals
                .set("CopyData", proxy)
                .map_err(|error| anyhow!("{error}"))?;
            let proxy = lua
                .create_proxy::<RcData>()
                .map_err(|error| anyhow!("{error}"))?;
            globals
                .set("RcData", proxy)
                .map_err(|error| anyhow!("{error}"))?;
        }

        #[cfg(feature = "mlua_luau")]
        {
            lua.sandbox(true).map_err(|error| anyhow!("{error}"))?;
            lua.set_jit_options(mlua::state::JitOptions::new().inliner(true));
            lua.set_compiler(mlua::chunk::Compiler::new().set_optimization_level(2));
        }

        let script = read_script(script_path)?;
        lua.load(script.as_str())
            .exec()
            .map_err(|error| anyhow!("{error}"))?;

        let entry = lua
            .globals()
            .get::<Function>(case.entry)
            .map_err(|error| anyhow!("{error}"))?;
        let entry = if is_callback {
            entry.call::<Function>(()).map_err(|error| anyhow!("{error}"))?
        } else {
            entry
        };

        Ok(Some(Self {
            entry,
            _lua: lua,
            mode,
            result: case.result,
        }))
    }

    fn run(&mut self) -> Result<()> {
        match self.mode {
            MluaMode::Call => {
                self.entry
                    .call::<mlua::Value>(())
                    .map_err(|error| anyhow!("{error}"))?;
            }
            MluaMode::CallIn { iterations } => {
                let mut sum = 0.0f64;
                for _ in 0..iterations {
                    let value: f64 = self
                        .entry
                        .call(host::UPDATE_DT)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum += value;
                }
                std::hint::black_box(sum);
            }
            MluaMode::Callback { iterations } => {
                let mut sum = 0i64;
                for index in 0..iterations {
                    let value: i64 = self
                        .entry
                        .call(index as i64)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(value);
                }
                std::hint::black_box(sum);
            }
            MluaMode::Userdata => {
                let value: f64 = self.entry.call(()).map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(value);
            }
            MluaMode::BulkListIn { value, size } => {
                let table = self
                    ._lua
                    .create_table()
                    .map_err(|error| anyhow!("{error}"))?;
                match value {
                    BulkValue::F64 => {
                        for (index, value) in
                            host::bulk_f64_values(size).into_iter().enumerate()
                        {
                            table
                                .set(index + 1, value)
                                .map_err(|error| anyhow!("{error}"))?;
                        }
                    }
                    BulkValue::U8 => {
                        for (index, value) in
                            host::bulk_u8_values(size).into_iter().enumerate()
                        {
                            table
                                .set(index + 1, value)
                                .map_err(|error| anyhow!("{error}"))?;
                        }
                    }
                }
                let result: f64 = self
                    .entry
                    .call(table)
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(result);
            }
            MluaMode::BulkListOut => {
                let result: f64 = self
                    .entry
                    .call(())
                    .map_err(|error| anyhow!("{error}"))?;
                std::hint::black_box(result);
            }
            MluaMode::BulkBufferIn { value, size } => {
                #[cfg(feature = "mlua_luau")]
                {
                    let result = self.buffer_in(value, size)?;
                    std::hint::black_box(result);
                }
                #[cfg(not(feature = "mlua_luau"))]
                {
                    let _ = (value, size);
                    unreachable!("Lua 5.5 does not support buffers");
                }
            }
            MluaMode::BulkBufferOut => {
                #[cfg(feature = "mlua_luau")]
                {
                    let result = self.buffer_out()?;
                    std::hint::black_box(result);
                }
                #[cfg(not(feature = "mlua_luau"))]
                {
                    unreachable!("Lua 5.5 does not support buffers");
                }
            }
            MluaMode::SerdeToScript => {
                let mut sum = 0i64;
                for _ in 0..host::SERDE_ITERATIONS {
                    let table = self.serde_table()?;
                    let result: f64 = self
                        .entry
                        .call(table)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(result as i64);
                }
                std::hint::black_box(sum);
            }
            MluaMode::SerdeFromScript => {
                let mut sum = 0i64;
                for _ in 0..host::SERDE_ITERATIONS {
                    let table: Table =
                        self.entry.call(()).map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(self.serde_checksum(table)?);
                }
                std::hint::black_box(sum);
            }
            MluaMode::Error { propagates } => {
                let result = self.entry.call::<f64>(());
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
            MluaMode::Call => {
                let number: f64 = self
                    .entry
                    .call(())
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, number)
            }
            MluaMode::CallIn { iterations } => {
                let mut sum = 0.0f64;
                for _ in 0..iterations {
                    let value: f64 = self
                        .entry
                        .call(host::UPDATE_DT)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum += value;
                }
                Ok(Value::Float(sum))
            }
            MluaMode::Callback { iterations } => {
                let mut sum = 0i64;
                for index in 0..iterations {
                    let value: i64 = self
                        .entry
                        .call(index as i64)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(value);
                }
                Ok(Value::Int(sum))
            }
            MluaMode::Userdata => {
                let value: f64 = self.entry.call(()).map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, value)
            }
            MluaMode::BulkListIn { value, size } => {
                let table = self
                    ._lua
                    .create_table()
                    .map_err(|error| anyhow!("{error}"))?;
                match value {
                    BulkValue::F64 => {
                        for (index, value) in
                            host::bulk_f64_values(size).into_iter().enumerate()
                        {
                            table
                                .set(index + 1, value)
                                .map_err(|error| anyhow!("{error}"))?;
                        }
                    }
                    BulkValue::U8 => {
                        for (index, value) in
                            host::bulk_u8_values(size).into_iter().enumerate()
                        {
                            table
                                .set(index + 1, value)
                                .map_err(|error| anyhow!("{error}"))?;
                        }
                    }
                }
                let result: f64 = self
                    .entry
                    .call(table)
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, result)
            }
            MluaMode::BulkListOut => {
                let result: f64 = self
                    .entry
                    .call(())
                    .map_err(|error| anyhow!("{error}"))?;
                number_value(self.result, result)
            }
            MluaMode::BulkBufferIn { value, size } => {
                #[cfg(feature = "mlua_luau")]
                {
                    let result = self.buffer_in(value, size)?;
                    number_value(self.result, result)
                }
                #[cfg(not(feature = "mlua_luau"))]
                {
                    let _ = (value, size);
                    unreachable!("Lua 5.5 does not support buffers");
                }
            }
            MluaMode::BulkBufferOut => {
                #[cfg(feature = "mlua_luau")]
                {
                    let result = self.buffer_out()?;
                    number_value(self.result, result)
                }
                #[cfg(not(feature = "mlua_luau"))]
                {
                    unreachable!("Lua 5.5 does not support buffers");
                }
            }
            MluaMode::SerdeToScript => {
                let mut sum = 0i64;
                for _ in 0..host::SERDE_ITERATIONS {
                    let table = self.serde_table()?;
                    let result: f64 = self
                        .entry
                        .call(table)
                        .map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(result as i64);
                }
                Ok(Value::Int(sum))
            }
            MluaMode::SerdeFromScript => {
                let mut sum = 0i64;
                for _ in 0..host::SERDE_ITERATIONS {
                    let table: Table =
                        self.entry.call(()).map_err(|error| anyhow!("{error}"))?;
                    sum = sum.wrapping_add(self.serde_checksum(table)?);
                }
                Ok(Value::Int(sum))
            }
            MluaMode::Error { propagates } => {
                let result = self.entry.call::<f64>(());
                match result {
                    Ok(value) => {
                        if propagates {
                            bail!("expected the script to raise an error");
                        }
                        number_value(self.result, value)
                    }
                    Err(_) if propagates => Ok(Value::Int(1)),
                    Err(error) => Err(anyhow!("{error}")),
                }
            }
        }
    }
}

fn register_host_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    globals
        .set("interop_ping", Function::wrap_raw(|| host::ping()))
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_id_i64",
            Function::wrap_raw(|value: i64| host::id_i64(value)),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_sum_i64_3",
            Function::wrap_raw(|a: i64, b: i64, c: i64| host::sum_i64_3(a, b, c)),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_sum_i64_5",
            Function::wrap_raw(|a: i64, b: i64, c: i64, d: i64, e: i64| {
                host::sum_i64_5(a, b, c, d, e)
            }),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_id_f64",
            Function::wrap_raw(|value: f64| host::id_f64(value)),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_strlen",
            Function::wrap_raw(|value: String| host::strlen(&value)),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_string_out",
            Function::wrap_raw(|| host::string_out()),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_callback",
            Function::wrap_raw(|value: i64| host::callback_value(value)),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_sum_f64_list",
            Function::wrap_raw(|table: Table| -> f64 {
                table
                    .sequence_values::<f64>()
                    .map(|value| value.unwrap_or(0.0))
                    .sum()
            }),
        )
        .map_err(|error| anyhow!("{error}"))?;
    globals
        .set(
            "interop_sum_u8_list",
            Function::wrap_raw(|table: Table| -> f64 {
                table
                    .sequence_values::<f64>()
                    .map(|value| value.unwrap_or(0.0))
                    .sum()
            }),
        )
        .map_err(|error| anyhow!("{error}"))?;
    #[cfg(feature = "mlua_luau")]
    {
        globals
            .set(
                "interop_sum_f64_buffer",
                Function::wrap_raw(|buffer: mlua::Buffer| -> f64 {
                    let mut sum = 0.0f64;
                    let mut offset = 0;
                    while offset + 8 <= buffer.len() {
                        let bytes = buffer.read_bytes::<8>(offset);
                        sum += f64::from_le_bytes(bytes);
                        offset += 8;
                    }
                    sum
                }),
            )
            .map_err(|error| anyhow!("{error}"))?;
        globals
            .set(
                "interop_sum_u8_buffer",
                Function::wrap_raw(|buffer: mlua::Buffer| -> f64 {
                    let mut sum = 0.0f64;
                    let mut offset = 0;
                    while offset < buffer.len() {
                        let bytes = buffer.read_bytes::<1>(offset);
                        sum += f64::from(bytes[0]);
                        offset += 1;
                    }
                    sum
                }),
            )
            .map_err(|error| anyhow!("{error}"))?;
    }
    globals
        .set(
            "interop_fail",
            Function::wrap(|| -> mlua::Result<i64> {
                Err(mlua::Error::RuntimeError(host::HOST_ERROR.into()))
            }),
        )
        .map_err(|error| anyhow!("{error}"))?;

    Ok(())
}
