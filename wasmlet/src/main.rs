//! # WASMlet
//!
//! A simple program that formats text using WASM plugins.
#![feature(error_generic_member_access)]

use clap::Parser;
use plugin::Plugin;
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

    println!("Got {} plugins", args.plugins.len());
    for plugin in args.plugins {
        let mut plugin = match Plugin::new(plugin) {
            Ok(plugin) => plugin,
            Err(err) => {
                eprintln!("Failed to load plugin: {}", err);
                continue;
            }
        };

        let text = plugin.apply(&args.text).unwrap();
        println!("{}", text);
    }

    println!("{}", args.text);
}
