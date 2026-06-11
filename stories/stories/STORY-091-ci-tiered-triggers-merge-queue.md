---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-091
title: "CI: Tiered triggers + GitHub merge queue (remove slow legs from per-PR critical path)"
epic: EPIC-19
wave: 5
points: 5
priority: NEXT
tdd_mode: facade
status: draft
spec_version: "1.4"
behavioral_contracts: []
# BC status: pending PO authorship — no product BC governs CI workflow restructuring.
# These stories anchor to NFRs (NFR-026 through NFR-030 multi-platform gates) and the
# devops CLAUDE.md quality bar. Status=draft until PO assigns BCs or explicitly confirms
# NFR-only anchoring is acceptable for CI-infra stories (per STORY-051/052/053 precedent).
verification_properties: []
nfr_refs: [NFR-026, NFR-027, NFR-028, NFR-029, NFR-030]
target_module: .github/workflows/ci.yml
subsystems: []
depends_on: []
blocks: [STORY-092, STORY-093]
estimated_days: 1
# Human priority override 2026-06-10: deliver BEFORE remaining Wave-5 feature stories.
# The merge queue restructuring (STORY-091) must land first so STORY-092 and STORY-093
# can be measured against the tiered baseline.
---

# STORY-091: CI — Tiered Triggers + GitHub Merge Queue

## Subsystem Anchor Justification

EPIC-19 is the CI/CD Infrastructure epic (cross-cutting, no SS-ID). This story belongs
there because it modifies `.github/workflows/ci.yml` exclusively and its scope is CI
pipeline orchestration, matching EPIC-19's mandate: "GitHub Actions workflows for the
full quality bar." STORY-051 through STORY-054, STORY-080, and STORY-090 all reside
in EPIC-19 with the same rationale.

No product subsystem (SS-01 through SS-18) is touched. Architecture compliance rules
are drawn from the devops CLAUDE.md quality bar and the ci-speed-research sidecar
(`.factory/planning/ci-speed-research.md`, Q8 findings).

## Dependency Anchor Justifications

- `depends_on: []` — CI workflow restructuring has no product story prerequisites.
  The `ci.yml` file exists and is deployed; this story modifies it in-place. All
  required GitHub features (merge queue, `merge_group:` trigger) are GA.
- `blocks: [STORY-092, STORY-093]` — STORY-091 must land first because (a) STORY-092's
  cache-reliability work benefits from the tiered trigger being in place (cache warm-path
  matters most on the merge-queue leg where the full matrix runs), and (b) STORY-093's
  arm64 profiling baselines should be captured after the tiered trigger is live so timing
  measurements reflect production conditions, not the old every-PR-full-matrix baseline.

## Summary

Every PR push currently triggers the full 4-platform matrix (1 fast leg + 3 slow legs)
including the 22-minute native arm64 leg, 15-minute+ macOS arm64 leg, and 20-minute
Windows leg. The per-PR feedback wall-clock is dominated by these slow legs even though
the linux-x86_64 fast leg typically completes in 6-8 minutes.

This story restructures `ci.yml` to implement tiered triggers:

1. **Fast tier (every PR push):** `fmt` + `clippy` + `test (linux-x86_64)` + `doctest` +
   `check-panic-profile` + `check-pdf-deps` + `supply-chain`. These complete in ~6-8 min.
2. **Full tier (merge-queue + develop push + main push + manual dispatch + nightly + `full-ci` label):** the complete
   4-platform matrix including `test (linux-arm64)`, `test (macos-arm64)`,
   `test (windows-x86_64)`, `snapshots`, `visual-regression`, `bench`, `perf-smoke`, `msrv`, `docs`.
   Note: `test (macos-x86_64)` (macos-13 Intel runner) is NOT part of ci.yml's test matrix
   — that leg was removed before this story due to chronic runner availability issues. Intel
   macOS binary coverage is handled in release.yml via a native macos-13 build job instead.
3. **Aggregator stays:** the existing `CI / all-checks-pass` synthetic job is made the
   SINGLE required status check. It handles the skipped-slow-legs case correctly by
   treating `skipped` as pass (only fires failure on a leg that actually ran and failed,
   including in merge_group where all legs run).

