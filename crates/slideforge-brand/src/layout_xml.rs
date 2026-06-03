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

/// `PresentationML` namespace URI (`p:` prefix in layout XML).
const NS_P: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
/// `DrawingML` namespace URI (`a:` prefix in layout XML).
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

// ─── Master XML serialization ─────────────────────────────────────────────────

/// Canonical OOXML `<a:clrMap>` color-map attribute set for a standard light theme.
///
/// These 8 tokens map OOXML display roles to theme color slots. The slide master
/// element order is defined by ECMA-376 §19.3.1.14 (`CT_ColorMapping`).
const CLR_MAP_ATTRS: &[(&str, &str)] = &[
    ("bg1", "lt1"),
    ("tx1", "dk1"),
    ("bg2", "lt2"),
    ("tx2", "dk2"),
    ("accent1", "accent1"),
    ("accent2", "accent2"),
    ("accent3", "accent3"),
    ("accent4", "accent4"),
    ("accent5", "accent5"),
    ("accent6", "accent6"),
    ("hlink", "hlink"),
    ("folHlink", "folHlink"),
];

/// The 5 canonical master placeholder types required by ECMA-376 §19.3.1.27.
///
/// Each entry is `(ph_type, idx, accessibility_name, x, y, cx, cy)` in EMU.
/// Geometry matches standard 10-inch wide × 7.5-inch tall slide canvas (914400 EMU/inch).
const MASTER_PLACEHOLDER_DEFS: &[(&str, u32, &str, i64, i64, i64, i64)] = &[
    // title: full-width title region
    (
        "title",
        0,
        "Title Placeholder",
        457_200,
        274_638,
        8_229_600,
        1_143_000,
    ),
    // body: main content region
    (
        "body",
        1,
        "Content Placeholder",
        457_200,
        1_600_200,
        8_229_600,
        4_525_963,
    ),
    // dt: date/time footer — bottom left
    (
        "dt",
        10,
        "Date Placeholder",
        457_200,
        6_356_350,
        2_286_000,
        365_125,
    ),
    // ftr: footer text — bottom center
    (
        "ftr",
        11,
        "Footer Placeholder",
        3_657_600,
        6_356_350,
        2_743_200,
        365_125,
    ),
    // sldNum: slide number — bottom right
    (
        "sldNum",
        12,
        "Slide Number Placeholder",
        7_086_000,
        6_356_350,
        1_600_200,
        365_125,
    ),
];

