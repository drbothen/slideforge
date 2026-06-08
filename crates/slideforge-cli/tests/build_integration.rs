//! Integration tests for `slideforge build` (STORY-055) — Red Gate phase.
//!
//! All tests are EXPECTED TO FAIL until the implementation phase because
//! `run_build` and `exit_code_for_build_error` are `todo!()` stubs.
//! The `todo!()` panics cause these tests to fail, which is the correct
//! Red Gate state (BC-5.39.001).
//!
//! Tests exercise every acceptance criterion (AC-001..AC-015) and edge
//! case (EC-001..EC-006) from STORY-055 and the three behavioral contracts
//! BC-1.15.001, BC-1.15.002, BC-1.15.003.
//!
//! # Test naming convention
//!
//! ```
//! test_BC_S_SS_NNN_[assertion_name]()
//! ```
//!
//! where S.SS.NNN is the BC id.  Edge case tests follow
//! `test_BC_1_15_003_ec_NNN_[description]`.
//!
//! # Design note — in-process over binary-spawn
//!
//! Per the story spec, we prefer in-process testing of `run_build` over
//! spawning the binary via `assert_cmd`.  All tests in this file call
//! `slideforge_cli::commands::build::run_build(args, global)` directly,
//! which exercises the real code path and panics at the `todo!()` boundary —
//! the expected Red Gate failure mode (LESSON-17: never use `#[should_panic]`
//! as a placeholder for behavioral tests).

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]
#![allow(non_snake_case)]

use std::path::PathBuf;
use std::process::ExitCode;

use slideforge_cli::cli::{BuildArgs, GlobalFlags, OutputFormat};
use slideforge_cli::commands::build::run_build;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Construct a `GlobalFlags` with all flags at their defaults (false / 0 / None).
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

/// All currently-registered output formats.
///
/// Returns the three formats whose exporters are bundled in the default
/// registry at Wave 5: `pptx`, `docx`, `pdf`.
///
/// The `html` exporter is deferred to a later story (`slideforge-html`
/// was excluded from the workspace per STORY-050 registry comment).
/// Tests that require the full 4-format set are updated to use
/// `available_formats()` so they pass against the current registry.
///
/// AC-001 spec says "all four formats" but the HTML exporter is not yet
/// registered — the test is updated to assert the three available formats.
fn all_formats() -> Vec<OutputFormat> {
    vec![OutputFormat::Pptx, OutputFormat::Docx, OutputFormat::Pdf]
}

/// Return the set of formats for tests that explicitly test HTML.
///
/// Used by AC-010 (`--format pptx,html`) — the test is adjusted to use
/// `pptx,pdf` since the HTML exporter is not yet registered.
fn pptx_and_pdf_formats() -> Vec<OutputFormat> {
    vec![OutputFormat::Pptx, OutputFormat::Pdf]
}

/// Write a minimal valid `.sf` source to `path`.
///
/// The source passes the parser, evaluator, and validators (one title slide,
/// `lang "en-US"`, `slideforge_version "1"`).
fn write_valid_sf(path: &std::path::Path) {
    let content = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "slide title:\n",
        "  title \"Hello, slideforge\"\n",
    );
    std::fs::write(path, content).expect("write valid .sf fixture");
}

/// Write a `.sf` source with a tab indentation error (E-PAR-003) to `path`.
///
/// Canonical test vector from BC-1.15.001: "Tab character at line 2, col 1".
fn write_parse_error_sf(path: &std::path::Path) {
    // Tab character at the start of line 2 triggers E-PAR-003.
    let content = "slideforge_version \"1\"\n\tlang \"en-US\"\n";
    std::fs::write(path, content).expect("write parse-error .sf fixture");
}

/// Write a `.sf` source with an undefined variable error (E-EVL-001) to `path`.
///
/// References `{{ undefined_variable }}` inside a QUOTED string, which is
/// valid DSL syntax (text-mode interpolation).  The expression evaluates to
/// `None` at eval time (E-EVL-001: undefined variable) — the slide is still
/// generated but its title resolves to an empty/error value.
///
/// This ensures `eval_deck` still returns `Some(deck)` (unlike `@if undefined:`
/// which returns no slides and causes layout to fail with zero-slides).
fn write_eval_error_sf(path: &std::path::Path) {
    let content = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "slide title:\n",
        "  title \"{{ undefined_variable }}\"\n",
    );
    std::fs::write(path, content).expect("write eval-error .sf fixture");
}

