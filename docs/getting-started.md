# Getting started

AbstractAvatar generates images locally. Rust 1.87+ builds the native library; Python 3.10+ runs
the Python API and preview command.

## Install from source

From the cloned repository:

```sh
python -m pip install .
python -m abstractavatar catalog
python -m abstractavatar render --style halo --state speaking --level 0.6 --size 128 --output avatar.png
```

Choose a style key from the catalog output. PNG previews use the same procedural generator as
live integrations; they capture one explicit time and state.

## Present native frames

```python
from abstractavatar import Renderer

with Renderer(width=128, height=128) as renderer:
    pixels = renderer.render(style="halo", state="speaking", level=0.6, time=1.2)
    assert len(pixels) == 128 * 128 * 4
```

`pixels` contains tightly packed RGBA bytes. Your UI toolkit owns the image object and draw loop.
See [the frame contract](frame-contract.md) for dimensions, ownership and reduced motion.

## Use Rust

Add the crate from crates.io:

```sh
cargo add abstractavatar
cargo add abstractavatar --features ffi,json   # C ABI/WebAssembly exports and the descriptor
```

```rust
use abstractavatar::{AvatarPose, AvatarStyle, Frame, Gaze, Palette};

let frame = Frame::new(AvatarStyle::Halo, AvatarPose::Speaking, 0.6, 1.2, Gaze::default());
// Replace these flat ramps with your theme's 256 shades per material.
let palette: Palette = [[0xffc060ff; 256], [0xff7070ff; 256], [0xd8e0ffff; 256]];
let rgba = frame.rgba(0.2, -0.1, 2.0 / 128.0, &palette);
```

RGBA samples retain overlapping material colors. Composite the returned foreground over your
background. Build one frame per state/time combination and sample it across the image, rather
than reconstructing geometry for every pixel. See [rendering](rendering.md) for pixel footprints.

## Build for browsers

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --features ffi --target wasm32-unknown-unknown
```

The WebAssembly artifact uses the same generator. A browser adapter supplies palette values and
copies RGBA pixels into Canvas. WebAssembly execution requires no geometry interpreter written
in JavaScript. See [API](api.md), [architecture](architecture.md) and
[troubleshooting](troubleshooting.md).
