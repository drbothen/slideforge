# STORY-042 Demo Evidence Report

**Story:** DOCX: Auto-Generated Document Sections  
**Story ID:** STORY-042  
**Crate:** `slideforge-docx`  
**Behavioral contract:** BC-4.02.002  
**Recording method:** VHS terminal recording of `demo_sections` runnable example  
**Convergence status:** CONVERGED (adversary 3/3 clean)

---

## Recording Inventory

| File | Format | Purpose |
|------|--------|---------|
| `AC-ALL-demo-sections.gif` | Animated GIF (202 KB) | PR embed — all 8 ACs in one run |
| `AC-ALL-demo-sections.webm` | WebM (269 KB) | Archival recording |
| `AC-ALL-demo-sections.tape` | VHS tape script | Recording source |
| `../../crates/slideforge-docx/examples/demo_sections.rs` | Rust example | Live-execution evidence source |

---

## Acceptance Criteria Coverage

### AC-001: executive_summary section from takeaway fields
**Traces to:** BC-4.02.002 postcondition 2

**Demonstrated by:** `demo_sections.rs` section "=== AC-001 ==="`

The example builds a `GeneratedSection` with `SectionKind::ExecutiveSummary` and two
`SectionItem::TakeawayBullet` items ("Revenue grew 12%..." and "Competitive pressure
increasing."). After export, `word/document.xml` is inspected:

```
  'Executive Summary' heading at offset 896
  Takeaway 1: 'Revenue grew 12%'               PRESENT
  Takeaway 2: 'Competitive pressure increasing' PRESENT
  PASS
```

**Unit test:** `test_BC_4_02_002_ac001_executive_summary_from_takeaway` — PASS  
**Canonical vector test:** `test_BC_4_02_002_executive_summary_two_takeaway_bullets` — PASS

---

### AC-002: risk_register table from severity_cards slides
**Traces to:** BC-4.02.002 postcondition 3

**Demonstrated by:** `demo_sections.rs` section "=== AC-002 ==="`

The example builds a 3-row risk register (HIGH/MED/LOW). The exported `document.xml`
is asserted to contain:

```
  'Risk Register' heading              PRESENT
  <w:tbl> element                      PRESENT
  <w:tblGrid> + 3 <w:gridCol>          PRESENT (ECMA-376 compliant)
  Table rows: 4 (1 header + 3 data)    CORRECT
  Risk/Severity/Description columns    VERIFIED in header row
  PASS
```

The `<w:tblGrid>` with 3 `<w:gridCol>` elements satisfies the ECMA-376 CT_Tbl
`minOccurs=1` requirement (F-042-P1-001).

**Unit test:** `test_BC_4_02_002_ac002_risk_register_table_row_count` — PASS  
**Canonical vector test:** `test_BC_4_02_002_risk_register_three_rows_severity_labels` — PASS  
**OOXML parse-back test:** `test_BC_4_02_002_ooxml_parse_back_document_with_sections` — PASS

---

### AC-003: manually-authored section at declared position
**Traces to:** BC-4.02.002 postcondition 4

**Demonstrated by:** `demo_sections.rs` section "=== AC-003 ==="`

The example includes a `GeneratedSection` with `SectionSource::ManuallyAuthored`, heading
"Methodology", and report content "mixed-methods". Assertions confirm both heading and body
text appear in `document.xml`:

```
  'Methodology' Heading1               PRESENT
  Manual section body text             PRESENT
  PASS
```

**Unit test:** `test_BC_4_02_002_ac003_manual_section_heading_present` — PASS

---

### AC-004: section ordering (narrative → auto-generated → manually-authored)
**Traces to:** BC-4.02.002 postcondition 5

**Demonstrated by:** `demo_sections.rs` section "=== AC-004 ==="`

Byte-offset ordering in `document.xml` is asserted for three heading types:

```
  Narrative heading ('Q1 Market Analysis') at offset 285
  Executive Summary (auto)             at offset 896
  Risk Register (auto)                 at offset 1230
  Methodology (manual)                 at offset 2744
  Order: narrative(285) < auto(896/1230) < manual(2744)
  NOTES_SENTINEL                       ABSENT from document.xml
  PASS
```

The `notes` register sentinel is verified absent — notes content must never appear in
the DOCX body (BC-4.02.001 invariant 1 / DI-012).

