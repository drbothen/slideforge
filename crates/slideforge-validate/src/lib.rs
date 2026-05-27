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
mod mode;
mod utils;
mod zero_slide;

pub use alt_text::AltTextValidator;
pub use canvas_overflow::CanvasOverflowValidator;
pub use error_slide::{ERROR_PLACEHOLDER_SLIDE_TYPE, error_slide_placeholder};
pub use mode::{ValidationConfig, ValidationMode};
pub use utils::is_blank;
pub use zero_slide::ZeroSlideValidator;