/// Serialize `BrandTemplate` data to a complete `slideMaster1.xml` document.
///
/// The returned `Vec<u8>` is a valid `slideMaster1.xml` document containing:
/// - `<p:sldMaster>` root with `PresentationML` + `DrawingML` namespace declarations.
/// - `<p:cSld><p:spTree>` with 5 master placeholder shapes (title, body, dt, ftr, sldNum).
/// - `<a:clrMap>` with all 12 OOXML color-map tokens (ECMA-376 §19.3.1.14).
/// - `<p:sldLayoutIdLst>` with one `<p:sldLayoutId>` per layout in `template.layouts`
///   (IDs from `template.master_ids.layout_id_start` onwards).
/// - `<p:hf>` footer visibility flags from `template.footer_flags`.
/// - `<p:txStyles>` with heading/body font names from `template.fonts`.
///
/// Element order follows ECMA-376 §19.3.1.42 CT_SlideMaster sequence model:
/// `cSld, clrMap, sldLayoutIdLst, hf, txStyles` (ADR-015 §A.3).
///
/// # Panics
///
/// In practice this function never panics. `quick_xml::Writer` with an in-memory
/// `Cursor<Vec<u8>>` does not return I/O errors.
#[must_use]
pub fn serialize_master_to_xml(template: &crate::template::BrandTemplate) -> Vec<u8> {
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

    // <p:sldMaster> root element with namespace declarations
    let mut root = BytesStart::new("p:sldMaster");
    root.push_attribute(("xmlns:p", NS_P));
    root.push_attribute(("xmlns:a", NS_A));
    root.push_attribute(("xmlns:r", NS_R));
    writer
        .write_event(Event::Start(root))
        .expect("write sldMaster start");

    // <p:cSld>
    writer
        .write_event(Event::Start(BytesStart::new("p:cSld")))
        .expect("write cSld start");

    // <p:spTree>
    writer
        .write_event(Event::Start(BytesStart::new("p:spTree")))
        .expect("write spTree start");

    write_nvgrpsppr(&mut writer);
    write_grpsppr(&mut writer);

    // Write the 5 master placeholder shapes
    for (sp_idx, (ph_type, idx, name, x, y, cx, cy)) in MASTER_PLACEHOLDER_DEFS.iter().enumerate() {
        // sp id starts at 2 (1 reserved for group shape)
        let sp_id = u32::try_from(sp_idx + 2).expect("sp_id always fits");
        write_master_placeholder(&mut writer, ph_type, *idx, name, sp_id, *x, *y, *cx, *cy);
    }

    // </p:spTree>
    writer
        .write_event(Event::End(BytesEnd::new("p:spTree")))
        .expect("write spTree end");

    // </p:cSld>
    writer
        .write_event(Event::End(BytesEnd::new("p:cSld")))
        .expect("write cSld end");

    // <a:clrMap> — required ECMA-376 §19.3.1.14
    let mut clr_map = BytesStart::new("a:clrMap");
    for (attr, val) in CLR_MAP_ATTRS {
        clr_map.push_attribute((*attr, *val));
    }
    writer
        .write_event(Event::Empty(clr_map))
        .expect("write clrMap");

    // <p:sldLayoutIdLst> with one entry per layout
    writer
        .write_event(Event::Start(BytesStart::new("p:sldLayoutIdLst")))
        .expect("write sldLayoutIdLst start");

    let layout_id_start = template.master_ids.layout_id_start;
    for (i, _layout) in template.layouts.iter().enumerate() {
        let layout_id = layout_id_start + u32::try_from(i).expect("layout idx fits");
        let id_str = layout_id.to_string();
        // rId matches the master's .rels file: rId1 for theme, then rId{2..=32} for layouts
        // The layout rId in .rels is assigned sequentially starting at rId2 (rId1=theme).
        let rid_n = u32::try_from(i + 2).expect("rid fits");
        let rid_str = format!("rId{rid_n}");
        let mut sld_layout_id = BytesStart::new("p:sldLayoutId");
        sld_layout_id.push_attribute(("id", id_str.as_str()));
        sld_layout_id.push_attribute(("r:id", rid_str.as_str()));
        writer
            .write_event(Event::Empty(sld_layout_id))
            .expect("write sldLayoutId");
    }

    writer
        .write_event(Event::End(BytesEnd::new("p:sldLayoutIdLst")))
        .expect("write sldLayoutIdLst end");

    // ECMA-376 §19.3.1.42 CT_SlideMaster sequence model: cSld, clrMap, sldLayoutIdLst, hf, txStyles
    // <p:hf> MUST precede <p:txStyles> — wrong order causes repair dialogs in PowerPoint/Keynote.
    // (ADR-015 §A.3 — F-PASS2-H1 fix)

    // <p:hf> — footer visibility flags
    write_master_hf(&mut writer, &template.footer_flags);

    // <p:txStyles> — heading and body font definitions
    write_master_tx_styles(&mut writer, template);

    // </p:sldMaster>
    writer
        .write_event(Event::End(BytesEnd::new("p:sldMaster")))
        .expect("write sldMaster end");

    writer.into_inner().into_inner()
}

/// Write a single master placeholder `<p:sp>` element.
// 9 arguments is one over the clippy::pedantic limit of 8 — all are required for
// ECMA-376 master placeholder serialization with no meaningful grouping alternative.
#[allow(clippy::too_many_arguments)]
fn write_master_placeholder(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    ph_type: &str,
    idx: u32,
    name: &str,
    sp_id: u32,
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
) {
    writer
        .write_event(Event::Start(BytesStart::new("p:sp")))
        .expect("write sp start");

    // <p:nvSpPr>
    writer
        .write_event(Event::Start(BytesStart::new("p:nvSpPr")))
        .expect("write nvSpPr start");

    let sp_id_str = sp_id.to_string();
    let mut cnvpr = BytesStart::new("p:cNvPr");
    cnvpr.push_attribute(("id", sp_id_str.as_str()));
    cnvpr.push_attribute(("name", name));
    writer
        .write_event(Event::Empty(cnvpr))
        .expect("write cNvPr");

    writer
        .write_event(Event::Start(BytesStart::new("p:cNvSpPr")))
        .expect("write cNvSpPr start");
    let mut locks = BytesStart::new("a:spLocks");
    locks.push_attribute(("noGrp", "1"));
    writer
        .write_event(Event::Empty(locks))
        .expect("write spLocks");
    writer
        .write_event(Event::End(BytesEnd::new("p:cNvSpPr")))
        .expect("write cNvSpPr end");

    writer
        .write_event(Event::Start(BytesStart::new("p:nvPr")))
        .expect("write nvPr start");

    // <p:ph type="..." idx="..."/> — idx=0 for title omits idx per ECMA convention
    let idx_str = idx.to_string();
    let mut ph_el = BytesStart::new("p:ph");
    ph_el.push_attribute(("type", ph_type));
    if idx > 0 {
        ph_el.push_attribute(("idx", idx_str.as_str()));
    }
    writer.write_event(Event::Empty(ph_el)).expect("write ph");

    writer
        .write_event(Event::End(BytesEnd::new("p:nvPr")))
        .expect("write nvPr end");
    writer
        .write_event(Event::End(BytesEnd::new("p:nvSpPr")))
        .expect("write nvSpPr end");

    // <p:spPr> with position/size
    writer
        .write_event(Event::Start(BytesStart::new("p:spPr")))
        .expect("write spPr start");

    writer
        .write_event(Event::Start(BytesStart::new("a:xfrm")))
        .expect("write xfrm start");
    let mut off = BytesStart::new("a:off");
    off.push_attribute(("x", x.to_string().as_str()));
    off.push_attribute(("y", y.to_string().as_str()));
    writer.write_event(Event::Empty(off)).expect("write off");
    let mut ext = BytesStart::new("a:ext");
    ext.push_attribute(("cx", cx.to_string().as_str()));
    ext.push_attribute(("cy", cy.to_string().as_str()));
    writer.write_event(Event::Empty(ext)).expect("write ext");
    writer
        .write_event(Event::End(BytesEnd::new("a:xfrm")))
        .expect("write xfrm end");

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

    writer
        .write_event(Event::End(BytesEnd::new("p:sp")))
        .expect("write sp end");
}

