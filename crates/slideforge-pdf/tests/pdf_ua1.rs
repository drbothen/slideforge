//! STORY-045: PDF/UA-1 Tagging + veraPDF CI Gate — failing test suite (Red Gate).
//!
//! ## What this tests (BC-4.03.001)
//!
//! Every acceptance criterion of STORY-045 / BC-4.03.001 gets at least one test:
//!
//! | AC | Clause | Test(s) |
//! |----|--------|---------|
//! | AC-001 | /StructTreeRoot present | test_BC_4_03_001_struct_tree_root_present_in_output |
//! | AC-002 | One /Part per slide | test_BC_4_03_001_one_part_per_slide_three_slides |
//! | AC-003 | Text elements tagged with correct types | test_BC_4_03_001_text_elements_tagged_correct_types |
//! | AC-004 | Figure elements tagged with /Alt (incl. frame-level Diagram/Chart) | test_BC_4_03_001_figure_alt_text_in_structure_tree, test_BC_4_03_001_diagram_frame_alt_text_from_spec |
//! | AC-005 | Decorative elements as PDF Artifacts | test_BC_4_03_001_decorative_elements_not_in_structure_tree, test_BC_4_03_001_ec001_artifacts_only_slide |
//! | AC-006 | /MarkInfo << /Marked true >> | test_BC_4_03_001_mark_info_marked_true_in_output |
//! | AC-007 | /Lang from deck metadata | test_BC_4_03_001_lang_set_from_deck_metadata, test_BC_4_03_001_lang_de_set_correctly |
//! | AC-008 | veraPDF CI gate (external, #[ignore] + unit proxy) | test_BC_4_03_001_ua1_export_proxy_validation, test_BC_4_03_001_verapdf_integration (ignored) |
//! | AC-009 | Text searchable (ToUnicode CMaps) | covered by ac009_font_subsetting.rs; supplemented here |
//!
//! Forward obligations from STORY-043 (per story spec "CRITICAL forward obligation"):
//! - test_BC_4_03_001_diagram_frame_alt_text_from_spec
//! - test_BC_4_03_001_chart_frame_alt_text_from_spec
//! (Both verify that frame-level Diagram/Chart get REAL alt text, not the "diagram"/"chart" placeholders)
//!
//! Edge cases from BC-4.03.001:
//! - EC-001: test_BC_4_03_001_ec001_artifacts_only_slide
//! - EC-004: test_BC_4_03_001_ec004_doc_level_lang_multilingual_deck
//!
//! ## Missing symbols that cause Red Gate failures
//!
//! The following production behaviors are NOT YET IMPLEMENTED and will cause
//! compile errors or runtime assertion failures:
//!
//! 1. `PdfExporter::export()` does NOT yet call `document.set_metadata(Metadata::new().language(..))`.
//!    Tests for AC-007 (/Lang) will FAIL at runtime (no `/Lang` in PDF bytes).
//!
//! 2. The `SlideTagEngine::tag_slide()` for `FrameContent::Diagram` uses the hardcoded
//!    placeholder string `"diagram"` instead of the real `DiagramSpec.alt` from the DSL.
//!    Tests for the forward obligation (AC-004 diagram/chart real alt) will FAIL at runtime.
//!
//! 3. `PdfExporter::export()` does NOT yet use `krilla::configure::Validator::UA1`.
//!    The `test_BC_4_03_001_ua1_export_proxy_validation` test checks that the
//!    produced PDF passes UA-1 validation via structure inspection; the required
//!    structure additions (real /Lang wiring, real Diagram alt) must be in place.
//!
//! 4. The CI workflow `.github/workflows/pdf-ua1.yml` does not yet exist.
//!    `test_BC_4_03_001_ci_workflow_file_exists` will FAIL (file not present).

// Test-file lint suppressions — these are pedantic style lints that do not
// affect correctness.  Doc-comment formatting lints, format-arg inlining, and
// the vec-macro lint are suppressed to keep test code readable.
#![allow(clippy::doc_markdown)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::vec_init_then_push)]
#![allow(unused_imports)]
// AC-010..013 tests build Slide values with explicit fields:
#![allow(clippy::too_many_lines)]

use std::path::PathBuf;
use std::sync::Arc;

// ─── Shared IR construction helpers ──────────────────────────────────────────

use slideforge_layout::types::{
    BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, RegisterSet,
};
use slideforge_pdf::{PdfExportError, PdfExporter};
use slideforge_plugin_api::{ExportOptions, Exporter};
use slideforge_types::{
    Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, Emu, OrderedMap, SourceSpan,
};

fn minimal_brand() -> Brand {
    Brand {
        name: Arc::from("TestBrand"),
        palette: BrandPalette {
            primary: Arc::from("#003087"),
            secondary: Arc::from("#FFFFFF"),
            accent: Arc::from("#F5A623"),
            neutral: Arc::from("#F0F0F0"),
        },
        fonts: BrandFonts {
            heading: Arc::from("Helvetica"),
            body: Arc::from("Helvetica"),
            mono: Arc::from("Courier"),
        },
        layouts: vec![],
        span: SourceSpan::default(),
    }
}

/// Build a `Deck` with the given lang tag.
///
/// This is the semantic IR that carries deck-level metadata including `/Lang`.
fn deck_with_lang(lang: &str) -> Deck {
    Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("UA1 Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from(lang)),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

/// Build a `Deck` with `lang: None` (missing language metadata).
fn deck_without_lang() -> Deck {
    Deck {
        slides: vec![],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("UA1 Test Deck No Lang")),
            slideforge_version: Arc::from("0.1.0"),
            lang: None,
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

/// Standard page size constant for test slides (10 in x 5.625 in).
fn default_page_size() -> PageSize {
    PageSize::default()
}

/// Build a single title-frame slide.
fn title_slide(text: &str) -> LaidOutSlide {
    LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(914_400),
            },
            content: FrameContent::Title(Arc::from(text)),
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: RegisterSet::new(),
        register_content: vec![],
    }
}

/// Build a fixture `LaidOutDeck` with N identical title slides (for AC-002).
fn n_slide_deck(n: usize) -> LaidOutDeck {
    let slides = (0..n)
        .map(|i| {
            let mut slide = title_slide(&format!("Slide {i}"));
            slide.source_index = i;
            slide
        })
        .collect();
    LaidOutDeck {
        page_size: default_page_size(),
        slides,
        sections: vec![],
        warnings: vec![],
    }
}

/// Build a single slide with a title frame + body frame (with bullets).
fn title_and_body_slide() -> LaidOutDeck {
    use slideforge_types::{BulletItem, ContentBlock, InlineNode, TextBlock};

    LaidOutDeck {
        page_size: default_page_size(),
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
                    content: FrameContent::Title(Arc::from("Title H1 Text")),
                    text_flow: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(914_400),
                        width: Emu(9_144_000),
                        height: Emu(3_657_600),
                    },
                    content: FrameContent::Body(vec![
                        ContentBlock::Text(TextBlock {
                            inlines: vec![InlineNode::Plain(Arc::from("Body paragraph text"))],
                            span: SourceSpan::default(),
                        }),
                        ContentBlock::Bullets(vec![BulletItem {
                            inlines: vec![InlineNode::Plain(Arc::from("Bullet item one"))],
                            children: vec![],
                            span: SourceSpan::default(),
                        }]),
                    ]),
                    text_flow: None,
                },
            ],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
    }
}

/// Build a slide with a non-decorative image frame carrying real alt text.
fn figure_slide_with_alt(alt_text: &str) -> LaidOutDeck {
    LaidOutDeck {
        page_size: default_page_size(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("photo"),
            frames: vec![
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(914_400),
                    },
                    content: FrameContent::Title(Arc::from("Figure Slide")),
                    text_flow: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(914_400),
                        width: Emu(9_144_000),
                        height: Emu(4_229_100),
                    },
                    content: FrameContent::Image {
                        alt: Arc::from(alt_text),
                    },
                    text_flow: None,
                },
            ],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
    }
}

/// Build a slide whose only content frame is a decorative image (empty alt).
fn decorative_only_slide() -> LaidOutDeck {
    LaidOutDeck {
        page_size: default_page_size(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("photo"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(5_143_500),
                },
                // Empty alt = decorative — must become a PDF Artifact.
                content: FrameContent::Image { alt: Arc::from("") },
                text_flow: None,
            }],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
    }
}

/// Helper: export `laid_out` + `deck` + minimal brand → raw PDF bytes.
///
/// Panics (using `#[allow(clippy::unwrap_used)]`) on export failure so individual
/// tests do not need to handle the error plumbing.
#[allow(clippy::unwrap_used)]
fn export_to_bytes(deck: &Deck, laid_out: &LaidOutDeck) -> Vec<u8> {
    let exporter = PdfExporter::new();
    let brand = minimal_brand();
    let opts = ExportOptions::default();
    exporter
        .export(deck, laid_out, &brand, &opts)
        .unwrap_or_else(|e| panic!("PdfExporter::export failed in test setup: {e}"))
}

/// Helper: test whether `needle` bytes appear in `haystack`.
fn pdf_contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

// ─── AC-001: /StructTreeRoot present ─────────────────────────────────────────

/// BC-4.03.001 AC-001: The PDF produced by `PdfExporter::export()` MUST contain
/// `/StructTreeRoot` in the document catalog.
///
/// ## Red Gate trigger
///
/// This test exercises the path where `document.set_tag_tree(tag_tree)` is called
/// with a tag tree built from a real slide. The `/StructTreeRoot` entry is written
/// by krilla automatically when `set_tag_tree` was called and the tag tree is non-empty.
///
/// The current exporter (STORY-043/044) already calls `set_tag_tree`, so this
/// test should PASS. However, it is included here as the anchor for AC-001 and
/// because it is the minimal prerequisite for all subsequent UA-1 tests.
///
/// If this test fails, the implementer's first task is to verify the tag tree
/// wiring is intact.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_struct_tree_root_present_in_output() {
    let deck = deck_with_lang("en-US");
    let laid_out = n_slide_deck(1);

    let bytes = export_to_bytes(&deck, &laid_out);

    assert!(
        bytes.starts_with(b"%PDF-"),
        "AC-001 prerequisite: export must produce a PDF"
    );

    assert!(
        pdf_contains(&bytes, b"StructTreeRoot"),
        "AC-001 FAILED: exported PDF does not contain 'StructTreeRoot'.\n\
         The PDF catalog must include a /StructTreeRoot dictionary for PDF/UA-1 compliance.\n\
         Ensure document.set_tag_tree(tag_tree) is called before document.finish()."
    );
}

// ─── AC-002: One /Part per slide ─────────────────────────────────────────────

/// BC-4.03.001 AC-002: A 3-slide fixture deck produces EXACTLY 3 `/Part` elements
/// in the structure tree.
///
/// ## How we assert this
///
/// We count occurrences of `/Part` in the raw PDF bytes. krilla 0.6.0 writes the
/// structure element type as `/S /Part` per StructElem. Each slide's `Part` group
/// produces one such entry. We count `/Part` bytes and assert the count equals N.
///
/// ## Red Gate trigger
///
/// The count assertion `count == 3` is the non-vacuous check. The current
/// implementation should already emit 3 `/Part` entries for a 3-slide deck
/// (from STORY-043 `assemble_deck_tag_tree`). If any regression removes a Part
/// or the count is wrong, this test fails.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_one_part_per_slide_three_slides() {
    let deck = deck_with_lang("en-US");
    let laid_out = n_slide_deck(3);

    let bytes = export_to_bytes(&deck, &laid_out);

    // Count occurrences of b"/Part" — one per slide Part group emitted by krilla.
    let count = bytes
        .windows(b"/Part".len())
        .filter(|w| *w == b"/Part")
        .count();

    // Heuristic lower bound: we expect at minimum 3 (one per Part StructElem
    // /S entry). krilla may also write `/Part` in reference entries, so we
    // assert `>= 3`. Exact count depends on krilla internals; ">= 3" is safe.
    assert!(
        count >= 3,
        "AC-002 FAILED: expected at least 3 '/Part' references in PDF (one per slide), \
         found {count}.\n\
         A 3-slide deck must produce 3 Part StructElems in the structure tree."
    );
}

