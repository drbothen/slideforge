---
story: STORY-096
title: "REND-007: PPTX slideMaster 16:9 geometry + progress_bar layout + lang on runs"
date: 2026-06-12
worktree_sha: 6e29bce17283c20c75211cad957f2064c67236df
branch: feature/STORY-096
cascade_status: LOCAL adversary CONVERGED 3/3 (clean strict)
evidence_generated_by: vsdd-factory:demo-recorder
---

# STORY-096 Demo Evidence

## Summary

All four acceptance criteria verified. Test suite: **11/11 PASS**. Direct artifact
inspection confirms OOXML values match spec to the exact EMU.

## Per-AC Verdict Table

| AC | Description | Verdict |
|----|-------------|---------|
| AC-001 | slideMaster has NO `<p:sldSz>`; presentation.xml sldSz matches page_size | **PASS** |
| AC-002 | progress_bar slide type has named layout slideLayout31.xml / SF Progress Bar | **PASS** |
| AC-003 | All `<a:rPr>` carry lang attribute; en-US / fr-FR round-trip / no-lang → "en" | **PASS** |
| AC-004 | Footer/date/slideNum placeholders all within 16:9 bounds (y+cy <= 5143500) | **PASS** |

---

## AC-001: Slide Master Contains NO sldSz; presentation.xml sldSz Matches page_size

**Spec contract:** `<p:sldSz>` is schema-invalid in CT_SlideMaster (ECMA-376 §19.3.1.42).
It MUST NOT appear in `slideMaster1.xml`. It belongs exclusively in `presentation.xml`,
derived from `LaidOutDeck.page_size`.

### Test suite evidence

```
cargo nextest run -p slideforge-pptx -E 'test(story_096)' --no-capture
```

```
PASS [0.047s] test_BC_4_01_001_master_has_no_sldsz_presentation_carries_16x9
PASS [0.047s] test_BC_4_01_001_master_has_no_sldsz_presentation_carries_custom_page_size
```

### Artifact inspection — 16:9 default (cx=9144000 cy=5143500)

Artifact written to `/tmp/story096_16x9.pptx` and unzipped to `/tmp/story096_unzip_16x9/`.

**slideMaster1.xml — grep for `<p:sldSz`:**

```
$ grep -c "<p:sldSz" ppt/slideMasters/slideMaster1.xml
0
```

Result: **0 occurrences**. No `<p:sldSz>` in the master. Schema-valid.

**slideMaster1.xml opening (trimmed to show structure — no sldSz element):**

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="..."><p:cSld><p:spTree>
  <p:nvGrpSpPr>...</p:nvGrpSpPr>
  <p:grpSpPr><a:xfrm>...</a:xfrm></p:grpSpPr>
  <p:sp><!-- Title Placeholder --></p:sp>
  <p:sp><!-- Content Placeholder --></p:sp>
  <p:sp><!-- Date Placeholder --></p:sp>
  <p:sp><!-- Footer Placeholder --></p:sp>
  <p:sp><!-- Slide Number Placeholder --></p:sp>
</p:spTree></p:cSld>
<a:clrMap .../>
<p:sldLayoutIdLst>
  <p:sldLayoutId id="2147483649" r:id="rId2"/>
  ... (31 entries) ...
</p:sldLayoutIdLst>
<!-- NO <p:sldSz> element anywhere -->
</p:sldMaster>
```

**presentation.xml — sldSz element:**

```
$ grep -o '<p:sldSz[^/]*/>' ppt/presentation.xml
<p:sldSz cx="9144000" cy="5143500" />
```

cx=9144000 (16:9 width), cy=5143500 (16:9 height). Exact match to `PageSize::default()`.

### Artifact inspection — EC-001: custom 4:3 brand (cx=6858000 cy=5143500)

Artifact written to `/tmp/story096_43custom.pptx`.

**slideMaster1.xml — grep for `<p:sldSz`:**

```
$ grep "<p:sldSz" ppt/slideMasters/slideMaster1.xml
(no output — zero matches)
```

Result: **PASS** — master still has no `<p:sldSz>` with custom page size.

**presentation.xml — sldSz element (4:3 custom):**

```
$ grep -o '<p:sldSz[^/]*/>' ppt/presentation.xml
<p:sldSz cx="6858000" cy="5143500" />
```

cx=6858000, cy=5143500. Exact match to the custom brand dimensions supplied.

**Verdict: PASS**

---

## AC-002: progress_bar Slide Has Named Layout slideLayout31.xml

**Spec contract:** A deck with a `progress_bar` slide must use a named layout
`<p:cSld name="SF Progress Bar">` at 0-based index 30 (1-based part 31), not the
generic `slideLayout2.xml` fallback.

### Test suite evidence

```
PASS [0.051s] test_BC_4_01_005_progress_bar_has_named_layout
PASS [0.015s] test_BC_4_01_005_progress_bar_slide_resolves_named_layout_not_fallback
```

Also from layout_tests.rs (STORY-038 coverage):

```
PASS [0.008s] tests::layout_tests::test_BC_4_01_005_ac009_progress_bar_maps_to_index_30
```

### Artifact inspection

Artifact: `/tmp/story096_progress_bar.pptx` — 1-slide deck, slide type `progress_bar`.

**slideLayout31.xml existence and name:**

```
$ ls -la ppt/slideLayouts/slideLayout31.xml
-rw-r--r--  1 jmagady  wheel  1126 Jan  1  1980 ppt/slideLayouts/slideLayout31.xml

