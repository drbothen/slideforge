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
capability: CAP-009
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

# BC-1.09.002: Alias Cannot Add New Fields — Only Preset Existing Fields

## Description

An `alias` block may only declare fields that already exist on the base slide type.
Attempting to add a field that is not part of the base type's field set is a compile
error. This constraint keeps aliases as syntactic sugar and prevents them from
fragmenting the type system with extended types that downstream validators and exporters
do not know about.

## Preconditions

1. An `alias <name> = <type>:` block exists with one or more field declarations.
2. The base `<type>` has a known, finite set of fields defined by its `SlideType` plugin trait.

## Postconditions

1. All fields declared inside the alias block are valid fields of the base type. If all are valid, the alias is registered successfully.
2. Any field declared in the alias block that is NOT a valid field of the base type produces a compile error naming the unknown field and listing valid fields for the type.
3. No alias can introduce a new field into the slide type system.

## Invariants

1. The set of valid fields for a slide type is fixed at compile time by the `SlideType` trait implementation.
2. Aliases are strictly additive presets, not extensions.
3. This constraint applies at parse time — unknown alias fields are caught before evaluation begins.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `alias exec_title = title:` + `new_custom_field "value"` | Compile error: 'new_custom_field' is not a field of slide type 'title'. Valid fields: [...] |
| EC-002 | Alias with zero field presets | Valid — equivalent to `set <alias> = <type>:` with no defaults; alias is just a rename |
| EC-003 | Alias presets a `required` field of the base type | Valid — the required field is pre-satisfied by the alias; slides using the alias inherit the preset |
| EC-004 | Alias presets a field to an invalid value type | Same type error as setting the field directly on a slide |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `alias exec_title = title:` + `subtitle "Exec Edition"` (subtitle is a valid title field) | Alias registered; slides using it have subtitle preset; exit 0 | happy-path |
| `alias exec_title = title:` + `executive_only true` (not a title field) | Compile error naming unknown field; exit 1 | error |
| `alias bare = content:` (empty body — no presets) | Valid; bare is a rename of content with no preset fields | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Alias with unknown field produces compile error; alias with valid field succeeds | unit test with fixtures |
| VP-TBD | Alias field set is a strict subset of base type field set (provable with type system) | type-level check in SlideType trait |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Parametric Slide-Type Aliases") per capabilities.md §CAP-009 |
| Capability Anchor Justification | CAP-009 ("Parametric Slide-Type Aliases") per capabilities.md §CAP-009 — "they cannot add new fields, only preset existing ones" is verbatim from CAP-009 |
| L2 Domain Invariants | (none directly — this is a structural constraint of the alias system) |
| Architecture Module | slideforge-syntax crate — AliasRegistry field validation (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.09.001 — composes with (this BC is the companion constraint to parse-time alias resolution)

## Architecture Anchors

- `architecture/system-overview.md#aliases` — alias field validation design
- `architecture/system-overview.md#slide-types` — SlideType trait field set definition

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