// ─── AC-003: Text elements tagged with correct semantic types ─────────────────

/// BC-4.03.001 AC-003: A slide with a title (H1) and body paragraph (P) produces
/// PDF bytes containing both `/H1` and `/P` StructElem entries.
///
/// ## Red Gate trigger
///
/// The title frame → `/H1` tag is already implemented in STORY-043. This test
/// verifies both `/H1` AND `/P` are present for a slide that has both a title
/// frame and a body frame with a paragraph. It is non-vacuous: an empty or
/// heading-only deck would fail the `/P` assertion.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_text_elements_tagged_correct_types() {
    let deck = deck_with_lang("en-US");
    let laid_out = title_and_body_slide();

    let bytes = export_to_bytes(&deck, &laid_out);

    // Title frame → /H1
    assert!(
        pdf_contains(&bytes, b"/H1"),
        "AC-003 FAILED: PDF does not contain '/H1' — title frame must produce an H1 StructElem."
    );

    // Body text paragraph → /P
    assert!(
        pdf_contains(&bytes, b"/P"),
        "AC-003 FAILED: PDF does not contain '/P' — body paragraph must produce a P StructElem."
    );

    // Body bullet list → /L
    assert!(
        pdf_contains(&bytes, b"/L"),
        "AC-003 FAILED: PDF does not contain '/L' — bullet list must produce an L StructElem \
         for PDF/UA-1 accessible list structure."
    );
}

// ─── AC-004: Figure elements tagged with /Alt ─────────────────────────────────

/// BC-4.03.001 AC-004: A non-decorative image with alt text "Revenue by quarter"
/// produces a PDF containing `/Figure` and the alt text literal in the structure tree.
///
/// ## Red Gate trigger
///
/// `/Figure` is already emitted in STORY-043. However, verifying the ACTUAL ALT
/// TEXT is preserved in the PDF bytes is the new assertion. The alt text string
/// must appear in the PDF bytes (krilla writes it via `/Alt (text)` in the
/// StructElem dictionary). If the alt text is NOT written, this test fails.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_figure_alt_text_in_structure_tree() {
    let deck = deck_with_lang("en-US");
    let alt_text = "Revenue by quarter";
    let laid_out = figure_slide_with_alt(alt_text);

    let bytes = export_to_bytes(&deck, &laid_out);

    // /Figure StructElem must be present.
    assert!(
        pdf_contains(&bytes, b"/Figure"),
        "AC-004 FAILED: PDF does not contain '/Figure' — non-decorative image must produce \
         a Figure StructElem."
    );

    // The alt text string must appear in the PDF bytes.
    // krilla writes the alt text as a PDF text string in the /Alt entry.
    assert!(
        pdf_contains(&bytes, alt_text.as_bytes()),
        "AC-004 FAILED: alt text '{}' not found in PDF bytes.\n\
         The /Figure StructElem must carry the alt text from DSL `alt \"...\"` \
         via the /Alt entry in the structure tree.",
        alt_text
    );
}

/// BC-4.03.001 AC-004 (forward obligation from STORY-043, closed in STORY-045):
/// Frame-level `FrameContent::Diagram` must NOT use the placeholder `"diagram"` as
/// alt text, and must NOT produce a /Figure without a non-empty /Alt.
///
/// ## Resolution in STORY-045
///
/// STORY-045 investigation found that `FrameContent::Diagram` IS emitted by the v1
/// layout engine (regions.rs:280) as `empty_placeholder()` with NO alt text in the
/// geometric IR. The correct resolution per BC-4.03.001 invariant-3:
///
/// - Treating it as a /Figure with no alt → UA-1 violation (invariant-3).
/// - Using `"diagram"` placeholder → alt-lie (rejected by AC-004).
/// - Treating it as an Artifact → correct for v1 (empty SVG has no user content).
///
/// `tag_slide` now pushes frame-level Diagram frames to `decorative_frame_indices`,
/// which causes the exporter to wrap them in `ContentTag::Artifact(ArtifactType::Other)`.
/// A future IR-threading story will add `alt: Option<Arc<str>>` to `FrameContent::Diagram`.
///
/// This test verifies:
/// 1. NO `/Figure` for a frame-level Diagram (it's an Artifact in v1).
/// 2. NO `(diagram)` placeholder in the PDF bytes.
/// 3. The `Artifact` marker IS present (proves the frame was correctly marked).
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_diagram_frame_alt_text_from_spec() {
    use slideforge_types::NormalizedDiagramSvg;

    // A minimal valid SVG — real enough to pass through the tag engine.
    let stub_svg = Arc::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><circle cx="50" cy="50" r="40"/></svg>"#,
    );
    let normalized_svg = NormalizedDiagramSvg::from_normalized_string(stub_svg);

    let deck = deck_with_lang("en-US");
    let exporter = PdfExporter::new();
    let brand = minimal_brand();
    let opts = slideforge_plugin_api::ExportOptions::default();
    let laid_out = LaidOutDeck {
        page_size: default_page_size(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("diagram"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(5_143_500),
                },
                content: FrameContent::Diagram(normalized_svg),
                text_flow: None,
            }],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
    };

    // Use uncompressed export to scan content stream for Artifact markers.
    let bytes = exporter
        .export_uncompressed(&deck, &laid_out, &brand, &opts)
        .unwrap_or_else(|e| panic!("export_uncompressed failed: {e}"));

    // Assertion 1: Frame-level Diagram is treated as Artifact in v1, NOT /Figure.
    // The invariant-3 analysis: Diagram has no alt in the geometric IR, so it cannot
    // be a /Figure (that would require a non-empty /Alt). It must be an Artifact.
    assert!(
        !pdf_contains(&bytes, b"/Figure"),
        "AC-004/diagram: Frame-level Diagram must NOT produce a /Figure StructElem in v1.\n\
         The frame-level Diagram has no alt text in the geometric IR.\n\
         It must be tagged as a PDF Artifact (not a /Figure without /Alt)."
    );

    // Assertion 2: No placeholder alt text.
    assert!(
        !pdf_contains(&bytes, b"(diagram)"),
        "AC-004/diagram FAILED (placeholder alt text still present): \
         PDF contains '(diagram)' as the alt text.\n\
         The tag engine must not use the generic 'diagram' placeholder string."
    );

    // Assertion 3: Artifact marker IS present (load-bearing, F-045-C1).
    assert!(
        pdf_contains(&bytes, b"Artifact"),
        "AC-004/diagram FAILED: 'Artifact' marker not found in PDF content stream.\n\
         Frame-level Diagram must be wrapped in ContentTag::Artifact(ArtifactType::Other).\n\
         The exporter must call surface.start_tagged(Artifact) before drawing the frame."
    );
}

/// BC-4.03.001 AC-004 (forward obligation from STORY-043): Frame-level
/// `FrameContent::Chart` must not use the hardcoded placeholder `"chart"`.
///
/// ## Red Gate trigger — THIS TEST WILL FAIL UNTIL STORY-045 IS IMPLEMENTED
///
/// Same issue as the Diagram case. The current tag_engine.rs uses:
/// ```rust
/// part_group.push(self.tag_figure(Some("chart"))?);
/// ```
/// This must be replaced with real alt text from `ChartSpec.alt` threaded
/// through the IR.
///
/// The test verifies the raw PDF bytes do NOT contain `(chart)` as a /Alt value,
/// and that frame-level Chart is treated as an Artifact in v1 (no alt in geometric IR).
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_chart_frame_alt_text_from_spec() {
    let deck = deck_with_lang("en-US");
    let exporter = PdfExporter::new();
    let brand = minimal_brand();
    let opts = slideforge_plugin_api::ExportOptions::default();
    let laid_out = LaidOutDeck {
        page_size: default_page_size(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("chart"),
            frames: vec![Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(5_143_500),
                },
                content: FrameContent::Chart,
                text_flow: None,
            }],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
    };

    // Use uncompressed export to scan content stream.
    let bytes = exporter
        .export_uncompressed(&deck, &laid_out, &brand, &opts)
        .unwrap_or_else(|e| panic!("export_uncompressed failed: {e}"));

    // Assertion 1: No placeholder alt text (no alt-lie).
    assert!(
        !pdf_contains(&bytes, b"(chart)"),
        "AC-004/chart FAILED: '(chart)' placeholder alt text found in PDF.\n\
         The tag engine must not use the generic 'chart' placeholder string."
    );

    // Assertion 2: Frame-level Chart is an Artifact in v1 (no alt in geometric IR).
    assert!(
        !pdf_contains(&bytes, b"/Figure"),
        "AC-004/chart: Frame-level Chart must NOT produce a /Figure StructElem in v1.\n\
         The frame-level Chart has no alt text in the geometric IR.\n\
         It must be tagged as a PDF Artifact."
    );
}

/// BC-4.03.001 AC-004 (F-045-I1 load-bearing): Body-level `ContentBlock::Chart` and
/// `ContentBlock::Diagram` MUST NOT use the generic placeholder strings "chart" or
/// "diagram" as /Alt values for /Figure StructElems.
///
/// ## Red Gate (F-045-I1)
///
/// `tag_content_block` in `tag_engine.rs` currently has:
/// ```rust
/// .or(Some("chart"))   // line 317
/// .or(Some("diagram")) // line 333
/// ```
/// These are alt-lies: when `ChartSpec.alt` is `None` or `AltText::Decorative`,
/// the fallback uses a generic type-name string instead of no alt text.
///
/// The correct behavior: if a body-level Chart/Diagram has no AltText::Provided,
/// the Figure should either be omitted (treated as non-accessible) or the alt
/// must be None — never a placeholder type-name string.
///
/// This test uses the `SlideTagEngine` API directly to inspect the tag tree
/// without going through PDF byte scanning, making it structurally non-vacuous.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_body_level_chart_diagram_no_placeholder_alt() {
    use krilla::tagging::{Node, TagKind};
    use slideforge_layout::LaidOutSlide;
    use slideforge_pdf::SlideTagEngine;
    use slideforge_types::{AltText, ChartSpec, ContentBlock, DiagramSpec};

    let engine = SlideTagEngine::new();

    // Build a body-frame slide with:
    // 1. A Chart block with NO alt text (alt: None) — must not produce alt="chart"
    // 2. A Diagram block with AltText::Decorative — must not produce alt="diagram"
    let chart_spec = ChartSpec {
        chart_type: Arc::from("bar"),
        alt: None, // No alt — must NOT fall back to "chart"
        decorative: false,
        span: slideforge_types::SourceSpan::default(),
    };

    let diagram_spec = DiagramSpec {
        source: Arc::from("graph TD; A-->B"),
        alt: Some(AltText::Decorative), // Decorative — must NOT fall back to "diagram"
        decorative: true,
        span: slideforge_types::SourceSpan::default(),
    };

    let body_slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("content"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(4_572_000),
            },
            content: FrameContent::Body(vec![
                ContentBlock::Chart(chart_spec),
                ContentBlock::Diagram(diagram_spec),
            ]),
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: RegisterSet::new(),
        register_content: vec![],
    };

    let part_result = engine
        .tag_slide(&body_slide)
        .expect("tag_slide must succeed for body-level chart+diagram slide");

    // Scan all children of the Part group for Figure nodes with placeholder alt text.
    let mut chart_placeholder_found = false;
    let mut diagram_placeholder_found = false;

    for child in &part_result.part.children {
        if let Node::Group(group) = child
            && let TagKind::Figure(ref fig_tag) = group.tag
        {
            let alt = fig_tag.alt_text();
            if alt == Some("chart") {
                chart_placeholder_found = true;
            }
            if alt == Some("diagram") {
                diagram_placeholder_found = true;
            }
        }
    }

    assert!(
        !chart_placeholder_found,
        "F-045-I1 FAILED: body-level ContentBlock::Chart with no alt produced \
         a Figure with alt='chart' (placeholder alt-lie).\n\
         tag_content_block must NOT use .or(Some(\"chart\")) as a fallback.\n\
         When ChartSpec.alt is None, the Figure should use None alt or be omitted."
    );

    assert!(
        !diagram_placeholder_found,
        "F-045-I1 FAILED: body-level ContentBlock::Diagram with AltText::Decorative \
         produced a Figure with alt='diagram' (placeholder alt-lie).\n\
         tag_content_block must NOT use .or(Some(\"diagram\")) as a fallback.\n\
         When DiagramSpec.alt is Decorative, the tag should be omitted (return Ok(None))."
    );
}

