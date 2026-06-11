# Playbook: Tiered CI + GitHub Merge Queue (slideforge)

**Repo:** `github.com/drbothen/slideforge` (PUBLIC)
**Introduced:** STORY-091 (Wave 5, EPIC-19)
**Status:** active
**References:** `.factory/planning/ci-speed-research.md` (Q8, HIGH confidence)

---

## Summary

This playbook documents the tiered CI trigger design and GitHub merge queue
configuration for slideforge. It exists so future contributors understand:

1. Why slow legs do not run on every PR push
2. The exact job-level `if:` expression that gates them
3. The deadlock anti-pattern that must never be reintroduced
4. How to configure branch protection correctly
5. The trade-offs of this design
6. The `schedule:` default-branch constraint

---

## 1. Tiered trigger design rationale

Every PR push previously triggered the full 4-platform matrix. The `test
(linux-arm64)` leg on `ubuntu-24.04-arm` (a native 4-vCPU arm64 runner, not
QEMU emulation) takes approximately 22 minutes for a cold workspace build of the
20-crate slideforge workspace. This means every developer iteration waits ~22
minutes even though the primary feedback signal (linux-x86_64) is available in
~6-8 minutes.

The tiered model removes slow legs from the per-PR critical path:

**Fast tier** — runs on every PR push and every trigger:
- `fmt` (format check)
- `clippy` (pedantic lint)
- `test (linux-x86_64)` (primary NFR-028 test leg)
- `doctest` (standalone doctest runner)
- `check-panic-profile` (panic=unwind safety perimeter)
- `check-pdf-deps` (PDF dep perimeter)
- `supply-chain` (cargo audit + cargo deny)

**Slow tier** — runs only on the full-tier condition (see section 2):
- `test (linux-arm64)` (NFR-029)
- `test (macos-arm64)` (NFR-026)
- `test (windows-x86_64)` (NFR-030)
- `msrv` (MSRV check)
- `docs` (rustdoc build)
- `snapshots` (cargo-insta snapshot check)
- `bench` (criterion benchmark gate, NFR-001/002)
- `visual-regression` (LibreOffice rendering + SSIM/PSNR check)
- `perf-smoke` (data-source performance gate, NFR-036/037)

The full matrix still runs before anything merges. The default branch (`develop`,
and ultimately `main`) remains fully protected. Per-PR developer iteration speed
improves; merge safety does not regress.

---

## 2. Full-tier `if:` condition (exact expression)

Every slow-tier job in `.github/workflows/ci.yml` carries this exact `if:`
expression at the job level:

```yaml
if: >
  github.event_name == 'merge_group' ||
  github.event_name == 'schedule' ||
  (github.event_name == 'push' && github.ref == 'refs/heads/develop') ||
  (github.event_name == 'pull_request' &&
   contains(github.event.pull_request.labels.*.name, 'full-ci'))
```

The four conditions that trigger the full tier:

| Condition | When it fires | Why |
|-----------|---------------|-----|
| `merge_group` | PR enters the merge queue | Full matrix validates before landing on `develop` |
| `schedule` | Nightly cron (06:00 UTC) | Catch platform-specific regressions within 24 hours |
| `push` to `refs/heads/develop` | Direct push or squash-merge completes | Redundant safety net on the protected branch |
| `pull_request` with `full-ci` label | Developer opts in | On-demand full matrix for cross-platform investigation |

**Maintainer rule:** when adding a new slow job to `ci.yml`, copy this `if:`
expression verbatim to that job. Do NOT invent a variation — inconsistency
between slow-job conditions leads to subtle coverage gaps.

---

## 3. Deadlock anti-pattern — do not reintroduce

> This is the single most common CI misconfiguration. Read before touching
> branch protection or adding `paths`/`paths-ignore` to ci.yml.

### What causes the deadlock

GitHub branch protection waits for a required status check to be reported. If the
workflow or job is skipped entirely, the check is never reported. Branch
protection then waits indefinitely — every merge attempt hangs permanently.

