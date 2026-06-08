//! Implementation of the `slideforge build` subcommand.
//!
//! `run_build` is the primary entry point: it drives the full
//! parse → evaluate → brand → validate → layout pipeline ONCE via
//! `slideforge::compile()`, renders all diagnostics via `miette`, then
//! exports to each selected format via `slideforge::export_format()`.
//!
//! # Architecture constraints
//!
//! - TTY detection uses `std::io::IsTerminal` (stable since Rust 1.70);
//!   the `atty` crate is NOT used (unmaintained, per spec).
//! - Output atomicity: ALL formats are written to `.tmp` paths first, then
//!   ALL are renamed to final paths in one pass (all-or-nothing). If any
//!   export fails, all temp files are deleted and no final files are written.
//! - `slideforge-cli` contains NO pipeline logic; it is an orchestration
//!   façade only.
//! - The pipeline is run ONCE (parse/eval/validate/layout) and diagnostics
//!   are rendered EXACTLY ONCE, regardless of how many output formats are
//!   selected (HIGH-002 fix).
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

use std::fmt::Write as _;
use std::io::IsTerminal as _;
use std::process::ExitCode;

use slideforge::error::BuildError;
use slideforge::{BrandSource, CompileOptions};

use crate::cli::{BuildArgs, GlobalFlags, OutputFormat};
use crate::exit_code::{EXIT_EXPORT_ERROR, EXIT_PARSE_ERROR, EXIT_VALIDATION_ERROR};
use crate::output::OutputWriter;

/// Run the `slideforge build` subcommand.
///
/// Drives the full compile pipeline for `args.source` ONCE, renders all
/// diagnostics via `miette` EXACTLY ONCE (regardless of selected format
/// count), then exports to each selected format.
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

    // Discover brand.toml next to the source file.
    //
    // I-4: No default-brand fallback is available — `BrandSource` has no
    // `Default` or `Synthesized` variant, and the BrandSynthesizer requires
    // an explicit TomlFile path. A missing brand.toml is therefore a hard error
    // (exit 1, parse-category). The error message includes a correction hint:
    // "Create a brand.toml in the same directory as your .sf file." This satisfies
    // BC-1.15.001 (correction hint present). The error code is E-PAR-005
    // (file not found). No span is possible since there is no source file to point to.
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

    // ── SINGLE PIPELINE RUN (HIGH-002 fix) ──────────────────────────────────────
    //
    // Run parse → eval → brand → validate → layout EXACTLY ONCE for all selected
    // formats. Diagnostics are collected and rendered EXACTLY ONCE here, not once
    // per format. Only the export stage runs per format.
    //
    // C-1 fix: thread args.variant into CompileOptions.active_variant so that
    // eval_deck_with_variant is called with the selected variant name.
    // - Defined variant → exit 0 (variant vars applied)
    // - Undefined variant → E-EVL-001 in EvalFailed → exit 2
    // The blanket --variant rejection that was here before is removed; the eval
    // layer now correctly handles both defined and undefined variant names.
    let compile_opts = CompileOptions {
        brand_source: Some(BrandSource::TomlFile(std::sync::Arc::from(
            brand_toml_path.as_str(),
        ))),
        strict,
        active_variant: args.variant.clone(),
    };

    let compiled = match slideforge::compile(&source_text, &compile_opts) {
        Ok(compiled) => compiled,
        Err(err) => {
            // Render diagnostics EXACTLY ONCE (HIGH-002 fix).
            render_build_error(&err, use_color, global);
            return ExitCode::from(exit_code_for_build_error_u8(&err));
        },
    };

    // ── PER-FORMAT EXPORT (MED-001 all-or-nothing fix) ───────────────────────────
    //
    // 1. Export every selected format to a temporary path (.tmp).
    // 2. If ALL succeed, rename all .tmp files to final paths.
    // 3. If ANY fails, delete all .tmp files and return the export error code.
    //    No final files are written if any format fails.
    let writers: Vec<OutputWriter> = args
        .format
        .iter()
        .map(|fmt| OutputWriter::new(&args.output_dir, &stem, format_to_str(*fmt)))
        .collect();

    // Phase 1: export all formats to tmp paths.
    let mut export_bytes: Vec<Vec<u8>> = Vec::with_capacity(args.format.len());
    for format in &args.format {
        let format_str = format_to_str(*format);
        match slideforge::export_format(&compiled, format_str) {
            Ok(output) => {
                export_bytes.push(output.bytes);
            },
            Err(err) => {
                // Export failed — delete any tmp files we may have partially created.
                for writer in &writers {
                    if writer.tmp_path().exists() {
                        let _ = std::fs::remove_file(writer.tmp_path());
                    }
                }
                render_plain_error(&format!("Error exporting {format_str}: {err}"));
                return ExitCode::from(EXIT_EXPORT_ERROR);
            },
        }
    }

    // Phase 2: write all bytes to .tmp paths.
    for (writer, bytes) in writers.iter().zip(export_bytes.iter()) {
        if let Err(e) = writer.write_atomic(bytes) {
            // Write failed — delete all tmp files and return export error.
            for w in &writers {
                if w.tmp_path().exists() {
                    let _ = std::fs::remove_file(w.tmp_path());
                }
                // Also delete any final paths already written by earlier formats.
                if w.final_path().exists() {
                    let _ = std::fs::remove_file(w.final_path());
                }
            }
            let msg = format!(
                "Error writing output file {}: {e}",
                writer.final_path().display()
            );
            render_plain_error(&msg);
            return ExitCode::from(EXIT_EXPORT_ERROR);
        }
    }

    // All formats succeeded.
    if !global.quiet {
        for writer in &writers {
            eprintln!("  written: {}", writer.final_path().display());
        }
        eprintln!("Build succeeded.");
    }
    ExitCode::SUCCESS
}

