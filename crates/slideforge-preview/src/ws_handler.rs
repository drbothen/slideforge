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

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::response::Response;
use tokio::sync::broadcast;

/// axum handler for `GET /live` — upgrades the HTTP connection to WebSocket.
///
/// # AC-002 / BC-4.03.004 postcondition 2
///
/// The upgrade must complete within 1 second of server start. axum's
/// `WebSocketUpgrade::on_upgrade` is non-blocking and delegates to the async
/// runtime, so this constraint is trivially satisfied.
pub async fn ws_upgrade_handler(
    _ws: WebSocketUpgrade,
    _rx: broadcast::Receiver<String>,
) -> Response {
    todo!("implement ws_upgrade_handler: call ws.on_upgrade(|socket| handle_ws_connection(socket, rx))")
}

/// Drive a single WebSocket connection, forwarding broadcast messages to the client.
///
/// # Lifecycle
///
/// 1. Subscribe to the broadcast receiver.
/// 2. Loop: await the next broadcast message; forward it as a WebSocket Text frame.
/// 3. Exit when the client disconnects or the broadcast channel is closed.
#[allow(dead_code)]
async fn handle_ws_connection(_socket: WebSocket, _rx: broadcast::Receiver<String>) {
    todo!("implement handle_ws_connection: recv loop forwarding JSON text frames")
}
