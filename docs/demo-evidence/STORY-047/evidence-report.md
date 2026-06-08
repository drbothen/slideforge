# Demo Evidence Report — STORY-047

**Story:** STORY-047 — Web Preview Server: axum + WebSocket + SVG Canvas
**Crate:** `slideforge-preview`
**Branch:** `feature/STORY-047`
**Adversary Cascade:** CONVERGED 3/3 clean passes
**Recorded:** 2026-06-08
**Tool:** VHS 0.10.0 (terminal recording)

---

## Coverage Summary

| AC | Title | Recording | Coverage |
|----|-------|-----------|----------|
| AC-001 | HTTP server starts, GET / returns 200 OK with text/html + `<article>` | `AC-001-004-ws-lifecycle` | Full — success path: 3 integration tests pass |
| AC-002 | WebSocket endpoint accepts connections within 1s | `AC-001-004-ws-lifecycle` | Full — 2 integration tests pass (single + multiple clients) |
| AC-003 | `push_update()` delivers `{type:"reload", slides:[...]}` to all WS clients | `AC-001-004-ws-lifecycle` | Full — 3 integration tests pass (push, shape, broadcast) |
| AC-004 | `push_error()` delivers `{type:"error", errors:[...]}` to all WS clients | `AC-001-004-ws-lifecycle` | Full — 2 integration tests pass |
| AC-005 | Graceful shutdown <2s + WS Close(1001 Going Away) | `AC-005-graceful-shutdown` | Full — 2 tests pass (shutdown-with-connected-client + abort terminates) |
| AC-006 | No output files written to disk | `AC-001-004-ws-lifecycle` | Covered by integration test `ac006_no_output_files_written_during_watch` in full-suite recording |
| AC-007 | Server runs in separate task; push_update() non-blocking | `AC-001-004-ws-lifecycle` | Covered by `ac007_start_returns_before_client_connects` + `ac007_push_update_completes_immediately_no_clients` |
| AC-008 | Debounce: 10 events in 50ms → exactly 1 evaluation | `AC-008-debounce-coalescing` | Full — 4 debounce tests pass (unit: coalesce, no-event, single, spaced; integration: ac008) |
| AC-009 | PortInUse returns `Err(PreviewError::PortInUse)` with hint, no panic | `AC-009-port-in-use` | Full — 3 integration tests pass (error type, hint message, port number, no-panic) |

---

## Recordings

### AC-001-004: WS Lifecycle — HTTP Serve, Connect, push_update, push_error

**File:** `AC-001-004-ws-lifecycle.gif` / `.webm` / `.tape`

Runs 12 integration tests covering the complete WebSocket server lifecycle:
- `GET /` returns 200 OK with `Content-Type: text/html` containing `<article>` slide content (AC-001)
- WS client connects to `ws://localhost:<port>/live` within 1s; multiple clients can connect simultaneously (AC-002)
- `push_update()` broadcasts `{"type":"reload","slides":[...]}` to all clients within 500ms; slide content matches exactly (AC-003)
- `push_error()` broadcasts `{"type":"error","errors":[...]}` with all diagnostic fields; prior slides unchanged (AC-004)

Tests covered:
- `test_BC_4_03_004_ac001_http_get_root_returns_200`
- `test_BC_4_03_004_ac001_content_type_is_text_html`
- `test_BC_4_03_004_ac001_initial_html_contains_slide_content`
- `test_BC_4_03_004_ac002_websocket_connect_within_1s`
- `test_BC_4_03_004_ac002_multiple_clients_can_connect`
- `test_BC_4_03_004_ac003_push_update_client_receives_reload_json`
- `test_BC_4_03_004_ac003_reload_message_slides_match_pushed_content`
- `test_BC_4_03_004_ac003_all_clients_receive_reload`
- `test_BC_4_03_004_ac004_push_error_client_receives_error_json`
- `test_BC_4_03_004_ac004_error_message_diagnostic_fields_complete`
- `test_BC_4_03_004_ac006_no_output_files_written_during_watch`
- `test_BC_4_03_004_ac007_start_returns_before_client_connects`
- `test_BC_4_03_004_ac007_push_update_completes_immediately_no_clients`
- `test_BC_4_03_004_ec001_server_continues_after_client_disconnect`
- `test_BC_4_03_004_ec001_reconnect_after_disconnect`
- `test_BC_4_03_004_ec002_error_does_not_send_blank_slides`

---

### AC-005: Graceful Shutdown

**File:** `AC-005-graceful-shutdown.gif` / `.webm` / `.tape`

Demonstrates:
- Real WebSocket client connects to the running server
- Shutdown is triggered programmatically (simulating SIGTERM/Ctrl+C)
- Server task completes within 2 seconds
- `PreviewHandle::abort()` terminates the server task cleanly

Tests covered:
- `test_BC_4_03_004_ac005_graceful_shutdown_with_connected_client_completes_within_2s`
- `test_BC_4_03_004_ac005_handle_abort_task_terminates`

---

### AC-008: Debounce Coalescing

**File:** `AC-008-debounce-coalescing.gif` / `.webm` / `.tape`

Demonstrates:
- Unit: `Debouncer` fires once for a single event; fires 0 times with no events; fires once for 10 rapid events within window; fires N times for N events spaced beyond the window
- Integration: 10 file change events simulated in 50ms result in exactly 1 evaluation trigger (not 10)

Tests covered:
- `test_BC_4_03_004_debounce_10_events_coalesce_to_1`
- `test_BC_4_03_004_debounce_no_event_no_callback`
- `test_BC_4_03_004_debounce_single_event_triggers_callback`
- `test_BC_4_03_004_debounce_spaced_events_each_trigger`
- `test_BC_4_03_004_ac008_debounce_10_events_50ms_max_1_eval`

---

### AC-009: PortInUse Error Path

**File:** `AC-009-port-in-use.gif` / `.webm` / `.tape`

Demonstrates the error path: when a port is already bound, `PreviewServer::start()` returns `Err(PreviewError::PortInUse { port })`. The error message includes the port number and the hint "Use --port <N> to specify a different port." The server does not panic.

Tests covered:
- `test_BC_4_03_004_ac009_port_in_use_returns_port_in_use_error`
- `test_BC_4_03_004_ac009_port_in_use_error_message_contains_hint`
- `test_BC_4_03_004_ac009_port_in_use_error_message_contains_port_number`
- `test_BC_4_03_004_ac009_no_panic_on_port_in_use`

---

## Suite Totals

```
45 tests run: 45 passed, 0 skipped
```

All 45 tests pass in the full `slideforge-preview` nextest run. Every acceptance criterion (AC-001 through AC-009) has at least one load-bearing test recorded in demo evidence.

---

## Toolchain

| Tool | Version |
|------|---------|
| VHS | 0.10.0 |
| Font | FiraCode Nerd Font Mono |
| Theme | Dracula |
| Rust | stable (per rust-toolchain.toml) |
| cargo-nextest | latest |
