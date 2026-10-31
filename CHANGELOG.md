# Changelog

## Unreleased

### Added

- Shared procedural avatar catalog with explicit activity states, audio level, time and gaze.
- Rust sampling API, native C interface and browser WebAssembly build.
- Python rendering API and PNG-preview command.
- Versioned parameter descriptor and bounded raster frame contract.

### Fixed

- Legacy Vortex responds to measured microphone loudness while Listening.

### Compatibility

- The six shaded style identifiers and their first six ABI catalog codes retain their ordering.
- Persist style keys. Numeric catalog codes are resolved at runtime and are not preference values.
