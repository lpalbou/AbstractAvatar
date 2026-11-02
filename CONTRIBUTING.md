# Contributing

Use Rust 1.87+ and Python 3.10+. Build the library before trying its native integrations:

```sh
cargo test
cargo test --all-features
cargo build --release --features ffi
python -m pip install .
```

Keep procedural geometry in the shared Rust implementation. UI-specific clocks, preferences,
colors, focus and input capture belong in the host. New styles need distinct visual behavior,
source attribution, valid catalog metadata and examples of all supported activity states.

Preserve existing style keys and the published ABI contract. Add rather than repurpose catalog
entries. Changes to ownership or frame semantics belong in [architecture decisions](docs/adr/0001-portable-procedural-generation.md).

Document public API changes in [the API reference](docs/api.md) and [changelog](CHANGELOG.md).
Keep documentation indexes current with `python scripts/docs.py`; `python scripts/docs.py --check`
checks freshness without writing files.

For raster changes, compare native and WebAssembly output with identical ramps and inputs:

```sh
rustup target add wasm32-unknown-unknown
python scripts/build_artifacts.py
python scripts/check_portability.py --native artifacts/libabstractavatar.so
```

The comparison uses Node.js and a built native library. On macOS use
`artifacts/libabstractavatar.dylib`; on Windows use `artifacts/abstractavatar.dll`.
It exercises every style/state at 64 pixels plus the focused particle, ribbon and shaded-surface
designs at 32 and 128 pixels, including partial material alpha and eight voice bands.
Visual review still matters: pixel agreement establishes portability, not design fidelity.

## Releases

A release publishes one version on two registries:

- **crates.io** (`abstractavatar` crate): published by the maintainer from a local checkout with
  `cargo publish`. The release workflow does not publish the crate.
- **PyPI** (`abstractavatar` distribution): published by
  [`.github/workflows/release.yml`](.github/workflows/release.yml), which builds an sdist and
  wheels for Linux x86_64/aarch64 (manylinux), macOS arm64/x86_64 and Windows x86_64.

The version in `Cargo.toml`, `pyproject.toml` and `python/abstractavatar/__init__.py` must be
equal and `CHANGELOG.md` must have a dated `## [X.Y.Z] - YYYY-MM-DD` section; the workflow
refuses to publish otherwise. It also refuses a version already on PyPI or not yet on crates.io.
Release in this order:

1. Bump the three version sources, fold `Unreleased` into the dated changelog section, commit and
   push to `main`, and wait for CI to pass.
2. Publish the crate: `cargo publish --locked`.
3. Push the annotated tag: `git tag -a vX.Y.Z -m "abstractavatar X.Y.Z" && git push origin vX.Y.Z`.
   The tag push builds, publishes to PyPI and creates the GitHub Release.

You can also start the PyPI lane from `main` by hand. Without `publish=true` the run only builds
and tests the distributions:

```sh
gh workflow run release.yml --ref main -f version=X.Y.Z -f publish=true \
  -f publish_confirmation=publish-abstractavatar-X.Y.Z
```

A dispatched run builds the commit of `vX.Y.Z` when that tag exists; otherwise it builds the
`main` commit and creates the tag once the build passes.

PyPI publishing uses trusted publishing, with no stored token. The PyPI project needs a trusted
publisher with project name `abstractavatar`, owner `lpalbou`, repository `AbstractAvatar`,
workflow file `release.yml` and environment `pypi`.

Report security concerns using [SECURITY.md](SECURITY.md).
