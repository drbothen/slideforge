---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-102
title: "REND-010b: Image binary embedding in PPTX and PDF"
epic: EPIC-08
wave: 5
points: 8
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.0"
created: "2026-06-11"
source_findings: [REND-010]
behavioral_contracts: [BC-3.07.002]
verification_properties: []
nfr_refs: []
closes_findings: [REND-010]
depends_on:
  - STORY-037
  - STORY-043
  - STORY-044
  - STORY-039
  - STORY-101
blocks: []
target_module: slideforge-pptx, slideforge-pdf
subsystems: [SS-06, SS-07]
estimated_days: 4
---

# STORY-102: REND-010b — Image Binary Embedding in PPTX and PDF

## Subsystem Anchor Justification

SS-06 (PPTX Export) owns the PPTX side of this story: image binary data must be
written into the PPTX ZIP as a media part, with a slide relationship entry and a
content-type override. SS-07 (PDF Export) owns the PDF side: image data must be
embedded as an XObject via the pdf-writer / krilla stack. EPIC-08 anchors this story
because the PPTX ZIP assembly (STORY-037) and alt-text threading (STORY-039) are the
direct structural predecessors. STORY-101 anchors the REND-010b lineage — that story
implements chart SVG embedding (line ~653 in `slide_serializer.rs`) and updates the
`build_image_picture` comment (line ~1686) to cite `STORY-102` explicitly. This story
delivers the image embedding that the cited comment deferred.

## Dependency Anchor Justifications

- `depends_on: [STORY-037]` — PPTX core serializer and ZIP assembler; all PPTX media
  parts are added to the ZIP managed in STORY-037's output. Image media parts follow the
  same ZIP assembly pattern as chart SVG parts (STORY-101).
- `depends_on: [STORY-043]` — PDF core backend (pdf-writer + krilla); image XObject
  embedding requires the PDF content stream infrastructure from STORY-043.
- `depends_on: [STORY-044]` — PDF EMU-to-PDF coordinate mapping; image placement in
  the PDF stream uses the coordinate mapping established in STORY-044.
- `depends_on: [STORY-039]` — Layout IR alt-threading; `FrameContent::Image` carries
  `AltText` (not bare `Arc<str>`) after STORY-039's scope expansion. This story reads
  that `AltText` value when writing `<p:cNvPr descr="..."/>` in PPTX and the PDF
  structure element per BC-4.01.004 and BC-4.03.001.
- `depends_on: [STORY-101]` — STORY-101 places the `// image media embedding is deferred
  to STORY-102` comment at line ~1686 of `slide_serializer.rs`. This story implements
  the code that replaces that comment. Dispatching STORY-102 before STORY-101 would
  produce a conflict at that site.

## Narrative

As a slideforge user, I want image slides to produce a .pptx with the image binary
correctly embedded as a media part (not a blip-less empty placeholder), and a PDF
with the image binary embedded as an XObject, so that images are actually visible when
the output is opened in PowerPoint, Keynote, Google Slides, or any PDF reader.

## Previous Story Intelligence

STORY-037 built the PPTX serializer and ZIP assembler. The `build_image_picture`
function at line ~1686 intentionally deferred image binary embedding with a code
comment — no story ID was cited at that time, which was a CLAUDE.md discipline
violation. STORY-101 corrects the discipline violation by updating the comment to cite
`STORY-102`. STORY-039 expanded `FrameContent::Image` to carry `AltText` (not bare
`Arc<str>`), which this story reads for PPTX `descr` and PDF structure element alt text.
STORY-043 and STORY-044 provide the PDF coordinate machinery needed for XObject placement.

A demo-validated 8-byte stub PNG (the minimal PNG header with no image content) was
used in the demo that exposed REND-010b. Handling invalid or corrupt image files is
required per BC-3.07.002 EC-006 (file not found → E-EXP-001) and the production-grade
default (no silent fallback). The 8-byte stub PNG case must be a build error, not a
silent empty image.

## Architecture Compliance Rules

- **No blip-less `<p:pic>` may reach output .pptx (BC-3.07.002 Invariant 1).** Any
  code path that produces `<a:blip/>` with no `r:embed` attribute is a contract
  violation. The `build_image_picture` function MUST populate `r:embed`.
- **PPTX media part naming (BC-3.07.002 postcondition 1a):** images go in
  `ppt/media/imageN.<ext>` where N is 1-based sequential, `<ext>` matches the
  detected MIME type. PNG → `.png`, JPEG → `.jpg`, SVG → `.svg` (after usvg
  normalization per BC-1.12.003 / STORY-034).
