---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-093
title: "CI: arm64 build-time reduction (mold linker + CI profile tuning)"
epic: EPIC-19
wave: 5
points: 5
priority: NEXT
tdd_mode: facade
status: draft
spec_version: "1.1"
behavioral_contracts: []
# BC status: pending PO authorship — no product BC governs CI build toolchain tuning.
# Anchors to NFR-029 (Linux arm64 matrix reliability) and NFR-001 (cold build < 500ms
# wall-clock gate on Linux x86_64 — profile tuning must not regress this benchmark).
# See STORY-051 precedent for NFR-only CI stories.
verification_properties: []
nfr_refs: [NFR-029, NFR-001]
target_module: .github/workflows/ci.yml
subsystems: []
depends_on: [STORY-091, STORY-092]
blocks: []
estimated_days: 2
# Human priority override 2026-06-10: deliver BEFORE remaining Wave-5 feature stories.
# STORY-093 depends on STORY-092 (cache must be reliable before profiling is meaningful;
# baseline timings captured on warm-cache runs reflect actual build performance, not
# cold-cache noise).
---

# STORY-093: CI — arm64 Build-Time Reduction (mold + CI Profile Tuning)

## Subsystem Anchor Justification

EPIC-19 (CI/CD Infrastructure, cross-cutting). Same rationale as STORY-091 and STORY-092:
changes are to `.github/workflows/ci.yml`, `.cargo/config.toml`, and `Cargo.toml`
(profile section). No source crate logic changes. EPIC-19's mandate covers the full CI
quality bar; arm64 build-time directly affects NFR-029 (Linux arm64 matrix reliability)
and the project's engineering velocity.

## Dependency Anchor Justifications

- `depends_on: [STORY-091]` — tiered trigger must be live so arm64 only runs in
  merge_group/develop/nightly. Profiling (`--timings`) on the arm64 leg is only
  meaningful once the run cadence stabilizes (not every-PR thrash). STORY-091 provides
  this stable context.
- `depends_on: [STORY-092]` — STORY-092 fixes cache reliability. All timing measurements
  in this story MUST be taken on a warm-cache run. Taking them on a cold-cache run would
  produce inflated baselines that misattribute cache-miss time as linker time. The
  `cargo build --timings` capture (Task A) MUST happen after STORY-092 lands.
- `blocks: []` — no other story depends on arm64 build-time reduction. This story
  improves engineering velocity but does not gate any feature delivery.

## Summary

After STORY-091 moves the arm64 leg off the per-PR critical path and STORY-092 makes the
arm64 cache reliable, the arm64 cold-build wall-clock (22 minutes on a warm cache: ~6-10
minutes) is no longer the primary bottleneck. However, it still matters for:

1. **Merge-queue gate time** — every PR that enters the merge queue triggers the full arm64
   leg. Reducing it from ~22m cold / ~6-10m warm to less makes the merge queue faster.
2. **Nightly run duration** — the nightly full-matrix run's total wall-clock.
3. **NFR-029 reliability** — a faster arm64 build is less likely to hit the 6-hour GHA
   job timeout on a very large workspace change.

This story applies two targeted improvements:

### A: mold linker on arm64 (linux-arm64 leg ONLY)

**Verify-at-implementation (GATING PRE-CHECK):** This story's primary lever assumes
`aarch64-unknown-linux-gnu` still uses GNU ld as the default linker. As of Rust 1.90
(Sept 2025), `rust-lld` became the DEFAULT on `x86_64-unknown-linux-gnu`. The CI
workflow uses `stable` (not a pinned Rust version), so if `rust-lld` subsequently
became the default for `aarch64-unknown-linux-gnu` in a Rust stable release after 1.90,
mold's benefit collapses significantly (lld→mold delta is much smaller than GNU-ld→mold).
The implementer MUST check the current default linker for `aarch64-unknown-linux-gnu`
on the CURRENT stable Rust channel before committing the mold configuration:
```bash
# Run on the arm64 CI runner:
rustc +stable --print cfg --target aarch64-unknown-linux-gnu | grep linker
# or check: rustc -vV + release notes for the current stable
```
If `rust-lld` is already the default for `aarch64-unknown-linux-gnu`, recalibrate the
expected speedup downward; the story still lands (correctness/direction confirmed) but
document the revised expectation in the PR description.

Configuration: `rui314/setup-mold` action on the arm64 CI leg + `.cargo/config.toml`
target-scoped linker override for `aarch64-unknown-linux-gnu`. Do NOT apply to x86_64
(already lld; marginal gain), macOS (Apple ld64 is fast; mold gain is marginal), or
Windows (lld-link is a real win but adds complexity and targets Windows).

**Research caveat:** The speedup magnitude is UNVERIFIED for slideforge specifically. It
depends on the link/codegen split of the cold build. The `cargo build --timings` capture
(Task A) MUST be done first to size the link-phase contribution before committing to mold.
If link phase is < 10% of total build time (codegen-dominated), mold's benefit is small
and the story should still land (correctness/direction confirmed) but expectations must
be calibrated accordingly.

