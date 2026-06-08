---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-017
lifecycle_status: active
introduced: v1.0.0
modified:
  - version: "1.2"
    date: 2026-06-08
    note: "P4 Composite Rendering Model clarification (ADR-008, 2026-06-08): Invariant 2 rewritten to specify HTML+CSS real elements + sibling SVG graphics layer; Postcondition 6 rewritten with per-frame ARIA labelledby pattern + usvg attribute injection note; Postcondition 7 rewritten with explicit h1/h2 rules per slide type. Description prose aligned to P4 model (removed stale SVG role=img top-level embedding description; replaced with HTML+CSS text layer + sibling SVG layer summary). EC-002 and Canonical Test Vector row for chart alt updated to match <g role=\"img\" aria-labelledby> pattern inside aria-hidden SVG."
  - version: "1.3"
    date: 2026-06-08
    note: "ADR-008 Pass-3 aria-hidden correction + id canonicalization: outer graphics <svg> corrected from aria-hidden=\"true\" to role=\"presentation\" — aria-hidden on an ancestor hides the whole subtree, silently preventing <g role=\"img\"> children from reaching AT; role=\"presentation\" keeps the subtree in the accessibility tree. frame_id → frame_index (0-based index of graphical frames only; text frames excluded) throughout Postcondition 6, EC-002, Description, Invariant 2, and Canonical Test Vectors. Postcondition 7 rewritten with exporter-level h1 assignment algorithm (slide_type==title priority, then first-Title-frame fallback, then first-body-frame with warn!) and heading-level pre-computation via render_deck_to_html() parameter threading."
  - version: "1.4"
    date: 2026-06-08
    note: "Pass-4 synthetic-h1 fallback: Postcondition 7 extended with a precise 4-step heading-assignment chain — step 3 narrows 'first body frame' to a strictly defined promotable text frame (Body block of type Text/Bullets/Math with non-empty content, or a TextRun; Tables/ColorBars/Shapes/empty-Body excluded); step 4 adds the previously missing fallback for decks with NO Title frame AND NO promotable text frame anywhere (e.g. chart-only/image-only) — exporter synthesizes a single visually-hidden <h1> from deck.metadata.title (falling back to the non-empty constant 'Presentation') and emits tracing::warn!. Empty-deck exception added (0 slides: no h1 required, no warn). Warn-emission semantics clarified: text-promotion warn fires only when a body frame is truly promoted; synthetic-h1 warn fires only on step 4. Added EC-006 and canonical test vector for the step-4 case."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-4.03.003: Static HTML Output Passes WCAG AA via axe-core on CI

## Description

The static HTML exporter produces accessible HTML pages for each slide, passed through
`@axe-core/playwright` (or equivalent) on every CI run. The output must achieve zero
WCAG AA violations. Using the P4 Composite Rendering Model (ADR-008): slide text (title,
body, bullets) is emitted as real HTML elements CSS-absolutely-positioned; graphical
frames (charts, diagrams, images, shapes) are rendered in a sibling
`<svg role="presentation">` layer — no `<canvas>`, no `<foreignObject>`. Within that
layer, non-decorative frames are wrapped in `<g role="img" aria-labelledby>` groups with
`<title>` children carrying the alt text; role/aria attributes are injected via quick-xml
after usvg processing. All visual elements have alt text (from DSL), the HTML document
has a `lang` attribute from the deck declaration, headings follow correct hierarchical
order, and color contrasts meet 4.5:1 for normal text and 3:1 for large text.

## Preconditions

1. A valid `LaidOutDeck` IR exists with all accessibility fields populated (alt text,
   lang, color-coded labels).
2. Static HTML export is invoked (e.g., `slideforge build --format html`).
3. `@axe-core/playwright` is available in the CI environment.
4. Playwright is installed for headless browser accessibility scanning.

## Postconditions

1. A valid HTML file (or directory of per-slide HTML files) is produced.
2. `@axe-core/playwright` runs against the output and reports zero WCAG AA violations.
3. `<html lang="...">` attribute is set to the deck's declared language.
4. Every non-decorative image has a non-empty `alt` attribute.
5. Every decorative image has `alt=""` and `role="presentation"`.
6. Non-decorative graphical frames (charts, diagrams, images) in the SVG graphics layer are wrapped in `<g role="img" aria-labelledby="sf-{slide_id}-{frame_index}">` with a `<title id="sf-{slide_id}-{frame_index}">` child containing the alt text. `{frame_index}` is the 0-based index of the graphical frame within the slide (graphical frames only; text frames excluded). The outer `<svg>` of the graphics layer carries `role="presentation"` — NOT `aria-hidden="true"` — so its subtree remains in the accessibility tree and the `<g role="img">` elements are genuinely exposed to AT. Decorative frames carry `aria-hidden="true"` individually on their own `<g>`. role/title/aria-labelledby attributes are injected via quick-xml raw XML manipulation after usvg processing (usvg 0.47.0 strips non-presentation attributes).
7. Exactly one `<h1>` exists per HTML document (for every non-empty deck). This is an exporter-level invariant. The heading-assignment chain executed by `render_deck_to_html()` is:

   **Step 1.** Find the first slide whose `slide_type == title` AND that slide has a Title frame → that Title frame's text is emitted as `<h1>`. No warn.

   **Step 2.** (No slide has `slide_type == title` with a Title frame.) Find the first slide that contains a Title frame → that Title frame's text is emitted as `<h1>`. No warn.

   **Step 3.** (No slide has any Title frame.) Find the first slide that has a **promotable text frame** → its first text block is promoted to `<h1>`, and `tracing::warn!` is emitted. "Promotable text" is defined as: a Body block of type `Text`, `Bullets`, or `Math` with non-empty content, or a `TextRun`. The following are **NOT** promotable: `Table`, `ColorBar`, `Shape`, or any Body block with empty/whitespace-only content.

   **Step 4.** (No Title frame AND no promotable text frame in ANY slide — e.g. a chart-only or image-only deck.) The exporter synthesizes a single **visually-hidden** `<h1>` at the document level. The text content is `deck.metadata.title`; if that field is empty or whitespace-only, it falls back to the non-empty constant `"Presentation"`. The synthetic `<h1>` must be non-empty. `tracing::warn!` is emitted.

   **Empty-deck exception.** A deck with zero slides produces no `<h1>` and no promotion warn.

   **Warn-emission semantics.** The text-promotion `tracing::warn!` is emitted only when step 3 actually fires. The synthetic-h1 `tracing::warn!` is emitted only when step 4 actually fires. Steps 1 and 2 never emit a warn.

   **Remainder of heading hierarchy.** Slides that have a Title frame but are not the `<h1>` slide emit `<h2>` for that Title frame. Slides with body content but no Title frame emit no heading element for the slide itself. Sub-headings within a slide are `<h3>`, `<h4>`, etc. No heading level is skipped. Heading level is pre-computed by `render_deck_to_html()` before any per-slide call and passed to `render_slide_to_html()` as a parameter — the per-slide function does not determine heading level itself.
