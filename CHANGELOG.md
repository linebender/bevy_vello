# Changelog

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

## Unreleased

### changed

- `DebugVisualizations` are now feature-gated behind the `debug` cargo feature

### removed

- `.svg.gz` support - [Since a Gzip plugin is now possble](https://github.com/bevyengine/bevy/issues/10518)
- `.json.gz` support - [Since a Gzip plugin is now possble](https://github.com/bevyengine/bevy/issues/10518)
- color swapping: color swapping will be re-introduced in a future version when it is stabilized

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
