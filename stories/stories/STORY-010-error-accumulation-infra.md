---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-010
title: "Error Accumulation + Diagnostic Infrastructure"
epic: EPIC-02
wave: 1
points: 5
priority: P0
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-syntax
behavioral_contracts:
  - BC-1.15.001
  - BC-1.15.002
  - BC-1.15.003
verification_properties: []
nfr_refs:
  - NFR-021
  - NFR-022
  - NFR-023
  - NFR-024
  - NFR-033
depends_on:
  - STORY-005
  - STORY-006
  - STORY-007
  - STORY-008
  - STORY-009
blocks:
  - STORY-055
subsystems:
  - SS-01
target_module: slideforge-syntax
---

# STORY-010: Error Accumulation + Diagnostic Infrastructure

## Summary

Harden and complete the diagnostic infrastructure for `slideforge-syntax`. The prior
parser stories (STORY-006 through STORY-009) introduced `SyntaxError` variants and basic
span threading. This story:

1. **Audits and completes** the `miette::Diagnostic` implementations on every
   `SyntaxError` variant to guarantee: non-None `SourceCode`, non-empty `help` text,
   and valid file:line:col spans (BC-1.15.001).
2. **Verifies** that the error accumulation model collects all independent errors in a
   single parse pass (BC-1.15.002): the `parse()` function returns
   `Err(Vec<SyntaxError>)` with all independent errors, not just the first.
3. **Establishes** the three-tier severity model (BC-1.15.003): parse errors (E-PAR-*)
   are always fatal; the `parse()` API communicates severity so the CLI layer (STORY-055)
   can determine exit codes correctly.
4. **Wires** `include_errors_from_file` into span attribution so errors from `@include`
   files carry the included file's path (not the entry file's path).
5. **Provides** a `DiagnosticSink` type that downstream crates (evaluator, validator)
   can push diagnostics into, enabling cross-stage accumulation.

This story is cross-cutting within `slideforge-syntax` and touches every parser module.
It is the final story in EPIC-02 and the sign-off gate before the parser is declared
complete.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-1.15.001 | All errors carry file:line:col span and correction hint | All 5 postconditions; all 3 invariants |
| BC-1.15.002 | All errors accumulated in single pass — no fail-on-first | All 4 postconditions; all 3 invariants |
| BC-1.15.003 | Parse errors always fatal; validation errors fatal in strict mode only | All 6 postconditions; all 4 invariants |

## Acceptance Criteria

- [ ] **AC-001** — Every `SyntaxError` variant implements `miette::Diagnostic` such that
  `<err>.source_code()` returns `Some(_)` and `<err>.help()` returns `Some(_)` with
  non-empty text. Verified by a unit test that iterates all variants via a fixture.
  (traces to BC-1.15.001 invariant 1, 2)

- [ ] **AC-002** — All source spans in `SyntaxError` reference valid positions: for any span
  `(file_id, start, end)`, the line derived from `start` is ≤ the file's total line count.
  A `span_is_valid(span, source_map)` helper function is provided and called from tests.
  (traces to BC-1.15.001 invariant 3)

- [ ] **AC-003** — Parsing a source with 2 independent indentation errors returns
  `Err(errors)` where `errors.len() == 2`. Neither error is a duplicate of the other.
  (traces to BC-1.15.002 postcondition 1, invariant 1)

- [ ] **AC-004** — Error ordering: `SyntaxError` implements `PartialOrd` ordering by
  `(file_id, line, col)`; `Vec<SyntaxError>` sorted by this order matches source-file order.
  (traces to BC-1.15.002 postcondition 2)

- [ ] **AC-005** — An error emitted from an `@include`-ed file carries a span whose `file_id`
  references the included file (not the entry file), and whose path resolves to the included
  file's path in `SourceMap`.
  (traces to BC-1.15.001 edge case EC-001; BC-1.15.002 invariant 3)

- [ ] **AC-006** — `SyntaxError` has a `severity()` method returning `ParseSeverity::Fatal`
  for all E-PAR-* variants. No E-PAR-* variant returns `ParseSeverity::Warning`.
  (traces to BC-1.15.003 postcondition 1, invariant 1)

