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
`krilla 0.6.0` as the primary PDF engine, with a custom `SlideTagEngine` for PDF/UA-1
structure tree generation. This story establishes:

1. The `PdfExporter` struct implementing the `Exporter` plugin trait.
2. The `SlideTagEngine` that maps semantic `LaidOutSlide` content to PDF structure tags
   via `krilla::tagging` (`TagKind::{Part,Sect,P,H1..H6,Figure,Table,TR,TH,TD,LI}`
   wrapped in `Tag` with attributes, assembled into a `TagTree` via `TagGroup`/`Node`,
   attached via `Document::set_tag_tree`). Note: `TagKind` has NO `Document` variant in
   krilla 0.6.0 — the PDF `/Document` root is auto-emitted by `TagTree::serialize()`.
3. Font loading and subsetting: krilla subsets fonts **internally** through its
   `Surface`/text API — this is the DEFAULT and satisfies the "subset only used glyphs"
   requirement. The `subsetter` crate (krilla 0.6.0 declares `^0.2.3`; Cargo.lock
   resolves to `0.2.4`) is only invoked directly when embedding a font glyph outside
   krilla's text engine (not expected in this story).
4. SVG path embedding from `NormalizedDiagramSvg` via `usvg` parse → krilla `Surface`
   path-drawing operations (vector, no rasterization). krilla does NOT depend on usvg;
   slideforge-pdf handles the usvg-parse → krilla-Surface path conversion.
5. No Chrome, no headless browser, no FFI to C-based PDF libraries — pure Rust only.

**Architecture note — krilla is the primary PDF API:**
`krilla 0.6.0` owns page content, text rendering, font subsetting, tagged PDF structure
tree, and PDF/UA-1 export mode. It wraps `pdf-writer 0.14.0` internally. **Do NOT add
a direct `pdf-writer` dependency** unless a concrete low-level need that krilla cannot
satisfy is proven during implementation — a parallel direct `pdf-writer` StructTreeRoot
would conflict with krilla's own tag tree. Similarly, **do NOT add a direct `subsetter`
dependency** unless manual glyph embedding outside krilla's Surface is required. Both
`pdf-writer 0.14.0` and `subsetter` arrive as transitive deps of krilla 0.6.0.
`pdf-writer` resolves to `0.14.0`; `subsetter` resolves to `0.2.4` (krilla declares
`^0.2.3`; Cargo.lock picks 0.2.4 — compatible, no API break). (Source:
`.factory/cycles/STORY-043/tech-validation.md`, RISK-1 and RISK-2.)

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
`slideforge-plugin-api`. The REAL trait signature (owned by STORY-002,
`crates/slideforge-plugin-api/src/traits/exporter.rs`) is:
```
fn export(&self, deck: &Deck, laid_out: &LaidOutDeck, brand: &Brand, opts: &ExportOptions)
    -> Result<Vec<u8>, ExportError>
```
The function receives BOTH the semantic `Deck` and the geometric `LaidOutDeck`,
plus `ExportOptions` (not a `Write` sink). It returns `Ok(Vec<u8>)` containing the
complete PDF byte stream on success. It never spawns a subprocess. Internally,
`Document::finish()` returns `KrillaResult<Vec<u8>>` (i.e., `Result<Vec<u8>, KrillaError>`)
— the implementation maps `KrillaError` into a `PdfExportError::Serialize` variant
via `?` (no `.unwrap()`). `Configuration::new_with(Validator, PdfVersion)` returns
`Option<Self>` — invalid combinations must also be mapped to a structured error, not
unwrapped.

### AC-002: No Chrome or browser dependency in Cargo.toml
(traces to BC-4.03.002 invariant 1)

`crates/slideforge-pdf/Cargo.toml` does NOT list `chromium`, `headless-chrome`,
`puppeteer-rs`, or `wkhtmltopdf` in any dependency section. `cargo tree -p
slideforge-pdf` produces no output containing those crate names. This is verified
as a CI assertion: `grep -r 'headless\|chromium\|puppeteer\|wkhtmltopdf'
Cargo.lock` must produce zero matches.

### AC-003: SlideTagEngine maps LaidOutSlide to PDF structure tags via krilla::tagging
(traces to BC-4.03.002 postcondition 3)

`SlideTagEngine::tag_slide(slide: &LaidOutSlide, doc: &mut Document) -> TagTree`
builds the structure tree using the `krilla::tagging` module and attaches it via
`Document::set_tag_tree(tag_tree)`. The implementation:

```rust
use krilla::tagging::{TagTree, TagGroup, TagKind, Tag, ContentTag, Node, Alt};

let mut tag_tree = TagTree::new();
// Per-slide: one Part group pushed directly onto the TagTree.
// NOTE: There is NO TagKind::Document variant in krilla 0.6.0 — do NOT attempt to
// create a TagGroup for Document. The PDF /Document root structure element is emitted
// AUTOMATICALLY by krilla's TagTree::serialize() as the implicit top-level container.
// User code builds: TagTree → Part* (one per slide) → {H1/H2/P/L+LI/Figure/Table...}
// Serialized PDF structure: /Document (auto) → /Part* → ...
// Adding an explicit Document group is impossible (no API) and would double the element.
let mut part_group = TagGroup::new(Tag::with(TagKind::Part));
// Text elements: tagged by semantic role
part_group.push(Node::Group(TagGroup::new(Tag::with(TagKind::H1))));
part_group.push(Node::Group(TagGroup::new(Tag::with(TagKind::P))));
// Figure elements: TagKind::Figure with /Alt from LaidOutElement::alt
// (NOT Tag::Figure — TagKind is the variant-bearing enum; Tag wraps kind+attrs)
let mut fig_tag = Tag::with(TagKind::Figure);
// set Alt via Tag attribute mechanism (consult docs.rs/krilla/0.6.0/krilla/tagging)
part_group.push(Node::Group(TagGroup::new(fig_tag)));
// Table: TagKind::Table → TagKind::TR → TagKind::TH / TagKind::TD
// Decorative (decorative:true, empty alt): mark as PDF Artifact, NOT tagged
tag_tree.push(Node::Group(part_group));
document.set_tag_tree(tag_tree);
```

Behavioral assertions:
- One `TagKind::Part` group per slide, pushed directly onto `TagTree` (NOT nested under
  a Document TagGroup — `TagKind::Document` does not exist in krilla 0.6.0).
- The PDF `/Document` root structure element is produced automatically by krilla's
  `TagTree::serialize()`; the serialized PDF structure is `/Document → /Part* → ...`
  even though user code only assembles `TagTree → Part*`.
- Text elements use `TagKind::{H1,H2,H3,H4,H5,H6,P,LI}` per semantic role in `LaidOutSlide`.
- Figure elements use `TagKind::Figure` (via `Tag`) with `/Alt` set from `LaidOutElement::alt`.
- Decorative elements (`decorative: true` + empty alt) are marked as PDF Artifacts — NOT wrapped in a `Tag`.
- Table elements: `TagKind::Table` → `TagKind::TR` → `TagKind::TH` / `TagKind::TD`.
- The complete `TagTree` is attached to the `Document` via `Document::set_tag_tree(tag_tree)` BEFORE `document.finish()` is called.

**RISK-3 (from tech-validation.md):** `TagKind` is the variant-bearing enum.
`Tag::Figure`-style references WILL NOT COMPILE — use `TagKind::Figure` wrapped in
`Tag::with(TagKind::Figure)`. `Tag` is the kind+attributes wrapper, not the enum.
Fine-grained `Tag` attribute builder ergonomics (how `/Alt` is set per-tag) must be
confirmed against the live `krilla::tagging` rustdoc during the RED test step.

### AC-004: Font loading into krilla — subsetting is krilla-internal (exercised in STORY-044)
(traces to BC-4.03.002 invariant 3)

The `font.rs` module provides font file loading helpers that make font data available
for use with krilla's Surface API. Subsetting happens automatically when text is drawn
via krilla's `Surface` in STORY-044. This story (043) establishes:

- `font::load_font_data(path: &Path) -> Result<Vec<u8>, PdfExportError>` — reads font
  bytes from disk.
- `font::system_font_fallback(family: &str) -> Option<PathBuf>` — locates a font by
  family name on the system.

No subsetting size assertion is required in STORY-043 because no glyphs are drawn in
043. The glyph-size regression test ("ASCII-only subset must be smaller than the full
font") is re-scoped to STORY-044 where text drawing makes it verifiable (see STORY-044
AC-009; scope rationale: `.factory/cycles/STORY-043/scope-directive.md` Decision 2).

