// All `.expect()` calls in this module are against an in-memory `Cursor<Vec<u8>>`.
// `quick_xml::Writer` never returns I/O errors for in-memory Cursors, so these
// paths are infallible. A clippy::expect_used suppress here is correct.
#![allow(clippy::expect_used)]
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

use std::fmt::Write as FmtWrite;
use std::io::Cursor;

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

use crate::layouts::{LayoutPlaceholder, SlideLayoutDef};

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

// ─── Namespace constants ─────────────────────────────────────────────────────

const NS_P: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const NS_A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
/// Relationship namespace — used by the PPTX exporter (STORY-037) when building
/// `.rels` files. Not emitted in layout XML body (no r:-prefixed attributes used).
pub const NS_R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

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
/// # Panics
///
/// In practice this function never panics. `quick_xml::Writer` with an in-memory
/// `Cursor<Vec<u8>>` does not return I/O errors, so all `.expect("...")` calls
/// are infallible. A panic would indicate a bug in `quick_xml` itself.
#[must_use]
pub fn serialize_layout_to_xml(def: &SlideLayoutDef) -> Vec<u8> {
    let buf = Cursor::new(Vec::new());
    let mut writer = Writer::new(buf);

    // XML declaration
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .expect("write xml decl");

    // <p:sldLayout> root element with namespaces
    // xmlns:r is intentionally omitted — no r:-prefixed attributes are used in layout XML.
    // The master relationship is expressed in the .rels sidecar, not in the layout body.
    let mut root = BytesStart::new("p:sldLayout");
    root.push_attribute(("xmlns:p", NS_P));
    root.push_attribute(("xmlns:a", NS_A));
    // type attribute: standard layouts use OOXML enum value; custom layouts use "cust"
    // (ECMA-376 §19.7.13 ST_SlideLayoutType: valid value is "cust", not "custom").
    let ooxml_type_str = def.ooxml_type.as_deref().unwrap_or("cust").to_owned();
    root.push_attribute(("type", ooxml_type_str.as_str()));
    // preserve attribute (required for layouts that should preserve master formatting)
    root.push_attribute(("preserve", "1"));
    writer
        .write_event(Event::Start(root))
        .expect("write sldLayout start");

    // <p:cSld name="...">
    let mut csld = BytesStart::new("p:cSld");
    csld.push_attribute(("name", def.name.as_ref()));
    writer
        .write_event(Event::Start(csld))
        .expect("write cSld start");

    // <p:spTree>
    writer
        .write_event(Event::Start(BytesStart::new("p:spTree")))
        .expect("write spTree start");

    // <p:nvGrpSpPr> — required group non-visual properties (ECMA-376 order)
    write_nvgrpsppr(&mut writer);

    // <p:grpSpPr> — required group shape properties (ECMA-376 order)
    write_grpsppr(&mut writer);

    // Write each placeholder as <p:sp>
    for (sp_id, ph) in def.placeholders.iter().enumerate() {
        // sp id starts at 2 (1 is reserved for the group shape).
        // Placeholder count is bounded by layout design (at most a dozen per layout),
        // so sp_id + 2 always fits in u32.
        let sp_id_u32 = u32::try_from(sp_id + 2).expect("placeholder sp_id always fits in u32");
        write_placeholder(&mut writer, ph, sp_id_u32, def.has_color_override);
    }

    // </p:spTree>
    writer
        .write_event(Event::End(BytesEnd::new("p:spTree")))
        .expect("write spTree end");

    // </p:cSld>
    writer
        .write_event(Event::End(BytesEnd::new("p:cSld")))
        .expect("write cSld end");

    // <p:clrMapOvr> for dark layouts (AC-009)
    if def.has_color_override {
        write_clr_map_ovr(&mut writer);
    }

    // <p:hf> — header/footer (required element in schema order)
    writer
        .write_event(Event::Empty(BytesStart::new("p:hf")))
        .expect("write hf");

    // NOTE: <p:txStyles> is NOT emitted here.
    // ECMA-376 §19.3.1.39: <p:txStyles> is a child of <p:sldMaster>, NOT <p:sldLayout>.
    // Emitting it in layout XML produces schema-invalid output. Text styles for layouts
    // are inherited from the slide master (generated in STORY-040).

    // </p:sldLayout>
    writer
        .write_event(Event::End(BytesEnd::new("p:sldLayout")))
        .expect("write sldLayout end");

    writer.into_inner().into_inner()
}

