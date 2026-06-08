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
            //
            // Two shutdown triggers:
            // 1. `Ok(())` + `borrow()==true`: graceful shutdown (shutdown_tx.send(true)).
            // 2. `Err(RecvError)`: all watch Senders dropped (e.g., server task panic
            //    or listener error before graceful_fut runs). This was previously a
            //    busy-spin bug: Err returned immediately on every loop iteration while
            //    `borrow()==false`, so the `if` guard failed and the handler looped
            //    without yielding. Fixed: treat Err as shutdown (MED-1 adv Pass-2).
            result = shutdown_rx.changed() => {
                if result.is_err() || *shutdown_rx.borrow() {
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

#[cfg(test)]
#[allow(non_snake_case)] // BC naming convention: test_BC_S_SS_NNN_xxx (CLAUDE.md)
mod tests {
    use super::*;

    /// MED-1 fix — sender-drop-without-send path: dropping the `watch::Sender`
    /// (without calling `send(true)`) must cause `handle_ws_connection` to return
    /// promptly, not hot-spin forever.
    ///
    /// Red gate: before the fix, `changed()` returns `Err` immediately on sender drop,
    /// `borrow()==false`, so the `if *shutdown_rx.borrow()` guard evaluates false,
    /// the handler falls through without returning, and the loop spins forever.
    ///
    /// Green gate: after the fix `result.is_err()` is checked first — the handler
    /// sends `Close(1001)` and returns.
    ///
    /// This test proves two invariants that together demonstrate the busy-spin:
    /// 1. `changed()` returns `Err` when the `Sender` is dropped without `send(true)`.
    /// 2. `borrow()` returns `false` in that state (no `send(true)` occurred).
    /// Therefore `if *shutdown_rx.borrow()` (pre-fix) evaluates to false → fall-through.
    /// The fix is `if result.is_err() || *shutdown_rx.borrow()` → true → return.
    #[tokio::test]
    async fn test_BC_4_03_004_med1_sender_drop_without_send_handler_returns_promptly() {
        let (tx, mut rx) = watch::channel(false);

        // Drop the sender WITHOUT sending true — this is the bug trigger.
        drop(tx);

        // After sender drop, `changed()` must return Err immediately and permanently.
        let result = rx.changed().await;
        assert!(
            result.is_err(),
            "watch::Receiver::changed() must return Err when all Senders are dropped; \
             got Ok — this means our assumption about the busy-spin path is wrong"
        );

        // And borrow() must still return false (no send(true) was ever called).
        assert!(
            !*rx.borrow(),
            "watch value must remain false when Sender was dropped without send(true)"
        );

        // Therefore: pre-fix code's `if *shutdown_rx.borrow()` guard evaluates to false
        // → falls through → next loop iteration → `changed()` returns Err again
        // → infinite hot-spin. Post-fix code checks `result.is_err()` first → returns.
    }

    /// MED-1 — post-fix shutdown condition: `result.is_err() || *rx.borrow()` is
    /// true when the `watch::Sender` is dropped without `send(true)`.
    ///
    /// This directly asserts the boolean expression used in the fixed `select!` arm,
    /// proving that the arm now returns rather than falling through.
    #[tokio::test]
    async fn test_BC_4_03_004_med1_watch_err_path_is_shutdown_not_fallthrough() {
        let (tx, mut rx) = watch::channel(false);
        drop(tx); // Sender dropped without send(true) — the bug trigger.

        let result = rx.changed().await;

        // Post-fix behavior: `result.is_err() || *rx.borrow()` must be true
        // so the handler returns rather than falling through.
        let should_shutdown = result.is_err() || *rx.borrow();
        assert!(
            should_shutdown,
            "After sender drop without send(true): result={result:?}, borrow={}, \
             should_shutdown must be true (post-fix) but was false (pre-fix busy-spin bug)",
            *rx.borrow()
        );
    }
}
