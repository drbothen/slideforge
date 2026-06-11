---
document_type: demo-evidence
story_id: STORY-092
title: "CI: Cache reliability + disk headroom — per-AC structural verification + pre-merge baseline"
spec_version: "1.3"
implementation_sha: c264910a
branch: feature/STORY-092
captured: 2026-06-10
tdd_mode: facade
---

# STORY-092 Demo Evidence

`tdd_mode: facade` — no Rust unit tests. Verification is structural
(YAML grep, line-number audit) plus live GitHub API for the pre-merge
cache baseline. Live `df -h` deltas and consecutive warm-cache timing
require post-merge CI runs and are deferred per spec (see AC-001, AC-006).

All captures are from `feature/STORY-092` at `c264910a`.
All structural checks run against `.worktrees/STORY-092/.github/workflows/`.

---

## AC-001: Disk-space reclamation step present in all build-heavy jobs

**Criterion (story spec):** A "Free disk space" step MUST appear immediately after
`actions/checkout` (before toolchain/cache/cargo steps) in all build-heavy jobs.
Each `rm -rf` MUST be guarded with `|| true`. `test-matrix-slow` MUST have
`if: runner.os == 'Linux'` so macOS and Windows legs skip the step.
`visual-regression` MUST use a narrowed cleanup (no `/usr/local/share/boost`;
no Python/apt removal). Live `df -h` deltas deferred to CI.

### Structural grep

```
$ grep -n "Free disk\|rm -rf /usr/share/dotnet\|df -h" \
    .worktrees/STORY-092/.github/workflows/ci.yml
```

```
152:      - name: Free disk space (Linux only)
156:          df -h
157:          sudo rm -rf /usr/share/dotnet || true
158:          sudo rm -rf /usr/local/lib/android || true
159:          sudo rm -rf /opt/ghc || true
160:          sudo rm -rf /opt/hostedtoolcache/CodeQL || true
161:          sudo rm -rf /usr/local/share/boost || true
162:          sudo docker image prune --all --force || true
163:          df -h
205:      - name: Free disk space (Linux only)
209:          df -h
210:          sudo rm -rf /usr/share/dotnet || true
...
216:          df -h
348:      - name: Free disk space (Linux only)
352:          df -h
353:          sudo rm -rf /usr/share/dotnet || true
...
359:          df -h
463:      - name: Free disk space (Linux only)
467:          df -h
468:          sudo rm -rf /usr/share/dotnet || true
...
474:          df -h
524:      - name: Free disk space (Linux only)
528:          df -h
529:          sudo rm -rf /usr/share/dotnet || true
...
535:          df -h
589:      - name: Free disk space (Linux only — narrowed for LibreOffice compatibility)
593:          df -h
599:          sudo rm -rf /usr/share/dotnet || true
600:          sudo rm -rf /usr/local/lib/android || true
601:          sudo rm -rf /opt/ghc || true
602:          sudo rm -rf /opt/hostedtoolcache/CodeQL || true
603:          sudo docker image prune --all --force || true
604:          df -h
```

### Job inventory (6 jobs with cleanup step)

| ci.yml line | Job name | Step name | Position in job |
|---|---|---|---|
| 146/152 | `clippy` | `Free disk space (Linux only)` | immediately after checkout |
| 199/205 | `test` (linux-x86_64) | `Free disk space (Linux only)` | immediately after checkout |
| 325/348 | `test-matrix-slow` (all 3 legs) | `Free disk space (Linux only)` | immediately after checkout; guarded `if: runner.os == 'Linux'` |
| 449/463 | `snapshots` | `Free disk space (Linux only)` | immediately after checkout |
| 510/524 | `bench` | `Free disk space (Linux only)` | immediately after checkout |
| 575/589 | `visual-regression` | `Free disk space (Linux only — narrowed for LibreOffice compatibility)` | immediately after checkout; **narrowed** (no `/usr/local/share/boost`; no Python/apt) |

### runner.os guard on test-matrix-slow (line 348–362 excerpt)