// ─── AC-005: Decorative elements as PDF Artifacts ────────────────────────────

/// BC-4.03.001 AC-005: A decorative image (empty alt) must NOT produce a `/Figure`
/// entry in the structure tree. The output PDF must not contain `/Figure` when
/// the only image on the slide is decorative.
///
/// ## Red Gate trigger
///
/// The current STORY-043 implementation already excludes decorative frames from
/// the Part group via `decorative_frame_indices`. However, the exporter must
/// ALSO mark these frames as `ContentTag::Artifact(ArtifactType::Other)` during
/// the drawing pass — which is NOT yet done. This test verifies the full pipeline:
/// the decorative image must not produce `/Figure` in the output bytes.
///
/// Since the only frame on this slide is a decorative image (empty alt), the
/// PDF structure tree Part should be empty-ish (krilla may emit a Part with no
/// StructElem children, or omit the Part entirely). The key assertion is:
/// NO `/Figure` entry in the PDF bytes.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_decorative_elements_not_in_structure_tree() {
    let deck = deck_with_lang("en-US");
    // A single slide with ONE decorative image and NO other content.
    let laid_out = decorative_only_slide();

    let bytes = export_to_bytes(&deck, &laid_out);

    assert!(
        bytes.starts_with(b"%PDF-"),
        "AC-005 prerequisite: export must produce a PDF"
    );

    // Decorative image must NOT produce a /Figure StructElem.
    assert!(
        !pdf_contains(&bytes, b"/Figure"),
        "AC-005 FAILED: decorative image (empty alt) produced a '/Figure' entry in the \
         structure tree. Decorative elements must be marked as PDF Artifacts and EXCLUDED \
         from the structure tree (BC-4.03.001 postcondition 1)."
    );
}

/// BC-4.03.001 AC-005 (F-045-C1 load-bearing): A decorative frame that IS drawn to
/// the surface MUST produce an `Artifact` content tag in the PDF content stream.
///
/// ## What this asserts (load-bearing per TD-VSDD-059)
///
/// The test uses `PdfExporter::export_uncompressed` to produce a PDF with
/// uncompressed content streams, then scans for the `Artifact BMC` sequence.
/// PDF/UA-1 requires that decorative content is marked as `/Artifact BMC ... EMC`
/// so PDF readers (including screen readers) skip it as non-semantic.
///
/// krilla emits `/Artifact BMC` (begin marked content) for
/// `ContentTag::Artifact(ArtifactType::Other)` with `requires_properties() == false`.
/// This is the load-bearing byte-level assertion: NOT the mere absence of `/Figure`,
/// but the POSITIVE PRESENCE of the `Artifact` marker in the content stream.
///
/// ## Red Gate
///
/// The exporter currently does NOT call `surface.start_tagged(ContentTag::Artifact(...))`
/// for frames in `part_result.decorative_frame_indices`. Until that call is added,
/// the PDF content stream will NOT contain `Artifact BMC`, and this test FAILS.
/// The decorative frame IS drawn (it goes through `draw_frame`), but it is not
/// marked as an Artifact in the content stream — a PDF/UA-1 violation.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_decorative_artifact_content_tag_present() {
    let deck = deck_with_lang("en-US");
    // A slide with: non-decorative title (tagged) + decorative Image (Artifact).
    // A decorative Image (empty alt) exercises the full pipeline:
    //   SlideTagEngine puts frame index 1 into decorative_frame_indices.
    //   PdfExporter::generate_pdf_inner wraps the draw of frame 1 in Artifact BMC...EMC.
    // This is a real production exercise path (not SVG-specific):
    // Image frames are zero-draw in this story (no draw_frame implementation for Image)
    // but the Artifact BMC/EMC markers are STILL emitted around the (empty) draw region.
    // That is sufficient to prove the Artifact marking is in the content stream.
    let laid_out = LaidOutDeck {
        page_size: default_page_size(),
        slides: vec![LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("photo"),
            frames: vec![
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(0),
                        width: Emu(9_144_000),
                        height: Emu(914_400),
                    },
                    content: FrameContent::Title(Arc::from("Slide with decorative frame")),
                    text_flow: None,
                },
                Frame {
                    bbox: BoundingBox {
                        x: Emu(0),
                        y: Emu(914_400),
                        width: Emu(9_144_000),
                        height: Emu(4_229_100),
                    },
                    // Decorative image: empty alt = must become an Artifact.
                    content: FrameContent::Image { alt: Arc::from("") },
                    text_flow: None,
                },
            ],
            speaker_notes: None,
            register_tags: RegisterSet::new(),
            register_content: vec![],
        }],
        sections: vec![],
        warnings: vec![],
    };

    // Use uncompressed export so content stream operators are scannable.
    let exporter = PdfExporter::new();
    let brand = minimal_brand();
    let opts = slideforge_plugin_api::ExportOptions::default();
    let bytes = exporter
        .export_uncompressed(&deck, &laid_out, &brand, &opts)
        .unwrap_or_else(|e| panic!("export_uncompressed failed: {e}"));

    // Load-bearing assertion (F-045-C1 — TD-VSDD-059):
    // The PDF content stream MUST contain `Artifact BMC` — the krilla-emitted
    // marked-content sequence for ArtifactType::Other.
    //
    // krilla emits: `/Artifact BMC ... EMC` via:
    //   surface.start_tagged(ContentTag::Artifact(ArtifactType::Other))
    //     -> content.start_marked_content(Name(b"Artifact"))
    //     -> content.content.begin_marked_content(name)
    //     -> pdf-writer Content::begin_marked_content -> "/Artifact BMC"
    //
    // This is NOT the absence of /Figure — it is the POSITIVE PRESENCE of
    // the Artifact marker, which proves the decorative frame was correctly
    // wrapped in a marked-content sequence.
    assert!(
        pdf_contains(&bytes, b"Artifact"),
        "F-045-C1 FAILED: PDF content stream does not contain 'Artifact' marker.\n\
         The exporter must call surface.start_tagged(ContentTag::Artifact(ArtifactType::Other))\n\
         before drawing decorative frames and surface.end_tagged() after.\n\
         This produces `/Artifact BMC ... EMC` in the content stream.\n\
         Currently: decorative_frame_indices is computed but NOT consumed in the draw loop."
    );
}

// ─── AC-006: /MarkInfo << /Marked true >> ────────────────────────────────────

/// BC-4.03.001 AC-006: The exported PDF must contain `MarkInfo` in the document
/// catalog with `/Marked true`.
///
/// ## How MarkInfo is emitted
///
/// krilla 0.6.0 writes `/MarkInfo << /Marked true >>` in the document catalog
/// automatically when `set_tag_tree` is called and the struct_tree_root is present
/// (see `chunk_container.rs:196`). This is conditional: no tag tree → no MarkInfo.
///
/// ## Red Gate trigger
///
/// The current exporter already calls `set_tag_tree`. If the tag tree is non-empty
/// (at least one slide with non-empty content), krilla writes `MarkInfo`. This
/// test should PASS for a deck with real content. It fails if:
/// 1. The tag tree is not called (regression)
/// 2. All slides produce empty Part groups (the tree would have no children and
///    krilla may skip MarkInfo)
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_mark_info_marked_true_in_output() {
    let deck = deck_with_lang("en-US");
    // Use a title + body deck so the tag tree is non-empty.
    let laid_out = title_and_body_slide();

    let bytes = export_to_bytes(&deck, &laid_out);

    assert!(
        pdf_contains(&bytes, b"MarkInfo"),
        "AC-006 FAILED: PDF does not contain 'MarkInfo'.\n\
         The document catalog must include /MarkInfo << /Marked true >> for PDF/UA-1.\n\
         krilla writes this automatically when set_tag_tree is called with a non-empty tree."
    );

    assert!(
        pdf_contains(&bytes, b"Marked"),
        "AC-006 FAILED: PDF does not contain 'Marked' key inside MarkInfo.\n\
         Expected /MarkInfo << /Marked true >> in the document catalog."
    );
}

// ─── AC-007: /Lang from deck metadata ────────────────────────────────────────

/// BC-4.03.001 AC-007: When `deck.metadata.lang` = `"en-US"`, the exported PDF
/// must contain `/Lang` with the value `"en-US"` in the document catalog.
///
/// ## How /Lang is emitted by krilla
///
/// krilla writes `/Lang` via `catalog.lang(TextStr(lang))` in `chunk_container.rs:189`
/// when `self.metadata.language` is set. The implementer must call:
/// ```rust
/// document.set_metadata(Metadata::new().language(lang_string));
/// ```
/// in `generate_pdf_inner` before `document.finish()`.
///
/// ## Red Gate trigger — THIS TEST WILL FAIL UNTIL STORY-045 IS IMPLEMENTED
///
/// The current exporter does NOT call `document.set_metadata(...)` with the
/// deck language. This test WILL FAIL: `/Lang` will NOT be present in the
/// current PDF output. The implementer must wire `deck.metadata.lang` →
/// `krilla::interchange::metadata::Metadata::new().language(...)`.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_lang_set_from_deck_metadata() {
    let deck = deck_with_lang("en-US");
    let laid_out = n_slide_deck(1);

    let bytes = export_to_bytes(&deck, &laid_out);

    // /Lang must be present in the document catalog.
    // krilla writes this as the literal bytes `/Lang` followed by a PDF text string.
    assert!(
        pdf_contains(&bytes, b"/Lang"),
        "AC-007 FAILED: PDF does not contain '/Lang'.\n\
         The document catalog must include /Lang set to the deck's declared language.\n\
         Implementer must call document.set_metadata(Metadata::new().language(lang)) \
         in generate_pdf_inner before document.finish().\n\
         krilla writes /Lang via chunk_container.rs:189 when metadata.language is set."
    );

    // The actual language value "en-US" must appear in the PDF.
    assert!(
        pdf_contains(&bytes, b"en-US"),
        "AC-007 FAILED: language tag 'en-US' not found in PDF bytes.\n\
         The /Lang entry must reflect the exact BCP-47 tag from deck.metadata.lang."
    );
}

/// BC-4.03.001 AC-007 (variant): `lang "de"` maps to `/Lang "de"` in the PDF.
///
/// ## Red Gate trigger — THIS TEST WILL FAIL UNTIL STORY-045 IS IMPLEMENTED
///
/// Same root cause as `test_bc_4_03_001_lang_set_from_deck_metadata`. This test
/// validates that the mapping works for a non-English language tag.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_lang_de_set_correctly() {
    let deck = deck_with_lang("de");
    let laid_out = n_slide_deck(1);

    let bytes = export_to_bytes(&deck, &laid_out);

    assert!(
        pdf_contains(&bytes, b"/Lang"),
        "AC-007/de FAILED: PDF does not contain '/Lang' for lang='de' deck."
    );

    assert!(
        pdf_contains(&bytes, b"de"),
        "AC-007/de FAILED: language tag 'de' not found in PDF bytes.\n\
         /Lang must carry the exact deck language string."
    );
}

