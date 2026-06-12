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
//! docProps/core.xml
//! docProps/app.xml
//! ```
//!
//! Note: `word/theme/theme1.xml` is NOT emitted by this crate. No BC-4.02.001
//! acceptance criterion requires a theme part for the DOCX exporter. PPTX-style
//! theme embedding (for custom color schemes) is deferred to a future story
//! under the brand system, not required for DOCX v1.0 correctness.
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
//! - **ooxmlsdk for all OOXML construction:** No raw XML string concatenation
//!   for dynamic content; static templates are XML-escaped where user data is
//!   embedded.
//! - **Effectful shell (SS-08):** Serialization is pure; file I/O is effectful.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod auto_sections;
pub mod content_types;
pub mod document_body;
pub mod error;
pub mod manual_sections;
pub mod numbering;
pub mod section_order;
pub mod styles;
pub mod xml_escape;
pub mod zip_assembler;

/// Default BCP-47 language tag when the deck carries no `lang` declaration.
///
/// The canonical default is `"en"` per BC-5.01.004 — NOT `"en-US"`. This
/// constant is the single source of truth for the no-lang default across all
/// DOCX surfaces: `word/document.xml` run `<w:lang>`, `docProps/core.xml`
/// `<dc:language>`, and any future DOCX surfaces that require a language tag.
pub const DEFAULT_DECK_LANG: &str = "en";

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
use tracing::instrument;

use crate::document_body::DocumentBodySerializer;
use crate::error::ExportError;
use crate::xml_escape::{xml_attr_escape, xml_content_escape};
use crate::zip_assembler::DocxZipAssembler;

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
    fn id(&self) -> &'static str {
        "docx"
    }

    fn extension(&self) -> &'static str {
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
    #[instrument(skip_all, fields(slides = laid_out.slides.len()))]
    fn export(
        &self,
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        _opts: &ExportOptions,
    ) -> Result<Vec<u8>, slideforge_plugin_api::ExportError> {
        build_docx(deck, laid_out, brand).map_err(slideforge_plugin_api::ExportError::from)
    }
}

/// Core export logic — builds all DOCX parts and assembles the ZIP.
///
/// Separated from the trait impl so errors use the richer local
/// [`ExportError`] type until the final conversion.
///
/// # Errors
///
/// Returns [`ExportError`] on any assembly failure.
fn build_docx(deck: &Deck, laid_out: &LaidOutDeck, brand: &Brand) -> Result<Vec<u8>, ExportError> {
    let mut asm = DocxZipAssembler::new();

    // ── `[Content_Types].xml` ─────────────────────────────────────────────
    let content_types = content_types::build_content_types()?;
    asm.add_part("[Content_Types].xml", content_types);

    // ── `_rels/.rels` ─────────────────────────────────────────────────────
    asm.add_part("_rels/.rels", build_root_rels());

    // ── `word/document.xml` and relationships ────────────────────────────
    let mut body_ser = DocumentBodySerializer::new();
    // Pass semantic_deck for STORY-081 I2 dual-title inline structure access.
    let document_xml = body_ser.serialize(laid_out, deck)?;
    asm.add_part("word/document.xml", document_xml);

    let doc_rels = build_document_rels(body_ser.relationships());
    asm.add_part("word/_rels/document.xml.rels", doc_rels);

    // ── `word/styles.xml` ────────────────────────────────────────────────
    let styles_xml = styles::build_styles(Some(brand))?;
    asm.add_part("word/styles.xml", styles_xml);

    // ── `word/numbering.xml` — abstract + concrete bullet definitions ──────
    let numbering_xml = numbering::build_numbering_xml()?;
    asm.add_part("word/numbering.xml", numbering_xml);

    // ── `word/settings.xml` (minimal stub) ───────────────────────────────
    asm.add_part("word/settings.xml", build_settings_xml());

    // ── `docProps/core.xml` with dc:language ─────────────────────────────
    let core_xml = build_core_xml(deck);
    asm.add_part("docProps/core.xml", core_xml);

    // ── `docProps/app.xml` (minimal stub) ────────────────────────────────
    asm.add_part("docProps/app.xml", build_app_xml());

    asm.finish()
}

/// Build `_rels/.rels` — the package-level relationships.
fn build_root_rels() -> Vec<u8> {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#
        .as_bytes()
        .to_vec()
}

/// Build `word/_rels/document.xml.rels` — relationships from the document body.
///
/// Includes relationships for `word/styles.xml`, `word/numbering.xml`,
/// `word/settings.xml`, and any hyperlinks collected during body serialization.
fn build_document_rels(hyperlinks: &[document_body::HyperlinkRel]) -> Vec<u8> {
    use std::fmt::Write as FmtWrite;

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n",
    );

    xml.push_str(
        "  <Relationship Id=\"rId1\" \
         Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" \
         Target=\"styles.xml\"/>\n",
    );
    xml.push_str(
        "  <Relationship Id=\"rId2\" \
         Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering\" \
         Target=\"numbering.xml\"/>\n",
    );
    xml.push_str(
        "  <Relationship Id=\"rId3\" \
         Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings\" \
         Target=\"settings.xml\"/>\n",
    );

    // Hyperlink relationships from body serialization.
    for hl in hyperlinks {
        let escaped_target = xml_attr_escape(&hl.target);
        // Using `write!` avoids a temporary String allocation (clippy::format_push_string).
        // `write!` on a `String` is infallible; the `let _ =` suppresses the
        // `must_use` warning on the `Result<(), fmt::Error>` return value.
        let _ = writeln!(
            xml,
            "  <Relationship Id=\"{}\" \
             Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink\" \
             Target=\"{}\" TargetMode=\"External\"/>",
            xml_attr_escape(&hl.r_id),
            escaped_target
        );
    }

    xml.push_str("</Relationships>");
    xml.into_bytes()
}

/// Build `word/settings.xml` — minimal stub.
fn build_settings_xml() -> Vec<u8> {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:defaultTabStop w:val="708"/>
</w:settings>
"#
    .as_bytes()
    .to_vec()
}

/// Build `docProps/core.xml` with `dc:language` from the deck metadata.
fn build_core_xml(deck: &Deck) -> Vec<u8> {
    let lang = deck.metadata.lang.as_deref().unwrap_or(DEFAULT_DECK_LANG);

    // Declare only the namespaces that are actually used: cp: and dc:.
    // xmlns:dcterms and xmlns:xsi are omitted — no dcterms:created/modified
    // is emitted and xsi would flag as unused by XML linters (S-2 fix).
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <cp:coreProperties \
           xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" \
           xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n\
           <dc:language>{lang}</dc:language>\n\
         </cp:coreProperties>",
        lang = xml_content_escape(lang),
    );

    xml.into_bytes()
}

/// Build `docProps/app.xml` — minimal stub.
fn build_app_xml() -> Vec<u8> {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Application>slideforge</Application>
</Properties>
"#
    .as_bytes()
    .to_vec()
}
