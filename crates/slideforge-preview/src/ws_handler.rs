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

/// Decide whether the WebSocket handler should shut down based on the result of
/// `shutdown_rx.changed()` and the current watch value.
///
/// This is the single source of truth for "should this WS handler stop":
///
/// | `changed` result | `current_value` | `should_shutdown` |
/// |------------------|-----------------|-----------------|
/// | `Ok(())`         | `true`           | `true`  — graceful shutdown signal sent  |
/// | `Ok(())`         | `false`          | `false` — spurious wake or pre-signal    |
/// | `Err(_)`         | any              | `true`  — all Senders dropped (e.g., server task panic); previously caused a busy-spin |
///
/// The `Err` case was the MED-1 bug: `changed()` returns `Err` immediately and
/// permanently when the last `watch::Sender` is dropped, so if only `current_value`
/// were checked the loop would spin without yielding.
fn should_shutdown(changed: &Result<(), watch::error::RecvError>, current_value: bool) -> bool {
    changed.is_err() || current_value
}

/// axum handler for `GET /live` — upgrades the HTTP connection to WebSocket.
///
/// # AC-002 / BC-4.03.004 postcondition 2
///
/// The upgrade must complete within 1 second of server start. axum's
/// `WebSocketUpgrade::on_upgrade` is non-blocking and delegates to the async
/// runtime, so this constraint is trivially satisfied.
///
/// Note: `server.rs`'s `live_handler` now calls `handle_ws_connection_inner`
/// directly (to manage the `WsConnGuard` RAII lifetime inside the upgraded
/// future). This wrapper is kept for API compatibility with existing tests.
pub fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    rx: broadcast::Receiver<String>,
    shutdown_rx: watch::Receiver<bool>,
) -> Response {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, rx, shutdown_rx))
}

/// Inner WebSocket message loop, exposed `pub(crate)` so `server::live_handler`
/// can call it directly and place the connection-cap RAII guard inside the
/// upgraded future's scope.
pub(crate) async fn handle_ws_connection_inner(
    socket: WebSocket,
    rx: broadcast::Receiver<String>,
    shutdown_rx: watch::Receiver<bool>,
) {
    handle_ws_connection(socket, rx, shutdown_rx).await;
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
    // Write-only loop: this select! has arms for shutdown_rx and the broadcast
    // rx, but no arm for socket.recv(). This is intentional — EC-001 (push-only
    // server) is satisfied and tested. A silently-disconnected client lingers
    // until the next push fails (socket.send returns Err), at which point the
    // handler returns. STORY-048 should add a socket.recv() arm to reap closed
    // connections promptly rather than waiting for the next push.
    loop {
        tokio::select! {
            // Cooperative shutdown path (AC-005): send Close(1001) and return.
            // Delegates to `should_shutdown` — the single source of truth for
            // the shutdown decision (see its doc comment for the full truth table).
            result = shutdown_rx.changed() => {
                if should_shutdown(&result, *shutdown_rx.borrow()) {
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

    // -------------------------------------------------------------------------
    // Load-bearing regression guard for MED-1 (TD-VSDD-059).
    //
    // These tests call `should_shutdown` directly — the production code path.
    // If the `changed.is_err()` branch is removed from `should_shutdown`, the
    // (Err, false) case returns `false` and `test_BC_4_03_004_med1_err_false`
    // fails immediately, proving the regression guard is load-bearing.
    // -------------------------------------------------------------------------

    /// (Ok(()), true) → `should_shutdown` returns true.
    ///
    /// Normal graceful shutdown: sender called `send(true)` and `changed()` fired.
    #[tokio::test]
    async fn test_BC_4_03_004_med1_ok_true_is_shutdown() {
        let (tx, mut rx) = watch::channel(false);
        tx.send(true).expect("send must succeed");
        let result = rx.changed().await;
        assert!(
            should_shutdown(&result, *rx.borrow()),
            "(Ok, true) must be shutdown=true"
        );
    }

    /// (Ok(()), false) → `should_shutdown` returns false.
    ///
    /// Spurious or intermediate wake where the value is still false — handler
    /// should NOT shut down; it re-enters the select! loop.
    #[tokio::test]
    async fn test_BC_4_03_004_med1_ok_false_is_not_shutdown() {
        let (tx, mut rx) = watch::channel(false);
        // Send false → changed() fires but value is still false.
        tx.send(false).expect("send must succeed");
        let result = rx.changed().await;
        assert!(
            !should_shutdown(&result, *rx.borrow()),
            "(Ok, false) must be shutdown=false"
        );
    }

    /// (Err(_), false) → `should_shutdown` returns true.
    ///
    /// This is the MED-1 busy-spin case: the last Sender was dropped without
    /// ever calling `send(true)`, so `changed()` returns `Err` and `borrow()`
    /// is `false`. Pre-fix code evaluated only `*borrow()` → false → fall-
    /// through → infinite hot-spin. Post-fix `should_shutdown` checks
    /// `changed.is_err()` first → true → handler exits.
    ///
    /// Regression evidence: removing the `changed.is_err()` branch from
    /// `should_shutdown` makes this assertion fail (`false != true`).
    #[tokio::test]
    async fn test_BC_4_03_004_med1_err_false_is_shutdown() {
        let (tx, mut rx) = watch::channel(false);
        drop(tx); // Drop without send(true) — the bug trigger.
        let result = rx.changed().await; // Returns Err immediately.
        assert!(
            result.is_err(),
            "changed() must return Err when all Senders are dropped"
        );
        assert!(
            !*rx.borrow(),
            "borrow() must remain false — no send(true) was called"
        );
        assert!(
            should_shutdown(&result, *rx.borrow()),
            "(Err, false) must be shutdown=true — this is the MED-1 busy-spin case; \
             if this fails the is_err() branch was removed from should_shutdown"
        );
    }

    /// (Err(_), true) → `should_shutdown` returns true.
    ///
    /// Unusual but possible: Sender sent `true` then was immediately dropped.
    /// Both branches of the OR are true; shutdown must still be signalled.
    #[tokio::test]
    async fn test_BC_4_03_004_med1_err_true_is_shutdown() {
        let (tx, mut rx) = watch::channel(false);
        tx.send(true).expect("send must succeed");
        drop(tx); // Drop after sending true.
        // Drain the pending change notification.
        let result = rx.changed().await;
        // Whether result is Ok or Err here (implementation detail of tokio watch),
        // borrow() is true, so shutdown must be signalled regardless.
        assert!(
            should_shutdown(&result, *rx.borrow()),
            "(Err|Ok, true) must be shutdown=true"
        );
    }
}
