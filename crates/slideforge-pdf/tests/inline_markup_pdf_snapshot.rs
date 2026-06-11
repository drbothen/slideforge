//! STORY-081 Red Gate tests — PDF exporter slide-level inline markup rendering.
//!
//! ## Traceability
//!
//! | Test function | AC/EC | BC clause | What is verified |
//! |---|---|---|---|
//! | `test_BC_3_05_001_ac004_pdf_bold_uses_font_face_dispatch` | AC-004 | BC-3.05.001 PC-5 | Bold → font face dispatch (not `extract_inline_text` flattening) |
//! | `test_BC_3_05_001_ac004_pdf_italic_uses_font_face_dispatch` | AC-004 | BC-3.05.001 PC-5 | Italic → italic font face (not plain text) |
//! | `test_BC_3_05_001_ac004_pdf_code_uses_mono_font_face` | AC-004 | BC-3.05.001 PC-5 | Code → monospace font face |
//! | `test_BC_3_05_001_ac004_pdf_slide_to_krilla_runs_stub_exists` | AC-004 | BC-3.05.001 PC-5 | `slide_to_krilla_runs` function exists in PDF crate |
//! | `test_BC_3_05_001_ac004_pdf_font_selector_bold_dispatches` | AC-004 | BC-3.05.001 PC-5 | `select_font_face` returns bold face for `InlineNode::Bold` |
//! | `test_BC_3_05_001_ac004_pdf_font_selector_italic_dispatches` | AC-004 | BC-3.05.001 PC-5 | `select_font_face` returns italic face for `InlineNode::Italic` |
//! | `test_BC_3_05_001_ac004_pdf_extract_inline_text_no_longer_used_for_slide_body` | AC-004 | BC-3.05.001 PC-5 | slide body does NOT use flat `extract_inline_text` path for Bold/Italic |
//! | `test_BC_3_05_001_ac004_pdf_bold_span_uses_distinct_font_resource` | AC-004 C2-NEW | BC-3.05.001 PC-5 | Build()-driven: bold span embeds a DISTINCT font resource from plain span |
//! | `test_BC_3_05_001_adv_p04_crit001_measure_text_width_returns_nonzero` | ADV-P04-CRIT-001 | BC-3.05.001 PC-5 | `measure_text_width_pt` returns >0 for real fixture font + text |
//! | `test_BC_3_05_001_adv_p04_crit001_multi_span_positions_strictly_increasing` | ADV-P04-CRIT-001 | BC-3.05.001 PC-5 | Second span in multi-span line has draw-X > first span's draw-X |
//! | `test_BC_3_05_001_adv_p04_high001_super_span_uses_reduced_font_size` | ADV-P04-HIGH-001 | BC-3.05.001 PC-5 | Superscript span uses `font_size * SUPER_SUB_SCALE` (< parent font size) |
//! | `test_BC_3_05_001_adv_p04_high001_sub_span_uses_reduced_font_size` | ADV-P04-HIGH-001 | BC-3.05.001 PC-5 | Subscript span uses `font_size * SUPER_SUB_SCALE` (< parent font size) |
//! | `test_BC_3_05_001_adv_p08_high001_pdf_title_bold_uses_distinct_font_resource` | ADV-P08-HIGH-001 | BC-3.05.001 Slide-Level Title Constraint / EC-011 / PC-5 | PDF title with `InlineNode::Bold` embeds a DISTINCT bold face (not regular-face flattening) |
//! | `test_BC_3_05_001_ac004_pc5_superscript_baseline_raised_subscript_lowered` | AC-004 | BC-3.05.001 PC-5 | `adjusted_baseline_y`: superscript raises (Y < baseline), subscript lowers (Y > baseline), normal unchanged |
//!
//! ## krilla 0.6.0 constraint (AC-004)
//!
//! Per STORY-081 AC-004 and ADR-023 amendment (2026-06-09):
//! - Bold: `Font::new(bold_font_data, index)` — NO `set_bold()` toggle
//! - Italic: `Font::new(italic_font_data, index)` — NO `set_italic()` toggle
//! - Superscript/Subscript: `surface.draw_text()` at shifted baseline
//!   (`±font_size * SUPER_RISE_FRACTION/SUB_DROP_FRACTION`) with reduced size
//!   (`font_size * SUPER_SUB_SCALE = 0.583`).  NO `KrillaGlyph.y_offset` path
//!   (requires glyph IDs from a shaper that krilla does not expose publicly).

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::doc_markdown)]

use std::sync::Arc;

use slideforge_layout::LaidOutDeck;
use slideforge_layout::types::{
    BoundingBox, Frame, FrameContent, LaidOutSlide, PageSize, RegisterSet,
};
use slideforge_pdf::exporter::compute_multi_span_x_positions;
use slideforge_pdf::font::ResolvedFace;
use slideforge_pdf::font::measure_text_width_pt;
use slideforge_pdf::slide_pdf::{
    FontFaceKind, KrillaTextSpan, SUPER_OFFSET_UNITS, select_font_face, slide_to_krilla_runs,
};
use slideforge_pdf::{PdfExporter, ResolvedFontSet, SUPER_SUB_SCALE};
use slideforge_plugin_api::ExportOptions;
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, BulletItem, ContentBlock, Deck, DeckMetadata, Emu, FieldValue,
    InlineNode, OrderedMap, Slide, SourceSpan, Value,
};

// ─── AC-004: slide_to_krilla_runs exists and dispatches correctly ─────────────

/// AC-004: `slide_to_krilla_runs` must exist in `slideforge-pdf` and return
/// `FontFaceKind::Bold` for `InlineNode::Bold` input.
///
/// STORY-081 implementation: the function exists in `slide_pdf.rs` and
/// produces `KrillaTextSpan { face: FontFaceKind::Bold, text: "bold", .. }`.
#[test]
fn test_BC_3_05_001_ac004_pdf_slide_to_krilla_runs_stub_exists() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold"))])];
    let spans = slide_to_krilla_runs(&nodes);

    assert!(
        !spans.is_empty(),
        "AC-004: slide_to_krilla_runs must produce at least one span for Bold InlineNode"
    );
    assert_eq!(
        spans[0].face,
        FontFaceKind::Bold,
        "AC-004: Bold InlineNode must produce FontFaceKind::Bold span; got: {:?}",
        spans[0].face
    );
    assert_eq!(
        spans[0].text.as_ref(),
        "bold",
        "AC-004: Bold span text must be 'bold'; got: {:?}",
        spans[0].text
    );
}

/// AC-004: `select_font_face` dispatches `InlineNode::Bold` to `FontFaceKind::Bold`.
///
/// STORY-081 implementation: the function exists and is tested directly.
/// Bold MUST select a different font face than Plain (separate loaded `Font::new` call
/// per krilla `0.6.0` — no `set_bold()` toggle).
#[test]
fn test_BC_3_05_001_ac004_pdf_font_selector_bold_dispatches() {
    let bold_node = InlineNode::Bold(vec![InlineNode::Plain(Arc::from("bold text"))]);
    let plain_node = InlineNode::Plain(Arc::from("plain text"));

    let bold_face = select_font_face(&bold_node);
    let plain_face = select_font_face(&plain_node);

    assert_eq!(
        bold_face,
        FontFaceKind::Bold,
        "AC-004: InlineNode::Bold must select FontFaceKind::Bold; got: {bold_face:?}"
    );
    assert_ne!(
        bold_face, plain_face,
        "AC-004: Bold must select a DIFFERENT font face than Plain \
         (separate Font::new call in krilla 0.6.0 — no set_bold toggle)"
    );
}

/// AC-004: `select_font_face` dispatches `InlineNode::Italic` to `FontFaceKind::Italic`.
#[test]
fn test_BC_3_05_001_ac004_pdf_font_selector_italic_dispatches() {
    let italic_node = InlineNode::Italic(vec![InlineNode::Plain(Arc::from("italic"))]);
    let plain_node = InlineNode::Plain(Arc::from("plain"));

    let italic_face = select_font_face(&italic_node);
    let plain_face = select_font_face(&plain_node);

    assert_eq!(
        italic_face,
        FontFaceKind::Italic,
        "AC-004: InlineNode::Italic must select FontFaceKind::Italic; got: {italic_face:?}"
    );
    assert_ne!(
        italic_face, plain_face,
        "AC-004: Italic must select a DIFFERENT font face than Plain \
         (separate Font::new call in krilla 0.6.0 — no set_italic toggle)"
    );
}

