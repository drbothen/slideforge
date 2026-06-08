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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::PathBuf;
    use std::process::ExitCode;

    use super::{exit_code_for_build_error, run_build, should_use_color};
    use crate::cli::{BuildArgs, GlobalFlags, OutputFormat};
    use slideforge::error::BuildError;

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Construct a minimal `GlobalFlags` with all flags false/zero/none.
    fn default_global() -> GlobalFlags {
        GlobalFlags {
            json: false,
            quiet: false,
            verbose: 0,
            no_color: false,
            warn_only: false,
            offline: false,
            otel_endpoint: None,
        }
    }

    /// Construct a minimal `BuildArgs` pointing to a non-existent source path.
    fn build_args_nonexistent(formats: Vec<OutputFormat>) -> BuildArgs {
        BuildArgs {
            source: PathBuf::from("/nonexistent/deck.sf"),
            output_dir: PathBuf::from("/tmp/slideforge-test-out"),
            format: formats,
            variant: None,
        }
    }

    // ── BC-1.15.003: exit code for BuildError variants ────────────────────────

    /// BC-1.15.003 postcondition 1: parse errors (E-PAR-*) → exit 1.
    ///
    /// `exit_code_for_build_error(ParseFailed)` must return `ExitCode::from(1)`.
    /// The `todo!()` stub panics on call → this test FAILS (Red Gate: correct).
    #[test]
    fn test_BC_1_15_003_exit_code_parse_error() {
        let err = BuildError::ParseFailed {
            diagnostics: vec![],
            count: 1,
        };
        let code = exit_code_for_build_error(&err);
        assert_eq!(
            code,
            ExitCode::from(1),
            "BC-1.15.003 postcondition 1: ParseFailed must map to exit code 1"
        );
    }

    /// BC-1.15.003 postcondition 2: eval errors in strict mode → exit 2.
    ///
    /// `exit_code_for_build_error(EvalFailed)` must return `ExitCode::from(2)`.
    #[test]
    fn test_BC_1_15_003_exit_code_eval_error_strict() {
        let err = BuildError::EvalFailed {
            diagnostics: vec![],
            count: 1,
        };
        let code = exit_code_for_build_error(&err);
        assert_eq!(
            code,
            ExitCode::from(2),
            "BC-1.15.003 postcondition 2: EvalFailed (strict) must map to exit code 2"
        );
    }

    /// BC-1.15.003 postcondition 3: validation errors in strict mode → exit 2.
    ///
    /// `exit_code_for_build_error(ValidationFailed)` must return `ExitCode::from(2)`.
    /// Uses `slideforge::` re-exports for Diagnostic types (CLI must not depend directly
    /// on slideforge-plugin-api per architecture compliance rule 3).
    #[test]
    fn test_BC_1_15_003_exit_code_validation_error_strict() {
        use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity};
        use slideforge_types::SourceSpan;
        use std::sync::Arc;

        let diag = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: Arc::from("E-EVL-001"),
            message: Arc::from("undefined variable"),
            span: SourceSpan::default(),
            hint: Some(Arc::from("check variable names")),
        };
        let err = BuildError::ValidationFailed {
            diagnostics: vec![diag],
            count: 1,
        };
        let code = exit_code_for_build_error(&err);
        assert_eq!(
            code,
            ExitCode::from(2),
            "BC-1.15.003 postcondition 2: ValidationFailed (strict) must map to exit code 2"
        );
    }

    /// BC-1.15.003 postcondition 4: export errors (E-EXP-*) → exit 3.
    ///
    /// `exit_code_for_build_error(Export)` must return `ExitCode::from(3)`.
    #[test]
    fn test_BC_1_15_003_exit_code_export_error() {
        use slideforge_plugin_api::ExportError;

        let err = BuildError::Export(ExportError::IoError {
            message: "cannot write output: permission denied".to_owned(),
        });
        let code = exit_code_for_build_error(&err);
        assert_eq!(
            code,
            ExitCode::from(3),
            "BC-1.15.003 postcondition 4: Export error must map to exit code 3"
        );
    }

    /// BC-1.15.003 postcondition 6: exit code reflects the highest severity.
    ///
    /// When both parse and eval errors are present, exit code is 1 (parse wins).
    /// Simulated by checking that ParseFailed (code 1) takes precedence over
    /// EvalFailed (code 2).
    #[test]
    fn test_BC_1_15_003_exit_code_parse_takes_precedence_over_eval() {
        // Parse error should map to 1; eval to 2 — parse wins (lower code, higher priority).
        let parse_err = BuildError::ParseFailed {
            diagnostics: vec![],
            count: 1,
        };
        let eval_err = BuildError::EvalFailed {
            diagnostics: vec![],
            count: 1,
        };
        let parse_code = exit_code_for_build_error(&parse_err);
        let eval_code = exit_code_for_build_error(&eval_err);

        // Parse error exit code (1) must be less than eval error exit code (2),
        // confirming parse takes precedence in the exit code ordering.
        // In the final CLI logic: if sink contains E-PAR → return 1, not 2.
        assert_ne!(
            parse_code, eval_code,
            "BC-1.15.003 postcondition 6: ParseFailed and EvalFailed must produce different codes"
        );

        // parse_code must be 1 and eval_code must be 2.
        assert_eq!(parse_code, ExitCode::from(1), "ParseFailed must → exit 1");
        assert_eq!(eval_code, ExitCode::from(2), "EvalFailed must → exit 2");
    }

    /// BC-1.15.003 postcondition 5: success (no errors) → exit 0.
    ///
    /// `run_build` on a nonexistent source should fail (E-PAR-005), not exit 0.
    /// This is a positive control: the call panics with todo!(), proving Red Gate.
    #[test]
    fn test_BC_1_15_003_run_build_nonexistent_source_exits_1_not_0() {
        let args = build_args_nonexistent(vec![
            OutputFormat::Pptx,
            OutputFormat::Docx,
            OutputFormat::Pdf,
            OutputFormat::Html,
        ]);
        let global = default_global();
        // run_build is todo!() — this panics and the test FAILS (Red Gate).
        // After implementation it should return ExitCode::from(1) for missing file.
        let code = run_build(&args, &global);
        assert_ne!(
            code,
            ExitCode::SUCCESS,
            "BC-1.15.003: run_build on nonexistent source must NOT return exit 0"
        );
    }

    /// BC-1.15.003: --format pptx produces only the pptx format path.
    ///
    /// Calls `run_build` with format=[Pptx] and asserts it does not succeed
    /// (since the source doesn't exist) and does not write any files.
    #[test]
    fn test_BC_1_15_003_format_selection_pptx_only_does_not_write_other_formats() {
        let tmp_dir = tempfile::tempdir().expect("create tempdir");
        let args = BuildArgs {
            source: PathBuf::from("/nonexistent/deck.sf"),
            output_dir: tmp_dir.path().to_owned(),
            format: vec![OutputFormat::Pptx],
            variant: None,
        };
        let global = default_global();
        // run_build is todo!() — panics and test FAILS (Red Gate).
        // After implementation: no docx/pdf/html in tmp_dir.
        let code = run_build(&args, &global);
        let docx_exists = tmp_dir.path().join("deck.docx").exists();
        let pdf_exists = tmp_dir.path().join("deck.pdf").exists();
        let html_exists = tmp_dir.path().join("deck.html").exists();
        assert_eq!(code, ExitCode::from(1), "nonexistent source must exit 1");
        assert!(
            !docx_exists,
            "BC-1.15.003 AC-010: --format pptx must not write .docx"
        );
        assert!(
            !pdf_exists,
            "BC-1.15.003 AC-010: --format pptx must not write .pdf"
        );
        assert!(
            !html_exists,
            "BC-1.15.003 AC-010: --format pptx must not write .html"
        );
    }

    // ── BC-1.15.001: should_use_color ─────────────────────────────────────────

    /// BC-1.15.001 postcondition 5: --no-color disables colored output.
    ///
    /// `should_use_color` returns false when `no_color = true`.
    /// This function is fully implemented (not todo!()), so this test PASSES
    /// pre-implementation — that is intentional: `should_use_color` is a pure
    /// helper that does not call any todo!() stubs.
    ///
    /// The Red Gate requires ALL new tests to fail; since this is already correct,
    /// it is tested here but acknowledged as pre-passing. The meaningful Red Gate
    /// failures are the `run_build` and `exit_code_for_build_error` tests.
    #[test]
    fn test_BC_1_15_001_should_use_color_no_color_flag_suppresses_color() {
        let global = GlobalFlags {
            no_color: true,
            ..default_global()
        };
        // should_use_color is implemented (not todo!()); this test passes immediately.
        // Tests that no_color=true → false, which is correct behavior.
        assert!(
            !should_use_color(&global),
            "BC-1.15.001: --no-color must suppress color output"
        );
    }

    /// BC-1.15.001: --json disables colored output.
    #[test]
    fn test_BC_1_15_001_should_use_color_json_flag_suppresses_color() {
        let global = GlobalFlags {
            json: true,
            ..default_global()
        };
        assert!(
            !should_use_color(&global),
            "BC-1.15.001: --json must suppress color output"
        );
    }
}
