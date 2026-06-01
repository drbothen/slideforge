---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-045
title: "PDF: PDF/UA-1 Tagging + veraPDF CI Gate (Full Compliance)"
epic: EPIC-13
wave: 4
points: 8
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.03.001]
verification_properties: []
nfr_refs: [NFR-012, NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-pdf
target_module: slideforge-pdf
subsystems: [SS-07]
depends_on:
  - STORY-043
  - STORY-044
blocks:
  - STORY-049
  - STORY-050
estimated_days: 4
---

# STORY-045: PDF — PDF/UA-1 Tagging + veraPDF CI Gate (Full Compliance)

## Subsystem Anchor Justification

SS-07 (PDF Export) owns PDF/UA-1 compliance because it is the PDF exporter's
responsibility to emit a correctly-tagged PDF byte stream. The compliance gate
(veraPDF) is the CI-level enforcement mechanism for the PDF accessibility contract.
ARCH-INDEX SS-07: "Effectful shell. pdf-writer + krilla. Phase 4."

## Dependency Anchor Justifications

- Depends on STORY-043: `SlideTagEngine` (established in STORY-043) is extended here
  to produce the full `/StructTreeRoot` hierarchy required by PDF/UA-1. Without the
  crate foundation and initial tag engine, this story cannot proceed.
- Depends on STORY-044: Correct element positioning in the PDF structure tree requires
  accurate coordinate mapping. veraPDF checks that element bounding boxes are within
  page dimensions; incorrect coordinates produce UA-1 violations.
- Blocks STORY-049/050: The plugin registry and E2E tests must be able to invoke
  `PdfExporter` and verify the output passes veraPDF validation.

## Summary

Complete the PDF/UA-1 tagging pipeline in `slideforge-pdf` and achieve full,
verifiable PDF/UA-1 compliance (`verapdf --flavour ua1` → `isCompliant: true`).
The expanded scope covers four workstreams:

1. **Structure tree (pre-existing scope):** Extend `SlideTagEngine` to produce a
   complete PDF/UA-1 structure tree:
   - `/StructTreeRoot` at document level.
   - `/MarkInfo << /Marked true >>`.
   - `/Lang` set from deck language.
   - Font `/ToUnicode` CMaps for all embedded fonts.
   - All text elements tagged with standard structure types.
   - All figures tagged as `/Figure` with `/Alt` from DSL `alt "..."`.
   - All decorative elements marked as PDF Artifacts.

2. **PDF document outline (bookmarks — NEW):** Generate a `/Outlines` tree in the
   document catalog. ISO 14289-1 §7.1 requires a document outline when the document
   contains headings. Every slideforge deck with H1/H2 content must have an outline.
   Default granularity: one bookmark entry per slide, using the slide title as the
   label, linking to the corresponding page destination.

3. **Heading-title text on Hn structure tags (NEW):** Every `/H1`–`/H6` structure
   element must carry a `/Title` attribute whose value is the slide or section heading
   text. Without this, veraPDF reports `MissingHeadingTitle` for every heading element.

4. **`Validator::UA1` enabled in production (NEW):** The `PdfExporter` production
   export path must use `Configuration::new_with_validator(Validator::UA1)` (krilla's
   built-in UA-1 validator). Any `KrillaError::Validation` is a fatal export error.
   The CI integration test must run with `Validator::UA1` and assert `isCompliant: true`.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.03.001 | PDF Output Passes veraPDF --flavour ua1 (PDF/UA-1 Compliant, Tagged) | AC-001 through AC-013 |

## Acceptance Criteria

### AC-001: StructTreeRoot present in PDF output
(traces to BC-4.03.001 postcondition 1 — /StructTreeRoot present)

The PDF produced by `PdfExporter::export()` contains a `/StructTreeRoot` dictionary
in the document catalog. Verified via:
1. `pdf-inspector` or similar tool parses the output and finds `/StructTreeRoot`.
2. Integration test: parse PDF bytes with `pdf-writer`'s reader API and assert the
   catalog contains `/StructTreeRoot`.

### AC-002: One /Part element per slide in structure tree
(traces to BC-4.03.001 postcondition 1 — one /Part per slide)

For a deck with N slides, the structure tree contains exactly N `/Part` elements
as direct children of the root. A 3-slide fixture deck produces exactly 3 `/Part`
elements in the structure tree.

### AC-003: Text elements tagged with correct semantic structure types
(traces to BC-4.03.001 postcondition 1 — text elements tagged)

For a fixture slide containing a title (H1), body bullets (P), and a list (LI),
the structure tree contains:
- Slide title → `/H1` or `/H` tag.
- Body paragraph → `/P` tag.
- List items → `/L` containing `/LI` tags.
No text content appears outside the structure tree (all marked content is tagged).

### AC-004: Figure elements tagged with /Alt text
(traces to BC-4.03.001 postcondition 1 — figures with /Alt)

For each non-decorative image/diagram/chart in `LaidOutSlide`, the structure tree
contains a `/Figure` element with `/Alt` set to the element's `alt` text from the
DSL. Verified: for a chart with `alt "Revenue by quarter"`, the structure tree
contains `/Figure /Alt (Revenue by quarter)`.

### AC-005: Decorative elements marked as PDF Artifacts
(traces to BC-4.03.001 postcondition 1 — decorative = Artifact)

Elements with `decorative: true` (empty alt text) are marked as PDF Artifacts
(`/Type /Marked /Marked true` or equivalent artifact marking). They do NOT appear
in the structure tree as tagged elements. Verified: a deck with one decorative
image produces zero `/Figure` elements with `/Alt` entries.

### AC-006: /MarkInfo << /Marked true >> present
(traces to BC-4.03.001 postcondition 1 — /MarkInfo)

The PDF document catalog contains `/MarkInfo << /Marked true >>`. This signals to
PDF readers that the document is a tagged PDF. Verified via structure inspection.

### AC-007: /Lang set from deck language
(traces to BC-4.03.001 postcondition 1 — /Lang set)

The document-level `/Lang` entry is set to the deck's `lang "..."` field value.
For `lang "en-US"`, `/Lang` = `"en-US"`. For `lang "de"`, `/Lang` = `"de"`.

### AC-008: veraPDF --flavour ua1 exits with isCompliant: true
(traces to BC-4.03.001 postcondition 2)

The CI job `pdf-ua1-check` runs:
```
verapdf --flavour ua1 tests/fixtures/output/test-fixture.pdf
```
and asserts `isCompliant: true` and `violations: 0` in the report output.
The CI job is defined in `.github/workflows/pdf-ua1.yml` and runs on every PR
that touches files matching `crates/slideforge-pdf/**` or
`.github/workflows/pdf-ua1.yml`.

### AC-009: Text is searchable and copyable
(traces to BC-4.03.001 postcondition 3)

The embedded font ToUnicode CMaps allow text extraction. Verified via:
`pdftotext output.pdf -` produces the expected slide text content (not empty, not
garbled). The CI job runs `pdftotext` on the fixture PDF and compares against a
known-good text snapshot.

### AC-010: PDF document outline (bookmarks) generated
(traces to BC-4.03.001 postcondition 1 — PDF document outline; added v1.1 full-UA-1 expansion)

The PDF produced by `PdfExporter::export()` contains a `/Outlines` dictionary in
the document catalog. For a deck with N slides, the outline contains exactly N
top-level outline entries (one per slide). Each entry:
- Has a label equal to the slide's title string (from `deck.slides[source_index].title_str()`
  or `""` if the slide has no `title` field, in which case the fallback label is
  `"Slide <N>"` where `<N>` is the 1-based slide number).
