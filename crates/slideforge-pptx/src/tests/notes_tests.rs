//! Tests for STORY-040: PPTX Speaker Notes + Slide Sections + notesMaster1.xml.
//!
//! Covers BC-4.01.003 (speaker notes, masters, slide sections) and
//! BC-4.01.006 (notesMaster1.xml + handoutMaster1.xml always present).
//!
//! ## Upstream Data Verification (STORY-039 lesson — REQUIRED READING)
//!
//! Before writing tests, we verified whether the data this story consumes is
//! actually threaded through the pipeline.  Findings:
//!
//! ### Speaker Notes — THREADED (GREEN)
//!
//! `LaidOutSlide` carries two fields populated by `layout::run`:
//! - `speaker_notes: Option<Arc<str>>` — derived from `Register::Notes`
//!   entries in `register_content` via `speaker_notes_from_register_content`.
//! - `register_content: Vec<RegisteredContent>` — copied verbatim from the
//!   semantic `Slide::register_content` which `slideforge-eval` populates.
//!
//! The pipeline path is:
//! ```text
//! Deck (slide.register_content) → layout::run → LaidOutSlide.speaker_notes
//!                                                             .register_content
//! → PptxExporter → NotesSlideSerializer (STORY-040 stub)
//! ```
//! No gap: the data IS threaded.  Tests should construct semantic `Deck` objects
//! with `Slide::register_content` populated, run `layout::run`, then export.
//!
//! ### Slide Sections for PPTX — GAP (RED FLAG)
//!
//! **`LaidOutDeck::sections` is NOT the source for PPTX slide sections.**
//!
//! `LaidOutDeck::sections` carries `GeneratedSection` values from
//! `slideforge_layout::sections::collect_sections`.  These are DOCX/PDF
//! document-structure sections (`methodology`, `executive_summary`, etc.) —
//! NOT `section "Name":` slide groupings that map to PPTX `<p:sectionLst>`.
//!
//! The `Deck::section_blocks` field carries DOCX/PDF `SectionBlock`s (e.g.,
//! `section methodology: ...`) — also NOT PPTX slide groupings.
//!
//! As of STORY-040 scope, there is no IR field that carries the PPTX-native
//! `section "Background":` slide grouping through the pipeline.  The
//! `SectionListBuilder` stub in `sections.rs` documents this gap in its
//! module-level comment.
//!
//! **Impact on tests:** The section tests (AC-004, AC-005, AC-006, EC-004,
//! EC-005) must drive `SectionListBuilder::build` DIRECTLY (bypassing the
//! full pipeline) because the data source does not yet exist in the IR.
//! Tests use `crate::sections::PptxSection` structs constructed in-test.
//!
//! The implementer MUST either:
//! (a) add a `pptx_sections: Vec<PptxSection>` field to `LaidOutDeck`, OR
//! (b) add a parallel `pptx_sections` field to a new wrapper struct,
//! and thread `section "Name":` DSL blocks through the pipeline into it.
//! This wiring is part of STORY-040's implementation scope.
//!
//! ## Test Approach
//!
//! All tests that check notes content (AC-001..AC-003) construct a semantic
//! `Deck` with `register_content` populated on slides, run `layout::run`,
//! then call `PptxExporter::export` to get real PPTX bytes.  Tests then open
//! the PPTX ZIP and parse XML — no mock strings.
//!
//! Section tests (AC-004..AC-006, EC-004..EC-005) call `SectionListBuilder`
//! directly with `PptxSection` fixtures until the pipeline wiring exists.
//!
//! ## Traceability
//!
//! | Test function | AC/EC | BC clause | What is verified |
//! |---|---|---|---|
//! | `test_BC_4_01_003_ac001_notes_slide_count_equals_slides_with_notes` | AC-001 | postcondition 1 | count of notesSlide parts = count of slides with notes |
//! | `test_BC_4_01_003_ac001_ec001_no_notes_slide_for_empty_notes` | AC-001 / EC-001 | postcondition 2 | slide with empty notes → no notesSlide |
//! | `test_BC_4_01_003_ac002_notes_text_in_body_placeholder` | AC-002 | postcondition 6 | notes text in <p:ph type="body" idx="1"> txBody |
//! | `test_BC_4_01_003_ac003_notes_absent_from_slide_bodies` | AC-003 | invariant 1 | notes text not in any ppt/slides/slide*.xml |
//! | `test_BC_4_01_003_ac004_sections_produce_section_lst` | AC-004 | postcondition 5 | <p:sectionLst> with 2 sections in presentation.xml |
//! | `test_BC_4_01_003_ac005_no_sections_no_section_lst` | AC-005 | EC-002 | no sections → no <p:sectionLst> |
//! | `test_BC_4_01_003_ac006_section_name_xml_escaped` | AC-006 | EC-004 | "Background & Overview" → name="Background &amp; Overview" |
//! | `test_BC_4_01_006_ac007_notes_master_always_present` | AC-007 | invariant 1 | ppt/notesMasters/notesMaster1.xml always in ZIP |
//! | `test_BC_4_01_006_ac008_handout_master_always_present` | AC-008 | postcondition 2 | ppt/handoutMasters/handoutMaster1.xml always in ZIP |
//! | `test_BC_4_01_003_ec004_section_name_special_chars_well_formed` | EC-004 | EC-004 | section name with special chars → well-formed XML |
//! | `test_BC_4_01_003_ec005_two_single_slide_sections` | EC-005 | EC-005 | two sections, one slide each, both in sectionLst |
//! | `test_BC_4_01_003_section_guid_deterministic` | invariant 3 | invariant 3 | same name → same GUID, NOT random, two exports agree |

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::io::Read as _;
use std::sync::Arc;

