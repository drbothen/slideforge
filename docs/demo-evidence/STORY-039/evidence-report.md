# STORY-039 Evidence Report — PPTX Accessibility Metadata + Layout IR Alt-Threading

**Story ID:** STORY-039  
**Feature branch:** `feature/S-039`  
**Evidence generated:** 2026-06-04  
**Product type:** Library (IR-level) — no CLI demo; evidence is library test harness output + real PPTX XML extraction  
**Tool chain:** `cargo nextest` + `PptxExporter::export` → ZIP → XML extraction

---

## Summary

All 18 tests pass. Evidence is produced from two sources:

1. **14 PPTX a11y tests** (`crates/slideforge-pptx/src/tests/a11y_tests.rs`) — each builds a `LaidOutDeck` programmatically, calls `PptxExporter::export`, opens the resulting ZIP, and asserts extracted XML values.

2. **4 layout alt-threading integration tests** (`crates/slideforge-layout/tests/alt_threading_integration.rs`) — each calls the real `layout::run` with semantic `Deck` blocks containing `ChartSpec`/`DiagramSpec`/`ImageSpec`, and asserts the resulting `FrameContent` carries the correct `AltText` variant.

A standalone example binary (`crates/slideforge-pptx/examples/demo_a11y_evidence.rs`) was added to run all 11 evidence scenarios interactively and dump extracted XML values.

---

## Test Run Results

### slideforge-pptx a11y tests (14/14 PASS)

```
Command: cargo nextest run -p slideforge-pptx -E 'test(a11y)' --no-fail-fast

PASS [0.049s] test_BC_4_01_004_ec005_300_char_alt_exact_length
PASS [0.047s] test_BC_5_01_005_ac007_no_lang_defaults_to_en
PASS [0.047s] test_BC_4_01_004_ac003_300_char_alt_not_truncated
PASS [0.051s] test_BC_4_01_004_ac001_non_decorative_image_has_non_empty_descr
PASS [0.051s] test_BC_4_01_004_ac004_special_chars_xml_escaped_well_formed
PASS [0.052s] test_BC_5_01_005_ac006_dc_language_exact_bcp47_en_us
PASS [0.053s] test_BC_4_01_004_ac005_diagram_frame_alt_on_enclosing_shape
PASS [0.054s] test_BC_4_01_004_ac005_chart_frame_alt_on_enclosing_shape
PASS [0.053s] test_BC_5_01_005_ac006_dc_language_exact_bcp47_zh_hant_tw
PASS [0.056s] test_BC_4_01_004_ec006_chart_none_alt_maps_to_decorative_descr_empty
PASS [0.056s] test_BC_4_01_004_ac002_decorative_image_has_empty_descr_attribute_present
PASS [0.057s] test_BC_4_01_004_ec007_diagram_none_alt_maps_to_decorative_descr_empty
PASS [0.058s] test_BC_4_01_004_ec001_all_decorative_slide
PASS [0.058s] test_BC_4_01_004_ec002_zh_tw_lang_bcp47_embedded

Summary: 14 tests run: 14 passed, 0 failed
```

### slideforge-layout alt-threading integration tests (4/4 PASS)

```
Command: cargo nextest run -p slideforge-layout -E 'test(test_bc_3_06_039)' --no-fail-fast

PASS [0.012s] test_bc_3_06_039_ec006_chart_alt_none_maps_to_decorative
PASS [0.013s] test_bc_3_06_039_ac005_chart_provided_alt_threads_through_layout_run
PASS [0.013s] test_bc_3_06_039_ac005_image_provided_alt_threads_through_layout_run
PASS [0.012s] test_bc_3_06_039_ac005_diagram_provided_alt_threads_through_layout_run

Summary: 4 tests run: 4 passed, 0 failed
```

---

## AC-to-Evidence Mapping

### AC-001: Non-decorative visual → non-empty descr (BC-4.01.004 postcondition 1)

**Test:** `test_BC_4_01_004_ac001_non_decorative_image_has_non_empty_descr`  
**Frame:** `FrameContent::Image { alt: AltText::Provided("Revenue chart Q1 2026") }`  
**Path:** `ppt/slides/slide1.xml`