- [ ] **AC-007** — A source with 1 E-PAR-* error and 1 E-EVL-* style diagnostic (hypothetical
  — the parser does not produce E-EVL-*; but the `DiagnosticSink` can hold mixed types)
  has `max_severity()` == `ParseSeverity::Fatal`.
  (traces to BC-1.15.003 postcondition 6 — highest severity wins)

- [ ] **AC-008** — `DiagnosticSink` is a `pub struct` in `slideforge-syntax` that:
  - Accepts `push(err: impl IntoDiagnostic)` from any stage.
  - Has `is_empty()`, `has_fatal()`, `errors() -> &[BoxDiagnostic]` accessors.
  - Implements `IntoIterator<Item = BoxDiagnostic>`.
  (traces to BC-1.15.002 invariant 3 — accumulation is a property of the BUILD run,
  spanning parser, evaluator, and validators collectively)

- [ ] **AC-009** — Parsing with redirected stdout (simulated via `DiagnosticRenderer::new(
  use_color: false)`) produces plain text without ANSI codes.
  (traces to BC-1.15.001 postcondition 4, edge case EC-004)

- [ ] **AC-010** — `DiagnosticRenderer::render_all(sink: &DiagnosticSink, source_map: &SourceMap,
  writer: &mut dyn Write)` writes all errors to the writer in source-file order.
  (traces to BC-1.15.002 postcondition 2)

- [ ] **AC-011** — A source with 100 undefined variable reference errors (simulated as
  100 `SyntaxError::UnexpectedToken` variants in a test) all appear in the `DiagnosticSink`
  without truncation. (traces to BC-1.15.002 edge case EC-003 — 100-error accumulation)

- [ ] **AC-012** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, and
  `clippy::pedantic` clean. (traces to NFR-021, NFR-022, NFR-024)

- [ ] **AC-013** — All public items have rustdoc; `cargo doc --no-deps` produces 0 warnings.
  (traces to NFR-023)

- [ ] **AC-014** — `DiagnosticSink::to_json()` produces valid JSON (parseable by `serde_json`)
  with all fields present (error code, file, line, col, message, hint).
  (traces to NFR-033 — structured JSON output complete and valid)

## Tasks

1. **Audit all `SyntaxError` variants** from STORY-006 through STORY-009:
   - For each variant, verify `source_code()` → `Some(_)` and `help()` → `Some(_)`.
   - If any variant is missing either, add the implementation.
   - Document the audit result in `error.rs` as a comment block.
2. Implement `span_is_valid(span: &Span, source_map: &SourceMap) -> bool`:
   - Returns `true` iff the byte range `[start, end]` maps to a valid line/col in the file.
   - Call from tests; also call in a debug-mode assertion inside `SyntaxError` constructors.
3. Implement `PartialOrd + Ord` for `SyntaxError` ordering by `(file_id, line, col)`.
   The line and col are computed lazily from the `Span` via `SourceMap` when needed.
4. Implement `ParseSeverity` enum:
   ```rust
   pub enum ParseSeverity {
       Fatal,    // E-PAR-*: always fatal, never demoted
       Error,    // E-EVL-*, E-DAT-*, E-LAY-*, E-A11-*: fatal in strict; demotable
       Warning,  // cosmetic / informational
   }
   ```
   Add `SyntaxError::severity() -> ParseSeverity` to the existing enum.
5. Implement `DiagnosticSink`:
   ```rust
   pub struct DiagnosticSink {
       diagnostics: Vec<Box<dyn miette::Diagnostic + Send + Sync>>,
   }
   impl DiagnosticSink {
       pub fn push(&mut self, err: impl miette::Diagnostic + Send + Sync + 'static);
       pub fn is_empty(&self) -> bool;
       pub fn has_fatal(&self) -> bool;
       pub fn len(&self) -> usize;
       pub fn errors(&self) -> &[Box<dyn miette::Diagnostic + Send + Sync>];
       pub fn to_json(&self, source_map: &SourceMap) -> serde_json::Value;
       pub fn max_severity(&self) -> Option<ParseSeverity>;
   }
   ```
