//! Tests for STORY-040: PPTX Speaker Notes + notesMaster1.xml + handoutMaster1.xml.
//!
//! Covers BC-4.01.003 (speaker notes) and BC-4.01.006 (notesMaster1.xml +
//! handoutMaster1.xml always present).
//!
//! Slide sections (`<p:sectionLst>`) were split to STORY-082 per human-authorized
//! scope split 2026-06-04. All section-related tests (former AC-004..AC-006,
//! EC-004..EC-005, GUID determinism) have been removed. `sections.rs` and the
//! `sha2` production dependency have been removed from this crate.
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
//! No gap: the data IS threaded.  Tests construct semantic `Deck` objects
//! with `Slide::register_content` populated, run through `PptxExporter::export`,
//! then open the PPTX ZIP and parse XML — no mock strings.
//!
//! ## Traceability
//!
//! | Test function | AC/EC | BC clause | What is verified |
//! |---|---|---|---|
//! | `test_BC_4_01_003_ac001_notes_slide_count_equals_slides_with_notes` | AC-001 | postcondition 1 | count of notesSlide parts = count of slides with notes |
//! | `test_BC_4_01_003_ac001_ec001_no_notes_slide_for_empty_notes` | AC-001 / EC-001 | postcondition 2 | slide with empty notes → no notesSlide |
//! | `test_BC_4_01_003_ac002_notes_text_in_body_placeholder` | AC-002 | postcondition 6 | notes text in <p:ph type="body" idx="1"> txBody |
//! | `test_BC_4_01_003_ac003_notes_absent_from_slide_bodies` | AC-003 | invariant 1 | notes text not in any ppt/slides/slide*.xml |
//! | `test_BC_4_01_006_ac004_notes_master_always_present` | AC-004 | invariant 1 | ppt/notesMasters/notesMaster1.xml always in ZIP |
//! | `test_BC_4_01_006_ac005_handout_master_always_present` | AC-005 | postcondition 2 | ppt/handoutMasters/handoutMaster1.xml always in ZIP |

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
use slideforge_types::register::RegisteredContent;
use slideforge_types::{Brand, BrandFonts, BrandPalette, Deck, Emu, Register, SourceSpan};
use zip::ZipArchive;

use crate::PptxExporter;

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
        .map(|t| vec![RegisteredContent::plain(Register::Notes, Arc::from(t))])
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
                    .map(|t| vec![RegisteredContent::plain(Register::Notes, Arc::from(t))])
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
                let local = std::str::from_utf8(e.local_name().as_ref())
                    .unwrap_or("")
                    .to_owned();
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
    let laid_out = make_laid_out_deck_with_notes(&[
        Some("Speaker note for slide 1"),
        None,
        Some("Speaker note for slide 3"),
    ]);

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
// AC-004: notesMaster1.xml always present
// BC-4.01.006 invariant 1
// ─────────────────────────────────────────────────────────────────────────────

/// AC-004 / BC-4.01.006 invariant 1:
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
fn test_BC_4_01_006_ac004_notes_master_always_present() {
    // Zero-notes deck to verify the "always present regardless of deck content" invariant.
    let deck = make_deck_with_notes(&[None]);
    let laid_out = make_laid_out_deck_with_notes(&[None]);
    let pptx = export_pptx(&deck, &laid_out);

    assert_zip_entry_present(&pptx, "ppt/notesMasters/notesMaster1.xml");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-005: handoutMaster1.xml always present
// BC-4.01.006 postcondition 2
// ─────────────────────────────────────────────────────────────────────────────

/// AC-005 / BC-4.01.006 postcondition 2:
/// `ppt/handoutMasters/handoutMaster1.xml` must be present in the PPTX ZIP
/// always, regardless of deck content.
///
/// Red Gate note: same as AC-004 — this was already wired by STORY-037.
/// This test should be GREEN. Included for BC-4.01.006 traceability.
#[test]
fn test_BC_4_01_006_ac005_handout_master_always_present() {
    let deck = make_deck_with_notes(&[None]);
    let laid_out = make_laid_out_deck_with_notes(&[None]);
    let pptx = export_pptx(&deck, &laid_out);

    assert_zip_entry_present(&pptx, "ppt/handoutMasters/handoutMaster1.xml");
}
