---
document_type: adr
id: ADR-023
title: PDF styled font-face resolution via fontdb metadata-aware lookup
status: accepted
date: 2026-06-09
approved: 2026-06-09
producer: architect
deciders: [architect, human]
story: STORY-081
traces_to: ARCH-INDEX.md
supersedes: null
superseded_by: null
---

# ADR-023 — PDF Styled Font-Face Resolution via fontdb Metadata-Aware Lookup

## Status

Accepted — human sign-off 2026-06-09. STORY-081 Phase 5 implementation proceeds against this ADR.

---

## Context

STORY-081 AC-004 requires the PDF exporter to render Bold, Italic, Code, Superscript,
and Subscript inline nodes using distinct `krilla::text::Font` instances — one per style.
The krilla 0.6.0 API has no `set_bold()` or `set_italic()` toggle; styled rendering
requires loading a physically distinct font face binary per style.

The current `BrandFonts` struct in `slideforge-types` carries three `Arc<str>`
family-name fields (`heading`, `body`, `mono`) and no font byte data. The existing
`resolve_brand_font()` function resolves a single `Option<krilla::text::Font>` with no
style parameter, and `extract_inline_text()` discards inline variant information before
reaching the draw site. As a result, Bold/Italic/Code inline nodes in PDF output are
rendered with the same font face as plain text — an incorrect implementation that fails
AC-004.

Three resolution strategies were evaluated:

**Option A — filename-heuristic styled resolution:** Extend the existing name-based
`system_font_fallback` with bold/italic stem guessing (`{family}-Bold`, `{family}Bold`,
etc.). Rejected: no naming convention is universal across macOS (`.ttc` collections),
Linux (various conventions), and Windows (`arialbd.ttf`). Fragile resolution produces
silent incorrect rendering when it guesses wrong.

**Option B — explicit styled paths in BrandFonts:** Add `bold_path`, `italic_path`
optional fields to `BrandFonts` and the brand `.toml` schema. Rejected for STORY-081
scope: cross-crate blast radius touches `slideforge-types`, `slideforge-brand`, all
exporter snapshot tests. Correct long-term design but does not help CI / the default
brand where no explicit paths are configured.

**Option C (selected) — fontdb metadata-aware lookup with optional brand override:**
Use the `fontdb` crate to resolve styled faces by reading font metadata (OS/2 table
`usWeightClass`/`fsSelection` bits, name table subfamily string) rather than filenames.
Scope is confined to `slideforge-pdf`. No `BrandFonts` schema change is required for
STORY-081.

---

## Decision

**Adopt `fontdb = "=0.23.0"` as a direct dependency in `slideforge-pdf/Cargo.toml`
for metadata-aware styled font face resolution in the PDF export path.**

### New construct: `ResolvedFontSet`

A new `ResolvedFontSet` struct is added to `slideforge-pdf::font`:

```rust
pub struct ResolvedFontSet {
    pub regular: Option<krilla::text::Font>,
    pub bold:    Option<krilla::text::Font>,
    pub italic:  Option<krilla::text::Font>,
    pub mono:    Option<krilla::text::Font>,
}
```

### Resolution priority per face

For `bold` (example; `italic` follows the same pattern):

1. If `brand.fonts.bold_path` is `Some` and resolves to a valid font file, use it.
   (This field does not exist in `BrandFonts` for STORY-081; step 1 is a no-op today
   and will become active when a Brand-enrichment story adds the optional path fields.)
2. Query a `fontdb::Database` populated via `fontdb::Database::load_system_fonts()` with
   `family = brand.fonts.body`, `weight = fontdb::Weight::BOLD`,
   `style = fontdb::Style::Normal`. fontdb reads OS/2/name table metadata — not
   filenames. If a matching face is found, load its bytes and construct
   `krilla::text::Font::new(data, 0)`.
3. If fontdb finds no match, fall back to the `regular` face and emit:
   `tracing::warn!(family, style, "styled font face not found on this system; falling back to regular face — PDF styled text will render without correct weight/style")`.
4. If `regular` is also absent, the span is skipped (existing graceful-degradation
   behavior, already tested).

For `mono`: resolve `brand.fonts.mono` with `fontdb::Weight::NORMAL`,
`fontdb::Style::Normal`. `Code` inline nodes always use `mono.or(regular)`.

### Dispatch in draw path

