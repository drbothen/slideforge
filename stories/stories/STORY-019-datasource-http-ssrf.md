---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-019
title: "DataSource: HTTP/HTTPS + SSRF Allowlist"
epic: EPIC-05
wave: 3
points: 8
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-data
subsystems: [SS-10]
target_module: slideforge-data
behavioral_contracts: [BC-1.03.002, BC-1.03.005]
verification_properties: []
nfr_refs: [NFR-019, NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-018
blocks:
  - STORY-021
estimated_days: 3
---

# STORY-019: DataSource: HTTP/HTTPS + SSRF Allowlist

## Summary

Implement the `DataSource` trait for HTTP/HTTPS URL-based data loading and the
SSRF-prevention allowlist enforcement mechanism in `slideforge-data`. The `HttpDataSource`
fetches a URL at compile time using `ureq = "=2.12.1"` (synchronous HTTP), parses the
response body as JSON/CSV/YAML based on `Content-Type` or an explicit `format:` hint,
and returns a `Value` tree. The allowlist check — using `[data].allowed_domains` in
`slideforge.toml` — MUST occur before any DNS resolution or TCP connection is opened.
One retry on network error before producing `E-DAT-002`.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,500 |
| `crates/slideforge-data/src/http.rs` | ~3,000 |
| `crates/slideforge-data/src/allowlist.rs` | ~2,000 |
| Test code (mock HTTP) | ~4,000 |
| BC files consulted (BC-1.03.002, BC-1.03.005) | ~2,000 |
| **Total** | **~15,500** |

Agent context budget: 200k tokens. This story is ~7.75% of budget — within limit.

## Acceptance Criteria

- [ ] **AC-001:** `HttpDataSource` struct implements the `DataSource` trait. Fields: `url: Arc<str>`, `format_hint: Option<DataFormat>`. The `load()` method performs the allowlist check first, then issues the HTTP request.
  (traces to BC-1.03.002 postcondition 1 — HTTP response body parsed into typed value tree)

- [ ] **AC-002:** When `[data].allowed_domains` is configured in `slideforge.toml` and the URL's host is NOT in the list, `DataError::SsrfBlocked { url: Arc<str>, domain: Arc<str>, span: SourceSpan }` is returned, mapping to `E-DAT-006: HTTP source '<url>' blocked by allowed_domains policy.`. Zero network traffic is sent.
  (traces to BC-1.03.005 postcondition 1 — no network request to blocked URL; and postcondition 2 — E-DAT-006 emitted)

- [ ] **AC-003:** The allowlist check is performed BEFORE any DNS resolution. The implementation checks the parsed `url::Host` component against the `allowed_domains` list using exact string matching (no glob, no wildcard). Domain matching is exact host + optional port: `api.example.com` does NOT match `example.com` or `sub.api.example.com` or `api.example.com:8443`.
  (traces to BC-1.03.005 invariant 1 — check before DNS; invariant 4 — exact host+port match)

- [ ] **AC-004:** When `allowed_domains` is absent from `slideforge.toml` (not configured), all HTTP/HTTPS domains are permitted and no allowlist check is performed. When `allowed_domains = []` (explicitly empty list), ALL HTTP/HTTPS sources are blocked regardless of URL.
  (traces to BC-1.03.005 invariant 2 — absent list permits all; invariant 3 — empty list blocks all; edge cases EC-001 and EC-002)

- [ ] **AC-005:** Successful HTTP 2xx response with `Content-Type: application/json` body is parsed via `parse_json()` from STORY-018. Successful response with `Content-Type: text/csv` is parsed via `parse_csv()`. `Content-Type: application/x-yaml` or `text/yaml` is parsed via `parse_yaml()`. If `Content-Type` is unrecognized and no `format_hint` is given, `DataError::ParseError` with `E-DAT-003` is returned.
  (traces to BC-1.03.002 postcondition 1 — typed value tree from response)

- [ ] **AC-006:** HTTP 4xx or 5xx response produces `DataError::HttpError { url: Arc<str>, status: u16, span: SourceSpan }` mapping to `E-DAT-001: HTTP <status>; exit 2`. HTTP 4xx is not retried. HTTP 5xx may be retried once (one retry policy).
  (traces to BC-1.03.002 edge case EC-001 — HTTP 404 → E-DAT-001)

- [ ] **AC-007:** Network unreachable / OS-level error produces one retry. If the retry also fails, `DataError::NetworkError { url: Arc<str>, cause: Arc<str>, span: SourceSpan }` is returned, mapping to `E-DAT-002: network error; exit 2`.
  (traces to BC-1.03.002 invariant 2 — retry once on network error; edge case EC-002)

- [ ] **AC-008:** `Content-Type: text/plain` response body that parses successfully as JSON is accepted with a lint warning emitted via `tracing::warn!`. The warning text is: `HTTP response from '<url>' has Content-Type: text/plain but parsed as JSON. Consider requesting application/json.`
  (traces to BC-1.03.002 edge case EC-003 — mismatched content-type handled gracefully)

- [ ] **AC-009:** HTTP (non-HTTPS) URLs are accepted but emit `tracing::warn!` in strict mode: `HTTP source '<url>' is non-HTTPS. Prefer HTTPS for data sources in production.` This is a cosmetic warning, not an error.
  (traces to BC-1.03.002 invariant 1 — HTTP supported with lint warning)

- [ ] **AC-010:** `file://` scheme data sources bypass the allowlist entirely. The allowlist check is ONLY applied to `http://` and `https://` schemes. If a `file://` URL somehow reaches `HttpDataSource`, it must be rejected with `DataError::UnsupportedFormat` (not silently proceed).
  (traces to BC-1.03.005 invariant 5 — file:// bypasses allowlist; edge case EC-006)

- [ ] **AC-011:** When a `@data` URL is produced via `{{ }}` interpolation, the interpolated URL is fully resolved first by the evaluator, and the resolved URL's domain is checked against the allowlist. The `HttpDataSource` receives the already-resolved URL — it does not perform interpolation itself.
  (traces to BC-1.03.005 invariant 6 — check applied to resolved URL regardless of how it was constructed)

- [ ] **AC-012:** `HttpDataSource::supports_offline()` returns `true`, indicating that HTTP sources SHOULD be skipped in offline mode. The actual offline gate is in `DataSourceContext` (STORY-021); this method is the trait-level signal.
  (traces to BC-1.03.002 edge case EC-005 — offline flag causes HTTP source to be skipped per BC-1.03.004)

- [ ] **AC-013:** NFR-019 is met: unit tests with a mock HTTP interceptor assert zero TCP connections to blocked domains when the allowlist is configured.
  (traces to BC-1.03.005 — no network request to blocked domain)

- [ ] **AC-014:** `#![forbid(unsafe_code)]` (NFR-024), `#![warn(missing_docs)]` (NFR-023), `clippy::pedantic` clean (NFR-022), `=` version pinning (NFR-025) all apply to new code in this story.

## Previous Story Intelligence

Continues from STORY-018. Key context:

- `DataError`, `DataFormat`, `parse_json()`, `parse_csv()`, `parse_yaml()` are already defined in STORY-018. This story adds `DataError::HttpError`, `DataError::NetworkError`, `DataError::SsrfBlocked` to the existing enum.
- `DataSourceContext` struct already exists from STORY-018. Add an `allowed_domains: Option<Vec<Arc<str>>>` field to it.
- The parse modules from STORY-018 (`parse_json`, `parse_csv`, `parse_yaml`) are reused here — `HttpDataSource` dispatches to them after receiving the response body bytes.

## Architecture Compliance Rules

1. **SS-10 Effectful — HTTP is the primary I/O:** `slideforge-data` is permitted to make synchronous HTTP calls. Use `ureq = "=2.12.1"` (synchronous, no async runtime). No `tokio`, no `reqwest`, no async.
2. **Allowlist check owns zero network state:** The allowlist check is a pure function `fn is_allowed(domain: &str, port: Option<u16>, allowed: &[Arc<str>]) -> bool`. No network state is inspected.
3. **Forbidden dependencies:** Same as STORY-018. No `slideforge-eval`, no `slideforge-syntax`, no exporter crates.
4. **Security gate is fail-closed:** When `allowed_domains` is `Some([])` (explicitly present but empty), the correct behavior is to block ALL domains. An absent key (`None`) permits all. Never mix these two behaviors.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `ureq` | `=2.12.1` | Synchronous HTTP/HTTPS client (no async runtime dependency) |
| `url` | `=2.5` | URL parsing for domain extraction before allowlist check |
| `thiserror` | `=2.0.18` | Extended `DataError` variants |
| All from STORY-018 | see STORY-018 | Reused |

No new dev dependencies needed (use `wiremock = "=0.6.5"` for mock HTTP server in tests, or implement a simple TCP listener in tests that returns canned HTTP responses).

Alternative to wiremock for a lighter approach: `httptest = "=0.15"` — any mock HTTP crate that supports intercepting requests without real network calls is acceptable.

Dev dependencies:
- `wiremock = "=0.6.5"` or `httptest = "=0.15"` — for mock HTTP server in unit tests (pick one and note the choice)

NOTE: wiremock requires tokio runtime. Add `tokio = { version = "=1.38.0", features = ["full"] }` as a dev-dependency for tests only. Alternatively, use a simple TCP listener approach if tokio dev-dep is undesirable.

## File Structure Requirements

Files to create (additions to STORY-018 structure):

```
crates/slideforge-data/src/
├── http.rs               # HttpDataSource struct + DataSource impl
├── allowlist.rs          # AllowlistConfig struct + is_allowed() function
```

Files to modify:

```
crates/slideforge-data/src/
├── error.rs              # Add: HttpError, NetworkError, SsrfBlocked variants
├── context.rs            # Add: allowed_domains: Option<Vec<Arc<str>>> field
├── lib.rs                # Add: pub mod http; pub mod allowlist;
```

## Tasks

1. **Extend `error.rs`** — add `HttpError`, `NetworkError`, `SsrfBlocked` variants to `DataError`. Add `E_DAT_001`, `E_DAT_002`, `E_DAT_006` constants. (15 min)
2. **Extend `context.rs`** — add `allowed_domains: Option<Vec<Arc<str>>>` to `DataSourceContext`. `None` = no allowlist (all domains permitted). `Some(vec![])` = all domains blocked. (10 min)
3. **Write `src/allowlist.rs`** — `AllowlistConfig { domains: Option<Vec<Arc<str>>> }`. Implement `is_allowed(url: &url::Url, config: &AllowlistConfig) -> bool`. Extract `host_str()` + optional port. Exact match against each entry in `domains`. Return `true` if `domains` is `None` (no allowlist). Return `false` for all entries if `domains` is `Some(vec![])`. (30 min)
4. **Write `src/http.rs`** — `HttpDataSource { url: Arc<str>, format_hint: Option<DataFormat> }`. Implement `DataSource`:
   - Parse URL via `url::Url::parse()`.
   - Call `is_allowed()` — return `SsrfBlocked` if blocked.
   - Check `ctx.offline` — if true, return `DataError::Offline` (new variant, consumed by STORY-021's gate).
   - Issue `ureq::get(url).call()` — on network error, retry once, then `NetworkError`.
   - On 4xx/5xx → `HttpError { status }`.
   - Read response body bytes.
   - Dispatch to `parse_json/csv/yaml` based on Content-Type or `format_hint`.
   - Emit `tracing::warn!` for HTTP (non-HTTPS) URLs.
   - Emit `tracing::warn!` for `text/plain` Content-Type that parses as JSON. (60 min)
5. **Write unit tests for `allowlist.rs`** — all 6 canonical test vectors from BC-1.03.005 (see Test Strategy). (30 min)
6. **Write unit tests for `http.rs`** — using mock HTTP server for happy path, 404, network error, blocked domain (see Test Strategy). (45 min)
7. **Run `cargo clippy -p slideforge-data -- -D warnings`** — fix all warnings. (15 min)
8. **Run `cargo test -p slideforge-data`** — all tests pass. (10 min)

## Test Strategy

### Unit tests for `allowlist.rs`

Test every canonical vector from BC-1.03.005:

| Test Name | Setup | Expected |
|-----------|-------|----------|
| `test_allowed_domain_passes` | domains = `["api.example.com"]`, URL = `https://api.example.com/data` | `is_allowed()` → `true` |
| `test_blocked_domain` | domains = `["api.example.com"]`, URL = `https://other.example.com/data` | `is_allowed()` → `false` |
| `test_empty_list_blocks_all` | domains = `[]`, URL = `https://api.example.com/data` | `is_allowed()` → `false` |
| `test_absent_allowlist_permits_all` | domains = `None`, URL = `https://api.example.com/data` | `is_allowed()` → `true` |
| `test_port_mismatch_blocked` | domains = `["api.example.com"]`, URL = `https://api.example.com:8443/data` | `is_allowed()` → `false` (host:port must match) |
| `test_file_scheme_bypasses` | Any `AllowlistConfig`, URL = `file:///./data.json` | Not handled by `is_allowed` (caller checks scheme first) |
| `test_subdomain_not_matched` | domains = `["api.example.com"]`, URL = `https://sub.api.example.com/data` | `is_allowed()` → `false` (exact match required) |

### Unit tests for `http.rs` (using mock HTTP server)

| Test Name | Mock Setup | Expected |
|-----------|-----------|----------|
| `test_http_json_happy_path` | Mock returns 200 with `Content-Type: application/json`, `{"count": 5}` | `Value::Map` with `count: Value::Int(5)` |
| `test_http_404_error` | Mock returns 404 | `DataError::HttpError { status: 404 }` |
| `test_network_unreachable_retries` | Mock rejects first connection; accepts second; returns 200 | Success on retry |
| `test_ssrf_blocked_before_network` | AllowlistConfig with `["other.com"]`, URL `https://api.example.com/` | `DataError::SsrfBlocked` before any connection attempt |
| `test_http_warning_emitted` | URL is `http://` (not `https://`) | 200 response + warning captured in tracing output |

Zero-TCP assertion for SSRF test: use `std::net::TcpListener::bind("127.0.0.1:0")` to get a free port, then point the blocked URL to `127.0.0.1:<port>`. After the test, assert that no bytes were received on the listener (connection count = 0). This satisfies NFR-019.

## Dependencies

**Depends on:**
- STORY-018 (DataSource File Formats) — provides `DataError`, `DataFormat`, parse functions, `DataSourceContext`. STORY-019 extends these.

Dependency justification: STORY-019 depends on STORY-018 because the HTTP data source shares the `DataError` enum, `DataFormat` format dispatching, and parse functions (`parse_json`, `parse_csv`, `parse_yaml`) all defined in STORY-018.

**Blocks:**
- STORY-021 (offline mode) — offline gate is applied at the `DataSourceContext` level, which requires `HttpDataSource::supports_offline()` = `true` from this story.

## Implementation Notes

### ureq vs reqwest

Use `ureq = "=2.12.1"` (synchronous). Do NOT use `reqwest` (async). The build pipeline runs synchronously. An async runtime would add `tokio` or `async-std` as a transitive dependency and significantly increase compile times, binary size, and complexity. `ureq` is purpose-built for synchronous CLI tools.

### Retry Policy

One retry means: on `ureq::Error::Io` (OS/network error), wait 0ms (no sleep — the build pipeline is not a server) and retry once. If the retry also fails, return `DataError::NetworkError`. Do NOT retry on 4xx or 5xx HTTP responses.

### URL Parsing for Allowlist

Use `url = "=2.5"` crate. Extract `url.host_str()` and `url.port()`:

```rust
let host = url.host_str().ok_or_else(|| DataError::ParseError { ... })?;
let port = url.port(); // Option<u16>
let domain_key = match port {
    Some(p) => format!("{}:{}", host, p),
    None => host.to_string(),
};
allowed_domains.iter().any(|d| d.as_ref() == domain_key)
```

This naturally handles the port-mismatch edge case: `api.example.com` vs `api.example.com:8443`.

### Content-Type Negotiation

Parse `Content-Type` header value (strip parameters like `; charset=utf-8`):

```rust
let ct = response.header("Content-Type").unwrap_or("").split(';').next().unwrap_or("").trim();
let format = match ct {
    "application/json" | "text/json" => DataFormat::Json,
    "text/csv" | "application/csv" => DataFormat::Csv,
    "application/x-yaml" | "text/yaml" | "application/yaml" => DataFormat::Yaml,
    "text/plain" => {
        tracing::warn!("HTTP response from '{}' has Content-Type: text/plain ...", url);
        // attempt JSON, then CSV, then return ParseError if both fail
        DataFormat::Json // try json first
    }
    _ => self.format_hint.ok_or_else(|| DataError::ParseError { ... })?,
};
```

### No HTTP Caching

BC-1.03.002 invariant 3: `No HTTP response is cached between builds`. `ureq` does not cache by default. Do not add any caching layer.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Server returns HTTP 404 | `DataError::HttpError { status: 404 }` → E-DAT-001 |
| EC-002 | Network unreachable | Retry once → `DataError::NetworkError` → E-DAT-002 |
| EC-003 | `text/plain` Content-Type body parses as JSON | Accept + `tracing::warn!` |
| EC-004 | Domain not in `allowed_domains` | `DataError::SsrfBlocked` → E-DAT-006; zero TCP connections |
| EC-005 | `--offline` flag set | Handled by STORY-021 offline gate; `supports_offline()` = `true` signals the gate |
| EC-006 | `allowed_domains = []` (empty list) | All HTTP/HTTPS blocked → E-DAT-006 |
| EC-007 | `allowed_domains` absent from config | All domains permitted; no check performed |
| EC-008 | Domain exact match passes; subdomain does not | `api.example.com` allows only that host, not `sub.api.example.com` |
| EC-009 | Port mismatch | `api.example.com` does NOT allow `api.example.com:8443` |
| EC-010 | Multiple HTTP sources; one allowed, one blocked | Allowed one loads; blocked one emits `E-DAT-006`; in strict mode build exits 2 |
