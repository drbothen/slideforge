---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-092
title: "CI: Cache reliability + disk headroom (eliminate cold-build flakes)"
epic: EPIC-19
wave: 5
points: 5
priority: NEXT
tdd_mode: facade
status: draft
spec_version: "1.2"
behavioral_contracts: []
# BC status: pending PO authorship — no product BC governs CI caching configuration.
# Anchors to NFR-026 through NFR-030 (multi-platform matrix reliability) and the
# devops CLAUDE.md quality bar. See STORY-051 precedent.
verification_properties: []
nfr_refs: [NFR-026, NFR-027, NFR-028, NFR-029, NFR-030]
target_module: .github/workflows/ci.yml
subsystems: []
depends_on: [STORY-091]
blocks: [STORY-093]
estimated_days: 1
# Human priority override 2026-06-10: deliver BEFORE remaining Wave-5 feature stories.
# STORY-092 depends on STORY-091 (tiered trigger must be in place so cache measurements
# reflect the correct baseline — merge-queue full-matrix runs).
---

# STORY-092: CI — Cache Reliability + Disk Headroom

## Subsystem Anchor Justification

EPIC-19 (CI/CD Infrastructure, cross-cutting). Same rationale as STORY-091: all changes
are to `.github/workflows/ci.yml` with no source crate modifications. EPIC-19's mandate
explicitly covers "GitHub Actions workflows for the full quality bar" and the devops
CLAUDE.md quality bar lists "fuzz harness in CI" and CI matrix reliability as
non-negotiable gates.

This story specifically addresses the ROOT CAUSE of the `runner lost communication /
No space left on device` class of CI failures that have required manual re-runs and
blocked story merges (referenced as "STORY-081-class merge flakiness" in the research).

## Dependency Anchor Justifications

- `depends_on: [STORY-091]` — STORY-091's tiered trigger must be in place first so
  cache measurements reflect the correct production conditions. In particular: after
  STORY-091, the arm64 leg only runs in merge_group/develop/nightly. STORY-092's
  cache audit (AC-005) uses `gh cache list` to verify the budget — this needs the
  tiered trigger to be live to get meaningful arm64-leg cache statistics.
- `blocks: [STORY-093]` — STORY-093's arm64 profiling baseline (mold linker timing,
  `--timings` capture) should be done AFTER STORY-092 lands so the profiling reflects a
  warm-cache run, not a cold-build run triggered by STORY-092 resetting cache keys.

## Summary

The 5-platform CI matrix currently suffers from two classes of persistent flakiness:

### Class A: `No space left on device`

The `ubuntu-24.04-arm` standard runner has a 14 GB SSD. After preinstalled tooling,
the effective free space is well below 14 GB. A full-workspace `--all-features` build
with debug info can exceed the available space, causing the job to fail mid-compile.
Each failure costs a full re-run (~22 minutes arm64 cold build). The same issue occurs
on `ubuntu-latest` and `macos-*` runners for snapshot and visual-regression jobs.

**Fix:** Add a disk-space reclamation step to all build-heavy jobs. The preferred
approach is a manual `rm -rf` of known large preinstalled toolchain directories (no
new third-party action dependency, ~10-15 GB freed, supply-chain clean). Escalate to
pinned `jlumbroso/free-disk-space@54081f138730dfa15788a46383842cd2f914a1be` only if
manual cleanup proves insufficient for a specific job.

**Verify-at-implementation (arm64 disk paths):** The `rm -rf` path list
(`/usr/share/dotnet`, `/usr/local/lib/android`, `/opt/ghc`, `/opt/hostedtoolcache/CodeQL`,
`/usr/local/share/boost`) is derived from the x86_64 Ubuntu image inventory. The
`ubuntu-24.04-arm` partner image tooling inventory DIFFERS from x86_64. Implementer
MUST verify which paths exist on the arm64 image and are worth removing, and MUST guard
each `rm -rf` call so that a missing path does not fail the step (use `rm -rf <path> || true`
or pre-check with `test -d`). See Edge Case EC-001.

### Class B: Perpetual cold builds on arm64 — CONFIRMED ROOT CAUSE

