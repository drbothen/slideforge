---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-051
title: "CI: fmt + clippy + nextest (5-platform matrix)"
epic: EPIC-19
wave: 1
points: 5
priority: P0
tdd_mode: facade
status: draft
producer: story-writer
phase: 2
target_module: "(infrastructure — .github/workflows/)"
subsystems: []
behavioral_contracts: []
# BC status: pending PO authorship (EPIC-19 enforces NFRs, not BCs)
nfr_refs:
  - NFR-021  # zero .unwrap() in non-test code
  - NFR-022  # clippy::pedantic, zero undocumented suppressions
  - NFR-023  # missing_docs on public API
  - NFR-024  # #![forbid(unsafe_code)]
  - NFR-025  # production dep version pinning (= prefix)
  - NFR-026  # macOS arm64 CI
  - NFR-027  # macOS x86_64 CI
  - NFR-028  # Linux x86_64 CI
  - NFR-029  # Linux arm64 CI
  - NFR-030  # Windows x86_64 CI
verification_properties: []
depends_on: []
blocks: []
estimated_days: 1
---

# STORY-051: CI: fmt + clippy + nextest (5-platform matrix)

## Summary

Establish the GitHub Actions CI workflow covering format checking, pedantic Clippy
linting, multi-platform nextest runs, MSRV validation, documentation build, and
snapshot test enforcement. This workflow gates every PR to `develop` and every push
to `develop`/`main`. It must be green before any other Wave 1 story can merge — it
is the quality floor the entire project builds on.

The existing `.github/workflows/ci.yml` implements the core structure described in
this story. The story's ACs validate that the workflow is correctly specified,
remains current with the project's toolchain, and enforces all quality bar NFRs
through CI configuration.

## Narrative

As a devops engineer, I want a single GitHub Actions CI workflow that validates
format, lints, tests, builds documentation, and runs snapshot checks across all 5
supported platforms, so that every merged PR satisfies the production-grade quality
bar from day one with no manual verification steps.

## Behavioral Contracts

No BCs govern CI infrastructure directly. This story enforces the following NFRs
through CI configuration:

| NFR-ID | Requirement | Workflow Job | Gate Type |
|--------|-------------|-------------|-----------|
| NFR-021 | Zero .unwrap() in non-test code | `clippy` | Blocking |
| NFR-022 | clippy::pedantic zero suppressions | `clippy` | Blocking |
| NFR-023 | Missing public docs = zero | `docs` | Blocking |
| NFR-024 | #![forbid(unsafe_code)] | `clippy` | Blocking |
| NFR-025 | = version pinning on prod deps | `supply-chain` / `pinning-audit` | Blocking |
| NFR-026 | macOS arm64 passes | `test (macos-arm64)` | Blocking |
| NFR-027 | macOS x86_64 passes | `test (macos-x86_64)` | Blocking |
| NFR-028 | Linux x86_64 passes | `test (linux-x86_64)` | Blocking |
| NFR-029 | Linux arm64 passes | `test (linux-arm64)` | Blocking |
| NFR-030 | Windows x86_64 passes | `test (windows-x86_64)` | Blocking |

## Acceptance Criteria

### AC-001: Platform Matrix Coverage

The `test` job in `.github/workflows/ci.yml` runs on exactly 5 runners:
`ubuntu-latest` (linux-x86_64), `ubuntu-24.04-arm` (linux-arm64),
`macos-latest` (macos-arm64), `macos-13` (macos-x86_64), `windows-latest`
(windows-x86_64). `fail-fast: false` is set so all platforms report independently.
Each run uses `cargo nextest run --workspace --all-features --no-fail-fast --profile ci`.

Validates: NFR-026, NFR-027, NFR-028, NFR-029, NFR-030.

### AC-002: Format Check Job

The `fmt` job runs `cargo fmt --all -- --check` on `ubuntu-latest` only (formatting
is platform-independent). Uses `rustfmt` component. Times out at 5 minutes. Does not
use Rust cache (fast unconditional check).

### AC-003: Clippy Job (pedantic)

