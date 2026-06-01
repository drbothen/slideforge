---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-080
title: "Test reliability: de-flake http_4xx_not_retried (slideforge-data) + cold_budget (slideforge-diagrams)"
epic: EPIC-19
wave: 5
points: 3
priority: P2
tdd_mode: strict
status: draft
behavioral_contracts: [BC-1.03.002]
verification_properties: []
nfr_refs: [NFR-003, NFR-021, NFR-022, NFR-024, NFR-025, NFR-026, NFR-027, NFR-028, NFR-029, NFR-030]
crate: slideforge-data
target_module: slideforge-data
subsystems: [SS-10, SS-11]
depends_on: []
blocks: []
estimated_days: 1
# BC status: BC-1.03.002 invariant 2 ("HTTP requests are retried once on network error") is directly tested by the flaky test being hardened. Status=draft pending PO sign-off on the two-subsystem span (SS-10 + SS-11).
---

# STORY-080: Test Reliability — De-flake Cross-Platform Flaky Tests (http_4xx + cold_budget)

## Subsystem Anchor Justifications

**SS-10 (Data Sources)** owns the `http_4xx_not_retried` fix because `slideforge-data`
is the implementation crate and the flaky assertion directly tests BC-1.03.002 retry
policy — an SS-10 contract. Per ARCH-INDEX, SS-10 (`slideforge-data`) is "Effectful shell.
All external I/O."

**SS-11 (Diagrams)** owns the `cold_budget` fix because `slideforge-diagrams` is the
implementation crate and `tests/cold_budget.rs` tests the BC-1.12.003 normalization
pipeline's NFR-003 timing contract. Per ARCH-INDEX, SS-11 (`slideforge-diagrams`) is
"Effectful shell (font DB scan). Mostly pure." Both subsystems are touched by this
story because both flaky tests were documented together in the same STATE.md drift item
(DI-1) and have identical root cause structure: timing-sensitive assertions that are
non-deterministic under CI load. A single story is the correct scope — they share the
same tasks, test-infrastructure owner, and wave assignment.

## Dependency Anchor Justifications

- No `depends_on` entries: this is a test-code-only change. Both flaky tests already
  exist; this story makes them deterministic. No new production features are required.
- No `blocks` entries: test reliability improvements are orthogonal to feature stories.
  Wave 5 stories that depend on slideforge-data or slideforge-diagrams are not blocked
  on this fix — they just benefit from a more stable CI matrix.

## Summary

Two pre-existing flaky tests reduce CI confidence and mask regressions on the Windows
x86_64 runner:

### Flaky Test 1 — `slideforge-data::http::tests::test_bc_1_03_002_http_4xx_not_retried`

**Root cause (connection-count race):**
The test spawns a TCP server on a background thread using `set_nonblocking(true)` and
polls for connections in a `while std::time::Instant::now() < deadline && accepted < 3`
loop with 5ms sleep intervals. The test then issues `src.load(...)` on the main thread
and immediately joins the server thread.

On Windows, the OS TCP stack can race between the `accept()` call and the connection
counter. Specifically: `ureq` internally opens a socket, completes the TCP handshake,
and closes the connection. On some Windows kernel versions, the keep-alive probe or the
`Connection: close` flush causes a second SYN to be issued before the first connection
is fully torn down — the non-blocking accept loop sees it as a second accepted connection.
This manifests as `connection_counter == 2` on the assert-equals-1 check, causing a
spurious test failure.

**Fix approach:**
Replace the raw `TcpListener` + `set_nonblocking(true)` + manual poll loop with a
deterministic design: use a `Barrier` or `channel` to synchronize the server's
"connection received and response sent" event with the main thread assertion, ensuring
the counter is read only after the server has finished serving all connections and the
socket has been fully closed (TIME_WAIT elapsed or FIN_WAIT2 complete). Alternatively,
configure `ureq` in the test to explicitly disable keep-alive by adding a
`Connection: close` expectation and only counting connections AFTER the full response
has been written and the stream dropped.

The canonical fix is: use a **one-shot server with a bounded channel** — the server
thread sends the final connection count over a `mpsc::channel` after the `deadline`
expires, and the main thread reads it. This eliminates the race between the
non-blocking poll and the connection counter. The assertion logic is unchanged
(exactly 1 connection for 4xx, no retry), but the counting mechanism is reliable.

