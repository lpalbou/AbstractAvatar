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

Report security concerns using [SECURITY.md](SECURITY.md).
