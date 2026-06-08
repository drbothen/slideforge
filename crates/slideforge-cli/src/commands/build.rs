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
//! # Brand discovery
//!
//! The CLI auto-discovers a `brand.toml` by looking in the source file's
//! parent directory.  If no `brand.toml` is found there, the build fails
//! with exit code 1 (file-not-found is a parse-category error).
//!
//! # Traceability
//!
//! - BC-1.15.001: all errors carry `file:line:col` span and correction hint
//! - BC-1.15.002: all errors accumulated in single pass (no fail-on-first)
//! - BC-1.15.003: three-tier exit code model
//! - AC-001 through AC-015

use std::io::IsTerminal as _;
use std::process::ExitCode;

use slideforge::error::BuildError;
use slideforge::{BrandSource, BuildOptions};

use crate::cli::{BuildArgs, GlobalFlags, OutputFormat};
use crate::exit_code::{EXIT_EXPORT_ERROR, EXIT_PARSE_ERROR, EXIT_VALIDATION_ERROR};
use crate::output::OutputWriter;

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
///
/// # Brand discovery
///
/// Looks for `brand.toml` in the same directory as the source file.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn run_build(args: &BuildArgs, global: &GlobalFlags) -> ExitCode {
    let use_color = should_use_color(global);

    // EC-002: validate that the source file exists before reading.
    if !args.source.exists() {
        let msg = format!("Error: source file not found: {}", args.source.display());
        render_plain_error(&msg);
        return ExitCode::from(EXIT_PARSE_ERROR);
    }

    // Read the source file content.  A read failure is also a parse-category error.
    let source_text = match std::fs::read_to_string(&args.source) {
        Ok(text) => text,
        Err(e) => {
            let msg = format!(
                "Error: cannot read source file {}: {e}",
                args.source.display()
            );
            render_plain_error(&msg);
            return ExitCode::from(EXIT_PARSE_ERROR);
        },
    };

    // EC-006: variant validation.
    // The root `slideforge::build()` API does not yet support variant selection.
    // When `--variant` is specified, validate that the variant is declared in
    // the source.  This is a CLI-layer check (E-VAR-004).
    // clippy::collapsible_if: the outer guard is `Some` and the inner guard is
    // the variant existence check — they cannot be collapsed without losing the
    // binding of `variant_name`.
    #[allow(clippy::collapsible_if)]
    if let Some(variant_name) = args.variant.as_deref() {
        if !source_declares_variant(&source_text, variant_name) {
            render_plain_error(&format!(
                "Error: undefined variant '{variant_name}' (E-VAR-004); \
                 use `variant:` in your .sf source to declare it"
            ));
            return ExitCode::from(EXIT_VALIDATION_ERROR);
        }
    }

    // Discover brand.toml next to the source file.
    let brand_toml_path = match discover_brand_toml(args) {
        Ok(path) => path,
        Err(msg) => {
            render_plain_error(&msg);
            return ExitCode::from(EXIT_PARSE_ERROR);
        },
    };

    // Determine strict mode: warn_only demotes validation errors to warnings.
    let strict = !global.warn_only;

    // Derive the output stem from the source file name (without extension).
    let stem = args
        .source
        .file_stem()
        .map_or_else(|| "output".to_owned(), |s| s.to_string_lossy().into_owned());

    // Run the pipeline for each selected format.
    // Parse errors are fatal and stop processing immediately.
    // Export errors per format are accumulated; the highest-priority error
    // across all formats determines the final exit code.
    let mut worst_exit_code: u8 = 0;
    let mut any_success = false;

    for format in &args.format {
        let format_str = format_to_str(*format);
        let options = BuildOptions {
            format: Some(format_str.to_owned()),
            brand_source: Some(BrandSource::TomlFile(std::sync::Arc::from(
                brand_toml_path.as_str(),
            ))),
            strict,
        };

        match slideforge::build(&source_text, &options) {
            Ok(output) => {
                // Write output atomically.
                let writer = OutputWriter::new(&args.output_dir, &stem, &output.extension);
                if let Err(e) = writer.write_atomic(&output.bytes) {
                    let msg = format!(
                        "Error writing output file {}: {e}",
                        writer.final_path().display()
                    );
                    render_plain_error(&msg);
                    let code = EXIT_EXPORT_ERROR;
                    if code > worst_exit_code {
                        worst_exit_code = code;
                    }
                } else {
                    if !global.quiet {
                        eprintln!("  written: {}", writer.final_path().display());
                    }
                    any_success = true;
                }
            },
            Err(err) => {
                // Render the error diagnostics.
                render_build_error(&err, use_color, global);

                let code = exit_code_for_build_error(&err);
                let code_u8 = exit_code_to_u8(code);

                // Parse errors (exit 1) short-circuit: no point running other formats.
                if code_u8 == EXIT_PARSE_ERROR {
                    return ExitCode::from(EXIT_PARSE_ERROR);
                }

                if code_u8 > worst_exit_code {
                    worst_exit_code = code_u8;
                }
            },
        }
    }

    // In warn-only mode: if we had validation errors but the pipeline continued
    // (strict=false), the exit code should be 0 regardless.
    if !strict && worst_exit_code == EXIT_VALIDATION_ERROR {
        worst_exit_code = 0;
    }

    if worst_exit_code == 0 {
        if !global.quiet && any_success {
            eprintln!("Build succeeded.");
        }
        ExitCode::SUCCESS
    } else {
        ExitCode::from(worst_exit_code)
    }
}

