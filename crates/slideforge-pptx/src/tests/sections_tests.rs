//! Tests for STORY-082: PPTX Slide-Grouping Sections (sectionLst) — pptx-level tests.
//!
//! Covers AC-003 through AC-009 and AC-011 GUID test which are pptx-specific.
//! AC-001, AC-002, AC-010, AC-011 (parser tests) are in slideforge-syntax tests.
//!
//! ## Traceability
//!
//! | Test function | AC | BC clause |
//! |---|---|---|
//! | `test_BC_4_01_003_ac003_presentation_xml_has_p14_section_lst` | AC-003 | BC-4.01.003 PC5 |
//! | `test_BC_4_01_003_ac003_extlst_is_last_child_of_presentation` | AC-003 | BC-4.01.003 inv4 |
//! | `test_BC_4_01_003_ac003_ext_uri_is_correct_fixed_constant` | AC-003 | BC-4.01.003 PC5 |
//! | `test_BC_4_01_003_ac003_xmlns_p14_declared_on_presentation` | AC-003 | BC-4.01.003 PC5 |
//! | `test_BC_4_01_003_ac004_no_ext_lst_when_no_sections` | AC-004 | BC-4.01.003 EC-002 |
//! | `test_BC_4_01_003_inject_no_op_empty_sections` | AC-004 | BC-4.01.003 inv4 |
//! | `test_BC_4_01_003_ac005_xml_escaped_section_name` | AC-005 | BC-4.01.003 EC-004 |
//! | `test_BC_4_01_003_ac006_two_single_slide_sections` | AC-006 | BC-4.01.003 EC-005 |
//! | `test_BC_4_01_003_ac007_section_lst_skipped_for_non_pptx` | AC-007 | BC-4.01.003 EC-006 |
//! | `test_BC_1_14_003_ac008_no_register_content_in_ext_lst` | AC-008 | BC-1.14.003 PC3+PC5 |
//! | `test_BC_4_01_003_ac009_deterministic_guids_identical_across_builds` | AC-009 | BC-4.01.003 inv3 |
//! | `test_BC_4_01_003_ac009_guid_format_braced_uppercase` | AC-009 | BC-4.01.003 inv3 |
//! | `test_BC_4_01_003_ac011_duplicate_section_name_same_guid` | AC-011 | BC-4.01.003 PC8+inv5 |
//! | `test_BC_4_01_003_inject_well_formed_xml_with_two_sections` | AC-003 | BC-4.01.003 PC5 |
//! | `test_BC_4_01_003_slide_section_entry_implements_hash_eq_clone` | layout | BC-4.01.003 |
//! | `test_BC_4_01_003_format_guid_produces_correct_format` | AC-009 | BC-4.01.003 inv3 |

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::uninlined_format_args)]

use std::sync::Arc;

use slideforge_layout::{
    BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, SlideSectionEntry,
};
use slideforge_types::{Emu, InlineNode, Register, RegisteredContent};

use crate::sections::{
    P14_NS_URI, SECTION_LST_EXT_URI, SectionListBuilder, derive_section_guid, format_guid,
};

// ─── Fixture helpers ──────────────────────────────────────────────────────────

fn make_bare_slide(index: usize) -> LaidOutSlide {
    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(457_200),
                y: Emu(274_638),
                width: Emu(8_229_600),
                height: Emu(1_143_000),
            },
            content: FrameContent::Title(Arc::from(format!("Slide {}", index + 1))),
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

fn make_laid_out_deck_with_sections(
    n: usize,
    slide_sections: Vec<SlideSectionEntry>,
) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: (0..n).map(make_bare_slide).collect(),
        sections: vec![],
        warnings: vec![],
        slide_sections,
    }
}

/// Minimal `presentation.xml` bytes for injection testing.
fn stub_presentation_xml() -> Vec<u8> {
    br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation
    xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
    xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
    xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldMasterIdLst/>
  <p:sldIdLst>
    <p:sldId id="256" r:id="rId4"/>
    <p:sldId id="257" r:id="rId5"/>
    <p:sldId id="258" r:id="rId6"/>
    <p:sldId id="259" r:id="rId7"/>
  </p:sldIdLst>
  <p:sldSz cx="12192000" cy="6858000"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#
        .to_vec()
}

// ─── AC-003 tests ─────────────────────────────────────────────────────────────

/// AC-003 — Two sections produce `<p:extLst>` / `<p14:sectionLst>` with two
/// `<p14:section>` children, correct URI, and correct names.
///
/// Traces to BC-4.01.003 postcondition 5.
#[test]
fn test_BC_4_01_003_ac003_presentation_xml_has_p14_section_lst() {
    let sections = vec![
        SlideSectionEntry {
            name: Arc::from("Background"),
            slide_ids: vec![256, 257],
        },
        SlideSectionEntry {
            name: Arc::from("Analysis"),
            slide_ids: vec![258, 259],
        },
    ];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    assert!(xml_str.contains("<p:extLst>"), "must contain <p:extLst>");
    assert!(
        xml_str.contains(SECTION_LST_EXT_URI),
        "must contain ext uri"
    );
    assert!(
        xml_str.contains("p14:sectionLst"),
        "must contain p14:sectionLst"
    );
    assert_eq!(
        xml_str.matches("<p14:section ").count(),
        2,
        "must have 2 p14:section elements"
    );
    assert!(
        xml_str.contains("name=\"Background\""),
        "must have Background section"
    );
    assert!(
        xml_str.contains("name=\"Analysis\""),
        "must have Analysis section"
    );
}

/// AC-003 — `p:extLst` is the LAST child of `p:presentation`.
///
/// Traces to BC-4.01.003 architecture rule 4.
#[test]
fn test_BC_4_01_003_ac003_extlst_is_last_child_of_presentation() {
    let sections = vec![SlideSectionEntry {
        name: Arc::from("Background"),
        slide_ids: vec![256],
    }];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    let ext_end = xml_str
        .rfind("</p:extLst>")
        .expect("</p:extLst> must be present");
    let prs_end = xml_str
        .rfind("</p:presentation>")
        .expect("</p:presentation> must be present");
    let between = xml_str[ext_end + "</p:extLst>".len()..prs_end].trim();
    assert!(
        between.is_empty(),
        "p:extLst must be last child of p:presentation; got: {:?}",
        between
    );
}

/// AC-003 — `p:ext/@uri` equals the fixed constant `{BB962C8B-...}`.
///
/// Traces to BC-4.01.003 postcondition 5.
#[test]
fn test_BC_4_01_003_ac003_ext_uri_is_correct_fixed_constant() {
    let sections = vec![SlideSectionEntry {
        name: Arc::from("Background"),
        slide_ids: vec![256],
    }];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");
    assert!(
        xml_str.contains(&format!("uri=\"{SECTION_LST_EXT_URI}\"")),
        "p:ext must have correct uri; got:\n{xml_str}"
    );
}

