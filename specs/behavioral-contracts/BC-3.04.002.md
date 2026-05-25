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
capability: CAP-023
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

# BC-3.04.002: raw Keyword in User .sf Files Is Rejected at Parse Time

## Description

The `raw` keyword (and `raw pptx:`, `raw html:`, `raw xml:` variants) is reserved and
rejected at parse time when encountered in user-authored .sf files. It is an IR-internal
mechanism used by the layout engine and exporters only. Users who need custom shapes must
use the `shape:` DSL (CAP-023). The parse error names the `raw` keyword and explains
the correct alternative.

## Preconditions

1. The parser encounters a `raw` keyword in a user .sf file (at any scope: top-level,
   inside a slide, inside a section, etc.).

## Postconditions

1. E-PAR-009 is emitted:
   `'raw' keyword is not available in user .sf files at <file>:<line>:<col>. Use the shape: DSL instead.`
2. The `raw` block is skipped; subsequent content continues to be parsed (error accumulation).
3. Build exits with code 1.

## Invariants

1. The `raw` keyword is unconditionally rejected in user .sf files — no flag or mode allows
   it in user files. (DI-021)
2. This rejection applies to all `raw` variants: `raw:`, `raw pptx:`, `raw html:`,
   `raw xml:`, `raw docx:`.
3. Internal use of `raw` by the layout engine and exporters in the Rust codebase is not
   affected by this BC — this is purely a DSL parse-time contract.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `raw pptx: <ooxml>` inside a slide block | E-PAR-009; rest of slide parsed normally; error accumulated |
| EC-002 | `raw` as a variable name (`vars: { raw: "data" }`) | E-PAR-008: variable name 'raw' collides with reserved keyword |
| EC-003 | `raw` inside an @include file | E-PAR-009 reported with the @include file's path, not the entry file |
| EC-004 | Comment containing the word "raw" (`# raw OOXML here`) | Not a keyword; comments are ignored by the parser |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide content: raw pptx: <p:sp>...</p:sp>` | E-PAR-009; exit 1 | error |
| `slide content: title "Slide"` (no raw keyword) | Parsed successfully; no error | happy-path |
| `vars: { raw: "something" }` | E-PAR-008: "Variable name 'raw' collides with reserved keyword" | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Any `raw` keyword in user .sf always triggers E-PAR-009 | unit test (fuzz-style: inject raw at multiple positions) |
| VP-TBD | E-PAR-009 message always includes the alternative "Use the shape: DSL instead" | unit test (parse error message string) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-023 ("Structured Shape DSL") per capabilities.md §CAP-023 |
| Capability Anchor Justification | CAP-023 ("Structured Shape DSL") per capabilities.md §CAP-023 — "raw keyword is reserved and rejects at parse time" is verbatim in CAP-023; this is the enforcement contract for that statement |
| L2 Domain Invariants | DI-021 (reserved keywords must be rejected with descriptive errors) |
| Architecture Module | slideforge-syntax crate — reserved keyword table; E-PAR-009 error path (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.04.001 — composes with (shape: DSL is the correct alternative that this BC's rejection directs users toward)
- BC-1.01.005 — composes with (raw is one entry in the DI-021 reserved keyword list)

## Architecture Anchors

- `architecture/authoring-subsystem.md#reserved-keywords` — full reserved keyword table (DI-021)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
