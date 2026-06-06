---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-050
title: "End-to-End Integration Test Suite"
epic: EPIC-21
wave: 4
points: 8
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-5.02.001, BC-5.02.002, BC-5.01.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-032, NFR-033, NFR-034, NFR-035]
crate: slideforge
target_module: slideforge
subsystems: [SS-14, SS-18]
depends_on:
  - STORY-049
blocks: [STORY-055]
estimated_days: 4
spec_version: "1.3"
last_updated: 2026-06-05
---

# STORY-050: End-to-End Integration Test Suite

## Subsystem Anchor Justification

SS-14 (Plugin API) and SS-18 (CLI Orchestrator) jointly own this story because
the E2E test suite exercises the full plugin registry assembly (SS-14) through
the CLI pipeline orchestration (SS-18). The tests live in the root `slideforge`
crate, which is the assembly point for both subsystems.

## Dependency Anchor Justifications

- Depends on STORY-049: The plugin registry (`PluginRegistry::default()`) must be
  fully assembled before E2E tests can drive the full pipeline. Without the complete
  registry, the tests have nothing to exercise.
- Blocks STORY-055 (Wave 5 CLI build command). STORY-050's E2E suite must pass
  before Wave 5 CLI work begins, as the CLI depends on a functional end-to-end pipeline.

## Summary

Implement a comprehensive end-to-end integration test suite in the root `slideforge`
crate that drives the full pipeline from `.sf` source text to output bytes for all
four Wave-4 output formats (PPTX, DOCX, PDF, HTML is Wave 5 — tested in STORY-046):

1. **Fixture-based pipeline tests**: For each supported format, a known `.sf` fixture
   is parsed → evaluated → validated → laid out → exported. The output bytes are
   validated for structural correctness.

2. **Plugin registry dog-fooding tests**: Verify BC-5.02.001 (all 10 surfaces
   registered) and BC-5.02.002 (no bypass paths) via inspection of the registry.

3. **Error propagation tests**: Verify that errors from any pipeline stage propagate
   correctly to the caller with structured error information.

4. **Observability tests**: Verify that all 6 tracing spans appear during a build
   (NFR-032).

5. **Multi-format tests**: A single `.sf` fixture produces structurally valid output
   in all supported formats.

## Behavioral Contracts

| BC | Version | Title | Covered ACs |
|----|---------|-------|-------------|
| BC-5.02.001 | v1.5 | All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API (Validator Has Pre- and Post-Layout Dispatch) | AC-001 through AC-003, AC-007, AC-008, AC-009 |
| BC-5.02.002 | current | No Bundled Plugin Bypasses the Registered Trait Interface (Dog-Fooding Guarantee) | AC-004 through AC-005 |
| BC-5.01.001 | v1.2 | Missing alt on Visual Element Is Compile Error with Element Location (Post-Layout Enforcement via Stage 6b) | AC-009, EC-001 |

## Deferred Features (Out of Scope for STORY-050)

The following items are explicitly out of scope and anchored to future stories:

1. **Gap 4 — `BuildOptions` convenience constructors** (`BuildOptions::pptx()`,
   `BuildOptions::docx()`, `BuildOptions::pdf()`, `BuildOptions::all_formats()`):
   These are ergonomic sugar with no behavioral impact. The test suite uses
   struct-literal syntax throughout (`BuildOptions { format: Some("pptx"), ..Default::default() }`).
   Convenience constructors are deferred to a future DX/API-polish story.

2. **Deck-level `metadata: title "..."` DSL feature**: The `metadata:` block with
   `title`, `author`, and `date` fields is not yet implemented in the parser
   (`DeckNode` has no `title` field). Gap 1's PDF fix derives the title from the
   first title slide's `title` field — this is the correct interim behavior. The
   full `metadata:` block DSL feature (parser + DeckNode + eval threading) is
   deferred to a dedicated future story (Wave 3+, after slideforge-syntax evolution).

## Acceptance Criteria

### AC-001: Full pipeline produces valid PPTX from fixture
(traces to BC-5.02.001 postcondition 4 — cargo test passes for all plugin impls)