/// Write `<p:nvGrpSpPr>` required by ECMA-376 layout schema.
fn write_nvgrpsppr(writer: &mut Writer<Cursor<Vec<u8>>>) {
    writer
        .write_event(Event::Start(BytesStart::new("p:nvGrpSpPr")))
        .expect("write nvGrpSpPr start");

    let mut cnvpr = BytesStart::new("p:cNvPr");
    cnvpr.push_attribute(("id", "1"));
    cnvpr.push_attribute(("name", ""));
    writer
        .write_event(Event::Empty(cnvpr))
        .expect("write cNvPr");

    writer
        .write_event(Event::Empty(BytesStart::new("p:cNvGrpSpPr")))
        .expect("write cNvGrpSpPr");

    writer
        .write_event(Event::Empty(BytesStart::new("p:nvPr")))
        .expect("write nvPr");

    writer
        .write_event(Event::End(BytesEnd::new("p:nvGrpSpPr")))
        .expect("write nvGrpSpPr end");
}

/// Write `<p:grpSpPr>` with identity transform required by ECMA-376.
fn write_grpsppr(writer: &mut Writer<Cursor<Vec<u8>>>) {
    writer
        .write_event(Event::Start(BytesStart::new("p:grpSpPr")))
        .expect("write grpSpPr start");

    writer
        .write_event(Event::Start(BytesStart::new("a:xfrm")))
        .expect("write xfrm start");

    let mut off = BytesStart::new("a:off");
    off.push_attribute(("x", "0"));
    off.push_attribute(("y", "0"));
    writer.write_event(Event::Empty(off)).expect("write off");

    let mut ext = BytesStart::new("a:ext");
    ext.push_attribute(("cx", "0"));
    ext.push_attribute(("cy", "0"));
    writer.write_event(Event::Empty(ext)).expect("write ext");

    let mut child_off = BytesStart::new("a:chOff");
    child_off.push_attribute(("x", "0"));
    child_off.push_attribute(("y", "0"));
    writer
        .write_event(Event::Empty(child_off))
        .expect("write chOff");

    let mut child_ext = BytesStart::new("a:chExt");
    child_ext.push_attribute(("cx", "0"));
    child_ext.push_attribute(("cy", "0"));
    writer
        .write_event(Event::Empty(child_ext))
        .expect("write chExt");

    writer
        .write_event(Event::End(BytesEnd::new("a:xfrm")))
        .expect("write xfrm end");

    writer
        .write_event(Event::End(BytesEnd::new("p:grpSpPr")))
        .expect("write grpSpPr end");
}

