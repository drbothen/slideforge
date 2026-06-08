//! STORY-040 evidence dump: PPTX Speaker Notes + notesMaster1.xml + handoutMaster1.xml.
//!
//! Exercises the real `PptxExporter::export` path, extracts XML from the produced
//! PPTX ZIP, and writes concrete evidence files to
//! `docs/demo-evidence/STORY-040/`.  Every file maps to a specific AC or
//! behavioral requirement from the story.
//!
//! Run from the worktree root:
//! ```
//! cargo run --example demo_notes_evidence -p slideforge-pptx 2>&1
//! ```

// Demo binary: allow test-style assertions and stdout.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::too_many_lines,
    // ZIP entry names use ".xml" suffix — all lowercase in the PPTX spec.
    clippy::case_sensitive_file_extension_comparisons,
    clippy::uninlined_format_args
)]

use std::fmt::Write as FmtWrite;
use std::io::Read as _;
use std::sync::Arc;

use slideforge_layout::{BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, Deck, Emu, InlineNode, Register, SourceSpan,
    deck::DeckMetadata, ordered_map::OrderedMap, register::RegisteredContent, slide::Slide,
};
use zip::ZipArchive;

use slideforge_pptx::PptxExporter;

// ─── Fixture builders (mirrors notes_tests.rs) ────────────────────────────────