`slideforge::build(FIXTURE_SF, BuildOptions { format: Some("pptx"), ..Default::default() })`
returns `Ok(output)` where `output.bytes` is a valid PPTX ZIP archive. Verified by:
- Parsing the output bytes as a ZIP archive (no error).
- Asserting `[Content_Types].xml` is present in the ZIP.
- Asserting `ppt/presentation.xml` is present.

Note: `BuildOptions::pptx()` convenience constructor is DEFERRED to a future DX story
(Gap 4 from story-050-gap-analysis.md). Use struct-literal syntax throughout this story.

### AC-002: Full pipeline produces valid DOCX from fixture
(traces to BC-5.02.001 postcondition 4)

`slideforge::build(FIXTURE_SF, BuildOptions { format: Some("docx"), ..Default::default() })`
returns `Ok(output)` where `output.bytes` is a valid DOCX ZIP archive. Verified by
parsing as ZIP and asserting `[Content_Types].xml` and `word/document.xml` are present.

### AC-003: Full pipeline produces valid PDF from fixture
(traces to BC-5.02.001 postcondition 4)

`slideforge::build(FIXTURE_SF, BuildOptions { format: Some("pdf"), ..Default::default() })`
returns `Ok(output)` where `output.bytes` begins with `%PDF-`. The PDF contains the
expected number of slides (verified by counting `/Type /Page` entries in the PDF structure).

### AC-004: PluginRegistry::default() has no bypass paths
(traces to BC-5.02.002 postcondition 1 and invariant 1)

`cargo tree -p slideforge-pptx` does NOT list `slideforge-pdf` or `slideforge-html`
as transitive dependencies. Each exporter crate depends ONLY on `slideforge-plugin-api`
and `slideforge-types`. This is verified by a test that shells out to `cargo tree`
and asserts no forbidden cross-exporter dependencies.

### AC-005: External test plugin compiles and registers alongside bundled plugins
(traces to BC-5.02.002 postcondition 4)

(Carried forward from STORY-049 AC-007.) The `external_plugin_test.rs` fixture test
confirms that a minimal plugin using only `slideforge-plugin-api` can be registered
in a `PluginRegistry` alongside the bundled plugins and the `build()` function
completes successfully with the custom plugin active.

### AC-006: Pipeline error from parse stage propagates with file:line:col
(traces to BC-5.02.001 postcondition 4 — all plugin tests pass)

`slideforge::build(INVALID_SF, BuildOptions { format: Some("pptx"), ..Default::default() })`
returns `Err(BuildError::ParseFailed(errors))` where `errors` is a non-empty
`Vec<Diagnostic>` each containing `file`, `line`, `col`, and `message` fields.
Verified by unit test using a fixture with a known syntax error at a known line.

### AC-007: All 6 tracing pipeline spans emitted with canonical names
(traces to BC-5.02.001 canonical test vector AC-007 / NFR-032 — 6 spans per build)

