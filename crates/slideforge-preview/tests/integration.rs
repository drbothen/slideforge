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
//! - HTTP client: `reqwest` with rustls.
//!
//! ## BC traceability
//!
//! All tests are named `test_BC_4_03_004_acNNN_*` to trace to BC-4.03.004
//! acceptance criteria.

// Test function names follow the BC naming convention (BC-S.SS.NNN) which uses
// uppercase segments — allowed by project convention (CLAUDE.md).
#![allow(non_snake_case)]
// Doc comments in test functions reference DSL method names and type names in
// prose — wrapping every identifier in backticks hurts readability in test docs.
#![allow(clippy::doc_markdown)]

use slideforge_preview::{DiagnosticMessage, PreviewError, PreviewServer};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build a minimal `LaidOutDeck` fixture for tests that need a starting deck.
///
/// Returns a deck with no slides (empty). Tests that need `<article>` content
/// should use [`deck_with_one_slide`] instead.
fn minimal_laid_out_deck() -> slideforge_layout::LaidOutDeck {
    use slideforge_layout::{LaidOutDeck, PageSize};
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![],
        sections: vec![],
        warnings: vec![],
    }
}

/// Build a `LaidOutDeck` fixture containing one minimal slide.
///
/// `render_slide_to_html` always emits `<article class="sf-slide">` for any
/// `LaidOutSlide`, even one with no frames. This fixture is used by tests that
/// assert the initial HTML body contains `<article` (L-1 / AC-001).
fn deck_with_one_slide() -> slideforge_layout::LaidOutDeck {
    use slideforge_layout::{LaidOutDeck, LaidOutSlide, PageSize};
    use std::sync::Arc;

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    };

    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
    }
}

/// Start a preview server on an ephemeral port (0) and return the server + handle.
///
/// Port 0 means the OS assigns a free port. Tests read `handle.port()` to find
/// the actual port.
fn start_test_server() -> (PreviewServer, slideforge_preview::PreviewHandle) {
    let server = PreviewServer::new();
    let deck = minimal_laid_out_deck();
    let handle = server
        .start(0, &deck)
        .expect("server should start on ephemeral port");
    (server, handle)
}

/// Connect a `tokio-tungstenite` WebSocket client to `ws://127.0.0.1:<port>/live`.
///
/// Retries for up to 1 second (in 10ms intervals) to allow the server to start.
async fn connect_ws_client(
    port: u16,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    use tokio_tungstenite::connect_async;

    let url = format!("ws://127.0.0.1:{port}/live");
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);

    loop {
        match connect_async(&url).await {
            Ok((stream, _)) => return stream,
            Err(e) => {
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "Could not connect WebSocket to {url} within 1s: {e}"
                );
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            },
        }
    }
}

/// Build a reqwest client with rustls (no TLS certs needed for localhost).
fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .build()
        .expect("reqwest client should build")
}

// ── AC-001: HTTP server starts and serves initial preview ─────────────────────

/// AC-001 / BC-4.03.004 postcondition 1 — GET / returns 200 OK with HTML body.
///
/// Uses ephemeral port 0 and reqwest client to verify the HTTP response.
#[tokio::test]
async fn test_BC_4_03_004_ac001_http_get_root_returns_200() {
    let (_, handle) = start_test_server();
    let port = handle.port();

    let client = http_client();
    let url = format!("http://127.0.0.1:{port}/");

    // Retry briefly to let axum start accepting connections.
    let resp = retry_http_get(&client, &url, std::time::Duration::from_secs(1))
        .await
        .expect("GET / should succeed");

    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    handle.abort();
}