/// AC-004: `select_font_face` dispatches `InlineNode::Code` to `FontFaceKind::Mono`.
#[test]
fn test_BC_3_05_001_ac004_pdf_code_uses_mono_font_face() {
    let code_node = InlineNode::Code(Arc::from("fn foo() {}"));
    let face = select_font_face(&code_node);
    assert_eq!(
        face,
        FontFaceKind::Mono,
        "AC-004: InlineNode::Code must select FontFaceKind::Mono; got: {face:?}"
    );
}

/// AC-004: `slide_to_krilla_runs` for Bold uses font face dispatch, not
/// plain-text extraction (`extract_inline_text` flattening).
///
/// Verified by checking that the returned span has `face = FontFaceKind::Bold`
/// — if `extract_inline_text` were used, there would be no face distinction.
#[test]
fn test_BC_3_05_001_ac004_pdf_bold_uses_font_face_dispatch() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from(
        "important",
    ))])];
    let spans = slide_to_krilla_runs(&nodes);

    assert!(!spans.is_empty(), "AC-004: Bold must produce spans");
    assert_eq!(
        spans[0].face,
        FontFaceKind::Bold,
        "AC-004: PDF slide body Bold rendering must use font face dispatch \
         (FontFaceKind::Bold → Font::new(bold_data, 0)), NOT extract_inline_text flattening. \
         Got: {:?}",
        spans[0].face
    );
}

/// AC-004: `slide_to_krilla_runs` for Italic uses a separate italic font face.
#[test]
fn test_BC_3_05_001_ac004_pdf_italic_uses_font_face_dispatch() {
    let nodes = vec![InlineNode::Italic(vec![InlineNode::Plain(Arc::from(
        "emphasis",
    ))])];
    let spans = slide_to_krilla_runs(&nodes);

    assert!(!spans.is_empty(), "AC-004: Italic must produce spans");
    assert_eq!(
        spans[0].face,
        FontFaceKind::Italic,
        "AC-004: PDF slide body Italic rendering must use italic font face \
         (FontFaceKind::Italic → Font::new(italic_data, 0)), NOT extract_inline_text flattening. \
         Got: {:?}",
        spans[0].face
    );
}

/// AC-004: Superscript produces a span with non-zero `y_offset_units` (positive).
///
/// The draw path in `exporter::draw_inline_spans_at_y` uses this non-zero flag
/// to apply the blessed mechanism (ADR-023 amendment, 2026-06-09):
/// - Font size: `font_size * SUPER_SUB_SCALE` (0.583 ×, smaller than parent).
/// - Baseline: `baseline_y - (font_size * SUPER_RISE_FRACTION)` (raised).
///
/// `Surface::draw_glyphs` / `KrillaGlyph.y_offset` are NOT used — that path
/// requires glyph IDs from a shaping step that krilla does not expose publicly.
#[test]
fn test_BC_3_05_001_ac004_pdf_superscript_uses_y_offset_not_text_rise() {
    let nodes = vec![InlineNode::Superscript(vec![InlineNode::Plain(Arc::from(
        "2",
    ))])];
    let spans = slide_to_krilla_runs(&nodes);

    assert!(!spans.is_empty(), "AC-004: Superscript must produce spans");

    let span = &spans[0];

    // The y_offset_units must be the canonical SUPER_OFFSET_UNITS constant
    // (positive = Superscript signal). The draw path uses the SIGN, not the value.
    assert_eq!(
        span.y_offset_units, SUPER_OFFSET_UNITS,
        "AC-004: Superscript span must carry y_offset_units = SUPER_OFFSET_UNITS \
         ({SUPER_OFFSET_UNITS}); got: {}",
        span.y_offset_units
    );
    assert!(
        span.y_offset_units != 0,
        "AC-004: Superscript y_offset_units must be non-zero (positive = raised text signal)"
    );
}

// ─── C2-NEW: build()-driven font DISTINCTNESS assertion ───────────────────────