During `slideforge::build()`, exactly 6 pipeline stage spans are emitted with these
canonical names: `parse`, `evaluate`, `brand`, `validate`, `layout`, `export`.
Verified using a `tracing-test` subscriber configured with `RUST_LOG=slideforge=info`
to capture INFO-level events from the `slideforge` crate. The test asserts all 6
named spans are present after a successful build. Span-name canonicalization:
`brand_load` → `brand`, `eval` → `evaluate` (stale names from pre-gap-analysis
implementation must be updated in `lib.rs` as part of this story's Gap 3 fix).

### AC-008: Multi-format build via three separate build() calls produces correct output
(traces to BC-5.02.001 postcondition 4 and canonical test vector AC-008)

For the same `FIXTURE_SF` source, three separate `build()` calls — one per format —
each return `Ok(BuildOutput)`:
- `build(FIXTURE_SF, BuildOptions { format: Some("pptx"), ..Default::default() })` →
  PPTX ZIP with 3 slides.
- `build(FIXTURE_SF, BuildOptions { format: Some("docx"), ..Default::default() })` →
  DOCX ZIP with report-register content as body paragraphs.
- `build(FIXTURE_SF, BuildOptions { format: Some("pdf"), ..Default::default() })` →
  PDF with 3 pages (one per slide).
Verified by slide/page count assertions on each output independently.

Note: There is no `build_all()` or `BuildOptions::all_formats()` API. The multi-format
pattern is three separate `build()` calls (Gap 4 from story-050-gap-analysis.md —
`all_formats()` convenience API is DEFERRED to a future DX story).

### AC-009: Strict mode produces ValidationFailed when chart has no alt text
(traces to BC-5.02.001 postcondition 7 / BC-5.01.001 postcondition 2 — post-layout
alt-text enforcement via ADR-018 Stage 6b)

`slideforge::build(FIXTURE_WITH_MISSING_ALT_SF, BuildOptions { format: Some("pptx"), strict: true, ..Default::default() })`
returns `Err(BuildError::ValidationFailed(diagnostics))`. No output bytes are written.
`diagnostics` contains at least one `Diagnostic` where `diagnostic.code == "E-A11-001"`.

**Mechanism (ADR-018):** E-A11-001 is emitted by `AltTextValidator.validate_post_layout()`
during Stage 6b (the post-layout validation pass), AFTER `layout::run` has produced
the `LaidOutDeck`. `AltTextValidator.validate()` (Stage 5, pre-layout) is a no-op stub
— `Deck.slides[*].blocks` is always `vec![]` after eval (`for_eval.rs:342`), so
ContentBlock-level alt-text checking is only reachable via Stage 6b. The fixture
`test-missing-alt.sf` contains `slide chart:` with no `alt "..."` field, which
causes `AltTextValidator.validate_post_layout()` to emit E-A11-001 on the
`FrameContent::Chart` entry in the laid-out deck.

Verified by:
- Asserting `result.is_err()`
- Pattern-matching `Err(BuildError::ValidationFailed(diagnostics))`
- Asserting `diagnostics.iter().any(|d| d.code == "E-A11-001")`

Complement test: same fixture with `strict: false` returns `Ok(BuildOutput)` (warning
emitted via `tracing::warn!`, output produced — warn-only mode does not return
`ValidationFailed`).

## Tasks

All test files are confirmed created in the worktree. The remaining tasks are
implementation-side fixes required to make the Red Gate tests pass end-to-end.

### Test Infrastructure (DONE — files present in worktree)
- [x] Create `crates/slideforge/tests/e2e_tests.rs` — root test module entry point
- [x] Create `crates/slideforge/tests/e2e/mod.rs` — shared fixture loader
- [x] Create `crates/slideforge/tests/e2e/pipeline_pptx.rs` — AC-001, AC-008
- [x] Create `crates/slideforge/tests/e2e/pipeline_docx.rs` — AC-002, AC-008
- [x] Create `crates/slideforge/tests/e2e/pipeline_pdf.rs` — AC-003, AC-008
- [x] Create `crates/slideforge/tests/e2e/registry.rs` — AC-004, AC-005
- [x] Create `crates/slideforge/tests/e2e/error_propagation.rs` — AC-006, AC-009
- [x] Create `crates/slideforge/tests/e2e/observability.rs` — AC-007
- [x] Create `crates/slideforge/tests/e2e/multi_format.rs` — AC-008 multi-format
- [x] Add `tracing-test = "=0.2.5"` as dev-dependency for AC-007
- [x] Add `zip = "=4.2.0"` as dev-dependency for ZIP structure assertions

### Gap 1 Fix — PDF NoDocumentTitle (AC-003, AC-008 PDF, EC-005)
- [ ] Fix `slideforge-eval/src/eval.rs` `eval_deck_with_variant`: after evaluating
  slides, set `DeckMetadata.title` by scanning `slides` for the first slide with a
  non-empty `title` field (as `Value::Str`). If no title slide exists, `title`
  remains `None` — PDF export then correctly fails PDF/UA-1 with `NoDocumentTitle`,
  prompting the author to provide a real title. No fabricated/placeholder title
  (e.g., `"Untitled Presentation"`) is ever emitted; emitting one would constitute
  an accessibility anti-pattern by defeating the PDF/UA-1 meaningful-title
  requirement (adjudicated 2026-06-05: fabricated fallback title → a11y
  anti-pattern). Closes 4 failing PDF tests.

### Gap 2 Fix — Post-Layout Alt-Text Validation (AC-009) — ADR-018
- [ ] `crates/slideforge-plugin-api/src/traits/validator.rs`: Add `validate_post_layout`
  defaulted method per ADR-018 Decision 2 (default no-op `{ vec![] }`); import
  `slideforge_layout::LaidOutDeck` (existing dep edge — no Cargo.toml change needed).
- [ ] `crates/slideforge-validate/src/alt_text.rs`: Migrate content-block loop from
  `validate()` to `validate_post_layout()`; replace `validate()` body with no-op stub
  citing ADR-018 and `for_eval.rs:342`. Add `slideforge-layout` to
  `crates/slideforge-validate/Cargo.toml`.
- [ ] `crates/slideforge-validate/Cargo.toml`: Add `slideforge-layout` dependency.
- [ ] `crates/slideforge/src/lib.rs`: Add Stage 6b after `layout_run` call — loop all
  registered validators calling `validate_post_layout(&laid_out, &validator_opts)`,
  extend `all_validator_diagnostics` with results, apply strict-mode gate to combined
  list. Closes 2 failing AC-009 tests.

### Gap 3 Fix — Tracing Span Names and Subscriber Filter (AC-007)
- [ ] `crates/slideforge/src/lib.rs` `build_inner`: Rename `brand_load` → `brand` and
  `eval` → `evaluate` in the 6 canonical `tracing::info_span!` entries to match NFR-032
  and BC-5.02.001 canonical test vector AC-007.
- [ ] `crates/slideforge/tests/e2e/observability.rs`: Configure `tracing_test`
  subscriber with `RUST_LOG=slideforge=info` filter (or equivalent) so INFO-level
  events from the `slideforge` crate are captured. Closes 1 failing observability test.

## Previous Story Intelligence

STORY-049 established `PluginRegistry::default()` and `slideforge::build()`. This
story writes the tests that validate those functions. This is the test-writer's
primary reference: all tests invoke `slideforge::build()` from the public API.

Key observation from STORY-049: the `build()` function is the single entry point.
E2E tests should NOT call into individual pipeline stages directly — they test the
black-box behavior of `build()`.

The `test-3slide.sf` fixture should include:
- `lang "en-US"` declaration.
- A title slide (uses `slide title:`).
- A content slide with `alt "..."` on all visual elements.
- A severity_cards slide with proper label for color coding.
- `notes:` register content on at least one slide.
- `report:` register content on at least one slide.

## Architecture Compliance Rules

1. **Tests use public API only**: All E2E tests call `slideforge::build()` — no
   imports from internal crates (no `use slideforge_pptx::`, etc.).
2. **Fixture files are `.sf` source text**: Fixtures are real DSL source files, not
   pre-computed IR. The pipeline must parse them from scratch for each test.
3. **No integration test hardcodes binary output**: Tests validate output structure
   (ZIP structure, PDF header, page count) rather than exact bytes. Exact byte
   comparison would make tests fragile.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge` (self, STORY-049) | workspace | Public `build()` API under test |
| `tracing-test` | `=0.2.5` | Span assertion in AC-007 |
| `zip` | `=4.2.0` | ZIP structure assertion for PPTX/DOCX output (compatible with ooxmlsdk dep) |

Note: `tempfile` is NOT a dependency of the `slideforge` crate for this story — the E2E
tests use in-memory output bytes (no temporary files). `tempfile` removed from
`crates/slideforge/Cargo.toml` (unused; adversary OBS-050-P2-002, 2026-06-05).
`tempfile = "=3.27.0"` is retained only in `slideforge-brand`, `slideforge-data`, and
`slideforge-pdf` where it is actually used.

## File Structure Requirements

The following files are confirmed present in the worktree (verified 2026-06-05).
Status column reflects disk reality post-Red-Gate.

| File | Status | Purpose |
|------|--------|---------|
| `crates/slideforge/tests/e2e_tests.rs` | EXISTS | Root test module entry point (`mod e2e;`) |
| `crates/slideforge/tests/e2e/mod.rs` | EXISTS | Fixture loader utility (shared across e2e modules) |
| `crates/slideforge/tests/e2e/pipeline_pptx.rs` | EXISTS | PPTX pipeline E2E tests (AC-001, AC-008) |
| `crates/slideforge/tests/e2e/pipeline_docx.rs` | EXISTS | DOCX pipeline E2E tests (AC-002, AC-008) |
| `crates/slideforge/tests/e2e/pipeline_pdf.rs` | EXISTS | PDF pipeline E2E tests (AC-003, AC-008) |
| `crates/slideforge/tests/e2e/registry.rs` | EXISTS | Registry dog-fooding tests (AC-004, AC-005) |
| `crates/slideforge/tests/e2e/error_propagation.rs` | EXISTS | Error handling tests (AC-006, AC-009) |
| `crates/slideforge/tests/e2e/observability.rs` | EXISTS | Tracing span tests (AC-007) |
| `crates/slideforge/tests/e2e/multi_format.rs` | EXISTS | Multi-format tests (AC-008 three-call pattern) |
| `crates/slideforge/tests/fixtures/test-3slide.sf` | EXISTS | Happy-path fixture |
| `crates/slideforge/tests/fixtures/test-invalid-syntax.sf` | EXISTS | Error-propagation fixture |
| `crates/slideforge/tests/fixtures/test-missing-alt.sf` | EXISTS | Validation error fixture (chart with no alt) |
| `crates/slideforge/Cargo.toml` | EXISTS (modified) | Dev-dependencies: tracing-test, zip (tempfile removed — unused) |
| `crates/slideforge-eval/src/eval.rs` | MODIFY (Gap 1 fix) | Derive DeckMetadata.title from first title slide |
| `crates/slideforge-plugin-api/src/traits/validator.rs` | MODIFY (Gap 2 fix) | Add validate_post_layout defaulted method (ADR-018) |
| `crates/slideforge-validate/src/alt_text.rs` | MODIFY (Gap 2 fix) | Migrate loop to validate_post_layout; no-op stub in validate() |
| `crates/slideforge-validate/Cargo.toml` | MODIFY (Gap 2 fix) | Add slideforge-layout dependency |
| `crates/slideforge/src/lib.rs` | MODIFY (Gap 2+3 fix) | Add Stage 6b post-layout pass; rename brand_load→brand, eval→evaluate spans |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,800 |
| BC-5.02.001 v1.5 | ~2,000 |
| BC-5.01.001 v1.2 | ~1,200 |
| BC-5.02.002 | ~1,000 |
| ADR-018 (post-layout validation pass) | ~2,000 |
| STORY-049 build() API (context) | ~1,000 |
| BC files (3 BCs) | ~4,200 total (included above) |
| Fixture .sf files (3 fixtures) | ~1,500 |
| Test files (8 e2e modules) | ~5,000 |
| Implementation fix files (5 files) | ~2,000 |
| **Total** | **~19,500** |

Context budget: ~20% of a 100k-token context window. At the ceiling — do not expand
scope further without splitting into a follow-on story.

## Test Strategy

- **E2E (integration) tests**: The primary test type. Drive `build()` with known
  fixtures, assert structural properties of output. No unit tests in this story —
  this story IS the test layer.
- **Structure validation**: PPTX/DOCX via ZIP inspection. PDF via `%PDF-` header
  and `/Type /Page` count.
- **Error propagation**: Known-bad fixtures produce known error codes.
- **Observability**: `tracing-test` subscriber captures all spans; assert 6 required
  spans are present.
- **Dog-fooding**: `cargo tree` assertions confirm no cross-exporter dependencies.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Fixture with empty slide count (no slides) | `Err(BuildError::ValidationFailed(diagnostics))` where at least one diagnostic carries `E-LAY-002` |
| EC-002 | Fixture with circular @include | `Err(BuildError::ParseFailed(errors))` where at least one error carries `E-PAR-004` |
| EC-003 | Fixture with @data pointing to non-existent file | `Err(BuildError::DataFailed(errors))` — see error taxonomy for E-DAT-001 code |
| EC-004 | Build with all 3 register types (notes/report/detail) | Notes in PPTX/DOCX; report in DOCX only; detail in DOCX/PDF only |
| EC-005 | Multi-format build (three separate build() calls) | Three separate `build()` calls — `format: Some("pptx")`, `format: Some("docx")`, `format: Some("pdf")` — each return `Ok(BuildOutput)`; each structurally valid. No `BuildOptions::all_formats()` API exists (DEFERRED to future DX story). |

## Forbidden Dependencies

E2E test files must NOT import from:
- `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, `slideforge-html` directly
- Any internal crate with non-public APIs

All E2E test assertions go through `slideforge::build()` public API and the output
bytes inspection utilities (ZIP, `%PDF-` prefix check, etc.).