**Confirmed finding (2026-06-10):** The cache-budget-exceeded hypothesis is no longer
a hypothesis — it is DATA-BACKED. API query `gh api repos/drbothen/slideforge/actions/cache/usage`
(2026-06-10) returned **9.77 GB across 23 active caches**, or **97.7% of GitHub's ~10 GB
per-repo limit**. The `v0-rust-test-linux-arm64-*` cache is ABSENT from the active set
(LRU-evicted), which directly explains the arm64 cold-build / re-run flakiness. This is
the confirmed root cause of the arm64 cold-build flake.

Contributing factors that explain how the budget reached 97.7%:
- Multiple lockfile-hash versions per shared-key (e.g., two clippy caches, multiple
  `target/` variants per key) consuming duplicate budget. NOTE: these duplicate entries
  are NORMAL lockfile-hash rotation and will persist post-fix; they are not eliminated
  by the SHA bump.
- The previously-pinned rust-cache SHA (`42dc69e1aa15d09112580998cf2ef0119e2e91ae`)
  is the annotated TAG OBJECT for the floating `v2` tag, not a peeled commit SHA. Tag
  objects become unreachable and GC-eligible when upstream re-points the floating tag,
  causing a repo-wide `uses:` resolution failure. The bump to v2.9.1's peeled commit
  SHA (`c19371144df3bb44fab255c43d04cbc2ab54d1c4`) eliminates this fragility. NOTE: the
  old SHA already ran v2.9.1 code (the tag object peeled to `e18b4977` carrying
  package.json 2.9.1); the bump has ZERO behavioral delta (no key-format change, no
  orphaned caches, no cold rebuild caused by the SHA change itself).
- `cache-on-failure: "true"` is NOT present on several heavy compile jobs (`clippy`,
  `msrv`, `docs`, `snapshots`, `supply-chain`, `doctest`, `visual-regression`, `perf-smoke`).
  When these jobs time out or fail, the cache is not saved, breaking the warm path.

**Fix:** Bump rust-cache from the tag-object SHA `42dc69e1aa15d09112580998cf2ef0119e2e91ae`
to the peeled commit SHA `c19371144df3bb44fab255c43d04cbc2ab54d1c4` (v2.9.1). This
eliminates tag-object GC fragility (the primary value of the bump). Add
`cache-on-failure: "true"` to all build-heavy jobs; consolidate shared-keys to bring
total active cache under ~8 GB; verify arm64 cache survives across consecutive runs.

**TD-VSDD-060 note:** rust-cache SHA replacement must update ALL call-sites in `ci.yml`
where `Swatinem/rust-cache@<old-sha>` appears. A grep-zero check that no `42dc69e1`
remains in `ci.yml` is a REQUIRED post-implementation step.

**Implementation note (2026-06-11):** The actual implementation (feature/STORY-092 @
c264910a) updated 11 call-sites in `ci.yml` plus 8 call-sites across companion
workflows (security.yml, pdf-ua1.yml, html-wcag.yml, release.yml) — 19 total bumps.
The `codeql-action` pin in `security.yml` had the same tag-object fragility and was
re-pinned to peeled commit `03e4368a...`. These companion-workflow changes are
implementation-level details; they do not add new ACs (companion workflows were not
in-scope when the story was authored).

### Connection to STORY-080

STORY-080 de-flaked two specific test assertions. STORY-092 addresses the INFRASTRUCTURE
ROOT CAUSE that triggered STORY-080-class re-run loops: the disk and cache flakiness that
forced re-runs which then hit the non-deterministic test harness. STORY-092 does not
modify any test code.

## Behavioral Contracts

No product BCs govern CI caching. NFR anchoring:

| NFR | Title | Covered ACs |
|-----|-------|-------------|
| NFR-026 | macOS arm64 binary builds and passes tests | AC-002 (cache-on-failure) |
| NFR-027 | macOS x86_64 binary builds and passes tests | AC-002 (cache-on-failure) |
| NFR-028 | Linux x86_64 binary builds and passes tests | AC-001, AC-002 |
| NFR-029 | Linux arm64 binary builds and passes tests | AC-001, AC-002, AC-003, AC-005, AC-006 |
| NFR-030 | Windows x86_64 binary builds and passes tests | AC-002 (cache-on-failure) |

## Acceptance Criteria

### AC-001: No job fails with `No space left on device` (traces to NFR-028, NFR-029)

