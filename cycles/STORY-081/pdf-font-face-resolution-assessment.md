---
document_type: architecture-assessment
story: STORY-081
topic: PDF styled font-face resolution (AC-004 gap)
author: architect
date: 2026-06-09
status: final
verdict: IN-SCOPE for STORY-081 — Option C recommended; ADR-023 draft produced
---

# PDF Font-Face Resolution Assessment — STORY-081 AC-004

## 1. Problem Statement

AC-004 of STORY-081 requires the PDF exporter to render Bold, Italic, Code (monospace),
and Superscript/Subscript inline nodes using distinct krilla `Font` instances — one per
style. The krilla API has no `set_bold()` or `set_italic()` toggle; styled rendering
requires loading a physically distinct font face binary.

The current `BrandFonts` struct carries three `Arc<str>` family-name fields (`heading`,
`body`, `mono`) and no font byte data. The existing `resolve_brand_font()` / `try_resolve_font()`
stack in `exporter.rs` resolves a single `Option<krilla::text::Font>` (the body/heading
face) with no style parameter. `extract_inline_text()` discards `.face` and `.y_offset_units`
before reaching the draw site, making the `FontFaceKind` enum in the story spec a
dead-wired stub.

---

## 2. Option Analysis

### Option A — Style-aware system-font resolution inside slideforge-pdf

**Mechanism:** Extend `try_resolve_font(family, style)` to accept a style hint
(`Regular`, `Bold`, `Italic`, `Mono`). For Bold/Italic, apply a naming heuristic on top
of the existing filename-stem search: look for stems like `{family}-Bold`, `{family}Bold`,
`{family}-Italic`, `{family}Italic`, etc., in addition to exact matches. The `mono`
family name from `BrandFonts` is already the full monospace family; resolve it with
`Regular` style.

**Assessment — cross-platform reliability:**

The current `system_font_fallback` implementation already acknowledges its fragility in
a code comment: `"Arial-Bold.ttf" → "arialbold" → no match` (font.rs:475). Under Option A
this problem is compounded rather than fixed. Consider these known cases:

- macOS system bold face: `/System/Library/Fonts/Helvetica.ttc` — `.ttc` collection
  bundling regular+bold+italic in one file. The stem `helvetica` matches for all three;
  there is no separate `helvetica-bold` file.
- Linux (Ubuntu CI runner): font files named `DejaVuSans-Bold.ttf` — stem normalizes to
  `dejavusansbold`; the plain face stem is `dejavusans`. These DO follow the hyphenated
  convention.
- Windows: `arialbd.ttf` (bold) and `arial.ttf` (regular) — the bold stem is `arialbd`,
  not `arial-bold` or `arialbold`. No naming heuristic covers this without a table.

**No naming convention is universal.** Reliable styled font resolution requires reading
font metadata (OS/2 table `usWeightClass`/`fsSelection` bits, name table subfamily
string). Filename heuristics are fundamentally fragile and will silently fall back to the
wrong face on platforms where the convention differs.

**Graceful degradation under Option A:** When the styled face is not found by name,
the exporter would fall back to the plain face. This is a **silent incorrect rendering**
— a "bold" span gets a regular face in the PDF. This violates the "no silent drop"
requirement stated in the adversary finding (C2-NEW requires a build()-driven test that
proves the bold span uses a DISTINCT font resource).

**Verdict: REJECTED.** Fragile cross-platform story; cannot satisfy the distinctness
invariant reliably in CI without a metadata-aware resolver.

---

### Option B — Extend BrandFonts with explicit styled font paths/references

**Mechanism:** Add fields to `BrandFonts` (in `slideforge-types`) and the brand `.toml`
schema (in `slideforge-brand`) for explicit styled faces:
```toml
[fonts]
heading = "Calibri Light"
body = "Calibri"
mono = "Courier New"
bold_path = "/path/to/Calibri-Bold.ttf"
italic_path = "/path/to/Calibri-Italic.ttf"
bold_italic_path = "/path/to/Calibri-BoldItalic.ttf"
```

The PDF exporter reads these paths, loads the bytes, and constructs distinct
`krilla::text::Font` instances.

**Assessment:**

This is a correct design for a fully-specified brand (user supplies all faces). However:

1. **Cross-crate blast radius.** `BrandFonts` is in `slideforge-types` (SS-15, pure
   types). Adding optional path fields touches the type directly, which propagates to all
   consumers of `BrandFonts` (PPTX, DOCX, HTML exporters, layout, config, the brand
   synthesis path). Downstream snapshot tests that construct `BrandFonts` in test helpers
   must all be updated to add the new `None`/empty fields.

2. **Default-brand gap in CI.** The slideforge default brand (`BrandFonts::default()`)
   uses family names with no explicit paths. In CI, `bold_path` would be `None` and the
   styled-face path would fall through to a fallback. The fallback must exist — see
   Option C.

3. **Brand synthesis complexity.** When synthesizing a brand from a `.pptx` template,
   the brand extractor reads only family names from `<a:latin typeface="..."/>` elements.
   There is no mechanism to derive styled font file paths from an OOXML source. The
   synthesis path would always produce `None` for the styled path fields unless the user
   additionally configures them manually — making the feature opt-in even when a fully
   styled typeface is available on the system.

4. **No supply-chain change required** — this is pure Rust struct fields and TOML parsing.

**Verdict: NOT RECOMMENDED AS PRIMARY.** Too much blast radius for STORY-081 scope;
correct as a long-term brand enrichment, but the short-term CI/default-brand gap means
a system-resolution fallback is still needed, which is Option C's value-add.

---

### Option C (Recommended) — fontdb-backed style-aware resolution with brand optional override

**Mechanism:** Introduce a `ResolvedFontSet` struct in `slideforge-pdf` that carries
four `Option<krilla::text::Font>` members: `regular`, `bold`, `italic`, `mono`. Populate
it at export time using the `fontdb` crate's metadata-aware face selection, with the
brand explicit path (if present) as the highest-priority override.

Resolution priority per face (e.g., bold):

1. If `brand.fonts.bold_path` is `Some` and resolves, use it.
2. Otherwise, query `fontdb::Database` with `family = brand.fonts.body`, weight =
   `fontdb::Weight::BOLD`, style = `fontdb::Style::Normal`. `fontdb` reads font metadata
   (OS/2 table, name table), not filenames — reliable on all platforms.
3. If fontdb finds no match, fall back to the plain `regular` face and emit a structured
   `tracing::warn!` (non-fatal, graceful degradation with logging, NOT silent).
4. If the plain face is also absent, skip drawing this span (existing behavior, already
   tested).

For `mono`: always use `brand.fonts.mono` family resolved with `fontdb` at
`fontdb::Weight::NORMAL`, `fontdb::Style::Normal`. The `Code` inline node maps to mono
regardless of weight.

**Cross-platform reliability:**

`fontdb` reads the OS/2 table `usWeightClass` (400 = regular, 700 = bold) and
`fsSelection` bits (bit 0 = italic, bit 5 = bold) and the name table subfamily string.
It does NOT use filenames. This is reliable on macOS (`.ttc` collections are indexed
by face), Linux (font metadata embedded in `.ttf`), and Windows (`arialbd.ttf` has
OS/2 weight 700 — fontdb finds it as Arial Bold correctly).

**Supply-chain implications:**

`fontdb =0.23.0` is already in `Cargo.lock` as a transitive dependency of `usvg =0.47.0`
(which is in the workspace). Adding it as a direct pinned dependency to `slideforge-pdf`
(`fontdb = "=0.23.0"`) does NOT expand the supply chain — the crate is already compiled
and audited. The `cargo deny` configuration does not need to add a new exception.

The supply-chain pin policy (Quality Bar: `= ` exact pins for all production deps) is
satisfied by adding `fontdb = "=0.23.0"` to `slideforge-pdf/Cargo.toml`.

**`slideforge-types` / `BrandFonts` change scope:**

Option C in its minimal form does NOT require `BrandFonts` changes for STORY-081.
The `fontdb`-backed resolution lives entirely in `slideforge-pdf::font`, consuming only
`brand.fonts.body` and `brand.fonts.mono` (existing fields). The `bold_path`/`italic_path`
optional override fields on `BrandFonts` should be added as a follow-on in a Brand-enrichment
story — they are the long-term correct design, but they are not required for STORY-081 to
pass AC-004. This keeps the STORY-081 blast radius minimal.

**Graceful degradation (no silent drop):**