- Points to the correct page destination (the PDF page corresponding to that slide).

Verified by:
1. Unit test: `build_outlines()` on a 3-slide deck with titles `["Overview", "Data", "Summary"]`
   produces exactly 3 outline entries with those labels.
2. Integration test: parse the exported PDF and assert the `/Outlines` dictionary
   is present and has `Count: 3` for the 3-slide fixture deck.
3. veraPDF `isCompliant: true` assertion (AC-008) subsumes this check — outline
   absence triggers a veraPDF UA-1 violation.

### AC-011: Every Hn structure tag carries /Title attribute text
(traces to BC-4.03.001 postcondition 1 — heading-title text on Hn tags; added v1.1 full-UA-1 expansion)

Every `/H1`–`/H6` element in the structure tree carries a `/Title` attribute whose
value is the heading text string for that element. For a slide of type `title` with
`title: "Revenue Outlook"`, the `/H1` structure element has `/Title (Revenue Outlook)`.

The title string is sourced from the same field used for the bookmark label:
`deck.slides[laid_out_slide.source_index].title_str()`. For body-level headings
(H2–H6) that appear in bullet list frames, the title text is the text content of the
first inline run in that frame.

Verified by:
1. Unit test: `tag_text_element()` for a `FrameContent::Title` frame with title
   `"Revenue Outlook"` produces a structure tag with `/Title (Revenue Outlook)`.
