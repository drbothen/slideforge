//! `slideforge-pptx` — PPTX exporter plugin for the slideforge pipeline.
//!
//! This crate implements the [`slideforge_plugin_api::Exporter`] trait for the
//! `.pptx` format. It takes a [`slideforge_types::Deck`] (semantic IR) and a
//! [`slideforge_layout::LaidOutDeck`] (geometric IR) together with a resolved
//! [`slideforge_types::Brand`] and produces a valid PPTX ZIP archive as
//! `Vec<u8>`.
//!
//! ## Architecture (STORY-037 / BC-4.01.001)
//!
//! ```text
//! PptxExporter::export(deck, laid_out, brand, opts)
//!   │
//!   ├─ SlideSerializer::build()     → ppt/slides/slide{n}.xml
//!   ├─ PresentationSerializer::build() → ppt/presentation.xml
//!   ├─ ContentTypesBuilder::build() → [Content_Types].xml
//!   ├─ RelsBuilder::build()         → _rels/.rels, ppt/_rels/…, etc.
//!   └─ ZipAssembler::assemble()     → final ZIP bytes
//! ```
//!
//! ## Constraints
//!
//! - All OOXML element construction goes through `ooxmlsdk` typed builders
//!   (ADR-001). No XML string concatenation.
//! - All coordinates are integer [`slideforge_types::Emu`] (`i64`). No `f64`.
//! - Report and detail register content MUST NOT appear in slide XML bodies.
//! - `notesMaster1.xml` and `handoutMaster1.xml` are always written.
//! - Slide IDs start at 256; master ID is 2^31.
//! - Output is deterministic: same inputs → byte-identical output.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod content_types;
pub mod error;
pub mod presentation;
pub mod rels;
pub mod slide_serializer;
pub mod zip_assembler;

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used
)]
mod tests {
    mod core_tests;
}

use slideforge_layout::LaidOutDeck;
use slideforge_plugin_api::{ExportError, ExportOptions, Exporter};
use slideforge_types::{Brand, Deck};

use crate::error::PptxError;

/// The built-in PPTX exporter plugin.
///
/// `PptxExporter` implements the [`Exporter`] plugin trait for the `.pptx`
/// output format. It is registered with the [`slideforge_plugin_api::PluginRegistry`]
/// under the id `"pptx"`.
///
/// ## Thread safety
///
/// `PptxExporter` holds no mutable state; it is `Send + Sync` by construction.
#[derive(Debug, Clone, Copy, Default)]
pub struct PptxExporter;

impl PptxExporter {
    /// Create a new `PptxExporter` instance.
    ///
    /// The exporter is stateless; this function is equivalent to
    /// `PptxExporter::default()`.
    #[must_use]
    pub fn new() -> Self {
        PptxExporter
    }

    /// Core serialisation logic: build all PPTX parts and assemble the ZIP.
    ///
    /// Separated from the trait method so it can return [`PptxError`] directly
    /// before the trait boundary converts it to [`ExportError`].
    fn export_inner(
        &self,
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        opts: &ExportOptions,
    ) -> Result<Vec<u8>, PptxError> {
        todo!(
            "PptxExporter::export_inner — \
             1. build slide XMLs via SlideSerializer, \
             2. build presentation.xml via PresentationSerializer, \
             3. embed brand master/layout/theme XML verbatim, \
             4. build notesMaster1.xml + handoutMaster1.xml stubs, \
             5. build docProps/core.xml + app.xml, \
             6. build all .rels files via RelsBuilder, \
             7. build [Content_Types].xml via ContentTypesBuilder, \
             8. assemble into ZIP via ZipAssembler"
        )
    }
}

impl Exporter for PptxExporter {
    fn id(&self) -> &str {
        "pptx"
    }

    fn extension(&self) -> &str {
        "pptx"
    }

    fn export(
        &self,
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        opts: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        self.export_inner(deck, laid_out, brand, opts)
            .map_err(|e| ExportError::RenderError {
                message: e.to_string(),
            })
    }
}
