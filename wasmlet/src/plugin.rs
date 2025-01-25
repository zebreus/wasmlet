use std::{fs::File, path::PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Failed to load module: {0}")]
    FailedToLoadModule(std::io::Error),
}

pub struct Plugin {}

impl Plugin {
    /// Load the plugin from the given file.
    pub fn new(wasm_file: PathBuf) -> Result<Self, PluginError> {
    fn new(&self, wasm_file: PathBuf) -> Result<Self, PluginError> {
        let file = File::open(wasm_file).map_err(PluginError::FailedToLoadModule)?;
        return Ok(Plugin {});
    }

    /// Apply this plugin to a text.
    pub fn apply(&self, text: &str) -> Result<String, PluginError> {
        return Ok(text.to_string());
    }
}
