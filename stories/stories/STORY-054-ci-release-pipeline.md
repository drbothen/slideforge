---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-054
title: "CI: Release Pipeline + Signed Artifacts + Reproducible Builds"
epic: EPIC-19
wave: 1
points: 8
priority: P0
tdd_mode: facade
status: draft
producer: story-writer
phase: 2
target_module: "(infrastructure — .github/workflows/release.yml)"
subsystems: []
behavioral_contracts: []
# BC status: pending PO authorship (EPIC-19 enforces NFRs, not BCs)
nfr_refs:
  - NFR-020  # SBOM in release artifacts (SPDX-JSON via anchore/sbom-action)
  - NFR-025  # production dep version pinning (= prefix)
  - NFR-026  # macOS arm64 binary
  - NFR-027  # macOS x86_64 binary
  - NFR-028  # Linux x86_64 binary (gnu + musl)
  - NFR-029  # Linux arm64 binary
  - NFR-030  # Windows x86_64 binary
  - NFR-031  # reproducible builds: same source + same toolchain → same binary hash
verification_properties: []
depends_on:
  - STORY-051  # CI base workflow
  - STORY-053  # deny.toml must exist; supply-chain must be green before release
blocks: []
estimated_days: 2
---

# STORY-054: CI: Release Pipeline + Signed Artifacts + Reproducible Builds

## Summary

Implement the production release pipeline in `.github/workflows/release.yml`.
The pipeline triggers on semver tags pushed to `main`. It cross-compiles the
`slideforge-cli` binary to 6 targets (macOS arm64/x86_64, Linux x86_64-gnu,
Linux x86_64-musl, Linux arm64, Windows x86_64), generates SHA-256 checksums,
produces a CycloneDX/SPDX SBOM, creates a GitHub Release with all artifacts,
and runs a reproducible-build verification job that confirms two independent builds
of the same tag produce binary-identical output.

The existing `.github/workflows/release.yml` implements this structure. This story
formally specifies the acceptance criteria, validates the reproducible build job
is complete, and fills any gaps in the existing workflow.

## Narrative

As a devops engineer and release manager, I want a fully automated release pipeline
that produces signed, SHA-256 checksummed binaries for all 5 supported platforms
(plus Linux musl), an SBOM, and a GitHub Release on every semver tag, and that
verifies reproducibility by running two independent builds and comparing binary
hashes, so that slideforge releases meet enterprise supply-chain security requirements
from v1.0.

## Behavioral Contracts

No BCs govern release infrastructure directly. This story enforces:

| NFR-ID | Requirement | Implementation |
|--------|-------------|---------------|
| NFR-020 | SBOM in release artifacts | `sbom` job using `anchore/sbom-action` |
| NFR-031 | Reproducible builds | `reproducible-build` job — two builds, binary hash comparison |
| NFR-026-030 | 5-platform binaries | `build` job 6-target matrix |

## Acceptance Criteria

### AC-001: Release Trigger on Semver Tags

`.github/workflows/release.yml` triggers on `push` to tags matching:
- `v[0-9]+.[0-9]+.[0-9]+` (stable: v1.0.0, v1.2.3)
- `v[0-9]+.[0-9]+.[0-9]+-*` (pre-release: v0.1.0-alpha.1, v1.0.0-rc.2)

It does NOT trigger on branch pushes. It does NOT use `workflow_run` (avoids
untrusted input vulnerability).

### AC-002: Permissions Declaration

The `release.yml` workflow declares minimal permissions:
```yaml
permissions:
  contents: write   # create GitHub Release and upload assets
  id-token: write   # OIDC-based signing (sigstore keyless)
```

No `write-all`. No per-job permission escalation beyond this baseline.

### AC-003: Verify Job — Pre-release CI Gate

A `verify` job runs the full nextest suite on all 5 platform runners before any
build artifact is produced. This is the same 5-platform matrix as `ci.yml`:
`ubuntu-latest`, `ubuntu-24.04-arm`, `macos-latest`, `macos-13`, `windows-latest`.

