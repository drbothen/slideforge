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
capability: CAP-014
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

# BC-1.12.002: Invalid Mermaid Syntax Produces Compile Error with Line Number Within Source

## Description

When a `slide diagram:` block's `source:` field contains invalid Mermaid syntax, the
DiagramRenderer (mermaid-rs-renderer) detects the error during the build and returns
it as a structured diagnostic. The compile error must include the line number within
the Mermaid source block — not just the slide block's start line — so the user can
pinpoint the specific line of the diagram definition that is malformed.

## Preconditions

1. A `slide diagram:` block exists with a `source:` field containing syntactically invalid Mermaid markup.
2. The `lang: mermaid` field is set (or defaulted to mermaid for diagram slides).

## Postconditions

1. E-EXP-008 is emitted: "Diagram render failed for slide '<title>': <mermaid-error> at source line <N>".
2. The `<N>` in the error is the 1-based line number within the Mermaid source block (not the .sf file line).
3. Build exits with code 3 (export error) in strict mode.
4. In `--warn-only` mode: error-slide placeholder is rendered for the diagram slide.
5. Other slides in the deck continue to be processed (DI-018 error accumulation).

## Invariants

1. The error always includes a line number within the Mermaid source block.
2. The error always names the slide title for locating the diagram.
3. DI-018: this error is accumulated; it does not halt processing of other slides.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-015) | Flowchart with an arrow to an undefined node | E-EXP-008 with diagram source line number; exit 3 |
| EC-002 | Completely empty `source:` field (empty string) | E-EXP-008: "empty Mermaid source" at line 1 |
| EC-003 | Valid Mermaid type but malformed content at line 3 of a 10-line diagram | Error reports "at source line 3" |
| EC-004 | Two diagram slides, one valid and one invalid | Valid slide renders; invalid emits E-EXP-008; exit 3 in strict |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `source:` with invalid Mermaid (e.g., `flowchart LR\n  A --> [broken`) | E-EXP-008 with source line; exit 3 | error (DEC-015) |
| Valid Mermaid source | No E-EXP-008; SVG produced; exit 0 | happy-path |
| Invalid diagram + `--warn-only` | E-EXP-008 warning; error-slide placeholder; exit 0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E-EXP-008 includes source line number within Mermaid block | unit test: parse error message, verify "at source line N" present |
| VP-TBD | Invalid diagram does not crash the renderer; returns structured error | unit test + fuzz boundary |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 ("Diagram Rendering (Mermaid)") per capabilities.md §CAP-014 |
| Capability Anchor Justification | CAP-014 ("Diagram Rendering (Mermaid)") per capabilities.md §CAP-014 — error reporting with line number in source block is required by the "every error carries file:line:col span" product principle applied to diagram source |
| L2 Domain Invariants | DI-018 (error accumulation in one pass) |
| Architecture Module | slideforge-diagrams crate — MermaidRenderer error path (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.12.001 — related to (this is the error path; BC-1.12.001 is the happy path)
- BC-1.15.001 — composes with (error carries span per BC-1.15.001 contract)
- BC-3.03.003 — composes with (warn-only error-slide placeholder)

## Architecture Anchors

- `architecture/authoring-subsystem.md#diagram-renderer` — error reporting from mermaid-rs-renderer

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
