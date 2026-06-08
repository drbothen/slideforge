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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::process::ExitCode;

    use super::{
        EXIT_EXPORT_ERROR, EXIT_PARSE_ERROR, EXIT_SUCCESS, EXIT_VALIDATION_ERROR,
        exit_code_for_severity,
    };
    use slideforge::ParseSeverity;

    // ── BC-1.15.003 invariant: severity → exit code mapping ───────────────────

    /// BC-1.15.003 invariant: None severity (empty sink, success) → exit 0.
    ///
    /// This function is implemented; the test validates the constant and mapping.
    #[test]
    fn test_BC_1_15_003_invariant_exit_severity_none_maps_to_zero() {
        assert_eq!(
            exit_code_for_severity(None),
            ExitCode::SUCCESS,
            "BC-1.15.003: empty diagnostic sink must produce exit 0"
        );
        assert_eq!(
            EXIT_SUCCESS, 0,
            "EXIT_SUCCESS constant must be 0"
        );
    }

    /// BC-1.15.003 postcondition 5: warnings-only → exit 0.
    #[test]
    fn test_BC_1_15_003_invariant_exit_severity_warning_maps_to_zero() {
        assert_eq!(
            exit_code_for_severity(Some(ParseSeverity::Warning)),
            ExitCode::SUCCESS,
            "BC-1.15.003: Warning severity must produce exit 0"
        );
    }

    /// BC-1.15.003 postcondition 2: Error severity (strict) → exit 2.
    #[test]
    fn test_BC_1_15_003_invariant_exit_severity_error_maps_to_two() {
        assert_eq!(
            exit_code_for_severity(Some(ParseSeverity::Error)),
            ExitCode::from(EXIT_VALIDATION_ERROR),
            "BC-1.15.003: Error severity (strict) must produce exit 2"
        );
        assert_eq!(
            EXIT_VALIDATION_ERROR, 2,
            "EXIT_VALIDATION_ERROR constant must be 2"
        );
    }

    /// BC-1.15.003 postcondition 1: Fatal severity (parse) → exit 1.
    #[test]
    fn test_BC_1_15_003_invariant_exit_severity_fatal_maps_to_one() {
        assert_eq!(
            exit_code_for_severity(Some(ParseSeverity::Fatal)),
            ExitCode::from(EXIT_PARSE_ERROR),
            "BC-1.15.003: Fatal severity (parse) must produce exit 1"
        );
        assert_eq!(
            EXIT_PARSE_ERROR, 1,
            "EXIT_PARSE_ERROR constant must be 1"
        );
    }

    /// BC-1.15.003 postcondition 4: export errors are distinct from parse errors.
    ///
    /// The EXIT_EXPORT_ERROR constant must be 3, distinct from EXIT_PARSE_ERROR (1)
    /// and EXIT_VALIDATION_ERROR (2). This is a constant invariant test.
    #[test]
    fn test_BC_1_15_003_invariant_export_exit_code_is_three() {
        assert_eq!(
            EXIT_EXPORT_ERROR, 3,
            "BC-1.15.003 postcondition 4: EXIT_EXPORT_ERROR must be 3"
        );
        assert_ne!(
            EXIT_EXPORT_ERROR, EXIT_PARSE_ERROR,
            "Export and parse exit codes must be distinct"
        );
        assert_ne!(
            EXIT_EXPORT_ERROR, EXIT_VALIDATION_ERROR,
            "Export and validation exit codes must be distinct"
        );
    }

    /// BC-1.15.003 invariant: exit_code_for_build_error (in build.rs) correctly
    /// maps BuildError variants to exit codes.
    ///
    /// This test exercises the build.rs stub — it panics (todo!()), which is the
    /// Red Gate failure.
    #[test]
    fn test_BC_1_15_003_invariant_build_error_parse_failed_maps_to_exit_1_via_build_module() {
        use crate::commands::build::exit_code_for_build_error;
        use slideforge::error::BuildError;

        let err = BuildError::ParseFailed {
            diagnostics: vec![],
            count: 1,
        };
        // exit_code_for_build_error is todo!() — this panics → Red Gate FAIL.
        let code = exit_code_for_build_error(&err);
        assert_eq!(
            code,
            ExitCode::from(1),
            "BC-1.15.003: ParseFailed must produce exit 1 via exit_code_for_build_error"
        );
    }
}
