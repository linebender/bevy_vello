<div align="center">

# Bevy Vello

**A vector graphics rendering integration for [Bevy game engine](https://bevyengine.org) using [Vello](https://vello.dev).**

[![Linebender Zulip](https://img.shields.io/badge/Linebender-%23gpu-blue?logo=Zulip)](https://xi.zulipchat.com/#narrow/stream/197075-gpu)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#license)
[![Vello](https://img.shields.io/badge/vello-v0.1.0-purple.svg)](https://crates.io/crates/vello)
[![Following released Bevy versions](https://img.shields.io/badge/bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/quick-start/plugin-development/#main-branch-tracking)\
[![Dependency status](https://deps.rs/repo/github/linebender/bevy_vello/status.svg)](https://deps.rs/repo/github/linebender/bevy_vello)
[![Crates.io](https://img.shields.io/crates/v/bevy_vello.svg)](https://crates.io/crates/bevy_vello)
[![Docs](https://img.shields.io/docsrs/bevy_vello)](https://docs.rs/bevy_vello)
[![Build status](https://github.com/linebender/bevy_vello/workflows/CI/badge.svg)](https://github.com/linebender/bevy_vello/actions)

</div>

`bevy_vello` is a cross-platform, 2D compute-centric vector graphics rendering library for Bevy. There is default support for rendering text and scenes, with additional opt-in features for SVG and Lottie. Experimental support for dotLottie is coming.

Quickstart to run the demo:

```shell
cargo run -p demo
```

![Alt text](image.png)

## Bevy version support

|bevy|bevy_vello|
|---|---|
|0.13|0.1-0.4, main|
|< 0.13| unsupported |

## Cargo features

> [!WARNING]
> The support of SVG and Lottie is limited. If there is an SVG-related issue, please file the issue in [`vello_svg`](https://github.com/linebender/vello_svg). If there is a Lottie-related issue, please file the issue in [`velato`](https://github.com/linebender/velato). Please see the respective backends for for more information about limitations.

|Cargo feature|Description|Default?|
|---|---|----|
|`svg`|Render `.svg` files with [`vello_svg`](https://github.com/linebender/vello_svg)|Yes|
|`lottie`|Render `.json` Lottie files with [`velato`](https://github.com/linebender/velato)|Yes|
|`experimental-dotLottie`|Render `.lottie` Lottie files. **Work in Progress**|No|

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

There is also a web demo [available here](https://linebender.github.io/bevy_vello) on supporting web browsers.

> [!WARNING]
> The web is not currently a primary target for Vello, and WebGPU implementations are incomplete, so you might run into issues running this example.

## Community

Discussion of Vello development happens in the [Linebender Zulip](https://xi.zulipchat.com/), specifically the [#gpu stream](https://xi.zulipchat.com/#narrow/stream/197075-gpu). All public content can be read without logging in.

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
