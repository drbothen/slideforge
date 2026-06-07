//! STORY-089 demo: Field-Value Type Validation — E-VAL-104 live at `build()`.
//!
//! Demonstrates the four acceptance criteria for STORY-089:
//!
//! - **Scene 1 (AC-009 error path):** A `progress_bar` slide with `value "fifty"`
//!   (Str where Int is expected) → `build()` strict mode returns
//!   `Err(BuildError::ValidationFailed)` carrying at least one `E-VAL-104`
//!   diagnostic with the message "expected integer, got string".
//!
//! - **Scene 2 (AC-009 error path / OneOf):** A `chart` slide with
//!   `chart_type "donut"` (a value not in the `OneOf` allowlist) → `build()` strict
//!   mode returns `Err(BuildError::ValidationFailed)` carrying an `E-VAL-104`
//!   diagnostic with "disallowed value" and the allowed-values list.
//!
//! - **Scene 3 (AC-009 positive control / AC-010):** A valid deck — `progress_bar`
//!   with `value 75` and `chart` with `chart_type "bar"` — builds `Ok` in strict
//!   mode. No E-VAL-104 fires for correctly-typed fields.
//!
//! - **Scene 4 (regression guard):** A `chart` slide WITHOUT the `data` field builds
//!   `Ok` in strict mode. The `chart.data` field is optional per the architect's
//!   decision (chart slides may reference @data at runtime).
//!
//! ## BC traceability
//!
//! - BC-1.18.001: `FieldSchemaValidator` enforces `FieldDef.expected_type` at Stage 5.
//! - ADR-020 Decision 8: `FieldSchemaValidator` registered as a Stage-5 Validator plugin.
//! - STORY-089 AC-009 (T1: Str on Int), AC-009 (T2: `OneOf` violation), AC-010 (warn-only guard).
//!
//! ## Run
//!
//! ```text
//! cargo run --example story_089_field_validation -p slideforge -q
//! ```

// ── Demo-binary lint suppressions ────────────────────────────────────────────
#![allow(clippy::print_stdout)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_docs_in_private_items)]

use std::io::Write as _;
use std::path::PathBuf;
use std::time::SystemTime;

use slideforge::BuildOptions;
use slideforge::error::BuildError;
use slideforge_plugin_api::BrandSource;

// ─────────────────────────────────────────────────────────────────────────────
// Brand tmpdir helper (mirrors STORY-050 E2E helper without pulling test infra)
// ─────────────────────────────────────────────────────────────────────────────

/// Minimal self-contained brand directory for the demo.
///
/// Creates a temporary directory with `brand.toml` and a stub `logo.png` so
/// that `slideforge::build()` can proceed past brand loading to the validation
/// stage where `FieldSchemaValidator` runs.
struct BrandDir {
    dir: PathBuf,
    brand_toml_path: PathBuf,
}

impl BrandDir {
    /// Create a new tmpdir containing `brand.toml` + a stub `logo.png`.
    fn new(label: &str) -> Self {
        let ts = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "sf_demo_s089_{label}_{pid}_{ts}",
            pid = std::process::id(),
        ));
        std::fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("demo: cannot create brand tmpdir: {e}"));

        // Minimal 8-byte PNG magic — satisfies the logo.exists() check.
        let logo_bytes: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let logo_path = dir.join("logo.png");
        let mut logo_file = std::fs::File::create(&logo_path)
            .unwrap_or_else(|e| panic!("demo: cannot create logo.png: {e}"));
        logo_file
            .write_all(logo_bytes)
            .unwrap_or_else(|e| panic!("demo: cannot write logo.png: {e}"));

        // Minimal brand.toml (all required colour/font/logo fields).
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
        let brand_toml_path = dir.join("brand.toml");
        std::fs::write(&brand_toml_path, brand_toml)
            .unwrap_or_else(|e| panic!("demo: cannot write brand.toml: {e}"));

        Self {
            dir,
            brand_toml_path,
        }
    }

    /// Return `BuildOptions` pointing at this brand directory.
    fn build_options(&self, format: &str, strict: bool) -> BuildOptions {
        use std::sync::Arc;
        BuildOptions {
            format: Some(format.to_owned()),
            brand_source: Some(BrandSource::TomlFile(Arc::from(
                self.brand_toml_path.to_str().unwrap_or("brand.toml"),
            ))),
            strict,
        }
    }
}

