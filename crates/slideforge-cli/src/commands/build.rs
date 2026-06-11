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
    //
    // F-094-P4-006 fix: thread the real source file path as source_name so that
    // all span-carrying diagnostics (parse, eval, layout) cite the actual filename
    // (e.g., "quarterly-review.sf") instead of the library fallback "<source>".
    // to_string_lossy() is correct here: PathBuf paths are OS-native and may
    // contain non-UTF-8 components on some platforms; lossy conversion is
    // acceptable because source_name is used only for human-readable diagnostic output.
    let source_name = std::sync::Arc::from(args.source.to_string_lossy().as_ref());
    let compile_opts = CompileOptions {
        brand_source: Some(BrandSource::TomlFile(std::sync::Arc::from(
            brand_toml_path.as_str(),
        ))),
        strict,
        active_variant: args.variant.clone(),
        source_name: Some(source_name),
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
/// | `MultistageFailed` (strict) | 2 |
/// | `Export` | 3 |
/// | Other (`Registry`, `Brand`, `Layout`, `Plugin`, `UnknownFormat`) | 1 |
///
/// This function operates on `u8` directly (HIGH-004 fix: avoids the
/// `exit_code_to_u8` guessing pattern that had a silent fallback to
/// `EXIT_PARSE_ERROR` on unrecognized variants).
fn exit_code_for_build_error_u8(err: &BuildError) -> u8 {
    match err {
        BuildError::EvalFailed { .. }
        | BuildError::ValidationFailed { .. }
        | BuildError::MultistageFailed { .. } => EXIT_VALIDATION_ERROR,
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
///
/// BC-1.15.002 PC2 / HIGH-P3-001 fix: `ValidationFailed` diagnostics are
/// rendered in source-file order (already guaranteed by the sort in
/// `compile_inner`). `MultistageFailed` diagnostics are merge-sorted at render
/// time using `slideforge::box_diag_sort_key` (for eval entries) and
/// `slideforge::validator_diag_sort_key` (for validator entries).
///
/// BC-1.15.002 invariant 1 / EC-004: exact-duplicate diagnostics are removed
/// at accumulation time (in `compile_inner`); the render path trusts the
/// deduped list it receives.
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
            //
            // BC-1.15.002 PC2 / MED-P3-002: sort by (file, line, col) at render
            // time so Stage-5 and Stage-6b diagnostics are interleaved in source
            // order regardless of accumulation order.
            //
            // BC-1.15.002 invariant 1 / EC-004: dedup exact duplicates at render
            // time (same sort + dedup logic as accumulation in compile_inner, but
            // also applied here so direct ValidationFailed construction in tests
            // and in the layout-early-return path is covered).
            for diag in sort_and_dedup_validator_diags_for_render(diagnostics) {
                let rendered = render_validation_diagnostic_to_string(diag, use_color);
                buf.push_str(&rendered);
                buf.push('\n');
            }
        },
        BuildError::MultistageFailed {
            eval_diagnostics,
            eval_sort_keys,
            eval_severities,
            validator_diagnostics,
            ..
        } => {
            // HIGH-P3-001 fix: interleave eval + validator diagnostics in source-file
            // order (BC-1.15.002 PC2) using a merge-sort at render time.
            //
            // Both eval_diagnostics and validator_diagnostics are individually sorted
            // (by compile_inner). We merge them here using their sort keys.
            // eval_sort_keys is parallel to eval_diagnostics (same index).
            //
            // This is the SINGLE sort site for MultistageFailed rendering — shared
            // between text and JSON paths via the `interleave_multistage_diagnostics`
            // helper (TD-VSDD-060: no third drift site).
            let interleaved = interleave_multistage_diagnostics(
                eval_diagnostics,
                eval_sort_keys,
                eval_severities,
                validator_diagnostics,
            );
            for item in &interleaved {
                let rendered = match item {
                    MultiDiag::Eval { diag, .. } => {
                        let mut s = render_box_diagnostic(diag.as_ref(), use_color);
                        s.push('\n');
                        s
                    },
                    MultiDiag::Validator(d) => {
                        let mut s = render_validation_diagnostic_to_string(d, use_color);
                        s.push('\n');
                        s
                    },
                };
                buf.push_str(&rendered);
            }
        },
        BuildError::Layout(layout_err) => {
            // F-094-P4-004 / error-taxonomy v2.30 §234 / DI-018:
            // Layout errors render with their structured message (which embeds the
            // error code prefix and source span from the thiserror Display).
            //
            // When `layout_err` is `LayoutError::Multiple { inner }` (the accumulation
            // wrapper for E-LAY-008 cross-slide accumulation), render EACH inner error
            // on its own line so all instances are visible to the user in one build pass.
            //
            // For all other (non-Multiple) LayoutError variants, render the error directly.
            match layout_err {
                slideforge::LayoutError::Multiple { inner } => {
                    for (i, err) in inner.iter().enumerate() {
                        if i > 0 {
                            buf.push('\n');
                        }
                        let _ = write!(buf, "{err}");
                    }
                },
                other => {
                    let _ = write!(buf, "{other}");
                },
            }
        },
        other => {
            let _ = write!(buf, "Error: {other}");
        },
    }

    buf
}

