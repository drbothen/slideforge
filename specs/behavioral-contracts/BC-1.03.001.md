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

# BC-1.03.001: Load @data from JSON/CSV/YAML/TOML File at Compile Time

## Description

The `@data name from "path"` directive loads structured data from a local file at
compile time, before any slide evaluation begins. Supported formats are JSON, CSV,
YAML, and TOML. The loaded data is bound to the declared name and is accessible via
`{{ name.field }}` interpolation throughout the deck.

## Preconditions

1. A `@data name from "path"` directive appears in the deck source.
2. The file at `path` is readable and in a supported format (JSON, CSV, YAML, TOML).
3. The file format is determined by file extension (`.json`, `.csv`, `.yaml`/`.yml`, `.toml`).

## Postconditions

1. The data is parsed into a typed value tree (object, array, or scalar).
2. The bound name is added to the deck-level scope; `{{ name }}` and `{{ name.field }}` are valid.
3. Build exits with code 0 on success.
4. Data is immutable during build — no runtime mutation.

## Invariants

1. All `@data` sources are resolved before any slide evaluation begins.
2. JSON null fields are preserved as null (not empty string, not omitted).
3. CSV rows are loaded as a list of maps keyed by the header row.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File does not exist at declared path | E-DAT-004 with resolved path and source span |
| EC-002 | File extension is `.json` but contents are malformed | E-DAT-003: cannot parse response as JSON |
| EC-003 | CSV with duplicate column headers | E-DAT-003: CSV has duplicate column header '<name>' |
| EC-004 | YAML file uses YAML 1.1 boolean coercion (`YES`, `on`, `no`) | These values are loaded as strings per DI-004; no coercion to bool |
| EC-005 | Empty JSON file (`{}` or `[]`) | Loaded as empty object/array; no error at load time (access errors occur later per BC-1.03.003) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@data kpis from "kpis.json"` / `stat "{{ kpis.revenue }}"` (file has revenue field) | stat rendered with revenue value | happy-path |
| `@data items from "items.csv"` / `@for item in items:` | Slides generated per CSV row | happy-path |
| `@data d from "missing.json"` | E-DAT-004: file not found; exit 2 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Data from all 4 file formats is accessible via dot-notation field access | unit test per format |
| VP-TBD | Missing file always produces E-DAT-004 (never panics) | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 |
| Capability Anchor Justification | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 — local file loading is the primary data binding mechanism |
| L2 Domain Invariants | DI-004 (no implicit type coercion — YAML booleans stay as strings) |
| Architecture Module | slideforge-eval crate — DataSource plugin (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.03.002 — related to (HTTP variant of the same capability)
- BC-1.03.003 — composes with (error contract for missing field access after load)
- BC-1.02.001 — depends on (expression evaluation uses the loaded data)

## Architecture Anchors

- `architecture/system-overview.md#data-binding` — DataSource plugin surface

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