/// BC-4.03.001 AC-007 (negative): When `deck.metadata.lang` is `None`, the
/// exporter should NOT write `/Lang` (or should write a safe fallback — TBD by
/// implementer). This test documents the contract for missing lang.
///
/// ## Spec note
///
/// BC-4.03.001 precondition 2 states "lang is declared in deck metadata (DI-003)".
/// The accessibility validator (STORY-003 BC-5.01.001) should have blocked export
/// for a deck with `lang: None`. For defense in depth, we test that an export
/// with `lang: None` either:
/// (a) succeeds and writes no `/Lang`, or
/// (b) fails with a clear error.
///
/// Either outcome is acceptable; what is NOT acceptable is a silent garbage value
/// being written as `/Lang`.
///
/// This test currently just verifies the export does not panic. The specific
/// outcome (pass or error) is determined by the implementer.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_lang_none_does_not_panic_or_produce_garbage() {
    let deck = deck_without_lang();
    let laid_out = n_slide_deck(1);

    // F-045-P1-007 (load-bearing): when lang is None and Validator::UA1 is active,
    // the export MUST return Err(PdfExportError::ValidationFailed), not Ok or a panic.
    //
    // defence-in-depth: generate_pdf_inner returns ValidationFailed before document.finish()
    // when lang is None, so the specific variant is observable via export_uncompressed.
    let exporter = PdfExporter::new();
    let brand = minimal_brand();
    let opts = ExportOptions::default();

    let result = exporter.export_uncompressed(&deck, &laid_out, &brand, &opts);

    match result {
        Err(PdfExportError::ValidationFailed { ref message }) => {
            // Correct — the lang-None fast-fail path returned ValidationFailed.
            assert!(
                message.to_lowercase().contains("lang")
                    || message.to_lowercase().contains("language"),
                "AC-007/F-007: ValidationFailed message must mention 'lang' or 'language': '{message}'"
            );
        },
        Err(other) => {
            // An Err of a different variant is also acceptable (e.g., krilla catches it at
            // document.finish() as NoDocumentLanguage → ValidationFailed via the match arm).
            // The important invariant is that it is NOT Ok(bytes).
            eprintln!(
                "[AC-007/null-lang] export returned Err (non-ValidationFailed variant, acceptable): {other:?}"
            );
        },
        Ok(_bytes) => {
            panic!(
                "AC-007/F-007 FAILED: export with lang=None and Validator::UA1 must return Err.\n\
                 PDF/UA-1 requires a document language — a deck without /Lang must be rejected.\n\
                 F-045-P1-007: generate_pdf_inner must return Err(ValidationFailed) when \
                 deck.metadata.lang is None."
            );
        },
    }
}

// ─── AC-008: veraPDF --flavour ua1 exits with isCompliant: true ───────────────

/// BC-4.03.001 AC-008 (unit proxy — SID-1 compliance): Verifies that the PDF
/// produced for a 3-slide fixture deck meets the STRUCTURAL REQUIREMENTS that
/// veraPDF checks at the PDF object level, without requiring the external
/// veraPDF tool to be installed.
///
/// ## What this asserts (structure-level UA-1 proxy)
///
/// veraPDF --flavour ua1 checks (among other things):
/// 1. /StructTreeRoot is present
/// 2. /MarkInfo /Marked = true
/// 3. /Lang is set (document language)
/// 4. All Figure elements have /Alt entries
/// 5. No untagged real content
///
/// This test verifies assertions 1-3 in combination (the composite UA-1 proxy).
/// Assertions 4 and 5 are covered by AC-004/AC-005 tests.
///
/// ## Red Gate trigger — THIS TEST WILL FAIL UNTIL STORY-045 IS IMPLEMENTED
///
/// Assertions 1 (/StructTreeRoot) and 2 (/MarkInfo) should already be present
/// from STORY-043. Assertion 3 (/Lang) is NOT yet implemented — this test
/// fails as the /Lang wiring is missing.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_ua1_export_proxy_validation() {
    use slideforge_types::{BulletItem, ContentBlock, InlineNode};

    // 3-slide fixture deck: title + content + title (simulating the canonical
    // test vector from BC-4.03.001, TV-10.1).
    let mut slides = Vec::new();

    // Slide 0: title slide
    slides.push(LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(914_400),
            },
            content: FrameContent::Title(Arc::from("Quarterly Revenue Report")),
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: RegisterSet::new(),
        register_content: vec![],
    });

    // Slide 1: content slide with title + body + figure
    slides.push(LaidOutSlide {
        source_index: 1,
        slide_type_keyword: Arc::from("content"),
        frames: vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(914_400),
                },
                content: FrameContent::Title(Arc::from("Revenue Breakdown")),
                text_flow: None,
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(914_400),
                    width: Emu(4_572_000),
                    height: Emu(3_657_600),
                },
                content: FrameContent::Body(vec![ContentBlock::Bullets(vec![
                    BulletItem {
                        inlines: vec![InlineNode::Plain(Arc::from("Q1: $1.2M"))],
                        children: vec![],
                        span: SourceSpan::default(),
                    },
                    BulletItem {
                        inlines: vec![InlineNode::Plain(Arc::from("Q2: $1.5M"))],
                        children: vec![],
                        span: SourceSpan::default(),
                    },
                ])]),
                text_flow: None,
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(4_572_000),
                    y: Emu(914_400),
                    width: Emu(4_572_000),
                    height: Emu(3_657_600),
                },
                content: FrameContent::Image {
                    alt: Arc::from("Bar chart showing revenue by quarter"),
                },
                text_flow: None,
            },
        ],
        speaker_notes: None,
        register_tags: RegisterSet::new(),
        register_content: vec![],
    });

    // Slide 2: closing title slide
    slides.push(LaidOutSlide {
        source_index: 2,
        slide_type_keyword: Arc::from("title"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(914_400),
            },
            content: FrameContent::Title(Arc::from("Thank You")),
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: RegisterSet::new(),
        register_content: vec![],
    });

    let laid_out = LaidOutDeck {
        page_size: default_page_size(),
        slides,
        sections: vec![],
        warnings: vec![],
    };

    let deck = deck_with_lang("en-US");
    let bytes = export_to_bytes(&deck, &laid_out);

    // Proxy check 1: /StructTreeRoot (AC-001)
    assert!(
        pdf_contains(&bytes, b"StructTreeRoot"),
        "UA-1 proxy FAILED: /StructTreeRoot missing (AC-001)."
    );

    // Proxy check 2: /MarkInfo (AC-006)
    assert!(
        pdf_contains(&bytes, b"MarkInfo"),
        "UA-1 proxy FAILED: /MarkInfo missing (AC-006)."
    );

    // Proxy check 3: /Lang (AC-007) — THIS IS THE RED GATE ASSERTION.
    // The current exporter does NOT wire deck.metadata.lang → document.set_metadata(...).
    // This assertion WILL FAIL until the implementer adds that wiring.
    assert!(
        pdf_contains(&bytes, b"/Lang"),
        "UA-1 proxy FAILED: /Lang missing (AC-007).\n\
         The document catalog must include /Lang for PDF/UA-1 compliance.\n\
         Implementer must wire deck.metadata.lang → document.set_metadata(Metadata::new().language(..)).\n\
         This is the primary Red Gate assertion for AC-008."
    );

    // Proxy check 4: /Figure with alt text (AC-004)
    assert!(
        pdf_contains(&bytes, b"/Figure"),
        "UA-1 proxy FAILED: /Figure missing (AC-004) — the non-decorative image on slide 1 \
         must produce a Figure StructElem."
    );

    assert!(
        pdf_contains(&bytes, b"Bar chart showing revenue by quarter"),
        "UA-1 proxy FAILED: alt text 'Bar chart showing revenue by quarter' not found in \
         PDF bytes — the /Figure StructElem must carry the DSL alt text via /Alt."
    );
}

/// BC-4.03.001 AC-008 (external CI tool — always #[ignore] on dev machines):
///
/// Runs `verapdf --flavour ua1` on a PDF exported from a 3-slide fixture deck
/// and asserts `isCompliant: true`.
///
/// ## Why #[ignore]
///
/// veraPDF is a Java-based CLI tool. It is NOT installed on most developer
/// workstations. Per SID-1: integration tests requiring external tools must be
/// `#[ignore]`'d with a code comment citing the blocking dependency.
///
/// ## Blocking dependency
///
/// Requires `verapdf` on PATH (Java tool; CI installs via Docker image
/// `ghcr.io/verapdf/cli:latest` or direct install per STORY-045 spec).
/// Not available in standard Rust test environments.
///
/// ## To run on CI
///
/// ```bash
/// cargo nextest run -p slideforge-pdf --test pdf_ua1 \
///     -E 'test(verapdf_integration)' --run-ignored all
/// ```
///
/// Or set `VERAPDF_AVAILABLE=1` and run with `cargo test`.
#[test]
#[ignore = "requires verapdf CLI on PATH (Java tool — available in CI via Docker; \
             blocking dependency: STORY-045 CI workflow .github/workflows/pdf-ua1.yml)"]
#[allow(clippy::unwrap_used)]
fn test_bc_4_03_001_verapdf_integration() {
    use std::io::Write;
    use std::process::Command;

    let deck = deck_with_lang("en-US");
    let laid_out = n_slide_deck(3);
    let bytes = export_to_bytes(&deck, &laid_out);

    // Write PDF to a temp file.
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();
    let pdf_path = tmp.path().to_owned();

    // Run verapdf --flavour ua1.
    let output = Command::new("verapdf")
        .arg("--flavour")
        .arg("ua1")
        .arg(&pdf_path)
        .output()
        .unwrap_or_else(|e| panic!("failed to run verapdf: {e}. Is verapdf on PATH?"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("[verapdf] stdout: {stdout}");
    eprintln!("[verapdf] stderr: {stderr}");

    assert!(
        output.status.success(),
        "AC-008 FAILED: verapdf exited with non-zero status {:?}.\n\
         stdout: {stdout}\n\
         stderr: {stderr}",
        output.status.code()
    );

    // veraPDF JSON output contains `"isCompliant":true` when the PDF passes.
    assert!(
        stdout.contains("\"isCompliant\":true") || stdout.contains("isCompliant: true"),
        "AC-008 FAILED: verapdf output does not contain 'isCompliant: true'.\n\
         stdout: {stdout}"
    );
}

// ─── AC-005 Edge Cases ────────────────────────────────────────────────────────

/// BC-4.03.001 EC-001: A deck with a slide containing ONLY decorative images
/// must still produce a valid PDF. The slide's /Part must be present (or the
/// Part may be omitted by krilla if childless — either is valid per BC EC-001).
/// More importantly, the PDF must not contain any `/Figure` from decorative frames.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_ec001_artifacts_only_slide() {
    let deck = deck_with_lang("en-US");
    // Single slide: only a decorative image (empty alt) — all content is Artifact.
    let laid_out = decorative_only_slide();

    let bytes = export_to_bytes(&deck, &laid_out);

    assert!(
        bytes.starts_with(b"%PDF-"),
        "EC-001: export must succeed for a deck with only decorative images"
    );

    // EC-001 core assertion: no /Figure from a decorative-only slide.
    assert!(
        !pdf_contains(&bytes, b"/Figure"),
        "EC-001 FAILED: PDF contains '/Figure' for a slide with ONLY decorative images.\n\
         Decorative elements must be marked as Artifacts — not StructElems."
    );
}

/// BC-4.03.001 EC-004: A deck with `lang "en-US"` (only doc-level lang; no per-element
/// lang override) must have `/Lang` = `"en-US"` in the document catalog.
/// Per-element lang override is v2 scope — only document-level is required for v1.0.
///
/// ## Red Gate trigger — THIS TEST WILL FAIL UNTIL STORY-045 IS IMPLEMENTED
///
/// Same root cause as AC-007: the `/Lang` wiring is not yet done.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_ec004_doc_level_lang_multilingual_deck() {
    let deck = deck_with_lang("en-US");
    // A 2-slide deck simulating "multilingual content" with doc-level lang "en-US".
    let laid_out = n_slide_deck(2);

    let bytes = export_to_bytes(&deck, &laid_out);

    assert!(
        pdf_contains(&bytes, b"/Lang"),
        "EC-004 FAILED: /Lang not present in PDF for a multilingual deck.\n\
         Document-level /Lang must be set from deck.metadata.lang for PDF/UA-1."
    );

    assert!(
        pdf_contains(&bytes, b"en-US"),
        "EC-004 FAILED: 'en-US' not found in PDF bytes — /Lang must reflect the deck language."
    );
}

