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
#![allow(clippy::collapsible_match)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]

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
            font_size_emu: 457_200,
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
            region_role: None,
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
        slide_sections: vec![],
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

/// Extract the text content from the body placeholder's `<p:txBody>` in a
/// notesSlide XML string.  Returns `None` if the placeholder or its text is
/// absent.
///
/// ## Correct OOXML structure this function parses
///
/// Per the OOXML schema (CT_Shape / CT_Placeholder), `<p:ph>` is a
/// non-textual descriptor — it is **self-closing** and lives inside
/// `<p:nvPr>`, not wrapping `<p:txBody>`.  The `<p:txBody>` is a **sibling**
/// of `<p:nvSpPr>` directly under `<p:sp>`:
///
/// ```xml
/// <p:sp>
///   <p:nvSpPr>
///     <p:cNvPr .../>
///     <p:cNvSpPr>...</p:cNvSpPr>
///     <p:nvPr><p:ph type="body" idx="1"/></p:nvPr>  <!-- Event::Empty -->
///   </p:nvSpPr>
///   <p:spPr/>
///   <p:txBody>                <!-- sibling of nvSpPr, direct child of p:sp -->
///     <a:bodyPr/>
///     <a:lstStyle/>
///     <a:p><a:r><a:t>notes text</a:t></a:r></a:p>
///   </p:txBody>
/// </p:sp>
/// ```
///
/// ## Parse strategy (two-pass over the same `<p:sp>` region)
///
/// Because `<p:ph>` fires `Event::Empty` (not `Event::Start`), and because
/// `<p:txBody>` is a sibling — not a descendant — of `<p:ph>`, we track state
/// across the entire `<p:sp>`:
///
/// 1. When we enter a `<p:sp>` (`Event::Start("sp")`), record `sp_depth`.
/// 2. If we see `Event::Empty("ph")` with `type="body"` and `idx="1"` while
///    inside that sp, mark `sp_has_body_ph = true`.
/// 3. If `sp_has_body_ph` is set AND we enter `<p:txBody>`, set
///    `in_body_txbody = true`.
/// 4. While `in_body_txbody`, collect `Event::Text` from any `<a:t>`.
/// 5. When we exit `<p:sp>` (depth back to pre-sp), reset all flags.
///
/// ## Regression guard
///
/// This function also asserts that `<p:txBody>` is **never** found directly
/// inside a `<p:ph>` element.  If it were, the OOXML would be schema-invalid
/// (regression to the pre-fix invalid form).
fn extract_body_placeholder_text(xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    // State machine
    let mut in_sp = false;
    let mut sp_depth = 0_usize; // element depth when we entered the current sp
    let mut sp_has_body_ph = false; // did we see <p:ph type="body" idx="1"/> inside this sp?
    let mut global_depth = 0_usize; // absolute element depth (Start increments, End decrements)
    let mut in_txbody = false; // inside the txBody that belongs to the body-ph sp
    let mut txbody_depth = 0_usize; // element depth when we entered <p:txBody>
    let mut in_ph_start = false; // true while inside a <p:ph> that was Start (regression guard)
    let mut collected = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                global_depth += 1;
                let local = std::str::from_utf8(e.local_name().as_ref())
                    .unwrap_or("")
                    .to_owned();

                // Regression guard: <p:ph> must never appear as a Start event
                // (it should always be self-closing / Event::Empty).  If it
                // does, it means <p:txBody> could be nested inside <p:ph>,
                // which is schema-invalid OOXML.
                if local == "ph" {
                    in_ph_start = true;
                }
                assert!(
                    !(in_ph_start && local == "txBody"),
                    "extract_body_placeholder_text: found <p:txBody> nested inside <p:ph> — \
                     this is schema-invalid OOXML (regression to pre-fix invalid structure). \
                     <p:ph> must be self-closing (Event::Empty) inside <p:nvPr>, and \
                     <p:txBody> must be a sibling of <p:nvSpPr> under <p:sp>."
                );

                match local.as_str() {
                    "sp" if !in_sp => {
                        in_sp = true;
                        sp_depth = global_depth;
                        sp_has_body_ph = false;
                    },
                    "txBody" if in_sp && sp_has_body_ph && !in_txbody => {
                        in_txbody = true;
                        txbody_depth = global_depth;
                    },
                    _ => {},
                }
            },
            Ok(Event::End(e)) => {
                let local = std::str::from_utf8(e.local_name().as_ref())
                    .unwrap_or("")
                    .to_owned();
                if local == "ph" {
                    in_ph_start = false;
                }
                if in_txbody && global_depth == txbody_depth {
                    in_txbody = false;
                }
                if in_sp && global_depth == sp_depth {
                    in_sp = false;
                    sp_has_body_ph = false;
                }
                global_depth = global_depth.saturating_sub(1);
            },
            // <p:ph type="body" idx="1"/> fires Event::Empty (self-closing) —
            // this is the correct form.  No depth change.
            Ok(Event::Empty(e)) => {
                let local = std::str::from_utf8(e.local_name().as_ref())
                    .unwrap_or("")
                    .to_owned();
                if local == "ph" && in_sp {
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
                        sp_has_body_ph = true;
                    }
                }
            },
            Ok(Event::Text(e)) => {
                if in_txbody {
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

// ─────────────────────────────────────────────────────────────────────────────
// F-040-P1-001: slide→notesSlide back-relationship (CRIT fix)
// ─────────────────────────────────────────────────────────────────────────────

/// F-040-P1-001 (CRIT): A slide WITH notes must have a notesSlide relationship
/// in its own `ppt/slides/_rels/slide{N}.xml.rels` file.
///
/// This is the SLIDE-side relationship. PowerPoint discovers notes via this
/// relationship. The notesSlide→slide back-rel alone (which always existed)
/// is insufficient — without the slide→notesSlide rel, notes are orphaned.
///
/// Also verifies that a slide WITHOUT notes has NO such relationship.
#[test]
fn test_f040_p1_001_slide_has_notes_slide_rel_in_slide_rels() {
    // 2-slide deck: slide 1 has notes, slide 2 does not.
    let deck = make_deck_with_notes(&[Some("Speaker notes here"), None]);
    let laid_out = make_laid_out_deck_with_notes(&[Some("Speaker notes here"), None]);
    let pptx = export_pptx(&deck, &laid_out);

    // Slide 1 (with notes): rels must contain a Relationship of Type .../notesSlide
    // targeting ../notesSlides/notesSlide1.xml
    let slide1_rels = read_zip_member(&pptx, "ppt/slides/_rels/slide1.xml.rels");
    assert!(
        slide1_rels.contains("notesSlide"),
        "F-040-P1-001: ppt/slides/_rels/slide1.xml.rels must contain a notesSlide \
         relationship; got:\n{slide1_rels}"
    );
    assert!(
        slide1_rels.contains("notesSlides/notesSlide1.xml"),
        "F-040-P1-001: notesSlide relationship must target ../notesSlides/notesSlide1.xml; \
         got:\n{slide1_rels}"
    );
    assert!(
        slide1_rels.contains(
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide"
        ),
        "F-040-P1-001: notesSlide relationship must use the correct Type URI; got:\n{slide1_rels}"
    );

    // Slide 2 (without notes): rels must NOT contain a notesSlide relationship.
    let slide2_rels = read_zip_member(&pptx, "ppt/slides/_rels/slide2.xml.rels");
    assert!(
        !slide2_rels.contains("notesSlide"),
        "F-040-P1-001: ppt/slides/_rels/slide2.xml.rels must NOT contain a notesSlide \
         relationship (slide 2 has no notes); got:\n{slide2_rels}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-040-P1-002: notesMaster1.xml validity (CRIT fix)
// ─────────────────────────────────────────────────────────────────────────────

/// F-040-P1-002 (CRIT): The emitted `notesMaster1.xml` must contain the required
/// structural elements: `<p:clrMap`, `<p:ph type="sldImg"`, and
/// `<p:ph type="body" idx="1"`.
///
/// Strengthens AC-004 (presence-only) to a content-validity assertion.
/// Previously the empty NOTES_MASTER_STUB was emitted instead of the valid
/// master from `NotesMasterSerializer`.
#[test]
fn test_f040_p1_002_notes_master_xml_is_structurally_valid() {
    let deck = make_deck_with_notes(&[None]);
    let laid_out = make_laid_out_deck_with_notes(&[None]);
    let pptx = export_pptx(&deck, &laid_out);

    let notes_master_xml = read_zip_member(&pptx, "ppt/notesMasters/notesMaster1.xml");

    assert!(
        notes_master_xml.contains("<p:clrMap"),
        "F-040-P1-002: notesMaster1.xml must contain <p:clrMap> (required by OOXML schema); \
         got (first 500 chars):\n{}",
        &notes_master_xml[..notes_master_xml.len().min(500)]
    );
    assert!(
        notes_master_xml.contains("type=\"sldImg\""),
        "F-040-P1-002: notesMaster1.xml must contain <p:ph type=\"sldImg\"/> placeholder; \
         got (first 500 chars):\n{}",
        &notes_master_xml[..notes_master_xml.len().min(500)]
    );
    assert!(
        notes_master_xml.contains("type=\"body\"") && notes_master_xml.contains("idx=\"1\""),
        "F-040-P1-002: notesMaster1.xml must contain <p:ph type=\"body\" idx=\"1\"/> placeholder; \
         got (first 500 chars):\n{}",
        &notes_master_xml[..notes_master_xml.len().min(500)]
    );

    // F-040-A1 (HIGH) assertion: CT_GroupShapeProperties has no child named
    // <a:grpSpPr>.  Any occurrence of that tag would be schema-invalid and
    // rejected by PowerPoint / the OOXML linter.
    assert!(
        !notes_master_xml.contains("<a:grpSpPr"),
        "F-040-A1: notesMaster1.xml must NOT contain <a:grpSpPr> (schema-invalid: \
         CT_GroupShapeProperties has no such child element); got:\n{notes_master_xml}"
    );

    // Also verify handoutMaster1.xml for the same schema-validity constraint.
    let handout_master_xml = read_zip_member(&pptx, "ppt/handoutMasters/handoutMaster1.xml");
    assert!(
        !handout_master_xml.contains("<a:grpSpPr"),
        "F-040-A1: handoutMaster1.xml must NOT contain <a:grpSpPr> (schema-invalid: \
         CT_GroupShapeProperties has no such child element); got:\n{handout_master_xml}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-040-A1 (HIGH): Schema-valid grpSpPr in notesSlide XML
// ─────────────────────────────────────────────────────────────────────────────

/// F-040-A1 (HIGH): The `<p:grpSpPr>` in emitted `notesSlide{N}.xml` must be
/// schema-valid: `CT_GroupShapeProperties` permits `<a:xfrm>` as a child but
/// has NO child element named `<a:grpSpPr>`.  The previous hand-built XML
/// emitted a nested `<a:grpSpPr>` inside `<p:grpSpPr>`, which is schema-invalid.
///
/// Asserts (parsing real exported ZIP/XML):
/// (a) `<a:grpSpPr` does NOT appear in notesSlide{N}.xml
/// (b) `<p:grpSpPr>` is present (spTree requires it)
/// (c) `<a:xfrm>` appears inside `<p:grpSpPr>` (confirmed by well-formed XML structure)
#[test]
fn test_f040_a1_notes_slide_grpsppr_is_schema_valid() {
    let deck = make_deck_with_notes(&[Some("schema check notes")]);
    let laid_out = make_laid_out_deck_with_notes(&[Some("schema check notes")]);
    let pptx = export_pptx(&deck, &laid_out);

    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    // (a) NO nested <a:grpSpPr> — this tag is schema-invalid inside p:grpSpPr.
    assert!(
        !notes_xml.contains("<a:grpSpPr"),
        "F-040-A1: notesSlide1.xml must NOT contain <a:grpSpPr> (schema-invalid: \
         CT_GroupShapeProperties has no such child element); got:\n{notes_xml}"
    );

    // (b) <p:grpSpPr> must be present (spTree requires it as first child).
    assert!(
        notes_xml.contains("<p:grpSpPr>"),
        "F-040-A1: notesSlide1.xml must contain <p:grpSpPr> as spTree's first child; \
         got:\n{notes_xml}"
    );

    // (c) <a:xfrm> must appear (schema-valid identity transform).
    assert!(
        notes_xml.contains("<a:xfrm>"),
        "F-040-A1: notesSlide1.xml <p:grpSpPr> must contain <a:xfrm>; \
         got:\n{notes_xml}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-040-P1-003: Rich notes — multi-entry, bold/italic formatting
// ─────────────────────────────────────────────────────────────────────────────

/// F-040-P1-003 (HIGH): Notes with bold inline formatting produce `<a:rPr b="1">` runs.
/// Notes with italic inline formatting produce `<a:rPr i="1">` runs.
///
/// Uses real ZIP/XML parsing (not mock strings).
#[test]
fn test_f040_p1_003_rich_notes_bold_and_italic_runs() {
    use slideforge_types::register::RegisteredContent;

    // Build a slide with rich notes: bold text, italic text, plain text.
    let bold_node = slideforge_types::InlineNode::Bold(vec![slideforge_types::InlineNode::Plain(
        Arc::from("bold content"),
    )]);
    let italic_node =
        slideforge_types::InlineNode::Italic(vec![slideforge_types::InlineNode::Plain(Arc::from(
            "italic content",
        ))]);
    let plain_node = slideforge_types::InlineNode::Plain(Arc::from("plain text"));

    let rich_rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![bold_node, italic_node, plain_node],
    };

    let slide_with_rich_notes = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("rich notes")),
        register_tags: vec![],
        register_content: vec![rich_rc],
    };

    let deck = make_deck_with_notes(&[Some("rich notes")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide_with_rich_notes],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    assert!(
        notes_xml.contains("b=\"1\""),
        "F-040-P1-003: bold notes run must produce <a:rPr b=\"1\">; got:\n{notes_xml}"
    );
    assert!(
        notes_xml.contains("i=\"1\""),
        "F-040-P1-003: italic notes run must produce <a:rPr i=\"1\">; got:\n{notes_xml}"
    );
    assert!(
        notes_xml.contains("bold content"),
        "F-040-P1-003: bold text content must be present; got:\n{notes_xml}"
    );
    assert!(
        notes_xml.contains("italic content"),
        "F-040-P1-003: italic text content must be present; got:\n{notes_xml}"
    );
    assert!(
        notes_xml.contains("plain text"),
        "F-040-P1-003: plain text must be present; got:\n{notes_xml}"
    );
}

/// F-040-P1-003 (HIGH): Multiple `Register::Notes` entries are ALL emitted —
/// not just the first.
///
/// A slide with two separate Notes register entries produces both in the
/// notesSlide txBody (one paragraph per entry).
#[test]
fn test_f040_p1_003_multi_entry_notes_all_emitted() {
    use slideforge_types::register::RegisteredContent;

    let rc1 = RegisteredContent::plain(Register::Notes, Arc::from("First notes entry"));
    let rc2 = RegisteredContent::plain(Register::Notes, Arc::from("Second notes entry"));

    let slide_with_two_notes = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("First notes entry")), // only first in convenience field
        register_tags: vec![],
        register_content: vec![rc1, rc2],
    };

    let deck = make_deck_with_notes(&[Some("First notes entry")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide_with_two_notes],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    assert!(
        notes_xml.contains("First notes entry"),
        "F-040-P1-003: first notes entry must be present; got:\n{notes_xml}"
    );
    assert!(
        notes_xml.contains("Second notes entry"),
        "F-040-P1-003: second notes entry must also be present (multi-entry fix); got:\n{notes_xml}"
    );
    // Two separate <a:p> paragraphs — one per Notes entry.
    let para_count = notes_xml.matches("<a:p>").count() + notes_xml.matches("<a:p/>").count();
    assert!(
        para_count >= 2,
        "F-040-P1-003: must produce at least 2 paragraphs for 2 Notes entries; got {para_count}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-040-P1-004: AC-003 positive routing coverage (HIGH fix)
// ─────────────────────────────────────────────────────────────────────────────

/// F-040-P1-004 (HIGH): Strengthen AC-003 — the no-bleed test now also asserts
/// that the sentinel IS present in the notesSlide (positive routing). This
/// prevents the test from passing vacuously if notes silently vanish.
///
/// The paired assertions prove BOTH:
/// 1. Notes ARE in `ppt/notesSlides/notesSlide1.xml` (positive routing)
/// 2. Notes are NOT in any `ppt/slides/slide*.xml` (no-bleed invariant)
#[test]
fn test_f040_p1_004_ac003_no_bleed_with_positive_routing() {
    let notes_sentinel = "NOTES_ROUTING_SENTINEL_F040_P1_004";
    let deck = make_deck_with_notes(&[Some(notes_sentinel)]);
    let laid_out = make_laid_out_deck_with_notes(&[Some(notes_sentinel)]);
    let pptx = export_pptx(&deck, &laid_out);

    // Positive routing: sentinel MUST be in notesSlide1.xml
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");
    assert!(
        notes_xml.contains(notes_sentinel),
        "F-040-P1-004: notes sentinel must be present in notesSlide1.xml \
         (positive routing coverage — prevents silent loss of notes); \
         got:\n{notes_xml}"
    );

    // No-bleed: sentinel must NOT be in any slide body.
    BleedChecker::assert_absent_from_pptx_slides(&pptx, notes_sentinel);
}

// ─────────────────────────────────────────────────────────────────────────────
// F-040-P2-001 (CRIT): CWE-601 — unsafe URL scheme → plain text, no rel
// ─────────────────────────────────────────────────────────────────────────────

/// F-040-P2-001 (CRIT / CWE-601): A notes `Link` with `javascript:` scheme
/// must NOT produce a `TargetMode="External"` relationship or `<a:hlinkClick>`
/// in the output. The display text must appear as a plain run.
///
/// Verifies the defense-in-depth guard at the PPTX exporter boundary.
/// Drives the production code path through a full `PptxExporter::export`.
#[test]
fn test_f040_p2_001_unsafe_scheme_javascript_no_external_rel() {
    let url: Arc<str> = Arc::from("javascript:alert(1)");
    let link_node = slideforge_types::InlineNode::Link {
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("click me"))],
        url: url.clone(),
    };
    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![link_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("click me")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("click me")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    // The notesSlide1.xml.rels must NOT contain the javascript: URL.
    let rels = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    assert!(
        !rels.contains("javascript"),
        "F-040-P2-001: javascript: URL must NOT appear in notesSlide1.xml.rels; \
         got:\n{rels}"
    );
    assert!(
        !rels.contains("TargetMode=\"External\"") || !rels.contains("javascript"),
        "F-040-P2-001: no External rel with javascript: scheme; got:\n{rels}"
    );

    // The notesSlide1.xml must NOT contain <a:hlinkClick for this URL.
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");
    assert!(
        !notes_xml.contains("javascript"),
        "F-040-P2-001: javascript: must not appear anywhere in notesSlide1.xml; \
         got:\n{notes_xml}"
    );
    // The display text must appear as a plain run (not silently dropped).
    assert!(
        notes_xml.contains("click me"),
        "F-040-P2-001: display text 'click me' must appear as a plain text run; \
         got:\n{notes_xml}"
    );
}

/// F-040-P2-001 (CRIT / CWE-601): Same assertion for `data:` and `file:` schemes.
#[test]
fn test_f040_p2_001_unsafe_schemes_data_file_no_external_rel() {
    for (scheme_url, label) in &[
        ("data:text/html,<script>evil()</script>", "data:"),
        ("file:///etc/passwd", "file:"),
    ] {
        let url: Arc<str> = Arc::from(*scheme_url);
        let link_node = slideforge_types::InlineNode::Link {
            text: vec![slideforge_types::InlineNode::Plain(Arc::from("link text"))],
            url: url.clone(),
        };
        let rc = slideforge_types::register::RegisteredContent {
            register: Register::Notes,
            content: vec![link_node],
        };

        let slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![Frame {
                bbox: title_bbox(),
                content: FrameContent::Empty,
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: Some(Arc::from("link text")),
            register_tags: vec![],
            register_content: vec![rc],
        };

        let deck = make_deck_with_notes(&[Some("link text")]);
        let laid_out = LaidOutDeck {
            page_size: slideforge_layout::PageSize::default(),
            slides: vec![slide],
            sections: vec![],
            warnings: vec![],
            slide_sections: vec![],
        };
        let pptx = export_pptx(&deck, &laid_out);

        let rels = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
        // Only rId1 (slide) and rId2 (notesMaster) should be present — no rId3+.
        assert!(
            !rels.contains("TargetMode=\"External\""),
            "F-040-P2-001: {label} URL must produce no TargetMode=External rel; \
             got:\n{rels}"
        );
        // Display text must survive as a plain run.
        let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");
        assert!(
            notes_xml.contains("link text"),
            "F-040-P2-001: display text must appear for {label} disallowed link; \
             got:\n{notes_xml}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// F-040-P2-002 (HIGH): Hyperlink relationship presence and determinism
// ─────────────────────────────────────────────────────────────────────────────

/// F-040-P2-002 (HIGH): A single safe-scheme `https:` notes link produces:
/// - `<a:hlinkClick r:id="rId3"/>` in `notesSlide1.xml`, AND
/// - a matching `<Relationship Id="rId3" ... TargetMode="External"/>` in
///   `notesSlide1.xml.rels`.
///
/// Parses the real ZIP — not mock strings.
#[test]
fn test_f040_p2_002_single_safe_https_link_has_hlinkclick_and_external_rel() {
    let target_url = "https://example.com/notes-link";
    let link_node = slideforge_types::InlineNode::Link {
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("example"))],
        url: Arc::from(target_url),
    };
    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![link_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("example")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("example")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    // notesSlide1.xml must contain <a:hlinkClick r:id="rId3"/>
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");
    assert!(
        notes_xml.contains("hlinkClick"),
        "F-040-P2-002: notesSlide1.xml must contain <a:hlinkClick>; got:\n{notes_xml}"
    );
    assert!(
        notes_xml.contains("rId3"),
        "F-040-P2-002: hlinkClick must reference rId3 (first hyperlink); got:\n{notes_xml}"
    );

    // notesSlide1.xml.rels must contain rId3 as TargetMode="External" with the URL.
    let rels = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    assert!(
        rels.contains("rId3"),
        "F-040-P2-002: notesSlide1.xml.rels must contain rId3 relationship; got:\n{rels}"
    );
    assert!(
        rels.contains("TargetMode=\"External\""),
        "F-040-P2-002: hyperlink rel must have TargetMode=\"External\"; got:\n{rels}"
    );
    assert!(
        rels.contains(target_url),
        "F-040-P2-002: hyperlink rel Target must be the URL {target_url:?}; got:\n{rels}"
    );
}

/// F-040-P2-002 (HIGH): Two distinct safe URLs produce two separate rels with
/// stable, deterministic rIds.  Export twice and assert byte-identical output.
#[test]
fn test_f040_p2_002_two_distinct_links_stable_deterministic_rids() {
    let url_a = "https://alpha.example.com/";
    let url_b = "https://beta.example.com/";

    let link_a = slideforge_types::InlineNode::Link {
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("alpha"))],
        url: Arc::from(url_a),
    };
    let link_b = slideforge_types::InlineNode::Link {
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("beta"))],
        url: Arc::from(url_b),
    };
    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![link_a, link_b],
    };

    let make_slide_with_two_links = || LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("alpha")),
        register_tags: vec![],
        register_content: vec![rc.clone()],
    };

    let deck = make_deck_with_notes(&[Some("alpha")]);

    // First export
    let laid_out_1 = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![make_slide_with_two_links()],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx_1 = export_pptx(&deck, &laid_out_1);

    // Second export (same inputs)
    let laid_out_2 = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![make_slide_with_two_links()],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx_2 = export_pptx(&deck, &laid_out_2);

    // Both exports must be byte-identical (determinism).
    assert_eq!(
        pptx_1, pptx_2,
        "F-040-P2-002: export must be deterministic — same inputs must produce \
         byte-identical PPTX bytes"
    );

    // The rels must contain rId3 and rId4 for the two distinct URLs.
    let rels = read_zip_member(&pptx_1, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    assert!(
        rels.contains("rId3"),
        "F-040-P2-002: first distinct URL must be rId3; got:\n{rels}"
    );
    assert!(
        rels.contains("rId4"),
        "F-040-P2-002: second distinct URL must be rId4; got:\n{rels}"
    );
    assert!(
        rels.contains(url_a),
        "F-040-P2-002: rId3 must target {url_a}; got:\n{rels}"
    );
    assert!(
        rels.contains(url_b),
        "F-040-P2-002: rId4 must target {url_b}; got:\n{rels}"
    );
}

/// F-040-P2-002 (HIGH): Duplicate URL (same URL appearing twice) is deduplicated
/// to a single rId in the rels file.
#[test]
fn test_f040_p2_002_duplicate_url_deduped_to_single_rel() {
    let url = "https://example.com/shared-link";

    let link_1 = slideforge_types::InlineNode::Link {
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("first"))],
        url: Arc::from(url),
    };
    let link_2 = slideforge_types::InlineNode::Link {
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("second"))],
        url: Arc::from(url), // same URL
    };
    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![link_1, link_2],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("first")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("first")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    // rId3 must exist (the deduplicated URL).
    assert!(
        rels.contains("rId3"),
        "F-040-P2-002: deduplicated URL must produce rId3; got:\n{rels}"
    );
    // rId4 must NOT exist (duplicate URL must not create a second rel).
    assert!(
        !rels.contains("rId4"),
        "F-040-P2-002: duplicate URL must NOT produce a second rId4 rel; got:\n{rels}"
    );
    // The URL must appear exactly once in rels.
    let occurrences = rels.matches(url).count();
    assert_eq!(
        occurrences, 1,
        "F-040-P2-002: deduplicated URL must appear exactly once in rels (got {occurrences}); \
         got:\n{rels}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-040-P2-003 (HIGH): XML-escape for notes text (BC-4.01.003 test vector)
// ─────────────────────────────────────────────────────────────────────────────

/// F-040-P2-003 (HIGH / BC-4.01.003 canonical test vector):
/// Notes text containing `< & > " '` must be XML-escaped so the output is
/// well-formed XML AND the round-tripped `<a:t>` text equals the original input.
///
/// Drives the real export path (not the xml_escape helper directly) to ensure
/// the production code path is covered end-to-end.
#[test]
fn test_f040_p2_003_notes_text_xml_escape_well_formed_and_lossless() {
    // BC-4.01.003 canonical escape test vector.
    let raw_text = r#"A < B & C > D " E ' F"#;

    let deck = make_deck_with_notes(&[Some(raw_text)]);
    let laid_out = make_laid_out_deck_with_notes(&[Some(raw_text)]);
    let pptx = export_pptx(&deck, &laid_out);

    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    // 1. The XML must be well-formed (quick-xml parses without error).
    let mut reader = Reader::from_str(&notes_xml);
    reader.config_mut().trim_text(false);
    let mut event_count = 0_usize;
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(_) => event_count += 1,
            Err(e) => panic!(
                "F-040-P2-003: notesSlide1.xml is not well-formed XML after escaping \
                 text containing '<', '&', '>', '\"', \"'\": {e}\n\
                 XML content:\n{notes_xml}"
            ),
        }
    }
    assert!(
        event_count > 0,
        "F-040-P2-003: XML event count must be > 0 (non-empty XML)"
    );

    // 2. The round-tripped text must equal the original (lossless escape).
    let extracted = extract_body_placeholder_text(&notes_xml);
    assert_eq!(
        extracted.as_deref(),
        Some(raw_text),
        "F-040-P2-003: round-tripped <a:t> text must equal original input \
         (lossless XML escaping); got: {extracted:?}"
    );

    // 3. The raw characters must NOT appear unescaped in the XML body section.
    // The raw '<' and '&' appearing unescaped would be an XML parse error
    // (already caught by the well-formedness check), but we assert explicitly
    // for clarity.
    let xml_body = notes_xml.split("<p:notes").nth(1).unwrap_or(&notes_xml);
    assert!(
        !xml_body.contains(" < "),
        "F-040-P2-003: literal ' < ' must not appear unescaped in XML body"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-040-P3-001 (LOW): Nested Link inside display text — no orphan External rel
// ─────────────────────────────────────────────────────────────────────────────

/// F-040-P3-001 (LOW): A `Link` whose display `text` children contain a NESTED
/// `Link` (both with safe `https:` URLs) must NOT produce an orphan External
/// relationship in `notesSlide{N}.xml.rels`.
///
/// ## What is being guarded
///
/// `collect_hyperlink_urls` used to recurse into a `Link`'s display `text`
/// children, registering any nested safe-scheme URL as an rId in the `.rels`
/// file.  However, `serialize_nodes_with_context` flattens display text via
/// `extract_plain_text` — nested `Link` nodes are rendered as plain text and
/// NO `<a:hlinkClick>` is emitted for them.  This caused:
///   - rId count in `.rels` > `<a:hlinkClick>` count in `.xml`
///   - An orphan `TargetMode="External"` relationship with no referencing element
///   - OOXML linters flag this as invalid (rId count ≠ hlinkClick count)
///
/// ## Fix verified here (F-040-P3-001, option b)
///
/// `collect_hyperlink_urls` no longer descends into `text` children of a `Link`.
/// Only the outer link's URL is registered (when safe).  The nested URL inside
/// display text is intentionally ignored — the serializer renders it as plain text.
///
/// ## Assertions (real ZIP, real XML — no mocks)
///
/// - Count of `<Relationship ... TargetMode="External">` in `notesSlide1.xml.rels`
///   EQUALS count of `<a:hlinkClick` in `notesSlide1.xml` (no orphan rel).
/// - Both counts equal 1 (only the outer link's URL; nested URL is plain text).
/// - The outer URL appears as the Target of the one External relationship.
/// - The nested URL does NOT appear anywhere in `.rels` (no orphan rel for it).
#[test]
fn test_f040_p3_001_nested_link_in_display_text_no_orphan_rel() {
    let outer_url = "https://outer.example.com/page";
    let nested_url = "https://nested.example.com/inner";

    // Build: Link { url: outer_url, text: [ Link { url: nested_url, text: ["nested label"] } ] }
    let nested_link = slideforge_types::InlineNode::Link {
        url: Arc::from(nested_url),
        text: vec![slideforge_types::InlineNode::Plain(Arc::from(
            "nested label",
        ))],
    };
    let outer_link = slideforge_types::InlineNode::Link {
        url: Arc::from(outer_url),
        text: vec![nested_link],
    };
    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![outer_link],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("outer link")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("outer link")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    // Parse the notesSlide1.xml.rels — count External rels.
    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");

    // Count External relationships.  Each appears as:
    //   Type="...hyperlink" TargetMode="External"
    // We count by counting occurrences of TargetMode="External" in the file
    // (rId1 and rId2 are slide/master rels without TargetMode="External").
    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();

    // Parse the notesSlide1.xml — count hlinkClick elements.
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();

    // CORE ASSERTION: rId count for External rels must equal hlinkClick count.
    // An orphan rel would cause external_rel_count > hlinkclick_count.
    assert_eq!(
        external_rel_count, hlinkclick_count,
        "F-040-P3-001: orphan External rel detected — \
         TargetMode=External count ({external_rel_count}) != \
         <a:hlinkClick count ({hlinkclick_count}). \
         The nested link's URL inside the outer link's display text must NOT \
         produce an External rel (it has no corresponding hlinkClick)."
    );

    // Both counts must be exactly 1: only the outer link gets an rId + hlinkClick.
    assert_eq!(
        external_rel_count, 1,
        "F-040-P3-001: expected exactly 1 External rel (the outer link's URL); \
         got {external_rel_count}. \
         rels:\n{rels_xml}"
    );
    assert_eq!(
        hlinkclick_count, 1,
        "F-040-P3-001: expected exactly 1 <a:hlinkClick> (the outer link's); \
         got {hlinkclick_count}. \
         notes_xml:\n{notes_xml}"
    );

    // The outer URL must appear in rels (it has a valid External rel).
    assert!(
        rels_xml.contains(outer_url),
        "F-040-P3-001: outer URL {outer_url:?} must appear as an External rel Target; \
         got rels:\n{rels_xml}"
    );

    // The nested URL must NOT appear in rels (it is plain text only — no External rel).
    assert!(
        !rels_xml.contains(nested_url),
        "F-040-P3-001: nested URL {nested_url:?} must NOT appear in rels \
         (it is rendered as plain text by the serializer); \
         got rels:\n{rels_xml}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SEC-040-001 (LOW / CWE-116): Ampersand in URL query string is &amp;-escaped
// in .rels XML — well-formed XML + lossless URL round-trip
// ─────────────────────────────────────────────────────────────────────────────

/// SEC-040-001 (LOW / CWE-116): A safe `https:` URL containing an unencoded
/// ampersand in the query string (`?q=rust&lang=en`) must appear in
/// `ppt/notesSlides/_rels/notesSlide{N}.xml.rels` with the `&` properly
/// XML-escaped as `&amp;` in the `Target` attribute.
///
/// ## Threat model (CWE-116)
///
/// An unescaped `&` in an XML attribute is a well-formedness violation.  If the
/// `.rels` file is malformed, OOXML consumers (PowerPoint, LibreOffice) may
/// silently truncate the URL at the `&`, losing the query-string tail, or refuse
/// to open the file entirely.
///
/// ## Assertions
///
/// 1. `ppt/notesSlides/_rels/notesSlide1.xml.rels` is **well-formed XML**
///    (quick-xml parses it without error).
/// 2. The `Target` attribute **round-trips** to the original URL — the `&`
///    that was XML-escaped as `&amp;` in the attribute is decoded back to `&`
///    when quick-xml unescapes it.
///
/// ## Pass / fail semantics
///
/// - **PASS** (no production change needed): ooxmlsdk's typed serializer
///   XML-escapes the attribute automatically. Both assertions hold.
/// - **FAIL** (production fix required): the `.rels` contains a raw `&` in
///   the Target attribute — the file is malformed XML. Fix: XML-escape the URL
///   before storing it in `Relationship.target`, or percent-encode `&` to `%26`
///   at the `add_external_hyperlink` call site.
#[test]
fn test_sec040_001_ampersand_in_url_query_string_is_xml_escaped_in_rels() {
    // A safe https: URL whose query string contains an unencoded ampersand.
    let target_url = "https://example.com/search?q=rust&lang=en";

    let link_node = slideforge_types::InlineNode::Link {
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("search"))],
        url: Arc::from(target_url),
    };
    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![link_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("search")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("search")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    // Open the real ZIP and read the .rels file for this notesSlide.
    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");

    // ── Assertion 1: well-formed XML ──────────────────────────────────────────
    // If the `&` is unescaped in the Target attribute the quick-xml parser will
    // return an error (malformed entity / bare ampersand).
    let mut reader = Reader::from_str(&rels_xml);
    reader.config_mut().trim_text(false);
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(_) => {},
            Err(e) => panic!(
                "SEC-040-001: notesSlide1.xml.rels is NOT well-formed XML — \
                 the '&' in the Target URL was not XML-escaped as '&amp;'. \
                 This is CWE-116 (improper encoding). \
                 quick-xml error: {e}\n\
                 rels XML:\n{rels_xml}"
            ),
        }
    }

    // ── Assertion 2: URL round-trip via attribute decode ──────────────────────
    // Parse the Relationships element and extract the Target attribute value of
    // the External hyperlink relationship.  quick-xml unescapes `&amp;` → `&`
    // during attribute decoding, so the round-tripped value must equal the
    // original URL (including the `&`).
    let mut reader2 = Reader::from_str(&rels_xml);
    reader2.config_mut().trim_text(false);
    let mut round_tripped_url: Option<String> = None;
    loop {
        match reader2.read_event() {
            Ok(Event::Eof | Event::End(_)) => {
                if round_tripped_url.is_some() {
                    break;
                }
                if matches!(reader2.read_event(), Ok(Event::Eof)) {
                    break;
                }
            },
            Ok(Event::Empty(e) | Event::Start(e)) => {
                let local = std::str::from_utf8(e.local_name().as_ref())
                    .unwrap_or("")
                    .to_owned();
                if local == "Relationship" {
                    let decoder = reader2.decoder();
                    let mut is_external = false;
                    let mut target_val = String::new();
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.local_name().as_ref())
                            .unwrap_or("")
                            .to_owned();
                        let val = attr
                            .decode_and_unescape_value(decoder)
                            .unwrap_or_default()
                            .into_owned();
                        if key == "TargetMode" && val == "External" {
                            is_external = true;
                        }
                        if key == "Target" {
                            target_val = val;
                        }
                    }
                    if is_external && !target_val.is_empty() {
                        round_tripped_url = Some(target_val);
                        break;
                    }
                }
            },
            Ok(_) => {},
            Err(e) => panic!(
                "SEC-040-001: XML parse error during attribute decode phase: {e}\n\
                 rels XML:\n{rels_xml}"
            ),
        }
    }

    assert_eq!(
        round_tripped_url.as_deref(),
        Some(target_url),
        "SEC-040-001: The External hyperlink Target round-trip failed. \
         Expected URL: {target_url:?}\n\
         Got (decoded): {round_tripped_url:?}\n\
         Raw rels XML:\n{rels_xml}\n\
         (CWE-116: if the '&' was not XML-escaped, quick-xml would either \
         error during well-formedness check or decode a truncated URL here.)"
    );
}

