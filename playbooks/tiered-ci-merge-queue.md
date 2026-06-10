# Playbook: Tiered CI + Merge Queue

**document_type:** playbook
**version:** 1.0
**date:** 2026-06-10
**status:** active
**portability:** repo-agnostic (slideforge worked example in Appendix A)

> A copy of this playbook belongs in each adopting repo's `docs/playbooks/` directory.

---

## What problem this solves / when to use it

In a multi-platform CI matrix, slow or resource-intensive legs — full cross-platform test matrices,
benchmarks, visual-regression jobs — run on every PR push by default. Every developer iteration waits
on the slowest, most flake-prone leg. Per-PR feedback times that could be 3–8 minutes instead stretch
to 20–60+ minutes. Infrastructure flakes (cache thrash, disk exhaustion, runner resource starvation)
hit on every push and compound the cost.

This playbook applies the **tiered CI + merge queue** pattern to address that:

- **Fast legs** (single-platform build + test, lint, format, docs, security scan) run on every PR
  push. They catch the large majority of issues within minutes.
- **Slow legs** (full multi-platform matrix, benchmarks, expensive validators) move to the **merge
  queue** — they run only once, just before a PR lands, on the merge-queue synthetic branch. They
  also run nightly and on protected branches for ongoing health monitoring.

The result: per-PR developer feedback time drops to the fast-leg time. The full matrix still guards
the default branch — no regressions slip through — but developers stop waiting on it for every
iteration.

**When to apply this pattern:**

- You have a CI matrix with at least one leg meaningfully slower than the others (2x or more).
- Slow legs are compile-bound, emulation-bound, or involve large external tool installations.
- Per-PR feedback time is a friction point for the team.
- Your hosting platform supports a merge queue mechanism (this playbook uses GitHub Actions as the
  worked example; the pattern is transferable to any platform with equivalent primitives).

---

## Before / After

### Critical path per PR push

| | Before | After |
|---|---|---|
| What runs | All legs: fast + slow (full matrix) | Fast legs only |
| Typical wall-clock | Slowest leg dominates (e.g. 20–60 min) | Fast legs only (e.g. 3–8 min) |
| Full matrix runs | On every push | On merge-queue entry, nightly, default-branch push, or `full-ci` label |
| Default branch protection | Same slow gates | Same gates, still enforced — at merge time |
| Flake blast radius | Every push | Contained to merge-queue and nightly runs |

The full matrix still runs before anything merges. The default branch remains protected. Developer
iteration speed improves; merge safety does not regress.

---

## Step-by-step adoption checklist

The steps below are written generically. GitHub Actions snippets are included as illustrative
examples; adapt them to your platform and workflow file names.

### Step 1 — Classify your jobs as FAST or SLOW

Walk your current CI jobs and label each:

| Leg | Criteria for FAST | Criteria for SLOW |
|-----|-------------------|-------------------|
| Lint / format check | Typically fast — FAST | — |
| Single-platform build + test | Fast wall-clock, high-signal — FAST | — |
| Security scan (SAST, audit) | Usually fast — FAST | — |
| Doc build | Usually fast — FAST | — |
| Full multi-platform test matrix | Cross-platform, slow or flaky — SLOW | — |
| Benchmarks / performance gate | Expensive, noise-sensitive — SLOW | — |
| Visual regression / screenshot diff | Requires rendering toolchain — SLOW | — |
| Fuzz / mutation testing | Long-running by design — SLOW | — |

The "FAST single-platform" leg should be your highest-signal, fastest-feedback leg — usually the
same platform as most developers' machines (commonly linux-x86_64 or macOS-arm64 for Apple Silicon
teams).

### Step 2 — Enable the merge queue on your repository

In GitHub: **Repository Settings → General → Pull Requests → Merge queue** — enable it.

Configure the merge queue method (merge commit, squash, or rebase) to match your branch model.

### Step 3 — Add `merge_group:` to your workflow trigger

In every workflow file that produces required status checks, add `merge_group:` to the `on:` block:

```yaml
on:
  pull_request:
    branches: [main, develop]
  push:
    branches: [main, develop]
  merge_group:              # <-- add this
  schedule:
    - cron: "0 6 * * *"    # nightly — keeps slow legs from going stale
```

Without `merge_group:`, required checks are never reported on the merge-queue synthetic branch and
the queue stalls permanently.

### Step 4 — Gate slow legs with a job-level `if:` condition

