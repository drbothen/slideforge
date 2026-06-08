---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-046
version: "1.3"
title: "Static HTML Exporter: WCAG AA via axe-core"
epic: EPIC-14
wave: 5
points: 8
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.03.003]
verification_properties: []
nfr_refs: [NFR-013, NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-html
target_module: slideforge-html
subsystems: [SS-09]
depends_on:
  - STORY-026
  - STORY-034
  - STORY-049
blocks:
  - STORY-047
  - STORY-048
estimated_days: 4
---

# STORY-046: Static HTML Exporter — WCAG AA via axe-core

## Subsystem Anchor Justification

SS-09 (HTML/Preview) owns this story because it implements the `slideforge-html`
crate, which is one of the two crates in SS-09 per ARCH-INDEX ("slideforge-html,
slideforge-preview"). The static HTML exporter shares the rendering logic with the
web preview server (STORY-047), making SS-09 the correct anchor.

## Dependency Anchor Justifications

- Depends on STORY-026: `LaidOutDeck` is the input to `HtmlExporter`. Cannot produce
  HTML without a laid-out deck.
- Depends on STORY-034: `NormalizedDiagramSvg` is embedded in the SVG graphics layer as
  a `<g role="img" aria-labelledby="..."><title>...</title>{svg content}</g>` group —
  role/title injected via quick-xml after usvg geometry pass. Requires normalized SVG
  (no foreignObject, absolute dims) per P4 Composite Rendering Model (ADR-008 2026-06-08).
- Depends on STORY-049: The plugin registry must register `HtmlExporter` via the
  `Exporter` trait. This story is the `Exporter` implementation that gets registered.
- Blocks STORY-047: The web preview server uses the same HTML rendering functions as
  the static exporter. STORY-047 imports `render_slide_to_html()` from this crate.
- Blocks STORY-048: Live reload builds on the static rendering infrastructure.
- Does NOT block STORY-050 directly: STORY-046 is Wave 5; STORY-050 is Wave 4. The E2E
  tests (STORY-050) are structured to test HTML output via the plugin registry path, not
  directly from the html-exporter story. STORY-050 is blocked by STORY-049, not STORY-046.

## Summary

Implement the `slideforge-html` crate: a static HTML exporter producing one HTML file
per slide (or a single multi-page HTML file) that passes WCAG AA validation via
`@axe-core/playwright` in CI.

Design decisions (from ADR-008 binding 2026-06-08, BC-4.03.003 v1.4):
1. **P4 Composite Rendering Model** — Slide text (title, body, bullets) is emitted as
   real HTML elements (`<h1>`, `<h2>`, `<p>`, `<ul><li>`) CSS-absolutely-positioned
   from `LaidOutFrame` EMU coordinates. Graphical elements use a sibling
   `<svg role="presentation">` graphics layer. The `<article style="position:relative">`
   is the slide container. No `<canvas>`. No `<foreignObject>` (usvg 0.47.0 drops it
   silently — its presence is a rendering regression).
2. All non-decorative images have non-empty `alt` attributes.
3. All decorative images have `alt=""` and `role="presentation"`.
4. Graphical frames (charts, diagrams) in the SVG layer use a wrapping `<g role="img"
   aria-labelledby="sf-{slide_id}-{frame_index}">` with `<title>` child — injected via
   `quick-xml` after usvg geometry pass (usvg strips non-presentation attrs).
5. `<html lang="...">` derived from deck `lang` field (never hardcoded).
6. Heading level derived from an exporter-level pre-pass (4-step chain) — title-slide
   with a Title frame → `<h1>` (step 1); first slide with any Title frame → `<h1>` (step
   2); first slide with a promotable text frame → promoted `<h1>` + warn (step 3);
   chart/image-only deck → synthesized visually-hidden `<h1>` from `deck.metadata.title`
   (or constant `"Presentation"`) + warn (step 4). Empty deck (0 slides): no `<h1>`.
   Exactly one `<h1>` per non-empty HTML document.
7. ARIA landmarks: `<header>`, `<main>`, `<nav>` for slide navigation.

The `HtmlExporter` implements the `Exporter` trait from `slideforge-plugin-api`.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.03.003 | Static HTML Output Passes WCAG AA via axe-core on CI | AC-001 through AC-009 |
| (security) | Link/Xref URL scheme allowlist — CWE-601 (OBS-1 from STORY-085) | AC-010 |

## Acceptance Criteria

### AC-001: HtmlExporter implements the Exporter plugin trait
(traces to BC-4.03.003 postcondition 1)

`HtmlExporter` in `slideforge-html` implements the `Exporter` trait. The export
method writes a valid HTML document to `output: &mut dyn Write`. The output is
well-formed HTML5 with `<!DOCTYPE html>`.

### AC-002: html[lang] attribute derived from deck lang field
(traces to BC-4.03.003 postcondition 3 and invariant 3)

The `<html>` element has `lang="<value>"` where `<value>` is `deck.lang` (e.g.,
`en-US`, `de`, `ja`). This attribute is NEVER hardcoded as `lang="en"`. Unit test:
two fixture decks (lang "en-US" and lang "ja") produce HTML with matching `lang`
attributes.

### AC-003: Non-decorative graphical frames in the SVG layer have role="img" and <title> injected via quick-xml
(traces to BC-4.03.003 postcondition 4)

For every graphical `LaidOutFrame` (chart, diagram, or image) with `decorative: false` in the SVG graphics layer, the wrapping `<g>` element has `role="img"` and `aria-labelledby="sf-{slide_id}-{frame_index}"`, and a `<title id="sf-{slide_id}-{frame_index}">` child containing the frame's alt text. Text frames (title, body, bullets) are NOT in the SVG layer — they are real HTML elements and carry no `role="img"`. Verified by parsing the HTML output with `scraper` and asserting every chart/diagram `<g>` in the SVG layer has a non-empty `<title>` child.

**CRITICAL — usvg 0.47.0 limitation (export-architecture v1.2):** usvg strips all
non-presentation attributes during SVG tree processing (no `role`, no `aria-*`). usvg
MUST NOT be used to inject `role="img"` or `<title>` — it will silently discard them.
The correct approach is raw SVG string/XML manipulation via `quick-xml` or `roxmltree`
after usvg geometry processing is complete:
1. Use usvg for geometry normalization and validation only.
2. After usvg processing, use `quick-xml` XML manipulation to wrap the normalized SVG
   in a `<g role="img" aria-labelledby="sf-{slide_id}-{frame_index}">` and prepend a
   `<title id="sf-{slide_id}-{frame_index}">` child with the alt text.
This ensures accessibility attributes survive through to the rendered HTML.

### AC-004: Decorative images have alt="" and role="presentation"
(traces to BC-4.03.003 postcondition 5)

For every `LaidOutElement` with `decorative: true`, the HTML element has
`alt=""` and `role="presentation"`. Verified by unit test.

### AC-005: Chart/diagram SVG elements have role="img" and <title>
(traces to BC-4.03.003 postcondition 6)

For each `FrameContent::ChartSvg` or `FrameContent::DiagramSvg` element, the wrapping `<g>` in the slide's SVG graphics layer has `role="img"` and `aria-labelledby="sf-{slide_id}-{frame_index}"`, and a `<title id="sf-{slide_id}-{frame_index}">` child containing the element's alt text. `{frame_index}` is the 0-based index of the graphical frame within the slide (graphical frames only; text frames excluded). The chart/diagram SVG content nested inside the `<g>` is `aria-hidden="true"` at its root. The outer `<svg>` of the graphics layer carries `role="presentation"` (NOT `aria-hidden="true"` — WAI-ARIA forbids descendant re-exposure through an aria-hidden ancestor; using aria-hidden on the outer svg would hide the <g role="img"> children from AT and produce a false-green axe-core result). Verified by HTML structure tests.

**Implementation note (usvg 0.47.0 — export-architecture v1.2):** usvg strips
non-presentation attributes including `role` and `aria-*`. Inject `role="img"` and
`<title>` via `quick-xml` raw XML manipulation after usvg processing, not via the
usvg tree API. usvg is used for geometry/validation only; the accessibility attributes
are written at the serialization step.

### AC-006: P4 DOM structure — no <canvas>, no <foreignObject>, article container
(traces to BC-4.03.003 invariant 2)

The HTML output uses `<svg>` elements for slide content rendering. No `<canvas>`
elements appear in the HTML output. This is verified by an assertion in the unit
test using the correct `scraper` API (scraper 0.27.0):
```rust
let doc = scraper::Html::parse_document(&html);
let sel = scraper::Selector::parse("canvas").expect("valid selector");
assert_eq!(doc.select(&sel).count(), 0);
```
Note: `Html::parse_document(s).select("canvas")` is invalid — `select()` requires a
`&Selector` reference, not a string literal. Always construct a `Selector` first.

Additionally, assert no `<foreignObject>` elements appear in any `<svg>` output (usvg drops them silently — their presence indicates a rendering regression): `let fobj_sel = scraper::Selector::parse("foreignObject").expect("valid selector"); assert_eq!(doc.select(&fobj_sel).count(), 0);` Assert the presence of at least one `<article>` element (the slide container): `let art_sel = scraper::Selector::parse("article").expect("valid selector"); assert!(doc.select(&art_sel).count() > 0);`

### AC-007: Zero WCAG AA violations via axe-core in CI
(traces to BC-4.03.003 postcondition 2 and invariant 4)

The CI job `html-wcag-check` runs `@axe-core/playwright` against the HTML output
from a fixture deck and asserts `violations.length === 0` for WCAG AA rules. The
job is defined in `.github/workflows/html-wcag.yml` and triggers on every PR
touching `crates/slideforge-html/**`.

### AC-008: Heading level determined by 4-step chain — exactly one <h1> per non-empty document
(traces to BC-4.03.003 postcondition 7)

`render_deck_to_html()` executes an exporter-level pre-pass over the full `LaidOutDeck`
before any per-slide rendering begins. It pre-computes a `HeadingLevel` value for each
slide and passes it to `render_slide_to_html()` as a parameter — the per-slide function
does NOT determine heading level itself. Exactly one `<h1>` exists per non-empty HTML
document. No heading level is skipped. The heading-assignment chain is:

**Step 1.** Find the first slide whose `slide_type == title` AND that slide has a Title
frame → that Title frame's text is emitted as `<h1>`. No warn emitted.

**Step 2.** (No `slide_type == title` slide with a Title frame exists.) Find the first
slide that contains a Title frame → that Title frame's text is emitted as `<h1>`. No
warn emitted.

**Step 3.** (No slide has any Title frame.) Find the first slide that has a **promotable
text frame** → its first text block is promoted to `<h1>`, and `tracing::warn!` is
emitted. "Promotable text" is defined as: a Body block of type `Text`, `Bullets`, or
`Math` with non-empty content, or a `TextRun`. The following are **NOT** promotable:
`Table`, `ColorBar`, `Shape`, or any Body block with empty/whitespace-only content. The
promotion targets the **first slide that has a promotable text frame** — not assumed to
be slide 0; purely graphical leading slides are skipped.

**Step 4.** (No Title frame AND no promotable text frame in ANY slide — e.g. a
chart-only or image-only deck.) The exporter synthesizes a single **visually-hidden**
`<h1>` at the document level. The text content is `deck.metadata.title`; if that field
is empty or whitespace-only, it falls back to the non-empty constant `"Presentation"`.
The synthetic `<h1>` MUST be non-empty. `tracing::warn!` is emitted.

**Empty-deck exception.** A deck with zero slides produces no `<h1>` and no warn.

**Warn-emission semantics.** The text-promotion `tracing::warn!` is emitted only when
step 3 actually fires. The synthetic-h1 `tracing::warn!` is emitted only when step 4
actually fires. Steps 1 and 2 never emit a warn.

**Remainder of heading hierarchy.** Slides that have a Title frame but are not the
`<h1>` slide emit `<h2>` for that Title frame. Sub-headings within slide content are
`<h3>`, `<h4>`, etc.

**Testable acceptance checks:**
- A 3-slide fixture deck where slide 1 is `slide_type == title` with a Title frame →
  exactly one `<h1>` on slide 1; slides 2 and 3 emit `<h2>` for their Title frames.
- A deck whose first slide is chart-only (no Title frame, no promotable text) and second
  slide is a body slide (has a `Text` Body block) → step 3 fires, single `<h1>` is on
  the second slide, `tracing::warn!` emitted, no `<h1>` on slide 1.
- A deck containing only chart slides (no Title frame, no promotable text frame in any
  slide), with `deck.metadata.title = "Q3 Charts"` → step 4 fires; the document-level
  synthetic `<h1 class="sf-visually-hidden">Q3 Charts</h1>` is present; `tracing::warn!`
  emitted; axe-core `page-has-heading-one` passes.
- Same chart-only deck with `deck.metadata.title = ""` → step 4 fires with fallback
  constant; output contains `<h1 class="sf-visually-hidden">Presentation</h1>`.
- Empty deck (0 slides) → no `<h1>` in output, no warn emitted.

This is validated by the axe-core `heading-order` and `page-has-heading-one` rules in
AC-007.

### AC-009: All body text meets contrast ratio targets
(traces to BC-4.03.003 postcondition 8)

Low-contrast brand color pairs are caught at compile time by BC-5.01.003 (before
reaching HTML export). The HTML exporter emits the brand colors as inline CSS
`color` and `background-color` properties. The axe-core `color-contrast` rule in
the CI gate confirms 4.5:1 for normal text, 3:1 for large text. If the brand
passes the compile-time validator, it passes axe-core.

### AC-010: Link/Xref URL scheme allowlist — CWE-601 open-redirect/XSS prevention
(security acceptance criterion — added 2026-06-05, anchored from STORY-085 OBS-1)

Before rendering any `Link` or `Xref` inline node to an HTML `<a href="...">` attribute,
the `HtmlExporter` MUST validate the URL scheme against an explicit allowlist
(`http`, `https`, `mailto`, `tel`). URLs with disallowed schemes (`javascript:`,
`data:`, `vbscript:`, `blob:`, etc.) MUST be silently dropped (href omitted or element
rendered as plain text), and a `tracing::warn!` must be emitted with the rejected scheme.

Background: `DefaultInlineFormat` in `slideforge-plugin-api` intentionally does NOT
perform scheme filtering — it produces format-agnostic output and delegates security
enforcement to the exporter layer. `slideforge-pptx` implements `is_safe_link_scheme`
in `link_safety.rs` for OOXML hyperlinks (the PPTX rId path is already guarded).
`slideforge-html` MUST apply equivalent protection at HTML rendering time. CWE-601
(Open Redirect) / reflected XSS via `javascript:` href are the relevant attack vectors.

Verification: unit test — construct a `LaidOutSlide` (or inline render context) with
a `Link` node whose URL is `javascript:alert(1)`. Assert the rendered HTML output
contains NO `href` attribute (or the element is rendered as `<span>` instead of
`<a>`). Repeat for `data:text/html,...` and `vbscript:foo`. Assert a `tracing::warn!`
is emitted for each rejected URL (use `tracing-test = "=0.2.5"` dev-dependency).

Note: `slideforge-pdf` and `slideforge-preview` will require analogous scheme filtering
when they implement inline node rendering. This AC establishes the pattern for SS-09.

## Tasks

- [ ] Create `crates/slideforge-html/Cargo.toml` with dependencies:
  `minijinja = { workspace = true }` (templating; centralized in [workspace.dependencies] per ADR-022;
  pin =2.20.0; autoescape is keyed on template name — `.html` suffix enables HTML autoescape automatically),
  `usvg = { workspace = true }` (SVG geometry/validation only — NOT for role/aria injection; see AC-003/AC-005),
  `html-escape = { workspace = true }` (use `encode_text` for text content, `encode_double_quoted_attribute`
  for attribute values; unnecessary when output is built via quick-xml which auto-escapes),
  `quick-xml` (for role/title injection into SVG after usvg processing),
  `scraper = "=0.27.0"` (dev-dep test-only), `slideforge-plugin-api`, `slideforge-types`.
  Centralized versions per ADR-022; pin =2.20.0 for minijinja, =0.47.0 for usvg, =0.2.13 for html-escape.
- [ ] Create `crates/slideforge-html/src/lib.rs` — re-export `HtmlExporter`
- [ ] Create `crates/slideforge-html/src/exporter.rs` — `HtmlExporter` + `Exporter` impl
- [ ] Create `crates/slideforge-html/src/render.rs`:
  - `render_slide_to_html(slide: &LaidOutSlide, brand: &Brand, heading_level: HeadingLevel) -> String` — emits `<article style="position:relative">` container; HTML text layer (h1/h2/p/ul via heading_level parameter) absolutely positioned from EMU coords; sibling `<svg role="presentation">` graphics layer for charts/images/shapes.
  - `render_text_frame(frame: &LaidOutFrame, heading_level: HeadingLevel) -> String` — converts a text frame to the appropriate HTML element with inline position/style CSS.
  - `render_graphics_layer(frames: &[LaidOutFrame], slide_id: &str) -> String` — builds the `<svg role="presentation">` layer; calls `render_chart_frame` per non-decorative graphical frame.
  - `render_chart_frame(svg: &str, alt: &str, frame_index: usize, slide_id: &str) -> String` — injects `role="img"`, `aria-labelledby`, and `<title>` via `quick-xml` after usvg geometry pass. No foreignObject.
- [ ] Create `crates/slideforge-html/templates/page.html.jinja` — full HTML page wrapper
- [ ] Write unit tests:
  - `render_slide_to_html()` on 1-slide fixture → valid HTML5 structure
  - lang attribute = deck.lang (not "en")
  - Chart SVG → `<g role="img" aria-labelledby="sf-{slide_id}-{frame_index}"><title id="...">alt text</title>{svg content}</g>` in the graphics layer
  - Decorative image → `alt="" role="presentation"`
  - No `<canvas>` elements in output
  - Heading level (4-step chain): (a) title-slide type with Title frame → `<h1>`, other slides → `<h2>`; exactly one `<h1>` per document. (b) chart-only leading slide followed by body slide → step 3 fires, `<h1>` on body slide, warn emitted. (c) all-chart deck with non-empty `deck.metadata.title` → step 4 fires, synthesized `<h1 class="sf-visually-hidden">` at document level, warn emitted. (d) all-chart deck with empty title → step 4 fallback constant `"Presentation"`. (e) empty deck → no `<h1>`, no warn.
  - No `<foreignObject>` in output SVG.
  - Article container present for every slide.
- [ ] Add `slideforge-html` to root workspace `Cargo.toml`
- [ ] Write CI workflow `.github/workflows/html-wcag.yml`:
  - Job: `ubuntu-latest`
  - `cargo test -p slideforge-html --test html_fixture` (generates HTML output)
  - Install Playwright + @axe-core/playwright
  - Run axe-core against fixture HTML output
  - Assert zero WCAG AA violations

## Previous Story Intelligence

N/A — first story in EPIC-14 (HTML Export and Web Preview). Consumes:
- `LaidOutDeck` and `LaidOutSlide` from STORY-026.
- `NormalizedDiagramSvg` from STORY-034.
- `Exporter` trait from STORY-002.
- `Brand` from STORY-022.

STORY-047 (web preview server) will import `render_slide_to_html()` from this
crate — design the function signature to be reusable in a WebSocket push context.
Specifically: `render_slide_to_html()` should return a `String` (not write to a
`dyn Write`) so STORY-047 can serialize it as JSON for WebSocket push.

## Architecture Compliance Rules

1. **P4 Composite Rendering Model (BC-4.03.003 invariant 2, ADR-008 binding 2026-06-08):** Slide text (title, body, bullets) is emitted as real HTML elements (`<h1>`, `<h2>`, `<p>`, `<ul><li>`) CSS-absolutely-positioned from `LaidOutFrame` EMU coordinates. Graphical elements (charts, images, decorative shapes) are rendered in a sibling `<svg role="presentation">` graphics layer. No `<canvas>` elements. No `<foreignObject>` — usvg 0.47.0 drops it silently, which would lose all text in the shared PDF/preview pipeline. The `<article style="position:relative">` is the slide container.
2. **lang is never hardcoded (BC-4.03.003 invariant 3)**: The HTML template must
   reference `{{ lang }}` from the deck metadata. Hardcoded `lang="en"` is a
   defect.
3. **axe-core runs on CI on every PR (BC-4.03.003 invariant 1)**: The CI job is not
   optional. It must gate merges to `develop`.
4. **Plugin trait boundary (BC-5.02.002)**: `HtmlExporter` must compile using only
   the public API from `slideforge-plugin-api`. No imports from `slideforge-pptx`.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `minijinja` | `{ workspace = true }` = `=2.20.0` (ADR-022) | Jinja2-compatible HTML templating; `.html` suffix enables HTML autoescape automatically |
| `usvg` | `{ workspace = true }` = `=0.47.0` (ADR-022) | SVG geometry/validation ONLY — does NOT round-trip role/aria attributes (stripped); use quick-xml for accessibility injection |
| `html-escape` | `{ workspace = true }` = `=0.2.13` (ADR-022) | `encode_text` for text nodes; `encode_double_quoted_attribute` for attr values; not needed when using quick-xml (auto-escapes) |
| `quick-xml` | workspace | Raw XML manipulation for role/title injection into SVG after usvg pass |
| `scraper` | `=0.27.0` (dev-dep only) | HTML parsing in tests — use `Selector::parse("...")` + `doc.select(&sel)`, not `.select("canvas")` directly |
| `slideforge-plugin-api` | workspace | `Exporter` trait |
| `slideforge-types` | workspace | `LaidOutDeck`, `LaidOutSlide`, `Brand` |
| `@axe-core/playwright` | CI only | WCAG AA validation (not a Rust dep) |
| `playwright` | CI only | Headless browser for axe-core (not a Rust dep) |

All workspace-pinned crates centralized in `[workspace.dependencies]` per ADR-022.
Forbidden: `<canvas>` in output HTML. `pulldown-cmark` (Markdown is not needed;
the DSL produces structured IR, not Markdown).

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-html/Cargo.toml` | Create | Crate manifest |
| `crates/slideforge-html/src/lib.rs` | Create | Re-exports, forbid(unsafe_code) |
| `crates/slideforge-html/src/exporter.rs` | Create | HtmlExporter + Exporter impl |
| `crates/slideforge-html/src/render.rs` | Create | Slide-to-HTML rendering functions |
| `crates/slideforge-html/templates/page.html.jinja` | Create | Full page wrapper template |
| `crates/slideforge-html/tests/html_fixture.rs` | Create | Integration test + fixture |
| `.github/workflows/html-wcag.yml` | Create | CI: axe-core WCAG AA gate |
| `Cargo.toml` (workspace root) | Modify | Add slideforge-html to members |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,800 |
| BC-4.03.003 | ~1,400 |
| LaidOutDeck/LaidOutSlide types | ~2,000 |
| Exporter trait | ~800 |
| minijinja 2.20.0 API reference | ~1,000 |
| HTML templates to write | ~1,500 |
| Test files to write | ~2,000 |
| **Total** | **~11,500** |

Context budget: ~12% of a 100k-token context window. Within limit.

## Test Strategy

- **Unit tests**: `render_slide_to_html()` produces HTML5 with correct lang, P4 DOM structure (`<article>` container, text as real HTML elements, `<svg role="presentation">` graphics layer), `<g role="img">` wrappers with `<title>`, exactly one `<h1>` per non-empty document via the 4-step heading-assignment chain (step 1: title-slide + Title frame; step 2: first Title frame; step 3: first promotable text frame with warn; step 4: synthetic visually-hidden `<h1>` from `deck.metadata.title` or `"Presentation"` fallback with warn; empty-deck exception), no `<canvas>`, no `<foreignObject>`. Decorative images have `alt=""` and `role="presentation"`.
- **Integration test**: Full 3-slide fixture deck → HTML file → Playwright + axe-core
  → zero WCAG AA violations.
- **Snapshot tests**: `insta` snapshots for rendered HTML of reference slide types
  (title slide, content slide, chart slide). Snapshots catch regressions in the
  template output structure.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with only decorative images | All `alt="" role="presentation"`; axe-core passes |
| EC-002 | Chart with complex nested SVG | `<g role="img" aria-labelledby="...">` wrapper in graphics layer; chart SVG root `aria-hidden="true"` |
| EC-003 | Deck with lang "de" | `<html lang="de">`; axe-core: no lang violation |
| EC-004 | Low-contrast color pair | Blocked at compile time by validator (BC-5.01.003); never reaches HTML |
| EC-005 | HTML output on Windows | CRLF line endings; axe-core still passes |
| EC-006 | Deck with NO Title frame and NO promotable text frame anywhere (all-chart / all-image deck) | Step 4 fires: `<h1 class="sf-visually-hidden">` synthesized from `deck.metadata.title` (or `"Presentation"` if empty); `tracing::warn!` emitted; axe-core `page-has-heading-one` passes; exactly one non-empty `<h1>` present |

## Forbidden Dependencies

`slideforge-html` MUST NOT depend on:
- `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf` (no cross-exporter deps)
- `image` crate for rasterization (SVG is always embedded as `<svg>`, never `<img>`)
- Any crate that injects `<canvas>` elements into the output

If `slideforge-html` gains a dependency on another exporter crate, the CI build
MUST fail (enforced by `cargo deny`).

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.3 | 2026-06-08 | story-writer | BC-4.03.003 v1.4 Pass-4 synthetic-h1 extension: AC-008 fully rewritten to match the 4-step heading-assignment chain — step 3 narrowed to a precisely defined "promotable text frame" (Body of type Text/Bullets/Math with non-empty content, or TextRun; Table/ColorBar/Shape/empty-Body excluded; promotion targets the first slide with a promotable text frame, not assumed slide 0); step 4 added for chart-only/image-only decks (synthesize visually-hidden `<h1>` from `deck.metadata.title` or constant `"Presentation"`, emit `tracing::warn!`); empty-deck exception added (0 slides: no `<h1>`, no warn); warn-emission semantics clarified (step-3 and step-4 warns fire only when those steps actually execute); 5 testable acceptance checks added (steps 1–4 + empty deck). Sibling reconciliations: Summary design decision 6 updated to 4-step chain synopsis; Summary BC reference bumped to v1.4; Tasks unit-test checklist updated with sub-cases (a)–(e) covering all 4 steps + empty deck; Test Strategy updated to reference 4-step chain; EC-006 added to Edge Cases table (chart-only / image-only deck). |
| 1.2 | 2026-06-08 | story-writer | ADR-008 Pass-3 correction + BC-4.03.003 v1.3: AC-005 rewritten — outer `<svg>` corrected from `aria-hidden="true"` to `role="presentation"` (WAI-ARIA forbids descendant re-exposure through aria-hidden ancestor; false-green axe-core risk); canonical frame ID placeholder renamed `{frame_id}` → `{frame_index}` (0-based graphical-frame index) in AC-003 body, quick-xml step, Tasks checklist, and AC-005; AC-008 expanded with exporter-level single-h1 enforcement rule (`render_deck_to_html()` pre-pass, `HeadingLevel` parameter threading, fallback promotion with `tracing::warn!`); Architecture Compliance Rule 1 updated to `<svg role="presentation">` graphics layer; Summary design decision 1 updated to match; `render_slide_to_html` and `render_graphics_layer` Tasks bullets updated to P4 signatures and `role="presentation"`; Test Strategy updated; `slide.html.jinja` row deleted from File Structure table and Tasks checklist (ADR-rejected stale artifact); BC reference updated to v1.3. |
| 1.1 | 2026-06-08 | story-writer | Applied P4 Composite Rendering Model (ADR-008 binding 2026-06-08, BC-4.03.003 v1.2): Architecture Rule 1 rewritten to P4 model; AC-003 title+body updated to `<g role="img">` wrapper pattern with `aria-labelledby`; AC-005 opening rewritten to `<g>` wrapper + outer `<svg aria-hidden="true">` semantics; AC-006 title changed, `<foreignObject>` + `<article>` assertions added; AC-008 replaced with `slide_type`-driven heading-level rule + single `<h1>` invariant; Tasks `render.rs` bullets updated to P4 function signatures; 3 new unit test checklist items added (heading-level, no-foreignObject, article-present); Summary design decisions updated to P4; STORY-034 dep anchor note updated; EC-002 updated. |
| 1.0 | 2026-05-25 | story-writer | Initial story decomposition. |
