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
