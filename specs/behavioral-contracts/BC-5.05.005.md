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
capability: CAP-027
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

# BC-5.05.005: WebSocket Connection Drop from Web Preview Reconnects Automatically

## Description

When the browser's WebSocket connection to the `slideforge watch` server is dropped
(network interruption, laptop sleep/wake, tab suspended, server restart), the browser-side
JavaScript client automatically attempts reconnection with exponential backoff. On
successful reconnect, the server sends the current full deck state so the browser is
immediately up-to-date. The user does not need to manually refresh the page.

## Preconditions

1. `slideforge watch` is running and the web preview is open in a browser.
2. The WebSocket connection between browser and server is dropped (any cause).

## Postconditions

1. The browser-side WebSocket client detects the disconnection within 5 seconds.
2. Reconnection is attempted with exponential backoff: 500ms, 1s, 2s, 4s, ... up to
   a maximum of 30s between attempts.
3. On successful reconnect, the server immediately sends a `{type: "full-state",
   slides: [...current state...]}` message.
4. The browser updates to the current deck state without a full page reload.
5. The browser shows a visible "Reconnecting..." indicator while disconnected.
6. After a configurable maximum reconnect time (default: 5 minutes), the browser shows
   a "Disconnected — refresh to reconnect" message and stops attempting.

## Invariants

1. Reconnection is handled entirely client-side (browser JS) — the axum server does
   not detect or react to disconnections beyond the WebSocket close event.
2. Each new WebSocket connection receives a full-state sync message — no assumption
   about prior state.
3. The backoff cap is 30 seconds — reconnection attempts never stop entirely until the
   5-minute maximum timeout.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Laptop sleep/wake (OS network suspension) | WebSocket close detected on wake; reconnect within 500ms; full-state sync |
| EC-002 | Server restarted (watch process restarted) | Browser detects close; reconnects when server is back; full-state sync |
| EC-003 | Multiple browser tabs; one tab loses connection | Only that tab reconnects; other tabs unaffected |
| EC-004 | Server sends deck update before browser finishes reconnect | Client receives update after reconnect via full-state sync; no missed updates |
| EC-005 | 5-minute maximum timeout reached | Browser shows "Disconnected" message; no further reconnect attempts |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Simulate WebSocket close event in browser | Browser shows "Reconnecting..."; reconnects within 500ms; full-state sync | happy-path |
| Server restarted while browser connected | Browser reconnects when server is back; preview updated | happy-path |
| 5+ minutes of server downtime | Browser shows "Disconnected — refresh" message | edge-case |
| Two tabs; one closes and reopens WebSocket | Only affected tab reconnects; other tab unchanged | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Reconnection attempt within 500ms of WebSocket close | integration test: Playwright + mock WebSocket close; measure time-to-reconnect-attempt |
| VP-TBD | Full-state sync message sent on reconnect | integration test: inspect WebSocket messages after reconnect |
| VP-TBD | Backoff does not exceed 30s between attempts | integration test: mock server down; measure reconnect intervals |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-027 ("Watch Mode with Live Data Refresh") per capabilities.md §CAP-027 |
| Capability Anchor Justification | CAP-027 ("Watch Mode with Live Data Refresh") per capabilities.md §CAP-027 — "push incremental updates to the web preview via websocket" implies reliable WebSocket connectivity including reconnection |
| L2 Domain Invariants | DI-017 (watch mode is always warn-only; reconnect does not change the evaluation mode) |
| Architecture Module | slideforge-preview browser JS client + axum WebSocket server (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.03.004 — depends on (WebSocket server is specified in BC-4.03.004; this BC specifies client reconnection)
- BC-5.05.001 — composes with (reconnect is part of the overall watch mode reliability contract)

## Architecture Anchors

- `architecture/export-architecture.md#web-preview` — WebSocket client reconnection logic
- `architecture/plugin-architecture.md#watch-mode` — watch mode reliability

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
