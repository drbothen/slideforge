//! `SectionListBuilder` — build and inject `p:extLst`/`p14:sectionLst` XML.
//!
//! ## Purpose
//!
//! `SectionListBuilder` receives the `ppt/presentation.xml` bytes produced by
//! [`crate::presentation::PresentationSerializer::build`]. If
//! `slide_sections` is empty it returns the bytes unchanged (no `p:extLst`
//! injection). Otherwise it:
//!
//! 1. Constructs the `p:extLst` / `p14:sectionLst` block using `quick-xml`.
//! 2. Locates the closing `</p:presentation>` in the bytes.
//! 3. Injects the block immediately before `</p:presentation>` (so that
//!    `p:extLst` is the LAST child of `p:presentation` — schema-required by
//!    CT_Presentation's ordered sequence).
//! 4. Patches the `<p:presentation` opening tag to declare
//!    `xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main"`.
//!
//! ## Why raw-XML injection?
//!
//! `ooxmlsdk =0.6.1` has no typed `p14` structs and silently drops unknown
//! extension children. `SectionListBuilder` uses `quick-xml =0.36.0` for
//! building the extension block and byte-slice injection for placement — the
//! same W1/W2 post-processing pattern established in `opc_postprocess.rs`
//! and `notes_master.rs`.
//!
//! ## sectionLst XML structure (authoritative)
//!
//! ```xml
//! <p:extLst>
//!   <p:ext uri="{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}">
//!     <p14:sectionLst xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main">
//!       <p14:section name="Background" id="{...UUID v5-like...}">
//!         <p14:sldIdLst>
//!           <p14:sldId id="256"/>
//!         </p14:sldIdLst>
//!       </p14:section>
//!     </p14:sectionLst>
//!   </p:ext>
//! </p:extLst>
//! ```
//!
//! ## Deterministic GUID derivation (BC-4.01.003 invariant 3)
//!
//! Section `id` GUIDs are derived deterministically from the section name via
//! SHA-256 + UUID v5-like bit manipulation:
//!
//! ```text
//! raw = sha2::Sha256::digest(section_name.as_bytes())  // 32 bytes
//! uuid_bytes[0..16] = raw[0..16]
//! uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x50   // version nibble = 5
//! uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80   // RFC 4122 variant
//! // Format: {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}  (uppercase)
//! ```
//!
//! `Uuid::new_v4()` is FORBIDDEN — it is non-deterministic.
//!
//! ## STORY-082
//!
//! Introduced in STORY-082 (PPTX: Slide-Grouping Sections).

use slideforge_layout::SlideSectionEntry;

use crate::error::PptxError;

// ─── Constants ────────────────────────────────────────────────────────────────

/// The fixed `p:ext/@uri` for the sectionLst extension slot.
///
/// This value is the same in every PPTX file — it is not per-deck generated.
pub const SECTION_LST_EXT_URI: &str = "{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}";

/// The `p14` namespace URI for PowerPoint 2010 extensions.
pub const P14_NS_URI: &str = "http://schemas.microsoft.com/office/powerpoint/2010/main";

// ─── SectionListBuilder ───────────────────────────────────────────────────────

/// Builds and injects `p:extLst`/`p14:sectionLst` into `presentation.xml`.
///
/// Stateless; all methods are free functions operating on byte slices.
pub struct SectionListBuilder;