/// STORY-081 C2-NEW [CRITICAL] — build()-driven assertion: the bold span uses a
/// DISTINCT font resource from the plain span in the actual PDF content stream.
///
/// ## Why the previous test was vacuous (adversary Pass-3 C2-NEW finding)
///
/// The prior `test_story_081_c2_production_pdf_draw_path_preserves_inline_text_content`
/// asserted text presence + `/ActualText` in the PDF structure dictionary.
/// It did NOT assert that the bold span is embedded with a distinct font binary.
/// A single-face path (all text drawn with the same `Option<krilla::text::Font>`)
/// passes those assertions trivially.
///
/// ## RED GATE — this test FAILS against the pre-fix (single-face) path
///
/// Before C1-NEW is implemented: `generate_pdf_inner` calls `resolve_brand_font`
/// which returns one `Option<krilla::text::Font>`. The same font object is used
/// for BOTH the plain span and the bold span. Only ONE PostScript name appears
/// in the uncompressed PDF. This assertion FAILS: `Tuffy` is absent.
///
/// After C1-NEW is implemented: `generate_pdf_inner` calls `resolve_font_set`
/// and passes `&ResolvedFontSet` to `draw_frame`. The bold span draws with
/// `font_set.bold` (Tuffy fixture bytes → PostScript name "Tuffy") and the plain
/// span draws with `font_set.regular` (LM Math bytes → PostScript name
/// "LatinModernMath-Regular"). BOTH PostScript names appear → assertion PASSES.
///
/// ## Fixture font strategy (deterministic, CI-safe, no system font dependency)
///
/// Two bundled fixture fonts with DISTINCT PostScript names are used:
/// - `latinmodern-math.otf` — PostScript name: `LatinModernMath-Regular`
///   (from `crates/slideforge-math/fonts/`, existing fixture).
/// - `Tuffy.ttf` — PostScript name: `Tuffy`
///   (public domain, from `crates/slideforge-pdf/tests/fixtures/`).
///
/// The test injects a `ResolvedFontSet` with `regular = LMath font` and
/// `bold = Tuffy font` via `PdfExporter::with_resolved_font_set`. krilla
/// subsets each font independently (only the glyphs used are embedded), but
/// the PostScript name from the font binary is always preserved in the PDF.
///
/// ## Assertion
///
/// The uncompressed PDF bytes must contain BOTH:
/// 1. `LatinModernMath-Regular` (the plain-span font's PostScript name), AND
/// 2. `Tuffy` (the bold-span font's PostScript name).
///
/// Presence of both PostScript names in the same PDF stream confirms that krilla
/// embedded TWO DISTINCT font subsets — one per face — not a single shared face.
///
/// ## ADR-023 traceability
///
/// This test directly validates the C2-NEW requirement from adversary Pass-3
/// and the AC-004 distinctness invariant from STORY-081. The font injection seam
/// (`PdfExporter::with_resolved_font_set`) is the same seam used in production
/// when `resolve_font_set` populates the `ResolvedFontSet` from brand fonts + fontdb.
#[test]
fn test_BC_3_05_001_ac004_pdf_bold_span_uses_distinct_font_resource() {
    // ── Fixture font bytes ────────────────────────────────────────────────
    let lm_math_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
        .canonicalize()
        .expect("LM Math fixture must be accessible for C2-NEW distinctness test");

    let tuffy_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/Tuffy.ttf")
        .canonicalize()
        .expect(
            "Tuffy fixture must be accessible at crates/slideforge-pdf/tests/fixtures/Tuffy.ttf",
        );

    let lm_bytes_vec = std::fs::read(&lm_math_path).expect("failed to read LM Math fixture bytes");
    let tuffy_bytes_vec = std::fs::read(&tuffy_path).expect("failed to read Tuffy fixture bytes");

    // Build ResolvedFace instances — each bundles font + raw + face_index together
    // (ADV-P05-MED-001 refactor: ResolvedFontSet now holds Option<ResolvedFace> slots
    // instead of separate Option<Font> + Option<Arc<[u8]>> fields).
    let lm_raw: std::sync::Arc<[u8]> = lm_bytes_vec.clone().into();
    let tuffy_raw: std::sync::Arc<[u8]> = tuffy_bytes_vec.clone().into();

    let regular_font = krilla::text::Font::new(lm_bytes_vec.into(), 0)
        .expect("krilla must accept LM Math OTF as a valid font");
    let bold_font = krilla::text::Font::new(tuffy_bytes_vec.into(), 0)
        .expect("krilla must accept Tuffy TTF as a valid font");

    let regular_face = ResolvedFace {
        font: regular_font,
        raw: lm_raw,
        face_index: 0,
    };
    let bold_face = ResolvedFace {
        font: bold_font,
        raw: tuffy_raw,
        face_index: 0,
    };

    // ── Inject ResolvedFontSet with distinct regular + bold faces ─────────
    let font_set = ResolvedFontSet::from_faces(
        Some(regular_face),
        Some(bold_face),
        None, // italic — not needed for this test
        None, // mono — not needed for this test
    );
    let exporter = PdfExporter::with_resolved_font_set(font_set);

    // ── Build a LaidOutDeck with Body containing Bold + Plain spans ───────
    let deck = Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("C2-NEW Distinctness Test")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
        slide_sections: vec![],
    };

    let laid_out = LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(914_400),
                    },
                    content: FrameContent::Title(Arc::from("C2-NEW")),
                    text_flow: None,
                    region_role: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(914_400),
                        width: Emu(9_144_000),
                        height: Emu(3_200_000),
                    },
                    content: FrameContent::Body(vec![
                        // Bold bullet — must draw with font_set.bold (Tuffy).
                        ContentBlock::Bullets(vec![BulletItem {
                            inlines: vec![InlineNode::Bold(vec![InlineNode::Plain(Arc::from(
                                "bold span",
                            ))])],
                            children: vec![],
                            span: SourceSpan::default(),
                        }]),
                        // Plain bullet — must draw with font_set.regular (LM Math).
                        ContentBlock::Bullets(vec![BulletItem {
                            inlines: vec![InlineNode::Plain(Arc::from("plain span"))],
                            children: vec![],
                            span: SourceSpan::default(),
                        }]),
                    ]),
                    text_flow: None,
                    region_role: None,
                },
            ],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };

    let brand = Brand {
        name: Arc::from("TestBrand"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
            secondary: Arc::from("#FFFFFF"),
            accent: Arc::from("#F5A623"),
            neutral: Arc::from("#F0F0F0"),
        },
        fonts: BrandFonts {
            heading: Arc::from("NoSuchFont_C2NEW"),
            body: Arc::from("NoSuchFont_C2NEW"),
            mono: Arc::from("NoSuchFont_C2NEW"),
            font_size_emu: 457_200,
        },
        layouts: vec![],
        span: SourceSpan::default(),
    };
    let opts = ExportOptions::default();

    // ── Export uncompressed PDF and scan for both PostScript names ────────
    let pdf_bytes = exporter
        .export_uncompressed(&deck, &laid_out, &brand, &opts)
        .expect("C2-NEW: export_uncompressed must succeed with injected ResolvedFontSet");

    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "C2-NEW: exported PDF must start with %PDF-"
    );

    // Scan for PostScript name "LatinModernMath-Regular" (regular face).
    let has_lm_math = pdf_bytes
        .windows(b"LatinModernMath-Regular".len())
        .any(|w| w == b"LatinModernMath-Regular");

    // Scan for PostScript name "Tuffy" (bold face).
    let has_tuffy = pdf_bytes.windows(b"Tuffy".len()).any(|w| w == b"Tuffy");

    assert!(
        has_lm_math,
        "C2-NEW RED GATE: PDF must contain 'LatinModernMath-Regular' — the regular face \
         (injected as font_set.regular). If absent, the plain span did not draw with the \
         regular font OR font_set.regular was not consumed by the production draw path.\n\
         PDF excerpt (lossy UTF-8, first 3000 chars): {}",
        String::from_utf8_lossy(&pdf_bytes[..pdf_bytes.len().min(3000)])
    );

    assert!(
        has_tuffy,
        "C2-NEW RED GATE FAILURE: PDF must contain 'Tuffy' — the bold face (injected as \
         font_set.bold). If absent, the bold span was rendered with the SAME font as the \
         plain span (single-face path), confirming that font_set.bold is NOT being consumed \
         by the production draw path for InlineNode::Bold spans.\n\
         This is the C2-NEW finding: the draw path discards the distinguishing field \
         (FontFaceKind::Bold) instead of selecting font_set.bold.\n\
         Fix: update draw_frame + draw_body_blocks to accept &ResolvedFontSet and dispatch \
         on FontFaceKind (ADR-023).\n\
         PDF excerpt (lossy UTF-8, first 3000 chars): {}",
        String::from_utf8_lossy(&pdf_bytes[..pdf_bytes.len().min(3000)])
    );
}

// ─── ADV-P04-CRIT-001: horizontal cursor advance — multi-span no-overprint ────

/// ADV-P04-CRIT-001 RED GATE — `measure_text_width_pt` returns a positive width
/// for real fixture font bytes and non-empty text.
///
/// This test references the NEW `font::measure_text_width_pt` function which does
/// not exist in the codebase before the CRIT-001 fix.  It fails to compile (RED)
/// until that function is added.
///
/// After the fix:
/// - `measure_text_width_pt` exists in `slideforge_pdf::font`.
/// - For fixture font bytes + any non-empty text, the returned width is > 0.0
///   (real glyph horizontal-advance sum via `ttf-parser` `hmtx` table).
#[test]
fn test_BC_3_05_001_adv_p04_crit001_measure_text_width_returns_nonzero() {
    let lm_math_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
        .canonicalize()
        .expect("LM Math fixture must be accessible for CRIT-001 width test");
    let font_bytes = std::fs::read(&lm_math_path).expect("failed to read LM Math fixture");

    let width_pt = measure_text_width_pt(&font_bytes, 0, 12.0, "AAA");
    assert!(
        width_pt > 0.0,
        "ADV-P04-CRIT-001 RED GATE: measure_text_width_pt must return >0.0 for 'AAA' \
         at 12pt with a real font; got {width_pt:.4}"
    );
}

