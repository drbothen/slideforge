//! `PreviewServer` — the live development HTTP/WebSocket server.
//!
//! ## Architecture (BC-4.03.004 invariant 4)
//!
//! The server runs in a dedicated `tokio::spawn()` task so it never blocks the
//! file-watcher or evaluation pipeline. Communication with the server uses a
//! `tokio::sync::broadcast::Sender<String>` (JSON-encoded `WebSocketMessage`).
//!
//! ## Graceful shutdown (AC-005 / H-1 / H-2)
//!
//! Shutdown is injectable via `start_with_shutdown(port, deck, shutdown_fut)`.
//! `start(port, deck)` passes a `shutdown_signal()` future that awaits
//! `tokio::signal::ctrl_c()` and on Unix also
//! `tokio::signal::unix::signal(SignalKind::terminate())`.
//!
//! Connected WebSocket clients are notified via a `tokio::sync::watch::Sender<bool>`.
//! When the shutdown future resolves, the watch is set to `true`, which causes
//! every `handle_ws_connection` to `tokio::select!` into the cooperative shutdown
//! branch, send `Close(1001 Going Away)`, and return — allowing
//! `with_graceful_shutdown` to drain within 2 seconds (AC-005).
//!
//! ## No output files (BC-4.03.004 invariant 3)
//!
//! `PreviewServer` has no filesystem write calls. It receives pre-rendered
//! `SlideHtml` strings from the evaluation pipeline and forwards them over WebSocket.

use std::future::Future;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, watch};
use tokio::task::JoinHandle;

use crate::error::PreviewError;
use crate::messages::{DiagnosticMessage, SlideHtml, WebSocketMessage};
use crate::ws_handler::ws_upgrade_handler;

/// Capacity of the broadcast channel for WebSocket messages.
///
/// 16 is sufficient for development use — if a slow client falls behind by
/// more than 16 messages it receives `RecvError::Lagged` and recovers gracefully.
const BROADCAST_CAPACITY: usize = 16;

/// Shared application state for axum route handlers.
///
/// Holds the broadcast sender so that WebSocket handlers can subscribe to it,
/// the initial HTML to serve at `GET /`, and the shutdown watch receiver so
/// every WS connection can cooperatively close on server shutdown.
#[derive(Clone)]
struct AppState {
    /// Broadcast sender; WebSocket handlers call `.subscribe()` to receive messages.
    tx: broadcast::Sender<String>,
    /// Initial HTML page served at `GET /`.
    initial_html: Arc<String>,
    /// Shutdown watch receiver; when value becomes `true` WS handlers close with 1001.
    shutdown_rx: watch::Receiver<bool>,
}

/// Handle returned by [`PreviewServer::start`] for lifecycle management.
///
/// Wraps the `JoinHandle<()>` for the axum server task.
///
/// # AC-001 / AC-009
///
/// `start()` returns `Result<PreviewHandle, PreviewError>` so that port-in-use
/// errors can be surfaced before the server task is spawned.
#[derive(Debug)]
pub struct PreviewHandle {
    /// The join handle for the axum server task.
    pub join_handle: JoinHandle<()>,
    /// The actual port the server is listening on (useful for port=0 ephemeral tests).
    pub port: u16,
}

impl PreviewHandle {
    /// Abort the server task immediately.
    pub fn abort(&self) {
        self.join_handle.abort();
    }

    /// Returns the port the server is bound to.
    ///
    /// Useful when port 0 was requested and the OS assigned an ephemeral port.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }
}

/// The live development preview server.
///
/// Starts an axum HTTP/WebSocket server and exposes methods to push updates
/// to all connected browser clients.
///
/// # Usage
///
/// ```rust,no_run
/// # async fn example() -> Result<(), slideforge_preview::PreviewError> {
/// # let initial_deck = unimplemented!();
/// let server = slideforge_preview::PreviewServer::new();
/// let handle = server.start(3000, &initial_deck)?;
/// // Push updates:
/// server.push_update(vec!["<article>...</article>".to_string()]);
/// // Shutdown:
/// handle.abort();
/// # Ok(())
/// # }
/// ```
pub struct PreviewServer {
    /// Broadcast sender for dispatching WebSocket messages to all connected clients.
    tx: broadcast::Sender<String>,
}