2. veraPDF `isCompliant: true` assertion (AC-008) — missing `/Title` on any Hn
   element triggers `MissingHeadingTitle` violation, causing `isCompliant: false`.

### AC-012: Validator::UA1 enabled in production PdfExporter export path
(traces to BC-4.03.001 precondition 6 and invariant 5; added v1.1 full-UA-1 expansion)

`PdfExporter::export()` and `PdfExporter::export_uncompressed()` (the production paths)
MUST construct their krilla `Document` with `Configuration::new_with_validator(Validator::UA1)`.
Any `KrillaError::Validation` returned from krilla is propagated as a fatal
`PdfExportError::ValidationFailed(String)` — it MUST NOT be silently swallowed.

Test-only helpers (e.g., unit-test internal constructors) MAY use `Validator::None`
to avoid the overhead of UA-1 validation in fast unit tests, provided they are clearly
marked `#[cfg(test)]` and do NOT call the production `export()` path.

Verified by:
1. Code inspection: grep `Validator::None` must not appear in any non-`#[cfg(test)]`
   path in `src/exporter.rs`.
2. Unit test: call `PdfExporter::export()` with a structurally invalid deck (e.g.,
   a figure with empty alt and `decorative: false`) and assert the result is
   `Err(PdfExportError::ValidationFailed(_))`.

### AC-013: veraPDF integration test un-ignored in CI pdf-ua1 job
(traces to BC-4.03.001 postcondition 2; added v1.1 full-UA-1 expansion)

The integration test `tests/pdf_ua1.rs` contains at minimum one test function that:
- Calls `PdfExporter::export()` on the 3-slide fixture deck with `Validator::UA1`.
- Runs `verapdf --flavour ua1` on the exported PDF bytes (written to a temp file).
- Asserts `isCompliant: true` and `violations: 0` in the veraPDF JSON output.

This test MUST be `#[ignore]` for local development runs (because `verapdf` is not
available on all developer machines). In the CI workflow `pdf-ua1.yml`, the job MUST
invoke `cargo test -p slideforge-pdf --test pdf_ua1 -- --include-ignored` to un-ignore
the test and run it against the veraPDF CLI installed in CI.

The CI job gates on `isCompliant: true`. A `false` result or a missing veraPDF output
fails the job.

Verified by:
1. The `#[ignore]` attribute is present on the veraPDF-calling test function.
2. `.github/workflows/pdf-ua1.yml` contains `-- --include-ignored` in the cargo test
   invocation step.
3. CI run on a PR touching `crates/slideforge-pdf/**` passes the job.

## Tasks

### Workstream 1: Structure Tree (pre-existing scope)

- [ ] Extend `SlideTagEngine` in `src/tag_engine.rs`:
  - `build_struct_tree(deck: &Deck, laid_out: &LaidOutDeck) -> PdfStructTree`
  - `tag_text_element(frame: &Frame, slide_title: Option<&str>) -> StructTag`
    (H1/H2/P/LI/Table — carries `/Title` attribute for Hn tags, see Workstream 3)
  - `tag_figure(frame: &Frame) -> StructTag` (sets /Alt from DSL alt field)
  - `mark_artifact(frame: &Frame) -> ArtifactMark`