/// Build a minimal `Brand` fixture for evidence generation.
fn make_brand() -> Brand {
    Brand {
        name: Arc::from("evidence-brand"),
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

/// Build a minimal `DeckMetadata` fixture for evidence generation.
fn make_metadata() -> DeckMetadata {
    DeckMetadata {
        title: Some(Arc::from("Notes Evidence Deck")),
        slideforge_version: Arc::from("0.1.0"),
        lang: Some(Arc::from("en-US")),
        author: None,
        section_order: None,
    }
}

/// Return a standard title-placeholder bounding box for evidence slides.
fn title_bbox() -> BoundingBox {
    BoundingBox {
        x: Emu(457_200),
        y: Emu(274_638),
        width: Emu(8_229_600),
        height: Emu(1_143_000),
    }
}

/// Build a single `LaidOutSlide` with an optional speaker-notes string.
fn make_laid_out_slide(index: usize, notes_text: Option<&str>) -> LaidOutSlide {
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

/// Build a `Deck` whose slides carry the given per-slide notes strings.
fn make_deck_with_notes(notes_per_slide: &[Option<&str>]) -> Deck {
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
        vars: OrderedMap::new(),
        metadata: make_metadata(),
        registers: OrderedMap::new(),
        section_blocks: vec![],
        slide_sections: vec![],
    }
}

/// Build a `LaidOutDeck` whose slides carry the given per-slide notes strings.
fn make_laid_out_deck(notes_per_slide: &[Option<&str>]) -> LaidOutDeck {
    let slides: Vec<LaidOutSlide> = notes_per_slide
        .iter()
        .enumerate()
        .map(|(i, maybe_notes)| make_laid_out_slide(i, *maybe_notes))
        .collect();
    LaidOutDeck {
        page_size: PageSize::default(),
        slides,
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    }
}

/// Export the given `Deck` + `LaidOutDeck` through `PptxExporter` and return raw PPTX bytes.
fn export_pptx(deck: &Deck, laid_out: &LaidOutDeck) -> Vec<u8> {
    let brand = make_brand();
    let opts = ExportOptions::default();
    PptxExporter::new()
        .export(deck, laid_out, &brand, &opts)
        .expect("PptxExporter::export must succeed for valid input")
}

// ─── ZIP helpers ──────────────────────────────────────────────────────────────

/// Return all ZIP entry names found in the given PPTX byte slice.
fn list_zip_entries(pptx_bytes: &[u8]) -> Vec<String> {
    let cursor = std::io::Cursor::new(pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("valid ZIP");
    (0..archive.len())
        .map(|i| archive.by_index(i).expect("valid index").name().to_owned())
        .collect()
}

/// Read a ZIP entry by name, returning `None` if the entry is absent.
fn read_zip_member_opt(pptx_bytes: &[u8], member_name: &str) -> Option<String> {
    let cursor = std::io::Cursor::new(pptx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("valid ZIP");
    for i in 0..archive.len() {
        let name = {
            let entry = archive.by_index(i).expect("valid index");
            entry.name().to_owned()
        };
        if name == member_name {
            let mut entry = archive.by_index(i).expect("valid index");
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).expect("readable entry");
            return Some(String::from_utf8_lossy(&buf).into_owned());
        }
    }
    None
}

/// Read a ZIP entry by name, panicking if the entry is absent.
fn read_zip_member(pptx_bytes: &[u8], member_name: &str) -> String {
    read_zip_member_opt(pptx_bytes, member_name)
        .unwrap_or_else(|| panic!("ZIP member {member_name:?} not found"))
}

/// Best-effort XML pretty-printer: adds newlines at tag boundaries and indents.
fn pretty_xml(xml: &str) -> String {
    let rough = xml.replace("><", ">\n<");
    let mut out = String::new();
    let mut depth: usize = 0;
    for line in rough.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let is_self_closing = trimmed.starts_with('<')
            && !trimmed.starts_with("</")
            && !trimmed.starts_with("<?")
            && !trimmed.starts_with("<!--")
            && trimmed.ends_with("/>");
        let is_close = trimmed.starts_with("</");
        if is_close {
            depth = depth.saturating_sub(1);
        }
        out.push_str(&"  ".repeat(depth));
        out.push_str(trimmed);
        out.push('\n');
        if !is_close && !is_self_closing && trimmed.starts_with('<') {
            depth += 1;
        }
    }
    out
}

/// Extract a short XML snippet around a keyword for evidence purposes.
fn xml_snippet(xml: &str, keyword: &str, context_chars: usize) -> String {
    if let Some(pos) = xml.find(keyword) {
        let start = pos.saturating_sub(context_chars);
        let end = (pos + keyword.len() + context_chars).min(xml.len());
        format!("...{}...", &xml[start..end])
    } else {
        format!("(keyword {keyword:?} not found in XML)")
    }
}

/// Append a boxed section header to `buf` for evidence report formatting.
fn section_header(buf: &mut String, title: &str) {
    let line = "=".repeat(title.len() + 4);
    let _ = writeln!(buf, "\n{line}");
    let _ = writeln!(buf, "= {title} =");
    let _ = writeln!(buf, "{line}\n");
}

// ─── Per-AC evidence functions ────────────────────────────────────────────────

/// Write AC-001 evidence: notesSlide ZIP entry count matches slides-with-notes count.
fn evidence_ac001(out_dir: &std::path::Path) {
    println!("[AC-001] Exporting 3-slide deck (slides 1+3 have notes, slide 2 none)...");
    let deck = make_deck_with_notes(&[
        Some("Speaker note for slide 1"),
        None,
        Some("Speaker note for slide 3"),
    ]);
    let laid_out = make_laid_out_deck(&[
        Some("Speaker note for slide 1"),
        None,
        Some("Speaker note for slide 3"),
    ]);
    let pptx = export_pptx(&deck, &laid_out);

    let entries = list_zip_entries(&pptx);
    let notes_entries: Vec<&str> = entries
        .iter()
        .filter(|e| e.starts_with("ppt/notesSlides/notesSlide") && e.ends_with(".xml"))
        .map(String::as_str)
        .collect();

    let mut buf = String::new();
    section_header(
        &mut buf,
        "AC-001: notesSlide ZIP entries (3-slide deck, slides 1+3 have notes)",
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_BC_4_01_003_ac001_notes_slide_count_equals_slides_with_notes"
    );
    let _ = writeln!(&mut buf, "BC:   BC-4.01.003 postcondition 1\n");
    let _ = writeln!(
        &mut buf,
        "ZIP entries matching ppt/notesSlides/notesSlide*.xml ({} found):",
        notes_entries.len()
    );
    for e in &notes_entries {
        let _ = writeln!(&mut buf, "  - {e}");
    }
    let _ = writeln!(&mut buf, "\nAll ZIP entries (ppt/ only, for context):");
    for e in entries.iter().filter(|e| e.starts_with("ppt/")) {
        let _ = writeln!(&mut buf, "  {e}");
    }
    let _ = writeln!(
        &mut buf,
        "\nAssertion: notes_entries.len() == 2 -> {}",
        notes_entries.len() == 2
    );

    let path = out_dir.join("AC-001-notes-slide-zip-entries.txt");
    std::fs::write(&path, &buf).expect("write AC-001");
    println!("  -> wrote {}", path.display());
}

/// Write EC-001 evidence: no notesSlide ZIP entry is produced when the deck has no notes.
fn evidence_ec001(out_dir: &std::path::Path) {
    println!("[EC-001] Exporting 1-slide deck with no notes...");
    let deck = make_deck_with_notes(&[None]);
    let laid_out = make_laid_out_deck(&[None]);
    let pptx = export_pptx(&deck, &laid_out);

    let entries = list_zip_entries(&pptx);
    let notes_count = entries
        .iter()
        .filter(|e| e.starts_with("ppt/notesSlides/notesSlide") && e.ends_with(".xml"))
        .count();
    let slide_count = entries
        .iter()
        .filter(|e| e.starts_with("ppt/slides/slide") && e.ends_with(".xml"))
        .count();

    let mut buf = String::new();
    section_header(
        &mut buf,
        "EC-001: no notesSlide when deck has no notes (no-notes deck)",
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_BC_4_01_003_ac001_ec001_no_notes_slide_for_empty_notes"
    );
    let _ = writeln!(&mut buf, "BC:   BC-4.01.003 postcondition 2 / EC-001\n");
    let _ = writeln!(
        &mut buf,
        "notesSlide ZIP entries (expected 0): {notes_count}"
    );
    let _ = writeln!(&mut buf, "slide ZIP entries (expected >=1): {slide_count}");
    let _ = writeln!(
        &mut buf,
        "\nAssertion: notes_count == 0 -> {}",
        notes_count == 0
    );
    let _ = writeln!(
        &mut buf,
        "Assertion: slide_count >= 1 -> {}",
        slide_count >= 1
    );

    let path = out_dir.join("EC-001-no-notes-slide-for-empty-notes.txt");
    std::fs::write(&path, &buf).expect("write EC-001");
    println!("  -> wrote {}", path.display());
}

/// Write AC-002 evidence: notes text appears inside the body placeholder txBody.
fn evidence_ac002(out_dir: &std::path::Path) {
    println!("[AC-002] Exporting deck with specific notes text, extracting body placeholder...");
    let notes_text = "Click here to rehearse your speaking points carefully.";
    let deck = make_deck_with_notes(&[Some(notes_text)]);
    let laid_out = make_laid_out_deck(&[Some(notes_text)]);
    let pptx = export_pptx(&deck, &laid_out);

    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let has_text = notes_xml.contains(notes_text);
    let has_body_ph = notes_xml.contains("type=\"body\"") && notes_xml.contains("idx=\"1\"");
    // <p:ph> must be self-closing: no </p:ph> close tag.
    let ph_self_closing = !notes_xml.contains("</p:ph>");

    let mut buf = String::new();
    section_header(
        &mut buf,
        "AC-002: notes text in body placeholder <p:ph type=\"body\" idx=\"1\"> txBody",
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_BC_4_01_003_ac002_notes_text_in_body_placeholder"
    );
    let _ = writeln!(&mut buf, "BC:   BC-4.01.003 postcondition 6\n");
    let _ = writeln!(&mut buf, "Input notes_text: {:?}\n", notes_text);
    let _ = writeln!(
        &mut buf,
        "ppt/notesSlides/notesSlide1.xml (formatted):\n{}",
        pretty_xml(&notes_xml)
    );
    let _ = writeln!(&mut buf, "\nKey evidence:");
    let _ = writeln!(&mut buf, "  <a:t> text node contains notes: {has_text}");
    let _ = writeln!(
        &mut buf,
        "  Body placeholder marker present:  {has_body_ph}"
    );
    let _ = writeln!(
        &mut buf,
        "  <p:ph> is self-closing (correct): {ph_self_closing}"
    );

    std::fs::write(out_dir.join("AC-002-notes-text-body-placeholder.txt"), &buf)
        .expect("write AC-002");
    std::fs::write(out_dir.join("AC-002-notesSlide1-raw.xml"), &notes_xml)
        .expect("write AC-002 raw");
    println!(
        "  -> wrote {}",
        out_dir
            .join("AC-002-notes-text-body-placeholder.txt")
            .display()
    );
    println!(
        "  -> wrote {}",
        out_dir.join("AC-002-notesSlide1-raw.xml").display()
    );
}

/// Write AC-003 evidence: notes sentinel string is absent from all slide body XML files.
fn evidence_ac003(out_dir: &std::path::Path) {
    println!("[AC-003] BleedChecker evidence -- sentinel absent from slide bodies...");
    let notes_sentinel = "NOTES_BLEED_SENTINEL_X7Q2Z9";
    let deck = make_deck_with_notes(&[Some(notes_sentinel)]);
    let laid_out = make_laid_out_deck(&[Some(notes_sentinel)]);
    let pptx = export_pptx(&deck, &laid_out);

    let entries = list_zip_entries(&pptx);
    let slide_entries: Vec<String> = entries
        .iter()
        .filter(|e| e.starts_with("ppt/slides/slide") && e.ends_with(".xml"))
        .cloned()
        .collect();

    let mut bleed_found = false;
    let mut checked: Vec<String> = Vec::new();
    for name in &slide_entries {
        let xml = read_zip_member(&pptx, name);
        let hit = xml.contains(notes_sentinel);
        checked.push(format!("  {name}: contains sentinel = {hit}"));
        if hit {
            bleed_found = true;
        }
    }

    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");
    let in_notes_slide = notes_xml.contains(notes_sentinel);

    let mut buf = String::new();
    section_header(
        &mut buf,
        "AC-003: notes text NOT in any ppt/slides/slide*.xml (BleedChecker)",
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_BC_4_01_003_ac003_notes_absent_from_slide_bodies"
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_f040_p1_004_ac003_no_bleed_with_positive_routing"
    );
    let _ = writeln!(&mut buf, "BC:   BC-4.01.003 invariant 1 / DI-012\n");
    let _ = writeln!(&mut buf, "Sentinel: {:?}", notes_sentinel);
    let _ = writeln!(
        &mut buf,
        "\nSlide bodies checked ({} slides):",
        slide_entries.len()
    );
    for line in &checked {
        let _ = writeln!(&mut buf, "{line}");
    }
    let _ = writeln!(
        &mut buf,
        "\nSentinel in notesSlide1.xml (positive routing): {in_notes_slide}"
    );
    let _ = writeln!(
        &mut buf,
        "Sentinel bleeds to any slide body:              {bleed_found}"
    );
    let _ = writeln!(
        &mut buf,
        "\nAssertion: bleed_found == false -> {}",
        !bleed_found
    );
    let _ = writeln!(
        &mut buf,
        "Assertion: in_notes_slide == true  -> {in_notes_slide}"
    );

    let path = out_dir.join("AC-003-no-bleed-sentinel-check.txt");
    std::fs::write(&path, &buf).expect("write AC-003");
    println!("  -> wrote {}", path.display());
}

/// Write AC-004 evidence: `notesMaster1.xml` is always present and structurally valid.
fn evidence_ac004(out_dir: &std::path::Path) {
    println!("[AC-004] Extracting notesMaster1.xml from no-notes deck...");
    let deck = make_deck_with_notes(&[None]);
    let laid_out = make_laid_out_deck(&[None]);
    let pptx = export_pptx(&deck, &laid_out);

    let master_xml = read_zip_member(&pptx, "ppt/notesMasters/notesMaster1.xml");

    let has_clrmap = master_xml.contains("<p:clrMap");
    let has_sldimg = master_xml.contains("type=\"sldImg\"");
    let has_body_ph = master_xml.contains("type=\"body\"") && master_xml.contains("idx=\"1\"");
    let no_grpsppr = !master_xml.contains("<a:grpSpPr");

    let mut buf = String::new();
    section_header(
        &mut buf,
        "AC-004: notesMaster1.xml always present and structurally valid",
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_BC_4_01_006_ac004_notes_master_always_present"
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_f040_p1_002_notes_master_xml_is_structurally_valid"
    );
    let _ = writeln!(
        &mut buf,
        "BC:   BC-4.01.006 invariant 1 / F-040-P1-002 (CRIT)\n"
    );
    let _ = writeln!(
        &mut buf,
        "ppt/notesMasters/notesMaster1.xml (formatted):\n{}",
        pretty_xml(&master_xml)
    );
    let _ = writeln!(&mut buf, "\nStructural assertions:");
    let _ = writeln!(
        &mut buf,
        "  <p:clrMap> present:                {has_clrmap}"
    );
    let _ = writeln!(
        &mut buf,
        "  <p:ph type=\"sldImg\"/> present:       {has_sldimg}"
    );
    let _ = writeln!(
        &mut buf,
        "  <p:ph type=\"body\" idx=\"1\"/> present: {has_body_ph}"
    );
    let _ = writeln!(
        &mut buf,
        "  NO <a:grpSpPr> (schema-valid):     {no_grpsppr}  [F-040-A1]"
    );
    let _ = writeln!(
        &mut buf,
        "\nAll assertions pass: {}",
        has_clrmap && has_sldimg && has_body_ph && no_grpsppr
    );

    std::fs::write(out_dir.join("AC-004-notes-master-validity.txt"), &buf).expect("write AC-004");
    std::fs::write(out_dir.join("AC-004-notesMaster1-raw.xml"), &master_xml)
        .expect("write AC-004 raw");
    println!(
        "  -> wrote {}",
        out_dir.join("AC-004-notes-master-validity.txt").display()
    );
    println!(
        "  -> wrote {}",
        out_dir.join("AC-004-notesMaster1-raw.xml").display()
    );
}

/// Write AC-005 evidence: `handoutMaster1.xml` is always present in the ZIP.
fn evidence_ac005(out_dir: &std::path::Path) {
    println!("[AC-005] Confirming handoutMaster1.xml presence...");
    let deck = make_deck_with_notes(&[None]);
    let laid_out = make_laid_out_deck(&[None]);
    let pptx = export_pptx(&deck, &laid_out);

    let handout_xml = read_zip_member(&pptx, "ppt/handoutMasters/handoutMaster1.xml");
    let no_grpsppr = !handout_xml.contains("<a:grpSpPr");

    let mut buf = String::new();
    section_header(&mut buf, "AC-005: handoutMaster1.xml always present");
    let _ = writeln!(
        &mut buf,
        "Test: test_BC_4_01_006_ac005_handout_master_always_present"
    );
    let _ = writeln!(&mut buf, "BC:   BC-4.01.006 postcondition 2\n");
    let _ = writeln!(
        &mut buf,
        "ppt/handoutMasters/handoutMaster1.xml (formatted):\n{}",
        pretty_xml(&handout_xml)
    );
    let _ = writeln!(&mut buf, "\nStructural assertions:");
    let _ = writeln!(
        &mut buf,
        "  NO <a:grpSpPr> (schema-valid): {no_grpsppr}  [F-040-A1]"
    );
    let _ = writeln!(&mut buf, "\nAssertion: handoutMaster1.xml present -> true");
    let _ = writeln!(
        &mut buf,
        "Assertion: no schema-invalid <a:grpSpPr> -> {no_grpsppr}"
    );

    let path = out_dir.join("AC-005-handout-master-presence.txt");
    std::fs::write(&path, &buf).expect("write AC-005");
    println!("  -> wrote {}", path.display());
}

/// Write RELS evidence: slide-to-notesSlide relationship in `_rels` files.
fn evidence_rels(out_dir: &std::path::Path) {
    println!("[RELS] Extracting slide1 and slide2 .rels to show notesSlide relationship...");
    let deck = make_deck_with_notes(&[Some("Speaker notes here"), None]);
    let laid_out = make_laid_out_deck(&[Some("Speaker notes here"), None]);
    let pptx = export_pptx(&deck, &laid_out);

    let slide1_rels = read_zip_member(&pptx, "ppt/slides/_rels/slide1.xml.rels");
    let slide2_rels = read_zip_member(&pptx, "ppt/slides/_rels/slide2.xml.rels");

    let has_notes_rel = slide1_rels.contains("notesSlide")
        && slide1_rels.contains("notesSlides/notesSlide1.xml")
        && slide1_rels.contains(
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide",
        );
    let no_notes_rel_slide2 = !slide2_rels.contains("notesSlide");

    let mut buf = String::new();
    section_header(
        &mut buf,
        "Slide-to-notesSlide relationship (F-040-P1-001, CRIT)",
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_f040_p1_001_slide_has_notes_slide_rel_in_slide_rels"
    );
    let _ = writeln!(
        &mut buf,
        "Requirement: slide WITH notes must have notesSlide rel in _rels file.\n"
    );
    let _ = writeln!(
        &mut buf,
        "ppt/slides/_rels/slide1.xml.rels (slide WITH notes):\n{}",
        pretty_xml(&slide1_rels)
    );
    let _ = writeln!(
        &mut buf,
        "\nppt/slides/_rels/slide2.xml.rels (slide WITHOUT notes):\n{}",
        pretty_xml(&slide2_rels)
    );
    let _ = writeln!(&mut buf, "\nAssertions:");
    let _ = writeln!(
        &mut buf,
        "  slide1 has notesSlide relationship:    {has_notes_rel}"
    );
    let _ = writeln!(
        &mut buf,
        "  slide2 has NO notesSlide relationship: {no_notes_rel_slide2}"
    );
    let _ = writeln!(
        &mut buf,
        "\nAll assertions pass: {}",
        has_notes_rel && no_notes_rel_slide2
    );

    let path = out_dir.join("RELS-slide-to-notes-slide-relationship.txt");
    std::fs::write(&path, &buf).expect("write RELS");
    println!("  -> wrote {}", path.display());
}

/// Write RICH evidence: bold and italic `InlineNode` trees produce correct `<a:rPr>` attributes.
fn evidence_rich(out_dir: &std::path::Path) {
    println!("[RICH] Exporting deck with bold+italic notes, extracting rPr...");
    let bold_node = InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold content"))]);
    let italic_node = InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic content"))]);
    let plain_node = InlineNode::Plain(Arc::from("plain text"));

    let rich_rc = RegisteredContent {
        register: Register::Notes,
        content: vec![bold_node, italic_node, plain_node],
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
        speaker_notes: Some(Arc::from("rich notes")),
        register_tags: vec![],
        register_content: vec![rich_rc],
    };

    let deck = make_deck_with_notes(&[Some("rich notes")]);
    let laid_out = LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let has_bold = notes_xml.contains("b=\"1\"");
    let has_italic = notes_xml.contains("i=\"1\"");
    let has_bold_text = notes_xml.contains("bold content");
    let has_italic_text = notes_xml.contains("italic content");
    let has_plain_text = notes_xml.contains("plain text");

    let mut buf = String::new();
    section_header(
        &mut buf,
        "Rich notes: bold + italic inline formatting (F-040-P1-003, HIGH)",
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_f040_p1_003_rich_notes_bold_and_italic_runs"
    );
    let _ = writeln!(
        &mut buf,
        "Requirement: bold InlineNode -> <a:rPr b=\"1\">, italic -> <a:rPr i=\"1\">\n"
    );
    let _ = writeln!(
        &mut buf,
        "notesSlide1.xml (formatted):\n{}",
        pretty_xml(&notes_xml)
    );
    let _ = writeln!(&mut buf, "\nFormatting evidence:");
    let _ = writeln!(&mut buf, "  <a:rPr b=\"1\"> (bold run):     {has_bold}");
    let _ = writeln!(&mut buf, "  <a:rPr i=\"1\"> (italic run):   {has_italic}");
    let _ = writeln!(&mut buf, "  'bold content' text present:   {has_bold_text}");
    let _ = writeln!(
        &mut buf,
        "  'italic content' text present: {has_italic_text}"
    );
    let _ = writeln!(
        &mut buf,
        "  'plain text' text present:     {has_plain_text}"
    );
    let _ = writeln!(
        &mut buf,
        "\nAll assertions pass: {}",
        has_bold && has_italic && has_bold_text && has_italic_text && has_plain_text
    );
    let _ = writeln!(
        &mut buf,
        "\nSnippet around b=\"1\": {}",
        xml_snippet(&notes_xml, "b=\"1\"", 80)
    );
    let _ = writeln!(
        &mut buf,
        "Snippet around i=\"1\": {}",
        xml_snippet(&notes_xml, "i=\"1\"", 80)
    );

    let path = out_dir.join("RICH-bold-italic-formatting.txt");
    std::fs::write(&path, &buf).expect("write RICH");
    println!("  -> wrote {}", path.display());
}

/// Write SAFEURL evidence: `javascript:` links degrade to plain text with no External rel.
fn evidence_safeurl(out_dir: &std::path::Path) {
    println!(
        "[SAFEURL] Exporting deck with javascript: link, confirming plain text degradation..."
    );
    let url: Arc<str> = Arc::from("javascript:alert(1)");
    let link_node = InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("click me"))],
        url: url.clone(),
    };
    let rc = RegisteredContent {
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
        page_size: PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let rels = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");
    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let no_js_in_rels = !rels.contains("javascript");
    let no_external_rel = !rels.contains("TargetMode=\"External\"");
    let no_js_in_xml = !notes_xml.contains("javascript");
    let display_text_present = notes_xml.contains("click me");

    // Extract just the txBody section for the evidence snippet.
    let txbody_excerpt = if let Some(start) = notes_xml.find("<p:txBody") {
        let end = notes_xml[start..]
            .find("</p:txBody>")
            .map_or(notes_xml.len(), |i| start + i + 11);
        pretty_xml(&notes_xml[start..end])
    } else {
        pretty_xml(&notes_xml)
    };

    let mut buf = String::new();
    section_header(
        &mut buf,
        "SafeUrl: javascript: link -> plain text, NO External rel (F-040-P2-001, CWE-601)",
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_f040_p2_001_unsafe_scheme_javascript_no_external_rel"
    );
    let _ = writeln!(
        &mut buf,
        "BC:   CWE-601 defense: unsafe URL schemes must degrade to plain text.\n"
    );
    let _ = writeln!(&mut buf, "Input URL: {:?}", url);
    let _ = writeln!(
        &mut buf,
        "\nppt/notesSlides/_rels/notesSlide1.xml.rels:\n{}",
        pretty_xml(&rels)
    );
    let _ = writeln!(
        &mut buf,
        "\nppt/notesSlides/notesSlide1.xml (txBody excerpt):\n{txbody_excerpt}"
    );
    let _ = writeln!(&mut buf, "\nSecurity assertions (CWE-601):");
    let _ = writeln!(
        &mut buf,
        "  'javascript' absent from .rels:          {no_js_in_rels}"
    );
    let _ = writeln!(
        &mut buf,
        "  NO TargetMode=External rel:              {no_external_rel}"
    );
    let _ = writeln!(
        &mut buf,
        "  'javascript' absent from notesSlide xml: {no_js_in_xml}"
    );
    let _ = writeln!(
        &mut buf,
        "  'click me' display text preserved:       {display_text_present}"
    );
    let _ = writeln!(
        &mut buf,
        "\nAll assertions pass: {}",
        no_js_in_rels && no_external_rel && no_js_in_xml && display_text_present
    );

    let path = out_dir.join("SAFEURL-javascript-plain-text-degradation.txt");
    std::fs::write(&path, &buf).expect("write SAFEURL");
    println!("  -> wrote {}", path.display());
}

