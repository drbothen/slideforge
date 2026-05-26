---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-053
title: "CI: Supply-Chain Audit (cargo audit + deny + SBOM)"
epic: EPIC-19
wave: 1
points: 5
priority: P0
tdd_mode: facade
status: draft
producer: story-writer
phase: 2
target_module: "(infrastructure — .github/workflows/ + deny.toml)"
subsystems: []
behavioral_contracts: []
# BC status: pending PO authorship (EPIC-19 enforces NFRs, not BCs)
nfr_refs:
  - NFR-016  # 0 high/critical CVEs (cargo audit)
  - NFR-017  # 0 denied licenses (cargo deny)
  - NFR-020  # SBOM in release artifacts
  - NFR-025  # production dep version pinning (= prefix) — enforced in security.yml
verification_properties: []
depends_on:
  - STORY-051  # CI base workflow must exist
blocks:
  - STORY-054  # release pipeline includes SBOM in artifacts; deny.toml must exist first
estimated_days: 1
---

# STORY-053: CI: Supply-Chain Audit (cargo audit + deny + SBOM)

## Summary

Establish the supply-chain security CI configuration: `cargo audit` for CVE
scanning, `cargo deny` for license/ban/source enforcement, `deny.toml` with all
four sections populated, and a `security.yml` workflow that runs on every
Cargo-touching PR and on a weekly schedule. Also specifies the SBOM generation
step (CycloneDX/SPDX format via `anchore/sbom-action`) that is activated in the
release pipeline (STORY-054).

This story produces the complete `deny.toml` configuration and validates that the
existing `security.yml` workflow covers all required NFRs. The `supply-chain` job
already exists in `ci.yml` (from STORY-051); this story ensures `deny.toml` is
fully populated so that job does not fail on a missing config file.

## Narrative

As a devops engineer, I want `cargo audit`, `cargo deny`, and SBOM generation
configured with explicit policy files, so that no high or critical CVEs, no denied
licenses, and no untracked dependency sources can enter the production build
without breaking CI.

## Behavioral Contracts

No BCs govern supply-chain security directly. This story enforces:

| NFR-ID | Requirement | Enforcement Mechanism |
|--------|-------------|----------------------|
| NFR-016 | 0 high/critical CVEs | `cargo audit --deny warnings` in `ci.yml supply-chain` + `security.yml audit` |
| NFR-017 | 0 denied licenses | `cargo deny check licenses` in `ci.yml supply-chain` + `security.yml audit` |
| NFR-020 | SBOM in release artifacts | `anchore/sbom-action` in `release.yml` sbom job (STORY-054 activates) |
| NFR-025 | = version pinning | `security.yml pinning-audit` Python script inspection of Cargo.toml files |

## Acceptance Criteria

### AC-001: deny.toml — Advisories Section

`deny.toml` exists at the repository root and contains an `[advisories]` section:
```toml
[advisories]
# Maximum vulnerability severity to permit.
# "medium" allows low/medium; "none" allows nothing; "all" allows all (unsafe).
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"      # any CVSS-scored vulnerability = deny
unmaintained = "warn"       # unmaintained crate = warn (not block)
unsound = "deny"            # unsound crate = deny
yanked = "deny"             # yanked crate version = deny
notice = "warn"             # informational advisories = warn
```

No `ignore = [...]` entries are permitted without a dated comment explaining why
the advisory is accepted and when it will be resolved.

### AC-002: deny.toml — Licenses Section

`deny.toml` contains a `[licenses]` section permitting only OSI-approved and
compatible licenses:
```toml
[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
    "CC0-1.0",
    "Zlib",
]
deny = []
copyleft = "warn"
allow-osi-fsf-free = "neither"
default = "deny"
private = { ignore = false }
```

Any `allow-list` exception for a crate must include a comment with the crate name,
its license, and justification.

Validates: NFR-017.

### AC-003: deny.toml — Bans Section

`deny.toml` contains a `[bans]` section:
```toml
[bans]
multiple-versions = "warn"      # warn on duplicate versions, not deny (common in ecosystem)
wildcards = "deny"              # wildcard version specs forbidden
highlight = "all"
skip = []
skip-tree = []
```