The `clippy` job runs on `ubuntu-latest` with timeout 20 minutes. Flags:
`-D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items`.
Uses `Swatinem/rust-cache` with `shared-key: clippy`. Covers `--all-targets --all-features`.

Validates: NFR-021 (unwrap_used), NFR-022 (pedantic).

### AC-004: Documentation Build Job

The `docs` job runs `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"`.
Runs on `ubuntu-latest` with timeout 15 minutes. Uses `Swatinem/rust-cache` with
`shared-key: docs`. Fails if any public item is undocumented.

Validates: NFR-023.

### AC-005: MSRV Validation Job

The `msrv` job runs `cargo check --workspace --all-features` using toolchain `"1.88"`
(matching `rust-version` in workspace `Cargo.toml`). Runs on `ubuntu-latest` with
timeout 20 minutes. Uses `Swatinem/rust-cache` with `shared-key: msrv`.

### AC-006: Snapshot Test Job

The `snapshots` job runs `cargo insta test --check --workspace --unreferenced=warn`.
Uses `cargo-insta` installed via `taiki-e/install-action`. Runs on `ubuntu-latest`
with timeout 20 minutes. `--unreferenced=warn` (not error) until Phase 3 fixtures
land; will switch to `--unreferenced=reject` when fixtures are committed.

### AC-007: SHA-Pinned Actions

All `uses:` references in the workflow are pinned to full commit SHAs (not tags).
Validated by grepping `.github/workflows/ci.yml` for any `uses:` line matching
`@v[0-9]` or `@main` or `@master` — zero matches allowed.

Pinned SHAs at time of authorship (VERIFY AT IMPLEMENTATION TIME — tags advance):
- `actions/checkout` v4: `11bd71901bbe5b1630ceea73d27597364c9af683` (updated from prior `34e114...` SHA)
- `Swatinem/rust-cache` v2: `42dc69e1aa15d09112580998cf2ef0119e2e91ae` (verify current)
- `dtolnay/rust-toolchain` HEAD: `3c5f7ea28cd621ae0bf5283f0e981fb97b8a7af9` (verify current)
- `taiki-e/install-action` v2: `d9be7d8cda89035c9c843f78bd44d4f72d8403d4` (verify current)

**NOTE:** SHA pins must be verified against the actual current tag commit at implementation time. Run `git ls-remote https://github.com/actions/checkout refs/tags/v4` to confirm. The SHAs above were last validated 2026-05-25.

### AC-008: Concurrency Cancellation

The workflow-level `concurrency:` block uses
`${{ github.workflow }}-${{ github.ref }}` as the group key. `cancel-in-progress`
is `true` for non-protected branches (develop/main do not cancel). This prevents
redundant runs consuming runner minutes.

### AC-009: Synthetic All-Checks-Pass Job

A final job named `all-checks-pass` depends on all required jobs via `needs:` and
runs `if: always()`. It fails if any required job result is not `success` or
`skipped`. Branch protection rules gate on `CI / all-checks-pass` as the single
required status check. `skipped` is treated as success to allow fixture-gated jobs
(visual regression, bench) to pass before Phase 3.

### AC-010: Rust Cache Correctness

The `test` job matrix uses `shared-key: test-${{ matrix.target.name }}` to avoid
cache collisions between platforms. The `clippy` job uses `shared-key: clippy`.
The `docs` job uses `shared-key: docs`. No two jobs share the same cache key unless
they are identical builds.

### AC-011: Environment Variables

The workflow sets at the `env:` top level:
- `CARGO_TERM_COLOR: always` (colored terminal output in logs)
- `RUSTFLAGS: "-D warnings"` (warnings-as-errors for all cargo invocations)
- `RUST_BACKTRACE: 1` (better panic diagnostics in test output)
- `CARGO_INCREMENTAL: 0` (no incremental build in CI — cold caches per run)

### AC-012: Workflow Triggers

The workflow triggers on:
- `push` to `develop` and `main`
- `pull_request` targeting `develop` and `main`

Feature branches run CI only through PRs (not direct pushes) per the git workflow
in `CLAUDE.md`.

## Tasks

