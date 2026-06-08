---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-048
title: "Web Preview: Live Reload + Accessibility"
epic: EPIC-14
wave: 5
points: 8
priority: P1
tdd_mode: strict
status: draft
behavioral_contracts: [BC-5.05.005]
verification_properties: []
nfr_refs: [NFR-013, NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-preview
target_module: slideforge-preview
subsystems: [SS-09]
depends_on:
  - STORY-047
blocks: []
estimated_days: 4
---

# STORY-048: Web Preview — Live Reload + Accessibility

## Subsystem Anchor Justification

SS-09 (HTML/Preview) owns this story because it extends `slideforge-preview` with
the browser-side client JavaScript (embedded in the served HTML) and the file watcher
integration. Both features are within the preview subsystem's scope. ARCH-INDEX SS-09
covers "slideforge-html, slideforge-preview."

## Dependency Anchor Justifications

- Depends on STORY-047: The WebSocket server infrastructure (broadcast channel,
  `push_update()`, `push_error()`) established in STORY-047 is required before
  the browser-side client can receive messages. This story adds the client-side
  JavaScript and the file watcher integration.
- Does not directly block any story: STORY-050 (E2E integration tests) depends on
  STORY-049 (plugin registry) which is Wave 4 — the E2E suite exercises preview
  behavior via `slideforge-cli` (STORY-055/056), not directly from this crate.

## Summary

Complete the web preview feature by adding:

1. **Browser-side WebSocket client** (embedded JavaScript in the served HTML):
   - Connects to `ws://localhost:<port>/live` on page load.
   - Applies `{type: "reload", slides: [...]}` messages by updating the `<svg>`
     elements in-place (no full page reload).
   - Applies `{type: "error", errors: [...]}` messages by showing an error overlay.
   - Applies `{type: "full-state", slides: [...]}` messages (sent on reconnect).
   - Shows "Reconnecting..." indicator while disconnected.
   - Implements exponential backoff reconnection: 500ms → 1s → 2s → 4s ... max 30s.
   - After 5 minutes total disconnection time, shows "Disconnected — refresh to
     reconnect" and stops attempting.

2. **File watcher integration** (in `slideforge-cli`, calling into `slideforge-preview`):
   - `notify =8.2.0` watches the .sf file and data source files (bumped from workspace
     baseline of =6.1.1 to =8.2.0 per ADR-022 root bump).
   - Debounce is handled by `notify-debouncer-full =0.7.0` (preferred over hand-rolled
     debounce; consistent with STORY-047's 100ms debounce in `debounce.rs`).
   - On debounced change event, re-evaluate, then call `push_update()` or `push_error()`.

3. **Accessibility features** of the preview page:
   - `aria-live="polite"` region for connection status announcements.
   - `aria-label` on slide navigation (`<nav>`).
   - Keyboard navigation: arrow keys move between slides.
   - `prefers-reduced-motion: reduce` media query disables slide transition animations.
   - Error overlay is accessible: `role="alert"` with `aria-live="assertive"`.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-5.05.005 | WebSocket Connection Drop from Web Preview Reconnects Automatically | AC-001 through AC-009 |

## Acceptance Criteria

### AC-001: Browser detects disconnection within 5 seconds
(traces to BC-5.05.005 postcondition 1)

The browser-side WebSocket `onclose` event handler fires within 5 seconds of
connection loss. The implementation uses the WebSocket API's built-in close
detection (native browser event, no polling required). Integration test:
Playwright simulates WebSocket close and asserts the "Reconnecting..." indicator
appears within 5 seconds.

### AC-002: Exponential backoff reconnection: 500ms, 1s, 2s, 4s, ... max 30s
(traces to BC-5.05.005 postcondition 2 and invariant 3)

The browser-side client implements exponential backoff with the sequence:
`delay = min(500ms * 2^attempt, 30000ms)`. Attempts are made until either
reconnection succeeds or 5 minutes total elapse. Integration test (Playwright):
mock server down, measure 3 consecutive reconnect attempt intervals, assert
they are approximately 500ms, 1000ms, 2000ms.

### AC-003: Full-state sync on reconnect
(traces to BC-5.05.005 postcondition 3 and invariant 2)

When a new WebSocket connection is established (or re-established), the server
immediately sends a `{type: "full-state", slides: [...], current_slide: N}` message.
The server maintains the current deck state in memory for this purpose.
Integration test: disconnect and reconnect WebSocket client, assert `full-state`
message is the first message received after reconnect.

### AC-004: Browser updates to current state without full page reload
(traces to BC-5.05.005 postcondition 4)

When a `{type: "reload"}` or `{type: "full-state"}` message is received, the
JavaScript client updates the `<svg>` elements in the DOM without triggering
`window.location.reload()`. Verified by Playwright test: monitor `window.performance
.getEntriesByType("navigation")` — navigation events should not increase after a
reload message.

### AC-005: "Reconnecting..." indicator visible while disconnected
(traces to BC-5.05.005 postcondition 5)

While the WebSocket is disconnected and reconnect attempts are in progress, the
HTML page shows a visible `<div role="status" aria-live="polite">Reconnecting...
</div>` element. This element is hidden (CSS `display: none`) when connected.
Playwright integration test: simulate disconnect, assert element is visible.

### AC-006: "Disconnected" message shown after 5-minute timeout
(traces to BC-5.05.005 postcondition 6 and edge case EC-005)

After 300 seconds (5 minutes) of failed reconnection attempts, the status
indicator changes to "Disconnected — refresh to reconnect" and no further
reconnect attempts are made. The user can refresh to re-establish the connection.

### AC-007: aria-live region announces connection status
(traces to NFR-013 and BC-4.03.003)

The connection status element uses `role="status"` with `aria-live="polite"`.
State changes ("Connected", "Reconnecting...", "Disconnected") are announced by
screen readers. The error overlay uses `role="alert"` with `aria-live="assertive"`
for immediate announcement of compilation errors.

### AC-008: Keyboard navigation between slides
(traces to NFR-013)

The preview page supports keyboard navigation:
- `ArrowRight` / `PageDown` → advance to next slide.
- `ArrowLeft` / `PageUp` → go to previous slide.
- `Home` → first slide.
- `End` → last slide.
The current slide `<article>` element receives `aria-current="true"`.

### AC-009: prefers-reduced-motion is respected
(traces to BC-4.03.003 invariant 2 and NFR-013)

If the browser reports `prefers-reduced-motion: reduce`, slide transitions
(CSS animations or SMIL animations in SVG) are disabled. The embedded CSS
includes:
```css
@media (prefers-reduced-motion: reduce) {
  .slide-transition { animation: none; transition: none; }
}
```

## Tasks

- [ ] Create embedded JavaScript in `crates/slideforge-preview/src/client_js.rs`
  (included in the served HTML as inline `<script>`):
  - WebSocket connect/onmessage/onclose handlers
  - Exponential backoff reconnection (AC-002)
  - DOM update for `{type: "reload"}` and `{type: "full-state"}`
  - Error overlay toggle for `{type: "error"}`
  - "Reconnecting..." / "Disconnected" indicator
  - Keyboard navigation (AC-008)
  - `prefers-reduced-motion` CSS (AC-009)
- [ ] Update `server.rs` (STORY-047) to handle `{type: "full-state"}`:
  - Store current `Vec<SlideHtml>` in `PreviewServer` state
  - Send `full-state` message as first message to each new WebSocket connection
- [ ] Create `crates/slideforge-preview/src/watcher.rs`:
  - `FileWatcher::watch(path: &Path, on_change: impl Fn() + Send + 'static)`
  - Uses `notify =8.2.0` (workspace per ADR-022; bumped from =6.1.1 baseline)
  - Debounce via `notify-debouncer-full =0.7.0` (preferred over hand-rolled debounce;
    aligns with STORY-047's 100ms debounce contract)
  - `EventHandler` trait is implemented via blanket impl for closures (`FnMut`);
    event types are re-exported from `notify-types`
  - Enable `crossbeam-channel` feature on `notify` if bounded queue is needed
    (default is `crossbeam-channel = false`); queue overflow surfaces via
    `EventKind::Other` — trigger a full rescan on overflow
- [ ] Update `slideforge-cli` `watch` subcommand (STORY-056 will finalize CLI; this
  story adds the plumbing between notify and the preview server)
- [ ] Write integration tests (Playwright-based, in `tests/e2e/`):
  - Connect client → disconnect → assert "Reconnecting..." visible
  - Reconnect → assert "full-state" received → assert slides updated
  - 5-minute timeout behavior (mocked with shorter timeout for testing)
  - `prefers-reduced-motion` reduces animations
  - Keyboard navigation: ArrowRight advances to slide 2
- [ ] Add `notify = { workspace = true }` and `notify-debouncer-full = { workspace = true }`
  to `slideforge-preview` Cargo.toml (pins =8.2.0 and =0.7.0 respectively per ADR-022)

## Previous Story Intelligence

STORY-047 established `PreviewServer` with `push_update()`, `push_error()`,
and the WebSocket endpoint at `/live`. The WebSocket broadcast channel from
STORY-047 is reused here for the `full-state` message dispatch.

Key lesson from STORY-047: the 100ms debounce contract is established in
`debounce.rs`. This story uses `notify-debouncer-full =0.7.0` as the debounce
mechanism, which wraps notify's event stream and fires after a 100ms quiet window.
This is consistent with STORY-047's debounce contract and replaces any hand-rolled
debounce. Do not re-implement debounce — use `notify-debouncer-full` in
`watcher.rs`, which provides the same 100ms coalescing behavior as `debounce::Debouncer`.

## Architecture Compliance Rules

1. **Client-side reconnect (BC-5.05.005 invariant 1)**: Reconnection is handled
   entirely in browser JavaScript. The axum server does not detect client disconnects
   beyond the WebSocket close event. The server never attempts to re-push to a
   disconnected client.
2. **Full-state on every new connection (BC-5.05.005 invariant 2)**: Every new or
   reconnecting WebSocket client receives `{type: "full-state", ...}` as its first
   message. No assumption about prior client state.
3. **Backoff cap at 30s (BC-5.05.005 invariant 3)**: The JavaScript client's backoff
   delay must not exceed 30,000ms between attempts. Verified by integration test
   in AC-002.
4. **aria-live for status (NFR-013)**: All dynamic status changes must go through
   `aria-live` regions. No status change is communicated only via visual means.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `notify` | `{ workspace = true }` = `=8.2.0` (ADR-022) | File system event watcher. Bumped from workspace baseline =6.1.1 to =8.2.0 per ADR-022 root bump. `EventHandler` trait has blanket impl for `FnMut` closures. Event types re-exported from `notify-types`. `crossbeam-channel` feature default-off; enable only if bounded queue is needed. Queue overflow surfaces via `EventKind::Other` → trigger full rescan. |
| `notify-debouncer-full` | `{ workspace = true }` = `=0.7.0` (ADR-022) | Debounce wrapper for notify — preferred over hand-rolled debounce; handles the 100ms window consistent with STORY-047 `debounce.rs` contract |
| `slideforge-preview` (self, STORY-047) | workspace | Extends server with full-state + watcher |
| `slideforge-html` | workspace | `render_slide_to_html()` |
| (Browser JavaScript) | Inline | WebSocket reconnect logic; no npm packages in output |

All workspace-pinned crates centralized in `[workspace.dependencies]` per ADR-022.
Note: The browser-side JavaScript is embedded inline in the served HTML as a
`<script>` block. No npm/node.js packages are used in the shipped HTML — the
JavaScript is minimal, vanilla JS, with no external dependencies. This is a
deliberate choice for simplicity and offline usability.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-preview/src/client_js.rs` | Create | Embedded JS (const str, included in HTML) |
| `crates/slideforge-preview/src/watcher.rs` | Create | notify 8.2.0 + notify-debouncer-full 0.7.0 file watcher integration (ADR-022) |
| `crates/slideforge-preview/src/server.rs` | Modify | Add full-state dispatch on new WS connection |
| `crates/slideforge-preview/Cargo.toml` | Modify | Add `notify = { workspace = true }` (=8.2.0) and `notify-debouncer-full = { workspace = true }` (=0.7.0) per ADR-022 |
| `crates/slideforge-preview/tests/e2e/` | Create | Playwright integration tests directory |
| `crates/slideforge-preview/tests/e2e/reconnect.spec.ts` | Create | Reconnect + full-state test |
| `crates/slideforge-preview/tests/e2e/keyboard.spec.ts` | Create | Keyboard navigation test |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,800 |
| BC-5.05.005 | ~1,200 |
| STORY-047 server.rs context | ~1,500 |
| STORY-047 debounce.rs context | ~600 |
| notify 8.2.0 + notify-debouncer-full 0.7.0 API docs | ~900 |
| JavaScript client code to write | ~2,000 |
| Playwright test files to write | ~2,000 |
| **Total** | **~10,900** |

Context budget: ~11% of a 100k-token context window. Within limit.

## Test Strategy

- **Unit tests**: `Debouncer` coalesces events (from STORY-047). `FileWatcher`
  triggers callback on file write.
- **Integration tests (Playwright)**: Full reconnect cycle: server stop → client
  shows "Reconnecting..." → server restart → client receives `full-state` → slides
  updated. Keyboard navigation advances slides. `prefers-reduced-motion` disables
  animations.
- **Accessibility audit**: The preview page passes axe-core on CI (covered by
  STORY-046's `html-wcag.yml` CI gate, extended to also test the preview page).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Laptop sleep/wake (OS network suspension) | WebSocket close on wake; reconnect within 500ms |
| EC-002 | Server restarted (watch process restarted) | Client reconnects when server is back |
| EC-003 | Multiple browser tabs; one loses connection | Only that tab reconnects; others unaffected |
| EC-004 | Server sends update before client finishes reconnect | Client receives via full-state sync on reconnect |
| EC-005 | 5-minute maximum timeout reached | "Disconnected — refresh" shown; no more attempts |

## Forbidden Dependencies

Same as STORY-047. No npm packages in the shipped HTML (inline vanilla JS only).
`notify` is the only new Rust dependency introduced in this story.