Extracted XML from real PPTX ZIP:
```xml
<p:cNvPr id="1" name="Image 0" descr="Revenue chart Q1 2026"></p:cNvPr>
```

**Result:** `descr` is non-empty, matches the input alt text exactly. PASS.

**Evidence file:** `AC-001-non-decorative-image-descr.txt`

---

### AC-002: Decorative → descr="" (attribute present, empty) (BC-4.01.004 postcondition 2)

**Test:** `test_BC_4_01_004_ac002_decorative_image_has_empty_descr_attribute_present`  
**Frame:** `FrameContent::Image { alt: AltText::Decorative }`  
**Path:** `ppt/slides/slide1.xml`

Extracted XML from real PPTX ZIP:
```xml
<p:cNvPr id="1" name="Image 0" descr=""></p:cNvPr>
```

**Key invariant:** The `descr` attribute is PRESENT with an empty value. An absent `descr` attribute is different from `descr=""` — OOXML accessibility tools distinguish these.  
**Result:** PASS.

**Evidence file:** `AC-002-decorative-image-descr-empty.txt`

---

### AC-003: 300-char alt not truncated (BC-4.01.004 invariant 1)

**Tests:**  
- `test_BC_4_01_004_ac003_300_char_alt_not_truncated`  
- `test_BC_4_01_004_ec005_300_char_alt_exact_length`  
**Frame:** `FrameContent::Image { alt: AltText::Provided("A" * 300) }`

Measured values:
- Input alt length: **300**
- Extracted `descr` attribute length: **300**
- Truncated: **NO**

Extracted cNvPr (first 80 chars):
```
<p:cNvPr id="1" name="Image 0" descr="AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA...
```

**Result:** Full 300-char string embedded verbatim. XML well-formed after embedding. PASS.

**Evidence file:** `AC-003-300-char-alt-not-truncated.txt`

---

### AC-004: Special chars (& " < >) XML-escaped, well-formed (BC-4.01.004 EC-001)

**Test:** `test_BC_4_01_004_ac004_special_chars_xml_escaped_well_formed`  
**Frame:** `FrameContent::Image { alt: AltText::Provided(r#"Revenue & Cost "Q1" <2026> 'fin'"#) }`

Extracted cNvPr (as serialized in PPTX ZIP — showing escaped bytes):
```xml
<p:cNvPr id="1" name="Image 0" descr="Revenue &amp; Cost &quot;Q1&quot; &lt;2026&gt; &apos;fin&apos;"/>
```

Escaping verified: `&` → `&amp;`, `"` → `&quot;`, `<` → `&lt;`, `'` → `&apos;`  
XML well-formed: `quick_xml::Reader` parses slide1.xml to EOF without error — YES  
Round-trip: unescaped value from `attr.value` equals original input string.

**Result:** PASS.

**Evidence file:** `AC-004-special-chars-xml-escaped.txt`

---

### AC-005: Chart AND Diagram → alt on enclosing pic (REAL frames; name="Chart N"/"Diagram N") (BC-4.01.004 EC-005 / F-039-C2)

**Tests (PPTX exporter):**  
- `test_BC_4_01_004_ac005_chart_frame_alt_on_enclosing_shape`  
- `test_BC_4_01_004_ac005_diagram_frame_alt_on_enclosing_shape`  
**Tests (layout threading):**  
- `test_bc_3_06_039_ac005_chart_provided_alt_threads_through_layout_run`  
- `test_bc_3_06_039_ac005_diagram_provided_alt_threads_through_layout_run`  
- `test_bc_3_06_039_ac005_image_provided_alt_threads_through_layout_run`

**Chart frame** — `FrameContent::Chart { alt: AltText::Provided("Bar chart: Q1 revenue by region") }`:
```xml
<p:cNvPr id="1" name="Chart 0" descr="Bar chart: Q1 revenue by region"></p:cNvPr>
```
- `name="Chart 0"` (starts with "Chart", NOT "Image") — F-039-P-LOW-001 compliance
- `descr="Bar chart: Q1 revenue by region"` on enclosing shape cNvPr

