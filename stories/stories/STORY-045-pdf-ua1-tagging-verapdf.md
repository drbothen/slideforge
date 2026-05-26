---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-045
title: "PDF: PDF/UA-1 Tagging + veraPDF CI Gate"
epic: EPIC-13
wave: 4
points: 5
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
estimated_days: 3
---

# STORY-045: PDF — PDF/UA-1 Tagging + veraPDF CI Gate

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

Complete the PDF/UA-1 tagging pipeline in `slideforge-pdf` and add a veraPDF
validation CI gate:

1. Extend `SlideTagEngine` to produce a complete PDF/UA-1 structure tree:
   - `/StructTreeRoot` at document level.
   - `/MarkInfo << /Marked true >>`.
   - `/Lang` set from deck language.
   - Font `/ToUnicode` CMaps for all embedded fonts.
   - All text elements tagged with standard structure types.
   - All figures tagged as `/Figure` with `/Alt` from DSL `alt "..."`.
   - All decorative elements marked as PDF Artifacts.

2. Wire `PdfExporter::export()` to call `SlideTagEngine` for every slide.

3. Add a CI GitHub Actions job (`pdf-ua1-check`) that runs `verapdf --flavour ua1`
   against the test fixture PDF and fails if `isCompliant` is not `true`.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.03.001 | PDF Output Passes veraPDF --flavour ua1 (PDF/UA-1 Compliant, Tagged) | AC-001 through AC-009 |

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

## Tasks

- [ ] Extend `SlideTagEngine` in `src/tag_engine.rs`:
  - `build_struct_tree(deck: &LaidOutDeck) -> PdfStructTree`
  - `tag_text_element(element: &LaidOutElement) -> StructTag` (H1/H2/P/LI/Table)
  - `tag_figure(element: &LaidOutElement) -> StructTag` (sets /Alt)
  - `mark_artifact(element: &LaidOutElement) -> ArtifactMark`
- [ ] Update `PdfExporter::export()` to write `/StructTreeRoot` via pdf-writer
- [ ] Write `/MarkInfo` to document catalog
- [ ] Write `/Lang` to document catalog from `brand.lang` or `deck.lang`
- [ ] Write font `/ToUnicode` CMaps for each embedded font (using subsetter output)
- [ ] Write CI workflow `.github/workflows/pdf-ua1.yml`:
  - Job: `ubuntu-latest`, install `verapdf` via `apt` or Docker image
  - Build fixture PDF via `cargo test -p slideforge-pdf --test pdf_fixture`
  - Run `verapdf --flavour ua1 test-fixture.pdf`
  - Assert `isCompliant: true`
- [ ] Write integration test `tests/pdf_ua1.rs`:
  - Build 3-slide fixture: title + content + severity_cards, lang "en-US", all alts present
  - Export to PDF bytes
  - Run `verapdf --flavour ua1` (if available on CI), assert zero violations
  - Fallback: parse PDF structure tree and assert /StructTreeRoot, /MarkInfo, N /Part elements
- [ ] Write unit tests:
  - `SlideTagEngine::tag_text_element()` produces `/H1` for a title element
  - Decorative element → Artifact (not in structure tree)
  - `/Lang` = deck.lang value

## Previous Story Intelligence

STORY-043 established `SlideTagEngine` with stubs for `tag_slide()`, `tag_figure()`,
and `mark_artifact()`. STORY-044 established `emu_to_pt()` and `ir_y_to_pdf_y()`.

This story completes the `SlideTagEngine` implementation and adds the document-level
metadata (`/StructTreeRoot`, `/MarkInfo`, `/Lang`, font ToUnicode CMaps) that
PDF/UA-1 requires.

The CI workflow (`pdf-ua1.yml`) in this story depends on the CI infrastructure
established by STORY-051 (CI matrix job), which is in Wave 1. The pdf-ua1 job is
a separate workflow file, not an addition to the matrix job.

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
| `crates/slideforge-pdf/src/tag_engine.rs` | Modify | Complete SlideTagEngine for PDF/UA-1 |
| `crates/slideforge-pdf/src/exporter.rs` | Modify | Wire StructTreeRoot + MarkInfo + Lang |
| `crates/slideforge-pdf/tests/pdf_ua1.rs` | Create | Integration test with verapdf assertion |
| `crates/slideforge-pdf/tests/fixtures/fixture-deck.sf` | Create | 3-slide test fixture |
| `.github/workflows/pdf-ua1.yml` | Create | CI job: verapdf validation gate |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,500 |
| BC-4.03.001 | ~1,500 |
| STORY-043 tag_engine.rs (context) | ~1,500 |
| STORY-044 coords.rs (context) | ~800 |
| pdf-writer StructTreeRoot API | ~1,500 |
| Test files to write | ~2,000 |
| **Total** | **~9,800** |

Context budget: ~10% of a 100k-token context window. Within limit.

## Test Strategy

- **Unit tests**: `SlideTagEngine` produces correct tags for known IR inputs. Decorative
  element → Artifact. `/Lang` derived from deck metadata.
- **Integration test**: 3-slide fixture deck → PDF bytes → parse structure tree with
  pdf-writer reader → assert `/StructTreeRoot`, `/MarkInfo`, 3 `/Part` elements, `/Lang
  "en-US"`. Run `verapdf --flavour ua1` if available (skip via feature flag on dev
  machines, always run on CI).
- **CI gate**: `pdf-ua1.yml` workflow runs on every PR touching `slideforge-pdf/**`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Deck with only decorative images on one slide | Artifacts-only slide; /Part still present; verapdf passes |
| EC-002 | Chart SVG with > 10,000 path commands | Embedded as paths; may be large but verapdf passes (no raster images) |
| EC-003 | Math expression in slide title (PDF path rendering) | Math rendered as PDF paths; tagged as /P or /H; verapdf passes |
| EC-004 | Multilingual deck (doc-level lang "en-US") | /Lang at document = "en-US"; per-element lang override = v2 |
| EC-005 | Font unavailable at export time | E-BRD-004 warning; fallback font; ToUnicode CMap for fallback; verapdf passes |

## Forbidden Dependencies

Same as STORY-043. No Chrome, no headless browsers, no C-based PDF libraries.
`verapdf` is a CI tool, not a build dependency.