The `wildcards = "deny"` rule enforces production dependency version specificity
(complements NFR-025 `=` pinning check).

### AC-004: deny.toml — Sources Section

`deny.toml` contains a `[sources]` section:
```toml
[sources]
unknown-registry = "deny"       # deny crates not from crates.io or allowed registries
unknown-git = "deny"            # deny git dependencies from unknown sources
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

`allow-git` starts empty. Any git dependency added must have an entry here with a
dated comment explaining why crates.io is not used.

### AC-005: cargo audit Integration in ci.yml supply-chain Job

The `supply-chain` job in `ci.yml` runs:
1. `cargo audit --deny warnings` — exits non-zero on any advisory
2. `cargo deny check --config deny.toml advisories licenses bans sources` — exits
   non-zero on any violation

Both steps must succeed for the job to pass. The `supply-chain` job is in the
`needs:` array of `all-checks-pass`.

Validates: NFR-016, NFR-017.

### AC-006: security.yml — Scheduled + PR Audit Workflow

`.github/workflows/security.yml` exists and triggers on:
- `pull_request` targeting `develop`/`main` when `Cargo.toml`, `Cargo.lock`,
  `**/Cargo.toml`, or `deny.toml` are modified
- Weekly schedule: Monday at 06:00 UTC (`cron: "0 6 * * 1"`)
- `workflow_dispatch` for manual triggers

On schedule failure: opens a GitHub Issue with title
`[Security] Dependency vulnerabilities found (YYYY-MM-DD)` with cargo audit and
cargo deny output (truncated to 4000 chars each).

On PR failure: posts a PR comment with the audit output.

### AC-007: security.yml — Pinning Audit

`security.yml` contains a `pinning-audit` job that runs a Python script to verify
all production `Cargo.toml` files use `=` version prefixes (NFR-025). The script:
- Reads all `Cargo.toml` files in the workspace (excluding `target/` and `.worktrees/`)
- Parses `[workspace.dependencies]` and per-crate `[dependencies]` sections
- Skips path dependencies, dev-only crates (insta, proptest, criterion, kani), and
  workspace-member references (`workspace = true`)
- Fails with a list of violations if any production dep lacks `=` prefix

Validates: NFR-025.

### AC-008: security.yml — Semgrep Static Analysis

`security.yml` contains a `semgrep` job that:
- Installs `semgrep==1.90.0` (pinned version) via pip3
- Runs `semgrep scan --config "p/rust" --config "p/secrets" --severity ERROR --error .`
- Uploads SARIF output to GitHub Security tab via
  `github/codeql-action/upload-sarif@fee9466b8957867761f2d78f922ab084e3e2dd17`
  (continue-on-error for SARIF upload since GitHub Advanced Security may not be enabled)
- Fails the job if semgrep finds ERROR-severity findings

### AC-009: SBOM Generation (release activation)

The SBOM specification for the release pipeline is established here but activated
in STORY-054. The SBOM is generated using
`anchore/sbom-action@e22c389904149dbc22b58101806040fa8d37a610` in SPDX-JSON format.
Output file: `slideforge-{tag}-sbom.spdx.json`.

This story validates:
- `anchore/sbom-action` SHA is documented in the release workflow action comments
- The SBOM format is SPDX-JSON (not CycloneDX — SPDX has broader tooling support)
- The SBOM artifact is uploaded with `retention-days: 1` in the release build job

### AC-010: SHA-Pinned Actions in security.yml

All `uses:` references in `security.yml` are SHA-pinned. Validated actions:
- `actions/checkout`: `34e114876b0b11c390a56381ad16ebd13914f8d5`
- `dtolnay/rust-toolchain`: `3c5f7ea28cd621ae0bf5283f0e981fb97b8a7af9`
- `Swatinem/rust-cache`: `42dc69e1aa15d09112580998cf2ef0119e2e91ae`
- `taiki-e/install-action`: `d9be7d8cda89035c9c843f78bd44d4f72d8403d4`
- `github/codeql-action/upload-sarif`: `fee9466b8957867761f2d78f922ab084e3e2dd17`
- `actions/github-script`: `f28e40c7f34bde8b3046d885e986cb6290c5673b`

### AC-011: Minimal Permissions in security.yml

`security.yml` declares `permissions:` at the workflow level:
```yaml
permissions:
  contents: read
  pull-requests: write    # post PR comment on findings
  issues: write           # open issue on scheduled findings
  security-events: write  # upload SARIF to GitHub Security tab
