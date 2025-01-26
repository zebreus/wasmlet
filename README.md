<!-- cargo-rdme start -->

# ![WASMlet](https://github.com/user-attachments/assets/acd1daf0-5d2a-46e2-a551-2f0d82c76624)

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

![Screenshot of a terminal showing the text `WASMlet` in big colored letters](https://github.com/user-attachments/assets/b469de43-f2fc-4225-96b0-4252afbde4a8)

## Download plugins from the internet

Plugins are run in an isolated containers using wasmer. This makes it is safe to download and run plugins from the internet. WASMlet supports this by allowing you to load plugins from `https` urls.

Example:

```sh
wasmlet -p https://0x0.st/8XIj.wasm Hello World!
```

## Plugin Resolution

When you specify plugins with the `-p` flag, WASMlet uses the following strategy to find plugins:

1. If the specifier starts with `https://`: Stop here and attempt to download the file
2. Try to interpret the specifier as a path to a file.
3. Try the specifier with an appended `.wasm` extension.
4. Try to load the specifier relative to the directory specified in `WASMLET_PLUGIN_DIR` (defaults to `/etc/wasmlet/plugins`).
5. Try to load the specifier from a rust crate next to this project.

<!-- cargo-rdme end -->