- [ ] Update `PdfExporter::export()` and `PdfExporter::export_uncompressed()` to write
  `/StructTreeRoot` via pdf-writer
- [ ] Write `/MarkInfo` to document catalog
- [ ] Write `/Lang` to document catalog from `deck.metadata.lang`
- [ ] Write font `/ToUnicode` CMaps for each embedded font (using subsetter output)

### Workstream 2: PDF Document Outline (NEW — AC-010)

- [ ] Add `build_outlines(deck: &Deck, laid_out: &LaidOutDeck) -> PdfOutlines` to
  `src/tag_engine.rs` (or a new `src/outline.rs` module):
  - Iterate `laid_out.slides` in order (slide index 0..N-1).
  - For each slide, resolve title: `deck.slides[laid_out_slide.source_index].title_str()`
    or fallback `format!("Slide {}", i + 1)` if `title_str()` returns `None`.
  - Produce one `OutlineEntry` per slide with the title label and a `GoTo` action
    pointing to the page destination for that slide.
- [ ] Wire `PdfExporter::export()` to write the `/Outlines` tree to the document catalog
  after the structure tree pass.
- [ ] Write unit tests:
  - `build_outlines()` on a 3-slide deck with titles produces 3 entries in order.
  - Slide with no `title` field gets fallback label `"Slide N"`.

### Workstream 3: Heading-Title Text on Hn Tags (NEW — AC-011)

- [ ] Update `tag_text_element()` (Workstream 1 task) to accept `slide_title: Option<&str>`:
  - For `FrameContent::Title` frames: set `/Title` on the `/H1` tag to the slide title
    string (from `deck.slides[source_index].title_str()`).
  - For `FrameContent::Body` frames rendered as H2–H6: set `/Title` to the first inline
    run's text content.
  - `/P`, `/LI`, `/L`, `/Table` tags do NOT need `/Title`.
- [ ] Write unit tests:
  - `tag_text_element()` for a Title frame with `slide_title = Some("Revenue Outlook")`
    produces a tag with attribute `/Title (Revenue Outlook)`.
  - `tag_text_element()` for a Body frame with H2 content produces `/Title` equal to the
    first inline run text.
  - `tag_text_element()` for a `P` frame does NOT produce a `/Title` attribute.

### Workstream 4: Validator::UA1 Enablement (NEW — AC-012)

- [ ] Replace `Validator::None` (or any non-UA-1 validator setting) in the production
  `PdfExporter::export()` and `PdfExporter::export_uncompressed()` paths with
  `Configuration::new_with_validator(Validator::UA1)`.
- [ ] Propagate `KrillaError::Validation` as `PdfExportError::ValidationFailed(String)`.
  Do NOT silently swallow it.
- [ ] Write unit test: export a structurally invalid deck (figure with empty alt +
  `decorative: false`) with `Validator::UA1` and assert `Err(PdfExportError::ValidationFailed(_))`.
  (Note: test-only helpers that use `Validator::None` for speed are acceptable, but must
  be `#[cfg(test)]` scoped and must NOT call the production `export()` entry point.)

### Workstream 5: CI Gate + Integration Test (updated — AC-008, AC-013)

- [ ] Write CI workflow `.github/workflows/pdf-ua1.yml`:
  - Job: `ubuntu-latest`, install `verapdf` via Docker image (`ghcr.io/verapdf/cli:latest`)
    or direct install with Java pre-installed.
  - Build fixture PDF via integration test in `tests/pdf_ua1.rs`.
  - Run `cargo test -p slideforge-pdf --test pdf_ua1 -- --include-ignored` to un-ignore
    the veraPDF-calling test.
  - Gate on `isCompliant: true` in veraPDF JSON output.
