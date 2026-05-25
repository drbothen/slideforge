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

# BC-1.15.002: All Errors Accumulated in Single Pass — No Fail-on-First

## Description

The parser and validator accumulate ALL errors during a single compilation pass.
The first error encountered does not terminate processing — subsequent errors are
also collected and reported. Users see the complete set of problems from a build
in one run. This eliminates the "fix one error, rediscover another" development
cycle that competitor tools impose.

## Preconditions

1. A .sf source with N > 1 independent errors (errors that are not caused by earlier errors).

## Postconditions

1. All N errors are reported in a single build output.
2. Errors are reported in source-file order (ascending by file, then by line number, then by column).
3. Exit code reflects the HIGHEST severity error encountered (parse errors → exit 1; validation errors → exit 2).
4. No output files are produced if ANY error of severity "broken" exists in strict mode.

## Invariants

1. Error count in output = actual number of independent errors in the source (no duplicates, no omissions).
2. If a parse error prevents further parsing (e.g., an unclosed block that makes indentation inference impossible), accumulation stops at that point but reports all errors found so far.
3. The error accumulation is a property of the BUILD run — it applies to the parser, evaluator, data source fetcher, and validators collectively.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two parse errors and one accessibility error | All 3 errors reported; exit 1 (parse errors take precedence) |
| EC-002 | Parse error that makes further parsing impossible (e.g., corrupt indentation throughout) | Reports errors found before the point of failure; notes that processing was truncated; exit 1 |
| EC-003 | 100 undefined variable references | All 100 E-EVL-001 errors reported; exit 2 |
| EC-004 | Same error at the same location from two validators | Deduplicated — reported once |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 2 independent indentation errors (TV-1.2) | 2 E-PAR-001 errors; exit 1 | happy-path |
| 3 independent errors: 2 undefined vars + 1 missing alt (TV-13.1) | All 3 errors reported; exit 2 | happy-path |
| 1 error only | 1 error reported; exit non-zero | edge-case |
| 0 errors | Empty diagnostics; exit 0 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | For a source with N independent errors, output contains exactly N error entries | unit test with N-error fixtures (N = 1, 2, 5, 10) |
| VP-TBD | Error ordering: errors are sorted by (file, line, col) in output | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-030 ("Diagnostic Reporting with Source Spans") per capabilities.md §CAP-030 |
| Capability Anchor Justification | CAP-030 ("Diagnostic Reporting with Source Spans") per capabilities.md §CAP-030 — error accumulation in one pass is explicitly called out in CAP-030 as "never fail on first error" |
| L2 Domain Invariants | DI-018 (error accumulation in one pass) |
| Architecture Module | slideforge-syntax + slideforge-eval (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.15.001 — composes with (errors accumulated here use the span format from BC-1.15.001)
- BC-1.01.001 — depends on (parse stage demonstrates this invariant)
- BC-1.02.002 — depends on (eval error accumulation example)
- BC-5.01.001 — depends on (accessibility errors accumulated with all other errors)

## Architecture Anchors

- `architecture/system-overview.md#error-accumulation` — chumsky error recovery design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
