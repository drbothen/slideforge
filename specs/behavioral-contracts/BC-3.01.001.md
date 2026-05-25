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
capability: CAP-010
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

# BC-3.01.001: Each Slide Type Enforces Required Fields via SlideType Trait

## Description

Every one of the 31 built-in slide types declares a set of required fields via the
`SlideType` plugin trait. When the evaluator processes a slide block, it invokes the
`SlideType::required_fields()` method and verifies that all listed fields are present
and non-empty. Missing required fields are compile errors, not runtime warnings.

## Preconditions

1. A slide block of the given type has been parsed into the AST.
2. The evaluator has resolved all `{{ }}` interpolations in the slide's fields.
3. The `SlideType` plugin for the slide type is registered in the plugin registry.

## Postconditions

1. If all required fields are present and non-empty, the slide is added to the `Deck` IR.
2. If any required field is missing or empty, an E-PAR-007-class error is emitted per
   missing field (with field name and slide title in the message), and the slide is not
   added to the IR.
3. Optional fields that are absent are populated with their declared defaults (or None
   if no default exists).

## Invariants

1. Required-field validation is performed by the `SlideType` trait, not by ad-hoc
   per-type code. All 31 slide types go through the same validation path. (DI-008)
2. An empty string (`title ""`) counts as a missing required field — it triggers the
   same error as an absent field.
3. Fields declared in the DSL but not recognized by the `SlideType` produce an
   E-PAR-007-class "unknown field" warning (not an error) so users can detect typos.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slide content:` missing `title` | E-PAR-007 equivalent: "Required field 'title' missing on slide at line <N>"; error accumulated |
| EC-002 | `slide content:` with `title ""` (empty string) | Same error as absent field: "Required field 'title' is empty" |
| EC-003 | `slide content:` with an unrecognized field `color: "red"` | Warning: "Unknown field 'color' for slide type 'content'. Known fields: [title, bullets, takeaway, ...]" |
| EC-004 | SlideType plugin not found for declared type | E-PAR-007: "Unknown slide type '<keyword>'"; see BC-3.01.003 |
| EC-005 | All required fields present but one is `{{ undefined_var }}` | E-EVL-001 (undefined variable) is emitted; required-field check considers the field absent after failed eval |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide content: title "Risks"` (bullets optional for this type) | Slide added to Deck IR; exit 0 | happy-path |
| `slide content:` with no title field | E-PAR-007-class error; slide not in IR; exit 1 | error |
| `slide stat_callout:` with all 4 required stat fields populated | Slide added to Deck IR; exit 0 | happy-path |
| `slide content: title "" bullets [...]` | "Required field 'title' is empty" error | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 31 slide type plugins declare required_fields() with at least one entry (title or ctrTitle) | unit test (iterate registered plugins, assert required_fields non-empty) |
| VP-TBD | Required field check is triggered via SlideType trait, not type-specific code | code review + unit test (mock trait, verify call) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 |
| Capability Anchor Justification | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 — "Each type enforces its own required fields and layout rules via the SlideType plugin trait" is the verbatim description of what this BC specifies |
| L2 Domain Invariants | DI-008 (all bundled plugins must use plugin trait APIs) |
| Architecture Module | slideforge-eval crate — slide validation pass; slideforge-plugin-api crate — SlideType trait (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.01.002 — composes with (this BC defines the validation mechanism; BC-3.01.002 specifies the error format for missing fields)
- BC-3.01.003 — related to (unknown slide type keyword is a different error path from missing field)
- BC-5.02.001 — depends on (SlideType trait is one of the 10 plugin surfaces in CAP-021)

## Architecture Anchors

- `architecture/crate-architecture.md` — SlideType trait API definition

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