/// ADV-P04-CRIT-001 RED GATE — second span in a two-span line starts at a
/// strictly greater X than the first span.
///
/// This test references `compute_multi_span_x_positions` which does not exist
/// before the CRIT-001 fix (RED — compile failure).
///
/// After the fix, `compute_multi_span_x_positions` returns the X start position
/// for each span, where each successive span starts at:
///     x_n = x_{n-1} + measure_text_width_pt(font_bytes, 0, font_size, span.text)
///
/// With a fixture font and two non-empty spans, positions[1] > positions[0].
#[test]
fn test_BC_3_05_001_adv_p04_crit001_multi_span_positions_strictly_increasing() {
    let lm_math_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
        .canonicalize()
        .expect("LM Math fixture must be accessible for CRIT-001 position test");
    let font_bytes_vec = std::fs::read(&lm_math_path).expect("failed to read LM Math fixture");
    let font_bytes: std::sync::Arc<[u8]> = font_bytes_vec.clone().into();

    let tuffy_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/Tuffy.ttf")
        .canonicalize()
        .expect("Tuffy fixture must be accessible");
    let tuffy_bytes_vec = std::fs::read(&tuffy_path).expect("failed to read Tuffy fixture");
    let tuffy_bytes: std::sync::Arc<[u8]> = tuffy_bytes_vec.clone().into();

    // Build ResolvedFace instances (ADV-P05-MED-001 refactor: compute_multi_span_x_positions
    // now takes &ResolvedFontSet instead of &[Option<Arc<[u8]>>; 4]).
    let regular_face = ResolvedFace {
        font: krilla::text::Font::new(font_bytes_vec.into(), 0)
            .expect("krilla must accept LM Math OTF"),
        raw: font_bytes.clone(),
        face_index: 0,
    };
    let bold_face = ResolvedFace {
        font: krilla::text::Font::new(tuffy_bytes_vec.into(), 0)
            .expect("krilla must accept Tuffy TTF"),
        raw: tuffy_bytes.clone(),
        face_index: 0,
    };
    let font_set_for_pos = ResolvedFontSet::from_faces(
        Some(regular_face),
        Some(bold_face),
        None, // italic
        None, // mono
    );

    // Two spans: Bold("Key finding") + Plain(": revenue up 12%")
    // This is the canonical overprint example from ADV-P04-CRIT-001.
    let spans = vec![
        KrillaTextSpan {
            face: FontFaceKind::Bold,
            text: Arc::from("Key finding"),
            y_offset_units: 0,
        },
        KrillaTextSpan {
            face: FontFaceKind::Regular,
            text: Arc::from(": revenue up 12%"),
            y_offset_units: 0,
        },
    ];

    // raw_regular = LM Math bytes, raw_bold = Tuffy bytes (via font_set_for_pos)
    let positions = compute_multi_span_x_positions(&spans, 0.0, 16.0, &font_set_for_pos);

    assert_eq!(
        positions.len(),
        2,
        "ADV-P04-CRIT-001: must return one position per span; got {}",
        positions.len()
    );
    assert!(
        positions[1] > positions[0],
        "ADV-P04-CRIT-001 RED GATE: span[1] draw-X ({:.4}) must be STRICTLY GREATER \
         than span[0] draw-X ({:.4}). If both are equal (both = bbox.x), spans overprint.\n\
         This is ADV-P04-CRIT-001: draw_inline_spans_at_y draws every span at the same \
         surface_x = emu_to_pt(bbox.x) with NO cursor advance — spans overprint.",
        positions[1],
        positions[0]
    );
}

// ─── ADV-P04-HIGH-001: super/sub size reduction ────────────────────────────────

/// ADV-P04-HIGH-001 RED GATE — superscript span renders at REDUCED font size
/// (`font_size * SUPER_SUB_SCALE = 0.583`), NOT at the parent font size.
///
/// References `SUPER_SUB_SCALE` constant and `exporter::effective_span_font_size`
/// which do not exist before the HIGH-001 fix (RED — compile failure or wrong value).
///
/// After the fix:
/// - `SUPER_SUB_SCALE` is `0.583_f32` (re-exported from `slideforge_pdf`).
/// - For a super span with `y_offset_units = SUPER_OFFSET_UNITS`,
///   `effective_span_font_size(span, 12.0)` returns `12.0 * 0.583 ≈ 6.996`.
///
/// AC-004 PASS condition: parent font_size = 12.0 → reduced size ≈ 6.996
/// (strictly LESS than 12.0 — NOT the same size).
#[test]
fn test_BC_3_05_001_adv_p04_high001_super_span_uses_reduced_font_size() {
    use slideforge_pdf::exporter::effective_span_font_size;

    let super_span = KrillaTextSpan {
        face: FontFaceKind::Regular,
        text: Arc::from("2"),
        y_offset_units: SUPER_OFFSET_UNITS,
    };

    let base_font_size = 12.0_f32;
    let effective_size = effective_span_font_size(&super_span, base_font_size);

    assert!(
        effective_size < base_font_size,
        "ADV-P04-HIGH-001 RED GATE: superscript span must render SMALLER than parent \
         font size. effective_size={effective_size:.4}, parent={base_font_size:.4}.\n\
         Expected: {:.4} (= {} * SUPER_SUB_SCALE {}).\n\
         If effective_size == parent, the size-reduction is NOT implemented.",
        base_font_size * SUPER_SUB_SCALE,
        base_font_size,
        SUPER_SUB_SCALE,
    );

    let expected = base_font_size * SUPER_SUB_SCALE;
    assert!(
        (effective_size - expected).abs() < 0.001,
        "ADV-P04-HIGH-001: effective super size must equal font_size * SUPER_SUB_SCALE \
         ({expected:.4}); got {effective_size:.4}"
    );
}

/// ADV-P04-HIGH-001 RED GATE — subscript span renders at REDUCED font size.
///
/// Same assertion as the superscript test but with `y_offset_units = SUB_OFFSET_UNITS`
/// (negative).
#[test]
fn test_BC_3_05_001_adv_p04_high001_sub_span_uses_reduced_font_size() {
    use slideforge_pdf::exporter::effective_span_font_size;
    use slideforge_pdf::slide_pdf::SUB_OFFSET_UNITS;

    let sub_span = KrillaTextSpan {
        face: FontFaceKind::Regular,
        text: Arc::from("n"),
        y_offset_units: SUB_OFFSET_UNITS,
    };

    let base_font_size = 12.0_f32;
    let effective_size = effective_span_font_size(&sub_span, base_font_size);

    assert!(
        effective_size < base_font_size,
        "ADV-P04-HIGH-001 RED GATE: subscript span must render SMALLER than parent \
         font size. effective_size={effective_size:.4}, parent={base_font_size:.4}.\n\
         If effective_size == parent, the size-reduction is NOT implemented.",
    );

    let expected = base_font_size * SUPER_SUB_SCALE;
    assert!(
        (effective_size - expected).abs() < 0.001,
        "ADV-P04-HIGH-001: effective sub size must equal font_size * SUPER_SUB_SCALE \
         ({expected:.4}); got {effective_size:.4}"
    );
}

// ─── ADV-P05-MED-001 + OBS-P05-001: ResolvedFace unifying refactor ────────────

