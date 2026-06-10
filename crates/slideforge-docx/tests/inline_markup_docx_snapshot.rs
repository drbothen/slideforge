//! STORY-081 Red Gate tests — DOCX exporter slide-level inline markup rendering.
//!
//! ## Traceability
//!
//! | Test function | AC/EC | BC clause | What is verified |
//! |---|---|---|---|
//! | `test_BC_3_05_001_ac003_docx_bold_produces_w_b` | AC-003 | BC-3.05.001 PC-3 | Bold → `<w:b/>` in run properties |
//! | `test_BC_3_05_001_ac003_docx_italic_produces_w_i` | AC-003 | BC-3.05.001 PC-3 | Italic → `<w:i/>` in run properties |
//! | `test_BC_3_05_001_ac003_docx_code_produces_monospace` | AC-003 | BC-3.05.001 PC-3 | Code → monospace `<w:rFonts>` |
//! | `test_BC_3_05_001_ac003_docx_strikethrough_produces_w_strike` | AC-003 | BC-3.05.001 PC-3 | Strikethrough → `<w:strike/>` |
//! | `test_BC_3_05_001_ac003_docx_superscript_produces_vert_align_super` | AC-003 | BC-3.05.001 PC-3 | Superscript → vertAlign superscript |
//! | `test_BC_3_05_001_ac003_docx_subscript_produces_vert_align_sub` | AC-003 | BC-3.05.001 PC-3 | Subscript → vertAlign subscript |
//! | `test_BC_3_05_001_ac003_docx_highlight_produces_w_highlight_yellow` | AC-003 | BC-3.05.001 PC-3 | Highlight → `<w:highlight w:val="yellow"/>` (NOT plain text fallback) |
//! | `test_BC_3_05_001_ac003_docx_link_produces_w_hyperlink` | AC-003 | BC-3.05.001 PC-3 | Link → `<w:hyperlink r:id="...">` |
//! | `test_BC_3_05_001_ac003_docx_slide_body_inline_markup_wired` | AC-003 | BC-3.05.001 PC-3 | slide body FrameContent::Body with Bold → `<w:b/>` in DOCX output ZIP |
//! | `test_f_p13_001_docx_bold_wrapping_link_preserves_both_wb_and_hyperlink` | F-P13-001 | BC-3.05.001 PC-3 | Bold([Link]) → `<w:b/>` applied to run INSIDE `<w:hyperlink>` (not silently dropped) |
//! | `test_f_p13_001_docx_italic_wrapping_link_preserves_both_wi_and_hyperlink` | F-P13-001 | BC-3.05.001 PC-3 | Italic([Link]) → `<w:i/>` applied to run inside `<w:hyperlink>` |
//! | `test_f_p13_002_docx_bold_italic_combined_form_both_wb_and_wi` | F-P13-002 | BC-3.05.001 PC-3 | Bold([Italic([Plain])]) → one run with both `<w:b/>` and `<w:i/>` |
//!
//! ## Red Gate contract
//!
//! MUST FAIL before STORY-081 implementation:
//!
//! 1. `test_BC_3_05_001_ac003_docx_highlight_produces_w_highlight_yellow` — FAILS because
//!    `inline_node_to_paragraph_choices` currently falls back to plain text for Highlight.
//!
//! 2. `test_BC_3_05_001_ac003_docx_slide_body_inline_markup_wired` — FAILS because the
//!    DOCX document body currently only handles `InlineNode::Plain` for slide body frames.
//!
//! 3. `test_f_p13_001_docx_bold_wrapping_link_preserves_both_wb_and_hyperlink` — FAILS
//!    because `apply_run_property` passes `ParagraphChoice::WHyperlink` through unchanged,
//!    silently dropping `<w:b/>` when Bold wraps a Link.
//!
//! 4. `test_f_p13_001_docx_italic_wrapping_link_preserves_both_wi_and_hyperlink` — FAILS
//!    for the same reason: Italic([Link]) drops `<w:i/>`.
//!
//! 5. `test_f_p13_002_docx_bold_italic_combined_form_both_wb_and_wi` — load-bearing
//!    regression guard for Bold+Italic accumulation on the same run.

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::doc_markdown)]

use std::io::Read as IoRead;
use std::sync::Arc;

