# AbstractAvatar backlog overview

AbstractAvatar is the shared procedural avatar core (Rust crate, C/WebAssembly ABI, Python
binding). This backlog tracks the package's own planned and proposed work. It is maintainer
material and is not part of the published documentation index.

Seeded 2026-11-07 from the operator's review of the 44-design catalog (0.2.0 retires 17 designs).

## Counts
- Planned: 2 (0001, 0003)
- Proposed: 1 (0002)
- Completed: 0
- Deprecated: 0
- Recurrent: 0

## Priority bands (next recommended work)
1. **0003** listening differs from speaking in the 9 remaining designs (small, visible).
2. **0001** redesign the 12 retired watch faces faithfully from the Swift originals, one design
   at a time, recognisability judged by the operator.
3. **0002** designs as data, once the visual language has settled.

## Planned
| ID | Item | Summary |
|---|---|---|
| 0001 | [Redesign the retired watch faces faithfully](planned/0001_redesign_retired_watch_faces.md) | 12 keys (HAL 9000, R2-D2, Metropolis, KITT, Cylon, the faces, …) return under their keys once recognisable |
| 0003 | [Listening differs from speaking](planned/0003_listening_differs_from_speaking.md) | 15 designs drew both states alike; rings fixed in 0.2.0, 9 remain |

## Proposed
| ID | Item | Promotion criterion |
|---|---|---|
| 0002 | [Designs as data](proposed/0002_data_driven_design_descriptor.md) | after 0001/0003, with an operator decision on parameters vs layers |

## Completed
None yet.

## Deprecated
None.

## Process
- New item: next unused global number, `NNNN_slug.md`, in `planned/` (committed) or `proposed/`.
- Completing: move the file to `completed/`, add a completion report (date, PR, evidence), update
  the counts and ledgers here in the same change.
- Deprecating: move to `deprecated/` with the reason.
- The design rules live in [the design guide](../catalog.md#design-guide); items cite it rather
  than restating it.