### B: CI build-profile tuning (CORRECTNESS NOTE — read before implementing)

**Critical correction (verified 2026-06-10):** The `--profile ci` flag in
`ci.yml:168` is a **nextest execution profile** (defined in `.config/nextest.toml:32`),
NOT a Cargo build profile. These are entirely different mechanisms:

- **Nextest execution profile** (`--profile ci` in the `cargo nextest run` command):
  controls nextest's test execution behavior (retries, output capture, timeouts).
  Defined in `.config/nextest.toml`. Already present in the repo.

- **Cargo build profile** (`[profile.ci]` in `Cargo.toml` + `--cargo-profile ci` in
  the `cargo nextest run` command): controls how Rust compiles test binaries (debug level,
  codegen-units, etc.). Currently ABSENT from `Cargo.toml`. Adding `[profile.ci]` to
  `Cargo.toml` has NO effect unless nextest is ALSO invoked with `--cargo-profile ci`
  (a separate flag the workflow does NOT currently pass).

To apply a compile-speed-tuned Cargo build profile, this story must do ALL THREE:

1. Add `[profile.ci]` to `Cargo.toml` (inheriting `test`, with `debug = "line-tables-only"`).
2. Add `--cargo-profile ci` to the nextest invocation in `ci.yml` (for the test jobs).
3. Keep `--profile ci` (nextest execution profile) unchanged.

Removing the false assumption that a `[profile.ci]` already exists or that `--profile ci`
is the same flag.

**Important cache/disk interaction:** Adding `--cargo-profile ci` causes Cargo to build
into `target/ci/` instead of `target/debug/` or `target/test/`. This is a NEW output
directory that does not have a cached warm state initially. The first run after this
change will be a cold build for the `target/ci/` directory; subsequent runs will be warm.
Factor this into the cache budget (STORY-092 AC-004): the `target/ci/` directory size
will be added to the cache until the old `target/test/` or `target/debug/` entries age out.
This is a temporary increase (one LRU cycle); document it in the PR description.

`Cargo.toml` CI/test profile adjustments to shrink `target/` and reduce compile time:

- `debug = "line-tables-only"` — preserves useful backtraces while cutting debug-info
  size substantially. This reduces `target/` size (helps both the disk flake from STORY-092
  and the cache budget), and reduces link time (less debug section to process).
- Keep `codegen-units` high (do NOT set to 1 or 16 in test/ci profile) — more parallelism
  → faster compile. The dev profile default is 256; confirm the CI profile does not override
  this downward.
- Keep `CARGO_INCREMENTAL=0` — confirmed correct for CI (research Q7). Do not remove.
- Keep `strip = "none"` for test/ci — stripping costs time and is not needed for CI.

## Behavioral Contracts

No product BCs govern build toolchain. NFR anchoring:

| NFR | Title | Covered ACs |
|-----|-------|-------------|
| NFR-029 | Linux arm64 binary builds and passes tests | AC-001, AC-002, AC-003, AC-005 |
| NFR-001 | Cold build < 500ms for 25-slide deck on Linux x86_64 | AC-004 (profile tuning MUST NOT regress x86_64 benchmark) |

## Acceptance Criteria

### AC-001: `cargo build --timings` baseline captured for arm64 BEFORE mold (traces to NFR-029)

Before committing the mold configuration, the implementer MUST run `cargo build --timings`
on the arm64 CI runner (or reproduce locally via `cross`/`cargo` with a matching environment)
to capture the link-phase wall-clock as a percentage of total build time. The result MUST
be recorded in the PR description as "arm64 timings baseline: total=Xs, link=Ys (Z%)."
If link phase is < 5% of total time, the mold benefit is small; proceed with the story
but document the calibrated expectation in the PR description.

The `--timings` artifact MUST be uploaded as a GitHub Actions artifact from the arm64
test leg for one run, to make the before state inspectable. This is a ONE-OFF profiling
task; the artifact upload is removed after the baseline is recorded.

### AC-002: mold installed and active on linux-arm64 leg ONLY (traces to NFR-029)

The `rui314/setup-mold@9c9c13bf4c3f1adef0cc596abc155580bcb04444` step MUST appear in the
`test (linux-arm64)` job ONLY (not in `test (linux-x86_64)`, `test (macos-arm64)`,
`test (macos-x86_64)`, or `test (windows-x86_64)`).

**RUSTFLAGS interaction (correctness requirement):** CI sets a global `RUSTFLAGS=-D warnings`
environment variable. A global `RUSTFLAGS` env var OVERRIDES (does NOT merge with)
`[target.<triple>].rustflags` entries in `.cargo/config.toml`. This means:
- If mold linker args are added via `[target.aarch64-unknown-linux-gnu].rustflags` in
  `.cargo/config.toml`, they will be CLOBBERED by the global `RUSTFLAGS=-D warnings`.
