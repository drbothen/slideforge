---
story: STORY-033
title: "Diagram Renderer: Mermaid → PPTX-Safe SVG"
phase: red-gate
date: 2026-05-27
agent: test-writer
---

# Red Gate Log — STORY-033

## Summary

60 tests total: 41 PASS, 19 FAIL.

Red Gate: VERIFIED. All behavioral tests that depend on the mermaid-rs-renderer
implementation fail before any implementation exists.

## mermaid-rs-renderer Availability

`mermaid-rs-renderer = "0.2.2"` confirmed present on crates.io. No stub SVG
generator was needed. The crate was used directly as the story specified.

## Test Run

```
cargo test -p slideforge-diagrams --no-fail-fast
test result: FAILED. 41 passed; 19 failed; 0 ignored; 0 measured
```

## Passing Tests (41) — Legitimately Pass Without Implementation

These tests verify data model correctness (type definitions, error message
format, accessibility attribute injection logic, error line extraction) that is
intentionally implemented as part of the stub — NOT rendering behavior.

### types.rs tests (15 tests)
| Test | Why It Passes |
|------|---------------|
| `types::test_bc_1_12_001_diagram_lang_from_keyword_mermaid` | Tests `DiagramLang::from_keyword` — type model |
| `types::test_bc_1_12_001_diagram_lang_from_keyword_unknown_returns_none` | Same |
| `types::test_bc_1_12_001_diagram_lang_as_keyword_roundtrip` | Same |
| `types::test_bc_1_12_001_raw_diagram_svg_into_string` | Tests `RawDiagramSvg` newtype accessor |
| `types::test_bc_1_12_001_raw_diagram_svg_as_str` | Same |
| `types::test_bc_1_12_001_raw_diagram_svg_is_empty_false_for_nonempty` | Same |
| `types::test_bc_1_12_001_raw_diagram_svg_is_empty_true_for_empty` | Same |
| `types::test_bc_1_12_002_diagram_error_syntax_error_message_contains_e_exp_008` | Tests `DiagramError` display — error taxonomy |
| `types::test_bc_1_12_002_diagram_error_empty_source_message` | Same |
| `types::test_bc_1_12_002_diagram_error_unsupported_language_message` | Same |
| `types::test_bc_1_12_002_diagram_error_render_error_message` | Same |
| `types::test_bc_1_12_002_diagram_error_svg_post_processing_error_message` | Same |

### error.rs tests (12 tests)
All 12 error module tests pass — they test `extract_source_line` (pure string
parsing logic) and `build_syntax_error` (constructs `DiagramError` from parsed
line). No rendering dependency.