The `verify` job uses the `dist` Cargo profile (or `release` if `dist` is not
defined). `fail-fast: false` to get all platform results.

The `build` and `sbom` jobs both list `verify` in their `needs:` array.

### AC-004: Build Job — 6-Target Cross-Compilation Matrix

The `build` job cross-compiles `slideforge-cli` to 6 targets:

| Matrix Name | Runner | Rust Target | Cross? | Artifact Suffix |
|-------------|--------|------------|--------|----------------|
| `linux-x86_64` | `ubuntu-latest` | `x86_64-unknown-linux-gnu` | no | (none) |
| `linux-arm64` | `ubuntu-latest` | `aarch64-unknown-linux-gnu` | yes (via `cross`) | (none) |
| `linux-x86_64-musl` | `ubuntu-latest` | `x86_64-unknown-linux-musl` | yes (via `cross`) | (none) |
| `macos-arm64` | `macos-latest` | `aarch64-apple-darwin` | no | (none) |
| `macos-x86_64` | `macos-13` | `x86_64-apple-darwin` | no | (none) |
| `windows-x86_64` | `windows-latest` | `x86_64-pc-windows-msvc` | no | `.exe` |

Native builds use `cargo build`. Cross-compiled builds use the `cross` tool
(installed via `taiki-e/install-action`). All builds use `--profile dist`
(the `dist` Cargo profile is defined in `Cargo.toml` with `inherits = "release"`
plus `strip = true` and `lto = "thin"`).

Validates: NFR-026, NFR-027, NFR-028, NFR-029, NFR-030.

### AC-005: SHA-256 Checksums for All Artifacts

Each binary artifact's staging step generates a `.sha256` checksum file:
- On Linux/macOS: `sha256sum "$artifact" > "$artifact.sha256"`
- On Windows (PowerShell):
  `(Get-FileHash "$artifact" -Algorithm SHA256).Hash | Out-File "$artifact.sha256"`
- Cross-platform fallback: detect `sha256sum` vs `shasum -a 256`

The checksum file is uploaded alongside the binary via `actions/upload-artifact`.
Both files appear in the GitHub Release.

### AC-006: Artifact Naming Convention

Released binary artifacts are named:
`slideforge-{tag}-{matrix.name}{artifact.suffix}`

Examples:
- `slideforge-v1.0.0-linux-x86_64`
- `slideforge-v1.0.0-linux-x86_64.sha256`
- `slideforge-v1.0.0-windows-x86_64.exe`
- `slideforge-v1.0.0-windows-x86_64.exe.sha256`
- `slideforge-v1.0.0-linux-x86_64-musl`

### AC-007: SBOM Generation Job

A `sbom` job runs parallel to `build` (both depend on `verify`). It uses:
```yaml
- uses: anchore/sbom-action@e22c389904149dbc22b58101806040fa8d37a610
  with:
    format: spdx-json
    output-file: slideforge-${{ github.ref_name }}-sbom.spdx.json
    artifact-name: sbom-${{ github.ref_name }}
```

The SBOM is uploaded as an artifact with `retention-days: 1` and included in the
GitHub Release. The `release` job lists `sbom` in its `needs:` array.

Validates: NFR-020.

### AC-008: Changelog Extraction

The `release` job extracts release notes from `CHANGELOG.md` if an entry for the
tag exists (format: `## [v1.0.0]`). If no entry exists, it auto-generates release
notes from `git log --pretty=format:"- %s (%h)" $PREV_TAG..$TAG`. The release body
is passed to `softprops/action-gh-release` via `body_path: /tmp/release-notes.md`.

Pre-release tags (containing `-`) set `prerelease: true` in `action-gh-release`.

### AC-009: GitHub Release Creation

