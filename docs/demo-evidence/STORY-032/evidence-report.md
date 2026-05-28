---
document_type: demo-evidence-report
product: "slideforge"
story_id: "STORY-032"
title: "Chart: Empty Data Error-Slide Placeholder"
pipeline_run: "2026-05-28"
demo_type: "library"
recording_tool: "cargo-nextest"
status: complete
---

# Demo Evidence Report — STORY-032

## Product: slideforge
## Story: STORY-032 — Chart: Empty Data Error-Slide Placeholder
## BC: BC-1.11.002
## Pipeline Run: 2026-05-28
## Demo Type: library (unit tests — no CLI binary or web UI in scope)

---

## Summary

STORY-032 is a pure library story (no CLI entry point). Demo evidence is captured
as `cargo nextest` test output. All 31 BC-1.11.002 tests pass in the
`slideforge-charts` crate. Each AC below maps to the specific tests that prove it.

**Overall result: 31/31 tests PASS.**

```
cargo nextest run -p slideforge-charts -E 'test(/bc_1_11_002/)'

────────────
 Nextest run ID 8e171f28-fd60-4ec6-b77b-4cb7f09ba720
    Starting 31 tests across 1 binary (115 tests skipped)
        PASS [   0.011s] ( 1/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_svg_has_800x450_viewbox
        PASS [   0.011s] ( 2/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_svg_has_root_element
        PASS [   0.011s] ( 3/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_svg_has_aria_label
        PASS [   0.011s] ( 4/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_svg_has_explicit_width_height
        PASS [   0.011s] ( 5/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_svg_contains_message
        PASS [   0.011s] ( 6/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_svg_has_title_element
        PASS [   0.011s] ( 7/31) slideforge-charts tests::test_bc_1_11_002_guard_blocks_renderer_dispatch_strict
        PASS [   0.012s] ( 8/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_title_element_escapes_slide_title
        PASS [   0.012s] ( 9/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_svg_no_script
        PASS [   0.013s] (10/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_svg_no_foreign_object
        PASS [   0.013s] (11/31) slideforge-charts tests::test_bc_1_11_002_placeholder_constructible_from_empty_data_error
        PASS [   0.013s] (12/31) slideforge-charts types::tests::test_bc_1_11_002_chart_error_empty_data_displays_e_lay_003
        PASS [   0.014s] (13/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_xml_escapes_user_input
        PASS [   0.015s] (14/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_svg_contains_error_code
        PASS [   0.017s] (15/31) slideforge-charts placeholder::tests::test_bc_1_11_002_placeholder_svg_nonempty
        PASS [   0.017s] (16/31) slideforge-charts tests::test_bc_1_11_002_multi_slide_one_empty_two_valid
        PASS [   0.009s] (17/31) slideforge-charts validation::tests::test_bc_1_11_002_data_is_not_empty_for_nonempty_map
        PASS [   0.009s] (18/31) slideforge-charts validation::tests::test_bc_1_11_002_data_is_not_empty_for_non_list_value
        PASS [   0.010s] (19/31) slideforge-charts validation::tests::test_bc_1_11_002_data_is_empty_for_empty_list
        PASS [   0.010s] (20/31) slideforge-charts types::tests::test_bc_1_11_002_chart_error_empty_data_variant_exists
        PASS [   0.009s] (21/31) slideforge-charts validation::tests::test_bc_1_11_002_diagnostic_hint_includes_expression
        PASS [   0.010s] (22/31) slideforge-charts validation::tests::test_bc_1_11_002_data_is_not_empty_for_outer_list_with_inner_empty_list
        PASS [   0.010s] (23/31) slideforge-charts validation::tests::test_bc_1_11_002_diagnostic_hint_includes_remediation
        PASS [   0.010s] (24/31) slideforge-charts validation::tests::test_bc_1_11_002_data_is_not_empty_for_single_element_list
        PASS [   0.011s] (25/31) slideforge-charts validation::tests::test_bc_1_11_002_data_is_not_empty_for_multi_element_list
        PASS [   0.009s] (26/31) slideforge-charts validation::tests::test_bc_1_11_002_diagnostic_message_contains_slide_title
        PASS [   0.010s] (27/31) slideforge-charts validation::tests::test_bc_1_11_002_diagnostic_is_error_severity_by_default
        PASS [   0.012s] (28/31) slideforge-charts validation::tests::test_bc_1_11_002_data_is_empty_for_empty_map
        PASS [   0.010s] (29/31) slideforge-charts validation::tests::test_bc_1_11_002_diagnostic_message_contains_data_expression
        PASS [   0.012s] (30/31) slideforge-charts validation::tests::test_bc_1_11_002_diagnostic_preserves_span
        PASS [   0.009s] (31/31) slideforge-charts validation::tests::test_bc_1_11_002_diagnostic_uses_e_lay_003_code
────────────
     Summary [   0.027s] 31 tests run: 31 passed, 115 skipped
```

