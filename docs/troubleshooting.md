# Troubleshooting

## The native library cannot be loaded

Install a built wheel for your platform, or build from source with Rust 1.87+ and
`python -m pip install .`. A source checkout alone does not include a compiled native library.
Use `library_path` only for a trusted explicit build. See [getting started](getting-started.md).

## A style or descriptor is rejected

Run `python -m abstractavatar catalog` and use a supported stable key. Check schema version,
state, finite numeric values and unknown fields against [the frame contract](frame-contract.md).
Do not pass file paths, URLs or shader code as a descriptor.

## Frames are empty or look stretched

Confirm width/height, tightly packed RGBA channel order and image stride. Keep the intended
aspect ratio; use a transparent image over your UI's background. Verify the palette is populated
when using the low-level C interface. See [API](api.md).

## The renderer refuses another handle

Reuse a renderer across frames and close renderers belonging to disposed views. Galleries
should create only the visible presenters, or render preview images sequentially. Do not keep
one live renderer for every off-screen catalog entry.

## Animation uses CPU while hidden

Stop the host's frame clock when the view is hidden or reduced motion applies. The library has
no background animation thread. A static error/idle picture does not need a continuous redraw.
See [lifetime and scheduling](frame-contract.md#lifetime-and-scheduling).
