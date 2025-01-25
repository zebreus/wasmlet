use std::path::PathBuf;

use thiserror::Error;
use wasmer::{
    CompileError, ExportError, Instance, InstantiationError, Memory, MemoryAccessError, Module,
    RuntimeError, Store, TypedFunction, WasmPtr, imports,
};

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Failed to load plugin: {0}")]
    FailedToLoadModule(std::io::Error),
    #[error("Failed to compile plugin: {0}")]
    CompileError(#[from] CompileError),
    // Clippy recommended that we box the error and I agree with the reasoning
    // https://rust-lang.github.io/rust-clippy/master/index.html#result_large_err
    #[error("Failed to instantiate plugin: {0}")]
    InstantiationError(#[from] Box<InstantiationError>),
    #[error("The plugin does not provide the required function `{0}` in its exports ({1})")]
    PluginDoesNotExportRequiredFunction(String, ExportError),
    #[error("The plugin does not export memory: `memory`")]
    PluginDoesNotExportMemory(#[source] ExportError),
    #[error("The plugin crashed while allocating a buffer: {0}")]
    RuntimeErrorWhileFreeingBuffer(#[source] RuntimeError),
    #[error("The plugin crashed while freeing a buffer: {0}")]
    RuntimeErrorWhileAllocatingBuffer(#[source] RuntimeError),
    #[error("The plugin crashed while processing your input: {0}")]
    RuntimeErrorWhileProcessingText(#[source] RuntimeError),
    #[error("The plugin failed to process the input: {0}")]
    GuestError(String),
    #[error("Failed to free a shared buffer")]
    FailedToFreeSharedBuffer,
    /// Umbrella error when `process` returns a pointer to something that is not a valid result.
    #[error(
        "Process returned a malformed datastructure, please check that the plugin returns a correctly formatted buffer. (1 byte success flag, 4 byte length, length bytes utf8-formatted string)"
    )]
    ProcessReturnedMalformedDatastructure(#[source] MemoryAccessError),
    #[error("The plugin failed to allocate a valid buffer for the input.")]
    AllocatedBufferCausedMemoryError(#[source] MemoryAccessError),
}

pub struct Plugin {
    allocate_shared_buffer: TypedFunction<u32, WasmPtr<u8>>,
    free_shared_buffer: TypedFunction<WasmPtr<u8>, u32>,
    process: TypedFunction<WasmPtr<u8>, WasmPtr<u8>>,
    store: Store,
    memory: Memory,
}

impl Plugin {
    /// Load the plugin from the given file.
    pub fn new(wasm_file: PathBuf) -> Result<Self, PluginError> {
        let wasm_bytes = std::fs::read(wasm_file).map_err(PluginError::FailedToLoadModule)?;

        let mut store = Store::default();
        let module = Module::new(&store, &wasm_bytes)?;
        let instance = Instance::new(&mut store, &module, &imports! {}).map_err(Box::new)?;

        let allocate_shared_buffer = instance
            .exports
            .get_typed_function::<u32, WasmPtr<u8>>(&store, "allocate_shared_buffer")
            .map_err(|e| {
                PluginError::PluginDoesNotExportRequiredFunction(
                    "allocate_shared_buffer".to_string(),
                    e,
                )
            })?;
        let free_shared_buffer = instance
            .exports
            .get_typed_function::<WasmPtr<u8>, u32>(&store, "free_shared_buffer")
            .map_err(|e| {
                PluginError::PluginDoesNotExportRequiredFunction(
                    "free_shared_buffer".to_string(),
                    e,
                )
            })?;
        let process = instance
            .exports
            .get_typed_function::<WasmPtr<u8>, WasmPtr<u8>>(&store, "process")
            .map_err(|e| {
                PluginError::PluginDoesNotExportRequiredFunction("process".to_string(), e)
            })?;

        let memory = instance
            .exports
            .get_memory("memory")
            .map_err(PluginError::PluginDoesNotExportMemory)?
            .clone();

        Ok(Plugin {
            allocate_shared_buffer,
            free_shared_buffer,
            process,
            store,
            memory,
        })
    }

    /// Create a shared buffer in guest memory.
    ///
    /// You need to free it afterwards using `free_shared_buffer`.
    fn create_shared_buffer(&mut self, data: &[u8]) -> Result<WasmPtr<u8>, PluginError> {
        let address = self
            .allocate_shared_buffer
            .call(&mut self.store, data.len() as u32)
            .map_err(PluginError::RuntimeErrorWhileAllocatingBuffer)?;
        let view = self.memory.view(&self.store);
        address
            .slice(&view, data.len() as u32)
            .and_then(|slice| slice.write_slice(data))
            .map_err(PluginError::AllocatedBufferCausedMemoryError)?;
        Ok(address)
    }

    /// Free a shared buffer in guest memory.
    fn free_shared_buffer(&mut self, address: WasmPtr<u8>) -> Result<(), PluginError> {
        let result = self
            .free_shared_buffer
            .call(&mut self.store, address)
            .map_err(PluginError::RuntimeErrorWhileFreeingBuffer)?;

        if result == 0 {
            return Err(PluginError::FailedToFreeSharedBuffer);
        }

        Ok(())
    }

    fn process(&mut self, input: WasmPtr<u8>) -> Result<String, PluginError> {
        let output_ptr = self
            .process
            .call(&mut self.store, input)
            .map_err(PluginError::RuntimeErrorWhileProcessingText)?;

        let view = self.memory.view(&self.store);
        let success_ptr = output_ptr;
        let length_ptr: WasmPtr<u32, _> = output_ptr
            .add_offset(1)
            .map_err(PluginError::ProcessReturnedMalformedDatastructure)?
            .cast();
        let string_ptr = output_ptr
            .add_offset(1 + 4)
            .map_err(PluginError::ProcessReturnedMalformedDatastructure)?;

        let success = success_ptr
            .deref(&view)
            .read()
            .map_err(PluginError::ProcessReturnedMalformedDatastructure)?
            != 0;

        let length = length_ptr
            .read(&view)
            .map_err(PluginError::ProcessReturnedMalformedDatastructure)?;

        let string_slice = string_ptr
            .read_utf8_string(&view, length)
            .map_err(PluginError::ProcessReturnedMalformedDatastructure)?;

        self.free_shared_buffer(output_ptr)?;

        if !success {
            return Err(PluginError::GuestError(string_slice));
        }
        Ok(string_slice)
    }

    /// Apply this plugin to a text.
    pub fn apply(&mut self, input: &str) -> Result<String, PluginError> {
        let input_ptr = self.create_shared_buffer(input.as_bytes())?;

        let result = self.process(input_ptr)?;

        self.free_shared_buffer(input_ptr)?;
        Ok(result)
    }
}