/// AC-003 — `xmlns:p14` declared on `<p:presentation>`.
///
/// Traces to BC-4.01.003 postcondition 5.
#[test]
fn test_BC_4_01_003_ac003_xmlns_p14_declared_on_presentation() {
    let sections = vec![SlideSectionEntry {
        name: Arc::from("Background"),
        slide_ids: vec![256],
    }];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");
    assert!(
        xml_str.contains(&format!("xmlns:p14=\"{P14_NS_URI}\"")),
        "p:presentation must declare xmlns:p14; got:\n{xml_str}"
    );
}

// ─── AC-004 tests ─────────────────────────────────────────────────────────────

/// AC-004 — Empty sections → bytes unchanged, no extLst/sectionLst.
///
/// Traces to BC-4.01.003 EC-002 / invariant 4.
#[test]
fn test_BC_4_01_003_ac004_no_ext_lst_when_no_sections() {
    let prs_xml = stub_presentation_xml();
    let prs_xml_copy = prs_xml.clone();
    let result =
        SectionListBuilder::inject(prs_xml, &[]).expect("inject with empty sections must succeed");
    assert_eq!(
        result, prs_xml_copy,
        "bytes must be unchanged when sections is empty"
    );
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");
    assert!(
        !xml_str.contains("<p:extLst>"),
        "no extLst when sections empty"
    );
    assert!(
        !xml_str.contains("p14:sectionLst"),
        "no sectionLst when sections empty"
    );
}

/// AC-004 — inject with empty sections returns `Ok`.
///
/// Traces to BC-4.01.003 invariant 4.
#[test]
fn test_BC_4_01_003_inject_no_op_empty_sections() {
    assert!(
        SectionListBuilder::inject(stub_presentation_xml(), &[]).is_ok(),
        "inject with empty sections must return Ok"
    );
}

// ─── AC-005 test ──────────────────────────────────────────────────────────────

/// AC-005 — `"Background & Overview"` → `&amp;` in `p14:section/@name`.
///
/// Traces to BC-4.01.003 EC-004.
#[test]
fn test_BC_4_01_003_ac005_xml_escaped_section_name() {
    let sections = vec![SlideSectionEntry {
        name: Arc::from("Background & Overview"),
        slide_ids: vec![256],
    }];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    assert!(
        xml_str.contains("Background &amp; Overview"),
        "& must be XML-escaped as &amp; in p14:section name; got:\n{xml_str}"
    );

    // Verify no raw & outside XML entities.
    for (pos, _) in xml_str.match_indices('&') {
        let after = &xml_str[pos..];
        let is_entity = after.starts_with("&amp;")
            || after.starts_with("&lt;")
            || after.starts_with("&gt;")
            || after.starts_with("&quot;")
            || after.starts_with("&apos;");
        assert!(
            is_entity,
            "raw '&' at byte {pos} is not a valid XML entity — malformed XML"
        );
    }
}

// ─── AC-006 test ──────────────────────────────────────────────────────────────

/// AC-006 — Two single-slide sections produce two `p14:section` and two `p14:sldId`.
///
/// Traces to BC-4.01.003 EC-005.
#[test]
fn test_BC_4_01_003_ac006_two_single_slide_sections() {
    let sections = vec![
        SlideSectionEntry {
            name: Arc::from("Section A"),
            slide_ids: vec![256],
        },
        SlideSectionEntry {
            name: Arc::from("Section B"),
            slide_ids: vec![257],
        },
    ];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    assert_eq!(
        xml_str.matches("<p14:section ").count(),
        2,
        "must have 2 p14:section"
    );
    assert_eq!(
        xml_str.matches("<p14:sldId ").count(),
        2,
        "must have 2 p14:sldId (one per section)"
    );
    assert!(
        xml_str.contains("<p14:sldId id=\"256\""),
        "must have sldId 256"
    );
    assert!(
        xml_str.contains("<p14:sldId id=\"257\""),
        "must have sldId 257"
    );
}

// ─── AC-007 test ──────────────────────────────────────────────────────────────

/// AC-007 — `SectionListBuilder` is PPTX-only. slide_sections field exists in
/// LaidOutDeck but non-PPTX exporters never call SectionListBuilder.
///
/// Traces to BC-4.01.003 EC-006.
#[test]
fn test_BC_4_01_003_ac007_section_lst_skipped_for_non_pptx() {
    let deck = make_laid_out_deck_with_sections(
        2,
        vec![SlideSectionEntry {
            name: Arc::from("Section A"),
            slide_ids: vec![256, 257],
        }],
    );
    assert_eq!(
        deck.slide_sections.len(),
        1,
        "slide_sections must be accessible"
    );

    // Non-PPTX path = call inject with empty sections (no-op).
    let prs_xml = stub_presentation_xml();
    let result = SectionListBuilder::inject(prs_xml, &[])
        .expect("inject with no sections (non-PPTX simulation) must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");
    assert!(
        !xml_str.contains("p14:sectionLst"),
        "no sectionLst for non-PPTX path"
    );
}

// ─── AC-008 test ──────────────────────────────────────────────────────────────

/// AC-008 — Register content (detail/report) does not appear in `<p:extLst>`.
///
/// Traces to BC-1.14.003 postconditions 3 + 5.
#[test]
fn test_BC_1_14_003_ac008_no_register_content_in_ext_lst() {
    let detail_sentinel = "DETAIL_SENTINEL_AC008_MUST_NOT_BLEED";
    let report_sentinel = "REPORT_SENTINEL_AC008_MUST_NOT_BLEED";

    let mut slide = make_bare_slide(0);
    slide.register_content = vec![
        RegisteredContent {
            register: Register::Detail,
            content: vec![InlineNode::Plain(Arc::from(detail_sentinel))],
        },
        RegisteredContent {
            register: Register::Report,
            content: vec![InlineNode::Plain(Arc::from(report_sentinel))],
        },
    ];

    // SectionListBuilder reads ONLY the sections argument, not slide register_content.
    let sections = vec![SlideSectionEntry {
        name: Arc::from("Background"),
        slide_ids: vec![256],
    }];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    let ext_start = xml_str
        .find("<p:extLst>")
        .expect("p:extLst must be present");
    let ext_end = xml_str
        .find("</p:extLst>")
        .map(|i| i + "</p:extLst>".len())
        .expect("</p:extLst> must be present");
    let ext_xml = &xml_str[ext_start..ext_end];

    assert!(
        !ext_xml.contains(detail_sentinel),
        "p:extLst must NOT contain detail content (BC-1.14.003 PC3)"
    );
    assert!(
        !ext_xml.contains(report_sentinel),
        "p:extLst must NOT contain report content (BC-1.14.003 PC5)"
    );
}

