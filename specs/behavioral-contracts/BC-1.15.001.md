---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-030
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.15.001: All Errors Carry file:line:col Span and Correction Hint

## Description

Every error and warning emitted by the slideforge build system — whether from the parser,
evaluator, data source fetcher, validator, or exporter — must include a precise
`file:line:col` source span pointing to the location of the problem in the .sf source,
and a correction hint suggesting what the user should do to fix it. Errors are rendered
via miette/ariadne with colored source-code pointers in terminal output.

## Preconditions

1. A .sf source (or multi-file project) contains one or more errors or warnings.
2. The build system encounters the error during any phase: parse, evaluate, validate, or export.

## Postconditions

1. Every emitted diagnostic (error or warning) includes: file path, 1-based line number, 1-based column number.
2. Every emitted diagnostic includes at least one correction hint (e.g., "Did you mean 'X'?", "Add alt \"...\" to fix this.", "Use spaces, not tabs.").
3. When a diagnostic spans multiple lines (e.g., an unclosed block), the span covers the full range from start to end.
4. Diagnostics are rendered via miette or ariadne with colored output when stdout is a TTY; plain text in CI/pipe contexts.
5. The `--no-color` flag disables colored output.

## Invariants

1. No diagnostic is ever emitted without a source span — a generic "unknown location" is not acceptable.
2. No diagnostic is ever emitted without a correction hint — even if the hint is a documentation reference.
3. The span always references a valid position in the source file (line ≤ file line count, col ≤ line length + 1).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Error from an @include file | Span points to the line/col in the INCLUDED file, not the entry file |
| EC-002 | Error from a data source (network error) | Span points to the `@data` directive line in the .sf source; hint: "use --offline" |
| EC-003 | Error in a math block | Span points into the math expression (line within `$...$` or `$$...$$` block) |
| EC-004 | CLI invoked with redirected stdout (no TTY) | Plain text output without ANSI color codes |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Tab character at line 5, col 3 | E-PAR-003 with span "file.sf:5:3" and hint "Use spaces for indentation" | happy-path |
| Undefined variable at line 12, col 10 | E-EVL-001 with span "file.sf:12:10" and hint listing in-scope variables | happy-path |
| Missing alt on chart in included file | E-A11-001 with span pointing to the included file's chart line | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Every error struct has a non-None span (type-level guarantee via miette SourceSpan) | type-level check in Diagnostic trait impl |
| VP-TBD | Span line/col is within source file bounds | proptest: random errors with valid source inputs; verify span in-bounds |
| VP-TBD | Every error has a non-empty help text / correction hint | unit test: inspect all E-* error variants for Help annotation |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-030 ("Diagnostic Reporting with Source Spans") per capabilities.md §CAP-030 |
| Capability Anchor Justification | CAP-030 ("Diagnostic Reporting with Source Spans") per capabilities.md §CAP-030 — "Emit all errors and warnings with file:line:col spans and correction hints rendered via miette/ariadne" is verbatim from CAP-030 |
| L2 Domain Invariants | DI-018 (error accumulation in one pass) |
| Architecture Module | slideforge-syntax + slideforge-eval — miette Diagnostic trait impls (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.15.002 — composes with (accumulation is the mechanism by which all these spanned errors are collected)
- BC-1.15.003 — composes with (severity determines mode; spans are present regardless of severity)

## Architecture Anchors

- `architecture/system-overview.md` — miette Diagnostic trait + ariadne rendering

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
