# STORY-080 Demo Evidence Report

**Story:** STORY-080 — Test Reliability: De-flake `http_4xx_not_retried` (slideforge-data) + `cold_budget` (slideforge-diagrams)
**Branch:** `feature/STORY-080`
**Recorded:** 2026-06-08
**Toolchain:** VHS (terminal recording)
**Product type:** CLI / test harness (Rust workspace)

---

## Coverage Summary

| AC | Title | Status | Artifacts |
|----|-------|--------|-----------|
| AC-001 | `test_bc_1_03_002_http_4xx_not_retried` determinism — 5x consecutive passes | COVERED | `AC-001-http-4xx-determinism.gif`, `AC-001-http-4xx-determinism.webm`, `AC-001-http-4xx-determinism.tape` |
| AC-002 | `cold_budget` deflake — catastrophic gate + correctness always-on; `test_cold_budget_under_200ms` `#[ignore]`'d | COVERED | `AC-002-cold-budget-deflake.gif`, `AC-002-cold-budget-deflake.webm`, `AC-002-cold-budget-deflake.tape` |
| AC-003 | Behavioral assertions not weakened (`connection_count == 1`; correctness assertion preserved) | COVERED | Folded into AC-001 and AC-002 recordings above; see notes below |

## Committed Artifact Inventory

```
docs/demo-evidence/STORY-080/
├── evidence-report.md                   (this file)
├── AC-001-http-4xx-determinism.gif
├── AC-001-http-4xx-determinism.webm
├── AC-001-http-4xx-determinism.tape
├── AC-002-cold-budget-deflake.gif
├── AC-002-cold-budget-deflake.webm
└── AC-002-cold-budget-deflake.tape
```

AC-003 has no separate recording; its assertions are visible in both AC-001 and AC-002 recordings.

---

## Artifact Details

### AC-001 — HTTP 4xx Determinism

**File:** `AC-001-http-4xx-determinism.gif` / `.webm`
**Tape source:** `AC-001-http-4xx-determinism.tape`

**What is shown:**
- `cargo nextest run -p slideforge-data -E 'test(test_bc_1_03_002_http_4xx_not_retried)'` runs 5 consecutive times.
- Every run produces: `PASS slideforge-data http::tests::test_bc_1_03_002_http_4xx_not_retried`
- No failures across all 5 runs.
- Final banner: `5/5 PASSED — mpsc harness is deterministic (AC-001 + AC-003 confirmed)`

**Root cause fixed:**
The original harness used `set_nonblocking(true)` + a polling loop that raced with the OS TCP stack on Windows (EC-001). The replacement uses `spawn_deterministic_counting_server_404` which sends the final connection count over an `mpsc::channel` only after the server has completed all I/O — eliminating the race.

**AC-003 note:** The `assert_eq!(connection_count, 1, ...)` assertion in the test is unchanged and load-bearing. The recording demonstrates 5 passes where exactly 1 connection was observed per run.

---

### AC-002 — Cold Budget Deflake

**File:** `AC-002-cold-budget-deflake.gif` / `.webm`
**Tape source:** `AC-002-cold-budget-deflake.tape`

**What is shown (4 steps):**

1. **Default CI matrix run** (`-E 'test(cold_budget)'`):
   - `test_cold_budget_catastrophic_regression_gate` — PASS (1s/2s budget)
   - `test_cold_render_correctness` — PASS (no timing assertion)
   - `test_cold_budget_under_200ms` — SKIPPED (correctly `#[ignore]`'d)

2. **`--ignored` flag** shows `test_cold_budget_under_200ms` exists and is listed — it is present as a named on-demand test, not deleted.

3. **`test_cold_render_correctness` explicitly** — PASS; confirms correctness contract is always-on (AC-003).

4. **`test_cold_budget_catastrophic_regression_gate` explicitly** — PASS; confirms EC-005 (per-call font loading) is caught in CI.

**Three-test structure (STORY-080 Option A):**

| Test | Runs in CI | Budget | Purpose |
|------|-----------|--------|---------|
| `test_cold_budget_catastrophic_regression_gate` | Always (not `#[ignore]`'d) | 1s macOS/Linux, 2s Windows | EC-005 catastrophe detection |
| `test_cold_render_correctness` | Always (not `#[ignore]`'d) | None (correctness only) | AC-003 behavioral contract |
| `test_cold_budget_under_200ms` | On-demand only (`#[ignore]`'d) | 300ms macOS/Linux, 500ms Windows | NFR-003 precision gate |

**AC-003 note:** `test_cold_render_correctness` asserts `result.is_ok()` and `!normalized.is_empty()` unconditionally — the behavioral contract is never ungated even when the precision timing gate is skipped.

---

## Success Path vs. Error Path Coverage

For a test-reliability story, "error path" means demonstrating that the PREVIOUS race condition no longer manifests.

| Path | Demonstrated By |
|------|----------------|
| Success path (AC-001) | 5x consecutive PASS, 1 connection per run | `AC-001-*.gif/.webm` |
| Determinism proof (AC-001) | No single failure in 5 runs — no retry-count race | `AC-001-*.gif/.webm` |
| Success path (AC-002 always-on) | Catastrophic gate PASS, correctness PASS | `AC-002-*.gif/.webm` |
| Ignore-path proof (AC-002) | Precision gate present but `#[ignore]`'d — shown with `--ignored` flag | `AC-002-*.gif/.webm` |
| AC-003 non-weakening | `assert_eq!(connection_count, 1, ...)` in `test_bc_1_03_002_http_4xx_not_retried`; correctness assertion (`result.is_ok()` + `!normalized.is_empty()`) in `test_cold_render_correctness` | Both recordings |

---

## Pre-Push Gate Results

Run after recording (see commit message for full output):

```
cargo fmt --all -- --check              : PASS
cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used : PASS
cargo nextest run -p slideforge-data -p slideforge-diagrams --no-fail-fast : PASS
```

Main repo clean: `git -C /Users/jmagady/Dev/slideforge status --porcelain` — empty (no main-repo changes).
