---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-081
title: "Slide-Level Inline Markup: eval + layout + all-exporter structural formatting"
epic: EPIC-18
wave: 5
points: 13
priority: P0
tdd_mode: strict
status: draft
target_module: slideforge-eval, slideforge-layout, slideforge-pptx, slideforge-docx, slideforge-pdf, slideforge-html
subsystems: [SS-01, SS-02, SS-03, SS-04, SS-05, SS-06, SS-07, SS-08]
behavioral_contracts: [BC-3.02.002]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
depends_on:
  - STORY-077
  - STORY-041
  - STORY-042
  - STORY-043
  - STORY-044
  - STORY-046
blocks: []
estimated_days: 6
# BC status: BC-3.02.002 postcondition 8 covers the observable-consequence clause (bold renders as bold
# in ALL output formats). Product-owner is amending BC-3.02.002 to PC8 v1.5 wording per DIR-077-002 §9.1
# (parallel burst). This story MUST NOT be marked ready until that amendment lands and BC version >= 1.5.
---

# STORY-081: Slide-Level Inline Markup — eval + layout + all-exporter structural formatting

## Subsystem Anchor Justification

- SS-02 (Evaluator) owns the eval-stage conversion of slide field values: `eval_slide_node`
  must convert `FieldValue::Template(chunks)` containing inline markup `TemplateChunk`
  variants to `FieldValue::Inlines(Vec<InlineNode>)` for fields that semantically carry
  inline content (bullets, body text). Per ARCH-INDEX, SS-02 (`slideforge-eval`) is the
  single evaluation authority.
- SS-03 (Layout Engine) owns the layout-pass update: `LaidOutSlide` field types where
  inline structure matters must be updated to carry `FieldValue::Inlines`, and
  `FrameContent::TextRun` must consume `InlineNode` sequences for bullets/body.
- SS-04 (PPTX Exporter) owns PPTX rendering: OOXML `<a:rPr b="1"/>` for Bold,
  `<a:rPr i="1"/>` for Italic, `<a:t>` for Code spans (monospace font switch), etc.
  PPTX title fields are single-run — only bullet/body carries inline formatting (enforced
  by the layout engine; the PPTX exporter renders what layout provides).
- SS-05 (DOCX Exporter) owns DOCX rendering: `<w:b/>` for Bold, `<w:i/>` for Italic,
  `<w:rStyle w:val="CodeSpan"/>` for Code, `<w:hyperlink>` for Link, etc.
- SS-06 (PDF Exporter) — krilla `=0.6.0` text runs (pinned in slideforge-pdf/Cargo.toml,
  NOT workspace); bold/italic/mono faces resolved via a `ResolvedFontSet` struct populated
  by `resolve_font_set(brand, override_path)` using `fontdb =0.23.0` (already in
  Cargo.lock as transitive dep of `usvg =0.47.0`; added as a direct pinned dep to
  `slideforge-pdf/Cargo.toml`). `fontdb` queries font metadata (OS/2 table
  `usWeightClass`/`fsSelection` bits) — not filenames — for reliable cross-platform
  resolution. Fallback on no match: `tracing::warn!` then use the plain regular face
  (non-silent). `BrandFonts` is NOT modified for this story — `ResolvedFontSet` is
  internal to `slideforge-pdf`. Super/subscript via per-glyph `KrillaGlyph.y_offset` in
  `Surface::draw_glyphs` (normalized by units_per_em; no text-rise setter in 0.6.0).
  (per export-architecture v1.2 + ADR-023)
- SS-07 (HTML Exporter) — `<strong>` for Bold, `<em>` for Italic, `<code>` for Code,
  `<a href="...">` for Link, `<sup>` for Superscript, `<sub>` for Subscript,
  `<del>` for Strikethrough, `<mark>` for Highlight.
- SS-08 (Web Preview) — same HTML rendering as SS-07 (shared presenter).

SS-01 (DSL Parser) is NOT modified in this story. The `TemplateChunk` extension delivered
by STORY-077 already causes `template_value()` to produce `TemplateChunk::Bold` etc. for
slide field values. This story's work is purely eval + layout + exporters.

## Dependency Anchor Justifications

- Depends on STORY-077: the inline-markup parser (`TemplateChunk` extension +
  `chunks_to_inline_nodes`) is the foundation this story builds on. Without STORY-077,
  no `TemplateChunk::Bold` variants exist and no conversion function is available.
  STORY-081 is BLOCKED until STORY-077 merges.
- Depends on STORY-041 (DOCX Core Serialization): the DOCX run-level formatting wired
  here requires the core OOXML serialization infrastructure from STORY-041.
- Depends on STORY-042 (DOCX Auto-Generated Document Sections): section-level inline
  markup (handled by STORY-077) is consumed by STORY-042's section rendering. This story
  extends the same infrastructure to slide-level fields.
- Depends on STORY-043 (PDF Core backend): the krilla text-run APIs wired here require
  the PDF backend from STORY-043.
- Depends on STORY-044 (PDF EMU-to-PDF coordinate mapping): slide content geometry is
  required before any text content can be placed on the PDF canvas.
- Depends on STORY-046 (HTML Exporter): the HTML inline element rendering (`<strong>`,
  `<em>`, etc.) requires the HTML exporter infrastructure from STORY-046.
