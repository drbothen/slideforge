# STORY-040 Evidence Report
## PPTX Speaker Notes + notesMaster1.xml + handoutMaster1.xml

**BC clauses:** BC-4.01.003 (speaker notes), BC-4.01.006 (notes/handout masters)
**Feature branch:** `feature/S-040`
**Evidence generated:** `cargo run --example demo_notes_evidence -p slideforge-pptx`
**Test suite:** `cargo nextest run -p slideforge-pptx --no-fail-fast` — 19 notes_tests, 160 total: **all PASS**

---

## Recording Method

This is a **library/IR-level demo** — not a CLI or web product. The `.sf → .pptx` pipeline for notes content goes through the IR; evidence is produced by:

1. Calling `PptxExporter::export` with a real `Deck` + `LaidOutDeck` built from fixture code that mirrors the production tests.
2. Opening the produced PPTX ZIP with the `zip` crate.
3. Parsing extracted XML with `quick-xml`.
4. Writing the extracted values (XML snippets, assertion results, ZIP entry lists) to `docs/demo-evidence/STORY-040/`.

All evidence files contain **real extracted XML from the real PPTX output** — no mock strings.

---

## AC Coverage

### AC-001: notesSlide part count equals slides with non-empty notes

**File:** `AC-001-notes-slide-zip-entries.txt`
**Test:** `test_BC_4_01_003_ac001_notes_slide_count_equals_slides_with_notes`
**BC:** BC-4.01.003 postcondition 1

**Concrete evidence:** 3-slide deck where slides 1 and 3 have notes, slide 2 does not. ZIP extraction shows exactly 2 `notesSlide*.xml` entries:

```
ZIP entries matching ppt/notesSlides/notesSlide*.xml (2 found):
  - ppt/notesSlides/notesSlide1.xml
  - ppt/notesSlides/notesSlide3.xml

Assertion: notes_entries.len() == 2 -> true
```

**Success path:** 2 notes → 2 notesSlide parts.
**Error/boundary path (EC-001):** 0 notes → 0 notesSlide parts (see `EC-001-no-notes-slide-for-empty-notes.txt`).

---

### AC-002: Notes text in body placeholder txBody

**File:** `AC-002-notes-text-body-placeholder.txt`
**Raw XML:** `AC-002-notesSlide1-raw.xml`
**Test:** `test_BC_4_01_003_ac002_notes_text_in_body_placeholder`
**BC:** BC-4.01.003 postcondition 6

**Concrete evidence:** Notes text `"Click here to rehearse your speaking points carefully."` appears in the `<p:txBody>` of `<p:ph type="body" idx="1"/>`. The `<p:ph>` is self-closing (schema-correct: it is inside `<p:nvPr>` and `<p:txBody>` is a sibling, not nested inside `<p:ph>`).

Key structural assertions all `true`:
```
  <a:t> text node contains notes: true
  Body placeholder marker present:  true
  <p:ph> is self-closing (correct): true
```

See `AC-002-notesSlide1-raw.xml` for the full notesSlide XML.

---

### AC-003: Notes text NOT in any ppt/slides/slide*.xml

**File:** `AC-003-no-bleed-sentinel-check.txt`
**Tests:** `test_BC_4_01_003_ac003_notes_absent_from_slide_bodies`, `test_f040_p1_004_ac003_no_bleed_with_positive_routing`
**BC:** BC-4.01.003 invariant 1 / DI-012

**Concrete evidence:** Sentinel string `"NOTES_BLEED_SENTINEL_X7Q2Z9"` is placed in notes. The file shows:
- Sentinel IS present in `ppt/notesSlides/notesSlide1.xml` (positive routing — not silently lost)
- Sentinel is NOT present in any `ppt/slides/slide*.xml` entry

```
  ppt/slides/slide1.xml: contains sentinel = false

Assertion: bleed_found == false -> true
Assertion: in_notes_slide == true  -> true
```

---

### AC-004: notesMaster1.xml always present and structurally valid

**File:** `AC-004-notes-master-validity.txt`
**Raw XML:** `AC-004-notesMaster1-raw.xml`
**Tests:** `test_BC_4_01_006_ac004_notes_master_always_present`, `test_f040_p1_002_notes_master_xml_is_structurally_valid`
**BC:** BC-4.01.006 invariant 1 / F-040-P1-002 (CRIT)