/// Write `<p:txStyles>` with heading and body font names from `template.fonts`.
fn write_master_tx_styles(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    template: &crate::template::BrandTemplate,
) {
    writer
        .write_event(Event::Start(BytesStart::new("p:txStyles")))
        .expect("write txStyles start");

    // <p:titleStyle>
    writer
        .write_event(Event::Start(BytesStart::new("p:titleStyle")))
        .expect("write titleStyle start");
    write_tx_style_level(writer, template.fonts.heading.as_ref(), 1);
    writer
        .write_event(Event::End(BytesEnd::new("p:titleStyle")))
        .expect("write titleStyle end");

    // <p:bodyStyle>
    writer
        .write_event(Event::Start(BytesStart::new("p:bodyStyle")))
        .expect("write bodyStyle start");
    for lvl in 1..=5u8 {
        write_tx_style_level(writer, template.fonts.body.as_ref(), lvl);
    }
    writer
        .write_event(Event::End(BytesEnd::new("p:bodyStyle")))
        .expect("write bodyStyle end");

    // <p:otherStyle>
    writer
        .write_event(Event::Start(BytesStart::new("p:otherStyle")))
        .expect("write otherStyle start");
    write_tx_style_level(writer, template.fonts.body.as_ref(), 1);
    writer
        .write_event(Event::End(BytesEnd::new("p:otherStyle")))
        .expect("write otherStyle end");

    writer
        .write_event(Event::End(BytesEnd::new("p:txStyles")))
        .expect("write txStyles end");
}

/// Write a single `<a:lvlNpPr>` within a txStyles paragraph style list.
fn write_tx_style_level(writer: &mut Writer<Cursor<Vec<u8>>>, typeface: &str, lvl: u8) {
    let tag = format!("a:lvl{lvl}pPr");

    writer
        .write_event(Event::Start(BytesStart::new(tag.clone())))
        .expect("write lvlNpPr start");

    writer
        .write_event(Event::Start(BytesStart::new("a:defRPr")))
        .expect("write defRPr start");

    let mut latin = BytesStart::new("a:latin");
    latin.push_attribute(("typeface", typeface));
    writer
        .write_event(Event::Empty(latin))
        .expect("write latin");

    writer
        .write_event(Event::End(BytesEnd::new("a:defRPr")))
        .expect("write defRPr end");

    writer
        .write_event(Event::End(BytesEnd::new(&tag)))
        .expect("write lvlNpPr end");
}

/// Write `<p:hf>` footer visibility flags.
fn write_master_hf(writer: &mut Writer<Cursor<Vec<u8>>>, flags: &crate::footer::FooterFlags) {
    let mut hf = BytesStart::new("p:hf");
    let ftr_str = if flags.show_footer { "1" } else { "0" };
    let dt_str = if flags.show_date { "1" } else { "0" };
    let sld_num_str = if flags.show_slide_number { "1" } else { "0" };
    hf.push_attribute(("ftr", ftr_str));
    hf.push_attribute(("dt", dt_str));
    hf.push_attribute(("sldNum", sld_num_str));
    writer.write_event(Event::Empty(hf)).expect("write hf");
}

// ─── Theme XML serialization ───────────────────────────────────────────────────