`draw_frame` and its text-drawing sub-functions receive `&ResolvedFontSet` instead of
`Option<&krilla::text::Font>`. When iterating `InlineNode` variants:

| InlineNode variant | Font used |
|-------------------|-----------|
| `Plain` | `regular` |
| `Bold(_)` | `bold.or(regular)` |
| `Italic(_)` | `italic.or(regular)` |
| `Code(s)` | `mono.or(regular)` |
| `Superscript(_)` | `regular` with per-glyph `y_offset = +(units_per_em / 3)` via `draw_glyphs` |
| `Subscript(_)` | `regular` with per-glyph `y_offset = -(units_per_em / 3)` via `draw_glyphs` |
| All others | `regular` |

Superscript/Subscript use `Surface::draw_glyphs` with `KrillaGlyph { y_offset, .. }`
per AC-004 (there is no text-rise setter in krilla 0.6.0).

### `extract_inline_text` retirement

The `extract_inline_text` helper (which discards variant information) is replaced by a
span-aware render function in the draw path. `extract_inline_text` may be retained as
an internal utility for cases that genuinely need only plain text (e.g., the outline
title extraction), but must not be called from the main inline-markup draw path.

---

## Supply-Chain Implications

`fontdb =0.23.0` is already present in `Cargo.lock` as a transitive dependency of
`usvg =0.47.0` (workspace dependency). Adding it as a DIRECT pinned dep to
`slideforge-pdf/Cargo.toml` does NOT expand the compiled dependency set — it only
makes the existing transitive dep explicit and pinned at the same version.

The workspace `cargo deny` configuration does not require a new exception. The exact-pin
supply-chain policy (Quality Bar NFR-025) is satisfied by `fontdb = "=0.23.0"`.

`fontdb =0.23.0` contains `unsafe` in its `memmap2` transitive dependency for memory-
mapped font file loading. This transitive unsafe is already accepted in the project
via the ADR-014 decision for `mermaid-rs-renderer` (which pulls the same `fontdb` and
`memmap2` chain). No new unsafe surface is introduced.

---

## Consequences

### Positive

- Reliable styled face resolution on all platforms without filename guessing.
- `BrandFonts` in `slideforge-types` is not touched — zero blast radius to PPTX/DOCX/
  HTML exporters, layout engine, or brand synthesis paths.
- Graceful degradation is non-silent: `tracing::warn!` on fallback, never a silent
  wrong-face or wrong bytes.
- Deterministic CI tests via fixture-font strategy (no system font dependency in test).
- Positions the architecture for the follow-on Brand-enrichment story that adds
  `bold_path`/`italic_path` optional fields to `BrandFonts` — the `resolve_font_set`
  resolution priority order already has step 1 reserved for the brand override path.

### Negative / Trade-offs

- `fontdb::Database::load_system_fonts()` has a cold-start cost (~50–300ms on Linux,
  ~10–50ms on macOS) documented in the `slideforge-diagrams` module. This cost is
  already paid per export pass in `slideforge-diagrams`. For the PDF exporter, the
  database is constructed once per export call and not cached across calls. A future
  optimization could share the `fontdb::Database` instance via a `OnceLock` (as
  `slideforge-diagrams` does), but that optimization is out of scope for STORY-081.
- The `fontdb`-resolved face may differ from what a system's application font resolver
  would pick (different priority ordering). This is acceptable: slideforge's font
  resolution model has always been best-effort for brand family names.

### Deferred

- `BrandFonts` optional `bold_path`/`italic_path`/`mono_path` fields — Brand-enrichment
  story, post-STORY-081. The `resolve_font_set` function has resolution step 1 reserved
  for this.
- Caching `fontdb::Database` across export calls via `OnceLock` — performance
  optimization story, not required for v1.0 correctness.

---

## Rejected Alternatives

| Option | Reason for rejection |
|--------|---------------------|
| A — filename heuristics | No universal naming convention; silent wrong-face on mismatch |
| B — BrandFonts explicit paths only | Cross-crate blast radius; default brand still needs fallback |

---

## Decision Log

| Date | Author | Note |
|------|--------|------|
| 2026-06-09 | architect | Initial ADR draft produced in response to Pass-3 adversary finding C2-NEW. |
| 2026-06-09 | human | Approved. Status promoted DRAFT → Accepted. STORY-081 Phase 5 implementation proceeds. |