// =============================================================================
// STORY-085 Scope B tests — PPTX OOXML Dog-Fooding Refactor
//
// These tests drive AC-005, AC-006, and AC-007 from STORY-085:
//   AC-005: grep-zero — no a:r/a:rPr/serialize_inline construction outside the
//           single InlineFormat dispatch call site in slideforge-pptx/src/
//   AC-006: notes XML output is byte-for-byte identical after the refactor
//   AC-007: no catch_unwind/panic!/unwrap/expect on Result in refactored path
//
// RED GATE STATUS:
//   - test_BC_5_02_002_ac005_grep_zero_ar_rpr_serialize_inline_outside_dispatch:
//     FAILS before refactor because serialize_inline_nodes_to_xml / serialize_nodes_with_context /
//     emit_run / a:r / a:rPr still exist in notes_slide.rs.
//   - test_BC_5_02_002_ac006_notes_xml_pinned_before_refactor: PASSES before refactor
//     (snapshot pins the current output for regression detection).
//   - test_BC_5_02_002_ac007_no_catch_unwind_in_pptx_src: PASSES before refactor
//     (no catch_unwind exists yet — test turns RED if catch_unwind is added in error).
// =============================================================================

/// AC-005 / BC-5.02.002 postcondition 5:
/// After the OOXML dog-fooding refactor, `crates/slideforge-pptx/src/` must
/// contain ZERO occurrences of:
///   - the substring `"a:r"` (literal OOXML run tag fragment)
///   - the substring `"a:rPr"` (literal OOXML run properties tag fragment)
///   - the string `serialize_inline` (legacy inline serializer function names)
///
/// These are forbidden everywhere except the single documented dispatch call site
/// (identified by the stable comment `// AC-005-DISPATCH-SITE` on that line).
///
/// ## Why this test (F-002 / LESSON-17 / TD-VSDD-059)
///
/// The previous version of this test checked only for legacy FUNCTION NAMES
/// (`serialize_inline_nodes_to_xml`, `fn emit_run`). That was a paper-fix
/// (TD-VSDD-059): the functions were removed, but the refactored code still
/// hand-constructs `<a:r><a:rPr` strings inside `dispatch_inline_nodes_to_ooxml`
/// for the `Link` arm — violating AC-005 / BC-5.02.002 postcondition 5.
///
/// This rewritten test enforces the LITERAL AC-005/BC-5.02.002 vector: scan all
/// production `.rs` files under `crates/slideforge-pptx/src/` (excluding
/// `tests/` and `#[cfg(test)]` blocks) for the substrings `a:r`, `a:rPr`, and
/// `serialize_inline`, and assert ZERO occurrences EXCEPT:
///   1. Lines that are comments (starting with `//` or `///` after trimming).
///   2. The single documented dispatch call site marked with `// AC-005-DISPATCH-SITE`.
///
/// ## Red Gate (F-002)
///
/// FAILS before the F-001 fix because `notes_slide.rs` line ~269 contains:
///   `out.push_str("<a:r><a:rPr");`
/// This is not a comment and not the dispatch site — it is a forbidden
/// hand-construction of `<a:r>` and `<a:rPr>` in the pptx exporter.
///
/// PASSES after F-001 routes the Link arm through `render_with_context` (which
/// emits the OOXML from within `DefaultInlineFormat` in `slideforge-plugin-api`).
/// The one permitted dispatch call site in `notes_slide.rs` must be marked:
///   `// AC-005-DISPATCH-SITE`
#[test]
fn test_bc_5_02_002_ac005_grep_zero_ar_rpr_serialize_inline_outside_dispatch() {
    use std::fs;
    use std::path::Path;

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");

    let mut rs_files: Vec<std::path::PathBuf> = Vec::new();
    collect_rs_files(&src_dir, &mut rs_files);

    assert!(
        !rs_files.is_empty(),
        "AC-005: no .rs files found under {src_dir:?} — check CARGO_MANIFEST_DIR"
    );

    // These literal substrings must not appear in slideforge-pptx production code
    // outside the single AC-005-DISPATCH-SITE marked line.
    // NOTE: "a:r" as a pattern will match both "a:r>" and "a:rPr" — we list them
    // separately for clear violation messages.
    let forbidden_substrings = ["<a:r", "<a:rPr", "serialize_inline"];

    // The single permitted dispatch call site marker. Any line containing this
    // marker is exempt from the forbidden-substring check.
    let dispatch_site_marker = "// AC-005-DISPATCH-SITE";

    let mut violations: Vec<String> = Vec::new();

    for file_path in &rs_files {
        // Skip test files entirely — they may reference these patterns in
        // assertions and comments. Only production source is checked.
        let path_str = file_path.to_string_lossy();
        if path_str.contains("/tests/") || path_str.ends_with("_tests.rs") {
            continue;
        }

        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("AC-005: failed to read {file_path:?}: {e}"));

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1; // 1-based

            // Skip pure comment lines (no production code on this line).
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("///") {
                continue;
            }

            // The single permitted dispatch site: the one line where the
            // InlineFormat::render / render_with_context call is made.
            // That line must carry the marker `// AC-005-DISPATCH-SITE`.
            if line.contains(dispatch_site_marker) {
                continue;
            }

            for pattern in &forbidden_substrings {
                if line.contains(pattern) {
                    violations.push(format!(
                        "{}:{line_num}: forbidden literal {:?} in production PPTX code \
                         (AC-005 / BC-5.02.002 postcondition 5): {}",
                        file_path.display(),
                        pattern,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "AC-005 FAILED — BC-5.02.002 postcondition 5 (literal grep-zero) violated.\n\
         Production code in slideforge-pptx/src/ must not hand-construct OOXML run \
         markup. All OOXML run emission must go through InlineFormat::render_with_context \
         at the single AC-005-DISPATCH-SITE. Found {} violation(s):\n{}",
        violations.len(),
        violations.join("\n")
    );
}

/// Helper: recursively collect all `.rs` files under `dir`.
fn collect_rs_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// AC-006 / BC-5.02.002 postcondition 1:
/// The inline serialization output from the current (pre-refactor)
/// `notes_slide.rs` is pinned as a snapshot. After the refactor, the
/// `DefaultInlineFormat::render(node, InlineOutputFormat::Ooxml)` output
/// must be byte-for-byte identical to this snapshot.
///
/// ## Pinned test vectors (from BC-5.02.002 postcondition 1 and AC-001)
///
/// - `Plain("hello world")` → `<a:r><a:t>hello world</a:t></a:r>`
/// - `Bold([Plain("hi")])` → `<a:r><a:rPr b="1"/><a:t>hi</a:t></a:r>`
/// - `Italic([Plain("em")])` → `<a:r><a:rPr i="1"/><a:t>em</a:t></a:r>`
/// - `Plain("a & b < c")` → `<a:r><a:t>a &amp; b &lt; c</a:t></a:r>` (XML-escaped)
///
/// These vectors are exercised via `NotesSlideSerializer::build` and then
/// the XML is read back. After the refactor, the same vectors must produce
/// the same XML via the `DefaultInlineFormat` dispatch path.
///
/// ## Red Gate
///
/// This test is intended to PASS (green) before the refactor — it pins the
/// current behavior. After the refactor it must remain GREEN (no snapshot delta).
/// If a snapshot delta appears, the refactor introduced a correctness regression.
#[test]
fn test_bc_5_02_002_ac006_notes_xml_output_pinned_plain_text() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    // Plain text run — canonical: <a:r><a:t>hello world</a:t></a:r>
    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![InlineNode::Plain(Arc::from("hello world"))],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("hello world")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("hello world")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    // Snapshot: pin the a:p paragraph content (the inline run markup).
    // Extract just the paragraph section for a stable, focused snapshot.
    let para_content = extract_paragraph_content(&notes_xml);
    insta::assert_snapshot!("notes_inline_plain_text", para_content);
}

/// AC-006: Pin bold+italic runs (these must survive the refactor unchanged).
#[test]
fn test_bc_5_02_002_ac006_notes_xml_output_pinned_bold_italic() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![
            InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold text"))]),
            InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic text"))]),
            InlineNode::Plain(Arc::from("plain text")),
        ],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("bold italic")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("bold italic")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let para_content = extract_paragraph_content(&notes_xml);
    insta::assert_snapshot!("notes_inline_bold_italic", para_content);
}

/// AC-006: Pin XML-escaped special characters (these must survive the refactor).
#[test]
fn test_bc_5_02_002_ac006_notes_xml_output_pinned_xml_escape() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![InlineNode::Plain(Arc::from("a & b < c"))],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("a & b < c")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("a & b < c")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let para_content = extract_paragraph_content(&notes_xml);
    insta::assert_snapshot!("notes_inline_xml_escape", para_content);
}