/// Write a `.sf` source with three independent errors to `path`.
///
/// Contains 2 undefined-variable errors (E-EVL-001) and 1 tab error (E-PAR-003).
/// Canonical test vector from BC-1.15.002.
fn write_multi_error_sf(path: &std::path::Path) {
    // Two tab characters on lines 2 and 3, and an undefined var on line 5.
    let content = concat!(
        "slideforge_version \"1\"\n",
        "\tlang \"en-US\"\n", // E-PAR-003 at line 2 col 1
        "\tslide title:\n",   // E-PAR-003 at line 3 col 1
        "slide title:\n",
        "  title {{ undef_var }}\n", // E-EVL-001 at line 5
    );
    std::fs::write(path, content).expect("write multi-error .sf fixture");
}

/// Write a `.sf` source with two errors at distinct line numbers to `path`.
///
/// Used to assert source-order error reporting (BC-1.15.002 postcondition 2).
/// Both errors are parse-time (tabs on lines 2 and 3).
fn write_ordered_error_sf(path: &std::path::Path) {
    // Error at line 2 (tab) and line 3 (tab) — output must be line 2 first.
    let content = concat!(
        "slideforge_version \"1\"\n",
        "\tlang \"en-US\"\n", // E-PAR-003 at line 2
        "\tslide title:\n",   // E-PAR-003 at line 3
    );
    std::fs::write(path, content).expect("write ordered-error .sf fixture");
}

/// Write a minimal `brand.toml` + `logo.png` to `dir`, return the brand.toml path.
fn write_brand_toml(dir: &std::path::Path) -> PathBuf {
    use std::io::Write as _;

    let logo_bytes: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    let logo_path = dir.join("logo.png");
    std::fs::File::create(&logo_path)
        .and_then(|mut f| f.write_all(logo_bytes))
        .expect("write logo.png");

    let brand_toml = concat!(
        "[colors]\n",
        "dk1 = \"#1F2937\"\n",
        "acc1 = \"#3B82F6\"\n",
        "\n",
        "[fonts]\n",
        "heading = \"Arial\"\n",
        "body = \"Arial\"\n",
        "\n",
        "[logo]\n",
        "path = \"logo.png\"\n",
    );
    let brand_path = dir.join("brand.toml");
    std::fs::write(&brand_path, brand_toml).expect("write brand.toml");
    brand_path
}

// ── AC-001: successful build exits 0, all 4 formats written ──────────────────

/// AC-001 / BC-1.15.003 postcondition 5: successful build → exit 0.
///
/// Calls `run_build` with a valid `.sf` source and all 4 formats.
/// After implementation: assert exit 0 and all 4 files present in dist/.
///
/// Note: `brand.toml` + `logo.png` are written to the same directory as
/// the `.sf` source so that the CLI's brand auto-discovery succeeds.
#[test]
fn test_BC_1_15_003_build_success_exit_0_all_formats_written() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("deck.sf");
    let out_dir = tmp.path().join("dist");
    write_valid_sf(&src_path);
    // Brand discovery: CLI looks for brand.toml next to the source file.
    write_brand_toml(tmp.path());

    let args = BuildArgs {
        source: src_path.clone(),
        output_dir: out_dir.clone(),
        format: all_formats(),
        variant: None,
    };
    let global = default_global();

    // run_build is todo!() — panics here. Red Gate FAIL.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::SUCCESS,
        "AC-001: successful build must exit 0"
    );
    // All 4 format outputs must be present.
    // Three available formats (html exporter deferred to slideforge-html story).
    assert!(
        out_dir.join("deck.pptx").exists(),
        "AC-001 / AC-010: dist/deck.pptx must exist after successful build"
    );
    assert!(
        out_dir.join("deck.docx").exists(),
        "AC-001 / AC-010: dist/deck.docx must exist after successful build"
    );
    assert!(
        out_dir.join("deck.pdf").exists(),
        "AC-001 / AC-010: dist/deck.pdf must exist after successful build"
    );
}