- **Relationship entry (BC-3.07.002 postcondition 1b):** each image gets a relationship
  in `slide1.xml.rels` with Type `.../image` and Target `../media/imageN.<ext>`. Use
  `ooxmlsdk` 0.6.1 `Relationship` type per ADR-001.
- **Content-type override (BC-3.07.002 postcondition 1c):** `[Content_Types].xml` must
  have an `<Override>` or `<Default>` for the image MIME type.
- **Image path normalization (BC-3.07.002 postcondition 3):** resolve relative image
  paths against the normalized parent of the source file (same rule as BC-3.07.003 /
  STORY-101). A bare image path resolves against the deck directory, not `cwd`.
- **Verbatim bytes (BC-3.07.002 Invariant 3):** image bytes are written verbatim. No
  transcoding, resizing, or recompression in v1.0.
- **Alt text threading (BC-3.07.002 Invariant 4 + BC-4.01.004):** every `FrameContent::Image`
  carries `AltText::Provided(s)` or `AltText::Decorative`. Thread to `<p:cNvPr descr="..."/>`
  in PPTX and to the PDF structure element per BC-4.03.001.
- **Path-traversal check (BC-3.07.002 Invariant 5 + BC-1.16.001 EC-012 / E-VAL-012):**
  image paths have already passed E-VAL-012 before reaching this embedding stage. Do NOT
  re-open unvalidated paths.
- **Missing file error (BC-3.07.002 EC-006):** if the resolved image path does not exist
  at embed time, emit E-EXP-001 with the missing path; exit 3 (per error-taxonomy); no
  partial .pptx written (all-or-nothing, BC-3.03.002).
- **Invalid/corrupt image (production-grade default + CLAUDE.md "no silent fallback"):**
  an image that is present but has invalid content (e.g., a truncated PNG, an 8-byte
  stub PNG with a valid header but no IDAT chunk) MUST produce a build error (E-EXP-001
  or a new E-EXP sub-case), NOT a silently corrupt output. The demo's 8-byte stub PNG
  case is the canonical test vector for this requirement.
- **PDF (BC-3.07.002 postcondition 2):** each `FrameContent::Image` frame is embedded as
  an `XObject` in the PDF content stream; positioned at `frame.bbox` using BC-4.03.005
  coordinate mapping. veraPDF CI gate remains clean.
- **Multi-image deduplication (BC-3.07.002 postcondition 4):** two slides referencing the
  SAME image source file MAY share a single media part. Deduplication is allowed but not
  required in v1.0.
- Per CLAUDE.md: `Arc<str>` for string content, integer EMU for all position/size fields,
  `#![forbid(unsafe_code)]`, zero `.unwrap()` outside tests.

## Library & Framework Requirements

- `ooxmlsdk` 0.6.1 — `Relationship`, `ContentTypeOverride` types for PPTX media part
  registration (same API used by STORY-101 for chart SVG parts).
- `pdf-writer` + `krilla` stack (from STORY-043) — `XObject` image embedding.
- `std::fs::read` for loading image bytes verbatim; `std::path::Path` for path resolution.
- All version pins from `Cargo.lock`.

## File Structure Requirements

Files to modify:
- `crates/slideforge-pptx/src/slide_serializer.rs` — implement
  `build_image_picture` to embed image binary as media part; populate
  `<a:blip r:embed="rIdN"/>`. Remove the `// image media embedding is deferred to
  STORY-102` comment (placed by STORY-101) and replace it with the actual implementation.
- `crates/slideforge-pptx/src/lib.rs` — register image media parts in ZIP assembly;
  add relationship entries per slide; add content-type overrides.
- `crates/slideforge-pdf/src/exporter.rs` — implement `FrameContent::Image` path to
  embed image as XObject at `frame.bbox` coordinates.
- `crates/slideforge-pptx/src/tests/core_tests.rs` — add failing tests for image
  embedding (AC-001, AC-002, AC-003).
- `crates/slideforge-pdf/src/tests/` — add failing test for PDF XObject embedding (AC-004).

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| This story spec | ~2,500 |
| BC-3.07.002 (read in full) | ~1,500 |
| `crates/slideforge-pptx/src/slide_serializer.rs` (build_image_picture + blip fill) | ~3,000 |
| `crates/slideforge-pptx/src/lib.rs` (ZIP assembly, rels, content-types) | ~3,000 |
| `crates/slideforge-pdf/src/exporter.rs` (XObject embedding) | ~3,000 |
| Test files | ~3,000 |
| **Total** | **~16,000** |

