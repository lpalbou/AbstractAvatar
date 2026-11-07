# Planned: Redesign the retired watch faces faithfully

## Metadata
- Created: 2026-11-07
- Status: Planned
- Completed: N/A
- Area: catalog, `src/scenes.rs`

## ADR status
- Governing ADRs: ADR-0001 (portable procedural generation), ADR-0002 (material compositing)
- ADR impact: None

## Context
The operator reviewed the 44-design catalog on 2026-11-07 and judged the faces and character
machines "terrible": the Rust ports lost what made the Apple Watch originals recognisable
("apple watch is better overall in their designs"). Version 0.2.0 retires these 12 keys
(`RETIRED` in `src/catalog.rs`, `retired` in `catalog.json`). A stored identity that names one
draws an interim replacement until the redesign lands.

## Current code reality
- The retired ports were deleted from `src/scenes.rs` in 0.2.0 (`portrait()`, `face_relief()`
  and the Lens, Dome, Cruiser and generic Valet/Retro/Deco/Android/Sentinel branches of
  `machine()`); git history before 0.2.0 keeps them, but they are not the reference.
- The reference is the Swift originals, read-only, in
  `abstractassistant/watch/apple/Sources/App/Shared/`: `OrbScenes.swift` (`drawFace`),
  `OrbAvatars.swift` (`drawImprint`), `OrbLines.swift` (`drawContour`, `drawMembrane`),
  `OrbRobots.swift` (`drawLens`, `drawDome`, `drawValet`, `drawRetro`, `drawDeco`),
  `OrbMachines.swift` (`drawCruiser`, `drawAndroid`, `drawSentinel`).
- Interim replacements: the four faces draw `halo`; the eight machines draw `watch-hover`.
- The scene vocabulary is strokes, dots, ellipses, rounded boxes and triangles with additive or
  over compositing, three host materials (`src/scenes.rs` `Scene`, `src/raster.rs`).

## Problem
The faces did not read as faces or as the characters they were meant to be.

## What we want to do
Port each original faithfully, one design at a time, and return it under its original key.

| Key | Intended reference | Swift source |
|---|---|---|
| `watch-face` | a friendly human face whose lips follow its voice; it cups its ear to listen | `OrbScenes.swift` `drawFace` |
| `watch-imprint` | a face seen only where it presses into a pane of glass (pin-art) | `OrbAvatars.swift` `drawImprint` |
| `watch-contour` | a face raised in engraved lines of light | `OrbLines.swift` `drawContour` |
| `watch-membrane` | a face pressing through a fabric of light | `OrbLines.swift` `drawMembrane` |
| `watch-lens` | HAL 9000 (2001: A Space Odyssey): the red camera eye | `OrbRobots.swift` `drawLens` |
| `watch-dome` | R2-D2 (Star Wars): the dome-headed droid | `OrbRobots.swift` `drawDome` |
| `watch-valet` | the fussy gilded valet (C-3PO, Star Wars; inferred from the description, confirm) | `OrbRobots.swift` `drawValet` |
| `watch-retro` | the gentle fifties giant (Robby the Robot, Forbidden Planet; inferred, confirm) | `OrbRobots.swift` `drawRetro` |
| `watch-deco` | the Maschinenmensch of Metropolis (1927) | `OrbRobots.swift` `drawDeco` |
| `watch-cruiser` | KITT, the Knight Rider car: black car, red scanner | `OrbMachines.swift` `drawCruiser` |
| `watch-android` | the pale gold android with amber eyes (Data, Star Trek TNG; inferred, confirm) | `OrbMachines.swift` `drawAndroid` |
| `watch-sentinel` | a Cylon centurion (Battlestar Galactica): chrome helmet, sweeping red eye | `OrbMachines.swift` `drawSentinel` |

## Requirements
- **Recognisable.** A viewer who knows the reference names it from a 128 px still, without the
  label. A face reads as a face. This is the acceptance bar, judged by the operator.
- Faithful to the Swift original: proportions, silhouette, signature details and each state's
  motion; deviate only where the material model forces it, and say where.
- Follow the [design guide](../../catalog.md#design-guide): thinking inward and slow, working
  outward and busy, listening takes in, speaking gives out, the voice moves a part (lips, lamps,
  scanner) rather than the whole.
- Every state distinct; `thinking_and_working_are_different_animations` and the Python
  five-state test pass; `ready` ≠ `speaking` visibly (the old machines failed this).
- `gaze: "eyes"` only where the design draws eyes that can look (lens, dome, valet, retro, deco,
  android, face); the gaze tests decide.
- Return under the original key: remove the key from `RETIRED` and `catalog.json` `retired`,
  append it to the active catalog, update the changelog.

## Scope
The 12 keys above, one PR per design or per small family, each with before/after screenshots
at desktop and 400 px.

## Non-goals
- No new keys for these characters; no renaming.
- No change to the material model (see 0002 for appearance as data).

## Dependencies and related tasks
- 0002 data-driven descriptor (if it lands first, port into data instead of Rust).
- 0003 listening differs from speaking (apply the same rule here).

## Expected outcomes
Each redesigned key is back in the catalog and recognisable; stored identities pick it up.

## Validation
- Operator review of the review page (desktop and 400 px), against the watch screenshot of the
  same state.
- `cargo test --locked --all-features`, `python -m pytest -q tests`, `scripts/docs.py --check`.

## Progress checklist
- [ ] Faces: `watch-face`, `watch-imprint`, `watch-contour`, `watch-membrane`
- [ ] `watch-lens` (HAL 9000)
- [ ] `watch-dome` (R2-D2)
- [ ] `watch-deco` (Metropolis)
- [ ] `watch-cruiser` (KITT)
- [ ] `watch-sentinel` (Cylon)
- [ ] `watch-valet`, `watch-retro`, `watch-android` (confirm references first)

## Guidance for the implementing agent
Render the Swift original on the watch simulator or device first and keep its screenshots beside
yours; port from that, not from the deleted Rust code.
