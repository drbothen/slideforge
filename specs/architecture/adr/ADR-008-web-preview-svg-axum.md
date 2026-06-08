---
document_type: adr
adr_id: ADR-008
title: Web preview via SVG + axum + WebSocket
status: accepted
date: 2026-05-24
spike_input: S3-wcag-tooling-choice.md
traces_to: ARCH-INDEX.md
supersedes: ~
modified:
  - "2026-06-08: Scope clarification (binding) — SVG-canvas requirement applies to the
     graphical rendering surface only. Slide text is real positioned HTML. Added
     P4 Composite Rendering Model section specifying the article+HTML-text+SVG-graphics
     DOM structure that resolves the foreignObject/role-collapse conflict found during
     STORY-046 adversary Pass 2. Applies to both static HTML export (slideforge-html)
     and live web preview (slideforge-preview). See 'Rendering Model Scope Clarification'
     section below."
  - "2026-06-08: ARIA correction (Pass 3) — Fixed invalid aria-hidden inheritance claim in
     Layer Rules 5/6: the outer <svg> MUST NOT carry aria-hidden=\"true\" because WAI-ARIA
     forbids descendant re-exposure through an aria-hidden ancestor; the correct model is
     role=\"presentation\" on the outer <svg> plus individual aria-hidden=\"true\" on each
     decorative <g>. Canonicalized id format to sf-{slide_id}-{frame_index} (0-based,
     graphical frames only). Made the single-h1/no-title-fallback/title-absent rules
     explicit and exporter-level. Updated DOM code block and heading rule table accordingly."
---

# ADR-008: Web Preview via SVG + axum + WebSocket

## Context

`slideforge serve` must provide a live web preview that re-renders on source file changes.
The preview must be accessible (WCAG AA, NFR-013). S3 spike established that `<canvas>`
is opaque to axe-core and screen readers (WCAG 1.1.1), requiring SVG-based rendering.

## Decision

The web preview server is an embedded `axum` server. Slides render as SVG elements with
ARIA attributes. Changes are pushed to connected browsers via WebSocket. The Node.js
footprint is test-harness only.

## Consequences

**SVG canvas requirement (binding):** The slide rendering surface MUST use `<svg>` elements,
not `<canvas>`. Each slide element is a separately addressable SVG child with ARIA
attributes (`role`, `aria-label`, etc.). This is an architecture constraint, not a
preference — a `<canvas>` surface fails `@axe-core/playwright` WCAG 1.1.1 checks.

**Production server (zero Node.js):**
- axum serves the initial HTML + JavaScript shell
- WebSocket sends rendered SVG updates on change
- `notify` crate file watcher triggers re-build
- HTTP data source polling also triggers via the same re-build path

**Test harness (Node.js allowed):**
- `@axe-core/playwright` WCAG tests in `crates/slideforge-preview/tests/package.json`
- Playwright E2E tests in the same location
- `npm ci` during CI setup; never in production binary

**Live reload protocol:** WebSocket message is a JSON envelope containing the updated
SVG content and the slide index that changed. The browser replaces only the changed
slide's SVG node, avoiding full-page re-render.

**axum version:** `axum = "=0.8.1"` with `tokio` runtime. The preview server is only
active during `slideforge serve`; all other commands (`build`, `export`) run synchronously
without starting the server.

---

## Rendering Model Scope Clarification (Binding — 2026-06-08)

### Problem

The original "SVG canvas requirement" was underspecified. Two attempted interpretations
both fail:

- **Interpretation A (rejected):** Wrap the entire slide in one `<svg role="img">` canvas.
  Fails WCAG 1.3.1/2.4.6/4.1.2: collapsing a slide to a single AT leaf hides headings,
  body text, and chart structure from screen readers. axe-core reports `heading-order`,
  `page-has-heading-one`, and `landmark` violations.

- **Interpretation B (rejected):** Emit slide text inside `<svg><foreignObject>`.
  Fails at the SVG pipeline boundary: `usvg 0.47.0` DROPS `foreignObject` during tree
  construction. The static HTML exporter, PDF exporter (STORY-047), and web preview
  (STORY-048) all pass SVG through usvg for geometry normalization; `foreignObject`
  content is silently deleted in all three pipelines.

