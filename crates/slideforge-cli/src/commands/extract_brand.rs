//! Stub for the `slideforge extract-brand` subcommand (STORY-057).
//!
//! `slideforge extract-brand` extracts brand configuration from an existing
//! PPTX or DOCX file and writes a `brand.toml`.  This module is a stub that
//! will be fully implemented in STORY-057.

use std::process::ExitCode;

use crate::cli::GlobalFlags;

/// Arguments for the `slideforge extract-brand` subcommand.
///
/// Full argument definition is deferred to STORY-057.
#[derive(Debug, clap::Args)]
pub struct ExtractBrandArgs {
    /// Path to the PPTX or DOCX file to extract brand from.
    pub source: std::path::PathBuf,
}

/// Run the `slideforge extract-brand` subcommand.
///
/// Stub implementation — will be completed in STORY-057.
#[must_use]
pub fn run_extract_brand(_args: &ExtractBrandArgs, _global: &GlobalFlags) -> ExitCode {
    todo!("slideforge extract-brand is not yet implemented — see STORY-057")
}