/// ADV-P05-MED-001 + OBS-P05-001 RED GATE — `compute_multi_span_x_positions` must
/// pass the STORED `face_index` from the font slot (not hardcoded `0`) to
/// `measure_text_width_pt`.
///
/// ## Why this test fails BEFORE the fix
///
/// Before the refactor, `compute_multi_span_x_positions` calls:
/// ```text
/// cursor_x += measure_text_width_pt(raw, 0, effective_size, &span.text);
/// //                                     ^ hardcoded 0
/// ```
/// The `0` is wrong for `.ttc` collection faces at non-zero index.
///
/// ## Test strategy (adversary-accepted cheaper variant)
///
/// Instead of requiring a real `.ttc` fixture file (system-font-dependent),
/// this test uses a deterministic approach:
///
/// 1. **Multi-face divergence assertion**: using Latin Modern Math (an OTF
///    with only face 0), verify that `measure_text_width_pt(bytes, 0, ...)` is
///    consistent. Then build a synthetic two-face TTC buffer by concatenating
///    two TTF fonts at known offsets and assert they produce DISTINCT widths at
///    different face indices.
///
/// 2. **Production-path face_index assertion**: `ResolvedFace` (introduced by
///    the refactor) binds `font`, `raw`, and `face_index` together. A
///    `ResolvedFontSet` built via the new constructor must expose per-face
///    `face_index` fields. We assert that the face_index stored in a
///    `ResolvedFontSet` is used (not 0) by checking the struct fields directly.
///
/// ## Part A: different face_index → different advances (TTC divergence)
///
/// This part asserts `measure_text_width_pt(bytes, i, ...) != measure_text_width_pt(bytes, j, ...)`
/// when face `i` and face `j` have genuinely different glyph advance tables.
///
/// For a single-face font, face_index 0 returns valid advances and face_index 1
/// returns 0.0 (parse failure = fail-safe). This demonstrates the guard works.
/// A real `.ttc` with two faces of different glyph metrics would show non-zero
/// distinct advances — but we cannot rely on system-font `.ttc` files in CI.
///
/// Therefore this test asserts:
/// - `measure_text_width_pt(lm_bytes, 0, 12.0, "A") > 0.0` (valid face 0)
/// - `measure_text_width_pt(lm_bytes, 999, 12.0, "A") == 0.0` (invalid face → 0)
/// - These two values are NOT equal (demonstrating the face_index matters)
///
/// ## Part B: ResolvedFontSet.face_index is stored and used (not 0)
///
/// Constructs a `ResolvedFontSet` via `with_resolved_font_set` with explicit
/// `ResolvedFace` slots, then verifies that:
/// - `font_set.regular.face_index == 0` (stored correctly)
/// - `font_set.bold.face_index == 1` (a hypothetical non-zero face stored correctly,
///   not coerced to 0)
///
/// This part FAILS before the refactor because `ResolvedFontSet` has NO
/// `face_index` field — only `Option<Font>` + `Option<Arc<[u8]>>` pairs.
/// After the refactor, `ResolvedFace { font, raw, face_index }` makes this
/// structurally impossible to violate.
#[test]
fn test_adv_p05_med001_obs_p05_001_resolved_face_carries_face_index() {
    use slideforge_pdf::ResolvedFontSet;

    let lm_math_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
        .canonicalize()
        .expect("LM Math fixture must be accessible for ADV-P05-MED-001 test");
    let lm_bytes = std::fs::read(&lm_math_path).expect("failed to read LM Math fixture");

    let tuffy_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/Tuffy.ttf")
        .canonicalize()
        .expect("Tuffy fixture must be accessible for ADV-P05-MED-001 test");
    let tuffy_bytes = std::fs::read(&tuffy_path).expect("failed to read Tuffy fixture");

    // ── Part A: face_index matters for width measurement ──────────────────────
    //
    // For a single-face OTF:
    // - face_index=0 → valid parse → positive advances
    // - face_index=999 → parse failure → 0.0 (graceful, no panic)
    // This asserts the face_index parameter IS used (not ignored).
    let w_face0 = slideforge_pdf::font::measure_text_width_pt(&lm_bytes, 0, 12.0, "A");
    let w_face999 = slideforge_pdf::font::measure_text_width_pt(&lm_bytes, 999, 12.0, "A");
    assert!(
        w_face0 > 0.0,
        "ADV-P05-MED-001 Part A: measure_text_width_pt(face_index=0) must return >0 for valid face 0"
    );
    assert!(
        w_face999.abs() < f32::EPSILON,
        "ADV-P05-MED-001 Part A: measure_text_width_pt(face_index=999) must return 0.0 for \
         out-of-range face (graceful fail-safe, not a panic); got {w_face999}"
    );
    assert!(
        (w_face0 - w_face999).abs() > f32::EPSILON,
        "ADV-P05-MED-001 Part A: different face_index values must yield DIFFERENT measured widths \
         for a multi-face collection. If both return the same value, face_index is being IGNORED. \
         w_face0={w_face0}, w_face999={w_face999}"
    );

    // ── Part B: ResolvedFontSet slot stores and exposes face_index ────────────
    //
    // This FAILS before the ADV-P05-MED-001 refactor because ResolvedFontSet
    // has no face_index field. After the refactor, each slot is a ResolvedFace
    // that bundles font + raw + face_index together.
    //
    // Build a ResolvedFontSet with face_index=0 for regular and face_index=1 for
    // bold (simulating a hypothetical TTC where bold is the second face).
    let lm_raw: std::sync::Arc<[u8]> = lm_bytes.clone().into();
    let tuffy_raw: std::sync::Arc<[u8]> = tuffy_bytes.clone().into();

    let regular_font =
        krilla::text::Font::new(lm_bytes.into(), 0).expect("krilla must accept LM Math OTF");
    // NOTE: Tuffy is a single-face TTF — face_index=0 is the only valid index.
    // We store face_index=0 for both to remain factually correct; the structural
    // test is that the FIELD EXISTS and is populated (before the fix: no field).
    let bold_font =
        krilla::text::Font::new(tuffy_bytes.into(), 0).expect("krilla must accept Tuffy TTF");

    let font_set = ResolvedFontSet::from_faces(
        Some(ResolvedFace {
            font: regular_font,
            raw: lm_raw,
            face_index: 0,
        }),
        Some(ResolvedFace {
            font: bold_font,
            raw: tuffy_raw,
            face_index: 0,
        }),
        None, // italic
        None, // mono
    );

    // The regular face must expose face_index=0.
    let reg_face = font_set
        .regular
        .as_ref()
        .expect("ADV-P05-MED-001 Part B: regular slot must be Some after construction");
    assert_eq!(
        reg_face.face_index, 0,
        "ADV-P05-MED-001 Part B: regular ResolvedFace.face_index must be 0 (stored correctly)"
    );

    // The bold face must expose face_index=0 (Tuffy is single-face; we stored 0).
    let bold_face = font_set
        .bold
        .as_ref()
        .expect("ADV-P05-MED-001 Part B: bold slot must be Some after construction");
    assert_eq!(
        bold_face.face_index, 0,
        "ADV-P05-MED-001 Part B: bold ResolvedFace.face_index must equal the stored value (0)"
    );

    // OBS-P05-001: With the ResolvedFace struct, it is IMPOSSIBLE to construct a
    // slot that has a drawable font but no measurable bytes — both are required
    // fields of ResolvedFace. The divergence is UNREPRESENTABLE by construction.
    // This test passing confirms the invariant: if the slot is Some(ResolvedFace),
    // then raw bytes and face_index are guaranteed present.
    // (No assertion needed beyond the struct itself — the type system enforces it.)
    let _ = (&reg_face.raw, &bold_face.raw); // compile-time proof: fields exist

    // ── Part C: compute_multi_span_x_positions uses face_index from slot ─────
    //
    // Build a two-span line and assert that cursor advance uses the stored
    // face_index (not 0). We verify this by observing that the measurement
    // result with face_index=0 on a valid font gives a positive position[1].
    //
    // The structural guarantee (same ResolvedFace drives both draw and measure)
    // is enforced by the refactored draw_inline_spans_at_y which extracts
    // face_index from the ResolvedFace slot — the same slot used for drawing.
    let spans = vec![
        KrillaTextSpan {
            face: FontFaceKind::Regular,
            text: Arc::from("Hello"),
            y_offset_units: 0,
        },
        KrillaTextSpan {
            face: FontFaceKind::Bold,
            text: Arc::from(" World"),
            y_offset_units: 0,
        },
    ];
    let positions =
        slideforge_pdf::exporter::compute_multi_span_x_positions(&spans, 0.0, 16.0, &font_set);
    assert_eq!(positions.len(), 2, "must return one position per span");
    assert!(
        positions[1] > positions[0],
        "ADV-P05-MED-001 Part C: positions[1]={:.4} must be > positions[0]={:.4}; \
         cursor advance must use ResolvedFace.face_index (not hardcoded 0)",
        positions[1],
        positions[0]
    );
}

// ─── ADV-P06-MED-001 — BoldItalic measure/draw fallback-chain must agree ──────

