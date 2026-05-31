//! BC-4.03.002 AC-002 / AC-007 / AC-009: Forbidden dependency guard.
//!
//! These tests verify that `slideforge-pdf` does NOT depend on browser-based
//! or C-library PDF backends.
//!
//! ## What is tested
//!
//! - `Cargo.lock` contains no `chromium`, `headless-chrome`, `puppeteer-rs`,
//!   `wkhtmltopdf` entries (AC-002 / AC-009).
//! - `Cargo.lock` contains no `libharu`, `cairo`, `pango`, `freetype-sys`,
//!   `harfbuzz-sys` entries (AC-007).
//! - `crates/slideforge-pdf/Cargo.toml` does NOT list `pdf-writer` or
//!   `subsetter` as direct dependencies (RISK-1 / RISK-2: they arrive
//!   transitively via krilla 0.6.0 only).
//!
//! ## Test strategy (SID-1 compliance)
//!
//! These tests read static text files (Cargo.lock, Cargo.toml) without
//! spawning external processes, making them runnable without any OS-level
//! tooling. They pass immediately after crate scaffolding and serve as the
//! CI-equivalent guard for dep hygiene.

use std::path::PathBuf;

/// Path to the workspace root Cargo.lock.
///
/// The test file lives at `crates/slideforge-pdf/tests/no_forbidden_deps.rs`.
/// `CARGO_MANIFEST_DIR` = `.../crates/slideforge-pdf`.
/// The workspace root is 2 levels up: `../..`.
fn cargo_lock_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .join("../..")
        .canonicalize()
        .expect("workspace root must be resolvable from CARGO_MANIFEST_DIR");
    let lock_path = workspace_root.join("Cargo.lock");
    assert!(
        lock_path.exists(),
        "Cargo.lock not found at {path}. Run `cargo build --workspace` first.",
        path = lock_path.display()
    );
    lock_path
}

/// Path to the slideforge-pdf Cargo.toml.
fn pdf_cargo_toml_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("Cargo.toml")
        .canonicalize()
        .expect("Cargo.toml must exist in slideforge-pdf crate")
}

/// Read a text file to string, panicking with a clear message on failure.
fn read_file(path: &PathBuf) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

/// BC-4.03.002 AC-002 / AC-009: Cargo.lock must contain no browser-based
/// PDF crate names.
///
/// Forbidden crates: `chromium`, `headless-chrome`, `puppeteer-rs`, `wkhtmltopdf`.
///
/// PASSES NOW — krilla is pure Rust and has no browser dependencies.
#[test]
fn test_bc_4_03_002_no_browser_pdf_deps_in_cargo_lock() {
    let lock_path = cargo_lock_path();
    let lock_text = read_file(&lock_path);

    let forbidden = [
        "name = \"chromium\"",
        "name = \"headless-chrome\"",
        "name = \"puppeteer-rs\"",
        "name = \"wkhtmltopdf\"",
    ];

    for entry in &forbidden {
        assert!(
            !lock_text.contains(entry),
            "Cargo.lock must NOT contain browser-based PDF dep: {entry}\n\
             Cargo.lock path: {path}\n\
             This violates BC-4.03.002 invariant 1 (no Chrome/headless browser).",
            path = lock_path.display(),
        );
    }
}

/// BC-4.03.002 AC-007: Cargo.lock must contain no FFI bindings to C PDF libs.
///
/// Forbidden: `libharu`, `cairo` (cairo-rs), `pango`, `freetype-sys`,
/// `harfbuzz-sys`. These would indicate a non-pure-Rust PDF stack.
///
/// PASSES NOW — krilla and pdf-writer are fully pure Rust.
#[test]
fn test_bc_4_03_002_no_ffi_pdf_deps_in_cargo_lock() {
    let lock_path = cargo_lock_path();
    let lock_text = read_file(&lock_path);

    let forbidden = [
        "name = \"libharu\"",
        "name = \"cairo-rs\"",
        "name = \"pango-sys\"",
        "name = \"freetype-sys\"",
        "name = \"harfbuzz-sys\"",
    ];

    for entry in &forbidden {
        assert!(
            !lock_text.contains(entry),
            "Cargo.lock must NOT contain C-library PDF FFI dep: {entry}\n\
             Cargo.lock path: {path}\n\
             This violates BC-4.03.002 invariant 4 (no FFI to C PDF libs).",
            path = lock_path.display(),
        );
    }
}

/// RISK-1 guard: `crates/slideforge-pdf/Cargo.toml` must NOT list `pdf-writer`
/// as a direct dependency. It arrives transitively via krilla 0.6.0.
///
/// Adding a direct `pdf-writer` dep risks version splits and conflicts with
/// krilla's internal `StructTreeRoot` generation (tech-validation RISK-1).
///
/// PASSES NOW — Cargo.toml has no `pdf-writer` entry in `[dependencies]`.
#[test]
fn test_bc_4_03_002_no_direct_pdf_writer_dep() {
    let toml_path = pdf_cargo_toml_path();
    let toml_text = read_file(&toml_path);

    // Check the [dependencies] section (non-comment lines) does not directly
    // list pdf-writer. We filter out comment lines to avoid false positives
    // from inline comments referencing the transitive dep.
    let dep_lines: Vec<&str> = toml_text
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect();
    let dep_text = dep_lines.join("\n");

    assert!(
        !dep_text.contains("pdf-writer"),
        "crates/slideforge-pdf/Cargo.toml must NOT have a direct pdf-writer dependency.\n\
         pdf-writer arrives transitively via krilla =0.6.0 (tech-validation RISK-1).\n\
         Cargo.toml path: {path}",
        path = toml_path.display(),
    );
}

/// RISK-2 guard: `crates/slideforge-pdf/Cargo.toml` must NOT list `subsetter`
/// as a direct dependency. It arrives transitively via krilla 0.6.0.
///
/// Adding a direct `subsetter` dep is unnecessary for the normal text-drawing
/// path (krilla subsets internally) and risks version splits (RISK-2).
///
/// PASSES NOW — Cargo.toml has no `subsetter` entry in `[dependencies]`.
#[test]
fn test_bc_4_03_002_no_direct_subsetter_dep() {
    let toml_path = pdf_cargo_toml_path();
    let toml_text = read_file(&toml_path);

    // Same comment-filtering as the pdf-writer guard above.
    let dep_lines: Vec<&str> = toml_text
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect();
    let dep_text = dep_lines.join("\n");

    assert!(
        !dep_text.contains("subsetter"),
        "crates/slideforge-pdf/Cargo.toml must NOT have a direct subsetter dependency.\n\
         subsetter arrives transitively via krilla =0.6.0 (tech-validation RISK-2).\n\
         Cargo.toml path: {path}",
        path = toml_path.display(),
    );
}

/// BC-4.03.002 AC-009: Cargo.lock contains krilla =0.6.0 as expected.
///
/// This is a positive guard: if krilla is missing, the build is broken.
///
/// PASSES NOW after workspace build updates Cargo.lock.
/// Note: This test may not pass until after the first `cargo build -p slideforge-pdf`
/// run that downloads krilla.
#[test]
fn test_bc_4_03_002_krilla_present_in_cargo_lock() {
    let lock_path = cargo_lock_path();
    let lock_text = read_file(&lock_path);

    assert!(
        lock_text.contains("name = \"krilla\""),
        "Cargo.lock must contain an entry for krilla after workspace build.\n\
         Run `cargo build -p slideforge-pdf` first to populate Cargo.lock.\n\
         Cargo.lock path: {path}",
        path = lock_path.display(),
    );
}