```

No `write-all` or `contents: write` without justification. Each job inherits only
what it needs; no job-level permissions override grants more than the workflow level.

## Tasks

1. Create `deny.toml` at repository root with all four sections (ACs 001-004)
2. Audit `security.yml` against ACs 006-011; fill any gaps
3. Audit `ci.yml` `supply-chain` job against AC-005; confirm `deny.toml` is passed
4. Verify `deny.toml` is valid: run `cargo deny check --config deny.toml --dry-run`
   locally before committing
5. Verify `cargo audit` passes on the current `Cargo.lock` (no outstanding CVEs)
6. Document SBOM tooling choice (SPDX-JSON) in `deny.toml` header comment
7. Add `cargo-audit` and `cargo-deny` version comments in `deny.toml` header
8. Confirm `security.yml` is in `needs:` of `all-checks-pass` — it is NOT (security
   runs on a different trigger). Add a note in `ci.yml` about this separation.
9. Confirm `pinning-audit` Python script in `security.yml` handles workspace
   member `[dependencies]` tables correctly (not just `workspace.dependencies`)
10. Test: temporarily introduce an unlicensed dep in a branch, confirm `cargo deny`
    fails the `supply-chain` job

## File List

| File | Action | Purpose |
|------|--------|---------|
| `deny.toml` | Create | cargo deny config — all 4 sections populated |
| `.github/workflows/security.yml` | Verify/update | Supply-chain + semgrep + pinning-audit workflow |
| `.github/workflows/ci.yml` | Verify | Confirm supply-chain job uses deny.toml correctly |

## Test Strategy

Facade mode validation:

1. **deny.toml structural validation:** `cargo deny check --config deny.toml`
   passes on the current workspace with zero violations (against a clean Cargo.lock).
2. **cargo audit baseline:** `cargo audit` passes with zero high/critical advisories
   at time of commit. If any are found, fix the dependency (not suppress) per the
   production-grade default.
3. **pinning-audit script correctness:** Run the pinning-audit Python script locally
   against the workspace; confirm all production deps pass the `=` check. Verify
   the dev-only exemption list is correct.
4. **semgrep validation:** Run `semgrep scan --config "p/rust" --dry-run .` locally;
   confirm no ERROR-severity findings in the current codebase.
5. **Live workflow test:** Push a Cargo.toml change to a feature branch targeting
   `develop`; verify `security.yml` triggers and all three jobs (`audit`, `semgrep`,
   `pinning-audit`) pass.
6. **Negative test:** Temporarily add a crate with a GPL license and confirm `cargo
   deny check licenses` exits non-zero (then revert).

## Previous Story Intelligence

N/A — first story in EPIC-19, Wave 1. This story produces `deny.toml` which STORY-054
(release pipeline) depends on for SBOM generation and license compliance at release time.

## Architecture Compliance Rules

1. **`deny.toml` is the single source of truth for license policy.** Any crate-level
   license exceptions must be in `deny.toml`, not scattered in CI script comments.
2. **`cargo audit --deny warnings` is the CI flag.** `--deny high` misses medium
   advisories that may aggregate into supply-chain risks. Use `--deny warnings` to
   catch all levels.
3. **No `ignore = [...]` in `[advisories]` without dated rationale.** Suppressions
   that age without removal become silent risks.
4. **SBOM format is SPDX-JSON.** CycloneDX is also acceptable but SPDX has broader
   compatibility with NIST SSDF and OpenSSF Scorecard requirements.
5. **semgrep version is pinned.** `semgrep==1.90.0` — not `semgrep>=1.x.x`. Ruleset
   changes between versions can silently suppress or add findings.

## Library and Framework Requirements

| Tool | Version / Pinning | Usage |
|------|------------------|-------|
| `cargo-audit` | latest stable via taiki-e/install-action | CVE scanning |
| `cargo-deny` | latest stable via taiki-e/install-action | License/ban/source enforcement |
| `anchore/sbom-action` | SHA `e22c389904149dbc22b58101806040fa8d37a610` (v0) | SBOM generation |
| `semgrep` | `1.90.0` (pip3 install) | Static security analysis |
| `actions/github-script` | SHA `f28e40c7f34bde8b3046d885e986cb6290c5673b` (v7) | PR/Issue comment API |
| `github/codeql-action/upload-sarif` | SHA `fee9466b8957867761f2d78f922ab084e3e2dd17` (v3) | SARIF upload |

## File Structure Requirements

```
deny.toml                             ← cargo deny policy (advisories + licenses + bans + sources)
.github/
  workflows/
    security.yml                      ← audit + semgrep + pinning-audit (on PR + schedule)
    ci.yml                            ← supply-chain job references deny.toml
