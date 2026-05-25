---
document_type: ux-spec-screen
screen_id: "SCR-007"
screen_name: "Web Preview"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements:
  - "PRD §2.4 BC-4.03.003 (web preview)"
  - "PRD §2.4 BC-4.03.004"
  - "PRD §4 NFR-005 (WCAG AA)"
  - "BC-5.05.001-005 (watch mode)"
  - "S3 spike: SVG-not-canvas constraint"
  - "q1-decision-final.md §0 (live data-reactive deck)"
---

# Screen: Web Preview (SCR-007)

> **Sharded UX screen (DF-021).** Navigate via `UX-INDEX.md`.
> **ARCHITECTURE CONSTRAINT:** The slide canvas MUST use SVG-based rendering,
> NOT `<canvas>`. This is a hard constraint from spike S3 (WCAG 1.1.1 + 4.1.2
> compliance). A bare `<canvas>` is opaque to axe-core and screen readers.

## Purpose and User Context

The web preview is a localhost HTTP server started by `slideforge watch`. It shows
a live view of the compiled slides in the browser, updating via WebSocket when
source files change. The preview is optional — slideforge works without a browser —
but it is the primary feedback loop for authors iterating on deck content.

The web preview is NOT a WYSIWYG editor. Users cannot click to edit. It is a
read-only viewer with slide navigation and live rebuild feedback.

Access: `http://127.0.0.1:3000` (default port, configurable via `--port`).

---

## Page Structure

```
┌─────────────────────────────────────────────────────┐
│  [slideforge preview]    deck.sf     [●] Live        │  <- Header bar
├────────────────┬────────────────────────────────────┤
│                │                                    │
│  Slide list    │     Slide canvas (SVG)             │
│  (thumbnail    │     - Full-width SVG render        │
│   strip)       │     - Aspect ratio: 16:9           │
│                │     - Slide title as <h2>          │
│  [Slide 1]     │     - All shapes as SVG elements   │
│  [Slide 2]  ►  │       with ARIA labels             │
│  [Slide 3]     │                                    │
│  ...           │                                    │
│                │                                    │
├────────────────┴────────────────────────────────────┤
│  ◀ Prev   Slide 3 of 25   Next ▶         [Full] [↻] │  <- Navigation bar
└─────────────────────────────────────────────────────┘
```

---

## Elements

| ID | Type | Label | ARIA | Notes |
|----|------|-------|------|-------|
| ELM-001 | Header bar | "slideforge preview" wordmark | `role="banner"` | Fixed top |
| ELM-002 | Source filename | "deck.sf" | `aria-label="Source file: deck.sf"` | Current source |
| ELM-003 | Connection indicator | Colored dot + status text | `aria-live="polite"` | "Live" (green), "Connecting..." (yellow), "Disconnected" (red) |
| ELM-004 | Slide thumbnail strip | Scrollable list of slide thumbnails | `role="list"`, each `role="listitem"` | Left panel; active slide highlighted |
| ELM-005 | Active thumbnail indicator | Highlighted border on active slide | `aria-current="true"` on active thumbnail | `preview.accent` color |
| ELM-006 | Slide canvas container | SVG slide content | `role="main"`, `aria-label="Slide [N]: [Title]"` | Scales with viewport |
| ELM-007 | Slide SVG | Full slide content | Each SVG shape has `aria-label` or `aria-hidden="true"` | Generated from LaidOutDeck IR |
| ELM-008 | Slide title heading | Slide title text as `<h2>` | Natural heading | VISIBLE inside slide rendering; also present as screen-reader `<h2>` outside SVG |
| ELM-009 | Navigation bar | Prev/Next controls + counter | `role="navigation"`, `aria-label="Slide navigation"` | Fixed bottom |
| ELM-010 | Prev button | "◀ Prev" | `aria-label="Previous slide"`, `disabled` on slide 1 | `color.hint` when disabled |
| ELM-011 | Slide counter | "Slide 3 of 25" | `aria-live="polite"` | Updates on navigation |
| ELM-012 | Next button | "Next ▶" | `aria-label="Next slide"`, `disabled` on last slide | |
| ELM-013 | Fullscreen button | "[Full]" | `aria-label="Toggle fullscreen"` | Hides thumbnail strip |
| ELM-014 | Force reload button | "[↻]" | `aria-label="Force rebuild and reload"` | Triggers rebuild via WebSocket |
| ELM-015 | Error overlay | Red overlay on slide canvas | `role="alert"` | Shown when slide has build warning (warn-only mode) |
| ELM-016 | Loading overlay | Dim overlay with spinner | `aria-label="Rebuilding..."` | During incremental rebuild |

---

## SVG Slide Rendering Requirements

### Structure

Each compiled slide is rendered as an SVG element within the page:

```svg
<svg viewBox="0 0 9144000 5143500" aria-label="Slide [N]: [Title]"
     role="img" focusable="true">
  <!-- Background fill -->
  <rect width="9144000" height="5143500" fill="[bg-color]" aria-hidden="true"/>

  <!-- Title placeholder -->
  <text aria-label="Title: [slide title text]" role="text" ...>
    [title text]
  </text>

  <!-- Each visual element -->
  <image href="logo.png" aria-label="[alt text from DSL]" .../>
  <g aria-label="[chart alt text]" role="img">...</g>  <!-- chart SVG -->
  <g aria-hidden="true" role="presentation">...</g>    <!-- decorative: true -->
</svg>
```