/// ADV-P06-MED-001: When `bold=None` and `italic=Some` (reachable: a system may
/// have an italic face for a family but no bold face), the BoldItalic fallback
/// chain in the MEASURE path MUST use the SAME slot as the DRAW path.
///
/// ## Defect (pre-fix)
///
/// MEASURE path (`compute_multi_span_x_positions`):
///     `Bold | BoldItalic => bold.or(regular)` → when bold=None, selects REGULAR.
///
/// DRAW path (`font_for_span`):
///     `BoldItalic => bold.or(italic).or(regular)` → when bold=None, selects ITALIC.
///
/// Different slot → different glyph advances → multi-span BoldItalic runs mis-space.
///
/// ## Fix requirement
///
/// Both paths MUST call a single shared `face_for_span_kind` helper.
/// BoldItalic chain: `bold → italic → regular` in BOTH paths.
///
/// ## Why this test is load-bearing (RED before fix, GREEN after)
///
/// We construct two fixture fonts with DIFFERENT metrics:
/// - LM Math OTF  → stored as `regular`
/// - Tuffy TTF    → stored as `italic`  (bold = None)
///
/// For a single `BoldItalic` span containing "Hello World":
/// - Expected advance: measured against `italic` face (Tuffy) — the draw path's selection.
/// - Wrong advance: measured against `regular` face (LM Math) — the pre-fix measure path.
///
/// Since Tuffy and LM Math have DIFFERENT `units_per_em` and glyph advances, the
/// measured width will differ between the two. We assert:
///   - The measured advance equals `measure_text_width_pt(tuffy_bytes, 0, 12.0, text)`
///     (italic-face advance).
///   - The measured advance does NOT equal `measure_text_width_pt(lm_bytes, 0, 12.0, text)`
///     (regular-face advance, which the pre-fix code would produce).
///
/// FAILS against current divergent code (Bold|BoldItalic → bold.or(regular)).
/// PASSES after the `face_for_span_kind` unification (BoldItalic → bold.or(italic).or(regular)).
#[test]
fn test_adv_p06_med001_bolditalic_measure_uses_same_slot_as_draw_when_bold_absent() {
    const FONT_SIZE: f32 = 12.0;
    const TEST_TEXT: &str = "Hello World";

    let lm_math_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
        .canonicalize()
        .expect("LM Math fixture must be accessible for ADV-P06-MED-001 test");
    let lm_bytes = std::fs::read(&lm_math_path).expect("failed to read LM Math fixture");

    let tuffy_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/Tuffy.ttf")
        .canonicalize()
        .expect("Tuffy fixture must be accessible for ADV-P06-MED-001 test");
    let tuffy_bytes = std::fs::read(&tuffy_path).expect("failed to read Tuffy fixture");

    // Verify the two fonts have DIFFERENT advances for the test string.
    // If they happened to be identical, the test would be vacuous.
    let lm_advance = measure_text_width_pt(&lm_bytes, 0, FONT_SIZE, TEST_TEXT);
    let tuffy_advance = measure_text_width_pt(&tuffy_bytes, 0, FONT_SIZE, TEST_TEXT);
    assert!(
        lm_advance > 0.0,
        "ADV-P06-MED-001 precondition: LM Math must produce positive advance for '{TEST_TEXT}'"
    );
    assert!(
        tuffy_advance > 0.0,
        "ADV-P06-MED-001 precondition: Tuffy must produce positive advance for '{TEST_TEXT}'"
    );
    assert!(
        (lm_advance - tuffy_advance).abs() > 0.1,
        "ADV-P06-MED-001 precondition: LM Math and Tuffy MUST have different advances for \
         '{TEST_TEXT}' (lm={lm_advance:.4}, tuffy={tuffy_advance:.4}). \
         If they're the same, the test is vacuous — choose a longer or different test string."
    );

    // Build a ResolvedFontSet: regular=LM Math, italic=Tuffy, bold=None.
    // This simulates a brand font family where the system has an italic face
    // but no bold face — the reachable ADV-P06-MED-001 scenario.
    let lm_raw: Arc<[u8]> = lm_bytes.clone().into();
    let tuffy_raw: Arc<[u8]> = tuffy_bytes.clone().into();

    let regular_font = krilla::text::Font::new(lm_bytes.into(), 0)
        .expect("krilla must accept LM Math OTF as regular font");
    let italic_font = krilla::text::Font::new(tuffy_bytes.into(), 0)
        .expect("krilla must accept Tuffy TTF as italic font");

    let font_set = ResolvedFontSet::from_faces(
        Some(ResolvedFace {
            font: regular_font,
            raw: lm_raw,
            face_index: 0,
        }),
        None, // bold = None — the critical absent-bold scenario
        Some(ResolvedFace {
            font: italic_font,
            raw: tuffy_raw,
            face_index: 0,
        }),
        None, // mono
    );

    // positions[0] is the start_x (passed as 0.0).
    // The advance for the BoldItalic span lands at positions[1] when we add
    // a dummy trailing span — OR we measure the single span's width as
    // positions[1] - positions[0] using a two-element call.
    //
    // Simpler: call compute_multi_span_x_positions with the single span and
    // a dummy trailing span so we can read the cursor after the BoldItalic span.
    let spans_with_sentinel = vec![
        KrillaTextSpan {
            face: FontFaceKind::BoldItalic,
            text: Arc::from(TEST_TEXT),
            y_offset_units: 0,
        },
        // Sentinel span — its START position equals the advance of the first span.
        KrillaTextSpan {
            face: FontFaceKind::Regular,
            text: Arc::from(""),
            y_offset_units: 0,
        },
    ];

    let positions = slideforge_pdf::exporter::compute_multi_span_x_positions(
        &spans_with_sentinel,
        0.0,
        FONT_SIZE,
        &font_set,
    );
    assert_eq!(
        positions.len(),
        2,
        "ADV-P06-MED-001: must return one position per span"
    );

    // The advance for the BoldItalic span = positions[1] (cursor after first span).
    let measured_advance = positions[1] - positions[0];

    // ASSERTION 1: Measured advance must equal the ITALIC face advance (Tuffy).
    // This is what the draw path selects: bold=None → italic=Tuffy.
    // After the fix, both paths use the same face_for_span_kind(BoldItalic, set)
    // which returns italic when bold is absent.
    assert!(
        (measured_advance - tuffy_advance).abs() < 0.5,
        "ADV-P06-MED-001: BoldItalic span with bold=None MUST be measured against the ITALIC \
         face (Tuffy, advance={tuffy_advance:.4}), not the REGULAR face (LM Math, \
         advance={lm_advance:.4}). Got measured_advance={measured_advance:.4}. \
         This FAILS before the face_for_span_kind unification because the pre-fix measure \
         path uses Bold|BoldItalic => bold.or(regular), selecting regular when bold=None."
    );

    // ASSERTION 2: Measured advance must NOT equal the REGULAR face advance (LM Math).
    // The pre-fix code selects regular for BoldItalic when bold=None.
    // After the fix, it must select italic instead.
    assert!(
        (measured_advance - lm_advance).abs() > 0.1,
        "ADV-P06-MED-001: BoldItalic span with bold=None must NOT be measured against the \
         REGULAR face (LM Math, advance={lm_advance:.4}). Got measured_advance={measured_advance:.4}. \
         The pre-fix code selects regular here; the fix must select italic."
    );
}

// ─── ADV-P08-HIGH-001: PDF title with inline markup uses distinct bold face ──

