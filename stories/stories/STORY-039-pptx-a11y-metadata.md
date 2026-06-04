---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-039
title: "PPTX Accessibility Metadata + Layout IR Alt-Threading"
epic: EPIC-08
wave: 4
points: 8
priority: P0
tdd_mode: strict
status: done
behavioral_contracts: [BC-4.01.004, BC-5.01.005]
verification_properties: []
nfr_refs: [NFR-014, NFR-015, NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-layout
target_module: slideforge-layout
additional_crates: [slideforge-pptx, slideforge-pdf]
subsystems: [SS-05, SS-06, SS-07]
depends_on:
  - STORY-038
  - STORY-015
blocks:
  - STORY-049
  - STORY-050
estimated_days: 3
---

# STORY-039: PPTX Accessibility Metadata + Layout IR Alt-Threading

## Subsystem Anchor Justification

This story touches three subsystems:

- **SS-05 (Layout Engine — slideforge-layout)**: Primary. The layout IR refactor
  (`FrameContent`) lives here. `slideforge-layout` owns the `LaidOutDeck` type;
  threading `AltText` through `FrameContent` is a change to SS-05's data model per
  ARCH-INDEX Subsystem Registry.
- **SS-06 (PPTX Export — slideforge-pptx)**: Secondary. `SlideSerializer` consumes
  the amended `FrameContent` and routes alt-text decisions through `AltTextEmbedder`.
  `build_doc_props` in `lib.rs` already writes `dc:language` (corrected scope per F-039-O1).
- **SS-07 (PDF Export — slideforge-pdf)**: Secondary. `tag_engine.rs` and `exporter.rs`
  currently treat all `FrameContent::Diagram` and `FrameContent::Chart` frames as
  Artifacts. With `AltText` in the IR, they can branch correctly on
  `AltText::{Provided, Decorative}`.

## Dependency Anchor Justifications

- Depends on STORY-038 (Layout Compliance): Alt text embedding requires shapes
  to be correctly structured with `<p:cNvPr>` elements, which are established by
  the layout compliance work. This story adds the `descr` attribute to those elements.
- Depends on STORY-015 (Alt Text Enforcement): By the time the PPTX exporter runs,
  every visual element in `LaidOutDeck` is guaranteed to have an `alt` field (or
  `decorative: true`) — enforced compile-time by STORY-015. This story embeds those
  values. The layout IR threading in this story is the data pipe that makes the
  STORY-015 guarantee available to exporters.
- Blocks STORY-049/050: Integration tests require accessibility metadata to pass
  the a11y assertions. Those tests exercise the full layout → export path, which
  requires both the IR alt-threading and the PPTX embedding to be in place.

## Summary

This story has two components that are tightly coupled and must be delivered
together:

### Component 1: Layout IR Alt-Threading (slideforge-layout — SS-05)

**Approved Scope Expansion — human-authorized 2026-06-03.**

A fresh-context adversary + architect assessment found that the layout IR
(`FrameContent` in `crates/slideforge-layout/src/types.rs`) drops alt text for
`Chart` and `Diagram` frames and loses the `Decorative` distinction for `Image`
frames, even though the semantic IR (`ChartSpec.alt`, `DiagramSpec.alt`,
`ImageSpec.alt` in `slideforge-types::specs`) carries it.

As a result, `slideforge-pptx` physically cannot embed chart or diagram alt text
(there is no field to read from the `LaidOutDeck`), and `slideforge-pdf` falls
back to treating all diagram/chart frames as PDF Artifacts — which is wrong for
non-decorative charts.

The fix: thread `AltText` through the layout IR. No new ADR is required — this
is governed by ADR-005 (Two-IR Model). Architect confirmed.

**Changes to `FrameContent` enum:**
- `Image { alt: Arc<str> }` → `Image { alt: AltText }` (add decorative distinction)
- `Chart` (no fields) → `Chart { alt: AltText }` (add alt text)
- `Diagram(NormalizedDiagramSvg)` → `Diagram { svg: NormalizedDiagramSvg, alt: AltText }`

**Changes to layout pass (`layout.rs`):**
- Thread `ChartSpec.alt`, `DiagramSpec.alt`, `ImageSpec.alt` through the
  `ContentBlock::Chart/Diagram/Image` arms when building `FrameContent`.

**Changes to region map (`regions.rs`):**
- Update hardcoded `FrameContent::Image { alt: Arc::from("") }` placeholders to
  `FrameContent::Image { alt: AltText::Decorative }`. These placeholders are
  structural stubs that the layout pass overwrites with real content — using
  `AltText::Decorative` is the safe default for stubs (explicitly opted out of
  alt, not silently empty).

### Component 2: PPTX Accessibility Embedding (slideforge-pptx — SS-06)

Extend the PPTX exporter with accessibility metadata embedding:

1. **Alt text**: Every non-decorative image, chart, or diagram shape in the PPTX
   gets `<p:cNvPr descr="alt text value">`. Decorative elements get `descr=""`.
2. **Language**: `docProps/core.xml` gets `<dc:language>LANG</dc:language>` from
   `LaidOutDeck.lang`. **NOTE:** `build_doc_props` in `lib.rs` is the correct
   implementation site (per STORY-037), but its fallback default was `"en-US"` — a
   bug. STORY-039 corrects the fallback to `"en"` (per BC-5.01.004). The function
   signature and XML-escaping are otherwise correct; only the `unwrap_or` value
   changes. The task also includes updating `SlideSerializer` to call `AltTextEmbedder`
   via the new `FrameContent::Chart { alt }` and `FrameContent::Diagram { svg, alt }` shapes.

### Component 3: PDF Tag Engine Fix (slideforge-pdf — SS-07)

Update `tag_engine.rs` and `exporter.rs` to branch on `AltText::{Provided, Decorative}`
for diagram and chart frames, replacing the current "treat all Diagram/Chart as Artifact"
fallback.

This story does NOT handle WCAG contrast (that is validated at compile time by
STORY-017). It only threads and embeds accessibility metadata that has already
been validated upstream.

### F-039-O1: Reconciliation — `doc_props.rs` vs `lib.rs`

The original story spec described `CorePropertiesBuilder` in `src/doc_props.rs`.
The actual STORY-037 implementation placed `build_doc_props` as a module-private
function in `crates/slideforge-pptx/src/lib.rs`. The function location is correct
and production-grade (it XML-escapes `dc:language` per F-037-006). However, its
`None`-fallback default was `"en-US"` — a bug relative to BC-5.01.004, which
specifies `"en"` as the canonical default. STORY-039 corrects that `unwrap_or` value
to `"en"`. This story does NOT create `src/doc_props.rs` or `CorePropertiesBuilder`.
Instead, STORY-039 also ensures that `build_doc_props` reads `LaidOutDeck.lang`
(currently it reads `deck.metadata.lang` from the semantic IR — verify this is the
canonical source of the already-resolved lang value and leave it if correct, or
update to read from `LaidOutDeck` if the evaluator resolves it there). File structure
table updated accordingly.

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

The `docProps/core.xml` update for BC-5.01.005 is already in place from STORY-037.
The `build_doc_props` function in `lib.rs` reads `deck.metadata.lang` and writes
`<dc:language>{lang}</dc:language>` with XML escaping. STORY-039 verifies this
code path with dedicated unit tests and ensures consistency with the `LaidOutDeck.lang`
field (which must equal `deck.metadata.lang` after evaluation — if not, update the
call site to use `laid_out.lang` when `LaidOutDeck` gains a `lang` field).

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties
  xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
  xmlns:dc="http://purl.org/dc/elements/1.1/"
  xmlns:dcterms="http://purl.org/dc/terms/">
  <dc:creator>slideforge</dc:creator>
  <dc:language>en-US</dc:language>   <!-- from deck.metadata.lang -->
  <dcterms:created xsi:type="dcterms:W3CDTF">1980-01-01T00:00:00Z</dcterms:created>
</cp:coreProperties>
```

The `dc:language` value is the exact BCP-47 tag from the source. No normalization,
no case change. "zh-Hant-TW" stays "zh-Hant-TW".

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

### AC-005: Charts and diagrams: alt on enclosing group shape from real FrameContent::Chart/Diagram frame
(traces to BC-4.01.004 edge case EC-005 — chart shape alt text)

A slide with a `slide chart:` block has its chart alt text stored in
`FrameContent::Chart { alt: AltText::Provided("...") }` in the `LaidOutDeck`, and
this alt text is embedded as `descr` on the `<p:grpSp>` or `<p:pic>` shape that
contains the chart SVG in the final PPTX. The alt text originates from `ChartSpec.alt`
in the semantic IR and is threaded through the layout IR — NOT sourced from an
`Image` frame proxy. A unit test constructs a `LaidOutSlide` with a
`FrameContent::Chart { alt: AltText::Provided(Arc::from("Q1 bar chart")) }` frame
and asserts the slide XML contains `descr="Q1 bar chart"` on the enclosing shape.
A parallel test covers `FrameContent::Diagram { svg: ..., alt: AltText::Provided(...) }`.

### AC-006: dc:language in core.xml matches deck lang exactly
(traces to BC-5.01.005 postcondition 1 — PPTX dc:language is exact BCP-47 tag)

A deck with `lang "en-US"` produces `<dc:language>en-US</dc:language>` in
`docProps/core.xml`. A deck with `lang "zh-Hant-TW"` produces
`<dc:language>zh-Hant-TW</dc:language>`. A unit test parses `core.xml` from the PPTX
ZIP and asserts the `dc:language` text content equals the deck lang exactly.

### AC-007: No lang declaration → core.xml contains default "en"
(traces to BC-5.01.005 edge case EC-003 — default is "en" per BC-5.01.004)

A deck with no `lang` declaration has `deck.metadata.lang = None`, which `build_doc_props`
must resolve to `"en"` (the canonical default per BC-5.01.004, not "en-US"). STORY-039
corrects the STORY-037 implementation from `unwrap_or("en-US")` to `unwrap_or("en")`.
A unit test verifies the PPTX embeds `<dc:language>en</dc:language>` when no lang is declared.

## Tasks

### IR Threading (slideforge-layout — primary SS-05 changes)

- [ ] Amend `FrameContent` in `crates/slideforge-layout/src/types.rs`:
  - `Image { alt: Arc<str> }` → `Image { alt: slideforge_types::AltText }`
  - `Chart` → `Chart { alt: slideforge_types::AltText }`
  - `Diagram(NormalizedDiagramSvg)` → `Diagram { svg: NormalizedDiagramSvg, alt: slideforge_types::AltText }`
- [ ] Update all `FrameContent::Image/Chart/Diagram` match sites in `slideforge-layout`
  (layout.rs, and any other match sites in the crate):
  - In `layout.rs`: thread `ChartSpec.alt`, `DiagramSpec.alt`, `ImageSpec.alt` into the new fields
    when constructing `FrameContent::Chart { alt }`, `FrameContent::Diagram { svg, alt }`,
    and `FrameContent::Image { alt }` from `ContentBlock::Chart/Diagram/Image`
  - Map `Option<AltText>` to `AltText`: `None` → `AltText::Decorative` (safe default; validator
    upstream guarantees this is an error path, but layout must not panic)
- [ ] Update `crates/slideforge-layout/src/regions.rs`:
  - Change `FrameContent::Image { alt: std::sync::Arc::from("") }` placeholder stubs to
    `FrameContent::Image { alt: slideforge_types::AltText::Decorative }`
  - These are structural stubs that `layout.rs` overwrites with real content; decorative is
    the correct sentinel value (explicit opt-out, not silent empty string)
- [ ] Fix all compile errors in downstream crates (`slideforge-pptx`, `slideforge-pdf`,
  and any other crate that matches on `FrameContent`) caused by the enum shape change
- [ ] Add unit tests in `crates/slideforge-layout/src/types.rs` (or nearby test module):
  - `FrameContent::Chart { alt: AltText::Provided(...) }` variant constructs and pattern-matches
  - `FrameContent::Diagram { svg: ..., alt: AltText::Decorative }` constructs and pattern-matches
  - `FrameContent::Image { alt: AltText::Provided(...) }` constructs and pattern-matches

### AltTextEmbedder (slideforge-pptx — SS-06)

- [ ] Implement `AltTextEmbedder` in `crates/slideforge-pptx/src/a11y.rs`:
  - Walk `LaidOutSlide.frames` for `FrameContent::Image { alt }`,
    `FrameContent::Chart { alt }`, `FrameContent::Diagram { svg: _, alt }` variants
  - For each frame: set `cNvPr.descr` to alt text string (non-decorative) or `""` (decorative)
  - `AltTextEmbedder` is the SINGLE authoritative code path for all `descr` decisions.
    No parallel inline `descr`-setting logic anywhere in `slideforge-pptx`
  - This module is NOT dead code — it is called from `SlideSerializer`
- [ ] Update `SlideSerializer` in `crates/slideforge-pptx/src/slide_serializer.rs` to call
  `AltTextEmbedder` for every image, chart, and diagram shape:
  - Update match arms for `FrameContent::Chart { alt }` and `FrameContent::Diagram { svg, alt }`
    to route through `AltTextEmbedder` (previously these had no `alt` field)
  - Remove any inline `descr`-setting logic and redirect to `AltTextEmbedder`

### Language Embedding (slideforge-pptx — SS-06)

- [ ] Verify `build_doc_props` in `crates/slideforge-pptx/src/lib.rs` reads
  `deck.metadata.lang` correctly (already implemented in STORY-037; no `src/doc_props.rs`
  or `CorePropertiesBuilder` needed — this is the correct implementation site)
- [ ] Update `ZipAssembler` assembly call (if needed) to ensure updated `core.xml`
  with `dc:language` is always included — verify it is already done in STORY-037

### PDF Tag Engine Fix (slideforge-pdf — SS-07)

- [ ] Update `crates/slideforge-pdf/src/tag_engine.rs`:
  - Replace "treat all Diagram/Chart as Artifact" fallback with proper
    `AltText::{Provided, Decorative}` branching:
    - `AltText::Provided(text)` → emit `Tag::Figure(Some(text.to_string()))` (tagged Figure)
    - `AltText::Decorative` → emit as Artifact (existing behavior for decorative-only case)
- [ ] Update `crates/slideforge-pdf/src/exporter.rs`:
  - Update `FrameContent::{Image, Chart, Diagram}` match sites to use the new struct fields
    (`FrameContent::Chart { alt }`, `FrameContent::Diagram { svg, alt }`)

### Tests

- [ ] Write unit tests in `crates/slideforge-pptx/src/tests/a11y_tests.rs` (create file):
  - Non-decorative image → non-empty `descr` (AC-001)
  - Decorative image → `descr=""` (AC-002)
  - Long alt text (300 chars) → not truncated (AC-003)
  - Special characters in alt text → XML-escaped (AC-004)
  - `FrameContent::Chart { alt: AltText::Provided(...) }` → alt on enclosing shape (AC-005)
  - `FrameContent::Diagram { svg: ..., alt: AltText::Provided(...) }` → alt on enclosing shape (AC-005)
  - `lang "en-US"` → `<dc:language>en-US</dc:language>` in core.xml (AC-006)
  - `lang "zh-Hant-TW"` → exact embedding (AC-006)
  - No lang → `<dc:language>en</dc:language>` (AC-007)
- [ ] Update existing test construction sites in `crates/slideforge-pptx/src/tests/`
  that construct `FrameContent::Chart`, `FrameContent::Diagram`, or `FrameContent::Image`
  to use the new struct-field syntax
- [ ] Update existing test construction sites in `crates/slideforge-pdf/tests/pdf_ua1.rs`
  that construct `FrameContent::Chart`, `FrameContent::Diagram`, or `FrameContent::Image`
  to use the new struct-field syntax

## Previous Story Intelligence

STORY-037 created `docProps/core.xml` via `build_doc_props` in `lib.rs`, which already
writes `<dc:language>{lang}</dc:language>` with XML escaping of the value from
`deck.metadata.lang`. There is no `src/doc_props.rs` or `CorePropertiesBuilder` — the
original story spec description of that file was a spec artifact that diverged from
the actual implementation. The real implementation site is the `build_doc_props`
function in `lib.rs`. This story verifies that code path with unit tests and does NOT
create a duplicate implementation.

STORY-015 (Alt Text Enforcement) guaranteed that every `ContentBlock` with a visual
element has either a non-empty `alt` field or `decorative: true` by the time layout
runs. This story threads those values from the semantic IR through the layout IR so
that exporters can read them. The layout IR is the only way exporters access content —
they never read the semantic IR directly (ADR-005).

STORY-038 established the `<p:cNvPr>` shape structure this story depends on.

## Architecture Compliance Rules

1. **`descr` on `<p:cNvPr>`, not `<p:ph altText>` (BC-4.01.004 invariant 2)**: The
   `altText` attribute on `<p:ph>` is for placeholder names only, not shape alt text.
   `AltTextEmbedder` must set `cNvPr.descr`, not `ph.altText`.
2. **Alt text never truncated (BC-4.01.004 invariant 1)**: The full string from
   `AltText::Provided(s)` is used. No length limit applied.
3. **Language propagation is lossless (BC-5.01.005 invariant 1)**: No truncation,
   normalization, or case change to the lang value. "zh-Hant-TW" stays "zh-Hant-TW".
4. **Single source of truth for lang (BC-5.01.005 invariant 2)**: Lang is read once
   from `deck.metadata.lang`. No hardcoded language values anywhere in `slideforge-pptx`.
5. **AltText threaded losslessly through layout IR (ADR-005 — Two-IR Model)**: The
   `AltText` value from `ChartSpec.alt`, `DiagramSpec.alt`, and `ImageSpec.alt` in
   the semantic IR MUST appear verbatim in `FrameContent::Chart { alt }`,
   `FrameContent::Diagram { svg, alt }`, and `FrameContent::Image { alt }` in the
   layout IR. No coercion, no silent dropping. `None` in the semantic IR (upstream
   validation error) maps to `AltText::Decorative` as a safe sentinel — this must
   emit a `tracing::warn!` because it indicates a validator miss upstream.
6. **`AltTextEmbedder` is the single authoritative `descr`-setting path (ADR-001)**: No
   code in `slideforge-pptx` may set `cNvPr.descr` outside of `AltTextEmbedder`. This
   prevents the parallel-path divergence that produced the original bug. `AltTextEmbedder`
   is the `descr` oracle; `SlideSerializer` calls it, never duplicates its logic.
7. **`src/doc_props.rs` must NOT be created (F-039-O1 reconciliation)**: The language
   embedding code lives in `build_doc_props` in `lib.rs`. Creating a parallel file
   would introduce duplicate logic. The spec error is corrected here.

## Forbidden Dependencies

- `slideforge-pptx` must NOT depend on `slideforge-pdf` or `slideforge-html`.
- `slideforge-layout` must NOT depend on any exporter crate (`slideforge-pptx`,
  `slideforge-pdf`, `slideforge-docx`, `slideforge-html`).
- If either of these dependencies appears in `Cargo.toml` after this story's work,
  the build MUST fail (enforced by `cargo deny`).

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `ooxmlsdk` | `=0.6.1` | `cNvPr.descr` attribute setting |
| `slideforge-types` | workspace | `AltText`, `FrameContent` field types, `ShapeFrame.alt`, `Deck.metadata.lang` (`DeckMetadata.lang: Option<Arc<str>>`) |
| `slideforge-layout` | workspace | `FrameContent` enum (now with alt fields); consumed by pptx + pdf |
| `quick-xml` | `=0.36.0` | Parse core.xml in tests (to verify `dc:language`; workspace pin — single Cargo.lock resolution entry) |
| `zip` | `=4.2.0` | Open PPTX ZIP in tests (compatible with ooxmlsdk 0.6.1 dep) |
| `krilla` | `=0.6.0` | PDF tagging API for `tag_engine.rs` fix (SS-07) |

## File Structure Requirements

| File | Crate | Action | Purpose |
|------|-------|--------|---------|
| `crates/slideforge-layout/src/types.rs` | slideforge-layout | Modify | Reshape `FrameContent::Image { alt }` to `AltText`, add `alt: AltText` to `Chart`, add `alt: AltText` to `Diagram` |
| `crates/slideforge-layout/src/layout.rs` | slideforge-layout | Modify | Thread `ChartSpec.alt` / `DiagramSpec.alt` / `ImageSpec.alt` through `run` when constructing `FrameContent` variants |
| `crates/slideforge-layout/src/regions.rs` | slideforge-layout | Modify | Update hardcoded `FrameContent::Image { alt: Arc::from("") }` stubs to `FrameContent::Image { alt: AltText::Decorative }` |
| `crates/slideforge-pptx/src/a11y.rs` | slideforge-pptx | Create | `AltTextEmbedder` — single authoritative `descr`-setting path |
| `crates/slideforge-pptx/src/slide_serializer.rs` | slideforge-pptx | Modify | Call `AltTextEmbedder` for every image/chart/diagram shape; update match arms to new `FrameContent` shape |
| `crates/slideforge-pptx/src/lib.rs` | slideforge-pptx | Verify | Confirm `build_doc_props` reads `deck.metadata.lang` correctly — no new file (`src/doc_props.rs` must NOT be created) |
| `crates/slideforge-pdf/src/tag_engine.rs` | slideforge-pdf | Modify | Replace Artifact fallback for Chart/Diagram with `AltText::{Provided, Decorative}` branching |
| `crates/slideforge-pdf/src/exporter.rs` | slideforge-pdf | Modify | Update `FrameContent::{Chart, Diagram}` match sites to new struct-field syntax |
| `crates/slideforge-pptx/src/tests/a11y_tests.rs` | slideforge-pptx | Create | AC-001 through AC-007 tests |
| `crates/slideforge-pptx/src/tests/core_tests.rs` | slideforge-pptx | Modify | Update `FrameContent::Chart/Diagram/Image` constructions to new struct syntax |
| `crates/slideforge-pdf/tests/pdf_ua1.rs` | slideforge-pdf | Modify | Update `FrameContent::Chart/Diagram/Image` constructions to new struct syntax |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~4,500 |
| BC-4.01.004 | ~1,500 |
| BC-5.01.005 | ~1,500 |
| slideforge-layout/src/types.rs (existing) | ~2,500 |
| slideforge-layout/src/layout.rs (relevant sections) | ~1,500 |
| slideforge-layout/src/regions.rs (relevant sections) | ~1,000 |
| slideforge-pptx/src/lib.rs (build_doc_props + build_slide_parts) | ~2,000 |
| slideforge-pptx/src/slide_serializer.rs | ~2,000 |
| slideforge-pdf/src/tag_engine.rs | ~1,500 |
| slideforge-pdf/src/exporter.rs (relevant sections) | ~1,000 |
| STORY-037/038 code reference | ~1,000 |
| Test files | ~3,000 |
| **Total** | **~23,000** |

Context budget: ~23k tokens across 3 crates. Well within the 20-30% context window
limit for a single implementer dispatch.

## Test Strategy

- **Unit tests**: All seven ACs; edge cases for special characters, long strings, lang variants.
- **Snapshot test**: `core.xml` with `dc:language` set to "fr-FR".
- **Compile-time coverage**: Exhaustive `match` on `FrameContent` variants catches any
  missed callsites when the enum shapes change.
- **Integration marker**: Microsoft Accessibility Checker pass is tested manually per
  release (automated only via `NFR-015` PPTX linter in CI — not in this story).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All images on slide are decorative | All `descr=""` in slide XML; no violations |
| EC-002 | lang "zh-Hant-TW" (4-part BCP-47) | Embedded unchanged; no normalization |
| EC-003 | lang not declared (default "en" per BC-5.01.004) | `<dc:language>en</dc:language>` |
| EC-004 | Alt text contains XML special chars (`&`, `"`, `<`, `>`) | XML-escaped by ooxmlsdk automatically |
| EC-005 | 300-character alt text | Embedded in full; no truncation |
| EC-006 | `ChartSpec.alt = None` reaches layout (upstream validator miss) | Map to `AltText::Decorative`; emit `tracing::warn!`; do NOT panic |
| EC-007 | `DiagramSpec.alt = None` reaches layout (upstream validator miss) | Same as EC-006 |
| EC-008 | Chart frame in PDF with non-decorative alt | `Tag::Figure(Some(alt_text))` emitted; NOT treated as Artifact |
