//! Stub for the `slideforge init` subcommand (STORY-057).
//!
//! `slideforge init` scaffolds a new slideforge project in a target directory.
//! This module is a stub that will be fully implemented in STORY-057.

use std::process::ExitCode;

use crate::cli::GlobalFlags;

/// Arguments for the `slideforge init` subcommand.
///
/// Full argument definition is deferred to STORY-057.
#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Name or path for the new project.
    pub name: Option<String>,
}

/// Run the `slideforge init` subcommand.
///
/// Stub implementation — will be completed in STORY-057.
#[must_use]
pub fn run_init(_args: &InitArgs, _global: &GlobalFlags) -> ExitCode {
    todo!("slideforge init is not yet implemented — see STORY-057")
}
