//! Built-in inline formatting plugin — [`DefaultInlineFormat`].
//!
//! This module provides the default `InlineFormat` implementation that handles
//! all 12 [`slideforge_types::InlineNode`] variants across all three output
//! formats defined by [`crate::traits::InlineOutputFormat`]:
//!
//! - [`crate::traits::InlineOutputFormat::Ooxml`] — OOXML `<a:r>` runs
//! - [`crate::traits::InlineOutputFormat::Html`] — HTML element fragments
//! - [`crate::traits::InlineOutputFormat::Markdown`] — `CommonMark` notation
//!
//! ## Dog-fooding guarantee (BC-5.02.002)
//!
//! `DefaultInlineFormat` imports ONLY from this crate's public API and
//! `slideforge-types`. No bundled plugin bypasses the trait API.
//!
//! ## Forbidden dependencies
//!
//! This module MUST NOT import from: `slideforge-pptx`, `slideforge-docx`,
//! `slideforge-pdf`, `slideforge-html`, `slideforge-eval`, `slideforge-syntax`,
//! or `slideforge-layout`. The crate-level `Cargo.toml` enforces this at
//! compile time.

pub mod default_formatter;

pub use default_formatter::DefaultInlineFormat;
