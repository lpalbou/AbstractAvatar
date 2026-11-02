# Changelog

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
