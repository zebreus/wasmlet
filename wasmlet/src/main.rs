//! # WASMlet
//!
//! A simple program that formats text using WASM plugins.
#![feature(error_generic_member_access)]

use clap::Parser;
use plugin::Plugin;
use std::{path::PathBuf, process::ExitCode};
mod plugin;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The text that should get printed
    text: String,

    /// WASM plugins that should process the text
    #[arg(short, long)]
    plugins: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    eprintln!("Got {} plugins", args.plugins.len());
    let mut plugins = match args
        .plugins
        .into_iter()
        .map(Plugin::new)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(plugins) => plugins,
        Err(err) => {
            eprintln!("{}", err);
            return 1.into();
        }
    };

    let input_text: String = args.text.to_string();
    let result = match plugins
        .iter_mut()
        .try_fold(input_text, |text, plugin| plugin.apply(&text))
    {
        Ok(result) => result,
        Err(err) => {
            eprintln!("{}", err);
            return 1.into();
        }
    };

    println!("{}", result);
    return 0.into();
}
