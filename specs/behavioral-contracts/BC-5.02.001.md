---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - version: "1.5"
    date: 2026-06-05
    author: product-owner
    reason: "STORY-050 Gap-2 / ADR-018 (human-authorized 2026-06-05): Extend Validator surface with additive-defaulted validate_post_layout(&LaidOutDeck, &ValidatorOptions) -> Vec<Diagnostic> method per ADR-018 Decision 2. Add Postcondition 5 (post-layout pass ordering), Postcondition 6 (combined diagnostic list / strict gate fires once), Postcondition 7 (AltTextValidator migration to post-layout), and Invariant 4 (validator classification rule). Add EC-005 (AltTextValidator post-layout) and EC-006 (combined diagnostic collection). Add canonical test vectors TV-P5.1, TV-P5.2, TV-P5.3 for AC-009 alignment. Reconcile AC-009 to use BuildError::ValidationFailed (not ValidationErrors). Reconcile AC-008 to three-separate-build()-calls (not all_formats()). Reconcile AC-007 to 6 named spans."
  - version: "1.4"
    date: 2026-06-05
    author: product-owner
    reason: "STORY-085 / ADR-017 Option A (human-authorized 2026-06-05): Clarify Invariant 2 — additive defaulted trait methods (new methods with a default body that do not break existing implementors) are PERMITTED and do not constitute a contract violation. Non-additive changes (removal, rename, signature change of existing methods) and any bypass API remain forbidden. References ADR-017 as the authorizing decision for the render_with_context extension."
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

# BC-5.02.001: All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API (Validator Has Pre- and Post-Layout Dispatch)

## Description

The `slideforge-plugin-api` crate defines 10 `dyn Trait` extensibility surfaces. Each
surface must have at least one bundled plugin implementation that compiles and passes
tests using only the public trait API. This verifies that the trait API is complete,
usable, and not a leaky abstraction. The 10 surfaces are: DataSource, Exporter,
ChartRenderer, DiagramRenderer, Validator, MathRenderer, BrandProvider, SlideType,
SectionType, InlineFormat.

The `Validator` surface has two dispatch methods: `validate(&Deck, &ValidatorOptions)`
(pre-layout, Stage 5) and `validate_post_layout(&LaidOutDeck, &ValidatorOptions)`
(post-layout, Stage 6b). Both methods are called by `build_inner` on every registered
Validator. `validate_post_layout` carries a complete default no-op body so existing
implementations compile unchanged (ADR-018, human-authorized 2026-06-05).

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
   - Validator: accessibility validator (`AltTextValidator`), overflow validator;
     both dispatch through `validate()` and optionally `validate_post_layout()`
   - MathRenderer: pulldown-latex + KaTeX math renderer
   - BrandProvider: file-based brand provider
   - SlideType: all 31 slide type implementations
   - SectionType: 7 bundled implementations (executive_summary, risk_register, methodology, scope, approval, appendix, glossary — per CANONICAL_MANUAL_SECTION_TYPES in deck.rs)
   - InlineFormat: all 12 InlineNode variants (Plain, Bold, Italic, Code, Link, Math, Footnote, Xref, Superscript, Subscript, Strikethrough, Highlight)
3. Each bundled plugin implementation compiles using only the public API exposed by
   `slideforge-plugin-api` — no direct imports of non-plugin-api crate internals.
4. `cargo test --workspace` passes for all plugin implementations.
5. **Post-layout validation pass (Stage 6b).** `build_inner` invokes
   `validate_post_layout(&laid_out, &validator_opts)` on every registered `Validator`
   AFTER `layout::run` succeeds (Stage 6) and BEFORE the exporter is invoked (Stage 7).
   Diagnostics from Stage 6b are appended to the same accumulated diagnostic list as
   Stage 5 (pre-layout) diagnostics.
   - Stage ordering: Stage 5 (`validate(&Deck)`) → Stage 6 (`layout::run`) →
     Stage 6b (`validate_post_layout(&LaidOutDeck)`) → Stage 7 (export).
   - Stage 6b runs unconditionally (not only in strict mode), so validators that
     override `validate_post_layout` emit warnings even when `strict = false`.
   - A `BuildError::ValidationFailed` from Stage 5 does NOT abort before Stage 6b —
     both passes complete fully before the strict-mode gate evaluates the combined list.