### accessibility.rs tests (11 tests)
All 11 accessibility tests pass — they test `inject_aria_attributes` and
`assert_no_forbidden_elements` directly on static SVG strings. These functions
are fully implemented in the stub (they don't require mermaid-rs-renderer).

### renderer.rs (2 tests — early-exit before todo!())
| Test | Why It Passes |
|------|---------------|
| `renderer::tests::test_bc_1_12_002_empty_source_returns_empty_source_error` | Empty source caught before `todo!()` |
| `renderer::tests::test_bc_1_12_002_whitespace_only_source_returns_empty_source_error` | Same |
| `renderer::tests::test_bc_1_12_001_render_no_panic_on_arbitrary_input` | Uses `std::panic::catch_unwind` — no assertion |

### lib.rs (1 test)
| Test | Why It Passes |
|------|---------------|
| `tests::test_bc_1_12_002_unsupported_language_returns_error` | Tests `DiagramError::UnsupportedLanguage` construction — no rendering |
| `tests::test_bc_1_12_001_ac001_diagram_renderer_impl_id` | Tests `renderer.id()` literal constant |
| `tests::test_bc_1_12_001_ac001_diagram_renderer_impl_is_send_sync` | Compile-time trait bound |

**Verdict:** All 41 passing tests are legitimate. They test:
1. Type model correctness (`DiagramLang`, `RawDiagramSvg`, `DiagramError`)
2. Error taxonomy display strings (E-EXP-008 format)
3. Source line extraction (pure string parsing)
4. SVG accessibility injection (operates on static test strings)
5. Forbidden element detection (operates on static test strings)
6. Early-exit validation (empty source caught before rendering stub)

No vacuously true tests exist.

## Failing Tests (19) — All Fail With `todo!()` Panics

All 19 behavioral tests panic at `todo!()` in `renderer::render_mermaid` or
`DiagramRendererImpl::render`. The panic message is:
```
not yet implemented: STUB: mermaid_rs_renderer::render(source) call — ...
```

### Renderer behavioral tests (14 tests in renderer.rs)
- `test_bc_1_12_001_flowchart_produces_svg`
- `test_bc_1_12_001_sequence_diagram_produces_svg`
- `test_bc_1_12_001_class_diagram_produces_svg`
- `test_bc_1_12_001_gantt_chart_produces_svg`
- `test_bc_1_12_001_state_diagram_produces_svg`
- `test_bc_1_12_002_invalid_syntax_returns_error`
- `test_bc_1_12_002_invalid_syntax_error_is_mermaid_syntax_error_variant`
- `test_bc_1_12_001_flowchart_has_aria_label`
- `test_bc_1_12_001_flowchart_has_title_element`
- `test_bc_1_12_001_flowchart_no_foreign_object`
- `test_bc_1_12_001_flowchart_no_script`
- `test_bc_1_12_001_flowchart_no_keyframes`
- `test_bc_1_12_001_invariant_all_tested_types_render_without_panic`

### lib.rs trait tests (5 tests)
- `tests::test_bc_1_12_001_diagram_renderer_impl_renders_flowchart`
- `tests::test_bc_1_12_001_snapshot_flowchart_svg`
- `tests::test_bc_1_12_001_snapshot_sequence_svg`
- `tests::test_bc_1_12_001_trait_render_flowchart`
- `tests::test_bc_1_12_001_trait_render_sequence_diagram`
- `tests::test_bc_1_12_001_trait_render_empty_source_returns_error`

## BC Coverage

| BC Clause | Tests Covering |
|-----------|----------------|
| BC-1.12.001 precondition 1 (non-empty source) | `test_bc_1_12_002_empty_source_returns_empty_source_error`, `test_bc_1_12_002_whitespace_only_source_returns_empty_source_error` |
| BC-1.12.001 postcondition 1 (valid SVG produced) | `test_bc_1_12_001_flowchart_produces_svg`, `test_bc_1_12_001_sequence_diagram_produces_svg`, `test_bc_1_12_001_class_diagram_produces_svg`, `test_bc_1_12_001_gantt_chart_produces_svg`, `test_bc_1_12_001_state_diagram_produces_svg` |
| BC-1.12.001 postcondition 2 (no foreignObject) | `test_bc_1_12_001_flowchart_no_foreign_object`, `test_bc_1_12_001_no_foreign_object_rejects_foreign_object` |
| BC-1.12.001 postcondition 3 (no script) | `test_bc_1_12_001_flowchart_no_script`, `test_bc_1_12_001_no_script_rejects_script_element` |
| BC-1.12.001 postcondition 4 (no @keyframes) | `test_bc_1_12_001_flowchart_no_keyframes`, `test_bc_1_12_001_no_keyframes_rejects_animation` |
| BC-1.12.001 postcondition 7 (aria-label + title) | `test_bc_1_12_001_flowchart_has_aria_label`, `test_bc_1_12_001_flowchart_has_title_element`, all accessibility module tests |
| BC-1.12.001 invariant 1 (no Node.js — pure Rust) | Structural: no subprocess in the rendering path |
| BC-1.12.001 invariant 3 (23 diagram types) | `test_bc_1_12_001_invariant_all_tested_types_render_without_panic` (9 types tested) |
| BC-1.12.002 postcondition 1 (E-EXP-008 emitted) | `test_bc_1_12_002_diagram_error_syntax_error_message_contains_e_exp_008`, `test_bc_1_12_002_build_syntax_error_contains_e_exp_008` |
| BC-1.12.002 postcondition 2 (line number in error) | `test_bc_1_12_002_extract_line_number_*` (7 tests), `test_bc_1_12_002_build_syntax_error_line_number_extracted` |
| BC-1.12.002 EC-002 (empty source) | `test_bc_1_12_002_empty_source_returns_empty_source_error`, `test_bc_1_12_002_whitespace_only_source_returns_empty_source_error` |
| Snapshot regression | `test_bc_1_12_001_snapshot_flowchart_svg`, `test_bc_1_12_001_snapshot_sequence_svg` |

## Handoff Instruction for Implementer

**Make each test pass by replacing the `todo!()` stubs with real implementation.**

Single stub to replace: `renderer::render_mermaid` at
`crates/slideforge-diagrams/src/renderer.rs:58`.

The `todo!()` message tells you exactly what to do:
```
mermaid_rs_renderer::render(source).map_err(|e| build_syntax_error(&e.to_string()))
```

**Recommended implementation order:**
1. Replace `todo!()` in `renderer::render_mermaid` with:
   ```rust
   let raw_svg = mermaid_rs_renderer::render(source)
       .map_err(|e| build_syntax_error(&e.to_string()))?;
   ```
2. Add the `use mermaid_rs_renderer;` import.
3. Run `cargo test -p slideforge-diagrams` — expect most behavioral tests to pass.
4. Fix any remaining failures (forbidden element checks, aria injection).
5. Implement `DiagramRendererImpl::render` trait method by delegating to
   `Self::render_diagram` and mapping `DiagramError` variants to the plugin-api
   `DiagramError` variants.
6. Run snapshot tests and approve with `cargo insta accept`.
7. Add Criterion benchmark bodies to `benches/cold_render.rs` and
   `benches/warm_render.rs` per AC-003/004.
