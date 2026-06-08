---
document_type: devops-reference
section: cicd-setup
version: "1.1"
status: approved
producer: devops-engineer
timestamp: 2026-05-24T00:00:00
modified: 2026-06-07
modification_note: "NFR-002 incremental-rebuild bench gate deferred to v1.x (human-approved 2026-06-07; nfr-catalog v1.3); bench tooling note corrected (jq+estimates.json absolute gate; critcmp =0.1.8 relative regression)"
traces_to: architecture/ARCH-INDEX.md, architecture/tooling-selection.md, prd-supplements/nfr-catalog.md
---

# CI/CD Setup — slideforge v1.0

## Overview

Three GitHub Actions workflow files cover the full CI/CD surface for slideforge.
All actions are pinned to full commit SHAs (supply chain security). All jobs
have explicit `timeout-minutes`. No secrets are hardcoded.

---

## Workflow Files

| File | Trigger | Purpose |
|------|---------|---------|
| `.github/workflows/ci.yml` | Push/PR to `develop` or `main` | Format, lint, test, docs, snapshots, supply chain, benchmarks, visual regression |
| `.github/workflows/release.yml` | Tag push `v*` | Verify, cross-compile binaries, generate SBOM, create GitHub Release |
| `.github/workflows/security.yml` | PR (Cargo changes) + weekly schedule | CVE audit, license check, Semgrep SAST, pinning audit |

---

## CI Workflow Detail (ci.yml)

### Jobs

| Job | Runs On | Gate | NFR |
|-----|---------|------|-----|
| `fmt` | ubuntu-latest | Blocking: `cargo fmt --all -- --check` | NFR-022 |
| `clippy` | ubuntu-latest | Blocking: `clippy::pedantic`, `clippy::unwrap_used` | NFR-021, NFR-022 |
| `test` | 5-platform matrix | Blocking: `cargo nextest run --workspace --all-features` | NFR-026–030 |
| `msrv` | ubuntu-latest | Blocking: `cargo check` with toolchain 1.88 | Cargo.toml `rust-version` |
| `docs` | ubuntu-latest | Blocking: `RUSTDOCFLAGS="-D warnings" cargo doc` | NFR-023 |
| `snapshots` | ubuntu-latest | Blocking: `cargo insta test --check` | Quality bar |
| `supply-chain` | ubuntu-latest | Blocking: `cargo audit` + `cargo deny check` | NFR-016, NFR-017 |
| `bench` | ubuntu-latest | Blocking when fixtures exist: criterion threshold for cold build (NFR-001); NFR-002 incremental gate **DEFERRED to v1.x** — not a v1.0 CI gate | NFR-001 (active); NFR-002 (deferred) |
| `visual-regression` | ubuntu-latest | Blocking when fixtures exist: SSIM ≥ 0.99, PSNR ≥ 35dB | NFR-007, NFR-008 |
| `all-checks-pass` | ubuntu-latest | Synthetic gate — depends on all above | Branch protection target |

### Platform Matrix (test job)

| Runner | Rust Target | NFR |
|--------|-------------|-----|
| ubuntu-latest | x86_64-unknown-linux-gnu | NFR-028 |
| ubuntu-24.04-arm | aarch64-unknown-linux-gnu | NFR-029 |
| macos-latest | aarch64-apple-darwin | NFR-026 |
| macos-13 | x86_64-apple-darwin | NFR-027 |
| windows-latest | x86_64-pc-windows-msvc | NFR-030 |

### Deferred Gates (activate when Phase 3 artifacts land)

- `bench` job: activates when `crates/slideforge-cli/benches/build_bench.rs` exists
- `visual-regression` job: activates when `tests/fixtures/test-fixture.pptx` and `tests/fixtures/reference-pngs/` exist
- `snapshots` job: uses `--unreferenced=warn` until Phase 3 snapshot fixtures are committed

### Branch Protection Recommendation

Set `CI / all-checks-pass` as the single required status check on `develop`.
This avoids enumerating individual job names in the branch protection rule when
jobs are added or renamed.

---

## Release Workflow Detail (release.yml)

### Trigger

Tag push matching `v[0-9]+.[0-9]+.[0-9]+` or pre-release variants (e.g. `v0.1.0-alpha.1`).

### Jobs

| Job | Purpose |
|-----|---------|
| `verify` | Run full test suite on all 5 platforms before producing artifacts |
| `build` | Cross-compile 6 target binaries using `cross` for Linux musl + arm64 |
| `sbom` | Generate SPDX SBOM via anchore/sbom-action (NFR-020) |
| `release` | Assemble all artifacts + SHA-256 checksums → GitHub Release |

### Cross-Compilation Matrix

| Artifact | Runner | Tool | Target |
|----------|--------|------|--------|
| `slideforge-vX.Y.Z-linux-x86_64` | ubuntu-latest | cargo (native) | x86_64-unknown-linux-gnu |
| `slideforge-vX.Y.Z-linux-arm64` | ubuntu-latest | cross | aarch64-unknown-linux-gnu |
| `slideforge-vX.Y.Z-linux-x86_64-musl` | ubuntu-latest | cross | x86_64-unknown-linux-musl |
| `slideforge-vX.Y.Z-macos-arm64` | macos-latest | cargo (native) | aarch64-apple-darwin |
| `slideforge-vX.Y.Z-macos-x86_64` | macos-13 | cargo (native) | x86_64-apple-darwin |
| `slideforge-vX.Y.Z-windows-x86_64.exe` | windows-latest | cargo (native) | x86_64-pc-windows-msvc |

