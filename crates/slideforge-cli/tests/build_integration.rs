//! Integration tests for `slideforge build` (STORY-055).
//!
//! These tests are written in the Red Gate phase (failing tests before
//! implementation) per the TDD workflow.  All tests in this file are
//! EXPECTED TO FAIL until the implementation phase (STORY-055 implementer).
//!
//! Tests exercise:
//! - AC-001: successful build exits 0 and writes dist/ artifacts
//! - AC-002: parse error exits 1, no output files
//! - AC-003: validation error in strict mode exits 2, no output files
//! - AC-004: validation error with --warn-only exits 0, output with placeholders
//! - AC-005: export failure exits 3, no output files
//! - AC-006: multiple errors all reported in single pass
//! - AC-007: errors printed in source-file order (file, line, col)
//! - AC-009: no ANSI codes in non-TTY output
//! - AC-011: six tracing spans emitted (parse, evaluate, brand, validate, layout, export)
//!
//! This file will be populated by the test-writer agent in the next phase.