After the disk-space reclamation step is added to all build-heavy jobs, no CI run MUST
fail with `No space left on device` or `ENOSPC`. The step MUST appear BEFORE the first
`cargo build`/`cargo nextest`/`cargo clippy` command in each build-heavy job. Jobs
in scope: `test` (all matrix legs), `clippy`, `snapshots`, `visual-regression`, `bench`.
The manual `rm -rf` approach is preferred over a third-party action (see "Pinned
Versions / SHAs" section for the escalation path if manual cleanup is insufficient).

Each `rm -rf` MUST be guarded (`|| true` or `test -d`) so that a missing path on the
arm64 image does not fail the step. The implementer MUST verify which paths exist on
`ubuntu-24.04-arm` before implementing (see verify-at-implementation note in Summary).

### AC-002: `cache-on-failure: "true"` present on all build-heavy jobs (traces to NFR-026 through NFR-030)

Every job that compiles Rust code MUST have `cache-on-failure: "true"` set on its
`Swatinem/rust-cache` step. This covers: `test` (all matrix legs), `clippy`, `msrv`,
`docs`, `snapshots`, `visual-regression`, `bench`, `doctest`, `supply-chain`,
`perf-smoke`. The value MUST be the string `"true"` (not the boolean `true`) per the
rust-cache action's expected type.

### AC-003: rust-cache bumped to peeled commit SHA at ALL call-sites in ci.yml (traces to NFR-029)

All `Swatinem/rust-cache` uses in `ci.yml` MUST reference SHA
`c19371144df3bb44fab255c43d04cbc2ab54d1c4` (v2.9.1, confirmed via `git ls-remote`
2026-06-10 — annotated-tag deref to peeled commit). The value of this bump is
**tag-object GC fragility elimination**: the old SHA `42dc69e1aa15d09112580998cf2ef0119e2e91ae`
is an annotated tag-object SHA, not a peeled commit SHA. Annotated tag-object SHAs
become unreachable and GC-eligible when upstream re-points the floating `v2` tag,
causing repo-wide `uses:` resolution failures. The peeled commit SHA is stable regardless
of tag movement. There is NO behavioral delta in this bump (the old tag-object already
peeled to `e18b4977` carrying package.json v2.9.1 — the same code runs before and after).
Duplicate per-shared-key cache entries (multiple lockfile-hash versions) are NORMAL
rotation and will NOT be eliminated by this bump; budget reduction comes from key
consolidation (AC-004) and cache-on-failure (AC-002).

**The old SHA (`42dc69e1aa15d09112580998cf2ef0119e2e91ae`) MUST NOT remain in ANY job
in `ci.yml`.** After implementation, run:

```bash
grep -c "42dc69e1" .github/workflows/ci.yml
```

This MUST return `0`. If it returns any non-zero value, the implementation is incomplete
(TD-VSDD-060 sibling-site sweep — all call-sites must be updated).

### AC-004: Cache budget documented and within ~10 GB limit with headroom (traces to NFR-029)

The `docs/playbooks/tiered-ci-merge-queue.md` file (created in STORY-091) MUST be
updated with a cache budget table listing all `shared-key`s, their approximate
`target/` sizes, and the estimated total. The implementer MUST aim for the total
to fit within ~8 GB (not just the 10 GB cap — leave ~2 GB headroom to prevent
future LRU eviction of the arm64 cache). Levers to reduce budget:
- Consolidate shared-keys for non-matrix lint/doc jobs where safe (e.g., `docs`, `msrv`,
  `supply-chain` may share a `build-only` key if they use the same feature flags and target).
  Note: duplicate caches per shared-key (multiple lockfile-hash versions) contribute to
  bloat — consolidating keys reduces the number of live cache variants.
- STORY-093's `debug = "line-tables-only"` profile change will further reduce `target/`
  size (complementary, not a substitute for key consolidation here).
DO NOT collapse the matrix legs into one shared-key — each target triple requires its
own `target/`.

### AC-005: Cache budget confirmed DATA-BACKED; fix brings total under ~8 GB (traces to NFR-029)

**Baseline (confirmed 2026-06-10):** `gh api repos/drbothen/slideforge/actions/cache/usage`
returned **9.77 GB across 23 active caches** (97.7% of the ~10 GB limit). The
`v0-rust-test-linux-arm64-*` cache was ABSENT (LRU-evicted) — this is the confirmed
cause of arm64 cold-build flakiness.

After this story lands, re-run `gh api repos/drbothen/slideforge/actions/cache/usage`
(or `gh cache list --repo drbothen/slideforge`) and record the result in the PR description.
The AC passes when:
1. Total active cache size is measurably below the 9.77 GB baseline (target ≤ ~8 GB).
2. The `v0-rust-test-linux-arm64-*` (or equivalent arm64 key) IS present in the active
   cache set (not evicted).

This replaces the prior "verify the hypothesis" framing — the hypothesis is confirmed.
The AC now measures the FIX's effectiveness.

### AC-006: arm64 test job shows warm cache on consecutive runs (traces to NFR-029)

After this story lands, the `test (linux-arm64)` job MUST show a cache HIT (rust-cache
log line "restored cache" or equivalent) on the second consecutive run to develop or
merge_group (i.e., a run that follows a prior SUCCESSFUL arm64 run within the 7-day
cache expiry window). A single subsequent warm run is acceptable evidence.

**Wall-clock framing:** Measure cold vs warm `test (linux-arm64)` wall-clock; record
BOTH figures in the PR description. The AC passes if: (a) the arm64 cache survives across
consecutive runs (verified via `gh cache list --repo drbothen/slideforge`) AND (b) a warm
run is measurably faster than cold. Do NOT gate on a fixed <12-minute threshold — the
prior "< 12 minutes" figure was a directional estimate (cold baseline: ~22 min). Record
the actual warm-run measurement; the comparison against cold is the meaningful metric.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| CI workflow (modified) | `.github/workflows/ci.yml` | Effectful (GitHub Actions DSL) |
| Cache budget doc (updated) | `docs/playbooks/tiered-ci-merge-queue.md` | N/A (documentation) |

Architecture section files: N/A — no source crate changes.

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~3,600 |
| `.github/workflows/ci.yml` (full file, ~490 lines) | ~7,000 |
| ci-speed-research.md Q2 + Q3 sections | ~2,500 |
| `docs/playbooks/tiered-ci-merge-queue.md` (update, ~200 lines) | ~2,500 |
| `gh cache list` / `gh api` output + PR evidence | ~1,000 |
| **Total** | **~16,600** |

16,600 tokens is within 20% of a 200k-token agent context window. No split needed.

## Tasks

### A: Disk-space reclamation

- [ ] Read `.github/workflows/ci.yml` fully; identify all build-heavy jobs
- [ ] Add the following step as the FIRST step in each build-heavy job
      (test matrix, clippy, snapshots, visual-regression, bench):
      ```yaml
      - name: Free disk space
        run: |
          sudo rm -rf /usr/share/dotnet || true
          sudo rm -rf /usr/local/lib/android || true
          sudo rm -rf /opt/ghc || true
          sudo rm -rf /opt/hostedtoolcache/CodeQL || true
          sudo rm -rf /usr/local/share/boost || true
          sudo docker image prune --all --force || true
          df -h
        # Frees ~10-15 GB of preinstalled tooling not needed for Rust CI.
        # Each rm -rf is guarded (|| true) so missing paths on ubuntu-24.04-arm
        # do not fail the step. Path list is x86_64-derived; verify on arm64 image.
        # Manual rm is preferred over jlumbroso/free-disk-space (supply-chain clean).
        # Revisit if insufficient for a specific job.
      ```
- [ ] **[VERIFY-AT-IMPL]** Confirm which of the above paths actually exist on the live
      `ubuntu-24.04-arm` partner image (the inventory differs from x86_64). Add a
      `ls -la /usr/share/dotnet /usr/local/lib/android /opt/ghc` diagnostic step
      OR check the ubuntu-24.04-arm image release notes / partner image manifest.
      Remove paths from the list that are absent on arm64 to keep the step clean.
- [ ] For `visual-regression` job ONLY: skip removing categories needed by LibreOffice
      (do NOT remove `/usr/local/lib/python*` or general apt packages the job relies on);
      only remove .NET, Android SDK, GHC, and Docker images
- [ ] `df -h` line confirms > 10 GB free after cleanup (check in CI log)

### B: rust-cache bump + cache-on-failure

- [ ] Replace ALL occurrences of `Swatinem/rust-cache@42dc69e1aa15d09112580998cf2ef0119e2e91ae`
      (and any other `Swatinem/rust-cache@<old-sha>` variant) in `ci.yml` with
      `Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4`
      (v2.9.1 peeled commit SHA, confirmed via `git ls-remote` 2026-06-10 — eliminates
      tag-object GC fragility; no behavioral delta in this bump)
- [ ] After replacement, run the grep-zero check:
      `grep -c "42dc69e1" .github/workflows/ci.yml` → MUST return `0`
      (TD-VSDD-060: all 11 call-sites must be updated; zero old-SHA residue allowed)
- [ ] Add `cache-on-failure: "true"` to the rust-cache step in every job that does not
      already have it: `clippy`, `msrv`, `docs`, `snapshots`, `supply-chain`, `doctest`,
      `visual-regression`, `perf-smoke`
- [ ] Confirm `test` and `bench` already have `cache-on-failure: "true"` (research states
      they do at ci.yml:145-151); add if absent

### C: Cache budget audit

- [ ] Run `gh api repos/drbothen/slideforge/actions/cache/usage` and record result
      (baseline confirmed: 9.77 GB / 23 caches / arm64 absent — now verify POST-FIX)
- [ ] Run `gh cache list --repo drbothen/slideforge`; enumerate all `shared-key` values
      currently in `ci.yml`; estimate per-key size from output
- [ ] If total > 8 GB (target budget): identify candidates for shared-key consolidation
      (e.g., `docs`, `msrv`, `supply-chain` may share a `build-only` key if they use
      the same feature flags and target triple)
- [ ] Document the budget table in `docs/playbooks/tiered-ci-merge-queue.md`, including
      the confirmed baseline (9.77 GB / 23 caches / 2026-06-10) and post-fix measurement
- [ ] Confirm arm64 cache KEY is present in `gh cache list` output after fix

### D: Verification

- [ ] Push to a PR branch; confirm no `No space left on device` errors
- [ ] After merge to develop (or via merge queue), confirm the SUBSEQUENT develop push
      shows rust-cache HIT for arm64; record before/after timings (cold vs warm) in PR
- [ ] `cargo fmt --all -- --check` and `cargo clippy --workspace -- -D warnings` still
      clean (no Rust code changed)

## Previous Story Intelligence

STORY-080 (Test reliability: de-flake http_4xx + cold_budget, Wave 5, EPIC-19) fixed
the SYMPTOM of flaky CI. This story fixes the INFRASTRUCTURE ROOT CAUSE. Key lesson
from STORY-080 (Previous Story Intelligence section):

> "STATE.md DI-1 lesson: 'LOCAL adversary cascade does not reproduce Windows-specific
> failures.' The correct response is cross-platform CI verification, not a LOCAL fix."

Analogy: the disk and cache flakiness cannot be reproduced locally. The correct fix
is in the CI configuration, not in production code. This story follows that principle.

## Architecture Compliance Rules

1. **No new third-party GitHub Actions without SHA pins** (NFR-025 supply chain). If
   `jlumbroso/free-disk-space` is escalated to (manual cleanup proved insufficient),
   use SHA `54081f138730dfa15788a46383842cd2f914a1be` (v1.3.1, confirmed via `git ls-remote`
   2026-06-10 — lightweight tag, direct SHA; date discrepancy from research is resolved).
   ESCALATION ONLY.
2. **CARGO_INCREMENTAL=0 must remain.** Research Q7 confirms it is correct for CI
   (incremental state bloats `target/`, worsens disk and cache pressure). Do not remove.
3. **Do not collapse matrix `shared-key`s.** Each target triple needs its own `target/`
   cache (arm64 binaries cannot reuse x86_64 objects). Only lint/doc single-platform
   jobs are candidates for key sharing.
4. **Do not add sccache.** It shares the same ~10 GB GHA cache budget as rust-cache and
   rarely fires when `target/` is warm. See "Rejected/Deferred Alternatives" below.
5. **visual-regression job special case.** Do not strip Python or apt packages that
   LibreOffice depends on. The disk-cleanup step for this job must be scoped more
   narrowly than for pure-Rust jobs.
6. **TD-VSDD-060 sibling-site sweep.** All `Swatinem/rust-cache` call-sites in
   `ci.yml` must be updated atomically (11 sites confirmed at authoring time; verify
   count at implementation). The grep-zero check (`grep -c "42dc69e1" .github/workflows/ci.yml`)
   is a required post-implementation verification step, not optional. Companion workflow
   call-sites (security.yml, pdf-ua1.yml, html-wcag.yml, release.yml) should also be
   swept for the same tag-object fragility; they are outside AC-003's scope but their
   update is tracked as an implementation note (see Class B Fix paragraph).

## Library and Framework Requirements

| Dependency | Version/SHA | Status | Notes |
|------------|-------------|--------|-------|
| `Swatinem/rust-cache` | v2.9.1 = `c19371144df3bb44fab255c43d04cbc2ab54d1c4` | CONFIRMED via `git ls-remote` 2026-06-10 (annotated-tag deref to peeled commit) | Replace old tag-object SHA `42dc69e1aa...` at ALL call-sites in ci.yml; eliminates tag-object GC fragility (no behavioral delta) |
| Manual `rm -rf` disk cleanup | stdlib bash | N/A | Preferred; no new action dependency; guard each path with `|| true` |
| `jlumbroso/free-disk-space` | v1.3.1 = `54081f138730dfa15788a46383842cd2f914a1be` | CONFIRMED via `git ls-remote` 2026-06-10 (lightweight tag, direct SHA); date discrepancy RESOLVED | ESCALATION ONLY if manual cleanup insufficient |
| `taiki-e/install-action` | v2.81.9 = `fd2f5e3d644b484055ebf4268f474c565f148f25` | SHA confirmed (deferred — only relevant if nextest archiving is adopted); current pin `d9be7d8c` stays | DEFERRED; nextest archiving not adopted in this story |

## Pinned Versions / SHAs

All SHAs have been confirmed or deferred as of 2026-06-10. No open uncertainty flags remain.

| Action | SHA | Status | Notes |
|--------|-----|--------|-------|
| `Swatinem/rust-cache` v2.9.1 | `c19371144df3bb44fab255c43d04cbc2ab54d1c4` | CONFIRMED 2026-06-10 via `git ls-remote --tags https://github.com/Swatinem/rust-cache` (annotated-tag deref to peeled commit) | Replaces tag-object SHA `42dc69e1aa...`; eliminates GC fragility (no behavioral delta — old SHA already ran v2.9.1 code); all call-sites in ci.yml must be updated |
| `jlumbroso/free-disk-space` v1.3.1 | `54081f138730dfa15788a46383842cd2f914a1be` | CONFIRMED 2026-06-10 via `git ls-remote` (lightweight tag, direct SHA); date discrepancy resolved | ESCALATION ONLY — use only if manual rm -rf insufficient |
| `taiki-e/install-action` v2.81.9 | `fd2f5e3d644b484055ebf4268f474c565f148f25` | SHA noted; DEFERRED — only relevant if nextest archiving adopted; current pin `d9be7d8c` stays | Not required in this story |

## File Structure Requirements

| Action | File | Change |
|--------|------|--------|
| Modify | `.github/workflows/ci.yml` | Add disk-cleanup step to build-heavy jobs (with `|| true` guards); bump rust-cache SHA at all 11 call-sites; add cache-on-failure to jobs missing it |
| Modify | `docs/playbooks/tiered-ci-merge-queue.md` | Add cache budget table + confirmed baseline (9.77GB/23 caches/arm64-absent) + post-fix measurement |
| No change | Any `*.rs` or `Cargo.toml` file | No Rust code changes |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `rm -rf` path absent on `ubuntu-24.04-arm` partner image | Expected: guarded with `|| true`; step does not fail; path is a no-op. Implementer must verify the arm64 image inventory before finalizing the path list. |
| EC-002 | Manual `rm -rf` removes a directory that LibreOffice needs | Scoped per AC-001: visual-regression job has a narrower cleanup list. If LibreOffice fails to start, investigate which category to exclude |
| EC-003 | Cache HIT after STORY-092 but build still slow | Investigate: (a) incremental build turned on accidentally, (b) Cargo.lock changed invalidating cache, (c) rust-cache shared-key differs between runs (check ci.yml env that feeds the key) |
| EC-004 | `gh cache list` shows arm64 cache present but stale (>7 day old) | 7-day expiry is GHA default. Cache was saved but expired. The consolidation and cache-on-failure changes ensure it is refreshed more frequently |
| EC-005 | `grep -c "42dc69e1" .github/workflows/ci.yml` returns non-zero after implementation | Implementation is incomplete — one or more of the 11 call-sites was missed. Find and fix all remaining occurrences before committing. |
| EC-006 | `CARGO_INCREMENTAL=0` accidentally removed from env | Restoring it is mandatory: incremental mode bloats `target/` and is confirmed incorrect for GHA cache hit-rate (research Q7) |

## Forbidden Dependencies

- Do NOT add `sccache` or `RUSTC_WRAPPER=sccache` to any job.
- Do NOT use `actions/cache` directly for Rust caching (continue using rust-cache).
- Do NOT add any new third-party action without a pinned SHA.
- Do NOT set `CARGO_INCREMENTAL=1` or remove `CARGO_INCREMENTAL=0`.

## Rejected/Deferred Alternatives

| Alternative | Decision | Reason |
|-------------|----------|--------|
| sccache (mozilla/sccache-action) | REJECTED | Shares the same ~10 GB GHA cache budget as rust-cache; rarely fires when `target/` is warm (Cargo skips `rustc` entirely on warm cache); would WORSEN cache thrashing. Research Q5, HIGH confidence. |
| nextest archive / build-once-test-many | DEFERRED | Test-execution optimization; bottleneck is compile-time on a single arm64 box, not test execution time. Revisit when test suite execution time exceeds compile time. Research Q4, HIGH confidence on "defer" recommendation. |
| Larger runners (8/16-core) | REJECTED | Never free on public repos; requires Team/Enterprise plan. Not worth it for an OSS project. Research Q9, HIGH confidence. |
| `prefix-key` cache reset | Not needed | This is a break-glass tool (hard-bust all caches). Not needed here; cache health is being improved, not reset. |
| Collapse all jobs into one shared-key | REJECTED | Each target triple produces different binaries; a shared key across targets would produce incorrect cache hits or continuous misses. |

## Test Strategy

`tdd_mode: facade`. No Rust unit tests. Verification is via CI observation:

1. **AC-001:** Confirm no `No space left` in CI logs for 3 consecutive merge-queue runs.
2. **AC-002/AC-003:** Inspect CI YAML diff; confirm rust-cache SHA updated at ALL 11
   call-sites (grep-zero check); confirm `cache-on-failure` presence on all listed jobs.
3. **AC-005:** `gh api repos/drbothen/slideforge/actions/cache/usage` post-fix output
   captured and recorded in PR description; compare against 9.77 GB confirmed baseline;
   confirm arm64 cache key present.
4. **AC-006:** PR description includes before/after arm64 wall-clock (cold vs warm)
   from two consecutive develop pushes.

## Complexity Estimate

5 story points. The changes are mechanical (find-replace old SHA at 11 sites, add
`cache-on-failure`, add disk-cleanup steps) but require careful scoping for the
visual-regression job and the cache budget audit adds investigation time. The
sibling-site sweep (all 11 rust-cache call-sites) is mandatory (TD-VSDD-060).
Estimated 1 day including CI verification.

## Uncertainty Resolution Log

### Resolved (2026-06-10, remove-uncertainty pass)

| Item | Resolution |
|------|-----------|
| `Swatinem/rust-cache` v2.9.1 SHA | **CONFIRMED** `c19371144df3bb44fab255c43d04cbc2ab54d1c4` via `git ls-remote --tags https://github.com/Swatinem/rust-cache` 2026-06-10 (annotated-tag deref to peeled commit). The old SHA `42dc69e1aa...` is the annotated tag-object for the floating `v2` tag, which already peeled to `e18b4977` carrying v2.9.1. The bump has zero behavioral delta; its value is tag-object GC fragility elimination. Remove uncertainty flag. |
| `jlumbroso/free-disk-space` v1.3.1 SHA | **CONFIRMED** `54081f138730dfa15788a46383842cd2f914a1be` via `git ls-remote` 2026-06-10 (lightweight tag, direct SHA). Date discrepancy from research is RESOLVED (the SHA is correct; v1.3.1 is current latest). Escalation-only usage retained. Remove uncertainty flag. |
| `taiki-e/install-action` v2.81.9 SHA | SHA `fd2f5e3d644b484055ebf4268f474c565f148f25` noted. DEFERRED — only relevant if nextest archiving is adopted; current pin `d9be7d8c` stays. Uncertainty flag changed from "re-confirm required" to "deferred-not-applicable." |
| Cache budget hypothesis | **CONFIRMED DATA-BACKED.** `gh api repos/drbothen/slideforge/actions/cache/usage` (2026-06-10) = 9.77 GB / 23 active caches = 97.7% of limit. `v0-rust-test-linux-arm64-*` cache ABSENT (LRU-evicted). Root cause confirmed. AC-005 reframed from "verify hypothesis" to "measure fix effectiveness against confirmed 9.77 GB baseline." |
| Repo owner placeholder (`<owner>`) | Confirmed: `drbothen/slideforge`, PUBLIC. All `<owner>` placeholders replaced. |
| AC-006 fixed threshold ("< 12 min") | Reframed: measure cold vs warm wall-clock; record both; do not gate on a fixed threshold. The 12-minute figure was a directional estimate. |
| rust-cache old SHA call-site count | Research stated 11 call-sites in `ci.yml`. Implementer must grep-verify at implementation time; the grep-zero check is required regardless. |

### Deferred to Implementation (verify-at-implementation markers)

| Item | Resolution Command / Check |
|------|---------------------------|
| arm64 disk-cleanup path list | Run `ls -la /usr/share/dotnet /usr/local/lib/android /opt/ghc /opt/hostedtoolcache/CodeQL /usr/local/share/boost` on the live `ubuntu-24.04-arm` runner (or check the ubuntu-24.04-arm partner image manifest) to confirm which paths exist. Guard all `rm -rf` with `|| true` regardless. |
| Post-fix cache budget | Re-run `gh api repos/drbothen/slideforge/actions/cache/usage` after implementation; confirm total < ~8 GB and arm64 key present. |

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-06-10 | story-writer | Initial creation per human direction 2026-06-10. Grounded in ci-speed-research.md Q2 + Q3. Fixes root cause of arm64 cold-build flakiness and `No space left` class. Depends on STORY-091 (tiered trigger must be live first). |
| 1.1 | 2026-06-10 | story-writer | remove-uncertainty pass: confirmed rust-cache v2.9.1 SHA `c19371144...` and free-disk-space v1.3.1 SHA `54081f13...` via `git ls-remote` 2026-06-10; promoted cache-budget finding from hypothesis to DATA-BACKED (9.77 GB/23 caches/arm64-evicted confirmed); reframed AC-005 from "verify hypothesis" to "measure fix against confirmed baseline"; reframed AC-006 wall-clock from fixed <12 min to measure-and-record; added grep-zero check requirement for 11-site rust-cache SHA sweep (TD-VSDD-060); added arm64 disk-path verify-at-impl marker with `|| true` guards; resolved `<owner>` placeholder as `drbothen/slideforge`; noted `taiki-e/install-action` SHA as deferred; added Uncertainty Resolution Log. |
| 1.2 | 2026-06-11 | story-writer | Spec amendment: ci-workflow-analyzer MED findings 1-2 on feature/STORY-092 @ c264910a (VERIFIED via GitHub API). Replaced false Class B contributing-factor claim ("hash-calculation regression fix") with accurate tag-object-fragility rationale: old SHA `42dc69e1aa...` is the annotated tag-object for floating `v2` — already running v2.9.1 code (`e18b4977`), zero behavioral delta from bump; value is GC fragility elimination. Updated AC-003, Fix paragraph, Task B, Library Requirements table, Pinned Versions table, Architecture Compliance Rule 6, and Uncertainty Resolution Log entry accordingly. Added note that duplicate per-shared-key cache entries are normal lockfile-hash rotation and will persist post-fix. Reconciled implementation site counts: 11 in ci.yml + 8 in companion workflows (security.yml codeql-action had same tag-object defect, re-pinned to `03e4368a...`) = 19 total bumps — recorded as implementation note in Class B Fix paragraph; no new ACs added. |