/// Write LINK evidence: safe `https:` links produce `<a:hlinkClick>` and an External rel.
fn evidence_link(out_dir: &std::path::Path) {
    println!("[LINK] Exporting deck with https: link, confirming hlinkClick + External rel...");
    let target_url = "https://example.com/notes-link";
    let link_node = InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("example"))],
        url: Arc::from(target_url),
    };
    let rc = RegisteredContent {
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
        page_size: PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };
    let pptx = export_pptx(&deck, &laid_out);

    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");
    let rels = read_zip_member(&pptx, "ppt/notesSlides/_rels/notesSlide1.xml.rels");

    let has_hlinkclick = notes_xml.contains("hlinkClick");
    let has_rid3 = notes_xml.contains("rId3");
    let rels_has_rid3 = rels.contains("rId3");
    let rels_external = rels.contains("TargetMode=\"External\"");
    let rels_target = rels.contains(target_url);

    let mut buf = String::new();
    section_header(
        &mut buf,
        "Safe https: link -> hlinkClick + External rel (F-040-P2-002, HIGH)",
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_f040_p2_002_single_safe_https_link_has_hlinkclick_and_external_rel"
    );
    let _ = writeln!(
        &mut buf,
        "Requirement: https: URL -> <a:hlinkClick r:id=\"rId3\"/> + TargetMode=External rel.\n"
    );
    let _ = writeln!(&mut buf, "Input URL: {:?}", target_url);
    let _ = writeln!(
        &mut buf,
        "\nppt/notesSlides/_rels/notesSlide1.xml.rels:\n{}",
        pretty_xml(&rels)
    );
    let _ = writeln!(
        &mut buf,
        "\nSnippet around hlinkClick: {}",
        xml_snippet(&notes_xml, "hlinkClick", 120)
    );
    let _ = writeln!(&mut buf, "\nAssertions:");
    let _ = writeln!(
        &mut buf,
        "  <a:hlinkClick> present in notesSlide xml: {has_hlinkclick}"
    );
    let _ = writeln!(
        &mut buf,
        "  rId3 referenced in notesSlide xml:        {has_rid3}"
    );
    let _ = writeln!(
        &mut buf,
        "  rId3 Relationship in .rels:               {rels_has_rid3}"
    );
    let _ = writeln!(
        &mut buf,
        "  TargetMode=External in .rels:             {rels_external}"
    );
    let _ = writeln!(
        &mut buf,
        "  Target URL in .rels:                      {rels_target}"
    );
    let _ = writeln!(
        &mut buf,
        "\nAll assertions pass: {}",
        has_hlinkclick && has_rid3 && rels_has_rid3 && rels_external && rels_target
    );

    let path = out_dir.join("LINK-safe-https-hlinkclick-external-rel.txt");
    std::fs::write(&path, &buf).expect("write LINK");
    println!("  -> wrote {}", path.display());
}

