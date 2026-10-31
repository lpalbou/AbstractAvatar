---
name: abstractavatar
description: Render and integrate AbstractAvatar procedural characters using its local Python, Rust or native/WebAssembly APIs. Use for catalog selection, activity-driven frames, PNG previews and platform frame players; not for generating artwork with an AI model or loading arbitrary animation files.
---

# Use AbstractAvatar

AbstractAvatar is a local procedural generator. It has no network, inference, microphone or
application-preference authority. A host supplies state/time/level/gaze and presents the frame.

## Install and verify

From the cloned AbstractAvatar repository, with Python 3.10+ and Rust 1.87+:

```sh
python -m pip install .
python -m abstractavatar catalog
```

The source installation builds the native library. A built platform wheel bundles that library.
If loading fails, stop and report the missing native artifact; do not search system caches or
download an arbitrary library. An explicit `library_path` is for a trusted chosen build.

## Working model

- Style keys identify designs; store keys, not display names or numeric catalog codes.
- States are `ready`, `thinking`, `working`, `listening`, `speaking`, `waiting`, `error`.
- Inputs are explicit. Time is finite elapsed seconds, level is 0..1, gaze is two values in -1..1.
- Frames are tightly packed RGBA8: width × height × 4 bytes, top-to-bottom rows.
- A renderer owns a reusable native handle; close it. Dimensions are 1..512 and live handles are bounded.
- The host controls palette, cadence and visibility. Hidden/reduced-motion views do not run animation clocks.

## Preview a design

```sh
python -m abstractavatar render --style halo --state speaking --level 0.6 --time 1.2 --size 128 --output avatar.png
```

This writes a PNG at the requested path. Enumerate the catalog before choosing another style.
Use distinct filenames when keeping several previews.

## Present live frames

```python
from abstractavatar import Renderer

with Renderer(width=128, height=128) as renderer:
    rgba = renderer.render(style="halo", state="speaking", level=0.6, time=1.2)
    assert len(rgba) == 128 * 128 * 4
```

Create one renderer per visible view and reuse it across frames. Pass owned returned bytes into
the UI's image object. A generic frame player presents those images; it does not duplicate the
procedural geometry. Rust consumers can sample `Frame`; browser consumers run the same compiled
generator through a trusted WebAssembly adapter.

## Descriptor boundary

```python
descriptor = {
    "schema_version": 1, "style": "halo", "state": "speaking",
    "time": 1.2, "level": 0.6, "gaze": [0.0, 0.0]
}
```

Use `renderer.render_descriptor(descriptor)` while the renderer is open. Descriptors are inert
parameters, not code or scene plug-ins. Unknown fields/styles/versions and invalid numbers are
errors. Never let a descriptor choose a native library, URL or WebAssembly module.

## Diagnose failures

- Missing library: install/build the package for this platform; do not silently substitute artwork.
- Unknown style: enumerate `catalog()` and preserve the stored key for an explicit recovery choice.
- Capacity refusal: close disposed renderers and lazily create visible gallery views.
- Corrupt image: verify RGBA ordering, dimensions and stride.
- Unnecessary CPU: stop host clocks while hidden and show a static frame for reduced motion.

Canonical references: `docs/getting-started.md`, `docs/api.md`, `docs/frame-contract.md`,
`docs/catalog.md` and `docs/troubleshooting.md`. This skill's examples operate locally and do not
contact a gateway or change client preferences.