- Conversely, if mold args are added to the global `RUSTFLAGS` env, `-D warnings` will
  be dropped.

The correct resolution is to configure the arm64 CI job's RUSTFLAGS to include BOTH
`-D warnings` AND the mold linker arguments, using one of these approaches:
- Set `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"` in
  the arm64 job's env block (target-specific env var takes precedence over global
  RUSTFLAGS for that target; does NOT affect other targets).
- OR inject the arm64 linker arg in the `.cargo/config.toml` `[target.aarch64-unknown-linux-gnu]`
  section AND pass RUSTFLAGS as `"-D warnings"` explicitly for that job step (ensuring
  BOTH flags are active).

The implementer MUST choose the approach that preserves both `-D warnings` and the mold
linker flag. Do NOT use `RUSTFLAGS` as a workspace-wide env to carry the mold linker
arg — it would break macOS/Windows legs.

All test suites and clippy checks on the arm64 leg MUST still pass after mold is activated.

**Verify-at-implementation:** Confirm `clang` (needed for `-fuse-ld=mold` driver support)
is available on the live `ubuntu-24.04-arm` partner image:
```bash
which clang && clang --version
```
If `clang` is absent, use `gcc` as the fallback driver (see EC-002). The standard
`ubuntu-24.04-arm` image is expected to include clang, but this must be verified at
implementation time.

### AC-003: arm64 link phase faster after mold (traces to NFR-029)

A `cargo build --timings` capture AFTER mold is activated MUST show a reduction in the
link-phase duration compared to the baseline from AC-001. The absolute magnitude is
expected to be proportional to the link/codegen split measured in AC-001 and conditioned
on the result of the aarch64 default-linker pre-check. Document the after state in the PR
description: "arm64 timings after mold: total=Xs, link=Ys (Z%); link reduction vs
baseline: W%." The story is DONE even if the reduction is modest (< 10%) because direction
is confirmed and the change is low-risk.

### AC-004: `[profile.ci]` in Cargo.toml + `--cargo-profile ci` in ci.yml; x86_64 NFR-001 benchmark unaffected (traces to NFR-001, NFR-029)

**Correctness requirements (verified 2026-06-10):**

`Cargo.toml` MUST have a `[profile.ci]` with at minimum:
```toml
[profile.ci]
inherits = "test"
debug = "line-tables-only"
# codegen-units: leave at default (256) for maximum parallelism
# incremental: false (enforced via CARGO_INCREMENTAL=0 env; do not set here)
# strip: "none" (do not strip test binaries)
```

The `cargo nextest run` command in `ci.yml` for the test jobs MUST be invoked with
`--cargo-profile ci` (in addition to the existing `--profile ci` execution profile flag):
```yaml
run: cargo nextest run --profile ci --cargo-profile ci ...
```

These are TWO DISTINCT FLAGS:
- `--profile ci` — nextest execution profile (already in ci.yml; controls retry/timeout behavior; defined in `.config/nextest.toml`)
- `--cargo-profile ci` — Cargo build profile (NEW; controls how Rust compiles test binaries; defined in `[profile.ci]` in `Cargo.toml`)

Both flags MUST be present for the build profile to take effect. The prior AC wording
that said "confirm this is already the case — ci.yml runs --profile ci" was INCORRECT.
`--profile ci` does NOT invoke a Cargo build profile; it is a nextest-only flag. The
`--cargo-profile ci` flag is new and must be explicitly added.

**Target directory note:** Adding `--cargo-profile ci` causes Cargo to use `target/ci/`
as the build output directory (not `target/debug/`). The first run will be a cold build
for `target/ci/`; subsequent runs warm. Factor into cache key configuration (ensure the
rust-cache `shared-key` for test jobs is updated to capture `target/ci/` if needed).

The existing NFR-001 criterion benchmark MUST still pass (< 500ms cold build for 25-slide
deck on linux-x86_64). The profile tuning applies to the test build profile; production
build profile (`release`) is NOT modified.

### AC-005: `target/` size reduced on arm64; no new `No space left` failures (traces to NFR-029)

The `debug = "line-tables-only"` change MUST produce a measurable reduction in the arm64
`target/ci/` directory size compared to the prior `target/debug/` or `target/test/`
size (capture `du -sh target/` before and after in CI logs). No new `No space left on
device` failures are acceptable after this story (STORY-092 addresses the primary fix;
this story provides defense-in-depth via smaller `target/`). Document before/after
`target/` size in the PR description.

Note: compare `du -sh target/ci/` against the old `target/debug/` or `target/test/`
size (whichever the prior nextest invocation used) — they are different directories.

### AC-006: All test suites pass on all 5 CI platforms after profile changes (traces to NFR-026 through NFR-030)

