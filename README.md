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

![expected-output](https://github.com/user-attachments/assets/28f5eea0-2c33-4d7d-bdfc-787c1d2513e1)

<!-- cargo-rdme end -->