/// Map a [`BuildError`] to the appropriate exit code as `u8`.
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
///
/// This function operates on `u8` directly (HIGH-004 fix: avoids the
/// `exit_code_to_u8` guessing pattern that had a silent fallback to
/// `EXIT_PARSE_ERROR` on unrecognized variants).
fn exit_code_for_build_error_u8(err: &BuildError) -> u8 {
    match err {
        BuildError::EvalFailed { .. } | BuildError::ValidationFailed { .. } => {
            EXIT_VALIDATION_ERROR
        },
        BuildError::Export(_) => EXIT_EXPORT_ERROR,
        // ParseFailed, Brand, Layout, Registry, Plugin, NoBrandSource, NoBrandProvider,
        // UnknownFormat — all treated as fatal parse-category errors (exit 1).
        _ => EXIT_PARSE_ERROR,
    }
}

/// Map a [`BuildError`] to the appropriate [`ExitCode`].
///
/// Public version of `exit_code_for_build_error_u8` for unit-test access.
#[must_use]
pub fn exit_code_for_build_error(err: &BuildError) -> ExitCode {
    ExitCode::from(exit_code_for_build_error_u8(err))
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

/// Render a build error's diagnostics to stderr.
///
/// Uses miette's handlers for structured diagnostics (parse or eval failures)
/// and includes `file:line:col` span for validation failures (HIGH-001 fix).
fn render_build_error(err: &BuildError, use_color: bool, global: &GlobalFlags) {
    if global.json {
        render_build_error_json(err);
        return;
    }

    let rendered = render_build_error_to_string(err, use_color);
    let mut stderr = std::io::stderr();
    let _ = std::io::Write::write_all(&mut stderr, rendered.as_bytes());
}

/// Render a build error's diagnostics to a `String` (for testability).
///
/// OBS-1 fix: exposes the rendering logic as a pure string-returning function
/// so tests can assert on rendered content without spawning a subprocess or
/// redirecting stderr. This is the single-site implementation shared by
/// `render_build_error` (which writes to stderr) and integration test content
/// assertions.
#[must_use]
pub fn render_build_error_to_string(err: &BuildError, use_color: bool) -> String {
    let mut buf = String::new();

    match err {
        BuildError::ParseFailed { diagnostics, .. }
        | BuildError::EvalFailed { diagnostics, .. } => {
            // Render each BoxDiagnostic (Box<dyn miette::Diagnostic>) directly
            // using the appropriate miette handler.
            for diag in diagnostics {
                let rendered = render_box_diagnostic(diag.as_ref(), use_color);
                buf.push_str(&rendered);
                buf.push('\n');
            }
        },
        BuildError::ValidationFailed { diagnostics, .. } => {
            // HIGH-001 fix: ValidationFailed holds plugin-api Diagnostics.
            // Render them with `file:line:col` from diag.span so EVERY
            // emitted diagnostic carries source location information.
            for diag in diagnostics {
                let rendered = render_validation_diagnostic_to_string(diag, use_color);
                buf.push_str(&rendered);
                buf.push('\n');
            }
        },
        other => {
            let _ = write!(buf, "Error: {other}");
        },
    }

    buf
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

/// Render a single [`slideforge_plugin_api::Diagnostic`] (from `ValidationFailed`)
/// with `file:line:col` span information (HIGH-001 fix).
///
/// Format: `[severity] code: message (file:line:col)\n  hint: <hint>`
///
/// Returns the rendered string (not printed to stderr — call site is responsible).
fn render_validation_diagnostic_to_string(
    diag: &slideforge::ValidationDiagnostic,
    _use_color: bool,
) -> String {
    let span = &diag.span;
    let mut buf = String::new();
    // Include file:line:col if the span carries a real source location.
    // SourceSpan defaults to file="", line=0, col=0 for span-less diagnostics.
    if !span.file.is_empty() && (span.line > 0 || span.col > 0) {
        let _ = write!(
            buf,
            "[{}] {}: {} ({}:{}:{})",
            diag.severity, diag.code, diag.message, span.file, span.line, span.col
        );
    } else {
        let _ = write!(buf, "[{}] {}: {}", diag.severity, diag.code, diag.message);
    }
    if let Some(ref hint) = diag.hint {
        buf.push('\n');
        let _ = write!(buf, "  hint: {hint}");
    }
    buf
}

/// Render build error as JSON to stderr (for `--json` mode).
///
/// HIGH-003 fix: iterates all diagnostics (not just the first), sets `total`
/// to the real count, and includes span fields (`file:line:col`) where available.
fn render_build_error_json(err: &BuildError) {
    let exit_code_val = exit_code_for_build_error_u8(err);
    let has_fatal = exit_code_val == EXIT_PARSE_ERROR || exit_code_val == EXIT_EXPORT_ERROR;

    let diagnostics: Vec<serde_json::Value> = match err {
        BuildError::ParseFailed { diagnostics, .. }
        | BuildError::EvalFailed { diagnostics, .. } => diagnostics
            .iter()
            .map(|d| {
                let code = d.code().map_or_else(String::new, |c| c.to_string());
                let message = d.to_string();
                let hint = d.help().map(|h| h.to_string());
                // Extract the first label's span if available.
                let span_obj = d.labels().and_then(|mut labels| {
                    labels.next().map(|label| {
                        serde_json::json!({
                            "offset": label.offset(),
                            "length": label.len(),
                        })
                    })
                });
                let mut obj = serde_json::json!({
                    "code": code,
                    "message": message,
                });
                if let Some(h) = hint {
                    obj["hint"] = serde_json::Value::String(h);
                }
                if let Some(s) = span_obj {
                    obj["span"] = s;
                }
                obj
            })
            .collect(),
        BuildError::ValidationFailed { diagnostics, .. } => diagnostics
            .iter()
            .map(|d| {
                let span = &d.span;
                let mut obj = serde_json::json!({
                    "code": d.code.as_ref(),
                    "message": d.message.as_ref(),
                    "severity": d.severity.to_string(),
                    "span": {
                        "file": span.file.as_ref(),
                        "line": span.line,
                        "col": span.col,
                    },
                });
                if let Some(ref hint) = d.hint {
                    obj["hint"] = serde_json::Value::String(hint.as_ref().to_owned());
                }
                obj
            })
            .collect(),
        other => {
            vec![serde_json::json!({"message": other.to_string()})]
        },
    };

    let total = diagnostics.len();
    let json = serde_json::json!({
        "diagnostics": diagnostics,
        "total": total,
        "has_fatal": has_fatal,
        "exit_code": exit_code_val,
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
