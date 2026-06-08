---
document_type: ux-spec-screen
screen_id: "SCR-008"
screen_name: "Watch Mode"
version: "1.1"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
modified: 2026-06-07
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements:
  - "PRD §2.5 BC-5.05.001-005"
  - "PRD §4 NFR-001 (< 500ms cold build — governs v1.0 watch rebuild latency)"
  - "PRD §4 NFR-002 (< 50ms incremental rebuild — DEFERRED to v1.x; see nfr-catalog v1.3)"
  - "interface-definitions.md §1.3"
  - "q16-q25-decisions.md Q23 (error recovery — error-slide placeholders)"
  - "q1-decision-final.md §1 (HTTP data sources polled in watch mode)"
---

# Screen: Watch Mode (SCR-008)

> **Sharded UX screen (DF-021).** Navigate via `UX-INDEX.md`.
> This screen covers the watch mode lifecycle as a combined terminal + browser
> experience. Terminal behavior is specified here; web preview browser behavior
> is in SCR-007. This screen focuses on the coordination between the two.

## Purpose and User Context

Watch mode is the primary authoring loop. The developer edits `.sf` files in their
editor, and the web preview updates automatically. The terminal shows rebuild events.
The v1.0 goal is a fast feedback loop bounded by the full rebuild path (NFR-001
< 500ms). The sub-50ms incremental target (NFR-002, comemo-based) is deferred to
v1.x.

The combined UX is: editor on left, terminal (optional) in middle, browser on right.
The terminal is optional — if the user ignores it after launch, the browser preview
is the entire feedback surface.

---

## Watch Mode States (Terminal + Browser Combined)

| State | Terminal | Browser |
|-------|----------|---------|
| Starting | Banner + initial build output | Loading splash |
| Idle (success) | "Waiting for changes..." | Live slides, green dot |
| File changed, rebuilding | "[watch] file.sf changed — rebuilding..." | Loading overlay on affected slide |
| Rebuild success | Success line (ms timing) | Slides update in-place; green dot |
| Rebuild with warnings | Warning block | Error overlays on affected slides; orange dot |
| Parse error (fatal even in warn-only) | Error block | Error overlay on all slides |
| HTTP data refreshed | Poll line | Same as rebuild success |
| HTTP data fetch failed | Warning line (previous data kept) | Previous data shown; warning badge |
| Disconnected (browser) | No change | Red dot; retry backoff |
| SIGINT | "Stopping. Bye." | Server gone; browser shows connection error |

---

## Incremental Rebuild UX

> **NFR-002 deferral (approved 2026-06-07):** The < 50ms incremental rebuild target
> (NFR-002, comemo-based) is DEFERRED to v1.x. In v1.0, watch mode performs a
> full re-evaluation on every change. Perceived latency is governed by NFR-001
> (< 500ms cold build). The terminal timing display and browser update path below
> are unchanged — only the performance commitment shifts from < 50ms to < 500ms
> for v1.0.

In v1.0, each file-save triggers a full rebuild. The terminal line shows actual
elapsed time; users will typically see 50-400ms depending on deck size.

Terminal example (v1.0 full rebuild):
```
[watch] deck.sf changed (14:22:01) — rebuilt in 87ms
```

Terminal for slower rebuild (multifile or data refresh):
```
[watch] deck.sf changed (14:22:01) — rebuilding...
  Rebuilt in 234ms — 25 slides (3 data sources refreshed)
```

The "rebuilding..." intermediate line appears only if the rebuild takes > 100ms.
Sub-100ms rebuilds show only the completion line (avoids flicker).

When NFR-002 lands in v1.x (comemo incremental), the typical fast-path timing
will drop to < 50ms and single-slide delta updates will replace full rebuilds.

---

## Error Recovery in Watch Mode (per Q23)

Watch mode is always in warn-only mode (per Q17 decision). The behavior:

1. **Parse error** (always fatal): Error printed to terminal; web preview shows "Build failed" overlay on all slides; error persists until fixed
2. **Validation error in warn-only**: Warning printed to terminal; affected slides show error-slide placeholders in preview; other slides update normally
3. **Data source error**: Warning printed; affected slides show "Data unavailable" placeholder; last-good data kept for unaffected slides

### Error-Slide Placeholder (Web Preview)

When a slide fails to render in warn-only mode, the SVG canvas for that slide
shows the error-slide placeholder (specified in SCR-007 ELM-015).

The placeholder slide in the thumbnail strip shows:
- Red thumbnail background (preview.error.bg)
- Error code text centered
- No normal slide content

### Recovery: When the error is fixed and file saved

1. Terminal: new rebuild line shows success
2. Browser: error overlay fades; normal slide content fades in
3. If `prefers-reduced-motion`: instant swap, no fade animation

---

## WebSocket Protocol (UX-Visible Behavior)

The WebSocket carries slide update payloads from server to browser. From a UX
perspective, the visible behaviors are:

| Message Type | Terminal | Browser |
|-------------|----------|---------|
| `build_started` | (silent) | Loading overlay appears |
| `build_success` | Success line | Overlays removed; SVG slides updated |
| `build_warning` | Warning block | Error overlays on affected slides |
| `build_error` | Error block | "Build failed" overlay on all slides |
| `slide_delta` | (silent) | Only changed slides re-render (not full page) |

The `slide_delta` message delivers efficient browser updates: only the SVG elements
that changed are swapped in the DOM, not the entire page. In v1.0 this follows a
full rebuild; in v1.x (NFR-002) it will follow a comemo incremental rebuild.

---

## Interactions

| ID | Trigger | Success Path | Error Path |
|----|---------|-------------|------------|
| INT-001 | Source file saved | Detect change; trigger pipeline; push `slide_delta` to browser | Error paths per error category |
| INT-002 | Included file saved | Same as INT-001 | Include cycle: E-PAR-004 |
| INT-003 | Data file saved (JSON/CSV) | Detect change; re-evaluate data-bound slides only; push delta | Data parse error: E-DAT-003 warning |
| INT-004 | HTTP poll interval expires | Fetch all HTTP sources; compare with previous; rebuild only if changed | HTTP fail: E-DAT-001 warning; keep previous data |
| INT-005 | Browser WebSocket disconnects | Server retries connection (exponential backoff); red dot in browser | If server stopped: "Disconnected" persists |
| INT-006 | Browser reconnects | Full rebuild state sent to new browser client | N/A |

---

## Accessibility

- Terminal: all watch events include timestamp and file path (scripting-safe)
- Browser: connection status changes announced via `aria-live="polite"`
- Error overlays: `role="alert"` for immediate screen reader announcement
- `prefers-reduced-motion`: all transition animations disabled; instant DOM swaps

## Responsive Adaptations

Watch mode terminal output follows SCR-002 responsive rules.
Web preview follows SCR-007 responsive rules.
