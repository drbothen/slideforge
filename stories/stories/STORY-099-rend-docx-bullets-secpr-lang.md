---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-099
title: "REND-006: DOCX bullet run content + numbering.xml + sectPr + lang on runs"
epic: EPIC-09
wave: 5
points: 8
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.2"
created: "2026-06-11"
source_findings: [REND-006]
behavioral_contracts: [BC-4.02.001, BC-5.01.005]
# BC status: BC-4.02.001 (serialize deck to .docx with correct body paragraphs — bullets
# emitted as empty <w:p/> is a postcondition violation; sectPr absence means DOCX
# page dimensions are undefined). BC-5.01.005 PC-4 (lang propagates to DOCX run-level
# <w:rPr><w:lang> — lang dropped from DOCX runs is a postcondition 4 violation).
# BC-3.05.001 REMOVED: no AC in this story traces to BC-3.05.001 postconditions.
# The depends_on: STORY-085 link references the run-properties path established there
# (a dependency reason), but the behavioral contract being closed here is BC-5.01.005
# PC-4, not BC-3.05.001 PC-3 inline-format universality. Both BCs are authored.
verification_properties: []
nfr_refs: []
closes_findings: [REND-006]
depends_on:
  - STORY-041
  - STORY-042
  - STORY-073
  - STORY-085
blocks: []
target_module: slideforge-docx
subsystems: [SS-08]
estimated_days: 4
---

# STORY-099: REND-006 — DOCX Bullet Run Content + numbering.xml + sectPr + lang

## Subsystem Anchor Justification

SS-08 (DOCX Export) owns all four defects: bullet run content, numbering definitions,
sectPr page dimensions, and run lang attribute. All fixes are in `slideforge-docx`.
EPIC-09 is the owning epic.

## Dependency Anchor Justifications

- `depends_on: [STORY-041]` — DOCX core serialization; bullet and run emission is in
  the serializer built here.
- `depends_on: [STORY-042]` — DOCX auto-generated sections; numbering.xml stub was
  created here for executive_summary section. This story populates it with actual
  numbering definitions.
- `depends_on: [STORY-073]` — ContentBlock::Bullets was established in STORY-073; the
  DOCX exporter must now correctly consume it.
- `depends_on: [STORY-085]` — InlineNode rendering in DOCX (BC-3.05.001 PC-3) — the
  run lang fix is part of the run properties path established here.

## Narrative

As a slideforge user, I want DOCX output to render bullet items as proper bulleted
paragraphs with numbered list definitions, correct page dimensions, and language
annotations on all text runs, so that the DOCX is editable in Word 365 and meets
accessibility requirements.

## Previous Story Intelligence

STORY-041 built the DOCX core. STORY-042 added auto-generated sections. STORY-085
implemented InlineNode rendering. Four defects remain in the merged code:

1. Bullet list items emit as empty `<w:p/>` elements — `<w:r>` run content is never
   populated.
2. `numbering.xml` exists as a stub with no abstract or concrete numbering definitions.
   Bullet paragraphs referencing numbering IDs have no backing definitions.
3. `sectPr` (section properties block carrying page width/height) is absent from
   `word/document.xml`. Word cannot determine page dimensions and falls back to
   defaults that may not match the deck's brand.
4. `<w:lang>` is dropped from all `<w:rPr>` run properties.

## Architecture Compliance Rules

- Per BC-4.02.001 postcondition 2: .docx must pass OOXML schema validation and open in
  Word 365 without schema errors. Absent sectPr and stub numbering.xml both cause
  schema violations.
- Per BC-5.01.005 PC-4 (DOCX run level): every `<w:rPr>` in `word/document.xml` must
  carry `<w:lang w:val="LANG"/>`. A `<w:rPr>` missing `<w:lang>` is a postcondition 4
  violation. Default when no lang declared: "en" (per BC-5.01.004) — NOT "en-US".
