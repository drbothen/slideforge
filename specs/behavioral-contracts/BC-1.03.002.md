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

# BC-1.03.002: Load @data from HTTP/HTTPS URL at Compile Time

## Description

The `@data name from "https://..."` directive fetches structured data from an HTTP or
HTTPS endpoint at compile time. The response is parsed according to the Content-Type
header (JSON, CSV, YAML) or a declared `format:` hint. HTTP requests are blocked by
default for domains not in the `[data].allowed_domains` allowlist in `slideforge.toml`.

## Preconditions

1. A `@data name from "https://..."` directive appears in the deck source.
2. The URL's domain is in the `[data].allowed_domains` list in `slideforge.toml` (or the list is empty/absent, permitting all domains).
3. Network access is available.
4. `--offline` flag is NOT set.

## Postconditions

1. The HTTP response body is parsed into a typed value tree.
2. The bound name is added to the deck-level scope.
3. Build exits with code 0 on success.

## Invariants

1. HTTP is supported (not only HTTPS) but a lint warning is emitted for non-HTTPS URLs in strict mode.
2. HTTP requests are retried once on network error before producing E-DAT-002.
3. No HTTP response is cached between builds (no implicit stale-serve in v1.0).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Server returns HTTP 404 | E-DAT-001: HTTP 404; exit 2 |
| EC-002 | Network unreachable (OS error) | E-DAT-002: network error; exit 2 |
| EC-003 | Response body is valid JSON but Content-Type is `text/plain` | Attempt JSON parse; if succeeds, warn about missing Content-Type; continue |
| EC-004 | Domain not in `allowed_domains` | E-DAT-006: domain blocked by policy |
| EC-005 | `--offline` flag is set | E-DAT-001 is NOT emitted; HTTP source is skipped per BC-1.03.004 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@data metrics from "https://api.example.com/metrics"` (domain allowed, returns JSON) | Data bound to `metrics`; slides render | happy-path |
| URL returns HTTP 500 | E-DAT-001: HTTP 500; exit 2 | error |
| URL unreachable | E-DAT-002: network error; exit 2 | error |
| Domain not in allowed_domains | E-DAT-006: domain blocked; exit 2 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | HTTP fetch errors always produce E-DAT-001 or E-DAT-002 (never panic) | unit test with mock HTTP server |
| VP-TBD | Domain allowlist check precedes any network request | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 |
| Capability Anchor Justification | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 — HTTP URL loading is explicitly listed in CAP-003 |
| Architecture Module | slideforge-data crate — DataSource plugin (SS-10) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.03.001 — related to (local file variant of the same capability)
- BC-1.03.003 — composes with (missing field access error after HTTP load)
- BC-1.03.004 — composes with (--offline skips HTTP sources)

## Architecture Anchors

- `architecture/system-overview.md` — HTTP DataSource plugin surface

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