// ─── AC-008 pipeline test (OBS-1 strengthening) ──────────────────────────────

/// AC-008 pipeline — Full eval→export pipeline with `detail:`/`report:` content
/// on a sectioned slide must produce NO register content in `<p:extLst>` /
/// `<p14:sectionLst>`.
///
/// The existing AC-008 test verifies this at the `SectionListBuilder::inject`
/// level (hand-built sections arg). This test exercises the FULL pipeline to
/// confirm no register content leaks from the eval stage through to the PPTX
/// XML section list.
///
/// Traces to BC-1.14.003 postconditions 3 + 5 (OBS-1 strengthening).
#[test]
fn test_BC_1_14_003_ac008_no_register_bleed_full_pipeline() {
    use slideforge_eval::{EvalConfig, eval_deck};
    use slideforge_layout::run as layout_run;
    use slideforge_plugin_api::{ExportOptions, Exporter};
    use slideforge_syntax::DiagnosticSink;
    use slideforge_syntax::span::SourceMap;
    use slideforge_types::{Brand, BrandFonts, BrandPalette, SourceSpan};

    // Use distinctive sentinel strings that must NEVER appear in p:extLst.
    let detail_sentinel = "DETAIL_SENTINEL_AC008_PIPELINE_MUST_NOT_BLEED";
    let report_sentinel = "REPORT_SENTINEL_AC008_PIPELINE_MUST_NOT_BLEED";

    // Build a deck where the sectioned slide has detail: and report: content.
    // These are register-routed fields and must NOT appear in p:extLst / p14:sectionLst.
    let src = format!(
        "slideforge_version \"1\"\nlang \"en-US\"\nsection \"Background\":\n  \
         slide title:\n    title \"Background Slide\"\n    detail \"{detail_sentinel}\"\n    \
         report \"{report_sentinel}\"\n"
    );

    let mut sm = SourceMap::default();
    let file_id = sm.add_file(
        std::sync::Arc::from("ac008.sf"),
        std::sync::Arc::from(src.as_str()),
    );
    let parse_result = slideforge_syntax::parse(&src, file_id, &sm).expect("parse must succeed");

    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&parse_result.deck, &EvalConfig::default(), &mut sink)
        .expect("eval must succeed");
    assert!(!sink.has_fatal(), "eval must produce no fatal errors");

    let brand = Brand {
        name: std::sync::Arc::from("test"),
        palette: BrandPalette {
            primary: std::sync::Arc::from("#003087"),
            secondary: std::sync::Arc::from("#0066CC"),
            accent: std::sync::Arc::from("#FF6B35"),
            neutral: std::sync::Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: std::sync::Arc::from("Calibri"),
            body: std::sync::Arc::from("Calibri"),
            mono: std::sync::Arc::from("Courier New"),
        },
        layouts: vec![],
        span: SourceSpan::default(),
    };
    let laid_out = layout_run(&deck, &brand).expect("layout must succeed");

    let exporter = crate::PptxExporter::new();
    let pptx_bytes = exporter
        .export(&deck, &laid_out, &brand, &ExportOptions::default())
        .expect("PPTX export must succeed");

    // Extract ppt/presentation.xml from the produced ZIP.
    let cursor = std::io::Cursor::new(pptx_bytes);
    let mut zip = zip::ZipArchive::new(cursor).expect("must be valid ZIP");
    let mut prs_xml_bytes = Vec::new();
    {
        let mut prs_file = zip
            .by_name("ppt/presentation.xml")
            .expect("ppt/presentation.xml must be present");
        std::io::Read::read_to_end(&mut prs_file, &mut prs_xml_bytes)
            .expect("must read presentation.xml");
    }
    let prs_xml = String::from_utf8(prs_xml_bytes).expect("presentation.xml must be valid UTF-8");

    // Isolate the p:extLst block (which contains p14:sectionLst).
    let ext_start = prs_xml
        .find("<p:extLst>")
        .expect("p:extLst must be present — section produces extLst");
    let ext_end = prs_xml
        .find("</p:extLst>")
        .map(|i| i + "</p:extLst>".len())
        .expect("</p:extLst> must be present");
    let ext_xml = &prs_xml[ext_start..ext_end];

    // Neither detail nor report sentinel must appear in the section XML.
    assert!(
        !ext_xml.contains(detail_sentinel),
        "p:extLst / p14:sectionLst MUST NOT contain detail content in full-pipeline test; \
         BC-1.14.003 PC3 (OBS-1)"
    );
    assert!(
        !ext_xml.contains(report_sentinel),
        "p:extLst / p14:sectionLst MUST NOT contain report content in full-pipeline test; \
         BC-1.14.003 PC5 (OBS-1)"
    );
}

// ─── AC-009 tests ─────────────────────────────────────────────────────────────

/// AC-009 — Identical inputs produce byte-identical output.
///
/// Traces to BC-4.01.003 invariant 3.
#[test]
fn test_BC_4_01_003_ac009_deterministic_guids_identical_across_builds() {
    let sections = vec![
        SlideSectionEntry {
            name: Arc::from("Background"),
            slide_ids: vec![256, 257],
        },
        SlideSectionEntry {
            name: Arc::from("Analysis"),
            slide_ids: vec![258, 259],
        },
    ];
    let result1 = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("first inject must succeed");
    let result2 = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("second inject must succeed");
    assert_eq!(
        result1, result2,
        "two identical calls must produce byte-identical output"
    );
}

/// AC-009 — `derive_section_guid` is deterministic, brace-wrapped, uppercase,
/// version nibble=5, RFC 4122 variant.
///
/// Traces to BC-4.01.003 invariant 3.
#[test]
fn test_BC_4_01_003_ac009_guid_format_braced_uppercase() {
    let guid1 = derive_section_guid("Background");
    let guid2 = derive_section_guid("Background");

    assert_eq!(guid1, guid2, "derive_section_guid must be deterministic");
    assert!(
        guid1.starts_with('{') && guid1.ends_with('}'),
        "GUID must be brace-wrapped; got: {guid1}"
    );

    let inner = &guid1[1..guid1.len() - 1];
    assert!(
        inner
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-'),
        "GUID inner must be uppercase hex + hyphens; got: {inner}"
    );

    let parts: Vec<&str> = inner.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);

    assert!(
        parts[2].starts_with('5'),
        "version nibble must be 5; third group: {}",
        parts[2]
    );

    let variant_char = parts[3].chars().next().expect("fourth part non-empty");
    assert!(
        matches!(variant_char, '8' | '9' | 'A' | 'B'),
        "variant nibble must be 8/9/A/B; got: {variant_char}"
    );

    let other = derive_section_guid("Analysis");
    assert_ne!(guid1, other, "different names must produce different GUIDs");
}

