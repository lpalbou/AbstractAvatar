# Proposed: Designs as data (data-driven design descriptor)

## Metadata
- Created: 2026-11-07
- Status: Proposed
- Completed: N/A
- Area: `catalog.json`, `schema/`, `src/scenes.rs`

## ADR status
- Governing ADRs: ADR-0001 (portable procedural generation)
- ADR impact: May revise existing ADR (designs would become data interpreted by the core)

## Context
Every design is Rust code. `src/scenes.rs` holds one generator per family; a designer cannot add
or tune a design without a crate release. The frame descriptor
(`schema/descriptor.schema.json`, `src/descriptor.rs`) is a frame request (style, state, time,
level, gaze, bands), not a description of an appearance.

## Current code reality
- Scene vocabulary: strokes, dots, ellipses, paths, rounded boxes, triangles; additive or over
  compositing; three host materials (`src/scenes.rs` `Scene`, `src/raster.rs`).
- Per-state speeds and amplitudes are constants inside each generator (for example
  `State::motion`, the `thinking()`/`working()` terms).
- `catalog.json` carries identity and provenance only.

## Problem
Tuning a design, adding a variant or giving a design its own hues needs Rust changes and a
release, and the hosts cannot express a design at all.

## What we want to do
Keep the generators in Rust, but move each design's parameters into a schema-validated document:
hues, speeds, amplitudes, per-state intensity, which band drives which part. Longer term, a
small declarative layer (layers of primitives with bounded motion curves) interpreted by the core.

## Why
It turns the package into the standard procedural description its goal names, and lets the
0001 redesigns and future designs be reviewed as data.

## Promotion criteria
Promote once the visual language has settled (after 0001 and 0003), with an operator decision on
how much is data (parameters only, or full layers).

## Non-goals
No executable content, file paths or URLs in the document; the descriptor stays inert and
bounded.

## Validation
A design expressed as data renders byte-identical to its Rust generator; the schema rejects
out-of-range parameters; native and Wasm agree.
