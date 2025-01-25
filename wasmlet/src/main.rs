//! # WASMlet
//!
//! A simple program that formats text using WASM plugins.
//!
//! ## Usage
//!
//! Compile the plugin to wasm
//!
//! ```sh
//! cd rainbow
//! cargo build --release
//! ```
//!
//! Compile WASMlet
//!
//! ```sh
//! cd ../wasmlet
//! cargo build --release
//! ```
//!
//! Use the plugin with WASMlet
//!
//! ```sh
//! ./target/release/wasmlet -p ../rainbow/target/wasm32-unknown-unknown/release/rainbow.wasm This is a rainbow
//! ```
//!
//! Expected output:
//!
//! ![expected-output](https://github.com/user-attachments/assets/28f5eea0-2c33-4d7d-bdfc-787c1d2513e1)
//!
#![feature(error_generic_member_access)]

use clap::Parser;
use env_logger::Builder;
use plugin::Plugin;
use std::{path::PathBuf, process::ExitCode};
mod plugin;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The text that should get printed
    #[arg(required = true)]
    text: Vec<String>,

    /// WASM plugins that should process the text
    #[arg(short, long)]
    plugins: Vec<PathBuf>,
}

fn main() -> ExitCode {
    Builder::new()
        .target(env_logger::Target::Stderr)
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    let args = Args::parse();

    let mut plugins = match args
        .plugins
        .into_iter()
        .map(Plugin::new)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(plugins) => plugins,
        Err(err) => {
            log::error!("{}", err);
            return 1.into();
        }
    };

    let input_text: String = args.text.join(" ");
    let result = match plugins
        .iter_mut()
        .try_fold(input_text, |text, plugin| plugin.apply(&text))
    {
        Ok(result) => result,
        Err(err) => {
            log::error!("{}", err);
            return 1.into();
        }
    };

    println!("{}", result);
    0.into()
}
