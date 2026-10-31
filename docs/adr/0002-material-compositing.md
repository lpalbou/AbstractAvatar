# ADR 0002 — Shared material compositing

Status: Accepted. Date: 2026-10-31.

## Context

The material sampler returns one material, light and coverage value. That interface is useful
for terminal-oriented consumers, but cannot represent simultaneous colored light or translucent
surfaces. Built-in Apple-derived scenes include layered ribbons, shaded shells and soft light.

## Decision

Add `Frame::rgba` and the `Palette` alias without removing `Frame::sample`. Native and WebAssembly
render through the RGBA sampler. Color is premultiplied internally and straight RGBA8 at the
boundary. Light composes additively; shaded surfaces compose in paint order. The host still owns
all three material ramps and background color.

Keep the primitive and pixel-footprint budgets explicit. Pixel integration and analytic filled
edges run inside the generator; presenters do not implement private procedural interpreters.
ABI version 1, descriptor version 1 and all style identities remain compatible.

## Alternatives

| Option | Benefit | Cost |
|---|---|---|
| Refine geometry while retaining one winning material | Small implementation change | Loses color layers and surface composition |
| Publish a general scene/shader language | Arbitrary platform rendering | New execution, resource and compatibility contract |
| Shared composited RGBA sampling over host ramps | One bounded algorithm and palette owner | Additional raster work; hosts adopt the RGBA sampler |

## Boundaries and follow-up

This decision concerns raster fidelity. It does not define a stateful animation controller or
extend activity into emotion/personality. A future transition contract must specify continuous
phase weights, monotonic elapsed time, bounded time steps, measured audio attack/release and
band smoothing, deterministic history/reset semantics, hidden-view resume and reduced motion.
It must also distinguish host-observed activity from optional expression input, with explicit
validation and a neutral fallback. Those inputs require their own versioned design and review.

This repository has no local backlog. Cross-package implementation and validation are tracked
by AbstractFramework [1081](https://github.com/lpalbou/AbstractFramework/blob/main/docs/backlog/completed/1081_avatar_rendering_fidelity_and_platform_quality.md).