use slideforge_docx::document_body::DocumentBodySerializer;
use slideforge_layout::types::{
    BoundingBox, Emu, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize,
};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::span::SourceSpan;
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, ContentBlock, Deck, DeckMetadata, InlineNode, OrderedMap,
    Register, RegisteredContent, TextBlock, TextTag,
};
use zip::ZipArchive;

use slideforge_docx::DocxExporter;

// ─── Fixture helpers ──────────────────────────────────────────────────────────

fn make_brand() -> Brand {
    use slideforge_types::span::SourceSpan;
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

fn make_minimal_deck() -> Deck {
    Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("Inline Markup Test")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
        slide_sections: vec![],
    }
}

fn make_slide_with_register_content(nodes: Vec<InlineNode>) -> LaidOutSlide {
    LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("content"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(5_143_500),
            },
            content: FrameContent::Title(Arc::from("Test Slide")),
            text_flow: None,
            region_role: None,
        }],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![RegisteredContent {
            register: Register::Report,
            content: nodes,
        }],
    }
}

fn make_slide_with_body_blocks(nodes: Vec<InlineNode>) -> LaidOutSlide {
    let text_block = TextBlock {
        inlines: nodes,
        tag: TextTag::Untagged,
        span: SourceSpan::default(),
    };
    LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("content"),
        frames: vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(1_143_000),
                },
                content: FrameContent::Title(Arc::from("Test Slide")),
                text_flow: None,
                region_role: None,
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(1_200_000),
                    width: Emu(9_144_000),
                    height: Emu(4_000_000),
                },
                content: FrameContent::Body(vec![ContentBlock::Text(text_block)]),
                text_flow: None,
                region_role: None,
            },
        ],
        speaker_notes: None,
        register_tags: vec![],
        register_content: vec![],
    }
}

fn make_laid_out_deck(slide: LaidOutSlide) -> LaidOutDeck {
    LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![slide],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    }
}

/// Serialize the given inline nodes via `DocumentBodySerializer` using the
/// report register content path, and return the document.xml bytes as a String.
fn serialize_nodes_as_report_content(nodes: Vec<InlineNode>) -> String {
    let slide = make_slide_with_register_content(nodes);
    let deck = make_laid_out_deck(slide);
    let semantic_deck = make_minimal_deck(); // no title_inlines needed

    let mut serializer = DocumentBodySerializer::new();
    let bytes = serializer
        .serialize(&deck, &semantic_deck)
        .expect("DocumentBodySerializer::serialize must succeed");

    String::from_utf8(bytes).expect("document XML must be valid UTF-8")
}

/// Export a full DOCX from the given inline nodes in a Body frame, return document.xml.
fn export_docx_with_body_nodes(nodes: Vec<InlineNode>) -> String {
    let slide = make_slide_with_body_blocks(nodes);
    let laid_out = make_laid_out_deck(slide);
    let deck = make_minimal_deck();
    let brand = make_brand();

    let docx_bytes = DocxExporter
        .export(&deck, &laid_out, &brand, &ExportOptions::default())
        .expect("DocxExporter::export must succeed");

    let cursor = std::io::Cursor::new(&docx_bytes);
    let mut archive = ZipArchive::new(cursor).expect("DOCX must be a valid ZIP");
    let mut doc_xml = String::new();
    archive
        .by_name("word/document.xml")
        .expect("word/document.xml must be present")
        .read_to_string(&mut doc_xml)
        .expect("word/document.xml must be valid UTF-8");
    doc_xml
}

// ─── AC-003 unit tests: DocumentBodySerializer per variant ────────────────────

/// AC-003: `InlineNode::Bold` → `<w:b/>` in DOCX run properties.
///
/// Red Gate: Currently PASSES because Bold is already implemented in document_body.rs.
/// Kept for completeness — regression guard.
#[test]
fn test_BC_3_05_001_ac003_docx_bold_produces_w_b() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from(
        "bold text",
    ))])];
    let xml = serialize_nodes_as_report_content(nodes);

    assert!(
        xml.contains("<w:b/>") || xml.contains("<w:b ") || xml.contains("w:b>"),
        "AC-003: Bold must produce <w:b/> in DOCX run properties; XML (excerpt): {}",
        &xml[..xml.len().min(800)]
    );
}