6. Implement `DiagnosticRenderer`:
   ```rust
   pub struct DiagnosticRenderer {
       use_color: bool,
   }
   impl DiagnosticRenderer {
       pub fn new(use_color: bool) -> Self;
       /// Render all diagnostics in source order to `writer`.
       pub fn render_all(
           &self,
           sink: &DiagnosticSink,
           source_map: &SourceMap,
           writer: &mut dyn Write,
       ) -> io::Result<()>;
   }
   ```
7. Update `parse()` in `parser/mod.rs` to collect all chumsky errors into a
   `DiagnosticSink` and return `Err(sink)` (not `Err(Vec<SyntaxError>)`) — or keep the
   `Vec<SyntaxError>` return type and separately expose a `parse_into_sink()` variant.
   Design decision: return `Result<DeckNode, Vec<SyntaxError>>` from the internal
   `parse()` function (for type clarity); expose `parse_checked(src, file_id, source_map,
   sink)` that appends to a shared `DiagnosticSink`. The CLI (STORY-055) uses
   `parse_checked`.
8. Wire `@include` span attribution: in `IncludeResolver.resolve_include()` (STORY-008),
   ensure the `file_id` for the included file is registered BEFORE parsing, so that all
   chumsky spans use the correct `file_id` automatically.
9. Write unit tests for all ACs. Write integration test `test_multi_error_accumulation()`
   that parses a fixture with 5 independent errors and asserts `errors.len() == 5`.
10. Write snapshot test for `DiagnosticRenderer` output (plain text).

## File List

- `crates/slideforge-syntax/src/error.rs` — updated: all variants audited for
  `Diagnostic` completeness; `ParseSeverity` enum; `SyntaxError::severity()`;
  `PartialOrd + Ord` impl; `span_is_valid()` helper
- `crates/slideforge-syntax/src/sink.rs` — `DiagnosticSink` (new)
- `crates/slideforge-syntax/src/render.rs` — `DiagnosticRenderer` (new)
- `crates/slideforge-syntax/src/lib.rs` — updated: re-export `DiagnosticSink`,
  `DiagnosticRenderer`, `ParseSeverity`; update `parse` signature documentation
- `crates/slideforge-syntax/src/parser/mod.rs` — updated: `parse_checked()` function
- `crates/slideforge-syntax/tests/fixtures/five_independent_errors.sf` — test fixture
- `crates/slideforge-syntax/tests/fixtures/include_error_attribution.sf` — includes a
  file that has a parse error

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~4 000 |
| BC-1.15.001, BC-1.15.002, BC-1.15.003 | ~4 500 |
| STORY-006 error.rs + STORY-008 include.rs (for audit) | ~3 000 |
| STORY-007/009 SyntaxError variants (for audit) | ~2 500 |
| Target source files to write | ~4 000 |
| Test files | ~3 000 |
| **Total** | **~21 000** |

Context budget: 21 000 / 200 000 ≈ 10.5% — within limit.

## Test Strategy

**Unit tests** (`crates/slideforge-syntax/src/error.rs #[cfg(test)]`):

- `test_all_variants_have_source_code()`: iterate all `SyntaxError` variants via a
  helper fixture; assert `err.source_code().is_some()` for each.
- `test_all_variants_have_help()`: same fixture; assert `err.help().is_some()` and
  `!err.help().unwrap().is_empty()`.
- `test_span_is_valid_happy_path()`: create a span within a known source; assert
  `span_is_valid() == true`.
- `test_span_is_valid_out_of_bounds()`: create a span beyond the source end; assert
  `span_is_valid() == false`.
- `test_severity_par_errors_fatal()`: all E-PAR-* variants return `ParseSeverity::Fatal`.
- `test_error_ordering()`: 3 errors with different (file_id, line, col); sort;
  assert ascending order.

**Unit tests** (`crates/slideforge-syntax/src/sink.rs #[cfg(test)]`):

- `test_sink_push_and_len()`: push 3 errors; assert `sink.len() == 3`.
- `test_sink_has_fatal_true()`: push 1 E-PAR-* error; assert `has_fatal() == true`.
- `test_sink_has_fatal_false()`: push 0 errors; assert `has_fatal() == false`.
- `test_sink_max_severity_fatal()`: push mix of Warning + Fatal; assert
  `max_severity() == Some(ParseSeverity::Fatal)`.
- `test_sink_to_json_valid()`: push 2 errors; call `to_json()`; parse JSON with
  `serde_json::from_value()`; assert no deserialization errors.
