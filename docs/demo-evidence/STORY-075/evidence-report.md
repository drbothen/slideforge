# Demo Evidence Report — STORY-075

**Story:** Brand Loader: Footer Detection from .pptx Slide Master/Layout `<p:hf>`
**Story ID:** STORY-075
**Status:** 3/3 strict-CLEAN (converged)
**Demo method:** VHS terminal recordings of focused `cargo nextest` runs exercising the production code path in `slideforge-brand`
**Evidence location:** `docs/demo-evidence/STORY-075/` (feature branch `feature/S-075`)
**Recorded:** 2026-06-01
**Test suite result:** 260 tests run, 260 passed, 0 skipped, 0 failed

---

## Coverage Summary

| AC | Recording | Tests Demonstrated | Result |
|----|-----------|-------------------|--------|
| AC-001 (footer text from master) | `AC-001-footer-text-from-master.gif/.webm` | `test_bc_2_01_001_detect_footer_text_from_master`, `test_bc_2_01_001_brand_template_has_footer_flags_field` | PASS |
| AC-001 (multi-run concatenation) | `AC-001-multirun-concatenation.gif/.webm` | `test_bc_2_01_001_detect_footer_text_multirun` | PASS |
| AC-002 (layout fallback) | `AC-002-layout-fallback.gif/.webm` | `test_bc_2_01_001_detect_footer_empty_master_fallback_layout`, `test_bc_2_01_001_detect_footer_empty_master_no_layout_yields_none`, `test_bc_2_01_001_detect_footer_both_empty_yields_none` | PASS |
| AC-003 (`<p:hf>` visibility flags) | `AC-003-hf-element-visibility-flags.gif/.webm` | `test_bc_2_01_001_detect_footer_flags_from_hf_element`, `test_bc_2_01_001_detect_footer_flags_all_three_from_hf_element`, `test_bc_2_01_001_v1_3_canonical_vector_ftr1_dt1_sldnum0`, `test_bc_2_01_001_detect_footer_flags_hf_boolean_string_form`, `test_bc_2_01_001_detect_footer_flags_hf_partial_attrs`, `test_bc_2_01_001_detect_footer_flags_absent_hf_element` | PASS |
| AC-004 (fields populated before return) | `AC-004-005-006-integration-paths.gif/.webm` | `test_bc_2_01_001_load_pptx_with_footer`, `test_bc_2_01_001_load_pptx_without_footer`, `test_bc_2_01_001_load_pptx_footer_flags_populated_from_hf_element` | PASS |
| AC-005 (absent slideMaster1.xml — silent skip) | `AC-004-005-006-integration-paths.gif/.webm` | `test_bc_2_01_001_detect_footer_absent_master_xml` | PASS |
| AC-006 (DOCX always default) | `AC-004-005-006-integration-paths.gif/.webm` | `test_bc_2_01_001_detect_footer_docx_returns_default`, `test_bc_2_01_001_load_docx_footer_always_none` | PASS |
| AC-007 (end-to-end [footer] section in brand.toml) | `AC-007-extract-footer-section.gif/.webm` | `test_bc_2_01_001_extract_brand_toml_includes_footer_section` | PASS |
| AC-008 (NFR compliance) | `AC-008-nfr-compliance.gif/.webm` | `cargo clippy -p slideforge-brand --all-targets --all-features -- -D warnings` exits 0 | PASS |

---

## Per-AC Detail

### AC-001: Footer text from master + multi-run concatenation
**Traces to:** BC-2.01.001 postcondition 1 v1.3; adversary M-1 clarification

**Recording 1:** `AC-001-footer-text-from-master.gif/.webm`

Exercises the production path: `detect_footer()` in `footer.rs` opens
`ppt/slideMasters/slideMaster1.xml` from the in-memory ZIP, finds
`<p:ph type="ftr"/>`, collects `<a:t>Confidential</a:t>`, and returns
`FooterDetection { text: Some("Confidential"), ... }`. The `BrandTemplate`
struct is confirmed to carry the new `footer_flags: FooterFlags` field.

**Recording 2:** `AC-001-multirun-concatenation.gif/.webm`

Exercises EC-008 / adversary M-1: two `<a:r>` runs `"Acme Confidential "` +
`"2026"` are concatenated to `"Acme Confidential 2026"`. First-run-only
semantics are forbidden — the SAX state machine pushes all runs into a shared
`sp_text_accum` before trimming.

Both recordings show `1 test run: 1 passed` in nextest output.

---

### AC-002: Layout fallback when master placeholder has no text
**Traces to:** BC-2.01.001 postcondition 1 (best-effort detection covers master + layout)

**Recording:** `AC-002-layout-fallback.gif/.webm`

Three tests demonstrate the three outcomes:
1. `test_bc_2_01_001_detect_footer_empty_master_fallback_layout` — master has empty
   `<a:t>`, layout1.xml has `"Q4 Report"` → `footer_text = Some("Q4 Report")`.