Rules:
- Slide dimensions: 9,144,000 × 5,143,500 EMU (10 × 5.625 inches, 16:9)
- SVG `viewBox` uses EMU coordinates (no floating-point)
- Every non-decorative visual element has `aria-label` from DSL `alt` field
- Decorative elements (DSL `decorative: true`): `aria-hidden="true"`, `role="presentation"`
- SVG is NOT a single `<canvas>` element — this is the hard constraint from S3 spike

### Element ARIA Mapping

| DSL Element | SVG Output | ARIA |
|-------------|-----------|------|
| `image "logo.png" alt "Company logo"` | `<image>` | `aria-label="Company logo"` |
| `image "bg.png" decorative: true` | `<image>` | `aria-hidden="true"` |
| `chart: ... alt "Bar chart of ARR"` | `<g role="img">` | `aria-label="Bar chart of ARR"` |
| `diagram: ... alt "Architecture diagram"` | `<g role="img">` | `aria-label="Architecture diagram"` |
| Slide title text | `<text>` | `aria-label="Title: [text]"` |
| Slide body text | `<text>` | Inline (readable by AT) |

---

## Interactions

| ID | Trigger | Success Path | Error Path |
|----|---------|-------------|------------|
| INT-001 | Page load | WebSocket connects; first slide renders; thumbnail strip populates | If WS fails: show "Disconnected" status; display last-known slide |
| INT-002 | Click thumbnail | Slide canvas updates to selected slide; counter updates; URL hash updates (`#slide-3`) | N/A |
| INT-003 | Prev/Next button click | Navigate to adjacent slide; thumbnail strip scrolls to keep active thumb visible | At boundary: button disabled |
| INT-004 | Left/Right arrow key | Navigate prev/next slide (keyboard shortcut) | At boundary: no action, no error |
| INT-005 | File saved (WS message received) | Loading overlay appears; SVG slides update in-place; overlay removed; connection dot stays green | If rebuild fails: error overlay on affected slides; "1 warning" badge on connection dot |
| INT-006 | Force rebuild button | Sends rebuild message via WebSocket; loading overlay appears | Same as INT-005 error path |
| INT-007 | Fullscreen toggle | Thumbnail strip hides; slide canvas expands; button changes to "[Exit Full]" | N/A |
| INT-008 | URL with `#slide-N` | Opens directly to slide N | If N > total slides: show slide 1 |

---

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `→` or `→ arrow` | Next slide |
| `←` or `← arrow` | Previous slide |
| `Home` | First slide |
| `End` | Last slide |
| `F` | Toggle fullscreen |
| `R` | Force rebuild |
| `Tab` | Move focus between interactive elements (thumbnail list, nav buttons) |
| `Enter` / `Space` | Activate focused interactive element |
| `Escape` | Exit fullscreen |

---

## Connection Status States

| State | Indicator | Behavior |
|-------|-----------|---------|
| Connecting | Yellow dot, "Connecting..." | WebSocket attempting connection; auto-retry 3x at 1s intervals |
| Live | Green dot, "Live" | WebSocket connected; `aria-live="polite"` announcement on change |
| Rebuilding | Yellow dot + spinner, "Rebuilding..." | Build in progress after file change |
| Build Error | Orange dot, "1 warning" (or "N warnings") | Warn-only mode; some slides show error overlays |
| Disconnected | Red dot, "Disconnected" | WebSocket lost; auto-retry with exponential backoff (1s, 2s, 4s, max 30s) |

Status changes announced via `aria-live="polite"` region.

---

## Error Overlay (Warn-Only Slides)

When a slide has an associated build warning (in `--warn-only` mode), an overlay
renders over that slide in the preview:

```
┌─────────────────────────────────────────┐
│  ⚠  Build Warning                       │
│                                         │
│  E-EVL-001                              │
│  Undefined variable '{{ client_name }}'  │
│                                         │
│  Fix in deck.sf and save to refresh     │
└─────────────────────────────────────────┘
```

Overlay styling (tokens from UX-INDEX.md):
- Background: `preview.error.bg` with 90% opacity (allows original slide to show dimly)
- Icon + "Build Warning": `preview.error.text`
- Error code: `preview.error.text` bold
- Message: `preview.error.text` (truncated at 60 chars)
- Fix instruction: `preview.text.secondary`

The overlay has `role="alert"` and is announced immediately by screen readers.

---

## Accessibility

- **Tab order**: Header controls → Thumbnail strip → Slide canvas → Navigation bar
- **ARIA landmarks**: `role="banner"` (header), `role="main"` (slide canvas), `role="navigation"` (nav bar), `role="complementary"` (thumbnail strip)
- **Keyboard shortcut disclosure**: `title` attribute on buttons explains key shortcut (e.g., `title="Next slide (→)"`)
- **Focus management**: When slide changes via keyboard, focus remains on canvas; when slide changes via thumbnail click, focus moves to canvas
- **Heading hierarchy**: Page `<h1>` = "slideforge preview — [filename]"; each slide `<h2>` = slide title
- **SVG not canvas**: All slide content in SVG with per-element ARIA (per S3 spike hard constraint)
- **prefers-reduced-motion**: Slide transition animation disabled; WebSocket reconnect pulse disabled
- **prefers-contrast**: Focus rings increase to `outline: 3px solid preview.accent`
- **High contrast**: Error/warning overlays increase background opacity to 95%

## Responsive Adaptations

| Breakpoint | Layout Change |
|-----------|--------------|
| < 768px | Thumbnail strip hidden by default; accessible via "Slides" button |
| 768px–1023px | Thumbnail strip collapses to icon-only thumbnails (no titles) |
| 1024px+ | Full layout as specified above |
| Portrait mobile | Slide canvas letterboxed; nav bar stack-layout |