/// Sort and dedup a slice of `ValidationDiagnostic` for rendering.
///
/// Returns an iterator over references to the sorted, deduped diagnostics.
///
/// Sorting is by `(span.file, span.line, span.col)` ascending
/// (BC-1.15.002 PC2 / MED-P3-002).
///
/// Deduplication removes exact duplicates with the same `(code, span.file,
/// span.line, span.col, message)` — the first occurrence is kept
/// (BC-1.15.002 invariant 1 / EC-004).
///
/// This helper is used by both text and JSON render paths for `ValidationFailed`
/// and is the SINGLE canonical site for render-time sort+dedup of validator
/// diagnostics (TD-VSDD-060: no third drift site).
fn sort_and_dedup_validator_diags_for_render(
    diagnostics: &[slideforge::ValidationDiagnostic],
) -> Vec<&slideforge::ValidationDiagnostic> {
    use std::collections::HashSet;

    // Build sorted indices by sort key.
    let mut indices: Vec<usize> = (0..diagnostics.len()).collect();
    indices.sort_by(|&a, &b| {
        let ka = slideforge::validator_diag_sort_key(&diagnostics[a]);
        let kb = slideforge::validator_diag_sort_key(&diagnostics[b]);
        ka.cmp(&kb)
    });

    // Iterate in sorted order, deduplicating.
    let mut seen: HashSet<(String, String, u32, u32, String)> = HashSet::new();
    indices
        .into_iter()
        .filter_map(|i| {
            let d = &diagnostics[i];
            let key = (
                d.code.to_string(),
                d.span.file.to_string(),
                d.span.line,
                d.span.col,
                d.message.to_string(),
            );
            if seen.insert(key) { Some(d) } else { None }
        })
        .collect()
}

/// Enum used by [`interleave_multistage_diagnostics`] to represent a single
/// diagnostic from either the eval or validator stage.
///
/// Borrowing refs to avoid cloning the entire `BoxDiagnostic` vector.
enum MultiDiag<'a> {
    /// A diagnostic from the eval stage (a `BoxDiagnostic` / `OwnedDiag`).
    ///
    /// `severity` is the ACTUAL [`slideforge::ParseSeverity`] captured at push
    /// time — used by the JSON renderer to emit the real `"severity"` field
    /// instead of hardcoding `"error"` (OBS-P4-003 fix).
    Eval {
        /// The type-erased diagnostic.
        diag: &'a slideforge::BoxDiagnostic,
        /// The actual severity of this eval diagnostic.
        severity: slideforge::ParseSeverity,
    },
    /// A diagnostic from the validator stage (`slideforge_plugin_api::Diagnostic`).
    Validator(&'a slideforge::ValidationDiagnostic),
}

