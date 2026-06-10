# CI-Speed Optimization Research Sidecar

**Project:** slideforge (public repo `github.com/drbothen/slideforge`) — Rust, 20-crate Cargo
workspace, edition 2024, `--all-features`, `RUSTFLAGS=-D warnings`, `CARGO_INCREMENTAL=0`,
GitHub Actions CI, all actions pinned by full commit SHA.

**Date:** 2026-06-10
**Author:** research-agent (vsdd-factory)
**Status:** complete — see "Items NOT confidently verified" at the end for the uncertainty pass.

---

## CRITICAL PRECONDITION CORRECTION (read first)

The task brief describes `test (linux-arm64)` as **"QEMU-emulated arm64."** That premise is
**STALE / INCORRECT** as of the current `ci.yml`. The committed workflow already targets the
**native** GitHub-hosted arm64 runner:

```yaml
- { name: "linux-arm64",   runner: "ubuntu-24.04-arm"   }
```

`ubuntu-24.04-arm` is a **native** arm64 standard runner (4 vCPU / 16 GB RAM / 14 GB SSD for
public repos), GA since 2025-08-07 — **not** a QEMU emulation. The 22m24s baseline is therefore
**native cold-build time**, not emulation overhead. (Source: github.blog changelog 2025-08-07;
docs.github.com github-hosted-runners.) See `ci.yml` lines 118-124 and 133-136.