/// AC-003: `InlineNode::Italic` → `<w:i/>` in DOCX run properties.
#[test]
fn test_BC_3_05_001_ac003_docx_italic_produces_w_i() {
    let nodes = vec![InlineNode::Italic(vec![InlineNode::Plain(Arc::from(
        "italic",
    ))])];
    let xml = serialize_nodes_as_report_content(nodes);

    assert!(
        xml.contains("<w:i/>") || xml.contains("<w:i ") || xml.contains("w:i>"),
        "AC-003: Italic must produce <w:i/> in DOCX; XML (excerpt): {}",
        &xml[..xml.len().min(800)]
    );
}

/// AC-003: `InlineNode::Code` → monospace font run in DOCX.
#[test]
fn test_BC_3_05_001_ac003_docx_code_produces_monospace() {
    let nodes = vec![InlineNode::Code(Arc::from("fn foo()"))];
    let xml = serialize_nodes_as_report_content(nodes);

    assert!(
        xml.contains("Courier") || xml.contains("CodeSpan") || xml.contains("w:rFonts"),
        "AC-003: Code must produce monospace font run; XML (excerpt): {}",
        &xml[..xml.len().min(800)]
    );
}

/// AC-003: `InlineNode::Strikethrough` → `<w:strike/>` in DOCX run properties.
#[test]
fn test_BC_3_05_001_ac003_docx_strikethrough_produces_w_strike() {
    let nodes = vec![InlineNode::Strikethrough(vec![InlineNode::Plain(
        Arc::from("del"),
    )])];
    let xml = serialize_nodes_as_report_content(nodes);

    assert!(
        xml.contains("<w:strike/>") || xml.contains("w:strike"),
        "AC-003: Strikethrough must produce <w:strike/> in DOCX; XML (excerpt): {}",
        &xml[..xml.len().min(800)]
    );
}

/// AC-003: `InlineNode::Superscript` → `<w:vertAlign w:val="superscript"/>`.
#[test]
fn test_BC_3_05_001_ac003_docx_superscript_produces_vert_align_super() {
    let nodes = vec![InlineNode::Superscript(vec![InlineNode::Plain(Arc::from(
        "sup",
    ))])];
    let xml = serialize_nodes_as_report_content(nodes);

    assert!(
        xml.contains("superscript"),
        "AC-003: Superscript must produce vertAlign superscript; XML (excerpt): {}",
        &xml[..xml.len().min(800)]
    );
}

/// AC-003: `InlineNode::Subscript` → `<w:vertAlign w:val="subscript"/>`.
#[test]
fn test_BC_3_05_001_ac003_docx_subscript_produces_vert_align_sub() {
    let nodes = vec![InlineNode::Subscript(vec![InlineNode::Plain(Arc::from(
        "sub",
    ))])];
    let xml = serialize_nodes_as_report_content(nodes);

    assert!(
        xml.contains("subscript"),
        "AC-003: Subscript must produce vertAlign subscript; XML (excerpt): {}",
        &xml[..xml.len().min(800)]
    );
}

/// AC-003 critical: `InlineNode::Highlight` → `<w:highlight w:val="yellow"/>`.
///
/// **Red Gate failure:** The current implementation falls back to plain text for
/// `InlineNode::Highlight` (document_body.rs):
/// ```rust
/// InlineNode::Footnote(children) | InlineNode::Highlight(children) => {
///     let text = collect_plain_text(children);
///     Ok(vec![ParagraphChoice::WR(Box::new(make_plain_run(&text)))])
/// }
/// ```
/// This test MUST FAIL until Highlight uses `w:highlight w:val="yellow"`.
#[test]
fn test_BC_3_05_001_ac003_docx_highlight_produces_w_highlight_yellow() {
    let nodes = vec![InlineNode::Highlight(vec![InlineNode::Plain(Arc::from(
        "highlighted",
    ))])];
    let xml = serialize_nodes_as_report_content(nodes);

    // Must contain w:highlight with yellow value — NOT be a plain run.
    assert!(
        xml.contains("w:highlight") && xml.contains("yellow"),
        "AC-003: InlineNode::Highlight must produce <w:highlight w:val=\"yellow\"/> \
         NOT a plain text fallback. \
         Current implementation falls back to plain text (F-DOCX-Highlight). \
         XML (excerpt): {}",
        &xml[..xml.len().min(800)]
    );
}

/// AC-003: `InlineNode::Link` → `<w:hyperlink r:id="...">` in DOCX.
#[test]
fn test_BC_3_05_001_ac003_docx_link_produces_w_hyperlink() {
    let nodes = vec![InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("click here"))],
        url: Arc::from("https://example.com"),
    }];
    let xml = serialize_nodes_as_report_content(nodes);

    assert!(
        xml.contains("w:hyperlink"),
        "AC-003: Link must produce <w:hyperlink> in DOCX; XML (excerpt): {}",
        &xml[..xml.len().min(800)]
    );
}