### Binding Decision: P4 Composite Rendering Model

The "SVG canvas requirement" applies to **the graphical rendering surface only** —
charts, diagrams, images, and decorative shapes. Slide text (title, body, bullets,
captions) is emitted as **real HTML elements** CSS-absolutely-positioned over a sibling
SVG graphics layer.

This is the authoritative rendering model for `slideforge-html` and `slideforge-preview`.
No deviation is permitted without an explicit new ADR.

### DOM Structure (binding)

```html
<article class="sf-slide" style="position: relative; width: Wpx; height: Hpx;">

  <!-- Layer 1: Real HTML text — semantically structured, AT-visible -->
  <h1 class="sf-title"
      style="position: absolute; left: Xpx; top: Ypx; width: Wpx; height: Hpx;
             font-size: Fps; color: #RRGGBB;">
    Slide Title Text
  </h1>
  <p class="sf-body"
     style="position: absolute; left: Xpx; top: Ypx; ...">
    Body paragraph text
  </p>
  <ul class="sf-bullets" style="position: absolute; ...">
    <li>Bullet item</li>
  </ul>

  <!-- Layer 2: SVG graphics — charts/images/decorative shapes -->
  <!-- role="presentation": outer SVG is a layout container, not a semantic landmark.
       MUST NOT use aria-hidden="true" here — WAI-ARIA forbids descendant re-exposure
       through an aria-hidden ancestor; the <g role="img"> children would be silently
       removed from the AT tree and axe-core would give a FALSE GREEN (hidden content
       is not scanned). Individual decorative frames carry their own aria-hidden="true". -->
  <svg role="presentation"
       viewBox="0 0 W H"
       style="position: absolute; top: 0; left: 0; width: 100%; height: 100%;
              pointer-events: none;">

    <!-- Non-decorative chart/diagram/image frame — AT-visible via role="img" + <title> -->
    <g role="img" aria-labelledby="sf-{slide_id}-{frame_index}">
      <title id="sf-{slide_id}-{frame_index}">Alt text for chart</title>
      <!-- chart SVG content; injected via quick-xml after usvg processing -->
    </g>

    <!-- Decorative shape frame — hidden individually -->
    <g aria-hidden="true">
      <!-- decorative SVG path/rect/etc. -->
    </g>

  </svg>

</article>
```

### Layer Rules (binding)

**HTML text layer:**

1. Every text frame from `LaidOutDeck` that carries textual content is rendered as a real
   HTML element: title → `<h1>` or `<h2>` (see heading rule below), body paragraph →
   `<p>`, bullet list → `<ul><li>`, caption → `<figcaption>`.
2. Position comes from `LaidOutFrame` EMU coordinates: convert to px at 96 dpi
   (`px = emu / 914400 * 96`). Emit as `position: absolute; left: Xpx; top: Ypx;
   width: Wpx; height: Hpx;` inline styles on each element.
3. Font size, color, bold/italic from `LaidOutFrame` brand/style fields → inline CSS
   `font-size`, `color`, `font-weight`, `font-style`.
4. Text frames MUST NOT appear in the SVG layer. Text that exists in the SVG layer
   (e.g., axis labels inside a chart SVG) is governed by the chart's own alt text —
   the chart is treated as an opaque graphic with a single `<title>` describing it.

**SVG graphics layer:**

5. The outer `<svg>` MUST carry `role="presentation"` (not `aria-hidden="true"`).
   **CRITICAL — WAI-ARIA inheritance rule (corrected from Pass-3 adversary finding):**
   `aria-hidden="true"` on an ancestor removes the ENTIRE subtree from the accessibility
   tree. A descendant `<g role="img">` cannot re-expose itself through an `aria-hidden`
   ancestor — WAI-ARIA forbids it. axe-core gives a FALSE GREEN in this case because
   it does not scan hidden subtrees. Using `role="presentation"` on the outer `<svg>`
   removes it as a semantic landmark while leaving its subtree fully accessible; the
   non-decorative `<g role="img">` children are then genuinely visible to AT.
