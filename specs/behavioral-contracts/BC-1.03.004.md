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
capability: CAP-003
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

# BC-1.03.004: Support --offline Flag to Skip HTTP Sources

## Description

When the `--offline` flag is passed to any slideforge build command, all `@data`
directives pointing to HTTP/HTTPS URLs are skipped without attempting a network
request. The affected data names are unbound. Any slide that references an HTTP-bound
variable produces E-EVL-001 (undefined variable) — the offline flag does not suppress
missing-variable errors, it prevents the HTTP request.

## Preconditions

1. `slideforge build` or `slideforge watch` is invoked with the `--offline` flag.
2. The deck contains one or more `@data name from "https://..."` directives.

## Postconditions

1. No HTTP requests are made.
2. HTTP-bound data names are not added to the deck scope.
3. Any `{{ name.field }}` referencing an HTTP-bound source produces E-EVL-001.
4. File-based `@data` sources are unaffected and loaded normally.
5. Build exits with code 2 if there are undefined-variable errors from skipped sources.

## Invariants

1. `--offline` never silently produces empty data — it causes the binding to be absent.
2. File-based data sources are always loaded regardless of `--offline`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Deck with no HTTP sources, `--offline` flag passed | No behavior change; build succeeds normally |
| EC-002 | Deck with HTTP source that is never referenced in slides | No error — the skipped binding is simply absent, no slides reference it |
| EC-003 | Deck with both file and HTTP sources; `--offline` set | File sources loaded; HTTP sources skipped |
| EC-004 | `slideforge watch --offline` | Watch mode starts; HTTP sources not polled; file sources polled normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@data d from "https://api.example.com/data"` / build without `--offline` | HTTP fetch attempted | happy-path |
| Same deck with `--offline` flag | No HTTP fetch; E-EVL-001 for any `{{ d.field }}` reference | offline mode |
| Deck with only file sources, `--offline` | Build completes normally; no network access | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | With `--offline`, zero HTTP requests are made | unit test with mock network asserting no outbound calls |
| VP-TBD | `--offline` does not affect file-based data loading | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 |
| Capability Anchor Justification | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 — the --offline flag is a specified mode of the data binding capability |
| Architecture Module | slideforge-eval crate — data source resolver (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.03.002 — composes with (HTTP source loading — this BC is its offline mode)
- BC-1.02.002 — composes with (undefined variable error triggered when skipped source is referenced)

## Architecture Anchors

- `architecture/authoring-subsystem.md#data-binding` — offline mode flag handling

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
