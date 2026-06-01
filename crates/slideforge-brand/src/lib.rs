//! Brand loading and synthesis for slideforge.
//!
//! This crate implements the `BrandProvider` plugin trait for:
//!
//! - **Loading** brand configuration from existing `.pptx` and `.docx` template
//!   files via [`BrandLoader`].
//! - **Synthesizing** a complete brand from a `brand.toml` file via
//!   [`BrandSynthesizer`].
//!
//! ## Brand Loading ([`BrandLoader`])
//!
//! [`BrandLoader`] opens a `.pptx` or `.docx` template and:
//!
//! 1. Opens the file as a ZIP archive.
//! 2. Detects the file type from internal paths (`ppt/theme/theme1.xml` vs
//!    `word/theme/theme1.xml`).
//! 3. Extracts all 12 OOXML scheme color slots in ECMA-376 order.
//! 4. Extracts heading and body font names.
//! 5. Optionally detects and extracts a logo image.
//! 6. Returns a [`BrandTemplate`] consumed by the layout and export stages.
//!
//! ## Brand Synthesis ([`BrandSynthesizer`])
//!
//! [`BrandSynthesizer`] synthesizes a complete [`BrandTemplate`] from a
//! `brand.toml` file (STORY-023). The synthesis pipeline:
//!
//! 1. Validates the `[logo]` path (required for synthesized brands).
//! 2. Applies deterministic color inference for any absent of the 12 OOXML
//!    color slots (ac-003: hue rotation, lightness adjustment).
//! 3. Generates 31 slide layout definitions.
//! 4. Returns `(BrandTemplate, Vec<BrandError>)` — the template plus any
//!    cosmetic warnings for inferred color slots.
//!
//! Layout XML serialization is performed **lazily** by the PPTX exporter
//! (STORY-037), which calls [`layout_xml::serialize_layout_to_xml`] per
//! layout when building each ZIP entry. The synthesizer itself is pure
//! (no I/O) and does not produce XML bytes.
//!
//! ## Error Codes
//!
//! | Code | Variants | Severity |
//! |------|----------|---------|
//! | `E-BRD-001` | [`BrandError::FileNotFound`], [`BrandError::LogoRequired`], [`BrandError::TomlReadError`] | broken (exit 4) |
//! | `E-BRD-002` | [`BrandError::ParseError`], [`BrandError::TomlParseError`] | broken (exit 4) |
//! | `E-BRD-003` | [`BrandError::MissingColorSlot`] | cosmetic (exit 0) |
//! | `E-BRD-004` | [`BrandError::FontUnavailable`], [`BrandError::DeclaredFontUnavailable`] | cosmetic (exit 0) |
//! | `E-BRD-005` | [`BrandError::InvalidHexColor`] | cosmetic (exit 0) |
//! | `E-BRD-006` | [`BrandError::OutputExists`] | broken (exit 4) |
//! | `E-BRD-007` | [`BrandError::LogoOutsideBrandDir`] | broken (exit 4) |
//!
//! ## Design Constraints
//!
//! This crate MUST NOT depend on `slideforge-eval`, `slideforge-syntax`,
//! `slideforge-validate`, `slideforge-layout`, `slideforge-pptx`,
//! `slideforge-pdf`, `slideforge-html`, or `slideforge-cli`
//! (Architecture Compliance Rule 5 from STORY-022).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod color;
pub mod context;
pub mod error;
pub mod extractor;
pub mod font;
pub mod footer;
pub mod inference;
pub mod layout_xml;
pub mod layouts;
pub mod loader;
pub mod logo;
pub mod overlay;
pub mod synthesizer;
pub mod template;
pub mod toml_schema;

// Re-export the most commonly used types at the crate root.
pub use context::BrandLoadContext;
pub use error::{
    BrandError, E_BRD_001, E_BRD_002, E_BRD_003, E_BRD_004, E_BRD_005, E_BRD_006, E_BRD_007,
};
pub use extractor::{BrandExtractionResult, BrandExtractor};
pub use layouts::{LayoutPlaceholder, SlideLayoutDef};
pub use loader::BrandLoader;
pub use overlay::{BrandOverlay, LogoOverride, resolve_overlay};
pub use synthesizer::BrandSynthesizer;
pub use template::{BrandFonts, BrandTemplate, COLOR_SLOT_NAMES, ColorSlot, LogoAsset, MasterIds};
pub use toml_schema::{BrandConfig, ColorConfig, FontConfig, FooterConfig, LogoConfig};

// STORY-076 Red Gate tests: Transform-Aware Theme Color Extraction.
// These tests exercise the `ColorSlot.is_derived` field and the tracing::warn!
// emitted by `parse_theme_colors` for srgbClr elements with transform children.
// All tests fail to compile until `ColorSlot.is_derived: bool` is added to
// `template.rs` and threaded through `color.rs` and `extractor.rs`.
#[cfg(test)]
mod srgbclr_transform_tests;