/// Write a `<p:sp>` placeholder element in ECMA-376 schema order.
///
/// For dark layouts, adds explicit white `<a:solidFill>` on run elements
/// for `LibreOffice` 7.x compatibility (R4 mitigation, AC-009).
fn write_placeholder(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    ph: &LayoutPlaceholder,
    sp_id: u32,
    dark_layout: bool,
) {
    writer
        .write_event(Event::Start(BytesStart::new("p:sp")))
        .expect("write sp start");

    // <p:nvSpPr>
    writer
        .write_event(Event::Start(BytesStart::new("p:nvSpPr")))
        .expect("write nvSpPr start");

    // <p:cNvPr id="N" name="Semantic Name"/>
    let sp_id_str = sp_id.to_string();
    let mut cnvpr = BytesStart::new("p:cNvPr");
    cnvpr.push_attribute(("id", sp_id_str.as_str()));
    cnvpr.push_attribute(("name", ph.accessibility_name.as_ref()));
    writer
        .write_event(Event::Empty(cnvpr))
        .expect("write cNvPr");

    // <p:cNvSpPr>
    writer
        .write_event(Event::Start(BytesStart::new("p:cNvSpPr")))
        .expect("write cNvSpPr start");

    // <a:spLocks noGrp="1"/>
    let mut locks = BytesStart::new("a:spLocks");
    locks.push_attribute(("noGrp", "1"));
    writer
        .write_event(Event::Empty(locks))
        .expect("write spLocks");

    writer
        .write_event(Event::End(BytesEnd::new("p:cNvSpPr")))
        .expect("write cNvSpPr end");

    // <p:nvPr>
    writer
        .write_event(Event::Start(BytesStart::new("p:nvPr")))
        .expect("write nvPr start");

    // <p:ph type="..." idx="..."/>
    let idx_str = ph.idx.to_string();
    let mut ph_el = BytesStart::new("p:ph");
    // The title placeholder with idx=0 doesn't need explicit idx attribute
    // (it defaults to 0), but we include it for clarity.
    ph_el.push_attribute(("type", ph.ph_type.as_ref()));
    if ph.idx > 0 {
        ph_el.push_attribute(("idx", idx_str.as_str()));
    }
    writer.write_event(Event::Empty(ph_el)).expect("write ph");

    writer
        .write_event(Event::End(BytesEnd::new("p:nvPr")))
        .expect("write nvPr end");

    writer
        .write_event(Event::End(BytesEnd::new("p:nvSpPr")))
        .expect("write nvSpPr end");

    // <p:spPr> — shape properties with position/size
    writer
        .write_event(Event::Start(BytesStart::new("p:spPr")))
        .expect("write spPr start");

    // <a:xfrm>
    writer
        .write_event(Event::Start(BytesStart::new("a:xfrm")))
        .expect("write xfrm start");

    let x_str = ph.x.to_string();
    let y_str = ph.y.to_string();
    let width_str = ph.cx.to_string();
    let height_str = ph.cy.to_string();

    let mut off = BytesStart::new("a:off");
    off.push_attribute(("x", x_str.as_str()));
    off.push_attribute(("y", y_str.as_str()));
    writer.write_event(Event::Empty(off)).expect("write off");

    let mut ext = BytesStart::new("a:ext");
    ext.push_attribute(("cx", width_str.as_str()));
    ext.push_attribute(("cy", height_str.as_str()));
    writer.write_event(Event::Empty(ext)).expect("write ext");

    writer
        .write_event(Event::End(BytesEnd::new("a:xfrm")))
        .expect("write xfrm end");

    // <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
    let mut prst = BytesStart::new("a:prstGeom");
    prst.push_attribute(("prst", "rect"));
    writer
        .write_event(Event::Start(prst))
        .expect("write prstGeom start");
    writer
        .write_event(Event::Empty(BytesStart::new("a:avLst")))
        .expect("write avLst");
    writer
        .write_event(Event::End(BytesEnd::new("a:prstGeom")))
        .expect("write prstGeom end");

    writer
        .write_event(Event::End(BytesEnd::new("p:spPr")))
        .expect("write spPr end");

    // <p:txBody> with explicit white text for dark layouts (R4 mitigation)
    if dark_layout {
        write_dark_txbody(writer);
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:sp")))
        .expect("write sp end");
}

/// Write `<p:txBody>` with explicit white text for dark layouts (AC-009, R4 mitigation).
///
/// `LibreOffice` 7.x has partial `clrMapOvr` support. Adding explicit white color
/// on run properties ensures text is visible on dark backgrounds.
fn write_dark_txbody(writer: &mut Writer<Cursor<Vec<u8>>>) {
    writer
        .write_event(Event::Start(BytesStart::new("p:txBody")))
        .expect("write txBody start");

    // <a:bodyPr/>
    writer
        .write_event(Event::Empty(BytesStart::new("a:bodyPr")))
        .expect("write bodyPr");

    // <a:lstStyle> with defRPr white defaults for lvl1pPr..lvl5pPr.
    // Empty lstStyle causes LibreOffice 7.x to ignore clrMapOvr and render
    // user-typed text in the default (dark) color. Explicit defRPr on each
    // paragraph level forces white inheritance even with partial clrMapOvr
    // support (R4 mitigation, F-PASS3-M1).
    writer
        .write_event(Event::Start(BytesStart::new("a:lstStyle")))
        .expect("write lstStyle start");
    for lvl in 1..=5u8 {
        let pp_tag = format!("a:lvl{lvl}pPr");
        writer
            .write_event(Event::Start(BytesStart::new(pp_tag.clone())))
            .expect("write lvlNpPr start");
        writer
            .write_event(Event::Start(BytesStart::new("a:defRPr")))
            .expect("write defRPr start");
        writer
            .write_event(Event::Start(BytesStart::new("a:solidFill")))
            .expect("write solidFill start");
        let mut srgb_def = BytesStart::new("a:srgbClr");
        srgb_def.push_attribute(("val", "FFFFFF"));
        writer
            .write_event(Event::Empty(srgb_def))
            .expect("write srgbClr");
        writer
            .write_event(Event::End(BytesEnd::new("a:solidFill")))
            .expect("write solidFill end");
        writer
            .write_event(Event::End(BytesEnd::new("a:defRPr")))
            .expect("write defRPr end");
        writer
            .write_event(Event::End(BytesEnd::new(&pp_tag)))
            .expect("write lvlNpPr end");
    }
    writer
        .write_event(Event::End(BytesEnd::new("a:lstStyle")))
        .expect("write lstStyle end");

    // <a:p>
    writer
        .write_event(Event::Start(BytesStart::new("a:p")))
        .expect("write a:p start");

    // <a:r>
    writer
        .write_event(Event::Start(BytesStart::new("a:r")))
        .expect("write a:r start");

    // <a:rPr lang="en-US" dirty="0"> with explicit white fill
    let mut rpr = BytesStart::new("a:rPr");
    rpr.push_attribute(("lang", "en-US"));
    rpr.push_attribute(("dirty", "0"));
    writer
        .write_event(Event::Start(rpr))
        .expect("write rPr start");

    // <a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill>
    writer
        .write_event(Event::Start(BytesStart::new("a:solidFill")))
        .expect("write solidFill start");

    let mut srgb = BytesStart::new("a:srgbClr");
    srgb.push_attribute(("val", "FFFFFF"));
    writer
        .write_event(Event::Empty(srgb))
        .expect("write srgbClr");

    writer
        .write_event(Event::End(BytesEnd::new("a:solidFill")))
        .expect("write solidFill end");

    writer
        .write_event(Event::End(BytesEnd::new("a:rPr")))
        .expect("write rPr end");

    // <a:t/> (empty text run)
    writer
        .write_event(Event::Start(BytesStart::new("a:t")))
        .expect("write a:t start");
    writer
        .write_event(Event::Text(BytesText::new("")))
        .expect("write empty text");
    writer
        .write_event(Event::End(BytesEnd::new("a:t")))
        .expect("write a:t end");

    writer
        .write_event(Event::End(BytesEnd::new("a:r")))
        .expect("write a:r end");

    writer
        .write_event(Event::End(BytesEnd::new("a:p")))
        .expect("write a:p end");

    writer
        .write_event(Event::End(BytesEnd::new("p:txBody")))
        .expect("write txBody end");
}

/// Write `<p:clrMapOvr>` for dark layouts (AC-009, R2 finding).
///
/// All 8 color map tokens are explicitly specified with the override values
/// for `bg1="dk2" tx1="lt1"` and standard passthrough for the rest.
fn write_clr_map_ovr(writer: &mut Writer<Cursor<Vec<u8>>>) {
    writer
        .write_event(Event::Start(BytesStart::new("p:clrMapOvr")))
        .expect("write clrMapOvr start");

    // <a:overrideClrMapping> with full attribute set
    let mut override_el = BytesStart::new("a:overrideClrMapping");
    override_el.push_attribute(("bg1", "dk2"));
    override_el.push_attribute(("tx1", "lt1"));
    override_el.push_attribute(("bg2", "lt2"));
    override_el.push_attribute(("tx2", "dk2"));
    override_el.push_attribute(("accent1", "accent1"));
    override_el.push_attribute(("accent2", "accent2"));
    override_el.push_attribute(("accent3", "accent3"));
    override_el.push_attribute(("accent4", "accent4"));
    override_el.push_attribute(("accent5", "accent5"));
    override_el.push_attribute(("accent6", "accent6"));
    override_el.push_attribute(("hlink", "hlink"));
    override_el.push_attribute(("folHlink", "folHlink"));
    writer
        .write_event(Event::Empty(override_el))
        .expect("write overrideClrMapping");

    writer
        .write_event(Event::End(BytesEnd::new("p:clrMapOvr")))
        .expect("write clrMapOvr end");
}

/// Generate the `[Content_Types].xml` registration string for all 31 layouts.
///
/// Returns a `String` containing one `<Override PartName="...">` entry per
/// layout, covering `slideLayout1.xml` through `slideLayout31.xml`.
///
/// This string is stored in `BrandTemplate` and used by the PPTX exporter
/// (STORY-037) to populate `[Content_Types].xml` (AC-013).
#[must_use]
pub fn generate_content_types_layout_entries(layout_count: usize) -> String {
    let content_type =
        "application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml";

    let mut entries = String::new();
    for n in 1..=layout_count {
        writeln!(
            entries,
            "<Override PartName=\"/ppt/slideLayouts/slideLayout{n}.xml\" ContentType=\"{content_type}\"/>"
        )
        .expect("writeln to String is infallible");
    }
    entries
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
        assert!(
            !NOTES_MASTER_STUB.is_empty(),
            "notesMaster stub must not be empty"
        );
        assert!(
            NOTES_MASTER_STUB.starts_with(b"<?xml"),
            "notesMaster stub must be XML"
        );
    }

    /// AC-012 — `HANDOUT_MASTER_STUB` is non-empty valid XML bytes.
    #[test]
    fn test_bc_2_01_002_handout_master_stub_is_non_empty() {
        assert!(
            !HANDOUT_MASTER_STUB.is_empty(),
            "handoutMaster stub must not be empty"
        );
        assert!(
            HANDOUT_MASTER_STUB.starts_with(b"<?xml"),
            "handoutMaster stub must be XML"
        );
    }

    /// AC-013 — `generate_content_types_layout_entries` produces 31 Override entries.
    #[test]
    fn test_bc_2_01_005_content_types_layout_entries_count() {
        let entries = generate_content_types_layout_entries(31);
        // Must contain exactly 31 <Override> elements (one per layout)
        let override_count = entries.matches("<Override").count();
        assert_eq!(
            override_count, 31,
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

    /// `serialize_layout_to_xml` — output contains `<p:sldLayout>` root element.
    #[test]
    fn test_layout_xml_serializes_with_valid_xml() {
        let layout = minimal_light_layout();
        let xml_bytes = serialize_layout_to_xml(&layout);
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
    #[test]
    fn test_bc_2_01_005_dark_layout_xml_has_clr_map_ovr() {
        let layout = minimal_dark_layout();
        let xml_bytes = serialize_layout_to_xml(&layout);
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
    /// (`<a:srgbClr val="FFFFFF"/>`) for `LibreOffice` 7.x compatibility (R4 mitigation).
    #[test]
    fn test_bc_2_01_005_dark_layout_xml_has_explicit_white_text() {
        let layout = minimal_dark_layout();
        let xml_bytes = serialize_layout_to_xml(&layout);
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
        // R4 mitigation (F-PASS3-M1): lstStyle must have defRPr defaults for all
        // 5 paragraph levels so user-typed text on dark slides inherits white in
        // LibreOffice 7.x (which has partial clrMapOvr support).
        assert!(
            xml.contains("defRPr"),
            "dark layout XML must contain <a:defRPr> in lstStyle for LibreOffice white-text inheritance"
        );
        assert!(
            xml.contains("lvl1pPr"),
            "dark layout XML must contain <a:lvl1pPr> level defaults in lstStyle"
        );
        assert!(
            xml.contains("lvl5pPr"),
            "dark layout XML must contain all 5 level defaults (lvl1pPr..lvl5pPr) in lstStyle"
        );
    }

    /// AC-010 — layout XML does NOT use "Shape N" placeholder names.
    #[test]
    fn test_bc_2_01_005_layout_xml_uses_semantic_names() {
        let layout = minimal_light_layout();
        let xml_bytes = serialize_layout_to_xml(&layout);
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
    #[test]
    fn test_bc_2_01_005_light_layout_xml_has_no_clr_map_ovr() {
        let layout = minimal_light_layout();
        let xml_bytes = serialize_layout_to_xml(&layout);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            !xml.contains("clrMapOvr"),
            "non-dark layout XML must NOT contain clrMapOvr"
        );
    }

    /// F2 — custom layout uses OOXML enum "cust" (not "custom").
    #[test]
    fn test_f2_custom_layout_type_is_cust_not_custom() {
        let layout = minimal_dark_layout(); // ooxml_type = None → "cust"
        let xml_bytes = serialize_layout_to_xml(&layout);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("type=\"cust\""),
            "custom layout must use type=\"cust\" (ECMA-376 §19.7.13), got: {}",
            &xml[..xml.len().min(300)]
        );
        assert!(
            !xml.contains("type=\"custom\""),
            "custom layout must NOT use type=\"custom\" (not a valid ECMA-376 enum value)"
        );
    }

    /// F2 — standard layout type is preserved as-is (e.g. "title").
    #[test]
    fn test_f2_standard_layout_type_is_preserved() {
        let layout = minimal_light_layout(); // ooxml_type = Some("title")
        let xml_bytes = serialize_layout_to_xml(&layout);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("type=\"title\""),
            "standard layout must preserve ooxml_type value, got: {}",
            &xml[..xml.len().min(300)]
        );
    }

    /// F1 — serialized layout XML must NOT contain `<p:txStyles>`.
    ///
    /// ECMA-376 §19.3.1.39: txStyles is only valid inside sldMaster, NOT sldLayout.
    #[test]
    fn test_f1_layout_xml_does_not_contain_txstyles() {
        let layout = minimal_light_layout();
        let xml_bytes = serialize_layout_to_xml(&layout);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            !xml.contains("txStyles"),
            "F1: layout XML must NOT contain txStyles (only valid in sldMaster), got: {xml}"
        );
    }

    /// F11 — snapshot test of full standard layout XML.
    ///
    /// This snapshot would have caught both F1 (txStyles) and F2 (custom vs cust)
    /// before they became adversarial findings. Blessed via `cargo insta accept`.
    #[test]
    fn test_f11_snapshot_standard_layout_xml() {
        let layout = minimal_light_layout();
        let xml_bytes = serialize_layout_to_xml(&layout);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        insta::assert_snapshot!("standard_layout_xml", xml);
    }

    /// F11 — snapshot test of full dark layout XML.
    ///
    /// Verifies clrMapOvr, explicit white text, and absence of txStyles.
    #[test]
    fn test_f11_snapshot_dark_layout_xml() {
        let layout = minimal_dark_layout();
        let xml_bytes = serialize_layout_to_xml(&layout);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        insta::assert_snapshot!("dark_layout_xml", xml);
    }

    /// F-PASS13-MED-2 — `quick_xml` escapes special characters in `accessibility_name`.
    ///
    /// Even though no current user-facing code path populates `accessibility_name` with
    /// user input, this test prevents future regressions: an `accessibility_name`
    /// containing XML-special characters must be properly escaped in the attribute.
    ///
    /// Characters tested: `"` (→ `&quot;`), `<` (→ `&lt;`), `>` (→ `&gt;`),
    /// `&` (→ `&amp;`).
    ///
    /// Also verifies round-trip: the serialized XML parses back without error.
    #[test]
    fn test_xml_attribute_escaping_for_special_characters() {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let layout = SlideLayoutDef {
            index: 1,
            name: Arc::from("Test Layout"),
            ooxml_type: Some(Arc::from("title")),
            placeholders: vec![LayoutPlaceholder {
                ph_type: Arc::from("ctrTitle"),
                idx: 0,
                // Accessibility name with XML-special characters
                accessibility_name: Arc::from(r#"Acme™ "Corp" <Special> & Things"#),
                x: 457_200,
                y: 274_638,
                cx: 8_229_600,
                cy: 1_143_000,
            }],
            has_color_override: false,
            color_override_bg: None,
            color_override_tx: None,
        };
        let xml_bytes = serialize_layout_to_xml(&layout);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");

        // The raw special characters must NOT appear unescaped inside attribute values.
        // `quick_xml` should emit `&quot;` for `"`, `&lt;` for `<`, `&amp;` for `&`.
        // Note: `>` inside attributes is technically allowed but often escaped; we
        // test that the XML is well-formed (parseable) rather than a specific escape form.
        assert!(
            !xml.contains(r#"name="Acme™ "Corp""#),
            "unescaped double-quote must not appear in attribute value; xml snippet: {}",
            &xml[..xml.len().min(500)]
        );

        // Round-trip: the serialized XML must be parseable by quick_xml::Reader.
        let mut reader = Reader::from_str(xml);
        reader.config_mut().check_end_names = true;
        let mut event_count = 0usize;
        loop {
            match reader.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => event_count += 1,
                Err(e) => panic!("XML produced by serialize_layout_to_xml failed to parse: {e}"),
            }
        }
        assert!(event_count > 0, "round-trip parse must produce events");
    }
}
