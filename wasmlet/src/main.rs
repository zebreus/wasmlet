//! # WASMlet
//!
//! A simple program that formats text using WASM plugins.

use clap::Parser;
use std::path::PathBuf;
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

fn main() {
    let args = Args::parse();

    println!("{}", args.text);
}
