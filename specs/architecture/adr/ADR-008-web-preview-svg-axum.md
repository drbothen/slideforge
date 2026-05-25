---
document_type: adr
adr_id: ADR-008
title: Web preview via SVG + axum + WebSocket
status: accepted
date: 2026-05-24
spike_input: S3-wcag-tooling-choice.md
traces_to: ARCH-INDEX.md
supersedes: ~
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
