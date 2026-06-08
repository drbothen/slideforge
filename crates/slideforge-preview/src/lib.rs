//! Live web preview server for slideforge.
//!
//! This crate implements the `slideforge-preview` crate: a local development
//! HTTP/WebSocket server that serves a live deck preview using `axum 0.8.9`.
//!
//! ## Architecture
//!
//! The server runs a local HTTP server on `localhost:<port>` (default 3000) and
//! exposes two endpoints:
//!
//! - `GET /` — serves the initial HTML page with the deck preview (SVG slides).
//! - `GET /live` — upgrades to a WebSocket connection for live updates.
//!
//! When a `.sf` source file changes on disk, the evaluation pipeline calls
//! [`PreviewServer::push_update`] or [`PreviewServer::push_error`] to push a
//! JSON message to all connected browser clients.
//!
//! ## Key design invariants (BC-4.03.004)
//!
//! 1. Watch mode is always warn-only — this crate only receives pre-rendered
//!    slide HTML and forwards it; it does not control evaluation mode.
//! 2. WebSocket message format is stable within v1.x (type-discriminated JSON).
//! 3. No output files are written — only WebSocket messages are sent.
//! 4. The axum server runs in its own `tokio::spawn()` task and never blocks
//!    the file-watcher or evaluation pipeline.
//!
//! ## Async runtime (ADR-021)
//!
//! Tokio is the exclusive async runtime. Features: `rt-multi-thread, macros, net,
//! time, sync, signal, fs, io-util` (NOT `features = ["full"]`).
//!
//! ## Forbidden dependencies (runtime)
//!
//! - `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`
//! - `tokio-tungstenite` (dev-dep for test clients only)
//! - `warp`, `actix-web`

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod debounce;
pub mod error;
pub mod messages;
pub mod server;
pub mod ws_handler;

pub use error::PreviewError;
pub use messages::{DiagnosticMessage, SlideHtml, WebSocketMessage};
pub use server::{PreviewHandle, PreviewServer};
