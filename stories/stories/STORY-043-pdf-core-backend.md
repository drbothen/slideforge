---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-043
title: "PDF Core: pdf-writer + krilla + SlideTagEngine"
epic: EPIC-13
wave: 4
points: 8
priority: P0
tdd_mode: strict
status: draft
# BC status: pending PO authorship — behavioral_contracts already specified below
behavioral_contracts: [BC-4.03.002]
verification_properties: []
nfr_refs: [NFR-012, NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
crate: slideforge-pdf
target_module: slideforge-pdf
subsystems: [SS-07]
depends_on:
  - STORY-026
  - STORY-034
blocks:
  - STORY-044
  - STORY-045
  - STORY-049
  - STORY-050
estimated_days: 4
---

# STORY-043: PDF Core — pdf-writer + krilla + SlideTagEngine

## Subsystem Anchor Justification

SS-07 (PDF Export) owns this story because it establishes the foundational PDF
generation stack for the `slideforge-pdf` crate. SS-07 per ARCH-INDEX Subsystem
Registry is "Effectful shell. pdf-writer + krilla. Phase 4." All subsequent PDF
stories (STORY-044, STORY-045) build on the core backend established here.

## Dependency Anchor Justifications

- Depends on STORY-026: `LaidOutDeck` and `LaidOutSlide` types from the layout engine
  are the inputs to the PDF exporter. This story cannot start until the IR types are
  stable.
- Depends on STORY-034: `NormalizedDiagramSvg` (from the SVG normalization step) is
  embedded as vector paths in PDF. The PDF backend must consume `NormalizedDiagramSvg`,
  not raw SVG strings.
- Blocks STORY-044: Y-axis coordinate mapping functions are implemented on top of the
  pdf-writer + krilla page abstraction established here.
- Blocks STORY-045: PDF/UA-1 tagging via `SlideTagEngine` uses the page-writing API
  established in this story.
- Blocks STORY-049/050: Plugin registry assembly and E2E tests require a working
  `PdfExporter` implementation.

## Summary

Implement the core `slideforge-pdf` crate: a pure-Rust PDF generation backend using
`pdf-writer 0.14.0` and `krilla 0.6.0`, with a custom `SlideTagEngine` for PDF/UA-1
structure tree generation. This story establishes:

1. The `PdfExporter` struct implementing the `Exporter` plugin trait.
2. The `SlideTagEngine` that maps semantic `LaidOutSlide` content to PDF structure tags
   (Document, Part, Sect, P, Figure, Table, H1-H6).
3. Font loading and subsetting via `krilla`'s built-in font subsetting (which
   internally uses the `subsetter` crate). For any paths outside krilla's text API
   (e.g., manual glyph positioning), the `subsetter` crate is used directly.
4. SVG path embedding from `NormalizedDiagramSvg` via `usvg` → pdf-writer path
   conversion.
5. No Chrome, no headless browser, no FFI to C-based PDF libraries — pure Rust only.

**Architecture note on pdf-writer vs krilla:** `krilla` is a high-level wrapper
around `pdf-writer` that handles text rendering, font subsetting, tagged PDF
structure, and page management. Use `krilla` as the primary API for page content
and text. Use `pdf-writer` directly only for low-level operations that krilla
does not expose (e.g., custom metadata dictionaries, raw content stream manipulation).
`krilla` re-exports or depends on `pdf-writer` internally — do NOT add a separate
`pdf-writer` dependency if krilla provides the needed API.

The crate compiles and passes tests on all 5 CI targets (macOS arm64/x86_64, Linux
x86_64/arm64, Windows x86_64) with no platform-specific code paths.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.03.002 | PDF Produced via pdf-writer + krilla + SlideTagEngine (No Chrome/Headless) | AC-001 through AC-009 |

## Acceptance Criteria

### AC-001: PdfExporter implements the Exporter plugin trait
(traces to BC-4.03.002 postcondition 1)

`PdfExporter` in `slideforge-pdf` implements the `Exporter` trait from
`slideforge-plugin-api`. The method signature is:
```
fn export(&self, deck: &LaidOutDeck, brand: &Brand, output: &mut dyn Write)
    -> Result<(), ExportError>
```
The function writes a valid PDF byte stream to `output` and returns `Ok(())` on
success. It never spawns a subprocess.

