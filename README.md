<!-- cargo-rdme start -->

# WASMlet

A simple program that formats text using WASM plugins.

## Usage

Compile the plugins to WASM

```sh
cd rainbow
cargo build --release
cd ../bigfont
cargo build --release
```

Compile WASMlet

```sh
cd ../wasmlet
cargo build --release
```

Use the plugin with WASMlet

```sh
./target/release/wasmlet -p bigfont -p rainbow WASMlet
```

Expected output:

![expected-output](https://github.com/user-attachments/assets/28f5eea0-2c33-4d7d-bdfc-787c1d2513e1)

## Plugin Resolution

When you specify plugins with the `-p` flag, WASMlet uses the following strategy to find plugins:

1. Try to interpret the specifier as a path to a file.
2. Try the specifier with an appended `.wasm` extension.
3. Try to load the specifier relative to the directory specified in `WASMLET_PLUGIN_DIR` (defaults to `/etc/wasmlet/plugins`).
4. Try to load the specifier from a rust crate next to this project.

<!-- cargo-rdme end -->
