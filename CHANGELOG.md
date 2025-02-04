# Changelog

<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

The latest published Bevy Vello release is [0.6.1](#061---2024-08-14) which was released on 2024-08-14.
You can find its changes [documented below](#061---2024-08-14).

## [Unreleased]

This release supports Bevy version 0.14 and has an [MSRV][] of 1.80.

### Changed

- bevy_vello now uses Bevy 0.15
- `VelloAsset` assets have been separated into `VelloSvg` and `VelloLottie`
- `VelloAssetBundle` has been separated into `VelloSvgBundle` and `VelloLottieBundle`
- `Handle<VelloAsset>` has been separated into `VelloSvgHandle` and `VelloLottieHandle`

## [0.6.1] - 2024-08-14

This release supports Bevy version 0.14 and has an [MSRV][] of 1.80.

### Fixed

- Text is now properly skipped when a `SkipEncoding` component is present. ([#77] by [@simbleau])

## [0.6.0] - 2024-08-09

This release supports Bevy version 0.14 and has an [MSRV][] of 1.80.

### Added

- There is now a `default_font` feature that uses the same `FiraMono-subset.ttf` font used in the bevy/default_font feature.
- There is now a `render_layers` example.
- There is now a `cube_3d` example.
- You may now choose the render layers for the Vello canvas. This can be configured through the `VelloPlugin`.
- You may now choose to use CPU rendering with Vello and configure anti-aliasing. This can be configured through the `VelloPlugin`.
- Added the `SkipEncoding` component, which allows you to skip encoding any renderable asset without removing the asset.

### Changed

- `VelloPlugin` now has configuration. To retain previous behavior, use `VelloPlugin::default()`.
- `VelloRenderer` is now a resource.
- The `VelloRenderer` will attempt CPU fallback if it cannot obtain a GPU.
- The font API has changed significantly. Please visit `examples/text` for further usage. This is to prepare for additional text features such as linebreak behavior, bounded text, and text justification.
  - `VelloText` has been renamed to `VelloTextSection`.
  - `VelloText.content` has been renamed to `VelloText.value`.
  - There is now a `VelloTextStyle` struct and it is a required field of `VelloText`.
  - `VelloFont` has been removed from `VelloTextBundle` and moved into `VelloTextStyle`.
- The field `VelloAssetBundle.vector` was renamed to `VelloAssetBundle.asset`.
- Renamed `VelloAssetAlignment` to `VelloAssetAnchor`. Fields were renamed `alignment` were renamed to `asset_anchor`.
- Renamed `VelloTextAlignment` to `VelloTextAnchor`. Fields were renamed `alignment` were renamed to `text_anchor`.
- The `SSRenderTarget` (fullscreen quad that renders your frame) no longer renders at a zepth of `-0.001`. This was a legacy hack used to ensure Gizmos rendered on-top. `RenderLayers` should be used now (there's an example).

### Removed

- Removed `ZFunction`s from the render pipeline. Now ordering is based solely on the `Transform`'s z component. If you depended on this behavior, you'll need to adjust the transform Z in a system prior to render.
- `VelloRenderPlugin` is now private, as it is not helpful for downstream developers to add manually.
- Removed `VelloCanvasMaterial` from prelude, as it is not typical to use.

### Fixed

- Text, assets, and scenes rendered will now correctly respect camera `RenderLayers`.

## [0.5.1] - 2024-07-04

This release supports Bevy version 0.14 and has an [MSRV][] of 1.79.

### Fixed

- Updated to patch vello 0.2.1. It is now no-longer possible to panic when the vello encodings are empty.
- The demo CI now deploys that bevy_pancam has been updated to bevy 0.14

## [0.5.0] - 2024-07-04

This release supports Bevy version 0.14 and has an [MSRV][] of 1.79.

### Added

- New `scene_ui` example demonstrating a `VelloScene` attached to a `bevy::ui::Node`.

### Changed

- Updated to bevy 0.14
- Updated to vello 0.2
- Updated to velato 0.3
- Updated to vello_svg 0.3

### Fixed

- Removed `Arc` in another `Arc` for `VelloFont`
- Opacity now correctly applies to SVG assets.
- Opacity now applies correctly to the lottie image group, rather than each element and path within it, causing overdraw.
- `VelloScene` components on `bevy::ui::Node` entities now account for Bevy's UI layout systems and render at the expected viewport coordinates

### Removed

- Pancam and/or egui from all examples besides the demo, as external dependencies can bottleneck upgrading to the next bevy version.

## [0.4.2] - 2024-05-26

This release supports Bevy version 0.13 and has an [MSRV][] of 1.78.

### Fixed

- Updated to vello_svg v0.2.0, fixing viewboxes.
- Updates to velato v0.2.0, fixing viewboxes.

## [0.4.1] (Yanked) - 2024-05-26

## [0.4.0] - 2024-05-21

This release supports Bevy version 0.13 and has an [MSRV][] of 1.78.

### Added

- New `svg` example
- New `lottie` example

### Changed

- The GitHub repo has migrated into the linebender org: <https://github.com/linebender>
  - You may need to update your git ref from `loopystudios` to `linebender`
- SVG and Lottie features are now feature-gated
  - SVG (.svg) support is now added through a cargo feature `svg`.
  - Lottie (.json) support is now added through the cargo feature `lottie`.
  - experimental `dotLottie` features (`LottiePlayer`, `PlayerTransition`, `PlayerState`) are now feature-gated through the cargo feature `experimental-dotLottie`. This is only partial support, and a work in progress.
  - `Theme` is now activated through the `lottie` feature, as it was only possible to style runtime lotties.
  - `VelloAsset.metadata()` is no longer available, as it is specific to Lottie. There is now a trait, `LottieExt` that can be imported to call `.metadata()` on a `Composition` instead. This is no longer fallible as a result.
- `PlaybackAlphaOverride` was removed in favor of an `alpha` field on `VelloAsset`.
- `LottiePlayer` was renamed to `DotLottiePlayer`.
- Paths to several locations have changed, e.g. `bevy_vello::assets` -> `bevy_vello::integrations`

### Fixed

- A slow startup delay for lottie assets to begin rendering
- A dotLottie issue where the first frame can jump on web platforms.

## [0.3.3] - 2024-05-13

This release supports Bevy version 0.13 and has an [MSRV][] of 1.78.

### Fixed

- Projects with a 2D and 3D camera should no longer conflict with `bevy_vello`'s queries.

## [0.3.2] - 2024-05-04

This release supports Bevy version 0.13 and has an [MSRV][] of 1.78.

### Added

- Inverse `ZFunction` options added for `BbTop`, `BbBottom`, `BbLeft`, and `BbRight`.

### Fixed

- A panic that can happen in the extract schedule of lottie files.
- Z-ordering now works correctly for `Bb` functions.

## [0.3.1] - 2024-05-01

This release supports Bevy version 0.13 and has an [MSRV][] of 1.77.

### Fixed

- `bevy_vello::prelude::Scene` was removed, since it conflicts with `bevy::prelude::Scene`.

## [0.3.0] - 2024-05-01

This release supports Bevy version 0.13 and has an [MSRV][] of 1.77.

### Added

- `VelloAssetAlignment` was added to the `VelloAssetBundle`.

### Changed

- `VectorFile` enum variants were flattened into tuple structs.

### Removed

- `bevy_vello::VelloPlugin` was removed from the prelude.

## [0.2.2] - 2024-04-22

This release supports Bevy version 0.13 and has an [MSRV][] of 1.77.

### Fixed

- Now when a `VelloScene` and `VelloText` have the same Z-Index, text will be rendered above the scene.

## [0.2.1] - 2024-04-21

This release supports Bevy version 0.13 and has an [MSRV][] of 1.77.

### Fixed

- `VelloTextAlignment` is now in the `bevy_vello::prelude`.
- The playhead now will now always be bounded
- A rare issue where, if an asset was not available, parts of a state would not transition properly.

## [0.2.0] - 2024-04-17

This release supports Bevy version 0.13 and has an [MSRV][] of 1.77.

### Added

- Added the `VelloTextAlignment` component to `VelloTextBundle`, which now helps control the alignment of text.
- Added the `VelloTextAlignment` to the `bevy_vello::prelude`.

### Fixed

- Text bounding boxes are now tighter as they are capped by the baseline.

## [0.1.2] - 2024-04-08

This release supports Bevy version 0.13 and has an [MSRV][] of 1.77.

### Fixed

- Fixes a window hang issue in bevy on native platforms

## [0.1.1] - 2024-04-04

This release supports Bevy version 0.13 and has an [MSRV][] of 1.77.

### Fixed

- fixed panic on Windows when window is minimized

## [0.1.0] - 2024-03-26

This release supports Bevy version 0.13 and has an [MSRV][] of 1.77.

- Initial release

[#77]: https://github.com/linebender/bevy_vello/pull/77

[@simbleau]: https://github.com/simbleau

[Unreleased]: https://github.com/linebender/bevy_vello/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/linebender/bevy_vello/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/linebender/bevy_vello/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/linebender/bevy_vello/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/linebender/bevy_vello/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/linebender/bevy_vello/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/linebender/bevy_vello/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/linebender/bevy_vello/compare/v0.3.3...v0.4.0
[0.3.3]: https://github.com/linebender/bevy_vello/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/linebender/bevy_vello/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/linebender/bevy_vello/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/linebender/bevy_vello/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/linebender/bevy_vello/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/linebender/bevy_vello/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/linebender/bevy_vello/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/linebender/bevy_vello/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/linebender/bevy_vello/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/linebender/bevy_vello/releases/tag/v0.1.0

[MSRV]: README.md#minimum-supported-rust-version-msrv