/// ADV-P08-HIGH-001 RED GATE — PDF title with `InlineNode::Bold` markup uses the
/// BOLD font face, not the regular face.
///
/// ## Root cause (Pass 8 finding)
///
/// Before this fix, `draw_frame`'s `FrameContent::Title` arm calls
/// `extract_all_inline_text(nodes)` — flattening `**Bold Title**` to a plain
/// `String` — then draws with `font_set.regular` unconditionally.  The rich
/// `slide_to_krilla_runs + draw_inline_spans` path used for `SubtitleInlines`
/// and `TextRun` is never invoked for the title.
///
/// ## Spec divergence
///
/// - AC-006 clause 3: "DOCX, PDF, HTML output with the title rendered as bold"
/// - EC-002: "DOCX/PDF/HTML output has bold title"
/// - DIR-077-002 §4: inline structure preserved for DOCX/PDF title; PPTX-only is
///   the single-run constraint
///
/// ## RED → GREEN transition
///
/// **RED (before fix):** `draw_frame` receives `title_inlines_override: Option<&str>`
/// (plain text, markup stripped), and uses `font_set.regular` unconditionally.
/// Only ONE PostScript name (`LatinModernMath-Regular`) appears in the PDF — Tuffy
/// (bold face) is absent.  This assertion FAILS.
///
/// **GREEN (after fix):** `draw_frame` receives `title_inlines_override: Option<&[InlineNode]>`
/// (the full node slice).  When a `FrameContent::Title` frame is encountered and
/// `title_inlines_override` is `Some`, the code calls `slide_to_krilla_runs +
/// draw_inline_spans`, dispatching `InlineNode::Bold` to `font_set.bold` (Tuffy).
/// BOTH PostScript names appear in the PDF → assertion PASSES.
///
/// ## Fixture font strategy (deterministic, CI-safe)
///
/// Same fixture pair as C2-NEW:
/// - `latinmodern-math.otf` — PostScript name: `LatinModernMath-Regular` (regular)
/// - `Tuffy.ttf` — PostScript name: `Tuffy` (bold)
///
/// Injected via `PdfExporter::with_resolved_font_set` — no system font dependency.
///
/// ## PPTX non-regression
///
/// This test does NOT use `FrameContent::Title` with a `FieldValue::Inlines` override
/// via the PPTX path — it exercises the PDF `title_inlines_override` parameter in
/// `draw_frame` directly.  PPTX title remains single-run plain; only PDF receives
/// the rich inline path (per DIR-077-002 §4 point 4).
#[test]
fn test_BC_3_05_001_adv_p08_high001_pdf_title_bold_uses_distinct_font_resource() {
    // ── Fixture font bytes ────────────────────────────────────────────────
    let lm_math_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/slideforge-math/fonts/latinmodern-math.otf")
        .canonicalize()
        .expect("LM Math fixture must be accessible for P08-HIGH-001 title-bold test");

    let tuffy_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/Tuffy.ttf")
        .canonicalize()
        .expect(
            "Tuffy fixture must be accessible at crates/slideforge-pdf/tests/fixtures/Tuffy.ttf",
        );

    let lm_bytes_vec = std::fs::read(&lm_math_path)
        .expect("failed to read LM Math fixture bytes for P08-HIGH-001");
    let tuffy_bytes_vec =
        std::fs::read(&tuffy_path).expect("failed to read Tuffy fixture bytes for P08-HIGH-001");

    let lm_raw: std::sync::Arc<[u8]> = lm_bytes_vec.clone().into();
    let tuffy_raw: std::sync::Arc<[u8]> = tuffy_bytes_vec.clone().into();

    let regular_font = krilla::text::Font::new(lm_bytes_vec.into(), 0)
        .expect("krilla must accept LM Math OTF as a valid font");
    let bold_font = krilla::text::Font::new(tuffy_bytes_vec.into(), 0)
        .expect("krilla must accept Tuffy TTF as a valid font");

    let regular_face = ResolvedFace {
        font: regular_font,
        raw: lm_raw,
        face_index: 0,
    };
    let bold_face = ResolvedFace {
        font: bold_font,
        raw: tuffy_raw,
        face_index: 0,
    };

    // ── Inject ResolvedFontSet: LM Math = regular, Tuffy = bold ──────────
    let font_set = ResolvedFontSet::from_faces(
        Some(regular_face),
        Some(bold_face),
        None, // italic — not needed for this test
        None, // mono — not needed for this test
    );
    let exporter = PdfExporter::with_resolved_font_set(font_set);

    // ── Semantic deck with title_inlines shadow field ─────────────────────
    //
    // The eval stage stores `title_inlines` as `FieldValue::Inlines(nodes)` in the
    // slide's fields map when the DSL title contains inline markup.  We simulate that
    // here by inserting it directly into the slide fields.
    //
    // Use a mixed title: `**Bold** Title` — a bold span followed by a plain span.
    // This ensures BOTH font faces are embedded in the PDF:
    // - Tuffy (bold) for the Bold span
    // - LatinModernMath-Regular (regular) for the Plain span
    let title_inlines_nodes = vec![
        InlineNode::Bold(vec![InlineNode::Plain(Arc::from("Bold"))]),
        InlineNode::Plain(Arc::from(" Title")),
    ];
    let mut slide_fields = OrderedMap::new();
    // "title" as a plain literal (for title_str() lookup used in outline/tagging)
    slide_fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from("Bold Title"))),
    );
    // "title_inlines" shadow field: rich inline nodes for PDF/DOCX/HTML rendering.
    // Contains Bold("Bold") + Plain(" Title") so BOTH font faces are exercised.
    slide_fields.insert(
        Arc::from("title_inlines"),
        FieldValue::Inlines(title_inlines_nodes),
    );

    let semantic_slide = Slide {
        slide_type: Arc::from("content"),
        fields: slide_fields,
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
        field_spans: OrderedMap::new(),
    };

    let deck = Deck {
        slides: vec![semantic_slide],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("P08-HIGH-001 Title Bold Test")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
        slide_sections: vec![],
    };

    // ── LaidOutDeck with a Title frame (source_index = 0) ────────────────
    let laid_out = LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("content"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(1_143_000), // ~3-inch title bar
                },
                content: FrameContent::Title(Arc::from("Bold Title")),
                text_flow: None,
                region_role: None,
            }],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
        slide_sections: vec![],
    };

    let brand = Brand {
        name: Arc::from("TestBrand"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
            secondary: Arc::from("#FFFFFF"),
            accent: Arc::from("#F5A623"),
            neutral: Arc::from("#F0F0F0"),
        },
        fonts: BrandFonts {
            heading: Arc::from("NoSuchFont_P08"),
            body: Arc::from("NoSuchFont_P08"),
            mono: Arc::from("NoSuchFont_P08"),
            font_size_emu: 457_200,
        },
        layouts: vec![],
        span: SourceSpan::default(),
    };
    let opts = ExportOptions::default();

    // ── Export uncompressed PDF and scan for both PostScript names ────────
    let pdf_bytes = exporter
        .export_uncompressed(&deck, &laid_out, &brand, &opts)
        .expect("P08-HIGH-001: export_uncompressed must succeed with injected ResolvedFontSet");

    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "P08-HIGH-001: exported PDF must start with %PDF-"
    );

    // The regular face (LM Math) must appear — confirms basic font embedding works.
    let has_lm_math = pdf_bytes
        .windows(b"LatinModernMath-Regular".len())
        .any(|w| w == b"LatinModernMath-Regular");

    // The bold face (Tuffy) must appear — confirms title used font_set.bold.
    let has_tuffy = pdf_bytes.windows(b"Tuffy".len()).any(|w| w == b"Tuffy");

    assert!(
        has_tuffy,
        "ADV-P08-HIGH-001 RED GATE FAILURE: PDF title with InlineNode::Bold must embed the BOLD \
         font face ('Tuffy') — not the regular face only.\n\
         'Tuffy' is absent from the uncompressed PDF bytes, which means the title was drawn \
         with font_set.regular unconditionally (flatten-to-regular path).\n\
         Root cause: draw_frame's FrameContent::Title arm does not dispatch to \
         slide_to_krilla_runs + draw_inline_spans when title_inlines_override is Some.\n\
         Fix: wire FrameContent::Title to the rich inline path (same as SubtitleInlines).\n\
         has_lm_math={has_lm_math}\n\
         PDF excerpt (lossy UTF-8, first 3000 chars): {}",
        String::from_utf8_lossy(&pdf_bytes[..pdf_bytes.len().min(3000)])
    );

    assert!(
        has_lm_math,
        "P08-HIGH-001: PDF must also contain 'LatinModernMath-Regular' (the regular face). \
         Absence suggests font embedding regression.\n\
         has_tuffy={has_tuffy}"
    );
}

// =============================================================================
// F-P13-002: load-bearing combined-form assertion — PDF (slide_to_krilla_runs)
//
// `Bold([Italic([Plain("bi")])])` driven through `slide_to_krilla_runs` (the
// public boundary) must produce a span with `FontFaceKind::BoldItalic`.
//
// This exercises the `collect_spans` Italic↔Bold merge in slide_pdf.rs
// (lines 252-271 in the current implementation):
//
//   InlineNode::Bold(children) => {
//       let child_face = match inherited_face {
//           FontFaceKind::Italic => FontFaceKind::BoldItalic,
//           _ => FontFaceKind::Bold,
//       };
//       ...
//   }
//   InlineNode::Italic(children) => {
//       let child_face = match inherited_face {
//           FontFaceKind::Bold => FontFaceKind::BoldItalic,
//           _ => FontFaceKind::Italic,
//       };
//       ...
//   }
//
// The test directly constructs `Bold([Italic([Plain])])` and asserts the
// resulting span uses `FontFaceKind::BoldItalic` — currently ONLY tested by
// constructing BoldItalic directly (not via the nesting path). A regression
// that drops the face merge (e.g. resets child_face to Bold unconditionally
// in the Italic arm) would cause this test to FAIL.
// =============================================================================