- `test_sink_to_json_all_fields()`: assert JSON contains keys: `error_code`, `file`,
  `line`, `col`, `message`, `hint` for each error entry.

**Integration tests** (`crates/slideforge-syntax/tests/integration_tests.rs`):

- `test_multi_error_accumulation()`: parse `five_independent_errors.sf`; assert
  `sink.len() == 5` and `sink.has_fatal() == true`.
- `test_include_error_span_attribution()`: parse `include_error_attribution.sf`;
  assert the included file's error span references the included file's `file_id`.
- `test_renderer_plain_text_no_ansi()`: render 2 errors with `use_color: false`; assert
  output contains no ANSI escape sequences (regex check: `\x1b\[`).

**Snapshot tests**:

- `test_snapshot_renderer_output()`: render 3 errors to string; snapshot.

## Dependencies

- **Depends on:** STORY-005 (lexer token infrastructure used by `SyntaxError` spans)
- **Depends on:** STORY-006 (base `SyntaxError` enum, `Span`, `SourceMap`)
- **Depends on:** STORY-007 (additional `SyntaxError` variants from control-flow parsing)
- **Depends on:** STORY-008 (include error paths, variant error codes — E-VAR-001)
- **Depends on:** STORY-009 (E-PAR-009, E-PAR-010, E-PAR-008 variants)
- **Blocks:** STORY-055 (CLI build command — consumes `DiagnosticSink` and
  `DiagnosticRenderer` to display errors and determine exit codes)

## Dependency Anchor Justifications

- SS-01 owns this story's scope because SS-01 is the DSL Parser subsystem owning
  `slideforge-syntax` per ARCH-INDEX Subsystem Registry. Diagnostic infrastructure is
  part of the parser crate even though it is consumed by downstream crates.
- STORY-010 depends on STORY-006/007/008/009 because this story audits and extends all
  `SyntaxError` variants defined across those four stories — it must be written last.
- STORY-010 blocks STORY-055 (CLI) because the CLI's exit-code determination and error
  rendering depends on `ParseSeverity`, `DiagnosticSink`, and `DiagnosticRenderer`
  defined here.

## Architecture Compliance Rules

1. `DiagnosticSink` is a **pure core** type — it holds no file handles, no network
   connections, no async state. It is `Send + Sync`.
2. `DiagnosticRenderer` performs I/O (writes to `dyn Write`) and is therefore an
   **effectful shell** type, but it lives in `slideforge-syntax` because the rendering
   logic (span extraction, color output) is a syntax/diagnostic concern.
3. `DiagnosticSink::to_json()` must use `serde_json` — do NOT add a separate JSON
   serialization library.
4. The `BoxDiagnostic` type alias is `Box<dyn miette::Diagnostic + Send + Sync>`. Do
   not use `Arc<dyn Diagnostic>` — `Box` is sufficient; no shared ownership needed.
5. `ParseSeverity` must NOT be a bitflag — use a plain enum with `PartialOrd` derived
   by field ordering (`Warning < Error < Fatal`).
6. `#![forbid(unsafe_code)]` — already at crate root; verify not removed.

**Forbidden dependencies for `slideforge-syntax`:** same as STORY-006/007/008/009.
Additionally: `DiagnosticSink` must NOT depend on `slideforge-eval` or any downstream
crate — it is the bottom of the error-handling stack.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `miette` | `=7.6.0` | `Diagnostic` trait; `GraphicalReportHandler` for color rendering |
| `thiserror` | `=2.0.18` | Error derives |
| `serde_json` | `=1.0.140` | `DiagnosticSink::to_json()` (verify latest 1.0.x patch at implementation time) |
| `serde` | `=1.0.219` | Serialization derive on the JSON output struct (verify latest 1.0.x patch at implementation time) |

Note on miette rendering: use `miette::GraphicalReportHandler` for colored output and
`miette::JSONReportHandler` or a manual JSON serializer for `--json` mode. In
`DiagnosticRenderer`, the `use_color: bool` flag selects between:
- `use_color = true`: `miette::GraphicalReportHandler::new()` (default, TTY-detected)
- `use_color = false`: `miette::GraphicalReportHandler::new().without_syntax_highlighting()`
  or plain text formatting.