impl PreviewServer {
    /// Create a new `PreviewServer`.
    ///
    /// Does not start the HTTP server — call [`start`][PreviewServer::start] to
    /// bind the port and begin serving.
    #[must_use]
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self { tx }
    }

    /// Start the axum server on `localhost:<port>` with OS-signal shutdown.
    ///
    /// Uses `shutdown_signal()` which awaits `Ctrl+C` / `SIGTERM`.
    /// For testable shutdown, use [`start_with_shutdown`][PreviewServer::start_with_shutdown].
    ///
    /// # Errors
    ///
    /// - [`PreviewError::PortInUse`] if the port is already bound.
    /// - [`PreviewError::Io`] for other I/O errors.
    pub fn start(
        &self,
        port: u16,
        initial_deck: &slideforge_layout::LaidOutDeck,
    ) -> Result<PreviewHandle, PreviewError> {
        self.start_with_shutdown(port, initial_deck, shutdown_signal())
    }

    /// Start the axum server with an injectable shutdown future (testable).
    ///
    /// # AC-005 / H-2
    ///
    /// Accepts any `Future<Output = ()>` as the shutdown signal. Tests inject
    /// a `tokio::sync::oneshot` receiver; production code passes `shutdown_signal()`.
    ///
    /// On shutdown:
    /// 1. The shutdown future resolves.
    /// 2. The `watch::Sender<bool>` is set to `true`.
    /// 3. Every connected WS handler receives the watch change, sends `Close(1001)`,
    ///    and returns — allowing `with_graceful_shutdown` to drain within 2 seconds.
    ///
    /// # Errors
    ///
    /// - [`PreviewError::PortInUse`] if the port is already bound.
    /// - [`PreviewError::Io`] for other I/O errors.
    pub fn start_with_shutdown(
        &self,
        port: u16,
        initial_deck: &slideforge_layout::LaidOutDeck,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> Result<PreviewHandle, PreviewError> {
        // Bind the TCP listener first to detect port-in-use before spawning the task.
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let std_listener = std::net::TcpListener::bind(addr).map_err(|e| {
            if e.kind() == ErrorKind::AddrInUse {
                PreviewError::PortInUse { port }
            } else {
                PreviewError::Io(e)
            }
        })?;

        // Determine the actual port (important when port=0 gives an ephemeral port).
        let actual_port = std_listener.local_addr().map_or(port, |a| a.port());

        // Set non-blocking mode required by tokio.
        std_listener
            .set_nonblocking(true)
            .map_err(PreviewError::Io)?;

        // Render the initial HTML page.
        let initial_html = build_initial_html(initial_deck);

        // Shutdown watch: WS handlers select! on this to send Close(1001) on shutdown.
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let state = AppState {
            tx: self.tx.clone(),
            initial_html: Arc::new(initial_html),
            shutdown_rx,
        };

        let join_handle = tokio::spawn(async move {
            // Convert std listener to tokio listener inside the task.
            let listener = match TcpListener::from_std(std_listener) {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("Failed to convert std listener to tokio: {e}");
                    return;
                },
            };

            let app = build_router(state);

            tracing::info!("Preview server listening on http://127.0.0.1:{actual_port}");

            // Wrap the shutdown future so we notify WS handlers before axum drains.
            let graceful_fut = async move {
                shutdown.await;
                tracing::info!("Preview server shutdown signal received");
                // Notify all WS handlers to send Close(1001) and return.
                // Ignore errors — if all receivers dropped, no clients are connected.
                let _ = shutdown_tx.send(true);
            };

            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(graceful_fut)
                .await
            {
                tracing::error!("Preview server error: {e}");
            }
        });

        Ok(PreviewHandle {
            join_handle,
            port: actual_port,
        })
    }

    /// Push a successful deck update to all connected WebSocket clients.
    ///
    /// Sends `{"type": "reload", "slides": [...]}` to every connected client.
    ///
    /// # AC-003 / AC-007 / BC-4.03.004 postcondition 3b
    ///
    /// This method is non-blocking. If no clients are connected, it returns
    /// immediately (`broadcast::Sender::send` returns `Err` when no receivers, but
    /// that is silently ignored — fire-and-forget semantics).
    pub fn push_update(&self, slides: Vec<SlideHtml>) {
        let msg = WebSocketMessage::Reload { slides };
        match msg.to_json() {
            Ok(json) => {
                // Ignore Err — means no receivers currently subscribed, which is fine.
                let _ = self.tx.send(json);
            },
            Err(e) => {
                tracing::error!("Failed to serialize reload message: {e}");
            },
        }
    }

    /// Push an evaluation error to all connected WebSocket clients.
    ///
    /// Sends `{"type": "error", "errors": [...]}` to every connected client.
    ///
    /// # AC-004 / BC-4.03.004 postcondition 3c
    ///
    /// This method is non-blocking. Prior slide content remains visible in the
    /// browser — this crate only sends the error message; the browser decides
    /// how to display it (error overlay).
    pub fn push_error(&self, errors: Vec<DiagnosticMessage>) {
        let msg = WebSocketMessage::Error { errors };
        match msg.to_json() {
            Ok(json) => {
                // Ignore Err — means no receivers currently subscribed, which is fine.
                let _ = self.tx.send(json);
            },
            Err(e) => {
                tracing::error!("Failed to serialize error message: {e}");
            },
        }
    }

    /// Subscribe to the broadcast channel.
    ///
    /// Returns a `Receiver` that receives all future broadcast messages.
    /// Used internally by the WebSocket handler (invoked from `live_handler`).
    /// Also used in unit tests to verify broadcast semantics.
    #[allow(dead_code)] // Used in tests and will be used by STORY-048 file-watcher wiring.
    pub(crate) fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

