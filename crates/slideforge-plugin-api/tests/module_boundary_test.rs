//! Module-boundary fitness function for `inline_formats/`.
//!
//! Architect-specified fitness function (F-008, STORY-085): the `inline_formats/`
//! sub-module is documented to import ONLY from this crate's own public API and
//! `slideforge-types`. Because `slideforge-plugin-api` already links
//! `slideforge-layout` (required by `slide_types/` and `traits/exporter.rs`),
//! a forbidden import inside `inline_formats/` would compile silently — the linker
//! would not catch it. This test makes the boundary machine-enforceable.
//!
//! ## What is checked
//!
//! Every `*.rs` file under `src/inline_formats/` is scanned for lines whose trimmed
//! form begins with a forbidden `use` prefix:
//!
//! - `use slideforge_layout`
//! - `use slideforge_eval`
//! - `use slideforge_syntax`
//! - `use slideforge_pptx`
//! - `use slideforge_docx`
//! - `use slideforge_pdf`
//! - `use slideforge_html`
//!
//! If any such line is found the test fails, printing `<file>:<line_number>: <line>`
//! for each violation so the author knows exactly what to fix.

use std::fs;
use std::path::Path;

/// Crates that `inline_formats/` must never import.
const FORBIDDEN_PREFIXES: &[&str] = &[
    "use slideforge_layout",
    "use slideforge_eval",
    "use slideforge_syntax",
    "use slideforge_pptx",
    "use slideforge_docx",
    "use slideforge_pdf",
    "use slideforge_html",
];

/// Scan a single source file for forbidden imports.
///
/// Returns a list of `"<path>:<line_number>: <trimmed_line>"` strings for every
/// violation found.  An empty `Vec` means the file is clean.
fn scan_file_for_violations(path: &Path) -> Vec<String> {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    // Closure 1: decide whether a single source line is a forbidden import.
    let is_forbidden = |line: &str| {
        let trimmed = line.trim();
        FORBIDDEN_PREFIXES
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
    };

    // Closure 2: format a violation into a human-readable string.
    let format_violation = |line_number: usize, line: &str| {
        format!("{}:{}: {}", path.display(), line_number, line.trim())
    };

    source
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            if is_forbidden(line) {
                Some(format_violation(idx + 1, line))
            } else {
                None
            }
        })
        .collect()
}

/// Collect every `*.rs` path under a directory tree (non-recursive sub-dirs
/// are also walked to handle nested modules).
fn collect_rs_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("failed to read dir {}: {e}", dir.display()));

    for entry in entries {
        let entry = entry
            .unwrap_or_else(|e| panic!("failed to read dir entry under {}: {e}", dir.display()));
        let path = entry.path();

        if path.is_dir() {
            paths.extend(collect_rs_files(&path));
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            paths.push(path);
        }
    }

    paths
}

/// F-008: `inline_formats/` module boundary fitness function.
///
/// Asserts that no source file under `src/inline_formats/` imports any of the
/// forbidden downstream crates listed in [`FORBIDDEN_PREFIXES`].
///
/// This test PASSES when no violations exist (current state) and FAILS immediately
/// if a forbidden import is introduced, printing every offending `file:line: import`
/// so the author can locate and remove it.
#[test]
fn inline_formats_module_boundary() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let inline_formats_dir = manifest_dir.join("src").join("inline_formats");

    assert!(
        inline_formats_dir.exists(),
        "expected src/inline_formats/ to exist at {}",
        inline_formats_dir.display()
    );

    let rs_files = collect_rs_files(&inline_formats_dir);

    assert!(
        !rs_files.is_empty(),
        "no *.rs files found under {}; directory may have moved",
        inline_formats_dir.display()
    );

    let violations: Vec<String> = rs_files
        .iter()
        .flat_map(|path| scan_file_for_violations(path))
        .collect();

    assert!(
        violations.is_empty(),
        "inline_formats/ module boundary violated — forbidden imports found:\n{}",
        violations.join("\n")
    );
}