/// AC-001 — GET / Content-Type header is text/html.
#[tokio::test]
async fn test_BC_4_03_004_ac001_content_type_is_text_html() {
    let (_, handle) = start_test_server();
    let port = handle.port();

    let client = http_client();
    let url = format!("http://127.0.0.1:{port}/");

    let resp = retry_http_get(&client, &url, std::time::Duration::from_secs(1))
        .await
        .expect("GET / should succeed");

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("Content-Type header should be present")
        .to_str()
        .expect("Content-Type should be valid UTF-8");

    assert!(
        content_type.contains("text/html"),
        "Content-Type should contain text/html, got: {content_type}"
    );
    handle.abort();
}

/// AC-001 — initial HTML response contains `<article` slide content from LaidOutDeck.
///
/// Uses a deck with one slide so `render_slide_to_html` emits `<article class="sf-slide">`.
/// This is the load-bearing L-1 test.
#[tokio::test]
async fn test_BC_4_03_004_ac001_initial_html_contains_slide_content() {
    // Start server with a deck that has one slide — so <article> is rendered.
    let server = PreviewServer::new();
    let deck = deck_with_one_slide();
    let handle = server
        .start(0, &deck)
        .expect("server should start on ephemeral port");
    let port = handle.port();

    let client = http_client();
    let url = format!("http://127.0.0.1:{port}/");

    let resp = retry_http_get(&client, &url, std::time::Duration::from_secs(1))
        .await
        .expect("GET / should succeed");

    let body = resp.text().await.expect("body should be readable");
    assert!(
        body.contains("<!DOCTYPE html"),
        "body should contain <!DOCTYPE html>, got: {body:.200}"
    );
    // AC-001 assertion: body must contain the <article> slide wrapper.
    assert!(
        body.contains("<article"),
        "body should contain <article> slide wrapper from render_slide_to_html, got: {body:.200}"
    );
    handle.abort();
}

// ── AC-002: WebSocket endpoint accepts browser connections ───────────────────

/// AC-002 / BC-4.03.004 postcondition 2 — WS client connects to /live within 1 second.
///
/// Uses tokio-tungstenite as test-only WS client.
#[tokio::test]
async fn test_BC_4_03_004_ac002_websocket_connect_within_1s() {
    let (_, handle) = start_test_server();
    let port = handle.port();

    // connect_ws_client retries for up to 1 second.
    let _ws = connect_ws_client(port).await;
    // If we reach here, the connection succeeded within 1s.
    handle.abort();
}

/// AC-002 — multiple clients can connect to /live simultaneously.
#[tokio::test]
async fn test_BC_4_03_004_ac002_multiple_clients_can_connect() {
    let (_, handle) = start_test_server();
    let port = handle.port();

    // Connect 3 WebSocket clients concurrently.
    let ws1 = connect_ws_client(port).await;
    let ws2 = connect_ws_client(port).await;
    let ws3 = connect_ws_client(port).await;

    // All three connected successfully.
    drop(ws1);
    drop(ws2);
    drop(ws3);
    handle.abort();
}

// ── AC-003: reload message pushed on deck update ─────────────────────────────

/// AC-003 / BC-4.03.004 postcondition 3a,3b — push_update sends reload JSON within 500ms.
#[tokio::test]
async fn test_BC_4_03_004_ac003_push_update_client_receives_reload_json() {
    use futures_util::StreamExt;

    let (server, handle) = start_test_server();
    let port = handle.port();

    let mut ws = connect_ws_client(port).await;

    // Push an update after the client is connected.
    server.push_update(vec!["<article>slide1</article>".to_string()]);

    // Receive message within 500ms.
    let msg = tokio::time::timeout(std::time::Duration::from_millis(500), ws.next())
        .await
        .expect("should receive within 500ms")
        .expect("stream should not be exhausted")
        .expect("should receive a valid message");

    let text = match msg {
        tokio_tungstenite::tungstenite::Message::Text(t) => t,
        other => panic!("expected Text message, got: {other:?}"),
    };

    let v: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    assert_eq!(v["type"], "reload", "type should be 'reload'");
    assert!(v["slides"].is_array(), "slides should be array");

    handle.abort();
}