**Non-vacuity requirement:**
The fixed test MUST still prove the retry-count contract. The assertion
`connection_counter == 1` MUST remain. Removing the assertion, weakening it to
`connection_counter <= 2`, or replacing it with `assert!(result.is_err())` alone are
NOT acceptable — they destroy the load-bearing behavioral contract proof.

### Flaky Test 2 — `slideforge-diagrams::cold_budget::test_cold_budget_under_200ms`

**Root cause (timing assertion under CPU contention):**
`tests/cold_budget.rs` gates on `elapsed < budget` where `budget` is 300ms (macOS/Linux)
or 500ms (Windows). Under full-workspace `cargo nextest run` with CPU contention from
parallel test jobs, the font DB scan (`fontdb::Database::load_system_fonts()`) can
exceed the budget on shared CI runners even when the actual algorithm is correct.

The test is ALREADY structured as a timing gate (NFR-003 compliance check), not a
correctness check. Timing gates require special treatment: they are inherently
platform-dependent and should be labeled to run on-demand (like criterion benchmarks),
not as part of the default test matrix.

**Fix approach:**
There are two acceptable resolutions (implementer chooses based on usvg test runner
architecture):

Option A — **`#[ignore]` gate with documentation**: Mark `test_cold_budget_under_200ms`
with `#[ignore = "timing-sensitive NFR-003 gate — run with: cargo test -p slideforge-diagrams -- --include-ignored cold_budget"]`.
Add a companion **correctness-only test** (no timing assertion) that verifies the cold
render succeeds and returns non-empty output. The timing test must include a code comment
citing NFR-003 and the blocking reason ("timing-sensitive, run explicitly").

Option B — **Retry with headroom**: Retry the timing measurement up to 3 times if the
first measurement exceeds the budget (the retry is cheap — the font DB is already warm
after the first call, so only the first measurement is the cold-path measurement). Keep
only the first measurement as the authoritative value; the retries are for detecting
"CI was transiently overloaded, not a real regression." If all 3 measurements exceed
the budget, fail the test. This approach preserves the test in the default matrix
without false failures.

Option C — **Platform-specific budget widening**: Expand the CI budget from 300ms to
500ms on macOS/Linux (matching Windows). This is the least rigorous option and is only
acceptable if Options A and B are infeasible due to test binary architecture constraints.

**Non-vacuity requirement:**
Regardless of which option is chosen, a correctness assertion MUST remain: the cold
render MUST return `Ok(NormalizedDiagramSvg)` with non-empty content. The timing gate
may be relaxed or made on-demand, but the behavioral correctness is non-negotiable.

### Connection to DI-1

STATE.md records DI-1 as "LOCAL adversary platform-blindness" — the observation that
local adversary passes (developer machine, macOS arm64) do not catch Windows-specific
test failures. This story is evidence that the cross-platform CI matrix is the correct
catch point. The fix here reinforces the principle by making both tests reliable on all
5 CI platforms (macos-14, macos-13, ubuntu-latest, ubuntu-24.04-arm, windows-latest).

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-1.03.002 | Load @data from HTTP/HTTPS URL at Compile Time | AC-001 (retry policy determinism) |

