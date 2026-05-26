---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-032
title: "Chart: Empty Data Error-Slide Placeholder"
epic: EPIC-11
wave: 3
points: 3
priority: P1
tdd_mode: strict
status: draft
behavioral_contracts: [BC-1.11.002]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-charts
target_module: slideforge-charts
subsystems: [SS-12]
depends_on: [STORY-031, STORY-016]
blocks:
  - STORY-037
  - STORY-046
estimated_days: 1
---

# STORY-032: Chart — Empty Data Error-Slide Placeholder

## Subsystem Anchor Justification

SS-12 (Charts) owns the empty-data guard because the `ChartRenderer` plugin decides
whether data is valid before invocation. Per BC-1.11.002 invariant 2, the `ChartRenderer`
plugin is NEVER called with empty data — the validation layer intercepts. However, the
`ChartError::EmptyData` variant and its error-slide production live in the charts crate's
companion validation logic.

## Dependency Anchor Justifications

- Depends on STORY-031: `ChartSpec`, `ChartError`, and `ChartRendererImpl` must exist.
  This story extends them with empty-data handling.
- Depends on STORY-016: Strict/warn-only mode behavior is established in `slideforge-validate`.
  This story references that mode flag to determine whether to error or produce a placeholder.
- Blocks STORY-037/046: Those exporters handle `FrameContent::ErrorSlidePlaceholder` which
  this story adds for empty-data chart slides.

## Summary

Add empty-data guard to the chart rendering pipeline. When a `slide chart:` block's
data binding evaluates to an empty collection:

1. **Do NOT call** `ChartRendererImpl::render()` (BC-1.11.002 invariant 2).
2. **Emit** `E-LAY-003` diagnostic with slide title and data binding expression.
3. **In strict mode**: accumulate error; no output written.
4. **In warn-only mode**: produce `FrameContent::ErrorSlidePlaceholder` for the slide.

The placeholder SVG is a slide-sized rectangle with an error message rendered as text.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-1.11.002 | Chart with empty data produces error-slide placeholder not a crash | AC-001, AC-002, AC-003, AC-004, AC-005 |

## Acceptance Criteria

### AC-001: E-LAY-003 emitted on empty data
(traces to BC-1.11.002 postcondition 1)

When chart data evaluates to an empty collection (`Value::List([])` or `Value::List`
with zero elements), before the `ChartRenderer::render()` call, the pipeline emits:
```
E-LAY-003: Chart data is empty for slide '<title>'. Rendering error-slide placeholder.
  --> deck.sf:12:3
  |
  12 |   data {{ kpis.monthly }}
  |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^
  hint: Ensure the data source contains at least one row.
```
The diagnostic is added to the `DiagnosticSink`.

### AC-002: ChartRenderer never called with empty data
(traces to BC-1.11.002 invariant 2)

The empty-data check runs BEFORE `ChartRendererImpl::render()` is called. No code
path in the chart pipeline passes an empty `data: Vec<DataSeries>` to
`render()`. This can be verified by a unit test that mocks `render()` and asserts
it is not called when data is empty.

### AC-003: Strict mode exits with code 2 and no output
(traces to BC-1.11.002 postcondition 2)

In strict mode (default), accumulating `E-LAY-003` causes the build to exit with
code 2 (validation error). No output file is written. This behavior is consistent
with `BC-3.03.002` (strict mode produces no output on validation error).

### AC-004: Warn-only mode produces error-slide placeholder
(traces to BC-1.11.002 postcondition 3)

In `--warn-only` mode, the chart slide's position in `LaidOutDeck.slides` gets a
`Frame` with `FrameContent::ErrorSlidePlaceholder`:
```rust
FrameContent::ErrorSlidePlaceholder {
    error_code: "E-LAY-003",
    message: "Chart data is empty for slide '{title}'",
    slide_title: Arc<str>,
}
```
The PPTX exporter (STORY-037) renders this as a slide with the error message
overlaid on a light gray background. Other slides in the deck render normally.

