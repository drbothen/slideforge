//! `slideforge-layout` — Core layout engine for the slideforge pipeline.
//!
//! This crate implements the `Deck → LaidOutDeck` transformation. It accepts a
//! fully evaluated [`slideforge_types::Deck`] and a [`slideforge_types::Brand`]
//! configuration and produces a [`types::LaidOutDeck`] with per-slide
//! positioned content frames in integer EMU coordinates.
//!
//! ## Two-IR Model
//!
//! ```text
//! Deck (semantic, pre-layout)
//!    title: String
//!    slides: Vec<Slide>
//!    vars: HashMap<Arc<str>, Value>
//!         |
//!         | layout::run(deck, brand)
//!         v
//! LaidOutDeck (geometric, post-layout)
//!    page_size: PageSize          -- EMU width × height
//!    slides: Vec<LaidOutSlide>    -- one per Deck.slides entry
//! ```
//!
//! ## Invariants
//!
//! - **Slide count (BC-3.06.001 / VP-011):** `layout::run` preserves slide
//!   count. If the output count does not match the input count, the function
//!   returns `Err(LayoutError::SlideCountMismatch)`.
//! - **Determinism (BC-3.06.002):** Identical inputs always produce identical
//!   output (no random seeds, no non-deterministic data structures).
//! - **Valid EMU coordinates (BC-3.06.003):** Every `BoundingBox` for
//!   non-shape frames (region/placeholder/text-run) satisfies `x >= 0`,
//!   `y >= 0`, `width > 0`, `height > 0`, `x + width <= page_width`,
//!   `y + height <= page_height`. Shape frames may legitimately fall outside
//!   the page edges (`x < 0`, `y < 0`, or extending beyond `page_width`/
//!   `page_height`) per BC-3.04.001 EC-002 and produce
//!   [`LayoutWarning::OffCanvas`]. Shape frames are still required to satisfy
//!   `width > 0` and `height > 0`; violations return
//!   [`crate::error::LayoutError::InvalidBoundingBox`].
//!   Per BC-3.04.001 v1.5.2 Invariant 11 (current version per BC v1.5.2, latest):
//!   when both `alt` text and `decorative: true` are set on a shape, `alt` takes
//!   precedence and the layout result carries `AltText::Provided`.
//! - **Pure function (AC-008):** No I/O, no side effects, no panics.
//!
//! ## Forbidden dependencies
//!
//! This crate must NOT depend on `slideforge-pptx`, `slideforge-docx`,
//! `slideforge-pdf`, `slideforge-html`, `slideforge-preview`, `slideforge-data`,
//! `slideforge-brand`, or `slideforge-cli` (AC-009).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod inline;
pub mod layout;
pub mod regions;
pub mod sections;
pub mod shapes;
pub mod text_flow;
pub mod types;

// Re-export the primary entry point and most-used types at crate root.
pub use error::LayoutError;
pub use layout::run;
pub use sections::{GeneratedSection, OutputFormat, SectionItem, SectionKind, SectionSource};
pub use types::{
    BoundingBox, FillSpec, Frame, FrameContent, LaidOutDeck, LaidOutSlide, LayoutWarning, PageSize,
    RegisterSet, RegisterTag, Rgb, ShapeFrame, ShapeType, TextFlow, TextOverflow,
};

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used
)]
mod tests {
    use std::sync::Arc;

    use slideforge_types::{
        Brand, BrandFonts, BrandPalette, Deck, DeckMetadata, FieldValue, OrderedMap, Register,
        SectionBlock, Slide, SourceSpan, Value,
    };

    use super::*;
    use crate::types::{DEFAULT_PAGE_HEIGHT, DEFAULT_PAGE_WIDTH};

    // ─────────────────────────────────────────────────────────────────────────
    // Test helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn make_metadata() -> DeckMetadata {
        DeckMetadata {
            title: Some(Arc::from("Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        }
    }

    fn make_deck(slides: Vec<Slide>) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
            section_blocks: vec![],
        }
    }

    fn make_slide(slide_type: &str) -> Slide {
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

    fn make_slide_with_title(slide_type: &str, title: &str) -> Slide {
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
                heading: Arc::from("Calibri"),
                body: Arc::from("Calibri"),
                mono: Arc::from("Courier New"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.06.001 — layout::run signature, slide count invariant, empty deck
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-001 — `layout::run` exists and returns `Result<LaidOutDeck, LayoutError>`.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_001_layout_run_returns_result() {
        let deck = make_deck(vec![make_slide("title")]);
        let brand = make_brand();
        // Must return a Result — either Ok or Err (not panic with todo!())
        let _result: Result<LaidOutDeck, LayoutError> = run(&deck, &brand);
    }

    /// AC-002 / BC-3.06.001 — `layout::run` preserves slide count (3 slides → 3 slides).
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_001_layout_run_preserves_slide_count() {
        let slides = vec![
            make_slide("title"),
            make_slide("content"),
            make_slide("blank"),
        ];
        let deck = make_deck(slides);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed for 3 valid slides");
        assert_eq!(
            result.slides.len(),
            3,
            "LaidOutDeck must have exactly as many slides as input Deck"
        );
    }

    /// EC-001 — `layout::run` on a zero-slide `Deck` returns `Err(LayoutError::EmptyDeck)`.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_001_layout_run_empty_deck_error() {
        let deck = make_deck(vec![]);
        let brand = make_brand();
        let result = run(&deck, &brand);
        match result {
            Err(LayoutError::EmptyDeck { .. }) => {
                // Correct: empty deck rejected
            },
            Ok(lod) => panic!(
                "layout::run must return Err(EmptyDeck) for zero-slide deck, got Ok with {} slides",
                lod.slides.len()
            ),
            Err(other) => {
                panic!("layout::run must return Err(EmptyDeck) for zero-slide deck, got {other:?}")
            },
        }
    }

    /// BC-3.06.002 — `layout::run` is deterministic: same deck + brand → identical output.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_002_layout_run_deterministic() {
        let slides = vec![
            make_slide_with_title("title", "Hello World"),
            make_slide("content"),
        ];
        let deck = make_deck(slides);
        let brand = make_brand();
        let result1 = run(&deck, &brand).expect("first run must succeed");
        let result2 = run(&deck, &brand).expect("second run must succeed");
        assert_eq!(
            result1, result2,
            "layout::run must be deterministic — identical inputs must produce identical output"
        );
    }

    /// AC-004 — Default brand produces `LaidOutDeck` with the 16:9 default page size.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_001_layout_run_default_page_size() {
        let deck = make_deck(vec![make_slide("title")]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        assert_eq!(
            result.page_size.width, DEFAULT_PAGE_WIDTH,
            "default page width must be DEFAULT_PAGE_WIDTH (9_144_000 EMU)"
        );
        assert_eq!(
            result.page_size.height, DEFAULT_PAGE_HEIGHT,
            "default page height must be DEFAULT_PAGE_HEIGHT (5_143_500 EMU)"
        );
    }

    /// AC-004 — Brand with custom canvas dimensions overrides the default page size.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_001_layout_run_page_size_from_brand() {
        use slideforge_types::{Emu, LayoutDefinition, SourceSpan};

        let custom_width = Emu(10_000_000);
        let custom_height = Emu(7_000_000);

        let mut brand = make_brand();
        // Add a layout definition with custom dimensions to indicate non-default size.
        // The layout engine reads brand.layouts[0].canvas_width/height if present.
        brand.layouts.push(LayoutDefinition {
            name: Arc::from("default"),
            canvas_width: custom_width,
            canvas_height: custom_height,
            span: SourceSpan::default(),
        });

        let deck = make_deck(vec![make_slide("title")]);
        let result = run(&deck, &brand).expect("layout::run must succeed with custom page size");
        assert_eq!(
            result.page_size.width, custom_width,
            "page width must match brand canvas_width when brand provides a layout override"
        );
        assert_eq!(
            result.page_size.height, custom_height,
            "page height must match brand canvas_height when brand provides a layout override"
        );
    }

