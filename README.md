# Bevy Vello

![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![crates.io](https://img.shields.io/crates/v/bevy_vello.svg)](https://crates.io/crates/bevy_vello)
[![docs.rs](https://img.shields.io/docsrs/bevy_vello)](https://docs.rs/bevy_vello)
[![Discord](https://img.shields.io/discord/913957940560531456.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/zrjnQzdjCB)

A bevy plugin which provides rendering support for [lottie](https://lottiefiles.com/what-is-lottie) animations and SVGs on [Bevy](https://bevyengine.org/) using [Vello](https://github.com/linebender/vello.git) and [Velato](https://github.com/linebender/velato). Supports **wasm** and **native**. Untested on Android/iOS.

![Alt text](image.png)

## Features

- Spawn vector graphics on separate layers
  |Layer|Render order|
  |---|---|
  |Background|Always behind all other layers|
  |Ground|2.5D-style render ordering via Y-coordinate|
  |Foreground|Always on top of Ground/Background|
  |UI|On top of Foreground layer; shows Bevy UI Nodes bundled with a `VelloVector` |
- Support for fonts
  - NOTE: to avoid conflict with bevy's built-in font loader, rename fonts used by bevy_vello to something else (example: `*.vtff`). This can probably be an improvement in the future.
- Debug draw gizmos for the objects local origin (red X) and canvas size (white bounding box)

## Run Demo

- Native

  ```bash
  cargo run
  ```

- WASM (requires `cargo install trunk`)

  ```bash
  trunk serve
  ```

## Bevy version support

|bevy|bevy_vello|
|---|---|
|0.10|0.1|
|0.11|0.2|

## Attributions

The animated vector graphic in the README and demo is a lottie file available from Google Fonts' [Noto Animated Emojis](https://googlefonts.github.io/noto-emoji-animation/documentation).

## License

This project is dual-licensed under both [Apache 2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT) licenses.
