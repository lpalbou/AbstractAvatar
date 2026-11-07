# Frame contract

## Parameter descriptor

A versioned JSON object describes a frame request:

```json
{
  "schema_version": 1,
  "style": "watch-halo",
  "state": "speaking",
  "time": 1.2,
  "level": 0.6,
  "gaze": [0.0, 0.0]
}
```

A machine-readable [JSON Schema](../schema/descriptor.schema.json) defines the fields. Optional
`bands` contains exactly eight normalized audio-band levels, from low to high frequency. Omit it
when you only measure a level: band-aware designs such as `watch-rings` then derive a spread of
motion from the level and time.
The descriptor names trusted built-in generation. It contains no code, shader source, file path,
URL or library name. Unsupported versions, states and fields are explicit errors. `style` is any
id of 1 to 64 bytes: an active id draws that design, a retired id draws its replacement, and any
other id draws the default design `watch-halo` (see [retired designs](catalog.md#retired-designs)).
An experimental id draws that design too: status only decides what a host offers.
Dimensions and palette are supplied by the renderer/host, outside the descriptor.

Time is elapsed seconds in 0..86400. Audio level is normalized from 0 to 1. Gaze uses centered coordinates
from -1 to 1 (see [Gaze](#gaze)). Numeric values must be finite; a JSON descriptor is limited to 4096 UTF-8 bytes. Runtime adapters
should use a local elapsed clock, not an absolute epoch timestamp.

## Gaze

Gaze moves eyes only. Each catalog design declares a `gaze` component:

| `gaze` | Designs | Effect of a gaze input |
|---|---|---|
| `eyes` | Designs that draw distinct eyes: pupils, irises or eye lamps (`hover`, `rover`, `watch-eye`, `watch-hover`, `watch-rover`, `watch-spark`) | The eyes shift toward the gaze, at most `GAZE_REACH` = 0.05 of the avatar radius horizontally and 0.025 vertically. The body, head, shell and every other part stay still. |
| `none` | Orbs, forms, cosmic scenes, nature scenes and the woven orb | Ignored. The frame is identical with or without gaze. |

Gaze applies in the `ready` state; every other state ignores it so that activity motion stays
readable. The generator is stateless and frames are a pure function of their inputs, so temporal
smoothing belongs to the host: ease the gaze value toward the pointer (for example exponential
smoothing over about 150 ms), send gaze only for designs whose `gaze` is `eyes`, and return it to
`[0, 0]` after inactivity. Read the component from the installed catalog rather than hardcoding a
list.

## Raster layout

- RGBA8: four bytes per pixel in red, green, blue, alpha order.
- Rows are tightly packed with stride `width * 4`, top to bottom.
- A frame contains exactly `width * height * 4` bytes.
- Each dimension is between 1 and 512 inclusive; at most 16 native handles can be live.
- Transparent output carries alpha; the host chooses its background and palette.

Output uses straight alpha: RGB channels are not premultiplied by the returned alpha. Internally,
light and surfaces compose in premultiplied form, preserving contributions from all three host
material ramps. Pixels with zero alpha are transparent black. Native frame rendering derives
the antialiasing footprint from the dimensions, capped at 0.125 normalized units. See
[rendering](rendering.md) for the Rust sampling equivalent.

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

Descriptor version 1 selects one activity state for an independent frame. It does not contain
transition weights, velocity, personality, emotional interpretation or audio history. Switching
states can change the shape immediately; a future transition contract requires explicit timing,
continuity and reduced-motion semantics. Hosts should keep elapsed time monotonic across state
changes, and report observed activity and measured voice inputs rather than inventing behavior.

## Browser execution

[WebAssembly](https://webassembly.github.io/spec/core/intro/introduction.html) executes the shared
generator in the browser. Load a trusted bundled artifact, verify its ABI/catalog/memory
requirements, and give it no ambient host imports. Descriptors must never choose arbitrary
modules. The browser presenter only schedules and presents frames.
