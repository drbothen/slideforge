//! OOXML XML serialization for individual slide layouts (BC-2.01.005).
//!
//! [`serialize_layout_to_xml`] converts a [`SlideLayoutDef`] to the complete
//! `slideLayoutN.xml` bytes required by the OOXML ZIP package.
//!
//! XML is generated using `quick_xml::Writer` with explicit element ordering
//! matching ECMA-376 schema order (Architecture Compliance Rule 3 from STORY-023).
//!
//! ## Dark Layout Handling (AC-009, R4 mitigation)
//!
//! For layouts with `has_color_override = true` (CL-01 and CL-11), the
//! generated XML includes:
//! - `<p:clrMapOvr><a:overrideClrMapping bg1="dk2" tx1="lt1" .../>` element.
//! - Explicit white text on all placeholder run elements (`<a:solidFill>` with
//!   `<a:srgbClr val="FFFFFF"/>`), providing `LibreOffice` 7.x compatibility.
//!
//! ## Notes/Handout Master Stubs (AC-012)
//!
//! Minimal stub XML bytes for `notesMaster1.xml` and `handoutMaster1.xml`
//! are exposed as constants for use by the PPTX exporter (STORY-037).

use crate::layouts::SlideLayoutDef;

// ─── Master stubs ─────────────────────────────────────────────────────────────

/// Minimal valid `notesMaster1.xml` stub bytes (AC-012).
///
/// Required for OOXML compliance even when notes are not used.
/// The PPTX exporter (STORY-037) writes this to `ppt/notesMasters/notesMaster1.xml`.
pub const NOTES_MASTER_STUB: &[u8] = b"\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<p:notesMaster xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" \
               xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
  <p:cSld><p:spTree/></p:cSld>\
  <p:hf/><p:notesStyle/>\
</p:notesMaster>";

/// Minimal valid `handoutMaster1.xml` stub bytes (AC-012).
///
/// Required for OOXML compliance even when handouts are not used.
/// The PPTX exporter (STORY-037) writes this to `ppt/handoutMasters/handoutMaster1.xml`.
pub const HANDOUT_MASTER_STUB: &[u8] = b"\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<p:handoutMaster xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" \
                 xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
  <p:cSld><p:spTree/></p:cSld>\
  <p:hf/>\
</p:handoutMaster>";

// ─── XML serialization ────────────────────────────────────────────────────────

/// Serialize a [`SlideLayoutDef`] to complete OOXML `slideLayoutN.xml` bytes.
///
/// The returned `Vec<u8>` contains a valid `slideLayoutN.xml` document that
/// includes:
/// - `<p:sldLayout>` root element with appropriate namespace declarations.
/// - `<p:cSld><p:spTree>` containing one `<p:sp>` per placeholder.
/// - Each `<p:sp>` has `<p:nvSpPr>` with semantic `<p:cNvPr name="...">` (AC-010).
/// - Each `<p:sp>` has `<p:ph type="..." idx="...">` in ECMA-376 schema order.
/// - For dark layouts (`has_color_override = true`): `<p:clrMapOvr>` and
///   explicit white `<a:solidFill>` on all run elements (AC-009).
///
/// `master_rel_id` is the relationship ID string pointing to the slide master
/// (e.g., `"rId1"`). This is written into the `<p:sldLayout>` relationship.
///
/// # Errors
///
/// Returns a `Vec<u8>` unconditionally. XML generation with `quick_xml::Writer`
/// does not return errors on in-memory `Cursor<Vec<u8>>` writes.
#[must_use]
pub fn serialize_layout_to_xml(_def: &SlideLayoutDef, _master_rel_id: &str) -> Vec<u8> {
    todo!()
}

/// Generate the `[Content_Types].xml` registration string for all 31 layouts.
///
/// Returns a `String` containing one `<Override PartName="...">` entry per
/// layout, covering `slideLayout1.xml` through `slideLayout31.xml`.
///
/// This string is stored in `BrandTemplate` and used by the PPTX exporter
/// (STORY-037) to populate `[Content_Types].xml` (AC-013).
#[must_use]
pub fn generate_content_types_layout_entries(_layout_count: usize) -> String {
    todo!()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-012 — `NOTES_MASTER_STUB` is non-empty valid XML bytes.
    #[test]
    fn test_bc_2_01_002_notes_master_stub_is_non_empty() {
        assert!(!NOTES_MASTER_STUB.is_empty(), "notesMaster stub must not be empty");
        assert!(
            NOTES_MASTER_STUB.starts_with(b"<?xml"),
            "notesMaster stub must be XML"
        );
    }

    /// AC-012 — `HANDOUT_MASTER_STUB` is non-empty valid XML bytes.
    #[test]
    fn test_bc_2_01_002_handout_master_stub_is_non_empty() {
        assert!(!HANDOUT_MASTER_STUB.is_empty(), "handoutMaster stub must not be empty");
        assert!(
            HANDOUT_MASTER_STUB.starts_with(b"<?xml"),
            "handoutMaster stub must be XML"
        );
    }

    /// AC-013 — `generate_content_types_layout_entries` produces 31 Override entries.
    #[test]
    fn test_bc_2_01_005_content_types_layout_entries_count() {
        // Deferred until generate_content_types_layout_entries() is implemented.
        // Expected: 31 <Override ...> lines, each with PartName="/ppt/slideLayouts/slideLayout{N}.xml"
    }

    /// BC-2.01.005 / AC-009 — dark layout XML contains clrMapOvr element.
    #[test]
    fn test_bc_2_01_005_dark_layout_xml_has_clr_map_ovr() {
        // Deferred until serialize_layout_to_xml() is implemented.
        // Expected: output contains "clrMapOvr" and "overrideClrMapping"
    }

    /// BC-2.01.005 / AC-009 — dark layout XML contains explicit white text color.
    #[test]
    fn test_bc_2_01_005_dark_layout_xml_has_explicit_white_text() {
        // Deferred until serialize_layout_to_xml() is implemented.
        // Expected: output contains "<a:srgbClr val=\"FFFFFF\"/>"
    }

    /// AC-010 — layout XML uses semantic placeholder names, not "Shape N".
    #[test]
    fn test_bc_2_01_005_layout_xml_uses_semantic_names() {
        // Deferred until serialize_layout_to_xml() is implemented.
        // Expected: no occurrence of r#"name="Shape "#
    }
}
