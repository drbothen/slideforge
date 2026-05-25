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

# BC-1.01.003: Reject Tab Indentation with Hard Error

## Description

slideforge requires spaces for indentation. Any tab character (`\t`) appearing as
leading whitespace on any line is a hard parse error. The error is reported with the
exact file:line:col of the tab character. No output is produced. Tab rejection is
not configurable — there is no `--allow-tabs` flag.

## Preconditions

1. A .sf source file is being parsed.
2. At least one line contains a tab character as leading whitespace.

## Postconditions

1. E-PAR-003 is emitted for each tab character found in leading whitespace: `Tab character at <file>:<line>:<col>. slideforge requires spaces for indentation.`
2. All tab errors in the file are accumulated before halting (DI-018).
3. Build exits with code 1.
4. No AST is produced; no output files are written.

## Invariants

1. Tabs within string values (not as indentation) do not trigger this error.
2. Tab rejection is not configurable — spaces are always required.
3. Each tab occurrence in leading whitespace produces its own E-PAR-003.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Tab character inside a quoted string value | No error — tab is valid inside string literals |
| EC-002 | File has both tab and space indentation violations | Both E-PAR-003 (tabs) and E-PAR-001 (bad space count) accumulated |
| EC-003 | Mixed tab+space indentation on same line (`\t  field:`) | E-PAR-003 at the tab position |
| EC-004 | Tab character in a comment | No error — tab in comments does not trigger this rule |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Line beginning with `\tfield: value` | E-PAR-003 at col 1; exit 1 | error |
| Line with spaces followed by tab (`  \t`) | E-PAR-003 at the tab col; exit 1 | error |
| `title "contains\ttab"` (tab in string) | No error; tab preserved in string value | happy-path |
| All-space file | No E-PAR-003; parse proceeds normally | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Every leading-whitespace tab produces exactly one E-PAR-003 | unit test with fixture |
| VP-TBD | Tabs inside string literals never trigger E-PAR-003 | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 |
| Capability Anchor Justification | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 — tab rejection is explicitly called out in the indentation-significant grammar requirement |
| L2 Domain Invariants | DI-018 (error accumulation in one pass) |
| Architecture Module | slideforge-syntax crate — lexer (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.01.002 — related to (sibling indentation error; space inconsistency)
- BC-1.15.001 — depends on (all errors carry file:line:col per BC-1.15.001)

## Architecture Anchors

- `architecture/system-overview.md` — lexer tab-rejection rule

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