### AC-002: No Chrome or browser dependency in Cargo.toml
(traces to BC-4.03.002 invariant 1)

`crates/slideforge-pdf/Cargo.toml` does NOT list `chromium`, `headless-chrome`,
`puppeteer-rs`, or `wkhtmltopdf` in any dependency section. `cargo tree -p
slideforge-pdf` produces no output containing those crate names. This is verified
as a CI assertion: `grep -r 'headless\|chromium\|puppeteer\|wkhtmltopdf'
Cargo.lock` must produce zero matches.

### AC-003: SlideTagEngine maps LaidOutSlide to PDF structure tags
(traces to BC-4.03.002 postcondition 3)

`SlideTagEngine::tag_slide(slide: &LaidOutSlide, doc: &mut Document) -> TaggedSlide`
produces a tagged representation:
- One `/Part` element per slide in the PDF structure tree.
- Text elements tagged as H1/H2/P/LI per their semantic role in the IR.
- Figure elements tagged as `/Figure` with `/Alt` set from `LaidOutElement::alt`.
- Decorative elements (alt is empty and `decorative: true`) marked as PDF Artifacts.
- Table elements tagged as `/Table` with `/TR`, `/TH`, `/TD` sub-tags.

### AC-004: Font subsetting via subsetter crate
(traces to BC-4.03.002 invariant 3)

Font embedding uses the `subsetter` crate (v0.2.3). For each font used in the deck,
only the glyph subset required by that deck is embedded. The subsetting API:
```rust
use subsetter::{subset, GlyphRemapper};

// Create a remapper and register glyphs to keep
let mut remapper = GlyphRemapper::new();
for glyph_id in used_glyph_ids {
    remapper.remap(*glyph_id);
}

// Subset the font (index=0 for single-face fonts)
let subsetted: Vec<u8> = subsetter::subset(font_data, 0, &remapper)?;

// Get new glyph IDs for CID mapping in PDF
let new_id: u16 = remapper.get(old_glyph_id).unwrap();
```
is invoked before writing the font program to the PDF stream. The `GlyphRemapper`
provides the old-to-new glyph ID mapping needed for PDF CID font ToUnicode CMaps.
No system font tooling (`fonttools`, `pyftsubset`, etc.) is invoked.

