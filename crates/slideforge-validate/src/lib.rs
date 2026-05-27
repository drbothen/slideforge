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

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod alt_text;
mod utils;

pub use alt_text::AltTextValidator;
pub use utils::is_blank;