impl SectionListBuilder {
    /// Inject the `p:extLst`/`p14:sectionLst` block into `presentation_xml`.
    ///
    /// # Parameters
    ///
    /// - `presentation_xml`: the `ppt/presentation.xml` bytes from
    ///   [`crate::presentation::PresentationSerializer::build`].
    /// - `sections`: the slide-grouping sections from
    ///   [`slideforge_layout::LaidOutDeck::slide_sections`].
    ///
    /// # Returns
    ///
    /// - If `sections` is empty: returns `Ok(presentation_xml)` unchanged.
    ///   No `p:extLst` is injected.
    /// - Otherwise: returns `Ok(modified_bytes)` with `p:extLst` injected
    ///   immediately before `</p:presentation>` and `xmlns:p14` declared on
    ///   the `<p:presentation` opening tag.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if:
    /// - The closing `</p:presentation>` tag is not found in the input bytes.
    /// - The `<p:presentation` opening tag is not found (needed for `xmlns:p14`
    ///   injection).
    /// - `quick-xml` writer fails to serialize the `p:extLst` block.
    ///
    /// # Architecture Compliance
    ///
    /// - `p:extLst` MUST be injected immediately before `</p:presentation>` —
    ///   CT_Presentation is an ordered sequence; `p:extLst` is the terminal
    ///   optional element (BC-4.01.003 invariant 4 / STORY-082 AC-003).
    /// - `xmlns:p14` MUST be declared on `<p:presentation` (BC-4.01.003 PC5).
    /// - GUIDs MUST be deterministic (SHA-256 derivation) — `Uuid::new_v4()`
    ///   is FORBIDDEN (BC-4.01.003 invariant 3 / STORY-082 AC-009).
    ///
    /// # STORY-082
    ///
    /// Introduced in STORY-082.
    pub fn inject(
        presentation_xml: Vec<u8>,
        sections: &[SlideSectionEntry],
    ) -> Result<Vec<u8>, PptxError> {
        todo!(
            "STORY-082: implement SectionListBuilder::inject — \
             if sections is empty return presentation_xml unchanged; \
             otherwise build p:extLst block with quick-xml, inject before \
             </p:presentation>, patch xmlns:p14 onto <p:presentation opening tag"
        )
    }

    /// Build the raw `p:extLst` / `p14:sectionLst` XML bytes for `sections`.
    ///
    /// Uses `quick-xml` to construct the well-formed extension block.
    /// Section names are XML-escaped (per BC-4.01.003 EC-004).
    /// GUIDs are derived deterministically from section names (AC-009).
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if `quick-xml` writer fails.
    pub fn build_ext_lst(sections: &[SlideSectionEntry]) -> Result<Vec<u8>, PptxError> {
        todo!(
            "STORY-082: implement SectionListBuilder::build_ext_lst — \
             use quick-xml Writer to produce <p:extLst><p:ext uri='...'> \
             <p14:sectionLst xmlns:p14='...'><p14:section name='...' id='...'> \
             <p14:sldIdLst><p14:sldId id='...'/>...</p14:sldIdLst> \
             </p14:section>...</p14:sectionLst></p:ext></p:extLst>"
        )
    }
}

// ─── GUID derivation ─────────────────────────────────────────────────────────

/// Derive a deterministic UUID v5-like GUID from a section name.
///
/// ## Algorithm
///
/// ```text
/// raw = sha2::Sha256::digest(name.as_bytes())   // 32 bytes
/// uuid_bytes[0..16] = raw[0..16]
/// uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x50   // version nibble = 5
/// uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80   // RFC 4122 variant
/// result = "{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}"  (uppercase hex)
/// ```
///
/// Same name → same GUID → byte-identical output across builds.
/// `Uuid::new_v4()` is FORBIDDEN (non-deterministic, violates BC-4.01.003 inv 3).
///
/// # Returns
///
/// A brace-wrapped, hyphen-separated, UPPERCASE hex GUID string.
/// Example: `"{3D4F2B8A-1C9E-5F2A-B4D6-7E8A9B0C1D2E}"`.
///
/// # STORY-082
///
/// Introduced in STORY-082.
#[must_use]
pub fn derive_section_guid(name: &str) -> String {
    todo!(
        "STORY-082: implement derive_section_guid — \
         sha2::Sha256::digest(name) → first 16 bytes → set version nibble=5, \
         variant=RFC4122 → format as uppercase brace-wrapped GUID string"
    )
}

/// Format 16 raw UUID bytes as a brace-wrapped, hyphen-separated uppercase GUID.
///
/// Format: `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` (uppercase hex, 8-4-4-4-12).
///
/// # Panics
///
/// Panics if `bytes.len() != 16`.
#[must_use]
pub fn format_guid(bytes: &[u8]) -> String {
    assert_eq!(bytes.len(), 16, "GUID requires exactly 16 bytes");
    todo!(
        "STORY-082: implement format_guid — format bytes as uppercase hex \
         with hyphens at positions 4, 6, 8, 10 and braces"
    )
}