If fontdb cannot find a styled face for the brand family, the behavior is:
- Emit `tracing::warn!(family = .., style = .., "styled font face not found; falling back to regular face")`.
- Use the plain `regular` face for that span.
- The PDF content is readable; only styling is degraded.
- This is logged and observable — not silent.

**The distinctness test (adversary C2-NEW):**

The cross-platform CI concern is: if fontdb returns the SAME font file for both regular
and bold (because the system has no bold face for the brand family), the test asserting
"bold span uses a DISTINCT font resource" would pass on a developer machine (which has
a full font suite) and FAIL in CI (Linux runner with minimal fonts).

Resolution: the test fixture strategy uses the font override seam that already exists in
`PdfExporter::with_font_path`. For STORY-081's unit tests, a `ResolvedFontSet` is
constructed with:
- `regular`: `Font::new(REGULAR_FIXTURE_BYTES, 0)` — using a bundled test fixture font
  that is geometrically distinct (different file bytes, different `font_name()` result
  in the PDF output).
- `bold`: `Font::new(BOLD_FIXTURE_BYTES, 0)` — using a DIFFERENT bundled fixture.

The test asserts that `krilla::text::Font` instances created from distinct byte buffers
produce distinct font objects (by checking that font IDs or the embedded font program
names differ in the uncompressed PDF bytes). This is fully deterministic, filesystem-
independent, and CI-safe. Fixture font files: use the LM-Math `.otf` already in
`crates/slideforge-math/fonts/` as one fixture; a second small `.otf` from the Rust
test ecosystem (e.g., from the `tempfile` test pattern) or embed a minimal synthetic OTF
created with a binary fixture tool. The font fixture approach eliminates all system-font
flakiness from the distinctness assertion.

**Blast radius:**

- `slideforge-pdf`: new `ResolvedFontSet` struct + updated `resolve_brand_font` →
  `resolve_font_set` function + updated `draw_frame` signature to pass the set.
- `slideforge-types` (`BrandFonts`): NO changes required for STORY-081.
- `slideforge-brand`: NO changes required for STORY-081.
- No other crates affected.

This is implementable entirely within STORY-081's stated scope (Phase 5: PDF Exporter).

---

## 3. Recommended Option Summary

**Recommendation: Option C — fontdb-backed `ResolvedFontSet` in slideforge-pdf, no
`BrandFonts` schema change for STORY-081.**

| Dimension | Assessment |
|-----------|-----------|
| Cross-platform reliability | HIGH — fontdb reads OS/2/name table metadata, not filenames |
| New dependency | fontdb =0.23.0 — already in Cargo.lock, zero supply-chain expansion |
| Graceful degradation | Non-silent: `tracing::warn!` on fallback; never a silent wrong-face |
| BrandFonts change | NONE for STORY-081 (follow-on story adds optional path fields) |
| Blast radius | slideforge-pdf only |
| Distinctness test determinism | Fixture-font strategy: CI-safe, filesystem-independent |
| STORY-081 scope | IN-SCOPE — no new architecture decisions beyond fontdb direct dep |

---

## 4. Distinctness Test Design (adversary C2-NEW)

The test must assert that the bold span's font resource differs from the plain span's
resource in the actual PDF content stream.

**Implementation contract for the test-writer:**

1. Bundle two minimal fixture font files in `crates/slideforge-pdf/tests/fixtures/`:
   - `test-regular.otf` — any valid OTF with a known PostScript name, e.g., "TestRegular"
   - `test-bold.otf` — a DIFFERENT valid OTF with PostScript name "TestBold"
   (These can be the same glyph table with different metadata — or use two existing OSS
   fonts already in the repo, e.g., LM Math and a second font from slideforge-math.)

2. Construct a `ResolvedFontSet` with `regular = Font::new(REGULAR_BYTES, 0)` and
   `bold = Font::new(BOLD_BYTES, 0)` using the fixture bytes.

3. Build a `LaidOutDeck` containing a `FrameContent::TextRun` with
   `[InlineNode::Plain("plain"), InlineNode::Bold([InlineNode::Plain("bold")])]`.

4. Call `exporter.export_uncompressed(...)` (which already exists as a public test seam).

