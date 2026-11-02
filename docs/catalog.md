# Catalog and activity

`catalog()` and the native catalog exports enumerate the supported designs. Each entry carries a
stable style key, display name, description, family, source reference and gaze component. `catalog.json` records
the design provenance.

The catalog preserves shaded terminal characters separately from particle designs whose names
overlap. For example, a shaded Halo and the Watch Halo remain separate choices. Existing
`halo`, `hover`, `vortex`, `eclipse`, `eye` and `rover` keys keep their meaning; Apple-inspired
variants use their own keys.

Families include orbs, forms, cosmic scenes, portraits, machines, nature and the iPhone weave.
Additional point-body shapes are distinct from the similarly named portrait/form scenes.
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
microphone. Audio level and optional gaze add visual input to the selected state.

## Gaze component

`gaze` names the one part of a design that may follow a host-supplied gaze. `eyes` designs
(`hover`, `eye`, `rover`, `watch-face`, `watch-eye`, `watch-lens`, `watch-dome`, `watch-valet`,
`watch-retro`, `watch-deco`, `watch-rover`, `watch-hover`, `watch-spark` and `watch-android`)
shift their pupils, irises or eye lamps by a small bounded offset while ready. Every other
design is `none` and ignores gaze. See [the frame contract](frame-contract.md#gaze).

Persist keys, not a display name or array index. Query the installed catalog rather than assuming
a fixed number or ordering beyond the documented compatibility prefix. See [API](api.md) and
[frame contract](frame-contract.md).
