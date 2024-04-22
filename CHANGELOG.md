# Changelog

<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

## Unreleased

## 0.2.2 (2024-04-22)

### fixed

- Now when a `VelloScene` and `VelloText` have the same Z-Index, text will be rendered above the scene.

## 0.2.1 (2024-04-21)

### fixed

- `VelloTextAlignment` is now in the `bevy_vello::prelude`.
- The playhead now will now always be bounded
- A rare issue where, if an asset was not available, parts of a state would not transition properly.

## 0.2.0 (2024-04-17)

### added

- Added the `VelloTextAlignment` component to `VelloTextBundle`, which now helps control the alignment of text.
- Added the `VelloTextAlignment` to the `bevy_vello::prelude`.

### fixed

- Text bounding boxes are now tighter as they are capped by the baseline.

## 0.1.2 (2024-04-08)

### fixed

- Fixes a window hang issue in bevy on native platforms

## 0.1.1 (2024-04-04)

### fixed

- fixed panic on Windows when window is minimized

## 0.1.0 (2024-03-26)

- Initial release
