---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-4.03.004: Web Preview Served via axum+WebSocket+SVG Canvas; Updates on Save

## Description

`slideforge watch` starts a local axum HTTP server that serves a live web preview
of the deck. The preview renders slides as SVG on an HTML canvas. When a .sf source
file or data source changes, the pipeline re-evaluates and pushes an update to the
browser via a WebSocket connection. The browser applies the update without a full
page reload. This is the primary developer workflow for iterative deck authoring.

## Preconditions

1. `slideforge watch` is invoked in a directory with a valid .sf file.
2. A port is available for the local HTTP server (default: 3000, configurable).
3. The user's browser supports WebSocket and SVG.

## Postconditions

1. An HTTP server starts on `localhost:<port>` and serves the initial HTML+SVG preview.
2. A WebSocket endpoint at `ws://localhost:<port>/live` accepts browser connections.
3. When a watched .sf file changes on disk:
   a. The pipeline re-evaluates the deck (warn-only mode always active in watch).
   b. If evaluation succeeds: a JSON `{type: "reload", slides: [...]}` message is
      pushed via WebSocket. The browser updates slides without a full page reload.
   c. If evaluation fails: a JSON `{type: "error", errors: [...]}` message is pushed.
      The preview shows an error overlay with the diagnostic messages.
4. The server runs until Ctrl+C is received; `SIGTERM` triggers graceful shutdown.
5. The preview page is accessible at `localhost:<port>` without authentication.

## Invariants

1. Watch mode ALWAYS operates in warn-only mode (--warn-only is implied; no output
   files are written during watch). (BC-3.03.003 — error-slide placeholders)
2. The WebSocket message format is stable within v1.x (not a public API — internal use).
3. The preview updates slide content only — it does NOT write .pptx/.docx/.pdf output
   files. `slideforge build` is required for file output.
4. The axum server must not block the re-evaluation pipeline — they run concurrently.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Browser disconnects during watch | Server continues; reconnect sends full current state |
| EC-002 | File change triggers evaluation error | WebSocket pushes error JSON; preview shows error overlay; prior slides remain visible |
| EC-003 | Port 3000 already in use | Error: "Address in use" with hint to use --port <N>; process exits cleanly |
| EC-004 | .sf file deleted during watch | Watch mode emits E-PAR-005; error overlay shown; server remains running |
| EC-005 | Very rapid saves (10 saves/second) | Debounce: evaluate at most once per 100ms; no evaluation queue explosion |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slideforge watch deck.sf` | Server starts on :3000; browser shows initial preview | happy-path |
| Edit deck.sf title while watching | Browser updates title within 500ms of save; no reload | happy-path |
| Introduce syntax error in deck.sf | Browser shows error overlay with E-PAR-xxx message; prior content visible | error |
| Multiple browser tabs connected | All tabs receive the same WebSocket update simultaneously | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | WebSocket connection established within 1s of server start | integration test: connect ws client, assert connected |
| VP-TBD | Slide update pushed within 500ms of file change detection | integration test: modify file, measure time-to-WebSocket-message |
| VP-TBD | No output files written during watch | integration test: run watch 10s, assert no .pptx/.pdf in output dir |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 |
| Capability Anchor Justification | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 — "live web preview (axum + websocket + SVG canvas)" is verbatim from CAP-017 |
| L2 Domain Invariants | DI-017 (strict mode produces no output — watch mode is always warn-only) |
| Architecture Module | slideforge-preview crate or slideforge-cli watch subcommand (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.05.001 — depends on (watch mode file polling is specified in BC-5.05.001)
- BC-5.05.002 — composes with (HTTP data failure in watch mode shows error-slide; WebSocket delivers that error)
- BC-5.05.005 — composes with (WebSocket reconnection logic)
- BC-3.03.003 — depends on (warn-only mode error-slide rendering is used during watch)

## Architecture Anchors

- `architecture/export-architecture.md` — axum+WebSocket+SVG preview design
- `architecture/plugin-architecture.md` — file watcher + debounce + WebSocket pipeline

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