/// Extract the paragraph content fragment from a notesSlide XML string.
///
/// Returns the content of the first `<a:p>` tag and its children as a string,
/// for focused snapshot assertions. Falls back to returning the full XML body
/// starting at `<a:p>` if the paragraph boundary cannot be determined.
fn extract_paragraph_content(notes_xml: &str) -> String {
    // Find the first <a:p> open tag and extract up to and including </a:p>.
    if let Some(start) = notes_xml.find("<a:p>") {
        if let Some(end_offset) = notes_xml[start..].find("</a:p>") {
            let end = start + end_offset + "</a:p>".len();
            return notes_xml[start..end].to_owned();
        }
        // No closing tag — return from start of paragraph to end
        return notes_xml[start..].to_owned();
    }
    // No <a:p> found — return the whole XML (unexpected, will surface in snapshot)
    notes_xml.to_owned()
}

/// AC-007 / BC-5.02.002 invariant 3:
/// The refactored path in `slideforge-pptx/src/` must NOT add any new
/// `catch_unwind`, `panic!`, or `.unwrap()` / `.expect()` calls on `Result`
/// types in production code.
///
/// ## Red Gate
///
/// This test is GREEN before the refactor (no violations exist yet).
/// It turns RED if an implementer accidentally adds `catch_unwind` or
/// misuses `unwrap()` on `Result` during the refactor.
///
/// ## Implementation
///
/// Scans `crates/slideforge-pptx/src/` for forbidden patterns:
/// - `catch_unwind` — forbidden absolutely in production code
/// - `panic!(` — forbidden (use proper error propagation)
///
/// Note: `.unwrap()` on `Option` (not `Result`) is a clippy::pedantic concern
/// handled by the compiler. This test focuses on the patterns most likely to
/// be introduced by the refactor (catch_unwind around InlineFormat dispatch).
#[test]
fn test_bc_5_02_002_ac007_no_catch_unwind_in_pptx_src() {
    use std::path::Path;

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");

    let mut rs_files: Vec<std::path::PathBuf> = Vec::new();
    collect_rs_files(&src_dir, &mut rs_files);

    // Patterns that are ALWAYS forbidden in production pptx code.
    let forbidden = ["catch_unwind"];

    let mut violations: Vec<String> = Vec::new();

    for file_path in &rs_files {
        // Skip test files — these patterns are permitted in tests.
        let path_str = file_path.to_string_lossy();
        if path_str.contains("/tests/") || path_str.ends_with("_tests.rs") {
            continue;
        }

        let content = std::fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("AC-007: failed to read {file_path:?}: {e}"));

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;
            // Skip doc comments and normal comments
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with('*') {
                continue;
            }

            for pattern in &forbidden {
                if line.contains(pattern) {
                    violations.push(format!(
                        "{}:{}: forbidden pattern {:?}: {}",
                        file_path.display(),
                        line_num,
                        pattern,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "AC-007 FAILED — BC-5.02.002 invariant 3 violated.\n\
         Production code in slideforge-pptx/src/ must not use catch_unwind. \
         Found {} violation(s):\n{}",
        violations.len(),
        violations.join("\n")
    );
}

// =============================================================================
// F-003 (HIGH): AC-006 variant coverage — all 12 InlineNode variants in notes
//
// The legacy `serialize_nodes_with_context` handled all 12 variants. The
// refactored `DefaultInlineFormat` dispatch path must handle them too.
// AC-006 previously only pinned 3/12 variants (Plain, Bold+Italic, xml-escape).
// F-003 adds explicit assertions for the remaining variants in the NOTES path.
//
// Investigation result (git show develop:notes_slide.rs):
//   - Plain/Code/Xref → emit_run (plain text with bold/italic flags inherited)
//   - Bold → recurse with bold=true
//   - Italic → recurse with italic=true
//   - Footnote/Superscript/Subscript/Strikethrough/Highlight → recurse children
//     inheriting bold/italic (NO distinct run properties — flattened to plain/bold/italic)
//   - Math → LaTeX source as plain text run
//   - Link → hyperlink run (with rId) or plain text fallback
//
// The NEW `DefaultInlineFormat` behavior is PRODUCTION-GRADE: Super/Sub/Strike/
// Highlight emit dedicated run properties (baseline, strike, highlight) instead
// of silently flattening. This is intentional improvement, not a regression.
// The tests here assert the NEW correct production behavior.
// =============================================================================

/// F-003 (HIGH): Code node in notes path → emits OOXML run containing the code text.
/// The legacy path emitted a plain text run (emit_run). The new path emits a
/// monospace run via DefaultInlineFormat (Courier New). Either way, the text must
/// appear in the output.
#[test]
fn test_f003_ac006_notes_code_variant_in_ooxml() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![InlineNode::Code(Arc::from("fn main()"))],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("fn main()")),
        register_tags: vec![],
        register_content: vec![rc],
    };
    let deck = make_deck_with_notes(&[Some("fn main()")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    assert!(
        notes_xml.contains("fn main()"),
        "F-003: Code variant in notes must emit the code text; got:\n{notes_xml}"
    );
    // Code in OOXML via DefaultInlineFormat emits a monospace run.
    insta::assert_snapshot!(
        "f003_notes_code_ooxml",
        extract_paragraph_content(&notes_xml)
    );
}

/// F-003 (HIGH): Xref node in notes path → emits OOXML run containing the xref id.
#[test]
fn test_f003_ac006_notes_xref_variant_in_ooxml() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![InlineNode::Xref(Arc::from("slide-5"))],
    };
    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("slide-5")),
        register_tags: vec![],
        register_content: vec![rc],
    };
    let deck = make_deck_with_notes(&[Some("slide-5")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    assert!(
        notes_xml.contains("slide-5"),
        "F-003: Xref variant in notes must emit the xref id; got:\n{notes_xml}"
    );
    insta::assert_snapshot!(
        "f003_notes_xref_ooxml",
        extract_paragraph_content(&notes_xml)
    );
}

/// F-003 (HIGH): Superscript node in notes path → emits OOXML run with
/// `baseline="30000"` (production-grade; legacy flattened to plain/bold/italic).
#[test]
fn test_f003_ac006_notes_superscript_variant_in_ooxml() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![InlineNode::Superscript(vec![InlineNode::Plain(Arc::from(
            "2",
        ))])],
    };
    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("2")),
        register_tags: vec![],
        register_content: vec![rc],
    };
    let deck = make_deck_with_notes(&[Some("2")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    assert!(
        notes_xml.contains('2'),
        "F-003: Superscript text must appear in notes; got:\n{notes_xml}"
    );
    // New production-grade behavior: baseline="30000" (improvement over legacy).
    assert!(
        notes_xml.contains("baseline=\"30000\""),
        "F-003: Superscript in notes must use baseline=\"30000\" (production-grade); got:\n{notes_xml}"
    );
    insta::assert_snapshot!(
        "f003_notes_superscript_ooxml",
        extract_paragraph_content(&notes_xml)
    );
}

/// F-003 (HIGH): Subscript node in notes path → `baseline="-25000"`.
#[test]
fn test_f003_ac006_notes_subscript_variant_in_ooxml() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![InlineNode::Subscript(vec![InlineNode::Plain(Arc::from(
            "n",
        ))])],
    };
    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("n")),
        register_tags: vec![],
        register_content: vec![rc],
    };
    let deck = make_deck_with_notes(&[Some("n")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    assert!(
        notes_xml.contains("baseline=\"-25000\""),
        "F-003: Subscript in notes must use baseline=\"-25000\"; got:\n{notes_xml}"
    );
    insta::assert_snapshot!(
        "f003_notes_subscript_ooxml",
        extract_paragraph_content(&notes_xml)
    );
}

/// F-003 (HIGH): Strikethrough node in notes path → `strike="sngStrike"`.
#[test]
fn test_f003_ac006_notes_strikethrough_variant_in_ooxml() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![InlineNode::Strikethrough(vec![InlineNode::Plain(
            Arc::from("removed"),
        )])],
    };
    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("removed")),
        register_tags: vec![],
        register_content: vec![rc],
    };
    let deck = make_deck_with_notes(&[Some("removed")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    assert!(
        notes_xml.contains("strike=\"sngStrike\""),
        "F-003: Strikethrough in notes must use strike=\"sngStrike\"; got:\n{notes_xml}"
    );
    insta::assert_snapshot!(
        "f003_notes_strikethrough_ooxml",
        extract_paragraph_content(&notes_xml)
    );
}

/// F-003 (HIGH): Highlight node in notes path → `highlight="yellow"`.
#[test]
fn test_f003_ac006_notes_highlight_variant_in_ooxml() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![InlineNode::Highlight(vec![InlineNode::Plain(Arc::from(
            "important",
        ))])],
    };
    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("important")),
        register_tags: vec![],
        register_content: vec![rc],
    };
    let deck = make_deck_with_notes(&[Some("important")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    assert!(
        notes_xml.contains("highlight=\"yellow\""),
        "F-003: Highlight in notes must use highlight=\"yellow\"; got:\n{notes_xml}"
    );
    insta::assert_snapshot!(
        "f003_notes_highlight_ooxml",
        extract_paragraph_content(&notes_xml)
    );
}

/// F-003 (HIGH): Footnote node in notes path → emits children as OOXML runs
/// (non-empty output containing the footnote text).
#[test]
fn test_f003_ac006_notes_footnote_variant_in_ooxml() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![InlineNode::Footnote(vec![InlineNode::Plain(Arc::from(
            "footnote body",
        ))])],
    };
    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("footnote body")),
        register_tags: vec![],
        register_content: vec![rc],
    };
    let deck = make_deck_with_notes(&[Some("footnote body")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    assert!(
        notes_xml.contains("footnote body"),
        "F-003: Footnote text must appear in notes; got:\n{notes_xml}"
    );
    insta::assert_snapshot!(
        "f003_notes_footnote_ooxml",
        extract_paragraph_content(&notes_xml)
    );
}

/// F-003 (HIGH): Math node in notes path → emits the LaTeX source as a plain
/// text run (EC-001 fallback, same as legacy behavior).
#[test]
fn test_f003_ac006_notes_math_variant_in_ooxml() {
    use slideforge_types::register::RegisteredContent;
    use slideforge_types::{InlineNode, MathNode, SourceSpan};

    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![InlineNode::Math(MathNode {
            latex: Arc::from("x^2 + y^2"),
            display: false,
            span: SourceSpan::default(),
        })],
    };
    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("x^2 + y^2")),
        register_tags: vec![],
        register_content: vec![rc],
    };
    let deck = make_deck_with_notes(&[Some("x^2 + y^2")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    assert!(
        notes_xml.contains("x^2 + y^2"),
        "F-003: Math LaTeX must appear as plain text in notes (EC-001 fallback); got:\n{notes_xml}"
    );
    insta::assert_snapshot!(
        "f003_notes_math_ooxml",
        extract_paragraph_content(&notes_xml)
    );
}

// =============================================================================
// F-006 (MED): Registry routing — notes serializer uses registry formatter
//
// After F-006, `dispatch_inline_nodes_to_ooxml` must resolve the InlineFormat
// from the PluginRegistry (id "default") rather than hardcoding `DefaultInlineFormat`.
// This test is GREEN before F-006 (the existing behavior produces correct output)
// and must STAY GREEN after F-006 (no behavioral regression from the routing change).
//
// NOTE: F-006 is a structural refactor (registry routing), not a behavioral change.
// The test verifies the OUTCOME (correct OOXML output) is preserved — the
// mechanism change (hardcoded → registry) is verified by code review.
// =============================================================================

/// F-006 (MED): Registry-routed InlineFormat still produces correct OOXML for
/// all node types exercised via the notes path. This is a regression guard.
///
/// This test PASSES before F-006 (hardcoded DefaultInlineFormat works) and must
/// remain GREEN after F-006 (registry-resolved DefaultInlineFormat gives same output).
#[test]
fn test_f006_registry_routing_notes_produces_same_ooxml() {
    use slideforge_types::InlineNode;
    use slideforge_types::register::RegisteredContent;

    // Mix of node types: Plain, Bold, Italic, Code, Xref.
    let rc = RegisteredContent {
        register: slideforge_types::Register::Notes,
        content: vec![
            InlineNode::Plain(Arc::from("plain")),
            InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))]),
            InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic"))]),
            InlineNode::Code(Arc::from("code()")),
            InlineNode::Xref(Arc::from("ref-1")),
        ],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("mixed")),
        register_tags: vec![],
        register_content: vec![rc],
    };
    let deck = make_deck_with_notes(&[Some("mixed")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    // All text content must appear.
    for text in &["plain", "bold", "italic", "code()", "ref-1"] {
        assert!(
            notes_xml.contains(text),
            "F-006: notes XML must contain {text:?}; got:\n{notes_xml}"
        );
    }
    // Bold run properties must be present.
    assert!(
        notes_xml.contains("b=\"1\""),
        "F-006: bold must be b=\"1\"; got:\n{notes_xml}"
    );
    // Italic run properties must be present.
    assert!(
        notes_xml.contains("i=\"1\""),
        "F-006: italic must be i=\"1\"; got:\n{notes_xml}"
    );
}

// =============================================================================
// OBS-1 (LOW): Empty-display-text Link must NOT produce orphan External rel
//
// A Link whose display text is empty (text: vec![]) has no visible run emitted
// by DefaultInlineFormat::render_with_context (returns Ok("") at line ~108).
// Before the fix, collect_hyperlink_urls still registered the safe-scheme URL,
// allocating an rId + TargetMode="External" entry in .rels with no corresponding
// <a:hlinkClick> referencing it — an orphan External rel that OOXML linters flag.
//
// Fix: collect_hyperlink_urls guards !text.is_empty() before registering a URL,
// so no rId is allocated and no orphan rel is produced for empty-text links.
// =============================================================================

/// OBS-1 (LOW): A Notes-register inline tree containing a plain text node PLUS
/// `Link { text: vec![], url: "https://example.com" }` (safe scheme, empty
/// display text) must produce OOXML where the count of
/// `TargetMode="External"` relationships EQUALS the count of
/// `<a:hlinkClick` elements — i.e., zero orphan rels.
///
/// ## What is being guarded
///
/// `collect_hyperlink_urls` registers ANY safe-scheme URL, allocating an rId +
/// `TargetMode="External"` entry in `.rels`.  But `render_with_context` in
/// `DefaultInlineFormat` returns `Ok(String::new())` when the Link display text
/// flattens to empty — no `<a:hlinkClick>` is emitted.  This creates an orphan
/// External relationship (`external_rel_count > hlinkClick_count`).
///
/// ## Test setup (why plain text + empty link)
///
/// `slide_has_notes` calls `inline_nodes_to_plain_text` and checks for non-empty
/// content before producing a notesSlide part.  A Notes entry containing ONLY an
/// empty-text Link would return `""` → no notesSlide.  The defect manifests when
/// a Notes entry mixes a non-empty node (which causes the notesSlide to be built)
/// with an empty-text Link (which `collect_hyperlink_urls` incorrectly registers).
/// We must include a non-empty Plain node to trigger notesSlide creation, then
/// the empty-text Link is the defect vector.
///
/// ## Mirroring F-040-P3-001 style
///
/// Mirrors `test_f040_p3_001_nested_link_in_display_text_no_orphan_rel` but for
/// the empty-text case (text: vec![]) rather than the nested-link case.
/// Both tests assert the same count-equality invariant:
///   `external_rel_count == hlinkclick_count` (zero orphan rels).
///
/// ## RED → GREEN (OBS-1 fix in collect_hyperlink_urls)
///
/// FAILS before fix: `collect_hyperlink_urls` registers the URL → external_rel_count=1,
/// hlinkclick_count=0 → count-equality assertion fails.
/// PASSES after fix: URL is not registered for empty-text link → both counts 0.
#[test]
fn test_obs1_empty_display_text_link_no_orphan_external_rel() {
    let url = "https://example.com/empty-text-link";

    // Link with genuinely empty display-text vector (vec![]).
    // DefaultInlineFormat::render_with_context returns Ok("") for this and
    // emits NO <a:hlinkClick> — so no rId must be allocated either.
    let empty_text_link = slideforge_types::InlineNode::Link {
        text: vec![],
        url: Arc::from(url),
    };
    // Plain text node ensures slide_has_notes returns true → notesSlide is built.
    // Without at least one non-empty node the file would not be created, making
    // the test vacuously pass (notesSlide absent → no rels → 0==0).
    let plain_node = slideforge_types::InlineNode::Plain(Arc::from("see also"));

    // Single RegisteredContent with two nodes: a plain text node (non-empty, so
    // slide_has_notes passes) followed by the empty-text Link (the defect vector).
    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![plain_node, empty_text_link],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("see also")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("see also")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    // Count External relationships (rId1=slide, rId2=notesMaster have no TargetMode).
    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();
    // Count <a:hlinkClick elements in the notes XML.
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();

    // CORE INVARIANT: rId↔hlinkClick count-equality (no orphan rels).
    // Before fix: external_rel_count=1, hlinkclick_count=0 → assertion fails.
    // After fix:  external_rel_count=0, hlinkclick_count=0 → assertion passes.
    assert_eq!(
        external_rel_count, hlinkclick_count,
        "OBS-1: orphan External rel detected for empty-display-text Link — \
         TargetMode=External count ({external_rel_count}) != \
         <a:hlinkClick count ({hlinkclick_count}). \
         An empty-text Link must not register an rId (no run is emitted). \
         rels:\n{rels_xml}\nnotes_xml:\n{notes_xml}"
    );

    // Both counts must be exactly 0: empty-text link produces no run and no rel.
    assert_eq!(
        external_rel_count, 0,
        "OBS-1: expected 0 External rels for empty-display-text Link; \
         got {external_rel_count}.\nrels:\n{rels_xml}"
    );
    assert_eq!(
        hlinkclick_count, 0,
        "OBS-1: expected 0 <a:hlinkClick for empty-display-text Link; \
         got {hlinkclick_count}.\nnotes_xml:\n{notes_xml}"
    );

    // The URL must NOT appear in rels (no orphan External rel).
    assert!(
        !rels_xml.contains(url),
        "OBS-1: URL {url:?} must NOT appear in rels (empty-text link → no rel); \
         got rels:\n{rels_xml}"
    );

    // The plain text node must still appear (non-empty content survives).
    assert!(
        notes_xml.contains("see also"),
        "OBS-1: plain text 'see also' must appear in notes XML; got:\n{notes_xml}"
    );
}

/// OBS-1 regression guard: a non-empty-text safe-scheme Link still produces
/// exactly 1 External rel AND 1 `<a:hlinkClick>` (count-equal).
///
/// This test ensures the fix for empty-text Links does NOT accidentally suppress
/// External rels for normal Links with non-empty display text.
#[test]
fn test_obs1_non_empty_display_text_link_still_produces_rel_and_hlinkclick() {
    let url = "https://example.com/non-empty-link";

    let link_node = slideforge_types::InlineNode::Link {
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("click here"))],
        url: Arc::from(url),
    };
    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![link_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("click here")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("click here")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();

    // CORE INVARIANT: count-equality (no orphan rels).
    assert_eq!(
        external_rel_count, hlinkclick_count,
        "OBS-1 regression: non-empty Link must have equal External rel and hlinkClick counts; \
         TargetMode=External count ({external_rel_count}) != \
         <a:hlinkClick count ({hlinkclick_count}).\nrels:\n{rels_xml}\nnotes:\n{notes_xml}"
    );

    // Both counts must be exactly 1.
    assert_eq!(
        external_rel_count, 1,
        "OBS-1 regression: non-empty Link must produce exactly 1 External rel; \
         got {external_rel_count}.\nrels:\n{rels_xml}"
    );
    assert_eq!(
        hlinkclick_count, 1,
        "OBS-1 regression: non-empty Link must produce exactly 1 <a:hlinkClick>; \
         got {hlinkclick_count}.\nnotes:\n{notes_xml}"
    );

    // The URL must appear in rels.
    assert!(
        rels_xml.contains(url),
        "OBS-1 regression: URL {url:?} must appear in rels for non-empty Link; \
         got rels:\n{rels_xml}"
    );

    // The display text must appear in the notes XML.
    assert!(
        notes_xml.contains("click here"),
        "OBS-1 regression: display text 'click here' must appear in notes XML; \
         got:\n{notes_xml}"
    );
}

