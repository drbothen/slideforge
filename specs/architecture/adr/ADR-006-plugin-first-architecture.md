---
document_type: adr
adr_id: ADR-006
title: Plugin-first architecture with 10 extensibility surfaces
status: accepted
date: 2026-05-24
spike_input: ~
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-006: Plugin-First Architecture with 10 Extensibility Surfaces

## Context

slideforge must be extensible for future output formats, data sources, chart types, and
diagram engines. The Python reference implementation had tight coupling between formats
and rendering logic, making new format support difficult. Q3 decision locked the plugin-first
approach as the architectural foundation.

## Decision

All functionality flows through 10 plugin traits in `slideforge-plugin-api`. Bundled
plugins and external plugins use the same API (DI-008). The plugin registry is assembled
once at startup (`slideforge/src/registry.rs`).

## Consequences

**Dog-fooding guarantee:** If any bundled plugin needs to bypass the trait API, the API
is wrong and must be fixed — not the plugin. This constraint is tested by the CI check
that bundled plugin crates may only import from `slideforge-plugin-api` and `slideforge-types`.

**v1.0 (static):** All 10 surfaces are statically compiled into the single binary.
Plugin architecture is invisible to v1.0 users.

**v2+ (dynamic):** External plugins loaded via WASI modules. Same trait API — the v1.0
design decision determines what the external plugin API will look like in v2.

**10 trait surfaces (locked, q3-decision-final.md):**
DataSource, Exporter, ChartRenderer, DiagramRenderer, Validator, MathRenderer,
BrandProvider, SlideType, SectionType, InlineFormat.

**Registry lookup:** by `id()` string. Duplicate IDs at registration time are a panic
(programming error, not user error). Registry is `Send + Sync`; all trait objects are
`Box<dyn Trait + Send + Sync>`.

**Testing:** Each bundled plugin has its own unit tests. The plugin API itself is tested
by the fact that every bundled feature goes through it — the bundled plugins ARE the
integration test suite for the API.