BC-4.03.002 invariant 3 — "font subsetting via krilla, no system tooling" — is
unchanged. Only the test vehicle moves to STORY-044 where glyphs are actually drawn.

**Conditional direct `subsetter` usage (not expected in this story):**
If a specific glyph embedding path exists that bypasses krilla's Surface (e.g., manual
CID font embedding for a special shape), the `subsetter` API is:
```rust
use subsetter::{subset, GlyphRemapper};

let mut remapper = GlyphRemapper::new();
for glyph_id in &used_glyph_ids {
    remapper.remap(*glyph_id);
}
// index=0 for single-face fonts; 3rd param is named `mapper` in docs (positional call is identical)
let subsetted: Vec<u8> = subsetter::subset(font_data, 0, &remapper)?;
let new_id: Option<u16> = remapper.get(old_glyph_id);
```
This path is only activated if a concrete gap in krilla's text API is discovered during
the RED test step. **Do NOT add a direct `subsetter` Cargo dependency by default** —
it arrives as a transitive dep of krilla 0.6.0. (Source: tech-validation.md RISK-2.)

### AC-005: SVG paths embedded from NormalizedDiagramSvg via usvg → krilla Surface
(traces to BC-4.03.002 postcondition 4)

For each `FrameContent::DiagramSvg(NormalizedDiagramSvg)` or
`FrameContent::ChartSvg(svg)` in `LaidOutSlide`, the PDF exporter:
1. Parses the SVG via `usvg::Tree::from_str()`.
2. Walks the usvg node tree and issues corresponding path-drawing operations on the
   krilla `Surface` (e.g., `surface.draw_path(...)`, `surface.set_fill(...)`,
   `surface.set_stroke(...)`). **krilla owns the Surface; do NOT route SVG paths through
   pdf-writer path operators directly** — that would bypass krilla's coordinate space
   and layer management.
3. Embeds the result as PDF vector content (not a raster image).

Note: krilla 0.6.0 does NOT depend on usvg — there is no conflict with the workspace
`usvg =0.47.0`. The `krilla-svg` companion crate exists but is NOT used here;
slideforge-pdf handles its own usvg-parse → krilla-Surface path translation.

No rasterization occurs: `pdfimages -list output.pdf` must report zero images for a
chart-only deck. The `image` crate MUST NOT be used in `svg_embed.rs`.

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

The load-bearing in-scope assertion is a source-level grep enforced by
`scripts/check-pdf-deps.sh`: no occurrence of `std::process`, `Command::new`, or
`process::Command` may appear in `crates/slideforge-pdf/src/`. The script exits 1
if any such string is found, making this a CI hard failure.

The full syscall-tracer integration test — run `slideforge build --format pdf
fixture.sf` under `strace` (Linux) or `dtrace` (macOS) and assert zero `execve`
syscalls; assert no `CreateProcess` events on Windows — requires a complete CLI
binary and platform syscall tooling. That test is deferred to STORY-049:
`test_e2e_pdf_export_no_execve_syscall` (scope rationale: `.factory/cycles/STORY-043/
scope-directive.md` Decision 3; requires full CLI binary + syscall tracer).

### AC-009: Cargo.lock contains no browser-based PDF crates
(traces to BC-4.03.002 invariant 1)

After `cargo build --workspace`, inspect `Cargo.lock`. Assert: no entry for
`chromium`, `headless-chrome`, `puppeteer-rs`, or `wkhtmltopdf`. This is enforced
by a CI script (`scripts/check-pdf-deps.sh`) that greps `Cargo.lock` and exits 1
if a forbidden crate is found.

## Tasks

- [ ] Create `crates/slideforge-pdf/Cargo.toml` with dependencies:
  `krilla = "=0.6.0"` (PRIMARY PDF engine — owns page/surface/tagging/font-subsetting;
  brings in `pdf-writer ^0.14.0` resolving to `0.14.0` and `subsetter ^0.2.3` resolving
  to `0.2.4` as transitive deps at compatible versions),
  `usvg = { workspace = true }` (SVG parse → krilla Surface path conversion),
  `slideforge-plugin-api = { workspace = true }`,
  `slideforge-types = { workspace = true }`,
  `thiserror = { workspace = true }`.
  **Do NOT add `pdf-writer` or `subsetter` as direct deps by default** — they arrive
  transitively via krilla (`pdf-writer` resolves to `0.14.0`; `subsetter` resolves to
  `0.2.4` — krilla declares `^0.2.3`, Cargo.lock picks 0.2.4). Add a direct dep ONLY if
  a concrete krilla-impossible low-level need is proven during implementation (RISK-1 /
  RISK-2 per tech-validation.md). If added, pin `pdf-writer = "=0.14.0"` and
  `subsetter = "=0.2.4"` to match the Cargo.lock-resolved versions.
  krilla API: `Document::new()` / `Document::new_with(SerializeSettings)`,
  `document.start_page_with(PageSettings)`, `page.surface()` for drawing,
  `document.finish()` → `KrillaResult<Vec<u8>>` (map error → `PdfExportError::Serialize`).