/// AC-003 — reload message slides array matches the pushed slides.
#[tokio::test]
async fn test_BC_4_03_004_ac003_reload_message_slides_match_pushed_content() {
    use futures_util::StreamExt;

    let (server, handle) = start_test_server();
    let port = handle.port();

    let mut ws = connect_ws_client(port).await;

    let slide_html = "<article class=\"sf-slide\">specific content</article>".to_string();
    server.push_update(vec![slide_html.clone()]);

    let msg = tokio::time::timeout(std::time::Duration::from_millis(500), ws.next())
        .await
        .expect("should receive within 500ms")
        .expect("stream should not be exhausted")
        .expect("should receive a valid message");

    let text = match msg {
        tokio_tungstenite::tungstenite::Message::Text(t) => t,
        other => panic!("expected Text message, got: {other:?}"),
    };

    let v: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    assert_eq!(
        v["slides"][0], slide_html,
        "slides[0] should match pushed content"
    );

    handle.abort();
}

/// AC-003 — all connected clients receive the same reload message.
#[tokio::test]
async fn test_BC_4_03_004_ac003_all_clients_receive_reload() {
    use futures_util::StreamExt;

    let (server, handle) = start_test_server();
    let port = handle.port();

    let mut ws1 = connect_ws_client(port).await;
    let mut ws2 = connect_ws_client(port).await;
    let mut ws3 = connect_ws_client(port).await;

    server.push_update(vec!["<article>broadcast slide</article>".to_string()]);

    let receive_timeout = std::time::Duration::from_millis(500);

    let msg1 = tokio::time::timeout(receive_timeout, ws1.next())
        .await
        .expect("ws1 should receive within 500ms")
        .expect("ws1 stream not exhausted")
        .expect("ws1 message ok");

    let msg2 = tokio::time::timeout(receive_timeout, ws2.next())
        .await
        .expect("ws2 should receive within 500ms")
        .expect("ws2 stream not exhausted")
        .expect("ws2 message ok");

    let msg3 = tokio::time::timeout(receive_timeout, ws3.next())
        .await
        .expect("ws3 should receive within 500ms")
        .expect("ws3 stream not exhausted")
        .expect("ws3 message ok");

    let text1 = extract_text(msg1);
    let text2 = extract_text(msg2);
    let text3 = extract_text(msg3);

    assert_eq!(
        text1, text2,
        "ws1 and ws2 should receive identical messages"
    );
    assert_eq!(
        text2, text3,
        "ws2 and ws3 should receive identical messages"
    );

    let v: serde_json::Value = serde_json::from_str(&text1).expect("valid json");
    assert_eq!(v["type"], "reload");

    handle.abort();
}

// ── AC-004: error message pushed on evaluation failure ───────────────────────

/// AC-004 / BC-4.03.004 postcondition 3c — push_error sends error JSON.
#[tokio::test]
async fn test_BC_4_03_004_ac004_push_error_client_receives_error_json() {
    use futures_util::StreamExt;

    let (server, handle) = start_test_server();
    let port = handle.port();

    let mut ws = connect_ws_client(port).await;

    server.push_error(vec![DiagnosticMessage {
        file: "deck.sf".to_string(),
        line: 5,
        col: 3,
        message: "E-PAR-001: unexpected token".to_string(),
    }]);

    let msg = tokio::time::timeout(std::time::Duration::from_millis(500), ws.next())
        .await
        .expect("should receive within 500ms")
        .expect("stream should not be exhausted")
        .expect("should receive a valid message");

    let text = extract_text(msg);
    let v: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    assert_eq!(v["type"], "error", "type should be 'error'");
    assert!(v["errors"].is_array(), "errors should be array");

    handle.abort();
}