**Implication:** The single biggest assumed win in the original plan ("replace QEMU with native
arm64") is **already banked.** The remaining 22m pole is genuine cold-build cost on a 4-vCPU
machine doing a 20-crate `--all-features` workspace build from a cold/partial cache. The levers
that matter for it are now: **cache hit rate** (Swatinem/rust-cache), **disk headroom** (avoid the
`No space left` flakes that wipe progress), **faster linking on arm64** (mold — see Q6), and
**build-once/test-many** (nextest archive — see Q4). This reframes the whole plan: Tier 1 is no
longer "swap the runner" but "make the native arm64 runner's cache and disk reliable."

---

# TIER 1 — Highest impact

## Q1. Native arm64 Linux runners — VERIFIED

| Fact | Value | Source |
|------|-------|--------|
| Runner labels | `ubuntu-24.04-arm`, `ubuntu-22.04-arm` (no `ubuntu-latest-arm` alias exists) | docs.github.com github-hosted-runners; actions/partner-runner-images |
| GA for public repos | **2025-08-07** (preview began 2025-01-16) | github.blog/changelog/2025-08-07-…; …/2025-01-16-… |
| Private-repo support | 2026-01-29 (2 vCPU private / 4 vCPU public) | github.blog/changelog/2026-01-29-… |
| Free for PUBLIC repos? | **YES — free and unlimited**, same billing model as x64 standard runners | docs.github.com billing/about-billing-for-github-actions |
| Specs (public) | 4 vCPU, 16 GB RAM, 14 GB SSD, Cobalt-100 CPU | docs.github.com github-hosted-runners; github.blog 2025-01-16 |
| vs `ubuntu-latest` (x64) | x64 standard public runner is 4 vCPU / 16 GB / 14 GB; arm64 matches CPU/RAM | docs.github.com github-hosted-runners |
| `runs-on` syntax | `runs-on: ubuntu-24.04-arm` (already in use) | docs.github.com choosing-the-runner-for-a-job |

**Constraints/risks (verified):**
- Images are **Arm partner images** (managed in `actions/partner-runner-images`), not GitHub's own
  `runner-images`; tooling inventory can differ subtly from x64.
- **No nested virtualization** on arm64 runners (irrelevant to slideforge unless it ever needs
  KVM/Android emulators — it does not).
- Some **community actions ship x64-only binaries** and must be installed manually on arm64.
  slideforge's actions (checkout, dtolnay/rust-toolchain, Swatinem/rust-cache, taiki-e/install-action)
  all support arm64, so this is not currently a blocker.
- **No `ubuntu-latest-arm`** — version label must be pinned explicitly (already done).

**Bottom line for Q1:** Native arm64 is **already adopted.** No "swap from QEMU" win remains. The
only forward action is to keep the label pinned and ensure cache+disk reliability (Q2, Q3) so the
4-vCPU machine isn't repeatedly doing cold builds. **Expected additional speedup from this item
alone: ~0** (already realized). Confidence: HIGH.

---

## Q2. Swatinem/rust-cache — VERIFIED (versions + SHA)

| Fact | Value | Source |
|------|-------|--------|
| Latest release | **v2.9.1** (2026-03-12) | github.com/Swatinem/rust-cache/releases |
| v2.9.1 commit SHA | `c19371144df3bb44fab255c43d04cbc2ab54d1c4` | github.com/Swatinem/rust-cache/commits/v2.9.1 |
| Currently pinned in slideforge | `42dc69e1aa15d09112580998cf2ef0119e2e91ae` | ci.yml line 90 etc. |

**The currently-pinned SHA (`42dc69e…`) is an OLDER release** (predates v2.9.1; the v2.9.0/2.9.1
line added Nix-shell support, toolchain-in-cache-key, and a **hash-calculation regression fix** in
v2.9.1). Recommend bumping to v2.9.1 = `c19371144df3bb44fab255c43d04cbc2ab54d1c4`.
**FLAG:** the exact tag of `42dc69e…` was not resolved in this pass (commit-page fetch 404'd) — confirm
during implementation with `git ls-remote --tags https://github.com/Swatinem/rust-cache`.

**What it caches (verified, established behavior):** `~/.cargo/registry` (index + cache),
`~/.cargo/git`, and the workspace `target/` directory. Cache key is derived from `Cargo.lock`,
toolchain, `rustflags`, the job's env, and `lockfiles`. It also runs `cargo cache`-style cleanup so
only the relevant deps are saved.

**`cache-on-failure` semantics (verified, matches the in-repo comment at ci.yml:145-151):** sets
`CACHE_ON_FAILURE=true` so the rust-cache **post-step still saves the cache even when the job's main
step fails or the job is canceled** (post-if: `success() || env.CACHE_ON_FAILURE == 'true'`). This is
exactly what breaks the cold-build→timeout→cold-build loop. **It is already enabled on the `test` and
`bench` jobs but NOT on `clippy`, `msrv`, `docs`, `snapshots`, `supply-chain`, `doctest`,
`visual-regression`, `perf-smoke`.** Adding it to the heavy compile jobs (clippy especially) is a
cheap reliability win.

**`shared-key` vs `key` vs `prefix-key` (verified best practice):**
- `shared-key` — gives a **fixed, shared cache slot** reused across runs/jobs that set the same value;
  ideal for splitting caches by job-class. slideforge already uses per-job `shared-key`
  (`test-${{ matrix.target.name }}`, `clippy`, `docs`, …). This is correct.
- `key` — an **additive** component appended to the auto key; use to further segment (rarely needed).
- `prefix-key` — overrides the cache-version prefix (default `v0-rust`); bump it to globally bust all
  caches. Leave default unless you need a hard reset.
- **Recommendation:** keep distinct `shared-key`s per job-class (already done). Do **not** collapse
  the matrix legs into one shared-key — each target triple needs its own `target/`.

**Caveats for large multi-crate workspaces + cache-size limits (verified):**
- **GitHub Actions cache is ~10 GB per repository**, LRU-evicted. A 20-crate `--all-features` `target/`
  plus registry across **multiple distinct `shared-key`s (test-linux-x86_64, test-linux-arm64,
  test-macos-arm64, test-windows-x86_64, clippy, docs, msrv, snapshots, supply-chain, doctest, bench,
  visual-regression, perf-smoke)** can easily blow past 10 GB combined. When that happens, **caches
  thrash** (each run evicts another job's cache), and you get cold builds anyway. **This is the most
  likely root cause of the persistent 22m arm64 pole AND the `No space left` flakes.**
- Mitigation: rust-cache only saves the relevant slices, but with ~13 distinct shared-keys the repo's
  10 GB budget is the binding constraint. Consider consolidating non-matrix lint/doc jobs onto fewer
  shared-keys, and/or accept that only the most recently-run job-classes stay warm.

**Bottom line for Q2:** Bump to **v2.9.1 = `c19371144df3bb44fab255c43d04cbc2ab54d1c4`**; add
`cache-on-failure: "true"` to `clippy` (and the other heavy compile jobs); be aware the **10 GB repo
cache budget is shared across all ~13 shared-keys** and is probably the real bottleneck. Expected
impact: turns cold arm64 runs warm when the cache survives → **22m → ~6-10m on warm cache**.
Confidence: HIGH on versions/SHA/semantics; MEDIUM on the magnitude (depends on whether the 10 GB
budget lets the arm64 cache survive between runs).

---

## Q3. Disk-space reclamation — VERIFIED (version + SHA)

| Fact | Value | Source |
|------|-------|--------|
| `jlumbroso/free-disk-space` latest | **v1.3.1** (2026-10-27 per release list) | github.com/jlumbroso/free-disk-space/releases |
| v1.3.1 commit SHA | `54081f138730dfa15788a46383842cd2f914a1be` | github.com/jlumbroso/free-disk-space/commits/v1.3.1 |
| Approx GB reclaimed | up to ~25-30 GB on ubuntu-latest (Android SDK ~9 GB, .NET ~2 GB, Haskell/GHC, Docker images ~4 GB, tool-cache ~6 GB) | repo README; release notes |
| Runtime cost | ~1-3 min depending on which categories are enabled (Android/.NET removals are the slowest) | repo README (qualitative) |
| Default free disk, ubuntu standard | **~14 GB SSD** total on the runner (a chunk already consumed by preinstalled tooling) | docs.github.com github-hosted-runners |
| arm64 / larger runners disk | also **14 GB SSD** for arm64 standard runners | docs.github.com github-hosted-runners |

**FLAG (minor):** WebFetch returned an inconsistent date for the v1.3.1 commit (page said "Oct 18,
2023" while the release list says "Oct 27"). The **SHA `54081f13…` is from the commits/v1.3.1 page**
and should be re-confirmed at implementation with `git ls-remote --tags`.

**Constraints/risks:**
- The action **removes** large preinstalled toolchains (Android SDK, .NET, GHC, Docker images, swap).
  Safe for a pure-Rust workspace, but the **visual-regression job installs LibreOffice + ImageMagick
  + Python** — don't strip categories those depend on, and run free-disk-space **before** those installs.
- Manual alternative (no third-party action, lighter, ~10-15 GB freed, supply-chain-friction-free):
  ```bash
  sudo rm -rf /usr/share/dotnet /usr/local/lib/android /opt/ghc \
              /opt/hostedtoolcache/CodeQL /usr/local/share/boost
  sudo docker image prune --all --force || true
  df -h
  ```
  This avoids adding another pinned third-party action to the supply chain (relevant under
  slideforge's `cargo deny`/SBOM posture) and is often sufficient for Rust builds.

**Bottom line for Q3:** The `No space left` flakes are real and the 14 GB runner SSD minus
preinstalled tooling minus a 20-crate `target/` is genuinely tight. **Recommend the manual `rm -rf`
cleanup step** (no new action dependency) on the build-heavy jobs (test matrix, snapshots,
visual-regression) as the first move; escalate to pinned `jlumbroso/free-disk-space@54081f13…` only if
manual cleanup proves insufficient. Expected impact: **eliminates the disk-flake class** (which today
silently costs full re-runs). Confidence: HIGH on the problem and the fix; MEDIUM on exact GB figures.

---

# TIER 2 — Structural

## Q4. cargo-nextest "build once, test many" — semantics VERIFIED, version VERIFIED

| Fact | Value | Source |
|------|-------|--------|
| Current cargo-nextest | **0.9.137** (2026-05-26) | github.com/nextest-rs/nextest/releases |
| Install in CI | `taiki-e/install-action` (already used) or `cargo-binstall` | nexte.st docs |
| `taiki-e/install-action` latest | **v2.81.9** (2026-06-10) → SHA `fd2f5e3d644b484055ebf4268f474c565f148f25` | github.com/taiki-e/install-action |
| Currently pinned install-action | `d9be7d8cda89035c9c843f78bd44d4f72d8403d4` | ci.yml line 159 (older release) |

**Archive workflow (verified against nexte.st/docs/ci-features/archiving):**
- `cargo nextest archive --archive-file a.tar.zst` builds test binaries + a manifest into one archive.
- `cargo nextest run --archive-file a.tar.zst [--partition …]` runs without recompiling.
- **Target-triple binding (VERIFIED, critical):** the archive contains **compiled binaries for the
  build's target triple.** It can only be executed on a machine that can run that target's binaries.
  You **cannot** build on `x86_64-unknown-linux-gnu` and run on Windows or macOS or arm64-linux. So
  the "build on one runner, test on many" pattern is **per-target**, not cross-target.
- **Source checkout required (VERIFIED):** "The project source must be checked out to the same
  revision on the target machine" — nextest sets CWD relative to workspace root for fixtures/assets.
  slideforge's tests use font fixtures and (Phase 3) `.pptx`/PNG fixtures, so the source tree must be
  present on the test runner.
- Toolchain version need not match for *execution* (binaries are standalone), but **same toolchain +
  same nextest version is strongly recommended** to avoid rebuild drift.

**Partitioning (verified):**
- `--partition count:K/N` — splits the stable-ordered test list into N index buckets; balances by
  **test count**. Order-stability across nextest versions is **not formally guaranteed → pin nextest**.
- `--partition hash:K/N` — hashes each test id mod N; balances better when **runtimes vary**; robust to
  test additions. Hash stability also **not guaranteed across versions → pin nextest**.

**Applicability to slideforge (analysis):**
- The win is **WITHIN a single target** — e.g. build the linux-x86_64 archive once, fan it out to
  multiple x86_64 runners with `--partition`. But slideforge's matrix has **one runner per target**,
  and the bottleneck (arm64) is a **single 4-vCPU machine doing a cold compile.** Partitioning across
  multiple arm64 runners would parallelize *test execution* but **each partition runner still needs the
  compiled archive** — which must itself be built on arm64 (target-bound). So you'd add an arm64 build
  job + N arm64 run jobs. Net win only if **test execution time** (not compile time) dominates. Today
  the pole is **compile**, so nextest archiving gives **limited benefit until the suite grows large.**
- **More valuable near-term:** nextest archiving lets you **decouple `clippy`/`docs`/`test` from each
  recompiling the workspace.** But those run different cargo profiles, so artifact sharing is limited.

**Bottom line for Q4:** Bump nextest awareness to 0.9.137; bump `taiki-e/install-action` to v2.81.9 =
`fd2f5e3d644b484055ebf4268f474c565f148f25`. **Defer the archive/partition pattern** — it's a
test-execution optimization and slideforge's bottleneck is compile-time on a single arm64 box.
Revisit when the test suite is large enough that *execution* (not compilation) dominates a leg.
Expected near-term impact: **low**; latent impact when suite grows: medium. Confidence: HIGH on
semantics/versions; HIGH on the "defer" recommendation given the current bottleneck shape.

## Q5. sccache + GHA cache backend — semantics VERIFIED, version VERIFIED

| Fact | Value | Source |
|------|-------|--------|
| `mozilla/sccache-action` latest | **v0.0.10** (2026-04-22) | github.com/mozilla/sccache-action/releases |
| v0.0.10 commit SHA | `9e7fa8a12102821edf02ca5dbea1acd0f89a2696` | github.com/mozilla/sccache-action/commits/v0.0.10 |
| Setup | `SCCACHE_GHA_ENABLED=true` + `RUSTC_WRAPPER=sccache` (the action sets these) | mozilla/sccache docs |
| GHA cache limit | **~10 GB / repo**, LRU-evicted (shared with rust-cache!) | docs.github.com cache docs |

**Coexistence with Swatinem/rust-cache (verified, important):**
- They operate at **different layers**: rust-cache caches `target/` + registry directories;
  sccache caches **individual `rustc` invocations** via `RUSTC_WRAPPER`.
- They **can coexist**, BUT: if rust-cache restores a warm `target/`, Cargo **skips `rustc` entirely**
  for unchanged crates → **sccache is never consulted** and adds no value. sccache only helps when
  `target/` is cold/partial.
- **They share the same ~10 GB repo cache budget.** Running both **doubles cache pressure** and, given
  slideforge already has ~13 rust-cache shared-keys near/over budget (Q2), **adding sccache would make
  the thrashing WORSE, not better.**
- `CARGO_INCREMENTAL=0` (already set) is the **correct** setting for sccache: incremental mode produces
  many small artifacts that defeat per-invocation caching.

**Bottom line for Q5:** **Do NOT add sccache.** With rust-cache already present and the 10 GB repo
cache budget already the binding constraint (Q2), sccache would compete for the same budget and
rarely fire (warm `target/` skips `rustc`). It's the wrong tool when `target/`-caching already covers
the workspace and cache *capacity* is the limiter. Confidence: HIGH. (Re-evaluate only if you ever
move off rust-cache entirely or to an external sccache backend like S3 that doesn't share the GHA
budget.)

## Q6. Faster linker (mold / lld) — VERIFIED with an important Rust-default caveat

| Fact | Value | Source |
|------|-------|--------|
| `rui314/setup-mold` latest tag | **v1** → SHA `9c9c13bf4c3f1adef0cc596abc155580bcb04444` (default mold 2.41.0) | github.com/rui314/setup-mold |
| mold link speedup (Linux) | ~3-10× vs GNU ld/gold; **~2-4× vs lld** on link-bound builds; often only a few hundred ms on Rust where linking isn't the bottleneck | github.com/rui314/mold (README benchmarks); fasterthanli.me |
| **Rust 1.90 default** | **`rust-lld` is now the DEFAULT linker on `x86_64-unknown-linux-gnu`** (stable, 2025-09-01): ~7× link-time / ~40% total compile reduction on incremental, ~20% on from-scratch | blog.rust-lang.org/2025/09/01/rust-lld-on-1.90.0-stable |
| macOS | Apple `ld64`/ld-prime already fast + parallel; mold/sold gains **marginal** | mold README; perplexity synthesis |
| Windows MSVC | `lld-link` is a real but project-dependent speedup over `link.exe` | LLVM docs; perplexity synthesis |

**CRITICAL CAVEAT (verified):** As of **Rust 1.90 (Sept 2025)**, `rust-lld` is the **default linker on
`x86_64-unknown-linux-gnu`.** slideforge pins stable toolchain (≥ 1.88 MSRV but CI uses `stable`,
which is now well past 1.90). So the **linux-x86_64 leg is ALREADY using lld by default.** Adding mold
to x86_64 gives only the incremental **lld→mold** delta (~2-4× on link, but link is a minor fraction of
Rust compile time → often a few hundred ms). **Low marginal value on x86_64.**

**Where mold STILL helps:** the **`aarch64-unknown-linux-gnu`** target (the 22m pole) is **NOT** covered
by the Rust 1.90 lld-default change (that default is x86_64-linux only). So the arm64 leg is likely
**still using GNU ld**, where mold's 3-10× link speedup is fully available. **mold on the arm64 runner
is the targeted, high-leverage application.**

**Config (Linux only):**
```yaml
# arm64 (and optionally x86_64) Linux legs only:
- uses: rui314/setup-mold@9c9c13bf4c3f1adef0cc596abc155580bcb04444
```
or via `.cargo/config.toml` (committed, target-scoped so macOS/Windows are untouched):
```toml
[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```
**Risk:** `-fuse-ld=mold` requires `mold` on PATH and a driver (gcc/clang) that supports `-fuse-ld`.
The `setup-mold` action handles install. Do **not** put mold in global `RUSTFLAGS` (the env var is
workspace-wide and would break the macOS/Windows legs); scope it per-target in `.cargo/config.toml`
or set `RUSTFLAGS` only in the Linux job steps.

**Bottom line for Q6:** Apply **mold to the linux-arm64 leg only** (where GNU ld is still default and
mold's 3-10× link win is real). Skip it on x86_64 (already lld since Rust 1.90 → marginal). Skip
macOS/Windows (Apple ld64 fast; lld-link marginal & adds complexity). Expected impact on the arm64
pole: **link-phase only** — meaningful if linking is a non-trivial share of the 22m, modest if compile
codegen dominates (likely the latter for a cold full build). **Estimate: a few percent to ~10% off the
arm64 leg, larger on warm-cache incremental rebuilds.** Confidence: HIGH on the facts; MEDIUM on the
magnitude for slideforge (no link-phase profiling data available — see uncertainty pass).

---

# TIER 3 — Tuning / policy

## Q7. Cargo CI build-profile tuning for COMPILE speed — VERIFIED principles

Recommended `[profile.ci]`-style or test/bench profile settings to cut **compile** time (slideforge
already runs `--profile ci` for nextest):
- `debug = "line-tables-only"` (or `debug = 0`) — full debuginfo is expensive to generate and link;
  line-tables keep backtraces useful while cutting object size and link time substantially. **High
  leverage, low risk.**
- `codegen-units = 256` (default for dev is 256; ensure CI test/ci profile isn't forced to 16/1) —
  more codegen units = more parallelism = faster compile (at the cost of runtime perf, irrelevant for
  test/lint). Do **not** lower codegen-units in test/ci profiles.
- `incremental = false` — already enforced via `CARGO_INCREMENTAL=0` (see below).
- `strip = "none"` for test/ci (stripping costs time and isn't needed for CI artifacts).
- `[profile.bench]` — for the perf gate, keep `lto = false` and `codegen-units` high *for the build*,
  but the **bench measurement** itself must use the real release-like profile to be meaningful; keep
  benches in their own profile so cold-build doesn't pay LTO cost on every CI run unless measuring.

**`CARGO_INCREMENTAL=0` — confirmed STILL correct for CI (verified):** Incremental compilation stores
per-crate incremental state that (a) is rarely reused across ephemeral CI runs, (b) **bloats `target/`**
(worsening the 10 GB cache budget and the disk flakes), and (c) defeats per-invocation caching. With
rust-cache caching whole `target/`, incremental adds size without benefit. **Keep `CARGO_INCREMENTAL=0`.**
The in-repo comment at ci.yml:21-22 is correct.

**Bottom line for Q7:** Add `debug = "line-tables-only"` (or `0`) to the CI/test profile; keep
codegen-units high; keep `CARGO_INCREMENTAL=0`. Expected impact: **smaller `target/` (helps cache +
disk) and modestly faster link/codegen.** Confidence: HIGH on direction; MEDIUM on magnitude.

## Q8. Tiered CI + merge queue + required checks (avoid the paths-ignore deadlock) — VERIFIED

**The deadlock (verified):** A required status check whose **entire workflow** is skipped by
`paths`/`paths-ignore` is **never reported** → branch protection waits forever → merge blocked.
(docs.github.com workflow-syntax; troubleshooting-required-status-checks; community discussions.)

**GitHub-sanctioned solutions (verified semantics):**
1. **Never use workflow-level `paths`/`paths-ignore` on workflows that produce required checks.** Use
   **job-level `if:`** instead so the workflow always starts and the required job name always reports.
   slideforge's `all-checks-pass` aggregator (ci.yml:447-489) already implements the correct pattern:
   a single synthetic job (`needs:` all checks, `if: always()`, treats `skipped` as success) is the
   ideal **single required status** for branch protection. **Make `CI / all-checks-pass` the one
   required check** — this is exactly the aggregator/"gatherer" pattern GitHub's docs and community
   guidance endorse.
2. **Merge queue:** add `merge_group:` to the `on:` triggers of the workflow that produces required
   checks. The queue validates on a **synthetic branch** and requires the same named checks; without
   `merge_group` the required check never reports on the queue branch → queue stalls.
3. **Tiered triggers without path-filter deadlock:** keep fast checks (`fmt`, `clippy`,
   `check-panic-profile`, `check-pdf-deps`) on every `push`/`pull_request`; gate the **full
   multi-platform matrix** behind a job-level `if:`:
   ```yaml
   on:
     pull_request: { branches: [develop, main] }
     push:         { branches: [develop, main] }
     merge_group:
     schedule:     [{ cron: "0 6 * * *" }]   # nightly full matrix
   ...
   full_matrix:
     if: >
       github.event_name == 'merge_group' ||
       github.event_name == 'schedule' ||
       (github.event_name == 'push' && github.ref == 'refs/heads/develop') ||
       (github.event_name == 'pull_request' &&
        contains(github.event.pull_request.labels.*.name, 'full-ci'))
   ```
   Then make the **aggregator** (`all-checks-pass`, which `needs:` both quick and full jobs and treats
   `skipped` as success) the single required check, so PRs without `full-ci` still report green.

**slideforge-specific note:** the current matrix runs the **full 4-platform set on every PR**. Moving
windows + macos + arm64 behind a `merge_group`/`develop`/label gate (keeping linux-x86_64 fast on
every PR) would **remove the 22m arm64 pole from the per-push critical path** entirely — arm64 would
only run at merge-queue time. **This is arguably the single largest wall-clock win available**, larger
than any caching tweak, because it changes *when* the slow leg runs rather than *how fast* it runs.

**Bottom line for Q8:** (a) Keep/declare `CI / all-checks-pass` as the **single required status**;
(b) add `merge_group:` to `on:`; (c) gate windows/macos/arm64 behind a job-level `if:`
(merge_group/develop/nightly/`full-ci` label) so per-PR CI runs only linux-x86_64 fast checks.
Expected impact: **removes the 22m arm64 leg from the per-PR critical path → per-PR feedback drops to
the x86_64 + lint time (~6-8m).** Confidence: HIGH (this is a workflow-restructuring, not a perf gamble).

## Q9. Larger GitHub-hosted runners — VERIFIED

| Fact | Value | Source |
|------|-------|--------|
| Larger runners (4/8/16/…-core) | Team/Enterprise-Cloud plans only; configured at org level | docs.github.com about-larger-runners |
| Cost for PUBLIC repos | **NOT free — always billed per-minute, even on public repos** (e.g. Linux 4-core $0.016/min, 8-core $0.032/min, 16-core $0.064/min) | docs.github.com actions-runner-pricing; about-billing |
| Free tier | Standard runners only (incl. the free 4-vCPU `ubuntu-latest` and 4-vCPU `ubuntu-24.04-arm` for public) | docs.github.com about-billing |

**Bottom line for Q9:** For a **public repo**, larger runners are **never free** and are gated behind
Team/Enterprise plans. The free `ubuntu-24.04-arm` standard runner (already in use) is **already 4
vCPU** — the same core count as the cheapest larger runner. **Not worth it** unless slideforge adopts a
paid plan AND profiling shows the build is CPU-parallelism-bound at >4 cores (an 8/16-core larger
runner could roughly halve/quarter a CPU-bound cold build, but at per-minute cost on every run). For
an OSS project optimizing wall-clock-for-free, the better levers are caching reliability (Q2/Q3) and
tiered triggers (Q8). Confidence: HIGH.

---

# Per-Tier bottom line (what to do + expected gain + confidence)

| Tier | Action | Expected wall-clock impact | Confidence |
|------|--------|----------------------------|------------|
| **Pre** | (Correction) Native arm64 already in use — no QEMU swap remains | 0 (already banked) | HIGH |
| **T1-Q2** | Bump rust-cache → v2.9.1 (`c19371144df3bb44fab255c43d04cbc2ab54d1c4`); add `cache-on-failure` to clippy + heavy compile jobs; recognize 10 GB repo-cache budget is the real limiter across ~13 shared-keys | Warm arm64 22m→~6-10m **when cache survives**; main risk is cache eviction | HIGH (facts) / MED (magnitude) |
| **T1-Q3** | Add disk-cleanup (manual `rm -rf` preferred; else `jlumbroso/free-disk-space@54081f13…`) to build-heavy jobs | Eliminates `No space left` flakes (each costs a full re-run) | HIGH (fix) / MED (GB) |
| **T2-Q4** | Bump nextest awareness 0.9.137 + install-action → v2.81.9 (`fd2f5e3d…`); **DEFER** archive/partition (test-exec optimization; bottleneck is compile) | Low now; medium when suite grows | HIGH |
| **T2-Q5** | **Do NOT add sccache** — shares the same 10 GB budget as rust-cache and rarely fires with warm `target/` | Negative if added (worse thrashing) | HIGH |
| **T2-Q6** | Apply **mold to linux-arm64 leg only** (`setup-mold@9c9c13bf…`); skip x86_64 (already lld since Rust 1.90), macOS, Windows | ~few %–10% off arm64 leg (link phase only) | HIGH (facts) / MED (magnitude) |
| **T3-Q7** | `debug = "line-tables-only"` in CI/test profile; keep codegen-units high; **keep `CARGO_INCREMENTAL=0`** | Smaller `target/` (helps cache+disk) + modest compile win | HIGH (dir) / MED (mag) |
| **T3-Q8** | Make `CI / all-checks-pass` the single required check; add `merge_group:`; **gate windows/macos/arm64 behind merge_group/develop/nightly/`full-ci` label** | **Largest win: removes 22m arm64 from per-PR critical path → per-PR ~6-8m** | HIGH |
| **T3-Q9** | Skip larger runners (not free on public repos; standard arm64 already 4 vCPU) | n/a | HIGH |

**Recommended implementation order (highest leverage first):**
1. **Q8 tiered triggers** — biggest per-PR wall-clock reduction, no perf gamble (restructuring).
2. **Q3 disk cleanup** — stops the flake class that silently doubles runtimes.
3. **Q2 rust-cache bump + cache-on-failure everywhere** — keeps the warm path warm.
4. **Q6 mold on arm64** + **Q7 line-tables-only** — shrink the arm64 cold build itself.
5. Action SHA bumps (rust-cache, install-action) folded into the above.

---

# Items NOT confidently verified (for the uncertainty pass)

1. **Exact tag of the currently-pinned rust-cache SHA `42dc69e1aa15d09112580998cf2ef0119e2e91ae`.**
   The commit page fetch 404'd. It is an older release than v2.9.1. **Confirm with
   `git ls-remote --tags https://github.com/Swatinem/rust-cache` before bumping.**
2. **`jlumbroso/free-disk-space` v1.3.1 commit date discrepancy.** Release list says 2026-10-27; the
   commit page rendered "Oct 18, 2023." SHA `54081f138730dfa15788a46383842cd2f914a1be` is from the
   commits/v1.3.1 page but should be **re-confirmed via `git ls-remote --tags`** at implementation.
   (Also note: the v1.3.1 release date of "Oct 27" is in the future relative to today 2026-06-10 — the
   release-list rendering may have a year/date artifact; **verify the tag→SHA mapping directly**.)
3. **`setup-mold` versioning.** The repo has **no GitHub Releases** — only tags `v1` and `staging`,
   both at `9c9c13bf4c3f1adef0cc596abc155580bcb04444`. `v1` is a **moving major tag**; pinning the SHA
   is correct, but **confirm the SHA still resolves and pins mold ≥ 2.41.0** at implementation.
4. **Magnitude of every speedup is UNVERIFIED for slideforge specifically.** No link-phase profiling,
   no warm-vs-cold timing breakdown, and no per-crate compile profile was available. All "expected
   wall-clock" figures are **directional estimates** from general Rust-CI literature, not slideforge
   measurements. The arm64 mold win in particular depends on the link/codegen split of the cold build,
   which was not measured. **Recommend a one-off `cargo build --timings` + `-Clink-self-contained` /
   linker-timing capture on the arm64 runner** to size Q6/Q7 before committing to them.
5. **Whether the 10 GB repo cache budget is actually being exceeded.** This is inferred from ~13
   distinct shared-keys × a 20-crate `--all-features` `target/`, and is the leading hypothesis for the
   persistent arm64 cold builds — but it was **not confirmed against the repo's actual cache usage**
   (Actions → Caches UI / `gh cache list`). **Verify actual cache sizes/eviction before assuming
   thrashing is the cause.**
6. **cargo-nextest archive cross-glibc safety on the arm64 partner image.** If Q4 is ever adopted,
   confirm the Arm partner image's glibc baseline matches between build and run jobs (build on
   oldest-glibc runner). Not relevant unless archiving is adopted (currently deferred).
7. **sccache version vs the action.** `mozilla/sccache-action@v0.0.10` (`9e7fa8a1…`) installs a
   bundled sccache binary version that was not separately pinned/verified here. Moot under the
   "do not add sccache" recommendation, but note if that decision is revisited.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 5 | arm64 runners (verified w/ citations); rust-cache+free-disk-space (training-data fallback — re-verified via WebFetch); nextest+sccache (training-data fallback — re-verified); mold+profiles (training-data fallback — re-verified); merge-queue/required-checks/larger-runners (verified w/ citations) |
| Perplexity perplexity_ask | 1 | mold vs lld vs GNU ld link-time magnitudes + macOS/Windows linker situation (cited) |
| WebFetch | 11 | release pages + tag→SHA resolution for rust-cache, free-disk-space, setup-mold, sccache-action, install-action, nextest; nexte.st archiving docs; rust-cache pinned-SHA tag (404) |
| Read | 4 | existing ci.yml; 3 large persisted Perplexity research outputs |
| Training data | ~2 areas | Generic Rust-CI profile-tuning directions (Q7) and the magnitude estimates in the bottom-line table — flagged explicitly as directional, not measured |

**Total MCP tool calls:** 6 Perplexity + 11 WebFetch = 17 (plus 4 Read for local/persisted files).
**Training data reliance:** LOW-MEDIUM. All version numbers and commit SHAs were verified against live
registry/release/tag pages (NOT training data). The two Perplexity research calls that returned
"no live results" (rust-cache/free-disk-space, nextest/sccache, mold/profiles) were treated as
SUSPECT and every concrete version/SHA from them was independently re-verified via WebFetch; one
hallucinated SHA in the mold output was discarded. Semantic/behavioral claims (cache semantics,
archive target-binding, merge-queue deadlock, linker speedup ranges) are sourced to official docs and
cited repos. Speedup *magnitudes* for slideforge specifically are directional (see uncertainty item 4).
