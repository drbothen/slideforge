# Evidence Report — STORY-041: DOCX Core Serialization

**Story:** STORY-041 — DOCX Core Serialization: report register + ooxmlsdk  
**Crate:** `slideforge-docx`  
**Behavioral Contract:** BC-4.02.001  
**Status:** CONVERGED (adversary 3/3 clean passes)  
**Demo type:** Library/test-harness — VHS recording of `cargo run --example demo_docx`  
**Generated:** 2026-06-03

---

## Tool availability

| Tool | Available | Notes |
|------|-----------|-------|
| VHS | Yes — `/opt/homebrew/bin/vhs` | Used for terminal recording |
| LibreOffice headless | No | Not installed on this machine; CI gate covers `.docx`-to-PDF conversion check |

---

## Recordings

| File | Size | Description |
|------|------|-------------|
| `AC-001-009-docx-core-serialization.gif` | 187 KB | VHS-generated animated GIF — all 9 ACs in one run |
| `AC-001-009-docx-core-serialization.webm` | 295 KB | VHS-generated WebM video — archival format |
| `AC-001-009-docx-core-serialization.tape` | 751 B | VHS tape script source |
| `demo_docx_stdout.log` | 2.0 KB | Captured stdout from `cargo run --example demo_docx` |

---

## Acceptance Criterion Coverage

### AC-001: Exporter plugin trait implemented

**Requirement:** `DocxExporter` implements `Exporter` via the plugin trait API; `id()` and `extension()` match.

**Evidence:**
- Example file: `crates/slideforge-docx/examples/demo_docx.rs` — calls `DocxExporter.id()` and `DocxExporter.extension()` directly, asserts `"docx"` and `"docx"`
- Recording: AC-001-009 GIF/WebM — "AC-001: Exporter plugin trait" section with `PASS` output
- Test: `test_BC_4_02_001_exporter_trait_id_and_extension` in `src/tests/core_tests.rs`

**stdout excerpt:**
```
=== AC-001: Exporter plugin trait ===
  id()        = "docx"
  extension() = "docx"
  PASS
```

---

### AC-002: Valid .docx ZIP produced with all required parts

**Requirement:** A 1-slide deck produces a `.docx` ZIP containing `word/document.xml`, `word/styles.xml`, `word/_rels/document.xml.rels`, `[Content_Types].xml`, `_rels/.rels`, `docProps/core.xml`.

**Evidence:**
- Example: enumerates ZIP parts and asserts all required paths present
- Recording: AC-001-009 GIF/WebM — "AC-002: ZIP part list" section with all parts listed
- Test: `test_BC_4_02_001_zip_contains_required_parts` in `src/tests/core_tests.rs`

**stdout excerpt:**
```
=== AC-002: ZIP part list ===
  [Content_Types].xml
  _rels/.rels
  docProps/app.xml
  docProps/core.xml
  word/_rels/document.xml.rels
  word/document.xml
  word/numbering.xml
  word/settings.xml
  word/styles.xml
  PASS — all required parts present
```

---

### AC-003: report register content in document.xml body

**Requirement:** A slide with `report "Analysis follows."` produces a `<w:p>` with style `Normal` containing the text in `word/document.xml`.

**Evidence:**
- Example: searches `document.xml` for `"Analysis"` and asserts presence
- Recording: AC-001-009 GIF/WebM — "AC-003 + AC-004" section
- Test: `test_BC_4_02_001_report_register_in_document_xml`

**stdout excerpt:**
```
  Report content offset = 391
  PASS — Heading1 at 255 before report at 391
  Heading1 context: ...Pr><w:pStyle w:val="Heading1" /></w:pPr><w:r><w:t>Q1 Review</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Normal" /></w:pPr><w:r><w:rPr><w:b...
```

---

### AC-004: Slide title as Heading1 before report content

**Requirement:** Slide title produces `<w:p style="Heading1">` BEFORE the report paragraph.

**Evidence:**
- Example: asserts `heading1_pos < report_pos` (offsets 255 and 391 respectively)
- Recording: AC-001-009 GIF/WebM — "AC-003 + AC-004" section showing offset ordering
- Test: `test_BC_4_02_001_slide_title_as_heading1_before_report`

**stdout excerpt:**
```
  Heading1 offset = 255
  Report content offset = 391
  PASS — Heading1 at 255 before report at 391
```

---

### AC-005: notes register content absent from document.xml body

**Requirement:** A deck with `notes "NOTES_DOCX_SENTINEL"` produces a `document.xml` that does NOT contain "NOTES_DOCX_SENTINEL".

**Evidence:**
- Example: asserts `!doc_xml.contains("NOTES_DOCX_SENTINEL")`
- Recording: AC-001-009 GIF/WebM — "AC-005: notes sentinel absent" section
- Test: `test_BC_4_02_001_notes_sentinel_absent_from_docx_body` (un-ignored STORY-036 bleed test)

**stdout excerpt:**
```
=== AC-005: notes sentinel absent from document.xml ===
  PASS — NOTES_DOCX_SENTINEL is absent from document.xml
```

---

### AC-006: report register content present (positive bleed test)

