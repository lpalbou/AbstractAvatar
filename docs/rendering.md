# Rendering and theme integration

AbstractAvatar generates foreground RGBA frames from trusted built-in designs. The same Rust
compositing path serves Python, native and WebAssembly hosts. Hosts provide their theme, observed
activity, time, voice inputs and optional gaze; the generator owns geometry and rasterization.
See [the API](api.md) for entry points and [the frame contract](frame-contract.md) for ownership.

## Choose a sampling interface

Use `Frame::rgba(x, y, pixel_size, &palette)` for full rendering. It preserves overlapping colored
light, soft glow, translucent ribbons and shaded opaque surfaces. `Frame::sample` supplies a
single-material approximation for consumers that need material identity instead of color.

Coordinates are centered, with y downward. One unit is the avatar radius; normal square frames
sample x/y in -1..1. The supported sampling extent is -1.5..1.5 on each axis. RGBA sampling returns
transparent black outside this extent or for invalid numbers.

`pixel_size` is a pixel's full width in normalized coordinates. For a square 128-pixel image use
`2.0 / 128.0`. Non-square or terminal hosts can use the larger of their normalized horizontal and
vertical sample steps. Values must be finite and in 0..0.125 inclusive; zero requests analytic
point sampling. A host with a larger footprint can cap it at 0.125, as the native renderer does.

Thin strokes and particles integrate a Gaussian footprint to reduce subpixel flicker. Filled
edges use analytic coverage. Footprints above 0.03125 subdivide at most twice, producing at most
16 samples per output pixel; ordinary 64-pixel and larger images need one spatial lookup.

## Supply material ramps

`Palette` contains three arrays of 256 packed `0xRRGGBBAA` shades, in Warm, Eye and Dim order.
Each ramp runs from shadow at index 0 to highlight at index 255. Build ramps once per theme,
outside the pixel loop. Alpha in every ramp is respected.

```rust
use abstractavatar::{AvatarPose, AvatarStyle, Frame, Gaze, Palette};

let palette: Palette = [
    std::array::from_fn(|i| u32::from_be_bytes([i as u8, 80, 40, 255])),
    std::array::from_fn(|i| u32::from_be_bytes([40, i as u8, 140, 255])),
    std::array::from_fn(|i| u32::from_be_bytes([i as u8, i as u8, i as u8, 255])),
];
let frame = Frame::new(AvatarStyle::WatchHover, AvatarPose::Ready, 0.0, 1.2, Gaze::default());
let foreground = frame.rgba(0.0, 0.0, 2.0 / 128.0, &palette);
assert_eq!(foreground[3], 255);
```

Composition uses premultiplied color internally and returns straight-alpha RGBA8. An output
alpha of zero has RGB zero. Composite the foreground over the host background once. The native
renderer can do that final background composition when `transparent` is false.

Three theme anchors define the available color vocabulary. A Watch-derived design can therefore
have the same silhouette and movement in a warm, cool or monochrome host theme. A source design's
independent rainbow palette is not a separate implicit palette in this contract.

## Reuse prepared frames

Build one `Frame` for a state/time/input combination, then sample it across the image. Geometry
and spatial indexes are prepared once; `rgba` and `sample` allocate no per-pixel objects. Apple
scene generation is limited to 4096 total strokes and surfaces, with 48×48 spatial indexes.
Native raster buffers remain limited to 512 pixels per dimension and 16 live handles.

Hosts choose resolution, frame cadence and visibility. Stop clocks when hidden and use a static
frame for reduced motion. Supersampling cannot restore details lost when an image is converted
to a small terminal cell grid; pixel-capable presenters can retain more of the generated image.

Version 1 is a stateless frame contract. Transition history, accumulated phase and expressive
behavior need a separately specified continuity contract; they are not inferred from a pose
label. See [the frame contract](frame-contract.md) and [ADR 0002](adr/0002-material-compositing.md).
