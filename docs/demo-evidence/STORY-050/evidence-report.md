# STORY-050 Demo Evidence Report

**Story:** STORY-050 — End-to-End Integration Test Suite
**Crate:** `slideforge` (root)
**Branch:** `feature/STORY-050`
**Recorded:** 2026-06-05
**Recording tool:** VHS (terminal capture of `cargo nextest run -p slideforge`)

---

## Artifacts

| File | Format | Covered ACs |
|------|--------|-------------|
| `AC-001-009-full-e2e-suite.gif` | GIF (PR embed) | AC-001 through AC-009 (all) |
| `AC-001-009-full-e2e-suite.webm` | WEBM (archival) | AC-001 through AC-009 (all) |
| `AC-001-009-full-e2e-suite.tape` | VHS script | AC-001 through AC-009 (all) |
| `AC-001-003-008-full-pipeline-all-formats.gif` | GIF (PR embed) | AC-001, AC-002, AC-003, AC-008 |
| `AC-001-003-008-full-pipeline-all-formats.webm` | WEBM (archival) | AC-001, AC-002, AC-003, AC-008 |
| `AC-001-003-008-full-pipeline-all-formats.tape` | VHS script | AC-001, AC-002, AC-003, AC-008 |
| `AC-009-missing-alt-accessibility-enforcement.gif` | GIF (PR embed) | AC-009 (success + error paths), AC-006 |
| `AC-009-missing-alt-accessibility-enforcement.webm` | WEBM (archival) | AC-009 (success + error paths), AC-006 |
| `AC-009-missing-alt-accessibility-enforcement.tape` | VHS script | AC-009 (success + error paths), AC-006 |
| `AC-007-observability-spans.gif` | GIF (PR embed) | AC-007, AC-004, AC-005 |
| `AC-007-observability-spans.webm` | WEBM (archival) | AC-007, AC-004, AC-005 |
| `AC-007-observability-spans.tape` | VHS script | AC-007, AC-004, AC-005 |

**No example binary added.** All demo evidence drives the existing `cargo nextest run` harness
directly — the 62-test suite in `crates/slideforge/` is the demo surface.

---

## Coverage Map

### AC-001: Full pipeline produces valid PPTX from fixture

**Demonstrated in:** `AC-001-003-008-full-pipeline-all-formats.tape` and `AC-001-009-full-e2e-suite.tape`

**Evidence:** `cargo nextest run -p slideforge -E 'test(pipeline_pptx) or ...' -q` runs tests including:
- `test_bc_5_02_001_ac001_pptx_build_returns_ok_with_valid_zip` — asserts ZIP parses without error, `[Content_Types].xml` and `ppt/presentation.xml` present
- `test_bc_5_02_001_ac001_pptx_default_format_when_none` — asserts `format: None` defaults to PPTX
- `test_bc_5_02_001_ac008_pptx_slide_count_matches_fixture` — asserts 3 slides produced from `test-3slide.sf`

Recording shows all 3 tests pass.

**Result:** PASS

---

### AC-002: Full pipeline produces valid DOCX from fixture

**Demonstrated in:** `AC-001-003-008-full-pipeline-all-formats.tape`

**Evidence:**
- `test_bc_5_02_001_ac002_docx_build_returns_ok_with_valid_zip` — asserts ZIP contains `[Content_Types].xml` and `word/document.xml`
- `test_bc_5_02_001_ac008_docx_has_body_paragraphs` — asserts report-register content appears as body paragraphs

Recording shows both tests pass.

**Result:** PASS

---

### AC-003: Full pipeline produces valid PDF from fixture

**Demonstrated in:** `AC-001-003-008-full-pipeline-all-formats.tape`

**Evidence:**
- `test_bc_5_02_001_ac003_pdf_build_returns_ok_with_pdf_header` — asserts output begins with `%PDF-`
- `test_bc_5_02_001_ac008_pdf_page_count_matches_slide_count` — asserts page count matches slide count

Gap 1 fix: `DeckMetadata.title` derived from first title slide so PDF/UA-1 `NoDocumentTitle` does not fire for titled decks.

Recording shows both tests pass.

**Result:** PASS

---

### AC-004: PluginRegistry::default() has no bypass paths

**Demonstrated in:** `AC-007-observability-spans.tape`

**Evidence:**
- `test_bc_5_02_002_ac004_pptx_has_no_cross_exporter_deps` — shells out to `cargo tree -p slideforge-pptx`, asserts `slideforge-pdf` and `slideforge-html` are NOT transitive deps
- `test_bc_5_02_002_ac004_pdf_has_no_cross_exporter_deps` — same check for `slideforge-pdf`