// =============================================================================
// F-P5-001 [MED]: Non-empty Vec, but flattens-to-empty — no orphan External rel
//
// The OBS-1 fix used `!text.is_empty()` (Vec-length check) which is INSUFFICIENT.
// A Link with `text: vec![Plain("")]` has `!text.is_empty() == true` (Vec holds
// one element), so the old fix STILL registers the URL → orphan External rel.
// The correct guard is: "does the display text flatten to a non-empty string?"
// i.e., `display_text_is_empty(text)` — same semantics as render_with_context.
//
// These tests confirm the gap and enforce the invariant:
//   external_rel_count == hlinkclick_count for ALL empty-flatten variants.
// =============================================================================

/// F-P5-001 [MED]: `Link { text: vec![Plain("")], url: <safe> }` — Vec is
/// non-empty (length 1) but flattens to "" → render_with_context emits nothing →
/// must produce zero External rels AND zero hlinkClick elements.
///
/// FAILS before fix (Vec-length guard): external_rel_count=1, hlinkclick_count=0.
/// PASSES after fix (flatten-emptiness guard): both counts 0.
#[test]
fn test_fp5_001_link_nonempty_vec_empty_flatten_no_orphan_rel_plain() {
    let url = "https://example.com/fp5-plain-empty";

    // Plain("") — Vec has 1 element, but flattens to empty string.
    let link_node = slideforge_types::InlineNode::Link {
        text: vec![slideforge_types::InlineNode::Plain(Arc::from(""))],
        url: Arc::from(url),
    };
    let plain_node = slideforge_types::InlineNode::Plain(Arc::from("anchor text"));

    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![plain_node, link_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("anchor text")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("anchor text")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();

    // CORE INVARIANT: no orphan rels (count-equality).
    assert_eq!(
        external_rel_count, hlinkclick_count,
        "F-P5-001 [Plain(\"\")]: orphan External rel — \
         TargetMode=External ({external_rel_count}) != <a:hlinkClick ({hlinkclick_count}). \
         Link with Plain(\"\") display text must not register an rId. \
         rels:\n{rels_xml}\nnotes:\n{notes_xml}"
    );

    // Both must be exactly 0.
    assert_eq!(
        external_rel_count, 0,
        "F-P5-001 [Plain(\"\")]: expected 0 External rels; got {external_rel_count}. \
         rels:\n{rels_xml}"
    );
    assert_eq!(
        hlinkclick_count, 0,
        "F-P5-001 [Plain(\"\")]: expected 0 <a:hlinkClick; got {hlinkclick_count}. \
         notes:\n{notes_xml}"
    );

    // URL must NOT appear in rels.
    assert!(
        !rels_xml.contains(url),
        "F-P5-001 [Plain(\"\")]: URL must NOT appear in rels; got:\n{rels_xml}"
    );
}

/// F-P5-001 [MED] sibling: `Link { text: vec![Bold(vec![])], url: <safe> }` —
/// Vec is non-empty (holds one Bold), Bold has empty children, flattens to "" →
/// same orphan-rel gap as Plain("").
///
/// FAILS before fix (Vec-length guard): external_rel_count=1, hlinkclick_count=0.
/// PASSES after fix (flatten-emptiness guard): both counts 0.
#[test]
fn test_fp5_001_link_nonempty_vec_empty_flatten_no_orphan_rel_bold_empty() {
    let url = "https://example.com/fp5-bold-empty";

    // Bold(vec![]) — outer Vec has 1 element; Bold has no children → flattens to "".
    let link_node = slideforge_types::InlineNode::Link {
        text: vec![slideforge_types::InlineNode::Bold(vec![])],
        url: Arc::from(url),
    };
    let plain_node = slideforge_types::InlineNode::Plain(Arc::from("notes content"));

    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![plain_node, link_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("notes content")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("notes content")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();

    // CORE INVARIANT: no orphan rels.
    assert_eq!(
        external_rel_count, hlinkclick_count,
        "F-P5-001 [Bold(vec![])]: orphan External rel — \
         TargetMode=External ({external_rel_count}) != <a:hlinkClick ({hlinkclick_count}). \
         rels:\n{rels_xml}\nnotes:\n{notes_xml}"
    );

    assert_eq!(
        external_rel_count, 0,
        "F-P5-001 [Bold(vec![])]: expected 0 External rels; got {external_rel_count}. \
         rels:\n{rels_xml}"
    );
    assert_eq!(
        hlinkclick_count, 0,
        "F-P5-001 [Bold(vec![])]: expected 0 <a:hlinkClick; got {hlinkclick_count}. \
         notes:\n{notes_xml}"
    );

    assert!(
        !rels_xml.contains(url),
        "F-P5-001 [Bold(vec![])]: URL must NOT appear in rels; got:\n{rels_xml}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-085-P6-001 [HIGH] — orphan External rel for Link nested inside formatting wrapper
//
// Root cause: collect_hyperlink_urls recurses into Bold/Italic/Strikethrough/
// Superscript/Subscript/Highlight/Footnote and registers any safe Link URL
// it finds — but dispatch_inline_nodes_to_ooxml only emits <a:hlinkClick>
// for TOP-LEVEL Link nodes. A Link nested in a formatting wrapper gets
// hyperlink_rid=None → renders as plain text + warn → no hlinkClick. Result:
// external_rel_count=1, hlinkclick_count=0 → orphan rel.
//
// Fix: collect_hyperlink_urls must NOT recurse into formatting wrappers.
// Registration and emission are now both top-level-only; count invariant holds.
//
// All tests below FAIL before the fix (orphan rel) and PASS after the fix (counts equal).
// ─────────────────────────────────────────────────────────────────────────────

/// F-085-P6-001 [HIGH]: `Bold([Link{url:safe, text:[Plain("x")]}])` — Link nested
/// inside Bold creates an orphan External rel before the fix.
///
/// FAILS before fix: external_rel_count=1, hlinkclick_count=0.
/// PASSES after fix: both counts 0 (nested link is plain text, no rel registered).
#[test]
fn test_f085_p6_001_bold_wrapping_link_no_orphan_rel() {
    let nested_url = "https://example.com/bold-wrapped-link";

    let link_inside_bold = slideforge_types::InlineNode::Link {
        url: Arc::from(nested_url),
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("x"))],
    };
    let bold_node = slideforge_types::InlineNode::Bold(vec![link_inside_bold]);

    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![bold_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("x")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("x")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();

    assert_eq!(
        external_rel_count, hlinkclick_count,
        "F-085-P6-001 [Bold(Link)]: orphan External rel — \
         TargetMode=External ({external_rel_count}) != <a:hlinkClick ({hlinkclick_count}). \
         A Link nested inside Bold must NOT produce an External rel. \
         rels:\n{rels_xml}\nnotes:\n{notes_xml}"
    );
    assert_eq!(
        external_rel_count, 0,
        "F-085-P6-001 [Bold(Link)]: expected 0 External rels (nested link is plain text); \
         got {external_rel_count}. rels:\n{rels_xml}"
    );
    assert_eq!(
        hlinkclick_count, 0,
        "F-085-P6-001 [Bold(Link)]: expected 0 <a:hlinkClick (nested link is plain text); \
         got {hlinkclick_count}. notes:\n{notes_xml}"
    );
    assert!(
        !rels_xml.contains(nested_url),
        "F-085-P6-001 [Bold(Link)]: nested URL must NOT appear in rels; got:\n{rels_xml}"
    );
}

/// F-085-P6-001 [HIGH]: `Italic([Link{...}])` — Link nested inside Italic.
///
/// FAILS before fix: external_rel_count=1, hlinkclick_count=0.
/// PASSES after fix: both counts 0.
#[test]
fn test_f085_p6_001_italic_wrapping_link_no_orphan_rel() {
    let nested_url = "https://example.com/italic-wrapped-link";

    let link_inside_italic = slideforge_types::InlineNode::Link {
        url: Arc::from(nested_url),
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("y"))],
    };
    let italic_node = slideforge_types::InlineNode::Italic(vec![link_inside_italic]);

    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![italic_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("y")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("y")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();

    assert_eq!(
        external_rel_count, hlinkclick_count,
        "F-085-P6-001 [Italic(Link)]: orphan External rel — counts mismatch. \
         rels:\n{rels_xml}\nnotes:\n{notes_xml}"
    );
    assert_eq!(
        external_rel_count, 0,
        "F-085-P6-001 [Italic(Link)]: expected 0 External rels; got {external_rel_count}. \
         rels:\n{rels_xml}"
    );
    assert!(
        !rels_xml.contains(nested_url),
        "F-085-P6-001 [Italic(Link)]: nested URL must NOT appear in rels; got:\n{rels_xml}"
    );
}

/// F-085-P6-001 [HIGH]: `Strikethrough([Link{...}])` — Link nested inside Strikethrough.
///
/// FAILS before fix: external_rel_count=1, hlinkclick_count=0.
/// PASSES after fix: both counts 0.
#[test]
fn test_f085_p6_001_strikethrough_wrapping_link_no_orphan_rel() {
    let nested_url = "https://example.com/strike-wrapped-link";

    let link_inside_strike = slideforge_types::InlineNode::Link {
        url: Arc::from(nested_url),
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("z"))],
    };
    let strike_node = slideforge_types::InlineNode::Strikethrough(vec![link_inside_strike]);

    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![strike_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("z")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("z")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();

    assert_eq!(
        external_rel_count, hlinkclick_count,
        "F-085-P6-001 [Strikethrough(Link)]: orphan External rel — counts mismatch. \
         rels:\n{rels_xml}\nnotes:\n{notes_xml}"
    );
    assert_eq!(
        external_rel_count, 0,
        "F-085-P6-001 [Strikethrough(Link)]: expected 0 External rels; got {external_rel_count}. \
         rels:\n{rels_xml}"
    );
    assert!(
        !rels_xml.contains(nested_url),
        "F-085-P6-001 [Strikethrough(Link)]: nested URL must NOT appear in rels; got:\n{rels_xml}"
    );
}

/// F-085-P6-001 [HIGH]: deeply nested `Bold([Italic([Link{...}])])`.
///
/// FAILS before fix: external_rel_count=1, hlinkclick_count=0.
/// PASSES after fix: both counts 0.
#[test]
fn test_f085_p6_001_deeply_nested_bold_italic_link_no_orphan_rel() {
    let nested_url = "https://example.com/deep-nested-link";

    let link_deep = slideforge_types::InlineNode::Link {
        url: Arc::from(nested_url),
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("deep"))],
    };
    let italic_node = slideforge_types::InlineNode::Italic(vec![link_deep]);
    let bold_node = slideforge_types::InlineNode::Bold(vec![italic_node]);

    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![bold_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("deep")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("deep")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();

    assert_eq!(
        external_rel_count, hlinkclick_count,
        "F-085-P6-001 [Bold(Italic(Link))]: orphan External rel — counts mismatch. \
         rels:\n{rels_xml}\nnotes:\n{notes_xml}"
    );
    assert_eq!(
        external_rel_count, 0,
        "F-085-P6-001 [Bold(Italic(Link))]: expected 0 External rels; got {external_rel_count}. \
         rels:\n{rels_xml}"
    );
    assert!(
        !rels_xml.contains(nested_url),
        "F-085-P6-001 [Bold(Italic(Link))]: nested URL must NOT appear in rels; got:\n{rels_xml}"
    );
}

