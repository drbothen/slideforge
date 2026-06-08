//! Integration tests for `slideforge-preview`.
//!
//! These tests exercise the full server lifecycle: start → connect → push →
//! receive → shutdown. Each test uses ephemeral port 0 so the OS assigns a
//! free port and tests can run in parallel without port conflicts.
//!
//! ## Test isolation (LESSON-21)
//!
//! - Each test binds port 0 and reads the assigned port from the listener.
//! - Tests are fully independent — no shared global state.
//! - Tokio runtime is created per test via `#[tokio::test]`.
//! - WebSocket client: `tokio-tungstenite =0.29.0` (dev-dep only).
//! - HTTP client: `reqwest` with rustls-tls.
//!
//! ## BC traceability
//!
//! All tests are named `test_BC_4_03_004_acNNN_*` to trace to BC-4.03.004
//! acceptance criteria.

use slideforge_preview::{DiagnosticMessage, PreviewError, PreviewServer};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build a minimal `LaidOutDeck` fixture for tests that need a starting deck.
///
/// Returns a deck with a single blank slide to satisfy `PreviewServer::start`.
fn minimal_laid_out_deck() -> slideforge_layout::LaidOutDeck {
    use slideforge_layout::{LaidOutDeck, PageSize};
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![],
        sections: vec![],
        warnings: vec![],
    }
}

// ── AC-001: HTTP server starts and serves initial preview ─────────────────────

/// AC-001 / BC-4.03.004 postcondition 1 — GET / returns 200 OK with HTML body.
///
/// Uses ephemeral port 0 and reqwest client to verify the HTTP response.
#[tokio::test]
async fn test_BC_4_03_004_ac001_http_get_root_returns_200() {
    todo!(
        "implement: start server on port 0, GET /, assert status 200 and body contains <!DOCTYPE html>"
    )
}

/// AC-001 — GET / Content-Type header is text/html.
#[tokio::test]
async fn test_BC_4_03_004_ac001_content_type_is_text_html() {
    todo!(
        "implement: start server on port 0, GET /, assert Content-Type contains text/html"
    )
}

/// AC-001 — initial HTML response contains SVG slide content from LaidOutDeck.
#[tokio::test]
async fn test_BC_4_03_004_ac001_initial_html_contains_slide_content() {
    todo!(
        "implement: start server with a deck containing slides, GET /, assert body contains <article"
    )
}

// ── AC-002: WebSocket endpoint accepts browser connections ───────────────────

/// AC-002 / BC-4.03.004 postcondition 2 — WS client connects to /live within 1 second.
///
/// Uses tokio-tungstenite as test-only WS client.
#[tokio::test]
async fn test_BC_4_03_004_ac002_websocket_connect_within_1s() {
    todo!(
        "implement: start server on port 0, connect tokio-tungstenite client to ws://localhost:<port>/live, assert connected within 1s"
    )
}

/// AC-002 — multiple clients can connect to /live simultaneously.
#[tokio::test]
async fn test_BC_4_03_004_ac002_multiple_clients_can_connect() {
    todo!(
        "implement: start server, connect 3 WS clients, assert all connected"
    )
}

// ── AC-003: reload message pushed on deck update ─────────────────────────────

/// AC-003 / BC-4.03.004 postcondition 3a,3b — push_update sends reload JSON within 500ms.
#[tokio::test]
async fn test_BC_4_03_004_ac003_push_update_client_receives_reload_json() {
    todo!(
        "implement: start server, connect WS client, call push_update([\"<article>...</article>\"]), \
         assert client receives {{\"type\":\"reload\",\"slides\":[...]}} within 500ms"
    )
}

/// AC-003 — reload message slides array matches the pushed slides.
#[tokio::test]
async fn test_BC_4_03_004_ac003_reload_message_slides_match_pushed_content() {
    todo!(
        "implement: push_update with specific slide HTML, assert JSON slides array == pushed slides"
    )
}

/// AC-003 — all connected clients receive the same reload message.
#[tokio::test]
async fn test_BC_4_03_004_ac003_all_clients_receive_reload() {
    todo!(
        "implement: connect 3 WS clients, push_update, assert all 3 receive identical reload JSON"
    )
}

// ── AC-004: error message pushed on evaluation failure ───────────────────────

/// AC-004 / BC-4.03.004 postcondition 3c — push_error sends error JSON.
#[tokio::test]
async fn test_BC_4_03_004_ac004_push_error_client_receives_error_json() {
    todo!(
        "implement: start server, connect WS client, call push_error([...]), \
         assert client receives {{\"type\":\"error\",\"errors\":[...]}} within 500ms"
    )
}