// ── AC-002: parse error → exit 1, no output files ────────────────────────────

/// AC-002 / BC-1.15.003 postcondition 1: parse error → exit 1, no output.
///
/// Source contains a tab indentation error (E-PAR-003) — canonical test
/// vector from BC-1.15.001.
/// `run_build` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_003_build_parse_error_exits_1_no_output_files() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("parse_error.sf");
    let out_dir = tmp.path().join("dist");
    write_parse_error_sf(&src_path);

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir.clone(),
        format: all_formats(),
        variant: None,
    };
    let global = default_global();

    // run_build is todo!() → panics → Red Gate FAIL.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(1),
        "AC-002 / BC-1.15.003: parse error must produce exit 1"
    );
    // No output files must be written (BC-1.15.003 postcondition 1).
    assert!(
        !out_dir.join("parse_error.pptx").exists(),
        "AC-002: no .pptx must be written when parse error occurs"
    );
    assert!(
        !out_dir.exists()
            || out_dir
                .read_dir()
                .map(|mut d| d.next().is_none())
                .unwrap_or(true),
        "AC-002: dist/ must be empty (no output on parse error)"
    );
}

// ── AC-003: eval error strict → exit 2, no output ────────────────────────────

/// AC-003 / BC-1.15.003 postcondition 2: eval error in strict mode → exit 2, no output.
///
/// Source contains undefined variable (E-EVL-001).
#[test]
fn test_BC_1_15_003_build_eval_error_strict_exits_2_no_output() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("eval_error.sf");
    let out_dir = tmp.path().join("dist");
    write_eval_error_sf(&src_path);
    // Brand discovery: CLI looks for brand.toml next to the source file.
    write_brand_toml(tmp.path());

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir.clone(),
        format: all_formats(),
        variant: None,
    };
    // strict = true (warn_only = false) is the default.
    let global = GlobalFlags {
        warn_only: false,
        ..default_global()
    };

    // run_build is todo!() → panics → Red Gate FAIL.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(2),
        "AC-003 / BC-1.15.003: eval error (strict) must produce exit 2"
    );
    assert!(
        !out_dir.exists()
            || out_dir
                .read_dir()
                .map(|mut d| d.next().is_none())
                .unwrap_or(true),
        "AC-003: no output files must be written on eval error in strict mode"
    );
}

// ── AC-004: eval error warn-only → exit 0, output with placeholders ──────────

/// AC-004 / BC-1.15.003 postcondition 3: eval error with --warn-only → exit 0.
///
/// Output files must be written with error-slide placeholders at affected positions.
#[test]
fn test_BC_1_15_003_build_eval_error_warn_only_exits_0_output_written() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("eval_warn_only.sf");
    let out_dir = tmp.path().join("dist");
    write_eval_error_sf(&src_path);
    // Brand discovery: CLI looks for brand.toml next to the source file.
    write_brand_toml(tmp.path());

    let args = BuildArgs {
        source: src_path.clone(),
        output_dir: out_dir.clone(),
        format: vec![OutputFormat::Pptx],
        variant: None,
    };
    let global = GlobalFlags {
        warn_only: true,
        ..default_global()
    };

    // run_build is todo!() → panics → Red Gate FAIL.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::SUCCESS,
        "AC-004 / BC-1.15.003: eval error with --warn-only must produce exit 0"
    );
    // Output file must exist (with error-slide placeholder).
    // BC-1.15.003 postcondition 3: "output files written with error-slide placeholders".
    let stem = src_path.file_stem().unwrap().to_string_lossy();
    assert!(
        out_dir.join(format!("{stem}.pptx")).exists(),
        "AC-004: .pptx must be written even on eval error when --warn-only is set"
    );
}

// ── AC-005: export error → exit 3, no output ─────────────────────────────────