```

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~2,000 |
| `deny.toml` (new file) | ~300 |
| `.github/workflows/security.yml` (existing) | ~900 |
| NFR catalog (NFR-016, NFR-017, NFR-020, NFR-025) | ~200 |
| **Total** | **~3,400** |

Within 20% context budget. No splitting required.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A new crate added in a PR has an ambiguous license (e.g., "MIT OR Apache-2.0") | `cargo deny` handles SPDX expression syntax. The `allow` list includes both MIT and Apache-2.0 so the OR expression resolves. |
| EC-002 | `cargo audit` advisory database is stale (cached from previous run) | `cargo audit` always fetches fresh advisory-db in CI (`--no-local-crates` not needed; DB is cloned fresh per run via advisory-db URL). |
| EC-003 | A workspace crate uses `features = [...]` which triggers a path-dep that has no version | Path-dep detection in pinning-audit skips version check for path deps (`'path' in dep_spec`). Correct. |
| EC-004 | `semgrep scan` exits non-zero due to network timeout fetching ruleset | Add `continue-on-error: true` on the semgrep scan step; evaluate results in a subsequent step that can distinguish network failure from security findings. Already handled in security.yml. |
| EC-005 | GitHub Advanced Security not enabled — SARIF upload fails | `continue-on-error: true` on the SARIF upload step. The semgrep scan result still fails the job on findings; SARIF upload is bonus observability. |
| EC-006 | A transitive dependency is yanked from crates.io | `yanked = "deny"` in `[advisories]` catches this. The PR author must update the dep tree to remove the yanked version. |
| EC-007 | `cargo deny` `multiple-versions = "warn"` generates excessive noise for common crates | Use `skip = [{ name = "windows", version = "*" }]` entries in `[bans]` for unavoidable Windows API crate duplicates. Document each skip with a comment. |

## Dependencies

- **Depends on:** STORY-051 (CI base workflow must exist; supply-chain job references deny.toml)
- **Blocks:** STORY-054 (release pipeline depends on deny.toml being populated for the
  release SBOM step; SBOM requires a clean deny.toml baseline)

## Implementation Notes

The existing `security.yml` and the `supply-chain` job in `ci.yml` already implement
the core workflow. This story's primary deliverable is `deny.toml` — the policy file
that both jobs reference but which currently does not exist (resulting in `cargo deny`
failing with "config file not found").

The implementer should:
1. Create `deny.toml` exactly as specified in ACs 001-004
2. Run `cargo deny check` locally to confirm zero violations
3. Run `cargo audit` locally to confirm zero high/critical CVEs in `Cargo.lock`
4. Review `security.yml` against ACs 006-011 and fill any gaps
5. Ensure semgrep is pinned to `1.90.0` (not a floating version)

**SBOM note:** `anchore/sbom-action` is used in `release.yml` (STORY-054), not in
`security.yml`. This story specifies the SBOM format and tooling choice; STORY-054
activates it in the release pipeline.

**Forbidden dependencies:**
- Do not use `cargo-deny` in a way that requires nightly Rust (it supports stable)
- Do not use `CycloneDX` format for SBOM — use SPDX-JSON (per AC-009 rationale)
- Do not introduce `allow-all` or `copyleft = "allow"` in `deny.toml` without P0
  escalation to the architect (GPL in production artifacts is a legal risk)
- Do not suppress any `cargo audit` advisory with `ignore = ["RUSTSEC-XXXX-XXXX"]`
  without a dated comment and explicit human approval