/// Merge-sort eval and validator diagnostics by source position.
///
/// Both input slices must already be individually sorted by `(file, line, col)`.
/// This function performs a stable full sort (not a two-pointer merge) so that
/// tests that construct `MultistageFailed` directly (without going through
/// `compile_inner`) also produce correct output regardless of input order.
///
/// `eval_sort_keys` is the parallel sort-key vec from
/// `BuildError::MultistageFailed.eval_sort_keys` — index `i` of `eval_sort_keys`
/// is the `(file, line, col)` key for `eval_diags[i]`.
///
/// `eval_severities` is the parallel severity vec from
/// `BuildError::MultistageFailed.eval_severities` — index `i` is the
/// [`slideforge::ParseSeverity`] for `eval_diags[i]`. Used to populate
/// `MultiDiag::Eval { severity }` so the JSON renderer emits the actual
/// `"severity"` field (OBS-P4-003 fix).
///
/// The validator sort key is derived from `slideforge::validator_diag_sort_key(d)`.
/// Validator entries are deduped on `(code, file, line, col, message)` using the
/// shared `sort_and_dedup_validator_diags_for_render` helper (OBS-P4-001 fix /
/// TD-VSDD-060: single dedup helper, no drift site).
///
/// This is the SINGLE canonical merge helper for `MultistageFailed` rendering.
/// It is used by both `render_build_error_to_string` (text) and
/// `render_build_error_to_json_value` (JSON) to avoid a third drift site
/// (TD-VSDD-060).
///
/// BC-1.15.002 PC2 / HIGH-P3-001.
fn interleave_multistage_diagnostics<'a>(
    eval_diags: &'a [slideforge::BoxDiagnostic],
    eval_sort_keys: &'a [(String, u32, u32)],
    eval_severities: &'a [slideforge::ParseSeverity],
    validator_diags: &'a [slideforge::ValidationDiagnostic],
) -> Vec<MultiDiag<'a>> {
    // BC-1.15.002 PC2 / HIGH-P3-001: merge eval and validator diagnostics in
    // source-file order.
    //
    // Build a combined list of (sort_key, MultiDiag) pairs, then stable-sort
    // by (file, line, col). We do NOT assume the inputs are pre-sorted — a full
    // sort is performed so that tests constructing MultistageFailed directly
    // (without going through compile_inner) also produce correct output.

    let mut combined: Vec<((String, u32, u32), MultiDiag<'a>)> =
        Vec::with_capacity(eval_diags.len() + validator_diags.len());

    for (i, d) in eval_diags.iter().enumerate() {
        let key = eval_sort_keys
            .get(i)
            .cloned()
            .unwrap_or_else(|| ("<unknown>".to_owned(), 0u32, 0u32));
        // OBS-P4-003: thread actual severity rather than hardcoding Error.
        let severity = eval_severities
            .get(i)
            .copied()
            .unwrap_or(slideforge::ParseSeverity::Error);
        combined.push((key, MultiDiag::Eval { diag: d, severity }));
    }

    // OBS-P4-001: dedup validator entries on (code, file, line, col, message)
    // using the shared helper (TD-VSDD-060: single source, no drift).
    // This makes the render path robust regardless of whether compile_inner's
    // pre-dedup ran (guards against direct MultistageFailed construction in tests
    // or future callers that bypass the pipeline).
    let deduped_validators = sort_and_dedup_validator_diags_for_render(validator_diags);
    for d in deduped_validators {
        let key = slideforge::validator_diag_sort_key(d);
        combined.push((key, MultiDiag::Validator(d)));
    }

    // Stable sort by (file, line, col): preserves insertion order for ties.
    combined.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));

    combined.into_iter().map(|(_, d)| d).collect()
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

/// Serialize a single flat (non-`Multiple`) [`slideforge::LayoutError`] to a JSON
/// diagnostic object matching the schema used by `ValidationFailed` entries:
///
/// ```json
/// { "code": "E-LAY-008", "message": "…", "severity": "error", "span": { "file": "…", "line": N, "col": N } }
/// ```
///
/// `span` is included when the variant carries a [`slideforge_types::SourceSpan`].
/// `severity` is always `"error"` — layout errors are fatal by definition.
///
/// ## Code extraction
///
/// Layout error codes are embedded in the `thiserror` `#[error("...")]` strings as
/// `[E-LAY-NNN]` prefixes (e.g. `BulletsOnContentlessSlideType` embeds `[E-LAY-008]`).
/// For variants that carry a code prefix the code is extracted from the Display string;
/// for variants without a code prefix the code field is left empty (`""`).
///
/// ## Non-exhaustive guard
///
/// `LayoutError` is `#[non_exhaustive]`. All unrecognised variants fall through to the
/// `_ =>` arm, which emits an empty code + the Display string as the message.
///
/// ## Traceability
///
/// - F-094-P7-001: JSON render path for accumulated E-LAY-008 instances
/// - error-taxonomy v2.30 §E-LAY-008 accumulation on the machine-consumable surface
fn layout_error_to_json_diag(err: &slideforge::LayoutError) -> serde_json::Value {
    use slideforge::LayoutError;

    // Extract (code, span) by matching known variants that carry structured fields.
    // All layout errors are severity "error".
    //
    // `code` is `String` throughout to avoid lifetime issues in the fall-through
    // arm where the code is extracted from a temporary `to_string()` allocation.
    //
    // Future span-carrying variants: add `if let` branches here to avoid falling to the
    // display-parse path. For now, all other variants lack a machine-readable code
    // (they have no [E-LAY-NNN] prefix in their error strings), so we extract
    // whatever prefix is present from the Display string.
    let (code, span_opt) = if let LayoutError::BulletsOnContentlessSlideType { span, .. } = err {
        ("E-LAY-008".to_owned(), Some(span.clone()))
    } else {
        // Attempt to parse `[E-LAY-NNN]` from the Display string.
        // The extracted slice is turned into an owned String before `msg` is dropped.
        let msg = err.to_string();
        let code = if msg.starts_with('[') {
            msg.find(']')
                .and_then(|end| msg.get(1..end))
                .unwrap_or("")
                .to_owned()
        } else {
            String::new()
        };
        // Cannot recover a structured span from unknown variants.
        (code, None)
    };

    let message = err.to_string();
    let mut obj = serde_json::json!({
        "code": code,
        "message": message,
        "severity": "error",
    });
    if let Some(span) = span_opt {
        obj["span"] = serde_json::json!({
            "file": span.file.as_ref(),
            "line": span.line,
            "col": span.col,
        });
    }
    obj
}