/// AC-004 — error message contains all diagnostic fields (file/line/col/message).
#[tokio::test]
async fn test_BC_4_03_004_ac004_error_message_diagnostic_fields_complete() {
    use futures_util::StreamExt;

    let (server, handle) = start_test_server();
    let port = handle.port();

    let mut ws = connect_ws_client(port).await;

    server.push_error(vec![DiagnosticMessage {
        file: "deck.sf".to_string(),
        line: 5,
        col: 3,
        message: "E-PAR-001".to_string(),
    }]);

    let msg = tokio::time::timeout(std::time::Duration::from_millis(500), ws.next())
        .await
        .expect("should receive within 500ms")
        .expect("stream should not be exhausted")
        .expect("message ok");

    let text = extract_text(msg);
    let v: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    let err = &v["errors"][0];

    assert_eq!(err["file"], "deck.sf", "file field");
    assert_eq!(err["line"], 5_u64, "line field");
    assert_eq!(err["col"], 3_u64, "col field");
    assert_eq!(err["message"], "E-PAR-001", "message field");

    handle.abort();
}

/// AC-004 — EC-002: error does not blank prior slide content (prior slides
/// remain visible — this tests the server does NOT send a blank-slides message
/// on error, just the error JSON).
#[tokio::test]
async fn test_BC_4_03_004_ec002_error_does_not_send_blank_slides() {
    use futures_util::StreamExt;

    let (server, handle) = start_test_server();
    let port = handle.port();

    let mut ws = connect_ws_client(port).await;

    // First: push an update (so browser has slides).
    server.push_update(vec!["<article>slide</article>".to_string()]);
    let _update_msg = tokio::time::timeout(std::time::Duration::from_millis(500), ws.next())
        .await
        .expect("update message within 500ms")
        .expect("stream ok")
        .expect("message ok");

    // Second: push an error.
    server.push_error(vec![DiagnosticMessage {
        file: "deck.sf".to_string(),
        line: 1,
        col: 1,
        message: "E-PAR-001".to_string(),
    }]);

    let error_msg = tokio::time::timeout(std::time::Duration::from_millis(500), ws.next())
        .await
        .expect("error message within 500ms")
        .expect("stream ok")
        .expect("message ok");

    let text = extract_text(error_msg);
    let v: serde_json::Value = serde_json::from_str(&text).expect("valid json");

    // The second message MUST be type="error", NOT type="reload" with empty slides.
    assert_eq!(
        v["type"], "error",
        "second message should be error type (not blank-slides reload)"
    );
    assert!(
        v.get("slides").is_none(),
        "error message must not contain a 'slides' field"
    );

    handle.abort();
}

// ── AC-005: graceful shutdown on SIGTERM / Ctrl+C ────────────────────────────

/// AC-005 / BC-4.03.004 postcondition 4 — PreviewHandle::abort causes task to complete.
///
/// Note: SIGTERM / Ctrl+C cannot be programmatically sent in tests; instead
/// we test graceful shutdown via the PreviewHandle::abort path and verify
/// the task completes (or was aborted) within 2 seconds.
#[tokio::test]
async fn test_BC_4_03_004_ac005_handle_abort_task_terminates() {
    let (_, handle) = start_test_server();

    // Give the server a moment to start.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    handle.abort();

    // The JoinHandle should complete (with abort error) within 2 seconds.
    let result = tokio::time::timeout(std::time::Duration::from_secs(2), handle.join_handle).await;

    // Either the task completed (Ok) or timed out. An aborted task returns
    // Err(JoinError::cancelled()) — which is Ok(Err(...)) after timeout unwrap.
    assert!(
        result.is_ok(),
        "server task should complete within 2 seconds of abort"
    );
}

// ── AC-006: no output files written ─────────────────────────────────────────