6. Each non-decorative graphical frame (chart, diagram, image) is wrapped in a
   `<g role="img" aria-labelledby="sf-{slide_id}-{frame_index}">` with a
   `<title id="sf-{slide_id}-{frame_index}">` child containing the alt text.
   `{frame_index}` is the **0-based index** of the graphical frame within the slide
   (counting only graphical frames — text frames are excluded from this count).
   **Canonical id format:** `sf-{slide_id}-{frame_index}` — e.g., `sf-slide-3-0`,
   `sf-slide-3-1`. This format is unique within a document because `slide_id` is
   unique per slide and `frame_index` is unique per frame within that slide.
   The `<g>` itself carries NO `aria-hidden` attribute.
7. `role="img"` and `<title>` MUST be injected via `quick-xml` raw XML manipulation
   AFTER usvg geometry processing. usvg 0.47.0 strips all non-presentation attributes
   (including `role`, `aria-*`). Do not use the usvg tree API to set these.
8. Decorative shapes and backgrounds carry `aria-hidden="true"` on their own `<g>`.
   This is the per-element annotation; it does NOT propagate from the outer `<svg>`.

**Heading level rule (exporter-level invariant):**

| Slide type / frame condition | Title element | First sub-heading |
|------------------------------|--------------|-------------------|
| `slide_type == title` (deck-opening slide) | `<h1>` | `<h2>` |
| All other slide types that have a Title frame | `<h2>` | `<h3>` |
| Slide with body content but NO Title frame | No heading element | Content emits `<p>` / `<ul>` |
| Further nesting within any slide | `<h3>`, `<h4>` | No level-skipping |

**Exactly-one-h1 is an EXPORTER-LEVEL invariant**, not a per-slide local decision.
The exporter (not each slide renderer) is responsible for ensuring the resulting HTML
document contains exactly one `<h1>` element across all slides:

- The `<h1>` is assigned to the **first slide whose `slide_type == title`**.
- If NO slide has `slide_type == title`, the `<h1>` is assigned to the **first slide
  that contains a Title frame**, regardless of `slide_type`.
- If no slide has a Title frame at all (an edge case — a deck of pure body/chart
  slides), the first slide's first body frame is promoted to `<h1>` and a
  `tracing::warn!` is emitted: `"deck has no title slide and no Title frame; first
  body frame promoted to h1"`. This satisfies the WCAG `page-has-heading-one` rule.
- Heading levels MUST NOT skip (h1 → h3 without h2 is forbidden across the document).
- Heading level is determined by the exporter's heading-assignment pass over
  `LaidOutDeck` at render time, NOT by per-slide heuristic content inspection.
- `render_slide_to_html()` accepts the pre-computed `HeadingLevel` for its Title frame
  as a parameter — it does not determine heading level itself. The caller
  (`render_deck_to_html()`) performs the single exporter-level pass to assign levels
  before dispatching per-slide rendering.

**Rationale for this model vs alternatives:**

- Avoids `<canvas>` (ADR-008 original constraint, S3 WCAG finding): satisfied — no canvas.
- Survives usvg foreignObject drop (STORY-046 adversary finding): satisfied — text is in
  HTML, not SVG.
- axe-core `heading-order` passes deterministically across Chromium/Firefox/WebKit:
  satisfied — real `<h1>`/`<h2>` elements with compile-time level assignment.
- Shared model for static HTML export and web preview: satisfied — `render_slide_to_html()`
  produces the same DOM structure; STORY-047 wraps it in WebSocket push.
- Compatible with ADR-005 (Two-IR): satisfied — position and text both come from
  `LaidOutDeck`; no new IR fields required.
- STORY-047 (PDF) and STORY-048 (web preview) consume the same `render_slide_to_html()`
  function; no per-consumer rendering divergence is permitted.

### Impact on Downstream Stories

- **STORY-047 (web preview server):** Imports `render_slide_to_html()` from
  `slideforge-html`. The composite DOM structure is the shared model. WebSocket push
  sends the `<article>` HTML string; browser replaces the slide's article node.
- **STORY-048 (live reload):** Same as STORY-047 — no rendering change, only the
  WebSocket delta-push logic changes.
- **STORY-046 (static HTML export):** Implements P4. All AC and Architecture Compliance
  Rules in STORY-046 must align with this model (see story-writer delta list in the
  STORY-046 amendment record).