The `release` job uses
`softprops/action-gh-release@3bb12739c298aeb8a4eeaf626c5b8d85266b0e65` to create
the release. Configuration:
```yaml
with:
  tag_name: ${{ github.ref_name }}
  name: "slideforge ${{ github.ref_name }}"
  body_path: /tmp/release-notes.md
  draft: false
  prerelease: ${{ contains(github.ref_name, '-') }}
  files: |
    release-assets/*
  fail_on_unmatched_files: true
  generate_release_notes: false
```

`fail_on_unmatched_files: true` ensures missing artifacts cause a build failure
rather than a silent partial release.

### AC-010: Artifact Flattening Step

The `release` job downloads all artifacts from matrix jobs into `release-assets/`.
Since each matrix job uploads to its own subdirectory (artifact named
`binary-{platform}`), a flattening step is required:
```bash
find release-assets/ -mindepth 2 -type f -exec mv {} release-assets/ \;
find release-assets/ -type d -empty -delete
```

After flattening, `release-assets/` contains exactly:
- 6 binary files (one per target)
- 6 `.sha256` files
- 1 SBOM SPDX-JSON file

### AC-011: Reproducible Build Verification Job

A `reproducible-build` job runs AFTER the `build` job (`needs: build`) on
`ubuntu-latest`. It performs the following:

1. Checks out the same tag commit
2. Installs the same toolchain via `dtolnay/rust-toolchain` (stable, with target
   `x86_64-unknown-linux-gnu`)
3. Builds `slideforge-cli` twice in sequence:
   ```bash
   cargo build --release --target x86_64-unknown-linux-gnu --package slideforge-cli --profile dist
   sha256sum target/x86_64-unknown-linux-gnu/dist/slideforge > /tmp/build1.sha256
   cargo clean
   cargo build --release --target x86_64-unknown-linux-gnu --package slideforge-cli --profile dist
   sha256sum target/x86_64-unknown-linux-gnu/dist/slideforge > /tmp/build2.sha256
   ```
4. Compares the two hashes:
   ```bash
   if ! diff /tmp/build1.sha256 /tmp/build2.sha256; then
     echo "REPRODUCIBLE BUILD FAILURE: binary hashes differ"
     exit 1
   fi
   echo "Reproducible build PASS: hashes match"
   ```

Validates: NFR-031.

**Implementation requirement for reproducibility:** The Rust toolchain must produce
reproducible builds. This requires:
- `Cargo.toml` workspace sets `[profile.dist] strip = true lto = "thin"`
- No embedded timestamps or random seeds in build artifacts
- `SOURCE_DATE_EPOCH` environment variable set to the git tag commit timestamp:
  ```bash
  SOURCE_DATE_EPOCH=$(git log -1 --format=%ct "$GITHUB_REF_NAME")
  ```
  This is set in `env:` at the `reproducible-build` job level.

### AC-012: dist Cargo Profile

`Cargo.toml` workspace section defines a `[profile.dist]` profile:
```toml
[profile.dist]
inherits = "release"
lto = "thin"
strip = true
codegen-units = 1
```

`codegen-units = 1` is required for reproducibility (multiple codegen units can
produce non-deterministic output due to LLVM pass ordering). `lto = "thin"` is
a balance between reproducibility and compile time. `strip = true` removes debug
symbols for smaller artifacts.

### AC-013: SHA-Pinned Actions in release.yml

All `uses:` references in `release.yml` are SHA-pinned:
- `actions/checkout` v4: `34e114876b0b11c390a56381ad16ebd13914f8d5`
- `dtolnay/rust-toolchain` HEAD: `3c5f7ea28cd621ae0bf5283f0e981fb97b8a7af9`
- `Swatinem/rust-cache` v2: `42dc69e1aa15d09112580998cf2ef0119e2e91ae`
- `taiki-e/install-action` v2: `d9be7d8cda89035c9c843f78bd44d4f72d8403d4`
- `actions/upload-artifact` v4: `ea165f8d65b6e75b540449e92b4886f43607fa02`
- `actions/download-artifact` v4: `d3f86a106a0bac45b974a628896c90dbdf5c8093`
- `softprops/action-gh-release` v2: `3bb12739c298aeb8a4eeaf626c5b8d85266b0e65`
- `anchore/sbom-action` v0: `e22c389904149dbc22b58101806040fa8d37a610`