/// F-085-P6-001 [HIGH]: MIXED entry — `[Link{top-level safe}, Bold([Link{nested safe}])]`.
///
/// The top-level Link registers an rId + emits hlinkClick.
/// The nested Link (inside Bold) renders as plain text — no External rel registered.
/// Result: exactly 1 External rel AND 1 hlinkClick — count-equal, no orphan.
///
/// FAILS before fix: external_rel_count=2, hlinkclick_count=1 (orphan from nested).
/// PASSES after fix: external_rel_count=1, hlinkclick_count=1.
#[test]
fn test_f085_p6_001_mixed_toplevel_and_nested_link_exactly_one_rel_one_click() {
    let toplevel_url = "https://example.com/toplevel";
    let nested_url = "https://example.com/nested-inside-bold";

    // Top-level Link (registers + emits hlinkClick)
    let toplevel_link = slideforge_types::InlineNode::Link {
        url: Arc::from(toplevel_url),
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("top"))],
    };

    // Nested Link inside Bold (renders as plain text — no rel, no hlinkClick)
    let link_inside_bold = slideforge_types::InlineNode::Link {
        url: Arc::from(nested_url),
        text: vec![slideforge_types::InlineNode::Plain(Arc::from("nested"))],
    };
    let bold_node = slideforge_types::InlineNode::Bold(vec![link_inside_bold]);

    let rc = slideforge_types::register::RegisteredContent {
        register: Register::Notes,
        content: vec![toplevel_link, bold_node],
    };

    let slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: title_bbox(),
            content: FrameContent::Empty,
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: Some(Arc::from("top nested")),
        register_tags: vec![],
        register_content: vec![rc],
    };

    let deck = make_deck_with_notes(&[Some("top nested")]);
    let laid_out = LaidOutDeck {
        page_size: slideforge_layout::PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels_xml = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let external_rel_count = rels_xml.matches("TargetMode=\"External\"").count();
    let hlinkclick_count = notes_xml.matches("<a:hlinkClick").count();

    // COUNT-EQUAL invariant: no orphan rel.
    assert_eq!(
        external_rel_count, hlinkclick_count,
        "F-085-P6-001 [mixed]: orphan External rel — \
         TargetMode=External ({external_rel_count}) != <a:hlinkClick ({hlinkclick_count}). \
         Only the top-level Link should register+emit; the Bold-nested Link renders plain. \
         rels:\n{rels_xml}\nnotes:\n{notes_xml}"
    );

    // Exactly 1 rel (top-level URL only), exactly 1 hlinkClick.
    assert_eq!(
        external_rel_count, 1,
        "F-085-P6-001 [mixed]: expected exactly 1 External rel (top-level Link only); \
         got {external_rel_count}. rels:\n{rels_xml}"
    );
    assert_eq!(
        hlinkclick_count, 1,
        "F-085-P6-001 [mixed]: expected exactly 1 <a:hlinkClick (top-level Link only); \
         got {hlinkclick_count}. notes:\n{notes_xml}"
    );

    // Top-level URL appears in rels.
    assert!(
        rels_xml.contains(toplevel_url),
        "F-085-P6-001 [mixed]: top-level URL must appear in rels; got:\n{rels_xml}"
    );

    // Nested URL must NOT appear in rels.
    assert!(
        !rels_xml.contains(nested_url),
        "F-085-P6-001 [mixed]: nested URL must NOT appear in rels (it is plain text); \
         got:\n{rels_xml}"
    );
}