Do NOT use workflow-level `paths`/`paths-ignore` to skip slow jobs (see the deadlock pitfall
section below). Use a job-level `if:` instead, so the workflow always starts and always reports
a named status.

```yaml
jobs:
  # --- FAST legs: run on every trigger ---
  lint:
    runs-on: ubuntu-latest
    steps: [...]

  test-linux-x86_64:
    runs-on: ubuntu-latest
    steps: [...]

  # --- SLOW legs: only on merge_group, nightly, default branch, or label ---
  test-full-matrix:
    strategy:
      matrix:
        platform: [macos-latest, windows-latest, ubuntu-24.04-arm]
    runs-on: ${{ matrix.platform }}
    if: >
      github.event_name == 'merge_group' ||
      github.event_name == 'schedule' ||
      (github.event_name == 'push' &&
       github.ref == 'refs/heads/main') ||
      (github.event_name == 'pull_request' &&
       contains(github.event.pull_request.labels.*.name, 'full-ci'))
    steps: [...]
```

Adjust the branch name (`main`/`develop`/`trunk`) and label name (`full-ci`) to match your repo.

### Step 5 — Create an aggregator job

Add a single synthetic aggregator job that `needs:` every other job in the workflow. This job:

- Always runs (via `if: always()`)
- Checks whether any job that *actually ran* failed (treats `skipped` as passing)
- Is the **only** job you make a required status check

```yaml
  all-checks-pass:
    name: all-checks-pass
    runs-on: ubuntu-latest
    if: always()
    needs:
      - lint
      - test-linux-x86_64
      - test-full-matrix
      # list every job in the workflow here
    steps:
      - name: Check all jobs
        run: |
          results='${{ toJSON(needs) }}'
          # Fail if any job that actually ran ended in failure or was cancelled
          if echo "$results" | grep -q '"result": "failure"'; then
            echo "One or more required jobs failed."
            exit 1
          fi
          if echo "$results" | grep -q '"result": "cancelled"'; then
            echo "One or more jobs were cancelled."
            exit 1
          fi
          echo "All checks passed (skipped jobs treated as OK)."
```

The key invariant: `skipped` (a slow leg that did not run because the `if:` condition was false)
counts as passing. `failure` and `cancelled` count as failing.

### Step 6 — Make ONLY the aggregator a required status check

In **Repository Settings → Branches → Branch protection rules** (or rulesets):

- Required status checks: add **only** `CI / all-checks-pass` (or whatever your workflow's
  aggregator job is named — format is `<workflow name> / <job name>`)
- Remove any slow individual legs from the required checks list

This is the critical step. The aggregator is always reported on every trigger (it runs even when
slow legs are skipped), so branch protection never deadlocks.

### Step 7 — Add an opt-in label

Create a `full-ci` label (or equivalent) in your GitHub repo. Any PR author can apply it to
trigger the full matrix on their next push — useful for cross-platform bug investigations or
before marking a PR ready for review.

### Step 8 — Verify the nightly schedule

Confirm the `schedule:` trigger is present and set to a reasonable cadence (daily is typical).
The nightly run exercises all slow legs so you catch platform-specific regressions before merge
queue time.

### Step 9 — Test the configuration

1. Open a PR without the `full-ci` label — confirm only fast legs run, aggregator reports green.
2. Add the `full-ci` label to the PR — confirm slow legs are triggered on the next push.
3. Queue the PR for merge — confirm the merge queue runs both fast and slow legs.
4. Verify the nightly run in the Actions tab after 24 hours.

---

## The deadlock pitfall

> **This is the single most common misconfiguration. Read carefully before touching
> branch protection settings.**

### What causes the deadlock

GitHub branch protection waits for a required status check to be reported. If a required check
is never reported — because the workflow or job was skipped entirely — the branch protection
waits indefinitely and every merge attempt hangs.

This happens when:

1. A workflow file uses `paths` or `paths-ignore` at the **workflow level** to skip the entire
   workflow run on certain file changes. If that workflow contains a required check job, and a PR
   touches only the filtered-out paths, the workflow never starts, the check is never reported,
   and the PR is permanently blocked.

2. An individual job is made a required check AND that job has a conditional `if:` that evaluates
   to false for normal PRs. The job is `skipped`, which GitHub treats differently from `success` —
   branch protection does not accept `skipped` as satisfying a required check for an individual
   job.

### How this playbook avoids the deadlock

- Slow jobs use job-level `if:` (not workflow-level `paths`/`paths-ignore`). The workflow always
  starts and always runs the aggregator.
- The aggregator job runs with `if: always()` — it runs unconditionally regardless of what other
  jobs did.