This deadlock occurs when:

1. **Workflow-level `paths` or `paths-ignore`** on a workflow that contains a
   required check: if a PR touches only the filtered-out files, the entire
   workflow is skipped, the check is never reported, and the PR can never merge.

2. **Individual slow job as a required status check**: if that job uses a
   job-level `if:` condition that evaluates to false on normal PRs, the job is
   `skipped`. GitHub branch protection does NOT accept `skipped` for an
   individual required check — only `success` satisfies it. The PR is
   permanently blocked.

### How this design prevents the deadlock

- Slow jobs use **job-level `if:`** only. The workflow always starts, the
  `all-checks-pass` aggregator always runs, and its result is always reported.
- `all-checks-pass` uses `if: always()` — it runs unconditionally.
- `all-checks-pass` treats `skipped` as passing — only `failure` or `cancelled`
  from a job that actually ran causes it to fail.
- **Only `all-checks-pass` is a required status check.** The aggregator is never
  skipped, so branch protection always receives a report.

### Forbidden patterns

These patterns MUST NOT be added to ci.yml:

```yaml
# FORBIDDEN: workflow-level path filter on a workflow producing required checks
on:
  pull_request:
    paths-ignore:
      - '**.md'

# FORBIDDEN: individual slow job as a required status check
# (in GitHub branch protection settings — not in ci.yml itself)
# Required status checks: "CI / test (linux-arm64)"   ← DEADLOCK on normal PRs
```

### Safe pattern (correct)

```
branch protection requires: "CI / all-checks-pass"   ← single required check
aggregator: needs all jobs, if: always(), skip == OK
slow jobs:  job-level if: (merge_group || schedule || push-to-develop || full-ci-label)
```

---

## 4. Branch protection instruction

The branch protection rule on `develop` MUST be configured as follows:

**Required status checks:**
- `CI / all-checks-pass` — the aggregator job (the ONLY required check)

**Do NOT add:**
- `CI / test (linux-arm64)` — would deadlock on normal PR pushes
- `CI / test (macos-arm64)` — same
- `CI / test (windows-x86_64)` — same
- Any other individual slow-tier job name

**Merge queue:** enable the merge queue on `develop`'s branch protection rule.
This causes the `merge_group` event to fire when a PR is queued, triggering the
full matrix before the merge commit lands on `develop`.

### Enabling merge queue and updating required checks (gh CLI)

These commands must be run by a repository admin after this story's PR merges to
`develop`:

```bash
# 1. Update required status checks to ONLY all-checks-pass.
#    Replace the entire required_status_checks list — do not ADD to it.
gh api repos/drbothen/slideforge/branches/develop/protection \
  -X PUT \
  --input - <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["CI / all-checks-pass"]
  },
  "required_pull_request_reviews": {
    "required_approving_review_count": 0
  },
  "enforce_admins": false,
  "restrictions": null
}
EOF

# 2. Enable merge queue.
#    GitHub's merge queue is configured via repository rulesets in the UI
#    (Settings → Rules → Rulesets) or via the REST API for rulesets.
#    For legacy branch protection (the gh api command above), merge queue is
#    enabled separately in Settings → General → Pull Requests → Merge queue.
#
#    drbothen/slideforge is a PUBLIC repo — merge queue is FREE.
#    No Team or Enterprise plan is required.
#
# 3. Verify:
gh api repos/drbothen/slideforge/branches/develop/protection \
  | python3 -m json.tool | grep -A 5 "required_status_checks"
# Expected: only "CI / all-checks-pass" in the contexts list.
```

**Note on rulesets vs legacy branch protection:**
GitHub offers two branch protection mechanisms: legacy branch protection (used
above) and repository rulesets (newer). The merge queue feature may behave
slightly differently between the two. At implementation time (2026-06-10),
confirm that your chosen mechanism supports the `merge_group` event and that
`skipped` results satisfy the required-check gate for `all-checks-pass`. If
using rulesets, the required check name format and skip-semantics may differ —
check current GitHub documentation.