- Per ADR-001: use ooxmlsdk types; no raw XML string injection.
- `numbering.xml` must define at least one abstract numbering definition covering the
  bullet style. Multiple bullet levels (nested bullets, if supported) must each have a
  definition.
- `sectPr` must carry page dimensions from `brand.page_size` (or document-standard
  US Letter / A4 default if brand does not specify).
- DOCX crate MUST NOT depend on `slideforge-pptx`.

## Library & Framework Requirements

- `ooxmlsdk` 0.6.1 — `NumberingDefinitions`, `AbstractNum`, `Num`, `Level` types (confirm
  exact names by reading ooxmlsdk 0.6.1 source). `SectionProperties`, `PageSize` for sectPr.
- `slideforge-docx` — `numbering.rs` (may need creation), `document_body.rs`, `styles.rs`.

## File Structure Requirements

Files to create / modify:
- `crates/slideforge-docx/src/numbering.rs` — NEW or EXPAND: populate abstract numbering
  definitions for bullet lists (at least 3 levels). Build `numbering.xml` part.
- `crates/slideforge-docx/src/document_body.rs` — fix bullet paragraph emission to
  include `<w:r><w:t>...</w:t></w:r>` run content from `ContentBlock::Bullets` items;
  add `<w:lang>` to all `<w:rPr>` blocks.
- `crates/slideforge-docx/src/lib.rs` or exporter root — add `sectPr` with page
  dimensions as final element of `word/document.xml` body.
- Test files.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| This story spec | ~2,500 |
| `crates/slideforge-docx/src/document_body.rs` | ~4,000 |
| `crates/slideforge-docx/src/numbering.rs` (existing + expand) | ~2,500 |
| `crates/slideforge-docx/src/lib.rs` / exporter root | ~2,000 |
| ooxmlsdk types reference | ~1,500 |
| Test files | ~2,500 |
| **Total** | **~15,000** |

## Acceptance Criteria

### AC-001: Bullet items emit run content in DOCX paragraphs
(traces to BC-4.02.001 postcondition 7 — inline formatting preserved in DOCX runs)

A slide with `bullets: ["Point A", "Point B"]` produces a .docx where each bullet item
is a `<w:p>` element containing `<w:pPr><w:numPr>...</w:numPr></w:pPr>` for list
style AND a `<w:r><w:t>Point A</w:t></w:r>` run carrying the text. Empty `<w:p/>` elements
do NOT appear for bullet content.

Verified by: unit test building a content slide with bullets; unzip .docx; parse
`word/document.xml`; assert bullet paragraphs have non-empty `<w:r>` runs with `<w:t>`.

### AC-002: numbering.xml has abstract and concrete numbering definitions
(traces to BC-4.02.001 postcondition 2 — .docx passes OOXML schema validation)

`word/numbering.xml` is NOT a stub. It contains at least one `<w:abstractNum>` with
bullet symbol definitions for list levels 1-3, and at least one `<w:num>` referencing
that `<w:abstractNum>`. Bullet paragraphs reference a valid `<w:numId>` that maps back
to a defined concrete numbering.

Verified by: unit test; parse `word/numbering.xml`; assert `<w:abstractNum>` element
present with at least one `<w:lvl>` child. Assert `<w:num>` present referencing it.

### AC-003: sectPr present in word/document.xml with correct page dimensions
(traces to BC-4.02.001 postcondition 2 — OOXML schema compliance)

`word/document.xml` ends with a `<w:sectPr>` element containing `<w:pgSz>` with `w:w`
and `w:h` attributes matching the deck's brand page dimensions (in twentieths of a point,
i.e., EMU / 914400 * 1440). Default: A4 (11906 × 16838) or US Letter (12240 × 15840).

Verified by: unit test; unzip .docx; parse `word/document.xml`; assert final element is
`<w:sectPr>` containing `<w:pgSz w:w="..." w:h="..."/>`.