- The aggregator explicitly treats `skipped` as passing. It only fails when a job that actually
  ran returned `failure` or `cancelled`.
- **Only the aggregator is a required status check.** The aggregator is never skipped, so branch
  protection always gets a report.

Individual slow jobs are NOT required checks. They are inputs to the aggregator. The aggregator
is the single gate.

### Correct pattern (safe)

```
branch protection requires: "CI / all-checks-pass"
aggregator needs: [lint, test-x86_64, test-full-matrix, ...]
aggregator if: always()
aggregator logic: skip == OK, failure/cancelled == FAIL
slow job if: merge_group || schedule || push-to-main || full-ci-label
```

### Dangerous anti-pattern (deadlock)

```
branch protection requires: "CI / test-full-matrix"   # <-- individual slow job
test-full-matrix if: merge_group || ...                # <-- skipped on normal PRs
result: branch protection waits forever on normal PRs
```

---

## Cache reliability companion practices

### Save cache on failure and cancellation

Build-heavy jobs that time out or fail before saving their cache leave the next run cold. On a
CI system where jobs are cancelled or time out regularly (as can happen with resource-constrained
runners or disk-full failures), this creates a self-perpetuating loop: cold build → slow → timed
out → cache lost → cold build.

Fix: ensure your build cache action saves even when the job fails or is cancelled.

**For Rust projects using `Swatinem/rust-cache`:**

```yaml
- uses: Swatinem/rust-cache@<pinned-sha>
  with:
    cache-on-failure: "true"   # saves cache even when the job fails or is cancelled
    shared-key: "test-${{ matrix.platform }}"
```

Apply `cache-on-failure: "true"` to every build-heavy job (not just test jobs — also lint, doc
build, or any job that compiles significant amounts of code).

**For other ecosystems:** check whether your cache action has an equivalent "save on failure"
option or whether you need to add an explicit cache-save step in a `if: always()` post-step.

### Keep distinct cache keys within your platform's cache budget

GitHub Actions provides approximately 10 GB of cache storage per repository, evicted by LRU when
the limit is exceeded. In a large workspace with many CI jobs (each using a distinct cache key),
the combined cache size can easily exceed this budget. When eviction kicks in, caches thrash: each
run evicts another job's cache, and no leg stays consistently warm.

Strategies:

- Use a `shared-key` (or equivalent) per logical job class rather than per individual run. A test
  matrix with four platforms needs four shared keys; it does not need a new key per branch.
- For compiled languages, `CARGO_INCREMENTAL=0` (or equivalent) prevents incremental build
  artifacts from inflating the cache beyond useful size. Incremental artifacts are rarely reused
  across ephemeral CI runs and bloat `target/`.
- Keep the number of distinct cache slots to the minimum needed to give each job class a warm hit.
  Audit your actual cache usage (GitHub UI: Actions → Caches) to see whether eviction is
  occurring before optimizing.

### Do not combine two caching systems that share the same budget

If your build cache action already caches the compiled output directory, adding a compiler-
wrapper cache (e.g. sccache on top of `Swatinem/rust-cache`) means both systems draw from the
same ~10 GB budget. The compiler-wrapper cache rarely fires when the output directory is warm
(the compiler is not invoked), but it still occupies budget. The net effect is more budget
contention, not more cache hits.

When the budget is the binding constraint, remove redundant caching layers.

---

## Disk headroom companion practices

Hosted CI runners have a fixed disk size (GitHub's standard ubuntu runners provide approximately
14 GB total). Large build toolchains preinstalled on the runner image (Android SDK, .NET, Haskell
GHC, Docker images, CodeQL) consume several gigabytes of that space before your build starts.
A large compiled workspace can exhaust the remaining space mid-build, producing cryptic
"No space left on device" errors that silently cost a full re-run.

### Reclaim disk space before build-heavy jobs

A simple manual cleanup step reclaims 10–15 GB without adding third-party action dependencies:

```yaml
- name: Reclaim disk space
  run: |
    sudo rm -rf /usr/share/dotnet /usr/local/lib/android /opt/ghc \
                /opt/hostedtoolcache/CodeQL /usr/local/share/boost
    sudo docker image prune --all --force || true
    df -h
```

Add this step near the top of any job that does a large build (`df -h` at the end confirms what
was reclaimed). Tune which directories to remove based on what your job actually needs — for
example, do not remove a directory that a later step in the same job installs from.