```yaml
  test-matrix-slow:
    name: test (${{ matrix.target.name }})
    runs-on: ${{ matrix.target.runner }}
    ...
    strategy:
      matrix:
        target:
          - { name: "linux-arm64",    runner: "ubuntu-24.04-arm" }
          - { name: "macos-arm64",    runner: "macos-latest"      }
          - { name: "windows-x86_64", runner: "windows-latest"    }
    steps:
      - uses: actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5
      - name: Free disk space (Linux only)
        if: runner.os == 'Linux'          # ← guard: macos + windows legs skip this
        run: |
          ...
          sudo rm -rf /usr/share/dotnet || true
          sudo rm -rf /usr/local/lib/android || true
          sudo rm -rf /opt/ghc || true
          sudo rm -rf /opt/hostedtoolcache/CodeQL || true
          sudo rm -rf /usr/local/share/boost || true
          sudo docker image prune --all --force || true
          ...
          # Each rm -rf is guarded (|| true) so missing paths on ubuntu-24.04-arm
          # do not fail the step. Path list is x86_64-derived; arm64 image may
          # differ — guards prevent errors on absent paths (EC-001).
```

### visual-regression narrowed cleanup (lines 589–605 excerpt)

```yaml
      - name: Free disk space (Linux only — narrowed for LibreOffice compatibility)
        if: runner.os == 'Linux'
        run: |
          # Narrowed cleanup: remove .NET, Android SDK, GHC, CodeQL, and prune
          # Docker images only. Do NOT remove /usr/local/lib/python* or general
          # apt packages — LibreOffice and scikit-image depend on them (EC-002).
          sudo rm -rf /usr/share/dotnet || true
          sudo rm -rf /usr/local/lib/android || true
          sudo rm -rf /opt/ghc || true
          sudo rm -rf /opt/hostedtoolcache/CodeQL || true
          sudo docker image prune --all --force || true
          # /usr/local/share/boost deliberately omitted — not present on this image.
          # Each rm -rf is guarded (|| true) so missing paths do not fail the step.
```

### Deferred: live df -h deltas

Actual before/after disk deltas require a live CI run on the `ubuntu-24.04-arm`
runner. Deferred to post-merge CI observation per AC-001 spec framing
("Confirm no `No space left` in CI logs for 3 consecutive merge-queue runs").

**AC-001: PASS (structural)** — 6 jobs have the cleanup step immediately after
checkout; all 6 use `|| true` guards; test-matrix-slow has `if: runner.os == 'Linux'`;
visual-regression uses narrowed list (no boost, no Python/apt removal).

---

## AC-002: `cache-on-failure: "true"` present on all build-heavy jobs

**Criterion:** Every Rust-compiling job MUST have `cache-on-failure: "true"` on its
`Swatinem/rust-cache` step. Value must be the string `"true"`, not boolean.

### Structural grep

```
$ grep -n "cache-on-failure" \
    .worktrees/STORY-092/.github/workflows/ci.yml
```

```
173:          cache-on-failure: "true"
225:          cache-on-failure: "true"
260:          cache-on-failure: "true"
279:          cache-on-failure: "true"
370:          # cache-on-failure: "true" — saves cache even when the job is
373:          cache-on-failure: "true"
414:          cache-on-failure: "true"
442:          cache-on-failure: "true"
483:          cache-on-failure: "true"
544:          # cache-on-failure: "true" — saves cache even when the job is canceled
546:          cache-on-failure: "true"
612:          cache-on-failure: "true"
693:          cache-on-failure: "true"
```

11 occurrences. Each is the string `"true"` (quoted, not bare boolean).

### Job mapping

