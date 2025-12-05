# Changelog

<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

The latest published Bevy Vello release is [0.10.2](#0102---2025-06-29) which was released on 2025-06-29.
You can find its changes [documented below](#0102---2025-06-29).

## [Unreleased]

This release supports Bevy version 0.17 and has an [MSRV][] of 1.87.

### Added

- View culling is now automatically performed for all assets. All assets not within the view of a camera tagged with `VelloView` will be culled prior to rendering. This works by culling AABBs not seen by the cameras. see the view culling example.
  - **WARNING** There is no way to retrieve the content size of a `vello::Scene`. Hence, `VelloScene2d` should be given an `Aabb` manually by the developer. By default, `Aabb::default()` is used, which is never updated (or gets view-culled).
  - AABBs will automatically update for `VelloSvg2d` using the asset size.
  - AABBs will automatically update for `VelloLottie2d` using the asset size.
  - AABBs will automatically update for `VelloText2d` using the text size.
- `ContentSize` for Ui nodes will automatically update for all assets.
  - **WARNING** There is no way to retrieve the content size of a `vello::Scene`. Hence, `UiVelloScene` should have `ContentSize` manually managed by the developer. Otherwise, we assume a 0x0 content measurement, which takes no layout space.
  - UI will automatically measure `UiVelloSvg` using the asset size.
  - UI will automatically measure `UiVelloLottie` using the asset size.
  - UI will automatically measure `UiVelloText` using the text size.
  - Several diagnostics were added. See `bevy_vello::render::diagnostics` for all of them.

### Changed

- The default `VelloTextAnchor` is now `VelloTextAnchor::Center`.
- `VelloTextSection` (now `VelloText2d`/`UiVelloText`) no longer has `height`, and `width` has been renamed to `max_advance`.
- `VelloScene` has been split into `VelloScene2d` and `UiVelloScene`. The former is for world entities and the latter is for UI nodes.
- `VelloTextSection` has been split into `VelloText2d` and `UiVelloText`. The former is for world entities and the latter is for UI nodes.
- `VelloSvgHandle` has been split into `VelloSvg2d` and `UiVelloSvg`. The former is for world entities and the latter is for UI nodes.
- `VelloLottieHandle` has been split into `VelloLottie2d` and `UiVelloLottie`. The former is for world entities and the latter is for UI nodes.
- Several diagnostics changed names. See `bevy_vello::render::diagnostics` for all of them.

### Removed

- `VelloSceneBundle` no longer exists. Use `VelloScene2d` and `UiVelloScene` instead.
- `VelloTextBundle` no longer exists. Use `VelloText2d` and `UiVelloText` instead.
- `VelloSvgBundle` no longer exists. Use `VelloSvg2d` and `UiVelloSvg` instead.
- `VelloLottieBundle` no longer exists. Use `VelloLottie2d` and `UiVelloLottie` instead.
- `SkipScaling` no longer exists. If you need to scale something, use the entity's transform.
- `VelloWorldScale` no longer exists. If you need to scale something, use the entity's transform.
- `VelloScreenScale` no longer exists. If you need to scale something, use the entity's transform.
- `VelloScreenSpace` no longer exists. You should use a separate camera for UI and manually place items into screen space. There are now examples for screenspace to help.

### Fixed

- Renderables (text, images, scenes) now respect the camera projection scale.
- Objects are now scaled according to the scale factor (pixel density) of the viewport/window. This fixes scaling on retina displays.
- Render targets are now resized when camera viewport size changes

## [0.10.3] - 2025-07-09

This release supports Bevy version 0.16 and has an [MSRV][] of 1.87.

### Fixed

- Fixes a compile issue in the `text` cargo feature.

## [0.10.2] - 2025-06-29 (YANKED)

This release supports Bevy version 0.16 and has an [MSRV][] of 1.87.

### Fixed

- Systems that calculate the content size of `VelloTextSection` now run in the `PostUpdate` schedule to ensure that all `Handle<VelloFont>` are loaded before calculating the content size.

## [0.10.1] - 2025-06-27

This release supports Bevy version 0.16 and has an [MSRV][] of 1.87.

### Fixed

- `ZIndex` is now respected by vello render items that have a `Node` or `VelloScreenSpace` component.
- Screen space items (those with `Node` or `VelloScreenSpace`) now always render on top.
- The vello canvas now respects the correct camera viewport size (defaulting to window size if not present) which should be used when creating the render texture

## [0.10.0] - 2025-06-23

This release supports Bevy version 0.16 and has an [MSRV][] of 1.87.

### Added

- `VelloScreenScale` resource added to control the scale of screen space rendering and is exported via prelude.
- `VelloWorldScale` resource added to control the scale of world space rendering and is exported via prelude.
- `SkipScaling` component added for disabling scaling for specific entities and is exported via prelude.
- `VelloScreenSpace` component added and exported via prelude.
- Added systems that calculates the content size of `VelloTextSection` if it has a `ContentSize` component which is also `VelloScreenScale` aware.
- Example demonstrating scaling added to `examples/scaling`.

### Changed

- `VelloScene` can now be placed in screen space independent of bevy_ui by adding a `VelloScreenSpace` component.
- `VelloTextSection` now supports `VelloScreenSpace` to render text in screen space independent of bevy_ui.
- `VelloTextSection` now supports `Node` to render text in screen space with bevy_ui.
- `VelloTextSection` now supports `ContentSize` to calculate the content size of the text for bevy_ui.
- `VelloSvgHandle` now supports `VelloScreenSpace` to render SVGs in screen space independent of bevy_ui.
- `VelloLottieHandle` now supports `VelloScreenSpace` to render Lotties in screen space independent of bevy_ui.
- `local_transform_matrix` for lotties and svgs no longer defaults to y-up to reduce complexity of y-axis flips.

### Fixed

- Fixes rotation and scale matrix calculations for prepared affines in all spaces for all assets.

## [0.9.0] - 2025-05-31

This release supports Bevy version 0.16 and has an [MSRV][] of 1.87.

### Added

- Adds `view_culling` example demonstrating view culling with Bevy's `Aabb` component.
- Adds support for Bevy's view culling using `VisibilityClass` and `add_visibility_class` system.
- Adds `VisibilityClass` and `add_visibility_class` hook to `VelloScene`
- Adds `VisibilityClass` and `add_visibility_class` hook to `VelloSvgHandle`
- Adds `VisibilityClass` and `add_visibility_class` hook to `VelloLottieHandle`
- Adds `VisibilityClass` and `add_visibility_class` hook to `VelloTextSection`
- Adds `tracing` crate as bevy removed built-in tracing macros

### Changed

- Updates `bevy` to 0.16
- Updates `vello` to 0.5.0
- Updates `velato` to 0.6.0
- Updates `vello_svg` to 0.7.0
- Updates `parley` to 0.4.0
- Moves `hide_when_empty` system into `CheckVisibility` system set.

## [0.8.0] - 2025-04-29

This release supports Bevy version 0.15 and has an [MSRV][] of 1.85.

### Changed

- All text rendering is now locked behind the `text` cargo feature. See the text example for help.
- Replaces `skrifa` with `parley`, which is the preferred shaping library in the linebender ecosystem.
- Changes `VelloFont` struct internals.
- A `parley::FontContext` and `parley::LayoutContext` has been added in a lazy load multi threaded capacity.
- Replaces `Rubik-Medium.ttf` from the `text` example with variable font `RobotoFlex-VariableFont_GRAD,XOPQ,XTRA,YOPQ,YTAS,YTDE,YTFI,YTLC,YTUC,opsz,slnt,wdth,wght.ttf`.
- Updates the `text` example with an interactive, keyboard controlled, variable font example.
- Adds `text_align` and `width` to `VelloTextSection` for controlling text alignment and width.
- Adds `VelloTextAlign` enum for controlling text alignment.
- `VelloSvgHandle` now derives `Reflect`.
- `VelloSvgAnchor` now derives `Debug` and `Reflect`.
- `VelloLottieHandle` now derives `Reflect`.
- `VelloLottieAnchor` now derives `Debug` and `Reflect`.
- `PlaybackOptions` now derives `Reflect`.

### Fixed

- `vello_svg` is now only brought in when the `svg` feature is active.
- `velato` is now only brought in when the `lottie` feature is active.

## [0.7.1] - 2025-03-12

This release supports Bevy version 0.15 and has an [MSRV][] of 1.85.

### Fixed

- Render loop no longer attempts to render items with 0 opacity.
- A panic was removed for lotties that are still in the process of loading.

## [0.7.0] - 2025-02-27

This release supports Bevy version 0.15 and has an [MSRV][] of 1.85.

### Added

- Added `VelloView` marker component used for identifying cameras rendering vello content.
- Added `VelloEntityCountDiagnosticsPlugin` which can be used to provide vello entity type data at runtime. See the `diagnostics` example.
- Added `VelloFrameProfileDiagnosticsPlugin` which can be used to provide vello frame profile data at runtime. See the `diagnostics` example.

### Changed

- bevy_vello now uses Bevy 0.15
- `Camera2d` now requires a `VelloView` marker for rendering.
- `VelloAsset` assets have been separated into `VelloSvg` and `VelloLottie`
- `VelloAssetBundle` has been separated into `VelloSvgBundle` and `VelloLottieBundle`
- `Handle<VelloAsset>` has been separated into `VelloSvgHandle` and `VelloLottieHandle`
- `VelloAssetAnchor` has been separated into `VelloSvgAnchor` and `VelloLottieAnchor`
- The license on bevy_vello no longer includes OFL 1.1
- The `experimental-dotLottie` feature was removed and merged into the `lottie` feature.
- `DotLottiePlayer` was renamed to `LottiePlayer`.
- All render types (`VelloSvgHandle`, `VelloLottieHandle`, `VelloScene`, and `VelloTextSection`) now have required components as an alternative to their bundle counterparts.

### Removed

- Removed `DebugVisualizations`. Now you should use `BorderColor` for UI Nodes and `Gizmos` for world assets. Several examples show this capability, such as the `svg` and `svg_ui` examples.
- Removed `CoordinateSpace`. If you wish to render scene or asset UI, insert a `Node` component. For more information, see the `scene_ui`, `svg_ui`, or `lottie_ui` examples.
- `VelloText` (with `CoordinateSpace::ScreenSpace`) can no longer render text in screen space. You should be using bevy's native `Text` for UI text, which is more feature rich and widely used.

### Fixed

- We no longer bundle the default font twice when the `default_font` feature is active.

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

[Unreleased]: https://github.com/linebender/bevy_vello/compare/v0.10.3...HEAD
[0.10.3]: https://github.com/linebender/bevy_vello/compare/v0.10.2...v0.10.3
[0.10.2]: https://github.com/linebender/bevy_vello/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/linebender/bevy_vello/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/linebender/bevy_vello/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/linebender/bevy_vello/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/linebender/bevy_vello/compare/v0.7.1...v0.8.0
[0.7.1]: https://github.com/linebender/bevy_vello/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/linebender/bevy_vello/compare/v0.6.1...v0.7.0
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