### AC-005: Error does not halt other chart processing
(traces to BC-1.11.002 invariant 3 and DI-018)

When one chart slide has empty data, the `DiagnosticSink` accumulates `E-LAY-003`
and layout continues to the next slide. Other chart slides with non-empty data
render normally. This is tested by a multi-slide fixture with one empty-data chart
and two valid charts.

## Tasks

- [ ] Add empty-data check before `render()` call in the chart pipeline integration point (likely in `slideforge-layout` where chart slides are processed, or in an intermediate validation layer)
- [ ] Define `E-LAY-003` error code and message template
- [ ] Add `FrameContent::ErrorSlidePlaceholder { error_code, message, slide_title }` to `slideforge-layout` `FrameContent` enum (coordinate with STORY-026)
- [ ] Implement placeholder SVG generation for warn-only mode: light gray `rect` + error text in `crates/slideforge-charts/src/placeholder.rs`
- [ ] Write unit tests:
  - empty data → E-LAY-003 accumulated, `render()` not called
  - strict mode: E-LAY-003 produces exit code 2 intent (test via `DiagnosticSink` severity)
  - warn-only mode: `FrameContent::ErrorSlidePlaceholder` produced
  - multi-slide: one empty-data chart + two valid charts → two charts render, one placeholder
  - `Value::List([])` recognized as empty
  - `null` data binding → E-EVL-006 (caught before chart stage — verify this path)

## Previous Story Intelligence

STORY-031 implements the happy path. This story adds the error path. The pattern
follows BC-3.03.003 (warn-only error-slide placeholders) established in STORY-016.
The `FrameContent::ErrorSlidePlaceholder` variant should be reused for all error
placeholder types (not just charts) — verify with the STORY-016 implementer that
this variant was not already added.

## Architecture Compliance Rules

1. **ChartRenderer never called with empty data (BC-1.11.002 invariant 2)**: The
   check must happen before the plugin call, not inside the plugin.
2. **Error accumulation (DI-018)**: E-LAY-003 is accumulated, not raised as a panic
   or early return that halts other slides.
3. **Consistent with validate stage (BC-3.03.002, BC-3.03.003)**: Strict vs warn-only
   behavior mirrors the validation stage gate. This story does not implement that
   gate from scratch — it reads the mode flag from `BuildConfig`.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-charts` (self, from STORY-031) | workspace | Extends chart types |
| `slideforge-types` | workspace | `Value`, `BuildConfig` (strict/warn-only flag) |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-charts/src/placeholder.rs` | Create | Error-slide placeholder SVG generation |
| `crates/slideforge-charts/src/validation.rs` | Create | Empty-data guard logic |
| `crates/slideforge-layout/src/types.rs` | Modify | Add `FrameContent::ErrorSlidePlaceholder` if not already present |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~1,500 |
| BC-1.11.002 | ~1,500 |
| STORY-031 chart types | ~1,000 |
| STORY-016 mode flag | ~500 |
| Test files to write | ~1,500 |
| **Total** | **~6,000** |

## Test Strategy

- **Unit tests**: Empty `Value::List([])` → `E-LAY-003` + no `render()` call; warn-only
  mode → `FrameContent::ErrorSlidePlaceholder`; multi-slide accumulation.
- **Regression guard**: `render()` with non-empty data still works after this story's
  changes (run STORY-031 snapshot tests).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-014) | `data {{ kpis.monthly }}` where `kpis.monthly` is `[]` | E-LAY-003; warn-only → placeholder |
| EC-002 | Data binding resolves to `null` (not a list) | E-EVL-006 caught at eval stage before chart; never reaches empty-data check |
| EC-003 | Data has exactly 1 row | Non-empty; chart renders normally |
| EC-004 | Multiple chart slides, one empty | Only empty chart emits E-LAY-003; others render |

## Forbidden Dependencies

Same as STORY-031.