- Blocks nothing: all downstream consumers (CLI, holdout eval) work correctly with either
  `FieldValue::Str` (pre-STORY-081) or `FieldValue::Inlines` (post-STORY-081). This story
  is a v1.0 release gate, not a blocker for specific Wave 5 stories.

## Summary

STORY-077 delivers the inline-markup parser for **section sub-block** field values only.
After STORY-077 merges, slide-level fields (`title`, `bullets`, `body` in
`ContentBlock` and related slide content types) that contain inline markup
(e.g., `bullets: ["**Key finding**: revenue up 12%"]`) are parsed by `template_value()`
into `TemplateChunk::Bold(...)` correctly at parse time, but the **eval stage does NOT**
call `chunks_to_inline_nodes` for these fields. The existing `eval_field_value_to_value`
path evaluates them to `Value::Str`, losing the inline structure. Exporters then render
the literal asterisks as text.

This is the **temporary inconsistency** documented in DIR-077-002 §4 and in STORY-077
EC-013. It is explicitly scoped for this follow-up story.

**This story is a v1.0 release gate.** The Quality Bar requires bold to render as bold
in ALL output formats ("zero `.unwrap()` outside tests; `clippy::pedantic` clean" implicitly
requires correct structural rendering — shipping `**bold**` as literal asterisks in any
output format violates the production-grade default). It MUST land before v1.0 ships.

### Three-Area Work Plan

1. **Eval stage** (`slideforge-eval`): Extend `eval_slide_node` to call
   `chunks_to_inline_nodes` (already exists from STORY-077) for slide fields that
   semantically carry inline content: `bullets` list items, `body`, `caption`,
   `description`, `subtitle`. Store results as `FieldValue::Inlines`. Plain-text
   fields (`title` in PPTX — single-run constraint) produce `FieldValue::Str` from the
   evaluated plain-text content (stripped of markup delimiters), with a warning if markup
   was present and discarded.

2. **Layout engine** (`slideforge-layout`): Update `FrameContent::TextRun` and bullet
   frame generation to consume `FieldValue::Inlines` rather than flattening to plain
   string. The layout engine is responsible for enforcing the PPTX single-run constraint
   on `title` fields (per DIR-077-002 §4, point 4): if the evaluated `title` carries
   inline markup, emit a layout-stage warning and strip to plain text for PPTX output;
   for DOCX/PDF/HTML output, preserve inline structure.

3. **All exporters** (PPTX, DOCX, PDF, HTML): Update exporter text-run generation to
   consume `Vec<InlineNode>` from `FieldValue::Inlines` and produce format-specific
   inline formatting. This is the structural rendering work:
   - PPTX: `<a:rPr b="1"/>` for `InlineNode::Bold` children; `<a:rPr i="1"/>` for Italic;
     monospace character spacing for Code; `<a:hlinkClick>` for Link; other variants
     produce plain runs with appropriate rendering where OOXML supports it.
   - DOCX: `<w:b/>` for Bold; `<w:i/>` for Italic; `<w:rStyle w:val="CodeSpan"/>` for
     Code; `<w:hyperlink r:id="...">` for Link; `<w:vertAlign w:val="superscript"/>` for
     Superscript; `<w:vertAlign w:val="subscript"/>` for Subscript; `<w:strike/>` for
     Strikethrough; `<w:highlight w:val="yellow"/>` for Highlight.
   - PDF (krilla `=0.6.0`, pinned in slideforge-pdf/Cargo.toml): bold/italic/mono faces
     resolved via `ResolvedFontSet` populated by `resolve_font_set(brand, override_path)`
     using `fontdb =0.23.0` metadata-aware lookup (ADR-023). `draw_frame` receives
     `&ResolvedFontSet` and dispatches: `Bold` → `font_set.bold.or(font_set.regular)`;
     `Italic` → `font_set.italic.or(font_set.regular)`; `Code` → `font_set.mono.or(font_set.regular)`;
     URL annotations for Link; per-glyph `KrillaGlyph.y_offset` (normalized by units_per_em)
     for Super/Subscript via `Surface::draw_glyphs`. (per export-architecture v1.2 + ADR-023)
   - HTML: `<strong>`, `<em>`, `<code>`, `<a href>`, `<sup>`, `<sub>`, `<del>`, `<mark>`.

### PPTX Single-Run Title Constraint (Binding)

Per DIR-077-002 §4, point 4: PPTX title placeholder fields (`<p:ph type="title"/>`)
support a single paragraph with one or more runs, but **mixed inline formatting
within a title** is not standard across PPTX renderers (PowerPoint, Keynote,
Google Slides). The layout engine MUST detect when a `title` field was parsed with
inline markup and:
- Emit a `LayoutWarning::InlineMarkupInTitle { slide_title, stripped_text }` warning
- Produce a plain-text run for the PPTX title placeholder
- For DOCX/PDF/HTML output, preserve the inline structure in the title

The warning is non-fatal in `--warn-only` mode and fatal in strict mode (the default).

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-3.02.002 | Manually Authored Section Blocks appear in DOCX/PDF | AC-001 through AC-005 |