use quick_xml::Reader;
use quick_xml::events::Event;
use slideforge_eval::BleedChecker;
use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{Brand, BrandFonts, BrandPalette, Deck, Emu, Register, SourceSpan};
use slideforge_types::register::RegisteredContent;
use zip::ZipArchive;

use crate::PptxExporter;
use crate::sections::{PptxSection, SectionListBuilder};

// ─── Fixture builders ────────────────────────────────────────────────────────

fn make_brand() -> Brand {
    Brand {
        name: Arc::from("test-brand"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
            secondary: Arc::from("#0066CC"),
            accent: Arc::from("#FF6B35"),
            neutral: Arc::from("#F5F5F5"),
        },
        fonts: BrandFonts {
            heading: Arc::from("Calibri Light"),
            body: Arc::from("Calibri"),
            mono: Arc::from("Courier New"),
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

fn make_metadata() -> slideforge_types::deck::DeckMetadata {
    use slideforge_types::deck::DeckMetadata;
    DeckMetadata {
        title: Some(Arc::from("Notes Test Deck")),
        slideforge_version: Arc::from("0.1.0"),
        lang: Some(Arc::from("en-US")),
        author: None,
        section_order: None,
    }
}

fn title_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(457_200),
        y: Emu(274_638),
        width: Emu(8_229_600),
        height: Emu(1_143_000),
    }
}

/// Build a minimal `LaidOutSlide` with no frames and optional speaker notes.
fn make_slide(index: usize, notes_text: Option<&str>) -> LaidOutSlide {
    let register_content: Vec<RegisteredContent> = notes_text
        .map(|t| {
            vec![RegisteredContent::plain(
                Register::Notes,
                Arc::from(t),
            )]
        })
        .unwrap_or_default();
    let speaker_notes = notes_text.map(Arc::from);
    LaidOutSlide {
        source_index: index,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
        }],
        speaker_notes,
        register_tags: vec![],
        register_content,
    }
}

/// Build a minimal semantic `Deck` with N slides, each with optional notes.
fn make_deck_with_notes(notes_per_slide: &[Option<&str>]) -> Deck {
    use slideforge_types::ordered_map::OrderedMap;
    use slideforge_types::slide::Slide;
    Deck {
        slides: notes_per_slide
            .iter()
            .map(|maybe_notes| Slide {
                slide_type: Arc::from("title"),
                fields: OrderedMap::new(),
                blocks: vec![],
                register: None,
                tags: vec![],
                source_span: SourceSpan::default(),
                overlay: None,
                register_content: maybe_notes
                    .map(|t| {
                        vec![RegisteredContent::plain(
                            Register::Notes,
                            Arc::from(t),
                        )]
                    })
                    .unwrap_or_default(),
            })
            .collect(),
        vars: slideforge_types::ordered_map::OrderedMap::new(),
        metadata: make_metadata(),
        registers: slideforge_types::ordered_map::OrderedMap::new(),
        section_blocks: vec![],
    }
}