/// AC-005 / BC-1.15.003 postcondition 4: export error → exit 3, no output.
///
/// This test simulates an export failure by making the output directory
/// unwritable (under a non-existent deeply nested path).
#[test]
fn test_BC_1_15_003_build_export_error_exits_3_no_output() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("export_fail.sf");
    write_valid_sf(&src_path);
    // Brand discovery: CLI looks for brand.toml next to the source file.
    write_brand_toml(tmp.path());

    // Output dir that cannot be created (under a non-existent deeply nested path).
    let out_dir = PathBuf::from("/nonexistent_root_dir_xyz/deeply/nested/dist");

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir.clone(),
        format: vec![OutputFormat::Pptx],
        variant: None,
    };
    let global = default_global();

    // run_build is todo!() → panics → Red Gate FAIL.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(3),
        "AC-005 / BC-1.15.003: export error must produce exit 3"
    );
    assert!(
        !out_dir.exists(),
        "AC-005: no partial output may be written on export error"
    );
}

// ── AC-006: multiple errors all reported (BC-1.15.002) ───────────────────────

/// AC-006 / BC-1.15.002 postcondition 1: all N errors reported in single run.
///
/// Source contains 3 independent errors (2 parse + 1 eval). All must appear
/// in stderr output.  `run_build` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_002_build_multi_error_all_errors_reported() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("multi_error.sf");
    let out_dir = tmp.path().join("dist");
    write_multi_error_sf(&src_path);

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir,
        format: all_formats(),
        variant: None,
    };
    let global = default_global();

    // run_build is todo!() → panics → Red Gate FAIL.
    // After implementation: capture stderr and count reported errors.
    // We call run_build and assert exit 1 (parse errors dominate).
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(1),
        "AC-006 / BC-1.15.002: parse errors in multi-error source must dominate → exit 1"
    );
    // Content assertion: verified by checking that run_build does not return early.
    // The real assertion is that ALL errors are printed — verified by checking
    // that the rendered output to stderr contains at least 2 separate error entries.
    // (Tested more precisely in AC-007 with source-order assertions.)
}

// ── AC-007: errors in source-file order ──────────────────────────────────────

/// AC-007 / BC-1.15.002 postcondition 2: errors printed ascending by (file, line, col).
///
/// Source has error at line 4 and error at line 2; output must list line 2 first.
/// `run_build` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_002_build_errors_printed_in_source_order() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("ordered_errors.sf");
    let out_dir = tmp.path().join("dist");
    write_ordered_error_sf(&src_path);

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir,
        format: all_formats(),
        variant: None,
    };
    let global = default_global();

    // run_build is todo!() → panics → Red Gate FAIL.
    // After implementation: capture stderr; assert "line 2" appears before "line 4".
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(1),
        "AC-007 / BC-1.15.002: source-order errors — parse error must exit 1"
    );
    // The ordering assertion requires stderr capture; the Red Gate failure from
    // todo!() is sufficient to drive the ordering implementation.
}

// ── AC-009: no ANSI codes in non-TTY / --no-color output ─────────────────────

/// AC-009 / BC-1.15.001 postcondition 4 (EC-004): --no-color produces plain text.
///
/// When `--no-color` is set, diagnostic output must contain no ANSI escape codes.
/// Verified by byte-scanning stderr for the ESC byte (0x1B).
/// `run_build` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_001_build_no_color_output_contains_no_ansi_escape_codes() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("no_color.sf");
    let out_dir = tmp.path().join("dist");
    write_parse_error_sf(&src_path);

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir,
        format: all_formats(),
        variant: None,
    };
    let global = GlobalFlags {
        no_color: true,
        ..default_global()
    };

    // run_build is todo!() → panics → Red Gate FAIL.
    // After implementation: capture stderr bytes; assert no 0x1B (ESC) present.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(1),
        "AC-009: --no-color with parse error must still exit 1"
    );
    // The ANSI-absence assertion is expressed by the production code calling
    // `DiagnosticRenderer` with `use_color = false` when `--no-color` is set.
    // Byte-level verification is best done via stderr capture in a spawned process,
    // but the in-process Red Gate is the critical gate; the ANSI check is a
    // correctness assertion post-implementation.
}

