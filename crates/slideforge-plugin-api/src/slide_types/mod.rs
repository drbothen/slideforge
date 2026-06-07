//! Built-in slide type implementations and the [`SlideTypeRegistry`].
//!
//! This module provides:
//!
//! 1. **[`SlideTypeRegistry`]** — runtime dispatch from DSL keyword to
//!    [`SlideType`](crate::traits::SlideType) implementation, with typo suggestions.
//! 2. **[`validate_fields`]** — accumulates all field-validation diagnostics
//!    for a slide against its declared type schema.
//! 3. **Individual slide type structs** — one per built-in type keyword.
//!    34 built-in types: 31 implemented in STORY-003 + 3 color-coded types added in STORY-087.
//! 4. **[`SLIDE_TYPE_REGISTRY`]** — a process-wide lazy singleton holding the
//!    default registry for use by the evaluator and layout engine.
//! 5. **[`common_optional_fields`]** — returns the universal optional fields
//!    shared by all built-in slide types.
//!
//! ## Adding a new slide type
//!
//! 1. Create `crates/slideforge-plugin-api/src/slide_types/<name>.rs`
//! 2. Implement `SlideType` for the new struct
//! 3. `pub mod <name>;` in this file
//! 4. Add `r.register(Box::new(<Name>SlideType::new()))` in
//!    [`SlideTypeRegistry::default`]
//! 5. Update the count assertion in `test_bc_1_03_017_all_keywords_len_equals_34`
//!    (registry count) and the `SLIDE_TYPE_KEYWORDS` set in `slideforge-syntax::keywords`
//!    (keyword count — currently one more than the registry due to `severity_cards`)

use std::sync::Arc;
use std::sync::LazyLock;

use crate::traits::FieldDef;

pub mod agenda;
pub mod bio;
pub mod blank;
pub mod chart;
pub mod closing;
pub mod code_sample;
pub mod comparison;
pub mod content;
pub mod diagram;
pub mod executive_summary;
pub mod financials;
pub mod image;
pub mod kpi_dashboard;
pub mod matrix;
pub mod org_chart;
pub mod problem_statement;
pub mod process_flow;
pub mod progress_bar;
pub mod quote;
pub mod recommendation;
pub mod registry;
pub mod risk_register;
pub mod roadmap;
pub mod screenshot;
pub mod section_break;
pub mod stat_callout;
pub mod status;
pub mod survey_results;
pub mod team;
pub mod timeline;
pub mod title;
pub mod toc;
pub mod two_col;
pub mod video;
pub mod weighted_composite;

pub use registry::{SlideTypeRegistry, validate_fields};

/// Returns the universal optional fields shared by all 34 built-in slide types.
///
/// These fields are accepted on every slide regardless of type. Individual
/// slide types call this function and extend their type-specific optional
/// fields with the result.
///
/// # Universal optional fields
///
/// | Field | Purpose |
/// |-------|---------|
/// | `notes` | Presenter notes (writing register) |
/// | `report` | Reader-facing report register |
/// | `detail` | Document-only detail register |
/// | `tags` | User-defined tags for filtering and grouping |
/// | `alt` | Accessibility alt text override at slide level |
/// | `lang` | BCP-47 language tag override for this slide |
/// | `decorative` | When true, marks the slide as decorative |
/// | `footer` | Override footer text for this slide |
/// | `logo` | Override the brand logo for this slide |
#[must_use]
pub fn common_optional_fields() -> Vec<FieldDef> {
    vec![
        FieldDef {
            name: Arc::from("notes"),
            description: Arc::from("Presenter notes for this slide (notes writing register)."),
            required: false,
            default_value: None,
        },
        FieldDef {
            name: Arc::from("report"),
            description: Arc::from(
                "Reader-facing report content for this slide (report writing register).",
            ),
            required: false,
            default_value: None,
        },
        FieldDef {
            name: Arc::from("detail"),
            description: Arc::from(
                "Document-only detail content for this slide (detail writing register).",
            ),
            required: false,
            default_value: None,
        },
        FieldDef {
            name: Arc::from("tags"),
            description: Arc::from("User-defined tags for filtering and grouping slides."),
            required: false,
            default_value: None,
        },
        FieldDef {
            name: Arc::from("alt"),
            description: Arc::from(
                "Accessibility alt text override at the slide level. Overrides element-level alt.",
            ),
            required: false,
            default_value: None,
        },
        FieldDef {
            name: Arc::from("lang"),
            description: Arc::from(
                "BCP-47 language tag override for this slide (e.g., \"fr-FR\"). \
                 Overrides the deck-level `lang` setting.",
            ),
            required: false,
            default_value: None,
        },
        FieldDef {
            name: Arc::from("decorative"),
            description: Arc::from(
                "When true, marks the slide as decorative (empty alt in accessibility output). \
                 Overrides element-level `alt` fields.",
            ),
            required: false,
            default_value: None,
        },
        FieldDef {
            name: Arc::from("footer"),
            description: Arc::from("Override footer text for this slide."),
            required: false,
            default_value: None,
        },
        FieldDef {
            name: Arc::from("logo"),
            description: Arc::from("Override the brand logo for this slide."),
            required: false,
            default_value: None,
        },
    ]
}

/// Process-wide lazy singleton of the default slide type registry.
///
/// The evaluator and layout engine should use this singleton rather than
/// constructing their own registry. The registry is initialized once and
/// then immutable for the process lifetime.
///
/// # Thread safety
///
/// `LazyLock` guarantees that the registry is initialized exactly once.
/// The contained `SlideTypeRegistry` is not itself `Sync`, but access is
/// read-only after initialization (no mutation after construction).
///
/// For mutable registries (plugin testing, extension points), construct an
/// independent `SlideTypeRegistry` instance.
pub static SLIDE_TYPE_REGISTRY: LazyLock<SlideTypeRegistry> =
    LazyLock::new(SlideTypeRegistry::default);
