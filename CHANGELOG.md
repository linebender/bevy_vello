# Changelog

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

## Unreleased

### added

- State machines are now available behind the `player` cargo feature
- `PlaybackSettings` can now be bundled with `VelloAssetBundle` to augment playback

### changed

- `RenderMode` changed to `CoordinateSpace`
- `Vector` has been renamed to `VelloAssetData`
- `VelloVector`, anywhere mentioned, has changed to `VelloAsset`
- `DebugVisualizations` are now feature-gated behind the `debug` cargo feature
- Color swapping now swaps by layer name only and applies to more cases (animated, gradients, etc.)

### removed

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
