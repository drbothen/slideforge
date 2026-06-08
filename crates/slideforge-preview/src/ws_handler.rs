//! WebSocket upgrade handler for the `/live` endpoint.
//!
//! This module handles the axum WebSocket upgrade for `GET /live` and the
//! per-connection message loop that subscribes to the broadcast channel and
//! forwards messages to the browser client.
//!
//! ## axum 0.8 WebSocket API
//!
//! axum 0.8 provides built-in WebSocket support via
//! `axum::extract::ws::WebSocketUpgrade`. `tokio-tungstenite` is NOT used at
//! runtime — it is a dev-dependency for test clients only (STORY-047 task list).
//!
//! ## Broadcast channel
//!
//! The handler receives a `tokio::sync::broadcast::Receiver<String>` (JSON
//! encoded `WebSocketMessage`). When a message arrives it is forwarded as a
//! WebSocket text frame. Lagged receivers (slow clients) receive
//! `RecvError::Lagged` which is logged and recovered gracefully.
//!
//! ## Cooperative shutdown (AC-005 / H-1 fix)
//!
//! Each WS handler also receives a `tokio::sync::watch::Receiver<bool>`.
//! When the watch value becomes `true`, the handler sends a Close frame with
//! code 1001 (Going Away) and returns — allowing
//! `axum::serve(...).with_graceful_shutdown(...)` to drain within 2 seconds.

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use tokio::sync::{broadcast, watch};

/// axum handler for `GET /live` — upgrades the HTTP connection to WebSocket.
///
/// # AC-002 / BC-4.03.004 postcondition 2
///
/// The upgrade must complete within 1 second of server start. axum's
/// `WebSocketUpgrade::on_upgrade` is non-blocking and delegates to the async
/// runtime, so this constraint is trivially satisfied.
pub async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    rx: broadcast::Receiver<String>,
    shutdown_rx: watch::Receiver<bool>,
) -> Response {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, rx, shutdown_rx))
}

/// Drive a single WebSocket connection, forwarding broadcast messages to the client.
///
/// # Lifecycle
///
/// 1. Subscribe to the broadcast receiver.
/// 2. Loop: await the next broadcast message OR a shutdown notification.
/// 3. On shutdown: send `Close(1001 Going Away)` and return.
/// 4. Exit when the client disconnects or the broadcast channel is closed.
///
/// # AC-005 / H-1
///
/// The `shutdown_rx` watch receiver provides a cooperative shutdown path.
/// When the watch value becomes `true`, we send Close(1001) and return,
/// allowing `with_graceful_shutdown` to complete within 2 seconds.
async fn handle_ws_connection(
    mut socket: WebSocket,
    mut rx: broadcast::Receiver<String>,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            // Cooperative shutdown path (AC-005): send Close(1001) and return.
            result = shutdown_rx.changed() => {
                // `changed()` returns Ok when the value has changed (or Err if sender dropped).
                // Either way, we should shut down.
                let _ = result;
                if *shutdown_rx.borrow() {
                    tracing::debug!("WebSocket handler received shutdown signal — sending Close(1001)");
                    let close_frame = CloseFrame {
                        code: axum::extract::ws::close_code::AWAY,
                        reason: axum::extract::ws::Utf8Bytes::from_static("server shutting down"),
                    };
                    // Best-effort close — if send fails the client already disconnected.
                    let _ = socket.send(Message::Close(Some(close_frame))).await;
                    return;
                }
            }

            // Normal message forwarding path.
            recv_result = rx.recv() => {
                match recv_result {
                    Ok(json) => {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            // Client disconnected — stop forwarding.
                            tracing::debug!("WebSocket client disconnected during send");
                            return;
                        }
                    },
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        // Client is slow — log and continue; do not disconnect.
                        tracing::warn!(
                            skipped,
                            "WebSocket client lagged behind broadcast channel; skipped messages"
                        );
                    },
                    Err(broadcast::error::RecvError::Closed) => {
                        // Broadcast channel closed (server shutting down).
                        tracing::debug!("Broadcast channel closed — closing WebSocket connection");
                        return;
                    },
                }
            }
        }
    }
}