6. **Single strict-mode gate on combined diagnostics.** The strict-mode gate fires
   exactly once, after Stage 6b completes, on the combined `pre_diags + post_diags`
   list. Any `DiagnosticSeverity::Error` in the combined list causes
   `BuildError::ValidationFailed` carrying the full combined diagnostic list
   (Error + Warning + Info from both passes). Collect-all, not fail-on-first.
7. **AltTextValidator classification: post-layout only (ADR-018 Decision 3).**
   `AltTextValidator` overrides `validate_post_layout` to iterate
   `laid_out.slides[*].frames` for `FrameContent::Chart`, `FrameContent::Image`, and
   `FrameContent::Diagram` entries whose alt field is missing or decorative-but-not-explicit.
   `AltTextValidator.validate()` is a no-op stub (correct — `Deck.slides[*].blocks`
   is always `vec![]` after eval per `for_eval.rs:342`). A deck with a chart slide and
   no `alt "..."` must return `Err(BuildError::ValidationFailed)` containing at least
   one diagnostic with `code == "E-A11-001"` when `strict = true`.

## Invariants

1. The plugin API surface count is exactly 10. Adding an 11th surface requires updating
   this BC and BC-INDEX.
2. **Trait-signature stability: additive defaulted methods are permitted; non-additive
   changes are forbidden.**
   The trait definitions in `slideforge-plugin-api` are the contracts — no alternate
   unstable API exists. This invariant governs backward compatibility for external
   implementors:
   - **PERMITTED (additive-defaulted extension):** Adding a new trait method that
     carries a complete default body (i.e., the method has a sensible default
     implementation and existing implementors are NOT required to override it) does not
     break existing implementations and is allowed. Two authorized instances:
     - ADR-017 Option A: `InlineFormat::render_with_context(node, format, &InlineRenderContext)`
       with a default body that delegates to `render(node, format)`. (human-authorized 2026-06-05)
     - ADR-018 Decision 2: `Validator::validate_post_layout(&LaidOutDeck, &ValidatorOptions) -> Vec<Diagnostic>`
       with a default no-op body `{ vec![] }`. Existing `Validator` implementations compile
       unchanged and inherit the no-op. (human-authorized 2026-06-05)
   - **FORBIDDEN (non-additive changes):** Removing or renaming an existing method;
     changing the signature of an existing method (parameter types, return type, or
     generic bounds); adding a new method WITHOUT a default body that forces
     implementors to add code.
   - **FORBIDDEN (bypass API):** Any API path — function, associated constant, blanket
     impl, or re-export — that allows a caller to serialize an `InlineNode` or invoke
     plugin logic without going through the trait dispatch mechanism.
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
4. **Validator classification rule (ADR-018 Decision 4).** Each `Validator`
   implementation is classified as pre-layout, post-layout, or both, based on which IR
   it needs:
   - Pre-layout only (overrides `validate`, keeps default `validate_post_layout`): needs
     semantic slide fields, zero-slide count, lang presence, or structural invariants
     resolvable from `Deck` field values. Current members: `ZeroSlideValidator`,
     `LangValidator`.
   - Post-layout only (overrides `validate_post_layout`, `validate` returns `vec![]`):
     needs `ContentBlock`-level data present only in `LaidOutDeck.slides[*].frames`.
     Current members: `AltTextValidator`.
   - Both: needs semantic AND geometric data (no current members in v1.0).
   Future validators (CanvasOverflowValidator, ContrastValidator) MUST be classified at
   the time they are introduced using this rule.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | New slide type added (32nd type) | New SlideType trait implementation added; total remains covered by this BC |
