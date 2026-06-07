//! `slideforge-validate` — compile-time validation for the slideforge DSL.
//!
//! Validators check the semantic [`Deck`](slideforge_types::Deck) IR for
//! content and accessibility issues before export. All validators implement
//! the [`Validator`](slideforge_plugin_api::Validator) trait from
//! `slideforge-plugin-api`.
//!
//! ## Built-in validators
//!
//! | Validator | ID | Description |
//! |-----------|-----|-------------|
//! | [`AltTextValidator`] | `"alt-text"` | Checks all visual elements for required alt text (WCAG 1.1.1) |
//! | [`ZeroSlideValidator`] | `"zero-slide"` | Rejects decks that contain zero slides (E-LAY-002) |
//! | [`CanvasOverflowValidator`] | `"canvas-overflow"` | Heuristic bullet-overflow detection (E-LAY-001) |
//! | [`LabelCheckValidator`] | `"label-check"` | Enforces `label "..."` on color-coded slide types (WCAG 1.4.1) |
//! | [`LangValidator`] | `"lang"` | Warns on missing deck `lang` declaration; injects default "en" |
//! | [`ValueRangeValidator`] | `"value-range"` | Enforces numeric field ranges for color-coded slide types (E-VAL-011) |
//!
//! ## Validation pipeline configuration
//!
//! [`ValidationConfig`] controls the overall validation mode (strict vs.
//! warn-only) and per-validator knobs such as `strict_overflow`.
//!
//! ## Error-slide placeholders
//!
//! [`error_slide_placeholder`] constructs an internal placeholder slide
//! that the layout engine renders as an error card in watch-mode previews.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod alt_text;
mod canvas_overflow;
mod error_slide;
mod image_path;
mod label_check;
mod lang_validator;
mod mode;
mod utils;
pub mod value_range;
mod wcag;
mod zero_slide;

pub use alt_text::AltTextValidator;
pub use canvas_overflow::CanvasOverflowValidator;
pub use error_slide::{ERROR_PLACEHOLDER_SLIDE_TYPE, error_slide_placeholder};
pub use image_path::{E_VAL_012, ImagePathValidator};
pub use label_check::{COLOR_CODED_TYPES, LabelCheckValidator};
pub use lang_validator::{LangValidator, inject_lang_default};
pub use mode::{ValidationConfig, ValidationMode};
pub use utils::is_blank;
pub use value_range::ValueRangeValidator;
pub use wcag::{
    contrast_ratio, parse_hex_color, relative_luminance, srgb_component_to_linear, wcag_aa_passes,
};
pub use zero_slide::ZeroSlideValidator;

// ─── Full validation pipeline integration test ────────────────────────────────

#[cfg(test)]
mod integration_tests {
    //! Integration tests that exercise multiple validators together on a single
    //! deck, verifying combined diagnostic output and error code coverage.

    use std::sync::Arc;

    use slideforge_plugin_api::{Validator, ValidatorOptions};
    use slideforge_types::{
        Block, ContentBlock, Deck, DeckMetadata, FieldValue, OrderedMap, Slide, SourceSpan, Value,
        specs::ShapeSpec,
    };

    use crate::alt_text::{AltTextValidator, E_A11_001};
    use crate::label_check::{E_A11_002, LabelCheckValidator};
    use crate::lang_validator::{E_A11_003, LangValidator};

