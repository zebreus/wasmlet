use std::path::PathBuf;

use thiserror::Error;
use wasmer::{Instance, Module, Store, imports};

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Failed to load module: {0}")]
    FailedToLoadModule(std::io::Error),
    #[error(transparent)]
    CompileError(#[from] wasmer::CompileError),
    #[error(transparent)]
    InstantiationError(#[from] wasmer::InstantiationError),
    #[error("The plugin does not provide the required function `{0}` in its exports ({1})")]
    PluginDoesNotExportRequiredFunction(String, wasmer::ExportError),
    #[error(transparent)]
    RuntimeError(#[from] wasmer::RuntimeError),
    #[error("Failed to process the string: {0}")]
    GuestError(String),
}

pub struct Plugin {
    allocate_shared_buffer: wasmer::TypedFunction<u32, u32>,
    free_shared_buffer: wasmer::TypedFunction<u32, u32>,
    process: wasmer::TypedFunction<u32, u32>,
    instance: wasmer::Instance,
    store: Store,
}

impl Plugin {
    /// Load the plugin from the given file.
    pub fn new(wasm_file: PathBuf) -> Result<Self, PluginError> {
        let wasm_bytes = std::fs::read(wasm_file).map_err(PluginError::FailedToLoadModule)?;

        let mut store = Store::default();
        let module = Module::new(&store, &wasm_bytes)?;
        // The module doesn't import anything, so we create an empty import object.
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object)?;

        let allocate_shared_buffer: wasmer::TypedFunction<u32, u32> = instance
            .exports
            .get_typed_function::<u32, u32>(&store, "allocate_shared_buffer")
            .map_err(|e| {
                PluginError::PluginDoesNotExportRequiredFunction(
                    "allocate_shared_buffer".to_string(),
                    e,
                )
            })?;
        let free_shared_buffer = instance
            .exports
            .get_typed_function::<u32, u32>(&store, "free_shared_buffer")
            .map_err(|e| {
                PluginError::PluginDoesNotExportRequiredFunction(
                    "free_shared_buffer".to_string(),
                    e,
                )
            })?;
        let process = instance
            .exports
            .get_typed_function::<u32, u32>(&store, "process")
            .map_err(|e| {
                PluginError::PluginDoesNotExportRequiredFunction("process".to_string(), e)
            })?;

        // let result = add_one.call(&mut store, &[Value::I64(42)])?;
        // assert_eq!(result[0], Value::I32(43));

        return Ok(Plugin {
            allocate_shared_buffer,
            free_shared_buffer,
            process,
            store,
            instance: instance,
        });
    }

    /// Apply this plugin to a text.
    pub fn apply(&mut self, text: &str) -> Result<String, PluginError> {
        let input_bytes = text.as_bytes();

        let shared_memory = self.instance.exports.get_memory("memory").unwrap();

        let input_address = self
            .allocate_shared_buffer
            .call(&mut self.store, input_bytes.len() as u32)
            .unwrap();
        {
            let view = shared_memory.view(&self.store);
            view.write(input_address as u64, input_bytes).unwrap();
        }
        let output_address = self.process.call(&mut self.store, input_address).unwrap();
        let (success, output) = {
            let view = shared_memory.view(&self.store);
            let success_address = output_address as u64;
            let length_address = success_address + 1;
            let string_address = length_address + 4;
            let mut success_bytes = [0u8; 1];
            view.read(success_address, &mut success_bytes).unwrap();
            let mut length_bytes = [0u8; 4];
            view.read(length_address, &mut length_bytes).unwrap();
            let length = u32::from_le_bytes(length_bytes);
            let output_vec = view
                .copy_range_to_vec(string_address..string_address + length as u64)
                .unwrap();
            let output_string = String::from_utf8(output_vec).unwrap();
            (success_bytes[0] != 0, output_string)
        };

        // Free the shared buffers.
        self.free_shared_buffer
            .call(&mut self.store, input_address)
            .unwrap();
        self.free_shared_buffer
            .call(&mut self.store, output_address)
            .unwrap();

        if !success {
            return Err(PluginError::GuestError(output));
        }
        Ok(output)
    }
}