2. `test_bc_2_01_001_detect_footer_empty_master_no_layout_yields_none` — master empty,
   layout1.xml absent → `footer_text = None`.
3. `test_bc_2_01_001_detect_footer_both_empty_yields_none` — both master and layout
   have empty text runs → `footer_text = None`.

Recording shows `3 tests run: 3 passed`.

---

### AC-003: `<p:hf>` boolean attributes for footer visibility flags
**Traces to:** BC-2.01.001 postcondition 1 v1.3 corrected (adversary H-1)

**Recording:** `AC-003-hf-element-visibility-flags.gif/.webm`

Six tests cover the corrected `<p:hf>` source (NOT `presProps.xml`):
- `ftr="1" sldNum="1"` (no dt attr) → `{show_footer: true, show_date: false, show_slide_number: true}`
- `ftr="1" dt="1" sldNum="1"` → all three true (BC-2.01.001 v1.3 canonical test vector)
- `ftr="1" dt="1" sldNum="0"` → `{show_footer: true, show_date: true, show_slide_number: false}` (v1.3 canonical vector)
- `ftr="true" dt="true" sldNum="false"` → bool string form accepted per ECMA-376
- `<p:hf hdr="0">` (all three attrs absent) → all false (EC-007)
- `<p:hf>` element absent entirely → `FooterFlags::default()` all false (EC-004)

Recording shows `9 tests run: 9 passed` (the filter `test(footer_flags)` matches
all footer_flags tests including the struct/default tests).

**Key invariant demonstrated:** flags come from `<p:hf>` on `slideMaster1.xml`.
No `presProps.xml` is present in any of the test ZIPs; all flag tests pass
regardless — proving the corrected implementation reads the right element.

---

### AC-004: footer_text and footer_flags populated before BrandTemplate is returned
**Traces to:** BC-2.01.001 postcondition 1 (BrandTemplate complete on return)

**Recording:** `AC-004-005-006-integration-paths.gif/.webm` (shared with AC-005/006)

`test_bc_2_01_001_load_pptx_with_footer` calls `BrandLoader::load()` end-to-end
on a PPTX fixture ZIP containing `"Acme Corp Confidential"` in the master footer
placeholder and `<p:hf ftr="1"/>`. The returned `BrandTemplate` has:
- `footer_text = Some("Acme Corp Confidential")`
- `footer_flags.show_footer = true`

`test_bc_2_01_001_load_pptx_without_footer` confirms the STORY-022 PPTX fixture
(no footer placeholder) still returns `footer_text = None` (unchanged behavior).

`test_bc_2_01_001_load_pptx_footer_flags_populated_from_hf_element` asserts
flags are present on the same `BrandTemplate` that carries footer text — proving
both fields are populated in the same `BrandLoader::load()` call.

---

### AC-005: Absent slideMaster1.xml — silent skip, no panic
**Traces to:** BC-2.01.001 invariant 2 (read-only; does not affect build exit code)

**Recording:** `AC-004-005-006-integration-paths.gif/.webm` (shared)

`test_bc_2_01_001_detect_footer_absent_master_xml` builds a PPTX ZIP with no
`ppt/slideMasters/slideMaster1.xml` entry. `detect_footer()` returns
`FooterDetection::default()` without panicking and a `tracing::debug!` is emitted
(EC-001). Test passes with no panic assertion.

---

### AC-006: DOCX format — always default, no XML reads
**Traces to:** BC-2.01.001 postcondition 1 (DOCX has no footer in brand loading scope)

**Recording:** `AC-004-005-006-integration-paths.gif/.webm` (shared)

`test_bc_2_01_001_detect_footer_docx_returns_default` calls `detect_footer(zip, false)`
(DOCX path). The function returns immediately on the `if !is_pptx` guard with
`FooterDetection::default()` — no ZIP reads, no XML parsing.

`test_bc_2_01_001_load_docx_footer_always_none` verifies the full `BrandLoader::load()`
path on a DOCX fixture: `footer_text = None`, `footer_flags = FooterFlags::default()`.

---

### AC-007: End-to-end [footer] section in brand.toml (closes F-024A-OBS-1)
**Traces to:** BC-2.01.001 postcondition 1 cross-story — activates STORY-024's dormant [footer] writer

**Recording:** `AC-007-extract-footer-section.gif/.webm`

`test_bc_2_01_001_extract_brand_toml_includes_footer_section` exercises the full
cross-story path:
1. Build PPTX fixture ZIP with footer placeholder text `"Acme Corp Confidential"`.
2. Call `BrandLoader::load()` → `BrandTemplate { footer_text: Some("Acme Corp Confidential"), ... }`.
3. Pass `BrandTemplate` to `BrandExtractor::extract()` (STORY-024 component).
4. Assert the resulting `brand.toml` string contains:
   ```toml
   [footer]
   text = "Acme Corp Confidential"
   ```

