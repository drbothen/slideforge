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
    use std::sync::Arc;

    use super::*;
    use crate::layouts::{LayoutPlaceholder, SlideLayoutDef};

    /// Build a minimal non-dark layout for XML serialization tests.
    fn minimal_light_layout() -> SlideLayoutDef {
        SlideLayoutDef {
            index: 1,
            name: Arc::from("Title Slide"),
            ooxml_type: Some(Arc::from("title")),
            placeholders: vec![LayoutPlaceholder {
                ph_type: Arc::from("ctrTitle"),
                idx: 0,
                accessibility_name: Arc::from("Title Placeholder"),
                x: 457_200,
                y: 274_638,
                cx: 8_229_600,
                cy: 1_143_000,
            }],
            has_color_override: false,
            color_override_bg: None,
            color_override_tx: None,
        }
    }

    /// Build a dark layout (CL-01 equivalent) for XML serialization tests.
    fn minimal_dark_layout() -> SlideLayoutDef {
        SlideLayoutDef {
            index: 12,
            name: Arc::from("SF Section Divider"),
            ooxml_type: None,
            placeholders: vec![LayoutPlaceholder {
                ph_type: Arc::from("title"),
                idx: 0,
                accessibility_name: Arc::from("Section Title"),
                x: 457_200,
                y: 274_638,
                cx: 8_229_600,
                cy: 1_143_000,
            }],
            has_color_override: true,
            color_override_bg: Some(Arc::from("dk2")),
            color_override_tx: Some(Arc::from("lt1")),
        }
    }

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
    /// Tests that todo!() body panics (Red Gate).
    #[test]
    fn test_bc_2_01_005_content_types_layout_entries_count() {
        let entries = generate_content_types_layout_entries(31);
        // Must contain exactly 31 slideLayout path entries
        let override_count = entries.matches("slideLayout").count();
        assert_eq!(
            override_count,
            31,
            "must generate 31 layout entries, got {override_count}"
        );
        // Each must reference the correct path pattern
        for n in 1..=31u32 {
            let expected_path = format!("/ppt/slideLayouts/slideLayout{n}.xml");
            assert!(
                entries.contains(&expected_path),
                "content_types must contain PartName=\"{expected_path}\""
            );
        }
    }

    /// serialize_layout_to_xml — output contains `<p:sldLayout>` root element.
    /// Tests that todo!() body panics (Red Gate).
    #[test]
    fn test_layout_xml_serializes_with_valid_xml() {
        let layout = minimal_light_layout();
        let xml_bytes = serialize_layout_to_xml(&layout, "rId1");
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("<p:sldLayout"),
            "output must contain <p:sldLayout> root element, got: {}",
            &xml[..xml.len().min(200)]
        );
        // Must NOT contain raw browser-hostile elements
        assert!(
            !xml.contains("<foreignObject"),
            "layout XML must not contain <foreignObject>"
        );
        assert!(
            !xml.contains("<script"),
            "layout XML must not contain <script>"
        );
    }

    /// BC-2.01.005 / AC-009 — dark layout XML contains `clrMapOvr` element.
    /// Tests that todo!() body panics (Red Gate).
    #[test]
    fn test_bc_2_01_005_dark_layout_xml_has_clr_map_ovr() {
        let layout = minimal_dark_layout();
        let xml_bytes = serialize_layout_to_xml(&layout, "rId1");
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("clrMapOvr"),
            "dark layout XML must contain 'clrMapOvr', got: {}",
            &xml[..xml.len().min(300)]
        );
        assert!(
            xml.contains("overrideClrMapping"),
            "dark layout XML must contain 'overrideClrMapping'"
        );
    }

    /// BC-2.01.005 / AC-009 — dark layout XML contains explicit white text color
    /// (`<a:srgbClr val="FFFFFF"/>`) for LibreOffice 7.x compatibility (R4 mitigation).
    /// Tests that todo!() body panics (Red Gate).
    #[test]
    fn test_bc_2_01_005_dark_layout_xml_has_explicit_white_text() {
        let layout = minimal_dark_layout();
        let xml_bytes = serialize_layout_to_xml(&layout, "rId1");
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("FFFFFF"),
            "dark layout XML must contain explicit white text (FFFFFF), got: {}",
            &xml[..xml.len().min(400)]
        );
        assert!(
            xml.contains("srgbClr"),
            "dark layout XML must contain <a:srgbClr> for explicit white text"
        );
    }

    /// AC-010 — layout XML does NOT use "Shape N" placeholder names.
    /// Tests that todo!() body panics (Red Gate).
    #[test]
    fn test_bc_2_01_005_layout_xml_uses_semantic_names() {
        let layout = minimal_light_layout();
        let xml_bytes = serialize_layout_to_xml(&layout, "rId1");
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        // Must not contain the forbidden "Shape N" pattern
        assert!(
            !xml.contains(r#"name="Shape "#),
            "layout XML must not use 'Shape N' accessibility names; got snippet: {}",
            &xml[..xml.len().min(300)]
        );
        // The semantic name from the placeholder definition must appear
        assert!(
            xml.contains("Title Placeholder"),
            "layout XML must contain the semantic accessibility name 'Title Placeholder'"
        );
    }

    /// AC-010 — non-dark layout XML does NOT contain `clrMapOvr`.
    /// Tests that todo!() body panics (Red Gate).
    #[test]
    fn test_bc_2_01_005_light_layout_xml_has_no_clr_map_ovr() {
        let layout = minimal_light_layout();
        let xml_bytes = serialize_layout_to_xml(&layout, "rId1");
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            !xml.contains("clrMapOvr"),
            "non-dark layout XML must NOT contain clrMapOvr"
        );
    }
}
