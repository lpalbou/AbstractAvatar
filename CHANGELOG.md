# Changelog

## [0.1.1] - 2026-11-03

### Changed

- Gaze moves eyes only. Each catalog design declares a `gaze` component: `"eyes"` for the 14
  designs that draw distinct eyes (`hover`, `eye`, `rover`, `watch-face`, `watch-eye`,
  `watch-lens`, `watch-dome`, `watch-valet`, `watch-retro`, `watch-deco`, `watch-rover`,
  `watch-hover`, `watch-spark`, `watch-android`) and `"none"` for every other design. A gaze
  input shifts only the pupils, irises or eye lamps of `eyes` designs, by at most
  `GAZE_REACH` (0.05 of the avatar radius horizontally, 0.025 vertically), in the ready state.
  Orbs, forms and every `none` design no longer translate toward the gaze; their frames are
  identical with or without it.

### Added

- `AvatarStyle::gaze()` and `GAZE_REACH` in Rust, catalog field 5 (`gaze`) in the C/WebAssembly
  ABI, a `gaze` key in Python `catalog()` records and in `catalog.json`.

- PyPI release lane: a release workflow builds the `abstractavatar` Python sdist and wheels for
  Linux x86_64 and aarch64 (manylinux), macOS arm64 and x86_64, and Windows x86_64, and publishes
  them to PyPI through trusted publishing. Wheels bundle the native library and serve Python
  3.10+. Version 0.1.0 is published to PyPI from the `v0.1.0` release source, so the crate and
  the Python distribution carry the same version.

## [0.1.0] - 2026-11-02

First release of the `abstractavatar` crate on crates.io. AbstractAvatar is a local procedural
avatar generator: your application supplies a style key, an activity state, time, audio level and
gaze, and the crate returns composited RGBA pixels. One Rust implementation serves terminal,
native and browser hosts.

### Added

- A catalog of 44 procedural designs (shaded characters, particle orbs, forms, portraits,
  machines, nature scenes and a woven orb) with stable style keys, display names, families and
  design provenance.
- Seven explicit activity states: ready, thinking, working, listening, speaking, waiting and
  error. Audio level, eight optional audio bands and gaze add visual input to the chosen state;
  listening and speaking poses respond to measured loudness.
- Rust API: `AvatarStyle`, `AvatarPose`, `Gaze`, `Frame::new` for repeated sampling, and
  `Frame::rgba` for composited straight-alpha RGBA8 through host-owned `Palette` ramps, with
  bounded pixel footprints and antialiasing. `Frame::sample` returns a single-material sample.
- Layered material compositing: overlapping material light keeps each color contribution, and
  shaded surfaces and translucent ribbons composite in the shared rasterizer.
- `ffi` feature: a C ABI (`avatar.h`) with bounded handles (1..512 pixels, at most 16 live
  handles), palette, gaze and audio-band setters, and `avatar_render_copy` into caller-owned
  memory. The same exports build for `wasm32-unknown-unknown` for browser Canvas presenters.
- `json` feature: the versioned, inert parameter descriptor (`schema/descriptor.schema.json`).
- Python binding and `python -m abstractavatar` preview command, installed from a source
  checkout with `python -m pip install .`.

### Compatibility

- Persist style keys. Numeric catalog codes are resolved at runtime and are not preference values.
- The first six ABI catalog codes (`halo`, `hover`, `vortex`, `eclipse`, `eye`, `rover`) keep
  their ordering.
- Rust 1.87 or newer.