$ grep -o 'name="[^"]*"' ppt/slideLayouts/slideLayout31.xml | head -1
name="SF Progress Bar"
```

The 31st layout exists and its `<p:cSld name>` is `"SF Progress Bar"`.

**slideLayout31.xml — opening fragment:**

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:p="..." xmlns:a="..." type="cust" preserve="1">
  <p:cSld name="SF Progress Bar">
    <p:spTree>
      <p:sp><!-- Progress Bar Label (ph type="title") --></p:sp>
      <p:sp><!-- Progress Content (ph type="body" idx="1") --></p:sp>
    </p:spTree>
  </p:cSld>
</p:sldLayout>
```

**slide1.xml.rels — layout relationship:**

```
$ cat ppt/slides/_rels/slide1.xml.rels
<Relationships ...>
  <Relationship
    Target="../slideLayouts/slideLayout31.xml"
    Type=".../slideLayout"
    Id="rId1" />
</Relationships>
```

The slide resolves to `slideLayout31.xml`. The forbidden fallback `slideLayout2.xml`
does NOT appear.

**Total layout files in ZIP:**

```
$ ls ppt/slideLayouts/slideLayout*.xml | wc -l
31
```

31 layout parts present as required by BC-4.01.005.

**Verdict: PASS**

---

## AC-003: All `<a:rPr>` Carry lang Attribute

**Spec contract (BC-5.01.005 v1.3):**
- Explicit `lang "en-US"` deck → every `<a:rPr lang="en-US">` on all runs.
- Explicit `lang "fr-FR"` deck → every `<a:rPr lang="fr-FR">` (lossless BCP-47).
- `lang: None` deck → every `<a:rPr lang="en">` (default "en" per BC-5.01.004)
  AND `<dc:language>en</dc:language>` in `docProps/core.xml` (cross-surface identity).

### Test suite evidence

```
PASS [0.046s] test_BC_5_01_005_run_has_lang_attribute_explicit_en_us
PASS [0.046s] test_BC_5_01_005_run_has_lang_attribute_fr_fr_round_trip
PASS [0.047s] test_BC_5_01_005_ac003_all_rpr_carry_lang_universality
PASS [0.046s] test_BC_5_01_005_ec003_empty_slide_no_rpr_error
PASS [0.047s] test_BC_5_01_005_f096_002_no_lang_defaults_to_en_cross_surface
```

### Artifact inspection — (a) Explicit en-US

Artifact: `/tmp/story096_lang_en_us.pptx` — 1-slide deck with body text, `lang "en-US"`.

```
$ grep -o 'lang="[^"]*"' ppt/slides/slide1.xml
lang="en-US"
lang="en-US"
```

Both `<a:rPr>` elements (title run + body run) carry `lang="en-US"`. No rPr without lang.

**slide1.xml run fragments:**

```xml
<a:rPr lang="en-US"></a:rPr><a:t>Title Text</a:t>
<a:rPr lang="en-US"></a:rPr><a:t>Hello world</a:t>
```

### Artifact inspection — (b) fr-FR round-trip

Artifact: `/tmp/story096_lang_fr_fr.pptx` — 1-slide deck with body text, `lang "fr-FR"`.

```
$ grep -o 'lang="[^"]*"' ppt/slides/slide1.xml
lang="fr-FR"
lang="fr-FR"
```

Both runs carry `lang="fr-FR"`. BCP-47 tag is lossless — not normalized to "fr".

### Artifact inspection — (c) no-lang deck → "en" default + core.xml cross-surface

Artifact: `/tmp/story096_lang_nolang.pptx` — 1-slide deck, no `lang` declaration.

```
$ grep -o 'lang="[^"]*"' ppt/slides/slide1.xml
lang="en"
lang="en"
```

Both runs carry `lang="en"` (the BC-5.01.004 default). The string is `"en"`, NOT `"en-US"`.

**docProps/core.xml:**

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties ...>
  <dc:creator>slideforge</dc:creator>
  <dc:language>en</dc:language>
  <dcterms:created xsi:type="dcterms:W3CDTF">1980-01-01T00:00:00Z</dcterms:created>
