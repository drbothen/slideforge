//! Implementation of the `slideforge build` subcommand.
//!
//! `run_build` is the primary entry point: it drives the full
//! parse → evaluate → brand → validate → layout → export pipeline via
//! `slideforge::build()`, renders diagnostics via `DiagnosticRenderer`, and
//! returns the appropriate [`ExitCode`] from the three-tier model.
//!
//! # Architecture constraints
//!
//! - TTY detection uses `std::io::IsTerminal` (stable since Rust 1.70);
//!   the `atty` crate is NOT used (unmaintained, per spec).
//! - Output atomicity: write to `.tmp` then `fs::rename()` to final path.
//! - `slideforge-cli` contains NO pipeline logic; it is an orchestration
//!   façade only.
//!
//! # Traceability
//!
//! - BC-1.15.001: all errors carry file:line:col span and correction hint
//! - BC-1.15.002: all errors accumulated in single pass (no fail-on-first)
//! - BC-1.15.003: three-tier exit code model
//! - AC-001 through AC-015

use std::io::IsTerminal as _;
use std::process::ExitCode;

use crate::cli::{BuildArgs, GlobalFlags};

/// Run the `slideforge build` subcommand.
///
/// Drives the full compile pipeline for `args.source`, renders all diagnostics
/// via `miette`, and returns the appropriate [`ExitCode`].
///
/// # Exit codes (BC-1.15.003)
///
/// | Condition | Code |
/// |-----------|------|
/// | Success | 0 |
/// | Parse error (E-PAR-*) | 1 |
/// | Validation error in strict mode | 2 |
/// | Export error (E-EXP-*) | 3 |
///
/// # TTY detection
///
/// Color is enabled when stderr is a TTY, `--no-color` is absent, and `--json`
/// is absent. Uses `std::io::stderr().is_terminal()` — NOT the `atty` crate.
#[must_use]
pub fn run_build(args: &BuildArgs, global: &GlobalFlags) -> ExitCode {
    todo!()
}

/// Map a [`slideforge::error::BuildError`] to the appropriate [`ExitCode`].
///
/// Differentiates E-PAR (exit 1) from E-EXP (exit 3), both of which surface
/// as `Fatal` at the `ParseSeverity` level but require distinct exit codes
/// (BC-1.15.003 postconditions 1 and 4).
///
/// | Error variant | Exit code |
/// |---------------|-----------|
/// | `ParseFailed` | 1 |
/// | `EvalFailed` (strict) | 2 |
/// | `ValidationFailed` (strict) | 2 |
/// | `Export` | 3 |
/// | Other (`Registry`, `Brand`, `Layout`, `Plugin`, `UnknownFormat`) | 1 |
#[must_use]
pub fn exit_code_for_build_error(err: &slideforge::error::BuildError) -> ExitCode {
    todo!()
}

/// Determine whether colored diagnostic output should be used.
///
/// Returns `true` when:
/// - `stderr` is a TTY (detected via `std::io::IsTerminal`), AND
/// - `--no-color` is not set, AND
/// - `--json` is not set.
///
/// This is the canonical TTY check for all `slideforge-cli` subcommands.
/// The `atty` crate is NOT used (unmaintained; `std::io::IsTerminal` is
/// stable since Rust 1.70).
#[must_use]
pub fn should_use_color(global: &GlobalFlags) -> bool {
    std::io::stderr().is_terminal() && !global.no_color && !global.json
}