- [ ] Write integration test `tests/pdf_ua1.rs`:
  - Build 3-slide fixture: title slide (title: "Overview"), content slide (title: "Data"),
    severity_cards slide (title: "Risks"), lang "en-US", all alts present.
  - Export to PDF bytes via `PdfExporter::export()` (with `Validator::UA1`).
  - `#[ignore]` gate for the veraPDF assertion (local runs skip; CI un-ignores).
  - In the un-ignored path: write PDF bytes to `std::env::temp_dir()/pdf_ua1_fixture.pdf`,
    invoke `verapdf --flavour ua1 <path>`, parse JSON output, assert `isCompliant: true`
    and `violations: 0`.
  - Non-ignored fallback: parse PDF bytes with pdf-writer reader, assert `/StructTreeRoot`,
    `/MarkInfo`, `/Outlines`, 3 `/Part` elements, `/Lang "en-US"`.
- [ ] Existing unit tests (updated):
  - `SlideTagEngine::tag_text_element()` produces `/H1` with `/Title` for a title element.
  - Decorative element → Artifact (not in structure tree).
  - `/Lang` = `deck.metadata.lang` value.

## Previous Story Intelligence

STORY-043 established `SlideTagEngine` with stubs for `tag_slide()`, `tag_figure()`,
and `mark_artifact()`. STORY-044 established `emu_to_pt()` and `ir_y_to_pdf_y()`.

This story completes the `SlideTagEngine` implementation and adds the document-level
metadata (`/StructTreeRoot`, `/MarkInfo`, `/Lang`, font ToUnicode CMaps, `/Outlines`
tree, and `/Title` attributes on Hn tags) that PDF/UA-1 requires for full compliance.

The CI workflow (`pdf-ua1.yml`) in this story depends on the CI infrastructure
established by STORY-051 (CI matrix job), which is in Wave 1. The pdf-ua1 job is
a separate workflow file, not an addition to the matrix job.

**CRITICAL forward obligation from STORY-043 — frame-level Diagram/Chart placeholder alt:**
STORY-043's `SlideTagEngine` uses hardcoded placeholder alt strings (`"diagram"` /
`"chart"`) for frame-level `FrameContent::Diagram` and `FrameContent::Chart` because
`NormalizedDiagramSvg` carries no `alt` field in the IR. The non-UA-1 validator mode
used in STORY-043 does not reject these placeholders. STORY-045 MUST replace these
placeholder strings with real alt text sourced from `DiagramSpec.alt` /
`ChartSpec.alt` (or the corresponding IR field carrying the DSL `alt "..."` value)
before enabling `Validator::UA1`. If this is not done, `verapdf --flavour ua1` will
see a generic `"diagram"` or `"chart"` string as the `/Alt` value for every frame-level
diagram/chart and pass on an "alt lie" — a structural accessibility defect that
satisfies the syntax check but violates the semantic requirement that `/Alt` describes
the actual content. AC-004 in this story (figures tagged with /Alt text) MUST include
frame-level Diagram/Chart frames as part of its test fixture, not only inline images.

## Implementation Notes — Title String Sourcing (NO IR CHANGE REQUIRED)

**Where does the slide title come from for bookmarks and Hn /Title attributes?**

The `PdfExporter` already receives BOTH the semantic `Deck` and the geometric
`LaidOutDeck`. The title is accessible without any IR change:

```rust
// In PdfExporter (src/exporter.rs):
let slide_title: Option<&str> = deck.slides
    .get(laid_out_slide.source_index)
    .and_then(|s| s.title_str());
let label = slide_title.unwrap_or(&format!("Slide {}", i + 1));
```

- `deck: &Deck` is already a parameter of `export()` and `export_uncompressed()`.
- `Slide::title_str()` returns `Option<&str>` — `Some(title)` when `fields["title"]`
  is a resolved `Value::Str`, `None` otherwise.
- `LaidOutSlide::source_index` is the zero-based index into `deck.slides`.

**No changes to `slideforge-layout`, `slideforge-types`, or any upstream crate are
required.** The title chain is: `LaidOutSlide::source_index` → `Deck::slides[i]` →
`Slide::title_str()`. This is entirely within `slideforge-pdf`'s read access scope.

**Edge: slide with no title field.** Use fallback label `"Slide <N>"` (1-based). This
prevents a `None` from producing an empty or absent bookmark label, which would itself
be a UA-1 violation. The `/H1` tag on a no-title slide frame has `/Title` set to the
fallback label string.