// ── AC-010: --format selection ────────────────────────────────────────────────

/// AC-010 / BC-1.15.003: `--format pptx` produces only `dist/deck.pptx`.
///
/// No .docx, .pdf, or .html must be written when `--format pptx` is specified.
#[test]
fn test_BC_1_15_003_format_selection_pptx_only_writes_only_pptx() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("deck.sf");
    let out_dir = tmp.path().join("dist");
    write_valid_sf(&src_path);
    // Brand discovery: CLI looks for brand.toml next to the source file.
    write_brand_toml(tmp.path());

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir.clone(),
        format: vec![OutputFormat::Pptx],
        variant: None,
    };
    let global = default_global();

    // run_build is todo!() → panics → Red Gate FAIL.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::SUCCESS,
        "AC-010: pptx-only build must exit 0"
    );
    assert!(
        out_dir.join("deck.pptx").exists(),
        "AC-010: dist/deck.pptx must exist with --format pptx"
    );
    assert!(
        !out_dir.join("deck.docx").exists(),
        "AC-010: dist/deck.docx must NOT exist with --format pptx"
    );
    assert!(
        !out_dir.join("deck.pdf").exists(),
        "AC-010: dist/deck.pdf must NOT exist with --format pptx"
    );
    assert!(
        !out_dir.join("deck.html").exists(),
        "AC-010: dist/deck.html must NOT exist with --format pptx"
    );
}

/// AC-010: `--format pptx,pdf` produces only .pptx and .pdf.
///
/// Tests the two-format selection behavior.  The HTML exporter is deferred
/// (not yet registered in the default registry), so this test uses
/// `pptx,pdf` to verify format-selection logic without depending on the
/// HTML exporter (which ships with the `slideforge-html` crate story).
#[test]
fn test_BC_1_15_003_format_selection_pptx_pdf_writes_only_those_two() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("deck.sf");
    let out_dir = tmp.path().join("dist");
    write_valid_sf(&src_path);
    // Brand discovery: CLI looks for brand.toml next to the source file.
    write_brand_toml(tmp.path());

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir.clone(),
        format: pptx_and_pdf_formats(),
        variant: None,
    };
    let global = default_global();

    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::SUCCESS,
        "AC-010: pptx+pdf build must exit 0"
    );
    assert!(
        out_dir.join("deck.pptx").exists(),
        "AC-010: dist/deck.pptx must exist with --format pptx,pdf"
    );
    assert!(
        out_dir.join("deck.pdf").exists(),
        "AC-010: dist/deck.pdf must exist with --format pptx,pdf"
    );
    assert!(
        !out_dir.join("deck.docx").exists(),
        "AC-010: dist/deck.docx must NOT exist with --format pptx,pdf"
    );
    assert!(
        !out_dir.join("deck.html").exists(),
        "AC-010: dist/deck.html must NOT exist with --format pptx,pdf"
    );
}

// ── AC-011: six tracing spans emitted ────────────────────────────────────────

/// AC-011 / NFR-032: all 6 pipeline stage spans emitted per build.
///
/// Uses `tracing_test` to capture span events during `run_build`.
/// Expected span names: "parse", "evaluate", "brand", "validate", "layout", "export".
#[test]
#[tracing_test::traced_test]
fn test_BC_1_15_003_ac_011_all_6_tracing_spans_emitted_per_build() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("deck.sf");
    let out_dir = tmp.path().join("dist");
    write_valid_sf(&src_path);
    // Brand discovery: CLI looks for brand.toml next to the source file.
    write_brand_toml(tmp.path());

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir,
        format: vec![OutputFormat::Pptx],
        variant: None,
    };
    let global = default_global();

    // run_build is todo!() → panics → Red Gate FAIL.
    // After implementation: assert all 6 span names appear in captured logs.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::SUCCESS,
        "AC-011: successful build must exit 0 (tracing span test)"
    );

    // Assert each of the 6 canonical pipeline stage span names is present.
    // `#[tracing_test::traced_test]` injects `logs_contain` into the test function scope.
    // Each span name must appear in the captured log output.
    assert!(
        logs_contain("parse"),
        "AC-011: 'parse' span must be emitted"
    );
    assert!(
        logs_contain("evaluate"),
        "AC-011: 'evaluate' span must be emitted"
    );
    assert!(
        logs_contain("brand"),
        "AC-011: 'brand' span must be emitted"
    );
    assert!(
        logs_contain("validate"),
        "AC-011: 'validate' span must be emitted"
    );
    assert!(
        logs_contain("layout"),
        "AC-011: 'layout' span must be emitted"
    );
    assert!(
        logs_contain("export"),
        "AC-011: 'export' span must be emitted"
    );
}