1. Verify `.github/workflows/ci.yml` exists and matches the structure described in ACs
2. Confirm all 5 platform runners are present in `test` job matrix
3. Confirm all action `uses:` lines are SHA-pinned (no tag references)
4. Confirm `clippy` job uses `-D clippy::unwrap_used` and `-D clippy::pedantic`
5. Confirm `docs` job sets `RUSTDOCFLAGS="-D warnings"`
6. Confirm `msrv` job uses toolchain `"1.88"` (update if `rust-version` changes)
7. Confirm `snapshots` job uses `--unreferenced=warn` (will become `reject` in Phase 3)
8. Confirm `all-checks-pass` synthetic job exists and depends on all required jobs
9. Confirm `concurrency:` block is present with correct group key and cancel policy
10. Add `CI / all-checks-pass` as required branch protection status check on `develop`
11. Verify CI passes on an empty workspace (`cargo build --workspace` green state)

## File List

| File | Action | Purpose |
|------|--------|---------|
| `.github/workflows/ci.yml` | Verify/update | Main CI workflow — all jobs described in ACs |
| `rust-toolchain.toml` | Read-only reference | Pinned Rust stable channel used by dtolnay/rust-toolchain |
| `.cargo/config.toml` | Create if absent | `[profile.ci]` nextest profile with `retries = 2` |
| `deny.toml` | Stub if absent | cargo deny config for license/CVE/ban rules (supply-chain job uses it) |

## Test Strategy

Facade mode validation approach — no stubs/Red Gate:

1. **Workflow YAML lint:** `actionlint` or equivalent validates the YAML structure
   and expression syntax offline before pushing.
2. **SHA-pin audit:** CI includes a `pinning-audit` step (in `security.yml`) that
   greps all `uses:` for non-SHA references and fails on any match.
3. **Live CI run:** Push a trivial no-op commit to a feature branch targeting
   `develop`; confirm all jobs in `ci.yml` appear in the GitHub Actions run and
   reach `success` or `skipped` status.
4. **Matrix job presence:** Inspect the Actions run to confirm exactly 5 `test`
   job instances appear (one per platform) plus `fmt`, `clippy`, `msrv`, `docs`,
   `snapshots`, `supply-chain`, `bench`, `visual-regression`, `all-checks-pass`.
5. **Mutation test (cargo-mutants) at wave gate:** Wave gate check confirms no test
   escapes from the formatting/lint enforcement (quality gate per BC-8.30.001 for
   facade mode).

## Previous Story Intelligence

N/A — first story in EPIC-19. EPIC-19 has no predecessors in Wave 1 (it runs in
parallel with EPIC-01 and EPIC-02). Lessons from EPIC-01/02 will flow into later
waves via dependency graph.

## Architecture Compliance Rules

Extracted from `CLAUDE.md` and `epics.md`:

1. **No feature branches bypass CI.** The workflow triggers on PR targets only; direct
   feature branch pushes do not run CI (worktree development model per CLAUDE.md).
2. **SHA-pinned actions required.** Any action using `@v1`, `@v2`, `@main`, or `@master`
   is a supply-chain violation. Use the commit SHA at the known-good tag.
3. **`CARGO_INCREMENTAL: 0` is mandatory in CI.** Incremental compilation with warm
   caches produces non-deterministic results; disable it.
4. **`-D warnings` in `RUSTFLAGS`.** Warnings are errors for all `cargo` invocations
   in CI without exception.
5. **`--no-fail-fast` in nextest.** Fail-fast hides latent failures; all test results
   must be collected before reporting.

## Library and Framework Requirements

| Tool | Version / Pinning | Usage |
|------|------------------|-------|
| `actions/checkout` | SHA `34e114876b0b11c390a56381ad16ebd13914f8d5` (v4) | Repo checkout |
| `dtolnay/rust-toolchain` | SHA `3c5f7ea28cd621ae0bf5283f0e981fb97b8a7af9` | Stable toolchain setup |
| `Swatinem/rust-cache` | SHA `42dc69e1aa15d09112580998cf2ef0119e2e91ae` (v2) | Cargo dependency caching |
| `taiki-e/install-action` | SHA `d9be7d8cda89035c9c843f78bd44d4f72d8403d4` (v2) | Install cargo-nextest, cargo-insta |
| `cargo-nextest` | latest stable via taiki-e/install-action | Test runner |
| `cargo-insta` | latest stable via taiki-e/install-action | Snapshot test runner |
| Rust stable | per `rust-toolchain.toml` (currently stable channel) | Toolchain |
| Rust MSRV | `1.88` | Minimum supported Rust version declared in workspace Cargo.toml |

