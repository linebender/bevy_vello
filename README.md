# Bevy Vello

![MIT OR Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![crates.io](https://img.shields.io/crates/v/bevy-vello.svg)](https://crates.io/crates/bevy-vello)
[![docs.rs](https://img.shields.io/docsrs/bevy-vello)](https://docs.rs/bevy-vello)
[![Discord](https://img.shields.io/discord/913957940560531456.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/zrjnQzdjCB)

A bevy plugin which provides rendering support for [lottie](https://lottiefiles.com/what-is-lottie) animations and SVGs on [Bevy](https://bevyengine.org/) using [Vello](https://github.com/linebender/vello.git) and [Velato](https://github.com/linebender/velato). Supports **wasm** and **native**. Untested on Android/iOS (help needed).

![Alt text](image.png)

## Bevy version support

**NOTE**: You must use a git rev for now, and match our version of vello. We cannot publish to crates.io. See [issue #3](https://github.com/vectorgameexperts/bevy-vello/issues/3).

|bevy|bevy-vello|
|---|---|
|0.12|0.3-0.5, main|
|0.11|0.2|
|<= 0.10|0.1|

## Features

- Spawn vector graphics rendering either in screen-space or world-space coordinates.
- Runtime color swapping of Lottie files `Theme` component.
- Augment playback options with the `PlaybackOptions` component.
- Limited state machine support with the `LottiePlayer` component.
- Rudimentary support for text
  - NOTE: To avoid conflict with bevy's built-in font loader, rename fonts used by `bevy-vello` to end with `*.vtff`. This is a limitation of the bevy game engine, and can probably be an improvement in the future.
- Debug draw gizmos for the objects local origin (red X) and canvas size (white bounding box)

## Run Demo

- Native

  ```shell
  cargo run -p demo
  ```

- WASM (requires `cargo install trunk`)

  ```shell
  cd demo
  trunk serve
  ```

## Attributions

The animated vector graphic in the README and demo is a lottie file available from Google Fonts' [Noto Animated Emojis](https://googlefonts.github.io/noto-emoji-animation/documentation).

## License

This project is dual-licensed under both [Apache 2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT) licenses.