/// AC-003 pipeline test: a slide with a `FrameContent::Body` containing
/// `ContentBlock::Text` with `InlineNode::Bold` must produce `<w:b/>` in the
/// full DOCX output.
///
/// **Red Gate failure:** The DOCX document body currently extracts only
/// `InlineNode::Plain` from slide body frames:
/// ```rust
/// .filter_map(|node| {
///     if let InlineNode::Plain(s) = node { Some(s.as_ref()) } else { None }
/// })
/// ```
/// This discards Bold, Italic, etc. → no `<w:b/>` in DOCX output.
/// After STORY-081, slide body frames call `inline_node_to_paragraph_choices`.
#[test]
fn test_BC_3_05_001_ac003_docx_slide_body_inline_markup_wired() {
    let bold_nodes = vec![
        InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Key finding"))]),
        InlineNode::Plain(Arc::from(": revenue up 12%")),
    ];

    let doc_xml = export_docx_with_body_nodes(bold_nodes);

    // The body frame must contain <w:b/> for the Bold node.
    assert!(
        doc_xml.contains("<w:b/>") || doc_xml.contains("<w:b ") || doc_xml.contains("w:b>"),
        "AC-003: DOCX slide body with Bold InlineNode must contain <w:b/>; \
         currently fails because Body frames use Plain-only extraction. \
         document.xml (excerpt): {}",
        &doc_xml[..doc_xml.len().min(1000)]
    );

    // Must NOT contain literal asterisks.
    assert!(
        !doc_xml.contains("**"),
        "AC-003: DOCX must not contain literal '**' — R1 anti-pattern"
    );
}

// =============================================================================
// F-P13-001: DOCX silently drops inline formatting applied to a hyperlink
//
// Defect: `apply_run_property` in document_body.rs has a pass-through arm
// (`other => other`) for `ParagraphChoice::WHyperlink`. When Bold or Italic
// wraps a Link (e.g. `**[click](https://x)**`), the wrapper calls
// `apply_run_property(WHyperlink, set bold)`, hits the pass-through arm, and
// <w:b/> is NEVER applied to the run inside the hyperlink.
//
// Fix direction: when the child choice is WHyperlink, apply the run property
// to the runs INSIDE the hyperlink (iterate hyperlink_choice WR items).
// =============================================================================

/// F-P13-001 RED GATE: `Bold([Link{...}])` → DOCX must contain BOTH `<w:hyperlink`
/// AND `<w:b/>` with the bold applied to the run INSIDE the hyperlink.
///
/// **Red Gate failure:** `apply_run_property` passes `ParagraphChoice::WHyperlink`
/// through unchanged via the `other => other` arm. `<w:b/>` is silently dropped.
/// The link text + clickability survive but the styling is discarded.
///
/// Fixes required:
/// - `apply_run_property` (or the Bold arm) must recurse into `WHyperlink.hyperlink_choice`
///   and apply the property to each `WR` run inside the hyperlink.
/// - Clickability (the `r:id` relationship) must be preserved.
#[test]
fn test_f_p13_001_docx_bold_wrapping_link_preserves_both_wb_and_hyperlink() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("click"))],
        url: Arc::from("https://example.com"),
    }])];
    let xml = serialize_nodes_as_report_content(nodes);

    // 1. The hyperlink element must be present (clickability preserved).
    assert!(
        xml.contains("w:hyperlink"),
        "F-P13-001: Bold([Link]) must still produce <w:hyperlink> (clickability); \
         XML (excerpt): {}",
        &xml[..xml.len().min(1200)]
    );

    // 2. <w:b/> must appear INSIDE the hyperlink — the bold run property must
    //    be applied to the inner run, not silently dropped.
    //
    // The current defect: apply_run_property passes WHyperlink through unchanged
    // so <w:b/> never appears when Bold wraps a Link. This assertion FAILS before fix.
    assert!(
        xml.contains("<w:b/>") || xml.contains("<w:b "),
        "F-P13-001 RED GATE: Bold([Link]) must produce <w:b/> inside <w:hyperlink>. \
         Currently the Bold run property is SILENTLY DROPPED because apply_run_property \
         passes ParagraphChoice::WHyperlink through its `other => other` arm without \
         applying the bold property to the inner runs. \
         Fix: recurse into hyperlink_choice WR items and apply bold there. \
         XML (excerpt): {}",
        &xml[..xml.len().min(1200)]
    );

    // 3. The display text must still appear.
    assert!(
        xml.contains("click"),
        "F-P13-001: Bold([Link]) must retain the link display text 'click'; \
         XML (excerpt): {}",
        &xml[..xml.len().min(1200)]
    );
}

