---
document_type: adr
adr_id: ADR-016
title: Plugin registry Builder + surface ownership + root-crate pipeline driver
status: accepted
date: 2026-06-04
spike_input: ~
traces_to: ARCH-INDEX.md
supersedes: ~
context: LESSON-13 reconciliation for STORY-049 (Plugin Registry Assembly); three human-approved decisions recorded 2026-06-04.
---

# ADR-016: Plugin Registry Builder, Surface Ownership, and Root-Crate Pipeline Driver

## Context

STORY-049 reconciliation (LESSON-13) exposed three architecture contradictions that required
human decision:

1. The delivered `PluginRegistry` (STORY-002) had no Builder, no fail-on-empty semantics,
   and no introspection methods — violating BC-5.02.001 invariant 3 ("unregistered surfaces
   cause an initialization error, not a silent no-op"). A human decision was required on
   whether to add the Builder to `slideforge-plugin-api` or weaken the BC.

2. `plugin-architecture.md` listed `slideforge-types` as the owner crate for `SectionType`
   and `InlineFormat` bundled implementations. This is architecturally impossible: the
   only way to implement a plugin trait is to depend on `slideforge-plugin-api`, but
   `slideforge-plugin-api` already depends on `slideforge-types` for IR types (`Slide`,
   `InlineNode`, etc.). A `slideforge-types → slideforge-plugin-api` edge would create a
   dependency cycle. A human decision was required on the correct owner crate.

3. The root `slideforge` crate's role was described inconsistently across architecture
   documents: "thin re-export facade" and "No code lives in the root crate beyond registry
   construction" (ARCH-INDEX) vs. "pipeline orchestration" (system-overview.md, purity
   table) vs. the existing `Cargo.toml` already depending on `slideforge-syntax`,
   `slideforge-eval`, `slideforge-layout`, and `slideforge-validate`. A human decision
   was required on whether `build()` belongs in the root crate or in `slideforge-cli`.

## Decisions

### Decision 1: PluginRegistryBuilder in slideforge-plugin-api (STORY-083)

The `PluginRegistryBuilder`, `RegistryError`, and introspection methods belong in
`slideforge-plugin-api`, not in the root `slideforge` crate. The root crate is the
assembler (caller), not the registry-extension owner.

Specifically, `slideforge-plugin-api/src/registry.rs` gains:

- `PluginRegistryBuilder` — builder with per-surface `register_*` methods returning `&mut Self`
- `RegistryError::MissingSurface { surface: &'static str }` — typed error returned (not panicked)
  when `PluginRegistryBuilder::build()` is called with zero registrations for any of the
  10 required surfaces
- `PluginRegistry::surface_count() -> usize` — runtime auditability
- `PluginRegistry::surface_names() -> Vec<&'static str>` — runtime auditability

This closes the BC-5.02.001 invariant 3 gap in STORY-002's delivered code.
Prerequisite work is assigned to STORY-083.

### Decision 2: SectionType and InlineFormat bundled impls live in slideforge-plugin-api

`slideforge-plugin-api` is the correct owner crate for bundled `SectionType` and
`InlineFormat` implementations. This mirrors the existing pattern for `SlideType`:
the 31 built-in SlideType implementations already live in
`slideforge-plugin-api/src/slide_types/` (not in `slideforge-types`).

Bundled implementation locations:

- `slideforge-plugin-api/src/slide_types/` — 31 SlideType impls (already delivered)
- `slideforge-plugin-api/src/section_types/` — ExecutiveSummarySectionType,
  RiskRegisterSectionType, MethodologySectionType, ScopeSectionType,
  ApprovalSectionType, AppendixSectionType, GlossarySectionType (to be delivered
  by STORY-084; canonical set per CANONICAL_MANUAL_SECTION_TYPES in deck.rs)
- `slideforge-plugin-api/src/inline_formats/` — DefaultInlineFormat covering all
  12 InlineNode variants × 3 output formats (OOXML, HTML, Markdown) (to be delivered
  by STORY-085)

The previous documentation listing `slideforge-types` as owner for rows 9 and 10 of
the 10-surface table is CORRECTED by this ADR. `slideforge-types` remains a leaf crate
with zero workspace dependencies — it provides IR type definitions only.

### Decision 3: Root crate is the pipeline driver, not an assembly-only facade

The root `slideforge` crate exposes the public library API:

```rust
pub fn build(source: &str, options: BuildOptions) -> Result<BuildOutput, BuildError>
```

This function wires the parse → eval → validate → layout → export pipeline using
the assembled `PluginRegistry`. `slideforge-cli` depends on the root crate and
calls `slideforge::build()` — the CLI's job is transforming user input (paths, flags)
into `BuildOptions`, not implementing the pipeline.

The root crate's dependency on `slideforge-syntax`, `slideforge-eval`,
`slideforge-layout`, and `slideforge-validate` is correct and intentional.

The constraint is: **no plugin logic lives in the root crate**. All cross-crate
plugin interaction goes through `Box<dyn Trait>` dispatch — no internal-function
calls across crate boundaries. The root crate MUST NOT call e.g.
`PptxExporter::internal_method()` directly; it calls `Box<dyn Exporter>::export()`.

The previous documentation phrase "No code lives in the root crate beyond registry
construction" is SUPERSEDED. The correct formulation: "No plugin logic lives in
the root crate; pipeline wiring via `build()` is permitted; all cross-crate plugin
interaction goes through `Box<dyn Trait>` dispatch."

## Consequences

**BC-5.02.001 invariant 3:** Closed by Decision 1. The initialization-time error
for unregistered surfaces is enforced at `PluginRegistryBuilder::build()`.

**Dependency graph:** No new dependency edges introduced. `slideforge-plugin-api`
already depends on `slideforge-types`. The root `slideforge` crate already listed
`slideforge-syntax`, `slideforge-eval`, `slideforge-layout`, `slideforge-validate`
in its `Cargo.toml` — this ADR confirms those deps are correct, not erroneous.

**SS-14 scope expansion:** `slideforge-plugin-api` now formally owns bundled
implementations for SlideType (existing), SectionType (STORY-084), and InlineFormat
(STORY-085), in addition to the trait definitions. This is consistent with the
dog-fooding principle: trait definitions and their primary bundled implementations
are co-located to keep the plugin API honest.

**SS-15 scope correction:** `slideforge-types` is IR-types-only. It does not own
any plugin trait implementations.

**Prerequisite stories (Wave 4 Batch C):**
- STORY-083: Delivers `PluginRegistryBuilder`, `RegistryError::MissingSurface`,
  `surface_count()`, `surface_names()` (registry builder + enforcement only).
- STORY-084: Delivers bundled `SectionType` implementations (7 types per
  CANONICAL_MANUAL_SECTION_TYPES + auto-generated types from BC-3.02.001).
- STORY-085: Delivers `DefaultInlineFormat` (12 variants × 3 formats) + refactors
  `slideforge-pptx` inline serialization to route through the plugin trait.
STORY-049 (Plugin Registry Assembly) depends on all three being merged.
