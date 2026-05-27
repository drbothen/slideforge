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
//! - **Valid EMU coordinates (BC-3.06.003):** Every `BoundingBox` satisfies
//!   `x >= 0`, `y >= 0`, `width > 0`, `height > 0`, `x + width <= page_width`,
//!   `y + height <= page_height`.
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
pub mod layout;
pub mod regions;
pub mod text_flow;
pub mod types;

// Re-export the primary entry point and most-used types at crate root.
pub use error::LayoutError;
pub use layout::run;
pub use types::{
    BoundingBox, Frame, FrameContent, LaidOutDeck, LaidOutSlide, PageSize, RegisterTag, TextFlow,
    TextOverflow,
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
        Slide, SourceSpan, Value,
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
        }
    }

    fn make_deck(slides: Vec<Slide>) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
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

    /// FINDING-002 — slide with `notes` field set to a plain string populates `speaker_notes`.
    #[test]
    fn test_bc_3_06_001_speaker_notes_extracted_from_notes_field() {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("notes"),
            FieldValue::Literal(Value::Str(Arc::from("My notes"))),
        );
        let slide = Slide {
            slide_type: Arc::from("title"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
        };
        let deck = make_deck(vec![slide]);
        let brand = make_brand();
        let result = run(&deck, &brand).expect("layout::run must succeed");
        assert_eq!(
            result.slides[0].speaker_notes,
            Some(Arc::from("My notes")),
            "slide with notes field must have speaker_notes populated"
        );
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