    /// Full pipeline: deck missing lang (E-A11-003) + `severity_cards` slide missing
    /// label (E-A11-002) + shape block missing alt text (E-A11-001).
    ///
    /// Runs all three validators and asserts that combined diagnostics contain
    /// exactly one instance of each error code.
    ///
    /// Story STORY-017 Test Strategy integration test.
    ///
    /// NOTE (STORY-086 / ADR-018 v1.2 Decision-3): Pre-layout `AltTextValidator::validate()`
    /// is now restricted to `ContentBlock::Shape`. Image/Chart/Diagram are validated post-layout
    /// only. Updated to use `ContentBlock::Shape` to confirm E-A11-001 fires in the pipeline.
    #[test]
    fn test_full_validation_pipeline() {
        // Build a deck that simultaneously triggers all three validators:
        //   - No deck-level lang → LangValidator emits E-A11-003
        //   - severity_cards slide missing label → LabelCheckValidator emits E-A11-002
        //   - shape block without alt text → AltTextValidator emits E-A11-001
        //     (pre-layout validate() is Shape-only per ADR-018 v1.2 Decision-3)
        let shape_block = Block {
            content: ContentBlock::Shape(ShapeSpec {
                shape_type: slideforge_types::ShapeType::Rect,
                position: slideforge_types::specs::ShapePosition {
                    x: slideforge_types::specs::ShapeUnit::Inches(500),
                    y: slideforge_types::specs::ShapeUnit::Inches(1000),
                    width: slideforge_types::specs::ShapeUnit::Inches(2000),
                    height: slideforge_types::specs::ShapeUnit::Inches(1000),
                },
                fill: slideforge_types::FillSpec::None,
                text: None,
                alt: None,
                decorative: false,
                span: SourceSpan::default(),
            }),
            label: None,
            span: SourceSpan::default(),
        };

        // severity_cards with no label field — triggers E-A11-002
        let color_slide = Slide {
            slide_type: Arc::from("severity_cards"),
            fields: OrderedMap::new(), // no label field
            blocks: vec![shape_block], // has a shape missing alt — triggers E-A11-001
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };

        // Deck with lang: None — triggers E-A11-003
        let deck = Deck {
            slides: vec![color_slide],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: Some(Arc::from("Pipeline Test Deck")),
                slideforge_version: Arc::from("0.1.0"),
                lang: None, // triggers E-A11-003
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
        };

        let opts = ValidatorOptions::default();

        // Run all three validators and collect combined diagnostics.
        let mut all_diags = Vec::new();
        all_diags.extend(AltTextValidator.validate(&deck, &opts));
        all_diags.extend(LabelCheckValidator.validate(&deck, &opts));
        all_diags.extend(LangValidator.validate(&deck, &opts));

        // Assert each error code appears exactly once.
        let e_a11_001_count = all_diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_001)
            .count();
        let e_a11_002_count = all_diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_002)
            .count();
        let e_a11_003_count = all_diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_003)
            .count();

        assert_eq!(
            e_a11_001_count, 1,
            "expected exactly 1 E-A11-001 (missing alt text); got {e_a11_001_count} — \
             all diags: {all_diags:?}"
        );
        assert_eq!(
            e_a11_002_count, 1,
            "expected exactly 1 E-A11-002 (missing label); got {e_a11_002_count} — \
             all diags: {all_diags:?}"
        );
        assert_eq!(
            e_a11_003_count, 1,
            "expected exactly 1 E-A11-003 (missing lang); got {e_a11_003_count} — \
             all diags: {all_diags:?}"
        );

        // Verify combined count: 3 diagnostics total (no spurious extras).
        assert_eq!(
            all_diags.len(),
            3,
            "expected 3 total diagnostics (one per error code); got {} — \
             all diags: {all_diags:?}",
            all_diags.len()
        );
    }

    /// Verify the integration test also works when label IS valid and lang IS set —
    /// only the missing-alt shape should produce a diagnostic (regression guard).
    ///
    /// NOTE (STORY-086 / ADR-018 v1.2 Decision-3): Updated to use `ContentBlock::Shape`
    /// because pre-layout `validate()` is now Shape-only.
    #[test]
    fn test_full_validation_pipeline_only_alt_missing() {
        let shape_block = Block {
            content: ContentBlock::Shape(ShapeSpec {
                shape_type: slideforge_types::ShapeType::Rect,
                position: slideforge_types::specs::ShapePosition {
                    x: slideforge_types::specs::ShapeUnit::Inches(500),
                    y: slideforge_types::specs::ShapeUnit::Inches(1000),
                    width: slideforge_types::specs::ShapeUnit::Inches(2000),
                    height: slideforge_types::specs::ShapeUnit::Inches(1000),
                },
                fill: slideforge_types::FillSpec::None,
                text: None,
                alt: None,
                decorative: false,
                span: SourceSpan::default(),
            }),
            label: None,
            span: SourceSpan::default(),
        };

        let mut fields: OrderedMap<Arc<str>, FieldValue> = OrderedMap::new();
        fields.insert(
            Arc::from("label"),
            FieldValue::Literal(Value::Str(Arc::from("HIGH: action required"))),
        );

        let color_slide = Slide {
            slide_type: Arc::from("severity_cards"),
            fields,
            blocks: vec![shape_block],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };

        let deck = Deck {
            slides: vec![color_slide],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: Some(Arc::from("Partial Failure Deck")),
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from("en-US")), // valid
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
        };

        let opts = ValidatorOptions::default();

        let mut all_diags = Vec::new();
        all_diags.extend(AltTextValidator.validate(&deck, &opts));
        all_diags.extend(LabelCheckValidator.validate(&deck, &opts));
        all_diags.extend(LangValidator.validate(&deck, &opts));

        // Only E-A11-001 should be present (label and lang are valid).
        let e_a11_001_count = all_diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_001)
            .count();

        assert_eq!(
            all_diags.len(),
            1,
            "expected exactly 1 diagnostic (only missing alt); got {all_diags:?}"
        );
        assert_eq!(
            e_a11_001_count, 1,
            "the single diagnostic must be E-A11-001; got {all_diags:?}"
        );
    }
}
