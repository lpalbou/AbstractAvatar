# Frame contract

## Parameter descriptor

A versioned JSON object describes a frame request:

```json
{
  "schema_version": 1,
  "style": "halo",
  "state": "speaking",
  "time": 1.2,
  "level": 0.6,
  "gaze": [0.0, 0.0]
}
```

A machine-readable [JSON Schema](../schema/descriptor.schema.json) defines the fields. Optional
`bands` contains exactly eight normalized audio-band levels; omitting it uses the scalar level.
The descriptor names trusted built-in generation. It contains no code, shader source, file path,
URL or library name. Unsupported versions, styles, states and fields are explicit errors.
Dimensions and palette are supplied by the renderer/host, outside the descriptor.

Time is elapsed seconds in 0..86400. Audio level is normalized from 0 to 1. Gaze uses centered coordinates
from -1 to 1. Numeric values must be finite; a JSON descriptor is limited to 4096 UTF-8 bytes. Runtime adapters
should use a local elapsed clock, not an absolute epoch timestamp.

## Raster layout

- RGBA8: four bytes per pixel in red, green, blue, alpha order.
- Rows are tightly packed with stride `width * 4`, top to bottom.
- A frame contains exactly `width * height * 4` bytes.
- Each dimension is between 1 and 512 inclusive; at most 16 native handles can be live.
- Transparent output carries alpha; the host chooses its background and palette.

RGBA is a pixel layout, not a new media container. [PNG](https://www.w3.org/TR/png/) provides the
portable file representation for previews. The tiny descriptor is an AbstractAvatar contract;
it is not claimed to be a general procedural-media standard.

## Lifetime and scheduling

A renderer owns its handle and fixed frame buffer. Copy pixels into an owned image before
reusing or destroying that buffer. Native callers synchronize access across render, copy,
palette updates and destruction; the Python wrapper handles that boundary.

One visible view should reuse one renderer. Do not allocate a renderer for every animation
frame. Close on disposal, stop clocks while hidden, and present a static frame for reduced
motion. Expire pointer-derived gaze after inactivity rather than installing global listeners.

The host chooses the frame rate. The pose API provides bounded cadence suggestions: quiet
states need fewer frames than speaking or working states. A frame request does no background
work after it returns.

## Browser execution

[WebAssembly](https://webassembly.github.io/spec/core/intro/introduction.html) executes the shared
generator in the browser. Load a trusted bundled artifact, verify its ABI/catalog/memory
requirements, and give it no ambient host imports. Descriptors must never choose arbitrary
modules. The browser presenter only schedules and presents frames.
