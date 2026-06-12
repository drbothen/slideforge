//! STORY-050 End-to-End Integration Test Suite.
//!
//! This is the single integration test binary for STORY-050. All E2E test
//! sub-modules are declared here so they share the `e2e` helper module.
//!
//! ## Module structure
//!
//! ```text
//! tests/
//!   e2e_tests.rs          ← this file (test binary root)
//!   e2e/
//!     mod.rs              ← shared helpers (BrandTmpDir, fixture_source, ...)
//!     pipeline_pptx.rs    ← AC-001, AC-008 (PPTX)
//!     pipeline_docx.rs    ← AC-002, AC-008 (DOCX)
//!     pipeline_pdf.rs     ← AC-003, AC-008 (PDF)
//!     registry.rs         ← AC-004, AC-005
//!     error_propagation.rs← AC-006, AC-009, EC-001..EC-004
//!     observability.rs    ← AC-007
//!     multi_format.rs     ← AC-008 cross-format, EC-005
//! ```
//!
//! ## Architecture compliance (STORY-050)
//!
//! All test modules call `slideforge::build()` only — no imports from
//! `slideforge_pptx`, `slideforge_docx`, `slideforge_pdf`, or `slideforge_html`.
//!
//! ## Traceability
//!
//! - BC-5.02.001, BC-5.02.002
//! - STORY-050 AC-001 through AC-009
//! - STORY-050 EC-001 through EC-005

// Shared helpers — available to all sub-modules as `e2e::*`.
mod e2e;

// Sub-module declarations (each file lives at tests/e2e/<name>.rs).
#[path = "e2e/pipeline_pptx.rs"]
mod e2e_pipeline_pptx;

#[path = "e2e/pipeline_docx.rs"]
mod e2e_pipeline_docx;

#[path = "e2e/pipeline_pdf.rs"]
mod e2e_pipeline_pdf;

#[path = "e2e/registry.rs"]
mod e2e_registry;

#[path = "e2e/error_propagation.rs"]
mod e2e_error_propagation;

#[path = "e2e/observability.rs"]
mod e2e_observability;

#[path = "e2e/multi_format.rs"]
mod e2e_multi_format;

// STORY-086 — Stage 2b content-threading Red Gate tests
// AC-001 through AC-007, AC-018 (positive content vectors + alt-text discrimination).
#[path = "e2e/story_086_content_threading.rs"]
mod e2e_story_086_content_threading;

// STORY-087 — F-087-P1-001 value-range enforcement Red Gate tests
// Proves ValueRangeValidator is wired at Stage 5, reachable from build().
// progress_bar value=101 and value=-1 FAIL at Red Gate (no validator yet).
// weighted_composite tests are #[ignore]'d pending STORY-088 DSL list-literal support.
#[path = "e2e/story_087_value_range.rs"]
mod e2e_story_087_value_range;

// STORY-087 pass-2 — AC-002/008/015 content rendering Red Gate tests (§10.4).
// Tests that label text and ColorBar frames are VISIBLE in the LaidOutDeck output.
// Programmatic (SID-1) path: directly constructs Slide.blocks bypassing the parser.
// RED GATE: layout::run has no ColorLabel arm or ColorBar materialization pass yet.
#[path = "e2e/story_087_content_rendering.rs"]
mod e2e_story_087_content_rendering;

// STORY-087 pass-3 — BC-1.17.002 PC-9 bar-render Red Gate tests for PDF, HTML, DOCX.
// These tests prove the progress_bar bar is rendered visibly in ALL output formats
// (PC-9 requires ALL exporters, not just PPTX). HTML exporter not yet registered;
// PDF uses export_uncompressed boundary test; DOCX uses build()-level test.
// RED GATE: PDF ColorBar arm is tracing::debug! no-op; DOCX has no ColorBar arm.
#[path = "e2e/story_087_bar_export.rs"]
mod e2e_story_087_bar_export;

// Wave 4 Gate-5 — BC-1.16.001 PC-10 image-slide src+alt Red Gate tests.
// AC-GATE5-001: image+src+alt → build Ok + PPTX descr (passes pre-fix: threading already reads "src").
// AC-GATE5-002: image+image_keyword+alt → Err(E-A11-001) always (REVISED: canonical keyword is `src`, not `image`).
// AC-GATE5-003: image+src+no-alt → Err(E-A11-001) always (WCAG regression guard).
// AC-GATE5-004: screenshot+src+alt → build Ok + PPTX descr.
// AC-GATE5-005: known_fields("image"/"screenshot"/"bio") must contain "src" not "image"
//               (TRUE RED GATE unit test — currently FAILS: known_fields has "image").
#[path = "e2e/wave4_gate5_image_alt.rs"]
mod e2e_wave4_gate5_image_alt;

// STORY-089 — BC-1.18.001 Field-Schema Validator Pipeline Wiring tests.
// AC-009 (strict): progress_bar with value "fifty" (Str on Int field) → Err(ValidationFailed)
//   with E-VAL-104. Was RED at Red Gate (no FieldSchemaValidator registered → Ok returned);
//   now GREEN: FieldSchemaValidator is registered and strict build returns Err(E-VAL-104).
// AC-010 (warn-only): same fixture → Ok (guard test: documents post-wiring contract).
// Positive control: progress_bar value=75 (valid Int) → strict Ok (false-positive guard).
// Regression intent: chart without `data` → strict Ok (chart.data → optional per architect).
#[path = "e2e/story_089_field_schema.rs"]
mod e2e_story_089_field_schema;

// STORY-088 — BC-1.01.002 bullets list-literal DSL syntax E2E tests.
// AC-007 (direct list-literal): `bullets: ["Item A", "Item B", "Item C"]` produces
//   ≥3 <a:r> text runs in PPTX output (no @var binding — direct field literal).
// RED GATE: value_parser() has no [...]  arm → build() returns Err(ParseFailed).
#[path = "e2e/story_088_bullets_list_literal.rs"]
mod e2e_story_088_bullets_list_literal;

// STORY-081 — Slide-level inline markup end-to-end keystone test (C4 adversary finding).
// Builds a deck from DSL source with **bold**, _italic_, `code`, ~~del~~, ==highlight==,
// ^sup^, ~sub~ in the body field. Asserts FORMAT-NATIVE markup in each output (PPTX b="1",
// DOCX <w:b/>, HTML <strong>, PDF "bold text" in bytes) and absence of literal ** characters.
// RED GATE: C2 (body Inlines dropped) + C3 (PPTX Body flattens) block ALL format assertions.
#[path = "e2e/story_081_inline_markup_e2e.rs"]
mod e2e_story_081_inline_markup;

// STORY-098 — F-098-P1-004 + PO adjudication F-098-ADJ-BODY-CONTENT exit-code E2E tests.
// AC-001: body on non-declaring type (chart), strict → Err(ValidationFailed) + W-VAL-103 as Error.
// AC-001 warn-only: body on non-declaring type (chart), warn-only → Ok (W-VAL-103 non-blocking).
// AC-002: body on content slide (schema-VALID), strict → Ok + LaidOutDeck contains Body frame.
// AC-001 regression guard: body on quote slide (non-declaring), strict → Err + W-VAL-103.
// AC-004: chart with no data, strict → Err(ValidationFailed) + E-LAY-003.
// AC-005: chart with no data, warn-only → Ok (E-LAY-003 non-blocking).
#[path = "e2e/story_098_exit_codes.rs"]
mod e2e_story_098_exit_codes;