**IR collision risk: NONE.** The required title data is already present in `Deck` and
reachable via `source_index`. This story does not require any upstream PR, does not
change any crate API, and does not block or collide with any other in-progress story.

## Architecture Compliance Rules

1. **Chrome is forbidden (BC-4.03.001 invariant 1 and BC-4.03.002)**: The CI job
   must NOT install Chrome or Chromium. `verapdf` is the only PDF validation tool.
2. **Structure tree reading order (BC-4.03.001 invariant 2)**: `SlideTagEngine`
   must walk `LaidOutSlide::reading_order` in order when tagging elements. Random
   iteration over element maps is forbidden for tagging.
3. **Every /Figure has /Alt (BC-4.03.001 invariant 3)**: The `tag_figure()` method
   panics in debug builds (and returns `PdfTagError::MissingAlt` in release) if
   `element.alt` is empty and `element.decorative` is false. This should not reach
   this stage (BC-5.01.001 catches it at validation), but defense in depth applies.
4. **Font subsetting via krilla/subsetter only (BC-4.03.001 invariant 4)**: ToUnicode
   CMaps are generated from the krilla font embedding pipeline (which uses subsetter
   0.2.3 internally with `GlyphRemapper` for glyph-to-CID mapping).

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `pdf-writer` | `=0.14.0` | Write /StructTreeRoot, /MarkInfo, /Lang to PDF |
| `krilla` | `=0.6.0` | Page-level tagged content stream integration |
| `subsetter` | `=0.2.3` | Font ToUnicode CMap generation (via GlyphRemapper) |
| `slideforge-types` | workspace | `LaidOutDeck`, `LaidOutSlide`, `LaidOutElement` |
| `verapdf` | CI tool (not a Rust dep) | PDF/UA-1 validation in CI |

Note: `verapdf` is a Java-based CLI tool installed in CI. It is NOT a Rust crate
and does NOT appear in Cargo.toml. Installation methods for CI:

1. **Docker (recommended for CI):**
   ```yaml
   # In GitHub Actions workflow:
   services:
     verapdf:
       image: ghcr.io/verapdf/cli:latest
   # Or run directly:
   # docker run --rm -v "$(pwd):/data" ghcr.io/verapdf/cli:latest --flavour ua1 /data/output.pdf
   ```

2. **Direct install (alternative):**
   ```bash
   # Requires Java 8/11/17/21 pre-installed
   wget http://downloads.verapdf.org/rel/verapdf-installer.zip
   unzip verapdf-installer.zip
   cd verapdf-*
   ./verapdf-install  # interactive installer
   # After install: verapdf --flavour ua1 output.pdf
   ```

3. **CI invocation:**
   ```bash
   verapdf --flavour ua1 output.pdf
   # Returns JSON with isCompliant: true/false
   ```

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-pdf/src/tag_engine.rs` | Modify | Complete SlideTagEngine: struct tree, /Title on Hn tags |
| `crates/slideforge-pdf/src/outline.rs` | Create | `build_outlines()` — PDF /Outlines bookmark tree (one entry per slide) |
| `crates/slideforge-pdf/src/exporter.rs` | Modify | Wire StructTreeRoot + MarkInfo + Lang + /Outlines; enable Validator::UA1 |
| `crates/slideforge-pdf/tests/pdf_ua1.rs` | Create | Integration test (structural assertions always; veraPDF #[ignore] for local) |
| `crates/slideforge-pdf/tests/fixtures/fixture-deck.sf` | Create | 3-slide fixture: title/content/severity_cards, lang "en-US", all alts |
| `.github/workflows/pdf-ua1.yml` | Create | CI job: cargo test --include-ignored + verapdf isCompliant gate |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec (expanded) | ~4,000 |
| BC-4.03.001 v1.3 | ~2,000 |
| STORY-043 tag_engine.rs (context) | ~1,500 |
| STORY-044 coords.rs (context) | ~800 |
| pdf-writer StructTreeRoot + Outlines API | ~2,000 |
| krilla Validator::UA1 API | ~600 |
| Test files to write | ~3,000 |
| **Total** | **~13,900** |

Context budget: ~14% of a 100k-token context window. Within limit.

## Test Strategy

- **Unit tests (always run, no external deps):**
  - `SlideTagEngine::tag_text_element()` produces `/H1` with `/Title` attribute for a
    `FrameContent::Title` frame.
  - `SlideTagEngine::tag_text_element()` for `/P` frame does NOT produce `/Title`.
  - `build_outlines()` on a 3-slide deck with known titles produces 3 entries with
    correct labels in source order.
  - Slide with no `title` field → fallback bookmark label `"Slide N"`.
  - Decorative element → Artifact (not in structure tree).
  - `/Lang` = `deck.metadata.lang` value.
  - `PdfExporter::export()` with a figure missing alt + `decorative: false` and
    `Validator::UA1` → `Err(PdfExportError::ValidationFailed(_))`.

- **Integration test (structural assertions, always run):** 3-slide fixture deck →
  PDF bytes → parse with pdf-writer reader → assert `/StructTreeRoot`, `/MarkInfo`,
  `/Outlines` present, 3 `/Part` elements, `/Lang "en-US"`.

- **Integration test (veraPDF, `#[ignore]` locally, un-ignored in CI):** 3-slide
  fixture → PDF bytes → write to temp file → `verapdf --flavour ua1` → assert
  `isCompliant: true`, `violations: 0`.