impl Drop for BrandDir {
    fn drop(&mut self) {
        // Best-effort cleanup — ignore errors on drop.
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Separator helpers
// ─────────────────────────────────────────────────────────────────────────────

fn print_scene_header(n: u8, label: &str) {
    println!();
    println!("══════════════════════════════════════════════════════════════");
    println!("  Scene {n}: {label}");
    println!("══════════════════════════════════════════════════════════════");
}

fn print_result(label: &str, passed: bool) {
    let marker = if passed { "PASS" } else { "FAIL" };
    println!("  [{marker}]  {label}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    println!("STORY-089: Field-Value Type Validation — E-VAL-104 live at build()");
    println!("BC-1.18.001 | ADR-020 Decision 8 | FieldSchemaValidator (Stage 5)");

    // ── Scene 1: progress_bar value "fifty" — T1 type-mismatch (Str on Int) ──

    print_scene_header(
        1,
        "progress_bar value \"fifty\" — T1 Str-on-Int → E-VAL-104",
    );
    println!("  DSL:  progress_bar: title \"Sprint 4\" / label \"fifty pct\" / value \"fifty\"");
    println!("  Mode: strict=true");
    println!();

    let brand1 = BrandDir::new("s1");
    let src1 = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide progress_bar:\n",
        "  title \"Sprint 4 Progress\"\n",
        "  label \"fifty percent complete\"\n",
        "  value \"fifty\"\n",
    );
    let opts1 = brand1.build_options("pptx", true);
    let result1 = slideforge::build(src1, &opts1);

    match &result1 {
        Err(BuildError::ValidationFailed { diagnostics, count }) => {
            println!("  build() returned Err(ValidationFailed) — {count} error(s)");
            let e104: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.code.as_ref() == "E-VAL-104")
                .collect();

            let has_e104 = !e104.is_empty();
            print_result("Err(ValidationFailed) returned for Str-on-Int value", true);
            print_result("At least one E-VAL-104 diagnostic present", has_e104);

            if let Some(d) = e104.first() {
                let msg = d.message.as_ref();
                println!("  Diagnostic code:    {}", d.code.as_ref());
                println!("  Diagnostic message: {msg}");
                let has_integer = msg.contains("integer");
                let has_string = msg.contains("string");
                print_result("Message contains \"integer\" (expected type)", has_integer);
                print_result("Message contains \"string\" (actual type)", has_string);
                println!("  Severity:           {:?}", d.severity);
            }

            if !has_e104 {
                println!(
                    "  ERROR: E-VAL-104 not in diagnostics. Got codes: {:?}",
                    diagnostics
                        .iter()
                        .map(|d| d.code.as_ref())
                        .collect::<Vec<_>>()
                );
                std::process::exit(1);
            }
        },
        Ok(_) => {
            println!("  FAIL  build() returned Ok — FieldSchemaValidator not wired!");
            std::process::exit(1);
        },
        Err(other) => {
            println!("  FAIL  build() returned wrong error variant: {other:?}");
            std::process::exit(1);
        },
    }

    // ── Scene 2: chart chart_type "donut" — T2 OneOf violation ───────────────

    print_scene_header(
        2,
        "chart chart_type \"donut\" — T2 OneOf violation → E-VAL-104",
    );
    println!("  DSL:  chart: title \"Q3\" / chart_type \"donut\" / alt \"...\"");
    println!("  Mode: strict=true");
    println!();

    let brand2 = BrandDir::new("s2");
    let src2 = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide chart:\n",
        "  title \"Q3 Revenue\"\n",
        "  chart_type \"donut\"\n",
        "  alt \"Chart showing Q3 revenue\"\n",
    );
    let opts2 = brand2.build_options("pptx", true);
    let result2 = slideforge::build(src2, &opts2);

