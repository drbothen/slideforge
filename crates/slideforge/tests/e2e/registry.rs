//! Plugin registry dog-fooding E2E tests — STORY-050 AC-004, AC-005.
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-004**: `cargo tree -p slideforge-pptx` must NOT list `slideforge-pdf`
//!   or `slideforge-html` as transitive dependencies.
//! - **AC-005**: external test plugin (STORY-049 AC-007) confirmed via
//!   `external_plugin_test.rs`; this module adds a confirmation test that the
//!   default registry assembles without error.

#![allow(clippy::unwrap_used)]

use crate::e2e::{BrandTmpDir, fixture_source};

/// AC-004: `cargo tree -p slideforge-pptx` must NOT list `slideforge-pdf` or
/// `slideforge-html` as transitive dependencies.
///
/// Traceability: BC-5.02.002 postcondition 1 and invariant 1.
#[test]
fn test_bc_5_02_002_ac004_pptx_has_no_cross_exporter_deps() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .and_then(std::path::Path::parent) // workspace root
        .expect("AC-004: cannot locate workspace root from CARGO_MANIFEST_DIR");

    let output = std::process::Command::new("cargo")
        .args(["tree", "-p", "slideforge-pptx", "--no-dedupe"])
        .current_dir(workspace_root)
        .output()
        .unwrap_or_else(|e| panic!("AC-004: cargo tree invocation failed: {e}"));

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("slideforge-pdf"),
        "AC-004: slideforge-pptx must NOT depend (transitively) on slideforge-pdf; \
         this indicates a bypass path. cargo tree output:\n{stdout}"
    );
    assert!(
        !stdout.contains("slideforge-html"),
        "AC-004: slideforge-pptx must NOT depend (transitively) on slideforge-html; \
         cargo tree output:\n{stdout}"
    );
    assert!(
        stdout.contains("slideforge-plugin-api"),
        "AC-004: slideforge-pptx must depend on slideforge-plugin-api; \
         got:\n{stdout}"
    );
}

/// AC-004 (symmetry): `slideforge-pdf` must NOT list `slideforge-pptx` as a
/// transitive dependency.
#[test]
fn test_bc_5_02_002_ac004_pdf_has_no_cross_exporter_deps() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("AC-004: cannot locate workspace root");

    let output = std::process::Command::new("cargo")
        .args(["tree", "-p", "slideforge-pdf", "--no-dedupe"])
        .current_dir(workspace_root)
        .output()
        .unwrap_or_else(|e| panic!("AC-004 (PDF): cargo tree invocation failed: {e}"));

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("slideforge-pptx"),
        "AC-004: slideforge-pdf must NOT depend on slideforge-pptx; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("slideforge-html"),
        "AC-004: slideforge-pdf must NOT depend on slideforge-html; got:\n{stdout}"
    );
}

/// AC-005: the default registry assembles without error.
///
/// `build()` must not return `Err(BuildError::Registry(...))` with the fully-
/// assembled default registry.
///
/// Traceability: BC-5.02.002 postcondition 4.
#[test]
fn test_bc_5_02_002_ac005_default_registry_assembles_without_error() {
    let brand = BrandTmpDir::new("ac005_registry");
    let source = fixture_source("test-3slide.sf");
    let opts = brand.build_options("pptx", false);

    let result = slideforge::build(&source, &opts);
    if let Err(slideforge::error::BuildError::Registry(ref e)) = result {
        panic!(
            "AC-005: default_registry() failed to assemble — bundled plugins are incomplete; \
             error: {e:?}"
        );
    }
    // Any other outcome (Ok or non-Registry error) is acceptable for this assertion.
}
