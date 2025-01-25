<!-- cargo-rdme start -->

# WASMlet

A simple program that formats text using WASM plugins.

## Usage

Compile the plugin to wasm

```sh
cd rainbow
cargo build --release
```

Compile WASMlet

```sh
cd ../wasmlet
cargo build --release
```

Use the plugin with WASMlet

```sh
./target/release/wasmlet -p ../rainbow/target/wasm32-unknown-unknown/release/rainbow.wasm This is a rainbow
```

Expected output:

<!-- cargo-rdme end -->