**Unit test:** `test_BC_4_02_002_ac004_section_ordering_narrative_auto_manual` — PASS  
**Orderer unit test:** `test_BC_4_02_002_section_orderer_auto_before_manual` — PASS

---

### AC-005: auto-section absent when no source slides
**Traces to:** BC-4.02.002 invariant 2

**Demonstrated by:** `demo_sections.rs` section "=== AC-005 + AC-006 ===" (empty-sections export)

A second export with `sections: vec![]` (no auto-generated sections) confirms that
`Risk Register` and `<w:tbl>` are absent from `document.xml`:

```
  No severity_cards: 'Risk Register' absent        PASS (AC-005)
  No severity_cards: <w:tbl> absent               PASS (AC-005)
```

**Unit test:** `test_BC_4_02_002_ac005_risk_register_absent_when_no_severity_cards` — PASS  
**BleedChecker assertion:** `BleedChecker::assert_absent_from_docx_body` — PASS

---

### AC-006: executive_summary absent when all takeaway fields empty
**Traces to:** BC-4.02.002 edge case EC-004

**Demonstrated by:** `demo_sections.rs` section "=== AC-005 + AC-006 ===" (same empty export)

The same empty-sections export verifies "Executive Summary" is absent:

```
  No takeaways: 'Executive Summary' absent         PASS (AC-006)
```

**Unit test:** `test_BC_4_02_002_ac006_executive_summary_absent_when_all_takeaways_empty` — PASS  
**BleedChecker assertion:** `BleedChecker::assert_absent_from_docx_body` — PASS

---

### AC-007: manual section NOT merged with auto-generated section
**Traces to:** BC-4.02.002 invariant 3

**Demonstrated by:** `demo_sections.rs` section "=== AC-007 ==="`

`count_heading1_paragraphs_with_text` is used to count Heading1 paragraphs whose
`<w:t>` text is exactly "Methodology". Result must be exactly 1 — no spurious
duplicate from a merge bug:

```
  Heading1 paragraphs containing 'Methodology': 1
  (No auto-methodology rule exists — no spurious duplicate)
  PASS
```

**Unit test:** `test_BC_4_02_002_ac007_manual_section_not_merged_with_auto` — PASS

---

### AC-008: 20 severity_cards slides → 20-row table
**Traces to:** BC-4.02.002 edge case EC-005

**Demonstrated by:** `demo_sections.rs` section "=== AC-008 ==="`

A third export with 20 `SectionItem::RiskRow` entries asserts `<w:tr>` count == 21
(1 header + 20 data rows):

```
  20 RiskRow items → <w:tr> count: 21 (1 header + 20 data)
  PASS
```

**Unit test:** `test_BC_4_02_002_ac008_twenty_severity_cards_twenty_rows` — PASS

---

## Additional Covered Requirements

| Requirement | Test | Status |
|-------------|------|--------|
| `TableGrid` and `TableHeader` in `styles.xml` | `test_BC_4_02_002_styles_xml_contains_table_styles` | PASS |
| No dangling `<w:basedOn>` refs in styles.xml | `test_BC_4_02_002_styles_xml_no_dangling_based_on` | PASS |
| OOXML structural validity (ooxmlsdk parse-back with sections) | `test_BC_4_02_002_ooxml_parse_back_document_with_sections` | PASS |
| `<w:tblGrid>` with 3 `<w:gridCol>` (F-042-P1-001) | embedded in parse-back test | PASS |
| `<w:tblGrid>` precedes first `<w:tr>` (ECMA-376 element ordering) | embedded in parse-back test | PASS |
| Empty auto-section orderer guard (invariant 2) | `test_BC_4_02_002_section_orderer_empty_auto_not_returned` | PASS |
| `SectionOrderer::ordered_sections` auto-before-manual | `test_BC_4_02_002_section_orderer_auto_before_manual` | PASS |

---

## Complete Test Run

All 58 tests in `slideforge-docx` pass:

```
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## Recording Notes

- **Product type:** Library crate (CLI binary does not yet exist at Wave 4) — demo
  uses a runnable `cargo run --example demo_sections` approach as specified by the
  task instruction.
- **VHS recording method:** Pre-built binary executed directly to avoid VHS
  `Wait+Line` timeout on cold `cargo build`. The binary at
  `target/debug/examples/demo_sections` is identical to the output of
  `cargo run --example demo_sections -p slideforge-docx`.
- **Font used:** `FiraCode Nerd Font Mono` (detected via `fc-list`).
- **Theme:** Catppuccin Mocha.
