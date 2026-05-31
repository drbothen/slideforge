//! `slideforge-pdf` — Pure-Rust PDF export backend for slideforge.
//!
//! Uses `krilla 0.6.0` as the primary PDF engine (which wraps `pdf-writer
//! 0.14.0` and `subsetter 0.2.3` transitively) plus a custom
//! [`tag_engine::SlideTagEngine`] for PDF/UA-1 structure tree generation.
//!
//! ## Architecture (BC-4.03.002)
//!
//! - No Chrome, no headless browser, no FFI to C PDF libraries.
//! - `PdfExporter` implements the [`slideforge_plugin_api::Exporter`] trait.
//! - Font subsetting is handled internally by krilla's Surface/text API.
//! - SVG embedding (charts, diagrams) goes through `usvg` parse → krilla
//!   `Surface` path-drawing operations — no rasterization.
//! - PDF/UA-1 structure tree is built by [`tag_engine::SlideTagEngine`] using
//!   `krilla::tagging` and attached via `Document::set_tag_tree`.
//!
//! ## REAL Exporter trait signature (reconciled from codebase)
//!
//! The `Exporter` trait signature is:
//! ```text
//! fn export(
//!     &self,
//!     deck: &Deck,
//!     laid_out: &LaidOutDeck,
//!     brand: &Brand,
//!     opts: &ExportOptions,
//! ) -> Result<Vec<u8>, ExportError>
//! ```
//! Diverges from the story spec's claimed signature — see `exporter.rs` for
//! full reconciliation notes.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod exporter;
pub mod font;
pub mod svg_embed;
pub mod tag_engine;

pub use error::PdfExportError;
pub use exporter::PdfExporter;
pub use tag_engine::SlideTagEngine;
