# ADR 0001 — Shared generation and platform presentation

Status: Accepted. Date: 2026-10-31.

## Context

Terminal, web and native applications need the same procedural designs under explicit activity
states. Separate implementations duplicate geometry, while a mandatory avatar dependency couples
a general UI engine to a domain-specific catalog. Existing variants and persisted style keys
must retain their meaning.

## Decision

AbstractAvatar owns deterministic geometry, catalog metadata and bounded state/time/audio/gaze
inputs. Native/C and WebAssembly builds execute that same generation. Platform presenters own
timing, visibility, palette, preferences and input capture, and display standard RGBA frames.
PNG is the still-preview format. A versioned JSON descriptor selects trusted built-in generation;
it is not an executable scene language or remote plug-in.

AbstractTUI provides a generic frame source/player independently of AbstractAvatar. Avatar
convenience support is opt-in in the existing crate. No additional terminal adapter package is
introduced. AbstractUIC and native applications consume the shared generator through adapters.
Watch and iPhone integration is outside this change; their source is a read-only design reference.

## Alternatives

| Option | Strength | Cost |
|---|---|---|
| Separate per-platform procedural implementations | Direct access to each toolkit | Geometry and catalog drift; repeated maintenance |
| General scene DSL with an interpreter per platform | Authorable neutral scene description | New language, resource model and compatibility surface |
| Shared native/WebAssembly generation and thin frame presenters | One algorithm owner; bounded portable output | Native/Wasm packaging and sampling differences |

Choose shared generation. Native bindings copy/synchronize frame access; browser modules receive
no ambient host imports. Descriptors cannot select libraries, URLs or arbitrary code. Stable style
keys are persisted; numeric codes are resolved from the runtime catalog.

## Consequences

Native and browser build artifacts require provenance and API compatibility checks. Hosts can use
the engine without sharing application state, and general UI consumers can omit avatar support.
Geometry consistency is tested separately from fidelity to the source artwork and host lifecycle.