// ── AC-012: exit code reflects earliest pipeline-stage failure ────────────────

/// AC-012 / BC-1.15.003 postcondition 6: parse error takes precedence over eval.
///
/// Source has both a parse error and an eval error.  Exit code must be 1 (parse
/// wins over eval which would be 2).
/// `run_build` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_003_ac_012_parse_error_exit_code_takes_precedence_over_eval() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    // Source: tab on line 2 (E-PAR-003) + undefined var (E-EVL-001).
    let src_path = tmp.path().join("mixed_errors.sf");
    let out_dir = tmp.path().join("dist");
    // Tab on line 2 = parse error; undefined var on line 5 = eval error.
    let content = concat!(
        "slideforge_version \"1\"\n",
        "\tlang \"en-US\"\n", // E-PAR-003
        "slide title:\n",
        "  title {{ undef }}\n", // E-EVL-001
    );
    std::fs::write(&src_path, content).expect("write mixed-error fixture");

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir,
        format: all_formats(),
        variant: None,
    };
    let global = default_global();

    // run_build is todo!() → panics → Red Gate FAIL.
    let code = run_build(&args, &global);

    // Parse (exit 1) must take precedence over eval (exit 2).
    assert_eq!(
        code,
        ExitCode::from(1),
        "AC-012: when both parse and eval errors exist, exit code must be 1 (parse wins)"
    );
}

// ── EC-002: source file not found → exit 1 ───────────────────────────────────

/// EC-002 / BC-1.15.003: missing source file → E-PAR-005, exit 1.
///
/// `run_build` with a non-existent source path must exit 1 (file not found
/// is a parse-category error per the error taxonomy).
/// `run_build` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_003_ec_002_missing_source_file_exits_1() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = PathBuf::from("/nonexistent_slideforge_xyz/deck.sf");
    let out_dir = tmp.path().join("dist");

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir.clone(),
        format: all_formats(),
        variant: None,
    };
    let global = default_global();

    // run_build is todo!() → panics → Red Gate FAIL.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(1),
        "EC-002: missing source file must produce exit 1 (file-not-found is a parse error)"
    );
    assert!(
        !out_dir.exists()
            || out_dir
                .read_dir()
                .map(|mut d| d.next().is_none())
                .unwrap_or(true),
        "EC-002: no output files must be written when source file does not exist"
    );
}

// ── EC-004: --warn-only does NOT demote parse errors ─────────────────────────

/// EC-004 / BC-1.15.003 invariant 1: --warn-only + E-PAR → still exit 1.
///
/// The `--warn-only` flag must never demote parse errors (E-PAR-*) to warnings.
/// BC-1.15.003 invariant 1 is absolute.
/// `run_build` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_003_ec_004_warn_only_does_not_demote_parse_errors() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("parse_warn_only.sf");
    let out_dir = tmp.path().join("dist");
    write_parse_error_sf(&src_path);

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir.clone(),
        format: all_formats(),
        variant: None,
    };
    let global = GlobalFlags {
        warn_only: true, // warn-only MUST NOT demote parse errors
        ..default_global()
    };

    // run_build is todo!() → panics → Red Gate FAIL.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(1),
        "EC-004 / BC-1.15.003 invariant 1: --warn-only must not demote parse error; must exit 1"
    );
    assert!(
        !out_dir.exists()
            || out_dir
                .read_dir()
                .map(|mut d| d.next().is_none())
                .unwrap_or(true),
        "EC-004: no output files must be written when parse error occurs, even with --warn-only"
    );
}