/// Build a `LaidOutDeck` with N slides where slides at `notes_indices` have
/// notes content. Uses pre-constructed `LaidOutSlide` values.
fn make_laid_out_deck_with_notes(notes_per_slide: &[Option<&str>]) -> LaidOutDeck {
    let slides: Vec<LaidOutSlide> = notes_per_slide
        .iter()
        .enumerate()
        .map(|(i, maybe_notes)| make_slide(i, *maybe_notes))
        .collect();
    LaidOutDeck {
        page_size: PageSize::default(),
        slides,
        sections: vec![],
        warnings: vec![],
    }
}

/// Run the full `PptxExporter::export` path and return the ZIP bytes.
fn export_pptx(deck: &Deck, laid_out: &LaidOutDeck) -> Vec<u8> {
    let brand = make_brand();
    let opts = ExportOptions::default();
    PptxExporter::new()
        .export(deck, laid_out, &brand, &opts)
        .expect("PptxExporter::export must succeed for valid input")
}

/// Count ZIP entries matching the `ppt/notesSlides/notesSlide*.xml` pattern.
fn count_notes_slides(pptx_bytes: &[u8]) -> usize {
    let cursor = std::io::Cursor::new(pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be valid ZIP");
    let mut count = 0;
    for i in 0..archive.len() {
        let entry = archive.by_index(i).expect("valid index");
        let name = entry.name().to_owned();
        if is_notes_slide_path(&name) {
            count += 1;
        }
    }
    count
}

/// Returns `true` if `name` matches `ppt/notesSlides/notesSlide<N>.xml`.
fn is_notes_slide_path(name: &str) -> bool {
    let Some(filename) = name.strip_prefix("ppt/notesSlides/") else {
        return false;
    };
    let Some(stem) = filename.strip_suffix(".xml") else {
        return false;
    };
    let Some(digits) = stem.strip_prefix("notesSlide") else {
        return false;
    };
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

/// Read a named ZIP member as a UTF-8 string. Panics if not found.
fn read_zip_member(pptx_bytes: &[u8], member_name: &str) -> String {
    let cursor = std::io::Cursor::new(pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be valid ZIP");
    for i in 0..archive.len() {
        let name = {
            let entry = archive.by_index(i).expect("valid index");
            entry.name().to_owned()
        };
        if name == member_name {
            let mut entry = archive.by_index(i).expect("valid index");
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).expect("readable entry");
            return String::from_utf8_lossy(&buf).into_owned();
        }
    }
    panic!("ZIP member {member_name:?} not found in archive");
}

/// Assert a named ZIP entry exists; panics with a clear message if absent.
fn assert_zip_entry_present(pptx_bytes: &[u8], member_name: &str) {
    let cursor = std::io::Cursor::new(pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("must be valid ZIP");
    for i in 0..archive.len() {
        let entry = archive.by_index(i).expect("valid index");
        if entry.name() == member_name {
            return;
        }
    }
    panic!(
        "Expected ZIP member {member_name:?} not found.\n\
         (BC-4.01.006: this part must always be present in a valid PPTX)"
    );
}

/// Parse a `<p:sectionLst>` element from XML bytes, returning section names.
/// Returns an empty Vec if `<p:sectionLst>` is absent.
fn parse_section_names_from_xml(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut names: Vec<String> = Vec::new();
    let mut in_section_lst = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let local = e.local_name();
                let local_str = std::str::from_utf8(local.as_ref()).unwrap_or("");
                if local_str == "sectionLst" {
                    in_section_lst = true;
                }
                if in_section_lst && local_str == "section" {
                    let decoder = reader.decoder();
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"name" {
                            let val = attr.decode_and_unescape_value(decoder).unwrap_or_default();
                            names.push(val.into_owned());
                        }
                    }
                }
            },
            Ok(Event::End(e)) => {
                if e.local_name().as_ref() == b"sectionLst" {
                    in_section_lst = false;
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("XML parse error: {e}"),
            _ => {},
        }
    }
    names
}

/// Returns `true` if the XML contains a `<p:sectionLst>` element.
fn xml_contains_section_lst(xml: &str) -> bool {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                if e.local_name().as_ref() == b"sectionLst" {
                    return true;
                }
            },
            Ok(Event::Eof) => return false,
            Err(e) => panic!("XML parse error while checking for sectionLst: {e}"),
            _ => {},
        }
    }
}