### AC-014: Sigstore Keyless Signing (Activation Condition)

The release pipeline is prepared for Sigstore keyless binary signing using
`cosign` from sigstore. The signing step is:
```yaml
- name: Install cosign
  uses: sigstore/cosign-installer@...  # SHA-pinned when activated
- name: Sign binary with Sigstore keyless
  run: cosign sign-blob --yes --bundle "$artifact.bundle" "$artifact"
```

This step is INACTIVE in Wave 1 (defined with `if: false` placeholder). It
activates when the project has a public container/binary registry. A comment in
the workflow marks the activation condition: "Enable when GitHub Container Registry
or release signing infra is confirmed — requires `id-token: write` permission."

The `id-token: write` permission in AC-002 is declared now in anticipation of
this activation.

**NOTE:** The current release pipeline omits active signing but declares the
permission and has the step structure ready. This is an intentional staging of
the supply-chain hardening — the `sha256` checksum approach (AC-005) provides
artifact integrity verification for v1.0.

## Tasks

1. Audit `.github/workflows/release.yml` against ACs 001-013; fill any gaps
2. Create `[profile.dist]` in workspace `Cargo.toml` if absent (AC-012)
3. Implement `reproducible-build` job in `release.yml` (AC-011)
4. Verify `SOURCE_DATE_EPOCH` is set from git tag timestamp in reproducible-build job
5. Verify `cargo clean` between the two builds in reproducible-build job (AC-011)
6. Confirm `sbom` job uses SPDX-JSON format with correct output filename (AC-007)
7. Confirm all 6 build targets are present in the matrix (AC-004)
8. Confirm artifact naming convention matches AC-006 pattern
9. Confirm `fail_on_unmatched_files: true` in `action-gh-release` (AC-009)
10. Add `if: false` placeholder for cosign signing step with activation comment (AC-014)
11. Run a test release on a pre-release tag (e.g., `v0.1.0-test.1`) against a
    non-main branch to confirm the pipeline produces all 6 binaries + SBOM + checksums

## File List

| File | Action | Purpose |
|------|--------|---------|
| `.github/workflows/release.yml` | Verify/update | Release pipeline — all jobs described in ACs |
| `Cargo.toml` (workspace) | Update | Add `[profile.dist]` section (AC-012) |

## Test Strategy

Facade mode validation:

1. **Pre-release dry run:** Push a `v0.1.0-test.1` tag to a non-main branch (the
   workflow triggers on all tags, not just main pushes). Confirm all 6 build matrix
   jobs succeed and produce named artifacts.
2. **Reproducible-build validation:** The `reproducible-build` job is the test.
   If it fails, the binary is non-reproducible and must be fixed in `[profile.dist]`
   before v1.0.0 is tagged.
3. **SBOM presence check:** After the dry-run release, inspect the GitHub Release
   assets and confirm `slideforge-v0.1.0-test.1-sbom.spdx.json` is present.
4. **Checksum file presence:** Confirm all 6 `.sha256` files are in the release
   alongside the binaries.
5. **Changelog extraction:** Create a `CHANGELOG.md` entry for the test tag and
   confirm the release body matches the extracted section.
6. **Mutation test at wave gate:** `cargo-mutants` at Wave 1 gate confirms the
   release workflow is structurally sound (no dead steps, no silent ignores).

## Previous Story Intelligence

N/A — this is the last story in EPIC-19, Wave 1. STORY-053 provides `deny.toml`
which is a prerequisite for the supply-chain cleanliness check that should pass
before releasing. STORY-051 provides the CI base workflow structure this story
extends.