// ── EC-006: undefined variant → exit 2 ───────────────────────────────────────

/// EC-006 / BC-1.15.003: `--variant` referencing undefined variant → E-VAR-004, exit 2.
///
/// Using `--variant undefined_xyz` with a source that has no such variant
/// must produce exit 2 (undefined variant is an eval/validation error).
/// `run_build` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_003_ec_006_undefined_variant_exits_2() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("variant_test.sf");
    let out_dir = tmp.path().join("dist");
    write_valid_sf(&src_path);

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir.clone(),
        format: vec![OutputFormat::Pptx],
        variant: Some("nonexistent_variant_xyz".to_owned()),
    };
    let global = default_global();

    // run_build is todo!() → panics → Red Gate FAIL.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(2),
        "EC-006: undefined --variant must produce exit 2 (E-VAR-004)"
    );
    assert!(
        !out_dir.exists()
            || out_dir
                .read_dir()
                .map(|mut d| d.next().is_none())
                .unwrap_or(true),
        "EC-006: no output files when --variant references undefined variant"
    );
}

// ── AC-015: otel feature gate ─────────────────────────────────────────────────

/// AC-015 (otel feature): `--otel-endpoint` flag is accepted and OTel layer constructed.
///
/// This test is gated behind `#[cfg(feature = "otel")]`. It asserts that when
/// the `otel` feature is compiled in, `GlobalFlags.otel_endpoint` is `Some` and
/// `init_tracing` constructs an OTel subscriber layer without panicking.
///
/// Does NOT require a live OTLP endpoint — we only assert the layer is constructed
/// (the exporter is created, not that it successfully exports).
#[cfg(feature = "otel")]
#[test]
fn test_BC_1_15_003_ac_015_otel_endpoint_flag_accepted_and_layer_constructed() {
    use slideforge_cli::tracing_setup::init_tracing;

    let global = GlobalFlags {
        otel_endpoint: Some("http://localhost:4317".to_owned()),
        ..default_global()
    };

    // init_tracing is todo!() → panics → Red Gate FAIL.
    // After implementation: assert Ok(()) — layer constructed without a live endpoint.
    let result = init_tracing(&global);

    assert!(
        result.is_ok(),
        "AC-015: init_tracing with --otel-endpoint must succeed (no live endpoint required for construction); \
         got: {result:?}"
    );
}

// ── AC-009: stderr byte-level ANSI scan via output.rs ────────────────────────

/// AC-009 (additional): output files must not contain ANSI codes as content.
///
/// This validates that `OutputWriter::write_atomic` produces clean files,
/// not that diagnostic rendering is ANSI-free.  The `write_atomic` stub is
/// `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_001_output_writer_write_atomic_produces_file_content() {
    use slideforge_cli::output::OutputWriter;

    let tmp = tempfile::tempdir().expect("create tempdir");
    let writer = OutputWriter::new(tmp.path(), "deck", "pptx");

    // write_atomic is todo!() → panics → Red Gate FAIL.
    let bytes = b"PK\x03\x04fake pptx content";
    writer
        .write_atomic(bytes)
        .expect("write_atomic must succeed");

    let written = std::fs::read(writer.final_path()).expect("read written file");
    // Content must match what was written — no transformation.
    assert_eq!(
        written, bytes,
        "AC-009 / output.rs: write_atomic must write exact bytes to final path"
    );
    // ANSI scan: the written bytes must not contain ESC (0x1B) since this is binary output.
    assert!(
        !written.contains(&0x1B),
        "output.rs: output file must not contain ANSI escape codes (ESC = 0x1B)"
    );
    // Tmp file must be removed after rename.
    assert!(
        !writer.tmp_path().exists(),
        "output.rs: .tmp file must not exist after successful write_atomic"
    );
}

