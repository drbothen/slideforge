//! Core unit tests for `slideforge-pptx` (BC-4.01.001 AC-002 through AC-010).
//!
//! Tests are written in Step 3 (test-writer pass). This file is a skeleton
//! module only — no test functions are present yet. The test-writer will
//! populate this file with failing tests before implementation begins.
//!
//! ## Planned test coverage
//!
//! | Test name (planned) | AC | BC clause |
//! |---------------------|----|-----------|
//! | `test_BC_4_01_001_zip_contains_all_required_parts` | AC-002 | postcondition 2 |
//! | `test_BC_4_01_001_content_types_snapshot_3_slides` | AC-003 | postcondition 7 |
//! | `test_BC_4_01_001_placeholder_inheritance_chain` | AC-004 | postcondition 3 |
//! | `test_BC_4_01_001_all_coordinates_integer_i64` | AC-005 | precondition 5 |
//! | `test_BC_4_01_001_slide_xml_element_order_snapshot` | AC-006 | postcondition 2 |
//! | `test_BC_4_01_001_deterministic_output_sha256` | AC-007 | invariant 4 |
//! | `test_BC_4_01_001_relationship_chain_completeness` | AC-009 | postcondition 7 |
//! | `test_BC_4_01_001_report_detail_absent_from_slides` | AC-010 | invariant 1 |