If the manual approach proves insufficient (rare for most projects), a third-party disk-space
action such as `jlumbroso/free-disk-space` can reclaim additional space at the cost of a longer
setup step (~1–3 minutes) and an added supply-chain dependency.

**Apply disk reclamation to:** full-matrix test jobs, snapshot/visual-regression jobs, benchmark
jobs, and any job that downloads and builds large external tooling.

---

## Build-time levers (brief overview)

These are compile-time optimization techniques that reduce the cost of the slow legs themselves.
They are secondary to the tiered-trigger structural change (which removes slow legs from the
per-PR critical path entirely), but they matter for merge-queue latency and nightly run health.

### Faster linker

On Linux, the default GNU linker (`ld`) is slower than alternatives. Where it is not already the
default, switching to a faster linker reduces link time:

- **Rust on Linux (x86_64):** `rust-lld` is the default linker on `x86_64-unknown-linux-gnu`
  since Rust 1.90 (September 2025) — no action needed on that target.
- **Rust on Linux (aarch64):** `rust-lld` is not the default on `aarch64-unknown-linux-gnu`.
  `mold` provides substantial link-time improvement on this target. Apply it via a CI setup action
  or `.cargo/config.toml` scoped to the target triple. Do not put linker overrides in global
  `RUSTFLAGS` — that will break other platform legs.
- **macOS:** Apple's `ld64` / `ld-prime` is already fast and parallel; alternative linkers provide
  marginal benefit.
- **Windows (MSVC):** `lld-link` can reduce link time versus `link.exe`; evaluate per-project.
- **Other ecosystems:** `mold` and `lld` are available as drop-in linker replacements for
  C/C++ builds on Linux.

### Compile-speed-tuned build profile

For CI test and lint builds, you want fast compilation, not fast runtime. Reducing debug
information generation is the highest-leverage setting:

```toml
# In Cargo.toml or a CI-specific profile:
[profile.ci]
inherits = "dev"
debug = "line-tables-only"   # keeps backtraces, cuts object size and link time
codegen-units = 256          # maximum parallelism (default for dev; ensure not overridden)
incremental = false          # ephemeral CI; incremental artifacts waste cache budget
strip = "none"               # stripping costs time and is not needed for CI artifacts
```

The equivalent in other compiled languages: minimize debug information generation for CI builds
while keeping enough for useful test failure messages.

### Build-once / test-many test archiving

Some test runners support building test binaries once and distributing the compiled archive to
multiple runner shards for parallel test execution (without recompiling). This is a test-execution
optimization, not a compilation optimization.

**When to apply:** only when test execution time (not compile time) is the dominant contribution
to a leg's wall-clock. If the bottleneck is compilation, archiving does not help — each shard
still needs the compiled archive, which must be built first on the appropriate target.

In practice, for a large Rust workspace on a 4-vCPU runner, compilation typically dominates for
the first several thousand tests. Revisit archiving when the test suite has grown to the point
where execution time, not compile time, is the measured pole.

---

## Trade-offs

### Platform-specific bugs are caught at merge-queue time, not first-push

A bug that manifests only on macOS or Windows will not be caught until the PR enters the merge
queue (or the nightly run catches it first). This is an intentional trade-off: the cost of
catching those bugs slightly later is outweighed by the benefit of faster per-iteration feedback
for all developers, all the time.

Mitigations:
- Keep the nightly run healthy and monitor it. A platform-specific regression caught overnight is
  still caught before the next merge.
- The `full-ci` label gives any developer an escape hatch: apply it to run the full matrix on
  demand before queuing.
- The merge queue itself gates the default branch — a platform-specific failure at merge time
  blocks the merge, not silently.

### Merge queue introduces a brief additional waiting period before merge

The merge queue builds and validates a synthetic "what if this PR were merged now" branch. This
adds a few minutes between "queue entry" and "merge commit." For most teams this is negligible
compared to the time saved on per-PR iteration.

### The nightly schedule is a commitment, not a background job

If no one monitors the nightly run, platform-specific regressions can accumulate silently. Assign
someone (rotation or bot notification) to triage failing nightly runs within one business day.

### Required-check naming is workflow-level, not file-level

The required check name (`CI / all-checks-pass` in the examples above) includes the workflow
display name. If you rename the workflow file or change the `name:` field at the top of the
workflow, the required check name changes and branch protection must be updated. Document this
linkage explicitly in your repo's CI runbook.

---

## Adapting to your repo

When adopting this playbook in a new repository, parameterize the following:

