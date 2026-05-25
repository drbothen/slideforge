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

# BC-1.03.005: HTTP Domain Allowlist Enforcement (SSRF Prevention)

## Description

When `[data].allowed_domains` is configured in `slideforge.toml`, any `@data` HTTP/HTTPS
source whose domain is not in the list is blocked before a network request is made and
produces E-DAT-006. This is a fail-closed security control: if the allowlist is present
(even as an empty list `[]`), all HTTP domains are blocked unless explicitly listed.
`file://` data sources are unaffected by the allowlist — the check applies only to HTTP
and HTTPS schemes.

## Preconditions

1. `[data].allowed_domains` is configured in `slideforge.toml` (at least an empty list `[]`).
2. A `@data name from "http://..."` or `@data name from "https://..."` directive appears in the deck source.
3. The URL's domain (host + optional port) is NOT present in the `allowed_domains` list.

## Postconditions

1. No network request is issued to the blocked URL.
2. E-DAT-006 is emitted: `HTTP source '<url>' blocked by allowed_domains policy. Add domain to [data].allowed_domains in slideforge.toml.`
3. Build exits with code 2 in strict mode.
4. In `--warn-only` mode, an error-slide placeholder is rendered for all slides referencing the blocked data source; build continues.

## Invariants

1. The allowlist check is performed BEFORE any DNS resolution or TCP connection is opened — no network traffic reaches a blocked domain.
2. `allowed_domains` absent or not configured means all HTTP domains are permitted (allowlist is opt-in).
3. `allowed_domains: []` (empty list) blocks ALL HTTP/HTTPS sources — no exception for localhost or 127.0.0.1.
4. Domain matching is exact host + optional port: `api.example.com` does NOT match `example.com` or `sub.api.example.com`.
5. `file://` data sources bypass the allowlist entirely — the check applies only to `http://` and `https://` schemes.
6. The check is applied uniformly regardless of whether the URL is literal or the result of `{{ }}` interpolation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `allowed_domains: []` (explicitly empty) with any HTTP source | E-DAT-006: all domains blocked; exit 2 in strict mode |
| EC-002 | `allowed_domains` absent from slideforge.toml entirely | All HTTP sources permitted; no allowlist check performed |
| EC-003 | Domain matches a subdomain but only parent is listed (e.g., `sub.api.example.com` vs `api.example.com`) | E-DAT-006: exact match required; subdomain is NOT permitted |
| EC-004 | Domain matches exactly (e.g., `api.example.com` in list, URL is `https://api.example.com/data`) | Network request proceeds; this BC does not apply |
| EC-005 | URL contains non-standard port (e.g., `api.example.com:8443`) and list only contains `api.example.com` | E-DAT-006: host+port must match exactly — `api.example.com` does NOT cover `api.example.com:8443` |
| EC-006 | URL is a `file://` path; allowlist is configured | Allowlist check is skipped; `file://` source proceeds normally |
| EC-007 | `@data` URL is produced via `{{ }}` interpolation (e.g., `@data x from "https://{{ env.API_HOST }}/data"`) | Interpolated URL is fully resolved first; domain from resolved URL is checked against allowlist |
| EC-008 | --offline flag is set; allowlist is configured; HTTP source is present | BC-1.03.004 governs: source is skipped before allowlist check is reached |
| EC-009 | Multiple HTTP sources declared; one allowed, one blocked | Allowed source loads; blocked source emits E-DAT-006; in strict mode build exits 2; in warn-only error-slide for blocked source only |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `allowed_domains = ["api.example.com"]`, `@data x from "https://api.example.com/data"` | Network request proceeds; data bound to `x` | happy-path (allowed) |
| `allowed_domains = ["api.example.com"]`, `@data x from "https://other.example.com/data"` | E-DAT-006 emitted; no network request; exit 2 | error (blocked) |
| `allowed_domains = []`, `@data x from "https://api.example.com/data"` | E-DAT-006 emitted; no network request; exit 2 | error (empty list blocks all) |
| `allowed_domains` absent, `@data x from "https://api.example.com/data"` | Network request proceeds normally | happy-path (no allowlist) |
| `allowed_domains = ["api.example.com"]`, `@data x from "https://api.example.com:8443/data"` | E-DAT-006 emitted; exact match on host+port required | error (port mismatch) |
| `allowed_domains = ["api.example.com"]`, `@data x from "file://./data.json"` | File source loads; allowlist not consulted | happy-path (file bypass) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | No network request is ever issued for a blocked domain when allowlist is configured | unit test with mock HTTP interceptor — assert zero TCP connections to blocked host |
| VP-TBD | Allowlist check precedes DNS resolution (no dns lookup for blocked domains) | unit test verifying block occurs before any resolver call |
| VP-TBD | Empty `allowed_domains` list blocks all HTTP sources | unit test: `allowed_domains = []` with any HTTP URL → E-DAT-006 |
| VP-TBD | `file://` sources are unaffected by any allowlist configuration | unit test: file source proceeds when `allowed_domains = []` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 |
| Capability Anchor Justification | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 — SSRF prevention via allowlist is a security enforcement mechanism for the HTTP data source feature defined in CAP-003 (HTTP/HTTPS URLs loaded via `@data name from "source"`) |
| L2 Invariants | (none — no existing DI-NNN covers HTTP allowlist enforcement; this BC adds coverage for R-011 security risk) |
| Risk Source | R-011 ("HTTP data source plugin fetches attacker-controlled URLs, potentially enabling SSRF") per risks.md |
| NFR Source | NFR-019 ("HTTP data source URL allowlist enforced: No requests to non-allowlisted domains when allowlist configured") per nfr-catalog.md |
| Error Code | E-DAT-006 per error-taxonomy.md: `HTTP source '<url>' blocked by allowed_domains policy. Add domain to [data].allowed_domains in slideforge.toml.` |
| Architecture Module | slideforge-eval crate — DataSource HTTP plugin (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.03.002 — composes with (HTTP URL loading — this BC adds security enforcement to the same HTTP fetch path)
- BC-1.03.004 — composes with (--offline skips HTTP sources before allowlist check is reached)

## Architecture Anchors

- `architecture/system-overview.md#data-binding` — HTTP DataSource plugin surface
- `architecture/system-overview.md#security` — SSRF prevention, allowlist enforcement point

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
