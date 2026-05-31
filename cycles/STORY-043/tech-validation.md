# STORY-043 (slideforge-pdf) — Technology Validation

**Date:** 2026-05-31
**Scope:** Verify pinned crate versions + API claims against the LIVE crates.io / docs.rs registry BEFORE TDD implementation.
**Verdict:** All six pinned dependencies are REAL and version-compatible. The crate *versions* are safe. However, the spec's krilla **tagged-PDF / PDF-UA API symbol names are PARTLY WRONG** and must be corrected before AC-003 / STORY-045 are coded. See "API DRIFT / RISKS".

---

## Version Validation Table

| Dependency | Spec-claimed version | CONFIRMED / CORRECTED | Source | Notes |
|---|---|---|---|---|
| **krilla** | `0.6.0` | **CONFIRMED `0.6.0` exists** (published 2025-12-02). Latest is `0.8.1` (2026-05-29); `0.7.0` (2026-03-31) and `0.8.x` are newer. | crates.io API `/crates/krilla` | 0.6.0 is fine to pin. krilla churns heavily between minors — pinning `=0.6.0` is the right call. Do NOT chase latest mid-cycle. |
| **pdf-writer** | `0.14.0` (direct pin) | **CONFIRMED `0.14.0` exists** (2025-10-02) AND it is **exactly krilla 0.6.0's transitive requirement (`^0.14.0`)**. No conflict. | crates.io `/crates/pdf-writer`; krilla 0.6.0 `/dependencies` shows `pdf-writer ^0.14.0` | Pin `=0.14.0` is SAFE and unifies with krilla's tree. Latest pdf-writer is `0.15.0` (2026-05-27) — do NOT pin 0.15, it would split from krilla 0.6.0. **But see RISK-1: a direct pdf-writer dep is likely unnecessary.** |
| **subsetter** | `0.2.3` | **CONFIRMED `0.2.3` exists** (2025-09-09) AND it is **krilla 0.6.0's transitive requirement (`^0.2.3`)**. No conflict. | crates.io `/crates/subsetter`; krilla 0.6.0 `/dependencies` shows `subsetter ^0.2.3` | Latest is `0.2.4` (2026-05-28); `^0.2.3` resolves it but `=0.2.3` is fine and matches krilla. **See RISK-2: a direct subsetter dep is likely unnecessary.** |
| **usvg** | (workspace already `=0.47.0`) | **NO CONFLICT — krilla 0.6.0 does NOT depend on usvg at all.** | krilla 0.6.0 `/dependencies` — zero deps matching `*svg*`/`usvg`/`resvg` | SVG handling in krilla is in a SEPARATE companion crate (`krilla-svg`), which slideforge-pdf does NOT use. The workspace `usvg =0.47.0` (slideforge-diagrams, slideforge-math) is untouched by adding krilla. ✅ |
| **thiserror** | workspace `=2.0.18` | **CONFIRMED reuse is correct.** Use `thiserror = { workspace = true }`. | CLAUDE.md workspace policy | Do NOT re-pin in slideforge-pdf. |
| **skrifa** (font, transitive — FYI) | n/a | krilla 0.6.0 pulls `skrifa ^0.37.0`, `tiny-skia-path ^0.11.4` | krilla 0.6.0 `/dependencies` | Listed for awareness; do not direct-dep these. |

**The prior "`zip =4.2.0` does not exist" failure mode does NOT recur here** — every pinned version above was verified present on crates.io.

---

## Krilla 0.6.0 API Cheat-Sheet (VERIFIED against docs.rs/krilla/0.6.0)

### Document / Page / Surface / Finish — the spec's core claims are CORRECT

```rust
use krilla::Document;
use krilla::page::PageSettings;

let mut document = Document::new();                       // ✅ exists
// or: Document::new_with(serialize_settings)             // arg type: krilla::SerializeSettings

let settings = PageSettings::from_wh(width_f32, height_f32); // f32 points
let mut page = document.start_page_with(settings);        // ✅ exists; also start_page() for defaults
let mut surface = page.surface();                          // ✅ exists -> Surface<'_>
// ... surface.draw_text(...) / surface.draw_path(...) / surface.set_fill(...) ...
surface.finish();                                          // drop/finish surface
page.finish();                                             // drop/finish page

let pdf_bytes: Vec<u8> = document.finish()?;               // ✅ returns KrillaResult<Vec<u8>>
```