| Parameter | What to decide |
|-----------|---------------|
| **Fast leg definition** | Which single-platform combination (OS + arch) gives the highest-signal, fastest-feedback build? Typically your most common developer machine (linux-x86_64 or macos-arm64). |
| **Slow leg definition** | Which jobs are too slow, too flaky, or too resource-intensive to run on every push? Start with cross-platform matrix legs and benchmarks. |
| **Merge-queue trigger event name** | GitHub Actions: `merge_group`. GitLab: pipeline rules for merge request pipelines. Other platforms: consult your platform's docs. |
| **Required-check name** | Must exactly match `<workflow display name> / <aggregator job name>` in GitHub. Verify in the Actions → workflow run → job name. |
| **Cache action and "save on failure" option** | `Swatinem/rust-cache` for Rust; `actions/cache` with an explicit post-step for other ecosystems. Confirm the equivalent "save on failure" mechanism. |
| **Cache key strategy** | One distinct key per job class and platform. Audit total cache usage against your platform's budget before adding new cache keys. |
| **Runner labels** | Use your platform's standard runner labels for fast legs. Use the same labels for slow legs unless you have a specific reason to use larger or different runners. |
| **Disk reclamation targets** | Run `df -h` before and after on a sample job to measure what is actually large on your runner image. Only remove what your job does not need. |
| **Label name for opt-in full matrix** | `full-ci` is a readable convention; choose whatever fits your team's labeling conventions. |
| **Nightly cron** | Offset nightly runs by a few minutes from round hours to reduce runner contention on shared infrastructure. |

---

## Appendix A: slideforge as a worked example

> This appendix documents the specific design for the `github.com/drbothen/slideforge` repository.
> It is clearly separated from the generic guidance above. The generic sections are portable
> without this appendix.

### Repository characteristics

- **Language:** Rust, edition 2024, 20-crate Cargo workspace
- **Build flags:** `--all-features`, `RUSTFLAGS=-D warnings`, `CARGO_INCREMENTAL=0`
- **CI platform:** GitHub Actions, all actions pinned by full commit SHA
- **Current (pre-tiered) matrix:** 4 platforms × build + test = 4 legs running on every PR push
- **Observed slow leg:** `test (linux-arm64)` on `ubuntu-24.04-arm` — native arm64 standard
  runner (4 vCPU / 16 GB RAM / 14 GB SSD), cold-build time approximately 22 minutes
- **Note:** The arm64 runner is a native GitHub-hosted runner, not QEMU emulation. The 22-minute
  figure is genuine cold-build time on a 4-vCPU machine compiling a 20-crate workspace
  `--all-features` from a cold or partially-evicted cache.

### Fast legs (run on every PR push)

| Job | Runner | Rationale |
|-----|--------|-----------|
| `fmt` | ubuntu-latest (x86_64) | Format check, seconds |
| `clippy` | ubuntu-latest (x86_64) | Lint, ~3–5 min warm |
| `test (linux-x86_64)` | ubuntu-latest (x86_64) | Primary high-signal test leg |
| `docs` | ubuntu-latest (x86_64) | Doc build + `RUSTDOCFLAGS=-D warnings` |
| `supply-chain` | ubuntu-latest (x86_64) | `cargo audit`, `cargo deny` |
| `snapshots` | ubuntu-latest (x86_64) | `cargo insta test --check` |
| `msrv` | ubuntu-latest (x86_64) | MSRV build check |
| `check-panic-profile` | ubuntu-latest (x86_64) | Enforcement check, fast |
| `check-pdf-deps` | ubuntu-latest (x86_64) | Dependency check, fast |

### Slow legs (gated behind merge_group / schedule / develop push / full-ci label)

| Job | Runner | Rationale for gating |
|-----|--------|----------------------|
| `test (linux-arm64)` | ubuntu-24.04-arm | 22-minute cold-build pole; native arm64 |
| `test (macos-arm64)` | macos-latest | Slower macOS runner; cross-platform value |
| `test (windows-x86_64)` | windows-latest | Slowest leg; Windows-specific behavior |
| `bench` | ubuntu-latest | Long-running; not per-push useful |
| `doctest` | ubuntu-latest | Separate doctest run; secondary |
| `visual-regression` | ubuntu-latest | Requires LibreOffice + ImageMagick install |
| `perf-smoke` | ubuntu-latest | Timing-sensitive; not per-push useful |

### Aggregator

```yaml
all-checks-pass:
  name: all-checks-pass
  runs-on: ubuntu-latest
  if: always()
  needs:
    - fmt
    - clippy
    - test
    - docs
    - supply-chain
    - snapshots
    - msrv
    - check-panic-profile
    - check-pdf-deps
    - bench
    - doctest
    - visual-regression
    - perf-smoke
```

