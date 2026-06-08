---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-047
title: "Web Preview Server: axum + WebSocket + SVG Canvas"
epic: EPIC-14
wave: 5
points: 8
priority: P1
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.03.004]
verification_properties: []
nfr_refs: [NFR-002, NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-preview
target_module: slideforge-preview
subsystems: [SS-09]
depends_on:
  - STORY-046
blocks:
  - STORY-048
estimated_days: 4
---

# STORY-047: Web Preview Server — axum + WebSocket + SVG Canvas

## Subsystem Anchor Justification

SS-09 (HTML/Preview) owns this story because `slideforge-preview` is the second
of the two SS-09 crates per ARCH-INDEX ("slideforge-html, slideforge-preview").
The preview server is a live development tool that reuses the rendering functions
from `slideforge-html` (STORY-046) for SVG slide output.

## Dependency Anchor Justifications

- Depends on STORY-046: `slideforge-preview` imports `render_slide_to_html()` from
  `slideforge-html` to generate the initial page and WebSocket update payloads.
  Without the HTML rendering functions, the preview server has nothing to serve.
- Blocks STORY-048: Live reload (file watcher + WebSocket push) is built on top of
  the WebSocket endpoint established in this story.
- Does not directly block STORY-050: STORY-050 is Wave 4 — it depends on STORY-049
  (plugin registry assembly), not on Wave 5 preview stories. Preview behavior is
  exercised in E2E tests via the CLI watch mode (STORY-056), not by a direct dep.

## Summary

Implement the `slideforge-preview` crate: a local development HTTP/WebSocket server
that serves a live deck preview using `axum 0.8.9` (note: 0.8.2 was yanked; minimum
safe 0.8.x is 0.8.9). The async runtime is `tokio 1.52.3` introduced per ADR-021.

1. `PreviewServer` struct that starts an axum server on `localhost:<port>` (default 3000).
2. HTTP `GET /` serves the initial HTML page with the deck preview (SVG slides).
3. WebSocket endpoint `GET /live` upgrades to a WebSocket connection.
4. `push_update(slides: Vec<SlideHtml>)` sends a JSON message `{type: "reload",
   slides: [...]}` to all connected WebSocket clients.
5. `push_error(errors: Vec<DiagnosticMessage>)` sends `{type: "error", errors: [...]}`
   to all connected clients.
6. Graceful shutdown on Ctrl+C (`SIGTERM`).
7. Debounce: maximum one re-evaluation per 100ms even under rapid saves.
8. Runs in a separate async task — does not block the file watcher or evaluation pipeline.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.03.004 | Web Preview Served via axum+WebSocket+SVG Canvas; Updates on Save | AC-001 through AC-009 |

## Acceptance Criteria

### AC-001: HTTP server starts and serves initial preview
(traces to BC-4.03.004 postcondition 1)

`PreviewServer::start(port: u16, initial_deck: &LaidOutDeck) -> Result<PreviewHandle, PreviewError>`
starts the axum server on `localhost:<port>` and returns a `PreviewHandle` that wraps
the `JoinHandle<()>`. `GET /` returns a 200 OK response with `Content-Type: text/html`
containing the initial deck preview (SVG slides).
Integration test: `reqwest::get("http://localhost:<port>/")` returns 200 with body
containing `<!DOCTYPE html>`.

Note: `start()` returns `Result<PreviewHandle, PreviewError>` rather than bare
`JoinHandle<()>` so that port-in-use errors (AC-009) can be surfaced at call time
before the server task is fully spawned. `PreviewHandle` wraps the `JoinHandle<()>`
for join/abort access.

### AC-002: WebSocket endpoint accepts browser connections
(traces to BC-4.03.004 postcondition 2)

`GET /{id}` (axum 0.8 path syntax) — specifically `GET /live` — upgrades to a
WebSocket connection within 1 second of server start. axum 0.8 provides built-in
WebSocket support via `axum::extract::ws::WebSocketUpgrade`; `tokio-tungstenite` is
NOT used at runtime (it is a dev-dep test client only).
Integration test: `tokio-tungstenite =0.29.0` (dev-dep) client connects to
`ws://localhost:<port>/live` and receives a connection-established event within 1 second.

### AC-003: reload message pushed on deck update
(traces to BC-4.03.004 postcondition 3a and 3b)

When `PreviewServer::push_update(rendered_slides)` is called, all connected
WebSocket clients receive a JSON message:
```json
{"type": "reload", "slides": ["<svg>...</svg>", ...]}
```
Integration test: connect client, call `push_update`, assert client receives
`{"type": "reload", ...}` within 500ms.

### AC-004: error message pushed on evaluation failure
(traces to BC-4.03.004 postcondition 3c)

When `PreviewServer::push_error(errors)` is called, all connected clients receive:
```json
{"type": "error", "errors": [{"file": "...", "line": N, "col": N, "message": "..."}]}
```
Prior slide content remains visible (the browser does not blank the preview on error).
Integration test: connect client, call `push_error([...])`, assert client receives
`{"type": "error", ...}`.

### AC-005: Server shuts down gracefully on SIGTERM / Ctrl+C
(traces to BC-4.03.004 postcondition 4)

`PreviewServer` responds to `tokio::signal::ctrl_c()`. When Ctrl+C is received:
- All WebSocket connections are closed with code 1001 (Going Away).
- The HTTP server stops accepting new connections.
- The function returns (or the JoinHandle completes) within 2 seconds.

### AC-006: Preview updates slide content only — no file output
(traces to BC-4.03.004 invariant 3)

`PreviewServer` never writes `.pptx`, `.docx`, `.pdf`, or `.html` output files to
disk. Integration test: run `slideforge watch deck.sf` for 10 seconds while
making changes, assert no output files appear in the current directory.

### AC-007: axum server does not block re-evaluation
(traces to BC-4.03.004 invariant 4)

`PreviewServer` runs in a separate `tokio::spawn()` task. The file watcher and
evaluation pipeline are in the main task (or a separate task). `push_update()` is
a non-blocking send to a `tokio::sync::broadcast::Sender<WebSocketMessage>`. If no
clients are connected, `push_update()` completes immediately (no queuing).

### AC-008: Debounce — at most one evaluation per 100ms
(traces to BC-4.03.004 edge case EC-005)

Rapid file saves (10+ saves/second) do not trigger 10+ re-evaluations. The file
watcher integration applies a 100ms debounce before triggering re-evaluation.
Integration test: simulate 10 file change events in 50ms, assert only 1 evaluation
is triggered.

### AC-009: Port-in-use error is user-friendly
(traces to BC-4.03.004 edge case EC-003)

If `localhost:<port>` is already bound, `PreviewServer::start()` returns
`Err(PreviewError::PortInUse { port })`. The error message includes the hint:
"Use --port <N> to specify a different port." The server does not panic.

This is consistent with the AC-001 return type: `start() -> Result<PreviewHandle, PreviewError>`
where `PreviewError::PortInUse` is the `Err` variant. The `PreviewHandle` returned on
success wraps the `JoinHandle<()>` for lifecycle management.

## Tasks

- [ ] **[Task 1 — Workspace scaffold]** Create the `crates/slideforge-preview/` crate
  scaffold: `Cargo.toml` with `[dependencies]` section and `src/lib.rs` stub. Add
  `"crates/slideforge-preview"` to `[workspace] members` in the root `Cargo.toml`.
  Note: `slideforge-html` (STORY-046) and `slideforge-preview` (this story) are both
  added to the workspace at the start of Wave 5 before their respective stories are
  dispatched. The CI workspace build gate for Wave 5 requires both crates to compile.
- [ ] Create `crates/slideforge-preview/Cargo.toml`:
  - Runtime deps: `axum = { version = "=0.8.9", features = ["ws"] }` (note: 0.8.2 was yanked;
    axum 0.8 has built-in WebSocket via `axum::extract::ws::WebSocketUpgrade`; path syntax `/{id}`;
    graceful shutdown via `axum::serve(l, app).with_graceful_shutdown(...)`),
    `tokio = { version = "=1.52.3", features = ["rt-multi-thread", "macros", "net", "time", "sync", "signal", "fs", "io-util"] }`
    (NOT features = ["full"]; curated features per ADR-021 async runtime introduction),
    `serde = { workspace = true }` (=1.0.228 per ADR-022),
    `serde_json = { workspace = true }` (=1.0.150 per ADR-022),
    `thiserror = { workspace = true }` (=2.0.18 per ADR-022),
    `slideforge-html` (workspace), `slideforge-types` (workspace)
  - Dev deps: `tokio-tungstenite = "=0.29.0"` (test WebSocket client only — NOT a runtime dep;
    axum 0.8 built-in WebSocket replaces tungstenite at runtime)
- [ ] Create `crates/slideforge-preview/src/lib.rs` — re-export `PreviewServer`
- [ ] Create `crates/slideforge-preview/src/server.rs` — `PreviewServer` struct:
  - `pub fn start(port: u16, initial: &LaidOutDeck) -> Result<PreviewHandle, PreviewError>`
    (`PreviewHandle` wraps `JoinHandle<()>`; `PortInUse` is the `Err` variant — consistent with AC-001/AC-009)
  - `pub fn push_update(&self, slides: Vec<SlideHtml>)`
  - `pub fn push_error(&self, errors: Vec<DiagnosticMessage>)`
  - Internal: axum router with `GET /` and `GET /live`
  - Internal: broadcast channel for WebSocket message dispatch
- [ ] Create `crates/slideforge-preview/src/ws_handler.rs` — WebSocket upgrade + message loop
- [ ] Create `crates/slideforge-preview/src/messages.rs` — `WebSocketMessage` enum + serde
- [ ] Create `crates/slideforge-preview/src/debounce.rs` — 100ms debounce logic
- [ ] Create `crates/slideforge-preview/src/error.rs` — `PreviewError` enum
- [ ] Add `slideforge-preview` to workspace `Cargo.toml`
- [ ] Write integration tests:
  - Server starts → GET / → 200 OK with HTML
  - WebSocket connects → receives connection
  - `push_update()` → client receives reload JSON within 500ms
  - `push_error()` → client receives error JSON
  - Port in use → `Err(PreviewError::PortInUse)`
  - Debounce: 10 events in 50ms → 1 evaluation
- [ ] Write unit test: `debounce::Debouncer` coalesces 10 events into 1

## Previous Story Intelligence

STORY-046 established `slideforge-html::render_slide_to_html()`. This story imports
and calls that function to produce the SVG slide content for WebSocket messages.

Key design from STORY-046: `render_slide_to_html()` returns `String` (not
`&mut dyn Write`), making it directly usable in the JSON message payload.

The WebSocket `{type: "reload", slides: [...]}` message format is internal to v1.x
(BC-4.03.004 invariant 2). Changing it in a v1.x patch is not a breaking change from
a semver standpoint, but it would break existing browser clients. Design the format
to be extensible from the start: use a `type` discriminator field.

## Architecture Compliance Rules

1. **Non-blocking server (BC-4.03.004 invariant 4)**: `PreviewServer` MUST run in
   its own `tokio::spawn()` task. The evaluation pipeline is never blocked waiting
   for WebSocket clients. Use `broadcast::Sender` for fire-and-forget push.
2. **Watch mode is always warn-only (BC-4.03.004 invariant 1)**: `PreviewServer` does
   not control the evaluation mode — the `slideforge watch` CLI sets it. This crate
   only receives pre-rendered `SlideHtml` strings and pushes them.
3. **No output files written (BC-4.03.004 invariant 3)**: `PreviewServer` has no
   filesystem writes. This is verified by the integration test in AC-006.
4. **P4 Composite Rendering Model — consumed, not re-implemented (ADR-008 binding 2026-06-08)**: The HTML served by `GET /` is the `<article>` string produced by `render_slide_to_html()` from `slideforge-html` (STORY-046). This crate does NOT re-implement slide rendering. The P4 model (text as real HTML elements in an `<article style="position:relative">` container; graphical elements in a sibling `<svg aria-hidden="true">` layer; no `<canvas>`; no `<foreignObject>`) is enforced in `slideforge-html`. No `<canvas>` elements appear in the output of this server.
5. **Async runtime (ADR-021)**: Tokio is the exclusive async runtime for this crate.
   Use `tokio =1.52.3` with curated features (`rt-multi-thread,macros,net,time,sync,
   signal,fs,io-util`), NOT `features = ["full"]`. Graceful shutdown via
   `axum::serve(listener, app).with_graceful_shutdown(shutdown_signal())` where
   `shutdown_signal()` awaits `tokio::signal::ctrl_c()` and (cfg(unix))
   `tokio::signal::unix::signal(SignalKind::terminate())`.
6. **NFR-002 DEFERRED**: The <50ms incremental rebuild gate (NFR-002) is deferred to
   v1.x (see nfr-catalog NFR-002 for rationale). The v1.0 cold-build gate (NFR-001,
   <500ms for 25-slide deck) remains active and must be met.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `axum` | `=0.8.9` (note: 0.8.2 yanked) | HTTP server + built-in WebSocket (`axum::extract::ws::WebSocketUpgrade`); feature "ws"; path syntax `/{id}`; graceful shutdown via `axum::serve(...).with_graceful_shutdown(...)` |
| `tokio` | `=1.52.3` (ADR-021) | Async runtime — features: `rt-multi-thread,macros,net,time,sync,signal,fs,io-util` (NOT "full") |
| `tokio-tungstenite` | `=0.29.0` (DEV-DEP ONLY) | Test WebSocket client only — NOT a runtime dep; axum 0.8 built-in WebSocket replaces it at runtime |
| `serde` | `{ workspace = true }` = `=1.0.228` (ADR-022) | JSON message serialization |
| `serde_json` | `{ workspace = true }` = `=1.0.150` (ADR-022) | JSON encoding of WebSocket messages |
| `slideforge-html` | workspace | `render_slide_to_html()` for initial page and updates |
| `slideforge-types` | workspace | `LaidOutDeck`, `DiagnosticMessage` |
| `thiserror` | `{ workspace = true }` = `=2.0.18` (ADR-022) | `PreviewError` enum |

All workspace-pinned crates centralized in `[workspace.dependencies]` per ADR-022.
NFR-002 (<50ms incremental gate) is DEFERRED to v1.x — see nfr-catalog NFR-002.
NFR-001 (<500ms cold build, 25-slide deck) remains active for v1.0.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-preview/Cargo.toml` | Create | Crate manifest |
| `crates/slideforge-preview/src/lib.rs` | Create | Re-exports |
| `crates/slideforge-preview/src/server.rs` | Create | PreviewServer + axum router |
| `crates/slideforge-preview/src/ws_handler.rs` | Create | WebSocket upgrade handler |
| `crates/slideforge-preview/src/messages.rs` | Create | WebSocketMessage enum |
| `crates/slideforge-preview/src/debounce.rs` | Create | 100ms debounce |
| `crates/slideforge-preview/src/error.rs` | Create | PreviewError with thiserror |
| `crates/slideforge-preview/tests/integration.rs` | Create | Integration tests |
| `Cargo.toml` (workspace root) | Modify | Add slideforge-preview to members |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,800 |
| BC-4.03.004 | ~1,400 |
| slideforge-html render functions (STORY-046) | ~1,500 |
| axum 0.8.9 WebSocket API docs | ~2,000 |
| tokio broadcast channel docs | ~800 |
| Test files to write | ~2,500 |
| **Total** | **~11,000** |

Context budget: ~11% of a 100k-token context window. Within limit.

## Test Strategy

- **Unit tests**: `Debouncer` coalesces rapid events. `WebSocketMessage` serializes
  to correct JSON. `PreviewError::PortInUse` formats correct hint message.
- **Integration tests**: Full server lifecycle: start → connect → push_update →
  receive → shutdown. Port-in-use error. Debounce under rapid events.
- **No snapshot tests**: Server output is dynamic (timestamps, etc.); use
  structural assertions on JSON message shape instead.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Browser disconnects during watch | Server continues; reconnect sends full state (STORY-048) |
| EC-002 | File change triggers eval error | `push_error()` sends error JSON; prior slides remain |
| EC-003 | Port 3000 already in use | `Err(PreviewError::PortInUse)`; hint to use --port |
| EC-004 | .sf file deleted during watch | E-PAR-005 emitted; `push_error()`; server continues |
| EC-005 | 10 saves/second | 100ms debounce; at most 1 eval per 100ms |

## Forbidden Dependencies

`slideforge-preview` MUST NOT depend on (runtime):
- `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf` (no cross-exporter deps)
- Any file-writing crate for output purposes (only reads LaidOutDeck, writes to WebSocket)
- `warp` or `actix-web` (axum 0.8.9 is the canonical choice per ADR-008)
- `tokio-tungstenite` as a runtime (non-dev) dependency — axum 0.8 built-in WebSocket
  (`axum::extract::ws::WebSocketUpgrade`) is the server-side implementation; tungstenite
  is allowed ONLY as a dev-dependency for test clients (`=0.29.0`)
