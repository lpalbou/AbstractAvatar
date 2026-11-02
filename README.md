# AbstractAvatar

[![crates.io](https://img.shields.io/crates/v/abstractavatar.svg)](https://crates.io/crates/abstractavatar)
[![docs.rs](https://img.shields.io/docsrs/abstractavatar)](https://docs.rs/abstractavatar)
[![CI](https://github.com/lpalbou/AbstractAvatar/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/lpalbou/AbstractAvatar/actions/workflows/ci.yml)

AbstractAvatar generates procedural avatars from explicit activity state, time, audio level and
gaze. One Rust implementation supplies terminal, browser and native applications with a common
catalog and rendering behavior.

Use it for an assistant portrait, an activity indicator or a procedural character that responds
to your application's state. The host supplies state and presentation; AbstractAvatar generates
the picture locally.

![Procedural avatar catalog](docs/media/catalog.png)

**What's new in 0.1.0:** the first release of the `abstractavatar` crate. See the
[changelog](CHANGELOG.md#010---2026-11-02).

## Start with Rust

Add the crate from crates.io (Rust 1.87+):

```sh
cargo add abstractavatar
```

```rust
use abstractavatar::{AvatarPose, AvatarStyle, Frame, Gaze, Palette};

let frame = Frame::new(AvatarStyle::Halo, AvatarPose::Speaking, 0.6, 1.2, Gaze::default());
let palette: Palette = [[0xffc060ff; 256], [0xff7070ff; 256], [0xd8e0ffff; 256]];
let rgba = frame.rgba(0.2, -0.1, 2.0 / 128.0, &palette);
```

Enable the `ffi` feature for the C ABI and WebAssembly exports, and `json` for the versioned
parameter descriptor.

## Start with Python

Build from the repository with Python 3.10+ and Rust 1.87+:

```sh
git clone https://github.com/lpalbou/AbstractAvatar.git
cd AbstractAvatar
python -m pip install .
python -m abstractavatar catalog
python -m abstractavatar render --style halo --state speaking --level 0.6 --size 128 --output avatar.png
```

The Python distribution includes the compiled native library. A source installation builds it
with Cargo. See [getting started](docs/getting-started.md) for library use and platform builds.

## Integrate a player

- **Rust:** create a `Frame` and sample composited RGBA through your theme's material ramps.
- **Browser:** run the bundled WebAssembly generator and present RGBA pixels on Canvas.
- **Python/native:** request RGBA pixels through the binding or the C interface and present them
  with your UI toolkit.

Store catalog style identifiers, rather than numeric indices. Supply time explicitly, stop
animation while hidden, and use a static frame when reduced motion is enabled. Platform adapters
do not need their own geometry implementations.

The catalog includes shaded characters, particle orbs, forms, portraits, machines, nature scenes
and a woven orb. [Catalog guidance](docs/catalog.md) explains source variants and stable keys.

## Documentation

[Documentation index](docs/README.md) · [Architecture](docs/architecture.md) ·
[API](docs/api.md) · [Frame contract](docs/frame-contract.md) · [FAQ](docs/faq.md) ·
[Troubleshooting](docs/troubleshooting.md) · [Contributing](CONTRIBUTING.md) ·
[Security](SECURITY.md) · [Changelog](CHANGELOG.md)

## For AI agents

Use the [AbstractAvatar skill](skills/abstractavatar/SKILL.md) for the rendering workflow,
descriptor constraints and platform integration rules. The [LLM index](llms.txt) maps the public
documentation.