The GitHub required status check is set to `CI / all-checks-pass`. No individual job is a
required check.

### Cache configuration

- **Action:** `Swatinem/rust-cache` with `cache-on-failure: "true"` on all build-heavy jobs
  (test matrix, clippy, docs, snapshots, visual-regression, bench)
- **Shared keys:** one per job class (`test-linux-x86_64`, `test-linux-arm64`,
  `test-macos-arm64`, `test-windows-x86_64`, `clippy`, `docs`, etc.)
- **Budget awareness:** approximately 13 distinct shared keys on a 20-crate workspace risks
  exceeding the ~10 GB GitHub cache budget. Monitor via Actions → Caches. If eviction is
  occurring, consolidate non-matrix lint/doc jobs onto fewer keys.
- **`CARGO_INCREMENTAL=0`:** already enforced at the workflow level; keep it.

### Disk reclamation

Apply the manual reclamation step to: `test (linux-arm64)`, `test (linux-x86_64)`,
`visual-regression`, and `snapshots`. The 14 GB SSD on standard runners minus preinstalled
tooling minus a 20-crate `target/` is tight.

```yaml
- name: Reclaim disk space
  run: |
    sudo rm -rf /usr/share/dotnet /usr/local/lib/android /opt/ghc \
                /opt/hostedtoolcache/CodeQL /usr/local/share/boost
    sudo docker image prune --all --force || true
    df -h
```

### Linker

- **linux-x86_64:** already uses `rust-lld` by default since Rust 1.90; no action needed.
- **linux-arm64:** `rust-lld` is not the default on `aarch64-unknown-linux-gnu`; `mold` provides
  meaningful link-time improvement. Apply via `rui314/setup-mold` action on the arm64 leg, or
  scope via `.cargo/config.toml`:

  ```toml
  [target.aarch64-unknown-linux-gnu]
  linker = "clang"
  rustflags = ["-C", "link-arg=-fuse-ld=mold"]
  ```

  Do not put this in global `RUSTFLAGS` — it will break macOS and Windows legs.

### Build profile

```toml
[profile.ci]
inherits = "dev"
debug = "line-tables-only"
codegen-units = 256
incremental = false
```

slideforge already runs `--profile ci` for nextest; ensure the profile definition matches.

### Expected impact (directional — measure to confirm)

| Change | Expected effect |
|--------|----------------|
| Tiered triggers (slow legs → merge_group) | Per-PR critical path drops to fast-leg time (~3–8 min); arm64 pole removed from per-push path |
| Disk reclamation | Eliminates "No space left" flake class; each flake currently costs a full re-run |
| `cache-on-failure: "true"` on all build-heavy jobs | Breaks cold-build → cancel → cold-build loop; keeps arm64 cache warm |
| `debug = "line-tables-only"` | Smaller `target/` (helps cache budget + disk); modest compile speed improvement |
| mold on arm64 leg | Link-phase speedup on the arm64 cold build; magnitude depends on link/codegen split |

All figures are directional estimates from general Rust-CI research. Measure actual timings and
cache hit rates in your specific environment to confirm. A one-off `cargo build --timings` on
the arm64 runner provides a link/codegen split breakdown.

### Not adopted: sccache

`sccache` was evaluated and rejected for slideforge. With `Swatinem/rust-cache` already caching
`target/`, sccache's per-invocation cache is never consulted when the target directory is warm
(Cargo skips `rustc` invocations entirely). More critically, sccache would compete for the same
~10 GB GitHub cache budget already strained by ~13 distinct `rust-cache` shared keys. The
expected net effect is increased cache contention, not improved hit rates.

### Not adopted: larger runners

GitHub's larger runners (8+ vCPU) are not free for public repositories and require Team/Enterprise
plan access. The standard `ubuntu-24.04-arm` runner (already in use) provides 4 vCPU — the same
as the cheapest larger runner tier. For an open-source project, the higher-value levers are
caching reliability and tiered triggers, both of which are free.

### Research basis

This appendix is grounded in the research sidecar at
`.factory/planning/ci-speed-research.md` (date 2026-06-10, authored by `vsdd-factory:research-agent`).
All version numbers, commit SHAs, and behavioral claims in that document were verified against
live registry and documentation pages at research time. Speedup magnitudes are flagged as
directional in the research sidecar; they have not been measured against slideforge's actual
build timings.