/// Render a [`BuildError`] to a [`serde_json::Value`] (pure, side-effect-free).
///
/// OBS-P4-004 fix: extracted from `render_build_error_json` so that the JSON
/// rendering logic is testable without spawning a subprocess or capturing stderr.
/// `render_build_error_json` is the sole call site that writes the result to stderr.
///
/// ## Schema
///
/// ```json
/// {
///   "diagnostics": [ { "code": "…", "message": "…", "severity": "…", … }, … ],
///   "total": N,
///   "has_fatal": true,
///   "exit_code": 1
/// }
/// ```
///
/// Each diagnostic entry carries at minimum `"code"`, `"message"`, and
/// `"severity"`. Validator and `MultistageFailed` entries also carry a `"span"`
/// object with `"file"`, `"line"`, and `"col"` fields.
///
/// ## Severity fidelity (OBS-P4-003 fix)
///
/// `MultistageFailed` eval entries emit the ACTUAL severity from `eval_severities`
/// (e.g. `"warning"` for a warning-severity eval diagnostic) rather than
/// hardcoding `"error"` for all eval entries.
///
/// ## Dedup and order (OBS-P4-001 / OBS-P4-002)
///
/// Validator entries are deduped inside `interleave_multistage_diagnostics`.
/// Eval entries were deduped at accumulation time in `compile_inner`.
/// `MultistageFailed` entries are emitted in ascending `(file, line, col)` order
/// (via the shared `interleave_multistage_diagnostics` helper).
///
/// ## Traceability
///
/// - OBS-P4-003: eval severity fidelity
/// - OBS-P4-004: testability via pure function
/// - BC-1.15.002 PC2 / HIGH-P3-001: source-order interleaving
/// - TD-VSDD-060: single sort+dedup site
#[must_use]
pub fn render_build_error_to_json_value(err: &BuildError) -> serde_json::Value {
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
        BuildError::ValidationFailed { diagnostics, .. } => {
            // BC-1.15.002 PC2 / EC-004: sort+dedup at render time (same as text path).
            sort_and_dedup_validator_diags_for_render(diagnostics)
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
                .collect()
        },
        BuildError::MultistageFailed {
            eval_diagnostics,
            eval_sort_keys,
            eval_severities,
            validator_diagnostics,
            ..
        } => {
            // HIGH-P3-001: interleave eval + validator in source order using the
            // single canonical merge helper (same as the text renderer — TD-VSDD-060).
            // OBS-P4-001: validator dedup applied inside the helper.
            // OBS-P4-003: eval_severities threaded for actual severity per entry.
            let interleaved = interleave_multistage_diagnostics(
                eval_diagnostics,
                eval_sort_keys,
                eval_severities,
                validator_diagnostics,
            );
            interleaved
                .iter()
                .map(|item| match item {
                    MultiDiag::Eval { diag: d, severity } => {
                        let code = d.code().map_or_else(String::new, |c| c.to_string());
                        let message = d.to_string();
                        let hint = d.help().map(|h| h.to_string());
                        let span_obj = d.labels().and_then(|mut labels| {
                            labels.next().map(|label| {
                                serde_json::json!({
                                    "offset": label.offset(),
                                    "length": label.len(),
                                })
                            })
                        });
                        // OBS-P4-003: emit ACTUAL severity instead of hardcoding "error".
                        let severity_str = parse_severity_to_str(*severity);
                        let mut obj = serde_json::json!({
                            "code": code,
                            "message": message,
                            "severity": severity_str,
                        });
                        if let Some(h) = hint {
                            obj["hint"] = serde_json::Value::String(h);
                        }
                        if let Some(s) = span_obj {
                            obj["span"] = s;
                        }
                        obj
                    },
                    MultiDiag::Validator(d) => {
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
                    },
                })
                .collect()
        },
        // F-094-P7-001 / error-taxonomy v2.30 §E-LAY-008: Layout errors MUST appear
        // on the machine-consumable JSON surface with full code + span, not collapsed
        // into the catch-all Display string.
        //
        // `Multiple { inner }` is the E-LAY-008 accumulation wrapper: each inner error
        // becomes one JSON diagnostic object (same count as the text path).
        //
        // A bare (non-Multiple) LayoutError maps to one object.
        BuildError::Layout(layout_err) => match layout_err {
            slideforge::LayoutError::Multiple { inner } => {
                inner.iter().map(layout_error_to_json_diag).collect()
            },
            single => vec![layout_error_to_json_diag(single)],
        },
        other => {
            vec![serde_json::json!({"message": other.to_string()})]
        },
    };

    let total = diagnostics.len();
    serde_json::json!({
        "diagnostics": diagnostics,
        "total": total,
        "has_fatal": has_fatal,
        "exit_code": exit_code_val,
    })
}