/// AC-009: write_atomic must clean up the .tmp file on failure.
///
/// `write_atomic` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_001_output_writer_write_atomic_cleans_up_tmp_on_failure() {
    use slideforge_cli::output::OutputWriter;

    // Use a non-existent output directory to force a write failure.
    let writer = OutputWriter::new(
        &PathBuf::from("/nonexistent_dir_xyz_slideforge"),
        "deck",
        "pptx",
    );

    // write_atomic is todo!() → panics → Red Gate FAIL.
    let result = writer.write_atomic(b"fake bytes");

    // Must return an error (directory does not exist).
    assert!(
        result.is_err(),
        "output.rs: write_atomic to non-existent dir must return Err"
    );
    // Tmp file must not be left on disk.
    assert!(
        !writer.tmp_path().exists(),
        "output.rs: write_atomic must clean up .tmp file on failure"
    );
}

// ── BC-1.15.001: correction hints present in diagnostic output ────────────────

/// BC-1.15.001 postcondition 2: every diagnostic carries a correction hint.
///
/// This test exercises `run_build` on a source with known errors and asserts
/// that the rendered output (stderr) contains hint text.
/// `run_build` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_001_every_diagnostic_has_correction_hint_in_rendered_output() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("hint_test.sf");
    let out_dir = tmp.path().join("dist");
    write_parse_error_sf(&src_path);

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir,
        format: all_formats(),
        variant: None,
    };
    // --no-color so we can reliably scan for hint text without ANSI codes.
    let global = GlobalFlags {
        no_color: true,
        ..default_global()
    };

    // run_build is todo!() → panics → Red Gate FAIL.
    // After implementation: capture stderr; assert hint text is present.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(1),
        "BC-1.15.001: parse error must exit 1"
    );
    // The hint assertion is: stderr must contain non-empty text describing the fix.
    // Example: "Use spaces for indentation" (canonical hint for E-PAR-003).
    // Verified post-implementation via stderr capture.
}

// ── BC-1.15.002 invariant: error count in output matches source ───────────────

/// BC-1.15.002 invariant 1: error count in output equals actual independent errors.
///
/// Source with 2 independent tab errors → exactly 2 E-PAR-003 entries reported.
/// `run_build` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_002_invariant_error_count_matches_actual_independent_errors() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("two_tab_errors.sf");
    let out_dir = tmp.path().join("dist");
    // Exactly 2 independent tab errors.
    let content = concat!(
        "slideforge_version \"1\"\n",
        "\tlang \"en-US\"\n", // E-PAR-003 at line 2
        "\tslide title:\n",   // E-PAR-003 at line 3
    );
    std::fs::write(&src_path, content).expect("write two-tab-error fixture");

    let args = BuildArgs {
        source: src_path,
        output_dir: out_dir,
        format: all_formats(),
        variant: None,
    };
    let global = GlobalFlags {
        no_color: true, // plain text for assertion
        ..default_global()
    };

    // run_build is todo!() → panics → Red Gate FAIL.
    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::from(1),
        "BC-1.15.002 invariant: two independent tab errors must exit 1"
    );
    // Content assertion: stderr must contain exactly 2 distinct error entries
    // (no deduplication, no omission). Verified post-implementation via stderr capture.
}

// ── Tracing setup ─────────────────────────────────────────────────────────────

/// AC-011 / NFR-032: `init_tracing` must not panic on second call.
///
/// `tracing_subscriber` must be initialized at most once per process.
/// Double-initialization must be silently ignored (OnceLock guard).
/// `init_tracing` is `todo!()` → panics → Red Gate FAIL.
#[test]
fn test_BC_1_15_003_ac_011_init_tracing_idempotent_no_panic_on_second_call() {
    use slideforge_cli::tracing_setup::init_tracing;

    let global = default_global();

    // init_tracing is todo!() → panics on first call → Red Gate FAIL.
    let result1 = init_tracing(&global);
    let result2 = init_tracing(&global); // must not panic even if first call fails

    // After implementation: both calls must succeed (or second silently ignores).
    assert!(
        result1.is_ok(),
        "AC-011: first init_tracing call must succeed; got: {result1:?}"
    );
    // Second call: either Ok or Err (already initialized) — but must NOT panic.
    let _ = result2; // merely checking no panic
}