**Diagram frame** — `FrameContent::Diagram { svg: .., alt: AltText::Provided("Flowchart: Q1 deployment pipeline steps") }`:
```xml
<p:cNvPr id="1" name="Diagram 0" descr="Flowchart: Q1 deployment pipeline steps"/>
```
- `name="Diagram 0"` (starts with "Diagram")
- `descr="Flowchart: Q1 deployment pipeline steps"` on enclosing p:pic

**Layout threading** (IR semantic→geometric boundary):

| Test | Input (semantic) | Output (FrameContent) | Result |
|------|-----------------|----------------------|--------|
| Chart | `ChartSpec.alt = Some(Provided("Revenue by region"))` | `Chart { alt: Provided("Revenue by region") }` | PASS |
| Diagram | `DiagramSpec.alt = Some(Provided("Architecture overview"))` | `Diagram { alt: Provided("Architecture overview"), .. }` | PASS |
| Image | `ImageSpec.alt = Some(Provided("Dashboard screenshot"))` | `Image { alt: Provided("Dashboard screenshot") }` | PASS |

**Result:** All 5 tests PASS.

**Evidence files:** `AC-005-chart-diagram-alt-enclosing-shape.txt`, `alt-threading-layout-integration.txt`

---

### AC-006: dc:language exact BCP-47 (BC-5.01.005 postcondition 1)

**Tests:**  
- `test_BC_5_01_005_ac006_dc_language_exact_bcp47_en_us`  
- `test_BC_5_01_005_ac006_dc_language_exact_bcp47_zh_hant_tw`  
- `test_BC_4_01_004_ec002_zh_tw_lang_bcp47_embedded`  
**Path:** `docProps/core.xml`

Extracted XML from real PPTX ZIP:

**en-US case:**
```xml
<dc:language>en-US</dc:language>
```
`find_element_text(&core_xml, "language")` → `"en-US"` (exact, no normalization)

**zh-Hant-TW case (4-part BCP-47):**
```
dc:language value: "zh-Hant-TW"
```
Lossless: no case folding (NOT "zh-tw"), no subtag removal (NOT "zh-Hant"), no truncation.

**Result:** Both exact matches. PASS.

**Evidence file:** `AC-006-dc-language-exact-bcp47.txt`

---

### AC-007: No lang → dc:language "en" (BC-5.01.005 EC-003)

**Test:** `test_BC_5_01_005_ac007_no_lang_defaults_to_en`  
**Input:** `DeckMetadata.lang = None` (not declared)

Extracted value from real PPTX ZIP:
```
dc:language: "en"
```
- Value is `"en"` (bare subtag), NOT `"en-US"`.
- Element IS present (not absent) — postcondition 1 satisfied even in the no-lang case.

**Result:** PASS.

**Evidence file:** `AC-007-no-lang-defaults-en.txt`

---

### EC-006 / EC-007: None alt → AltText::Decorative → descr="" (BC-4.01.004 EC-006 + EC-007)

**Tests (PPTX exporter):**  
- `test_BC_4_01_004_ec006_chart_none_alt_maps_to_decorative_descr_empty`  
- `test_BC_4_01_004_ec007_diagram_none_alt_maps_to_decorative_descr_empty`  
**Test (layout):**  
- `test_bc_3_06_039_ec006_chart_alt_none_maps_to_decorative`

**Chart with `AltText::Decorative`** (simulating `ChartSpec.alt = None` path):
```xml
<p:cNvPr id="1" name="Chart 0" descr=""/>
```
- `descr=""` PRESENT (attribute present, empty value — not absent)
- `name="Chart 0"` (not "Image 0")

**Diagram with `AltText::Decorative`** (simulating `DiagramSpec.alt = None` path):
```xml
<p:cNvPr id="1" name="Diagram 0" descr=""/>
```

**Layout threading** for EC-006:  
`ChartSpec { alt: None }` → `layout::run` → `FrameContent::Chart { alt: AltText::Decorative }` (safe sentinel + `tracing::warn!`)

**Result:** All 3 tests PASS.

**Evidence file:** `EC-006-007-none-alt-maps-to-decorative.txt`

