# Bevy Vello

A bevy plugin which provides rendering support for [vello](https://github.com/linebender/vello.git).

## 🔧 Development

- **Dependencies**
  - [Rust](https://www.rust-lang.org/)
  - Submodules: `git submodule update --init --recursive`

- **Build** *(Native only)*

  ```bash
  cargo build
  ```

## Experimental WebGPU support
- **Build demo**
-*NOTE*: Might need this environment variable set to build this as a dependency `RUSTFLAGS=--cfg=web_sys_unstable_apis`
- Dependencies
  - `cargo install basic-http-server`
```bash
# 1. build target
cargo build --release --target wasm32-unknown-unknown --example demo
# 2. create wasm bundle
wasm-bindgen --out-dir target --out-name wasm_example --target web target/wasm32-unknown-unknown/release/examples/demo.wasm
# 3. run
basic-http-server .
```

## 🔏 License

This project is [LICENSED](LICENSE).