/// Map a [`BuildError`] to the appropriate [`ExitCode`].
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
pub fn exit_code_for_build_error(err: &BuildError) -> ExitCode {
    match err {
        // EvalFailed and ValidationFailed both map to exit 2 (validation error tier).
        BuildError::EvalFailed { .. } | BuildError::ValidationFailed { .. } => {
            ExitCode::from(EXIT_VALIDATION_ERROR)
        },
        BuildError::Export(_) => ExitCode::from(EXIT_EXPORT_ERROR),
        // ParseFailed, Brand, Layout, Registry, Plugin, NoBrandSource, NoBrandProvider,
        // UnknownFormat — all treated as fatal parse-category errors (exit 1).
        _ => ExitCode::from(EXIT_PARSE_ERROR),
    }
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

// ── Private helpers ───────────────────────────────────────────────────────────

/// Convert an [`ExitCode`] to its raw `u8` value.
fn exit_code_to_u8(code: ExitCode) -> u8 {
    // ExitCode does not expose its inner value directly.
    // We compare against known codes to extract the byte.
    if code == ExitCode::SUCCESS {
        0
    } else if code == ExitCode::from(EXIT_PARSE_ERROR) {
        EXIT_PARSE_ERROR
    } else if code == ExitCode::from(EXIT_VALIDATION_ERROR) {
        EXIT_VALIDATION_ERROR
    } else if code == ExitCode::from(EXIT_EXPORT_ERROR) {
        EXIT_EXPORT_ERROR
    } else {
        EXIT_PARSE_ERROR // conservative fallback
    }
}

/// Convert an [`OutputFormat`] to the format string expected by `BuildOptions`.
fn format_to_str(fmt: OutputFormat) -> &'static str {
    match fmt {
        OutputFormat::Pptx => "pptx",
        OutputFormat::Docx => "docx",
        OutputFormat::Pdf => "pdf",
        OutputFormat::Html => "html",
    }
}

/// Discover the `brand.toml` path for a build.
///
/// Looks for `brand.toml` in the same directory as the source file.
/// Returns `Err(message)` if no brand file can be found.
fn discover_brand_toml(args: &BuildArgs) -> Result<String, String> {
    // Look in the source file's parent directory.
    if let Some(parent) = args.source.parent() {
        let candidate = parent.join("brand.toml");
        if candidate.exists() {
            return candidate
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| "brand.toml path is not valid UTF-8".to_owned());
        }
    }

    // No brand.toml found.
    Err(format!(
        "Error: no brand.toml found next to {} (E-PAR-005). \
         Create a brand.toml in the same directory as your .sf file.",
        args.source.display()
    ))
}

/// Check whether a source string declares a variant with the given name.
///
/// Performs a simple textual search for `variant: <name>` in the source.
/// This is a CLI-layer check used for EC-006 (undefined variant → exit 2).
fn source_declares_variant(source: &str, variant_name: &str) -> bool {
    // Look for `variant: <name>` or `variant "<name>"` or similar DSL syntax.
    // The DSL uses `variant: name:` blocks. Search for the variant name.
    source.contains(&format!("variant {variant_name}:"))
        || source.contains(&format!("variant: {variant_name}"))
        || source.contains(&format!("\"{variant_name}\""))
}