/// OOXML element names for the 12 theme color slots in ECMA-376 order.
///
/// These are the element names used inside `<a:clrScheme>` — they differ from
/// the `COLOR_SLOT_NAMES` keys (`acc1`–`acc6`) which use abbreviated forms,
/// while `<a:clrScheme>` uses the full `accent1`–`accent6` element names.
const THEME_COLOR_ELEMENT_NAMES: [&str; 12] = [
    "a:dk1",
    "a:lt1",
    "a:dk2",
    "a:lt2",
    "a:accent1",
    "a:accent2",
    "a:accent3",
    "a:accent4",
    "a:accent5",
    "a:accent6",
    "a:hlink",
    "a:folHlink",
];

/// Serialize `BrandTemplate` color and font data to a complete `theme1.xml` document.
///
/// The returned `Vec<u8>` is a valid `theme1.xml` document conforming to
/// ECMA-376 §20.1.6.9 (`<a:theme>`), containing:
/// - `<a:clrScheme>` with all 12 color slots in `COLOR_SLOT_NAMES` order.
/// - `<a:fontScheme>` with `<a:majorFont>` (heading) and `<a:minorFont>` (body).
/// - `<a:fmtScheme>` with minimal valid fill, line, effect, and bg fill style lists.
///
/// # Panics
///
/// In practice this function never panics. `quick_xml::Writer` with an in-memory
/// `Cursor<Vec<u8>>` does not return I/O errors.
#[must_use]
pub fn serialize_theme_to_xml(template: &crate::template::BrandTemplate) -> Vec<u8> {
    let buf = Cursor::new(Vec::new());
    let mut writer = Writer::new(buf);

    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .expect("write xml decl");

    // <a:theme xmlns:a="..." name="slideforge">
    let mut root = BytesStart::new("a:theme");
    root.push_attribute(("xmlns:a", NS_A));
    root.push_attribute(("name", "slideforge"));
    writer
        .write_event(Event::Start(root))
        .expect("write theme start");

    // <a:themeElements>
    writer
        .write_event(Event::Start(BytesStart::new("a:themeElements")))
        .expect("write themeElements start");

    // <a:clrScheme name="slideforge">
    let mut clr_scheme = BytesStart::new("a:clrScheme");
    clr_scheme.push_attribute(("name", "slideforge"));
    writer
        .write_event(Event::Start(clr_scheme))
        .expect("write clrScheme start");

    for (i, color_slot) in template.colors.iter().enumerate() {
        let elem_name = THEME_COLOR_ELEMENT_NAMES[i];

        writer
            .write_event(Event::Start(BytesStart::new(elem_name)))
            .expect("write color slot start");

        // Emit <a:srgbClr val="RRGGBB"/> (strip leading '#' from hex value).
        let hex_val = match &color_slot.value {
            crate::template::ColorValue::Hex(h) => {
                h.strip_prefix('#').unwrap_or(h.as_ref()).to_owned()
            },
            crate::template::ColorValue::SchemeRef(r) => {
                // Scheme refs cannot be resolved here; emit a fallback neutral color
                // and warn. This matches the forbidden-silent-fallback rule: we
                // use a safe neutral and log the anomaly rather than producing
                // an invalid srgbClr element.
                tracing::warn!(
                    slot = %color_slot.name,
                    scheme_ref = r.as_ref(),
                    "theme color slot contains unresolved SchemeRef; \
                     emitting neutral fallback #808080 in theme1.xml"
                );
                "808080".to_owned()
            },
        };

        let mut srgb = BytesStart::new("a:srgbClr");
        srgb.push_attribute(("val", hex_val.as_str()));
        writer
            .write_event(Event::Empty(srgb))
            .expect("write srgbClr");

        writer
            .write_event(Event::End(BytesEnd::new(elem_name)))
            .expect("write color slot end");
    }

    writer
        .write_event(Event::End(BytesEnd::new("a:clrScheme")))
        .expect("write clrScheme end");

    // <a:fontScheme name="slideforge">
    let mut font_scheme = BytesStart::new("a:fontScheme");
    font_scheme.push_attribute(("name", "slideforge"));
    writer
        .write_event(Event::Start(font_scheme))
        .expect("write fontScheme start");

    // <a:majorFont> (heading)
    writer
        .write_event(Event::Start(BytesStart::new("a:majorFont")))
        .expect("write majorFont start");
    let mut latin_major = BytesStart::new("a:latin");
    latin_major.push_attribute(("typeface", template.fonts.heading.as_ref()));
    writer
        .write_event(Event::Empty(latin_major))
        .expect("write latin major");
    writer
        .write_event(Event::Empty(BytesStart::new("a:ea")))
        .expect("write ea");
    writer
        .write_event(Event::Empty(BytesStart::new("a:cs")))
        .expect("write cs");
    writer
        .write_event(Event::End(BytesEnd::new("a:majorFont")))
        .expect("write majorFont end");

    // <a:minorFont> (body)
    writer
        .write_event(Event::Start(BytesStart::new("a:minorFont")))
        .expect("write minorFont start");
    let mut latin_minor = BytesStart::new("a:latin");
    latin_minor.push_attribute(("typeface", template.fonts.body.as_ref()));
    writer
        .write_event(Event::Empty(latin_minor))
        .expect("write latin minor");
    writer
        .write_event(Event::Empty(BytesStart::new("a:ea")))
        .expect("write ea");
    writer
        .write_event(Event::Empty(BytesStart::new("a:cs")))
        .expect("write cs");
    writer
        .write_event(Event::End(BytesEnd::new("a:minorFont")))
        .expect("write minorFont end");

    writer
        .write_event(Event::End(BytesEnd::new("a:fontScheme")))
        .expect("write fontScheme end");

    // <a:fmtScheme name="slideforge"> — minimal valid format scheme (ECMA-376 §20.1.6.9)
    write_minimal_fmt_scheme(&mut writer);

    // </a:themeElements>
    writer
        .write_event(Event::End(BytesEnd::new("a:themeElements")))
        .expect("write themeElements end");

    // </a:theme>
    writer
        .write_event(Event::End(BytesEnd::new("a:theme")))
        .expect("write theme end");

    writer.into_inner().into_inner()
}

