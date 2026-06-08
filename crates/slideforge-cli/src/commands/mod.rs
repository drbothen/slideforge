//! Subcommand dispatch for the `slideforge` CLI.
//!
//! [`dispatch`] routes a parsed [`crate::cli::Cli`] to the appropriate
//! subcommand handler and returns the [`std::process::ExitCode`] to
//! propagate from `main()`.

pub mod build;
pub mod extract_brand;
pub mod init;
pub mod watch;

use std::process::ExitCode;

use crate::cli::{Cli, Command};

/// Dispatch the parsed CLI arguments to the appropriate subcommand handler.
///
/// Returns the [`ExitCode`] that `main()` should propagate to the OS.
/// The three-tier exit code model (BC-1.15.003):
///
/// - `0` — success
/// - `1` — parse error (E-PAR-*)
/// - `2` — validation error in strict mode
/// - `3` — export error (E-EXP-*)
#[must_use]
pub fn dispatch(cli: &Cli) -> ExitCode {
    match &cli.command {
        Command::Build(args) => build::run_build(args, &cli.global),
        Command::Watch(args) => watch::run_watch(args, &cli.global),
        Command::Init(args) => init::run_init(args, &cli.global),
        Command::ExtractBrand(args) => extract_brand::run_extract_brand(args, &cli.global),
    }
}