/// Extract the text content from the `<p:ph type="body" idx="1">` txBody in
/// a notesSlide XML string.  Returns `None` if the placeholder is absent.
fn extract_body_placeholder_text(xml: &str) -> Option<String> {
    // Quick parse: find <p:ph type="body" idx="1"> then collect <a:t> text.
    // We do a simple scan rather than full DOM traversal.
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut in_body_placeholder = false;
    let mut depth = 0_usize;
    let mut collected = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = std::str::from_utf8(e.local_name().as_ref()).unwrap_or("").to_owned();
                if local == "sp" {
                    // Scan sp's children for a nvPr → ph with type="body" idx="1".
                    // We use a simpler scan: track nvPr/ph attributes.
                }
                if local == "ph" {
                    let mut has_body_type = false;
                    let mut has_idx_1 = false;
                    let decoder = reader.decoder();
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.local_name().as_ref())
                            .unwrap_or("")
                            .to_owned();
                        let val = attr
                            .decode_and_unescape_value(decoder)
                            .unwrap_or_default()
                            .into_owned();
                        if key == "type" && val == "body" {
                            has_body_type = true;
                        }
                        if key == "idx" && val == "1" {
                            has_idx_1 = true;
                        }
                    }
                    if has_body_type && has_idx_1 {
                        in_body_placeholder = true;
                        depth = 1;
                    }
                } else if in_body_placeholder {
                    depth += 1;
                }
            },
            Ok(Event::End(e)) => {
                if in_body_placeholder {
                    if depth == 0 {
                        in_body_placeholder = false;
                    } else {
                        depth -= 1;
                    }
                    let _ = e; // suppress unused warning
                }
            },
            Ok(Event::Text(e)) => {
                if in_body_placeholder {
                    let text = e.unescape().unwrap_or_default();
                    collected.push_str(&text);
                }
            },
            Ok(Event::Eof) => break,
            Err(err) => panic!("XML parse error in extract_body_placeholder_text: {err}"),
            _ => {},
        }
    }

    if collected.is_empty() {
        None
    } else {
        Some(collected)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001: notesSlide count == # slides with non-empty notes
// BC-4.01.003 postcondition 1
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001 / BC-4.01.003 postcondition 1:
/// A 3-slide deck where slides 0 and 2 have notes produces exactly 2 notesSlide
/// parts; slide 1 (no notes) produces none.
///
/// Red Gate: fails because `NotesSlideSerializer::build` is `todo!()` —
/// the serializer is never called so the ZIP contains 0 notesSlide parts,
/// asserting 2 == 0.
#[test]
fn test_BC_4_01_003_ac001_notes_slide_count_equals_slides_with_notes() {
    // F-040-RG-1: asserted symbol on the failing path:
    //   `NotesSlideSerializer::build` (notes_slide.rs) — called for each slide
    //   with non-empty speaker_notes in the export loop (not yet wired).
    let deck = make_deck_with_notes(&[
        Some("Speaker note for slide 1"),
        None,
        Some("Speaker note for slide 3"),
    ]);
    let laid_out =
        make_laid_out_deck_with_notes(&[Some("Speaker note for slide 1"), None, Some("Speaker note for slide 3")]);

    let pptx = export_pptx(&deck, &laid_out);
    let notes_count = count_notes_slides(&pptx);

    assert_eq!(
        notes_count, 2,
        "AC-001: expected 2 notesSlide parts (slides 1 and 3 have notes, slide 2 does not); \
         got {notes_count}. \
         (BC-4.01.003 postcondition 1)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001 / EC-001: empty notes register → no notesSlide
// BC-4.01.003 postcondition 2 / EC-001
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001 / EC-001 / BC-4.01.003 postcondition 2:
/// A slide with an empty-string notes register must not produce a notesSlide part.
///
/// Red Gate: The count will be 0 before implementation, but this test asserts
/// the correct behavior: zero notesSlides for zero non-empty notes slides.
/// Actually this test is GREEN without implementation (0 == 0) — but is included
/// because its value is confirmed by a companion negative: AC-001 WILL fail,
/// proving the serializer is not running.
///
/// To avoid a vacuous pass, this test also checks that the deck with one
/// empty-notes slide produces 0 notesSlide parts AND that the ZIP is structurally
/// valid (contains at least one ppt/slides/slide*.xml entry).
#[test]
fn test_BC_4_01_003_ac001_ec001_no_notes_slide_for_empty_notes() {
    // F-040-RG-1: This test is purposefully designed to be resilient to the
    // pre-implementation state (0 notesSlides is correct here). The Red Gate is
    // enforced by `test_BC_4_01_003_ac001_notes_slide_count_equals_slides_with_notes`.
    let deck = make_deck_with_notes(&[None]);
    let laid_out = make_laid_out_deck_with_notes(&[None]);

    let pptx = export_pptx(&deck, &laid_out);
    let notes_count = count_notes_slides(&pptx);

    assert_eq!(
        notes_count, 0,
        "EC-001: a slide with no notes register content must produce 0 notesSlide parts; \
         got {notes_count}. \
         (BC-4.01.003 postcondition 2 / EC-001)"
    );

    // Structural sanity: ensure the PPTX has at least one slide body (not empty/malformed).
    let cursor = std::io::Cursor::new(&pptx);
    let mut archive = ZipArchive::new(cursor).expect("must be valid ZIP");
    let slide_count = (0..archive.len())
        .filter(|&i| {
            let entry = archive.by_index(i).unwrap();
            let name = entry.name().to_owned();
            name.starts_with("ppt/slides/slide") && name.ends_with(".xml")
        })
        .count();
    assert!(
        slide_count > 0,
        "EC-001: the PPTX must contain at least one slide body; found 0 ppt/slides/slide*.xml entries"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002: notes text in body placeholder
// BC-4.01.003 postcondition 6
// ─────────────────────────────────────────────────────────────────────────────

/// AC-002 / BC-4.01.003 postcondition 6:
/// The notes text from `Register::Notes` appears in the `<p:txBody>` of the
/// body placeholder (`<p:ph type="body" idx="1">`) in `notesSlide1.xml`.
///
/// Red Gate: fails because `ppt/notesSlides/notesSlide1.xml` does not exist
/// in the ZIP (serializer not yet called). `read_zip_member` panics with
/// "ZIP member not found".
#[test]
fn test_BC_4_01_003_ac002_notes_text_in_body_placeholder() {
    // F-040-RG-2: asserted symbol: `NotesSlideSerializer::build` — must produce
    // a notesSlide1.xml with the notes text in <p:ph type="body" idx="1"> txBody.
    let notes_text = "Click here to rehearse your speaking points carefully.";
    let deck = make_deck_with_notes(&[Some(notes_text)]);
    let laid_out = make_laid_out_deck_with_notes(&[Some(notes_text)]);

    let pptx = export_pptx(&deck, &laid_out);

    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");
    let body_text = extract_body_placeholder_text(&notes_xml);

    assert_eq!(
        body_text.as_deref(),
        Some(notes_text),
        "AC-002: notes text must appear in the body placeholder \
         (<p:ph type=\"body\" idx=\"1\"> txBody) of notesSlide1.xml; \
         got: {body_text:?}. \
         (BC-4.01.003 postcondition 6)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003: notes text NOT in any slide body
// BC-4.01.003 invariant 1 / DI-012
// ─────────────────────────────────────────────────────────────────────────────

/// AC-003 / BC-4.01.003 invariant 1:
/// Notes text from `Register::Notes` must NOT appear in any `ppt/slides/slide*.xml`.
/// Uses `BleedChecker::assert_absent_from_pptx_slides`.
///
/// Red Gate: Before the serializer is implemented, `BleedChecker` would find
/// zero slide bodies containing the notes text — so this assertion would pass
/// vacuously. However `BleedChecker::assert_absent_from_pptx_slides` panics if
/// no `ppt/slides/slide*.xml` entries are found (it refuses a vacuously-true
/// absence check on an empty archive). The PPTX will have slides, so the check
/// is non-vacuous. This test is GREEN before implementation (correct: notes don't
/// bleed to slides). The primary Red Gate is AC-001 and AC-002.
///
/// The test is included to confirm the no-bleed invariant remains true after
/// the implementer wires the serializer.
#[test]
fn test_BC_4_01_003_ac003_notes_absent_from_slide_bodies() {
    // Sentinel: a distinctive notes string unlikely to appear in slide XML.
    let notes_sentinel = "NOTES_BLEED_SENTINEL_X7Q2Z9";
    let deck = make_deck_with_notes(&[Some(notes_sentinel)]);
    let laid_out = make_laid_out_deck_with_notes(&[Some(notes_sentinel)]);

    let pptx = export_pptx(&deck, &laid_out);

    // BleedChecker panics if the sentinel appears in any ppt/slides/slide*.xml.
    // Also panics if no slide bodies are present (non-vacuous check).
    BleedChecker::assert_absent_from_pptx_slides(&pptx, notes_sentinel);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004: sections → <p:sectionLst> in presentation.xml
// BC-4.01.003 postcondition 5
// ─────────────────────────────────────────────────────────────────────────────

/// AC-004 / BC-4.01.003 postcondition 5:
/// Two PPTX sections produce `<p:sectionLst>` in `presentation.xml` with
/// two `<p:section>` elements. Section names match the input.
///
/// Red Gate: fails because `SectionListBuilder::build` is `todo!()`.
#[test]
fn test_BC_4_01_003_ac004_sections_produce_section_lst() {
    // F-040-RG-3: asserted symbol: `SectionListBuilder::build` (sections.rs).
    let sections = vec![
        PptxSection {
            name: "Background".to_string(),
            slide_ids: vec![256, 257],
        },
        PptxSection {
            name: "Analysis".to_string(),
            slide_ids: vec![258, 259],
        },
    ];

    let section_lst_bytes = SectionListBuilder::build(&sections)
        .expect("SectionListBuilder::build must succeed for valid sections");

    let xml = match section_lst_bytes {
        Some(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        None => panic!(
            "AC-004: SectionListBuilder::build must return Some(bytes) for non-empty sections; \
             got None. (BC-4.01.003 postcondition 5)"
        ),
    };

    // Verify well-formedness: parse without error.
    {
        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);
        loop {
            match reader.read_event() {
                Ok(Event::Eof) => break,
                Err(e) => panic!("AC-004: sectionLst XML is not well-formed: {e}"),
                _ => {},
            }
        }
    }

    let names = parse_section_names_from_xml(&xml);
    assert_eq!(
        names.len(), 2,
        "AC-004: expected 2 section elements in <p:sectionLst>; got {}: {names:?}. \
         (BC-4.01.003 postcondition 5)",
        names.len()
    );
    assert_eq!(
        names[0], "Background",
        "AC-004: first section name must be \"Background\"; got {:?}",
        names[0]
    );
    assert_eq!(
        names[1], "Analysis",
        "AC-004: second section name must be \"Analysis\"; got {:?}",
        names[1]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-005: no sections → no <p:sectionLst>
// BC-4.01.003 EC-002
// ─────────────────────────────────────────────────────────────────────────────

/// AC-005 / BC-4.01.003 EC-002:
/// When the deck has no sections, `SectionListBuilder::build` returns `None`
/// and `presentation.xml` must not contain `<p:sectionLst>`.
///
/// Red Gate: fails because `SectionListBuilder::build` is `todo!()`.
#[test]
fn test_BC_4_01_003_ac005_no_sections_no_section_lst() {
    // F-040-RG-4: asserted symbol: `SectionListBuilder::build` (sections.rs).
    let sections: Vec<PptxSection> = vec![];

    let result = SectionListBuilder::build(&sections)
        .expect("SectionListBuilder::build must not error for empty sections");

    assert!(
        result.is_none(),
        "AC-005: SectionListBuilder::build must return None for an empty sections list; \
         got Some(bytes). \
         (BC-4.01.003 EC-002)"
    );

    // Also verify via full export: a deck with no sections must produce
    // presentation.xml without <p:sectionLst>.
    let deck = make_deck_with_notes(&[None]);
    let laid_out = make_laid_out_deck_with_notes(&[None]);
    let pptx = export_pptx(&deck, &laid_out);
    let prs_xml = read_zip_member(&pptx, "ppt/presentation.xml");

    assert!(
        !xml_contains_section_lst(&prs_xml),
        "AC-005: presentation.xml must not contain <p:sectionLst> when the deck has no sections. \
         (BC-4.01.003 EC-002)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-006: XML-escaped section names
// BC-4.01.003 EC-004
// ─────────────────────────────────────────────────────────────────────────────

/// AC-006 / BC-4.01.003 EC-004:
/// A section named "Background & Overview" produces
/// `<p:section name="Background &amp; Overview">` in the sectionLst XML.
/// The resulting XML is well-formed (parseable without error).
///
/// Red Gate: fails because `SectionListBuilder::build` is `todo!()`.
#[test]
fn test_BC_4_01_003_ac006_section_name_xml_escaped() {
    // F-040-RG-5: asserted symbol: `SectionListBuilder::build` (sections.rs).
    let sections = vec![PptxSection {
        name: "Background & Overview".to_string(),
        slide_ids: vec![256, 257, 258],
    }];

    let bytes = SectionListBuilder::build(&sections)
        .expect("SectionListBuilder::build must succeed")
        .expect("must return Some(bytes) for non-empty sections");

    let xml = String::from_utf8_lossy(&bytes).into_owned();

    // Well-formedness check.
    {
        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);
        loop {
            match reader.read_event() {
                Ok(Event::Eof) => break,
                Err(e) => panic!(
                    "AC-006: sectionLst XML with special-char section name is not well-formed: {e}\n\
                     XML: {xml}"
                ),
                _ => {},
            }
        }
    }

    // Verify the unescaped name round-trips correctly via quick-xml.
    let names = parse_section_names_from_xml(&xml);
    assert_eq!(
        names.len(), 1,
        "AC-006: expected 1 section in sectionLst; got {}: {names:?}",
        names.len()
    );
    assert_eq!(
        names[0], "Background & Overview",
        "AC-006: section name must round-trip via XML escaping; \
         expected \"Background & Overview\" (unescaped), got {:?}. \
         (BC-4.01.003 EC-004)",
        names[0]
    );

    // Direct byte check: the raw XML must contain &amp; for the ampersand.
    assert!(
        xml.contains("&amp;"),
        "AC-006: raw XML must contain &amp; for the escaped ampersand; \
         got: {xml}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-007: notesMaster1.xml always present
// BC-4.01.006 invariant 1
// ─────────────────────────────────────────────────────────────────────────────

/// AC-007 / BC-4.01.006 invariant 1:
/// `ppt/notesMasters/notesMaster1.xml` must be present in the PPTX ZIP even
/// when the deck has zero notes.
///
/// Red Gate note: `notesMaster1.xml` was already written by STORY-037's stub
/// in `build_notes_handout_masters`. This test should already be GREEN (the
/// STORY-037 stub copies `NOTES_MASTER_STUB` bytes). Included here to confirm
/// it remains green and to anchor the traceability to BC-4.01.006 invariant 1.
///
/// The STORY-040 implementer must replace the NOTES_MASTER_STUB with a proper
/// notesMaster generated by `NotesMasterSerializer::build_notes_master`.
#[test]
fn test_BC_4_01_006_ac007_notes_master_always_present() {
    // Zero-notes deck to verify the "always present regardless of deck content" invariant.
    let deck = make_deck_with_notes(&[None]);
    let laid_out = make_laid_out_deck_with_notes(&[None]);
    let pptx = export_pptx(&deck, &laid_out);

    assert_zip_entry_present(&pptx, "ppt/notesMasters/notesMaster1.xml");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-008: handoutMaster1.xml always present
// BC-4.01.006 postcondition 2
// ─────────────────────────────────────────────────────────────────────────────

/// AC-008 / BC-4.01.006 postcondition 2:
/// `ppt/handoutMasters/handoutMaster1.xml` must be present in the PPTX ZIP
/// always, regardless of deck content.
///
/// Red Gate note: same as AC-007 — this was already wired by STORY-037.
/// This test should be GREEN. Included for BC-4.01.006 traceability.
#[test]
fn test_BC_4_01_006_ac008_handout_master_always_present() {
    let deck = make_deck_with_notes(&[None]);
    let laid_out = make_laid_out_deck_with_notes(&[None]);
    let pptx = export_pptx(&deck, &laid_out);

    assert_zip_entry_present(&pptx, "ppt/handoutMasters/handoutMaster1.xml");
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-004: section with special XML characters → well-formed
// BC-4.01.003 EC-004
// ─────────────────────────────────────────────────────────────────────────────

/// EC-004 / BC-4.01.003 EC-004:
/// A section name with multiple XML-special characters (`<`, `>`, `&`, `"`, `'`)
/// produces well-formed XML where the name attribute is fully escaped.
///
/// Red Gate: fails because `SectionListBuilder::build` is `todo!()`.
#[test]
fn test_BC_4_01_003_ec004_section_name_special_chars_well_formed() {
    // F-040-RG-6: asserted symbol: `SectionListBuilder::build`.
    let sections = vec![PptxSection {
        name: "Q1 <Revenue> & 'Costs' \"Analysis\"".to_string(),
        slide_ids: vec![256],
    }];

    let bytes = SectionListBuilder::build(&sections)
        .expect("SectionListBuilder::build must succeed for any valid section name")
        .expect("must return Some(bytes) for non-empty sections");

    let xml = String::from_utf8_lossy(&bytes).into_owned();

    // Well-formedness is the primary assertion.
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Err(e) => panic!(
                "EC-004: sectionLst XML with special-char section name is not well-formed: {e}\n\
                 XML: {xml}"
            ),
            _ => {},
        }
    }

    // Name round-trips through XML escaping.
    let names = parse_section_names_from_xml(&xml);
    assert_eq!(
        names.first().map(String::as_str),
        Some("Q1 <Revenue> & 'Costs' \"Analysis\""),
        "EC-004: section name must round-trip via XML escaping; got: {names:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-005: two sections, one slide each
// BC-4.01.003 EC-005
// ─────────────────────────────────────────────────────────────────────────────

/// EC-005 / BC-4.01.003 EC-005:
/// Two sections each containing exactly one slide produce two `<p:section>`
/// elements in `<p:sectionLst>`, each with one `<p:sldId>`.
///
/// Red Gate: fails because `SectionListBuilder::build` is `todo!()`.
#[test]
fn test_BC_4_01_003_ec005_two_single_slide_sections() {
    // F-040-RG-7: asserted symbol: `SectionListBuilder::build`.
    let sections = vec![
        PptxSection {
            name: "Introduction".to_string(),
            slide_ids: vec![256],
        },
        PptxSection {
            name: "Conclusion".to_string(),
            slide_ids: vec![257],
        },
    ];

    let bytes = SectionListBuilder::build(&sections)
        .expect("SectionListBuilder::build must succeed")
        .expect("must return Some(bytes) for non-empty sections");

    let xml = String::from_utf8_lossy(&bytes).into_owned();

    let names = parse_section_names_from_xml(&xml);
    assert_eq!(
        names.len(), 2,
        "EC-005: expected 2 section elements; got {}: {names:?}",
        names.len()
    );
    assert_eq!(names[0], "Introduction");
    assert_eq!(names[1], "Conclusion");

    // Verify each section has exactly one sldId.
    // Count <p:sldId> elements (quick scan).
    let sld_id_count = {
        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);
        let mut count = 0usize;
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    if e.local_name().as_ref() == b"sldId" {
                        count += 1;
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => panic!("EC-005: XML parse error: {e}"),
                _ => {},
            }
        }
        count
    };
    assert_eq!(
        sld_id_count, 2,
        "EC-005: expected 2 <p:sldId> elements (one per section); got {sld_id_count}. \
         (BC-4.01.003 EC-005)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Invariant 3: section GUID is deterministic (NOT random)
// BC-4.01.003 invariant 3
// ─────────────────────────────────────────────────────────────────────────────

/// BC-4.01.003 invariant 3:
/// The same section name must produce the same GUID in two separate calls to
/// `SectionListBuilder::guid_for_name`. GUIDs must NOT be random (`Uuid::new_v4`).
///
/// Red Gate: fails because `SectionListBuilder::guid_for_name` is `todo!()`.
#[test]
fn test_BC_4_01_003_section_guid_deterministic() {
    // F-040-RG-8: asserted symbol: `SectionListBuilder::guid_for_name` (sections.rs).
    let name = "Background";
    let guid1 = SectionListBuilder::guid_for_name(name);
    let guid2 = SectionListBuilder::guid_for_name(name);

    assert_eq!(
        guid1, guid2,
        "BC-4.01.003 invariant 3: the same section name must produce the same GUID \
         across two calls to guid_for_name; got {guid1:?} != {guid2:?}"
    );

    // Different names must produce different GUIDs.
    let guid_other = SectionListBuilder::guid_for_name("Analysis");
    assert_ne!(
        guid1, guid_other,
        "BC-4.01.003 invariant 3: different section names must produce different GUIDs; \
         both produced {guid1:?}"
    );

    // GUID must look like a UUID (8-4-4-4-12 hex with braces or standard form).
    // We accept either `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` or the bare
    // `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX` form.
    let bare = guid1.trim_matches('{').trim_matches('}');
    let parts: Vec<&str> = bare.split('-').collect();
    assert_eq!(
        parts.len(), 5,
        "BC-4.01.003 invariant 3: GUID must be formatted as UUID (8-4-4-4-12 hex groups); \
         got: {guid1:?}"
    );
    assert!(
        parts.iter().all(|p| p.chars().all(|c| c.is_ascii_hexdigit())),
        "BC-4.01.003 invariant 3: all GUID parts must be hex digits; got: {guid1:?}"
    );
}