Recording shows `1 test run: 1 passed`. This test was previously unreachable
(permanently dead code path in STORY-024's extractor) because `footer_text` was
always `None`. After STORY-075, the `[footer]` writer is live.

---

### AC-008: NFR compliance — clippy pedantic clean, no unsafe, docs present
**Traces to:** NFR-021, NFR-022, NFR-023, NFR-024, NFR-025

**Recording:** `AC-008-nfr-compliance.gif/.webm`

`cargo clippy -p slideforge-brand --all-targets --all-features -- -D warnings`
exits 0 with no warnings or errors. The recording shows the final `Finished`
line, confirming:
- `#![forbid(unsafe_code)]` active (NFR-024)
- `#![warn(missing_docs)]` satisfied on all public items (NFR-023)
- `clippy::pedantic` clean (NFR-022)
- `=` version pinning maintained in `Cargo.toml` (NFR-025, not shown in recording but
  enforced by the same `cargo deny` / workspace Cargo.toml checks)

---

## Error-Path Coverage

Each AC includes both a success path and at least one error/edge path:

| Path | Test | Outcome |
|------|------|---------|
| No footer placeholder | `test_bc_2_01_001_detect_footer_absent_placeholder` | `text = None` (not an error) |
| Empty footer text in master, layout has text | `test_bc_2_01_001_detect_footer_empty_master_fallback_layout` | `text = Some("Q4 Report")` from layout |
| Empty footer text in master, no layout | `test_bc_2_01_001_detect_footer_empty_master_no_layout_yields_none` | `text = None` |
| Both master and layout empty | `test_bc_2_01_001_detect_footer_both_empty_yields_none` | `text = None` |
| `slideMaster1.xml` absent from ZIP | `test_bc_2_01_001_detect_footer_absent_master_xml` | `text = None`, no panic, debug log |
| `<p:hf>` element absent | `test_bc_2_01_001_detect_footer_flags_absent_hf_element` | `FooterFlags::default()`, debug log |
| `<p:hf>` present but all attrs absent (EC-007) | `test_bc_2_01_001_detect_footer_flags_hf_partial_attrs` | all flags false, no error |
| Footer placeholder has `<a:fld>` only (EC-006) | `test_bc_2_01_001_detect_footer_field_element_treated_as_no_text` | `text = None` for that placeholder |
| Multiple footer placeholders (EC-003) | `test_bc_2_01_001_detect_footer_multiple_placeholders_first_wins` | first placeholder text wins |
| DOCX input | `test_bc_2_01_001_detect_footer_docx_returns_default`, `test_bc_2_01_001_load_docx_footer_always_none` | `None`/defaults, no XML reads |

---

## Recordings Index

| File | Size | AC(s) |
|------|------|-------|
| `AC-001-footer-text-from-master.gif` | 128 KB | AC-001 (success path: footer text detected) |
| `AC-001-footer-text-from-master.webm` | 281 KB | AC-001 (success path: footer text detected) |
| `AC-001-footer-text-from-master.tape` | VHS source | AC-001 |
| `AC-001-multirun-concatenation.gif` | 121 KB | AC-001 (multi-run concatenation, EC-008) |
| `AC-001-multirun-concatenation.webm` | 219 KB | AC-001 (multi-run concatenation, EC-008) |
| `AC-001-multirun-concatenation.tape` | VHS source | AC-001 |
| `AC-002-layout-fallback.gif` | 182 KB | AC-002 (layout fallback, empty/absent paths) |
| `AC-002-layout-fallback.webm` | 347 KB | AC-002 (layout fallback, empty/absent paths) |
| `AC-002-layout-fallback.tape` | VHS source | AC-002 |
| `AC-003-hf-element-visibility-flags.gif` | 207 KB | AC-003 (`<p:hf>` flags, 6 variants including EC-004/EC-007) |
| `AC-003-hf-element-visibility-flags.webm` | 561 KB | AC-003 (`<p:hf>` flags) |
| `AC-003-hf-element-visibility-flags.tape` | VHS source | AC-003 |
| `AC-004-005-006-integration-paths.gif` | 266 KB | AC-004, AC-005, AC-006 (integration paths) |
| `AC-004-005-006-integration-paths.webm` | 517 KB | AC-004, AC-005, AC-006 |
| `AC-004-005-006-integration-paths.tape` | VHS source | AC-004, AC-005, AC-006 |
| `AC-007-extract-footer-section.gif` | 127 KB | AC-007 (end-to-end [footer] in brand.toml) |
| `AC-007-extract-footer-section.webm` | 231 KB | AC-007 |
| `AC-007-extract-footer-section.tape` | VHS source | AC-007 |
| `AC-008-nfr-compliance.gif` | 82 KB | AC-008 (clippy clean, NFR-021..025) |
| `AC-008-nfr-compliance.webm` | 179 KB | AC-008 |
| `AC-008-nfr-compliance.tape` | VHS source | AC-008 |

---

## Commit Status

These files are UNCOMMITTED at demo-recorder handoff. The state-manager will commit
`docs/demo-evidence/STORY-075/` in the post-merge serialized burst to avoid
index-lock races with other in-flight `.factory/` writers.