**Requirement:** A deck with `report "REPORT_DOCX_SENTINEL"` produces a `document.xml` that DOES contain "REPORT_DOCX_SENTINEL".

**Evidence:**
- Example: asserts `doc_xml.contains("REPORT_DOCX_SENTINEL")`
- Recording: AC-001-009 GIF/WebM — "AC-006: report sentinel present" section
- Test: `test_BC_4_02_001_report_sentinel_present_in_docx_body`

**stdout excerpt:**
```
=== AC-006: report sentinel present in document.xml ===
  PASS — REPORT_DOCX_SENTINEL is present in document.xml
```

---

### AC-007: detail register content in extended section of document.xml

**Requirement:** `detail` content appears in an extended section (under `Heading2 "Appendix: <title>"`) AFTER all report body sections.

**Evidence:**
- Example: asserts `report_pos < appendix_pos < detail_text_pos`
- Recording: AC-001-009 GIF/WebM — "AC-007: detail in extended section" section
- Test: `test_BC_4_02_001_detail_in_extended_section_after_report`

**stdout excerpt:**
```
=== AC-007: detail in extended section ===
  Appendix heading offset = 993
  Detail text offset      = 1087
  Report content offset   = 391
  PASS — detail content appears in extended section after report
  Appendix context: ...<w:r><w:t>Appendix: Q1 Review</w:t></w:r></w:p>...
```

---

### AC-008: Inline formatting mapped to Word run properties

**Requirement:** Bold → `<w:b/>`, Italic → `<w:i/>`, Link → `<w:hyperlink r:id="...">` with `Hyperlink` char style.

**Evidence:**
- Example: asserts bold (`<w:b/>` or `<w:b />`), italic (`<w:i/>` or `<w:i />`), and hyperlink element presence; prints hyperlink rel entry from `word/_rels/document.xml.rels`
- Recording: AC-001-009 GIF/WebM — "AC-008: inline formatting" section
- Test: `test_BC_4_02_001_inline_bold_italic_run_properties`, `test_BC_4_02_001_f_docx_001_hyperlink_doc_declares_xmlns_r_and_parses_back`

**stdout excerpt:**
```
=== AC-008: inline formatting ===
  Bold run property present: true
  Italic run property present: true
  Hyperlink element present: true
  PASS — bold, italic, hyperlink all mapped to Word run properties
  Hyperlink rel entry: ...hyperlink" Target="https://slideforge.rs" TargetMode="External"/>
```

---

### AC-009: Slide with no report/detail content has heading + empty paragraph

**Requirement:** A visual-only slide (no register content) produces `Heading1` + empty `<w:p/>`.

**Evidence:**
- Example: Slide 2 ("Closing Remarks") has no `register_content`; asserts "Closing Remarks" appears in `document.xml` with no notes bleed
- Recording: AC-001-009 GIF/WebM — "AC-009: visual-only slide" section
- Test: `test_BC_4_02_001_visual_only_slide_heading_plus_empty_para`

**stdout excerpt:**
```
=== AC-009: visual-only slide heading + empty paragraph ===
  'Closing Remarks' Heading1 offset = 890
  PASS — visual-only slide has Heading1; no register bleed into body
```

---

## dc:language (BC-4.02.001 postcondition — docProps/core.xml)

**stdout excerpt:**
```
=== dc:language in core.xml ===
  dc:language excerpt: ...chema-instance">
<dc:language>en-US</dc:language>
</cp:coreProperties>...
  PASS — dc:language = en-US
```

---

## LibreOffice headless skip

LibreOffice was not found on the demo machine (`which libreoffice` returned not found).
The CI gate (`ci.yml` `libreoffice-render` job) covers `.docx`-to-PDF conversion and
visual verification. No action required for per-story demo evidence.

---

## Coverage summary

| AC | Requirement | Status | Evidence artifact |
|----|-------------|--------|-------------------|
| AC-001 | Exporter trait impl | PASS | example + recording + `test_BC_4_02_001_exporter_trait_id_and_extension` |
| AC-002 | Valid ZIP with all required parts | PASS | example + recording + `test_BC_4_02_001_zip_contains_required_parts` |
| AC-003 | report content in document.xml | PASS | example + recording + `test_BC_4_02_001_report_register_in_document_xml` |
| AC-004 | Heading1 before report | PASS | example + recording + `test_BC_4_02_001_slide_title_as_heading1_before_report` |
| AC-005 | notes absent (bleed test) | PASS | example + recording + `test_BC_4_02_001_notes_sentinel_absent_from_docx_body` |
| AC-006 | report present (positive bleed) | PASS | example + recording + `test_BC_4_02_001_report_sentinel_present_in_docx_body` |
| AC-007 | detail in extended section | PASS | example + recording + `test_BC_4_02_001_detail_in_extended_section_after_report` |
| AC-008 | Inline formatting → Word run props | PASS | example + recording + inline tests |
| AC-009 | Visual-only slide → heading + empty para | PASS | example + recording + `test_BC_4_02_001_visual_only_slide_heading_plus_empty_para` |

All 9 acceptance criteria have recorded demo evidence. Coverage: 9/9 (100%).
