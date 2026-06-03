//! DOCX exporter plugin for slideforge — SS-08 (DOCX Export).
//!
//! This crate implements the [`Exporter`] plugin trait to produce Word-compatible
//! `.docx` files from the [`slideforge_layout::LaidOutDeck`] IR.
//!
//! ## Register routing (BC-4.02.001)
//!
//! | Register | DOCX body treatment |
//! |----------|---------------------|
//! | `report` | Narrative body paragraphs under per-slide `Heading1` |
//! | `detail` | Extended sections after the main body, under `Heading2` |
//! | `notes`  | **Never** emitted — excluded from all DOCX body content |
//!
//! ## DOCX ZIP structure
//!
//! ```text
//! [Content_Types].xml
//! _rels/.rels
//! word/document.xml
//! word/_rels/document.xml.rels
//! word/styles.xml
//! word/numbering.xml
//! word/settings.xml
//! word/theme/theme1.xml
//! docProps/core.xml
//! docProps/app.xml
//! ```
//!
//! ## Dependencies
//!
//! This crate depends only on `slideforge-types`, `slideforge-plugin-api`, and
//! `slideforge-layout`. It must NOT depend on any sibling exporter crate
//! (`slideforge-pptx`, `slideforge-pdf`, `slideforge-html`).
//!
//! ## Architecture compliance
//!
//! - **Two-IR model (DI-009):** Reads from `LaidOutDeck` only.
//! - **ooxmlsdk for all OOXML construction:** No raw XML string concatenation.
//! - **Effectful shell (SS-08):** Serialization is pure; file I/O is effectful.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod content_types;
pub mod document_body;
pub mod error;
pub mod styles;
pub mod zip_assembler;

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used
)]
mod tests;

use slideforge_layout::LaidOutDeck;
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{Brand, Deck};

/// The DOCX exporter plugin.
///
/// Register with [`slideforge_plugin_api::PluginRegistry::register_exporter`]
/// to enable `.docx` export. Produces a Word-compatible ZIP archive where
/// `report` register content appears as body paragraphs and `detail` register
/// content appears in extended appendix sections.
///
/// `notes` register content is never included in the DOCX body
/// (BC-4.02.001 invariant 1 / DI-012).
pub struct DocxExporter;

impl Exporter for DocxExporter {
    fn id(&self) -> &str {
        "docx"
    }

    fn extension(&self) -> &str {
        "docx"
    }

    /// Produce `.docx` bytes from the slideforge IR.
    ///
    /// Reads only from `laid_out` (the geometric IR). Does not call the
    /// evaluator or parser (two-IR model / DI-009). Brand is used to source
    /// font names and palette colors for `word/styles.xml`.
    ///
    /// # Errors
    ///
    /// Returns [`slideforge_plugin_api::ExportError`] if ZIP assembly or
    /// OOXML construction fails.
    fn export(
        &self,
        _deck: &Deck,
        _laid_out: &LaidOutDeck,
        _brand: &Brand,
        _opts: &ExportOptions,
    ) -> Result<Vec<u8>, slideforge_plugin_api::ExportError> {
        todo!()
    }
}