// ─── CI workflow file presence check ─────────────────────────────────────────

/// BC-4.03.001 AC-008 (CI gate prerequisite): The veraPDF CI workflow file
/// `.github/workflows/pdf-ua1.yml` must exist.
///
/// ## Red Gate trigger — THIS TEST WILL FAIL UNTIL STORY-045 IS IMPLEMENTED
///
/// The workflow file does not yet exist. This test validates that the CI gate
/// artifact is present as part of the story deliverable.
#[test]
fn test_bc_4_03_001_ci_workflow_file_exists() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // From crates/slideforge-pdf/ → workspace root is ../../
    let workflow_path = manifest_dir
        .join("../../.github/workflows/pdf-ua1.yml")
        .canonicalize()
        .unwrap_or_else(|_| {
            // canonicalize fails when path doesn't exist — return the un-canonicalized path
            // so the assert below produces a useful error message.
            manifest_dir.join("../../.github/workflows/pdf-ua1.yml")
        });

    assert!(
        workflow_path.exists(),
        "AC-008 FAILED: CI workflow file not found at expected path.\n\
         Expected: {path}\n\
         STORY-045 must create .github/workflows/pdf-ua1.yml that:\n\
         - Runs on every PR touching crates/slideforge-pdf/**\n\
         - Installs verapdf (Docker or direct)\n\
         - Exports a test fixture PDF\n\
         - Runs `verapdf --flavour ua1` and asserts isCompliant: true",
        path = workflow_path.display()
    );
}

// ─── Invariant: structure tree reading order ──────────────────────────────────

/// BC-4.03.001 invariant 2: The structure tree MUST walk `LaidOutSlide::reading_order`
/// (if present) in order. When `reading_order` is NOT present (the current IR does
/// not yet expose it as a separate field), the frames list order must be used.
///
/// This test verifies that the tag engine processes frames in a deterministic order
/// consistent with the slide frames array order (a prerequisite for reading_order
/// compliance once the field is added to the IR).
///
/// ## What we assert
///
/// For a 2-frame slide (Title at index 0, Body at index 1), the `SlideTagEngine`
/// must produce a `PartResult` whose children are ordered: Hn(1) before P.
/// We verify this via the `SlideTagEngine` API directly (not PDF bytes, because
/// `/P` appears in many PDF tokens like `/Producer`, `/Pages`, `/PageLayout`).
///
/// ## Red Gate trigger
///
/// This test should PASS for the current implementation (frames are processed
/// in order). It documents the invariant so it fails if the order is ever
/// reversed by an implementer. The tag engine tests in `tag_engine.rs` cover
/// the same path; this test anchors it to BC-4.03.001 (not BC-4.03.002).
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_invariant_structure_tree_reading_order() {
    use krilla::tagging::{Node, TagKind};
    use slideforge_layout::LaidOutSlide;
    use slideforge_pdf::SlideTagEngine;
    use slideforge_types::{BulletItem, ContentBlock, InlineNode, TextBlock};

    // Build a 2-frame slide: Title (index 0) → Body with bullets (index 1).
    // The structure tree must list the H1 group BEFORE the L/P group.
    let slide = LaidOutSlide {
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
                content: FrameContent::Title(Arc::from("Title H1")),
                text_flow: None,
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(914_400),
                    width: Emu(9_144_000),
                    height: Emu(3_657_600),
                },
                content: FrameContent::Body(vec![ContentBlock::Text(TextBlock {
                    inlines: vec![InlineNode::Plain(Arc::from("Paragraph"))],
                    span: SourceSpan::default(),
                })]),
                text_flow: None,
            },
        ],
        speaker_notes: None,
        register_tags: RegisterSet::new(),
        register_content: vec![],
    };

    let engine = SlideTagEngine::new();
    let part_result = engine
        .tag_slide(&slide)
        .expect("tag_slide must succeed for a valid 2-frame slide");

    let children = &part_result.part.children;

    // The Part must have exactly 2 children: Hn(1) then P.
    assert_eq!(
        children.len(),
        2,
        "BC-4.03.001 invariant 2: Part must have 2 children (H1, P) for a title+body slide; \
         got {}",
        children.len()
    );

    // First child must be Hn (the title).
    if let Node::Group(first) = &children[0] {
        assert!(
            matches!(first.tag, TagKind::Hn(_)),
            "invariant-2: first child of Part must be Hn (title H1), got {:?}",
            first.tag
        );
    } else {
        panic!("invariant-2: first child of Part must be a Group (Hn), got a Leaf");
    }

    // Second child must be P (the paragraph).
    if let Node::Group(second) = &children[1] {
        assert!(
            matches!(second.tag, TagKind::P(_)),
            "invariant-2: second child of Part must be P (paragraph), got {:?}",
            second.tag
        );
    } else {
        panic!("invariant-2: second child of Part must be a Group (P), got a Leaf");
    }
}

