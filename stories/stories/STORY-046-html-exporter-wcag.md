---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-046
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
- Depends on STORY-034: `NormalizedDiagramSvg` is embedded as `<svg>` with `role="img"`
  in the HTML output. Requires normalized SVG (no foreignObject, absolute dims).
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

Design decisions (from ADR-008 and BC-4.03.003):
1. SVG-based canvas rendering — not `<canvas>` elements. Each slide is rendered as an
   `<svg>` element within an `<article>` semantic landmark.
2. All non-decorative images have non-empty `alt` attributes.
3. All decorative images have `alt=""` and `role="presentation"`.
4. Charts and diagrams embedded as `<svg>` with `role="img"` and `<title>` element.
5. `<html lang="...">` derived from deck `lang` field (never hardcoded).
6. Heading hierarchy (h1 → h2 → ...) is correct and non-skipped.
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

### AC-003: Non-decorative images have non-empty alt attributes
(traces to BC-4.03.003 postcondition 4)

For every `LaidOutElement` with `decorative: false`, the generated `<img>` or `<svg>`
element in the HTML output has a non-empty `alt` attribute (for `<img>`) or a
`<title>` child element (for `<svg>`). Verified by parsing the HTML output with
`scraper` and asserting no `img[alt=""]` where `decorative=false`.

### AC-004: Decorative images have alt="" and role="presentation"
(traces to BC-4.03.003 postcondition 5)

For every `LaidOutElement` with `decorative: true`, the HTML element has
`alt=""` and `role="presentation"`. Verified by unit test.

### AC-005: Chart/diagram SVG elements have role="img" and <title>
(traces to BC-4.03.003 postcondition 6)

For each `FrameContent::ChartSvg` or `FrameContent::DiagramSvg` element, the outer
`<svg>` element has `role="img"` and contains a `<title>` child whose text content
is the element's `alt` text. Inner SVG elements are `aria-hidden="true"`. Verified
by HTML structure tests.

### AC-006: SVG-based canvas (not <canvas> element)
(traces to BC-4.03.003 invariant 2)

The HTML output uses `<svg>` elements for slide content rendering. No `<canvas>`
elements appear in the HTML output. This is verified by an assertion in the unit
test: `scraper::Html::parse_document(&html).select("canvas").count() == 0`.

### AC-007: Zero WCAG AA violations via axe-core in CI
(traces to BC-4.03.003 postcondition 2 and invariant 4)

The CI job `html-wcag-check` runs `@axe-core/playwright` against the HTML output
from a fixture deck and asserts `violations.length === 0` for WCAG AA rules. The
job is defined in `.github/workflows/html-wcag.yml` and triggers on every PR
touching `crates/slideforge-html/**`.

### AC-008: Heading hierarchy is correct and non-skipped
(traces to BC-4.03.003 postcondition 7)

Slide titles map to `<h1>` elements. Sub-headings within slide content map to
`<h2>`, `<h3>`, etc. No heading level is skipped (h1 → h3 without h2). This is
validated by the axe-core `heading-order` rule in AC-007.

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
  `minijinja = "=2.3.0"` (templating), `usvg = "=0.47.0"` (SVG handling),
  `html-escape = "=0.2.13"` (text escaping), `scraper = "=0.21.0"` (test-only),
  `slideforge-plugin-api`, `slideforge-types`
- [ ] Create `crates/slideforge-html/src/lib.rs` — re-export `HtmlExporter`
- [ ] Create `crates/slideforge-html/src/exporter.rs` — `HtmlExporter` + `Exporter` impl
- [ ] Create `crates/slideforge-html/src/render.rs`:
  - `render_slide_to_html(slide: &LaidOutSlide, brand: &Brand) -> String`
  - `render_element_to_html(element: &LaidOutElement) -> String`
  - `render_svg_chart(svg: &str, alt: &str) -> String` (wraps in role=img + title)
- [ ] Create `crates/slideforge-html/templates/slide.html.jinja` — Jinja2 template for a slide
- [ ] Create `crates/slideforge-html/templates/page.html.jinja` — full HTML page wrapper
- [ ] Write unit tests:
  - `render_slide_to_html()` on 1-slide fixture → valid HTML5 structure
  - lang attribute = deck.lang (not "en")
  - Chart SVG → `<svg role="img"><title>alt text</title>`
  - Decorative image → `alt="" role="presentation"`
  - No `<canvas>` elements in output
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

1. **SVG canvas, not `<canvas>` (BC-4.03.003 invariant 2)**: Every slide visual is
   an `<svg>` element. This is a binding ADR-008 decision. No `<canvas>` elements.
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
| `minijinja` | `=2.3.0` | Jinja2-compatible HTML templating |
| `usvg` | `=0.47.0` | SVG parsing for role/title injection |
| `html-escape` | `=0.2.13` | Escape text content for HTML embedding |
| `scraper` | `=0.21.0` | HTML parsing in tests (dev-dependency only) |
| `slideforge-plugin-api` | workspace | `Exporter` trait |
| `slideforge-types` | workspace | `LaidOutDeck`, `LaidOutSlide`, `Brand` |
| `@axe-core/playwright` | CI only | WCAG AA validation (not a Rust dep) |
| `playwright` | CI only | Headless browser for axe-core (not a Rust dep) |

Forbidden: `<canvas>` in output HTML. `pulldown-cmark` (Markdown is not needed;
the DSL produces structured IR, not Markdown).

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-html/Cargo.toml` | Create | Crate manifest |
| `crates/slideforge-html/src/lib.rs` | Create | Re-exports, forbid(unsafe_code) |
| `crates/slideforge-html/src/exporter.rs` | Create | HtmlExporter + Exporter impl |
| `crates/slideforge-html/src/render.rs` | Create | Slide-to-HTML rendering functions |
| `crates/slideforge-html/templates/slide.html.jinja` | Create | Per-slide template |
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
| minijinja 2.3.0 API reference | ~1,000 |
| HTML templates to write | ~1,500 |
| Test files to write | ~2,000 |
| **Total** | **~11,500** |

Context budget: ~12% of a 100k-token context window. Within limit.

## Test Strategy

- **Unit tests**: `render_slide_to_html()` produces HTML5 with correct lang, SVG
  elements, role/alt attributes. No `<canvas>`. Decorative images have `alt=""`.
- **Integration test**: Full 3-slide fixture deck → HTML file → Playwright + axe-core
  → zero WCAG AA violations.
- **Snapshot tests**: `insta` snapshots for rendered HTML of reference slide types
  (title slide, content slide, chart slide). Snapshots catch regressions in the
  template output structure.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with only decorative images | All `alt="" role="presentation"`; axe-core passes |
| EC-002 | Chart with complex nested SVG | `role="img"` on outer; inner elements `aria-hidden="true"` |
| EC-003 | Deck with lang "de" | `<html lang="de">`; axe-core: no lang violation |
| EC-004 | Low-contrast color pair | Blocked at compile time by validator (BC-5.01.003); never reaches HTML |
| EC-005 | HTML output on Windows | CRLF line endings; axe-core still passes |

## Forbidden Dependencies

`slideforge-html` MUST NOT depend on:
- `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf` (no cross-exporter deps)
- `image` crate for rasterization (SVG is always embedded as `<svg>`, never `<img>`)
- Any crate that injects `<canvas>` elements into the output

If `slideforge-html` gains a dependency on another exporter crate, the CI build
MUST fail (enforced by `cargo deny`).
