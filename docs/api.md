# API

## Python

`Renderer(width=128, height=128, library_path=None)` owns one bounded native rendering handle.
Use it as a context manager or call `close()` when its host is disposed.

`render(style="halo", state="ready", level=0.0, time=0.0, gaze=(0, 0), transparent=True)`
returns owned RGBA bytes. Store keys from `catalog()`, and use the named activity states listed
in [the catalog guide](catalog.md). An explicit `library_path` selects a trusted native build;
normal usage loads the library bundled in the Python package. Optional `bands` supplies eight
normalized audio-band levels. `set_palette(warm=..., eye=..., dim=..., ground=...)` takes three
RGB color anchors and an RGBA background; the host retains theme ownership.

`render_descriptor()` consumes the bounded parameter object described in
[the frame contract](frame-contract.md). It does not load files, URLs or executable content
specified by that object.

## Rust

`AvatarStyle` names the built-in designs; `AvatarStyle::ALL` enumerates them. `AvatarPose` names
activity. `Frame::new(style, pose, level, time, gaze)` prepares geometry for repeated sampling.
`Frame::rgba(x, y, pixel_size, &palette)` returns a composited, straight-alpha RGBA8 foreground.
`Palette` is `[[u32; 256]; 3]`: Warm, Eye and Dim shade ramps packed as `0xRRGGBBAA`.
Prepare these ramps once per theme. `pixel_size` is the full normalized pixel width, usually
`2.0 / min(width, height)`, bounded to 0..0.125; zero requests an analytic point sample.
Non-finite inputs, out-of-range pixel sizes and coordinates outside -1.5..1.5 return transparent
black. Coordinates are centered, with y downward and one unit equal to the avatar radius.

`Frame::sample(x, y)` remains available as a single material/light/coverage approximation.
Use `rgba` to retain overlapping material colors, translucent ribbons and opaque shaded surfaces.
The native, Python and WebAssembly renderers use this same compositing path. See
[rendering](rendering.md) for antialiasing, palette behavior and scheduling boundaries.

The `ffi` feature supplies the native C/WebAssembly adapter. Geometry remains usable without a
UI toolkit. The optional descriptor feature parses the versioned parameter object.

## Native and WebAssembly

[`avatar.h`](../avatar.h) is the C ABI reference. The `avatar_*` exports enumerate metadata,
create and release handles, set palettes and gaze, render a frame and expose its pixel buffer.

`avatar_render_copy` renders and copies into caller-owned memory under the native registry lock;
use it for native integrations. The borrowed-pointer functions (`avatar_render` with `avatar_pixels`) require caller synchronization.
Native callers must follow [the frame contract](frame-contract.md). Browser adapters copy pixels from WebAssembly memory
before reusing or destroying the handle. Do not persist numeric catalog codes or native handles.

The UI toolkit and application remain outside the ABI. A host supplies a frame cadence and
activity state; the generator returns a picture.