---

## Per-AC Demo Evidence

| AC | Description | Test(s) | Module | Result |
|----|-------------|---------|--------|--------|
| AC-001 | E-LAY-003 emitted on empty data | `test_bc_1_11_002_diagnostic_uses_e_lay_003_code` | `validation::tests` | PASS |
| AC-002 | ChartRenderer never called with empty data | `test_bc_1_11_002_guard_blocks_renderer_dispatch_strict` | `tests` (lib.rs) | PASS |
| AC-003 | Strict mode: diagnostic is Error severity | `test_bc_1_11_002_diagnostic_is_error_severity_by_default` | `validation::tests` | PASS |
| AC-004 | Placeholder SVG well-formed | `test_bc_1_11_002_placeholder_svg_*` (10 tests) | `placeholder::tests` | PASS (10/10) |
| AC-005 | Multi-slide: one empty, two valid — accumulation | `test_bc_1_11_002_multi_slide_one_empty_two_valid` | `tests` (lib.rs) | PASS |

---

### AC-001: E-LAY-003 emitted on empty data

**Test:** `validation::tests::test_bc_1_11_002_diagnostic_uses_e_lay_003_code`
**Source:** `crates/slideforge-charts/src/validation.rs`
**Traces to:** BC-1.11.002 postcondition 1

```
cargo nextest run -p slideforge-charts \
  -E 'test(test_bc_1_11_002_diagnostic_uses_e_lay_003_code)'

    Starting 1 test across 1 binary (145 tests skipped)
        PASS [   0.008s] (1/1) slideforge-charts validation::tests::test_bc_1_11_002_diagnostic_uses_e_lay_003_code
     Summary [   0.008s] 1 test run: 1 passed, 145 skipped
```

**What this proves:** `build_empty_data_diagnostic()` sets `code = "E-LAY-003"` exactly.
No magic string drift — the constant `E_LAY_003` is tested directly.

Also covered by supporting diagnostic tests:
- `test_bc_1_11_002_diagnostic_message_contains_slide_title` — message includes slide name
- `test_bc_1_11_002_diagnostic_message_contains_data_expression` — message includes binding expr
- `test_bc_1_11_002_diagnostic_hint_includes_remediation` — hint is `Some(...)` not `None`
- `test_bc_1_11_002_diagnostic_hint_includes_expression` — hint restates the data expression
- `test_bc_1_11_002_diagnostic_preserves_span` — span is threaded through without mutation

---

### AC-002: ChartRenderer never called with empty data

**Test:** `tests::test_bc_1_11_002_guard_blocks_renderer_dispatch_strict`
**Source:** `crates/slideforge-charts/src/lib.rs`
**Traces to:** BC-1.11.002 invariant 2

```
cargo nextest run -p slideforge-charts \
  -E 'test(test_bc_1_11_002_guard_blocks_renderer_dispatch_strict)'

    Starting 1 test across 1 binary (145 tests skipped)
        PASS [   0.008s] (1/1) slideforge-charts tests::test_bc_1_11_002_guard_blocks_renderer_dispatch_strict
     Summary [   0.009s] 1 test run: 1 passed, 145 skipped
```

