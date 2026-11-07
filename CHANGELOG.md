# Changelog

## [0.2.0] - unreleased, date to set at release

This release changes the catalog: 17 designs leave it. Stored ids keep rendering (see
Migration).

### Removed

- `eye`, `watch-drop`, `watch-shape-kitty`, `watch-shape-face` and `watch-shape-wave` leave the
  catalog.
- `watch-face`, `watch-imprint`, `watch-contour`, `watch-membrane`, `watch-lens`, `watch-dome`,
  `watch-valet`, `watch-retro`, `watch-deco`, `watch-cruiser`, `watch-android` and
  `watch-sentinel` leave the catalog until they are redesigned from their Apple Watch originals.
  A redesigned face returns under its original id.
- The catalog lists 27 designs. Numeric catalog codes are positions in the installed catalog, so
  the codes after `eclipse` (3) change (`rover` is 4); ids do not. The 0.1.0 promise that the
  first six codes keep their ordering no longer holds: resolve codes from the catalog by id.

### Added

- Retired ids resolve instead of failing. `AvatarStyle::resolve(id)` returns the active design,
  a retired id's replacement, or `AvatarStyle::DEFAULT` (`halo`) for a retired id without one
  and for any unknown id. `AvatarStyle::retired(id)` and `RETIRED` describe each retired id
  (version, decision date, reason, replacement).
- `catalog.json` gains `default` and a `retired` list with the same records.
- C/WebAssembly exports `avatar_default_style`, `avatar_retired_count`,
  `avatar_retired_field_ptr` and `avatar_retired_field_len` (ABI 1, additive).
- Python `retired()` and `resolve_style(id)`. `Renderer.render` and `render_descriptor` draw a
  retired or unknown id as its resolved design.

### Changed

- `watch-rings` moves each of its six rings with its own part of the voice. With measured
  `bands`, ring 1 follows the lowest bands and ring 6 the highest; each ring tilts, swells and
  brightens with its band, pushing out while speaking and drawing in while listening. With a
  level only, each ring follows the level through its own rhythm, so the rings still move one
  by one.
- Thinking and working are distinct animations in every design. Thinking draws inward and moves
  slowly; working reaches outward and moves fast. Updated designs: `eclipse`, `rover`,
  `watch-halo`, `watch-vortex`, `watch-nebula`, `watch-rings`, `watch-liquid`, `watch-galaxy`,
  `watch-synapse`, `watch-accretion`, `watch-rover` and `watch-spark`.
- The JSON descriptor accepts any style id of 1 to 64 bytes and draws it through
  `AvatarStyle::resolve`. Python `validate_descriptor(..., styles=...)` still checks an id
  against a set you pass.

### Fixed

- The `iphone-weave` listening mesh expands with microphone energy. Its geometry previously
  ignored the level while the listening globe was selected.
- Artifact provenance (`artifacts/provenance.json`) records the crate version from
  `Cargo.toml`.

### Migration

- Persist ids, never catalog codes. Look codes up from the installed catalog.
- A stored retired id renders its replacement: `eye` draws `watch-eye`; `watch-lens`,
  `watch-dome`, `watch-valet`, `watch-retro`, `watch-deco`, `watch-cruiser`, `watch-android`
  and `watch-sentinel` draw `watch-hover`; `watch-shape-wave` draws `watch-wave`. The others
  draw `halo`. To offer a new choice, compare the stored id with the retired list.
- Hosts that resolve a style from the catalog themselves should apply the retired list
  (`avatar_retired_*` or `catalog.json`) before treating an id as unknown.

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
