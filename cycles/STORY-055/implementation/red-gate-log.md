# Red Gate Log — STORY-055: CLI build command + miette error rendering

**Date:** 2026-06-07
**Agent:** test-writer
**Commit SHA:** d8658dc3
**Branch:** feature/STORY-055

## Red Gate Result

PASS — all new behavioral tests fail against `todo!()` stubs.

```
Summary [   0.053s] 35 tests run: 7 passed, 28 failed, 0 skipped
```

## Test Files Written

| File | New Tests Added |
|------|----------------|
| `crates/slideforge-cli/src/commands/build.rs` (unit) | 9 |
| `crates/slideforge-cli/src/exit_code.rs` (unit) | 6 |
| `crates/slideforge-cli/tests/build_integration.rs` (integration) | 20 |
| **Total** | **35** |

## Failing Tests (28) — Red Gate Correct

All 28 behavioral tests fail with `not yet implemented` (panic from `todo!()`)
at either `run_build`, `exit_code_for_build_error`, `write_atomic`, or `init_tracing`.

| Test | BC/AC | Stub panicking |
|------|-------|---------------|
| `test_BC_1_15_003_exit_code_parse_error` | BC-1.15.003 PC-1 | `exit_code_for_build_error` |
| `test_BC_1_15_003_exit_code_eval_error_strict` | BC-1.15.003 PC-2 | `exit_code_for_build_error` |
| `test_BC_1_15_003_exit_code_validation_error_strict` | BC-1.15.003 PC-2 | `exit_code_for_build_error` |
| `test_BC_1_15_003_exit_code_export_error` | BC-1.15.003 PC-4 | `exit_code_for_build_error` |
| `test_BC_1_15_003_exit_code_parse_takes_precedence_over_eval` | BC-1.15.003 PC-6 | `exit_code_for_build_error` |
| `test_BC_1_15_003_run_build_nonexistent_source_exits_1_not_0` | AC-001/EC-002 | `run_build` |
| `test_BC_1_15_003_format_selection_pptx_only_does_not_write_other_formats` | AC-010 | `run_build` |
| `test_BC_1_15_003_invariant_build_error_parse_failed_maps_to_exit_1_via_build_module` | BC-1.15.003 | `exit_code_for_build_error` |
| `test_BC_1_15_003_build_success_exit_0_all_formats_written` | AC-001 | `run_build` |
| `test_BC_1_15_003_build_parse_error_exits_1_no_output_files` | AC-002 | `run_build` |
| `test_BC_1_15_003_build_eval_error_strict_exits_2_no_output` | AC-003 | `run_build` |
| `test_BC_1_15_003_build_eval_error_warn_only_exits_0_output_written` | AC-004 | `run_build` |
| `test_BC_1_15_003_build_export_error_exits_3_no_output` | AC-005 | `run_build` |
| `test_BC_1_15_002_build_multi_error_all_errors_reported` | AC-006 | `run_build` |
| `test_BC_1_15_002_build_errors_printed_in_source_order` | AC-007 | `run_build` |
| `test_BC_1_15_001_build_no_color_output_contains_no_ansi_escape_codes` | AC-009 | `run_build` |
| `test_BC_1_15_003_format_selection_pptx_only_writes_only_pptx` | AC-010 | `run_build` |
| `test_BC_1_15_003_format_selection_pptx_html_writes_only_those_two` | AC-010 | `run_build` |
| `test_BC_1_15_003_ac_011_all_6_tracing_spans_emitted_per_build` | AC-011 | `run_build` |
| `test_BC_1_15_003_ac_012_parse_error_exit_code_takes_precedence_over_eval` | AC-012 | `run_build` |
| `test_BC_1_15_003_ec_002_missing_source_file_exits_1` | EC-002 | `run_build` |
| `test_BC_1_15_003_ec_004_warn_only_does_not_demote_parse_errors` | EC-004 | `run_build` |
| `test_BC_1_15_003_ec_006_undefined_variant_exits_2` | EC-006 | `run_build` |
| `test_BC_1_15_001_output_writer_write_atomic_produces_file_content` | AC-009/output.rs | `write_atomic` |
| `test_BC_1_15_001_output_writer_write_atomic_cleans_up_tmp_on_failure` | AC-009/output.rs | `write_atomic` |
| `test_BC_1_15_001_every_diagnostic_has_correction_hint_in_rendered_output` | BC-1.15.001 PC-2 | `run_build` |
| `test_BC_1_15_002_invariant_error_count_matches_actual_independent_errors` | BC-1.15.002 INV-1 | `run_build` |
| `test_BC_1_15_003_ac_011_init_tracing_idempotent_no_panic_on_second_call` | AC-011/AC-015 | `init_tracing` |

## Passing Tests (7) — Pre-implemented Functions