// ─── AC-011 GUID test ─────────────────────────────────────────────────────────

/// AC-011 — Duplicate section names produce identical `p14:section/@id` GUIDs.
///
/// Traces to BC-4.01.003 postcondition 8 + invariant 5.
#[test]
fn test_BC_4_01_003_ac011_duplicate_section_name_same_guid() {
    let sections = vec![
        SlideSectionEntry {
            name: Arc::from("Background"),
            slide_ids: vec![256],
        },
        SlideSectionEntry {
            name: Arc::from("Background"),
            slide_ids: vec![257],
        },
    ];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject with duplicate-named sections must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    let mut ids: Vec<String> = Vec::new();
    let mut search = xml_str.as_str();
    loop {
        match search.find("<p14:section ") {
            None => break,
            Some(pos) => {
                let rest = &search[pos..];
                if let Some(id_pos) = rest.find(" id=\"") {
                    let id_start = id_pos + 5;
                    if let Some(id_end) = rest[id_start..].find('"') {
                        ids.push(rest[id_start..id_start + id_end].to_string());
                    }
                }
                search = &search[pos + 1..];
            },
        }
    }

    assert_eq!(
        ids.len(),
        2,
        "must find 2 p14:section id values; got: {ids:?}"
    );
    assert_eq!(
        ids[0], ids[1],
        "duplicate names must produce identical GUIDs; id[0]={} id[1]={}",
        ids[0], ids[1]
    );
}

// ─── Additional tests ─────────────────────────────────────────────────────────

/// Two sections of 2 slides each produce 4 `<p14:sldId>` elements.
///
/// Traces to BC-4.01.003 postcondition 5.
#[test]
fn test_BC_4_01_003_inject_well_formed_xml_with_two_sections() {
    let sections = vec![
        SlideSectionEntry {
            name: Arc::from("Background"),
            slide_ids: vec![256, 257],
        },
        SlideSectionEntry {
            name: Arc::from("Analysis"),
            slide_ids: vec![258, 259],
        },
    ];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");
    assert_eq!(
        xml_str.matches("<p14:sldId ").count(),
        4,
        "must have 4 sldId elements"
    );
    assert!(xml_str.contains("id=\"256\""));
    assert!(xml_str.contains("id=\"257\""));
    assert!(xml_str.contains("id=\"258\""));
    assert!(xml_str.contains("id=\"259\""));
}

/// `SlideSectionEntry` implements `Hash + Eq + Clone`.
#[test]
fn test_BC_4_01_003_slide_section_entry_implements_hash_eq_clone() {
    use std::collections::HashSet;
    let entry = SlideSectionEntry {
        name: Arc::from("Background"),
        slide_ids: vec![256, 257],
    };
    let entry2 = entry.clone();
    assert_eq!(entry, entry2);
    let mut set = HashSet::new();
    set.insert(entry);
    assert_eq!(set.len(), 1);
}

/// Different names → different entries (not equal).
#[test]
fn test_BC_4_01_003_slide_section_entry_different_names_not_equal() {
    let a = SlideSectionEntry {
        name: Arc::from("Background"),
        slide_ids: vec![256],
    };
    let b = SlideSectionEntry {
        name: Arc::from("Analysis"),
        slide_ids: vec![256],
    };
    assert_ne!(a, b);
}

// ─── HIGH-3: Keystone end-to-end wiring test (LESSON-21) ─────────────────────