- [ ] Create `crates/slideforge-pdf/src/lib.rs` — re-export `PdfExporter`, feature gates
- [ ] Create `crates/slideforge-pdf/src/exporter.rs` — `PdfExporter` struct + `Exporter` impl
- [ ] Create `crates/slideforge-pdf/src/tag_engine.rs` — `SlideTagEngine`:
  `tag_slide()`, `tag_figure()`, `tag_table()`, `mark_artifact()`.
  Use `krilla::tagging::{TagTree, TagGroup, TagKind, Tag, ContentTag, Node, Alt}`.
  **`TagKind` is the variant enum** (`TagKind::Part`, `TagKind::Figure`, etc.);
  `Tag` wraps a `TagKind` + attributes (incl. `/Alt`). Do NOT write `Tag::Figure` —
  that will not compile (RISK-3 per tech-validation.md). There is NO `TagKind::Document`
  variant — the PDF `/Document` root is auto-emitted by `TagTree::serialize()`. Push
  per-slide `TagKind::Part` groups directly onto `TagTree`.
  Attach via `document.set_tag_tree(tag_tree)` before `document.finish()`.
  PDF/UA-1 configuration (for STORY-045 forward compatibility): use
  `krilla::configure::validate::Validator::UA1` (note nested `::validate::` module),
  `krilla::configure::Configuration`, and top-level `krilla::SerializeSettings`
  (NOT `krilla::serialize::SerializeSettings`). `Configuration::new_with(Validator, PdfVersion)`
  returns `Option<Self>` — map `None` to a structured error.
- [ ] Create `crates/slideforge-pdf/src/font.rs` — font loading. Font subsetting is
  handled internally by krilla when drawing text via `Surface`; this module provides
  font file loading helpers. If a direct glyph-embedding path outside krilla's Surface
  is required, `subsetter::{subset, GlyphRemapper}` (transitive dep, no direct dep
  needed) may be used — but only if a concrete gap is proven during the RED test step.
- [ ] Create `crates/slideforge-pdf/src/svg_embed.rs` — `NormalizedDiagramSvg` →
  krilla `Surface` path-drawing operations. Parse via `usvg::Tree::from_str()`, walk
  node tree, issue `surface.draw_path(...)` / `surface.set_fill(...)` /
  `surface.set_stroke(...)` calls. No pdf-writer direct path operators; no rasterization.
- [ ] Create `crates/slideforge-pdf/src/error.rs` — `PdfExportError` enum with `thiserror`
- [ ] Add `slideforge-pdf` to root `Cargo.toml` workspace members
- [ ] Write unit tests:
  - `PdfExporter::export()` on minimal 1-slide deck produces non-empty bytes
  - `SlideTagEngine::tag_slide()` produces correct tag count for a known LaidOutSlide
  - `font::load()` loads a font file and exposes it for use with krilla's Surface
  - `svg_embed::embed_svg()` converts a simple SVG rect to krilla Surface draw calls
    (verify no rasterization: check PDF output via `%PDF-` header + no `/Image` entry)
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

### Tech Validation (2026-05-31)

Pre-implementation technology validation was run against the live crates.io registry
and docs.rs/krilla/0.6.0 before coding began. Full findings:
`.factory/cycles/STORY-043/tech-validation.md`

**Critical architecture decisions from that validation:**

1. **krilla owns tagging AND subsetting.** krilla 0.6.0's `krilla::tagging` module
   (`TagTree`/`TagGroup`/`TagKind`/`Tag`/`Node` → `Document::set_tag_tree`) owns the
   entire PDF structure tree. **Do NOT** hand-write `StructTreeRoot` via a parallel
   direct `pdf-writer` handle — it would double-emit and conflict with krilla's tree
   (RISK-1). Similarly, krilla subsets fonts internally via its Surface/text API —
   `subsetter::subset(...)` is not needed for the normal text-drawing path (RISK-2).

