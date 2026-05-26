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
behavioral_contracts: [BC-5.02.001, BC-5.02.002]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-032, NFR-033, NFR-034, NFR-035]
crate: slideforge
target_module: slideforge
subsystems: [SS-14, SS-18]
depends_on:
  - STORY-049
blocks: [STORY-055]
estimated_days: 4
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
- Blocks nothing: STORY-050 is the last story in Wave 4. It blocks no further
  Wave 4 stories.

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

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-5.02.001 | All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API | AC-001 through AC-003 |
| BC-5.02.002 | No Bundled Plugin Bypasses the Registered Trait Interface (Dog-Fooding Guarantee) | AC-004 through AC-005 |

## Acceptance Criteria

### AC-001: Full pipeline produces valid PPTX from fixture
(traces to BC-5.02.001 postcondition 4 — cargo test passes for all plugin impls)

`slideforge::build(FIXTURE_SF, BuildOptions::pptx())` returns `Ok(output)` where
`output.bytes` is a valid PPTX ZIP archive. Verified by:
- Parsing the output bytes as a ZIP archive (no error).
- Asserting `[Content_Types].xml` is present in the ZIP.
- Asserting `ppt/presentation.xml` is present.

### AC-002: Full pipeline produces valid DOCX from fixture
(traces to BC-5.02.001 postcondition 4)

`slideforge::build(FIXTURE_SF, BuildOptions::docx())` returns `Ok(output)` where
`output.bytes` is a valid DOCX ZIP archive. Verified by parsing as ZIP and asserting
`[Content_Types].xml` and `word/document.xml` are present.

### AC-003: Full pipeline produces valid PDF from fixture
(traces to BC-5.02.001 postcondition 4)

`slideforge::build(FIXTURE_SF, BuildOptions::pdf())` returns `Ok(output)` where
`output.bytes` begins with `%PDF-`. The PDF contains the expected number of slides
(verified by counting `/Type /Page` entries in the PDF structure).

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

`slideforge::build(INVALID_SF, BuildOptions::pptx())` returns
`Err(BuildError::ParseErrors(errors))` where `errors` is a non-empty `Vec<Diagnostic>`
each containing `file`, `line`, `col`, and `message` fields. Verified by unit test
using a fixture with a known syntax error at a known line.

### AC-007: All 6 tracing pipeline spans emitted
(traces to NFR-032 — 6 spans per build)

During `slideforge::build()`, all 6 pipeline stage spans are emitted:
`parse`, `evaluate`, `brand`, `validate`, `layout`, `export`. Verified using a
`tracing-test` subscriber that collects all span open/close events and asserts all
6 are present after a successful build.

### AC-008: Multi-format build produces structurally correct output for all formats
(traces to BC-5.02.001 postcondition 4)

For the same `FIXTURE_SF` source, calling `build()` with PPTX, DOCX, and PDF format
options produces three structurally distinct but content-equivalent outputs:
- PPTX: 3-slide presentation (matches slide count in fixture).
- DOCX: Document with report-register content as body paragraphs.
- PDF: 3-page PDF (one page per slide).
Verified by slide/page count assertions on each output.

### AC-009: Strict mode produces no output on validation error
(traces to BC-5.02.001 postcondition 4 and BC-3.03.002)

`slideforge::build(FIXTURE_WITH_MISSING_ALT_SF, BuildOptions::pptx())` returns
`Err(BuildError::ValidationErrors(...))`. No output bytes are written. The error
list includes `E-A11-001` for the missing alt text. Verified by asserting
`result.is_err()` and checking `errors[0].code == "E-A11-001"`.

## Tasks

- [ ] Create `crates/slideforge/tests/e2e/mod.rs` — shared fixture loader
- [ ] Create `crates/slideforge/tests/fixtures/` directory:
  - `test-3slide.sf` — 3-slide fixture (title + content + severity_cards)
  - `test-invalid-syntax.sf` — fixture with known syntax error at line 3
  - `test-missing-alt.sf` — fixture with a chart missing alt text
- [ ] Create `crates/slideforge/tests/e2e/pipeline_pptx.rs` — AC-001, AC-008
- [ ] Create `crates/slideforge/tests/e2e/pipeline_docx.rs` — AC-002, AC-008
- [ ] Create `crates/slideforge/tests/e2e/pipeline_pdf.rs` — AC-003, AC-008
- [ ] Create `crates/slideforge/tests/e2e/registry.rs` — AC-004, AC-005
- [ ] Create `crates/slideforge/tests/e2e/error_propagation.rs` — AC-006, AC-009
- [ ] Create `crates/slideforge/tests/e2e/observability.rs` — AC-007
- [ ] Add `tracing-test = "=0.2.5"` as dev-dependency for AC-007
- [ ] Add `zip = "=4.2.0"` as dev-dependency for ZIP structure assertions
- [ ] Write the AC-001 through AC-009 tests per the above breakdown

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
| `tempfile` | `=3.27.0` | Temporary directories for output file tests |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge/tests/fixtures/test-3slide.sf` | Create | Happy-path fixture |
| `crates/slideforge/tests/fixtures/test-invalid-syntax.sf` | Create | Error-propagation fixture |
| `crates/slideforge/tests/fixtures/test-missing-alt.sf` | Create | Validation error fixture |
| `crates/slideforge/tests/e2e/mod.rs` | Create | Fixture loader utility |
| `crates/slideforge/tests/e2e/pipeline_pptx.rs` | Create | PPTX pipeline E2E tests |
| `crates/slideforge/tests/e2e/pipeline_docx.rs` | Create | DOCX pipeline E2E tests |
| `crates/slideforge/tests/e2e/pipeline_pdf.rs` | Create | PDF pipeline E2E tests |
| `crates/slideforge/tests/e2e/registry.rs` | Create | Registry dog-fooding tests |
| `crates/slideforge/tests/e2e/error_propagation.rs` | Create | Error handling tests |
| `crates/slideforge/tests/e2e/observability.rs` | Create | Tracing span tests |
| `crates/slideforge/Cargo.toml` | Modify | Add dev-dependencies: tracing-test, zip, tempfile |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,800 |
| BC-5.02.001 | ~1,200 |
| BC-5.02.002 | ~1,000 |
| STORY-049 build() API (context) | ~1,000 |
| Fixture .sf files to write | ~1,500 |
| Test files to write (6 test modules) | ~4,000 |
| **Total** | **~11,500** |

Context budget: ~12% of a 100k-token context window. Within limit.

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
| EC-001 | Fixture with empty slide count (no slides) | `Err(BuildError::ValidationErrors([E-VAL-001]))` |
| EC-002 | Fixture with circular @include | `Err(BuildError::ParseErrors([E-INC-001]))` |
| EC-003 | Fixture with @data pointing to non-existent file | `Err(BuildError::DataErrors([E-DAT-001]))` |
| EC-004 | Build with all 3 register types (notes/report/detail) | Notes in PPTX/DOCX; report in DOCX only; detail in DOCX/PDF only |
| EC-005 | Multi-format build via BuildOptions::all_formats() | All 3 outputs (PPTX+DOCX+PDF) produced; each structurally valid |

## Forbidden Dependencies

E2E test files must NOT import from:
- `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, `slideforge-html` directly
- Any internal crate with non-public APIs

All E2E test assertions go through `slideforge::build()` public API and the output
bytes inspection utilities (ZIP, `%PDF-` prefix check, etc.).