/// HIGH-3 — Full pipeline wiring test: parse → eval → layout → pptx export.
///
/// Builds a real deck containing `section "Background":` groupings, runs the
/// complete pipeline, extracts `ppt/presentation.xml` from the produced ZIP,
/// and re-parses it with `quick-xml` to assert:
///
/// 1. `<p:extLst>` is the LAST child of `<p:presentation>`.
/// 2. `<p:ext uri="{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}">` is present.
/// 3. `<p14:sectionLst>` is present with the correct number of `<p14:section>`
///    children.
/// 4. `xmlns:p14` is declared on `<p:presentation>`.
///
/// This test is the keystone that proves CRIT-1 through CRIT-4 are wired
/// end-to-end: parser populates slide children → eval flows them into Deck.slides
/// AND Deck.slide_sections → layout passes slide_sections to LaidOutDeck →
/// exporter calls SectionListBuilder::inject.
///
/// Traces to BC-4.01.003 (all ACs), CRIT-1/2/3/4 adversary findings.
// This is a full-pipeline test by design; its length reflects the 8-step
// assertion chain required to prove end-to-end wiring (LESSON-21).
#[allow(clippy::too_many_lines)]
#[test]
fn test_BC_4_01_003_high3_e2e_pipeline_section_lst_in_presentation_xml() {
    use quick_xml::Reader;
    use quick_xml::events::Event;
    use slideforge_eval::{EvalConfig, eval_deck};
    use slideforge_layout::run as layout_run;
    use slideforge_plugin_api::{ExportOptions, Exporter};
    use slideforge_syntax::DiagnosticSink;
    use slideforge_syntax::span::SourceMap;
    use slideforge_types::{Brand, BrandFonts, BrandPalette, SourceSpan};

    let src = r#"slideforge_version "1"
lang "en-US"
section "Background":
  slide title:
    title "Background Slide 1"
  slide content:
    title "Background Slide 2"
section "Analysis":
  slide content:
    title "Analysis Slide"
slide title:
  title "Ungrouped Slide"
"#;

    // Step 1: Parse
    let mut sm = SourceMap::default();
    let file_id = sm.add_file(std::sync::Arc::from("e2e.sf"), std::sync::Arc::from(src));
    let parse_result =
        slideforge_syntax::parse(src, file_id, &sm).expect("parse must succeed for valid DSL");

    // Step 2: Eval
    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&parse_result.deck, &EvalConfig::default(), &mut sink)
        .expect("eval must succeed");
    assert!(!sink.has_fatal(), "eval must produce no fatal errors");

    // Assert CRIT-4: grouped slides appear in Deck.slides (no silent data loss)
    assert_eq!(
        deck.slides.len(),
        4,
        "Deck must contain all 4 slides (2+1 grouped + 1 ungrouped); \
         CRIT-4: grouped slides must not be silently dropped"
    );

    // Assert CRIT-2: slide_sections is populated on Deck
    assert_eq!(
        deck.slide_sections.len(),
        2,
        "Deck.slide_sections must have 2 entries (Background + Analysis); \
         CRIT-2: build_slide_sections_from_membership must wire into eval"
    );

    // Step 3: Layout
    let brand = Brand {
        name: std::sync::Arc::from("test"),
        palette: BrandPalette {
            primary: std::sync::Arc::from("#003087"),
            secondary: std::sync::Arc::from("#0066CC"),
            accent: std::sync::Arc::from("#FF6B35"),
            neutral: std::sync::Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: std::sync::Arc::from("Calibri"),
            body: std::sync::Arc::from("Calibri"),
            mono: std::sync::Arc::from("Courier New"),
        },
        layouts: vec![],
        span: SourceSpan::default(),
    };
    let laid_out = layout_run(&deck, &brand).expect("layout::run must succeed");

    // Assert CRIT-2 pass-through: LaidOutDeck.slide_sections must equal Deck.slide_sections
    assert_eq!(
        laid_out.slide_sections.len(),
        2,
        "LaidOutDeck.slide_sections must have 2 entries (passed through from Deck); \
         CRIT-2: layout pass-through wiring"
    );
    assert_eq!(laid_out.slide_sections[0].name.as_ref(), "Background");
    assert_eq!(laid_out.slide_sections[0].slide_ids, vec![256, 257]);
    assert_eq!(laid_out.slide_sections[1].name.as_ref(), "Analysis");
    assert_eq!(laid_out.slide_sections[1].slide_ids, vec![258]);

    // Step 4: PPTX Export
    let exporter = crate::PptxExporter::new();
    let pptx_bytes = exporter
        .export(&deck, &laid_out, &brand, &ExportOptions::default())
        .expect("PPTX export must succeed");

    // Step 5: Extract ppt/presentation.xml from the ZIP
    let cursor = std::io::Cursor::new(pptx_bytes);
    let mut zip = zip::ZipArchive::new(cursor).expect("must be valid ZIP");
    let mut prs_xml_bytes = Vec::new();
    {
        let mut prs_file = zip
            .by_name("ppt/presentation.xml")
            .expect("ppt/presentation.xml must be present in PPTX archive");
        std::io::Read::read_to_end(&mut prs_file, &mut prs_xml_bytes)
            .expect("must read presentation.xml");
    }
    let prs_xml = String::from_utf8(prs_xml_bytes).expect("presentation.xml must be valid UTF-8");

    // Step 6: Assert p:extLst is the LAST child of p:presentation (BC-4.01.003 inv4)
    let close_pos = prs_xml
        .rfind("</p:presentation>")
        .expect("closing </p:presentation> tag must be present");
    let ext_lst_pos = prs_xml.rfind("<p:extLst>")
        .expect("p:extLst must be present in presentation.xml — CRIT-1: SectionListBuilder::inject not called");
    assert!(
        ext_lst_pos < close_pos,
        "p:extLst must appear BEFORE </p:presentation>; \
         BC-4.01.003 invariant 4"
    );
    // Verify nothing structural appears between </p:extLst> and </p:presentation>
    let ext_lst_close_pos = prs_xml
        .rfind("</p:extLst>")
        .expect("closing </p:extLst> must be present");
    let between = prs_xml[ext_lst_close_pos + "</p:extLst>".len()..close_pos].trim();
    assert!(
        between.is_empty(),
        "p:extLst must be the LAST child of p:presentation; \
         found content between </p:extLst> and </p:presentation>: {between:?}"
    );

    // Step 7: Assert xmlns:p14 is declared on <p:presentation (BC-4.01.003 PC5)
    let open_tag_end = prs_xml
        .find('>')
        .expect("presentation must have an opening tag");
    let open_tag = &prs_xml[..open_tag_end];
    assert!(
        open_tag.contains("xmlns:p14="),
        "xmlns:p14 must be declared on <p:presentation>; got opening tag: {open_tag:?}"
    );

    // Step 8: Re-parse the XML with quick-xml to assert structural invariants
    let mut reader = Reader::from_str(&prs_xml);
    reader.config_mut().trim_text(true);

    let mut section_lst_found = false;
    let mut section_count = 0usize;
    let mut ext_uri_ok = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let local = e.name().local_name();
                let local_str = std::str::from_utf8(local.as_ref()).unwrap_or("");
                let prefix_bytes = e
                    .name()
                    .prefix()
                    .map(|p| p.as_ref().to_vec())
                    .unwrap_or_default();
                let prefix_str = std::str::from_utf8(&prefix_bytes).unwrap_or("");
                match (prefix_str, local_str) {
                    ("p", "ext") => {
                        // Check the uri attribute
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"uri" {
                                let val = String::from_utf8_lossy(&attr.value);
                                if val.as_ref() == SECTION_LST_EXT_URI {
                                    ext_uri_ok = true;
                                }
                            }
                        }
                    },
                    ("p14", "sectionLst") => {
                        section_lst_found = true;
                    },
                    ("p14", "section") => {
                        section_count += 1;
                    },
                    _ => {},
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("XML parse error: {e}"),
            _ => {},
        }
    }

    assert!(
        ext_uri_ok,
        "p:ext must have uri=\"{SECTION_LST_EXT_URI}\" (BC-4.01.003 PC5)"
    );
    assert!(
        section_lst_found,
        "p14:sectionLst must be present in presentation.xml — \
         CRIT-1: SectionListBuilder::inject is not wired"
    );
    assert_eq!(
        section_count, 2,
        "must have exactly 2 p14:section elements (Background + Analysis); \
         got {section_count}"
    );
}