After both mold and `debug = "line-tables-only"` changes land, the full CI matrix
(triggered via develop push or `full-ci` label per STORY-091) MUST show green on all 5
platforms. `line-tables-only` debug info MUST produce usable backtraces on test failures
(verify by inspecting a test failure output if one occurs, or by intentionally triggering
a panic in a test and confirming the backtrace contains file:line information).

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| CI workflow (modified) | `.github/workflows/ci.yml` | Effectful (GitHub Actions DSL) |
| Cargo config (modified) | `.cargo/config.toml` | Build config (target-scoped linker — see RUSTFLAGS interaction note in AC-002) |
| Cargo workspace manifest (modified) | `Cargo.toml` | Build config (profile section) |
| Profiling artifact (one-off) | `target/cargo-timings/*.html` | CI artifact (removed after baseline captured) |

Architecture section files: N/A — no source crate logic changes.

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~4,000 |
| `.github/workflows/ci.yml` (full file, ~490 lines) | ~7,000 |
| `Cargo.toml` (relevant profile section, ~50 lines) | ~700 |
| `.cargo/config.toml` (current content + new target section) | ~500 |
| `.config/nextest.toml` (verify nextest profile definition) | ~300 |
| ci-speed-research.md Q6 + Q7 sections | ~2,800 |
| `cargo build --timings` HTML output (reference for AC-001) | ~1,000 |
| **Total** | **~16,300** |

16,300 tokens is within 20% of a 200k-token agent context window. No split needed.

## Tasks

### A: Profiling baseline (MUST do before mold)

- [ ] Read `.github/workflows/ci.yml` fully; confirm the arm64 test leg is
      `test (linux-arm64)` with runner `ubuntu-24.04-arm`
- [ ] Read `Cargo.toml` to understand the existing profile structure (does a `[profile.ci]`
      already exist? What does it set for `debug`/`codegen-units`/`incremental`?)
- [ ] Read `.cargo/config.toml` to understand the current linker configuration
      (any existing `[target.aarch64-unknown-linux-gnu]` or `RUSTFLAGS` settings?)
- [ ] Read `.config/nextest.toml` to confirm the nextest `[profile.ci]` definition
      and verify it is the execution profile (not a Cargo build profile)
- [ ] **[VERIFY-AT-IMPL — GATING PRE-CHECK]** Confirm the default linker for
      `aarch64-unknown-linux-gnu` on the CURRENT stable Rust channel:
      ```bash
      rustc +stable --print cfg --target aarch64-unknown-linux-gnu | grep linker
      # or: check recent Rust release notes for lld-as-default changes on aarch64
      ```
      If `rust-lld` became the default for aarch64 after Rust 1.90, mold's benefit is
      the lld→mold delta (smaller than GNU-ld→mold). Calibrate expectation; proceed
      regardless.
- [ ] Add a TEMPORARY profiling step to the arm64 test job:
      ```yaml
      - name: Capture build timings (one-off baseline; remove after baseline recorded)
        run: cargo build --workspace --all-features --timings
      - name: Upload build timings
        uses: actions/upload-artifact@<existing-sha>
        with:
          name: cargo-timings-arm64-baseline
          path: target/cargo-timings/
        if: always()
      ```
- [ ] Trigger a develop push or `full-ci`-labeled PR to capture the baseline
- [ ] Record link/codegen split in PR description; remove the temporary steps before
      merging (or include them in the PR with a TODO note if timing is tight)

### B: mold configuration

- [ ] **[VERIFY-AT-IMPL]** Confirm `rui314/setup-mold` SHA still resolves to mold ≥ 2.41.0:
      `git ls-remote https://github.com/rui314/setup-mold refs/tags/v1`
      Expected: `9c9c13bf4c3f1adef0cc596abc155580bcb04444` (confirmed 2026-06-10).
      NOTE: `v1` is a MOVING major tag — re-resolve at implementation time to confirm
      the SHA has not advanced and that the bundled mold version is still ≥ 2.41.0.
- [ ] **[VERIFY-AT-IMPL]** Confirm `clang` is available on `ubuntu-24.04-arm`:
      ```bash
      which clang && clang --version
      ```
      If absent, fall back to `gcc` as the driver (see EC-002).
- [ ] Add `setup-mold` step to `test (linux-arm64)` job ONLY (BEFORE the build step):
      ```yaml
      - name: Install mold linker (arm64 only)
        uses: rui314/setup-mold@9c9c13bf4c3f1adef0cc596abc155580bcb04444
        # mold 3-10x faster than GNU ld on aarch64-unknown-linux-gnu.
        # x86_64 already uses rust-lld (Rust 1.90 default); mold gain is marginal there.
        # macOS/Windows: Apple ld64/lld-link; mold not applied.
        # SHA confirmed 2026-06-10; re-confirm at implementation (v1 is moving tag).
      ```
