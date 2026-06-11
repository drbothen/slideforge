---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: product-owner
timestamp: 2026-06-11T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-06
capability: CAP-015
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.07.002: Image Binary Embedding in PPTX and PDF Output

## Description

When a slide contains an image element (via the `image:` DSL block or an
`ImageSpec` in the evaluated IR), the image binary data MUST be embedded as a
media part within the PPTX ZIP archive and within the PDF stream. A `<p:pic>`
element with a blip-less `<a:blip/>` (no `r:embed` attribute) is a silent
corruption — the image renders as an empty placeholder in all consumers.
This contract provides the behavioral anchor for PPTX and PDF image-binary
embedding, closing the unanchored deferral flagged in REND-010b
(demo-deep-review-2026-06-11).

## Preconditions

1. The deck contains at least one slide with an `ImageSpec` resolved to a local
   file path (after BC-1.16.001 Stage-2b threading).
2. The image file exists on disk and passes the path-traversal security check
   (E-VAL-012 — path must not escape source root).
3. The PPTX ZIP assembler has been initialized (per BC-4.01.001).
4. The `LaidOutDeck` contains at least one `FrameContent::Image` with a non-empty
   resolved path.

## Postconditions

1. **PPTX (SS-06):** For each `FrameContent::Image` frame:
   a. The image binary is written to `ppt/media/imageN.<ext>` inside the PPTX ZIP,
      where `N` is a 1-based sequential integer per image (image1.png, image2.jpg, etc.)
      and `<ext>` matches the file's detected MIME type.
   b. A relationship entry is added to the slide's `.rels` file:
      `Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"`
      `Target="../media/imageN.<ext>"`
   c. `[Content_Types].xml` contains an `<Override>` or `<Default>` entry for the image
      MIME type (e.g., `ContentType="image/png"` for `.png`).
   d. The `<a:blip>` element within the image's `<p:pic>` carries a populated
      `r:embed="rIdN"` attribute referencing the relationship ID for the image media part.
      An empty `<a:blip/>` with no `r:embed` attribute is FORBIDDEN — it produces a
      silently invisible image in all PPTX consumers.

2. **PDF (SS-07):** For each `FrameContent::Image` frame:
   a. The image binary is embedded in the PDF content stream via the pdf-writer /
      krilla stack as an `XObject` image.
   b. The embedded image is referenced at the correct position and dimensions
      (derived from `frame.bbox` per BC-4.03.005 coordinate mapping).

3. **Image path normalization:** The resolved image path is the canonical absolute path
   produced by normalizing the author-supplied relative path against the source file's
   directory (same normalization rule as BC-3.07.003 / BC-1.15.001 path normalization).
   A bare filename with no directory component resolves against the source file's
   directory (not against `cwd`).

4. **Multi-image decks:** Each image receives a unique media part file name and a unique
   relationship ID. Two slides referencing the SAME image source file MAY share a single
   media part (deduplication is allowed but not required in v1.0).

5. **DOCX and HTML:** Image embedding for DOCX and HTML is governed by BC-4.02.001 and
   BC-4.03.003 respectively; those formats are out of scope for this BC.

## Invariants

1. No `<p:pic>` element with a blip-less `<a:blip/>` (missing `r:embed`) reaches the
   output .pptx. Any code path that produces a blip without `r:embed` is a contract
   violation. (CLAUDE.md "no silent fallback")

2. The `ppt/media/` folder in the PPTX ZIP is populated before slide XML is finalized.
   Relationship IDs must reference media parts that exist in the archive.

3. Image bytes are written verbatim — no transcoding, resizing, or recompression occurs
   in v1.0 (DI-021: no unauthorized transformations).

4. Per DI-001: every image frame MUST carry an `AltText::Provided(s)` or
   `AltText::Decorative` resolved value before this contract's postconditions apply.
   The alt text is threaded to `<p:cNvPr descr="..."/>` in PPTX and to the PDF
   structure element per BC-4.01.004 and BC-4.03.001.