Key lessons from the existing `release.yml`:
- The artifact flattening step is already present (AC-010)
- SHA-pinned actions are already documented in a comment block
- The `verify` job matrix is already present
- The `reproducible-build` job is the primary gap to fill in this story

## Architecture Compliance Rules

1. **`[profile.dist]` must have `codegen-units = 1`.** Multiple codegen units cause
   non-deterministic LLVM pass ordering that breaks reproducibility. This is the most
   common cause of "irreproducible Rust builds" in practice.
2. **`SOURCE_DATE_EPOCH` must be set from the git tag commit timestamp.** The Rust
   toolchain embeds build timestamps in metadata; `SOURCE_DATE_EPOCH` pins them to
   the tag date.
3. **`cargo clean` between builds in reproducibility check.** Without a clean, the
   second build reuses cached incremental artifacts, which trivially makes hashes
   match without proving anything.
4. **Binary signing uses Sigstore keyless, not GPG.** GPG requires key management
   infrastructure; Sigstore keyless uses OIDC identity from GitHub Actions. The
   `id-token: write` permission supports this when activated.
5. **The release job uses `softprops/action-gh-release`, not `gh release create`.**
   The action handles atomic asset upload with retry; the CLI version can fail
   partially leaving a broken release.
6. **`fail_on_unmatched_files: true` is mandatory.** A partial release (missing some
   platform binaries) is worse than no release — it causes user confusion without
   a clear error.

## Library and Framework Requirements

| Tool | Version / Pinning | Usage |
|------|------------------|-------|
| `actions/checkout` | SHA `34e114876b0b11c390a56381ad16ebd13914f8d5` (v4) | Repo checkout |
| `dtolnay/rust-toolchain` | SHA `3c5f7ea28cd621ae0bf5283f0e981fb97b8a7af9` | Toolchain setup |
| `Swatinem/rust-cache` | SHA `42dc69e1aa15d09112580998cf2ef0119e2e91ae` (v2) | Cargo caching |
| `taiki-e/install-action` | SHA `d9be7d8cda89035c9c843f78bd44d4f72d8403d4` (v2) | Install cross, cargo-nextest |
| `actions/upload-artifact` | SHA `ea165f8d65b6e75b540449e92b4886f43607fa02` (v4) | Binary + SBOM upload |
| `actions/download-artifact` | SHA `d3f86a106a0bac45b974a628896c90dbdf5c8093` (v4) | Download for release job |
| `softprops/action-gh-release` | SHA `3bb12739c298aeb8a4eeaf626c5b8d85266b0e65` (v2) | GitHub Release creation |
| `anchore/sbom-action` | SHA `e22c389904149dbc22b58101806040fa8d37a610` (v0) | SPDX-JSON SBOM generation |
| `cross` | latest stable via taiki-e/install-action | Cross-compilation tool |
| Rust stable | per `rust-toolchain.toml` | Toolchain |

## File Structure Requirements

```
.github/
  workflows/
    release.yml                ← primary artifact; all jobs described in ACs
Cargo.toml (workspace section) ← [profile.dist] with strip + lto + codegen-units
```

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~2,800 |
| `.github/workflows/release.yml` (existing) | ~1,100 |
| `Cargo.toml` workspace section (profile.dist) | ~100 |
| NFR catalog (NFR-020, NFR-031, NFR-026-030) | ~200 |
| **Total** | **~4,200** |