- [ ] Handle RUSTFLAGS interaction correctly (see AC-002). Set the linker flag WITHOUT
      clobbering `-D warnings`. Preferred: add to the arm64 job's env:
      ```yaml
      env:
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS: "-C link-arg=-fuse-ld=mold"
      ```
      This target-specific env var is additive with global `RUSTFLAGS=-D warnings`
      (they govern different targets). Verify both flags are active by checking CI build
      output for both `-D warnings` (any clippy/test warnings become errors) and mold
      (build output references mold linker or `target/cargo-timings/` shows link via mold).
- [ ] Run full arm64 test suite (via develop push or `full-ci` label) to confirm all
      tests pass with mold active
- [ ] Capture `cargo build --timings` AFTER mold to get the after state; document in PR

### C: CI profile tuning

- [ ] Read `.config/nextest.toml:32` to confirm the nextest `[profile.ci]` section;
      note that this is the nextest EXECUTION profile, NOT a Cargo build profile
- [ ] Add `[profile.ci]` to `Cargo.toml` (or update if it already exists):
      ```toml
      [profile.ci]
      inherits = "test"
      debug = "line-tables-only"   # Reduces target/ size; keeps backtraces usable
      # codegen-units: leave at default (256 for dev-derived profile) for parallelism
      # incremental: false (enforced via CARGO_INCREMENTAL=0 env; do not set here)
      # strip: "none" (do not strip test binaries; stripping costs time and not needed)
      ```
- [ ] Update the `cargo nextest run` command in `ci.yml` for ALL test jobs to include
      `--cargo-profile ci` in ADDITION to the existing `--profile ci`:
      ```yaml
      run: cargo nextest run --profile ci --cargo-profile ci ...
      ```
      This is REQUIRED for the `[profile.ci]` Cargo build profile to take effect.
      `--profile ci` alone is NOT sufficient (it is a nextest execution flag only).
- [ ] Note that adding `--cargo-profile ci` changes the build output directory from
      `target/debug/` (or `target/test/`) to `target/ci/`. Update rust-cache `shared-key`
      if needed to capture `target/ci/` in the cache.
- [ ] Capture `du -sh target/` (or `du -sh target/ci/`) before and after on an arm64 run
      to document size reduction
- [ ] Trigger NFR-001 criterion benchmark run on linux-x86_64 to confirm < 500ms
      (the profile change only affects test profile; production/release profile unchanged)

### D: Verification

- [ ] Full matrix (develop push or `full-ci` label): all 5 platforms green
- [ ] Confirm no new `No space left on device` errors
- [ ] Remove temporary profiling steps (upload-artifact for baseline) from ci.yml
      before final commit
- [ ] `cargo fmt --all -- --check` clean (`.cargo/config.toml` and `Cargo.toml` changes
      are not Rust source; fmt does not apply; confirm no Rust files were touched)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean

## Previous Story Intelligence

STORY-092 (cache reliability + disk headroom): the disk-cleanup step and rust-cache
reliability changes from STORY-092 reduce the `target/` growth rate. The profile tuning
in this story (AC-004/AC-005) adds a complementary reduction: smaller `target/ci/` from
`line-tables-only` debug info means:

1. Faster cache save/restore (fewer bytes to compress and transfer).
2. More cache budget headroom within the 10 GB GHA limit (STORY-092 AC-004).
3. Reduced risk of hitting the `No space left` threshold (STORY-092 AC-001).

These two stories reinforce each other. The combined effect should be additive, not
redundant. Note: the new `target/ci/` directory from `--cargo-profile ci` will temporarily
increase total cache size during the first LRU cycle after landing, until the old
`target/debug/` or `target/test/` entries age out. Document this in the PR description.

## Architecture Compliance Rules

1. **mold MUST be scoped to arm64 ONLY.** Use `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS`
   (target-specific env var) to carry the mold linker flag. Do NOT use the global `RUSTFLAGS`
   env var (it applies to ALL targets and would clobber `-D warnings`). Do NOT add mold
   to x86_64 (already lld since Rust 1.90; marginal gain). Do NOT add mold to macOS
   (Apple ld64 is fast; marginal gain and adds complexity).
2. **Profile tuning MUST NOT modify `[profile.release]` or `[profile.bench]`.** Those
   profiles govern production binary and benchmark binary quality; touching them requires
   explicit architectural sign-off. Only `[profile.ci]` (test/lint profile) is in scope.
3. **`#![forbid(unsafe_code)]` still enforced.** Changing the linker does not affect
   safe Rust semantics. No `unsafe` is introduced.
4. **`line-tables-only` must produce usable backtraces.** If a test failure during this
   story shows no file:line information in backtraces, escalate to `debug = 1`
   (full line info, less aggressive but safe) rather than removing the optimization
   entirely. `debug = 0` (no debug info) is FORBIDDEN for test builds.
5. **CARGO_INCREMENTAL=0 must remain.** Profile setting `incremental = false` would
   duplicate the env var; leave it as the env var to avoid confusion. Do not set it in
   `[profile.ci]`.