The `merge_group:` trigger is added to `on:` so the GitHub merge queue validates the
full matrix before landing commits on `develop`.

**Repo context (confirmed 2026-06-10):** Repository is `drbothen/slideforge`, PUBLIC.
GitHub merge queue is FREE for public repositories. Default branch is `main`; active
development branch is `develop`. The merge queue must be enabled on `develop`'s branch
protection ruleset (not the default branch) — see Verify-at-Implementation notes.

### The Deadlock Constraint (must encode in ACs)

A required status check whose workflow is skipped by workflow-level `paths`/`paths-ignore`
NEVER reports — branch protection waits forever. The correct pattern:

- Slow legs use **job-level `if:`** (not workflow-level path filters) so the workflow
  always starts and the aggregator job name always reports.
- `all-checks-pass` uses `if: always()` with a step that: (a) reads the outcome of every
  leg via `needs.<job>.result`, (b) treats `skipped` as PASS, (c) fails if any leg that
  DID run reported `failure` or `cancelled`.
- Branch protection must declare ONLY `CI / all-checks-pass` as required — never the slow
  legs themselves, or the deadlock recurs.

Reference: `.factory/planning/ci-speed-research.md` Q8 ("The deadlock (verified)").

### In-Repo Playbook

The companion document `docs/playbooks/tiered-ci-merge-queue.md` should be created as part
of this story. It serves as the portable, human-readable record of the tiered CI design so
future contributors understand why the job-level `if:` pattern is used and do not
accidentally reintroduce workflow-level path filters that recreate the deadlock.

## Behavioral Contracts

No product BCs govern CI workflow design. This story anchors to NFRs:

| NFR | Title | Covered ACs |
|-----|-------|-------------|
| NFR-026 | macOS arm64 binary builds and passes tests | AC-003 (full tier gate) |
| NFR-027 | macOS x86_64 binary builds and passes tests | NOT covered by ci.yml test matrix — `test (macos-x86_64)` leg was removed pre-story (chronic runner availability issues); Intel macOS binary coverage is provided by release.yml's native macos-13 build job, not by this story's ci.yml restructuring |
| NFR-028 | Linux x86_64 binary builds and passes tests | AC-001, AC-003 (fast tier) |
| NFR-029 | Linux arm64 binary builds and passes tests | AC-003 (full tier gate) |
| NFR-030 | Windows x86_64 binary builds and passes tests | AC-003 (full tier gate) |

Note: STORY-051 through STORY-054 (CI matrix stories, EPIC-19 Wave 1) are the primary
NFR-026 through NFR-030 anchors. This story is a CI OPTIMIZATION that maintains those
guarantees while restructuring when they are verified. The NFR-028 (Linux x86_64) gate
is preserved on every PR; NFR-026/029/030 are preserved in merge_group/nightly/develop.
NFR-027 (macOS x86_64) is NOT covered by ci.yml — see note above.

## Acceptance Criteria

### AC-001: Per-PR push runs only fast legs (traces to NFR-028)

When a PR push event triggers `ci.yml`, ONLY the following jobs execute:
`fmt`, `clippy`, `test (linux-x86_64)`, `doctest`, `check-panic-profile`,
`check-pdf-deps`, `supply-chain`. All other jobs (arm64, macos, windows, bench,
snapshots, visual-regression, msrv, docs, perf-smoke) MUST be in `skipped` state.

**Wall-clock framing:** Measure and record the fast-tier per-PR wall-clock. TARGET is
approximately 6-8 minutes (directional estimate from linux-x86_64 leg historical timing;
not yet measured for slideforge with the tiered configuration active). The AC passes if
the slow legs are successfully removed from the per-PR path AND the fast-tier wall-clock
is measured and recorded in the PR description — do NOT gate on a fixed ≤8 min threshold,
as that figure is a directional estimate, not a confirmed per-project measurement.

### AC-002: `all-checks-pass` correctly reports green on PR (traces to NFR-028)

When the fast legs from AC-001 complete successfully and slow legs are skipped, the
`CI / all-checks-pass` aggregator job MUST report `success` (not `skipped` or `failure`).
Branch protection configured to require only `CI / all-checks-pass` MUST allow the PR to
become mergeable. A PR with a FAILING fast leg MUST block the merge (all-checks-pass
reports `failure`). This confirms there is no deadlock: the aggregator always reports.