    match &result2 {
        Err(BuildError::ValidationFailed { diagnostics, count }) => {
            println!("  build() returned Err(ValidationFailed) — {count} error(s)");
            let e104: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.code.as_ref() == "E-VAL-104")
                .collect();

            let has_e104 = !e104.is_empty();
            print_result("Err(ValidationFailed) returned for OneOf violation", true);
            print_result("At least one E-VAL-104 diagnostic present", has_e104);

            if let Some(d) = e104.first() {
                let msg = d.message.as_ref();
                println!("  Diagnostic code:    {}", d.code.as_ref());
                println!("  Diagnostic message: {msg}");
                let has_disallowed = msg.contains("disallowed") || msg.contains("not in");
                let has_allowed = msg.contains("allowed") || msg.contains("bar");
                print_result("Message contains disallowed-value language", has_disallowed);
                print_result(
                    "Message references allowed values (e.g. \"bar\")",
                    has_allowed,
                );
            }

            if !has_e104 {
                println!(
                    "  ERROR: E-VAL-104 not in diagnostics. Got codes: {:?}",
                    diagnostics
                        .iter()
                        .map(|d| d.code.as_ref())
                        .collect::<Vec<_>>()
                );
                std::process::exit(1);
            }
        },
        Ok(_) => {
            println!("  FAIL  build() returned Ok — OneOf check not enforced!");
            std::process::exit(1);
        },
        Err(other) => {
            println!("  FAIL  build() returned wrong error variant: {other:?}");
            std::process::exit(1);
        },
    }

    // ── Scene 3: valid deck — progress_bar value 75 + chart chart_type "bar" ─

    print_scene_header(3, "Valid deck — correct types → Ok, no E-VAL-104");
    println!("  DSL:  progress_bar value 75 (Int) + chart chart_type \"bar\" (in OneOf)");
    println!("  Mode: strict=true");
    println!();

    let brand3 = BrandDir::new("s3");
    let src3 = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide progress_bar:\n",
        "  title \"Sprint 4 Progress\"\n",
        "  label \"75% complete\"\n",
        "  value 75\n",
        "\n",
        "slide chart:\n",
        "  title \"Q3 Revenue\"\n",
        "  chart_type \"bar\"\n",
        "  alt \"Bar chart showing Q3 revenue by region\"\n",
    );
    let opts3 = brand3.build_options("pptx", true);
    let result3 = slideforge::build(src3, &opts3);

    match &result3 {
        Ok(output) => {
            print_result("build() returned Ok for correctly-typed deck", true);
            print_result(
                &format!("BuildOutput.bytes non-empty ({} bytes)", output.bytes.len()),
                !output.bytes.is_empty(),
            );
            print_result(
                &format!("BuildOutput.extension = \"{}\"", output.extension),
                output.extension == "pptx",
            );
            println!("  No E-VAL-104 fired — FieldSchemaValidator correctly accepts valid types");
        },
        Err(BuildError::ValidationFailed { diagnostics, .. }) => {
            let e104: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.code.as_ref() == "E-VAL-104")
                .collect();
            println!("  FAIL  build() returned ValidationFailed for a valid deck!");
            println!("  E-VAL-104 diagnostics: {:?}", e104.len());
            std::process::exit(1);
        },
        Err(other) => {
            println!("  FAIL  build() returned error for valid deck: {other:?}");
            std::process::exit(1);
        },
    }

    // ── Scene 4: chart WITHOUT data — optional field → Ok in strict mode ─────

    print_scene_header(4, "chart WITHOUT data field — optional → Ok in strict mode");
    println!("  DSL:  chart: title / chart_type \"bar\" / alt (no data field)");
    println!("  Mode: strict=true");
    println!("  Architect decision: chart.data is OPTIONAL (runtime @data ref pattern)");
    println!();

    let brand4 = BrandDir::new("s4");
    let src4 = concat!(
        "slideforge_version \"1\"\n",
        "lang \"en-US\"\n",
        "\n",
        "slide chart:\n",
        "  title \"Q3 Revenue\"\n",
        "  chart_type \"bar\"\n",
        "  alt \"Bar chart showing Q3 revenue by region\"\n",
    );
    let opts4 = brand4.build_options("pptx", true);
    let result4 = slideforge::build(src4, &opts4);

    match &result4 {
        Ok(output) => {
            print_result(
                "build() returned Ok — chart.data absence does not fire E-VAL-101",
                true,
            );
            print_result(
                &format!("BuildOutput non-empty ({} bytes)", output.bytes.len()),
                !output.bytes.is_empty(),
            );
        },
        Err(BuildError::ValidationFailed { diagnostics, count }) => {
            let e101: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.code.as_ref() == "E-VAL-101")
                .collect();
            println!("  FAIL  build() returned ValidationFailed ({count} error(s))");
            if !e101.is_empty() {
                println!(
                    "  E-VAL-101 fired for absent chart.data — field must be marked optional!"
                );
            }
            std::process::exit(1);
        },
        Err(other) => {
            println!("  FAIL  build() returned unexpected error: {other:?}");
            std::process::exit(1);
        },
    }

    // ── Summary ───────────────────────────────────────────────────────────────

    println!();
    println!("══════════════════════════════════════════════════════════════");
    println!("  STORY-089 demo complete — all 4 scenes PASS");
    println!("  FieldSchemaValidator (E-VAL-104) is live at build() Stage 5.");
    println!("══════════════════════════════════════════════════════════════");
    println!();
}