## File Structure Requirements

```
.github/
  workflows/
    ci.yml           ← primary artifact; all jobs described in ACs
    release.yml      ← separate workflow (STORY-054)
    security.yml     ← separate workflow (STORY-053)
.cargo/
  config.toml        ← [profile.ci] with nextest retry config
deny.toml            ← cargo deny config (license allowlist, CVE policy)
rust-toolchain.toml  ← pinned Rust stable channel (pre-existing)
```

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,800 |
| `.github/workflows/ci.yml` (existing) | ~800 |
| `rust-toolchain.toml` | ~50 |
| `.cargo/config.toml` | ~100 |
| `deny.toml` (stub) | ~150 |
| NFR catalog (relevant rows) | ~200 |
| **Total** | **~3,100** |

Well within the 20% context budget for a typical 200k-token agent context window.
No splitting required.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Rust toolchain update changes stable channel | `rust-toolchain.toml` pins the channel; `dtolnay/rust-toolchain` reads it automatically. No workflow change needed. |
| EC-002 | `cargo insta` finds new unreferenced snapshots from other tooling | `--unreferenced=warn` prevents false failures until Phase 3. Comment in workflow explains the switch date. |
| EC-003 | One platform runner is temporarily unavailable (GitHub-side outage) | `fail-fast: false` ensures other platforms complete and report. The affected run is re-triggered after availability restores. |
| EC-004 | SHA-pinned action has a known CVE | `security.yml` `pinning-audit` detects tag drift. Update SHA by resolving the new tag to commit SHA and updating the comment block. |
| EC-005 | `ubuntu-24.04-arm` runner not available in the org | STORY-051 requires it. If unavailable, escalate to architect — this is not deferrable per production-grade default. |
| EC-006 | Windows runner fails clippy with platform-specific warning | The `clippy` job runs only on `ubuntu-latest` (linting is platform-independent). Matrix `test` job handles platform-specific compilation issues separately. |

## Dependencies

- **Depends on:** None (EPIC-19 Wave 1 — no predecessors)
- **Blocks:** None. CI stories run in PARALLEL with code stories in Wave 1 — they do not
  block code stories. Code stories merge THROUGH CI (the CI workflow validates their PRs),
  but CI infrastructure is not a prerequisite that code stories must wait for in the
  dependency graph sense. Wave gate coordination is handled by the wave scheduler, not
  by story-level `blocks:` edges.

## Implementation Notes

The workflow described in this story already exists in `.github/workflows/ci.yml`
as of project bootstrap. This story's role is to formally specify the acceptance
criteria, ensure the existing workflow is complete and correct, and lock down the
configuration so subsequent story PRs have a green gate to merge through.

The implementer (devops-engineer) should:
1. Audit the existing `ci.yml` against each AC above
2. Fill any gaps found (e.g., if `--unreferenced=warn` was missing from the `snapshots` job)
3. Ensure `deny.toml` exists as a stub (STORY-053 will fully populate it)
4. Ensure `.cargo/config.toml` defines `[profile.ci]` with `retries = 2` for nextest

The visual regression and bench jobs in `ci.yml` are intentionally skipped until
Phase 3 fixtures exist. They are defined now so branch protection can reference
`CI / visual-regression` and `CI / bench` without requiring separate workflow
updates later. The `all-checks-pass` synthetic job treats `skipped` as success.

**Forbidden Dependencies:**

The CI workflow itself has no Rust crate dependencies. However, the following must
never be introduced into the `ci.yml` workflow:
- Actions that use mutable tags (e.g., `@v1`) instead of SHA-pinned commits
- `continue-on-error: true` on security-critical steps without explicit justification
- Secrets used in steps that also write to PR comments (secret exfiltration risk)
- `workflow_run` triggers that could be exploited via untrusted input