</cp:coreProperties>
```

`<dc:language>en</dc:language>` matches the `<a:rPr lang="en">` value exactly.
Both surfaces derive from the same `DEFAULT_DECK_LANG = "en"` constant — cross-surface
identity confirmed.

**Verdict: PASS**

---

## AC-004: Footer and Date Placeholders Within Slide Bounds

**Spec contract:** Every placeholder `<a:off>/<a:ext>` in `slideMaster1.xml` satisfies
`y >= 0` AND `y + cy <= 5143500` (16:9 page height in EMU). Placeholder geometry is
derived from `brand.page_size.height_emu` via `serialize_master_to_xml`, not hardcoded.

### Test suite evidence

```
PASS [0.046s] test_BC_4_01_001_footer_date_placeholders_within_16x9_bounds
PASS [0.015s] test_BC_4_01_001_footer_y_saturating_on_tiny_page
```

### Artifact inspection — coordinate arithmetic

Extracted from `/tmp/story096_unzip_16x9/ppt/slideMasters/slideMaster1.xml`
(python3 parse of all `<p:sp>` blocks with their `<a:off>` and `<a:ext>`):

```
Page height: 5,143,500 EMU (16:9)

Placeholder               off.y       ext.cy     bottom (y+cy)   <= 5143500?
Title Placeholder         274,638    1,143,000     1,417,638      PASS
Content Placeholder     1,600,200    2,971,800     4,572,000      PASS
Date Placeholder        4,572,000      365,125     4,937,125      PASS
Footer Placeholder      4,572,000      365,125     4,937,125      PASS
Slide Number Placeholder 4,572,000    365,125     4,937,125      PASS
```

All 5 placeholders: y >= 0 AND y+cy <= 5143500. Maximum bottom edge is 4,937,125 —
margin of 206,375 EMU below the 5,143,500 page boundary.

The footer zone is at `y=4,572,000`, derived from `page_height - FOOTER_MARGIN_FROM_BOTTOM`
(not a hardcoded static value from a 4:3 assumption).

The content body placeholder bottom edge is 4,572,000 — exactly equal to the footer
zone top, confirming the body cy is derived from the footer zone top rather than
using an out-of-bounds static value.

### Pathological tiny-page clamp test

`test_BC_4_01_001_footer_y_saturating_on_tiny_page` calls `serialize_master_to_xml`
with `page_height=100,000` (< `FOOTER_MARGIN_FROM_BOTTOM=571,500`) and asserts no
`<a:off y>` value is negative. **PASS** — `footer_zone_top` is clamped via
saturating subtraction.

**Verdict: PASS**

---

## Full Test Run Transcript

```
$ cargo nextest run -p slideforge-pptx -E 'test(story_096)' --no-fail-fast

Nextest run ID b8d9dafa-... with nextest profile: default
    Starting 11 tests across 1 binary (262 tests skipped)
        PASS [0.014s] ( 1/11) tests::story_096_tests::test_BC_4_01_005_progress_bar_slide_resolves_named_layout_not_fallback
        PASS [0.015s] ( 2/11) tests::story_096_tests::test_BC_4_01_001_footer_y_saturating_on_tiny_page
        PASS [0.046s] ( 3/11) tests::story_096_tests::test_BC_4_01_001_footer_date_placeholders_within_16x9_bounds
        PASS [0.046s] ( 4/11) tests::story_096_tests::test_BC_5_01_005_run_has_lang_attribute_fr_fr_round_trip
        PASS [0.046s] ( 5/11) tests::story_096_tests::test_BC_5_01_005_ec003_empty_slide_no_rpr_error
        PASS [0.046s] ( 6/11) tests::story_096_tests::test_BC_5_01_005_run_has_lang_attribute_explicit_en_us
        PASS [0.047s] ( 7/11) tests::story_096_tests::test_BC_4_01_001_master_has_no_sldsz_presentation_carries_custom_page_size
        PASS [0.047s] ( 8/11) tests::story_096_tests::test_BC_5_01_005_f096_002_no_lang_defaults_to_en_cross_surface
        PASS [0.047s] ( 9/11) tests::story_096_tests::test_BC_5_01_005_ac003_all_rpr_carry_lang_universality
        PASS [0.047s] (10/11) tests::story_096_tests::test_BC_4_01_001_master_has_no_sldsz_presentation_carries_16x9
        PASS [0.051s] (11/11) tests::story_096_tests::test_BC_4_01_005_progress_bar_has_named_layout
────────────
     Summary [0.051s] 11 tests run: 11 passed, 262 skipped
```

## Anomalies

None. All ACs verified cleanly. Temp artifact files written to /tmp/* were cleaned up
(the integration test file `tests/dump_demo_artifacts.rs` was created transiently for
evidence generation and deleted after use — not committed).