---

## 5. Trade-off: platform-specific bugs surface at merge-queue time

Under the tiered model, a bug that manifests only on macOS, Windows, or arm64
is NOT caught until the PR enters the merge queue (or the nightly run catches it
first). This is an intentional trade-off:

**Why it is acceptable:**
- The full matrix still runs before any commit lands on `develop`. No regression
  slips through to the default branch.
- The nightly cron provides a safety net: platform-specific regressions are
  caught within ~24 hours even if no PR is queued.
- The `full-ci` label gives any developer an escape hatch: apply it to trigger
  the full matrix on a specific PR before queuing.
- The merge queue run itself blocks the merge on failure — a Windows-only bug
  seen at merge time prevents the merge, not silently.

**The cost:**
- A developer may iterate through several fast-tier PR pushes before a
  platform-specific failure is discovered at merge-queue time. The merge queue
  provides a clear failure signal, but the feedback loop is longer than if the
  slow leg ran on every push.
- The `full-ci` label mitigates this for PRs where cross-platform correctness
  is expected to be risky (e.g., PRs touching platform-specific paths, OOXML
  generation, or binary encoding).

---

## 6. `schedule:` default-branch constraint

**GitHub Actions only fires `schedule:` triggers from the workflow file on the
DEFAULT branch.** The default branch for `drbothen/slideforge` is `main` (not
`develop`).

Consequences:
- The nightly cron defined in `ci.yml` (`0 6 * * *`) does NOT fire while
  `ci.yml` is only on `develop`.
- The nightly leg becomes active once this story's changes are merged from
  `develop` to `main` (the normal release flow).
- Until then, nightly runs can be triggered manually via:
  ```bash
  gh workflow run ci.yml --repo drbothen/slideforge --ref main
  ```
  (This only works once `ci.yml` with the `schedule:` block is on `main`.)

**Developer action required:** after the develop → main release merge, verify
that the nightly run fires by checking the Actions tab the morning after the
merge. If the nightly does not fire, confirm the `schedule:` cron syntax is
present in the `main` branch's `ci.yml`.

---

## 7. Adding a new CI job — maintainer checklist

When adding a new job to `.github/workflows/ci.yml`:

**Is it a FAST-tier job?**
- Runs in ≤ 10 minutes on a cold ubuntu-latest runner
- Platform-independent (or linux-x86_64 is sufficient)
- Provides immediate developer feedback value
- Add it without any `if:` condition at the job level

**Is it a SLOW-tier job?**
- Copy the exact full-tier `if:` expression from section 2 verbatim
- Add the job name to `all-checks-pass`'s `needs:` list
- Do NOT add it to branch protection required checks
- Document in a comment above the job why it is slow-tier

**In both cases:**
- Add the job name to `all-checks-pass`'s `needs:` list (TD-VSDD-060: sweep all
  `needs:` references when changing job names)
- Confirm no workflow-level `paths`/`paths-ignore` was added to ci.yml

---

## 8. Edge cases reference

| Scenario | Behavior |
|----------|----------|
| PR pushed without `full-ci` label | Fast tier only; slow jobs `skipped`; aggregator reports `success` |
| PR pushed with `full-ci` label | Full tier runs immediately (not just fast) |
| `full-ci` label added after a run starts | Current run finishes with fast tier; NEXT push or re-run triggers full tier |
| Merge queue entry | Full tier runs via `merge_group` event; aggregator fails if any leg fails |
| Nightly cron (from `main`) | Full tier runs via `schedule` event; Slack/GitHub notification on failure |
| Nightly cron (before merged to `main`) | Does NOT fire (schedule only runs from default branch); expected behavior |
| New slow job added without full-tier `if:` | Runs on every PR — breaks tiered design; see maintainer checklist above |
| Branch protection accidentally adds a slow leg | Deadlock; fix: remove it from required checks immediately |
