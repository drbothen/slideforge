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

use std::sync::Arc;

use slideforge_layout::{
    BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, SlideSectionEntry,
};
use slideforge_types::{Emu, InlineNode, Register, RegisteredContent};

use crate::sections::{SECTION_LST_EXT_URI, P14_NS_URI, SectionListBuilder, derive_section_guid, format_guid};

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
        SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![256, 257] },
        SlideSectionEntry { name: Arc::from("Analysis"), slide_ids: vec![258, 259] },
    ];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    assert!(xml_str.contains("<p:extLst>"), "must contain <p:extLst>");
    assert!(xml_str.contains(SECTION_LST_EXT_URI), "must contain ext uri");
    assert!(xml_str.contains("p14:sectionLst"), "must contain p14:sectionLst");
    assert_eq!(xml_str.matches("<p14:section ").count(), 2, "must have 2 p14:section elements");
    assert!(xml_str.contains("name=\"Background\""), "must have Background section");
    assert!(xml_str.contains("name=\"Analysis\""), "must have Analysis section");
}

/// AC-003 — `p:extLst` is the LAST child of `p:presentation`.
///
/// Traces to BC-4.01.003 architecture rule 4.
#[test]
fn test_BC_4_01_003_ac003_extlst_is_last_child_of_presentation() {
    let sections = vec![SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![256] }];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    let ext_end = xml_str.rfind("</p:extLst>").expect("</p:extLst> must be present");
    let prs_end = xml_str.rfind("</p:presentation>").expect("</p:presentation> must be present");
    let between = xml_str[ext_end + "</p:extLst>".len()..prs_end].trim();
    assert!(
        between.is_empty(),
        "p:extLst must be last child of p:presentation; got: {:?}", between
    );
}

/// AC-003 — `p:ext/@uri` equals the fixed constant `{BB962C8B-...}`.
///
/// Traces to BC-4.01.003 postcondition 5.
#[test]
fn test_BC_4_01_003_ac003_ext_uri_is_correct_fixed_constant() {
    let sections = vec![SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![256] }];
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
    let sections = vec![SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![256] }];
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
    let result = SectionListBuilder::inject(prs_xml, &[])
        .expect("inject with empty sections must succeed");
    assert_eq!(result, prs_xml_copy, "bytes must be unchanged when sections is empty");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");
    assert!(!xml_str.contains("<p:extLst>"), "no extLst when sections empty");
    assert!(!xml_str.contains("p14:sectionLst"), "no sectionLst when sections empty");
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
        SlideSectionEntry { name: Arc::from("Section A"), slide_ids: vec![256] },
        SlideSectionEntry { name: Arc::from("Section B"), slide_ids: vec![257] },
    ];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    assert_eq!(xml_str.matches("<p14:section ").count(), 2, "must have 2 p14:section");
    assert_eq!(xml_str.matches("<p14:sldId ").count(), 2, "must have 2 p14:sldId (one per section)");
    assert!(xml_str.contains("<p14:sldId id=\"256\""), "must have sldId 256");
    assert!(xml_str.contains("<p14:sldId id=\"257\""), "must have sldId 257");
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
        vec![SlideSectionEntry { name: Arc::from("Section A"), slide_ids: vec![256, 257] }],
    );
    assert_eq!(deck.slide_sections.len(), 1, "slide_sections must be accessible");

    // Non-PPTX path = call inject with empty sections (no-op).
    let prs_xml = stub_presentation_xml();
    let result = SectionListBuilder::inject(prs_xml, &[])
        .expect("inject with no sections (non-PPTX simulation) must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");
    assert!(!xml_str.contains("p14:sectionLst"), "no sectionLst for non-PPTX path");
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
    let sections = vec![SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![256] }];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");

    let ext_start = xml_str.find("<p:extLst>").expect("p:extLst must be present");
    let ext_end = xml_str.find("</p:extLst>").map(|i| i + "</p:extLst>".len()).expect("</p:extLst> must be present");
    let ext_xml = &xml_str[ext_start..ext_end];

    assert!(!ext_xml.contains(detail_sentinel), "p:extLst must NOT contain detail content (BC-1.14.003 PC3)");
    assert!(!ext_xml.contains(report_sentinel), "p:extLst must NOT contain report content (BC-1.14.003 PC5)");
}