8. All body text meets 4.5:1 contrast ratio against background; large text meets 3:1.

## Invariants

1. axe-core is run in CI on every PR touching export code or templates — not just on
   release.
2. The HTML output uses the P4 Composite Rendering Model (ADR-008, 2026-06-08): slide text (title, body, bullets) is emitted as real HTML elements CSS-absolutely-positioned; graphical frames (charts, images, decorative shapes) are rendered in a sibling `<svg role="presentation">` layer. No `<canvas>` elements. No `<foreignObject>`. This model satisfies "SVG-based canvas with ARIA" (CLAUDE.md) while preserving full heading structure and AT-accessible text.
3. The lang attribute is never hardcoded — it always derives from the deck `lang` field.
4. Zero WCAG AA violations is the hard gate; warnings (WCAG AAA suggestions) are
   informational only.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with only decorative images | All images have `alt="" role="presentation"`; axe-core: zero violations |
| EC-002 | Chart with complex nested SVG structure | outer `<svg role="presentation">`; non-decorative frame wrapped in `<g role="img" aria-labelledby="sf-{slide_id}-{frame_index}">` with `<title id="sf-{slide_id}-{frame_index}">` child; individual decorative `<g>` elements carry `aria-hidden="true"` |
| EC-003 | Deck with non-English lang "de" | `<html lang="de">`; axe-core detects no lang mismatch |
| EC-004 | Low-contrast brand color pair | E-A11-004 emitted at compile time (BC-3.03.001 area); blocked before HTML export |
| EC-005 | HTML output on Windows (CRLF line endings) | axe-core still passes; line endings are cosmetic |
| EC-006 | Deck with NO Title frame and NO promotable text frame anywhere (e.g., all slides are chart-only or image-only) | Step 4 fires: exporter synthesizes a visually-hidden `<h1>` from `deck.metadata.title` (or `"Presentation"` fallback); `tracing::warn!` emitted; axe-core `page-has-heading-one` passes; exactly one `<h1>` present |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3-slide deck with all alts, lang "en-US", passing contrast | axe-core exit 0; zero WCAG AA violations | happy-path (TV-12.1) |
| Chart slide with `alt "Revenue by quarter"` | `<g role="img" aria-labelledby="sf-…">` + `<title id="sf-…">Revenue by quarter</title>` inside `<svg role="presentation">` | happy-path |
| Deck `lang "ja"` | `<html lang="ja">`; axe-core: no lang violation | edge-case |
| Deck with all-decorative slide | All images `alt="" role="presentation"`; axe-core passes | edge-case |
| 2-slide deck with two chart-only slides (no Title frames, no text frames), `deck.metadata.title = "Q3 Charts"` | Step 4 fires; `<h1 class="sf-visually-hidden">Q3 Charts</h1>` synthesized at document level; `tracing::warn!` emitted; axe-core `page-has-heading-one` passes; zero WCAG AA violations | edge-case (EC-006 / step 4 fallback) |
| 2-slide deck with two chart-only slides, `deck.metadata.title = ""` (empty) | Step 4 fires; fallback constant `"Presentation"` used; `<h1 class="sf-visually-hidden">Presentation</h1>` synthesized; `tracing::warn!` emitted; axe-core passes | edge-case (EC-006 / step 4 empty-title fallback) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | axe-core WCAG AA: zero violations on all fixture decks | CI: Playwright + axe-core on HTML output; assert violations.length === 0 |
| VP-TBD | All non-decorative images have non-empty alt attributes | integration test: parse HTML, count img[alt=""] vs img[alt!=""] |
| VP-TBD | html[lang] attribute matches deck lang declaration | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 |
| Capability Anchor Justification | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 — "static HTML (WCAG AA via axe-core)" is verbatim from CAP-017 |
| L2 Domain Invariants | DI-003 (lang declaration propagates to HTML lang attr), DI-014 (referenced for PDF; HTML has analogous WCAG AA requirement) |
| Architecture Module | slideforge-html crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.03.004 — composes with (web preview is a live version of the same HTML renderer)
- BC-5.01.001 — depends on (alt text compile enforcement ensures no missing alt reaches HTML export)
- BC-5.01.005 — depends on (lang propagation to HTML lang attr is enforced by this BC)
- BC-4.03.001 — related to (PDF has analogous accessibility requirement via veraPDF)

## Architecture Anchors

- `architecture/export-architecture.md` — static HTML export design
- `architecture/plugin-architecture.md` — WCAG AA enforcement pipeline

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
