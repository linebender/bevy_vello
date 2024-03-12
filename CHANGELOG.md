# Changelog

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.


## 0.6

### changed

- Bevy 0.13 support

## 0.5.1

### fixed

- `DebugVisualizations` for assets were erroneously rotated 90 degrees.

## 0.5

### added

- `VelloAssetBundle` now has a field `z_function` used for depth-sorting vector assets.
- `ZFunction` is now visually represented in some debug visualizations.

### changed

- Changed API to PlayerState to a builder pattern. Methods now follow the patterns `set_item`, `get_item`, `item`.
- Playheads are now advanced in the `First` schedule in Bevy.
- PlayerState's `reset_playhead_on_transition` was renamed to `reset_playhead_on_exit`

### fixed

- If no theme is provided to a state, it will no longer overwrite with an empty (default) theme.
- If no playback options are provided to a state, it will no longer overwrite with default playback options.
- `DebugVisualizations` now correctly render for assets in screen-space or world-space when not bundled with a UI Node.

## 0.4.4

### fixed

- A panic that can occur with `player.state()` or `player.state_mut()`

## 0.4.3

### fixed

- Player (state machine) transitions are ignored when the target state is the current state.
- An issue where the playhead can become unbounded when requested segments are smaller than an update's delta time.

## 0.4.2

### fixed

- An issue where playheads can not exist, causing a panic, when transitioning states in the `Update` set.

## 0.4.1

### fixed

- the default for `DebugVisualizations` is now `Hidden`, as was intended

## 0.4.0

### added

- A `prelude` module
- State machines are now available by adding a `LottiePlayer` and states (e.g. `player.with_state()`) to a vello asset bundle.
- `PlaybackOptions` can now be bundled with `VelloAssetBundle` to augment playback
- `Playhead` is now automatically created for all assets and can be queried to inspect and seek frames
- `DebugVisualizations` now apply to `VelloTextBundle`s to help render debug gizmos. Currently this only works for world space.

### changed

- `RenderMode` changed to `CoordinateSpace`
- `Vector` has been renamed to `VectorFile`
- `VelloVector`, anywhere mentioned, has changed to `VelloAsset` (e.g. `VelloVectorBundle` -> `VelloAssetBundle`)
- `ColorPaletteSwap` renamed to `Theme`
- `Theme` now swaps by layer name only, no longer shape numbers. In the future we may adopt [LSS](https://github.com/LottieFiles/lottie-styler/blob/main/apps/docs/docs/intro.md) and parse it with a [DSL macro](https://doc.rust-lang.org/rust-by-example/macros/dsl.html).
- Added `PlaybackAlphaOverride` component to override playback alpha
- The default behavior for `VelloText` is now to render up and to the right, instead of centered. In the future this may be modified with `TextAlignment`, but currently is a TODO.

### fixed

- Color swapping (now "themeing") now properly applies to more cases.

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
- switched to `velato` for rendering lotties
- switched to `vello-svg` for rendering svgs

## 0.2

### changed

- Updated to bevy 0.11

## 0.1

- Initialize release (bevy 0.10)