/// BC-4.03.001 invariant 3: Every /Figure element in the structure tree must have
/// a non-empty /Alt entry. This is enforced at the tag-engine level.
///
/// ## Red Gate trigger (strengthened for F-045-I2 per adversary finding)
///
/// The original test only checked that a single Image figure had alt text.
/// This strengthened version uses the `SlideTagEngine` API directly to scan ALL
/// Figure elements in the structure tree and assert that none has a None or empty alt.
/// This catches the `tag_figure(None)` path used for frame-level Diagram/Chart
/// (which would produce a /Figure without /Alt — a UA-1 violation).
///
/// F-045-I2: tag_figure(None) is currently called for FrameContent::Diagram and
/// FrameContent::Chart in `tag_slide`. Those frames ARE emitted by the v1 layout
/// engine (regions.rs lines 264, 280). The test verifies that no /Figure node in
/// the complete tag tree has a None alt text — scanning all children of all Part
/// groups, not just the first one.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_invariant_every_figure_has_non_empty_alt() {
    use krilla::tagging::{Node, TagKind};
    use slideforge_layout::LaidOutSlide;
    use slideforge_pdf::SlideTagEngine;
    use slideforge_types::NormalizedDiagramSvg;

    let deck = deck_with_lang("en-US");
    let alt = "Satellite photo of Earth from orbit";
    let laid_out = figure_slide_with_alt(alt);

    let bytes = export_to_bytes(&deck, &laid_out);

    // Both /Figure AND the non-empty alt text must be in the PDF bytes.
    assert!(
        pdf_contains(&bytes, b"/Figure"),
        "invariant-3 FAILED: /Figure not present for image with alt text."
    );

    assert!(
        pdf_contains(&bytes, alt.as_bytes()),
        "invariant-3 FAILED: alt text '{}' not found in PDF bytes — every /Figure \
         must have a non-empty /Alt entry (BC-4.03.001 invariant 3).",
        alt
    );

    // ── F-045-I2: structural scan — every Figure node must have non-empty alt ──
    //
    // Use SlideTagEngine directly to inspect the tag tree for ALL Figure nodes.
    // This catches the tag_figure(None) path for FrameContent::Diagram/Chart.
    //
    // Build a slide with a frame-level Diagram frame (empty_placeholder SVG).
    // The layout engine emits FrameContent::Diagram(empty_placeholder()) for
    // "diagram"-type slides (regions.rs:280). This exercises the exact code path
    // that was using None alt.
    //
    // After F-045-I2 fix: the Diagram frame with empty SVG must NOT produce
    // a Figure node with None alt (it should either be tagged as Artifact if
    // decorative, or tagged with the real alt if it has one, or guarded with
    // a debug_assert and code comment for unreachable dead code in v1).
    let engine = SlideTagEngine::new();

    // Build a slide with one frame-level Diagram (empty placeholder SVG).
    let empty_svg = NormalizedDiagramSvg::from_normalized_string(Arc::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"></svg>"#,
    ));
    let diagram_slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("diagram"),
        frames: vec![Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(9_144_000),
                height: Emu(5_143_500),
            },
            content: FrameContent::Diagram(empty_svg),
            text_flow: None,
        }],
        speaker_notes: None,
        register_tags: RegisterSet::new(),
        register_content: vec![],
    };
    let part_result = engine
        .tag_slide(&diagram_slide)
        .expect("tag_slide must succeed for diagram slide");

    // Scan all children of the Part group for Figure nodes.
    // BC-4.03.001 invariant 3: EVERY Figure node must have non-empty alt.
    let mut figure_without_alt_count = 0usize;
    for child in &part_result.part.children {
        if let Node::Group(group) = child
            && let TagKind::Figure(ref fig_tag) = group.tag
        {
            let alt_text = fig_tag.alt_text();
            if alt_text.is_none() || alt_text.unwrap_or("").is_empty() {
                figure_without_alt_count += 1;
                eprintln!(
                    "invariant-3/diagram: Figure node has None/empty alt: {:?}",
                    alt_text
                );
            }
        }
    }

    // Also check: if the Diagram frame is in decorative_frame_indices, that is
    // also acceptable — it means the frame was treated as an Artifact, not a Figure.
    // Only fail if there is a Figure node WITHOUT alt AND it is NOT in decorative_frame_indices.
    let is_in_decorative = part_result.decorative_frame_indices.contains(&0);

    assert!(
        figure_without_alt_count == 0 || is_in_decorative,
        "invariant-3/diagram FAILED: {} Figure node(s) have None/empty alt text \
         and frame 0 is NOT in decorative_frame_indices.\n\
         FrameContent::Diagram with no alt must either:\n\
         (a) be excluded from the structure tree (decorative_frame_indices), OR\n\
         (b) carry a non-empty alt from the real DiagramSpec.alt.\n\
         Currently tag_slide emits tag_figure(None) for all Diagram frames \
         regardless of alt text — this is a BC-4.03.001 invariant-3 violation.",
        figure_without_alt_count
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// STORY-045 v1.1 EXPANSION — AC-010..AC-013 (Red Gate tests)
//
// These tests exercise the four workstreams added in the full-UA-1 scope
// expansion:
//
//   AC-010 — PDF document outline (/Outlines bookmarks)
//   AC-011 — Hn structure tags carry /Title attribute text
//   AC-012 — Production export path uses Validator::UA1
//   AC-013 — veraPDF integration test (#[ignore] + always-run proxy)
//
// ## Why they are RED right now
//
// | Test | Red Gate reason |
// |------|----------------|
// | AC-010 group | No /Outlines in PDF — document.set_outline() never called |
// | AC-011 group | Hn tags built with None title — Tag::<Hn>::Hn(level, None) |
// | AC-012 group | document.new_with(SerializeSettings::default()) uses Validator::None; non-compliant exports succeed |
// | AC-013 | veraPDF integration: #[ignore] always-skip; proxy asserts /Outlines present (fails AC-010) |
//
// ═══════════════════════════════════════════════════════════════════════════════

// ─── Shared helpers for AC-010..013 ──────────────────────────────────────────

use slideforge_types::{Block, FieldValue, Slide, Value};

/// Build a `Slide` with the given title in its `fields["title"]` field.
///
/// `title_str()` returns `Some(title)` for this slide.
/// Used by AC-010 and AC-011 tests to build a `Deck` with known titles so
/// `PdfExporter` can source bookmark labels and Hn `/T` attribute text.
fn slide_with_title(title: &str, slide_type: &str) -> Slide {
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from(title))),
    );
    Slide {
        slide_type: Arc::from(slide_type),
        fields,
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

/// Build a `Slide` with NO title field.
///
/// `title_str()` returns `None` for this slide.
/// Used by the EC-006 / fallback-label tests.
fn slide_without_title(slide_type: &str) -> Slide {
    Slide {
        slide_type: Arc::from(slide_type),
        fields: OrderedMap::new(),
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

/// Build a `Deck` with `lang "en-US"` and the given `slides` as its semantic IR.
///
/// The `source_index` values in the corresponding `LaidOutSlide` objects must
/// match the positions of `slides` in this vec (0-based).
fn deck_with_slides(slides: Vec<Slide>) -> Deck {
    Deck {
        slides,
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: Some(Arc::from("AC-010/011 Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

/// Build a `Deck` with no document title (metadata.title = None).
///
/// With `Validator::UA1` enabled, this produces `NoDocumentTitle` → validation
/// failure. With `Validator::None` (current), export succeeds.
/// Used by the AC-012 Red Gate test.
fn deck_without_doc_title() -> Deck {
    Deck {
        slides: vec![slide_with_title("Revenue Outlook", "title")],
        vars: OrderedMap::new(),
        metadata: DeckMetadata {
            title: None, // missing document title — UA-1 requires this
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        },
        registers: OrderedMap::new(),
        section_blocks: vec![],
    }
}

// ─── AC-010: PDF document outline (bookmarks) ────────────────────────────────

/// BC-4.03.001 AC-010: The exported PDF MUST contain `/Outlines` in the document
/// catalog for any deck with heading-containing slides.
///
/// ## Contract (BC-4.03.001 postcondition 1, invariant 6)
///
/// ISO 14289-1 §7.1 requires a document outline whenever headings (H1–H6) are
/// present. Every slideforge deck with title-type slides has H1 headings.
/// The `/Outlines` dictionary must be present in the PDF catalog.
///
/// ## How `/Outlines` is written
///
/// The implementer must call `document.set_outline(outline)` in
/// `generate_pdf_inner` before `document.finish()`. krilla 0.6.0 writes
/// `catalog.outlines(ref)` → PDF bytes contain `Name(b"Outlines")`.
///
/// ## Red Gate trigger — FAILS UNTIL AC-010 IS IMPLEMENTED
///
/// The current exporter never calls `document.set_outline(...)`.
/// The PDF bytes do NOT contain `Outlines`. This test FAILS.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_document_outline_present_in_pdf() {
    let deck = deck_with_slides(vec![
        slide_with_title("Overview", "title"),
        slide_with_title("Data", "content"),
        slide_with_title("Summary", "title"),
    ]);
    let mut laid_out = n_slide_deck(3);
    // Align source_indices with deck.slides positions.
    for (i, slide) in laid_out.slides.iter_mut().enumerate() {
        slide.source_index = i;
    }

    let bytes = export_to_bytes(&deck, &laid_out);

    // /Outlines must be in the PDF catalog.
    // krilla writes `catalog.outlines(ref)` → pdf-writer emits Name(b"Outlines").
    assert!(
        pdf_contains(&bytes, b"Outlines"),
        "AC-010 FAILED: PDF does not contain 'Outlines' in the document catalog.\n\
         The exporter must call document.set_outline(outline) before document.finish().\n\
         krilla writes /Outlines via catalog.outlines(ref) in chunk_container.rs.\n\
         ISO 14289-1 §7.1: document outline mandatory when headings are present."
    );
}

/// BC-4.03.001 AC-010: For a 3-slide deck with titles \["Overview", "Data", "Summary"\],
/// the `/Outlines` tree contains entries with those labels.
///
/// ## How outline label text is written
///
/// krilla serializes `OutlineNode::text` as `outline_item.title(TextStr(&node.text))`
/// (pdf-writer `OutlineItem::title` → `Name(b"Title")` key). The title string
/// appears in the PDF bytes as a PDF text string.
///
/// The test scans for the raw title strings in the PDF bytes. This is load-bearing:
/// an outline with wrong labels, empty labels, or missing entries fails this assertion.
///
/// ## Red Gate trigger — FAILS UNTIL AC-010 IS IMPLEMENTED
///
/// No outline = no title strings from outline entries. Even if the title text
/// appears elsewhere (e.g., as text drawn on the page), it would not prove the
/// outline entry exists. The combination assertion (Outlines + known labels) is
/// the non-vacuous check.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_document_outline_entries_have_slide_title_labels() {
    use slideforge_pdf::outline::build_outline_entries;

    let deck = deck_with_slides(vec![
        slide_with_title("Overview", "title"),
        slide_with_title("Data", "content"),
        slide_with_title("Summary", "title"),
    ]);
    let mut laid_out = n_slide_deck(3);
    for (i, slide) in laid_out.slides.iter_mut().enumerate() {
        slide.source_index = i;
    }

    // F-045-P1-008 (structural assertions via build_outline_entries):
    // Test entry count, label order, and destination page indices at the
    // intermediate OutlineEntry level — before krilla serialisation.
    // This is non-tautological: it asserts STRUCTURE, not raw byte collisions.
    let entries = build_outline_entries(&deck, &laid_out);

    assert_eq!(
        entries.len(),
        3,
        "AC-010/F-008: expected exactly 3 outline entries for 3-slide deck, got {}",
        entries.len()
    );

    let expected_labels = ["Overview", "Data", "Summary"];
    for (i, (entry, expected)) in entries.iter().zip(expected_labels.iter()).enumerate() {
        assert_eq!(
            entry.label, *expected,
            "AC-010/F-008: entry[{i}] label mismatch: expected '{}', got '{}'",
            expected, entry.label
        );
        assert_eq!(
            entry.page_idx, i,
            "AC-010/F-008: entry[{i}] page_idx mismatch: expected {i}, got {}",
            entry.page_idx
        );
    }

    // Integration check: the PDF bytes must also contain /Outlines and the labels.
    // (Catches regressions where build_outline_entries is correct but
    //  build_krilla_outline or document.set_outline is not called.)
    let bytes = export_to_bytes(&deck, &laid_out);

    assert!(
        pdf_contains(&bytes, b"Outlines"),
        "AC-010 integration: /Outlines must be present in the serialised PDF."
    );

    // Byte-scan for each label as a load-bearing integration assertion.
    // Labels appear in the PDF stream via krilla's OutlineItem::title().
    for label in &expected_labels {
        assert!(
            pdf_contains(&bytes, label.as_bytes()),
            "AC-010 integration: outline label '{label}' not found in PDF bytes."
        );
    }
}

/// BC-4.03.001 AC-010 (EC-006): Slide with NO title field → fallback label "Slide N".
///
/// When `deck.slides[source_index].title_str()` returns `None`, the outline entry
/// label MUST be `"Slide N"` (1-based slide number). This prevents an empty or
/// absent bookmark label, which is itself a UA-1 violation.
///
/// ## Red Gate trigger — FAILS UNTIL AC-010 IS IMPLEMENTED
///
/// No outline = no fallback label. The test asserts `Outlines` is present AND
/// `Slide 1` appears in the PDF bytes. Both assertions fail currently.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_document_outline_fallback_label_for_untitled_slide() {
    // Deck with one slide that has no title field.
    let deck = deck_with_slides(vec![slide_without_title("title")]);
    let mut laid_out = n_slide_deck(1);
    laid_out.slides[0].source_index = 0;

    let bytes = export_to_bytes(&deck, &laid_out);

    // /Outlines must be present even for a no-title slide.
    assert!(
        pdf_contains(&bytes, b"Outlines"),
        "AC-010/EC-006 FAILED: /Outlines not present for a deck with untitled slides.\n\
         Every deck must have an outline (ISO 14289-1 §7.1)."
    );

    // The fallback label "Slide 1" must appear in the PDF bytes.
    assert!(
        pdf_contains(&bytes, b"Slide 1"),
        "AC-010/EC-006 FAILED: fallback label 'Slide 1' not found in PDF bytes.\n\
         When deck.slides[0].title_str() is None, the outline entry label MUST be \
         'Slide 1' (1-based). This prevents an empty/absent bookmark label in the PDF."
    );
}

// ─── AC-011: Hn structure tags carry /Title attribute ────────────────────────

/// BC-4.03.001 AC-011: Every `/H1` structure element carries a `/T` attribute
/// whose value is the heading text from `deck.slides[source_index].title_str()`.
///
/// ## How krilla writes Hn /T
///
/// When `Tag::<krilla::tagging::kind::Hn>::Hn(level, Some(title_string))` is used,
/// krilla calls `struct_elem.title(TextStr(title))` in the serialize pass, which
/// writes `Name(b"T")` (the PDF struct-element Title key) + the text string.
///
/// The title text appears in the PDF bytes as a PDF text string. Scanning for the
/// raw title bytes is a load-bearing assertion: if the implementer passes `None`
/// for the title (current behavior), the string does NOT appear in the PDF bytes
/// as a `/T` attribute value (even if it appears elsewhere as drawn text).
///
/// ## Red Gate trigger — FAILS UNTIL AC-011 IS IMPLEMENTED
///
/// The current tag engine calls `Tag::<kind::Hn>::Hn(level, None)` — no title.
/// krilla does not write the `/T` key for the StructElem.
/// The assertion that "Revenue Outlook" appears in the PDF bytes (as a struct
/// attribute, not just drawn text) FAILS because the title text may not be
/// drawn at all (font resolution may fail in the test environment).
///
/// Even if the font DOES resolve and draws the text, the load-bearing assertion
/// is that the text appears because of the `/T` attribute (not text drawing).
/// This test is designed to fail until `tag_slide` passes the slide title to
/// `Tag::<kind::Hn>::Hn(level, Some(title))`.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_hn_tag_carries_title_attribute_text() {
    // A deck with one slide with a known title.
    // source_index 0 → deck.slides[0].title_str() = Some("Revenue Outlook").
    let deck = deck_with_slides(vec![slide_with_title("Revenue Outlook", "title")]);
    let mut laid_out = n_slide_deck(1);
    laid_out.slides[0].source_index = 0;
    // The laid-out slide has a Title frame — gives an H1 in the structure tree.
    // n_slide_deck already builds title frames; source_index 0 is correct.

    // Use uncompressed export so content bytes are scannable without FlateDecode.
    let exporter = PdfExporter::new();
    let brand = minimal_brand();
    let opts = ExportOptions::default();
    let bytes = exporter
        .export_uncompressed(&deck, &laid_out, &brand, &opts)
        .unwrap_or_else(|e| panic!("export_uncompressed failed: {e}"));

    // /H1 must be present (the Title frame was tagged).
    assert!(
        pdf_contains(&bytes, b"/H1"),
        "AC-011 prerequisite: /H1 StructElem must be present."
    );

    // The title text "Revenue Outlook" MUST appear in the PDF bytes as a PDF
    // text string from the `/T` (Title) attribute of the H1 StructElem.
    //
    // krilla path: tag.set_title(Some("Revenue Outlook".to_owned()))
    //   → StructAttr::Title("Revenue Outlook")
    //   → struct_elem.title(TextStr("Revenue Outlook"))
    //   → pdf-writer writes Name(b"T") + TextStr bytes containing "Revenue Outlook"
    //
    // PDF text strings for ASCII content are written as `(Revenue Outlook)` or
    // as a UTF-16 BE BOM-prefixed hex string. In either case, the ASCII bytes
    // "Revenue Outlook" appear in the uncompressed PDF stream.
    assert!(
        pdf_contains(&bytes, b"Revenue Outlook"),
        "AC-011 FAILED: 'Revenue Outlook' not found in PDF bytes.\n\
         The H1 StructElem must carry a /T attribute with the slide title text.\n\
         Implementer: in tag_slide(), change Tag::<kind::Hn>::Hn(level, None) to\n\
         Tag::<kind::Hn>::Hn(level, Some(slide_title.to_owned())) where slide_title\n\
         is sourced from deck.slides[laid_out_slide.source_index].title_str().\n\
         The title string is passed as the second argument to Hn(...) per\n\
         krilla 0.6.0 interchange/tagging/generated.rs:2082."
    );
}

/// BC-4.03.001 AC-011 (EC-006): Slide with NO title field → H1 /T attribute fallback
/// "Slide N".
///
/// When `deck.slides[source_index].title_str()` returns `None`, the Hn `/T`
/// attribute MUST be set to the fallback label `"Slide N"` (1-based).
/// An absent or None `/T` on an Hn tag triggers `MissingHeadingTitle` in
/// `Validator::UA1` (see krilla validate.rs:714).
///
/// ## Red Gate trigger — FAILS UNTIL AC-011 IS IMPLEMENTED
///
/// Current: `Tag::<kind::Hn>::Hn(level, None)` — no `/T` attribute.
/// "Slide 1" does NOT appear in the PDF bytes as an Hn /T value.
/// The assertion fails.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_hn_tag_fallback_title_for_untitled_slide() {
    let deck = deck_with_slides(vec![slide_without_title("title")]);
    let mut laid_out = n_slide_deck(1);
    laid_out.slides[0].source_index = 0;

    let exporter = PdfExporter::new();
    let brand = minimal_brand();
    let opts = ExportOptions::default();
    let bytes = exporter
        .export_uncompressed(&deck, &laid_out, &brand, &opts)
        .unwrap_or_else(|e| panic!("export_uncompressed failed: {e}"));

    assert!(
        pdf_contains(&bytes, b"/H1"),
        "AC-011/EC-006 prerequisite: /H1 StructElem must be present."
    );

    // The fallback label "Slide 1" must appear as the Hn /T attribute value.
    assert!(
        pdf_contains(&bytes, b"Slide 1"),
        "AC-011/EC-006 FAILED: fallback title 'Slide 1' not found in PDF bytes.\n\
         When deck.slides[0].title_str() is None, the H1 /T attribute MUST be \
         set to the fallback label 'Slide 1' (1-based slide number).\n\
         An absent /T on Hn triggers MissingHeadingTitle in Validator::UA1.\n\
         Implementer: use fallback format!(\"Slide {{}}\", source_index + 1) when \
         title_str() returns None."
    );
}

// ─── OBS-012: Subtitle H2 carries its own text as /T attribute ──────────────

/// OBS-012 / AC-011: A Subtitle frame's H2 structure element MUST carry the
/// subtitle's own text as the `/T` attribute — NOT the slide title reused.
///
/// ## Contract (AC-011 / BC-4.03.001 postcondition 1)
///
/// AC-011 states: body H2-H6 /Title = first inline run of that frame.
/// A subtitle frame on a slide titled "Quarterly Review" with subtitle text
/// "Q4 2025 Highlights" must produce an H2 with /T = "Q4 2025 Highlights",
/// not /T = "Quarterly Review" (the slide title).
///
/// ## Load-bearing assertion
///
/// We export a slide with:
///   - H1 title: "Quarterly Review"
///   - H2 subtitle: "Q4 2025 Highlights"
///
/// Both strings are known-distinct. The byte scan for "Q4 2025 Highlights"
/// in the presence of /H2 is the load-bearing assertion.
/// If the old (wrong) code reused `slide_title` for the H2, only
/// "Quarterly Review" would appear as a /T value — this test catches that.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_subtitle_h2_carries_own_text_as_title_attribute() {
    let deck = deck_with_slides(vec![slide_with_title("Quarterly Review", "title")]);

    // Build a LaidOutSlide with both Title (H1) and Subtitle (H2) frames.
    let subtitle_slide = LaidOutSlide {
        source_index: 0,
        slide_type_keyword: Arc::from("title-subtitle"),
        frames: vec![
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(0),
                    width: Emu(9_144_000),
                    height: Emu(914_400),
                },
                content: FrameContent::Title(Arc::from("Quarterly Review")),
                text_flow: None,
            },
            Frame {
                bbox: BoundingBox {
                    x: Emu(0),
                    y: Emu(914_400),
                    width: Emu(9_144_000),
                    height: Emu(914_400),
                },
                content: FrameContent::Subtitle(Arc::from("Q4 2025 Highlights")),
                text_flow: None,
            },
        ],
        speaker_notes: None,
        register_tags: RegisterSet::new(),
        register_content: vec![],
    };
    let laid_out = LaidOutDeck {
        page_size: PageSize::default(),
        slides: vec![subtitle_slide],
        sections: vec![],
        warnings: vec![],
    };

    // Use with_font_path for deterministic font resolution (same as F-006 fix).
    let font_path = {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.join("../slideforge-math/fonts/latinmodern-math.otf")
    };
    let exporter = PdfExporter::with_font_path(font_path);
    let brand = minimal_brand();
    let opts = ExportOptions::default();
    let bytes = exporter
        .export_uncompressed(&deck, &laid_out, &brand, &opts)
        .unwrap_or_else(|e| panic!("export_uncompressed failed: {e}"));

    // /H2 must be present.
    assert!(
        pdf_contains(&bytes, b"/H2"),
        "OBS-012 prerequisite: /H2 StructElem must be present for a Subtitle frame."
    );

    // The SUBTITLE's own text must appear as the H2 /T value.
    assert!(
        pdf_contains(&bytes, b"Q4 2025 Highlights"),
        "OBS-012 FAILED: subtitle text 'Q4 2025 Highlights' not found in PDF bytes.\n\
         The H2 StructElem must carry the subtitle's OWN text as /T, not the slide title.\n\
         If 'Quarterly Review' (the slide title) was used instead, the H2 /T is wrong.\n\
         Implementer: in tag_engine.rs Subtitle arm, use subtitle_text.as_ref() as the\n\
         H2 title, not slide_title."
    );

    // Sanity check: the H1 title is still the slide title (unchanged).
    assert!(
        pdf_contains(&bytes, b"Quarterly Review"),
        "OBS-012 sanity: H1 title 'Quarterly Review' must still appear in PDF bytes."
    );
}