Each binary is accompanied by a `.sha256` checksum file in the release assets.

### Release Notes

If `CHANGELOG.md` contains a section for the tag (e.g. `## [v1.0.0]`), that
section is extracted verbatim. Otherwise, git log since the previous tag is used.

---

## Security Workflow Detail (security.yml)

### Triggers

- Pull request touching `Cargo.toml`, `Cargo.lock`, or `deny.toml`
- Weekly cron: Mondays at 06:00 UTC (catches upstream advisory additions)
- Manual via `workflow_dispatch`

### Jobs

| Job | Tools | On Failure |
|-----|-------|-----------|
| `audit` | `cargo audit`, `cargo deny check` | PR comment (PR) / Issue opened (schedule) |
| `semgrep` | semgrep 1.90.0, `p/rust` + `p/secrets` rulesets, SARIF upload | PR comment (PR) |
| `pinning-audit` | Python script checking `=` prefix on all production deps | Job fails |

### Semgrep Notes

- SARIF results are uploaded to the GitHub Security tab via `github/codeql-action/upload-sarif`
- This requires GitHub Advanced Security (available on public repos and paid plans)
- `continue-on-error: true` on the upload step means the job does not fail if the repo lacks Advanced Security
- High-severity findings still fail the job via the `evaluate semgrep results` step

---

## Support Files Created

| File | Purpose |
|------|---------|
| `deny.toml` | cargo-deny policy: license allowlist, CVE deny, source pinning |
| `.config/nextest.toml` | cargo-nextest profiles: `default` (local) and `ci` (CI, JUnit output) |
| `scripts/visual-diff.py` | SSIM + PSNR slide comparison (NFR-007, NFR-008) |
| `clippy.toml` | Updated `msrv` from 1.85 to 1.88 to match `Cargo.toml` |

---

## Pinned Action Versions

| Action | Version | Commit SHA |
|--------|---------|-----------|
| `actions/checkout` | v4 | `34e114876b0b11c390a56381ad16ebd13914f8d5` |
| `dtolnay/rust-toolchain` | HEAD (stable) | `3c5f7ea28cd621ae0bf5283f0e981fb97b8a7af9` |
| `Swatinem/rust-cache` | v2 | `42dc69e1aa15d09112580998cf2ef0119e2e91ae` |
| `taiki-e/install-action` | v2 | `d9be7d8cda89035c9c843f78bd44d4f72d8403d4` |
| `actions/upload-artifact` | v4 | `ea165f8d65b6e75b540449e92b4886f43607fa02` |
| `actions/download-artifact` | v4 | `d3f86a106a0bac45b974a628896c90dbdf5c8093` |
| `softprops/action-gh-release` | v2 | `3bb12739c298aeb8a4eeaf626c5b8d85266b0e65` |
| `anchore/sbom-action` | v0 | `e22c389904149dbc22b58101806040fa8d37a610` |
| `github/codeql-action` | v3 | `fee9466b8957867761f2d78f922ab084e3e2dd17` |
| `actions/github-script` | v7 | `f28e40c7f34bde8b3046d885e986cb6290c5673b` |

To update any pinned action: resolve the new tag to its commit SHA using
`gh api repos/<owner>/<repo>/git/refs/tags/<tag> --jq '.object.sha'`,
then update the SHA in the workflow file and this table.

---

## Phase Activation Notes

### Phase 3 (when exporter + test fixtures land)

1. Add `tests/fixtures/reference-pngs/` with committed ground-truth PNGs
2. Add `crates/slideforge-cli/benches/build_bench.rs` with criterion benchmark
3. The `bench` and `visual-regression` CI jobs activate automatically
4. Change `snapshots` job `--unreferenced=warn` to `--unreferenced=reject`

**Bench gate tooling (v1.0 scope — NFR-001 only):**

- **Absolute gate (NFR-001, blocking):** parse `target/criterion/<bench-name>/new/estimates.json`
  with `jq` and assert `mean.point_estimate < 500_000_000` (nanoseconds). Fails the
  `bench` job if the cold-build mean exceeds 500ms. `criterion-compare` does not exist
  as a standalone tool and must NOT be referenced.
- **Relative regression (non-blocking advisory):** install `critcmp =0.1.8`; run
  `critcmp main HEAD --threshold 5` to surface regressions > 5% versus the base branch.
  Results are posted as a PR comment but do not fail the job in v1.0.
- **NFR-002 incremental gate:** DEFERRED to v1.x — do NOT add a 50ms incremental
  threshold to the `bench` job for v1.0. The comemo integration that enables
  sub-50ms incremental rebuilds is a post-v1.0 feature.

### Phase 6 (formal hardening)

The CI workflows do not include Kani, cargo-fuzz, or cargo-mutants — these are
compute-intensive and require specialized runners. The formal-verifier agent owns
those gates and runs them as one-off jobs or via a separate `hardening.yml`
workflow targeting Linux only (Kani is Linux/macOS only per verification-architecture.md).

### veraPDF and axe-core

The accessibility gates (NFR-012, NFR-013) are not yet in CI workflows. They
activate in Phase 3/4 when the PDF and HTML exporters land. Expected placement:

- veraPDF: in the `test` matrix job, gated on `if: runner.os == 'Linux'`
  (Docker sidecar) or as a separate `accessibility` job
- `@axe-core/playwright`: separate `a11y` job triggered on PRs touching
  `crates/slideforge-preview/` or `crates/slideforge-html/`