## File Structure Requirements

```
crates/slideforge-syntax/
  src/
    error.rs      # updated: ParseSeverity, SyntaxError::severity(), PartialOrd, span_is_valid()
    sink.rs       # DiagnosticSink (new)
    render.rs     # DiagnosticRenderer (new)
    lib.rs        # updated: pub use sink::DiagnosticSink, render::DiagnosticRenderer,
                  #          error::ParseSeverity
    parser/
      mod.rs      # updated: parse_checked() alongside parse()
  tests/
    fixtures/
      five_independent_errors.sf
      include_error_attribution.sf   # deck with @include that has 1 error
      include_error_file.sf          # the included file with the error
    integration_tests.rs            # updated with new tests
```

## Previous Story Intelligence

From STORY-006: a common failure pattern in diagnostic infrastructure is `unwrap()`-ing
on `SourceMap::get(file_id)` when the file_id was never registered. Every `SyntaxError`
constructor must validate that `file_id` exists in `SourceMap` at construction time,
or accept a `SourceMap` reference during construction and embed the source text directly
into the `miette::NamedSource`. The latter is safer.

From STORY-008: `IncludeResolver` assigns `file_id` to included files when adding them
to `SourceMap`. Verify the assignment happens BEFORE the recursive `parse()` call, not
after — otherwise all spans from included-file parsing will carry the wrong `file_id`.

## Implementation Notes

### `DiagnosticSink::to_json()` output schema

```json
{
  "diagnostics": [
    {
      "error_code": "E-PAR-001",
      "severity": "fatal",
      "file": "src/deck.sf",
      "line": 5,
      "col": 3,
      "message": "Unexpected indentation at src/deck.sf:5:3. Expected 2 spaces, found 3.",
      "hint": "Use consistent 2-space indentation throughout the file."
    }
  ],
  "total": 1,
  "has_fatal": true
}
```

This schema must match the NFR-033 requirement: `cargo nextest run` parses this JSON
via a test helper and checks all fields are present.

### Audit checklist for SyntaxError variants

After writing this story, run the following check to confirm completeness:

```rust
// In a test:
fn assert_diagnostic_complete<E: miette::Diagnostic>(err: &E) {
    assert!(err.source_code().is_some(), "source_code must be Some");
    let help = err.help();
    assert!(help.is_some(), "help must be Some");
    assert!(!help.unwrap().to_string().is_empty(), "help must be non-empty");
}
```

Call `assert_diagnostic_complete(&err)` for an instance of each `SyntaxError` variant.

### Three-tier severity mapping

| Error category | Codes | `ParseSeverity` | Exit code (CLI, STORY-055) |
|---------------|-------|----------------|--------------------------|
| Parse errors | E-PAR-* | `Fatal` | 1 |
| Evaluation errors | E-EVL-*, E-DAT-* | `Error` | 2 (strict) / 0 (warn-only) |
| Layout/validation | E-LAY-*, E-A11-* | `Error` | 2 (strict) / 0 (warn-only) |
| Export errors | E-EXP-* | `Fatal` | 3 |
| Cosmetic/lint | E-BRD-003, E-BRD-004 | `Warning` | 0 |

`DiagnosticSink::max_severity()` returns the highest severity in the sink, enabling the
CLI to compute the final exit code with a single call.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `DiagnosticSink` with 0 diagnostics | `is_empty() == true`, `has_fatal() == false`, `max_severity() == None`, `to_json()` has `"total": 0` |
| EC-002 | 2 errors at identical (file_id, line, col) from two validators | `DiagnosticSink` does NOT deduplicate — deduplication is responsibility of the caller; store both. Reported: both |
| EC-003 | `render_all()` called with `use_color: true` in a non-TTY context | The CLI detects non-TTY and sets `use_color: false` before calling `render_all()` — `DiagnosticRenderer` itself does not detect TTY |
| EC-004 | Error from evaluator stage (E-EVL-*) pushed into `DiagnosticSink` via `push()` | Accepted — `DiagnosticSink` is stage-agnostic |
| EC-005 | `to_json()` called with a span referencing a `file_id` not in `SourceMap` | Returns `"file": "<unknown file_id=N>"` without panicking |
