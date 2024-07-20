# Changelog

<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

## Unreleased

### Changed

- The field `VelloAssetBundle.vector` was renamed to `VelloAssetBundle.asset`.
- Renamed `VelloAssetAlignment` to `VelloAssetAnchor`. Fields were renamed `alignment` were renamed to `asset_anchor`.
- Renamed `VelloTextAlignment` to `VelloTextAnchor`. Fields were renamed `alignment` were renamed to `text_anchor`.

### Removed

- Removed `ZFunction`s from the render pipeline. Now ordering is based solely on the `Transform`'s z component. If you dependeded on this behavior, you'll need to adjust the transform Z in a system prior to render.

## 0.5.1

### Fixed

- Updated to patch vello 0.2.1. It is now no-longer possible to panic when the vello encodings are empty.
- The demo CI now deploys that bevy_pancam has been updated to bevy 0.14

## 0.5.0

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

## 0.4.2

### Fixed

- Updated to vello_svg v0.2.0, fixing viewboxes.
- Updates to velato v0.2.0, fixing viewboxes.

## 0.4.1 (Yanked)

## 0.4.0

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

## 0.3.3

### Fixed

- Projects with a 2D and 3D camera should no longer conflict with `bevy_vello`'s queries.

## 0.3.2

### Added

- Inverse `ZFunction` options added for `BbTop`, `BbBottom`, `BbLeft`, and `BbRight`.

### Fixed

- A panic that can happen in the extract schedule of lottie files.
- Z-ordering now works correctly for `Bb` functions.

## 0.3.1 (2024-05-01)

### Fixed

- `bevy_vello::prelude::Scene` was removed, since it conflicts with `bevy::prelude::Scene`.

## 0.3.0 (2024-05-01)

### Added

- `VelloAssetAlignment` was added to the `VelloAssetBundle`.

### Changed

- `VectorFile` enum variants were flattened into tuple structs.

### Removed

- `bevy_vello::VelloPlugin` was removed from the prelude.

## 0.2.2 (2024-04-22)

### Fixed

- Now when a `VelloScene` and `VelloText` have the same Z-Index, text will be rendered above the scene.

## 0.2.1 (2024-04-21)

### Fixed

- `VelloTextAlignment` is now in the `bevy_vello::prelude`.
- The playhead now will now always be bounded
- A rare issue where, if an asset was not available, parts of a state would not transition properly.

## 0.2.0 (2024-04-17)

### Added

- Added the `VelloTextAlignment` component to `VelloTextBundle`, which now helps control the alignment of text.
- Added the `VelloTextAlignment` to the `bevy_vello::prelude`.

### Fixed

- Text bounding boxes are now tighter as they are capped by the baseline.

## 0.1.2 (2024-04-08)

### Fixed

- Fixes a window hang issue in bevy on native platforms

## 0.1.1 (2024-04-04)

### Fixed

- fixed panic on Windows when window is minimized

## 0.1.0 (2024-03-26)

- Initial release
