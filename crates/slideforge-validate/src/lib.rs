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
mod label_check;
mod lang_validator;
mod mode;
mod utils;
mod wcag;
mod zero_slide;

pub use alt_text::AltTextValidator;
pub use canvas_overflow::CanvasOverflowValidator;
pub use error_slide::{ERROR_PLACEHOLDER_SLIDE_TYPE, error_slide_placeholder};
pub use label_check::{COLOR_CODED_TYPES, LabelCheckValidator};
pub use lang_validator::{LangValidator, inject_lang_default};
pub use mode::{ValidationConfig, ValidationMode};
pub use utils::is_blank;
pub use wcag::{contrast_ratio, parse_hex_color, relative_luminance, srgb_component_to_linear, wcag_aa_passes};
pub use zero_slide::ZeroSlideValidator;