/// MED-2 — DOCX export must NOT produce sectionLst.
///
/// Non-PPTX exporters must not emit `p14:sectionLst` or any PPTX-specific
/// section metadata. This test builds a deck with section groups, exports to
/// DOCX, and asserts the output contains no `sectionLst` string.
///
/// Traces to BC-4.01.003 EC-006 (non-PPTX exporters skip sectionLst entirely).
#[test]
fn test_BC_4_01_003_med2_docx_has_no_section_lst() {
    use slideforge_eval::{EvalConfig, eval_deck};
    use slideforge_layout::run as layout_run;
    use slideforge_plugin_api::{ExportOptions, Exporter};
    use slideforge_syntax::DiagnosticSink;
    use slideforge_syntax::span::SourceMap;
    use slideforge_types::{Brand, BrandFonts, BrandPalette, SourceSpan};

    let src = r#"slideforge_version "1"
lang "en-US"
section "Background":
  slide title:
    title "Grouped Slide"
"#;

    let mut sm = SourceMap::default();
    let file_id = sm.add_file(std::sync::Arc::from("e2e.sf"), std::sync::Arc::from(src));
    let parse_result = slideforge_syntax::parse(src, file_id, &sm).expect("parse must succeed");

    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&parse_result.deck, &EvalConfig::default(), &mut sink)
        .expect("eval must succeed");

    let brand = Brand {
        name: std::sync::Arc::from("test"),
        palette: BrandPalette {
            primary: std::sync::Arc::from("#003087"),
            secondary: std::sync::Arc::from("#0066CC"),
            accent: std::sync::Arc::from("#FF6B35"),
            neutral: std::sync::Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: std::sync::Arc::from("Calibri"),
            body: std::sync::Arc::from("Calibri"),
            mono: std::sync::Arc::from("Courier New"),
        },
        layouts: vec![],
        span: SourceSpan::default(),
    };
    let laid_out = layout_run(&deck, &brand).expect("layout must succeed");

    // Use the DOCX exporter (not PPTX) — MED-2 requires no sectionLst in DOCX output.
    let docx_exporter = slideforge_docx::DocxExporter;
    let docx_bytes = docx_exporter
        .export(&deck, &laid_out, &brand, &ExportOptions::default())
        .expect("DOCX export must succeed");

    // The DOCX ZIP should contain no reference to p14:sectionLst or PPTX section metadata.
    // Read all file contents in the ZIP and check for the sectionLst string.
    let cursor = std::io::Cursor::new(docx_bytes);
    let mut zip = zip::ZipArchive::new(cursor).expect("must be valid ZIP");
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).expect("must access zip entry");
        let fname = file.name().to_ascii_lowercase();
        let fname_path = std::path::Path::new(&fname);
        if fname_path.extension().is_some_and(|e| e == "xml")
            || fname_path.extension().is_some_and(|e| e == "rels")
        {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap_or_default();
            assert!(
                !content.contains("sectionLst"),
                "DOCX XML part '{}' must NOT contain 'sectionLst'; \
                 MED-2: sectionLst is PPTX-only (BC-4.01.003 EC-006)",
                file.name()
            );
        }
    }
}

// ─── OBS: EC-004 escape coverage for < and > ─────────────────────────────────

/// OBS (STORY-082 pass-2) — `<` and `>` in section names are XML-escaped.
///
/// The existing AC-005 test covers `&` → `&amp;`.  This test covers the
/// remaining special XML characters `<` (→ `&lt;`) and `>` (→ `&gt;`) so that
/// the EC-004 escape coverage is non-vacuous.
///
/// Traces to BC-4.01.003 EC-004.
#[test]
fn test_BC_4_01_003_ec004_lt_gt_in_section_name_are_xml_escaped() {
    let sections = vec![SlideSectionEntry {
        name: std::sync::Arc::from("A <b> & C > D"),
        slide_ids: vec![256],
    }];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    assert!(
        xml_str.contains("&lt;b&gt;"),
        "< and > must be XML-escaped as &lt; and &gt; in p14:section name; got:\n{xml_str}"
    );
    assert!(
        xml_str.contains("&amp;"),
        "& must be XML-escaped as &amp; in p14:section name; got:\n{xml_str}"
    );

    // Verify no raw < or > appear inside the p14:section element (outside of tag syntax).
    // We look specifically in the sectionLst block.
    let ext_start = xml_str
        .find("<p:extLst>")
        .expect("p:extLst must be present");
    let ext_end = xml_str
        .find("</p:extLst>")
        .map(|i| i + "</p:extLst>".len())
        .expect("</p:extLst> must be present");
    let ext_xml = &xml_str[ext_start..ext_end];

    // The section name attribute value must not contain unescaped & < >
    // We parse the attribute value by looking for the name="..." attribute.
    let name_attr_start = ext_xml.find("name=\"").expect("must have name= attr");
    let name_attr_val_start = name_attr_start + "name=\"".len();
    let name_attr_val_end = ext_xml[name_attr_val_start..]
        .find('"')
        .expect("closing quote for name attr");
    let name_val = &ext_xml[name_attr_val_start..name_attr_val_start + name_attr_val_end];

    assert!(
        !name_val.contains('<'),
        "name attribute must not contain raw '<'; got: {name_val}"
    );
    assert!(
        !name_val.contains('>'),
        "name attribute must not contain raw '>'; got: {name_val}"
    );
    assert!(
        !name_val.contains('&')
            || name_val.contains("&amp;")
            || name_val.contains("&lt;")
            || name_val.contains("&gt;"),
        "name attribute '&' must be escaped; got: {name_val}"
    );
}

// ─── CRIT-A regression test: @for before section → correct slide IDs ─────────