/// Convert a [`slideforge::ParseSeverity`] to the lowercase string used in JSON output.
///
/// `"fatal"` is treated the same as `"error"` in the JSON output because
/// `MultistageFailed` carries only non-fatal eval diagnostics (the fatal path
/// takes a different code path through `EvalFailed`). A defensive fallback to
/// `"error"` is appropriate for all Error/Fatal variants.
fn parse_severity_to_str(sev: slideforge::ParseSeverity) -> &'static str {
    match sev {
        slideforge::ParseSeverity::Warning => "warning",
        slideforge::ParseSeverity::Error | slideforge::ParseSeverity::Fatal => "error",
    }
}

/// Render build error as JSON to stderr (for `--json` mode).
///
/// HIGH-003 fix: iterates all diagnostics (not just the first), sets `total`
/// to the real count, and includes span fields (`file:line:col`) where available.
///
/// OBS-P4-004 fix: this function is now a thin wrapper around the pure
/// `render_build_error_to_json_value` function so that the JSON content is
/// directly testable without capturing stderr.
fn render_build_error_json(err: &BuildError) {
    let json = render_build_error_to_json_value(err);
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

    use super::{
        exit_code_for_build_error, render_build_error_to_json_value, run_build, should_use_color,
    };
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
    ///
    /// `run_build` exits at the source-exists check (exit 1) before ever
    /// touching the output directory, so the output dir path is never accessed.
    /// Both paths are intentionally non-existent on every platform: the source
    /// file is guaranteed absent, and `std::env::temp_dir()` resolves to a
    /// platform-appropriate temp root (`/tmp` on Unix, `%TEMP%` on Windows).
    fn build_args_nonexistent(formats: Vec<OutputFormat>) -> BuildArgs {
        BuildArgs {
            source: PathBuf::from("/nonexistent/deck.sf"),
            output_dir: std::env::temp_dir().join("slideforge-test-out-nonexistent"),
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

    // ── OBS-P4-004: render_build_error_to_json_value tests ───────────────────
    //
    // Four load-bearing tests for the pure JSON render function extracted by
    // OBS-P4-004. These tests assert CONTENT+ORDER (not just "is JSON"), per
    // LESSON-14.

    /// OBS-P4-004 (a): `MultistageFailed` JSON entries are emitted in ascending
    /// `(file, line, col)` order — validator error interleaved between two eval errors.
    ///
    /// Fixture: eval at line 3, validator at line 5, eval at line 7.
    /// Input order is deliberately wrong (line 7 before line 3) to prove the
    /// interleave sorts correctly.
    #[test]
    #[allow(non_snake_case)]
    fn test_OBS_P4_004_a_multistage_json_entries_ascending_order() {
        use slideforge::ParseSeverity;
        use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity};
        use slideforge_types::SourceSpan;
        use std::sync::Arc;

        let eval_diag_line7 = slideforge::make_test_owned_diag("E-EVL-001", "eval error at line 7");
        let eval_diag_line3 = slideforge::make_test_owned_diag("E-EVL-001", "eval error at line 3");

        let err = BuildError::MultistageFailed {
            eval_diagnostics: vec![eval_diag_line7, eval_diag_line3],
            eval_sort_keys: vec![("deck.sf".to_owned(), 7, 1), ("deck.sf".to_owned(), 3, 1)],
            eval_severities: vec![ParseSeverity::Error, ParseSeverity::Error],
            validator_diagnostics: vec![Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: Arc::from("E-VAL-001"),
                message: Arc::from("validator error at line 5"),
                span: SourceSpan {
                    file: Arc::from("deck.sf"),
                    line: 5,
                    col: 1,
                    ..SourceSpan::default()
                },
                hint: None,
            }],
            eval_count: 2,
            validator_count: 1,
        };

        let json = render_build_error_to_json_value(&err);
        let arr = json["diagnostics"]
            .as_array()
            .expect("diagnostics must be array");
        assert_eq!(arr.len(), 3, "must have 3 entries (2 eval + 1 validator)");

        // Entries must be in source order: line 3, line 5, line 7.
        // Eval entries use OwnedDiag which doesn't carry span info in the span field,
        // but the messages contain the line numbers.
        let msg0 = arr[0]["message"].as_str().unwrap_or("");
        let msg1 = arr[1]["message"].as_str().unwrap_or("");
        let msg2 = arr[2]["message"].as_str().unwrap_or("");

        assert!(
            msg0.contains("line 3"),
            "OBS-P4-004a: first entry must be eval error at line 3; got: {msg0}"
        );
        assert!(
            msg1.contains("line 5"),
            "OBS-P4-004a: second entry must be validator error at line 5; got: {msg1}"
        );
        assert!(
            msg2.contains("line 7"),
            "OBS-P4-004a: third entry must be eval error at line 7; got: {msg2}"
        );
    }

    /// OBS-P4-004 (b): eval warning entries carry `"severity":"warning"` in JSON.
    ///
    /// This is the core OBS-P4-003 regression test: a Warning-severity eval
    /// diagnostic must emit `"severity":"warning"`, NOT `"severity":"error"`.
    #[test]
    #[allow(non_snake_case)]
    fn test_OBS_P4_004_b_eval_warning_carries_warning_severity_in_json() {
        use slideforge::ParseSeverity;
        use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity};
        use slideforge_types::SourceSpan;
        use std::sync::Arc;

        // One eval Warning + one validator Error — MultistageFailed requires both.
        // (In practice, eval_count tracks only Error+Fatal, but we test the JSON
        // severity field regardless of the count gating.)
        let eval_warning = slideforge::make_test_owned_diag("E-EVL-099", "eval warning diagnostic");

        let err = BuildError::MultistageFailed {
            eval_diagnostics: vec![eval_warning],
            eval_sort_keys: vec![("deck.sf".to_owned(), 2, 1)],
            eval_severities: vec![ParseSeverity::Warning],
            validator_diagnostics: vec![Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: Arc::from("E-VAL-001"),
                message: Arc::from("validator error"),
                span: SourceSpan {
                    file: Arc::from("deck.sf"),
                    line: 10,
                    col: 1,
                    ..SourceSpan::default()
                },
                hint: None,
            }],
            eval_count: 0, // warning doesn't count as error
            validator_count: 1,
        };

        let json = render_build_error_to_json_value(&err);
        let arr = json["diagnostics"]
            .as_array()
            .expect("diagnostics must be array");
        assert_eq!(arr.len(), 2, "must have 2 entries");

        // First entry is the eval warning (line 2 < line 10).
        let eval_sev = arr[0]["severity"].as_str().unwrap_or("");
        assert_eq!(
            eval_sev, "warning",
            "OBS-P4-003 / OBS-P4-004b: eval warning entry must carry severity='warning', \
             not 'error'; got: {eval_sev}"
        );

        // Second entry is the validator error.
        let val_sev = arr[1]["severity"].as_str().unwrap_or("");
        assert_eq!(
            val_sev, "error",
            "OBS-P4-004b: validator error entry must carry severity='error'; got: {val_sev}"
        );
    }

    /// OBS-P4-004 (c): `total` in JSON equals the post-dedup diagnostic count.
    ///
    /// Fixture: `ValidationFailed` with 2 identical diagnostics — dedup must
    /// collapse them to 1, and `total` must be 1 (not 2).
    #[test]
    #[allow(non_snake_case)]
    fn test_OBS_P4_004_c_total_equals_post_dedup_count() {
        use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity};
        use slideforge_types::SourceSpan;
        use std::sync::Arc;

        let span = SourceSpan {
            file: Arc::from("deck.sf"),
            line: 5,
            col: 3,
            ..SourceSpan::default()
        };
        let diag = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: Arc::from("E-VAL-001"),
            message: Arc::from("duplicate diagnostic"),
            span: span.clone(),
            hint: None,
        };
        // Two identical entries — dedup must collapse to 1.
        let err = BuildError::ValidationFailed {
            diagnostics: vec![diag.clone(), diag],
            count: 2,
        };

        let json = render_build_error_to_json_value(&err);
        let total = json["total"].as_u64().expect("total must be a number");
        assert_eq!(
            total, 1,
            "OBS-P4-004c: total must be 1 after dedup of 2 identical diagnostics; got: {total}"
        );
        let arr = json["diagnostics"]
            .as_array()
            .expect("diagnostics must be array");
        assert_eq!(
            arr.len(),
            1,
            "OBS-P4-004c: diagnostics array must have 1 entry after dedup; got: {}",
            arr.len()
        );
    }

    /// OBS-P4-004 (d): dedup collapses exact duplicates in `MultistageFailed`
    /// validator entries.
    ///
    /// Fixture: two identical validator entries on the validator side of a
    /// `MultistageFailed` — the render path's internal dedup (OBS-P4-001) must
    /// collapse them to 1 and `total` must reflect the deduped count.
    #[test]
    #[allow(non_snake_case)]
    fn test_OBS_P4_004_d_multistage_validator_dedup_collapses_duplicate() {
        use slideforge::ParseSeverity;
        use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity};
        use slideforge_types::SourceSpan;
        use std::sync::Arc;

        let span = SourceSpan {
            file: Arc::from("deck.sf"),
            line: 8,
            col: 1,
            ..SourceSpan::default()
        };
        let dup_val_diag = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: Arc::from("E-VAL-001"),
            message: Arc::from("duplicate validator diagnostic"),
            span: span.clone(),
            hint: None,
        };

        let err = BuildError::MultistageFailed {
            eval_diagnostics: vec![slideforge::make_test_owned_diag(
                "E-EVL-001",
                "eval error at line 2",
            )],
            eval_sort_keys: vec![("deck.sf".to_owned(), 2, 1)],
            eval_severities: vec![ParseSeverity::Error],
            // Two identical validator entries — render path dedup must collapse to 1.
            validator_diagnostics: vec![dup_val_diag.clone(), dup_val_diag],
            eval_count: 1,
            validator_count: 2,
        };

        let json = render_build_error_to_json_value(&err);
        let arr = json["diagnostics"]
            .as_array()
            .expect("diagnostics must be array");
        // Expected: 1 eval + 1 deduped validator = 2 total.
        assert_eq!(
            arr.len(),
            2,
            "OBS-P4-004d: 2 duplicate validator entries must collapse to 1 after dedup; \
             expected 2 total (1 eval + 1 validator), got: {}",
            arr.len()
        );
        let total = json["total"].as_u64().expect("total must be a number");
        assert_eq!(
            total, 2,
            "OBS-P4-004d: total must be 2 (1 eval + 1 deduped validator); got: {total}"
        );
    }

    // ── F-094-P4-004 — BuildError::Layout rendered via structured format ──────

    /// F-094-P4-004: `render_build_error_to_string` for `BuildError::Layout`
    /// (specifically `BulletsOnContentlessSlideType`) must render using the
    /// structured diagnostic format that includes the `[E-LAY-008]` code prefix,
    /// NOT the raw `"Error: layout failed: ..."` bypass string.
    ///
    /// Before fix: `BuildError::Layout` hits the `other =>` arm in
    /// `render_build_error_to_string` → raw `Display` via `thiserror` →
    /// `"Error: layout failed: [E-LAY-008] Slide 'title' at <byte:42>:0:0 ..."`.
    ///
    /// After fix: rendered via miette-style path that surfaces the span
    /// and error code in the expected format.
    ///
    /// RED: before fix, `rendered.starts_with("Error: layout failed:")` is true,
    /// and the assertion that it does NOT will fail.
    #[test]
    fn test_f094_p4_004_cli_render_layout_error_uses_structured_format() {
        use super::render_build_error_to_string;
        use slideforge::LayoutError;
        use slideforge::error::BuildError;
        use slideforge_types::SourceSpan;
        use std::sync::Arc;

        let span = SourceSpan::new(Arc::from("deck.sf"), 4, 3, 42);
        let layout_err = LayoutError::BulletsOnContentlessSlideType {
            slide_type: Arc::from("title"),
            source_slide_index: 0,
            span,
        };
        let build_err = BuildError::Layout(layout_err);
        let rendered = render_build_error_to_string(&build_err, /*use_color=*/ false);

        // Must contain the E-LAY-008 code prefix from the error taxonomy.
        assert!(
            rendered.contains("[E-LAY-008]"),
            "F-094-P4-004: rendered Layout error must contain '[E-LAY-008]'; got: {rendered}"
        );

        // Must NOT be the raw bypass form.
        assert!(
            !rendered.starts_with("Error: layout failed:"),
            "F-094-P4-004: rendered Layout error must not use raw Display bypass; got: {rendered}"
        );

        // Must surface the file name so the user can locate the issue.
        assert!(
            rendered.contains("deck.sf"),
            "F-094-P4-004: rendered Layout error must contain file name 'deck.sf'; got: {rendered}"
        );
    }

    // ── F-094-P7-001 — JSON render path reports all accumulated E-LAY-008 instances ──

    /// F-094-P7-001: `render_build_error_to_json_value` for `BuildError::Layout`
    /// wrapping `LayoutError::Multiple { inner }` with 2 `BulletsOnContentlessSlideType`
    /// instances MUST emit `total == 2` and each diagnostic object MUST carry the
    /// `"code"` field with value `"E-LAY-008"`, a `"message"` field, and a `"span"`
    /// object with `"file"`, `"line"`, and `"col"` keys.
    ///
    /// Before fix: `BuildError::Layout` falls to `other =>` catch-all which produces
    /// ONE entry whose `"message"` is the `Multiple` Display string
    /// `"layout error: 2 accumulated errors; first: ..."`, so `total == 1`,
    /// `code` is `""`, and both span and the second instance are dropped.
    ///
    /// After fix: iterates `Multiple { inner }`, emitting one JSON object per inner
    /// error with `code`, `message`, and `span` fields.
    ///
    /// JSON shape mirrors the `ValidationFailed` arm (code/message/severity/span:
    /// {file/line/col}) with `severity` hardcoded to `"error"` for layout errors.
    #[test]
    fn test_f094_p7_001_json_render_layout_multiple_reports_all_instances() {
        use super::render_build_error_to_json_value;
        use slideforge::LayoutError;
        use slideforge::error::BuildError;
        use slideforge_types::SourceSpan;
        use std::sync::Arc;

        // Two slides, each with bullets on a contentless 'title' slide type.
        let span0 = SourceSpan::new(Arc::from("deck.sf"), 4, 3, 42);
        let span1 = SourceSpan::new(Arc::from("deck.sf"), 8, 3, 99);

        let err0 = LayoutError::BulletsOnContentlessSlideType {
            slide_type: Arc::from("title"),
            source_slide_index: 0,
            span: span0,
        };
        let err1 = LayoutError::BulletsOnContentlessSlideType {
            slide_type: Arc::from("closing"),
            source_slide_index: 2,
            span: span1,
        };
        let multiple = LayoutError::Multiple {
            inner: vec![err0, err1],
        };
        let build_err = BuildError::Layout(multiple);

        let json = render_build_error_to_json_value(&build_err);

        // total MUST be 2 — not 1 (the old catch-all collapsed everything).
        assert_eq!(
            json["total"].as_u64(),
            Some(2),
            "F-094-P7-001: total must be 2 for Multiple with 2 inner errors; got: {}",
            json["total"]
        );

        let diags = json["diagnostics"]
            .as_array()
            .expect("F-094-P7-001: 'diagnostics' must be an array");
        assert_eq!(
            diags.len(),
            2,
            "F-094-P7-001: diagnostics array must have 2 entries; got {}",
            diags.len()
        );

        // First diagnostic.
        let d0 = &diags[0];
        assert_eq!(
            d0["code"].as_str(),
            Some("E-LAY-008"),
            "F-094-P7-001: diag[0] 'code' must be 'E-LAY-008'; got: {}",
            d0["code"]
        );
        assert!(
            d0["message"]
                .as_str()
                .is_some_and(|m| m.contains("E-LAY-008")),
            "F-094-P7-001: diag[0] 'message' must contain 'E-LAY-008'; got: {}",
            d0["message"]
        );
        assert_eq!(
            d0["span"]["file"].as_str(),
            Some("deck.sf"),
            "F-094-P7-001: diag[0] span.file must be 'deck.sf'; got: {}",
            d0["span"]["file"]
        );
        assert_eq!(
            d0["span"]["line"].as_u64(),
            Some(4),
            "F-094-P7-001: diag[0] span.line must be 4; got: {}",
            d0["span"]["line"]
        );
        assert_eq!(
            d0["span"]["col"].as_u64(),
            Some(3),
            "F-094-P7-001: diag[0] span.col must be 3; got: {}",
            d0["span"]["col"]
        );

        // Second diagnostic — must NOT be dropped.
        let d1 = &diags[1];
        assert_eq!(
            d1["code"].as_str(),
            Some("E-LAY-008"),
            "F-094-P7-001: diag[1] 'code' must be 'E-LAY-008'; got: {}",
            d1["code"]
        );
        assert_eq!(
            d1["span"]["file"].as_str(),
            Some("deck.sf"),
            "F-094-P7-001: diag[1] span.file must be 'deck.sf'; got: {}",
            d1["span"]["file"]
        );
        assert_eq!(
            d1["span"]["line"].as_u64(),
            Some(8),
            "F-094-P7-001: diag[1] span.line must be 8; got: {}",
            d1["span"]["line"]
        );
        assert_eq!(
            d1["span"]["col"].as_u64(),
            Some(3),
            "F-094-P7-001: diag[1] span.col must be 3; got: {}",
            d1["span"]["col"]
        );

        // A bare (non-Multiple) LayoutError must also produce code + span (regression guard).
        let span2 = SourceSpan::new(Arc::from("other.sf"), 2, 1, 10);
        let bare_err = LayoutError::BulletsOnContentlessSlideType {
            slide_type: Arc::from("title"),
            source_slide_index: 1,
            span: span2,
        };
        let bare_build_err = BuildError::Layout(bare_err);
        let bare_json = render_build_error_to_json_value(&bare_build_err);
        assert_eq!(
            bare_json["total"].as_u64(),
            Some(1),
            "F-094-P7-001: bare Layout error total must be 1; got: {}",
            bare_json["total"]
        );
        let bare_diags = bare_json["diagnostics"]
            .as_array()
            .expect("F-094-P7-001: bare diagnostics must be an array");
        assert_eq!(
            bare_diags[0]["code"].as_str(),
            Some("E-LAY-008"),
            "F-094-P7-001: bare diag 'code' must be 'E-LAY-008'; got: {}",
            bare_diags[0]["code"]
        );
        assert_eq!(
            bare_diags[0]["span"]["file"].as_str(),
            Some("other.sf"),
            "F-094-P7-001: bare diag span.file must be 'other.sf'; got: {}",
            bare_diags[0]["span"]["file"]
        );
    }
}
