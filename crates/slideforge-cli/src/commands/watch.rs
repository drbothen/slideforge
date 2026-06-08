//! Stub for the `slideforge watch` subcommand (STORY-056).
//!
//! Watch mode rebuilds the source file on every detected change and
//! renders diagnostics incrementally. This module is a stub that will be
//! fully implemented in STORY-056.

use std::process::ExitCode;

use crate::cli::GlobalFlags;

/// Arguments for the `slideforge watch` subcommand.
///
/// Full argument definition is deferred to STORY-056.
#[derive(Debug, clap::Args)]
pub struct WatchArgs {
    /// Path to the `.sf` source file to watch.
    pub source: std::path::PathBuf,
}

/// Run the `slideforge watch` subcommand.
///
/// Stub implementation — will be completed in STORY-056.
#[must_use]
pub fn run_watch(_args: &WatchArgs, _global: &GlobalFlags) -> ExitCode {
    todo!("slideforge watch is not yet implemented — see STORY-056")
}