/// Write a minimal but schema-valid `<a:fmtScheme>` element.
///
/// ECMA-376 §20.1.6.9 requires `<a:fmtScheme>` to contain `<a:fillStyleLst>`,
/// `<a:lnStyleLst>`, `<a:effectStyleLst>`, and `<a:bgFillStyleLst>` — each
/// with a minimum of 3 entries. This implementation emits 3 solid fills,
/// 3 minimal lines, 3 empty effect styles, and 3 solid background fills.
fn write_minimal_fmt_scheme(writer: &mut Writer<Cursor<Vec<u8>>>) {
    let mut fmt = BytesStart::new("a:fmtScheme");
    fmt.push_attribute(("name", "slideforge"));
    writer
        .write_event(Event::Start(fmt))
        .expect("write fmtScheme start");

    // fillStyleLst — 3 solid fills required
    writer
        .write_event(Event::Start(BytesStart::new("a:fillStyleLst")))
        .expect("write fillStyleLst start");
    for _ in 0..3 {
        writer
            .write_event(Event::Start(BytesStart::new("a:solidFill")))
            .expect("write solidFill start");
        let mut scheme_clr = BytesStart::new("a:schemeClr");
        scheme_clr.push_attribute(("val", "phClr"));
        writer
            .write_event(Event::Empty(scheme_clr))
            .expect("write schemeClr");
        writer
            .write_event(Event::End(BytesEnd::new("a:solidFill")))
            .expect("write solidFill end");
    }
    writer
        .write_event(Event::End(BytesEnd::new("a:fillStyleLst")))
        .expect("write fillStyleLst end");

    // lnStyleLst — 3 lines required (w=6350/12700/19050)
    writer
        .write_event(Event::Start(BytesStart::new("a:lnStyleLst")))
        .expect("write lnStyleLst start");
    for w in [6350u32, 12700, 19050] {
        let w_str = w.to_string();
        let mut ln = BytesStart::new("a:ln");
        ln.push_attribute(("w", w_str.as_str()));
        ln.push_attribute(("cap", "flat"));
        ln.push_attribute(("cmpd", "sng"));
        ln.push_attribute(("algn", "ctr"));
        writer
            .write_event(Event::Start(ln))
            .expect("write ln start");
        writer
            .write_event(Event::Start(BytesStart::new("a:solidFill")))
            .expect("write solidFill start");
        let mut scheme_clr = BytesStart::new("a:schemeClr");
        scheme_clr.push_attribute(("val", "phClr"));
        writer
            .write_event(Event::Empty(scheme_clr))
            .expect("write schemeClr");
        writer
            .write_event(Event::End(BytesEnd::new("a:solidFill")))
            .expect("write solidFill end");
        let mut prst_dash = BytesStart::new("a:prstDash");
        prst_dash.push_attribute(("val", "solid"));
        writer
            .write_event(Event::Empty(prst_dash))
            .expect("write prstDash");
        writer
            .write_event(Event::End(BytesEnd::new("a:ln")))
            .expect("write ln end");
    }
    writer
        .write_event(Event::End(BytesEnd::new("a:lnStyleLst")))
        .expect("write lnStyleLst end");

    // effectStyleLst — 3 empty effect styles required
    writer
        .write_event(Event::Start(BytesStart::new("a:effectStyleLst")))
        .expect("write effectStyleLst start");
    for _ in 0..3 {
        writer
            .write_event(Event::Start(BytesStart::new("a:effectStyle")))
            .expect("write effectStyle start");
        writer
            .write_event(Event::Empty(BytesStart::new("a:effectLst")))
            .expect("write effectLst");
        writer
            .write_event(Event::End(BytesEnd::new("a:effectStyle")))
            .expect("write effectStyle end");
    }
    writer
        .write_event(Event::End(BytesEnd::new("a:effectStyleLst")))
        .expect("write effectStyleLst end");

    // bgFillStyleLst — 3 solid background fills required
    writer
        .write_event(Event::Start(BytesStart::new("a:bgFillStyleLst")))
        .expect("write bgFillStyleLst start");
    for _ in 0..3 {
        writer
            .write_event(Event::Start(BytesStart::new("a:solidFill")))
            .expect("write solidFill start");
        let mut scheme_clr = BytesStart::new("a:schemeClr");
        scheme_clr.push_attribute(("val", "phClr"));
        writer
            .write_event(Event::Empty(scheme_clr))
            .expect("write schemeClr");
        writer
            .write_event(Event::End(BytesEnd::new("a:solidFill")))
            .expect("write solidFill end");
    }
    writer
        .write_event(Event::End(BytesEnd::new("a:bgFillStyleLst")))
        .expect("write bgFillStyleLst end");

    writer
        .write_event(Event::End(BytesEnd::new("a:fmtScheme")))
        .expect("write fmtScheme end");
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
            slide_type_keyword: None, // standard layout — matched by ooxml_type
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
            slide_type_keyword: Some(Arc::from("section_divider")),
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
            slide_type_keyword: None, // standard layout — matched by ooxml_type
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

    // ─── Tests for serialize_master_to_xml ───────────────────────────────────

    /// Helper: build a minimal `BrandTemplate` with 12 color slots, fonts, and
    /// a default `MasterIds`/`FooterFlags` for master/theme XML tests.
    fn minimal_brand_template() -> crate::template::BrandTemplate {
        use crate::template::{BrandFonts, BrandTemplate, ColorSlot, ColorValue, MasterIds};

        let make_slot = |name: &str, hex: &str| ColorSlot {
            name: Arc::from(name),
            value: ColorValue::Hex(Arc::from(hex)),
            is_derived: false,
        };
        BrandTemplate {
            colors: [
                make_slot("dk1", "#000000"),
                make_slot("lt1", "#FFFFFF"),
                make_slot("dk2", "#003087"),
                make_slot("lt2", "#F5F5F5"),
                make_slot("acc1", "#0066CC"),
                make_slot("acc2", "#FF6B35"),
                make_slot("acc3", "#28A745"),
                make_slot("acc4", "#FFC107"),
                make_slot("acc5", "#6F42C1"),
                make_slot("acc6", "#17A2B8"),
                make_slot("hlink", "#0000EE"),
                make_slot("folHlink", "#551A8B"),
            ],
            fonts: BrandFonts {
                heading: Arc::from("Calibri Light"),
                body: Arc::from("Calibri"),
            },
            logo: None,
            footer_text: None,
            footer_flags: crate::footer::FooterFlags::default(),
            layout_names: vec![],
            layouts: vec![SlideLayoutDef {
                index: 1,
                name: Arc::from("Title Slide"),
                ooxml_type: Some(Arc::from("title")),
                slide_type_keyword: None, // standard layout — matched by ooxml_type
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
            }],
            notes_master_stub: NOTES_MASTER_STUB.to_vec(),
            handout_master_stub: HANDOUT_MASTER_STUB.to_vec(),
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(""),
        }
    }

    /// ADR-015 §2 — `serialize_master_to_xml` produces a `<p:sldMaster>` root element.
    #[test]
    fn test_adr015_serialize_master_to_xml_produces_sld_master_root() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_master_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("<p:sldMaster"),
            "serialize_master_to_xml must produce <p:sldMaster> root element; got: {}",
            &xml[..xml.len().min(200)]
        );
    }

    /// ADR-015 §2 — master XML contains `<a:clrMap>` with 12 color-map token attributes.
    #[test]
    fn test_adr015_serialize_master_to_xml_has_clr_map() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_master_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("<a:clrMap"),
            "slideMaster1.xml must contain <a:clrMap> element; got snippet: {}",
            &xml[..xml.len().min(400)]
        );
        // All 8 OOXML standard clrMap tokens must be present.
        for token in &[
            "bg1", "bg2", "tx1", "tx2", "accent1", "accent2", "hlink", "folHlink",
        ] {
            assert!(
                xml.contains(token),
                "clrMap must contain token '{}'; xml snippet: {}",
                token,
                &xml[..xml.len().min(400)]
            );
        }
    }

    /// ADR-015 §2 — master XML contains `<p:sldLayoutIdLst>` with one entry per layout.
    #[test]
    fn test_adr015_serialize_master_to_xml_has_sld_layout_id_lst() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_master_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("<p:sldLayoutIdLst"),
            "slideMaster1.xml must contain <p:sldLayoutIdLst>; got: {}",
            &xml[..xml.len().min(400)]
        );
        // For a template with 1 layout, there must be exactly 1 <p:sldLayoutId id=...> entry.
        // We count `<p:sldLayoutId id=` to avoid matching the `<p:sldLayoutIdLst` tag.
        let count = xml.matches("<p:sldLayoutId id=").count();
        assert_eq!(
            count, 1,
            "sldLayoutIdLst must have 1 entry for a template with 1 layout; got {count}"
        );
    }

    /// ADR-015 §2 — master XML contains 5 required master placeholder types.
    #[test]
    fn test_adr015_serialize_master_to_xml_has_5_master_placeholder_types() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_master_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        for ph_type in &["title", "body", "dt", "ftr", "sldNum"] {
            assert!(
                xml.contains(ph_type),
                "slideMaster1.xml must contain master placeholder type '{}'; xml: {}",
                ph_type,
                &xml[..xml.len().min(500)]
            );
        }
    }

    /// ADR-015 §2 — master XML contains `<p:txStyles>` with heading/body font names.
    #[test]
    fn test_adr015_serialize_master_to_xml_has_tx_styles_with_fonts() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_master_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("<p:txStyles"),
            "slideMaster1.xml must contain <p:txStyles>; xml: {}",
            &xml[..xml.len().min(500)]
        );
        assert!(
            xml.contains("Calibri Light"),
            "txStyles must reference heading font 'Calibri Light'; xml: {}",
            &xml[..xml.len().min(600)]
        );
    }

    /// ADR-015 §2 — `serialize_master_to_xml` is round-trip parseable XML.
    #[test]
    fn test_adr015_serialize_master_to_xml_is_well_formed() {
        use quick_xml::Reader;
        let template = minimal_brand_template();
        let xml_bytes = serialize_master_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        let mut reader = Reader::from_str(xml);
        reader.config_mut().check_end_names = true;
        let mut count = 0usize;
        loop {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => count += 1,
                Err(e) => panic!("serialize_master_to_xml produced malformed XML: {e}"),
            }
        }
        assert!(count > 0, "round-trip parse produced no events");
    }

    /// ADR-015 §A.3 (F-PASS2-H1) — ECMA-376 §19.3.1.42 element order:
    /// `cSld, clrMap, sldLayoutIdLst, hf, txStyles`.
    ///
    /// `<p:hf>` MUST appear before `<p:txStyles>` in the serialized XML.
    /// Also verifies `<a:clrMap>` appears before `<p:sldLayoutIdLst>` which appears
    /// before `<p:hf>`.
    ///
    /// This is a load-bearing order test (TD-VSDD-059). Position is asserted by
    /// byte-offset comparison, not by tag counting.
    #[test]
    fn test_adr015_master_element_order_hf_before_tx_styles() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_master_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");

        let clr_map_pos = xml
            .find("<a:clrMap")
            .expect("master XML must contain <a:clrMap>");
        let sld_layout_lst_pos = xml
            .find("<p:sldLayoutIdLst")
            .expect("master XML must contain <p:sldLayoutIdLst>");
        let hf_pos = xml
            .find("<p:hf")
            .expect("master XML must contain <p:hf>");
        let tx_styles_pos = xml
            .find("<p:txStyles")
            .expect("master XML must contain <p:txStyles>");

        // ECMA-376 §19.3.1.42 CT_SlideMaster sequence model:
        // cSld < clrMap < sldLayoutIdLst < hf < txStyles
        assert!(
            clr_map_pos < sld_layout_lst_pos,
            "<a:clrMap> (byte {clr_map_pos}) must appear BEFORE <p:sldLayoutIdLst> \
             (byte {sld_layout_lst_pos}) — ECMA-376 §19.3.1.42 sequence model (ADR-015 §A.3)"
        );
        assert!(
            sld_layout_lst_pos < hf_pos,
            "<p:sldLayoutIdLst> (byte {sld_layout_lst_pos}) must appear BEFORE <p:hf> \
             (byte {hf_pos}) — ECMA-376 §19.3.1.42 sequence model (ADR-015 §A.3)"
        );
        assert!(
            hf_pos < tx_styles_pos,
            "<p:hf> (byte {hf_pos}) must appear BEFORE <p:txStyles> (byte {tx_styles_pos}) \
             — ECMA-376 §19.3.1.42 sequence model requires hf before txStyles (ADR-015 §A.3 / F-PASS2-H1)"
        );
    }

    // ─── Tests for serialize_theme_to_xml ────────────────────────────────────

    /// ADR-015 §3 — `serialize_theme_to_xml` produces `<a:theme>` root element.
    #[test]
    fn test_adr015_serialize_theme_to_xml_produces_theme_root() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_theme_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("<a:theme"),
            "serialize_theme_to_xml must produce <a:theme> root element; got: {}",
            &xml[..xml.len().min(200)]
        );
    }

    /// ADR-015 §3 — theme XML contains `<a:clrScheme>` with all 12 slots.
    #[test]
    fn test_adr015_serialize_theme_to_xml_has_12_color_slots() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_theme_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("<a:clrScheme"),
            "theme1.xml must contain <a:clrScheme>; got snippet: {}",
            &xml[..xml.len().min(400)]
        );
        // All 12 OOXML color slot element names must be present.
        for slot in &[
            "dk1", "lt1", "dk2", "lt2", "accent1", "accent2", "accent3", "accent4", "accent5",
            "accent6", "hlink", "folHlink",
        ] {
            let slot_elem = format!("a:{slot}");
            assert!(
                xml.contains(&slot_elem),
                "clrScheme must contain slot element 'a:{slot}'"
            );
        }
    }

    /// ADR-015 §3 — theme XML contains `<a:fontScheme>` with heading and body fonts.
    #[test]
    fn test_adr015_serialize_theme_to_xml_has_font_scheme() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_theme_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("<a:fontScheme"),
            "theme1.xml must contain <a:fontScheme>; got: {}",
            &xml[..xml.len().min(400)]
        );
        assert!(
            xml.contains("Calibri Light"),
            "fontScheme must reference heading font 'Calibri Light'; xml: {}",
            &xml[..xml.len().min(500)]
        );
        assert!(
            xml.contains("Calibri"),
            "fontScheme must reference body font 'Calibri'; xml: {}",
            &xml[..xml.len().min(500)]
        );
    }

    /// ADR-015 §3 — theme XML contains `<a:fmtScheme>` (required by ECMA-376 §20.1.6.9).
    #[test]
    fn test_adr015_serialize_theme_to_xml_has_fmt_scheme() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_theme_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        assert!(
            xml.contains("<a:fmtScheme"),
            "theme1.xml must contain <a:fmtScheme> (ECMA-376 §20.1.6.9); got: {}",
            &xml[..xml.len().min(400)]
        );
    }

    /// ADR-015 §3 — theme XML colors use the correct hex values from `BrandTemplate`.
    #[test]
    fn test_adr015_serialize_theme_to_xml_uses_brand_colors() {
        let template = minimal_brand_template();
        let xml_bytes = serialize_theme_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        // dk1 = #000000, lt1 = #FFFFFF (remove # prefix for OOXML srgbClr val).
        assert!(
            xml.contains("000000"),
            "theme1.xml must contain dk1 color '000000'; xml snippet: {}",
            &xml[..xml.len().min(600)]
        );
        assert!(
            xml.contains("FFFFFF"),
            "theme1.xml must contain lt1 color 'FFFFFF'; xml snippet: {}",
            &xml[..xml.len().min(600)]
        );
        // acc1 = #0066CC
        assert!(
            xml.contains("0066CC"),
            "theme1.xml must contain acc1 color '0066CC'; xml snippet: {}",
            &xml[..xml.len().min(600)]
        );
    }

    /// ADR-015 §3 — `serialize_theme_to_xml` is round-trip parseable XML.
    #[test]
    fn test_adr015_serialize_theme_to_xml_is_well_formed() {
        use quick_xml::Reader;
        let template = minimal_brand_template();
        let xml_bytes = serialize_theme_to_xml(&template);
        let xml = std::str::from_utf8(&xml_bytes).expect("output must be valid UTF-8");
        let mut reader = Reader::from_str(xml);
        reader.config_mut().check_end_names = true;
        let mut count = 0usize;
        loop {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => count += 1,
                Err(e) => panic!("serialize_theme_to_xml produced malformed XML: {e}"),
            }
        }
        assert!(count > 0, "round-trip parse produced no events");
    }
}
