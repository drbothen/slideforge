---
document_type: dtu-assessment
level: L3
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
phase: 1b
DTU_REQUIRED: false
inputs:
  - .factory/specs/architecture/dependency-graph.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/export-architecture.md
  - .factory/specs/behavioral-contracts/BC-1.03.002.md
traces_to: .factory/specs/architecture/ARCH-INDEX.md
---

# DTU Assessment: slideforge v1.0

## Verdict

**DTU_REQUIRED: false**

slideforge is a local CLI tool. It has zero fixed external service dependencies. All
network I/O it performs at build time is initiated by the user's `.sf` source file
(the `@data from "https://..."` directive) against user-owned endpoints. There are no
slideforge-owned service integrations, no auth providers, no SaaS APIs, no cloud
databases, and no telemetry export targets that require behavioral cloning.

---

## Summary

| Metric | Value |
|--------|-------|
| External dependencies identified | 0 (service dependencies) |
| User-directed HTTP fetches | 1 mechanism (user data source, not a service dep) |
| DTU clones recommended | 0 |
| Total clone story points | 0 |
| Estimated Wave 1 capacity needed | 0 points |

---

## Integration Surface Inventory

### Inbound Data Sources (External → Product)

None identified — rationale: slideforge does not poll any external API on its own
behalf. The `@data from "https://..."` directive in BC-1.03.002 fetches data that the
user's deck specification points to. The URL, domain allowlist, and data format are
entirely under the user's control. slideforge is the transport vehicle, not the
consumer of a fixed service. Tests for this mechanism use a mock HTTP server
(unit-test scope) — no DTU clone required.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| — | — | — | — | — | No inbound service dependencies identified |

### Outbound Operations (Product → External)

None identified — rationale: slideforge writes only to the local filesystem (output
`.pptx`, `.pdf`, `.docx`, `.html` files). It does not call notification APIs, payment
gateways, ticketing systems, or any outbound service. The `reqwest` HTTP client in
`slideforge-data` is used exclusively for the user-directed `@data` fetches described
above, not for slideforge-initiated outbound calls.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| — | — | — | — | — | No outbound service dependencies identified |

### Identity & Access (Bidirectional — auth flow)

None identified — rationale: slideforge has no authentication layer. There are no
OAuth providers, API key managers, credential stores, or SSO flows. The domain
allowlist in `slideforge.toml` (`[data].allowed_domains`) is a local configuration
file, not a remote identity service.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| — | — | — | — | — | No auth/identity service dependencies identified |

### Persistence & State (Product ↔ Storage)

None identified — rationale: slideforge uses two local storage mechanisms:

1. `rusqlite` (via `slideforge-data`) — reads local SQLite files the user provides as
   data sources. This is local file I/O, not a database service.
2. `calamine` (via `slideforge-data`) — reads local Excel `.xlsx` files. Local file
   I/O only.

There is no external database, cache, object store, or message queue that slideforge
connects to as a service.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| — | — | — | — | — | No external persistence service dependencies identified |

### Observability & Export (Product → Monitoring)

None identified — rationale: slideforge uses `tracing` (structured logging) and
`tracing-subscriber` for instrumentation. These write to local stderr/stdout and
optionally to local files. There is no configured export to a remote log aggregator,
metrics platform, or tracing backend in v1.0. The quality bar specifies
"opentelemetry-compatible export hooks" but these are hooks (plugin surfaces), not
active service connections. No service integration exists that requires a DTU clone.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| — | — | — | — | — | No observability export service dependencies identified |

### Enrichment & Lookup (External → Product, on-demand)

None identified — rationale: slideforge does not call threat intelligence feeds,
geocoding services, pricing services, CVE databases, or any other external enrichment
service during compilation. `cargo audit` and `cargo deny` run in CI pipelines under
human/devops control — they are not runtime dependencies of the slideforge binary.

| # | Service | Protocol | Fidelity | DTU? | Justification |
|---|---------|----------|----------|------|---------------|
| — | — | — | — | — | No enrichment/lookup service dependencies identified |

---

## Dependency Summary

No external service dependencies. No DTU clones required.

| # | Service | Category | Fidelity | DTU? | Points | Justification |
|---|---------|----------|----------|------|--------|---------------|
| — | (none) | — | — | No | 0 | slideforge is a self-contained local CLI tool |

---

## Services NOT Requiring DTU

| # | Service / Mechanism | Reason |
|---|---------------------|--------|
| 1 | User `@data from "https://..."` HTTP fetches | User-directed, not a slideforge service dependency. Tests use in-process mock HTTP server (no external service clone needed). |
| 2 | Local SQLite via rusqlite | Local file I/O; no database service. |
| 3 | Local Excel via calamine | Local file I/O; no external service. |
| 4 | axum web preview server | Embedded local server (outbound: none; inbound: browser on localhost). Not an external service. |

---

## Rationale Summary

slideforge's deployment topology is `single-service` (one CLI binary, one tech stack).
It has a deliberate offline-by-default design — the `--offline` flag in BC-1.03.004
exists precisely because even user-directed HTTP fetches are optional.

The only network-capable component is `reqwest` in `slideforge-data`, used exclusively
to fulfill user-declared `@data` directives. These are not slideforge's service
dependencies; they are user-owned data endpoints. Testing this mechanism requires a
mock HTTP server (addressable in unit tests with an in-process `axum` fixture), not a
DTU behavioral clone of a real third-party service.

All six DTU integration surface categories are empty for slideforge v1.0.

---

## Testing Strategy for Network-Capable Components

Because no DTU clones are needed, the `reqwest`-based HTTP data source is tested via:

1. **Unit tests** in `slideforge-data` using a test-local axum server (loopback, no
   external network).
2. **Offline-mode unit tests** verifying that `--offline` correctly skips HTTP sources
   (BC-1.03.004).
3. **Domain-allowlist unit tests** verifying that requests to non-allowlisted domains
   are blocked before any socket is opened (VP covering BC-1.03.002 postcondition 2).
4. **Error path unit tests** covering EC-001 through EC-005 from BC-1.03.002 using the
   same in-process mock server.

This strategy provides equivalent coverage to a DTU clone without the operational
overhead of a separate Docker service.
