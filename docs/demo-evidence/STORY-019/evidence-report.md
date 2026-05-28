# Demo Evidence Report — STORY-019

**Story:** HTTP/HTTPS DataSource + SSRF Allowlist (`slideforge-data`)
**Recorded:** 2026-05-27
**Tool:** VHS 0.10.0 (terminal recording) + cargo nextest
**Product type:** Library (Rust) — no CLI surface yet
**Evidence type:** Test-execution recordings showing load-bearing tests pass

---

## Coverage Summary

13 ACs covered (AC-011 is N/A — evaluator responsibility at a higher layer).
All referenced tests pass: 143/143 in `cargo test -p slideforge-data --lib`.

| AC | Status | Recording | Test(s) |
|----|--------|-----------|---------|
| AC-001 | PASS | [AC-001-http-datasource-struct.gif](AC-001-http-datasource-struct.gif) | `test_bc_1_03_002_http_source_id`, `test_bc_1_03_002_http_json_happy_path` |
| AC-002 | PASS | [AC-002-ssrf-blocked-remediation-hint.gif](AC-002-ssrf-blocked-remediation-hint.gif) | `test_bc_1_03_005_ssrf_blocked_no_network`, `test_bc_1_03_005_ssrf_error_contains_remediation_hint` |
| AC-003 | PASS | [AC-003-exact-host-port-match.gif](AC-003-exact-host-port-match.gif) | `test_bc_1_03_005_port_mismatch_blocked`, `test_bc_1_03_005_subdomain_not_matched` |
| AC-004 | PASS | [AC-004-empty-list-absent-allowlist.gif](AC-004-empty-list-absent-allowlist.gif) | `test_bc_1_03_005_empty_list_blocks_all`, `test_bc_1_03_005_absent_allowlist_permits` |
| AC-005 | PASS | [AC-005-content-type-negotiation.gif](AC-005-content-type-negotiation.gif) | `test_bc_1_03_002_http_json_happy_path`, `test_bc_1_03_002_http_csv_happy_path`, `test_bc_1_03_002_http_yaml_happy_path`, `test_bc_1_03_002_content_type_aliases` |
| AC-006 | PASS | [AC-006-4xx-5xx-status-handling.gif](AC-006-4xx-5xx-status-handling.gif) | `test_bc_1_03_002_http_404_error`, `test_bc_1_03_002_http_500_error`, `test_bc_1_03_002_http_500_retried_succeeds` |
| AC-007 | PASS | [AC-007-network-retry-once.gif](AC-007-network-retry-once.gif) | `test_bc_1_03_002_network_error_retries_once` |
| AC-008 | PASS | [AC-008-text-plain-json-warning.gif](AC-008-text-plain-json-warning.gif) | `test_bc_1_03_002_text_plain_warning_emitted` |
| AC-009 | PASS | [AC-009-http-warning-emitted.gif](AC-009-http-warning-emitted.gif) | `test_bc_1_03_002_http_warning_emitted` |
| AC-010 | PASS | [AC-010-file-scheme-rejected.gif](AC-010-file-scheme-rejected.gif) | `test_bc_1_03_002_file_scheme_rejected_with_code`, `test_bc_1_03_002_uppercase_file_scheme_rejected` |
| AC-011 | N/A | — | Evaluator responsibility — URL resolution happens at a layer above `HttpDataSource` |
| AC-012 | PASS | [AC-012-supports-offline.gif](AC-012-supports-offline.gif) | `test_bc_1_03_002_supports_offline_returns_true` |
| AC-013 | PASS | [AC-013-zero-tcp-blocked.gif](AC-013-zero-tcp-blocked.gif) | `test_bc_1_03_005_ssrf_blocked_no_network` (AtomicUsize TCP counter = 0) |
| AC-014 | PASS | [AC-014-crate-lint-attrs.gif](AC-014-crate-lint-attrs.gif) | `cargo clippy -p slideforge-data --all-targets --all-features -- -D warnings` |

---

## AC Detail

### AC-001 — HttpDataSource struct + DataSource impl

**Behavioral contracts:** BC-1.03.002
**Test command:** `cargo nextest run -p slideforge-data -E 'test(http_source_id) | test(http_json_happy_path)'`
**Tests:**
- `http::tests::test_bc_1_03_002_http_source_id` — verifies `source_id()` returns `"http"`
- `http::tests::test_bc_1_03_002_http_json_happy_path` — verifies `fetch()` returns parsed `DataValue` from a JSON endpoint

**Artifacts:**
- [AC-001-http-datasource-struct.gif](AC-001-http-datasource-struct.gif)
- [AC-001-http-datasource-struct.webm](AC-001-http-datasource-struct.webm)
- [AC-001-http-datasource-struct.tape](AC-001-http-datasource-struct.tape)

---

### AC-002 — SSRF blocked + remediation hint

