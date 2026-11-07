use std::rc::Rc;

use anyhow::{anyhow, Result};
use rand::RngExt;
#[cfg(feature = "wasmi")]
use wasmi::*;
#[cfg(feature = "wasmtime")]
use wasmtime::*;

use crate::harness::{read_script_bytes, Case, Value};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustData(Rc<str>);

#[derive(Default)]
struct HostState {
    store: Vec<Entry>,
    free: Option<u32>,
    memory: Option<Memory>,
}

enum Entry {
    Free { next: Option<u32> },
    Occupied { rd: RustData },
}

impl HostState {
    fn alloc(&mut self, rd: RustData) -> u32 {
        if let Some(id) = self.free {
            let entry = &mut self.store[id as usize];
            let Entry::Free { next } = entry else {
                panic!("corruption");
            };
            self.free = *next;
            *entry = Entry::Occupied { rd };
            return id;
        }
        self.store.push(Entry::Occupied { rd });
        self.store.len() as u32 - 1
    }

    fn get(&self, id: u32) -> &RustData {
        match &self.store[id as usize] {
            Entry::Free { .. } => panic!("use after free"),
            Entry::Occupied { rd } => rd,
        }
    }

    fn dealloc(&mut self, id: u32) {
        let entry = &mut self.store[id as usize];
        let Entry::Occupied { .. } = entry else {
            panic!("double free");
        };
        *entry = Entry::Free { next: self.free };
        self.free = Some(id);
    }
}

pub struct WasmEngine {
    store: Store<HostState>,
    bench: TypedFunc<u32, u32>,
    get_id: TypedFunc<u32, u32>,
}

impl crate::harness::Engine for WasmEngine {
    fn setup(case: &Case) -> Result<Self> {
        let engine = Engine::default();
        let wasm = read_script_bytes(case.scripts.wasm_module)?;
        let module = Module::new(&engine, wasm.as_slice()).map_err(|error| anyhow!("{error}"))?;

        let mut store = Store::new(&engine, Default::default());
        let rustdata_new = Func::wrap(
            &mut store,
            |mut caller: Caller<'_, HostState>, off: u32, len: u32| -> u32 {
                let buffer = &caller.data().memory.unwrap().data(&mut caller)
                    [off as usize..][..len as usize];
                let rd = RustData(std::str::from_utf8(buffer).unwrap().into());
                caller.data_mut().alloc(rd)
            },
        );
        let rustdata_delete = Func::wrap(
            &mut store,
            |mut caller: Caller<'_, HostState>, id: u32| -> () {
                caller.data_mut().dealloc(id);
            },
        );
        let rustdata_lt = Func::wrap(
            &mut store,
            |caller: Caller<'_, HostState>, i: u32, j: u32| -> u32 {
                let data = caller.data();
                (data.get(i) < data.get(j)) as u32
            },
        );

        let mut linker = <Linker<HostState>>::new(&engine);
        linker.func_wrap("RustData", "rand", |n: u32| {
            rand::rng().random_range(0..n)
        })?;
        #[cfg(feature = "wasmtime")]
        {
            linker
                .define(&store, "RustData", "rustdata_new", rustdata_new)?
                .define(&store, "RustData", "rustdata_delete", rustdata_delete)?
                .define(&store, "RustData", "rustdata_lt", rustdata_lt)?;
        }
        #[cfg(feature = "wasmi")]
        {
            linker
                .define("RustData", "rustdata_new", rustdata_new)?
                .define("RustData", "rustdata_delete", rustdata_delete)?
                .define("RustData", "rustdata_lt", rustdata_lt)?;
        }

        #[cfg(feature = "wasmtime")]
        let instance = linker.instantiate(&mut store, &module)?;
        #[cfg(feature = "wasmi")]
        let instance = linker.instantiate_and_start(&mut store, &module)?;
        store.data_mut().memory = instance.get_memory(&mut store, "memory");
        let bench = instance.get_typed_func::<u32, u32>(&mut store, "bench")?;
        let get_id = instance.get_typed_func::<u32, u32>(&mut store, "get_id")?;

        Ok(Self {
            store,
            bench,
            get_id,
        })
    }

    fn run(&mut self) -> Result<()> {
        self.bench.call(&mut self.store, 0)?;
        Ok(())
    }

    fn run_checked(&mut self) -> Result<Value> {
        let length = self.bench.call(&mut self.store, 1)?;
        let mut items = Vec::with_capacity(length as usize);
        for index in 0..length {
            let id = self.get_id.call(&mut self.store, index)?;
            items.push(Value::Object(
                self.store.data().get(id).0.to_string(),
            ));
        }
        Ok(Value::List(items))
    }
}
