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
//!    `CT_Presentation`'s ordered sequence).
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

use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use sha2::{Digest, Sha256};
use slideforge_layout::SlideSectionEntry;

use crate::error::PptxError;
use crate::xml_escape::strip_xml10_invalid_chars;

// ─── Constants ────────────────────────────────────────────────────────────────

/// The fixed `p:ext/@uri` for the sectionLst extension slot.
///
/// This value is the same in every PPTX file — it is not per-deck generated.
pub const SECTION_LST_EXT_URI: &str = "{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}";

/// The `p14` namespace URI for `PowerPoint` 2010 extensions.
pub const P14_NS_URI: &str = "http://schemas.microsoft.com/office/powerpoint/2010/main";

/// The closing tag for `<p:presentation>` used as the injection point.
const PRESENTATION_CLOSE_TAG: &[u8] = b"</p:presentation>";

/// The opening tag prefix for `<p:presentation` used for `xmlns:p14` patching.
const PRESENTATION_OPEN_TAG: &[u8] = b"<p:presentation";

/// The `xmlns:p14` attribute to inject into the `<p:presentation` opening tag.
const XMLNS_P14_ATTR: &str =
    " xmlns:p14=\"http://schemas.microsoft.com/office/powerpoint/2010/main\"";

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
    ///   `CT_Presentation` is an ordered sequence; `p:extLst` is the terminal
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
        // AC-004: if no sections, return bytes unchanged — no p:extLst injected.
        if sections.is_empty() {
            return Ok(presentation_xml);
        }

        // Build the p:extLst block.
        let ext_lst_bytes = Self::build_ext_lst(sections)?;

        // Find the closing </p:presentation> tag — injection point.
        let close_pos =
            find_subsequence(&presentation_xml, PRESENTATION_CLOSE_TAG).ok_or_else(|| {
                PptxError::OoxmlElement {
                    part: "ppt/presentation.xml".to_string(),
                    detail: "closing </p:presentation> tag not found in presentation XML bytes"
                        .to_string(),
                }
            })?;

        // Inject ext_lst immediately before </p:presentation>.
        let mut result =
            Vec::with_capacity(presentation_xml.len() + ext_lst_bytes.len() + XMLNS_P14_ATTR.len());
        result.extend_from_slice(&presentation_xml[..close_pos]);
        result.extend_from_slice(&ext_lst_bytes);
        result.extend_from_slice(PRESENTATION_CLOSE_TAG);

        // Patch xmlns:p14 into the <p:presentation opening tag.
        // The xmlns:p14 attribute is injected immediately after "<p:presentation"
        // (before any existing attributes or the closing ">").
        let open_pos = find_subsequence(&result, PRESENTATION_OPEN_TAG).ok_or_else(|| {
            PptxError::OoxmlElement {
                part: "ppt/presentation.xml".to_string(),
                detail: "<p:presentation opening tag not found in presentation XML bytes"
                    .to_string(),
            }
        })?;

        // Insert xmlns:p14 immediately after "<p:presentation".
        let insert_at = open_pos + PRESENTATION_OPEN_TAG.len();
        let mut patched = Vec::with_capacity(result.len() + XMLNS_P14_ATTR.len());
        patched.extend_from_slice(&result[..insert_at]);
        patched.extend_from_slice(XMLNS_P14_ATTR.as_bytes());
        patched.extend_from_slice(&result[insert_at..]);

        Ok(patched)
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
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);

        // <p:extLst>
        writer
            .write_event(Event::Start(BytesStart::new("p:extLst")))
            .map_err(|e| PptxError::OoxmlElement {
                part: "ppt/presentation.xml".to_string(),
                detail: format!("quick-xml write error: {e}"),
            })?;

        // <p:ext uri="{BB962C8B-...}">
        {
            let mut ext_start = BytesStart::new("p:ext");
            ext_start.push_attribute(("uri", SECTION_LST_EXT_URI));
            writer
                .write_event(Event::Start(ext_start))
                .map_err(|e| PptxError::OoxmlElement {
                    part: "ppt/presentation.xml".to_string(),
                    detail: format!("quick-xml write error: {e}"),
                })?;
        }

        // <p14:sectionLst xmlns:p14="...">
        {
            let mut slt_start = BytesStart::new("p14:sectionLst");
            slt_start.push_attribute(("xmlns:p14", P14_NS_URI));
            writer
                .write_event(Event::Start(slt_start))
                .map_err(|e| PptxError::OoxmlElement {
                    part: "ppt/presentation.xml".to_string(),
                    detail: format!("quick-xml write error: {e}"),
                })?;
        }

        for section in sections {
            let guid = derive_section_guid(&section.name);

            // Sanitize the section name: strip XML-1.0-invalid control characters
            // (U+0001–U+0008, U+000B, U+000C, U+000E–U+001F) before writing to the
            // name attribute.  quick-xml's push_attribute entity-escapes & < > " '
            // but does NOT strip XML-1.0-invalid bytes; a raw control char produces
            // malformed XML that PowerPoint 365 cannot open (SEC-100 / CWE-116).
            let safe_name = strip_xml10_invalid_chars(&section.name);

            // <p14:section name="..." id="{...}">
            {
                let mut sec_start = BytesStart::new("p14:section");
                sec_start.push_attribute(("name", safe_name.as_str()));
                sec_start.push_attribute(("id", guid.as_str()));
                writer.write_event(Event::Start(sec_start)).map_err(|e| {
                    PptxError::OoxmlElement {
                        part: "ppt/presentation.xml".to_string(),
                        detail: format!("quick-xml write error: {e}"),
                    }
                })?;
            }

            // <p14:sldIdLst>
            writer
                .write_event(Event::Start(BytesStart::new("p14:sldIdLst")))
                .map_err(|e| PptxError::OoxmlElement {
                    part: "ppt/presentation.xml".to_string(),
                    detail: format!("quick-xml write error: {e}"),
                })?;

            for &slide_id in &section.slide_ids {
                // <p14:sldId id="NNN"/>
                let id_str = slide_id.to_string();
                let mut sld_id = BytesStart::new("p14:sldId");
                sld_id.push_attribute(("id", id_str.as_str()));
                writer
                    .write_event(Event::Empty(sld_id))
                    .map_err(|e| PptxError::OoxmlElement {
                        part: "ppt/presentation.xml".to_string(),
                        detail: format!("quick-xml write error: {e}"),
                    })?;
            }

            // </p14:sldIdLst>
            writer
                .write_event(Event::End(BytesEnd::new("p14:sldIdLst")))
                .map_err(|e| PptxError::OoxmlElement {
                    part: "ppt/presentation.xml".to_string(),
                    detail: format!("quick-xml write error: {e}"),
                })?;

            // </p14:section>
            writer
                .write_event(Event::End(BytesEnd::new("p14:section")))
                .map_err(|e| PptxError::OoxmlElement {
                    part: "ppt/presentation.xml".to_string(),
                    detail: format!("quick-xml write error: {e}"),
                })?;
        }

        // </p14:sectionLst>
        writer
            .write_event(Event::End(BytesEnd::new("p14:sectionLst")))
            .map_err(|e| PptxError::OoxmlElement {
                part: "ppt/presentation.xml".to_string(),
                detail: format!("quick-xml write error: {e}"),
            })?;

        // </p:ext>
        writer
            .write_event(Event::End(BytesEnd::new("p:ext")))
            .map_err(|e| PptxError::OoxmlElement {
                part: "ppt/presentation.xml".to_string(),
                detail: format!("quick-xml write error: {e}"),
            })?;

        // </p:extLst>
        writer
            .write_event(Event::End(BytesEnd::new("p:extLst")))
            .map_err(|e| PptxError::OoxmlElement {
                part: "ppt/presentation.xml".to_string(),
                detail: format!("quick-xml write error: {e}"),
            })?;

        Ok(buf)
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
    let hash = Sha256::digest(name.as_bytes());
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes.copy_from_slice(&hash[..16]);

    // Set version nibble (byte 6, high nibble) = 5 (UUID v5 convention).
    uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x50;
    // Set variant bits (byte 8, high 2 bits) = 10 (RFC 4122 variant).
    uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80;

    format_guid(&uuid_bytes)
}

/// Format 16 raw UUID bytes as a brace-wrapped, hyphen-separated uppercase GUID.
///
/// Format: `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` (uppercase hex, 8-4-4-4-12).
///
/// The parameter type `&[u8; 16]` statically guarantees exactly 16 bytes —
/// no runtime assertion needed (MED-1 fix, STORY-082 adversary pass 1).
#[must_use]
pub fn format_guid(bytes: &[u8; 16]) -> String {
    format!(
        "{{{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3], // 8 hex chars
        bytes[4],
        bytes[5], // 4 hex chars
        bytes[6],
        bytes[7], // 4 hex chars
        bytes[8],
        bytes[9], // 4 hex chars
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15] // 12 hex chars
    )
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Find the byte offset of `needle` in `haystack`.
///
/// Returns `None` if `needle` is not found. Uses a simple sliding-window search.
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}