6. **`--profile ci` and `--cargo-profile ci` are DISTINCT flags.** The former is a
   nextest execution profile; the latter is a Cargo build profile. Both must be present
   in the nextest invocation for this story's changes to take effect.
7. **RUSTFLAGS override rule.** Global `RUSTFLAGS` env var OVERRIDES (does NOT merge with)
   `[target.<triple>].rustflags` in `.cargo/config.toml`. Use target-specific env vars
   (`CARGO_TARGET_<TRIPLE>_RUSTFLAGS`) to avoid this override and carry both `-D warnings`
   and the mold flag simultaneously.

## Library and Framework Requirements

No production Rust crate dependencies change. Build toolchain changes only.

| Component | Version/SHA | Status | Notes |
|-----------|-------------|--------|-------|
| `rui314/setup-mold` v1 | `9c9c13bf4c3f1adef0cc596abc155580bcb04444` | CONFIRMED 2026-06-10 via `git ls-remote https://github.com/rui314/setup-mold refs/tags/v1`; re-confirm at implementation (v1 is a MOVING tag) | v1 is a moving major tag — re-resolve SHA + confirm bundled mold ≥ 2.41.0 at implementation time |
| mold bundled version | ≥ 2.41.0 | Verify-at-implementation | Bundled with `setup-mold` at the pinned SHA; no separate mold version pin needed |
| clang (system) | Whatever is on ubuntu-24.04-arm partner image | Verify-at-implementation: `which clang && clang --version` | Required as the driver for `-fuse-ld=mold`; fallback to `gcc` if absent (EC-002) |

## Pinned Versions / SHAs

| Action | SHA | Status | Notes |
|--------|-----|--------|-------|
| `rui314/setup-mold` v1 | `9c9c13bf4c3f1adef0cc596abc155580bcb04444` | CONFIRMED 2026-06-10 via `git ls-remote https://github.com/rui314/setup-mold refs/tags/v1` | v1 is a MOVING major tag — re-resolve at implementation time; confirm bundled mold ≥ 2.41.0 |
| `taiki-e/install-action` v2.81.9 | `fd2f5e3d644b484055ebf4268f474c565f148f25` | SHA noted; NOT used in this story (deferred — only relevant if nextest archiving adopted) | Defer to the story that adopts nextest archiving |