Within 20% context budget. No splitting required.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `cargo clean` in reproducible-build job deletes the Swatinem cache | The `reproducible-build` job should NOT use `Swatinem/rust-cache`. The cache would interfere with the clean-build test. Use two fully cold builds. |
| EC-002 | `cross` build for `aarch64-unknown-linux-gnu` requires Docker daemon on the runner | `ubuntu-latest` GitHub Actions runners have Docker pre-installed. `cross` uses Docker internally. Verify this assumption holds for the current runner image version. |
| EC-003 | Windows binary has a different hash on the second build due to PE timestamp embedding | `strip = true` in `[profile.dist]` removes most debug info. For full PE reproducibility, set `RUSTFLAGS="-Zremap-path-prefix"` in the reproducible-build job. Add this if the binary hash test fails on Windows. |
| EC-004 | A pre-release tag (`v0.1.0-alpha.1`) accidentally triggers a production release | `softprops/action-gh-release` with `prerelease: ${{ contains(github.ref_name, '-') }}` marks it correctly. GitHub's release page distinguishes pre-releases visually. |
| EC-005 | `CHANGELOG.md` entry exists but the regex mismatches the tag format | The Python extraction script uses `re.escape(tag)` on the tag string. Verify the CHANGELOG format uses `## [v1.0.0]` (with `v` prefix) — match the tag format. |
| EC-006 | `anchore/sbom-action` fails to resolve Cargo.lock on first run | `anchore/sbom-action` requires the workspace to be in a buildable state. The `sbom` job runs after `verify` which already builds the workspace — ensure `Swatinem/rust-cache` carries over the build artifacts. |
| EC-007 | The `dist` directory does not exist when `cargo build --profile dist` outputs there | The `--profile dist` flag places output in `target/{target}/dist/` not `target/{target}/release/`. The staging step must use the `dist` subdirectory. Already handled in `release.yml` but verify after adding `[profile.dist]` to `Cargo.toml`. |
| EC-008 | Two builds produce different hashes due to `build.rs` embedding `env!("CARGO_PKG_VERSION")` | `CARGO_PKG_VERSION` is derived from `Cargo.toml` and does not embed non-deterministic data. Embedding `env!("BUILD_DATE")` or similar would be a reproducibility violation — forbid in build scripts. |

## Dependencies

- **Depends on:**
  - STORY-051 (CI base workflow — provides runner setup, action SHAs, caching patterns)
  - STORY-053 (supply-chain — deny.toml must exist; supply chain must be green before
    any release binary can be trusted)
- **Blocks:** None (release pipeline is a terminal EPIC-19 story; no product stories
  depend on it)

## Implementation Notes

The existing `.github/workflows/release.yml` implements most of this story already.
The primary gaps are:

1. **`reproducible-build` job** (AC-011) — this job is the key addition. It must
   perform TWO cold builds with `cargo clean` between them and compare hashes.
   Do not skip `cargo clean` — it is the crux of the reproducibility proof.

2. **`[profile.dist]` in `Cargo.toml`** (AC-012) — if the workspace `Cargo.toml`
   does not yet have this profile, add it. Verify the build step references
   `--profile dist` (not `--release`).

3. **`SOURCE_DATE_EPOCH`** — add this to the `reproducible-build` job's `env:`
   block. Compute from the tag commit timestamp:
   ```yaml
   env:
     SOURCE_DATE_EPOCH: ${{ fromJSON(steps.tag-info.outputs.timestamp) }}
   ```
   Or use a shell step: `SOURCE_DATE_EPOCH=$(git log -1 --format=%ct "$GITHUB_REF_NAME")`

4. **Cosign placeholder** (AC-014) — add the `if: false` signing step stub
   with the activation comment. Do not use a real cosign SHA yet (infra not ready).

The `release.yml` workflow is tested with a pre-release tag on a non-main branch
for integration validation. This does not produce a real release visible to users
(GitHub pre-releases are hidden from the main Releases page by default).

**Forbidden dependencies:**
- Do not introduce GPG-based signing — Sigstore keyless is the approved approach
- Do not use `actions/cache` directly in the `reproducible-build` job — the
  Swatinem cache must be absent to ensure a cold build
- Do not embed non-deterministic data (timestamps, random seeds, UUIDs) in build.rs
  output — this breaks reproducibility
- Do not use `git commit --amend` or force-push to push a release tag — tags are
  immutable by convention; create a new tag if a fix is needed