5. The image path has already passed E-VAL-012 path-traversal check before reaching
   the embedding stage. The embedding stage MUST NOT re-open an unvalidated path.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Deck with no image slides | No media parts created; no rels entries; clean PPTX |
| EC-002 | Two slides referencing the same image file | Both slides reference the same embedded media part (deduplication allowed); both <a:blip> elements carry correct r:embed |
| EC-003 | Image file is a PNG | Embedded as ppt/media/image1.png; ContentType image/png |
| EC-004 | Image file is a JPEG | Embedded as ppt/media/image1.jpg; ContentType image/jpeg |
| EC-005 | Image file is an SVG | SVG passes usvg normalization (BC-1.12.003); embedded as ppt/media/image1.svg; ContentType image/svg+xml |
| EC-006 | Image file not found at resolved path | E-EXP-001 (PPTX serialization error) with message citing the missing path; no output produced (all-or-nothing per BC-3.03.002) |
| EC-007 | PDF output with image | Image embedded as XObject; veraPDF CI gate passes |
| EC-008 | slideforge build deck.sf with bare filename (image path relative to deck dir) | Image path resolved against deck directory (same rule as BC-3.07.003 / path normalization) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck with one image slide; `image: path "logo.png"` | PPTX zip contains `ppt/media/image1.png`; `slide1.xml.rels` has image relationship; `<a:blip r:embed="rId_img1"/>` non-empty | happy-path PPTX |
| Same deck → pdf | PDF contains XObject image; veraPDF passes | happy-path PDF |
| Deck with two distinct images on two slides | `ppt/media/image1.png`, `ppt/media/image2.jpg`; each slide's `<a:blip>` references its respective rId | multi-image |
| Deck with no images | No `ppt/media/` image files; clean zip | absent-image |
| Image file missing at build time | E-EXP-001 emitted; exit 3; no .pptx written | missing file |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | PPTX zip contains ppt/media/imageN.ext for each image in the deck | integration test: build image deck; unzip; assert file exists |
| VP-TBD | <a:blip r:embed> attribute is non-empty for every image shape in PPTX | unit test: parse slide XML; assert r:embed populated |
| VP-TBD | PDF image embedding: XObject present in PDF stream | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 |
| Capability Anchor Justification | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 — producing a valid .pptx with all referenced media embedded is a direct requirement of PPTX correctness. A blip-less image shape is an invalid .pptx. |
| L2 Domain Invariants | DI-001 (alt text on visual elements — images require alt), DI-010 (integer EMU coordinates for image placement), DI-012 (single .sf source produces all formats consistently) |
| REND Source | REND-010b (demo-deep-review-2026-06-11) — "PPTX image serializer emits blip-less `<p:pic>` with code comment 'deferred to a later story' but cites NO story ID" |
| Architecture Modules | SS-06 (PPTX Export — ZIP assembly, media parts, rels), SS-07 (PDF Export — XObject image embedding) |
| Stories | STORY-102 |

## Related BCs

- BC-4.01.001 — depends on (PPTX ZIP assembly and relationship mechanism)
- BC-4.03.001 — downstream of (PDF/UA-1 requires properly tagged embedded images)
- BC-3.07.003 — composes with (path normalization must run before image file is opened for embedding)
- BC-5.01.001 — depends on (alt text on image elements required before embedding)
- BC-1.11.001 — sibling (chart SVG embedding in PPTX follows the same media-part pattern; governed by BC-1.11.001 EC-004 and STORY-101)

## Architecture Anchors

- `architecture/module-decomposition.md` — SS-06 PPTX Export; ZIP assembly; media parts
- demo-deep-review-2026-06-11.md §REND-010 — discipline violation and deferral audit

## Story Anchor

STORY-102

## VP Anchors

(filled after VP creation)
