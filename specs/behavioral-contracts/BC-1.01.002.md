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
capability: CAP-001
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

# BC-1.01.002: Reject Indentation Inconsistency with file:line:col Span

## Description

The slideforge parser enforces consistent space-based indentation. Any line whose
indentation is not a valid multiple of the declared indent width, or whose dedent
does not align to a prior indent level, produces E-PAR-001 with the exact
file:line:col span showing the expected and actual space count.

## Preconditions

1. A .sf source file is being parsed.
2. One or more lines contain an indentation level that is not a multiple of the
   declared indent width, or a dedent that does not match any prior indent level.
3. The file uses spaces (not tabs) — tab errors are covered by BC-1.01.003.

## Postconditions

1. E-PAR-001 is emitted for each indentation inconsistency: `Unexpected indentation at <file>:<line>:<col>. Expected <N> spaces, found <M>.`
2. All indentation errors in the file are accumulated before halting (DI-018).
3. Build exits with code 1.
4. No AST is produced; no output files are written.

## Invariants

1. Every E-PAR-001 includes the exact expected and actual column counts.
2. Errors are accumulated — the first error does not terminate parsing if further tokens can be parsed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First line of a block is indented 3 spaces when 2 are expected | E-PAR-001 at that line |
| EC-002 | Dedent lands between two valid indent levels (e.g., 4 spaces where only 0 or 6 are valid) | E-PAR-001 — dedent must align to a prior level |
| EC-003 | Multiple inconsistent lines in one file | All E-PAR-001 errors accumulated; reported together |
| EC-004 | Mixed 2-space and 4-space blocks in one file (inconsistent throughout) | All violations reported; parser continues best-effort |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Slide block indented 3 spaces (declared width: 2) | E-PAR-001: expected 2 spaces, found 3; exit 1 | error |
| Dedent from 4-space block to 1-space level | E-PAR-001: dedent does not match any open indent level; exit 1 | error |
| Consistent 2-space indentation throughout | No error; AST produced | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Every inconsistent indent line produces exactly one E-PAR-001 | unit test with multi-error fixture |
| VP-TBD | Error spans reference valid line/col positions in input | proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 |
| Capability Anchor Justification | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 — indentation consistency is a core requirement of the indentation-significant parser |
| L2 Domain Invariants | DI-018 (error accumulation in one pass) |
| Architecture Module | slideforge-syntax crate — parser (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.01.001 — depends on (this is an error path of the parse stage)
- BC-1.01.003 — related to (tab indentation is the sibling error)
- BC-1.15.001 — depends on (all errors carry file:line:col per BC-1.15.001)

## Architecture Anchors

- `architecture/authoring-subsystem.md` — chumsky 0.10 indentation-significant grammar

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