| Spec claim | Reality | Status |
|---|---|---|
| `Document::new()` | `Document::new()` | ✅ CONFIRMED |
| `document.start_page_with(PageSettings)` | `start_page_with(PageSettings) -> Page<'_>` (also `start_page()`) | ✅ CONFIRMED |
| `page.surface()` | `Page::surface() -> Surface<'_>` | ✅ CONFIRMED |
| `document.finish()` | `Document::finish() -> KrillaResult<Vec<u8>>` | ✅ CONFIRMED (note: returns `KrillaResult<Vec<u8>>`, i.e. `Result<Vec<u8>, KrillaError>`, NOT a bare `Vec<u8>`) |
| `PageSettings::from_wh(w, h)` | confirmed constructor | ✅ CONFIRMED |

### Tagged PDF / PDF-UA — VERIFIED symbols (spec names need correction)

Tag tree is built in memory, then attached to the Document. **krilla 0.6.0 OWNS the whole structure tree** — you do NOT (and must not) hand-write `StructTreeRoot` via pdf-writer.

```rust
use krilla::tagging::{TagTree, TagGroup, TagKind, Tag, ContentTag, Node};

// Build the tree (TagKind is the enum of structure types; Tag wraps a TagKind + attributes)
let mut tag_tree = TagTree::new();
let mut doc_group = TagGroup::new(Tag::with(TagKind::Document /* + attrs */));
//   nest paragraphs/figures/tables:
//   doc_group.push(TagGroup::new(Tag::with(TagKind::P)));
//   figures carry Alt text for accessibility (TagKind::Figure + alt)
tag_tree.push(Node::Group(doc_group));  // Node = Group | Leaf

document.set_tag_tree(tag_tree);        // ✅ attach to Document
```

Verified tagging symbols (docs.rs/krilla/0.6.0/krilla/tagging):
- `TagTree` — root; attached via `Document::set_tag_tree(TagTree)`. ✅
- `TagGroup` — hierarchical container; `.push(...)` to nest. ✅
- `TagKind` — **the enum** carrying variants `Document, Part, Sect, P, Figure, Table, …`. ✅
- `Tag` — wrapper exposing attributes for a given `TagKind` (this is where `/Alt` alternate text lives). ✅
- `ContentTag` — content tag associated with the content it wraps (marked-content / BDC-EMC). ✅
- `Node` — enum: group node vs leaf node. ✅
- `Alt` — alternate-text type alias used for accessible descriptions. ✅

The krilla snapshot fixtures confirm correct OOXML/PDF tag output: `StructTreeRoot`, `StructElem` with `/S /Document`, `/P`, `/Span`, `RoleMap`, `/MarkInfo << /Marked true >>`, and PDF/UA-1 XMP (`pdfuaid:part 1`). This is exactly what STORY-045 + veraPDF need.

### PDF/UA-1 export configuration — VERIFIED (spec's mental model is right; exact paths differ)

```rust
use krilla::SerializeSettings;                       // top-level
use krilla::configure::{Configuration, PdfVersion};
use krilla::configure::validate::Validator;          // NOTE the nested ::validate:: module

// Option A: let krilla pick a compatible PDF version for UA-1
let config = Configuration::new_with_validator(Validator::UA1);

// Option B: pin both (returns Option — None if the combo is invalid)
let config = Configuration::new_with(Validator::UA1, PdfVersion::Pdf17)
    .expect("UA1 + PDF 1.7 is a valid combination");

let settings = SerializeSettings { /* ..config.. */ ..Default::default() };
// (SerializeSettings carries the Configuration; consult the 0.6.0 struct fields when wiring)

let mut document = Document::new_with(settings);
// ... build pages + set_tag_tree(...) + set_metadata(Metadata{ language: "en-US", .. }) ...
let bytes = document.finish()?;  // emits PDF/UA-1-conformant bytes when tree + lang + alt are present
```

Verified configuration symbols (docs.rs/krilla/0.6.0):
- `krilla::SerializeSettings` — top-level struct, arg to `Document::new_with`. ✅
- `krilla::configure::Configuration` — constructors: `new()`, `new_with_validator(Validator)`, `new_with_version(PdfVersion)`, `new_with(Validator, PdfVersion) -> Option<Self>`. ✅
- `krilla::configure::validate::Validator` — enum. Variants verbatim: `None, A1_A, A1_B, A2_A, A2_B, A2_U, A3_A, A3_B, A3_U, UA1, A4, A4F, A4E`. **The PDF/UA-1 variant is `Validator::UA1`.** ✅
- `krilla::configure::PdfVersion` — enum (e.g. `Pdf17`, up to 2.0). ✅
- `Document::set_metadata(Metadata)` — used to set document `language` (PDF/UA-1 REQUIRES a document language). ✅