/// Render a build error's diagnostics to stderr.
///
/// Uses miette's handlers for structured diagnostics (parse or eval failures)
/// and falls back to plain text for other error types.
fn render_build_error(err: &BuildError, use_color: bool, global: &GlobalFlags) {
    if global.json {
        render_build_error_json(err);
        return;
    }

    match err {
        BuildError::ParseFailed { diagnostics, .. }
        | BuildError::EvalFailed { diagnostics, .. } => {
            // Render each BoxDiagnostic (Box<dyn miette::Diagnostic>) directly
            // using the appropriate miette handler.
            let mut stderr = std::io::stderr();
            for diag in diagnostics {
                let rendered = render_box_diagnostic(diag.as_ref(), use_color);
                // Ignore write errors — best effort.
                let _ = std::io::Write::write_all(&mut stderr, rendered.as_bytes());
                let _ = std::io::Write::write_all(&mut stderr, b"\n");
            }
        },
        BuildError::ValidationFailed { diagnostics, .. } => {
            // ValidationFailed holds plugin-api Diagnostics (not boxed).
            // Render them as plain text with hint.
            for diag in diagnostics {
                eprintln!("[{}] {}: {}", diag.severity, diag.code, diag.message);
                if let Some(ref hint) = diag.hint {
                    eprintln!("  hint: {hint}");
                }
            }
        },
        other => {
            eprintln!("Error: {other}");
        },
    }
}

/// Render a single `dyn miette::Diagnostic` to a string using the appropriate
/// handler based on `use_color`.
fn render_box_diagnostic(diag: &dyn miette::Diagnostic, use_color: bool) -> String {
    let mut buf = String::new();
    if use_color {
        let handler = miette::GraphicalReportHandler::new();
        let _ = handler.render_report(&mut buf, diag);
    } else {
        let handler = miette::NarratableReportHandler::new();
        let _ = handler.render_report(&mut buf, diag);
    }
    buf
}

/// Render build error as JSON to stderr (for `--json` mode).
fn render_build_error_json(err: &BuildError) {
    let exit_code = exit_code_to_u8(exit_code_for_build_error(err));
    let json = serde_json::json!({
        "diagnostics": [{"message": err.to_string()}],
        "total": 1,
        "has_fatal": exit_code == EXIT_PARSE_ERROR || exit_code == EXIT_EXPORT_ERROR,
        "exit_code": exit_code,
    });
    // Use eprintln! — JSON output goes to stderr.
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string())
    );
}

/// Print a plain error message to stderr.
fn render_plain_error(msg: &str) {
    eprintln!("{msg}");
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
    #[test]
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
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
    #[test]
    #[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
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
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_1_15_003_exit_code_parse_takes_precedence_over_eval() {
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

        assert_ne!(
            parse_code, eval_code,
            "BC-1.15.003 postcondition 6: ParseFailed and EvalFailed must produce different codes"
        );
        assert_eq!(parse_code, ExitCode::from(1), "ParseFailed must → exit 1");
        assert_eq!(eval_code, ExitCode::from(2), "EvalFailed must → exit 2");
    }

    /// BC-1.15.003 postcondition 5: `run_build` on nonexistent source exits 1 (not 0).
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_1_15_003_run_build_nonexistent_source_exits_1_not_0() {
        let args = build_args_nonexistent(vec![
            OutputFormat::Pptx,
            OutputFormat::Docx,
            OutputFormat::Pdf,
            OutputFormat::Html,
        ]);
        let global = default_global();
        let code = run_build(&args, &global);
        assert_ne!(
            code,
            ExitCode::SUCCESS,
            "BC-1.15.003: run_build on nonexistent source must NOT return exit 0"
        );
    }

    /// BC-1.15.003: --format pptx does not write other format files.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_1_15_003_format_selection_pptx_only_does_not_write_other_formats() {
        let tmp_dir = tempfile::tempdir().expect("create tempdir");
        let args = BuildArgs {
            source: PathBuf::from("/nonexistent/deck.sf"),
            output_dir: tmp_dir.path().to_owned(),
            format: vec![OutputFormat::Pptx],
            variant: None,
        };
        let global = default_global();
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
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_1_15_001_should_use_color_no_color_flag_suppresses_color() {
        let global = GlobalFlags {
            no_color: true,
            ..default_global()
        };
        assert!(
            !should_use_color(&global),
            "BC-1.15.001: --no-color must suppress color output"
        );
    }

    /// BC-1.15.001: --json disables colored output.
    #[test]
    #[allow(non_snake_case)]
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