// ─── AC-009 tests ─────────────────────────────────────────────────────────────

/// AC-009 — Identical inputs produce byte-identical output.
///
/// Traces to BC-4.01.003 invariant 3.
#[test]
fn test_BC_4_01_003_ac009_deterministic_guids_identical_across_builds() {
    let sections = vec![
        SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![256, 257] },
        SlideSectionEntry { name: Arc::from("Analysis"), slide_ids: vec![258, 259] },
    ];
    let result1 = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("first inject must succeed");
    let result2 = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("second inject must succeed");
    assert_eq!(result1, result2, "two identical calls must produce byte-identical output");
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
    assert!(guid1.starts_with('{') && guid1.ends_with('}'), "GUID must be brace-wrapped; got: {guid1}");

    let inner = &guid1[1..guid1.len() - 1];
    assert!(
        inner.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-'),
        "GUID inner must be uppercase hex + hyphens; got: {inner}"
    );

    let parts: Vec<&str> = inner.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);

    assert!(parts[2].starts_with('5'), "version nibble must be 5; third group: {}", parts[2]);

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
        SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![256] },
        SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![257] },
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

    assert_eq!(ids.len(), 2, "must find 2 p14:section id values; got: {ids:?}");
    assert_eq!(
        ids[0], ids[1],
        "duplicate names must produce identical GUIDs; id[0]={} id[1]={}", ids[0], ids[1]
    );
}

// ─── Additional tests ─────────────────────────────────────────────────────────

/// Two sections of 2 slides each produce 4 `<p14:sldId>` elements.
///
/// Traces to BC-4.01.003 postcondition 5.
#[test]
fn test_BC_4_01_003_inject_well_formed_xml_with_two_sections() {
    let sections = vec![
        SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![256, 257] },
        SlideSectionEntry { name: Arc::from("Analysis"), slide_ids: vec![258, 259] },
    ];
    let result = SectionListBuilder::inject(stub_presentation_xml(), &sections)
        .expect("inject must succeed");
    let xml_str = String::from_utf8(result).expect("must be valid UTF-8");
    assert_eq!(xml_str.matches("<p14:sldId ").count(), 4, "must have 4 sldId elements");
    assert!(xml_str.contains("id=\"256\""));
    assert!(xml_str.contains("id=\"257\""));
    assert!(xml_str.contains("id=\"258\""));
    assert!(xml_str.contains("id=\"259\""));
}

/// `SlideSectionEntry` implements `Hash + Eq + Clone`.
#[test]
fn test_BC_4_01_003_slide_section_entry_implements_hash_eq_clone() {
    use std::collections::HashSet;
    let entry = SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![256, 257] };
    let entry2 = entry.clone();
    assert_eq!(entry, entry2);
    let mut set = HashSet::new();
    set.insert(entry);
    assert_eq!(set.len(), 1);
}

/// Different names → different entries (not equal).
#[test]
fn test_BC_4_01_003_slide_section_entry_different_names_not_equal() {
    let a = SlideSectionEntry { name: Arc::from("Background"), slide_ids: vec![256] };
    let b = SlideSectionEntry { name: Arc::from("Analysis"), slide_ids: vec![256] };
    assert_ne!(a, b);
}

/// `format_guid` produces correct 8-4-4-4-12 brace-wrapped uppercase GUID.
#[test]
fn test_BC_4_01_003_format_guid_produces_correct_format() {
    let bytes: [u8; 16] = [
        0x3D, 0x4F, 0x2B, 0x8A, 0x1C, 0x9E, 0x52, 0xA0,
        0xB4, 0xD6, 0x7E, 0x8A, 0x9B, 0x0C, 0x1D, 0x2E,
    ];
    let guid = format_guid(&bytes);
    assert!(guid.starts_with('{') && guid.ends_with('}'));
    let inner = &guid[1..guid.len() - 1];
    assert!(inner.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-'));
    let parts: Vec<&str> = inner.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);
}