2. **`TagKind` is the variant enum, not `Tag`.** `Tag::Figure` will not compile.
   Use `TagKind::Figure` wrapped in `Tag::with(TagKind::Figure)` (RISK-3).

3. **PDF/UA-1 module paths are `configure::validate::Validator::UA1`,
   `configure::Configuration`, and top-level `SerializeSettings`** — NOT
   `configure::Validator` or `serialize::SerializeSettings` (RISK-4).

4. **`Document::finish()` returns `KrillaResult<Vec<u8>>`** (not `Vec<u8>`) — map
   `KrillaError` to `PdfExportError::Serialize` via `?` (RISK-5).

5. **`Configuration::new_with(Validator, PdfVersion)` returns `Option<Self>`** —
   map `None` to a structured error (RISK-6).

6. **`usvg` and `krilla` have no shared dependency.** krilla 0.6.0 does not depend
   on usvg; the workspace `usvg =0.47.0` is safe to use in slideforge-pdf alongside
   krilla with zero version conflicts.

## Architecture Compliance Rules

1. **No Chrome, no headless browser (BC-4.03.002 invariant 1)**: Any attempt to add
   a browser-based PDF dependency is a blocking CI failure via `check-pdf-deps.sh`.
2. **Effectful shell classification (ARCH-INDEX SS-07)**: `PdfExporter::export()` is
   the effectful boundary. All pure PDF-writing logic (coordinate math, tag engine)
   lives in pure functions that take no I/O parameters.
3. **Plugin trait boundary (BC-5.02.002)**: `PdfExporter` must compile using only the
   public API from `slideforge-plugin-api`. No direct imports from `slideforge-pptx`
   or other exporter crates.
4. **Font subsetting — no system font tooling (BC-4.03.002 invariant 3)**: The
   DEFAULT subsetting mechanism is krilla's internal subsetting via its Surface/text API.
   No `rustybuzz`, `fontdb`, `hb-subset`, or system `fonttools` may be used. If a
   direct glyph-embedding path is required, use `subsetter::{subset, GlyphRemapper}` —
   but only as a conditional, not the primary mechanism. (Tech-validation RISK-2.)
5. **No unsafe code (NFR-024)**: `#![forbid(unsafe_code)]` at the crate root. pdf-writer
   and krilla are both safe Rust.
6. **Frame-level Diagram/Chart alt is a known placeholder pending STORY-045.**
   `SlideTagEngine` in this story threads REAL alt text for body-level and Image
   `LaidOutElement` figures (from `LaidOutElement::alt`). However, for frame-level
   `FrameContent::Diagram` and `FrameContent::Chart`, `NormalizedDiagramSvg` carries
   no `alt` field in the IR — so the tag engine emits hardcoded placeholder alt strings
   (`"diagram"` / `"chart"`). With the DEFAULT (non-UA-1) `krilla` validator mode used
   in STORY-043, these placeholders do NOT trigger a validation failure. This is
   intentional and tracked: STORY-045 MUST replace these placeholders with real alt
   text sourced from `DiagramSpec.alt` / `ChartSpec.alt` before enabling
   `Validator::UA1`, otherwise veraPDF will pass on a placeholder "alt lie." See
   STORY-045 Previous Story Intelligence for the forward obligation.

## Library & Framework Requirements