These 7 tests exercise functions that were already implemented (not `todo!()`) in the
stub commit (511a6d5a). They validate the foundation the implementation will build on
and are not vacuously true — they assert real behavioral properties:

| Test | Function tested | Why pre-passing is correct |
|------|----------------|---------------------------|
| `test_BC_1_15_001_should_use_color_no_color_flag_suppresses_color` | `should_use_color` | Pure function, already implemented |
| `test_BC_1_15_001_should_use_color_json_flag_suppresses_color` | `should_use_color` | Pure function, already implemented |
| `test_BC_1_15_003_invariant_exit_severity_none_maps_to_zero` | `exit_code_for_severity` | Pure function, already implemented |
| `test_BC_1_15_003_invariant_exit_severity_warning_maps_to_zero` | `exit_code_for_severity` | Pure function, already implemented |
| `test_BC_1_15_003_invariant_exit_severity_error_maps_to_two` | `exit_code_for_severity` | Pure function, already implemented |
| `test_BC_1_15_003_invariant_exit_severity_fatal_maps_to_one` | `exit_code_for_severity` | Pure function, already implemented |
| `test_BC_1_15_003_invariant_export_exit_code_is_three` | Constants in `exit_code.rs` | Constants, already defined |

## AC Coverage Matrix

| AC | Test(s) | Status |
|----|---------|--------|
| AC-001 | `test_BC_1_15_003_build_success_exit_0_all_formats_written` | FAILING (Red Gate) |
| AC-002 | `test_BC_1_15_003_build_parse_error_exits_1_no_output_files` | FAILING |
| AC-003 | `test_BC_1_15_003_build_eval_error_strict_exits_2_no_output` | FAILING |
| AC-004 | `test_BC_1_15_003_build_eval_error_warn_only_exits_0_output_written` | FAILING |
| AC-005 | `test_BC_1_15_003_build_export_error_exits_3_no_output` | FAILING |
| AC-006 | `test_BC_1_15_002_build_multi_error_all_errors_reported` | FAILING |
| AC-007 | `test_BC_1_15_002_build_errors_printed_in_source_order` | FAILING |
| AC-008 | `test_BC_1_15_001_every_diagnostic_has_correction_hint_in_rendered_output` | FAILING |
| AC-009 | `test_BC_1_15_001_build_no_color_output_contains_no_ansi_escape_codes` + output writer tests | FAILING |
| AC-010 | `test_BC_1_15_003_format_selection_pptx_only_writes_only_pptx`, `_pptx_html_*` | FAILING |
| AC-011 | `test_BC_1_15_003_ac_011_all_6_tracing_spans_emitted_per_build`, `_init_tracing_idempotent_*` | FAILING |
| AC-012 | `test_BC_1_15_003_ac_012_parse_error_exit_code_takes_precedence_over_eval` | FAILING |
| AC-013 | Enforced by `#![forbid(unsafe_code)]` + clippy (no test needed) | N/A |
| AC-014 | Enforced by `cargo doc` gate (no test needed) | N/A |
| AC-015 | `test_BC_1_15_003_ac_015_otel_endpoint_flag_accepted_*` (otel feature gate) | FAILING (when otel feature active) |

## Edge Case Coverage

| EC | Test | Status |
|----|------|--------|
| EC-002 | `test_BC_1_15_003_ec_002_missing_source_file_exits_1` | FAILING |
| EC-004 | `test_BC_1_15_003_ec_004_warn_only_does_not_demote_parse_errors` | FAILING |
| EC-006 | `test_BC_1_15_003_ec_006_undefined_variant_exits_2` | FAILING |

## BC Clause Coverage

| Clause | Test Pattern | Count |
|--------|-------------|-------|
| BC-1.15.001 postconditions 1-5 | `test_BC_1_15_001_*` | 5+ |
| BC-1.15.002 postconditions 1-2, invariants 1-3 | `test_BC_1_15_002_*` | 4 |
| BC-1.15.003 postconditions 1-6, invariants 1-4 | `test_BC_1_15_003_*` | 19+ |

## Notes for Implementer

1. The 7 passing tests are on already-implemented pure functions. The implementer
   must NOT modify these functions in ways that make these tests regress.
2. `exit_code_for_build_error` needs to differentiate `ParseFailed` (→1),
   `EvalFailed`/`ValidationFailed` (→2), `Export` (→3). See the three-tier model
   in `exit_code.rs` module docs.
3. `run_build` must detect TTY via `std::io::IsTerminal` (not `atty`).
4. Output atomicity: write to `.tmp` then `fs::rename()`.
5. `init_tracing` must use `OnceLock` guard to be idempotent.
6. AC-015 (`otel` feature) test is gated with `#[cfg(feature = "otel")]`.