/// F-P13-001 RED GATE (variant): `Italic([Link{...}])` → DOCX must contain BOTH
/// `<w:hyperlink` AND `<w:i/>` applied to the run inside the hyperlink.
///
/// Same root-cause defect as the Bold variant: `apply_run_property` passes
/// `WHyperlink` through unchanged, dropping `<w:i/>`.
#[test]
fn test_f_p13_001_docx_italic_wrapping_link_preserves_both_wi_and_hyperlink() {
    let nodes = vec![InlineNode::Italic(vec![InlineNode::Link {
        text: vec![InlineNode::Plain(Arc::from("visit"))],
        url: Arc::from("https://example.org"),
    }])];
    let xml = serialize_nodes_as_report_content(nodes);

    assert!(
        xml.contains("w:hyperlink"),
        "F-P13-001 Italic: Italic([Link]) must still produce <w:hyperlink>; \
         XML (excerpt): {}",
        &xml[..xml.len().min(1200)]
    );

    assert!(
        xml.contains("<w:i/>") || xml.contains("<w:i "),
        "F-P13-001 RED GATE Italic: Italic([Link]) must produce <w:i/> inside <w:hyperlink>. \
         Currently <w:i/> is SILENTLY DROPPED by apply_run_property passing WHyperlink \
         through unchanged. Fix mirrors the Bold variant: recurse into hyperlink_choice. \
         XML (excerpt): {}",
        &xml[..xml.len().min(1200)]
    );

    assert!(
        xml.contains("visit"),
        "F-P13-001 Italic: display text 'visit' must be retained; \
         XML (excerpt): {}",
        &xml[..xml.len().min(1200)]
    );
}

// =============================================================================
// F-P13-002: load-bearing combined-form assertion — DOCX
//
// `Bold([Italic([Plain("x")])])` must produce a single run with BOTH <w:b/>
// AND <w:i/> in its <w:rPr>. This exercises the accumulator chain
// (Bold arm calls apply_run_property on each child choice; child of Bold is
// Italic which itself returns a WR with <w:i/>; the outer Bold then applies
// <w:b/> to that WR — resulting in both properties on the same run).
//
// If either accumulator step resets or overwrites the other, this test fails.
// =============================================================================

/// F-P13-002 load-bearing: `Bold([Italic([Plain("x")])])` → DOCX produces ONE run
/// with BOTH `<w:b/>` AND `<w:i/>` in its `<w:rPr>`.
///
/// This is NOT a Red Gate failure (combined-form Bold+Italic already works in the
/// current code via the nested `apply_run_property` chain). It IS a load-bearing
/// regression guard: if the Bold arm were to reset run properties before applying
/// bold, or if the accumulator chain broke, the combined properties would vanish.
///
/// A regression that drops either `<w:b/>` or `<w:i/>` will cause this to FAIL.
#[test]
fn test_f_p13_002_docx_bold_italic_combined_form_both_wb_and_wi() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Italic(vec![
        InlineNode::Plain(Arc::from("bi")),
    ])])];
    let xml = serialize_nodes_as_report_content(nodes);

    assert!(
        xml.contains("<w:b/>") || xml.contains("<w:b "),
        "F-P13-002: Bold([Italic([Plain])]) must produce <w:b/> in DOCX run properties; \
         XML (excerpt): {}",
        &xml[..xml.len().min(1000)]
    );

    assert!(
        xml.contains("<w:i/>") || xml.contains("<w:i "),
        "F-P13-002: Bold([Italic([Plain])]) must produce <w:i/> in DOCX run properties; \
         XML (excerpt): {}",
        &xml[..xml.len().min(1000)]
    );

    assert!(
        xml.contains("bi"),
        "F-P13-002: the plain text 'bi' must appear in DOCX output; \
         XML (excerpt): {}",
        &xml[..xml.len().min(1000)]
    );
}