**What this proves:** When `ChartSpec.data` is an empty `Value::List([])`, the
`render_chart_slide()` dispatch function returns `ChartError::EmptyData` before
ever constructing a `DataSeries` slice or calling `ChartRendererImpl::render()`.
A `SpyRenderer` mock with a call-counter confirms zero invocations on the
empty-data path.

---

### AC-003: Strict mode exits with Error severity (no output)

**Test:** `validation::tests::test_bc_1_11_002_diagnostic_is_error_severity_by_default`
**Source:** `crates/slideforge-charts/src/validation.rs`
**Traces to:** BC-1.11.002 postcondition 2

```
cargo nextest run -p slideforge-charts \
  -E 'test(test_bc_1_11_002_diagnostic_is_error_severity_by_default)'

    Starting 1 test across 1 binary (145 tests skipped)
        PASS [   0.007s] (1/1) slideforge-charts validation::tests::test_bc_1_11_002_diagnostic_is_error_severity_by_default
     Summary [   0.008s] 1 test run: 1 passed, 145 skipped
```

**What this proves:** `build_empty_data_diagnostic()` returns a `Diagnostic` with
`severity = Severity::Error`. A strict-mode `DiagnosticSink` accumulating an
`Error`-severity diagnostic causes the build pipeline to exit with code 2 and
produce no output file (behavior established in STORY-016 / BC-3.03.002 — the
charts crate feeds the same `DiagnosticSink` used by the validate stage).

---

### AC-004: Warn-only mode produces error-slide placeholder SVG

**Tests:** 10 tests in `placeholder::tests`
**Source:** `crates/slideforge-charts/src/placeholder.rs`
**Traces to:** BC-1.11.002 postcondition 3

```
cargo nextest run -p slideforge-charts \
  -E 'test(test_bc_1_11_002_placeholder_svg)'

    Starting 10 tests across 1 binary (136 tests skipped)
        PASS [   0.008s] ( 1/10) placeholder::tests::test_bc_1_11_002_placeholder_svg_contains_error_code
        PASS [   0.008s] ( 2/10) placeholder::tests::test_bc_1_11_002_placeholder_svg_has_title_element
        PASS [   0.008s] ( 3/10) placeholder::tests::test_bc_1_11_002_placeholder_svg_has_aria_label
        PASS [   0.008s] ( 4/10) placeholder::tests::test_bc_1_11_002_placeholder_svg_nonempty
        PASS [   0.008s] ( 5/10) placeholder::tests::test_bc_1_11_002_placeholder_svg_no_script
        PASS [   0.008s] ( 6/10) placeholder::tests::test_bc_1_11_002_placeholder_svg_contains_message
        PASS [   0.008s] ( 7/10) placeholder::tests::test_bc_1_11_002_placeholder_svg_has_800x450_viewbox
        PASS [   0.008s] ( 8/10) placeholder::tests::test_bc_1_11_002_placeholder_svg_no_foreign_object
        PASS [   0.008s] ( 9/10) placeholder::tests::test_bc_1_11_002_placeholder_svg_has_root_element
        PASS [   0.008s] (10/10) placeholder::tests::test_bc_1_11_002_placeholder_svg_has_explicit_width_height
     Summary [   0.009s] 10 tests run: 10 passed, 136 skipped
```

**What this proves (10 structural properties):**

| Property | Test |
|----------|------|
| SVG is non-empty | `_svg_nonempty` |
| Root element is `<svg>` | `_svg_has_root_element` |
| `viewBox="0 0 800 450"` | `_svg_has_800x450_viewbox` |
| Explicit `width`/`height` attrs | `_svg_has_explicit_width_height` |
| Contains `E-LAY-003` text | `_svg_contains_error_code` |
| Contains error message text | `_svg_contains_message` |
| `<title>` element present (a11y) | `_svg_has_title_element` |
| `aria-label` attribute present (a11y) | `_svg_has_aria_label` |
| No `<script>` (XSS safety) | `_svg_no_script` |
| No `<foreignObject>` (PPTX-safety) | `_svg_no_foreign_object` |

