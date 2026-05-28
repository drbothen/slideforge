//! Brand loading and extraction for slideforge.
//!
//! This crate implements the `BrandProvider` plugin trait for loading brand
//! configuration from existing `.pptx` and `.docx` template files.
//!
//! ## Overview
//!
//! The primary entry point is [`BrandLoader`], which:
//!
//! 1. Opens the `.pptx` or `.docx` file as a ZIP archive.
//! 2. Detects the file type from internal paths (`ppt/theme/theme1.xml` vs
//!    `word/theme/theme1.xml`).
//! 3. Extracts all 12 OOXML scheme color slots in ECMA-376 order.
//! 4. Extracts heading and body font names.
//! 5. Optionally detects and extracts a logo image.
//! 6. Returns a [`BrandTemplate`] consumed by the layout and export stages.
//!
//! ## Error Codes
//!
//! | Code | Variant | Severity |
//! |------|---------|---------|
//! | `E-BRD-001` | [`BrandError::FileNotFound`] | broken (exit 4) |
//! | `E-BRD-002` | [`BrandError::ParseError`] | broken (exit 4) |
//! | `E-BRD-003` | [`BrandError::MissingColorSlot`] | cosmetic (exit 0) |
//! | `E-BRD-004` | [`BrandError::FontUnavailable`] | cosmetic (exit 0) |
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
pub mod font;
pub mod inference;
pub mod layout_xml;
pub mod layouts;
pub mod loader;
pub mod logo;
pub mod synthesizer;
pub mod template;
pub mod toml_schema;

// Re-export the most commonly used types at the crate root.
pub use context::BrandLoadContext;
pub use error::{BrandError, E_BRD_001, E_BRD_002, E_BRD_003, E_BRD_004};
pub use layouts::{LayoutPlaceholder, SlideLayoutDef};
pub use loader::BrandLoader;
pub use synthesizer::BrandSynthesizer;
pub use template::{BrandFonts, BrandTemplate, COLOR_SLOT_NAMES, ColorSlot, LogoAsset, MasterIds};
pub use toml_schema::{BrandConfig, ColorConfig, FontConfig, FooterConfig, LogoConfig};
