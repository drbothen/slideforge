//! Exit code mapping for the `slideforge` CLI.
//!
//! The three-tier exit code model (BC-1.15.003):
//!
//! | Condition | Exit code |
//! |-----------|-----------|
//! | Success (no errors, or warnings only) | 0 |
//! | Parse errors (E-PAR-*) — always fatal | 1 |
//! | Validation errors in strict mode (E-EVL-*, E-A11-*, etc.) | 2 |
//! | Export errors (E-EXP-*) — always fatal | 3 |
//!
//! `--warn-only` NEVER demotes parse or export errors (BC-1.15.003
//! invariants 1 and 2).

use std::process::ExitCode;

use slideforge::ParseSeverity;

/// Exit code for successful builds (no errors, or only cosmetic diagnostics).
pub const EXIT_SUCCESS: u8 = 0;

/// Exit code for parse failures (E-PAR-*).
///
/// Parse errors are always fatal regardless of `--warn-only`
/// (BC-1.15.003 invariant 1).
pub const EXIT_PARSE_ERROR: u8 = 1;

/// Exit code for validation failures in strict mode (E-EVL-*, E-A11-*, etc.).
///
/// With `--warn-only` these are demoted to warnings and exit code is 0.
pub const EXIT_VALIDATION_ERROR: u8 = 2;

/// Exit code for export failures (E-EXP-*).
///
/// Export errors are always fatal regardless of `--warn-only`
/// (BC-1.15.003 invariant 2).
pub const EXIT_EXPORT_ERROR: u8 = 3;

/// Map a [`ParseSeverity`] to the appropriate [`ExitCode`].
///
/// This implements the basic two-axis mapping for severity alone.  The CLI
/// layer additionally differentiates E-PAR vs E-EXP at the `BuildError`
/// variant level (both map to `Fatal` in `ParseSeverity`, but exit code 1
/// vs 3 respectively).  For the full three-tier mapping see
/// [`exit_code_for_build_error`] in `commands/build.rs`.
///
/// | Severity | Exit code |
/// |----------|-----------|
/// | `None` (empty sink) | 0 |
/// | `Warning` | 0 |
/// | `Error` (strict) | 2 |
/// | `Fatal` | 1 (default — E-PAR path; E-EXP → 3 via variant) |
#[must_use]
pub fn exit_code_for_severity(sev: Option<ParseSeverity>) -> ExitCode {
    match sev {
        None | Some(ParseSeverity::Warning) => ExitCode::SUCCESS,
        Some(ParseSeverity::Error) => ExitCode::from(EXIT_VALIDATION_ERROR),
        Some(ParseSeverity::Fatal) => ExitCode::from(EXIT_PARSE_ERROR),
    }
}
