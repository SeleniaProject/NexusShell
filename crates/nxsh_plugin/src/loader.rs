use anyhow::Result;
use wasmtime::{Engine, Store, Module, Linker, ResourceLimiter};
use wasmtime_wasi::WasiCtxBuilder;
use crate::registrar::PluginRegistrar;

struct MemLimiter { max: usize }
impl ResourceLimiter for MemLimiter {
    fn memory_growing(&mut self, _current: usize, desired: usize, _maximum: Option<usize>) -> bool {
        desired < self.max
    }
    fn table_growing(&mut self, _current: u32, _desired: u32, _maximum: Option<u32>) -> bool { true }
}

pub fn load_wasm_plugin(path: &str, registrar: &mut PluginRegistrar) -> Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, path)?;
    let mut linker = Linker::new(&engine);
    let mut store = Store::new(&engine, MemLimiter { max: 16 * 1024 * 1024 }); //16MiB
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    let wasi = WasiCtxBuilder::new().inherit_stdio().build();
    store.set_wasi(wasi);
    let instance = linker.instantiate(&mut store, &module)?;
    let func = instance.get_typed_func::<i32, ()>(&mut store, "nx_plugin_register").ok();
    if let Some(f) = func {
        // pass dummy registrar pointer (not yet FFI)
        f.call(&mut store, 0)?;
    }
    Ok(())
} 