**Concrete evidence:** `ppt/notesMasters/notesMaster1.xml` is present in every PPTX, including decks with zero notes slides. The file contains the required structural elements:

```
Structural assertions:
  <p:clrMap> present:                true
  <p:ph type="sldImg"/> present:       true
  <p:ph type="body" idx="1"/> present: true
  NO <a:grpSpPr> (schema-valid):     true  [F-040-A1]
```

Sample from the real notesMaster1.xml:
```xml
<p:notesMaster ...>
  <p:cSld>
    <p:spTree>
      <p:grpSpPr>
        <a:xfrm>...</a:xfrm>
      </p:grpSpPr>
      <p:sp>
        <p:nvSpPr>
          <p:nvPr><p:ph type="sldImg"/></p:nvPr>
        </p:nvSpPr>
      </p:sp>
      <p:sp>
        <p:nvSpPr>
          <p:nvPr><p:ph type="body" idx="1"/></p:nvPr>
        </p:nvSpPr>
      </p:sp>
    </p:spTree>
  </p:cSld>
  <p:clrMap .../>
</p:notesMaster>
```

---

### AC-005: handoutMaster1.xml always present

**File:** `AC-005-handout-master-presence.txt`
**Test:** `test_BC_4_01_006_ac005_handout_master_always_present`
**BC:** BC-4.01.006 postcondition 2

**Concrete evidence:** `ppt/handoutMasters/handoutMaster1.xml` is present in every PPTX. The file also confirms no schema-invalid `<a:grpSpPr>` is emitted (F-040-A1):

```
Assertion: handoutMaster1.xml present -> true
Assertion: no schema-invalid <a:grpSpPr> -> true
```

---

## Key Behavioral Evidence (Beyond ACs)

### Slide-to-notesSlide Relationship (F-040-P1-001, CRIT)

**File:** `RELS-slide-to-notes-slide-relationship.txt`
**Test:** `test_f040_p1_001_slide_has_notes_slide_rel_in_slide_rels`

PowerPoint discovers notes via the slide's own `_rels` file. Evidence shows slide 1 (WITH notes) has the relationship; slide 2 (WITHOUT notes) does not:

```xml
<!-- slide1.xml.rels — WITH notes -->
<Relationships ...>
  <Relationship Target="../slideLayouts/slideLayout1.xml"
    Type=".../slideLayout" Id="rId1" />
  <Relationship Target="../notesSlides/notesSlide1.xml"
    Type=".../notesSlide" Id="rId2" />
</Relationships>

<!-- slide2.xml.rels — WITHOUT notes -->
<Relationships ...>
  <Relationship Target="../slideLayouts/slideLayout1.xml"
    Type=".../slideLayout" Id="rId1" />
</Relationships>
```

```
slide1 has notesSlide relationship:    true
slide2 has NO notesSlide relationship: true
All assertions pass: true
```

---

### Rich Notes: Bold + Italic Formatting (F-040-P1-003, HIGH)

**File:** `RICH-bold-italic-formatting.txt`
**Tests:** `test_f040_p1_003_rich_notes_bold_and_italic_runs`, `test_f040_p1_003_multi_entry_notes_all_emitted`

`InlineNode::Bold` produces `<a:rPr b="1">`, `InlineNode::Italic` produces `<a:rPr i="1">`:

```
  <a:rPr b="1"> (bold run):     true
  <a:rPr i="1"> (italic run):   true
  'bold content' text present:   true
  'italic content' text present: true
  'plain text' text present:     true
All assertions pass: true
```

---

### SafeUrl: javascript: Link Degrades to Plain Text (F-040-P2-001, CWE-601)

**File:** `SAFEURL-javascript-plain-text-degradation.txt`
**Tests:** `test_f040_p2_001_unsafe_scheme_javascript_no_external_rel`, `test_f040_p2_001_unsafe_schemes_data_file_no_external_rel`

`javascript:` URL input: no `TargetMode="External"` relationship is created; display text is preserved as a plain run:

```xml
<!-- notesSlide1.xml.rels — only slide and notesMaster rels, NO External rel -->
<Relationships ...>
  <Relationship Target="../slides/slide1.xml" Type=".../slide" Id="rId1" />
  <Relationship Target="../notesMasters/notesMaster1.xml" Type=".../notesMaster" Id="rId2" />
</Relationships>

<!-- txBody excerpt — 'click me' is a plain <a:r> with no hlinkClick -->
<p:txBody>
  <a:p>
    <a:r><a:t>click me</a:t></a:r>
  </a:p>
</p:txBody>
```

```
Security assertions (CWE-601):
  'javascript' absent from .rels:          true
  NO TargetMode=External rel:              true
  'javascript' absent from notesSlide xml: true
  'click me' display text preserved:       true
All assertions pass: true
```

Same behavior confirmed for `data:` and `file:` schemes.

---

### Safe https: Link Produces hlinkClick + External Rel (F-040-P2-002, HIGH)

**File:** `LINK-safe-https-hlinkclick-external-rel.txt`
**Tests:** `test_f040_p2_002_single_safe_https_link_has_hlinkclick_and_external_rel`, `test_f040_p2_002_two_distinct_links_stable_deterministic_rids`, `test_f040_p2_002_duplicate_url_deduped_to_single_rel`

`https:` URL produces `<a:hlinkClick r:id="rId3"/>` in the notesSlide XML and a matching `TargetMode="External"` relationship in the `.rels` file:

```
  <a:hlinkClick> present in notesSlide xml: true
  rId3 referenced in notesSlide xml:        true
  rId3 Relationship in .rels:               true
  TargetMode=External in .rels:             true
  Target URL in .rels:                      true
All assertions pass: true
```

---

### Schema-Valid grpSpPr (F-040-A1, HIGH)

**File:** `SCHEMA-grpSpPr-no-nested-a-grpSpPr.txt`
**Tests:** `test_f040_a1_notes_slide_grpsppr_is_schema_valid` (notesSlide), `test_f040_p1_002_notes_master_xml_is_structurally_valid` (also checks notesMaster + handoutMaster)

`CT_GroupShapeProperties` permits `<a:xfrm>` as child but has NO child named `<a:grpSpPr>`. Evidence confirms the correct structure:

```
Schema assertions:
  NO <a:grpSpPr> (would be schema-invalid): true
  <p:grpSpPr> present (spTree first child):  true
  <a:xfrm> identity transform present:        true
All assertions pass: true
```

---

## Test Run Log

**File:** `test-run-log-notes.txt`

All 19 notes_tests PASS. Key tests mapped to ACs:

| Test | AC/Behavior | Result |
|------|-------------|--------|
| `test_BC_4_01_003_ac001_notes_slide_count_equals_slides_with_notes` | AC-001 | PASS |
| `test_BC_4_01_003_ac001_ec001_no_notes_slide_for_empty_notes` | EC-001 | PASS |
| `test_BC_4_01_003_ac002_notes_text_in_body_placeholder` | AC-002 | PASS |
| `test_BC_4_01_003_ac003_notes_absent_from_slide_bodies` | AC-003 | PASS |
| `test_BC_4_01_006_ac004_notes_master_always_present` | AC-004 | PASS |
| `test_BC_4_01_006_ac005_handout_master_always_present` | AC-005 | PASS |
| `test_f040_p1_001_slide_has_notes_slide_rel_in_slide_rels` | Slide→notesSlide rel | PASS |
| `test_f040_p1_002_notes_master_xml_is_structurally_valid` | notesMaster validity + no grpSpPr | PASS |
| `test_f040_p1_003_rich_notes_bold_and_italic_runs` | Rich notes formatting | PASS |
| `test_f040_p1_003_multi_entry_notes_all_emitted` | Multi-entry notes | PASS |
| `test_f040_p1_004_ac003_no_bleed_with_positive_routing` | AC-003 positive routing | PASS |
| `test_f040_p2_001_unsafe_scheme_javascript_no_external_rel` | CWE-601 javascript: | PASS |
| `test_f040_p2_001_unsafe_schemes_data_file_no_external_rel` | CWE-601 data:/file: | PASS |
| `test_f040_p2_002_single_safe_https_link_has_hlinkclick_and_external_rel` | https: hlinkClick | PASS |
| `test_f040_p2_002_two_distinct_links_stable_deterministic_rids` | Deterministic rIds | PASS |
| `test_f040_p2_002_duplicate_url_deduped_to_single_rel` | URL deduplication | PASS |
| `test_f040_p2_003_notes_text_xml_escape_well_formed_and_lossless` | XML-escape lossless | PASS |
| `test_f040_a1_notes_slide_grpsppr_is_schema_valid` | Schema-valid grpSpPr | PASS |
| `test_f040_p3_001_nested_link_in_display_text_no_orphan_rel` | No orphan External rel | PASS |