**Behavioral contracts:** BC-1.03.005
**Test command:** `cargo nextest run -p slideforge-data -E 'test(ssrf_blocked_no_network) | test(ssrf_error_contains_remediation_hint)'`
**Tests:**
- `http::tests::test_bc_1_03_005_ssrf_blocked_no_network` — verifies private/loopback hosts are blocked before any TCP connection
- `http::tests::test_bc_1_03_005_ssrf_error_contains_remediation_hint` — verifies error message references `allowed_domains` config key

**Artifacts:**
- [AC-002-ssrf-blocked-remediation-hint.gif](AC-002-ssrf-blocked-remediation-hint.gif)
- [AC-002-ssrf-blocked-remediation-hint.webm](AC-002-ssrf-blocked-remediation-hint.webm)
- [AC-002-ssrf-blocked-remediation-hint.tape](AC-002-ssrf-blocked-remediation-hint.tape)

---

### AC-003 — Exact host[:port] match

**Behavioral contracts:** BC-1.03.005
**Test command:** `cargo nextest run -p slideforge-data -E 'test(port_mismatch_blocked) | test(subdomain_not_matched)'`
**Tests:**
- `allowlist::tests::test_bc_1_03_005_port_mismatch_blocked` — `api.example.com:443` in allowlist does NOT permit `api.example.com:8080`
- `allowlist::tests::test_bc_1_03_005_subdomain_not_matched` — `example.com` in allowlist does NOT permit `sub.example.com`

**Artifacts:**
- [AC-003-exact-host-port-match.gif](AC-003-exact-host-port-match.gif)
- [AC-003-exact-host-port-match.webm](AC-003-exact-host-port-match.webm)
- [AC-003-exact-host-port-match.tape](AC-003-exact-host-port-match.tape)

---

### AC-004 — Empty list blocks all / absent permits all

**Behavioral contracts:** BC-1.03.005
**Test command:** `cargo nextest run -p slideforge-data -E 'test(empty_list_blocks_all) | test(absent_allowlist_permits)'`
**Tests:**
- `allowlist::tests::test_bc_1_03_005_empty_list_blocks_all` — `allowed_domains: []` blocks all hosts
- `http::tests::test_bc_1_03_005_absent_allowlist_permits` — no `allowed_domains` config = permit-all (open by default)

**Artifacts:**
- [AC-004-empty-list-absent-allowlist.gif](AC-004-empty-list-absent-allowlist.gif)
- [AC-004-empty-list-absent-allowlist.webm](AC-004-empty-list-absent-allowlist.webm)
- [AC-004-empty-list-absent-allowlist.tape](AC-004-empty-list-absent-allowlist.tape)

---

### AC-005 — Content-Type negotiation

**Behavioral contracts:** BC-1.03.002
**Test command:** `cargo nextest run -p slideforge-data -E 'test(http_json_happy_path) | test(http_csv_happy_path) | test(http_yaml_happy_path) | test(content_type_aliases)'`
**Tests:**
- `test_bc_1_03_002_http_json_happy_path` — `application/json` dispatches JSON parser
- `test_bc_1_03_002_http_csv_happy_path` — `text/csv` dispatches CSV parser
- `test_bc_1_03_002_http_yaml_happy_path` — `application/yaml` dispatches YAML parser
- `test_bc_1_03_002_content_type_aliases` — `application/x-yaml`, `text/x-yaml`, `application/x-csv` all accepted

**Artifacts:**
- [AC-005-content-type-negotiation.gif](AC-005-content-type-negotiation.gif)
- [AC-005-content-type-negotiation.webm](AC-005-content-type-negotiation.webm)
- [AC-005-content-type-negotiation.tape](AC-005-content-type-negotiation.tape)

---

### AC-006 — 4xx/5xx status error handling

**Behavioral contracts:** BC-1.03.002
**Test command:** `cargo nextest run -p slideforge-data -E 'test(http_404_error) | test(http_500_error) | test(http_500_retried_succeeds)'`
**Tests:**
- `test_bc_1_03_002_http_404_error` — 404 response returns `DataError::HttpStatus { code: 404 }`
- `test_bc_1_03_002_http_500_error` — persistent 500 returns error after retry exhausted
- `test_bc_1_03_002_http_500_retried_succeeds` — 500 on first attempt, 200 on retry = success

**Artifacts:**
- [AC-006-4xx-5xx-status-handling.gif](AC-006-4xx-5xx-status-handling.gif)
- [AC-006-4xx-5xx-status-handling.webm](AC-006-4xx-5xx-status-handling.webm)
- [AC-006-4xx-5xx-status-handling.tape](AC-006-4xx-5xx-status-handling.tape)

---

### AC-007 — Network retry once

**Behavioral contracts:** BC-1.03.002
**Test command:** `cargo nextest run -p slideforge-data -E 'test(network_error_retries_once)'`
**Tests:**
- `test_bc_1_03_002_network_error_retries_once` — connection error on attempt 1, success on attempt 2; counter verified exactly 2 calls

**Artifacts:**
- [AC-007-network-retry-once.gif](AC-007-network-retry-once.gif)
- [AC-007-network-retry-once.webm](AC-007-network-retry-once.webm)
- [AC-007-network-retry-once.tape](AC-007-network-retry-once.tape)