### AC-003: merge_group runs the full matrix (traces to NFR-026, NFR-029, NFR-030)
<!-- NFR-027 (macOS x86_64) removed: test (macos-x86_64) leg absent from ci.yml pre-story;
     Intel macOS binary coverage is release.yml's responsibility, not this story's. -->

When `github.event_name == 'merge_group'`, ALL jobs in the matrix MUST execute (no
`skipped` slow legs). The full-tier condition MUST cover these event/ref combinations:
- `merge_group` event (always full matrix)
- `push` to `refs/heads/develop` (full matrix)
- `push` to `refs/heads/main` (full matrix — so release merges to main get full validation; added to close ci-workflow-analyzer finding #5)
- `schedule` (nightly cron — full matrix)
- `workflow_dispatch` (manual full-matrix trigger via `gh workflow run`; added to close ci-workflow-analyzer finding #3 — without this, manual dispatch leaves all slow legs skipped)
- `pull_request` with label `full-ci` present on the PR (full matrix on demand)
The `all-checks-pass` aggregator MUST fail in merge_group if any leg reports `failure`.

### AC-004: No deadlock — only `all-checks-pass` is a required status check (traces to NFR-028)

Branch protection on `develop` MUST declare ONLY `CI / all-checks-pass` as required.
No individual slow leg (e.g., `test (linux-arm64)`, `test (windows-x86_64)`) MUST be
listed as a required status check. The workflow MUST NOT use workflow-level `paths` or
`paths-ignore` filters on any job that feeds `all-checks-pass`. These constraints MUST
be documented in `docs/playbooks/tiered-ci-merge-queue.md` with a rationale explaining
the deadlock pattern they prevent.

### AC-005: Nightly and develop push still run the full matrix (traces to NFR-026 through NFR-030)

A `push` event to the `develop` branch MUST trigger the full matrix (same as merge_group).
The nightly `schedule` trigger MUST trigger the full matrix. This ensures slow-leg
regressions surface within 24 hours even if no PR is raised against develop directly.

**Verify-at-implementation:** `schedule:` cron triggers ONLY run from the workflow file
on the DEFAULT branch (`main`), NOT from `develop`. A nightly leg added to `ci.yml` on
`develop` will NOT fire until merged to `main`. The implementer MUST account for this:
the nightly leg activates only once the story is merged to `main`, or an alternative
must be documented (e.g., a separate workflow triggered from `main` that references the
nightly intent). Note this constraint explicitly in `docs/playbooks/tiered-ci-merge-queue.md`.

### AC-006: `docs/playbooks/tiered-ci-merge-queue.md` created (traces to devops CLAUDE.md quality bar)

The file MUST exist and contain: (a) the tiered trigger design rationale, (b) the
job-level `if:` condition expression for the full-tier gate, (c) the deadlock anti-pattern
explanation (workflow-level `paths` on required-check workflows = deadlock), (d) the
branch protection instruction (only `CI / all-checks-pass` required), and (e) the
trade-off: platform-specific bugs surface at merge-queue time rather than at first PR push.
The playbook MUST also document the `schedule:` default-branch constraint (AC-005
verify-at-implementation note) and how the nightly leg activates.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| CI workflow (modified) | `.github/workflows/ci.yml` | Effectful (GitHub Actions DSL) |
| Merge queue config | GitHub repo settings — `drbothen/slideforge` branch protection on `develop` | External config (document in playbook) |
| Playbook doc (new) | `docs/playbooks/tiered-ci-merge-queue.md` | N/A (documentation) |

Architecture section files: N/A — no source crate changes.

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~3,800 |
| `.github/workflows/ci.yml` (full file, ~490 lines) | ~7,000 |
| ci-speed-research.md Q8 section | ~1,200 |
| `docs/playbooks/` (new file, small) | ~500 |
| PR + CI run output feedback | ~2,000 |
| **Total** | **~14,500** |

14,500 tokens is well within 20% of any reasonable agent context window. No split needed.

## Tasks

- [ ] Read `.github/workflows/ci.yml` in full to understand the current job structure,
      especially the `all-checks-pass` aggregator logic (ci.yml:447-489 per research)
- [ ] Identify all "slow" jobs: `test (linux-arm64)`, `test (macos-arm64)`,
      `test (windows-x86_64)`, `bench`, `snapshots`, `visual-regression`, `msrv`, `docs`, `perf-smoke`
      (Note: `test (macos-x86_64)` / macos-13 leg does NOT exist in ci.yml — removed pre-story)
- [ ] Add `merge_group:` to the `on:` block in `ci.yml`
- [ ] Add `schedule:` nightly cron if not already present (target: `0 6 * * *` UTC);
      note in playbook that this only fires from `main` (default branch), not `develop`
- [ ] Define the full-tier `if:` condition expression (canonical 6-combo gate — verbatim
      from ci.yml HEAD a0388bd4, applied at 7 job sites):
      ```yaml
      if: >
        github.event_name == 'merge_group' ||
        github.event_name == 'schedule' ||
        github.event_name == 'workflow_dispatch' ||
        (github.event_name == 'push' && (github.ref == 'refs/heads/develop' || github.ref == 'refs/heads/main')) ||
        (github.event_name == 'pull_request' &&
         contains(github.event.pull_request.labels.*.name, 'full-ci'))
      ```
- [ ] Apply the full-tier `if:` to ALL slow legs (see "slow" job list above)
- [ ] Verify the `all-checks-pass` aggregator already uses `if: always()` and treats
      `skipped` as pass; if not, update it to do so
- [ ] Verify `all-checks-pass` FAILS when any non-skipped job fails; test this logic
      manually by reading the current aggregator step and tracing the logic path
- [ ] Remove any workflow-level `paths:` or `paths-ignore:` filters that could prevent
      the workflow from starting (deadlock check)
- [ ] Create `docs/playbooks/tiered-ci-merge-queue.md` with sections per AC-006;
      include the `schedule:` default-branch constraint and how nightly activates
- [ ] **[VERIFY-AT-IMPL]** Enable GitHub merge queue on `develop` branch (non-default)
      for repo `drbothen/slideforge` (public, so merge queue is free). Confirm that
      `develop`'s branch protection ruleset (or legacy branch protection) supports
      `merge_group:` event. Document steps in playbook; devops-engineer performs this
      via GitHub settings or `gh api` CLI. NOTE: rulesets vs legacy branch protection
      have different skip semantics — confirm required-check skip behavior against
      current GitHub docs at implementation time.
- [ ] Update branch protection on `develop`: remove any individual slow-leg required
      checks; set ONLY `CI / all-checks-pass` as required (document steps in playbook)
- [ ] Measure fast-tier wall-clock on a PR push after change; record in PR description
      (see AC-001 framing — do not gate on a fixed threshold; record the actual measurement)
- [ ] Dry-run validate: push a test branch and confirm only fast legs trigger; open a
      draft PR and confirm `all-checks-pass` reports green with slow legs skipped
- [ ] `cargo fmt --all -- --check` + `cargo clippy --workspace -- -D warnings` still
      pass (no Rust code changed, but confirm CI workflow references the same commands)

## Previous Story Intelligence

STORY-051 (CI: fmt + clippy + nextest 5-platform matrix, Wave 1, EPIC-19) established
the current `ci.yml` structure. Key architectural decisions from STORY-051 that constrain
this story:

1. **`all-checks-pass` is the existing aggregator.** Per STORY-051 design, it was
   explicitly created as the single required status check. This story's job is to make
   that invariant robust against the tiered-trigger pattern.

2. **SHA-pinned actions.** All actions in `ci.yml` are pinned by full commit SHA per
   NFR-025 (supply chain pinning). This story must NOT introduce any new actions without
   SHAs (see "Pinned Versions / SHAs" section below). If adding `merge_group:` requires
   any new action, pin it.

3. **The `full-ci` label route.** The `full-ci` label approach allows developers to
   opt in to the full matrix on a specific PR without waiting for merge-queue. This was
   a deliberate design choice in the research sidecar (Q8). Implement it; document it
   in the playbook.

## Architecture Compliance Rules

Extracted from CLAUDE.md quality bar and ci-speed-research.md:

1. **No workflow-level `paths`/`paths-ignore` on required-check workflows.** Adding them
   to prevent the slow tier from running on certain file changes is FORBIDDEN — it
   recreates the deadlock. Use job-level `if:` exclusively for conditional execution.
2. **`all-checks-pass` MUST remain the single required status check.** Do not add any
   new required status checks to branch protection for the slow legs.
3. **`merge_group:` MUST be present in `on:` triggers.** Without it, the merge queue
   posts to a synthetic branch that never receives the required status check, stalling
   indefinitely.
4. **The aggregator MUST treat `skipped` as PASS** in the `if: always()` step. If the
   current implementation uses `result == 'success'`, change it to
   `result == 'success' || result == 'skipped'`.
5. **CARGO_INCREMENTAL=0 must remain.** This story does not change any Rust build flags.
   The existing `CARGO_INCREMENTAL=0` setting (ci.yml:21-22) is research-confirmed correct
   and must not be removed.

## Library and Framework Requirements

No Rust crate dependencies change in this story. All changes are GitHub Actions YAML.

| Component | Version/SHA | Notes |
|-----------|-------------|-------|
| `.github/workflows/ci.yml` | (existing) | Modify in-place |
| GitHub merge queue | GitHub-native feature — FREE for public repos (confirmed: `drbothen/slideforge` is public) | GA; no action required beyond `merge_group:` trigger + branch protection config |
| `actions/checkout` | (existing SHA in ci.yml) | No change |
| All other pinned actions | (existing SHAs in ci.yml) | No change |

## Pinned Versions / SHAs

No new GitHub Actions are added in this story. All existing action SHAs remain
unchanged. The `merge_group:` trigger is a GitHub Actions built-in event requiring
no additional action.

No uncertainty flags remain for this story. The merge queue semantics and `merge_group:`
event were verified with HIGH confidence in the research (Q8), and the repo public/free
status is confirmed (2026-06-10). The only residual items are runtime-configuration
steps documented as Verify-at-Implementation tasks above.

## File Structure Requirements

| Action | File | Change |
|--------|------|--------|
| Modify | `.github/workflows/ci.yml` | Add `merge_group:` trigger; add job-level `if:` to slow legs; verify aggregator skipped-as-pass logic; add nightly schedule if absent |
| Create | `docs/playbooks/tiered-ci-merge-queue.md` | New file: tiered CI design rationale + deadlock anti-pattern + branch protection instructions + `schedule:` default-branch constraint |
| No change | Any `*.rs` or `Cargo.toml` file | No Rust code changes |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | PR pushed with `full-ci` label already applied | Full matrix runs immediately (not just fast tier) |
| EC-002 | `full-ci` label added to PR while a run is in progress | The in-progress run finishes as fast-tier only (its event payload does not contain the label). Adding the label fires a FRESH workflow run immediately — but ONLY if `on.pull_request.types` includes `labeled`. Re-running an existing run (via GitHub's "Re-run jobs" button) reuses the ORIGINAL event payload; the labels array in that payload does NOT contain `full-ci` if it was added after the run started — so a re-run still executes fast-tier only and MUST NOT be relied on to pick up a newly added label. The correct mechanism is the `labeled` event itself triggering a new run. |
| EC-003 | `all-checks-pass` has a slow leg in its `needs:` that gets skipped | Aggregator treats the `skipped` result as pass; PR becomes mergeable |
| EC-004 | Developer adds a new slow job to ci.yml without adding the full-tier `if:` | The new job runs on every PR, breaking the tiered design. Mitigation: document the full-tier `if:` pattern in the playbook and add a comment block above the slow-job section |
| EC-005 | merge_group event runs and a slow leg fails | `all-checks-pass` reports `failure`; merge is blocked; developer sees the failure in the merge queue |
| EC-006 | Branch protection accidentally adds a slow leg as required | Deadlock: if that job is skipped on a PR push, the PR can never merge. Fix: remove it from required checks |
| EC-007 | Nightly cron run fails a slow leg | `all-checks-pass` reports `failure` on the nightly run; Slack/GitHub notification; dev investigates before next PR merge |
| EC-008 | Nightly `schedule:` trigger does not fire because `develop` is not the default branch | Expected: `schedule:` triggers only from `main`. Resolution: documented in playbook; nightly activates only after story merges to `main` or via an alternative mechanism |

## Forbidden Dependencies

- Do NOT add workflow-level `paths:` or `paths-ignore:` filters to ci.yml.
- Do NOT add any new required status checks to branch protection beyond `CI / all-checks-pass`.
- Do NOT use `sccache` or any additional caching action (see STORY-092 for the
  narrowly-scoped cache work; STORY-091 touches only triggers and job conditions).

## Trade-Off Documentation

**Platform-specific bugs surface at merge-queue time, not at first PR push.** Under the
tiered model, a Windows-only or arm64-only regression in a PR is NOT caught until that PR
enters the merge queue. The merge queue runs are typically fast (minutes between a PR
being queued and the full matrix completing) but they are later in the feedback loop than
a first-push run. This trade-off is acceptable because: (a) the full matrix still runs
before any commit lands on `develop`, (b) the nightly cron provides an additional safety
net, (c) the per-PR wall-clock win is the single largest available improvement.

Document this trade-off in `docs/playbooks/tiered-ci-merge-queue.md` so future
contributors understand it is a deliberate design decision.

## Rejected/Deferred Alternatives

The following alternatives were considered and explicitly rejected for this story.
Record these so they are not re-litigated without new information.

| Alternative | Decision | Reason |
|-------------|----------|--------|
| Workflow-level `paths-ignore` to skip slow legs on docs-only PRs | REJECTED | Deadlock: the required status check never reports on a skipped workflow → PR is permanently unmergeable. Job-level `if:` is the only safe mechanism. |
| Make each slow leg a separate required check and skip it via `paths-ignore` | REJECTED | Same deadlock as above; GitHub recommends aggregator + job-level `if:` pattern for exactly this reason. |
| Larger runners (8/16-core) to speed up slow legs instead of tiering | REJECTED | Never free on public repos; gated behind Team/Enterprise plans. Standard 4-vCPU arm64 runner is already the max free tier. (Research Q9, HIGH confidence.) |
| Switching from native arm64 to QEMU arm64 to reduce cost | REJECTED | Native arm64 (`ubuntu-24.04-arm`) is already in use and is FREE for public repos (confirmed: `drbothen/slideforge` is public). QEMU would be a regression. (Research CRITICAL PRECONDITION CORRECTION, HIGH confidence.) |
| sccache for compile caching | REJECTED | Shares the same ~10 GB GHA cache budget as rust-cache; rarely fires on warm `target/`; would worsen cache thrashing. (Handled in STORY-092 Rejected Alternatives.) |

## Test Strategy

This is a CI configuration story (`tdd_mode: facade`). There are no Rust unit tests.
Verification is via CI observation:

1. **AC-001 verification:** Push to a PR branch; observe GitHub Actions UI — confirm only
   fast legs trigger; confirm slow legs show as `skipped`. Record measured wall-clock.
2. **AC-002 verification:** Confirm `CI / all-checks-pass` shows `green` on the PR with
   skipped slow legs; confirm PR is mergeable.
3. **AC-003 verification:** Trigger via `full-ci` label OR wait for a develop push;
   confirm all legs trigger. Alternatively, use `act` locally to simulate `merge_group`.
4. **AC-004 verification:** Inspect GitHub branch protection settings; confirm only
   `CI / all-checks-pass` is listed as required.
5. **AC-005 verification:** Check the nightly schedule (may require waiting for next
   nightly run from `main`, or using `gh workflow run` to trigger manually).

## Complexity Estimate

5 story points. The YAML changes to `ci.yml` are targeted (add `merge_group:` to `on:`,
add `if:` to slow jobs, verify aggregator logic). The implementation risk is the aggregator
`skipped`-as-pass logic — if the existing implementation uses strict `success` checks,
it needs updating. Creating the playbook doc adds ~1 point of writing effort.

Estimated 1 day including the CI observation verification cycle and branch protection
configuration.

## Uncertainty Resolution Log

### Resolved (2026-06-10, remove-uncertainty pass)

| Item | Resolution |
|------|-----------|
| Repo owner placeholder (`<owner>`) | Confirmed: repo is `drbothen/slideforge`, PUBLIC. Replace all `<owner>` placeholders with `drbothen`. |
| GitHub merge queue cost for public repos | Confirmed: FREE for public repos. `drbothen/slideforge` is public. No Team/Enterprise plan required. |
| `merge_group:` event GA status | Confirmed HIGH confidence per research Q8 (verified). No further confirmation needed. |
| Fast-tier wall-clock "≤ 8 minutes" fixed threshold | Reframed: directional target ~6-8 min; AC now requires measurement + recording, not a fixed gate. The figure was a directional estimate not yet measured for slideforge specifically. |

### Deferred to Implementation (verify-at-implementation markers)

| Item | Resolution Command / Check |
|------|---------------------------|
| Merge queue enabled on `develop` branch protection (non-default branch) | Implementer must configure via GitHub Settings > Branches > `develop` ruleset (or legacy branch protection) for `drbothen/slideforge`. Confirm rulesets vs legacy branch protection skip semantics against current GitHub docs at implementation time. |
| `schedule:` trigger fires only from default branch (`main`) | Implementer must note: nightly leg in `ci.yml` only activates after this story merges to `main`. The nightly leg added to `develop` will NOT fire until merged. Document resolution in playbook (e.g., nightly activates post-merge-to-main, or use an alternative). |
| Required-check skip semantics (rulesets vs legacy branch protection) | Confirm the exact behavior when a required check is `skipped` under current GitHub branch protection/ruleset docs. Rulesets and legacy branch protection have subtly different handling. |

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-06-10 | story-writer | Initial creation per human direction 2026-06-10. Grounded in ci-speed-research.md Q8 (merge queue + tiered triggers, HIGH confidence). Highest-priority CI story; blocks STORY-092 and STORY-093. |
| 1.1 | 2026-06-10 | story-writer | remove-uncertainty pass: pinned repo as `drbothen/slideforge` PUBLIC (free merge queue confirmed); reframed AC-001 wall-clock from fixed ≤8min gate to measure-to-confirm; added `schedule:` default-branch constraint to AC-005 and playbook task; added verify-at-impl markers for merge-queue-on-develop config and required-check skip semantics; added Uncertainty Resolution Log. |
| 1.2 | 2026-06-10 | story-writer | EC-002 corrected (source: ci-workflow-analyzer review finding #2, 2026-06-10; fixed at implementation a0388bd4). Previous text claimed "the NEXT push or re-run triggers full matrix" — both claims were wrong. Correct behavior: (1) `on.pull_request.types` MUST include `labeled`; the `labeled` event itself fires a fresh full-tier run immediately when the label is applied. (2) Re-running an existing run via GitHub's "Re-run jobs" button reuses the original event payload, which does NOT contain the label if it was added after the run started — re-run does NOT pick up newly added labels and MUST NOT be relied on. Companion playbook (`tiered-ci-merge-queue.md`) corrected identically in Steps 7 and 9. |
| 1.3 | 2026-06-10 | story-writer | AC-003 and Tasks `if:` expression aligned to 6-combo full-tier gate (F-091-P4-001). Two combos added: (1) `workflow_dispatch` — closes ci-workflow-analyzer finding #3; required so `gh workflow run` does not leave all slow legs skipped. (2) `push` to `refs/heads/main` — closes ci-workflow-analyzer finding #5; ensures release merges to main receive full matrix validation. Summary prose (§ Summary, item 2) updated to name all 6 triggers. Canonical expression is byte-identical to ci.yml HEAD a0388bd4 lines 294-300, applied at 7 job sites. |
| 1.4 | 2026-06-10 | story-writer | Reconciled to actual 4-platform ci.yml test matrix (F-091-P5-001). `test (macos-x86_64)` / macos-13 leg was removed from ci.yml before this story (chronic runner availability issues); Intel macOS binary coverage is release.yml's responsibility. Changed: (1) Summary §1 "5-platform" → "4-platform (1 fast + 3 slow legs)"; (2) Summary §2 full-tier list removed `test (macos-x86_64)`, added clarifying note; (3) NFR table NFR-027 row corrected — AC-003 coverage claim removed, binary coverage path documented as release.yml; (4) AC-003 header trace removed NFR-027 with inline comment; (5) Tasks slow-job list removed `test (macos-x86_64)` with note. |