Additional AC-004 coverage:
- `test_bc_1_11_002_placeholder_xml_escapes_user_input` — HTML entities in slide title/message are escaped
- `test_bc_1_11_002_placeholder_title_element_escapes_slide_title` — `<title>` escapes adversarial input
- `test_bc_1_11_002_placeholder_constructible_from_empty_data_error` — `FrameContent::ErrorSlidePlaceholder` variant roundtrip from `ChartError::EmptyData`

---

### AC-005: Multi-slide accumulation — one empty, two valid

**Test:** `tests::test_bc_1_11_002_multi_slide_one_empty_two_valid`
**Source:** `crates/slideforge-charts/src/lib.rs`
**Traces to:** BC-1.11.002 invariant 3 and DI-018

```
cargo nextest run -p slideforge-charts \
  -E 'test(test_bc_1_11_002_multi_slide_one_empty_two_valid)'

    Starting 1 test across 1 binary (145 tests skipped)
        PASS [   0.009s] (1/1) slideforge-charts tests::test_bc_1_11_002_multi_slide_one_empty_two_valid
     Summary [   0.009s] 1 test run: 1 passed, 145 skipped
```

**What this proves:** A fixture of three chart slides — `[empty, valid-bar, valid-line]` —
is processed. The result is:
- 1 `ChartError::EmptyData` accumulated in the `DiagnosticSink`
- 2 successful `ChartSvg` renderings (bar and line charts rendered to non-empty SVG)
- Processing continues past the empty slide without early abort (DI-018 compliance)

---

## Error Path Coverage

| Error Path | Covered by | Result |
|------------|-----------|--------|
| `Value::List([])` is empty | `test_bc_1_11_002_data_is_empty_for_empty_list` | PASS |
| `Value::Map({})` is empty | `test_bc_1_11_002_data_is_empty_for_empty_map` | PASS |
| `Value::List` with 1 element is NOT empty | `test_bc_1_11_002_data_is_not_empty_for_single_element_list` | PASS |
| `Value::Int/Null/Str/Bool` not flagged as empty (caught at eval stage) | `test_bc_1_11_002_data_is_not_empty_for_non_list_value` | PASS |
| Outer list with inner empty list NOT flagged (EC-003) | `test_bc_1_11_002_data_is_not_empty_for_outer_list_with_inner_empty_list` | PASS |
| XSS injection in slide title → XML-escaped in SVG | `test_bc_1_11_002_placeholder_xml_escapes_user_input` | PASS |
| Adversarial title with `<>&"` in `<title>` element | `test_bc_1_11_002_placeholder_title_element_escapes_slide_title` | PASS |

---

## Coverage Map

| Requirement | Source | Covered? |
|-------------|--------|----------|
| BC-1.11.002 postcondition 1 (E-LAY-003 emitted) | AC-001 | Yes — 6 diagnostic tests |
| BC-1.11.002 invariant 2 (renderer not called) | AC-002 | Yes — spy mock test |
| BC-1.11.002 postcondition 2 (Error severity in strict mode) | AC-003 | Yes — severity test |
| BC-1.11.002 postcondition 3 (warn-only placeholder) | AC-004 | Yes — 13 placeholder tests |
| BC-1.11.002 invariant 3 + DI-018 (accumulation, no abort) | AC-005 | Yes — multi-slide test |

---

## Toolchain

| Tool | Version | Status |
|------|---------|--------|
| cargo nextest | 0.9.x | installed |
| VHS | N/A | not applicable — library-only story |
| Playwright | N/A | not applicable — library-only story |

---

## Notes

- This is a pure library story (`slideforge-charts` crate). There is no CLI binary
  or web UI surface to record. VHS and Playwright are not applicable.
- Evidence is `cargo nextest` output — the authoritative execution record for library-layer ACs.
- 31 tests, all passing, covering both success paths (valid data renders) and error
  paths (empty data emits diagnostic, blocks renderer, produces placeholder SVG).
- STORY-031 regression tests (BC-1.11.001) continue to pass — STORY-032's guard did not
  break the happy path.
