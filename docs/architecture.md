# Architecture

AbstractAvatar owns the catalog and deterministic procedural generation. Hosts own activity
signals, preferences, palette, scheduling and presentation.

```mermaid
flowchart LR
    I[Host input: style key, state, time, audio level/bands, gaze] --> G[abstractavatar crate: catalog, scenes, rasterizer]
    P[Host palette: Warm/Eye/Dim ramps] --> G
    G -->|Frame::rgba| RS[Rust presenters, e.g. terminal frame player]
    G -->|ffi feature| A[C ABI avatar.h: bounded handles, RGBA buffers]
    A --> PY[Python ctypes binding and PNG preview command]
    A --> N[Native image presenter]
    A -->|wasm32-unknown-unknown| W[WebAssembly module]
    W --> C[Browser Canvas presenter]
    H[Host clock, visibility and reduced-motion preference] -.-> RS
    H -.-> N
    H -.-> C
```

The [descriptor](frame-contract.md) selects built-in generation; it does not introduce a new
programming language. Native and WebAssembly builds run the same algorithms. Platform-specific
presenters translate frames into terminal cells, a canvas image or a native image object.

Shared rasterization composes shaded surfaces and material light before returning RGBA. Built-in
scene primitives and spatial indexes are bounded; presenters do not recreate the geometry or its
color blending. Rust presenters can use `Frame::rgba` with the same palette ramps as the ABI.
See [rendering](rendering.md).

Core generation has no network, microphone, model, timer, global mouse listener or UI-framework
dependency. Gaze and audio levels are inputs supplied by the host. Keeping those responsibilities
explicit lets multiple applications present the same design without sharing application state.

Rendered frames use standard RGBA channel ordering. PNG is used for portable still-image
previews. WebAssembly is the browser execution format; it is not an avatar scene-file format.
The supported formats and limits are specified in [the frame contract](frame-contract.md).

Applications persist stable style keys and resolve numeric ABI codes from the runtime catalog.
See [catalog](catalog.md) and [ADR 0001](adr/0001-portable-procedural-generation.md).