### subsetter 0.2.3 API — VERIFIED (one parameter-name drift)

```rust
use subsetter::{subset, GlyphRemapper};

let mut remapper = GlyphRemapper::new();        // ✅ no args
remapper.remap(glyph_id);                       // ✅ adds glyph to subset, returns new id
let new_id: Option<u16> = remapper.get(old_id); // ✅ Option<u16>

// VERIFIED signature:
pub fn subset(data: &[u8], index: u32, mapper: &GlyphRemapper) -> Result<Vec<u8>, subsetter::Error>;
let out: Vec<u8> = subset(font_data, 0, &remapper)?;   // ✅ matches spec usage
```

| Spec claim | Reality | Status |
|---|---|---|
| `subsetter::subset(font_data, 0, &remapper) -> Result<Vec<u8>>` | `subset(data: &[u8], index: u32, mapper: &GlyphRemapper) -> Result<Vec<u8>, Error>` | ✅ CONFIRMED (3rd param is named `mapper`, not `remapper` — cosmetic only; positional call is identical) |
| `GlyphRemapper::new()` | `GlyphRemapper::new()` | ✅ CONFIRMED |
| `remapper.remap(glyph_id)` | `remap(glyph)` | ✅ CONFIRMED |
| `remapper.get(old_id) -> Option<u16>` | `get(glyph) -> Option<u16>` | ✅ CONFIRMED |
| return type `Result<Vec<u8>>` | `Result<Vec<u8>, subsetter::Error>` | ✅ CONFIRMED |

---

## API DRIFT / RISKS — read before coding

### RISK-1 (HIGH, architectural): Direct `pdf-writer` dep for `StructTreeRoot` is REDUNDANT and CONFLICT-PRONE
The spec wants to "use `pdf-writer` directly for `StructTreeRoot` manipulation." **This is the wrong approach for krilla 0.6.0.** krilla owns the entire structure tree via its `tagging` module (`TagTree`/`TagGroup`/`TagKind`/`Tag`/`Node` → `Document::set_tag_tree`). Hand-writing `StructTreeRoot` through a parallel pdf-writer handle would (a) double-emit / conflict with krilla's own struct tree, (b) couple slideforge-pdf to a pdf-writer internal that krilla controls, and (c) risk a version skew if krilla ever bumps pdf-writer.
**Recommendation:** Build the tag tree exclusively through `krilla::tagging`. Do NOT add a direct `pdf-writer` dependency unless a concrete, krilla-impossible low-level need is proven. If you keep it, pin `=0.14.0` to match krilla 0.6.0's `^0.14.0` (0.15.0 would split the tree).

### RISK-2 (MED, architectural): Direct `subsetter` dep is likely unnecessary
krilla performs font subsetting internally (it depends on `subsetter ^0.2.3` for exactly this). If slideforge-pdf draws glyphs through krilla's `Surface`/`Font` API, subsetting is automatic — you should not need to call `subsetter::subset` yourself. The verified API above is correct IF you have a standalone subsetting need, but confirm the use-case first. If kept, pin `=0.2.3` to match krilla's tree (0.2.4 resolves under `^0.2.3` but `=0.2.3` is cleaner).

### RISK-3 (MED, naming): Spec's tagging symbols are PARTLY WRONG
The spec lists "`Tag::*` enum" (i.e. `Tag` as the variant-bearing enum). In 0.6.0 the **variant-bearing enum is `TagKind`** (`TagKind::Document`, `TagKind::P`, `TagKind::Figure`, `TagKind::Table`, …). `Tag` is a *wrapper* that pairs a `TagKind` with attributes (incl. `/Alt`). Also the tree node enum is `Node` (Group/Leaf) and groups are `TagGroup`. Code written against a bare `Tag::Figure` enum **will not compile**. Use `TagKind` for the kind, `Tag` for kind+attrs, `TagGroup`/`Node` for the tree.

### RISK-4 (MED, naming): PDF/UA module path is `configure::validate::Validator`, not `configure::Validator`
Authoritative docs.rs places the enum at **`krilla::configure::validate::Validator`** (note the nested `::validate::`). `Configuration` is at `krilla::configure::Configuration`; `SerializeSettings` is **top-level `krilla::SerializeSettings`** (NOT `krilla::serialize::SerializeSettings`). An early Perplexity answer guessed `krilla::configuration::Configuration` / `krilla::serialize::SerializeSettings` and claimed the `configure`/`Validator` API is "0.7+ only" — **that is WRONG; it exists in 0.6.0** (verified on docs.rs/krilla/0.6.0). Trust the docs.rs paths in the cheat-sheet, not the Perplexity guess.

