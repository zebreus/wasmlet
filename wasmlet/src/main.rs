//! # ![WASMlet](https://github.com/user-attachments/assets/acd1daf0-5d2a-46e2-a551-2f0d82c76624)
//!
//! A simple program that formats text using WASM plugins.
//!
//! ## Usage
//!
//! Compile the plugins to WASM
//!
//! ```sh
//! cd rainbow
//! cargo build --release
//! cd ../bigfont
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
//! ./target/release/wasmlet -p bigfont -p rainbow WASMlet
//! ```
//!
//! Expected output:
//!
//! ![Screenshot of a terminal showing the text `WASMlet` in big colored letters](https://github.com/user-attachments/assets/b469de43-f2fc-4225-96b0-4252afbde4a8)
//!
//! ## Plugin Resolution
//!
//! When you specify plugins with the `-p` flag, WASMlet uses the following strategy to find plugins:
//!
//! 1. Try to interpret the specifier as a path to a file.
//! 2. Try the specifier with an appended `.wasm` extension.
//! 3. Try to load the specifier relative to the directory specified in `WASMLET_PLUGIN_DIR` (defaults to `/etc/wasmlet/plugins`).
//! 4. Try to load the specifier from a rust crate next to this project.
#![feature(error_generic_member_access)]

use clap::Parser;
use env_logger::Builder;
use plugin::Plugin;
use std::process::ExitCode;
mod plugin;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[clap(after_help = "
\x1b[1;4mPLUGIN RESOLUTION:\x1b[0m
  WASMlet uses the following strategy to find plugins:
  1. Try to interpret the specifier as a path to a file.
  2. Try the specifier with an appended `.wasm` extension.
  3. Try to load the specifier relative to the directory specified in `WASMLET_PLUGIN_DIR` (defaults to `/etc/wasmlet/plugins`).
  4. Try to load the specifier from a rust crate next to this project.
")]
struct Args {
    /// The text that should get printed
    #[arg(required = true)]
    text: Vec<String>,

    /// WASM plugins that should process the text
    #[arg(short, long)]
    plugins: Vec<String>,
}

fn main() -> ExitCode {
    Builder::from_default_env()
        .target(env_logger::Target::Stderr)
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    let args = Args::parse();

    let mut plugins = match args
        .plugins
        .iter()
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