/// Write SCHEMA evidence: notesSlide XML uses `<p:grpSpPr>` (not `<a:grpSpPr>`) — schema-valid.
fn evidence_schema(out_dir: &std::path::Path) {
    println!("[SCHEMA] Verifying grpSpPr schema validity in notesSlide XML...");
    let deck = make_deck_with_notes(&[Some("schema check notes")]);
    let laid_out = make_laid_out_deck(&[Some("schema check notes")]);
    let pptx = export_pptx(&deck, &laid_out);

    let notes_xml = read_zip_member(&pptx, "ppt/notesSlides/notesSlide1.xml");

    let no_a_grpsppr = !notes_xml.contains("<a:grpSpPr");
    let has_p_grpsppr = notes_xml.contains("<p:grpSpPr>");
    let has_xfrm = notes_xml.contains("<a:xfrm>");

    let mut buf = String::new();
    section_header(
        &mut buf,
        "Schema-valid grpSpPr: <p:grpSpPr> with <a:xfrm>, NOT <a:grpSpPr> (F-040-A1)",
    );
    let _ = writeln!(
        &mut buf,
        "Test: test_f040_a1_notes_slide_grpsppr_is_schema_valid"
    );
    let _ = writeln!(
        &mut buf,
        "Requirement: CT_GroupShapeProperties permits <a:xfrm> child but has NO <a:grpSpPr> child.\n"
    );
    let _ = writeln!(
        &mut buf,
        "Snippet around grpSpPr: {}",
        xml_snippet(&notes_xml, "grpSpPr", 200)
    );
    let _ = writeln!(&mut buf, "\nSchema assertions:");
    let _ = writeln!(
        &mut buf,
        "  NO <a:grpSpPr> (would be schema-invalid): {no_a_grpsppr}"
    );
    let _ = writeln!(
        &mut buf,
        "  <p:grpSpPr> present (spTree first child):  {has_p_grpsppr}"
    );
    let _ = writeln!(
        &mut buf,
        "  <a:xfrm> identity transform present:        {has_xfrm}"
    );
    let _ = writeln!(
        &mut buf,
        "\nAll assertions pass: {}",
        no_a_grpsppr && has_p_grpsppr && has_xfrm
    );

    let path = out_dir.join("SCHEMA-grpSpPr-no-nested-a-grpSpPr.txt");
    std::fs::write(&path, &buf).expect("write SCHEMA");
    println!("  -> wrote {}", path.display());
}

// ─── main ─────────────────────────────────────────────────────────────────────

fn main() {
    let out_dir = std::path::PathBuf::from("docs/demo-evidence/STORY-040");
    std::fs::create_dir_all(&out_dir).expect("create evidence dir");

    evidence_ac001(&out_dir);
    evidence_ec001(&out_dir);
    evidence_ac002(&out_dir);
    evidence_ac003(&out_dir);
    evidence_ac004(&out_dir);
    evidence_ac005(&out_dir);
    evidence_rels(&out_dir);
    evidence_rich(&out_dir);
    evidence_safeurl(&out_dir);
    evidence_link(&out_dir);
    evidence_schema(&out_dir);

    println!("\n[OK] All evidence written to docs/demo-evidence/STORY-040/");
}