    /// BC-3.06.003 — All frames in the produced `LaidOutDeck` have valid EMU coords.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_003_layout_run_all_frames_valid_bounding_boxes() {
        let slides = vec![
            make_slide("title"),
            make_slide("content"),
            make_slide("stat_callout"),
        ];
        let deck = make_deck(slides);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        let page_w = result.page_size.width;
        let page_h = result.page_size.height;
        for (si, slide) in result.slides.iter().enumerate() {
            for (fi, frame) in slide.frames.iter().enumerate() {
                assert!(
                    frame.bbox.is_valid(page_w, page_h),
                    "slide {si} frame {fi} has invalid bbox: {:?}",
                    frame.bbox
                );
            }
        }
    }

    /// AC-005 — `LaidOutSlide.source_index` matches the slide's position in input deck.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_001_laid_out_slide_source_index_correct() {
        let slides = vec![
            make_slide("title"),
            make_slide("content"),
            make_slide("blank"),
        ];
        let deck = make_deck(slides);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        for (expected_index, laid_out) in result.slides.iter().enumerate() {
            assert_eq!(
                laid_out.source_index, expected_index,
                "source_index must equal the slide's position in the input deck"
            );
        }
    }

    /// AC-005 — `LaidOutSlide.slide_type_keyword` matches the semantic `Slide.slide_type`.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_001_laid_out_slide_type_keyword_matches() {
        let slides = vec![
            make_slide("title"),
            make_slide("content"),
            make_slide("blank"),
        ];
        let expected_types = ["title", "content", "blank"];
        let deck = make_deck(slides);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        for (i, (laid_out, expected_kw)) in
            result.slides.iter().zip(expected_types.iter()).enumerate()
        {
            assert_eq!(
                laid_out.slide_type_keyword.as_ref(),
                *expected_kw,
                "slide {i} slide_type_keyword must match the semantic type"
            );
        }
    }

    /// AC-006 — title slide produces frames for title and subtitle regions.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_002_title_slide_has_frames() {
        let deck = make_deck(vec![make_slide("title")]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        assert!(!result.slides.is_empty());
        let title_slide = &result.slides[0];
        assert!(
            !title_slide.frames.is_empty(),
            "title slide must produce at least 1 frame"
        );
    }

    /// AC-006 — blank slide produces zero frames.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_002_blank_slide_has_zero_frames() {
        let deck = make_deck(vec![make_slide("blank")]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        assert_eq!(
            result.slides[0].frames.len(),
            0,
            "blank slide must have 0 frames"
        );
    }

    /// EC-002 — Unknown slide type keyword returns `Err(LayoutError::UnknownSlideType)`.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_002_unknown_slide_type_returns_error() {
        let deck = make_deck(vec![make_slide("not_a_real_slide_type_xyz")]);
        let brand = make_brand();
        let result = run(&deck, &brand);
        match result {
            Err(LayoutError::UnknownSlideType { .. }) => {
                // Correct
            },
            Ok(_) => panic!("layout::run must return Err(UnknownSlideType) for unknown keyword"),
            Err(other) => panic!(
                "layout::run must return Err(UnknownSlideType) for unknown keyword, got {other:?}"
            ),
        }
    }

    /// AC-010 — `LaidOutDeck` (and all IR types) implement `Hash + Eq + Clone`.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[test]
    fn test_bc_3_06_001_laid_out_deck_implements_hash_eq_clone() {
        use std::collections::HashSet;

        let deck = make_deck(vec![make_slide("blank")]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        let result2 = result.clone();
        assert_eq!(result, result2);

        let mut set = HashSet::new();
        set.insert(result);
        assert_eq!(set.len(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-001 — register_tags population
    // ─────────────────────────────────────────────────────────────────────────

    /// FINDING-001 — slide with `Register::Notes` produces `register_tags == [RegisterTag::Notes]`.
    #[test]
    fn test_bc_3_06_001_register_tags_notes() {
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: Some(Register::Notes),
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        assert_eq!(
            result.slides[0].register_tags,
            vec![RegisterTag::Notes],
            "slide with Register::Notes must produce register_tags == [RegisterTag::Notes]"
        );
    }

    /// FINDING-001 — slide with `Register::Detail` produces `register_tags == [RegisterTag::Detail]`.
    #[test]
    fn test_bc_3_06_001_register_tags_detail() {
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: Some(Register::Detail),
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        assert_eq!(
            result.slides[0].register_tags,
            vec![RegisterTag::Detail],
            "slide with Register::Detail must produce register_tags == [RegisterTag::Detail]"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-002 — speaker_notes extraction
    // ─────────────────────────────────────────────────────────────────────────

    /// FINDING-002 / F-004 (STORY-035) — `speaker_notes` is derived from `register_content`.
    ///
    /// After STORY-035 F-004: `speaker_notes` is a derived convenience field computed
    /// from the Notes entry in `LaidOutSlide.register_content`. Layout does NOT read
    /// `slide.fields.get("notes")` — it reads `slide.register_content` (canonical source).
    /// The test pre-populates `register_content` (simulating what `eval_deck` produces).
    #[test]
    fn test_bc_3_06_001_speaker_notes_extracted_from_notes_field() {
        use slideforge_types::{InlineNode, Register, RegisteredContent};

        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("notes"),
            FieldValue::Literal(Value::Str(Arc::from("My notes"))),
        );
        // Pre-populate register_content — simulates what eval_deck would produce
        // (eval_deck calls extract_register_content after field resolution).
        let register_content = vec![RegisteredContent {
            register: Register::Notes,
            content: vec![InlineNode::Plain(Arc::from("My notes"))],
        }];
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content,
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        assert_eq!(
            result.slides[0].speaker_notes,
            Some(Arc::from("My notes")),
            "slide with notes in register_content must have speaker_notes populated; \
             speaker_notes is derived from register_content (F-004 / STORY-035)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // STORY-027 / BC-3.02.001 — layout::run populates LaidOutDeck.sections
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-001 — `layout::run` on a deck with takeaway slides must produce a
    /// `LaidOutDeck` whose `sections` Vec contains an `ExecutiveSummary` section.
    ///
    /// This is the integration test that wires the section collection pass into
    /// the layout pipeline.  Currently `layout::run` seeds sections as
    /// `Vec::new()` with a comment citing STORY-027; the implementer must
    /// replace that placeholder with `collect_sections(&deck)`.
    ///
    /// FAILS at Red Gate: `layout::run` returns sections: `Vec::new()` (empty),
    /// so the assertion `!result.sections.is_empty()` fails.
    #[test]
    fn test_layout_run_populates_sections_from_takeaway_slides() {
        use slideforge_types::{Brand, BrandFonts, BrandPalette, SourceSpan};

        let mut fields = slideforge_types::OrderedMap::new();
        fields.insert(
            Arc::from("takeaway"),
            slideforge_types::FieldValue::Literal(slideforge_types::Value::Str(Arc::from(
                "Layout integration takeaway",
            ))),
        );
        let slide_with_takeaway = slideforge_types::Slide {
            slide_type: Arc::from("content"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = slideforge_types::Deck {
            slides: vec![slide_with_takeaway],
            vars: slideforge_types::OrderedMap::new(),
            metadata: slideforge_types::DeckMetadata {
                title: Some(Arc::from("Section Test Deck")),
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from("en-US")),
                author: None,
                section_order: None,
            },
            registers: slideforge_types::OrderedMap::new(),
            section_blocks: vec![],
        };
        let brand = Brand {
            name: Arc::from("test-brand"),
            palette: BrandPalette {
                primary: Arc::from("#003087"),
                secondary: Arc::from("#0066CC"),
                accent: Arc::from("#FF6B35"),
                neutral: Arc::from("#F5F5F5"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Calibri"),
                body: Arc::from("Calibri"),
                mono: Arc::from("Courier New"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        };
        let result = run(&deck, &brand).expect("layout::run must succeed for a content slide");
        assert!(
            !result.sections.is_empty(),
            "LaidOutDeck.sections must be non-empty when the deck has takeaway slides \
             (STORY-027: layout::run must call collect_sections)"
        );
        let has_exec = result
            .sections
            .iter()
            .any(|s| s.kind == sections::SectionKind::ExecutiveSummary);
        assert!(
            has_exec,
            "LaidOutDeck.sections must contain an ExecutiveSummary section when takeaway slides exist"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-006 — layout-level integration tests for severity_cards, manual
    // section blocks, and section_order
    // ─────────────────────────────────────────────────────────────────────────

    /// Helper: make a `severity_cards` slide with a `Value::List` of cards.
    fn make_severity_cards_slide(cards: Vec<(&str, &str, &str, &str)>) -> Slide {
        let card_values: Vec<Value> = cards
            .into_iter()
            .map(|(title, severity, description, owner)| {
                let mut m = OrderedMap::new();
                m.insert(Arc::from("title"), Value::Str(Arc::from(title)));
                m.insert(Arc::from("severity"), Value::Str(Arc::from(severity)));
                m.insert(Arc::from("description"), Value::Str(Arc::from(description)));
                m.insert(Arc::from("owner"), Value::Str(Arc::from(owner)));
                Value::Map(m)
            })
            .collect();
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("cards"),
            FieldValue::Literal(Value::List(card_values)),
        );
        Slide {
            slide_type: Arc::from("severity_cards"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    /// Helper: make a slide with a takeaway field.
    fn make_takeaway_slide(takeaway: &str) -> Slide {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("takeaway"),
            FieldValue::Literal(Value::Str(Arc::from(takeaway))),
        );
        Slide {
            slide_type: Arc::from("content"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    /// HIGH-006 / CRIT-001 — `layout::run` on a deck containing one `severity_cards`
    /// slide with 2 cards succeeds (no `UnknownSlideType`) and produces a
    /// `RiskRegister` section with 2 `RiskRow` items.
    #[test]
    fn test_layout_run_with_severity_cards_slide() {
        let deck = Deck {
            slides: vec![
                make_slide("title"),
                make_severity_cards_slide(vec![
                    ("Risk A", "High", "First risk", "Owner A"),
                    ("Risk B", "Medium", "Second risk", "Owner B"),
                ]),
            ],
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
            section_blocks: vec![],
        };
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed for severity_cards slide");

        // Slide count preserved.
        assert_eq!(result.slides.len(), 2);

        // RiskRegister section must be present with 2 rows.
        let risk = result
            .sections
            .iter()
            .find(|s| s.kind == sections::SectionKind::RiskRegister)
            .expect("LaidOutDeck.sections must contain a RiskRegister section");

        assert_eq!(
            risk.items.len(),
            2,
            "RiskRegister must contain exactly 2 RiskRow items"
        );
    }

    /// HIGH-006 — `layout::run` on a deck with manual `section_blocks` produces a
    /// `ManualSection` in `LaidOutDeck.sections`. When a manual `executive_summary`
    /// is also present, it supersedes the auto-generated one.
    #[test]
    fn test_layout_run_with_manual_section_blocks() {
        // A manual executive_summary block + a takeaway slide.
        // The manual block must supersede the auto-generated executive_summary.
        let exec_block = SectionBlock {
            name: Arc::from("executive_summary"),
            body: {
                let mut m = OrderedMap::new();
                m.insert(
                    Arc::from("heading"),
                    Value::Str(Arc::from("Custom Executive Summary")),
                );
                m
            },
            span: SourceSpan::default(),
        };
        let methodology_block = SectionBlock {
            name: Arc::from("methodology"),
            body: OrderedMap::new(),
            span: SourceSpan::default(),
        };
        let deck = Deck {
            slides: vec![make_slide("title"), make_takeaway_slide("Key finding 1")],
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
            section_blocks: vec![exec_block, methodology_block],
        };
        let brand = make_brand();
        let result =
            run(&deck, &brand).expect("layout::run must succeed with manual section blocks");

        // Manual executive_summary must be present.
        let has_manual_exec = result.sections.iter().any(|s| {
            s.kind == sections::SectionKind::ManualSection(Arc::from("executive_summary"))
        });
        assert!(
            has_manual_exec,
            "manual executive_summary section must appear in LaidOutDeck.sections"
        );

        // Auto-generated executive_summary must be suppressed by supersession rule.
        let has_auto_exec = result
            .sections
            .iter()
            .any(|s| s.kind == sections::SectionKind::ExecutiveSummary);
        assert!(
            !has_auto_exec,
            "auto-generated executive_summary must be suppressed when a manual one is present"
        );

        // Manual methodology must also be present.
        let has_methodology = result
            .sections
            .iter()
            .any(|s| s.kind == sections::SectionKind::ManualSection(Arc::from("methodology")));
        assert!(
            has_methodology,
            "manual methodology section must appear in LaidOutDeck.sections"
        );
    }

    /// HIGH-006 — `layout::run` on a deck with `section_order` metadata applies the
    /// declared ordering to `LaidOutDeck.sections`.
    #[test]
    fn test_layout_run_with_section_order() {
        // Without section_order: executive_summary first (default), risk_register second.
        // With section_order = ["risk_register", "executive_summary"]: reversed.
        let deck = Deck {
            slides: vec![
                make_takeaway_slide("Key finding"),
                make_severity_cards_slide(vec![("SQL Injection", "High", "DB risk", "DBA")]),
            ],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: Some(Arc::from("Test Deck")),
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from("en-US")),
                author: None,
                section_order: Some(vec![
                    Arc::from("risk_register"),
                    Arc::from("executive_summary"),
                ]),
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
        };
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed with section_order");

        assert_eq!(
            result.sections.len(),
            2,
            "must have exactly 2 sections (risk_register + executive_summary)"
        );
        assert_eq!(
            result.sections[0].kind,
            sections::SectionKind::RiskRegister,
            "risk_register must come first per section_order"
        );
        assert_eq!(
            result.sections[1].kind,
            sections::SectionKind::ExecutiveSummary,
            "executive_summary must come second per section_order"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-CRIT-001 / AC-INT-1 — layout::run wires shape layout + inline validation
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-INT-1 / F-CRIT-001 — `layout::run` integrates the shape layout pass:
    /// a slide with a `ContentBlock::Shape` block must produce a `FrameContent::Shape`
    /// frame appended after the region-map frames in the output `LaidOutSlide`.
    ///
    /// This is the end-to-end integration gate confirming that `layout_shapes` is
    /// wired into `layout::run` (BC-3.04.001 postcondition 4 / AC-INT-1).
    #[test]
    fn test_ac_int_1_layout_run_wires_shape_block_to_frame() {
        use slideforge_types::{
            AltText, Block, ContentBlock, FillSpec, ShapePosition, ShapeSpec, ShapeType, ShapeUnit,
        };

        let shape_spec = ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),
                y: ShapeUnit::Inches(500),
                width: ShapeUnit::Inches(1000),
                height: ShapeUnit::Inches(500),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("a test rectangle"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let block = Block {
            content: ContentBlock::Shape(shape_spec),
            label: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed with a shape block");

        // The title slide has 2 region-map frames (title + subtitle).
        // The shape block adds a third frame.
        let slide_out = &result.slides[0];
        assert!(
            slide_out.frames.len() >= 3,
            "slide with a shape block must produce at least 3 frames (2 region + 1 shape), got {}",
            slide_out.frames.len()
        );

        // The last frame must be FrameContent::Shape.
        let last_frame = slide_out.frames.last().expect("frames must be non-empty");
        assert!(
            matches!(last_frame.content, FrameContent::Shape(_)),
            "last frame must be FrameContent::Shape, got {:?}",
            last_frame.content
        );
    }

    /// AC-INT-1 / F-CRIT-001 step 4 — `run_inline_validation` (called by `layout::run`)
    /// detects an unknown xref target in a `TextRun` frame and produces
    /// `LayoutWarning::XrefTargetNotFound`.
    ///
    /// This test calls `run_inline_validation` directly (the same function wired into
    /// `layout::run`) with a manually-constructed laid-out slide containing a `TextRun`
    /// frame with an unknown `Xref` target. This proves the inline validation wire is
    /// live (BC-3.05.001 EC-002 / AC-007 / F-CRIT-001).
    #[test]
    fn test_ac_int_1_inline_validation_unknown_xref_produces_warning() {
        use crate::inline::run_inline_validation;
        use crate::types::{BoundingBox, Frame, FrameContent, LaidOutSlide};
        use slideforge_types::{Emu, InlineNode};

        let xref_target = Arc::from("__unknown_slide_target__");
        let text_run_frame = Frame {
            bbox: BoundingBox {
                x: Emu(0),
                y: Emu(0),
                width: Emu(1_000_000),
                height: Emu(500_000),
            },
            content: FrameContent::TextRun(vec![InlineNode::Xref(Arc::clone(&xref_target))]),
            text_flow: None,
        };
        let laid_out_slide = LaidOutSlide {
            source_index: 0,
            slide_type_keyword: Arc::from("title"),
            frames: vec![text_run_frame],
            speaker_notes: None,
            register_tags: vec![],
            register_content: vec![],
        };
        let deck = make_deck(vec![make_slide("title")]);

        let warnings = run_inline_validation(&deck, &[laid_out_slide])
            .expect("run_inline_validation must not error for unknown xref (only a warning)");

        assert_eq!(
            warnings.len(),
            1,
            "must produce exactly one XrefTargetNotFound warning, got: {warnings:?}"
        );
        assert!(
            matches!(
                &warnings[0],
                LayoutWarning::XrefTargetNotFound { target, .. }
                if target.as_ref() == "__unknown_slide_target__"
            ),
            "warning must be XrefTargetNotFound for the unknown xref target, got: {:?}",
            warnings[0]
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-HIGH-002 — AC-INT-1 canonical EMU vector (load-bearing bbox assertion)
    // ─────────────────────────────────────────────────────────────────────────

    /// F-HIGH-002 — `layout::run` produces exact EMU values for the canonical
    /// position vector x=0.5in, y=1.0in, width=2.0in, height=1.0in.
    ///
    /// Previously, `test_ac_int_1_layout_run_wires_shape_block_to_frame` only
    /// checked `frames.len() >= 3` and `FrameContent::Shape`. This test adds the
    /// canonical-vector assertion for the bbox values (load-bearing per F-HIGH-002).
    ///
    /// Load-bearing: if `unit_to_emu` is stubbed to return Emu(0) for all inputs,
    /// the bbox assertions below fail.
    #[test]
    fn test_f_high_002_layout_run_canonical_emu_vector() {
        use slideforge_types::{
            AltText, Block, ContentBlock, FillSpec, ShapePosition, ShapeSpec, ShapeType, ShapeUnit,
        };

        let shape_spec = ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),       // 0.5in → Emu(457_200)
                y: ShapeUnit::Inches(1000),      // 1.0in → Emu(914_400)
                width: ShapeUnit::Inches(2000),  // 2.0in → Emu(1_828_800)
                height: ShapeUnit::Inches(1000), // 1.0in → Emu(914_400)
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("canonical test rectangle"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let block = Block {
            content: ContentBlock::Shape(shape_spec),
            label: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");

        let slide_out = &result.slides[0];
        // Shape frame is the last frame (appended after region-map frames).
        let last_frame = slide_out.frames.last().expect("frames must be non-empty");
        assert!(
            matches!(last_frame.content, FrameContent::Shape(_)),
            "last frame must be FrameContent::Shape"
        );

        // Canonical EMU vector assertion (F-HIGH-002 load-bearing).
        let bbox = last_frame.bbox;
        assert_eq!(
            bbox.x,
            slideforge_types::Emu(457_200),
            "x: 0.5in → Emu(457_200)"
        );
        assert_eq!(
            bbox.y,
            slideforge_types::Emu(914_400),
            "y: 1.0in → Emu(914_400)"
        );
        assert_eq!(
            bbox.width,
            slideforge_types::Emu(1_828_800),
            "width: 2.0in → Emu(1_828_800)"
        );
        assert_eq!(
            bbox.height,
            slideforge_types::Emu(914_400),
            "height: 1.0in → Emu(914_400)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-HIGH-004 — End-to-end fill+text propagation through layout::run
    // ─────────────────────────────────────────────────────────────────────────

    /// F-HIGH-004 — `layout::run` preserves shape fill and text through the full pipeline.
    ///
    /// A `ShapeSpec` with `fill: FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 })`
    /// AND `text: Some(vec![InlineNode::Plain(Arc::from("Hello"))])` must produce
    /// a `FrameContent::Shape` frame where both fields are preserved verbatim.
    ///
    /// Load-bearing: if the layout code drops fill or text during shape→frame conversion
    /// (e.g., in `build_shape_frame`), the equality assertions below fail.
    #[test]
    fn test_h_high_004_shape_fill_and_text_propagate_through_layout_run() {
        use crate::types::FrameContent;
        use slideforge_types::{
            AltText, Block, ContentBlock, FillSpec, InlineNode, Rgb, ShapePosition, ShapeSpec,
            ShapeType, ShapeUnit,
        };

        let expected_fill = FillSpec::SolidColor(Rgb {
            r: 0,
            g: 55,
            b: 102,
        });
        let expected_text = vec![InlineNode::Plain(Arc::from("Hello"))];

        let shape_spec = ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),       // 0.5in
                y: ShapeUnit::Inches(1000),      // 1.0in
                width: ShapeUnit::Inches(2000),  // 2.0in
                height: ShapeUnit::Inches(1000), // 1.0in
            },
            fill: expected_fill.clone(),
            text: Some(expected_text.clone()),
            alt: Some(AltText::Provided(Arc::from("fill-text test rect"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let block = Block {
            content: ContentBlock::Shape(shape_spec),
            label: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed for F-HIGH-004 test");

        // Find the shape frame in the output.
        let slide_out = &result.slides[0];
        let shape_frame = slide_out
            .frames
            .iter()
            .find_map(|f| match &f.content {
                FrameContent::Shape(sf) => Some(sf),
                _ => None,
            })
            .expect("must have a FrameContent::Shape frame");

        // F-HIGH-004 load-bearing: fill must be preserved verbatim.
        assert_eq!(
            shape_frame.fill, expected_fill,
            "fill must propagate through layout::run unchanged; \
             expected SolidColor(Rgb(0,55,102)), got: {:?}",
            shape_frame.fill
        );

        // F-HIGH-004 load-bearing: text must be preserved verbatim.
        assert_eq!(
            shape_frame.text.as_deref(),
            Some(expected_text.as_slice()),
            "text must propagate through layout::run unchanged; \
             expected Some([Plain(\"Hello\")]), got: {:?}",
            shape_frame.text
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-049 — LaidOutDeck.warnings wired: off-canvas + xref warnings flow through
    // VP-050 — Placeholder-before-shape frame ordering (BC-3.04.001 v1.5.2 PC-3)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-049 — `layout::run` propagates off-canvas shape warnings to
    /// `LaidOutDeck.warnings` (BC-3.04.001 EC-002 / AC-003).
    ///
    /// A shape at x=-0.5in (negative x) is off-canvas. The warning must appear
    /// in `LaidOutDeck.warnings`, not silently dropped.
    ///
    /// Load-bearing: if `deck_warnings.extend(shape_output.warnings)` is removed
    /// from `layout::run`, this assertion fails (result.warnings would be empty).
    #[test]
    fn test_vp_049_layout_run_off_canvas_warning_in_laid_out_deck_warnings() {
        use crate::types::LayoutWarning;
        use slideforge_types::{
            AltText, Block, ContentBlock, FillSpec, ShapePosition, ShapeSpec, ShapeType, ShapeUnit,
        };

        let shape_spec = ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(-500), // -0.5in → Emu(-457_200) → off-canvas
                y: ShapeUnit::Inches(500),
                width: ShapeUnit::Inches(1000),
                height: ShapeUnit::Inches(500),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("off-canvas rectangle"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let block = Block {
            content: ContentBlock::Shape(shape_spec),
            label: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed for off-canvas shape");

        // VP-049: the off-canvas warning must appear in LaidOutDeck.warnings.
        assert!(
            result
                .warnings
                .iter()
                .any(|w| matches!(w, LayoutWarning::OffCanvas { .. })),
            "LaidOutDeck.warnings must contain OffCanvas warning for off-canvas shape; \
             got: {:?}",
            result.warnings
        );
    }

    /// VP-049 — `layout::run` propagates xref-not-found warnings to
    /// `LaidOutDeck.warnings` (BC-3.05.001 EC-002 / AC-007).
    ///
    /// A slide with a `ContentBlock::Text` block that contains an `Xref` to an
    /// unknown target must produce a `XrefTargetNotFound` warning in
    /// `LaidOutDeck.warnings` returned by `layout::run`.
    ///
    /// Load-bearing: if `deck_warnings.extend(inline_warnings)` is removed from
    /// `layout::run`, this assertion fails — the warning is produced by
    /// `run_inline_validation` but never surfaced on the returned `LaidOutDeck`.
    ///
    /// F-CRIT-002: this test calls `layout::run` end-to-end (not `run_inline_validation`
    /// directly), making it load-bearing per the VP-049 contract.
    #[test]
    fn test_vp_049_layout_run_xref_warning_in_laid_out_deck_warnings() {
        use crate::types::LayoutWarning;
        use slideforge_types::{Block, ContentBlock, InlineNode, TextBlock};

        let xref_target = Arc::from("__nonexistent_slide__");
        let text_block = TextBlock {
            inlines: vec![InlineNode::Xref(Arc::clone(&xref_target))],
            span: SourceSpan::default(),
        };
        let block = Block {
            content: ContentBlock::Text(text_block),
            label: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();

        // VP-049: call layout::run end-to-end. The TextRun frame produced from the
        // ContentBlock::Text block must trigger run_inline_validation, and the
        // XrefTargetNotFound warning must appear on LaidOutDeck.warnings.
        let result = run(&deck, &brand)
            .expect("layout::run must succeed for unknown xref (warning, not error)");

        // VP-049 load-bearing assertion: warnings must contain XrefTargetNotFound.
        // If deck_warnings.extend(inline_warnings) is removed from layout::run,
        // this fails (result.warnings would be empty).
        assert!(
            result.warnings.iter().any(|w| matches!(
                w,
                LayoutWarning::XrefTargetNotFound { target, .. }
                if target.as_ref() == "__nonexistent_slide__"
            )),
            "LaidOutDeck.warnings must contain XrefTargetNotFound for '__nonexistent_slide__'; \
             got: {:?}",
            result.warnings
        );
    }

    /// VP-050 — `layout::run` places shape frames AFTER all placeholder/region frames
    /// in `LaidOutSlide.frames` (BC-3.04.001 v1.5.2 PC-3 / interface-definitions §9.3).
    ///
    /// A slide with region-map placeholders AND a `ContentBlock::Shape` block must
    /// produce a `LaidOutSlide` where:
    /// - The first N frames correspond to the region-map placeholders (not Shape)
    /// - The shape frames appear after all placeholder frames
    ///
    /// Load-bearing:
    /// - If shape frames are inserted BEFORE region frames, the index assertion fails.
    /// - If placeholder frames are produced as Shape variants, the `!matches!` assertion fails.
    ///
    /// F-CRIT-001: this test uses `layout::run` end-to-end (not `layout_shapes` directly).
    #[test]
    fn test_vp_050_layout_run_shape_frame_after_regions() {
        use crate::types::FrameContent;
        use slideforge_types::{
            AltText, Block, ContentBlock, FillSpec, ShapePosition, ShapeSpec, ShapeType, ShapeUnit,
        };

        // Build a slide with a shape block — layout::run produces region frames first,
        // then appends shape frames (BC-3.04.001 postcondition 4 / PC-3).
        let shape_spec = ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),       // 0.5in
                y: ShapeUnit::Inches(1000),      // 1.0in
                width: ShapeUnit::Inches(2000),  // 2.0in
                height: ShapeUnit::Inches(1000), // 1.0in
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("vp050 rect"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let block = Block {
            content: ContentBlock::Shape(shape_spec),
            label: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();

        let result = run(&deck, &brand).expect("layout::run must succeed for VP-050 test");
        let slide_out = &result.slides[0];

        // Find the index of the first Shape frame.
        let first_shape_index = slide_out
            .frames
            .iter()
            .position(|f| matches!(f.content, FrameContent::Shape(_)))
            .expect("must have at least one shape frame");

        // VP-050 PC-3: shape frames appear AFTER all placeholder/region frames.
        // All frames before the first shape must NOT be Shape variants.
        let region_frame_count = first_shape_index;
        assert!(
            region_frame_count > 0,
            "a 'title' slide must have at least one region-map frame before the shape frame; \
             first_shape_index was 0, meaning no region frames precede it"
        );
        for (idx, frame) in slide_out.frames[..region_frame_count].iter().enumerate() {
            assert!(
                !matches!(frame.content, FrameContent::Shape(_)),
                "VP-050: placeholder frame at index {idx} must not be a Shape variant; \
                 got: {:?}",
                frame.content
            );
        }

        // Load-bearing: shape frame IS present.
        assert!(
            matches!(
                slide_out.frames[first_shape_index].content,
                FrameContent::Shape(_)
            ),
            "frame at first_shape_index ({first_shape_index}) must be FrameContent::Shape"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-P4-MED-002 — shape text inline nodes scanned by run_inline_validation
    // F-P4-LOW-001 — TextRun bbox validated against page size
    // ─────────────────────────────────────────────────────────────────────────

    /// F-P4-MED-002 — `run_inline_validation` scans xref nodes inside
    /// `FrameContent::Shape` text, not only `FrameContent::TextRun` frames.
    ///
    /// A `shape:` block with `text: Some([Xref("nonexistent")])` must produce
    /// `LaidOutDeck.warnings` containing `XrefTargetNotFound` for the unknown
    /// target. Without the fix (only scanning `TextRun`), no warning is emitted
    /// and this test fails.
    ///
    /// Load-bearing (TD-VSDD-059): removing the `FrameContent::Shape` arm from
    /// `run_inline_validation` causes the assertion to fail.
    #[test]
    fn test_p4_med_002_shape_text_xref_validated() {
        use crate::types::LayoutWarning;
        use slideforge_types::{
            AltText, Block, ContentBlock, FillSpec, InlineNode, ShapePosition, ShapeSpec,
            ShapeType, ShapeUnit,
        };

        let unknown_target = Arc::from("__nonexistent_shape_xref__");
        let shape_spec = ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),      // 0.5in
                y: ShapeUnit::Inches(500),      // 0.5in
                width: ShapeUnit::Inches(2000), // 2.0in
                height: ShapeUnit::Inches(500), // 0.5in
            },
            fill: FillSpec::None,
            // The text field carries an Xref to a slide that does not exist.
            text: Some(vec![InlineNode::Xref(Arc::clone(&unknown_target))]),
            alt: Some(AltText::Provided(Arc::from("shape with xref text"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let block = Block {
            content: ContentBlock::Shape(shape_spec),
            label: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();

        let result = run(&deck, &brand)
            .expect("layout::run must succeed — unknown xref is a warning, not an error");

        // F-P4-MED-002 load-bearing assertion: shape text xref must be validated.
        // Without the Shape arm in run_inline_validation, this fails (empty warnings).
        assert!(
            result.warnings.iter().any(|w| matches!(
                w,
                LayoutWarning::XrefTargetNotFound { target, .. }
                if target.as_ref() == "__nonexistent_shape_xref__"
            )),
            "LaidOutDeck.warnings must contain XrefTargetNotFound for shape text xref \
             '__nonexistent_shape_xref__'; got: {:?}",
            result.warnings
        );
    }

    /// F-P4-MED-002 — depth-bound check applies to shape text inline trees.
    ///
    /// A `shape:` block with text containing a 65-level-deep nested
    /// `InlineNode::Bold` must return `Err(LayoutError::InlineDepthExceeded)`
    /// from `layout::run` (BC-3.05.001 E-LAY-005 / F-MED-006).
    ///
    /// Load-bearing (TD-VSDD-059): without the Shape arm in
    /// `run_inline_validation`, depth-exceeded errors from shape text are
    /// silently swallowed and `layout::run` returns `Ok`, causing this test
    /// to fail.
    #[test]
    fn test_p4_med_002_shape_text_depth_bound() {
        use crate::inline::MAX_INLINE_DEPTH;
        use slideforge_types::{
            AltText, Block, ContentBlock, FillSpec, InlineNode, ShapePosition, ShapeSpec,
            ShapeType, ShapeUnit,
        };

        // Build a 65-level-deep Bold tree (exceeds MAX_INLINE_DEPTH = 64).
        // InlineNode::Bold(children: Vec<InlineNode>)
        let leaf = InlineNode::Plain(Arc::from("deep text"));
        let deeply_nested =
            (0..=MAX_INLINE_DEPTH).fold(leaf, |inner, _| InlineNode::Bold(vec![inner]));

        let shape_spec = ShapeSpec {
            shape_type: ShapeType::Rect,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),
                y: ShapeUnit::Inches(500),
                width: ShapeUnit::Inches(2000),
                height: ShapeUnit::Inches(500),
            },
            fill: FillSpec::None,
            text: Some(vec![deeply_nested]),
            alt: Some(AltText::Provided(Arc::from("deep shape"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let block = Block {
            content: ContentBlock::Shape(shape_spec),
            label: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();

        let result = run(&deck, &brand);

        assert!(
            matches!(result, Err(LayoutError::InlineDepthExceeded { .. })),
            "layout::run must return Err(InlineDepthExceeded) for 65-deep shape text; \
             got: {result:?}"
        );
    }

    /// F-P4-LOW-001 — `TextRun` bbox is validated against page dimensions.
    ///
    /// A brand with a tiny canvas (height = `100_000` EMU, smaller than the
    /// `914_400` EMU placeholder height) must NOT escape the `is_valid` check.
    /// The layout engine clamps the placeholder height to `min(914_400, page_h)`
    /// so the bbox always satisfies BC-3.06.003 invariants.
    ///
    /// This test verifies that a Text block on a tiny-canvas brand produces a
    /// valid laid-out deck (no `InvalidBoundingBox` error) because the height is
    /// clamped, AND that the `TextRun` frame bbox satisfies `is_valid`.
    ///
    /// Load-bearing (TD-VSDD-059): if the clamp is removed and the raw
    /// `914_400` height is used, `is_valid` would return false for `height=100_000`
    /// and `layout::run` would return `Err(InvalidBoundingBox)`, making this
    /// assertion fail.
    #[test]
    fn test_p4_low_001_text_run_bbox_validated_against_page_size() {
        use slideforge_types::{Block, ContentBlock, InlineNode, LayoutDefinition, TextBlock};

        // Brand with tiny canvas: height = 100_000 EMU (< 914_400 placeholder).
        let tiny_height = slideforge_types::Emu(100_000);
        let page_width = slideforge_types::Emu(9_144_000); // 10 inches
        let mut brand = make_brand();
        brand.layouts.push(LayoutDefinition {
            name: Arc::from("tiny"),
            canvas_width: page_width,
            canvas_height: tiny_height,
            span: SourceSpan::default(),
        });

        let text_block = TextBlock {
            inlines: vec![InlineNode::Plain(Arc::from("hello"))],
            span: SourceSpan::default(),
        };
        let block = Block {
            content: ContentBlock::Text(text_block),
            label: None,
            span: SourceSpan::default(),
        };
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields: OrderedMap::new(),
            blocks: vec![block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);

        // Must succeed — the placeholder height is clamped to tiny_height.
        let result = run(&deck, &brand).expect(
            "layout::run must succeed for tiny-canvas brand with Text block; \
             placeholder height must be clamped to page_height",
        );

        // Verify every frame in the result passes is_valid.
        let page_w = result.page_size.width;
        let page_h = result.page_size.height;
        for (si, slide_out) in result.slides.iter().enumerate() {
            for (fi, frame) in slide_out.frames.iter().enumerate() {
                assert!(
                    frame.bbox.is_valid(page_w, page_h),
                    "F-P4-LOW-001: slide {si} frame {fi} has invalid bbox on tiny canvas: {:?}",
                    frame.bbox
                );
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-003/F-004 (STORY-035): No-bleed invariant — register text must not appear
    // in frames; register_content must be populated (BC-1.14.004 invariant 3)
    // ─────────────────────────────────────────────────────────────────────────

    /// F-004 / BC-1.14.004 invariant 3: After layout, `LaidOutSlide.frames` must
    /// contain NONE of the register text, while `register_content` must contain all
    /// three register entries.
    ///
    /// This test builds a `Slide` with all three register fields pre-populated
    /// (simulating the post-eval state), runs `layout::run`, and asserts:
    /// 1. `LaidOutSlide.register_content` has 3 entries (Notes, Report, Detail).
    /// 2. No frame in `LaidOutSlide.frames` contains register text.
    ///
    /// This is the executable no-bleed assertion for BC-1.14.004 invariant 3.
    #[test]
    fn test_f004_no_bleed_register_text_not_in_frames() {
        use slideforge_types::{FieldValue, InlineNode, Register, RegisteredContent, Value};

        // Build a slide with register fields AND a visual title.
        // The slide's register_content is pre-populated (simulating post-eval state)
        // since the layout stage does NOT call extract_register_content — it reads
        // slide.register_content verbatim (Option D, architecture-directive STORY-035).
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Q1 Results"))),
        );
        fields.insert(
            Arc::from("notes"),
            FieldValue::Literal(Value::Str(Arc::from("Presenter: emphasise growth"))),
        );
        fields.insert(
            Arc::from("report"),
            FieldValue::Literal(Value::Str(Arc::from("Detailed narrative for readers"))),
        );
        fields.insert(
            Arc::from("detail"),
            FieldValue::Literal(Value::Str(Arc::from("Technical appendix text"))),
        );

        // Pre-populate register_content (as eval_deck would).
        let register_content = vec![
            RegisteredContent {
                register: Register::Notes,
                content: vec![InlineNode::Plain(Arc::from("Presenter: emphasise growth"))],
            },
            RegisteredContent {
                register: Register::Report,
                content: vec![InlineNode::Plain(Arc::from(
                    "Detailed narrative for readers",
                ))],
            },
            RegisteredContent {
                register: Register::Detail,
                content: vec![InlineNode::Plain(Arc::from("Technical appendix text"))],
            },
        ];

        let slide = Slide {
            slide_type: Arc::from("content"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content,
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();

        let result = run(&deck, &brand).expect("layout::run must succeed for content slide");
        assert_eq!(result.slides.len(), 1, "must produce 1 laid-out slide");

        let laid_out = &result.slides[0];

        // Assertion 1: register_content is populated with 3 entries (copied verbatim).
        assert_eq!(
            laid_out.register_content.len(),
            3,
            "LaidOutSlide.register_content must have 3 entries; \
             got: {:?}",
            laid_out.register_content
        );
        assert_eq!(laid_out.register_content[0].register, Register::Notes);
        assert_eq!(laid_out.register_content[1].register, Register::Report);
        assert_eq!(laid_out.register_content[2].register, Register::Detail);

        // Assertion 2 (no-bleed): none of the register texts must appear in any frame.
        // Collect all plain text from all frames.
        let register_texts = [
            "Presenter: emphasise growth",
            "Detailed narrative for readers",
            "Technical appendix text",
        ];
        for frame in &laid_out.frames {
            let frame_text = collect_frame_text(frame);
            for register_text in register_texts {
                assert!(
                    !frame_text.contains(register_text),
                    "BC-1.14.004 NO-BLEED violation: register text '{register_text}' found \
                     in frame (frame content: '{frame_text}'). Register text must NOT \
                     appear in frames — it belongs only in register_content."
                );
            }
        }
    }

    /// Collect all plain text from a frame (for no-bleed assertions).
    fn collect_frame_text(frame: &crate::types::Frame) -> String {
        use crate::types::FrameContent;
        use slideforge_types::InlineNode;

        fn inline_text(nodes: &[InlineNode]) -> String {
            nodes
                .iter()
                .map(|node| match node {
                    InlineNode::Plain(s) | InlineNode::Code(s) | InlineNode::Xref(s) => {
                        s.as_ref().to_owned()
                    },
                    InlineNode::Bold(c)
                    | InlineNode::Italic(c)
                    | InlineNode::Footnote(c)
                    | InlineNode::Superscript(c)
                    | InlineNode::Subscript(c)
                    | InlineNode::Strikethrough(c)
                    | InlineNode::Highlight(c) => inline_text(c),
                    InlineNode::Link { text, .. } => inline_text(text),
                    InlineNode::Math(m) => m.latex.as_ref().to_owned(),
                })
                .collect()
        }

        match &frame.content {
            FrameContent::TextRun(nodes) => inline_text(nodes),
            _ => String::new(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-011 skeleton: proptest for slide count preservation
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-011 — proptest: `layout::run` preserves slide count for any valid deck.
    ///
    /// This is the VP-011 proptest skeleton (to be expanded in STORY-069 with
    /// full Arbitrary impl for Deck). Uses bounded random generation.
    ///
    /// FAILS at Red Gate: `layout::run` returns `todo!()`.
    #[cfg(test)]
    mod proptest_vp011 {
        use super::*;
        use proptest::prelude::*;

        /// Known-valid slide type keywords for random deck generation.
        const VALID_TYPES: &[&str] = &["title", "content", "blank", "section_break", "quote"];

        proptest! {
            /// VP-011 skeleton: for any deck of 1–20 slides (from known types),
            /// layout::run preserves slide count.
            #[test]
            fn test_vp_011_layout_run_preserves_slide_count_proptest(
                slide_types in proptest::collection::vec(
                    proptest::sample::select(VALID_TYPES),
                    1..=20_usize,
                ),
            ) {
                let slides: Vec<Slide> = slide_types.iter().map(|kw| make_slide(kw)).collect();
                let expected_count = slides.len();
                let deck = make_deck(slides);
                let brand = make_brand();
                let result = run(&deck, &brand).expect("layout::run must succeed for valid slide types");
                prop_assert_eq!(
                    result.slides.len(),
                    expected_count,
                    "slide count must be preserved"
                );
            }

            /// VP-011 skeleton: layout::run is deterministic (identical inputs → equal output).
            #[test]
            fn test_vp_011_layout_run_is_deterministic_proptest(
                slide_types in proptest::collection::vec(
                    proptest::sample::select(VALID_TYPES),
                    1..=10_usize,
                ),
            ) {
                let slides: Vec<Slide> = slide_types.iter().map(|kw| make_slide(kw)).collect();
                let deck = make_deck(slides);
                let brand = make_brand();
                let r1 = run(&deck, &brand).expect("first run must succeed");
                let r2 = run(&deck, &brand).expect("second run must succeed");
                prop_assert_eq!(r1, r2, "layout::run must be deterministic");
            }
        }
    }
}
