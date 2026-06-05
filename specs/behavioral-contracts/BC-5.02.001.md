---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-021
lifecycle_status: active
introduced: v1.0.0
modified:
  - version: "1.3"
    date: 2026-06-04
    author: product-owner
    reason: "M-1 staleness fix (STORY-049 audit): correct SectionType count from ~15 to 7 bundled implementations (per CANONICAL_MANUAL_SECTION_TYPES in deck.rs) and InlineFormat count from 11 to 12 variants (Plain, Bold, Italic, Code, Link, Math, Footnote, Xref, Superscript, Subscript, Strikethrough, Highlight)."
  - version: "1.2"
    date: 2026-06-04
    author: product-owner
    reason: "LESSON-13 reconciliation (STORY-049): Strengthen Invariant 3 — make registry enforcement unambiguous: specifies all 10 surfaces are required, enforcement point is registry finalization (PluginRegistryBuilder::build()), and the error is typed RegistryError::MissingSurface { surface } not a panic (Decision 1 of 3 approved 2026-06-04)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-5.02.001: All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API

## Description

The `slideforge-plugin-api` crate defines 10 `dyn Trait` extensibility surfaces. Each
surface must have at least one bundled plugin implementation that compiles and passes
tests using only the public trait API. This verifies that the trait API is complete,
usable, and not a leaky abstraction. The 10 surfaces are: DataSource, Exporter,
ChartRenderer, DiagramRenderer, Validator, MathRenderer, BrandProvider, SlideType,
SectionType, InlineFormat.

## Preconditions

1. `slideforge-plugin-api` crate is compiled and exports all 10 trait definitions.
2. Bundled plugin crates exist for all 10 surfaces.
3. CI is running the full workspace build.

## Postconditions

1. All 10 plugin trait surfaces compile without error.
2. At least one bundled plugin exists for each of the 10 surfaces:
   - DataSource: JSON, CSV, YAML, TOML, HTTP file-data-source plugins
   - Exporter: PPTX, DOCX, PDF, HTML exporters
   - ChartRenderer: plotters-backed chart renderer
   - DiagramRenderer: mermaid-rs-renderer (Spike S14 resolution)
   - Validator: accessibility validator, overflow validator
   - MathRenderer: pulldown-latex + KaTeX math renderer
   - BrandProvider: file-based brand provider
   - SlideType: all 31 slide type implementations
   - SectionType: 7 bundled implementations (executive_summary, risk_register, methodology, scope, approval, appendix, glossary — per CANONICAL_MANUAL_SECTION_TYPES in deck.rs)
   - InlineFormat: all 12 InlineNode variants (Plain, Bold, Italic, Code, Link, Math, Footnote, Xref, Superscript, Subscript, Strikethrough, Highlight)
3. Each bundled plugin implementation compiles using only the public API exposed by
   `slideforge-plugin-api` — no direct imports of non-plugin-api crate internals.
4. `cargo test --workspace` passes for all plugin implementations.

## Invariants

1. The plugin API surface count is exactly 10. Adding an 11th surface requires updating
   this BC and BC-INDEX.
2. The trait definitions in `slideforge-plugin-api` are the contracts — no alternate
   unstable API exists.
3. **Registry finalization enforces all-10-surfaces coverage with a typed error.**
   All 10 surfaces (DataSource, Exporter, ChartRenderer, DiagramRenderer, Validator,
   MathRenderer, BrandProvider, SlideType, SectionType, InlineFormat) are "required"
   surfaces. The enforcement point is `PluginRegistryBuilder::build()` — the call that
   converts a builder-in-progress into a usable `PluginRegistry`. If any required surface
   has zero registered implementations at that point, `build()` MUST return
   `Err(RegistryError::MissingSurface { surface: &'static str })` naming the first
   unregistered surface. A silent no-op (returning an empty-surface registry without error)
   is a contract violation. A panic is a contract violation. The error MUST be a typed
   `RegistryError` variant so callers can match on it programmatically.
   Precondition on builder: zero or more `register_*` calls may precede `build()`.
   Postcondition on `build()`: returns `Ok(PluginRegistry)` only when all 10 surfaces have
   at least one registration; returns `Err(RegistryError::MissingSurface { surface })` otherwise.
   Test: `PluginRegistryBuilder::default().build()` (no registrations) MUST return
   `Err(RegistryError::MissingSurface { .. })`.
   Test: `PluginRegistryBuilder` with all 10 surfaces registered MUST return `Ok(..)`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | New slide type added (32nd type) | New SlideType trait implementation added; total remains covered by this BC |
| EC-002 | Plugin API trait method signature change | All implementations updated; compile error otherwise — CI enforces |
| EC-003 | Plugin registered but panics during execution | Panic is caught at plugin dispatch boundary; returned as E-EXP-NNN error |
| EC-004 | Two bundled plugins registered for same surface | Both registered; caller selects by name/priority (configurable) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `cargo build --workspace` | 10/10 plugin surfaces compile; 0 trait method errors | happy-path |
| `cargo test --workspace` | All plugin unit tests pass | happy-path |
| Minimal test plugin implementing `DataSource` without slideforge-internal imports | Compiles successfully; `cargo test` passes | happy-path (API completeness) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 10 trait surfaces have at least one implementation in the workspace | unit test: enumerate trait implementors via test fixture |
| VP-TBD | Each trait surface compiles in isolation (without slideforge internals) | CI: compile each plugin crate in isolation |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-021 ("Plugin Architecture with 10 Extensibility Surfaces") per capabilities.md §CAP-021 |
| Capability Anchor Justification | CAP-021 ("Plugin Architecture with 10 Extensibility Surfaces") per capabilities.md §CAP-021 — "Expose 10 dyn Trait plugin surfaces ... All bundled plugins dog-food the same traits" is verbatim from CAP-021 |
| L2 Domain Invariants | DI-008 (all bundled plugins must use plugin trait APIs) |
| Architecture Module | slideforge-plugin-api crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.02.002 — composes with (this BC verifies the API exists and is implemented; BC-5.02.002 verifies no bypasses)
- BC-4.01.001 — depends on (PPTX exporter is a bundled plugin that must satisfy this BC)
- BC-4.03.002 — depends on (PDF exporter is a bundled plugin that must satisfy this BC)
- BC-1.12.001 — depends on (DiagramRenderer bundled plugin must satisfy this BC)

## Architecture Anchors

- `architecture/plugin-architecture.md` — 10 extensibility surfaces, trait definitions, and bundled plugin requirements

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