**Speedup magnitude uncertainty:** All "expected wall-clock" figures for mold's benefit
on arm64 are DIRECTIONAL estimates from general Rust-CI literature (research Q6, "a few
percent to ~10% off the arm64 leg"). No slideforge-specific link-phase profiling was
available at research time. The `cargo build --timings` capture in AC-001 is the project-
specific measurement that will replace these directional estimates. The story MUST proceed
regardless of the measured magnitude (the direction is confirmed; the change is low-risk).
Conditioned also on the aarch64 default-linker pre-check (see Summary and Task A).

## File Structure Requirements

| Action | File | Change |
|--------|------|--------|
| Modify | `.github/workflows/ci.yml` | Add `setup-mold` step to arm64 test job only; set `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS` for arm64 job; add `--cargo-profile ci` to nextest invocation; add temporary `--timings` artifact step for baseline (remove before merge or note as one-off) |
| Modify | `.cargo/config.toml` | Add `[target.aarch64-unknown-linux-gnu]` section if needed for linker override; see RUSTFLAGS interaction note — may be handled via env var instead |
| Modify | `Cargo.toml` | Add `[profile.ci]` with `debug = "line-tables-only"` |
| No change | Any `*.rs` file | No Rust source changes |
| No change | `.config/nextest.toml` | Do NOT modify the nextest execution profile — only add Cargo build profile |
| No change | Any other `Cargo.toml` | Only workspace root manifest; no crate-level Cargo.toml changes |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `setup-mold` SHA resolves to a different mold version than 2.41.0 at implementation | Re-confirm SHA; if mold version changed significantly, verify linker flags still valid; note in PR |
| EC-002 | clang not available on ubuntu-24.04-arm partner image | Fallback: use `gcc` as the driver; set `linker = "gcc"` + `rustflags = ["-C", "link-arg=-fuse-ld=mold"]` — `gcc -fuse-ld=mold` is supported by mold |
| EC-003 | `line-tables-only` causes test failure with no backtrace | Escalate to `debug = 1` (full line info); do NOT set `debug = 0` (no info) |
| EC-004 | mold produces a binary that passes CI but fails a runtime assertion on arm64 | Indicates a mold bug; revert mold config and file upstream issue. Linker should not affect safe Rust semantics; full test suite verifies binary correctness. |
| EC-005 | `[profile.ci]` does not exist and nextest is not invoked with `--cargo-profile ci` | Create `[profile.ci]`; add `--cargo-profile ci` to nextest command in ci.yml; confirm all existing tests still pass |
| EC-006 | NFR-001 criterion benchmark regresses after profile changes | Profile change was scoped to `[profile.ci]`; bench job should use `[profile.bench]` or `release`. Inspect if bench job accidentally picks up `[profile.ci]`; separate bench and test profiles if conflated. |
| EC-007 | `.cargo/config.toml` does not exist yet | Create it; if using target-specific env var approach for RUSTFLAGS (preferred), `config.toml` may only need the `[target.aarch64-unknown-linux-gnu]` linker comment/documentation section, not an active rustflags entry. |
| EC-008 | `rust-lld` is already the default for aarch64 on current stable Rust | mold's benefit is smaller (lld→mold delta). Still proceed with the change; calibrate expectation; document in PR description. The gating pre-check in Task A surfaces this before implementation. |
| EC-009 | Global `RUSTFLAGS=-D warnings` clobbers mold linker flag from `.cargo/config.toml` | Use `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS` env var for the mold flag instead of config.toml rustflags. This is the PREFERRED approach per AC-002 and Architecture Compliance Rule 7. |

## Forbidden Dependencies

- Do NOT add mold to x86_64, macOS, or Windows legs.
- Do NOT set `RUSTFLAGS=-Clink-arg=-fuse-ld=mold` as a global env var in `ci.yml` (it
  would clobber `-D warnings` and apply to all targets).
- Do NOT modify `[profile.release]` or `[profile.bench]` (only `[profile.ci]`).
- Do NOT set `debug = 0` (no debug info) in `[profile.ci]` — backtraces become useless.
- Do NOT adopt nextest archive/partition in this story (deferred; see below).
- Do NOT modify `.config/nextest.toml` — the nextest execution `[profile.ci]` there is
  separate from and must not be confused with the Cargo build `[profile.ci]` in `Cargo.toml`.

## Rejected/Deferred Alternatives

| Alternative | Decision | Reason |
|-------------|----------|--------|
| nextest archive / build-once-test-many | DEFERRED | Test-execution optimization; the arm64 pole is compile-time, not test-execution time. mold targets the link step of the compile; nextest archiving saves test EXECUTION, which is not the bottleneck today. Research Q4: "Limited benefit until the suite grows large." Revisit when test execution time dominates. |
| sccache on arm64 | REJECTED | Same reasons as STORY-092. Shares 10 GB GHA cache budget with rust-cache; rarely fires on warm `target/`; worsens thrashing. Research Q5. |
| mold on all Linux legs (x86_64 + arm64) | NOT NEEDED | x86_64 already uses rust-lld by default since Rust 1.90 (Sept 2025). Adding mold to x86_64 would only give the lld→mold delta, which is ~2-4× on link vs GNU ld→mold's ~3-10×. Small marginal gain. Research Q6, HIGH confidence. |
| mold on macOS | NOT NEEDED | Apple ld64 is already fast and parallel; mold gains are marginal on macOS. Adds setup complexity without meaningful benefit. Research Q6. |
| lld-link on Windows | OUT OF SCOPE | A real speedup on Windows MSVC, but adds non-trivial configuration complexity. Defer to a dedicated Windows build-time story if this becomes a bottleneck. |
| `codegen-units = 1` in CI profile | REJECTED | Maximizes code quality but MAXIMIZES compile time. This is the opposite of what we want. Keep at default (256) for maximum parallelism in CI. |
| `strip = "debuginfo"` or `strip = "symbols"` | REJECTED | Stripping test binaries costs additional time and breaks backtrace usefulness. Not appropriate for CI test binaries. |
| `debug = 0` (no debug info) | REJECTED | Backtraces become unusable. `line-tables-only` is the correct balance: usable backtraces at significantly reduced object size. |
| Setting linker flag via `.cargo/config.toml` `rustflags` only | RISKY | Global `RUSTFLAGS=-D warnings` env var overrides config.toml `[target.*].rustflags`. Use `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS` env var instead to avoid the override. |

## Test Strategy

`tdd_mode: facade`. No Rust unit tests for CI configuration. Verification is via CI
observation and artifact inspection:

1. **AC-001:** `cargo build --timings` artifact uploaded from arm64 run; link/codegen
   split recorded in PR description before mold is added.
2. **AC-002/AC-003:** Inspect CI YAML diff; confirm `setup-mold` is ONLY in the arm64
   job; confirm `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS` is set correctly;
   after-mold timings in PR description; confirm both `-D warnings` and mold are active.
3. **AC-004:** `Cargo.toml` diff confirms `[profile.ci]` with `debug = "line-tables-only"`;
   CI YAML diff confirms `--cargo-profile ci` added to nextest invocation; NFR-001 bench
   run confirms < 500ms.
4. **AC-005:** Before/after `target/` size from CI log lines (`du -sh target/ci/`).
5. **AC-006:** Full matrix (develop push or `full-ci` label) shows all 5 platforms green.

## Complexity Estimate

5 story points. The mold configuration is mechanical but has a real correctness gotcha
(RUSTFLAGS override, `--cargo-profile ci` addition). The profile tuning adds 5-10 lines
to `Cargo.toml` and `ci.yml`. The complexity comes from: (a) the profiling baseline task
(AC-001, requires a CI run before and after), (b) carefully scoping the mold step and
RUSTFLAGS to arm64-only, (c) the aarch64 default-linker gating pre-check. Estimated
2 days including the two-phase CI observation cycle.

## Uncertainty Resolution Log

### Resolved (2026-06-10, remove-uncertainty pass)

| Item | Resolution |
|------|-----------|
| `rui314/setup-mold` v1 SHA | **CONFIRMED** `9c9c13bf4c3f1adef0cc596abc155580bcb04444` via `git ls-remote https://github.com/rui314/setup-mold refs/tags/v1` 2026-06-10. Remove prior "re-confirm required" flag; replace with "re-confirm at implementation because v1 is a MOVING tag." |
| `[profile.ci]` conflation with nextest `--profile ci` | **CORRECTNESS BUG FIXED.** The prior AC-004 stated "confirm this is already the case — ci.yml runs `--profile ci`." This was INCORRECT. `--profile ci` is a nextest execution profile flag; `[profile.ci]` in Cargo.toml requires `--cargo-profile ci` (a SEPARATE flag). Verified against repo: `ci.yml:168` runs nextest `--profile ci`; `Cargo.toml` has no `[profile.ci]`; there is no `.cargo/config.toml`. AC-004 fully rewritten to require (a) `[profile.ci]` in Cargo.toml, (b) `--cargo-profile ci` added to nextest in ci.yml, and (c) `--profile ci` kept as-is. |
| `taiki-e/install-action` v2.81.9 SHA | SHA `fd2f5e3d...` noted; DEFERRED — current pin `d9be7d8c` stays; only relevant if nextest archiving adopted. |
| Repo owner placeholder | Confirmed: `drbothen/slideforge`, PUBLIC. |
| RUSTFLAGS override interaction | **NEW FINDING added to story.** Global `RUSTFLAGS` env var overrides (does NOT merge with) `[target.<triple>].rustflags` in `.cargo/config.toml`. Added as Architecture Compliance Rule 7, updated AC-002, added EC-009, added verify-at-impl task, updated Tasks B and rejected alternatives. Resolution: use `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS` env var. |
| `target/ci/` new directory after `--cargo-profile ci` | **NEW FINDING noted.** Adding `--cargo-profile ci` means Cargo uses `target/ci/` (not `target/debug/` or `target/test/`). First run will be cold for this directory. Added to AC-004, AC-005, Previous Story Intelligence, and Tasks C. |

### Deferred to Implementation (verify-at-implementation markers)

| Item | Resolution Command / Check |
|------|---------------------------|
| aarch64 default linker on current stable Rust (GATING PRE-CHECK) | `rustc +stable --print cfg --target aarch64-unknown-linux-gnu` on the arm64 runner; or check recent Rust release notes. If `rust-lld` became default for aarch64 after Rust 1.90, recalibrate expected speedup. |
| `rui314/setup-mold` v1 SHA re-confirmation | `git ls-remote https://github.com/rui314/setup-mold refs/tags/v1` at implementation time; confirm SHA still resolves to mold ≥ 2.41.0 (v1 is a moving tag). |
| clang presence on ubuntu-24.04-arm | `which clang && clang --version` on the live image; fallback to gcc if absent (EC-002). |
| rust-cache shared-key coverage of `target/ci/` | After adding `--cargo-profile ci`, verify the `Swatinem/rust-cache` shared-key for test jobs correctly captures `target/ci/` in the cache (not just `target/debug/`). |

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-06-10 | story-writer | Initial creation per human direction 2026-06-10. Grounded in ci-speed-research.md Q6 + Q7. mold arm64-only (x86_64 already lld since Rust 1.90). debug=line-tables-only for profile tuning. Speedup magnitudes are directional — AC-001 requires profiling before commit. Depends on STORY-091 + STORY-092. |
| 1.1 | 2026-06-10 | story-writer | remove-uncertainty pass: confirmed rui314/setup-mold SHA `9c9c13bf...` via `git ls-remote` 2026-06-10 (re-confirm at impl since v1 is moving tag); CORRECTNESS FIX to AC-004 — removed false parenthetical "confirm --profile ci is already the case"; rewrote AC-004 to require BOTH `[profile.ci]` in Cargo.toml AND `--cargo-profile ci` in nextest invocation (they are distinct flags — nextest `--profile ci` does NOT invoke a Cargo build profile); added `target/ci/` directory implication and cache interaction note; added RUSTFLAGS override correctness finding (global RUSTFLAGS overrides config.toml target rustflags — use CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS); added aarch64 default-linker gating pre-check as verify-at-impl; added EC-008 and EC-009; updated Architecture Compliance Rules 6 and 7; resolved `<owner>` placeholder; added Uncertainty Resolution Log. |