Recording shows both registry/cargo-tree tests pass.

**Result:** PASS

---

### AC-005: External test plugin compiles and registers alongside bundled plugins

**Demonstrated in:** `AC-007-observability-spans.tape`

**Evidence:**
- `test_bc_5_02_002_external_plugin_compiles_and_registers_with_bundled_plugins` — minimal external plugin registers in `PluginRegistry` with bundled plugins; `build()` completes successfully
- `test_bc_5_02_002_external_plugin_id_returns_expected_value` — plugin ID accessor works
- `test_bc_5_02_002_external_plugin_usable_without_bundled_plugins` — plugin usable standalone

Recording shows all external plugin tests pass.

**Result:** PASS

---

### AC-006: Pipeline error from parse stage propagates with file:line:col

**Demonstrated in:** `AC-009-missing-alt-accessibility-enforcement.tape`

**Evidence:**
- `test_bc_5_02_001_ac006_parse_error_returns_parse_failed` — `build(INVALID_SF, ...)` returns `Err(BuildError::ParseFailed(...))`
- `test_bc_5_02_001_ac006_parse_failed_has_structured_diagnostics` — each `Diagnostic` has `file`, `line`, `col`, `message` fields

Recording shows both tests pass (error path confirmed).

**Result:** PASS

---

### AC-007: All 6 tracing pipeline spans emitted with canonical names

**Demonstrated in:** `AC-007-observability-spans.tape`

**Evidence:**
- `test_bc_5_02_001_ac007_all_pipeline_stage_markers_emitted` — `tracing-test` subscriber captures all 6 canonical span names: `parse`, `evaluate`, `brand`, `validate`, `layout`, `export`
- `test_bc_5_02_001_ac007_stage_markers_stop_at_parse_failure` — only `parse` span emitted when input is invalid

Gap 3 fix: span names `brand_load` → `brand` and `eval` → `evaluate` corrected in `lib.rs`.

Recording shows both observability tests pass.

**Result:** PASS

---

### AC-008: Multi-format build via three separate build() calls produces correct output

**Demonstrated in:** `AC-001-003-008-full-pipeline-all-formats.tape`

**Evidence:**
- `test_bc_5_02_001_ac008_multi_format_all_three_produce_valid_output` — three separate `build()` calls each return `Ok(BuildOutput)`: PPTX ZIP with 3 slides, DOCX ZIP with body paragraphs, PDF with 3 pages
- `test_bc_5_02_001_ac008_sequential_builds_produce_independent_outputs` — outputs do not share state
- `test_bc_5_02_001_ec005_all_formats_built_from_single_fixture` — single `.sf` fixture correctly drives all three formats

Recording shows all multi-format tests pass.

**Result:** PASS

---

### AC-009: Strict mode produces ValidationFailed when chart has no alt text

**Demonstrated in:** `AC-009-missing-alt-accessibility-enforcement.tape`

**Evidence (error path):**
- `test_bc_5_02_001_ac009_missing_alt_returns_validation_failed` — `build(FIXTURE_WITH_MISSING_ALT_SF, BuildOptions { strict: true, ... })` returns `Err(BuildError::ValidationFailed(...))`
- `test_bc_5_02_001_ac009_validation_failed_contains_e_a11_001` — `diagnostics.iter().any(|d| d.code == "E-A11-001")` is true

**Evidence (success path / complement):**
- `test_bc_5_02_001_ac009_warn_only_does_not_return_validation_failed` — same fixture with `strict: false` returns `Ok(BuildOutput)` (warning only)

Gap 2 fix: `AltTextValidator.validate_post_layout()` added per ADR-018 Stage 6b; Stage 6b pass wired in `lib.rs` after `layout::run`.

Recording shows all 3 AC-009 tests pass (both success and error paths confirmed).

**Result:** PASS

---

## Gap Fix Summary

| Gap | Fix | Evidenced By |
|-----|-----|-------------|
| Gap 1 — PDF NoDocumentTitle | `DeckMetadata.title` derived from first title slide in `eval_deck_with_variant` | AC-003, AC-008 PDF tests |
| Gap 2 — Post-layout alt-text enforcement | `validate_post_layout()` added to `Validator` trait (ADR-018 Stage 6b); AltTextValidator wired; Stage 6b pass in `lib.rs` | AC-009 error + complement tests |
| Gap 3 — Span name canonicalization | `brand_load` → `brand`, `eval` → `evaluate` in `lib.rs` `build_inner` | AC-007 observability tests |