### RISK-5 (LOW): `Document::finish()` returns `KrillaResult<Vec<u8>>`, not `Vec<u8>`
Propagate with `?` into slideforge-pdf's `thiserror` error enum (map `KrillaError` → a `PdfError::Serialize` variant). Do not `.unwrap()` (forbidden by CLAUDE.md).

### RISK-6 (LOW): `Configuration::new_with(Validator, PdfVersion)` returns `Option<Self>`
Invalid validator+version combos yield `None`. Handle it (map to a structured error), don't `.unwrap()`.

### RISK-7 (LOW, hygiene): PDF/UA-1 conformance is a CONTRACT, not just a flag
Setting `Validator::UA1` does not by itself make a conformant PDF — krilla validates and `finish()` will error if requirements are unmet. UA-1 requires: document `language` (via `set_metadata`), `/Alt` on every `Figure`/non-text element, a complete tag tree, `/MarkInfo /Marked true`. These map onto slideforge's existing `alt "..."` (compile-error-if-absent) and `lang "en-US"` (required) rules — good alignment, but AC-003's SlideTagEngine must emit a COMPLETE tree (every content run wrapped) or `finish()` will reject.

---

## Confidence

| Item | Confidence | Basis |
|---|---|---|
| All 6 versions exist & are compatible | **HIGH** | crates.io version lists + krilla 0.6.0 dependency manifest (primary registry data) |
| krilla has NO usvg dep (no conflict) | **HIGH** | krilla 0.6.0 `/dependencies` enumerated; zero `*svg*` matches |
| Document/Page/Surface/finish API | **HIGH** | docs.rs/krilla/0.6.0 Document struct page + module index |
| Validator enum variants incl. `UA1` | **HIGH** | docs.rs/krilla/0.6.0 `configure::validate::Validator` enum page (variants read verbatim) |
| Configuration constructors | **HIGH** | docs.rs/krilla/0.6.0 `configure::Configuration` struct page |
| tagging symbol names (`TagKind` vs `Tag`) | **MEDIUM-HIGH** | docs.rs tagging module index + krilla tagging snapshot fixtures. The exact builder ergonomics (how `Tag` wraps `TagKind`, how `Alt` is set per-tag) should be confirmed against the live `tagging` module rustdoc during the RED test, as krilla's tagging API has fine-grained method churn. |
| subsetter `subset`/`GlyphRemapper` API | **HIGH** | docs.rs/subsetter/0.2.3 fn + type pages |
| `SerializeSettings` exact field wiring for UA-1 | **MEDIUM** | Confirmed it exists top-level and carries Configuration; the precise field/builder to plug Configuration in should be read off the live 0.6.0 `SerializeSettings` rustdoc when writing the config code (not load-bearing for version pinning). |

---

## Research Methods

| Tool | Queries | Purpose |
|---|---|---|
| WebFetch (crates.io API) | 4 | Version existence for krilla, pdf-writer, subsetter; krilla 0.6.0 dependency manifest (pdf-writer/subsetter/usvg transitive versions) |
| WebFetch (docs.rs) | 6 | krilla 0.6.0 Document/Surface/PageSettings API; tagging module; `configure::validate::Validator` variants; `configure::Configuration` constructors; subsetter `subset` signature |
| Context7 query-docs (/laurenzv/krilla) | 2 | krilla document/page/surface + tagging structure (snapshot fixtures confirmed StructTreeRoot/StructElem/PDF-UA XMP output) |
| Context7 resolve-library-id | 1 | Resolve krilla library ID |
| Perplexity perplexity_ask | 1 | PDF/UA-1 configuration cross-check (FLAGGED: returned partly-wrong/speculative module paths; OVERRIDDEN by authoritative docs.rs) |
| Training data | 0 areas | Not relied upon for any version or symbol — all verified against live registry/docs |

**Total MCP/web tool calls:** 14
**Training data reliance:** LOW — every version and every load-bearing symbol was confirmed against crates.io or docs.rs/krilla/0.6.0 primary sources. The single Perplexity answer that conflicted with docs.rs was explicitly rejected in favor of the docs.rs evidence.
