# Changelog

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

## 0.4.0

### added

- State machines are now available by adding a `LottiePlayer` and states (e.g. `player.with_state()`) to a vello asset bundle.
- `PlaybackOptions` can now be bundled with `VelloAssetBundle` to augment playback
- `Playhead` is now automatically created for all assets and can be queried to inspect and seek frames

### changed

- `RenderMode` changed to `CoordinateSpace`
- `Vector` has been renamed to `VectorFile`
- `VelloVector`, anywhere mentioned, has changed to `VelloAsset` (e.g. `VelloVectorBundle` -> `VelloAssetBundle`)
- `ColorPaletteSwap` renamed to `Theme`
- `Theme` now swaps by layer name only, no longer shape numbers. In the future we may adopt [LSS](https://github.com/LottieFiles/lottie-styler/blob/main/apps/docs/docs/intro.md) and parse it with a [DSL macro](https://doc.rust-lang.org/rust-by-example/macros/dsl.html).
- Added `AlphaOverride` component to override playback alpha

### fixed

- Color swapping (now "themeing") now properly applies to more cases.
- `DebugVisualizations` in the screen `CoordinateSpace` now render correctly

### removed

- `Origin` on vector assets
- `.svg.gz` support - [Since a Gzip plugin is now possble](https://github.com/bevyengine/bevy/issues/10518)
- `.json.gz` support - [Since a Gzip plugin is now possble](https://github.com/bevyengine/bevy/issues/10518)

## 0.3

### added

- added `Origin` to `VelloVectorBundle` to give the user the option to change the vector's transform origin
- support for `.svg.gz` (gzipped svg) and `.json.gz` (gzipped lottie)

### changed

- updated to bevy 0.12
- switched to `vellottie` for rendering lotties
- switched to `vello-svg` for rendering svgs

## 0.2

### changed

- Updated to bevy 0.11

## 0.1

- Initialize release (bevy 0.10)