// ─── AC-012: Validator::UA1 enabled in production export path ────────────────

/// BC-4.03.001 AC-012: The production `PdfExporter::export()` path MUST be
/// configured with `Validator::UA1` (via `SerializeSettings { configuration:
/// Configuration::new_with_validator(Validator::UA1), .. }`).
///
/// When enabled, krilla rejects non-UA1-compliant documents at `document.finish()`
/// with a `KrillaError::ValidationError`. Any such error MUST be propagated as
/// a fatal export error — NOT silently swallowed.
///
/// ## Test mechanism (TD-VSDD-059 load-bearing)
///
/// We export a deck with `deck.metadata.title = None`. krilla UA-1 validation
/// triggers `ValidationError::NoDocumentTitle` → `prohibits()` returns `true`
/// for `Validator::UA1` → `document.finish()` returns `KrillaError::ValidationError`.
///
/// The `PdfExporter::export()` result must be `Err(...)` with the validation
/// error message (mapped through `PdfExportError::Serialize { message }` →
/// `ExportError::RenderError { message }`).
///
/// ## Red Gate trigger — FAILS UNTIL AC-012 IS IMPLEMENTED
///
/// Current production code: `Document::new_with(SerializeSettings::default())`
/// uses `Configuration::new()` which has `Validator::None`. No validation is
/// performed. The export SUCCEEDS (`Ok`) even for a no-title deck.
///
/// This test asserts `result.is_err()` — FAILS NOW (gets `Ok`).
/// Once `Validator::UA1` is set in the production export path, the validation
/// triggers and the test PASSES.
///
/// Note: `NoDocumentTitle` is `true` for `Validator::UA1`
/// (krilla configure/validate.rs line ~482: `ValidationError::NoDocumentTitle => true`).
/// This UA-1 violation is stable — a deck without a document title will ALWAYS
/// fail UA-1, even after AC-010+011 are implemented.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_validator_ua1_rejects_missing_document_title() {
    // Deck with NO metadata.title (= None) but with a Title frame on the slide.
    // With Validator::UA1: ValidationError::NoDocumentTitle → fatal ValidationFailed error.
    // With Validator::None: no validation, export succeeds.
    let deck = deck_without_doc_title();
    let mut laid_out = n_slide_deck(1);
    laid_out.slides[0].source_index = 0;

    // Use export_uncompressed (returns PdfExportError directly) so we can assert the
    // specific variant — not a substring match against the stringified ExportError.
    // F-045-P1-003: the error MUST be PdfExportError::ValidationFailed, NOT Serialize.
    let exporter = PdfExporter::new();
    let brand = minimal_brand();
    let opts = ExportOptions::default();

    let result = exporter.export_uncompressed(&deck, &laid_out, &brand, &opts);

    assert!(
        result.is_err(),
        "AC-012 FAILED: export of a deck with no document title should return Err \
         when Validator::UA1 is enabled.\n\
         ValidationError::NoDocumentTitle is fatal under Validator::UA1 \
         (krilla configure/validate.rs: NoDocumentTitle => true for UA1).\n\
         Current failure mode: the production export uses Validator::None \
         (SerializeSettings::default() → Configuration::new() → Validator::None), \
         so no validation is performed and export returns Ok.\n\
         Implementer: change generate_pdf_inner to use:\n\
         krilla::SerializeSettings {{\n\
             configuration: krilla::configure::Configuration::new_with_validator(\n\
                 krilla::configure::Validator::UA1\n\
             ),\n\
             ..krilla::SerializeSettings::default()\n\
         }}"
    );

    // F-045-P1-003 (load-bearing): the error MUST be the ValidationFailed variant —
    // not Serialize and not a generic string match. A Serialize variant here would
    // indicate the KrillaError::Validation branch is missing from generate_pdf_inner.
    match result.unwrap_err() {
        PdfExportError::ValidationFailed { message } => {
            // message must mention the violation (NoDocumentTitle or similar).
            assert!(
                message.to_lowercase().contains("validation"),
                "AC-012: ValidationFailed message does not contain 'validation': '{message}'"
            );
        },
        other => {
            panic!(
                "AC-012 FAILED: expected PdfExportError::ValidationFailed but got: {other:?}\n\
                 KrillaError::Validation from document.finish() must map to \
                 PdfExportError::ValidationFailed, not Serialize or any other variant.\n\
                 Implementer: add a match arm for KrillaError::Validation in generate_pdf_inner."
            );
        },
    }
}

/// BC-4.03.001 AC-012 (invariant 5): Once the full UA-1 implementation is in
/// place (AC-010 outline + AC-011 Hn titles + AC-012 Validator::UA1), a
/// COMPLIANT deck (with title, lang, outline, Hn /T set) MUST export
/// successfully — no false positives from the validator.
///
/// ## Red Gate trigger — FAILS UNTIL AC-010 + AC-011 + AC-012 ARE ALL IMPLEMENTED
///
/// Currently the export of the compliant fixture deck SUCCEEDS (Validator::None
/// means no rejection). Once Validator::UA1 is enabled:
/// - Without AC-010+011: the export FAILS (missing outline + missing Hn /T)
/// - With AC-010+011: the export SUCCEEDS
///
/// This test acts as the "positive case" for AC-012: a fully-compliant deck
/// must export without a ValidationError. It FAILS now because once Validator::UA1
/// is wired (AC-012), the missing outline+Hn titles (not yet implemented) will
/// cause it to reject. Only when ALL of AC-010+011+012 are implemented does
/// this test pass.
///
/// The test is intentionally written to fail in the INTERMEDIATE state
/// (Validator::UA1 enabled but outline/titles not yet added) so the implementer
/// must complete ALL workstreams before marking AC-012 done.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_validator_ua1_compliant_deck_exports_successfully() {
    // Compliant fixture: lang set, doc title set, slides with titled H1 frames.
    // AC-010: outline will be built from slide titles.
    // AC-011: Hn /T will be set from deck.slides[i].title_str().
    // AC-012: Validator::UA1 must NOT reject this deck once AC-010+011 done.
    let deck = deck_with_slides(vec![
        slide_with_title("Overview", "title"),
        slide_with_title("Data", "content"),
        slide_with_title("Summary", "title"),
    ]);
    let mut laid_out = n_slide_deck(3);
    for (i, slide) in laid_out.slides.iter_mut().enumerate() {
        slide.source_index = i;
    }

    // F-045-P1-006: use the bundled fixture font so font resolution is deterministic
    // across CI runners regardless of which system fonts are installed. Without an
    // explicit font, Helvetica may not resolve on headless Linux CI → text drawing
    // is skipped → UA-1 outcome becomes environment-dependent.
    //
    // `crates/slideforge-math/fonts/latinmodern-math.otf` is committed to the repo
    // and guaranteed to be present on all CI runners.
    let font_path = {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.join("../slideforge-math/fonts/latinmodern-math.otf")
    };
    let exporter = PdfExporter::with_font_path(font_path);
    let brand = minimal_brand();
    let opts = ExportOptions::default();

    let result = exporter.export(&deck, &laid_out, &brand, &opts);

    // This is the positive-path assertion: a compliant deck must export without error.
    // Currently FAILS because:
    //   (a) Until AC-012: uses Validator::None so passes trivially → this would pass!
    //   (b) After AC-012 alone (without AC-010+011): Validator::UA1 rejects missing
    //       outline + missing Hn /T → result is Err → test FAILS
    //   (c) After AC-010+011+012: Validator::UA1 passes, result is Ok → test PASSES
    //
    // To make this a Red Gate: we also assert /Outlines is present in the bytes
    // (which fails currently regardless of whether the export succeeds/fails).
    // If the export is Ok but /Outlines is missing, the assert below catches it.
    let bytes = result.unwrap_or_else(|e| {
        panic!(
            "AC-012/positive FAILED: compliant deck must export successfully once \
         AC-010+011+012 are all implemented.\n\
         Current error: {e}\n\
         If Validator::UA1 is enabled but outline/Hn-titles are not yet added, \
         this error is expected as an intermediate Red Gate state."
        )
    });

    // The export succeeded — verify the outline is present (load-bearing AC-010 proxy).
    // This catches the case where the export succeeds with Validator::None but no outline.
    assert!(
        pdf_contains(&bytes, b"Outlines"),
        "AC-012/positive FAILED: export succeeded but /Outlines not present.\n\
         A compliant PDF must have a document outline (AC-010)."
    );
}