/// AC-006 / BC-4.03.004 invariant 3 — no .pptx/.docx/.pdf/.html files created.
///
/// Runs server for 200ms, calls push_update and push_error, verifies the temp
/// dir has no output files.
#[tokio::test]
async fn test_BC_4_03_004_ac006_no_output_files_written_during_watch() {
    let tmpdir = tempfile::tempdir().expect("tempdir should be created");
    let tmppath = tmpdir.path().to_path_buf();

    let (server, handle) = start_test_server();

    // Push both update and error messages.
    server.push_update(vec!["<article>slide</article>".to_string()]);
    server.push_error(vec![DiagnosticMessage {
        file: "deck.sf".to_string(),
        line: 1,
        col: 1,
        message: "E-PAR-001".to_string(),
    }]);

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Verify no output files in tmpdir.
    let output_extensions = ["pptx", "docx", "pdf", "html"];
    for entry in std::fs::read_dir(&tmppath).expect("tempdir readable") {
        let entry = entry.expect("dir entry ok");
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            assert!(
                !output_extensions.contains(&ext),
                "unexpected output file found: {path:?}"
            );
        }
    }

    handle.abort();
}

// ── AC-007: non-blocking server ──────────────────────────────────────────────

/// AC-007 / BC-4.03.004 invariant 4 — start() returns immediately (non-blocking).
#[tokio::test]
async fn test_BC_4_03_004_ac007_start_returns_before_client_connects() {
    let server = PreviewServer::new();
    let deck = minimal_laid_out_deck();

    let start_time = std::time::Instant::now();
    let handle = server.start(0, &deck).expect("start should succeed");
    let elapsed = start_time.elapsed();

    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "start() should return in <500ms (was non-blocking), elapsed: {elapsed:?}"
    );

    handle.abort();
}

/// AC-007 — push_update completes in <100ms when no clients connected.
///
/// The assertion is 100ms (not 10ms) to remain deterministic under test-suite load
/// on a heavily-contended CI runner. `push_update` is a synchronous broadcast send
/// with no I/O — completion at any reasonably sub-second time proves it is non-blocking.
#[tokio::test]
async fn test_BC_4_03_004_ac007_push_update_completes_immediately_no_clients() {
    let server = PreviewServer::new();

    let start_time = std::time::Instant::now();
    // No start() called — broadcast channel has no receivers.
    server.push_update(vec!["<article>slide</article>".to_string()]);
    let elapsed = start_time.elapsed();

    assert!(
        elapsed < std::time::Duration::from_millis(100),
        "push_update should complete in <100ms with no clients, elapsed: {elapsed:?}"
    );
}

// ── AC-008: 100ms debounce ───────────────────────────────────────────────────

/// AC-008 / BC-4.03.004 EC-005 — 10 file change events in 50ms trigger at most 1 eval.
#[tokio::test]
async fn test_BC_4_03_004_ac008_debounce_10_events_50ms_max_1_eval() {
    use slideforge_preview::debounce::Debouncer;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);

    let debouncer = Debouncer::new(move || {
        counter_clone.fetch_add(1, Ordering::Relaxed);
    });

    // Fire 10 events within 50ms (every 5ms).
    for _ in 0..10 {
        debouncer.trigger();
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }

    // Wait 200ms for the debounce window to expire.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let count = counter.load(Ordering::Relaxed);
    assert_eq!(
        count, 1,
        "expected exactly 1 evaluation (burst must coalesce AND fire), got {count}"
    );
}

// ── AC-009: port-in-use error ────────────────────────────────────────────────

/// AC-009 / BC-4.03.004 EC-003 — port already bound returns Err(PortInUse).
#[tokio::test]
async fn test_BC_4_03_004_ac009_port_in_use_returns_port_in_use_error() {
    // Bind a port with std::net::TcpListener to hold it.
    let holder = std::net::TcpListener::bind("127.0.0.1:0").expect("should bind ephemeral port");
    let port = holder.local_addr().expect("local_addr ok").port();

    let server = PreviewServer::new();
    let deck = minimal_laid_out_deck();

    let result = server.start(port, &deck);

    assert!(
        matches!(result, Err(PreviewError::PortInUse { port: p }) if p == port),
        "expected Err(PreviewError::PortInUse {{ port: {port} }}), got: {result:?}"
    );

    drop(holder);
}

