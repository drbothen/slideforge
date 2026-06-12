//! Core IR types for the slideforge pipeline.
//!
//! This crate provides the foundational data types shared across all slideforge
//! crates. It is a pure leaf crate with no workspace crate dependencies.
//!
//! ## Two-IR Model
//!
//! The slideforge pipeline produces two intermediate representations:
//!
//! - [`Deck`] — semantic, pre-layout representation of slide content
//! - `LaidOutDeck` (in `slideforge-layout`) — geometric, post-layout representation with positioned shapes
//!
//! Exporters consume both: PPTX needs semantic info for placeholders;
//! PDF/HTML operate off `LaidOutDeck`.
//!
//! ## Design Invariants
//!
//! All public IR types implement `Hash + Eq + Clone + Debug` for comemo
//! compatibility and Kani bounded-model checking (Phase 6).
//!
//! All string fields use `Arc<str>` to avoid allocation on clone.
//!
//! All floating-point values use [`ordered_float::OrderedFloat`] to enable
//! `Hash + Eq` without sacrificing the standard `f64` arithmetic API.
//!
//! All coordinate and size fields use integer [`Emu`] (English Metric Units)
//! rather than `f64` to enable exact arithmetic proofs in Phase 6.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod block;
pub mod brand;
pub mod deck;
pub mod emu;
pub mod error;
pub mod inline;
pub mod math;
pub mod ordered_map;
pub mod precedence;
pub mod register;
pub mod shape_types;
pub mod slide;
pub mod slide_overlay;
pub mod span;
pub mod specs;
pub mod type_kind;
pub mod value;

// Re-export the most commonly used types at the crate root for ergonomics.
// STORY-087 pass-2: ColorBarSpec is re-exported for use in thread_fields_to_blocks
// and in tests that construct ContentBlock::ColorBar directly.
pub use block::{Block, BulletItem, ColorBarSpec, ContentBlock, TextBlock, TextTag};
pub use brand::{Brand, BrandFonts, BrandPalette, LayoutDefinition};
pub use deck::{
    CANONICAL_MANUAL_SECTION_TYPES, Deck, DeckMetadata, ERROR_PLACEHOLDER_SLIDE_TYPE,
    PPTX_SLIDE_ID_START, SectionBlock, SlideSectionEntry,
};
pub use emu::{CANVAS_HEIGHT, CANVAS_WIDTH, Emu, SLIDE_HEIGHT, SLIDE_WIDTH};
pub use error::TypeError;
pub use inline::{InlineNode, display_text_is_empty};
pub use math::MathNode;
pub use ordered_map::OrderedMap;
pub use precedence::MergePrecedence;
pub use register::{Register, RegisteredContent};
pub use shape_types::{FillSpec, LayoutWarning, Rgb, ShapeType, ShapeTypeError};
pub use slide::{FieldValue, Slide, StringPart};
pub use slide_overlay::SlideOverlay;
pub use span::SourceSpan;
pub use specs::{
    AltText, ChartSpec, DiagramSpec, ImageSpec, NormalizedDiagramSvg, ShapePosition, ShapeSpec,
    ShapeUnit, TableSpec,
};
pub use type_kind::TypeKind;
pub use value::Value;

/// Default BCP-47 language tag when a deck carries no `lang` declaration.
///
/// The canonical default is `"en"` per BC-5.01.004 — NOT `"en-US"`.
/// This constant is the shared source of truth for the no-lang default across
/// all exporter surfaces: `word/document.xml` run `<w:lang>`, `docProps/core.xml`
/// `<dc:language>`, PPTX `<a:rPr lang="">`, and any future exporter surface that
/// requires a language tag (BC-5.01.004 / BC-5.01.005).
///
/// Both `slideforge-docx` and `slideforge-pptx` import this constant so that a
/// single edit propagates to all surfaces simultaneously (TD-VSDD-060).
pub const DEFAULT_DECK_LANG: &str = "en";

/// Validate that `lang` contains no XML-1.0-illegal control characters.
///
/// XML 1.0 §2.2 defines legal characters as:
/// `#x9 | #xA | #xD | [#x20–#xD7FF] | [#xE000–#xFFFD] | [#x10000–#x10FFFF]`
///
/// The illegal ranges relevant here are: U+0000–U+0008, U+000B, U+000C,
/// U+000E–U+001F, U+FFFE, U+FFFF. A valid BCP-47 tag (ASCII alphanumeric + `-`)
/// always passes unchanged (lossless per BC-5.01.005 invariant 1).
///
/// On success `lang` is safe to embed in any XML-1.0 document. Each exporter
/// wraps the error case into its own error type so error messages carry
/// exporter-specific context (e.g., `DocxError::InvalidLanguageTag` for DOCX,
/// `PptxError::InvalidLanguageTag` for PPTX).
///
/// This function lives in `slideforge-types` — the leaf crate with no workspace
/// dependencies — so it can be used by every exporter without creating circular
/// dependencies (SEC-099-001 / SEC-039-001 / CWE-116).
///
/// # Errors
///
/// Returns `Err(ch)` where `ch` is the first XML-1.0-illegal character found in
/// `lang` (i.e. a character in U+0000–U+0008, U+000B, U+000C, U+000E–U+001F,
/// U+FFFE, or U+FFFF).
pub fn validate_xml_lang(lang: &str) -> Result<(), char> {
    for ch in lang.chars() {
        let code = ch as u32;
        let illegal = matches!(
            code,
            0x0000..=0x0008 | 0x000B | 0x000C | 0x000E..=0x001F | 0xFFFE | 0xFFFF
        );
        if illegal {
            return Err(ch);
        }
    }
    Ok(())
}
