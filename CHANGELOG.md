# Changelog

## Unreleased

### Added

- Shared procedural avatar catalog with explicit activity states, audio level, time and gaze.
- Rust sampling API, native C interface and browser WebAssembly build.
- Python rendering API and PNG-preview command.
- Versioned parameter descriptor and bounded raster frame contract.
- Composited Rust `Frame::rgba` sampling with host-owned palettes and bounded pixel footprints.

### Fixed

- Legacy Vortex responds to measured microphone loudness while Listening.
- Overlapping material light retains each color contribution in native and browser frames.
- Watch Wave uses five translucent ribbons and a working spiral; Watch Hover has a shaded shell,
  visor and engine plumes. Watch Rings uses six icosahedral great circles, and Watch Eye retains
  asymmetric lids, iris fibers, a brow and lashes. Watch Halo has uneven flowing streamers.

### Compatibility

- The six shaded style identifiers and their first six ABI catalog codes retain their ordering.
- Persist style keys. Numeric catalog codes are resolved at runtime and are not preference values.
