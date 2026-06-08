---
story: STORY-080
phase: red-gate
agent: test-writer
date: 2026-06-08
---

# Red Gate Log — STORY-080 Deflake Cross-Platform Tests

## Summary

Two new test artifacts were written for STORY-080. The Red Gate status differs
between the two flaky tests and is documented individually below.

---

## Fix 1 — `slideforge-data`: Deterministic 4xx-not-retried harness

### New test

`http::tests::test_BC_1_03_002_http_4xx_not_retried_deterministic_harness`

File: `crates/slideforge-data/src/http.rs` (inside `#[cfg(test)] mod tests`)

### New stub helper

`spawn_deterministic_counting_server_404(_max_connections: usize)` — stubbed
with `todo!("STORY-080: implement deterministic mpsc-channel-based 404 counting
server ...")`. Returns `(SocketAddr, Receiver<usize>, JoinHandle<()>)`.

### Red Gate result

**FAIL (correct Red Gate).**

```
thread '...test_BC_1_03_002_http_4xx_not_retried_deterministic_harness' panicked at
crates/slideforge-data/src/http.rs:2085:
not yet implemented: STORY-080: implement deterministic mpsc-channel-based 404
counting server (replaces set_nonblocking poll loop; AC-001 fix)
```

The test panics because `spawn_deterministic_counting_server_404` is a `todo!()`
stub. The implementer must replace the stub body with the real mpsc-channel-based
server design specified in AC-001. Once the helper is implemented, the test
exercises the full behavioral assertion: `connection_count == 1` (no retry on 4xx).

### What the implementer must do

Replace `todo!()` in `spawn_deterministic_counting_server_404` with:
- `TcpListener::bind("127.0.0.1:0")` in blocking mode (NO `set_nonblocking`)
- Spawn a thread that accepts up to `max_connections` connections (blocking accept)
- On each connection: `counter.fetch_add(1, ...)`, read the request, write 404
- After serving all connections (or after a hard deadline), send the final count
  via `mpsc::Sender<usize>` and let the thread return
- Return `(addr, receiver, handle)` to the caller

The existing test `test_bc_1_03_002_http_4xx_not_retried` (the flaky one) is NOT
deleted by this story — the implementer's next step is to DELETE or REPLACE it
with `test_BC_1_03_002_http_4xx_not_retried_deterministic_harness` once the
new harness is green. Or the implementer may rewrite the existing test body
directly and rename the new helper into the existing test function.

---

## Fix 2 — `slideforge-diagrams`: cold_budget companion correctness test

### New test

`test_BC_1_12_003_cold_render_correctness`

File: `crates/slideforge-diagrams/tests/cold_budget.rs`

### Red Gate result

**PASS (expected — see rationale below).**

The companion correctness test passes against the current production code because
`DiagramRendererImpl::render_diagram` already works correctly. This is not a
vacuously true test — it exercises the production render path end-to-end and
asserts `is_ok()` and `!is_empty()`.

### Why no `todo!()` stub Red Gate for cold_budget

The cold_budget deflake (STORY-080 AC-002) is Option A: add `#[ignore = "..."]`
to `test_cold_budget_under_200ms` and add a companion correctness test. The
companion test has no "new production helper" to stub — it calls the existing
`render_diagram` directly.

The `#[ignore]` annotation change has no driving test at the unit-test level
(it is a test-attribute change, not a production-code change). This is a known
limitation of TDD for test-infrastructure refactors: the change is simple
(one annotation), well-bounded, and covered by the companion correctness test
proving non-vacuity.

The Red Gate for STORY-080 overall is provided by the http harness test above.
The cold_budget companion test is load-bearing future coverage (it prevents a
regression if someone inadvertently removes the correctness assertion when
applying the `#[ignore]`).

### What the implementer must do for cold_budget

1. Add `#[ignore = "timing-sensitive NFR-003 gate — run with: cargo nextest run \
   -p slideforge-diagrams -- --include-ignored cold_budget"]` to
   `test_cold_budget_under_200ms`.
2. Add a code comment above the `#[ignore]` explaining that this is a timing gate,
   not a correctness gate, and that `test_BC_1_12_003_cold_render_correctness`
   covers correctness.
3. Verify `cargo nextest run -p slideforge-diagrams --no-fail-fast` shows:
   - `test_BC_1_12_003_cold_render_correctness` — PASS
   - `test_cold_budget_under_200ms` — IGNORED (not FAIL, not PASS)

---

## Test inventory

| Test | File | Status |
|------|------|--------|
| `test_BC_1_03_002_http_4xx_not_retried_deterministic_harness` | `crates/slideforge-data/src/http.rs` | FAIL (Red Gate, todo! stub) |
| `test_BC_1_12_003_cold_render_correctness` | `crates/slideforge-diagrams/tests/cold_budget.rs` | PASS (companion, no stub needed) |

Total new tests: 2
Red Gate failures: 1 (test_BC_1_03_002_http_4xx_not_retried_deterministic_harness)
Red Gate passes (intentional, see rationale): 1 (test_BC_1_12_003_cold_render_correctness)

---

## Handoff to implementer

1. Implement `spawn_deterministic_counting_server_404` in `http.rs` — replace
   `todo!()` with the mpsc-channel blocking-accept design.
2. Verify `test_BC_1_03_002_http_4xx_not_retried_deterministic_harness` passes.
3. Run `test_bc_1_03_002_http_4xx_not_retried` (existing flaky test) 5 times to
   decide whether to keep it alongside the new test or delete it (per story AC-001).
4. Apply `#[ignore = "..."]` to `test_cold_budget_under_200ms` in cold_budget.rs.
5. Verify `cargo nextest run -p slideforge-diagrams` shows correctness test PASS,
   timing test IGNORED.
6. Run `cargo clippy --workspace --all-targets -- -D warnings` clean.
7. Run `cargo fmt --all -- --check` clean.