Note: BC-3.02.002 postcondition 8 ("bold text within a section sub-block renders as bold
in DOCX/PDF output — not as literal asterisks") is amended by the product-owner
(DIR-077-002 §9.1) to explicitly cover slide-level fields too: "bold text in slide bullets
and body renders as bold in ALL output formats (PPTX, DOCX, PDF, HTML) — not as literal
asterisks." This story is the implementation of that amended clause for slide-level fields.

## Acceptance Criteria

### AC-001: Eval stage converts slide bullet inline markup to FieldValue::Inlines
(traces to BC-3.02.002 postcondition 8 — slide-level inline-structure preservation in evaluator)

**SCOPE BOUNDARY:** This AC covers slide field values only (bullets, body, caption,
description, subtitle). Section sub-blocks are covered by STORY-077 AC-002.

After evaluation, a slide containing:
```
slide content:
  bullets:
    - "**Key finding**: revenue up 12%"
    - "See _appendix_ for details"
```
produces `FieldValue::Inlines(...)` for each bullet item containing structural
`InlineNode::Bold`, `InlineNode::Plain`, and `InlineNode::Italic` nodes respectively.
NO bullet item is `FieldValue::Str("**Key finding**: revenue up 12%")` (literal asterisks).

A unit test drives the full `eval_slide_node` path (not a hand-built InlineNode vec) and
asserts the field value variant is `FieldValue::Inlines` containing the correct nodes.

### AC-002: PPTX exporter renders InlineNode::Bold as OOXML bold run
(traces to BC-3.02.002 postcondition 8 — observable consequence: bold renders as bold in PPTX)

A slide with a bullet containing `InlineNode::Bold([InlineNode::Plain(Arc::from("bold text"))])`
produces PPTX XML with:
```xml
<a:r>
  <a:rPr b="1" ... />
  <a:t>bold text</a:t>
</a:r>
```
NOT `<a:t>**bold text**</a:t>`. A snapshot test on the rendered PPTX XML asserts this
structure. The test uses a real PPTX ZIP output (not a mock), verifying the OOXML schema
is correct per the OOXML specification (element ordering is schema-significant per CLAUDE.md).

### AC-003: DOCX exporter renders all 8 inline markup forms structurally
(traces to BC-3.02.002 postcondition 8 — observable consequence: inline markup in DOCX runs)

A slide body containing all 8 inline markup forms (Bold, Italic, Code, Link, Superscript,
Subscript, Strikethrough, Highlight) produces DOCX XML with the correct OOXML run
properties for each form using ooxmlsdk `=0.6.1` typed builders for `w:rPr`:
`w:b` (Bold), `w:i` (Italic), `w:rStyle w:val="CodeSpan"` (Code),
`w:vertAlign w:val="superscript"` (Superscript), `w:vertAlign w:val="subscript"` (Subscript),
`w:strike` (Strikethrough), `w:highlight w:val="yellow"` (Highlight).
For `InlineNode::Highlight`: ooxmlsdk `=0.6.1` provides typed builders for `w:highlight`
in WordprocessingML. Use the typed API, not raw XML. Snapshot test on the DOCX XML.
`InlineNode::Plain` nodes produce plain `<w:r>` runs without formatting overrides.

### AC-004: PDF exporter renders Bold/Italic/Code via font switching (traces to BC-3.02.002 postcondition 8 — observable consequence: inline markup in PDF)

A slide body containing `InlineNode::Bold` and `InlineNode::Italic` nodes produces
PDF output where the bold text is rendered with a distinct font face from the plain text.
In krilla `=0.6.0` (pinned in `crates/slideforge-pdf/Cargo.toml`, NOT workspace), there is
NO `set_bold()` / `set_italic()` toggle — distinct font face instances are required. The
mechanism (per ADR-023):

- **Bold**: resolved via `ResolvedFontSet.bold` — populated by `resolve_font_set()` using
  `fontdb =0.23.0` querying the brand body family at `Weight::BOLD`. Falls back to
  `ResolvedFontSet.regular` with `tracing::warn!` if no bold face is found on the system.
- **Italic**: resolved via `ResolvedFontSet.italic` — same fontdb lookup at `Style::Italic`.
  Falls back to `ResolvedFontSet.regular` with `tracing::warn!` if not found.
- **Code (monospace)**: resolved via `ResolvedFontSet.mono` — fontdb lookup on
  `brand.fonts.mono` family at `Weight::NORMAL`. Falls back to `ResolvedFontSet.regular`
  with `tracing::warn!` if not found.
- **Superscript / Subscript**: rendered via `Surface::draw_glyphs` with per-glyph
  `y_offset` on each `KrillaGlyph` (normalized by `units_per_em`). There is NO
  text-rise setter in krilla 0.6.0 — offset is applied per-glyph.

Unit test (deterministic, CI-safe): construct a `ResolvedFontSet` from two distinct bundled
fixture OTF byte buffers (`test-regular.otf`, `test-bold.otf` in
`crates/slideforge-pdf/tests/fixtures/`). Call `export_uncompressed(...)`. Assert that
both fixture PostScript font names appear in the uncompressed PDF bytes — confirming that
two distinct font resources were embedded (C2-NEW distinctness assertion).
A PDF snapshot fixture test asserts structural equivalence. (per export-architecture v1.2 + ADR-023)

### AC-005: HTML exporter renders all 8 inline markup forms as semantic HTML elements
(traces to BC-3.02.002 postcondition 8 — observable consequence: inline markup in HTML/preview)

A slide body containing all 8 inline markup forms produces HTML with:
`<strong>` (Bold), `<em>` (Italic), `<code>` (Code), `<a href="...">` (Link),
`<sup>` (Superscript), `<sub>` (Subscript), `<del>` (Strikethrough), `<mark>` (Highlight).
Snapshot test on the rendered HTML. `axe-core` accessibility scan on the preview output
passes (inline semantic elements are WCAG AA neutral; `<a>` elements must have accessible
names satisfied by their text content).

### AC-006: PPTX title with inline markup triggers layout warning and strips to plain text
(traces to BC-3.02.002 postcondition 8 — PPTX single-run constraint enforced by layout engine)

A slide with `title: "**Bold Title**"` produces:
1. A `LayoutWarning::InlineMarkupInTitle` warning in the diagnostic sink.
2. PPTX `<p:ph type="title"/>` with a plain-text run `<a:t>Bold Title</a:t>` (no `<a:rPr b="1"/>`).
3. DOCX, PDF, HTML output with the title rendered as bold (layout preserves inline structure
   for non-PPTX outputs).
In strict mode (default), this warning is promoted to a fatal error and the build fails.
In `--warn-only` mode, it is a warning and output is produced.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `eval_slide_node` (extended) | `crates/slideforge-eval/src/eval.rs` | Pure |
| `chunks_to_inline_nodes` (reused from STORY-077) | `crates/slideforge-eval/src/register_routing.rs` | Pure |
| `FrameContent::TextRun` (updated) | `crates/slideforge-layout/src/frame.rs` | Pure |
| `slide_to_ooxml_runs` (PPTX inline runs) | `crates/slideforge-pptx/src/slide_xml.rs` | Pure |
| `slide_to_docx_runs` (DOCX inline runs) | `crates/slideforge-docx/src/slide_xml.rs` | Pure |
| `slide_to_krilla_runs` (PDF inline runs) | `crates/slideforge-pdf/src/slide_pdf.rs` | Pure |
| `slide_to_html_spans` (HTML inline elements) | `crates/slideforge-html/src/slide_html.rs` | Pure |
| Tests | per-crate `#[cfg(test)] mod tests` + snapshot fixtures | Pure |

Architecture section files:
- `architecture/module-decomposition.md` (subsystem boundaries)
- `architecture/dependency-graph.md` (no new Cargo edges added)

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~4,500 |
| BC-3.02.002 (v1.5, amended) | ~1,800 |
| DIR-077-002 (inline-markup directive, §3 mapping table) | ~2,000 |
| STORY-077 context (chunks_to_inline_nodes function) | ~2,000 |
| `slideforge-eval/src/eval.rs` (eval_slide_node context) | ~3,000 |
| `slideforge-layout/src/frame.rs` (FrameContent context) | ~2,000 |
| `slideforge-pptx/src/slide_xml.rs` (OOXML run generation) | ~2,500 |
| `slideforge-docx/src/slide_xml.rs` (DOCX run generation) | ~2,500 |
| `slideforge-pdf/src/slide_pdf.rs` (krilla run generation) | ~2,000 |
| `slideforge-html/src/slide_html.rs` (HTML element generation) | ~1,500 |
| Test files to write | ~5,000 |
| Snapshot fixtures | ~2,000 |
| **Total** | **~30,800** |

This is approximately 24% of a 128k-token agent context window. The scope spans 6 crates
but each exporter change is self-contained (pattern: dispatch on `InlineNode` variant,
produce format-specific markup). The implementer SHOULD load exporter files on demand
(one exporter at a time) rather than all at once to stay within budget. A sub-burst
split is acceptable: sub-burst A (eval + layout), sub-burst B (PPTX + DOCX), sub-burst C
(PDF + HTML) — but each sub-burst must pass `just check` before proceeding.

## Tasks

### Phase 1: Eval Stage Extension (slideforge-eval)
- [ ] In `eval_slide_node`, identify the slide field types that carry inline content:
  bullets (each list item string), body, caption, description, subtitle
- [ ] For each such field, if the `TemplateChunk` sequence from `template_value()` contains
  any inline markup variant (Bold, Italic, Code, Link, Superscript, Subscript, Strikethrough,
  Highlight), call `chunks_to_inline_nodes(chunks, env, sink)` (from STORY-077) and store
  `FieldValue::Inlines(nodes)` instead of flattening to `Value::Str`
- [ ] For `title` fields containing inline markup: strip markup and store `FieldValue::Str`
  for the PPTX path; emit `LayoutWarning::InlineMarkupInTitle` into the diagnostic sink;
  for DOCX/PDF/HTML, preserve `FieldValue::Inlines` via a separate evaluated-title field
  on the slide node
- [ ] Write unit tests: each field type with inline markup → `FieldValue::Inlines`; title
  with markup → warning emitted + plain string; plain-text fields unaffected

### Phase 2: Layout Engine Update (slideforge-layout)
- [ ] Update `FrameContent::TextRun` to carry `Vec<InlineNode>` instead of `Arc<str>` for
  bullet and body text content
- [ ] Update bullet frame generation (from STORY-073) to produce `Vec<InlineNode>` from
  `FieldValue::Inlines` bullet items
- [ ] Enforce PPTX single-run title constraint: `FrameContent::TitleRun` carries `Arc<str>`
  (plain text); layout stage enforces this by accepting only `FieldValue::Str` for title
- [ ] Update layout snapshot tests affected by the `FrameContent::TextRun` type change

### Phase 3: PPTX Exporter (slideforge-pptx)
- [ ] Add `inline_node_to_ooxml_runs(node: &InlineNode) -> Vec<DrawingMLRun>` function
  (or equivalent OOXML builder pattern) dispatching on all 12 `InlineNode` variants
- [ ] Wire into slide body/bullet XML generation
- [ ] Snapshot test: PPTX XML for a slide with Bold bullet renders `<a:rPr b="1"/>`
- [ ] Verify element ordering is schema-significant correct (per CLAUDE.md OOXML rules)

### Phase 4: DOCX Exporter (slideforge-docx)
- [ ] Add `inline_node_to_docx_runs(node: &InlineNode) -> Vec<WordprocessingMLRun>` function
  dispatching on all 12 variants per the OOXML mapping in Summary above
- [ ] Wire into slide body / bullet / section paragraph generation
- [ ] Snapshot test: DOCX XML for a slide body with all 8 markup forms

### Phase 5: PDF Exporter (slideforge-pdf)
- [ ] Add `ResolvedFontSet { regular, bold, italic, mono: Option<krilla::text::Font> }` to
  `crates/slideforge-pdf/src/font.rs`.
- [ ] Add `resolve_font_set(brand: &Brand, override_path: Option<&Path>) -> ResolvedFontSet`
  in `font.rs`, using `fontdb =0.23.0` (already in Cargo.lock as transitive dep of
  `usvg =0.47.0`) for metadata-aware style lookup. Resolution priority per face: (1) brand
  override path if `Some`; (2) `fontdb::Database::load_system_fonts()` query on
  `brand.fonts.body` with the target weight/style (OS/2 table metadata, not filename);
  (3) fall back to `regular` face with `tracing::warn!(family, style, "styled font face
  not found on this system; falling back to regular face")` (non-silent, never wrong-face).
- [ ] Update `generate_pdf_inner` to call `resolve_font_set` instead of `resolve_brand_font`.
  Pass `&ResolvedFontSet` to `draw_frame` (replacing `Option<&krilla::text::Font>`).
- [ ] Update `draw_frame` and its text-drawing sub-functions to accept `&ResolvedFontSet`
  and dispatch on `InlineNode` variant to select the appropriate face:
  - `InlineNode::Bold(_)` → `font_set.bold.as_ref().or(font_set.regular.as_ref())`
  - `InlineNode::Italic(_)` → `font_set.italic.as_ref().or(font_set.regular.as_ref())`
  - `InlineNode::Code(s)` → `font_set.mono.as_ref().or(font_set.regular.as_ref())`, render `s` directly
  - `InlineNode::Superscript/Subscript(_)` → `font_set.regular`, render via
    `Surface::draw_glyphs` with `KrillaGlyph { y_offset: ±(units_per_em / 3), .. }` (NO text-rise setter in 0.6.0)
  - `InlineNode::Link` → URL annotation via krilla link annotation API
  - All other variants → `font_set.regular`
- [ ] Replace `extract_inline_text` call-sites in `draw_frame` with a span-aware render
  that iterates `InlineNode` variants and dispatches to the appropriate font.
- [ ] Add `fontdb = "=0.23.0"` to `slideforge-pdf/Cargo.toml` `[dependencies]`
  (makes the existing transitive dep explicit and pinned; no new supply-chain surface).
- [ ] Unit test (deterministic, CI-safe): construct `ResolvedFontSet` from two distinct
  bundled fixture OTF byte buffers in `crates/slideforge-pdf/tests/fixtures/`
  (`test-regular.otf`, `test-bold.otf`). Call `export_uncompressed(...)`. Assert both
  fixture PostScript font names appear in uncompressed PDF bytes (C2-NEW distinctness assertion).
- [ ] Snapshot test: PDF text span/glyph sequence for Bold + Italic bullet.

### Phase 6: HTML Exporter (slideforge-html)
- [ ] Add `inline_node_to_html(node: &InlineNode) -> HtmlNode` dispatching all 12 variants
  to semantic HTML elements (per Summary above)
- [ ] Wire into slide body / bullet rendering
- [ ] Snapshot test: HTML output for all 8 markup forms
- [ ] Run `@axe-core/playwright` audit on preview output — inline elements must not
  introduce WCAG AA violations

### Phase 7: Integration + Gate
- [ ] End-to-end integration test: a deck with inline markup in bullets and body produces
  structurally correct output in all 4 formats (PPTX, DOCX, PDF, HTML) — no literal
  asterisks in any format
- [ ] Run `just check` (fmt + clippy + nextest + doctests + layout) — all clean
- [ ] Run visual regression tests: slide with inline markup renders visually correctly in
  LibreOffice (PPTX + DOCX) + browser screenshot (HTML) — no asterisk artifacts

## Previous Story Intelligence

STORY-077 is the direct predecessor and provides:
- `TemplateChunk::Bold/Italic/Code/Link/Superscript/Subscript/Strikethrough/Highlight` variants
- `chunks_to_inline_nodes(chunks, env, sink) -> Vec<InlineNode>` in `slideforge-eval`
- The two-phase architectural model (parse-time TemplateChunk, eval-time InlineNode)

Key lessons from STORY-077:
1. `chunks_to_inline_nodes` is a pure function — it can be called from any eval context.
   Do NOT duplicate it; call the existing function from register_routing.rs.
2. The crate dependency constraint is already solved: `slideforge-eval` depends on both
   `slideforge-syntax` and `slideforge-types`. No new Cargo edges needed for the eval work.
3. Exporters DO depend on `slideforge-types::InlineNode`. This is expected and correct —
   exporters are the final consumers of the IR. No new Cargo edges needed for exporters.

Key lesson from STORY-041/042 (DOCX): element ordering is schema-significant in OOXML.
The `<w:rPr>` element MUST appear before `<w:t>` in a `<w:r>`. The PPTX equivalent is
`<a:rPr>` before `<a:t>` in `<a:r>`. The implementer must verify element ordering against
the OOXML schema, not just produce logically correct content.

Key lesson from STORY-043 (PDF): krilla `=0.6.0` (pinned in slideforge-pdf/Cargo.toml,
NOT workspace — per export-architecture v1.2) has NO `set_bold()` / `set_italic()` /
text-rise setter. Bold requires a separate `krilla::text::Font` instance for the bold face;
italic requires a separate instance for the italic face. These instances are obtained from a
`ResolvedFontSet` populated by `resolve_font_set()` via `fontdb =0.23.0` metadata-aware
lookup (ADR-023) — NOT from `fonts.bold_data` / `fonts.italic_data` / `fonts.mono_data`
fields, which DO NOT EXIST on `BrandFonts`. Superscript / subscript are rendered via
`Surface::draw_glyphs` with per-glyph `KrillaGlyph.y_offset` (normalized by `units_per_em`).
If fontdb cannot find a styled face, it falls back to the regular face with `tracing::warn!`
(non-silent degradation).

## Architecture Compliance Rules

1. **One new direct Cargo dep, zero new supply-chain surface**: `fontdb = "=0.23.0"` is
   added as a direct pinned dependency to `slideforge-pdf/Cargo.toml`. It is already
   compiled into the build as a transitive dep of `usvg =0.47.0` — adding it as a direct
   dep only makes the version pin explicit. No other new Cargo edges are added.
   `slideforge-eval` → `slideforge-syntax` and `slideforge-types` (existing).
   Each exporter already depends on `slideforge-types::InlineNode`. No cross-exporter
   imports (each exporter is independent).
2. **chunks_to_inline_nodes is NOT duplicated**: The implementation in
   `crates/slideforge-eval/src/register_routing.rs` (from STORY-077) is the ONLY copy.
   Exporters receive `Vec<InlineNode>` from the evaluated IR, not raw `TemplateChunk`.
3. **No eval logic in exporters**: Exporters MUST NOT call `chunks_to_inline_nodes` or
   any eval-stage function. They receive `FieldValue::Inlines(Vec<InlineNode>)` from the
   evaluated `LaidOutSlide` and render it. If an exporter receives `FieldValue::Str`,
   it renders the string as-is (no post-hoc markup parsing in exporters).
4. **PPTX single-run title constraint enforced at layout**: The layout engine, not the
   PPTX exporter, is responsible for stripping markup from titles. The PPTX exporter
   receives only plain-text title content. If the PPTX exporter receives `FieldValue::Inlines`
   for a title, it MUST panic in debug mode (`debug_assert!`) and return an error in
   release mode — this is a contract violation by the layout engine.
5. **No exporter crates in slideforge-eval**: `slideforge-eval` must NOT import
   `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, or `slideforge-html`.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` (workspace) | workspace | `InlineNode`, `FieldValue::Inlines` — consumed by all layers |
| `slideforge-eval` (workspace) | workspace | `chunks_to_inline_nodes` (reused from STORY-077) |
| `slideforge-layout` (workspace) | workspace | `FrameContent::TextRun` type update |
| `ooxmlsdk` | `=0.6.1` | PPTX typed builders for `a:rPr` (b/i/strike/fill) and DOCX typed builders for `w:rPr` (w:b/w:i/w:rStyle/w:vertAlign/w:strike/w:highlight) — typed API, not raw XML |
| `krilla` | `=0.6.0` (pinned in `crates/slideforge-pdf/Cargo.toml`, NOT workspace — per export-architecture v1.2) | PDF: `Font` instances from `ResolvedFontSet`; `Surface::draw_glyphs` + `KrillaGlyph.y_offset` for super/subscript |
| `fontdb` | `=0.23.0` (direct dep in `crates/slideforge-pdf/Cargo.toml`; already in Cargo.lock as transitive dep of `usvg =0.47.0` — no new supply-chain surface) | PDF: `fontdb::Database::load_system_fonts()` + metadata-aware (OS/2 table) style query used by `resolve_font_set()` to populate `ResolvedFontSet` (ADR-023) |

`fontdb =0.23.0` is the only new direct dependency introduced by this story. It is already
compiled as a transitive dep of `usvg =0.47.0`; adding it as a direct dep only pins the
version explicitly per the supply-chain policy. Note: `axum` is NOT used by this story —
HTML inline-markup rendering lives in `slideforge-html` (minijinja/usvg), not axum.
`axum` belongs only to STORY-047.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-eval/src/eval.rs` | Modify | Extend `eval_slide_node` to call `chunks_to_inline_nodes` for inline-carrying fields |
| `crates/slideforge-layout/src/frame.rs` | Modify | Update `FrameContent::TextRun` to carry `Vec<InlineNode>`; enforce PPTX title constraint |
| `crates/slideforge-pptx/src/slide_xml.rs` | Modify | Add `inline_node_to_ooxml_runs`; wire into bullet/body XML generation |
| `crates/slideforge-docx/src/slide_xml.rs` | Modify | Add `inline_node_to_docx_runs`; wire into paragraph XML generation |
| `crates/slideforge-pdf/src/slide_pdf.rs` | Modify | Add `inline_node_to_krilla_spans`; wire into text placement |
| `crates/slideforge-html/src/slide_html.rs` | Modify | Add `inline_node_to_html`; wire into slide body rendering |
| `crates/slideforge-eval/src/tests/slide_inline_markup_eval_tests.rs` | Create | Unit tests for eval-stage conversion (AC-001 + title warning) |
| `crates/slideforge-pptx/tests/inline_markup_pptx_snapshot.rs` | Create | Snapshot test for PPTX OOXML run properties (AC-002) |
| `crates/slideforge-docx/tests/inline_markup_docx_snapshot.rs` | Create | Snapshot test for DOCX run properties (AC-003) |
| `crates/slideforge-pdf/tests/inline_markup_pdf_snapshot.rs` | Create | Snapshot test for PDF text spans (AC-004) |
| `crates/slideforge-html/tests/inline_markup_html_snapshot.rs` | Create | Snapshot test for HTML semantic elements (AC-005) |
| `tests/inline_markup_e2e.rs` (workspace integration test) | Create | End-to-end test: all 4 formats, no literal asterisks |

## Forbidden Dependencies

`slideforge-eval` must NOT depend on:
- `slideforge-pptx` — exporter crate
- `slideforge-docx` — exporter crate
- `slideforge-pdf` — exporter crate
- `slideforge-html` — exporter crate
- `slideforge-preview` — effectful shell

Each exporter must NOT depend on any sibling exporter crate (no cross-exporter imports).
Build fails if any of the above constraints are violated.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `title: "**Bold Title**"` in strict mode | Fatal `LayoutWarning::InlineMarkupInTitle` (promoted to error); build fails. PPTX title would be single-run anyway; user must rewrite title without markup or use `--warn-only`. |
| EC-002 | `title: "**Bold Title**"` in `--warn-only` mode | Warning emitted; PPTX output has plain-text title; DOCX/PDF/HTML output has bold title. |
| EC-003 | Nested inline markup in bullet: `"**_bold italic_**"` | `InlineNode::Bold([InlineNode::Italic([InlineNode::Plain("bold italic")])])`. PPTX: `<a:rPr b="1" i="1"/>`. DOCX: `<w:b/><w:i/>`. All exporters handle nested runs correctly. |
| EC-004 | `InlineNode::Link` in PPTX bullet | `<a:hlinkClick r:id="..."/>` in PPTX draw XML with the URL registered in `slide.xml.rels`. The layout engine registers the URL and produces the rId. |
| EC-005 | `InlineNode::Math` in bullet | Math inline in a bullet renders via the `MathRenderer` plugin. The PPTX exporter embeds the math SVG as an image run (same path as standalone math). DOCX uses OMML. HTML uses MathML. |
| EC-006 | Plain-text bullet with no inline markup | `FieldValue::Str` (or `FieldValue::Inlines([InlineNode::Plain(...)])`) — both are valid. Exporters must handle both forms without error. Prefer `FieldValue::Str` for pure-text bullets to avoid unnecessary allocation. |
| EC-007 | `{{ var }}` resolves to string containing `**bold**` | Resolved string is `InlineNode::Plain(Arc::from("**bold**"))` — NOT further parsed (DIR-077-002 §3 rule: inline markup is parsed from DSL source, not dynamically resolved values). |
| EC-008 | `InlineNode::Highlight` in PPTX | DrawingML does not have a `<a:highlight>` element equivalent to WordprocessingML's `<w:highlight>`. The PPTX exporter renders Highlight via a solid-fill/highlight child on `<a:rPr>` using the ooxmlsdk `=0.6.1` typed fill builder (emit a yellow solid-color highlight fill on the run via typed builder). If the DrawingML schema genuinely lacks a direct highlight element, emit a plain run plus a `tracing::warn!` noting the degradation. Use the typed builder API — not raw XML — in all cases. |
| EC-009 | `InlineNode::Strikethrough` in PPTX | `<a:rPr strike="sngStrike"/>` — single strikethrough. |
| EC-010 | Empty `bullets: []` | No `FieldValue::Inlines` produced; bullet frame content is empty. No error. |

## Test Strategy

- **Red Gate unit tests** (per-crate, must fail before implementation):
  - `crates/slideforge-eval/`: `test_bullet_with_bold_produces_field_value_inlines` —
    drives the eval path; asserts `FieldValue::Inlines` not `Value::Str`
  - `test_title_with_markup_emits_warning_and_plain_str` — drives title path
  - Per-exporter: one unit test per inline node variant for the lowest-level dispatch
    function (`inline_node_to_ooxml_runs`, etc.)

- **Snapshot tests** (per exporter, `insta` crate):
  One snapshot per exporter capturing the full generated markup for a slide body with
  all 8 inline markup forms. Snapshots regenerated on review; committed to fixtures.

- **End-to-end integration test** (`tests/inline_markup_e2e.rs`):
  Builds a deck from DSL source containing `**bold**`, `_italic_`, `` `code` ``,
  `[link](url)`, `^sup^`, `~sub~`, `~~del~~`, `==highlight==` in a bullet. Produces
  all 4 formats. For PPTX: asserts the slide XML contains no `**` or `*` characters
  in `<a:t>` elements. For HTML: asserts `<strong>` appears. For DOCX: asserts `<w:b/>`.

- **Visual regression test** (CI, `just ci-visual`):
  CI renders the `inline_markup_e2e` fixture deck in LibreOffice (headless). Screenshot
  compared to committed fixture. SSIM ≥ 0.98. Asterisk-as-text would produce a visible
  mismatch and fail the gate.

- **axe-core scan** (HTML exporter, CI):
  `@axe-core/playwright` scan on the HTML output of the `inline_markup_e2e` fixture.
  Must report 0 violations. `<a>` elements must have non-empty text content (satisfied by
  link text from the DSL source).

## Complexity Estimate

13 story points. Rationale:
- The eval-stage extension is straightforward (1-2 days): `chunks_to_inline_nodes` is
  already written; wiring it for slide fields follows the same pattern as section sub-blocks.
  The title constraint adds one conditional and a warning.
- The layout engine update is moderate (0.5 days): `FrameContent::TextRun` type change
  cascades to all construction sites; the PPTX title constraint is a well-defined check.
- PPTX exporter is substantial (1 day): 12 InlineNode variants → OOXML properties;
  element ordering must be correct; URL registration for Link nodes.
- DOCX exporter is substantial (1 day): same as PPTX but WXML properties.
- PDF exporter is moderate (0.5 days): font-weight/style dispatch; krilla API is well-typed.
- HTML exporter is light (0.5 days): semantic HTML elements are trivial to produce.
- Tests (snapshots + e2e + Red Gate) are significant (1 day): 6 snapshot files + e2e test.
- The cross-crate coordination and ensuring no new Cargo edges adds review overhead.

Total: 5.5 days estimated, points = 13 (largest allowable; justified by 6-crate scope and
snapshot/visual regression gating).

## References and Intelligence

- **DIR-077-002** (`.factory/cycles/STORY-077/inline-markup-directive.md`) — §4 documents
  the scope boundary (section vs. slide-level) and lists the 4 items this story delivers.
  This story is the mandatory follow-up identified in DIR-077-002 §4 ("A follow-up story is
  MANDATORY before v1.0 to close the inconsistency").
- **STORY-077** — delivers the inline-markup parser (`TemplateChunk` extension +
  `chunks_to_inline_nodes`). This story depends on STORY-077 and reuses its infrastructure.
- **BC-3.02.002 v1.5** (PO amendment per DIR-077-002 §9.1, in-progress) — amended PC8
  covers slide-level bold rendering in all output formats.
- **ADR-013** — comemo Hash compatibility; `InlineNode` already derives `Hash + Eq + Clone`.
- **ADR-023** (`.factory/specs/architecture/adr/ADR-023-pdf-styled-font-face-resolution.md`) —
  PDF styled font-face resolution via `fontdb` metadata-aware lookup. Adopted 2026-06-09.
  Defines `ResolvedFontSet`, `resolve_font_set()`, dispatch table, graceful degradation
  semantics, and the fixture-font deterministic test strategy. Supersedes the
  `fonts.bold_data`/`fonts.italic_data`/`fonts.mono_data` references in v1.1 of this story.
- **CLAUDE.md OOXML rules** — element ordering is schema-significant; `<a:rPr>` before
  `<a:t>` in `<a:r>` (DrawingML); `<w:rPr>` before `<w:t>` in `<w:r>` (WordprocessingML).
- **CLAUDE.md forbidden patterns** — "String-prefix-based bold (`"**header**"`) — Anti-pattern
  from Python reference; use structural `Inline::Bold` (R1 finding)." This story closes
  the R1 finding for slide-level fields.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-06-02 | story-writer | Initial creation per DIR-077-002 §4 and human authorization (2026-06-02). Follow-up to STORY-077. Covers eval + layout + all-exporter inline markup rendering for slide-level fields. Assigned Wave 5, P0, 13 points. Blocks v1.0 release. |
| 1.1 | 2026-06-07 | story-writer | Wave-5 remove-uncertainty propagation: fixed krilla mislabel — krilla=0.6.0 is pinned in slideforge-pdf/Cargo.toml (NOT workspace; per export-architecture v1.2); updated AC-004, Subsystem Anchor SS-06, Summary PDF description, and Phase-5 tasks to reflect correct krilla 0.6.0 API: bold/italic via separate Font::new(data,index) faces (no set_bold/set_italic toggle), super/subscript via KrillaGlyph.y_offset in Surface::draw_glyphs (no text-rise setter); updated AC-003 and Phase-4 DOCX tasks to use ooxmlsdk=0.6.1 typed builders for w:rPr; updated EC-008 PPTX highlight to typed-builder approach; removed axum Library table row (slideforge-html uses minijinja/usvg, not axum; axum belongs to STORY-047 only); cited export-architecture v1.2 throughout. |
| 1.2 | 2026-06-09 | story-writer | Mechanism correction per ADR-023 (approved 2026-06-09): replaced non-existent BrandFonts field references (fonts.bold_data, fonts.italic_data, fonts.mono_data) with the ADR-023 ResolvedFontSet / fontdb =0.23.0 metadata-aware resolution approach throughout — Subsystem Anchor SS-06, Summary PDF bullet, AC-004 mechanism text, Phase-5 Tasks, Previous Story Intelligence (STORY-043 lesson), Architecture Compliance Rule 1, Library table (added fontdb row), References (added ADR-023). Observable AC-004 contract UNCHANGED: bold renders in a distinct bold face, code in monospace, super/sub offset in output PDF. No BC change. |