/// CRIT-A (STORY-082 pass-2) — `@for` loop BEFORE a `section "Name":` must not
/// corrupt the section's slide IDs.
///
/// When `@for x in [1, 2]:` appears before `section "Background":`, the `@for`
/// expands to 2 slides (flat indices 0 and 1, IDs 256 and 257).  The Background
/// section slide is at flat index 2 → ID 258.  The superseded AST-walker
/// approach could not model `@for` expansion and incorrectly assigned ID 256
/// to the Background slide.
///
/// This test:
/// 1. Builds a deck with `@for` (2 iterations) → `section "Background": slide`
///    → `@for` inside the section (2 more iterations).
/// 2. Drives the full pipeline: eval → layout → PPTX export.
/// 3. Re-parses `ppt/presentation.xml` from the produced ZIP.
/// 4. Asserts that `<p14:sldId id="258"/>` (not 256) appears in the Background
///    section, and that the `<p:sldIdLst>` and `<p14:sectionLst>` are consistent.
///
/// Traces to BC-4.01.003 PC-5 (sectionLst groups slides by correct slide IDs).
// This is a full-pipeline test; its length reflects the multi-step assertion
// chain required to prove CRIT-A correctness end-to-end.
#[allow(clippy::too_many_lines)]
#[test]
fn test_BC_4_01_003_crit_a_for_before_section_correct_slide_ids() {
    use quick_xml::Reader;
    use quick_xml::events::Event;
    use slideforge_eval::{EvalConfig, eval_deck};
    use slideforge_layout::run as layout_run;
    use slideforge_plugin_api::{ExportOptions, Exporter};
    use slideforge_syntax::DiagnosticSink;
    use slideforge_syntax::span::SourceMap;
    use slideforge_types::{Brand, BrandFonts, BrandPalette, SourceSpan};

    // Deck layout:
    //   @for x in [1, 2]: → 2 slides (flat idx 0, 1 → IDs 256, 257)
    //   section "Background":
    //     slide title: "Background Slide"   ← flat idx 2 → ID 258
    //     @for y in [10, 20]: → 2 slides    ← flat idx 3, 4 → IDs 259, 260
    //   slide title: "Ungrouped"             ← flat idx 5 → ID 261
    let src = r#"slideforge_version "1"
lang "en-US"
@for x in [1, 2]:
  slide content:
    title "Loop slide {{ x }}"
section "Background":
  slide title:
    title "Background Slide"
  @for y in [10, 20]:
    slide content:
      title "Inner loop {{ y }}"
slide title:
  title "Ungrouped"
"#;

    let mut sm = SourceMap::default();
    let file_id = sm.add_file(std::sync::Arc::from("crit_a.sf"), std::sync::Arc::from(src));
    let parse_result =
        slideforge_syntax::parse(src, file_id, &sm).expect("parse must succeed for valid DSL");

    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&parse_result.deck, &EvalConfig::default(), &mut sink)
        .expect("eval must succeed");
    assert!(
        !sink.has_fatal(),
        "eval must produce no fatal errors; got: {:?}",
        sink.errors()
    );

    // Deck must have 6 slides total: 2 (@for) + 1 (section title) + 2 (@for inside section) + 1 (ungrouped)
    assert_eq!(
        deck.slides.len(),
        6,
        "CRIT-A: Deck must contain 6 slides; got {}",
        deck.slides.len()
    );

    // slide_sections must have exactly 1 entry: "Background"
    assert_eq!(
        deck.slide_sections.len(),
        1,
        "CRIT-A: Deck must have 1 section entry; got {}",
        deck.slide_sections.len()
    );
    assert_eq!(deck.slide_sections[0].name.as_ref(), "Background");
    // Background section contains flat indices 2, 3, 4 → IDs 258, 259, 260
    assert_eq!(
        deck.slide_sections[0].slide_ids,
        vec![258, 259, 260],
        "CRIT-A: Background section must have IDs [258, 259, 260] (not [256, ...]); got {:?}",
        deck.slide_sections[0].slide_ids
    );

    let brand = Brand {
        name: std::sync::Arc::from("test"),
        palette: BrandPalette {
            primary: std::sync::Arc::from("#003087"),
            secondary: std::sync::Arc::from("#0066CC"),
            accent: std::sync::Arc::from("#FF6B35"),
            neutral: std::sync::Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: std::sync::Arc::from("Calibri"),
            body: std::sync::Arc::from("Calibri"),
            mono: std::sync::Arc::from("Courier New"),
        },
        layouts: vec![],
        span: SourceSpan::default(),
    };
    let laid_out = layout_run(&deck, &brand).expect("layout::run must succeed");

    // LaidOutDeck slide_sections must match
    assert_eq!(laid_out.slide_sections.len(), 1);
    assert_eq!(laid_out.slide_sections[0].slide_ids, vec![258, 259, 260]);

    let exporter = crate::PptxExporter::new();
    let pptx_bytes = exporter
        .export(&deck, &laid_out, &brand, &ExportOptions::default())
        .expect("PPTX export must succeed");

    // Extract and re-parse presentation.xml
    let cursor = std::io::Cursor::new(pptx_bytes);
    let mut zip = zip::ZipArchive::new(cursor).expect("must be valid ZIP");
    let mut prs_xml_bytes = Vec::new();
    {
        let mut prs_file = zip
            .by_name("ppt/presentation.xml")
            .expect("ppt/presentation.xml must be present");
        std::io::Read::read_to_end(&mut prs_file, &mut prs_xml_bytes)
            .expect("must read presentation.xml");
    }
    let prs_xml = String::from_utf8(prs_xml_bytes).expect("presentation.xml must be valid UTF-8");

    // Gather all p14:sldId values from the sectionLst
    let mut section_sld_ids: Vec<u32> = Vec::new();
    let mut reader = Reader::from_str(&prs_xml);
    reader.config_mut().trim_text(true);
    let mut in_section_lst = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let prefix_bytes = e
                    .name()
                    .prefix()
                    .map(|p| p.as_ref().to_vec())
                    .unwrap_or_default();
                let prefix = std::str::from_utf8(&prefix_bytes).unwrap_or("");
                let local_name_owned = e.name().local_name().as_ref().to_vec();
                let local = std::str::from_utf8(&local_name_owned).unwrap_or("");
                match (prefix, local) {
                    ("p14", "sectionLst") => in_section_lst = true,
                    ("p14", "sldId") if in_section_lst => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                let val = std::str::from_utf8(&attr.value).unwrap_or("");
                                if let Ok(id) = val.parse::<u32>() {
                                    section_sld_ids.push(id);
                                }
                            }
                        }
                    },
                    _ => {},
                }
            },
            Ok(Event::End(ref e)) => {
                let end_local_owned = e.name().local_name().as_ref().to_vec();
                if std::str::from_utf8(&end_local_owned).unwrap_or("") == "sectionLst" {
                    in_section_lst = false;
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("XML parse error: {e}"),
            _ => {},
        }
    }

    // The sectionLst must list IDs 258, 259, 260 — NOT 256, 257, 258.
    // This is the CRIT-A correctness proof: @for expansion is accounted for.
    assert_eq!(
        section_sld_ids,
        vec![258, 259, 260],
        "CRIT-A: section sldId list in presentation.xml must be [258, 259, 260]; \
         got {:?}. The @for before the section must advance the flat slide index.",
        section_sld_ids
    );

    // OBS-2 strengthening: parse the FULL p:sldIdLst and assert disjoint+complete.
    //
    // BC VP: the sldIdLst must be complete (all 6 slide IDs 256..=261 present),
    // and every p14:sldId in the sectionLst must be a strict subset of sldIdLst.
    let mut sld_id_lst: std::collections::HashSet<u32> = std::collections::HashSet::new();
    {
        let mut reader2 = quick_xml::Reader::from_str(&prs_xml);
        reader2.config_mut().trim_text(true);
        let mut in_sld_id_lst = false;
        loop {
            match reader2.read_event() {
                Ok(
                    quick_xml::events::Event::Start(ref e) | quick_xml::events::Event::Empty(ref e),
                ) => {
                    let prefix_bytes = e
                        .name()
                        .prefix()
                        .map(|p| p.as_ref().to_vec())
                        .unwrap_or_default();
                    let prefix = std::str::from_utf8(&prefix_bytes).unwrap_or("");
                    let local_bytes = e.name().local_name().as_ref().to_vec();
                    let local = std::str::from_utf8(&local_bytes).unwrap_or("");
                    match (prefix, local) {
                        ("p", "sldIdLst") => in_sld_id_lst = true,
                        ("p", "sldId") if in_sld_id_lst => {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"id" {
                                    let val = std::str::from_utf8(&attr.value).unwrap_or("");
                                    if let Ok(id) = val.parse::<u32>() {
                                        sld_id_lst.insert(id);
                                    }
                                }
                            }
                        },
                        _ => {},
                    }
                },
                Ok(quick_xml::events::Event::End(ref e)) => {
                    let local_bytes = e.name().local_name().as_ref().to_vec();
                    if std::str::from_utf8(&local_bytes).unwrap_or("") == "sldIdLst" {
                        in_sld_id_lst = false;
                    }
                },
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => panic!("XML parse error in sldIdLst pass: {e}"),
                _ => {},
            }
        }
    }

    // Assert completeness: p:sldIdLst must contain all 6 IDs 256..=261.
    let expected_all_ids: std::collections::HashSet<u32> = (256u32..=261).collect();
    assert_eq!(
        sld_id_lst, expected_all_ids,
        "OBS-2/CRIT-A: p:sldIdLst must contain all 6 slide IDs 256..=261 (complete); \
         got {:?}",
        sld_id_lst
    );

    // Assert subset: every p14:sldId in sectionLst must be in p:sldIdLst.
    let section_ids_set: std::collections::HashSet<u32> = section_sld_ids.iter().copied().collect();
    assert!(
        section_ids_set.is_subset(&sld_id_lst),
        "OBS-2/CRIT-A: p14:sectionLst slide IDs must be a strict subset of p:sldIdLst \
         (disjoint+complete BC VP); section IDs {:?} not all in sldIdLst {:?}",
        section_ids_set,
        sld_id_lst
    );

    // Assert @for slides (256, 257) do NOT appear in the sectionLst (disjoint).
    for for_id in [256u32, 257u32] {
        assert!(
            !section_ids_set.contains(&for_id),
            "OBS-2/CRIT-A: @for slide ID {for_id} must NOT appear in p14:sectionLst; \
             Background section must only reference IDs [258, 259, 260]"
        );
    }
}

