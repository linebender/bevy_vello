<div align="center">

# Bevy Vello

**A vector graphics rendering integration for [Bevy game engine](https://bevyengine.org) using [Vello](https://vello.dev).**

[![Discord](https://img.shields.io/discord/913957940560531456.svg?label=Loopy&logo=discord&logoColor=ffffff&color=ffffff&labelColor=000000)](https://discord.gg/zrjnQzdjCB)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#license)
[![Vello](https://img.shields.io/badge/vello-v0.1.0-purple.svg)](https://crates.io/crates/vello)
[![Following released Bevy versions](https://img.shields.io/badge/bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/quick-start/plugin-development/#main-branch-tracking)

[![Dependency status](https://deps.rs/repo/github/loopystudios/bevy_vello/status.svg)](https://deps.rs/repo/github/loopystudios/bevy_vello)
[![Crates.io](https://img.shields.io/crates/v/bevy_vello.svg)](https://crates.io/crates/bevy_vello)
[![Docs](https://img.shields.io/docsrs/bevy_vello)](https://docs.rs/bevy_vello)
[![Build status](https://github.com/loopystudios/bevy_vello/workflows/CI/badge.svg)](https://github.com/loopystudios/bevy_vello/actions)

</div>

> [!WARNING]
> The support of SVG and Lottie is limited. If there is an SVG-related issue, please file the issue in [`vello_svg`](https://github.com/linebender/vello_svg). If there is a Lottie-related issue, please file the issue in [`velato`](https://github.com/linebender/velato). Please see the respective backends for for more information about limitations.

bevy_vello is a rendering integration for rendering vector graphics in the Bevy game engine. Currently it renders standard vello `Scene`s, as well as SVG and Lottie files.

Quickstart to run the demo:

```shell
cargo run -p demo
```

![Alt text](image.png)

It uses several support backends for assets:

- [`vello_svg`](https://github.com/linebender/vello_svg) - Converting SVG files to a vello `Scene`
- [`velato`](https://github.com/linebender/velato) - Converting Lottie files to a vello `Scene`

Visual inconsistencies discovered should be reported to the respective backend.

## Bevy version support

**NOTE**: You must use a git rev for now, but we are planning a publish. See [issue #3](https://github.com/loopystudios/bevy_vello/issues/3).

|bevy|bevy_vello|
|---|---|
|0.13|0.1-0.2, main|
|< 0.13| unsupported |

## Features

- Spawn vector graphics rendering in screen-space or world-space coordinates.
- Runtime color swapping of Lottie files with a `Theme` component.
- Augment playback options with a `PlaybackOptions` component.
- Limited state machine support with a `LottiePlayer` component.
- Text
  - NOTE: To avoid conflict with bevy's built-in font loader, rename fonts used by `bevy_vello` to end with `*.vtff`. This is a limitation of the bevy game engine, and can probably be an improvement in the future.
- Debug drawing for bounding boxes and origin
- Render immediate-mode vello `Scene`s

## Examples

### Cross platform (Bevy)

```shell
cargo run -p <demo name>
```

### Web platform

Because Vello relies heavily on compute shaders, we rely on the emerging WebGPU standard to run on the web.
Until browser support becomes widespread, it will probably be necessary to use development browser versions (e.g. Chrome Canary) and explicitly enable WebGPU.

This uses [`cargo-run-wasm`](https://github.com/rukai/cargo-run-wasm) to build the example for web, and host a local server for it

```shell
# Make sure the Rust toolchain supports the wasm32 target
rustup target add wasm32-unknown-unknown

# The binary name must also be explicitly provided as it differs from the package name
cargo run_wasm -p text
```

There is also a web demo [available here](https://loopystudios.github.io/bevy_vello) on supporting web browsers.

> [!WARNING]
> The web is not currently a primary target for Vello, and WebGPU implementations are incomplete, so you might run into issues running this example.

## Community

All Loopy projects and development happens in the [Loopy Discord](https://discord.gg/zrjnQzdjCB). The discord is open to the public.

Contributions are welcome by pull request. The [Rust code of conduct](https://www.rust-lang.org/policies/code-of-conduct) applies.

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option

The files in subdirectories of the [`examples/assets`](/examples/assets) directory are licensed solely under
their respective licenses, available in the `LICENSE` file in their directories.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
