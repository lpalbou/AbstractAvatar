# Catalog and activity

`catalog()` and the native catalog exports enumerate the supported designs. Each entry carries a
stable style key, display name, description, family, source reference and gaze component. `catalog.json` records
the design provenance, the default design and the retired designs.

The catalog preserves shaded terminal characters separately from particle designs whose names
overlap. For example, a shaded Halo and the Watch Halo remain separate choices. The `halo`,
`hover`, `vortex`, `eclipse` and `rover` keys keep their meaning; Apple-inspired variants use
their own keys.

Families include orbs, forms, cosmic scenes, machines, nature and the iPhone weave.
Platform sampling and palette can affect appearance; the shared generator supplies the geometry
and motion. The source Apple applications retain their own implementations until separately
integrated.

## Activity states

| State | Typical host condition |
|---|---|
| ready | Idle and available |
| thinking | Reasoning or preparing a response |
| working | Executing an operation |
| listening | Accepting audio/input |
| speaking | Presenting speech/output |
| waiting | Waiting for an answer or approval |
| error | An operation failed |

The application decides the state. AbstractAvatar does not infer it from a connection, model or
microphone. Audio level, optional audio bands and optional gaze add visual input to the selected
state.

Thinking and working are distinct animations in every design. As a rule, thinking is inward and
slow: forms gather or turn calmly, light travels toward the centre, eyes look up or narrow.
Working is outward and busy: forms spread, light travels away from the centre, parts spin, roll
or flare.

## Audio bands

`bands` carries eight normalized levels from low to high frequency. `watch-rings` gives each of
its six rings its own part of the spectrum: the first ring follows the lowest bands and the last
the highest, and each ring tilts, swells and brightens with its band. A ring pushes out while
speaking and draws in while listening. When a host sends only a level, each ring follows that
level through its own rhythm, so the rings still move one by one.

## Gaze component

`gaze` names the one part of a design that may follow a host-supplied gaze. `eyes` designs
(`hover`, `rover`, `watch-eye`, `watch-rover`, `watch-hover` and `watch-spark`) shift their
pupils, irises or eye lamps by a small bounded offset while ready. Every other design is `none`
and ignores gaze. See [the frame contract](frame-contract.md#gaze).

## Stable keys

Persist keys, not a display name or array index. A numeric catalog code is a position in the
installed catalog and changes when a design is retired. Query the installed catalog rather than
assuming a fixed number or ordering. See [API](api.md) and [frame contract](frame-contract.md).

## Retired designs

A retired key leaves the catalog but stays reserved, and it always renders: a stored identity
that names it draws its replacement, or the default design `halo` when it has none. Any other
unknown key also draws `halo`. `AvatarStyle::resolve` (Rust), `resolve_style` (Python) and the
`avatar_retired_*` exports (C/WebAssembly) apply this rule; `catalog.json` lists the same records
under `retired`.

| Retired key | Draws | Status |
|---|---|---|
| `eye` | `watch-eye` | Removed in 0.2.0 |
| `watch-drop` | `halo` | Removed in 0.2.0 |
| `watch-shape-kitty`, `watch-shape-face` | `halo` | Removed in 0.2.0 |
| `watch-shape-wave` | `watch-wave` | Removed in 0.2.0 |
| `watch-face`, `watch-imprint`, `watch-contour`, `watch-membrane` | `halo` | Awaiting a redesign |
| `watch-lens`, `watch-dome`, `watch-valet`, `watch-retro`, `watch-deco`, `watch-cruiser`, `watch-android`, `watch-sentinel` | `watch-hover` | Awaiting a redesign |

A design awaiting a redesign returns under its original key, so stored identities pick it up.

## Design guide

`halo`, `watch-wave`, `watch-well`, `watch-hover` and `iphone-weave` set the style for every
design:

- **One clear idea per state.** `watch-wave` is calm ribbons at rest, harmonics while voiced and
  a swirl while working; `iphone-weave` is loose cords that become a globe to listen and a raised
  hand to wait. Each state reads at a glance, even in a still frame.
- **Thinking inward, working outward.** `watch-well` draws its two masses close and circles them
  slowly to think, and swings them wider and faster to work; `halo` turns its crown and sweeps
  its highlight to think, and spins its orbit faster to work. Contemplation contracts and slows;
  activity spreads and speeds up.
- **Listening takes in, speaking gives out.** Intake travels toward the centre and speech travels
  away from it (`halo`'s streamers, `watch-wave`'s incoming and outgoing wavefronts).
- **The voice moves a part, not the whole.** Ribbons, rings or eyes answer the level and bands;
  the body stays recognisable.
- **Recognisable characters.** `watch-hover` is a believable object: an opaque shell, a visor,
  eyes that glow with its voice, arms that unfold to work. A face or machine must be recognisable
  as what it portrays; an abstract form must keep one silhouette across every state.
- **Restraint.** Few primitives, soft light, slow rest motion. A design never flickers and never
  goes blank.