**Note:** When using krilla's high-level text API (`surface.draw_text()` etc.),
krilla handles font subsetting internally -- the above manual subsetting flow is
only needed for direct pdf-writer font embedding paths (e.g., shapes not rendered
through krilla's text engine).

### AC-005: SVG paths embedded from NormalizedDiagramSvg
(traces to BC-4.03.002 postcondition 4)

For each `FrameContent::DiagramSvg(NormalizedDiagramSvg)` or
`FrameContent::ChartSvg(svg)` in `LaidOutSlide`, the PDF exporter:
1. Parses the SVG via `usvg::Tree::from_str()`.
2. Converts each SVG path to pdf-writer path operators.
3. Embeds the result as PDF vector content (not a raster image).
No rasterization occurs: `pdfimages -list output.pdf` must report zero images for a
chart-only deck.

### AC-006: Cross-platform compilation on all 5 CI targets
(traces to BC-4.03.002 postcondition 5)

`cargo build -p slideforge-pdf --target <T>` succeeds for all 5 targets:
- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

No platform-specific `#[cfg(target_os = "...")]` code paths in the PDF export
logic. pdf-writer and krilla are pure Rust.

### AC-007: PdfExporter is fully contained — no FFI
(traces to BC-4.03.002 invariant 4)

`cargo tree -p slideforge-pdf` produces no lines containing `libharu`, `cairo`,
`pango`, `freetype-sys`, `harfbuzz-sys`, or any C library binding. All PDF
operations use pure-Rust crates only.

### AC-008: Export does not spawn child processes
(traces to BC-4.03.002 postcondition 2)

Integration test: run `slideforge build --format pdf fixture.sf` under `strace`
(Linux) or `dtrace` (macOS) and assert zero `execve` syscalls are triggered by the
PDF export stage. On Windows, the integration test asserts that no
`CreateProcess` events are captured during PDF export via `tracert`.

### AC-009: Cargo.lock contains no browser-based PDF crates
(traces to BC-4.03.002 invariant 1)

After `cargo build --workspace`, inspect `Cargo.lock`. Assert: no entry for
`chromium`, `headless-chrome`, `puppeteer-rs`, or `wkhtmltopdf`. This is enforced
by a CI script (`scripts/check-pdf-deps.sh`) that greps `Cargo.lock` and exits 1
if a forbidden crate is found.

## Tasks

- [ ] Create `crates/slideforge-pdf/Cargo.toml` with dependencies:
  `krilla = "=0.6.0"` (primary PDF generation; brings in pdf-writer 0.14.0 + subsetter 0.2.3),
  `pdf-writer = "=0.14.0"` (direct dep for StructTreeRoot manipulation),
  `subsetter = "=0.2.3"` (direct dep only if manual font embedding needed),
  `usvg = "=0.47.0"`, `slideforge-plugin-api`, `slideforge-types`, `thiserror = "=2.0.18"`
  Note: krilla API uses `Document::new()`, `document.start_page_with(PageSettings)`,
  `page.surface()` for drawing, then `document.finish()` to get PDF bytes.
- [ ] Create `crates/slideforge-pdf/src/lib.rs` — re-export `PdfExporter`, feature gates
- [ ] Create `crates/slideforge-pdf/src/exporter.rs` — `PdfExporter` struct + `Exporter` impl
- [ ] Create `crates/slideforge-pdf/src/tag_engine.rs` — `SlideTagEngine`:
  `tag_slide()`, `tag_figure()`, `tag_table()`, `mark_artifact()`
- [ ] Create `crates/slideforge-pdf/src/font.rs` — font loading + subsetting via `subsetter`
- [ ] Create `crates/slideforge-pdf/src/svg_embed.rs` — `NormalizedDiagramSvg` → pdf-writer paths
- [ ] Create `crates/slideforge-pdf/src/error.rs` — `PdfExportError` enum with `thiserror`
- [ ] Add `slideforge-pdf` to root `Cargo.toml` workspace members
- [ ] Write unit tests:
  - `PdfExporter::export()` on minimal 1-slide deck produces non-empty bytes
  - `SlideTagEngine::tag_slide()` produces correct tag count for a known LaidOutSlide
  - `font::subset()` reduces font size for a single-glyph subset
  - `svg_embed::embed_svg()` converts a simple SVG rect to pdf-writer path ops
- [ ] Write integration test: `cargo build -p slideforge-pdf` produces no forbidden deps in Cargo.lock
- [ ] Create `scripts/check-pdf-deps.sh` CI enforcement script

## Previous Story Intelligence

N/A — first story in EPIC-13 (PDF Export). However, this story consumes:
- `LaidOutDeck` and `LaidOutSlide` from STORY-026 (layout core).
- `NormalizedDiagramSvg` from STORY-034 (SVG normalization).
- `Exporter` trait from STORY-002 (plugin trait API).
- `Brand` type from STORY-022 (brand loading).

The implementer must import these from their respective crates without violating
the dog-fooding principle (BC-5.02.002): use the public plugin-api trait only.

## Architecture Compliance Rules

1. **No Chrome, no headless browser (BC-4.03.002 invariant 1)**: Any attempt to add
   a browser-based PDF dependency is a blocking CI failure via `check-pdf-deps.sh`.
2. **Effectful shell classification (ARCH-INDEX SS-07)**: `PdfExporter::export()` is
   the effectful boundary. All pure PDF-writing logic (coordinate math, tag engine)
   lives in pure functions that take no I/O parameters.
3. **Plugin trait boundary (BC-5.02.002)**: `PdfExporter` must compile using only the
   public API from `slideforge-plugin-api`. No direct imports from `slideforge-pptx`
   or other exporter crates.
4. **Font subsetting via subsetter crate only (BC-4.03.002 invariant 3)**: Use
   `subsetter::subset(data, font_index, &remapper)` with `GlyphRemapper`. Do not use
   `rustybuzz`, `fontdb`, `hb-subset`, or system `fonttools` for subsetting.
5. **No unsafe code (NFR-024)**: `#![forbid(unsafe_code)]` at the crate root. pdf-writer
   and krilla are both safe Rust.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `pdf-writer` | `=0.14.0` | Low-level PDF stream/object writing (also a transitive dep of krilla; needed for direct struct tree manipulation) |
| `krilla` | `=0.6.0` | Higher-level PDF page abstraction (text, images, paths, tagged PDF, font subsetting). Supports PDF/UA-1 export mode. krilla internally depends on pdf-writer 0.14.0 and subsetter 0.2.3 -- no version conflict. |
| `subsetter` | `=0.2.3` | Font glyph subsetting -- only needed if manual font embedding is required outside krilla's text API. krilla already uses subsetter 0.2.3 internally with `variable-fonts` feature. |
| `usvg` | `=0.47.0` | SVG parsing for path extraction (shared with slideforge-diagrams) |
| `thiserror` | `=2.0.18` | `PdfExportError` enum derivation |
| `slideforge-plugin-api` | workspace | `Exporter` trait definition |
| `slideforge-types` | workspace | `LaidOutDeck`, `LaidOutSlide`, `Brand`, `Emu` |

Forbidden: `chromium`, `headless-chrome`, `puppeteer-rs`, `wkhtmltopdf`, `libharu`,
`cairo-rs`, `pango-sys`, `freetype-sys`.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-pdf/Cargo.toml` | Create | Crate manifest with pinned deps |
| `crates/slideforge-pdf/src/lib.rs` | Create | Re-exports, `#![forbid(unsafe_code)]` |
| `crates/slideforge-pdf/src/exporter.rs` | Create | `PdfExporter` + `Exporter` trait impl |
| `crates/slideforge-pdf/src/tag_engine.rs` | Create | `SlideTagEngine` struct |
| `crates/slideforge-pdf/src/font.rs` | Create | Font loading + subsetter integration |
| `crates/slideforge-pdf/src/svg_embed.rs` | Create | SVG → pdf-writer path conversion |
| `crates/slideforge-pdf/src/error.rs` | Create | `PdfExportError` with thiserror |
| `crates/slideforge-pdf/tests/no_forbidden_deps.rs` | Create | Cargo.lock assertion test |
| `scripts/check-pdf-deps.sh` | Create | CI guard against browser PDF deps |
| `Cargo.toml` (workspace root) | Modify | Add `slideforge-pdf` to members array |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,000 |
| BC-4.03.002 | ~1,200 |
| LaidOutDeck/LaidOutSlide types (STORY-026 output) | ~2,000 |
| Exporter trait (STORY-002) | ~800 |
| pdf-writer 0.14.0 API docs | ~2,000 |
| krilla 0.6.0 API docs | ~1,500 |
| subsetter 0.2.3 API docs | ~500 |
| Test files to write | ~2,500 |
| **Total** | **~13,500** |

Context budget: ~14% of a 100k-token context window. Within the 20-30% limit.

## Test Strategy

- **Unit tests**: `PdfExporter::export()` on 1-slide minimal deck → non-empty bytes
  that parse as valid PDF (check `%PDF-` header). `SlideTagEngine` produces correct
  `/Part` tags. Font subsetting reduces size. SVG rect → pdf-writer path ops.
- **Integration test**: `Cargo.lock` contains no forbidden PDF crates after workspace
  build. Cross-platform compilation via CI matrix (STORY-051).
- **Snapshot test**: None yet (PDF binary output not suitable for text snapshots —
  defer to STORY-045 which adds veraPDF structural validation).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | CI environment has no Chrome installed | Build succeeds; no PATH lookup for chrome |
| EC-002 | Chart SVG with complex clip paths | usvg normalizes; clip paths converted to standard paths |
| EC-003 | PDF export on Windows x86_64-pc-windows-msvc | Compilation succeeds; no Windows-specific PDF code |
| EC-004 | Concurrent PDF exports (multiple decks) | pdf-writer is thread-safe per export; no global mutable state |
| EC-005 | Empty font subset (no glyphs used) | `subsetter::subset()` returns minimal valid font or empty bytes; no panic |

## Forbidden Dependencies

The `slideforge-pdf` crate MUST NOT depend on:
- `slideforge-pptx`, `slideforge-docx`, `slideforge-html`, `slideforge-preview`
  (no cross-exporter dependencies — each exporter is independent)
- `chromium`, `headless-chrome`, `puppeteer-rs`, `wkhtmltopdf`
- `libharu`, `cairo-rs`, `pango-sys`, `freetype-sys` (no C library FFI)
- `image` crate for rasterization (SVG must be embedded as paths, not rasterized)

If `slideforge-pdf` gains any of these dependencies, the CI `check-pdf-deps.sh`
script MUST fail the build.