---

## Evidence File Index

| File | AC/Behavior | What it proves |
|------|-------------|----------------|
| `AC-001-notes-slide-zip-entries.txt` | AC-001 | ZIP entry list: 2 notesSlide parts for 3-slide deck with 2 notes slides |
| `EC-001-no-notes-slide-for-empty-notes.txt` | EC-001 | 0 notesSlide parts for no-notes deck |
| `AC-002-notes-text-body-placeholder.txt` | AC-002 | Formatted notesSlide1.xml showing body placeholder + text |
| `AC-002-notesSlide1-raw.xml` | AC-002 | Raw notesSlide1.xml from real PPTX ZIP |
| `AC-003-no-bleed-sentinel-check.txt` | AC-003 | Sentinel absent from slide bodies, present in notesSlide |
| `AC-004-notes-master-validity.txt` | AC-004 | notesMaster1.xml structural assertions all true |
| `AC-004-notesMaster1-raw.xml` | AC-004 | Raw notesMaster1.xml from real PPTX ZIP |
| `AC-005-handout-master-presence.txt` | AC-005 | handoutMaster1.xml present, no schema-invalid content |
| `RELS-slide-to-notes-slide-relationship.txt` | F-040-P1-001 | slide1 _rels with notesSlide rel; slide2 _rels without |
| `RICH-bold-italic-formatting.txt` | F-040-P1-003 | bold/italic InlineNode produces correct rPr attributes |
| `SAFEURL-javascript-plain-text-degradation.txt` | F-040-P2-001 | javascript: URL → no External rel, plain text preserved |
| `LINK-safe-https-hlinkclick-external-rel.txt` | F-040-P2-002 | https: URL → hlinkClick + TargetMode=External in .rels |
| `SCHEMA-grpSpPr-no-nested-a-grpSpPr.txt` | F-040-A1 | No schema-invalid `<a:grpSpPr>` inside `<p:grpSpPr>` |
| `test-run-log-notes.txt` | All tests | 19 notes_tests, 160 total: all PASS |

---

## AC Coverage Summary

| AC | Description | Evidence File | Test | Status |
|----|-------------|---------------|------|--------|
| AC-001 | notesSlide count == slides with notes | `AC-001-*.txt` | `..._ac001_notes_slide_count...` | COVERED |
| EC-001 | No notesSlide for slides without notes | `EC-001-*.txt` | `..._ec001_no_notes_slide...` | COVERED |
| AC-002 | Notes text in body placeholder txBody | `AC-002-*.txt` | `..._ac002_notes_text_in_body...` | COVERED |
| AC-003 | Notes text NOT in any slide body | `AC-003-*.txt` | `..._ac003_notes_absent...` | COVERED |
| AC-004 | notesMaster1.xml always present + valid | `AC-004-*.txt` | `..._ac004_notes_master...` + `..._p1_002...` | COVERED |
| AC-005 | handoutMaster1.xml always present | `AC-005-*.txt` | `..._ac005_handout_master...` | COVERED |
| F-040-P1-001 | Slide→notesSlide _rels back-rel | `RELS-*.txt` | `..._p1_001_slide_has_notes...` | COVERED |
| F-040-P1-003 | Rich notes: bold/italic + multi-entry | `RICH-*.txt` | `..._p1_003_rich_notes...` | COVERED |
| F-040-P2-001 | CWE-601: unsafe URL → plain text | `SAFEURL-*.txt` | `..._p2_001_unsafe_scheme...` | COVERED |
| F-040-P2-002 | Safe https: → hlinkClick + External rel | `LINK-*.txt` | `..._p2_002_single_safe_https...` | COVERED |
| F-040-A1 | Schema-valid grpSpPr (no `<a:grpSpPr>`) | `SCHEMA-*.txt` | `..._a1_notes_slide_grpsppr...` | COVERED |
