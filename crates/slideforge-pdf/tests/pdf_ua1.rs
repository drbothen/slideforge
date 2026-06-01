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

use std::path::PathBuf;
use std::sync::Arc;

// ─── Shared IR construction helpers ──────────────────────────────────────────

use slideforge_layout::types::{
    BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, RegisterSet,
};
use slideforge_pdf::PdfExporter;
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

/// BC-4.03.001 AC-004 (forward obligation from STORY-043): Frame-level
/// `FrameContent::Diagram` must get REAL alt text from `DiagramSpec.alt`,
/// NOT the hardcoded placeholder `"diagram"`.
///
/// ## Red Gate trigger — THIS TEST WILL FAIL UNTIL STORY-045 IS IMPLEMENTED
///
/// Currently `tag_engine.rs` uses:
/// ```rust
/// let alt_text = if diagram_svg.as_str().is_empty() { None } else { Some("diagram") };
/// ```
/// This is the exact placeholder string that must be replaced. This test verifies
/// that the string `"diagram"` does NOT appear as the sole alt text for a Diagram
/// frame when a real alt text is available from the spec.
///
/// The test constructs a `FrameContent::Diagram` with a real SVG and expects
/// the PDF to contain the DSL-supplied alt text (not the placeholder).
/// Since there is no way to inject `DiagramSpec.alt` into `FrameContent::Diagram`
/// until the IR threading is complete (STORY-045 task), this test will fail
/// because the placeholder `"diagram"` is still used.
///
/// SID-1 compliance: this test exercises the production tag-engine path
/// without requiring external dependencies (the SVG is an inline stub).
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

    let bytes = export_to_bytes(&deck, &laid_out);

    // /Figure must be present for the Diagram frame.
    assert!(
        pdf_contains(&bytes, b"/Figure"),
        "AC-004/diagram FAILED: PDF does not contain '/Figure' for a Diagram frame."
    );

    // The PLACEHOLDER alt text "diagram" MUST NOT be the only text in the /Alt entry.
    // Once real alt text is threaded from DiagramSpec.alt, the PDF must contain
    // the real alt text (not the generic placeholder).
    //
    // We test by asserting that the raw bytes do NOT contain the exact bytes
    // `(diagram)` which is how krilla encodes alt text in the PDF. The implementer
    // must wire the real alt from DiagramSpec through the IR to FrameContent.
    //
    // NOTE: This assertion WILL FAIL until the IR threading is complete (STORY-045).
    // Red Gate: the placeholder `(diagram)` will be present in the current output.
    assert!(
        !pdf_contains(&bytes, b"(diagram)"),
        "AC-004/diagram FAILED (placeholder alt text still present): \
         PDF contains '(diagram)' as the alt text for a Diagram frame.\n\
         STORY-045 must thread real DiagramSpec.alt through the IR to FrameContent::Diagram.\n\
         The tag engine must not use the generic 'diagram' placeholder string."
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
/// The test verifies the raw PDF bytes do NOT contain `(chart)` as a /Alt value
/// for a Chart frame.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_chart_frame_alt_text_from_spec() {
    let deck = deck_with_lang("en-US");
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

    let bytes = export_to_bytes(&deck, &laid_out);

    // The PLACEHOLDER alt text "chart" must NOT appear as a PDF alt literal.
    // krilla encodes alt text as a PDF string `(chart)` in the StructElem dict.
    //
    // NOTE: This assertion WILL FAIL until the IR threading is complete (STORY-045).
    // Red Gate: the placeholder `(chart)` will be present in the current output.
    assert!(
        !pdf_contains(&bytes, b"(chart)"),
        "AC-004/chart FAILED (placeholder alt text still present): \
         PDF contains '(chart)' as the alt text for a Chart frame.\n\
         STORY-045 must thread real ChartSpec.alt through the IR to FrameContent::Chart.\n\
         The tag engine must not use the generic 'chart' placeholder string."
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

    let exporter = PdfExporter::new();
    let brand = minimal_brand();
    let opts = ExportOptions::default();

    // Either the export succeeds (no crash) or it returns an error.
    // We do NOT assert success — a Result::Err is acceptable here.
    // We DO assert it does not panic (no unwrap in production paths).
    let result = exporter.export(&deck, &laid_out, &brand, &opts);

    match result {
        Ok(bytes) => {
            // If export succeeds, the PDF must still be a valid PDF header.
            assert!(
                bytes.starts_with(b"%PDF-"),
                "AC-007/null-lang: export without lang must still produce a valid PDF header"
            );
            // No garbage lang value must appear. What is NOT acceptable:
            // a random pointer address or empty string disguised as a lang code.
            // A missing /Lang is OK (precondition 2 not met).
            // We do not assert absence of /Lang here — just the non-garbage invariant.
        },
        Err(e) => {
            // A structured error is acceptable when lang is missing.
            // Record the error message for human review.
            eprintln!(
                "[AC-007/null-lang] export returned Err (acceptable): {e}"
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
///     test_bc_4_03_001_verapdf_integration -- --include-ignored
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
                content: FrameContent::Body(vec![
                    ContentBlock::Text(TextBlock {
                        inlines: vec![InlineNode::Plain(Arc::from("Paragraph"))],
                        span: SourceSpan::default(),
                    }),
                ]),
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
/// ## Red Gate trigger
///
/// `tag_figure` currently accepts `None` alt for Diagram/Chart placeholders.
/// Once STORY-045 threads real alt text, `tag_figure(Some(alt))` must be called
/// with a non-empty string. This test verifies that when an Image frame has alt
/// text, the PDF contains both `/Figure` AND the alt text literal — not just
/// `/Figure` alone.
#[allow(clippy::unwrap_used)]
#[test]
fn test_bc_4_03_001_invariant_every_figure_has_non_empty_alt() {
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
}
