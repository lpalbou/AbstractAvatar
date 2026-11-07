# API

## Python

`Renderer(width=128, height=128, library_path=None)` owns one bounded native rendering handle.
Use it as a context manager or call `close()` when its host is disposed.

`render(style="watch-halo", state="ready", level=0.0, time=0.0, gaze=(0, 0), transparent=True)`
returns owned RGBA bytes. Gaze shifts only the eyes of designs whose catalog `gaze` is `"eyes"`,
in the ready state; other designs ignore it (see [Gaze](frame-contract.md#gaze)). Store keys from `catalog()`, and use the named activity states listed
in [the catalog guide](catalog.md). An explicit `library_path` selects a trusted native build;
normal usage loads the library bundled in the Python package. Optional `bands` supplies eight
normalized audio-band levels, low to high frequency. A retired or unknown `style` draws its
resolved design rather than failing. `catalog()` lists the active designs, each with a `status`
of `"ready"` or `"experimental"`; `ready()` lists only the ready ones, the designs to offer in a
picker or as a default (an experimental id still renders); `retired()` lists the
retired ids with their `replacement` (or `None`), `retired_in`, `date` and `reason`;
`resolve_style(id)` returns the active id that draws for a stored id. `set_palette(warm=..., eye=..., dim=..., ground=...)` takes three
RGB color anchors and an RGBA background; the host retains theme ownership.

`render_descriptor()` consumes the bounded parameter object described in
[the frame contract](frame-contract.md). It does not load files, URLs or executable content
specified by that object.

## Rust

`AvatarStyle` names the built-in designs; `AvatarStyle::ALL` and `AvatarStyle::all()` enumerate
every active design, and `AvatarStyle::ready()` only the ready ones, the designs to offer users.
`AvatarStyle::status()` returns `DesignStatus::Ready` or `DesignStatus::Experimental`
(`DesignStatus::id()` is `"ready"` or `"experimental"`); `is_ready()` tests it. Status changes
discovery only: every active design renders.
`AvatarStyle::from_id(id)` finds an active design; `AvatarStyle::resolve(id)` always returns one:
the design itself, a retired id's replacement, or `AvatarStyle::DEFAULT` (`WatchHalo`).
`AvatarStyle::retired(id)` and `RETIRED` describe retired ids as `RetiredStyle` records. `AvatarPose` names
activity. `Frame::new(style, pose, level, time, gaze)` prepares geometry for repeated sampling;
`Frame::with_bands(..., bands)` adds eight measured audio bands.
`AvatarStyle::gaze()` returns the component a gaze moves, `"eyes"` or `"none"`; `GAZE_REACH`
is the largest eye shift, `[0.05, 0.025]` of the avatar radius.
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
Catalog field 5 is the design's gaze component (`eyes` or `none`); field 6 is its status
(`ready` or `experimental`). `avatar_ready_count` and `avatar_ready_style(index)` list the
catalog codes of the ready designs, in catalog order (`avatar_ready_style` returns `UINT32_MAX`
out of range). `avatar_abi_minor` is 1 for this revision of ABI 1; every ABI 1.0 export is
unchanged, so a host that reads only fields 0..5 keeps working. `avatar_retired_count` and
`avatar_retired_field_ptr`/`_len` list retired ids (fields: id, replacement id or empty, version,
date, reason); `avatar_default_style` returns the code an unknown id should draw.

`avatar_render_copy` renders and copies into caller-owned memory under the native registry lock;
use it for native integrations. The borrowed-pointer functions (`avatar_render` with `avatar_pixels`) require caller synchronization.
Native callers must follow [the frame contract](frame-contract.md). Browser adapters copy pixels from WebAssembly memory
before reusing or destroying the handle. Do not persist numeric catalog codes or native handles.

The UI toolkit and application remain outside the ABI. A host supplies a frame cadence and
activity state; the generator returns a picture.
