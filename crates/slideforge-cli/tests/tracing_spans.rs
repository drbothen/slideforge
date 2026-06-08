//! AC-011 / NFR-032: pipeline stage span emission tests.
//!
//! This file is a SEPARATE integration-test binary from `build_integration.rs`.
//! Keeping it separate is REQUIRED for correctness (LESSON-16):
//!
//! `#[tracing_test::traced_test]` calls `tracing::dispatcher::set_global_default`
//! via a `Once::call_once` closure that panics if the global is already set.
//! `build_integration.rs` contains `test_BC_1_15_003_ac_011_init_tracing_...`
//! which calls `init_tracing` and sets the global subscriber.  If both tests
//! ran in the same binary, whichever ran second would panic.
//!
//! By placing the `traced_test` spans test in THIS file, it always gets a fresh
//! process with a virgin global-dispatcher state, making `call_once` reliable.
//!
//! # Traceability
//!
//! - BC-1.15.003 AC-011: six pipeline stage spans emitted per build
//! - NFR-032: `tracing` instrumentation throughout the pipeline

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]
#![allow(non_snake_case)]

use std::process::ExitCode;

use slideforge_cli::cli::{BuildArgs, GlobalFlags, OutputFormat};
use slideforge_cli::commands::build::run_build;

/// Construct a `GlobalFlags` with all flags at their defaults.
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

/// Write a minimal valid `.sf` source to `path`.
fn write_valid_sf(path: &std::path::Path) {
    let content = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "slide title:\n",
        "  title \"Hello, slideforge\"\n",
    );
    std::fs::write(path, content).expect("write valid .sf fixture");
}

/// Write a minimal `brand.toml` to `dir/brand.toml`.
fn write_brand_toml(dir: &std::path::Path) {
    use std::io::Write as _;

    let logo_bytes: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
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
    std::fs::write(dir.join("brand.toml"), brand_toml).expect("write brand.toml");
}

/// AC-011 / NFR-032: all 6 pipeline stage spans emitted per build.
///
/// Uses `tracing_test` to capture span events during `run_build`.
/// Expected span names: "parse", "evaluate", "brand", "validate", "layout", "export".
///
/// **Isolation:**  this test lives in `tests/tracing_spans.rs` (a separate test
/// binary from `tests/build_integration.rs`) so that the global tracing subscriber
/// setup done by `#[tracing_test::traced_test]` is never in the same process as
/// `init_tracing` calls that also set the global dispatcher (LESSON-16).
///
/// The canonical span definition in `slideforge/src/lib.rs`:
/// ```rust,ignore
/// tracing::info_span!("parse",    stage = "parse",    source_len = ...)
/// tracing::info_span!("evaluate", stage = "evaluate")
/// tracing::info_span!("brand",    stage = "brand")
/// tracing::info_span!("validate", stage = "validate", strict = ...)
/// tracing::info_span!("layout",   stage = "layout")
/// tracing::info_span!("export",   stage = "export",   format = ...)
/// ```
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

    let code = run_build(&args, &global);

    assert_eq!(
        code,
        ExitCode::SUCCESS,
        "AC-011: successful build must exit 0 (tracing span test)"
    );

    // Assert each of the 6 canonical pipeline stage span names is present.
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