| ci.yml line | Job | Had it before STORY-092? |
|---|---|---|
| 173 | `clippy` | no — added by STORY-092 |
| 225 | `test` (linux-x86_64) | yes (PR #79) |
| 260 | `doctest` | no — added by STORY-092 |
| 279 | `supply-chain` | no — added by STORY-092 |
| 373 | `test-matrix-slow` (linux-arm64 leg) | yes (PR #79) |
| 414 | `msrv` | no — added by STORY-092 |
| 442 | `docs` | no — added by STORY-092 |
| 483 | `snapshots` | no — added by STORY-092 |
| 546 | `bench` | yes (PR #81) |
| 612 | `visual-regression` | no — added by STORY-092 |
| 693 | `perf-smoke` | no — added by STORY-092 |

**AC-002: PASS** — 11 rust-cache steps all have `cache-on-failure: "true"` (string).

---

## AC-003: rust-cache bumped to peeled commit SHA at ALL call-sites in ci.yml

**Criterion:** All `Swatinem/rust-cache` uses in `ci.yml` MUST reference
`c19371144df3bb44fab255c43d04cbc2ab54d1c4`. The old tag-object SHA
`42dc69e1aa15d09112580998cf2ef0119e2e91ae` MUST NOT remain.

### New SHA count and locations

```
$ grep -n "Swatinem/rust-cache" \
    .worktrees/STORY-092/.github/workflows/ci.yml
```

```
 74:#   Swatinem/rust-cache     v2.9.1  → c19371144df3bb44fab255c43d04cbc2ab54d1c4
170:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
222:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
257:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
276:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
367:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
411:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
439:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
480:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
541:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
609:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
690:      - uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4
```

11 `uses:` call-sites (line 74 is a comment, not a call-site). 1 comment header.

### Old SHA grep-zero check (TD-VSDD-060)

```
$ grep -c "42dc69e1" \
    .worktrees/STORY-092/.github/workflows/ci.yml
0
```

Result: **0** — no residual old SHA. TD-VSDD-060 sibling-site sweep: CLEAN.

### Companion workflow sweep (implementation note, outside AC-003 scope)

The story spec notes that companion workflows were swept as an implementation
detail (not a new AC). Counts at `c264910a`:

```
$ grep -c "c19371144df3bb44fab255c43d04cbc2ab54d1c4" \
    .worktrees/STORY-092/.github/workflows/security.yml
2

$ grep -c "c19371144df3bb44fab255c43d04cbc2ab54d1c4" \
    .worktrees/STORY-092/.github/workflows/pdf-ua1.yml
3

$ grep -c "c19371144df3bb44fab255c43d04cbc2ab54d1c4" \
    .worktrees/STORY-092/.github/workflows/html-wcag.yml
3

$ grep -c "c19371144df3bb44fab255c43d04cbc2ab54d1c4" \
    .worktrees/STORY-092/.github/workflows/release.yml
4
```

Total: 11 (ci.yml) + 2 + 3 + 3 + 4 (companions) = **23 call-sites** across
all 5 workflows. (The spec predicted 11 + 8 = 19; the discrepancy is 4 extra
companion sites, likely from release.yml having more rust-cache steps than
anticipated at story authoring time.)

### codeql-action re-pin in security.yml

The `github/codeql-action` pin had the same annotated-tag-object fragility.
Re-pinned to peeled commit `03e4368ac7daa2bd82b3e85212f3bf87ee112f57`
(security.yml line 221). Old tag-object SHA `fee9466b` count in security.yml:

```
$ grep -c "fee9466b" \
    .worktrees/STORY-092/.github/workflows/security.yml
0
```

Result: **0** — old codeql SHA absent.

**AC-003: PASS** — 11 ci.yml call-sites at new peeled commit SHA; old SHA
`42dc69e1` count = 0; companion workflows swept as implementation detail;
codeql re-pin also clean.

---

## AC-004: Cache budget documented in playbook §9

**Criterion:** `docs/playbooks/tiered-ci-merge-queue.md` MUST have a §9 section
with a cache budget table, the confirmed 9.77 GB baseline, projections, and
`gh cache` commands.

### Section listing

```
$ grep -n "^##\|^###" \
    .worktrees/STORY-092/docs/playbooks/tiered-ci-merge-queue.md
...
440:## 9. Cache budget (STORY-092)
451:### 9.1 Active cache keys and estimated sizes
504:### 9.2 STORY-092 fixes that reduce budget pressure
517:### 9.3 Verification
536:### 9.4 Jobs with `cache-on-failure: "true"` (post-STORY-092)
```

### §9 content verification

**Budget table (§9.1, lines 454–481)** — present. Lists 21 shared-keys with
workflow source, platform, and measured/estimated per-key `target/` sizes.
Key entries:

| shared-key | platform | measured size |
|---|---|---|
| `test-linux-arm64` | linux-arm64 | 0 (evicted) — primary reliability issue |
| `test-linux-x86_64` | linux-x86_64 | ~772 MiB |
| `snapshots` | linux-x86_64 | ~744 MiB |
| `test-windows-x86_64` | windows-x86_64 | ~759 MiB |
| `bench` | linux-x86_64 | ~620 MiB |
| (all others) | linux-x86_64 / multi | ~98–698 MiB |

Active (non-release) total: **~6.1 GiB**. With arm64 restored: **~7.0 GiB**
(within ~8 GiB target; ~1 GiB headroom).

**Baseline statement (§9, lines 443–449)** — present:
> "Baseline (2026-06-10, pre-fix): 9.77 GB / 23 active caches = 97.7% of the
> ~10 GB per-repo limit. `v0-rust-test-linux-arm64-*` was ABSENT (LRU-evicted)
> — confirmed root cause of arm64 cold-build flakiness."
> "Measured (2026-06-11, post-STORY-091 merge, pre-STORY-092 merge):
> `gh api repos/drbothen/slideforge/actions/cache/usage` → 9.66 GB / 22 active
> caches. The arm64 test cache remains absent."

**Projections / fix explanation (§9.2)** — present. Three levers documented:
re-pin rationale, `cache-on-failure` explanation, no-consolidation-needed
conclusion.

**`gh cache` commands (§9.3)** — present:

```bash
# Check total budget:
gh api repos/drbothen/slideforge/actions/cache/usage

# Verify arm64 cache is present:
gh cache list --repo drbothen/slideforge --limit 100 | grep arm64

# List entries for a specific shared-key:
gh cache list --repo drbothen/slideforge --key v0-rust-clippy

# Delete a specific stale cache entry by ID:
gh cache delete --repo drbothen/slideforge <cache-id>
```

**§9.4 jobs table** — present. Lists all 11 jobs with `cache-on-failure: "true"`,
tier, and whether STORY-092 added it or it pre-existed.

**AC-004: PASS** — §9 present with budget table, confirmed baseline
(9.77 GB / 23 caches / arm64 absent), projections (7.0 GiB post-fix), and
`gh cache` verification commands.

---

## AC-005: Cache budget confirmed DATA-BACKED; pre-merge baseline captured

**Criterion:** Capture pre-merge `gh api repos/drbothen/slideforge/actions/cache/usage`
as baseline. Post-merge measurement is deferred (requires PR merge to develop
plus a full-matrix develop push).

### Pre-merge baseline (captured 2026-06-10 at story authoring, confirmed 2026-06-11 pre-merge)

```
$ gh api repos/drbothen/slideforge/actions/cache/usage
{
  "full_name": "drbothen/slideforge",
  "active_caches_size_in_bytes": 10375369802,
  "active_caches_count": 22
}
```

Converted: **9.66 GiB** / **22 active caches**. (9.77 GB / 23 caches on
2026-06-10 at story authoring; one cache expired between then and now —
normal LRU rotation.)

**arm64 cache presence check:**

```
$ gh cache list --repo drbothen/slideforge --limit 100 | grep arm64
4890179221  v0-rust-test-macos-arm64-Darwin-arm64-90f75ef2-4188dcc0  697.63 MiB  ...
```

`v0-rust-test-linux-arm64-*` is **absent** from the active cache list.
Only `test-macos-arm64` appears. This confirms the pre-fix state documented
in the spec: linux-arm64 cache is LRU-evicted, perpetuating cold-build loops.

### Post-merge measurement (deferred)

After STORY-092 merges to develop and a subsequent full-matrix push completes:

1. Re-run `gh api repos/drbothen/slideforge/actions/cache/usage`.
2. Expected: total < 9.66 GiB (budget not worse; arm64 savings in subsequent
   runs as cache-on-failure kicks in).
3. Run `gh cache list --repo drbothen/slideforge --limit 100 | grep arm64`.
4. Expected: `v0-rust-test-linux-arm64-*` present with a recent `created_at`.

The AC-005 pass/fail on the fix effectiveness measure is a post-merge gate.
The structural fix (cache-on-failure, re-pin) is verified by AC-002 and AC-003.

**AC-005: PRE-MERGE BASELINE CAPTURED** — 9.66 GiB / 22 caches / arm64 absent
(2026-06-10/11). Post-merge measurement deferred per spec.

---

## AC-006: arm64 test job shows warm cache on consecutive runs

**Criterion:** After merge, `test (linux-arm64)` MUST show cache HIT on the
second consecutive run. Wall-clock cold vs warm figures to be recorded in PR.

### Pre-fix evidence: arm64 cold-build failure at run 27317895683

```
$ gh run view 27317895683 --repo drbothen/slideforge

  develop CI · 27317895683
  Triggered via push about 1 hour ago

  JOBS
  ✓ test (linux-x86_64) in 6m20s
  X test (linux-arm64) in 1h5m14s
    ✓ Set up job
    ✓ Run actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5
    ✓ Run dtolnay/rust-toolchain@...
    ✓ Run Swatinem/rust-cache@42dc69e1aa15d09112580998cf2ef0119e2e91ae
    ✓ Install core fonts (Linux only)
    ✓ Install cargo-nextest
    * cargo nextest run         ← timed out / lost communication after 1h05m
    * Post Run Swatinem/rust-cache@42dc69e1...   ← cache NOT saved (job failed)
  ✓ test (macos-arm64) in 9m30s
  ✓ bench in 16m27s
```

This is the pre-fix cold-build loop: arm64 ran for 1h05m (cold build, cache
absent), then the job failed or timed out before completing, and
`cache-on-failure: "true"` was NOT set — so the cache was not saved. The next
run starts cold again.

Note: the run shows `Swatinem/rust-cache@42dc69e1aa15d09112580998cf2ef0119e2e91ae`
(old tag-object SHA) — confirming this was a pre-STORY-092 run.

### STORY-092 fix rationale

STORY-092 adds two changes that together break the loop:
1. `cache-on-failure: "true"` on `test-matrix-slow` — the arm64 cache is now
   saved even when the job times out, so the next run starts warm.
2. rust-cache re-pin to stable peeled commit — eliminates the risk of
   tag-object GC causing a cache-save miss at the framework level.

### Deferred: warm run measurement

Two consecutive develop runs post-merge are needed to record:
- Cold run wall-clock (first run after STORY-092 merge — still cold, resets key)
- Warm run wall-clock (second run — should show cache HIT)

Record both in the PR description. The AC passes when arm64 shows "restored
cache" in the rust-cache step log on the warm run.

**AC-006: PRE-FIX EVIDENCE CAPTURED** — run 27317895683 shows 1h05m arm64
cold-build failure with no cache save (pre-STORY-092 state). Post-merge
warm-cache verification deferred per spec.

---

## Actionlint

**Scope:** 4 STORY-092-modified workflows (ci.yml, security.yml, pdf-ua1.yml,
html-wcag.yml). `release.yml` was modified for the rust-cache SHA sweep but
carries a pre-existing SC2086 shellcheck advisory on lines 240 (unrelated to
STORY-092 changes; present identically on develop).

### Run on 4 modified workflows

```
$ actionlint \
    .worktrees/STORY-092/.github/workflows/ci.yml \
    .worktrees/STORY-092/.github/workflows/security.yml \
    .worktrees/STORY-092/.github/workflows/pdf-ua1.yml \
    .worktrees/STORY-092/.github/workflows/html-wcag.yml

(no output)
exit code: 0
```

**CLEAN** — zero findings on all 4 STORY-092-modified workflows.

### Pre-existing finding in release.yml (not introduced by STORY-092)

```
release.yml:240:9: shellcheck reported issue in this script:
  SC2086:info:8:41: Double quote to prevent globbing and word splitting
release.yml:240:9: shellcheck reported issue in this script:
  SC2086:info:9:22: Double quote to prevent globbing and word splitting
```

Confirmed pre-existing by running actionlint against `develop:.github/workflows/release.yml`
(same lines, same finding). STORY-092 made only a SHA substitution in release.yml;
the SC2086 advisory is not in scope for this story.

---

## Coverage Summary

| AC | Description | Status | Evidence |
|---|---|---|---|
| AC-001 | Disk cleanup in 6 jobs, immediately after checkout, `\|\| true` guards, `runner.os` guard on test-matrix-slow, narrowed cleanup on visual-regression | **PASS (structural)** | Grep output above; live `df -h` deferred to CI |
| AC-002 | `cache-on-failure: "true"` at 11 rust-cache sites | **PASS** | 11 grep hits; all string `"true"` |
| AC-003 | New peeled commit SHA at all 11 ci.yml call-sites; old SHA count = 0 | **PASS** | grep counts above; TD-VSDD-060 clean |
| AC-004 | Playbook §9 with budget table, confirmed 9.77 GB baseline, projections, `gh cache` commands | **PASS** | Section listing + content excerpts above |
| AC-005 | Pre-merge baseline captured: 9.66 GiB / 22 caches / arm64 absent | **BASELINE CAPTURED** | API output above; post-merge measurement deferred |
| AC-006 | Pre-fix evidence: run 27317895683 — arm64 1h05m cold-build, no cache save | **PRE-FIX EVIDENCE CAPTURED** | `gh run view` output above; warm-run verification deferred |
| Actionlint | 4 modified workflows clean; release.yml advisory pre-existing | **CLEAN (4/4 modified)** | exit code 0; pre-existing advisory noted |

**Deferred-to-CI items** (require post-merge develop runs):
- AC-001: `df -h` before/after disk deltas on live `ubuntu-24.04-arm` runner
- AC-005: post-merge total cache size below 9.66 GiB baseline; arm64 cache key present
- AC-006: two consecutive develop runs; warm run shows "restored cache"; cold vs warm wall-clock figures

These deferrals are explicitly sanctioned by the story spec's Test Strategy section:
> "AC-001: Confirm no `No space left` in CI logs for 3 consecutive merge-queue runs."
> "AC-005: re-run `gh api` after fix lands; compare against 9.77 GB baseline."
> "AC-006: PR description includes before/after arm64 wall-clock from two consecutive develop pushes."