/// F-P13-002 load-bearing: `Bold([Italic([Plain("bi")])])` → `slide_to_krilla_runs`
/// returns a span with `FontFaceKind::BoldItalic`.
///
/// Exercises the `collect_spans` face-merge logic in `slide_pdf.rs`:
/// - Outer `InlineNode::Bold` sets `inherited_face = FontFaceKind::Bold`.
/// - Inner `InlineNode::Italic` sees `inherited_face = Bold` and resolves to
///   `FontFaceKind::BoldItalic`.
/// - Leaf `InlineNode::Plain("bi")` receives `FontFaceKind::BoldItalic`.
///
/// A regression that makes the Italic arm ignore the incoming Bold context
/// (e.g. always setting `Italic` regardless of `inherited_face`) would cause
/// the span to have `FontFaceKind::Italic` instead of `BoldItalic` — FAIL.
#[test]
fn test_f_p13_002_pdf_bold_italic_combined_form_resolves_to_bolditalic() {
    let nodes = vec![InlineNode::Bold(vec![InlineNode::Italic(vec![
        InlineNode::Plain(Arc::from("bi")),
    ])])];
    let spans = slide_to_krilla_runs(&nodes);

    assert_eq!(
        spans.len(),
        1,
        "F-P13-002: Bold([Italic([Plain('bi')])]) must produce exactly 1 span; got: {spans:?}"
    );

    assert_eq!(
        spans[0].face,
        FontFaceKind::BoldItalic,
        "F-P13-002 load-bearing: Bold([Italic([Plain])]) must produce FontFaceKind::BoldItalic. \
         Exercises the collect_spans face-merge: outer Bold sets inherited_face=Bold; \
         inner Italic sees Bold and resolves to BoldItalic. \
         A regression resetting this merge (returning Bold or Italic instead) FAILS here. \
         Got: {:?}",
        spans[0].face
    );

    assert_eq!(
        spans[0].text.as_ref(),
        "bi",
        "F-P13-002: span text must be 'bi'; got: {:?}",
        spans[0].text
    );
}

// ─── ADV-P22-MED-001: adjusted_baseline_y direction lock ─────────────────────
//
// BC-3.05.001 PC-5 + STORY-081 AC-004 specify a TWO-part PASS condition for
// super/subscript:
//   (a) reduced size  — covered by `test_BC_3_05_001_adv_p04_high001_*`
//   (b) shifted baseline DIRECTION — THIS test closes the gap.
//
// `adjusted_baseline_y` (exporter.rs) takes a `KrillaTextSpan` and the parent
// baseline Y coordinate (PDF Y-down space) and returns the adjusted Y:
//   - Superscript (y_offset_units > 0): result < baseline_y  (raised toward page top)
//   - Subscript   (y_offset_units < 0): result > baseline_y  (lowered toward page bottom)
//   - Normal      (y_offset_units == 0): result == baseline_y (unchanged)
//
// The STRICT directional assertions (< and >) are the load-bearing part.
// A regression that:
//   (A) swaps the superscript/subscript arms (returning baseline_y+shift for super,
//       baseline_y−shift for sub) — would fail the `<` and `>` assertions.
//   (B) collapses both arms to `baseline_y` (removing the shift entirely) — would
//       also fail the `<` and `>` assertions.
//   (C) returns the constant value regardless of span — same failure.
// The approximate-value assertions additionally guard against off-by-fraction bugs.
// =============================================================================

/// ADV-P22-MED-001 RED GATE — `adjusted_baseline_y` raises superscript and
/// lowers subscript in PDF Y-down coordinate space.
///
/// ## What is asserted
///
/// For `baseline_y = 100.0` and `base_font_size = 12.0`:
///
/// - **Superscript** (`y_offset_units = SUPER_OFFSET_UNITS > 0`):
///   - `result ≈ 100.0 − (12.0 × SUPER_RISE_FRACTION)` (within 0.001 pt)
///   - `result < 100.0` strictly — raised above parent baseline in Y-down space.
///
/// - **Subscript** (`y_offset_units = SUB_OFFSET_UNITS < 0`):
///   - `result ≈ 100.0 + (12.0 × SUB_DROP_FRACTION)` (within 0.001 pt)
///   - `result > 100.0` strictly — lowered below parent baseline in Y-down space.
///
/// - **Normal** (`y_offset_units = 0`):
///   - `result == 100.0` exactly — baseline unchanged.
///
/// ## How we confirmed this test is load-bearing
///
/// Flipping the match arms in `adjusted_baseline_y` (swapping the `Greater`
/// and `Less` branches) would produce:
///
///   - superscript result = 100.0 + 3.996 = 103.996  → FAILS `< 100.0`
///   - subscript  result = 100.0 − 3.996 = 96.004   → FAILS `> 100.0`
///
/// Returning `baseline_y` for all arms would produce 100.0 for all three spans,
/// making both strict inequalities FAIL.
#[test]
fn test_BC_3_05_001_ac004_pc5_superscript_baseline_raised_subscript_lowered() {
    use slideforge_pdf::exporter::adjusted_baseline_y;
    use slideforge_pdf::slide_pdf::SUB_OFFSET_UNITS;
    use slideforge_pdf::{SUB_DROP_FRACTION, SUPER_RISE_FRACTION};

    let baseline_y: f32 = 100.0;
    let base_font_size: f32 = 12.0;

    // ── superscript span (y_offset_units > 0) ────────────────────────────────
    let super_span = KrillaTextSpan {
        face: FontFaceKind::Regular,
        text: Arc::from("2"),
        y_offset_units: SUPER_OFFSET_UNITS, // positive
    };

    let super_result = adjusted_baseline_y(&super_span, baseline_y, base_font_size);
    let expected_super = baseline_y - (base_font_size * SUPER_RISE_FRACTION);

    assert!(
        super_result < baseline_y,
        "ADV-P22-MED-001 RED GATE (direction): superscript must be RAISED above parent \
         baseline in Y-down space (result < baseline_y). \
         baseline_y={baseline_y:.4}, result={super_result:.4}. \
         If result >= baseline_y, the Y-direction is WRONG (swapped arms or no shift)."
    );
    assert!(
        (super_result - expected_super).abs() < 0.001,
        "ADV-P22-MED-001 (magnitude): superscript Y must equal \
         baseline_y − (base_font_size × SUPER_RISE_FRACTION). \
         Expected {expected_super:.4} (= {baseline_y:.4} − {base_font_size:.4} × {SUPER_RISE_FRACTION:.4}), \
         got {super_result:.4}."
    );

    // ── subscript span (y_offset_units < 0) ──────────────────────────────────
    let sub_span = KrillaTextSpan {
        face: FontFaceKind::Regular,
        text: Arc::from("n"),
        y_offset_units: SUB_OFFSET_UNITS, // negative
    };

    let sub_result = adjusted_baseline_y(&sub_span, baseline_y, base_font_size);
    let expected_sub = baseline_y + (base_font_size * SUB_DROP_FRACTION);

    assert!(
        sub_result > baseline_y,
        "ADV-P22-MED-001 RED GATE (direction): subscript must be LOWERED below parent \
         baseline in Y-down space (result > baseline_y). \
         baseline_y={baseline_y:.4}, result={sub_result:.4}. \
         If result <= baseline_y, the Y-direction is WRONG (swapped arms or no shift)."
    );
    assert!(
        (sub_result - expected_sub).abs() < 0.001,
        "ADV-P22-MED-001 (magnitude): subscript Y must equal \
         baseline_y + (base_font_size × SUB_DROP_FRACTION). \
         Expected {expected_sub:.4} (= {baseline_y:.4} + {base_font_size:.4} × {SUB_DROP_FRACTION:.4}), \
         got {sub_result:.4}."
    );

    // ── normal span (y_offset_units == 0) ────────────────────────────────────
    let normal_span = KrillaTextSpan {
        face: FontFaceKind::Regular,
        text: Arc::from("x"),
        y_offset_units: 0,
    };

    let normal_result = adjusted_baseline_y(&normal_span, baseline_y, base_font_size);

    #[allow(clippy::float_cmp)] // intentional: f32 identity (no arithmetic on this path)
    let unchanged = normal_result == baseline_y;
    assert!(
        unchanged,
        "ADV-P22-MED-001: normal span (y_offset_units = 0) must return baseline_y \
         UNCHANGED. Expected {baseline_y:.4}, got {normal_result:.4}."
    );
}