- **CI gate**: `pdf-ua1.yml` workflow runs on every PR touching `crates/slideforge-pdf/**`
  or `.github/workflows/pdf-ua1.yml`. Uses `cargo test --include-ignored` to run the
  `#[ignore]`'d veraPDF assertion.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Deck with only decorative images on one slide | Artifacts-only slide; /Part still present; verapdf passes |
| EC-002 | Chart SVG with > 10,000 path commands | Embedded as paths; may be large but verapdf passes (no raster images) |
| EC-003 | Math expression in slide title (PDF path rendering) | Math rendered as PDF paths; tagged as /P or /H; verapdf passes |
| EC-004 | Multilingual deck (doc-level lang "en-US") | /Lang at document = "en-US"; per-element lang override = v2 |
| EC-005 | Font unavailable at export time | E-BRD-004 warning; fallback font; ToUnicode CMap for fallback; verapdf passes |
| EC-006 | Slide with no `title` field in DSL | Bookmark label = "Slide N" (fallback); /H1 `/Title` = "Slide N"; outline entry still present; no empty-label violation |
| EC-007 | Deck with only one slide | /Outlines has exactly 1 entry; structure tree has 1 /Part; verapdf passes |
| EC-008 | Slide title contains non-ASCII characters (e.g., "Überblick") | Bookmark label and /Title attribute carry UTF-8 string; PDF string encoding uses UTF-16 BE with BOM per PDF spec; verapdf passes |
| EC-009 | Validator::UA1 raises KrillaError::Validation on a malformed deck | PdfExporter::export() returns Err(PdfExportError::ValidationFailed(msg)); error NOT silently swallowed |

## Forbidden Dependencies

Same as STORY-043. No Chrome, no headless browsers, no C-based PDF libraries.
`verapdf` is a CI tool, not a build dependency.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-31 | story-writer | Initial story creation — PDF/UA-1 tagging pipeline + veraPDF CI gate (AC-001 through AC-009). |
| 1.1 | 2026-06-01 | product-owner | Human-directed full-UA-1 scope expansion: implementer found that Validator::UA1 in production requires (a) PDF document outline and (b) Hn /Title attributes for veraPDF isCompliant:true. Added AC-010 (document outline/bookmarks), AC-011 (heading-title text on Hn tags), AC-012 (Validator::UA1 enabled in production export path), AC-013 (veraPDF #[ignore] integration test un-ignored in CI). Added Workstreams 2-4 to Tasks. Added src/outline.rs to File Structure. Updated Test Strategy and Edge Cases (EC-006 through EC-009). Added Implementation Notes section documenting title-string sourcing (Deck + source_index, no IR change required). Points bumped 5 → 8; estimated_days 3 → 4. BC-4.03.001 bumped to v1.3 with matching postcondition/invariant additions. |
