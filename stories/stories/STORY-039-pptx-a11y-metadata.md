---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-039
title: "PPTX Accessibility Metadata"
epic: EPIC-08
wave: 4
points: 5
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.01.004, BC-5.01.005]
verification_properties: []
nfr_refs: [NFR-014, NFR-015, NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-pptx
target_module: slideforge-pptx
subsystems: [SS-06]
depends_on:
  - STORY-038
  - STORY-015
blocks:
  - STORY-049
  - STORY-050
estimated_days: 2
---

# STORY-039: PPTX Accessibility Metadata

## Subsystem Anchor Justification

SS-06 (PPTX Export) owns this story because it adds accessibility metadata to the PPTX
output. The ARCH-INDEX lists `slideforge-pptx` under SS-06. Compile-time enforcement
of alt text is SS-03 (STORY-015); this story is the embedding layer.

## Dependency Anchor Justifications

- Depends on STORY-038 (Layout Compliance): Alt text embedding requires shapes to be
  correctly structured with `<p:cNvPr>` elements, which are established by the layout
  compliance work. This story adds the `descr` attribute to those elements.
- Depends on STORY-015 (Alt Text Enforcement): By the time the PPTX exporter runs,
  every visual element in `LaidOutDeck` is guaranteed to have an `alt` field (or
  `decorative: true`) — enforced compile-time by STORY-015. This story embeds those
  values.
- Blocks STORY-049/050: Integration tests require accessibility metadata to pass the
  a11y assertions.

## Summary

Extend the PPTX exporter with accessibility metadata embedding:

1. **Alt text**: Every non-decorative image, chart, or diagram shape in the PPTX gets
   `<p:cNvPr descr="alt text value">`. Decorative elements get `descr=""`.
2. **Language**: `docProps/core.xml` gets `<dc:language>LANG</dc:language>` from
   `LaidOutDeck.lang`.

This story does NOT handle WCAG contrast (that is validated at compile time by
STORY-017). It only embeds the accessibility metadata that has already been validated
upstream.

### Alt Text Embedding Protocol

Per BC-4.01.004 invariant 2, alt text belongs on `<p:cNvPr>`, NOT on `<p:ph altText>`.
The `descr` attribute is the correct OOXML alt text location for shapes.

```xml
<!-- Non-decorative image with alt text -->
<p:pic>
  <p:nvPicPr>
    <p:cNvPr id="3" name="Revenue Chart" descr="Bar chart showing Q1 revenue by region"/>
    <p:cNvPicPr/>
    <p:nvPr/>
  </p:nvPicPr>
  ...
</p:pic>

<!-- Decorative image: empty descr -->
<p:pic>
  <p:nvPicPr>
    <p:cNvPr id="4" name="Decorative Background" descr=""/>
    ...
  </p:nvPicPr>
  ...
</p:pic>
```

For group shapes containing SVG (chart, diagram):
```xml
<p:grpSp>
  <p:nvGrpSpPr>
    <p:cNvPr id="5" name="Q1 Chart" descr="Bar chart: Q1 revenue by region"/>
    ...
  </p:nvGrpSpPr>
  ...
</p:grpSp>
```

### Language Embedding in core.xml

The `docProps/core.xml` update for BC-5.01.005:
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties
  xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
  xmlns:dc="http://purl.org/dc/elements/1.1/"
  xmlns:dcterms="http://purl.org/dc/terms/">
  <dc:creator>slideforge</dc:creator>
  <dc:language>en-US</dc:language>   <!-- from LaidOutDeck.lang -->
  <dcterms:created xsi:type="dcterms:W3CDTF">1980-01-01T00:00:00Z</dcterms:created>
</cp:coreProperties>
```

The `dc:language` value is the exact BCP-47 tag from `LaidOutDeck.lang`. No
normalization, no case change. "zh-Hant-TW" stays "zh-Hant-TW".

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.01.004 | PPTX contains accessibility metadata (alt text, lang, and WCAG contrast) | AC-001 through AC-005 |
| BC-5.01.005 | lang declaration propagates to PPTX Core Properties, PDF /Lang, HTML lang attr | AC-006, AC-007 |

## Acceptance Criteria

### AC-001: Non-decorative visual elements have non-empty descr
(traces to BC-4.01.004 postcondition 1 — non-empty descr on cNvPr for non-decorative shapes)

A slide with an image `alt "Revenue chart Q1 2026"` produces a PPTX where the
corresponding `<p:cNvPr>` (or `<p:cNvPr>` on the enclosing group shape) has
`descr="Revenue chart Q1 2026"`. A unit test parses slide XML, extracts all
`p:cNvPr[@descr]` attributes, and asserts none are empty for non-decorative elements.

### AC-002: Decorative elements have empty descr
(traces to BC-4.01.004 postcondition 2 — decorative elements have descr="")

A slide with `decorative: true` on an image produces `<p:cNvPr descr="">`. A unit
test parses slide XML and asserts the decorative shape's `descr` is the empty string
(not absent — the attribute must be present with an empty value).

### AC-003: Alt text values are not truncated
(traces to BC-4.01.004 invariant 1 — full alt string used, never truncated)

An alt text value of 300 characters is embedded in full. The `descr` attribute value
length equals the input length. OOXML schema permits long `descr` strings.

### AC-004: Alt text with special characters is XML-escaped
(traces to BC-4.01.004 edge case EC-001 — quotes and special characters)

An alt text value of `Revenue & Cost "Q1"` is embedded as
`descr="Revenue &amp; Cost &quot;Q1&quot;"` in the raw XML. ooxmlsdk handles this
automatically. A unit test provides a sentinel with `&`, `"`, `<`, `>` and verifies
the produced XML is well-formed (parses without error).

### AC-005: Charts and diagrams: alt on enclosing group shape
(traces to BC-4.01.004 edge case EC-005 — chart shape alt text)

A slide with a `slide chart:` block (chart SVG embedded as media) has the alt text
on the `<p:grpSp>` or `<p:pic>` shape containing the SVG, not on a text box. A unit
test parses slide XML and finds the `descr` on the picture/group shape matching the
chart's `alt "..."` field.

### AC-006: dc:language in core.xml matches deck lang exactly
(traces to BC-5.01.005 postcondition 1 — PPTX dc:language is exact BCP-47 tag)

A deck with `lang "en-US"` produces `<dc:language>en-US</dc:language>` in
`docProps/core.xml`. A deck with `lang "zh-Hant-TW"` produces
`<dc:language>zh-Hant-TW</dc:language>`. A unit test parses `core.xml` from the PPTX
ZIP and asserts the `dc:language` text content equals `LaidOutDeck.lang` exactly.

### AC-007: No lang declaration → core.xml contains default "en"
(traces to BC-5.01.005 edge case EC-003 — default is "en" per BC-5.01.004)

A deck with no `lang` declaration has `LaidOutDeck.lang = "en"` (set by upstream
evaluator per BC-5.01.004). The PPTX embeds `<dc:language>en</dc:language>`.

## Tasks

- [ ] Implement `AltTextEmbedder` in `src/a11y.rs`:
  - Walk `LaidOutSlide.frames` for image, chart, diagram frame types
  - For each frame: look up `alt` text or `decorative` flag from the `LaidOutFrame` metadata
  - Set `cNvPr.descr` to alt text (non-decorative) or `""` (decorative)
  - Handle group shapes (chart, diagram): set `descr` on the enclosing group
- [ ] Update `SlideSerializer` to call `AltTextEmbedder` for every shape
- [ ] Implement `CorePropertiesBuilder` in `src/doc_props.rs`:
  - Generate `docProps/core.xml` with `dc:language = deck.lang`
  - Use epoch timestamp for `dcterms:created`
- [ ] Update `ZipAssembler` to include the updated `core.xml` with `dc:language`
- [ ] Write unit tests for AC-001 through AC-007:
  - Non-decorative image → non-empty `descr`
  - Decorative image → `descr=""`
  - Long alt text (300 chars) → not truncated
  - Special characters in alt text → XML-escaped
  - Chart shape → alt on enclosing shape
  - `lang "en-US"` → `<dc:language>en-US</dc:language>` in core.xml
  - `lang "zh-Hant-TW"` → exact embedding
  - No lang → `<dc:language>en</dc:language>`

## Previous Story Intelligence

STORY-037 created `docProps/core.xml` with a placeholder `dc:language` or omitted it.
This story fills the `dc:language` field correctly. If STORY-037's `core.xml` already
has a static `dc:language`, this story replaces it with the dynamic value from
`LaidOutDeck.lang`.

STORY-015 (Alt Text Enforcement) guaranteed that every `LaidOutFrame` with a visual
element has either a non-empty `alt` string or `decorative: true`. This story trusts
that guarantee — it does NOT re-validate. If a frame reaches this code without an alt
decision, it is a programming error (upstream validation bug), not a user error.

## Architecture Compliance Rules

1. **`descr` on `<p:cNvPr>`, not `<p:ph altText>` (BC-4.01.004 invariant 2)**: The
   `altText` attribute on `<p:ph>` is for placeholder names only, not shape alt text.
   `AltTextEmbedder` must set `cNvPr.descr`, not `ph.altText`.
2. **Alt text never truncated (BC-4.01.004 invariant 1)**: The full string from
   `LaidOutFrame.alt` is used. No length limit applied.
3. **Language propagation is lossless (BC-5.01.005 invariant 1)**: No truncation,
   normalization, or case change to the lang value. "zh-Hant-TW" stays "zh-Hant-TW".
4. **Single source of truth for lang (BC-5.01.005 invariant 2)**: `LaidOutDeck.lang`
   is read once. No hardcoded language values anywhere in `slideforge-pptx`.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `ooxmlsdk` | `=0.6.1` | `cNvPr.descr` attribute setting |
| `slideforge-types` | workspace | `LaidOutFrame.alt`, `LaidOutFrame.decorative`, `LaidOutDeck.lang` |
| `quick-xml` | `=0.38.0` | Parse core.xml in tests (to verify `dc:language`; compatible with ooxmlsdk 0.6.1) |
| `zip` | `=4.2.0` | Open PPTX ZIP in tests (compatible with ooxmlsdk 0.6.1 dep) |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-pptx/src/a11y.rs` | Create | `AltTextEmbedder` |
| `crates/slideforge-pptx/src/doc_props.rs` | Modify | Add `dc:language` to `CorePropertiesBuilder` |
| `crates/slideforge-pptx/src/slide_serializer.rs` | Modify | Call `AltTextEmbedder` for each shape |
| `crates/slideforge-pptx/src/tests/a11y_tests.rs` | Create | AC-001 through AC-007 tests |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,500 |
| BC-4.01.004 | ~1,500 |
| BC-5.01.005 | ~1,500 |
| STORY-037/038 code reference | ~1,500 |
| Test files | ~2,000 |
| **Total** | **~9,000** |

## Test Strategy

- **Unit tests**: All seven ACs; edge cases for special characters, long strings, lang
  variants.
- **Snapshot test**: `core.xml` with `dc:language` set to "fr-FR".
- **Integration marker**: Microsoft Accessibility Checker pass is tested manually per
  release (automated only via `NFR-015` PPTX linter in CI — not in this story).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All images on slide are decorative | All `descr=""` in slide XML; no violations |
| EC-002 | lang "zh-Hant-TW" (4-part BCP-47) | Embedded unchanged; no normalization |
| EC-003 | lang not declared (default "en") | `<dc:language>en</dc:language>` |
| EC-004 | Alt text contains XML special chars (`&`, `"`, `<`, `>`) | XML-escaped by ooxmlsdk automatically |
| EC-005 | 300-character alt text | Embedded in full; no truncation |

## Forbidden Dependencies

Same as STORY-037 — no upstream crate deps, no sibling exporter deps.