Within 20-30% context budget. No split required.

## Acceptance Criteria

### AC-001: PPTX zip contains ppt/media/imageN.ext for each image in the deck
(traces to BC-3.07.002 postcondition 1a — image binary written to ppt/media/imageN.<ext>)

A .pptx built from a deck with one or more image slides contains the image binary at
`ppt/media/image1.png` (or `image1.jpg`, `image1.svg` per file extension). For decks
with N distinct images, files `ppt/media/image1.<ext>` through `ppt/media/imageN.<ext>`
exist in the PPTX ZIP.

Verified by: integration test building an image deck; unzip .pptx; assert file exists
at the expected path; assert file size > 0 (not empty/stub).

### AC-002: PPTX <a:blip r:embed> attribute populated for every image shape
(traces to BC-3.07.002 postcondition 1d + Invariant 1 — no blip-less <p:pic> in output)

For every image `<p:pic>` element in the PPTX output, the `<a:blip>` element within its
`<p:blipFill>` carries a non-empty `r:embed` attribute referencing the image's
relationship ID. The attribute value is NOT the empty string. No `<a:blip/>` with
no `r:embed` attribute reaches the output (BC-3.07.002 Invariant 1 — forbidden pattern).

Verified by: unit test parsing `slide1.xml`; assert every `<a:blip>` element has a
non-empty `r:embed` attribute value for image shapes.

### AC-003: slide rels and [Content_Types].xml correctly populated for each image
(traces to BC-3.07.002 postconditions 1b and 1c — relationship entry and content-type override)

The `slide1.xml.rels` file contains a relationship entry with:
- `Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"`
- `Target="../media/image1.png"` (or correct extension)

`[Content_Types].xml` has an `<Override>` or `<Default>` entry for the image MIME type
(`image/png`, `image/jpeg`, or `image/svg+xml`).

Verified by: integration test building image deck; unzip .pptx; parse `_rels/slide1.xml.rels`
and `[Content_Types].xml`; assert both entries present.

### AC-004: PDF output embeds image as XObject at correct position
(traces to BC-3.07.002 postcondition 2 — XObject embedded in PDF; BC-4.03.005 coordinate mapping)

A .pdf built from an image deck contains the image as an XObject in the PDF content
stream, positioned at the coordinates derived from `frame.bbox` via BC-4.03.005 mapping.
The veraPDF CI gate remains clean.

Verified by: integration test; assert PDF file contains embedded image data; veraPDF
gate passes.

### AC-005: Missing image file produces E-EXP-001 and exit 3
(traces to BC-3.07.002 EC-006 + BC-3.03.002 — all-or-nothing; no partial output)

`slideforge build deck.sf` where the deck references an image file that does not exist
at the resolved path exits with code 3 and emits E-EXP-001 citing the missing file path.
No .pptx or .pdf is written (all-or-nothing per BC-3.03.002).

Verified by: unit test building a deck with a non-existent image path; assert exit 3;
assert E-EXP-001 message in stderr citing the missing path; assert no output file written.

### AC-006: Invalid/corrupt image produces build error (8-byte stub PNG case)
(traces to BC-3.07.002 EC-006 / production-grade default — no silent fallback on corrupt image)

`slideforge build deck.sf` where the deck references a file that exists but has invalid
image content (specifically: a file with a valid PNG header magic bytes but no valid IDAT
chunk — the 8-byte stub PNG case from the demo) produces a build error (E-EXP-001 or an
image-format-specific variant), NOT a silently corrupt or blank output. The build exits
non-zero and no partial .pptx is written.

Verified by: unit test providing an 8-byte stub PNG (PNG magic: `\x89PNG\r\n\x1a\n`)
as the image source; assert non-zero exit and error message citing the corrupt image;
assert no output file written.

### AC-007: slide_serializer.rs "deferred to STORY-102" comment replaced with implementation
(discipline requirement — no unanchored deferrals per CLAUDE.md; continuity with STORY-101 AC-005)

After this story is merged, the code comment at line ~1686 of `slide_serializer.rs`
(placed by STORY-101 reading: `// image media embedding is deferred to STORY-102`) is
REMOVED (replaced by the actual implementation). No "deferred to STORY-102" comment
appears anywhere in the codebase after this merge.

Verified by: `grep -rn "deferred to STORY-102" crates/` returns zero results.

## Tasks

- [ ] **T-001:** Read BC-3.07.002 in full before writing any code. Read STORY-101's
  changes to `slide_serializer.rs` lines ~640-660 and ~1680-1690 to understand the
  chart SVG pattern and the image deferral comment location.
