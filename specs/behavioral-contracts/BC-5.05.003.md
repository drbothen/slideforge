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
capability: CAP-027
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

# BC-5.05.003: Watch Mode HTTP Schema Change (Field Rename) Surfaces Undefined-Var Errors

## Description

When an HTTP data source changes its response schema between polls (e.g., a field is
renamed from `revenue` to `total_revenue`), the next re-evaluation in watch mode
produces E-EVL-001 (undefined variable) for each affected `{{ }}` interpolation site.
These appear as error-slide placeholders. The behavior is identical to a static
undefined-variable error but occurs at poll time. This covers DEC-018.

## Preconditions

1. `slideforge watch` is running with an HTTP data source.
2. The HTTP data source returns a changed response where at least one field referenced
   by the deck's `{{ }}` interpolation is absent or renamed.
3. The poll interval fires and the new (changed) response is received.

## Postconditions

1. E-EVL-001 is emitted for each `{{ data.old_field_name }}` that no longer exists in
   the new response.
2. Each E-EVL-001 carries the file:line:col of the interpolation site and the list of
   currently-available variables.
3. Affected slides show error-slide placeholders in the web preview.
4. Non-affected slides (not referencing the renamed field) display normally with
   updated data from the new response.
5. If the HTTP source reverts to the old schema, the next poll restores the deck to
   the previous good state.

## Invariants

1. Undefined variable detection is the SAME code path in watch mode and batch build —
   there is no separate "watch mode undefined variable" handler.
2. The variable scope list in E-EVL-001 reflects the CURRENT poll response fields,
   giving the user an immediate hint for the rename.
3. Watch mode does NOT silently substitute empty string for missing fields. (DI-006)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Field renamed: data.revenue → data.total_revenue | E-EVL-001 for `{{ data.revenue }}`; hint shows `data.total_revenue` in scope |
| EC-002 | Field added to schema (no breakage) | Re-evaluation succeeds; new field available in scope; no E-EVL-001 |
| EC-003 | Schema changes back to original on next poll | Re-evaluation succeeds; deck restored to good state; error-slides cleared |
| EC-004 | Nested field renamed: data.metrics.rev → data.metrics.revenue | E-EVL-001 with full scope path shown |
| EC-005 | 10 field references to the renamed field | 10× E-EVL-001 entries (one per interpolation site); all affected slides show error-slide |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Poll returns `{total_revenue: 100}` when deck uses `{{ data.revenue }}` | E-EVL-001: "Undefined variable '{{ data.revenue }}'" with scope showing `data.total_revenue` | error (DEC-018) |
| Poll returns added field not referenced by deck | No errors; deck re-evaluates successfully | edge-case |
| Poll reverts schema to original | Next evaluation succeeds; error-slides cleared | happy-path (recovery) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E-EVL-001 emitted on schema field rename during watch | integration test: mock HTTP server with schema change; assert error emitted |
| VP-TBD | Available scope list in E-EVL-001 reflects current response fields | unit test |
| VP-TBD | No silent empty-string substitution for missing field | unit test: assert E-EVL-001, not empty string output |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-027 ("Watch Mode with Live Data Refresh") per capabilities.md §CAP-027 |
| Capability Anchor Justification | CAP-027 ("Watch Mode with Live Data Refresh") per capabilities.md §CAP-027 — this BC specifies the schema-change error sub-case of the live data refresh behavior |
| L2 Domain Invariants | DI-006 (undefined variables are compile errors — applies equally in watch mode re-evaluation) |
| Architecture Module | slideforge-cli watch + slideforge-eval undefined variable detection (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.05.001 — composes with (general watch mode; this BC is the schema-change sub-case)
- BC-5.05.002 — related to (HTTP failure vs schema change are distinct failure modes)
- BC-1.02.002 — depends on (undefined variable compile error; same E-EVL-001 path)

## Architecture Anchors

- `architecture/cross-cutting.md#watch-mode` — schema change detection during polling

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