impl Default for PreviewServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the axum router with `GET /` and `GET /live` routes.
fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/live", get(live_handler))
        .with_state(state)
}

/// `GET /` — serves the initial HTML preview page.
///
/// # AC-001 / BC-4.03.004 postcondition 1
///
/// Returns 200 OK with `Content-Type: text/html` containing the deck preview.
async fn root_handler(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        [("Content-Type", "text/html; charset=utf-8")],
        Html((*state.initial_html).clone()),
    )
}

/// `GET /live` — upgrades to a WebSocket connection.
///
/// # AC-002 / BC-4.03.004 postcondition 2
async fn live_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let rx = state.tx.subscribe();
    ws_upgrade_handler(ws, rx, state.shutdown_rx).await
}

/// Build a minimal [`slideforge_types::Brand`] for use when rendering preview HTML.
///
/// The preview server renders slides with a default brand (no user brand configuration
/// is available at this level — that is the job of the evaluation pipeline).
fn make_preview_brand() -> slideforge_types::Brand {
    use slideforge_types::{Brand, BrandFonts, BrandPalette, SourceSpan};
    use std::sync::Arc;

    Brand {
        name: Arc::from("preview"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
            secondary: Arc::from("#0066CC"),
            accent: Arc::from("#FF6B35"),
            neutral: Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: Arc::from("Calibri"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
            // 457_200 EMU = 36pt body font (STORY-074 default; preview uses generic brand)
            font_size_emu: 457_200,
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

/// Build the initial HTML page for the deck preview.
///
/// # P4 Composite Rendering Model (ADR-008)
///
/// Uses `slideforge_html::render::render_slide_to_html()` to render each slide.
/// This crate does NOT re-implement slide rendering.
fn build_initial_html(deck: &slideforge_layout::LaidOutDeck) -> String {
    use slideforge_html::render::{HeadingLevel, render_slide_to_html};

    let brand = make_preview_brand();
    let slides_html: String = deck
        .slides
        .iter()
        .enumerate()
        .map(|(i, slide)| {
            let heading_level = if i == 0 {
                HeadingLevel::H1
            } else {
                HeadingLevel::H2
            };
            render_slide_to_html(slide, &brand, heading_level, &deck.page_size)
        })
        .collect();

    format!(
        r#"<!DOCTYPE html>
<html lang="en-US">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>slideforge preview</title>
<style>
  body {{ margin: 0; background: #1a1a1a; display: flex; flex-direction: column; align-items: center; padding: 2rem; gap: 2rem; }}
  .sf-slide {{ box-shadow: 0 4px 24px rgba(0,0,0,0.5); background: white; }}
</style>
</head>
<body>
<main id="sf-preview" aria-label="Deck preview">
{slides_html}
</main>
<script>
(function() {{
  var ws = new WebSocket("ws://" + location.host + "/live");
  ws.onmessage = function(evt) {{
    var msg = JSON.parse(evt.data);
    if (msg.type === "reload") {{
      document.getElementById("sf-preview").innerHTML = msg.slides.join("");
    }} else if (msg.type === "error") {{
      var overlay = document.getElementById("sf-error-overlay") || document.createElement("div");
      overlay.id = "sf-error-overlay";
      overlay.setAttribute("role", "alert");
      overlay.setAttribute("aria-live", "assertive");
      overlay.style.cssText = "position:fixed;top:0;left:0;right:0;background:#c0392b;color:#fff;padding:1rem;font-family:monospace;z-index:9999;white-space:pre-wrap;";
      overlay.textContent = msg.errors.map(function(e) {{ return e.file + ":" + e.line + ":" + e.col + " " + e.message; }}).join("\n");
      if (!document.getElementById("sf-error-overlay")) {{
        document.body.appendChild(overlay);
      }}
    }}
  }};
  ws.onopen = function() {{
    var overlay = document.getElementById("sf-error-overlay");
    if (overlay) overlay.remove();
  }};
}})();
</script>
</body>
</html>"#
    )
}

/// Graceful shutdown signal — awaits Ctrl+C (SIGINT) and on Unix also SIGTERM.
///
/// # AC-005 / BC-4.03.004 postcondition 4 / H-3
///
/// On signal registration failure (e.g., inside a sandbox), logs the error and
/// falls back to a never-resolving future so the server keeps running rather than
/// panicking. This is correct: a preview server that cannot register signals
/// should continue serving — it can be stopped by other means (process kill).
async fn shutdown_signal() {
    let ctrl_c = async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {},
            Err(e) => {
                tracing::error!(
                    "Failed to install Ctrl+C handler: {e} — server will not respond to Ctrl+C"
                );
                // Fall back to never resolving — keeps server alive.
                std::future::pending::<()>().await;
            },
        }
    };

    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let terminate = async {
            match signal(SignalKind::terminate()) {
                Ok(mut sig) => {
                    sig.recv().await;
                },
                Err(e) => {
                    tracing::error!(
                        "Failed to install SIGTERM handler: {e} — server will not respond to SIGTERM"
                    );
                    // Fall back to never resolving — keeps server alive.
                    std::future::pending::<()>().await;
                },
            }
        };

        tokio::select! {
            () = ctrl_c => {},
            () = terminate => {},
        }
    }

    #[cfg(not(unix))]
    ctrl_c.await;

    tracing::info!("Preview server shutdown signal received");
}

#[cfg(test)]
#[allow(non_snake_case)] // BC naming convention: test_BC_S_SS_NNN_xxx (CLAUDE.md)
#[allow(clippy::doc_markdown)] // prose references to method names in test docs
mod tests {
    use super::*;

    /// BC-4.03.004 invariant 4 — push_update is non-blocking when no clients connected.
    #[tokio::test]
    async fn test_BC_4_03_004_invariant_push_update_nonblocking_no_clients() {
        let server = PreviewServer::new();
        // No receivers subscribed — send should return immediately (fire-and-forget).
        server.push_update(vec!["<article>slide</article>".to_string()]);
        // If we reach here, push_update did not block.
    }

    /// BC-4.03.004 invariant 4 — push_error is non-blocking when no clients connected.
    #[tokio::test]
    async fn test_BC_4_03_004_invariant_push_error_nonblocking_no_clients() {
        let server = PreviewServer::new();
        server.push_error(vec![DiagnosticMessage {
            file: "deck.sf".to_string(),
            line: 1,
            col: 1,
            message: "test error".to_string(),
        }]);
        // If we reach here, push_error did not block.
    }

    /// AC-003 — push_update broadcasts reload JSON to subscribed receiver.
    #[tokio::test]
    async fn test_BC_4_03_004_push_update_broadcasts_to_subscriber() {
        let server = PreviewServer::new();
        let mut rx = server.subscribe();

        server.push_update(vec!["<article>slide1</article>".to_string()]);

        let json = rx.recv().await.expect("should receive message");
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(v["type"], "reload");
        assert!(v["slides"].is_array());
    }

    /// AC-004 — push_error broadcasts error JSON to subscribed receiver.
    #[tokio::test]
    async fn test_BC_4_03_004_push_error_broadcasts_to_subscriber() {
        let server = PreviewServer::new();
        let mut rx = server.subscribe();

        server.push_error(vec![DiagnosticMessage {
            file: "deck.sf".to_string(),
            line: 5,
            col: 3,
            message: "E-PAR-001".to_string(),
        }]);

        let json = rx.recv().await.expect("should receive message");
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(v["type"], "error");
        assert!(v["errors"].is_array());
    }

    /// AC-003 — push_update message contains type=reload and slides array.
    #[tokio::test]
    async fn test_BC_4_03_004_push_update_message_has_correct_type() {
        let server = PreviewServer::new();
        let mut rx = server.subscribe();

        server.push_update(vec!["<article>slide</article>".to_string()]);

        let json = rx.recv().await.expect("should receive message");
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(v["type"], "reload", "type field should be 'reload'");
        assert!(v["slides"].is_array(), "slides field should be array");
        assert_eq!(v["slides"][0], "<article>slide</article>");
    }

    /// AC-004 — push_error message contains type=error and errors array.
    #[tokio::test]
    async fn test_BC_4_03_004_push_error_message_has_correct_type() {
        let server = PreviewServer::new();
        let mut rx = server.subscribe();

        server.push_error(vec![DiagnosticMessage {
            file: "deck.sf".to_string(),
            line: 1,
            col: 1,
            message: "E-PAR-001: test".to_string(),
        }]);

        let json = rx.recv().await.expect("should receive message");
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(v["type"], "error", "type field should be 'error'");
        assert!(v["errors"].is_array(), "errors field should be array");
    }

    /// BC-4.03.004 invariant 3 — PreviewServer::start does not write any output files.
    ///
    /// Structural check: `PreviewServer` contains only a broadcast sender (no file handles).
    #[test]
    fn test_BC_4_03_004_invariant_no_file_output() {
        // Structural: PreviewServer contains only a broadcast::Sender<String>.
        // No file handles, no path writers. This is verified by reading the struct definition.
        // The production code path (push_update, push_error) only calls broadcast::Sender::send.
        // This test documents the invariant.
        let server = PreviewServer::new();
        // Verify the struct compiles without file-system fields.
        // The assertion is that `push_update` and `push_error` call `.tx.send()` — no fs writes.
        drop(server);
    }

    /// AC-007 — server runs in separate task (structural check: start() returns PreviewHandle).
    ///
    /// We only test that `start()` returns promptly; the returned handle wraps a `JoinHandle`.
    #[tokio::test]
    async fn test_BC_4_03_004_ac007_server_in_separate_task() {
        let server = PreviewServer::new();
        let deck = slideforge_layout::LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![],
            sections: vec![],
            warnings: vec![],
        };
        // Port 0 = ephemeral (OS assigns free port).
        let handle = server
            .start(0, &deck)
            .expect("start should succeed on port 0");
        // If start() returned, it spawned a task (non-blocking).
        assert!(handle.port > 0, "OS should have assigned a valid port");
        handle.abort();
    }

    /// AC-005 / BC-4.03.004 postcondition 4 — graceful shutdown with connected WS client
    /// completes within 2 seconds and client receives Close(1001 Going Away).
    ///
    /// This is the load-bearing test for AC-005 (H-2). Uses `start_with_shutdown`
    /// with an injected oneshot so we can trigger shutdown programmatically without
    /// sending an OS signal.
    #[tokio::test]
    async fn test_BC_4_03_004_ac005_graceful_shutdown_with_connected_client_completes_within_2s() {
        use futures_util::StreamExt;
        use tokio::sync::oneshot;
        use tokio_tungstenite::connect_async;

        let server = PreviewServer::new();
        let deck = slideforge_layout::LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![],
            sections: vec![],
            warnings: vec![],
        };

        // Inject a oneshot as the shutdown signal so tests control timing.
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let shutdown_fut = async move {
            // Ignore error — if receiver dropped, that's fine.
            let _ = shutdown_rx.await;
        };

        let handle = server
            .start_with_shutdown(0, &deck, shutdown_fut)
            .expect("start should succeed on port 0");

        let port = handle.port;

        // Connect a WebSocket client and wait until connected.
        let url = format!("ws://127.0.0.1:{port}/live");
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);
        let (mut ws, _) = loop {
            if let Ok(pair) = connect_async(&url).await {
                break pair;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "Could not connect WS client within 1s"
            );
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        };

        // Trigger graceful shutdown.
        let shutdown_start = std::time::Instant::now();
        let _ = shutdown_tx.send(());

        // Assert (a): server task completes within 2 seconds.
        let join_result =
            tokio::time::timeout(std::time::Duration::from_secs(2), handle.join_handle).await;
        let elapsed = shutdown_start.elapsed();

        assert!(
            join_result.is_ok(),
            "server task should complete within 2 seconds of graceful shutdown (took >{elapsed:?})"
        );

        // Assert (b): client received Close(1001 Going Away) as the last message.
        // Read from the WebSocket until we get a Close frame or the stream ends.
        // The server sends Close(1001) then closes the connection.
        let mut got_close_1001 = false;
        // Drain any pending messages; we expect a Close frame.
        while let Ok(Some(Ok(msg))) =
            tokio::time::timeout(std::time::Duration::from_millis(500), ws.next()).await
        {
            if let tokio_tungstenite::tungstenite::Message::Close(Some(frame)) = msg
                && frame.code
                    == tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Away
            {
                got_close_1001 = true;
                break;
            }
        }

        assert!(
            got_close_1001,
            "WebSocket client should receive Close(1001 Going Away) on graceful shutdown"
        );
    }
}