Note: BC-1.03.002 invariant 2 ("HTTP requests are retried once on network error before
producing E-DAT-002") is the anchor clause — the flaky test exists to prove that 4xx
responses are NOT covered by this retry invariant (client errors are not retriable).
The BC traceability is indirect: this story ensures the test that validates invariant 2's
boundary condition (4xx = no retry) is reliably measurable.

## Acceptance Criteria

### AC-001: `test_bc_1_03_002_http_4xx_not_retried` passes on all 5 CI platforms ≥ 10/10 runs (traces to BC-1.03.002 invariant 2)

The test MUST pass on all 5 CI target platforms (macos-14, macos-13, ubuntu-latest,
ubuntu-24.04-arm, windows-latest) across 10 consecutive runs each without a single
failure. The assertion `connection_counter == 1` MUST be present and non-weakened.
The root cause (connection-count race in the non-blocking accept loop) MUST be
eliminated via a deterministic connection-counting mechanism (bounded channel or
synchronization primitive), not masked by retry logic or assertion weakening.

### AC-002: `test_cold_budget_under_200ms` does not cause spurious CI failures under load (traces to NFR-003)

The cold-budget timing test MUST NOT fail spuriously under full-workspace `cargo nextest`
with CPU contention. The implementer chooses from Option A (ignore + companion correctness
test), Option B (retry-with-headroom), or Option C (budget widening). Whichever option
is chosen:
- A correctness assertion (`result.is_ok()` + `!normalized.is_empty()`) MUST remain.
- The timing gate, if retained in the default matrix, MUST pass reliably under load.
- If `#[ignore]` is applied, the ignore string MUST include explicit re-run instructions.
- The test file MUST include a code comment explaining why the timing gate is
  on-demand rather than in the default matrix.

### AC-003: Existing behavioral assertions are not weakened (traces to BC-1.03.002 invariant 2)

Neither test regression fix may relax the behavioral contract it is testing:
- `test_bc_1_03_002_http_4xx_not_retried` MUST still assert `connection_counter == 1`
  (exactly one connection, no retry on 4xx).
- `test_cold_budget_under_200ms` (or its on-demand equivalent) MUST still assert that
  the cold render returns `Ok(...)` with non-empty content.
- The fixes address the TEST HARNESS reliability, not the CONTRACT being tested.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `test_bc_1_03_002_http_4xx_not_retried` (modified) | `crates/slideforge-data/src/http.rs` `#[cfg(test)]` | Effectful (spawns TCP server) |
| `test_cold_budget_under_200ms` (modified) | `crates/slideforge-diagrams/tests/cold_budget.rs` | Effectful (disk I/O: font DB scan) |
| Companion correctness test (new, if Option A chosen) | `crates/slideforge-diagrams/tests/cold_budget.rs` | Effectful (same process) |

Architecture section files:
- `architecture/module-decomposition.md` (SS-10, SS-11 boundaries)

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~3,000 |
| `crates/slideforge-data/src/http.rs` (relevant test section, ~100 lines) | ~1,500 |
| `crates/slideforge-diagrams/tests/cold_budget.rs` (~80 lines) | ~1,200 |
| BC-1.03.002 | ~800 |
| NFR-003 from nfr-catalog.md | ~300 |
| Test output / tool feedback | ~2,000 |
| **Total** | **~8,800** |

8,800 tokens is well within 20% of any reasonable agent context window. No split needed.

## Tasks

### Fix 1 — `test_bc_1_03_002_http_4xx_not_retried` (slideforge-data)

- [ ] Read `crates/slideforge-data/src/http.rs` test at line ~1944 to understand the current implementation
- [ ] Design replacement: replace the non-blocking poll loop with an `mpsc::channel`-based one-shot server that sends the final connection count after `deadline` expires
- [ ] Implement the replacement server design — keep the external behavior identical (serves 404, counts connections)
- [ ] Verify the `assert_eq!(connection_counter, 1, ...)` assertion is unchanged
- [ ] `cargo nextest run -p slideforge-data -E 'test(test_bc_1_03_002_http_4xx_not_retried)' --no-fail-fast` — run 5 times to verify stability
- [ ] `cargo nextest run -p slideforge-data --no-fail-fast` — verify no regressions in full crate

### Fix 2 — `test_cold_budget_under_200ms` (slideforge-diagrams)

- [ ] Read `crates/slideforge-diagrams/tests/cold_budget.rs` fully
- [ ] Choose Option A, B, or C based on test binary architecture (recommendation: Option A — it is the most honest about the nature of timing tests)
- [ ] Apply chosen option:
  - Option A: add `#[ignore = "..."]` + create companion correctness test `test_cold_render_correctness`
  - Option B: implement 3-attempt retry loop; keep assertion on first cold measurement; fail if all 3 exceed budget
  - Option C: widen budget to 500ms on macOS/Linux (requires justification comment citing CI runner observation data)
- [ ] Ensure correctness assertion remains (`result.is_ok()` + `!normalized.is_empty()`)
- [ ] `cargo nextest run -p slideforge-diagrams --no-fail-fast` — all tests pass

### Gate

- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo fmt --all -- --check` clean
- [ ] Full `cargo nextest run --workspace --no-fail-fast` passes with no flaky tests

## Previous Story Intelligence

N/A — first story in EPIC-19 maintenance slot for test infrastructure. However, note:

- STORY-034 (`cold_budget.rs` was created here) documents why the test is a separate
  Cargo binary: "each `tests/*.rs` file compiles to a separate binary" giving a genuine
  cold-path FONT_DB measurement. The fix must preserve this property — the font DB must
  still start uninitialized in the cold-path test.
- STORY-019 (`http.rs` was created here) established the `TcpListener` + `AtomicUsize`
  counting pattern. The fix replaces the unreliable non-blocking accept loop but keeps
  the `AtomicUsize` as the actual counter storage (or a `Mutex<usize>`, or an
  `mpsc::Sender<usize>` — whichever is cleanest).
- STATE.md DI-1 lesson: "LOCAL adversary cascade does not reproduce Windows-specific
  failures." The correct response is cross-platform CI verification, not a LOCAL fix.
  This story documents its own CI-matrix verification requirement in AC-001.

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and project ADRs:

1. **No new production code** (this story modifies test code only). No production
   `Cargo.toml` changes allowed.
2. **Test helpers are allowed new std dependencies** (no external crate additions
   needed here — `std::sync::mpsc`, `std::thread`, `std::net` are already in scope).
3. **Non-blocking accept pattern is the root cause** — the replacement must be
   synchronous or use a bounded blocking mechanism, not `set_nonblocking(true)` for
   the connection counter.
4. **Timing tests must be documented** — if `#[ignore]` is used, the ignore string
   MUST be human-readable and include explicit re-run instructions.
5. **Zero `.unwrap()` in non-test code** (NFR-021): these are test-only changes, so
   `.unwrap()` is permissible inside `#[cfg(test)]` blocks per project convention.

## Library and Framework Requirements

| Dependency | Version | Usage |
|------------|---------|-------|
| `std::sync::mpsc` | stdlib | Replacement channel for connection counting |
| `std::thread` | stdlib | Already used in both tests |
| `std::net::TcpListener` | stdlib | Still used (but in blocking mode, not `set_nonblocking`) |
| `std::time` | stdlib | `Instant` timing for `cold_budget` |

No new crate dependencies. Both fixes use only the standard library.

## File Structure Requirements

| Action | File | Change |
|--------|------|--------|
| Modify | `crates/slideforge-data/src/http.rs` | Rewrite `test_bc_1_03_002_http_4xx_not_retried` server harness |
| Modify | `crates/slideforge-diagrams/tests/cold_budget.rs` | Apply Option A/B/C to `test_cold_budget_under_200ms` |
| Create (if Option A) | `crates/slideforge-diagrams/tests/cold_budget.rs` | Add `test_cold_render_correctness` function |
| No change | Any production `.rs` file | Test-only story |
| No change | Any `Cargo.toml` | No dependency changes |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | CI runner accepts 2 connections due to TCP keep-alive on Windows | After fix: `connection_counter == 1` reliably — the server design must not count keep-alive probes |
| EC-002 | Cold budget test races with fontdb warm-up from another test in same binary | After fix: `cold_budget.rs` is a separate binary (each `tests/*.rs` is its own process), so FONT_DB always starts cold |
| EC-003 | Option A chosen but companion correctness test fails | Companion test must pass (correctness is non-negotiable even if timing is on-demand) |
| EC-004 | Fixing the race causes the test to never detect an actual retry regression | AC-003 prevents this — the `connection_counter == 1` assertion is preserved exactly |
| EC-005 | Budget widening (Option C) masks a real NFR-003 regression | Option C is only acceptable with a cited observation and must still catch catastrophic regressions (e.g., per-call font loading = 10× budget violation) |

## Forbidden Dependencies

The following patterns MUST NOT appear in the revised test code:

- `thread::sleep` used to "fix" the race by adding delay (this is flake masking, not fixing)
- Weakening the retry-count assertion (`<= 2` instead of `== 1`)
- `set_nonblocking(true)` on the replacement server (this is the root cause; do not reintroduce)
- New external crate in `[dev-dependencies]` (both fixes use std only)

## Test Strategy

Both changes are to existing test code. The test strategy is:

1. **Fix correctness first**: ensure the assertion logic is unchanged and load-bearing.
2. **Fix harness reliability**: make the connection-counting mechanism deterministic.
3. **CI verification**: run each fixed test 5× locally (`cargo nextest run -E 'test(name)'`)
   before declaring it stable. Report in the PR description the number of consecutive
   passes observed.
4. **Cross-platform CI**: the PR CI matrix (5 platforms) is the authoritative gate.
   The story is not DONE until CI shows green on all 5 platforms without a re-run.

## Complexity Estimate

3 story points. Both fixes are test-harness rewrites in existing test files. No new
algorithms, no new architecture decisions, no new production code. The `mpsc::channel`
replacement for the connection counter is a well-understood pattern. The cold_budget
Option A/B/C choice is a one-line (ignore annotation) to ten-line (retry loop) change.

The adversarial cascade for this story will focus on: (1) confirming the `connection_counter == 1`
assertion survives unchanged, (2) confirming Option A/B/C does not silently drop the
timing gate entirely, (3) confirming no new platform-specific `cfg` guards are needed
beyond what already exists.