| Library | Cargo.toml entry | Purpose |
|---------|-----------------|---------|
| `krilla` | `= "=0.6.0"` (direct, PRIMARY) | PDF page generation, text, images, paths, tagged PDF structure tree, internal font subsetting. Brings pdf-writer 0.14.0 and subsetter 0.2.4 as transitive deps (krilla declares `^0.2.3`; Cargo.lock resolves 0.2.4). Supports PDF/UA-1 via `configure::validate::Validator::UA1`. |
| `usvg` | `{ workspace = true }` (=0.47.0, direct) | SVG parsing for usvg-node → krilla Surface path conversion. No conflict: krilla 0.6.0 has zero usvg dependency. |
| `thiserror` | `{ workspace = true }` (=2.0.18, direct) | `PdfExportError` enum derivation. Do NOT re-pin independently. |
| `slideforge-plugin-api` | `{ workspace = true }` | `Exporter` trait definition |
| `slideforge-types` | `{ workspace = true }` | `LaidOutDeck`, `LaidOutSlide`, `Brand`, `Emu` |
| `pdf-writer` | **NOT a default direct dep** — transitive via krilla (=0.14.0). Add direct dep ONLY if a concrete krilla-impossible low-level need is proven. If added, pin `= "=0.14.0"` (NOT 0.15.0 — that splits from krilla 0.6.0's tree). | Low-level PDF stream/object writing |
| `subsetter` | **NOT a default direct dep** — transitive via krilla. krilla 0.6.0 declares `^0.2.3`; Cargo.lock resolves to `0.2.4` (compatible semver bump, no API break). Add direct dep ONLY if manual glyph embedding outside krilla's Surface is required. If added, pin `= "=0.2.4"` to match the resolved version. | Font glyph subsetting (krilla subsets internally for normal text paths) |

Forbidden: `chromium`, `headless-chrome`, `puppeteer-rs`, `wkhtmltopdf`, `libharu`,
`cairo-rs`, `pango-sys`, `freetype-sys`.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-pdf/Cargo.toml` | Create | Crate manifest with pinned deps |
| `crates/slideforge-pdf/src/lib.rs` | Create | Re-exports, `#![forbid(unsafe_code)]` |
| `crates/slideforge-pdf/src/exporter.rs` | Create | `PdfExporter` + `Exporter` trait impl |
| `crates/slideforge-pdf/src/tag_engine.rs` | Create | `SlideTagEngine` struct |
| `crates/slideforge-pdf/src/font.rs` | Create | Font file loading helpers (subsetting is krilla-internal by default) |
| `crates/slideforge-pdf/src/svg_embed.rs` | Create | SVG → krilla Surface path-drawing operations |
| `crates/slideforge-pdf/src/error.rs` | Create | `PdfExportError` with thiserror |
| `crates/slideforge-pdf/tests/no_forbidden_deps.rs` | Create | Cargo.lock assertion test |
| `scripts/check-pdf-deps.sh` | Create | CI guard against browser PDF deps |
| `Cargo.toml` (workspace root) | Modify | Add `slideforge-pdf` to members array |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,500 |
| BC-4.03.002 | ~1,200 |
| LaidOutDeck/LaidOutSlide types (STORY-026 output) | ~2,000 |
| Exporter trait (STORY-002) | ~800 |
| krilla 0.6.0 API docs (Document/Surface/tagging/configure) | ~2,000 |
| usvg 0.47.0 API docs (Tree/Node walk) | ~800 |
| tech-validation.md (pre-loaded cheat-sheet) | ~500 |
| Test files to write | ~2,500 |
| **Total** | **~13,300** |

Context budget: ~14% of a 100k-token context window. Within the 20-30% limit.

## Test Strategy

- **Unit tests**: `PdfExporter::export()` on 1-slide minimal deck → non-empty bytes
  that parse as valid PDF (check `%PDF-` header). `SlideTagEngine` produces correct
  `TagKind::Part` groups via krilla tagging. Font subsetting verified by output size
  (krilla internal path). SVG rect → krilla Surface draw calls, no `/Image` in output.
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
| EC-005 | Empty font subset (no glyphs used) | krilla's internal subsetting produces minimal or empty font program; no panic. If direct subsetter path is activated, `subsetter::subset()` must return a structured error, not panic. |
| EC-006 | Frame-level Diagram/Chart alt (STORY-043 known limitation) | `NormalizedDiagramSvg` has no `alt` field; `SlideTagEngine` emits placeholder alt `"diagram"` / `"chart"`. Non-UA-1 validator does not reject these. STORY-045 must thread real alt from `DiagramSpec.alt` / `ChartSpec.alt` before enabling UA-1 mode (see Architecture Compliance Rule 6). |

## Forbidden Dependencies

The `slideforge-pdf` crate MUST NOT depend on:
- `slideforge-pptx`, `slideforge-docx`, `slideforge-html`, `slideforge-preview`
  (no cross-exporter dependencies — each exporter is independent)
- `chromium`, `headless-chrome`, `puppeteer-rs`, `wkhtmltopdf`
- `libharu`, `cairo-rs`, `pango-sys`, `freetype-sys` (no C library FFI)
- `image` crate for rasterization (SVG must be embedded as paths, not rasterized)

If `slideforge-pdf` gains any of these dependencies, the CI `check-pdf-deps.sh`
script MUST fail the build.