- [ ] **T-002:** Read STORY-039 to confirm the `FrameContent::Image` `AltText` field
  structure as implemented.
- [ ] **T-003 (RED):** Write `test_pptx_image_media_part_exists()` in
  `crates/slideforge-pptx/src/tests/core_tests.rs`.
- [ ] **T-004 (RED):** Write `test_pptx_image_blip_has_r_embed()`.
- [ ] **T-005 (RED):** Write `test_pptx_image_rels_and_content_types()`.
- [ ] **T-006 (RED):** Write `test_pdf_image_xobject_present()` in
  `crates/slideforge-pdf/src/tests/`.
- [ ] **T-007 (RED):** Write `test_missing_image_file_exits_e_exp_001()`.
- [ ] **T-008 (RED):** Write `test_corrupt_image_stub_png_exits_error()` using 8-byte
  stub PNG test fixture.
- [ ] **T-009 (RED):** Write `test_no_deferred_comment_for_story_102()` — `grep`-based
  assertion that "deferred to STORY-102" is absent (or embed in T-012 as a post-merge
  check).
- [ ] **T-010 (GREEN):** Implement `build_image_picture` image binary loading, media
  part writing, relationship entry, content-type override, and `r:embed` blip population
  in `slideforge-pptx`.
- [ ] **T-011 (GREEN):** Implement `FrameContent::Image` XObject embedding in
  `slideforge-pdf/src/exporter.rs`.
- [ ] **T-012:** Remove "deferred to STORY-102" comment from `slide_serializer.rs`.
  Run `grep -rn "deferred to STORY-102" crates/` — must return zero.
- [ ] **T-013:** Run `cargo nextest run -p slideforge-pptx -p slideforge-pdf --no-fail-fast`.
- [ ] **T-014:** Run `just check` before declaring done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Deck with no image slides | No `ppt/media/imageN.*` files; no image rels; clean PPTX |
| EC-002 | Two slides referencing the same image file | Both slides' `<a:blip>` reference a valid rId; deduplication allowed (BC-3.07.002 postcondition 4) |
| EC-003 | PNG image | Embedded as `ppt/media/image1.png`; `ContentType image/png` |
| EC-004 | JPEG image | Embedded as `ppt/media/image1.jpg`; `ContentType image/jpeg` |
| EC-005 | SVG image (after usvg normalization) | Embedded as `ppt/media/image1.svg`; `ContentType image/svg+xml` |
| EC-006 | Image file not found at resolved path | E-EXP-001 citing missing path; exit 3; no output (all-or-nothing) |
| EC-007 | 8-byte stub PNG (valid header, no IDAT) | Build error (E-EXP-001 or image-format variant); non-zero exit; no output |
| EC-008 | PDF output with image | Image as XObject; veraPDF passes |
| EC-009 | Bare image path `image: path "logo.png"` with bare deck path | Image path resolved against deck directory via BC-3.07.003 normalize_parent rule |
| EC-010 | `image: decorative: true` (no alt text) | `<p:cNvPr descr=""/>` in PPTX; PDF structure element uses empty alt (decorative artifact) |

## Behavioral Contracts Table

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| BC-3.07.002 | Image Binary Embedding in PPTX and PDF Output | AC-001 through AC-007 |
| BC-4.01.001 | Serialize LaidOutDeck to Valid .pptx | AC-001, AC-002, AC-003 |
| BC-3.03.002 | Strict Mode Produces No Output on Validation Error | AC-005, AC-006 |
| BC-4.03.001 | PDF/UA-1 compliant | AC-004 |
| BC-4.01.004 | PPTX Accessibility Metadata | AC-002 (alt text in descr) |

## Test Strategy

TDD strict mode. Seven failing tests (T-003 through T-009) before any implementation.
The 8-byte stub PNG test fixture (T-008) must be committed to `tests/fixtures/` as a
binary file — it is a compile-time test asset, not runtime-generated. The `grep`-based
T-009/T-012 check must run as part of the CI `just check` gate (or as a standalone test
that invokes `grep`).

The chart SVG embedding pattern from STORY-101 is the structural template for image
embedding: same ZIP assembly path, same rels file, same content-type override pattern.
Read STORY-101's implementation before writing the first GREEN test.

PDF XObject embedding (T-011): use the pdf-writer crate's `XObjectStream` API to embed
the image at the position derived from `frame.bbox` via BC-4.03.005. Read STORY-043 and
STORY-044 implementations before writing T-006.