/// AC-004 — error message contains all diagnostic fields (file/line/col/message).
#[tokio::test]
async fn test_BC_4_03_004_ac004_error_message_diagnostic_fields_complete() {
    todo!(
        "implement: push_error with Diagnostic{{file:'deck.sf', line:5, col:3, message:'E-PAR-001'}}, \
         assert JSON contains all four fields"
    )
}

/// AC-004 — EC-002: error does not blank prior slide content (prior slides
/// remain visible — this tests the server does NOT send a blank-slides message
/// on error, just the error JSON).
#[tokio::test]
async fn test_BC_4_03_004_ec002_error_does_not_send_blank_slides() {
    todo!(
        "implement: push_update, then push_error, assert only one message received in second push_error call"
    )
}

// ── AC-005: graceful shutdown on SIGTERM / Ctrl+C ────────────────────────────

/// AC-005 / BC-4.03.004 postcondition 4 — PreviewHandle::abort causes task to complete.
///
/// Note: SIGTERM / Ctrl+C cannot be programmatically sent in tests; instead
/// we test graceful shutdown via the PreviewHandle::abort path and verify
/// the task completes (or was aborted) within 2 seconds.
#[tokio::test]
async fn test_BC_4_03_004_ac005_handle_abort_task_terminates() {
    todo!(
        "implement: start server, abort handle, await join_handle with 2s timeout, assert terminated"
    )
}

// ── AC-006: no output files written ─────────────────────────────────────────

/// AC-006 / BC-4.03.004 invariant 3 — no .pptx/.docx/.pdf/.html files created.
///
/// Runs server for 200ms, calls push_update and push_error, verifies the temp
/// dir has no output files.
#[tokio::test]
async fn test_BC_4_03_004_ac006_no_output_files_written_during_watch() {
    todo!(
        "implement: create tempdir, start server, push_update + push_error, assert no pptx/docx/pdf/html in tempdir"
    )
}

// ── AC-007: non-blocking server ──────────────────────────────────────────────

/// AC-007 / BC-4.03.004 invariant 4 — start() returns immediately (non-blocking).
#[tokio::test]
async fn test_BC_4_03_004_ac007_start_returns_before_client_connects() {
    todo!(
        "implement: measure time for start() to return, assert < 500ms"
    )
}

/// AC-007 — push_update completes in <10ms when no clients connected.
#[tokio::test]
async fn test_BC_4_03_004_ac007_push_update_completes_immediately_no_clients() {
    todo!(
        "implement: create PreviewServer (no start), push_update, assert returns in <10ms"
    )
}

// ── AC-008: 100ms debounce ───────────────────────────────────────────────────

/// AC-008 / BC-4.03.004 EC-005 — 10 file change events in 50ms trigger at most 1 eval.
#[tokio::test]
async fn test_BC_4_03_004_ac008_debounce_10_events_50ms_max_1_eval() {
    todo!(
        "implement: use Debouncer, fire 10 events within 50ms, wait 200ms, assert <= 1 callback"
    )
}

// ── AC-009: port-in-use error ────────────────────────────────────────────────

/// AC-009 / BC-4.03.004 EC-003 — port already bound returns Err(PortInUse).
#[tokio::test]
async fn test_BC_4_03_004_ac009_port_in_use_returns_port_in_use_error() {
    todo!(
        "implement: bind port with std::net::TcpListener, then call PreviewServer::start on same port, \
         assert Err(PreviewError::PortInUse {{ port }})"
    )
}

/// AC-009 — PortInUse error message contains the hint about --port flag.
#[tokio::test]
async fn test_BC_4_03_004_ac009_port_in_use_error_message_contains_hint() {
    todo!(
        "implement: construct PreviewError::PortInUse{{port:3000}}, assert Display contains '--port'"
    )
}

/// AC-009 — PortInUse error message contains the port number.
#[tokio::test]
async fn test_BC_4_03_004_ac009_port_in_use_error_message_contains_port_number() {
    todo!(
        "implement: construct PreviewError::PortInUse{{port:9876}}, assert Display contains '9876'"
    )
}

/// AC-009 — server does not panic on port-in-use (returns Err, no panic).
#[tokio::test]
async fn test_BC_4_03_004_ac009_no_panic_on_port_in_use() {
    todo!(
        "implement: bind a port, call start() on same port, assert no panic (just Err)"
    )
}

// ── EC-001: browser reconnects ───────────────────────────────────────────────

/// EC-001 — server continues running after a client disconnects.
#[tokio::test]
async fn test_BC_4_03_004_ec001_server_continues_after_client_disconnect() {
    todo!(
        "implement: connect WS client, drop client, push_update, assert no server crash"
    )
}

/// EC-001 — new client can connect after a previous client disconnected.
#[tokio::test]
async fn test_BC_4_03_004_ec001_reconnect_after_disconnect() {
    todo!(
        "implement: connect WS client A, drop A, connect new client B, assert B connected"
    )
}