---

### AC-008 — text/plain JSON + warning

**Behavioral contracts:** BC-1.03.002
**Test command:** `cargo nextest run -p slideforge-data -E 'test(text_plain_warning_emitted)'`
**Tests:**
- `test_bc_1_03_002_text_plain_warning_emitted` — `text/plain` body that is valid JSON: data parsed successfully + `tracing::warn!` emitted via `tracing-test`

**Artifacts:**
- [AC-008-text-plain-json-warning.gif](AC-008-text-plain-json-warning.gif)
- [AC-008-text-plain-json-warning.webm](AC-008-text-plain-json-warning.webm)
- [AC-008-text-plain-json-warning.tape](AC-008-text-plain-json-warning.tape)

---

### AC-009 — HTTP warning emitted

**Behavioral contracts:** BC-1.03.002
**Test command:** `cargo nextest run -p slideforge-data -E 'test(http_warning_emitted)'`
**Tests:**
- `test_bc_1_03_002_http_warning_emitted` — plain `http://` URL (not HTTPS) emits `tracing::warn!` diagnostic via `tracing-test`

**Artifacts:**
- [AC-009-http-warning-emitted.gif](AC-009-http-warning-emitted.gif)
- [AC-009-http-warning-emitted.webm](AC-009-http-warning-emitted.webm)
- [AC-009-http-warning-emitted.tape](AC-009-http-warning-emitted.tape)

---

### AC-010 — file:// scheme rejected

**Behavioral contracts:** BC-1.03.002
**Test command:** `cargo nextest run -p slideforge-data -E 'test(file_scheme_rejected_with_code) | test(uppercase_file_scheme_rejected)'`
**Tests:**
- `test_bc_1_03_002_file_scheme_rejected_with_code` — `file:///etc/passwd` returns `DataError::ForbiddenScheme { scheme: "file" }`
- `test_bc_1_03_002_uppercase_file_scheme_rejected` — `FILE:///etc/passwd` also rejected (case-insensitive scheme check)

**Artifacts:**
- [AC-010-file-scheme-rejected.gif](AC-010-file-scheme-rejected.gif)
- [AC-010-file-scheme-rejected.webm](AC-010-file-scheme-rejected.webm)
- [AC-010-file-scheme-rejected.tape](AC-010-file-scheme-rejected.tape)

---

### AC-011 — Resolved URL (N/A at this layer)

**Status:** Not applicable. URL resolution (variable interpolation, relative-to-base) is the evaluator's responsibility. `HttpDataSource` receives a fully-resolved URL and does not implement this logic.

---

### AC-012 — supports_offline returns true

**Behavioral contracts:** BC-1.03.002
**Test command:** `cargo nextest run -p slideforge-data -E 'test(supports_offline_returns_true)'`
**Tests:**
- `test_bc_1_03_002_supports_offline_returns_true` — `HttpDataSource::supports_offline()` returns `true` (allows cached/stub data in offline builds)

**Artifacts:**
- [AC-012-supports-offline.gif](AC-012-supports-offline.gif)
- [AC-012-supports-offline.webm](AC-012-supports-offline.webm)
- [AC-012-supports-offline.tape](AC-012-supports-offline.tape)

---

### AC-013 — Zero TCP for blocked hosts

**Behavioral contracts:** BC-1.03.005
**Test command:** `cargo nextest run -p slideforge-data -E 'test(ssrf_blocked_no_network)'`
**Tests:**
- `test_bc_1_03_005_ssrf_blocked_no_network` — uses `AtomicUsize` TCP connection counter; asserts counter remains 0 after a blocked fetch (no network contact made)

**Artifacts:**
- [AC-013-zero-tcp-blocked.gif](AC-013-zero-tcp-blocked.gif)
- [AC-013-zero-tcp-blocked.webm](AC-013-zero-tcp-blocked.webm)
- [AC-013-zero-tcp-blocked.tape](AC-013-zero-tcp-blocked.tape)

---

### AC-014 — Crate-level lint attributes

**Behavioral contracts:** BC-1.03.002 (implementation quality gate)
**Test command:** `cargo clippy -p slideforge-data --all-targets --all-features -- -D warnings`
**Evidence:** Clippy exits 0 with `Finished` — confirms `#![forbid(unsafe_code)]`, `#![warn(clippy::pedantic)]`, and `#![warn(missing_docs)]` are active and clean.

**Artifacts:**
- [AC-014-crate-lint-attrs.gif](AC-014-crate-lint-attrs.gif)
- [AC-014-crate-lint-attrs.webm](AC-014-crate-lint-attrs.webm)
- [AC-014-crate-lint-attrs.tape](AC-014-crate-lint-attrs.tape)

---

## Full Test Suite

**Verification command:** `cargo test -p slideforge-data --lib`
**Result:** `test result: ok. 143 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s`

All 143 library tests pass, including the 53 tests in `http::tests` and `allowlist::tests` directly covering STORY-019 ACs.