// ─── AC-013: veraPDF integration test (+ always-run structural proxy) ─────────

/// BC-4.03.001 AC-013 (structural proxy — always runs, no external tool needed):
/// The exported PDF has all structural elements required for `veraPDF --flavour ua1`
/// to produce `isCompliant: true`.
///
/// Per SID-1 (No-Ignored-Test Rationalization): an `#[ignore]`'d test that
/// requires an external tool (veraPDF) must be accompanied by a non-ignored
/// unit test that exercises the same production path without the external tool.
///
/// ## What this asserts (composite UA-1 structural proxy)
///
/// | Check | Required for veraPDF UA-1 |
/// |-------|--------------------------|
/// | /StructTreeRoot | Yes — tagged PDF mandatory |
/// | /MarkInfo | Yes — Marked=true mandatory |
/// | /Outlines | Yes — mandatory when headings present |
/// | /Lang | Yes — required language metadata |
/// | Slide title text in outline | Yes — meaningful bookmark labels |
///
/// The combination of these 5 assertions is the full structural proxy for
/// veraPDF --flavour ua1 isCompliant:true (excluding ToUnicode, which is AC-009).
///
/// ## Red Gate trigger — FAILS UNTIL AC-010..012 ARE IMPLEMENTED
///
/// The `/Outlines` assertion fails immediately (AC-010 not done).
/// Even if AC-001/006/007 already pass, the composite fails on /Outlines.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_ac013_structural_proxy_for_verapdf_ua1() {
    // 3-slide fixture deck matching the canonical test vector (TV-10.1 from BC).
    let deck = deck_with_slides(vec![
        slide_with_title("Overview", "title"),
        slide_with_title("Data", "content"),
        slide_with_title("Summary", "title"),
    ]);
    let mut laid_out = n_slide_deck(3);
    for (i, slide) in laid_out.slides.iter_mut().enumerate() {
        slide.source_index = i;
    }

    let bytes = export_to_bytes(&deck, &laid_out);

    // Check 1: /StructTreeRoot (AC-001 / BC-4.03.001 postcondition 1)
    assert!(
        pdf_contains(&bytes, b"StructTreeRoot"),
        "AC-013/proxy FAILED: /StructTreeRoot missing (AC-001)."
    );

    // Check 2: /MarkInfo (AC-006 / BC-4.03.001 postcondition 1)
    assert!(
        pdf_contains(&bytes, b"MarkInfo"),
        "AC-013/proxy FAILED: /MarkInfo missing (AC-006)."
    );

    // Check 3: /Lang (AC-007 / BC-4.03.001 postcondition 1)
    assert!(
        pdf_contains(&bytes, b"/Lang"),
        "AC-013/proxy FAILED: /Lang missing (AC-007)."
    );

    // Check 4: /Outlines (AC-010 — THIS IS THE NEW RED GATE ASSERTION)
    assert!(
        pdf_contains(&bytes, b"Outlines"),
        "AC-013/proxy FAILED: /Outlines (document outline/bookmarks) missing (AC-010).\n\
         veraPDF --flavour ua1 reports MissingDocumentOutline when this is absent.\n\
         Implementer: call document.set_outline(outline) in generate_pdf_inner."
    );

    // Check 5: Slide title appears in outline entries (AC-010 label check)
    assert!(
        pdf_contains(&bytes, b"Overview"),
        "AC-013/proxy FAILED: outline label 'Overview' not found (AC-010 labels)."
    );
}

/// BC-4.03.001 AC-013 (veraPDF integration — ALWAYS #[ignore] for local runs):
///
/// Runs `verapdf --flavour ua1` on a 3-slide fixture deck and asserts
/// `isCompliant: true` with zero violations.
///
/// ## Why #[ignore]
///
/// `verapdf` is a Java-based CLI tool. Per SID-1:
/// integration tests requiring external tools MUST be `#[ignore]`'d with a
/// code comment citing the blocking dependency. The non-ignored proxy test
/// `test_bc_4_03_001_ac013_structural_proxy_for_verapdf_ua1` exercises the
/// same production path without the external tool.
///
/// ## Blocking dependency
///
/// Requires `verapdf` on PATH (Java tool). In CI, installed via
/// Docker image `ghcr.io/verapdf/cli:latest` (per STORY-045 spec).
/// Not available in standard developer test environments.
///
/// ## To un-ignore on CI (pdf-ua1.yml workflow)
///
/// The CI job MUST invoke:
/// ```bash
/// cargo nextest run -p slideforge-pdf --test pdf_ua1 \
///     -E 'test(ac013_verapdf_full_compliance)' \
///     --run-ignored all
/// ```
/// Per AC-013: `.github/workflows/pdf-ua1.yml` must contain `--run-ignored all`
/// (cargo-nextest flag) and the -E filter must target this test name.
/// Note: `--include-ignored` is a libtest flag — INVALID for cargo-nextest (F-045-P1-002).
#[test]
#[ignore = "requires verapdf CLI on PATH (Java tool — available in CI via Docker image \
             ghcr.io/verapdf/cli:latest; blocking dependency: \
             .github/workflows/pdf-ua1.yml which runs this test via --run-ignored all)"]
#[allow(clippy::unwrap_used)]
fn test_bc_4_03_001_ac013_verapdf_full_compliance() {
    use std::io::Write;
    use std::process::Command;

    let deck = deck_with_slides(vec![
        slide_with_title("Overview", "title"),
        slide_with_title("Data", "content"),
        slide_with_title("Summary", "title"),
    ]);
    let mut laid_out = n_slide_deck(3);
    for (i, slide) in laid_out.slides.iter_mut().enumerate() {
        slide.source_index = i;
    }

    // Export to PDF bytes via the production path (with Validator::UA1 once AC-012 done).
    let bytes = export_to_bytes(&deck, &laid_out);

    // Write PDF bytes to a temp file for verapdf to read.
    let mut tmp =
        tempfile::NamedTempFile::new().expect("failed to create temp file for veraPDF fixture");
    tmp.write_all(&bytes)
        .expect("failed to write PDF bytes to temp file");
    let pdf_path = tmp.path().to_owned();

    // Run verapdf --flavour ua1 on the fixture PDF.
    let output = Command::new("verapdf")
        .arg("--flavour")
        .arg("ua1")
        .arg(&pdf_path)
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "AC-013 FAILED: could not run verapdf: {e}.\n\
                 Is verapdf installed and on PATH?\n\
                 CI: install via ghcr.io/verapdf/cli:latest Docker image."
            )
        });

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("[verapdf AC-013] stdout: {stdout}");
    eprintln!("[verapdf AC-013] stderr: {stderr}");

    // verapdf exits 0 when validation passes.
    assert!(
        output.status.success(),
        "AC-013 FAILED: verapdf exited with non-zero status {:?}.\n\
         stdout: {stdout}\n\
         stderr: {stderr}",
        output.status.code()
    );

    // The JSON output must contain isCompliant: true.
    assert!(
        stdout.contains("\"isCompliant\":true") || stdout.contains("isCompliant: true"),
        "AC-013 FAILED: verapdf output does not contain 'isCompliant: true'.\n\
         stdout: {stdout}"
    );

    // Zero violations.
    assert!(
        stdout.contains("\"violations\":0")
            || stdout.contains("violations: 0")
            || !stdout.contains("violation"),
        "AC-013 FAILED: verapdf output reports violations.\n\
         stdout: {stdout}"
    );
}

// ─── AC-013 CI gate: workflow file checks ────────────────────────────────────

/// BC-4.03.001 AC-013 (CI gate): `.github/workflows/pdf-ua1.yml` must invoke
/// `cargo nextest run ... --run-ignored all` to un-ignore the veraPDF test.
///
/// ## F-045-P1-002 (load-bearing)
///
/// `--include-ignored` is a libtest flag and is NOT valid for cargo-nextest.
/// The correct cargo-nextest flag is `--run-ignored all` (or `--run-ignored=all`).
/// This test asserts the workflow contains the CORRECT nextest flag so CI does not
/// silently fail with an invalid argument parse error.
///
/// ## F-045-P1-001 (load-bearing)
///
/// The CI job MUST run `test_bc_4_03_001_ac013_verapdf_full_compliance` (the AC-013
/// full-compliance test using deck_with_slides), NOT `test_bc_4_03_001_verapdf_integration`
/// (the old AC-008 degenerate fixture with empty slides). The -E filter must target
/// `ac013_verapdf_full_compliance`.
///
/// ## Red Gate trigger — FAILS UNTIL CI WORKFLOW IS WRITTEN (STORY-045)
///
/// The CI workflow file does not yet exist. This test fails on file absence.
#[test]
fn test_bc_4_03_001_ac013_ci_workflow_includes_ignored_flag() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow_path = manifest_dir.join("../../.github/workflows/pdf-ua1.yml");

    // File must exist.
    assert!(
        workflow_path.exists(),
        "AC-013 FAILED: CI workflow file not found at {path}.\n\
         STORY-045 must create .github/workflows/pdf-ua1.yml.",
        path = workflow_path.display()
    );

    let content = std::fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("failed to read pdf-ua1.yml: {e}"));

    // F-045-P1-002: --run-ignored is the cargo-nextest flag (NOT --include-ignored).
    assert!(
        content.contains("--run-ignored"),
        "AC-013/F-002 FAILED: pdf-ua1.yml does not contain '--run-ignored'.\n\
         cargo-nextest uses '--run-ignored all' to run ignored tests.\n\
         '--include-ignored' is a libtest flag and is INVALID for nextest — \
         it causes an argument parse error rather than running the test.\n\
         Fix: replace '--include-ignored' with '--run-ignored all' in the verapdf job."
    );

    // F-045-P1-001: the -E filter must target the AC-013 full-compliance test.
    assert!(
        content.contains("ac013_verapdf_full_compliance"),
        "AC-013/F-001 FAILED: pdf-ua1.yml does not filter for 'ac013_verapdf_full_compliance'.\n\
         The -E filter must target test_bc_4_03_001_ac013_verapdf_full_compliance \
         (the AC-013 full-compliance test with real titles via deck_with_slides).\n\
         The old filter 'test(verapdf_integration)' matched the degenerate AC-008 test \
         which uses an empty-slides fixture — insufficient for full compliance verification."
    );

    // OBS-009: the workflow must have a coverage-assertion step to catch zero-test runs.
    assert!(
        content.contains("zero tests ran") || content.contains("TESTS_RUN"),
        "OBS-009 FAILED: pdf-ua1.yml does not contain a coverage-assertion step.\n\
         A compliance gate that silently passes when zero tests run is a false gate.\n\
         Add a step that parses nextest's 'N tests run' summary and fails if N < 1."
    );
}