---

## Coverage Summary

| AC / EC | Criterion | Tests Passing | Concrete extracted value |
|---------|-----------|---------------|--------------------------|
| AC-001 | Non-decorative image → non-empty descr | 1 PASS | `descr="Revenue chart Q1 2026"` |
| AC-002 | Decorative → `descr=""` (present, empty) | 1 PASS | `descr=""` (present) |
| AC-003 | 300-char alt not truncated | 2 PASS | extracted descr length = 300 |
| AC-004 | Special chars XML-escaped, well-formed | 1 PASS | `&amp;`, `&quot;`, `&lt;`, `&apos;` in serialized XML |
| AC-005 chart | Chart alt on enclosing shape, `name="Chart 0"` | 2 PASS | `name="Chart 0" descr="Bar chart: Q1 revenue by region"` |
| AC-005 diagram | Diagram alt on enclosing shape, `name="Diagram 0"` | 2 PASS | `name="Diagram 0" descr="Flowchart: Q1 deployment pipeline steps"` |
| AC-005 threading | layout::run threads Chart/Diagram/Image alt | 3 PASS | `FrameContent::Chart { alt: Provided("Revenue by region") }` |
| AC-006 en-US | dc:language = "en-US" exact | 1 PASS | `<dc:language>en-US</dc:language>` |
| AC-006 zh-Hant-TW | dc:language = "zh-Hant-TW" exact | 2 PASS | `dc:language = "zh-Hant-TW"` |
| AC-007 | No lang → dc:language = "en" | 1 PASS | `dc:language = "en"` |
| EC-006 | Chart none alt → `descr=""` | 2 PASS | `name="Chart 0" descr=""` |
| EC-007 | Diagram none alt → `descr=""` | 1 PASS | `name="Diagram 0" descr=""` |

**Total: 18 tests across 2 crates — 18 PASS, 0 FAIL**

---

## Evidence Files

| File | Content |
|------|---------|
| `AC-001-non-decorative-image-descr.txt` | AC-001 extracted cNvPr, test result |
| `AC-002-decorative-image-descr-empty.txt` | AC-002 extracted cNvPr with `descr=""`, invariant explanation |
| `AC-003-300-char-alt-not-truncated.txt` | AC-003 + EC-005 length measurement, test results |
| `AC-004-special-chars-xml-escaped.txt` | AC-004 escaping map, well-formedness check |
| `AC-005-chart-diagram-alt-enclosing-shape.txt` | AC-005 chart + diagram + layout threading evidence |
| `AC-006-dc-language-exact-bcp47.txt` | AC-006 en-US + zh-Hant-TW extracted values |
| `AC-007-no-lang-defaults-en.txt` | AC-007 None → "en" default |
| `EC-006-007-none-alt-maps-to-decorative.txt` | EC-006 + EC-007 chart/diagram Decorative path |
| `alt-threading-layout-integration.txt` | All 4 layout threading tests with threading contracts |
| `xml-samples-from-real-pptx.txt` | 10 raw XML samples extracted from real PPTX output |
| `test-run-log-pptx-a11y.txt` | Full nextest run log — 14 PPTX a11y tests |
| `test-run-log-layout-alt-threading.txt` | Full nextest run log — 4 layout threading tests |
| `evidence-report.md` | This file |

---

## Implementation Notes

This story is a library/IR-level story. The end-to-end `.sf → .pptx` pipeline does not yet produce Chart/Diagram content from source DSL (those renderers are future scope), so evidence uses the library API directly:

- `PptxExporter::export(deck, laid_out, brand, opts)` is called with programmatically-constructed `LaidOutDeck` fixtures.
- The real ZIP is opened and XML parts are parsed by `quick_xml` to extract `descr` attribute values and `dc:language` text content.
- No mock strings are used — all extracted values come from bytes written by the real exporter.

The `AltTextEmbedder` component (`crates/slideforge-pptx/src/a11y.rs`) is the production module that drives all `descr` attribute decisions. Its `decisions_for_slide` method maps `FrameContent` variants to `AltDecision::Provided` / `AltDecision::Decorative` and is exercised by every PPTX a11y test.
