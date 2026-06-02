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
- SS-06 (PDF Exporter) — krilla text runs; bold via `FontFamily::SansSerifBold` or
  weight override; italic via `FontStyle::Italic`; code via monospace font switch.
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
   - PDF: bold weight via font selector; italic style; monospace font family switch for
     Code; URL annotations for Link; text raise/lower for Super/Subscript.
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
properties for each form (per Summary section above). Snapshot test on the DOCX XML.
`InlineNode::Plain` nodes produce plain `<w:r>` runs without formatting overrides.

### AC-004: PDF exporter renders Bold/Italic/Code via font switching
(traces to BC-3.02.002 postcondition 8 — observable consequence: inline markup in PDF)

A slide body containing `InlineNode::Bold` and `InlineNode::Italic` nodes produces
PDF output where the bold text is rendered with the bold font weight and italic text
with italic style. Unit test uses the krilla API path and verifies the font selector
dispatches correctly. A PDF snapshot fixture test asserts structural equivalence.

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
- [ ] Add `inline_node_to_krilla_spans(node: &InlineNode, ...) -> Vec<TextSpan>` dispatching
  on all 12 variants; bold via bold font weight, italic via font style, code via monospace
  font family, link via URL annotation, super/subscript via text rise
- [ ] Wire into slide body text placement
- [ ] Snapshot test: PDF text span sequence for Bold + Italic bullet

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

Key lesson from STORY-043 (PDF): krilla's text API uses `TextSpan` structs with font
selector fields. Bold is NOT produced by a `<b>` flag — it requires selecting the bold
font face. The implementer must use the brand's bold font family (or fall back to the
system-default bold face if no brand bold font is configured).

## Architecture Compliance Rules

1. **No new Cargo edges**: This story does not add new workspace crate dependencies.
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
| `ooxmlsdk` | `=0.6.1` | PPTX + DOCX OOXML element builders for run properties |
| `krilla` | workspace | PDF text span font-weight + font-style selectors |
| `axum` | workspace | Web preview HTML generation (unchanged; rendering function extended) |

No new external dependencies required. All listed crates are already in the respective
`Cargo.toml` files.

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
| EC-008 | `InlineNode::Highlight` in PPTX | OOXML does not have a standard highlight run property for DrawingML (only `<w:highlight>` in WordprocessingML). PPTX exporter renders Highlight as a yellow background color on the run (`<a:rPr>` with `<a:highlight a:val="yellow"/>` if schema allows, else plain run with warning). |
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
- **CLAUDE.md OOXML rules** — element ordering is schema-significant; `<a:rPr>` before
  `<a:t>` in `<a:r>` (DrawingML); `<w:rPr>` before `<w:t>` in `<w:r>` (WordprocessingML).
- **CLAUDE.md forbidden patterns** — "String-prefix-based bold (`"**header**"`) — Anti-pattern
  from Python reference; use structural `Inline::Bold` (R1 finding)." This story closes
  the R1 finding for slide-level fields.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-06-02 | story-writer | Initial creation per DIR-077-002 §4 and human authorization (2026-06-02). Follow-up to STORY-077. Covers eval + layout + all-exporter inline markup rendering for slide-level fields. Assigned Wave 5, P0, 13 points. Blocks v1.0 release. |