/// `format_guid` produces correct 8-4-4-4-12 brace-wrapped uppercase GUID.
#[test]
fn test_BC_4_01_003_format_guid_produces_correct_format() {
    let bytes: [u8; 16] = [
        0x3D, 0x4F, 0x2B, 0x8A, 0x1C, 0x9E, 0x52, 0xA0, 0xB4, 0xD6, 0x7E, 0x8A, 0x9B, 0x0C, 0x1D,
        0x2E,
    ];
    let guid = format_guid(&bytes);
    assert!(guid.starts_with('{') && guid.ends_with('}'));
    let inner = &guid[1..guid.len() - 1];
    assert!(
        inner
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
    );
    let parts: Vec<&str> = inner.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);
}

// ─── SEC-100 / CWE-116: XML-1.0 control-character sanitization in section names ─

/// SEC-100/CWE-116 — section names containing XML-1.0-invalid control chars are
/// sanitized before writing to the `name` attribute of `<p14:section>`.
///
/// `quick-xml`'s `push_attribute` only entity-escapes `& < > " '`.  It does NOT
/// strip XML-1.0-invalid bytes (U+0001–U+0008, U+000B, U+000C, U+000E–U+001F).
/// A raw control character in the `name` attribute renders the resulting
/// `presentation.xml` malformed — PowerPoint 365 cannot open such files (file
/// corruption / CWE-116).
///
/// This test is the load-bearing RED-gate that proves the defect before the fix
/// and proves the fix after.  It:
///
/// 1. Constructs a `SlideSectionEntry` whose name contains U+0001 (SOH) and
///    U+000B (VT) — two distinct classes of XML-1.0-invalid chars.
/// 2. Calls `SectionListBuilder::build_ext_lst` (the direct XML-write path).
/// 3. Asserts the emitted bytes contain NEITHER U+0001 NOR U+000B.
/// 4. Asserts the resulting bytes re-parse cleanly via `quick_xml::Reader`
///    (no malformed-XML error).
/// 5. Asserts the sanitized name (control chars stripped) still appears in the
///    attribute value.
///
/// Traces to BC-4.01.003 EC-004 / SEC-100 / CWE-116.
/// F-P12-HIGH-1 (adversary pass 12, HIGH severity, security/correctness).
#[test]
fn test_sec100_cwe116_section_name_control_chars_stripped_from_p14_section_attr() {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    // Section name with U+0001 (SOH) and U+000B (VT) — both XML-1.0-invalid.
    let name_with_controls = "Back\u{0001}ground\u{000B}Section";
    let expected_sanitized = "BackgroundSection";

    let sections = vec![SlideSectionEntry {
        name: Arc::from(name_with_controls),
        slide_ids: vec![256],
    }];

    let bytes = SectionListBuilder::build_ext_lst(&sections).expect("build_ext_lst must succeed");

    // Assert 1: U+0001 must NOT appear in the emitted bytes.
    assert!(
        !bytes.contains(&0x01u8),
        "SEC-100/CWE-116: U+0001 (SOH) must be stripped from the p14:section name \
         attribute — raw control char renders presentation.xml malformed; \
         found 0x01 byte in emitted XML"
    );

    // Assert 2: U+000B must NOT appear in the emitted bytes.
    assert!(
        !bytes.contains(&0x0Bu8),
        "SEC-100/CWE-116: U+000B (VT) must be stripped from the p14:section name \
         attribute — raw control char renders presentation.xml malformed; \
         found 0x0B byte in emitted XML"
    );

    // Assert 3: The sanitized name (without control chars) must appear in the output.
    let xml_str = String::from_utf8(bytes.clone()).expect("emitted XML must be valid UTF-8");
    assert!(
        xml_str.contains(expected_sanitized),
        "SEC-100/CWE-116: sanitized name '{expected_sanitized}' must appear in \
         p14:section name attribute; got:\n{xml_str}"
    );

    // Assert 4: The full ext_lst block must re-parse as well-formed XML via
    // quick_xml::Reader — no malformed-XML error.
    // Wrap in a root element because quick_xml expects a single root.
    let wrapped = format!("<root>{}</root>", xml_str);
    let mut reader = Reader::from_str(&wrapped);
    reader.config_mut().check_end_names = true;
    let mut event_count = 0usize;
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(_) => event_count += 1,
            Err(e) => panic!(
                "SEC-100/CWE-116: quick_xml::Reader reported malformed XML after \
                 build_ext_lst — raw control chars in name attribute cause XML \
                 well-formedness violation: {e}\nXML was:\n{xml_str}"
            ),
        }
    }
    assert!(
        event_count > 0,
        "quick_xml::Reader must emit at least one event for a non-empty XML block"
    );
}
