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

use std::path::{Path, PathBuf};

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

/// Recursively scan `.rs` files under `dir`, collecting lines that contain any
/// `forbidden` pattern and are not pure comment lines (trimmed start != `//`).
///
/// Returns the number of `.rs` files actually scanned. The caller must assert
/// that this count is `>= N` to guard against a silent no-op when the source
/// tree is relocated or renamed (positive-coverage guard, F-P6-002).
fn scan_dir_for_patterns(dir: &Path, forbidden: &[&str], violations: &mut Vec<String>) -> usize {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read dir {}: {e}", dir.display()));
    let mut files_scanned: usize = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files_scanned += scan_dir_for_patterns(&path, forbidden, violations);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            files_scanned += 1;
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            for (lineno, line) in content.lines().enumerate() {
                // Skip lines whose trimmed content starts with `//` (comments).
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") {
                    continue;
                }
                for pat in forbidden {
                    if line.contains(pat) {
                        violations.push(format!(
                            "{}:{}: {}",
                            path.display(),
                            lineno + 1,
                            line.trim()
                        ));
                    }
                }
            }
        }
    }
    files_scanned
}

/// BC-4.03.002 AC-008 (source-level no-subprocess check): No usage of
/// `std::process`, `Command::new`, or `process::Command` in any non-comment
/// line of `crates/slideforge-pdf/src/`.
///
/// This is the load-bearing Rust-level assertion for AC-008 (scope-directive
/// Decision 3, F-006 fix). The companion shell assertion is in
/// `scripts/check-pdf-deps.sh`.
///
/// Full strace/dtrace integration test deferred to STORY-049:
///   test name: `test_e2e_pdf_export_no_execve_syscall`
///   Reason: requires full CLI binary + syscall tracer.
#[test]
fn test_bc_4_03_002_no_subprocess_in_pdf_source() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    assert!(src_dir.is_dir(), "src/ directory must exist");

    // Subprocess API patterns forbidden in production code.
    let forbidden_patterns = ["std::process", "Command::new", "process::Command"];

    // Recursively scan all .rs files in src/.
    let mut violations: Vec<String> = Vec::new();
    let files_scanned = scan_dir_for_patterns(&src_dir, &forbidden_patterns, &mut violations);

    // Positive-coverage guard (F-P6-002): the scan must actually visit files.
    // The crate has at least lib.rs / exporter.rs / tag_engine.rs / font.rs /
    // svg_embed.rs / error.rs. If this assertion fires, the src/ tree was
    // relocated or the scan logic regressed into a silent no-op.
    assert!(
        files_scanned >= 5,
        "positive-coverage guard: expected to scan >=5 .rs files in slideforge-pdf/src/, \
         but only scanned {files_scanned}. The scan may have silently no-op'd due to a \
         src/ relocation or rename (F-P6-002)."
    );

    assert!(
        violations.is_empty(),
        "BC-4.03.002 AC-008: found subprocess usage in slideforge-pdf/src/:\n{}\n\
         \nAll PDF generation must be pure Rust — no std::process spawning.",
        violations.join("\n")
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