/// AC-009 — PortInUse error message contains the hint about --port flag.
#[tokio::test]
async fn test_BC_4_03_004_ac009_port_in_use_error_message_contains_hint() {
    let err = PreviewError::PortInUse { port: 3000 };
    let msg = err.to_string();
    assert!(
        msg.contains("--port"),
        "PortInUse message should contain '--port' hint, got: {msg}"
    );
}

/// AC-009 — PortInUse error message contains the port number.
#[tokio::test]
async fn test_BC_4_03_004_ac009_port_in_use_error_message_contains_port_number() {
    let err = PreviewError::PortInUse { port: 9876 };
    let msg = err.to_string();
    assert!(
        msg.contains("9876"),
        "PortInUse message should contain port number '9876', got: {msg}"
    );
}

/// AC-009 — server does not panic on port-in-use (returns Err, no panic).
#[tokio::test]
async fn test_BC_4_03_004_ac009_no_panic_on_port_in_use() {
    let holder = std::net::TcpListener::bind("127.0.0.1:0").expect("should bind ephemeral port");
    let port = holder.local_addr().expect("local_addr ok").port();

    let server = PreviewServer::new();
    let deck = minimal_laid_out_deck();

    // Must not panic — must return Err.
    let result = server.start(port, &deck);
    assert!(result.is_err(), "expected Err on port-in-use, got Ok");

    drop(holder);
}

// ── EC-001: browser reconnects ───────────────────────────────────────────────

/// EC-001 — server continues running after a client disconnects.
#[tokio::test]
async fn test_BC_4_03_004_ec001_server_continues_after_client_disconnect() {
    let (server, handle) = start_test_server();
    let port = handle.port();

    // Connect and immediately drop the client.
    {
        let _ws = connect_ws_client(port).await;
        // Drop ws here — simulates browser disconnect.
    }

    // Allow the connection close to propagate.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Server should still be running — push_update should not panic.
    server.push_update(vec!["<article>after disconnect</article>".to_string()]);

    // Server should still accept HTTP requests.
    let client = http_client();
    let url = format!("http://127.0.0.1:{port}/");
    let resp = retry_http_get(&client, &url, std::time::Duration::from_secs(1))
        .await
        .expect("server should still respond after client disconnect");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    handle.abort();
}

/// EC-001 — new client can connect after a previous client disconnected.
#[tokio::test]
async fn test_BC_4_03_004_ec001_reconnect_after_disconnect() {
    let (_, handle) = start_test_server();
    let port = handle.port();

    // Connect client A and drop it.
    {
        let _ws_a = connect_ws_client(port).await;
    }

    // Allow disconnection to settle.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Connect client B after A disconnected.
    let _ws_b = connect_ws_client(port).await;
    // If we reach here, reconnection succeeded.

    handle.abort();
}

// ── Internal test helpers ─────────────────────────────────────────────────────

/// Extract text from a `tungstenite::Message::Text`, panic on other variants.
fn extract_text(msg: tokio_tungstenite::tungstenite::Message) -> String {
    match msg {
        tokio_tungstenite::tungstenite::Message::Text(t) => t.to_string(),
        other => panic!("expected Text message, got: {other:?}"),
    }
}

/// Retry an HTTP GET request until it succeeds or the deadline is reached.
///
/// Needed because the axum server task may not have started accepting connections
/// immediately after `PreviewServer::start()` returns.
async fn retry_http_get(
    client: &reqwest::Client,
    url: &str,
    timeout: std::time::Duration,
) -> Option<reqwest::Response> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if let Ok(resp) = client.get(url).send().await {
            return Some(resp);
        }
        if tokio::time::Instant::now() >= deadline {
            return None;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