### AC-004: All DOCX text runs carry lang attribute
(traces to BC-5.01.005 PC-4 — every `<w:rPr>` in word/document.xml carries `<w:lang>`; default "en" per BC-5.01.004)

Every `<w:rPr>` in `word/document.xml` carries `<w:lang w:val="LANG"/>` (or the deck's
configured lang value) where LANG defaults to "en" when no lang is declared (per
BC-5.01.004, canonical default is "en" — NOT "en-US"). Runs with no existing `rPr`
element get one created with the lang child.

Verified by: unit test serializing a slide with body text; parse `word/document.xml`;
assert every `<w:r>` has `<w:rPr><w:lang w:val="en"/></w:rPr>` (no-lang-declared deck).
Second test with explicit `lang "fr-FR"` deck; assert `<w:lang w:val="fr-FR"/>` on runs.

### AC-005: DOCX file opens in Word 365 and LibreOffice Writer without errors
(traces to BC-4.02.001 postcondition 2)

The output .docx passes OOXML schema validation after AC-001 through AC-004 are
implemented. The integration test build produces a file that can be unzipped and
validated with the OOXML validator (or docx-rs schema check).

Verified by: existing integration test in STORY-041/042 + new schema validation assertion.

## Tasks

- [ ] **T-001 (RED):** Write `test_bullets_have_run_content()` in `slideforge-docx`.
- [ ] **T-002 (RED):** Write `test_numbering_xml_has_abstract_num()`.
- [ ] **T-003 (RED):** Write `test_document_xml_has_secpr()`.
- [ ] **T-004 (RED):** Write `test_runs_have_lang_attribute()`.
- [ ] **T-005 (GREEN):** Implement bullet run emission in `document_body.rs`.
- [ ] **T-006 (GREEN):** Populate `numbering.rs` with abstract + concrete numbering definitions.
- [ ] **T-007 (GREEN):** Add `sectPr` with `pgSz` from brand page dimensions to document body.
- [ ] **T-008 (GREEN):** Add `<w:lang>` to all `<w:rPr>` blocks in run emission path.
- [ ] **T-009:** Run `cargo nextest run -p slideforge-docx --no-fail-fast`.
- [ ] **T-010:** Run `just check` before declaring done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with no bullets | No numbering reference; clean paragraph; no crash |
| EC-002 | Nested bullets (2 levels) | Both levels reference correct abstract numId levels |
| EC-003 | Deck with no lang declaration | Default `"en"` on runs (per BC-5.01.004 — NOT "en-US") |
| EC-004 | Deck with explicit lang "de-DE" | All runs carry `<w:lang w:val="de-DE"/>` |
| EC-005 | Empty bullet string | Empty `<w:t/>` run (not dropped); consistent with Word behavior |

## Behavioral Contracts Table

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| BC-4.02.001 | Serialize Deck to .docx | AC-001, AC-002, AC-003, AC-005 |
| BC-5.01.005 | lang Declaration Propagates to PPTX Core Properties, PPTX Run rPr, PDF /Lang, HTML lang Attr | AC-004 |

## Test Strategy

TDD strict mode. Four failing tests first, one per defect area. These are independent
fixes within `slideforge-docx` and can each be tackled in order.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-06-11 | story-writer | Initial spec. |
| 1.2 | 2026-06-12 | story-writer | F-099-002 semantic mis-anchor fix: AC-004 re-routed from BC-3.05.001 (inline formats) to BC-5.01.005 PC-4 (DOCX run-level lang universality). BC-3.05.001 removed from behavioral_contracts — no AC in this story traces to its postconditions; the depends_on: STORY-085 note is a dependency reason, not a BC being implemented here. BC-5.01.005 added to behavioral_contracts. Architecture Compliance Rules updated to cite BC-5.01.005 PC-4 directly. Behavioral Contracts Table row updated to map AC-004 → BC-5.01.005. |
