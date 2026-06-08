//! `PreviewServer` — the live development HTTP/WebSocket server.
//!
//! ## Architecture (BC-4.03.004 invariant 4)
//!
//! The server runs in a dedicated `tokio::spawn()` task so it never blocks the
//! file-watcher or evaluation pipeline. Communication with the server uses a
//! `tokio::sync::broadcast::Sender<String>` (JSON-encoded `WebSocketMessage`).
//!
//! ## Graceful shutdown (AC-005)
//!
//! Shutdown is handled via `axum::serve(...).with_graceful_shutdown(shutdown_signal())`.
//! `shutdown_signal()` awaits `tokio::signal::ctrl_c()` and on Unix also
//! `tokio::signal::unix::signal(SignalKind::terminate())`.
//!
//! ## No output files (BC-4.03.004 invariant 3)
//!
//! `PreviewServer` has no filesystem write calls. It receives pre-rendered
//! `SlideHtml` strings from the evaluation pipeline and forwards them over WebSocket.

use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::error::PreviewError;
use crate::messages::{DiagnosticMessage, SlideHtml};

/// Capacity of the broadcast channel for WebSocket messages.
///
/// 16 is sufficient for development use — if a slow client falls behind by
/// more than 16 messages it receives `RecvError::Lagged` and recovers gracefully.
const BROADCAST_CAPACITY: usize = 16;

/// Handle returned by [`PreviewServer::start`] for lifecycle management.
///
/// Wraps the `JoinHandle<()>` for the axum server task.
///
/// # AC-001 / AC-009
///
/// `start()` returns `Result<PreviewHandle, PreviewError>` so that port-in-use
/// errors can be surfaced before the server task is spawned.
pub struct PreviewHandle {
    /// The join handle for the axum server task.
    pub join_handle: JoinHandle<()>,
}

impl PreviewHandle {
    /// Abort the server task immediately.
    pub fn abort(&self) {
        self.join_handle.abort();
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
    // allow(dead_code) during Red Gate: tx is used once push_update/push_error are implemented.
    #[allow(dead_code)]
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

    /// Start the axum server on `localhost:<port>`.
    ///
    /// # Errors
    ///
    /// - [`PreviewError::PortInUse`] if the port is already bound.
    /// - [`PreviewError::Io`] for other I/O errors.
    ///
    /// # AC-001 / AC-009 / BC-4.03.004 postcondition 1
    ///
    /// Returns `Ok(PreviewHandle)` on success. The handle wraps the axum task's
    /// `JoinHandle<()>`. The server is ready to accept connections immediately.
    pub fn start(
        &self,
        _port: u16,
        _initial_deck: &slideforge_layout::LaidOutDeck,
    ) -> Result<PreviewHandle, PreviewError> {
        todo!("implement PreviewServer::start: bind TCP listener, build axum router, spawn task")
    }

    /// Push a successful deck update to all connected WebSocket clients.
    ///
    /// Sends `{"type": "reload", "slides": [...]}` to every connected client.
    ///
    /// # AC-003 / AC-007 / BC-4.03.004 postcondition 3b
    ///
    /// This method is non-blocking. If no clients are connected, it returns
    /// immediately (broadcast::Sender::send returns Err when no receivers, but
    /// that is silently ignored — fire-and-forget semantics).
    pub fn push_update(&self, _slides: Vec<SlideHtml>) {
        todo!("implement push_update: serialize WebSocketMessage::Reload and broadcast")
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
    pub fn push_error(&self, _errors: Vec<DiagnosticMessage>) {
        todo!("implement push_error: serialize WebSocketMessage::Error and broadcast")
    }

    /// Subscribe to the broadcast channel.
    ///
    /// Returns a `Receiver` that receives all future broadcast messages.
    /// Used internally by the WebSocket handler.
    // allow(dead_code): called by ws_handler once start() is implemented.
    #[allow(dead_code)]
    pub(crate) fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

impl Default for PreviewServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BC-4.03.004 invariant 4 — push_update is non-blocking when no clients connected.
    #[tokio::test]
    async fn test_BC_4_03_004_invariant_push_update_nonblocking_no_clients() {
        todo!("implement: call push_update with no receivers, assert returns immediately")
    }

    /// BC-4.03.004 invariant 4 — push_error is non-blocking when no clients connected.
    #[tokio::test]
    async fn test_BC_4_03_004_invariant_push_error_nonblocking_no_clients() {
        todo!("implement: call push_error with no receivers, assert returns immediately")
    }

    /// AC-003 — push_update broadcasts reload JSON to subscribed receiver.
    #[tokio::test]
    async fn test_BC_4_03_004_push_update_broadcasts_to_subscriber() {
        todo!(
            "implement: subscribe receiver, call push_update, assert receiver gets reload JSON"
        )
    }

    /// AC-004 — push_error broadcasts error JSON to subscribed receiver.
    #[tokio::test]
    async fn test_BC_4_03_004_push_error_broadcasts_to_subscriber() {
        todo!(
            "implement: subscribe receiver, call push_error, assert receiver gets error JSON"
        )
    }

    /// AC-003 — push_update message contains type=reload and slides array.
    #[tokio::test]
    async fn test_BC_4_03_004_push_update_message_has_correct_type() {
        todo!(
            "implement: subscribe, push_update, parse JSON, assert type==reload and slides present"
        )
    }

    /// AC-004 — push_error message contains type=error and errors array.
    #[tokio::test]
    async fn test_BC_4_03_004_push_error_message_has_correct_type() {
        todo!(
            "implement: subscribe, push_error, parse JSON, assert type==error and errors present"
        )
    }

    /// BC-4.03.004 invariant 3 — PreviewServer::start does not write any output files.
    ///
    /// Verified in integration tests (tests/integration.rs AC-006).
    /// This unit test verifies push_update and push_error have no write syscalls
    /// (structural: they don't hold any file handles).
    #[test]
    fn test_BC_4_03_004_invariant_no_file_output() {
        todo!(
            "implement: verify PreviewServer struct contains no file handles or path writers"
        )
    }

    /// AC-007 — server runs in separate task (structural check: start() returns JoinHandle).
    #[test]
    fn test_BC_4_03_004_ac007_server_in_separate_task() {
        todo!(
            "implement: verify start() returns PreviewHandle wrapping JoinHandle (not blocking)"
        )
    }
}
