# Bevy Vello

![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![crates.io](https://img.shields.io/crates/v/bevy-async-task.svg)](https://crates.io/crates/bevy-async-task)
[![docs.rs](https://img.shields.io/docsrs/bevy-async-task)](https://docs.rs/bevy-async-task)
[![Discord](https://img.shields.io/discord/913957940560531456.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/zrjnQzdjCB)

A bevy plugin which provides rendering support for Lottie animations and SVGs via [vello](https://github.com/linebender/vello.git).

![Alt text](image.png)

Supports both **wasm/webgpu** and **native**.

Why vello?
- Fonts render with infinite resolution (no pixelation), even
when dynamically scaling
- Render SVGs and [Lottie](https://lottiefiles.com/what-is-lottie) animations with infinite resolution, no tessellation necessary
- Performant graphics--rendering in compute shaders

## Features
- Spawnable vector graphics on separate layers
  |Layer|Render order|
  |---|---|
  |Background|Always behind all other layers|
  |Ground|2.5D-style render ordering via Y-coordinate|
  |Foreground|Always on top of Ground/Background|
  |UI|On top of Foreground layer; shows Bevy UI Nodes bundled with a `VelloVector` |
- Support for fonts
  - NOTE: to avoid conflict with bevy's built-in font loader, rename fonts used by bevy_vello to something else (example: `*.vtff`). This can probably be an improvement in the future
- Option to debug draw gizmos for the objects local origin (red X) and canvas size (white box)

## Getting Started
We recommend checking out the [demo](https://github.com/vectorgameexperts/bevy_vello/blob/main/examples/demo.rs) in `examples/`

## Bevy version support

|bevy|bevy_vello|
|---|---|
|0.10|0.1|
|0.11|Coming Soon|

## Attributions

### Libraries Used

`bevy_vello` is only possible because of these awesome libraries
* [vello](https://github.com/linebender/vello): Rendering backend for drawing vector curves, paths, shapes, effects, etc.
* [velato](https://github.com/linebender/velato): integration and parsing library for Lottie files (vector graphic animations).

### Assets Used

The animated vector graphic in `examples/demo` is sourced from Google Fonts [Noto Animated Emojis](https://googlefonts.github.io/noto-emoji-animation/documentation)

## License

This project is dual-licensed under both [Apache 2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT) licenses.