| EC-002 | Plugin API trait method signature change | All implementations updated; compile error otherwise — CI enforces |
| EC-003 | Plugin registered but panics during execution | Panic is caught at plugin dispatch boundary; returned as E-EXP-NNN error |
| EC-004 | Two bundled plugins registered for same surface | Both registered; caller selects by name/priority (configurable) |
| EC-005 | Deck with `slide chart:` and no `alt "..."` built with `strict = true` | `validate_post_layout` on `AltTextValidator` fires after layout; `Err(BuildError::ValidationFailed)` returned carrying at least one `Diagnostic { code: "E-A11-001", .. }` in the combined list |
| EC-006 | Deck has both a missing `lang` (pre-layout E-A11-003) AND a missing chart `alt` (post-layout E-A11-001) | Both diagnostics appear in the combined list returned by `Err(BuildError::ValidationFailed)`; user sees both errors in one `build()` call without re-running |
| EC-007 | Existing `Validator` implementation (third-party) compiled against old trait that only had `validate()` | Compiles unchanged; inherits default `validate_post_layout` no-op; no source change required |
| EC-008 | `AltTextValidator.validate()` called with a `Deck` whose `slides[*].blocks` is `vec![]` (post-eval state) | Returns `vec![]` (no diagnostics); this is correct and intentional — `ContentBlock::Chart/Image/Diagram` are only present post-layout |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `cargo build --workspace` | 10/10 plugin surfaces compile; 0 trait method errors | happy-path |
| `cargo test --workspace` | All plugin unit tests pass | happy-path |
| Minimal test plugin implementing `DataSource` without slideforge-internal imports | Compiles successfully; `cargo test` passes | happy-path (API completeness) |
| `slideforge::build(FIXTURE_SF, BuildOptions { format: Some("pptx"), strict: true, ..Default::default() })` (3-slide fixture with `lang "en-US"` and `alt` on all visual elements) | `Ok(BuildOutput { bytes: <valid PPTX ZIP> })` — PPTX ZIP archive with `[Content_Types].xml` and `ppt/presentation.xml` present (AC-001) | happy-path (AC-001) |
| `slideforge::build(FIXTURE_SF, BuildOptions { format: Some("pptx"), strict: true, ..Default::default() })`, `slideforge::build(FIXTURE_SF, BuildOptions { format: Some("docx"), ..Default::default() })`, `slideforge::build(FIXTURE_SF, BuildOptions { format: Some("pdf"), ..Default::default() })` — three separate calls on the same fixture | All three return `Ok(BuildOutput)`; PPTX has 3 slides, DOCX has report body paragraphs, PDF has 3 pages (AC-008 — three separate `build()` calls, no `all_formats()` API) | happy-path (AC-008) |
| `slideforge::build(MISSING_ALT_FIXTURE_SF, BuildOptions { format: Some("pptx"), strict: true, ..Default::default() })` where fixture has `slide chart:` with no `alt "..."` | `Err(BuildError::ValidationFailed(diagnostics))` where `diagnostics` contains at least one entry with `code == "E-A11-001"`; no output bytes written (AC-009) | error (AC-009) |
| Same missing-alt fixture with `strict: false` | `Ok(BuildOutput)` — warning emitted via `tracing::warn!`, output produced; `ValidationFailed` NOT returned in warn-only mode (AC-009 warn-only complement) | warn-only (AC-009 complement) |
| `slideforge::build(FIXTURE_SF, ...)` with `tracing_test` subscriber capturing INFO events (via `RUST_LOG=slideforge=info` filter) | All 6 named pipeline spans present: `parse`, `evaluate`, `brand`, `validate`, `layout`, `export` — in that order (AC-007) | observability (AC-007) |
| `slideforge::build(INVALID_SYNTAX_SF, BuildOptions { format: Some("pptx"), strict: true, ..Default::default() })` where fixture has known syntax error at line 3 | `Err(BuildError::ParseFailed(errors))` where `errors` is non-empty `Vec<Diagnostic>` each containing `file`, `line`, `col`, `message` fields; line == 3 (AC-006) | error (AC-006) |

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
