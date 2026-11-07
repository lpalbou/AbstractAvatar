# Planned: Listening differs from speaking in every design

## Metadata
- Created: 2026-11-07
- Status: Planned
- Completed: N/A
- Area: `src/scenes.rs`, `src/lib.rs`

## ADR status
- Governing ADRs: None
- ADR impact: None

## Context
On the c11a15c catalog, listening and speaking rendered the same picture (mean pixel difference
0.0 at the same time and level, measured by the 2026-11-07 review probe) in 15 designs:
watch-nebula, watch-rings, watch-liquid, watch-galaxy, watch-eclipse, watch-black-hole,
watch-accretion, watch-aurora, watch-murmuration, watch-lens, watch-cruiser, watch-ocean and the
three shape silhouettes.

## Current code reality
- 0.2.0 retired watch-lens, watch-cruiser and the three silhouettes, and `watch-rings` now
  pushes its rings out while speaking and draws them in while listening (`rings()`,
  `ring_drive()` in `src/scenes.rs`). Remaining: watch-nebula, watch-liquid, watch-galaxy,
  watch-eclipse, watch-black-hole, watch-accretion, watch-aurora, watch-murmuration, watch-ocean
  (9 designs). Re-measure before starting.
- Root cause: `State::voice()` and `State::band()` fold both states into one term,
  `level * (listening + speaking)` (`src/scenes.rs`, `impl State`).

## Problem
A user cannot tell whether the assistant is hearing them or talking.

## What we want to do
Listening takes in (intake travels toward the centre, the form gathers or opens to receive);
speaking gives out (waves travel away, the form pulses with syllables). Both still answer the
level. See the [design guide](../../catalog.md#design-guide).

## Requirements
- Every active design: listening and speaking differ visibly at the same time and level, and
  the difference reads in a still frame.
- Both keep answering the measured level (`every_voice_pose_responds_to_measured_level`).
- Add a regression test like `thinking_and_working_are_different_animations` for this pair.

## Scope
The 9 designs above, plus any design the new test flags.

## Non-goals
No change to the descriptor or ABI.

## Dependencies and related tasks
0001 (the redesigned faces must meet the same rule).

## Expected outcomes
No design renders listening as speaking.

## Validation
The new Rust test; the review page in both states; `cargo test --locked --all-features`,
`python -m pytest -q tests`.

## Progress checklist
- [x] watch-rings (0.2.0)
- [ ] the 9 remaining designs
- [ ] regression test