5. Scan the uncompressed PDF bytes for TWO DISTINCT font program subsets (e.g., by
   scanning for `/Font` dictionary entries or looking for both "TestRegular" and "TestBold"
   in the PDF name table embedded strings). The assertion is: both PostScript names appear
   in the output, confirming two distinct font resources were embedded.

This test is fully deterministic, runs on all platforms (no filesystem font access), and
directly proves the production path is consuming `face` rather than discarding it.

---

## 5. Story-Spec Correction Required

The current STORY-081 Phase 5 Tasks reference non-existent `fonts.bold_data`,
`fonts.italic_data`, `fonts.mono_data` fields. Under the recommended Option C approach,
these do not exist on `BrandFonts` and will not be added in STORY-081.

**Correction for story-writer/product-owner — replace the Phase 5 Task bullets with:**

> - Add `ResolvedFontSet { regular, bold, italic, mono: Option<krilla::text::Font> }` to
>   `crates/slideforge-pdf/src/font.rs`.
> - Add `resolve_font_set(brand: &Brand, override_path: Option<&Path>) -> ResolvedFontSet`
>   in `font.rs`, using `fontdb =0.23.0` (already in Cargo.lock) for metadata-aware
>   style lookup. Priority: brand override path (if Some) > fontdb metadata query > plain
>   face fallback with `tracing::warn!`.
> - Update `generate_pdf_inner` to call `resolve_font_set` instead of `resolve_brand_font`.
>   Pass `&ResolvedFontSet` to `draw_frame` (replacing `Option<&krilla::text::Font>`).
> - Update `draw_frame` and its text-drawing sub-functions to accept `&ResolvedFontSet`
>   and dispatch on `InlineNode` variant to select the appropriate face:
>   - `InlineNode::Bold(_)` → `font_set.bold.or(font_set.regular)`
>   - `InlineNode::Italic(_)` → `font_set.italic.or(font_set.regular)`
>   - `InlineNode::Code(s)` → `font_set.mono.or(font_set.regular)`, render `s` directly
>   - `InlineNode::Superscript/Subscript(_)` → `font_set.regular`, render via
>     `Surface::draw_glyphs` with `KrillaGlyph { y_offset: ±(units_per_em / 3), .. }`
>   - All other variants → `font_set.regular`
> - Replace `extract_inline_text` call-sites in `draw_frame` with a span-aware render
>   that iterates `InlineNode` variants and dispatches to the appropriate font.
> - Add `fontdb = "=0.23.0"` to `slideforge-pdf/Cargo.toml` `[dependencies]`.
> - Unit test (deterministic fixture): `ResolvedFontSet` from two distinct fixture OTF
>   bytes → `export_uncompressed` → PDF bytes contain two distinct PostScript font names
>   (C2-NEW distinctness assertion).
> - Snapshot test: PDF text spans for Bold + Italic bullet.

**Remove** the references to `fonts.bold_data`, `fonts.italic_data`, `fonts.mono_data` —
these fields will not exist on `BrandFonts` in STORY-081.

---

## 6. ARCH-INDEX ADR-023 Proposed Registration

The decision to adopt `fontdb` as a direct dependency in `slideforge-pdf` for
metadata-aware styled font resolution is a design decision warranting an ADR.

**Proposed ID:** ADR-023
**Proposed title:** PDF styled font-face resolution via fontdb metadata-aware lookup
**ARCH-INDEX row to add:**
```
| ADR-023 | PDF styled font-face resolution via fontdb metadata-aware lookup | Accepted |
```

The ADR draft is at:
`.factory/specs/architecture/adr/ADR-023-pdf-styled-font-face-resolution.md`

---

## 7. Final Verdict

**VERDICT: IN-SCOPE for STORY-081.**

The implementer can proceed with Option C. No human decision is required beyond
ADR-023 sign-off (which is a local architectural decision — no product requirement change,
no new external service, no new license, no scope expansion beyond what AC-004 already
requires).

The only items routed to other agents:
1. **Story-writer**: update Phase 5 Task bullets per Section 5 above (remove
   `fonts.bold_data`/`italic_data`/`mono_data` references; substitute the `ResolvedFontSet`
   approach).
2. **Product-owner**: no BC change required — AC-004 contract is unchanged; only the
   implementation mechanism is clarified.
3. **State-manager**: record ADR-023 in ARCH-INDEX.md after ADR file is committed.
