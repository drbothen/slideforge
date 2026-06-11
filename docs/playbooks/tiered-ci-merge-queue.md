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
  github.event_name == 'workflow_dispatch' ||
  (github.event_name == 'push' && (github.ref == 'refs/heads/develop' || github.ref == 'refs/heads/main')) ||
  (github.event_name == 'pull_request' &&
   contains(github.event.pull_request.labels.*.name, 'full-ci'))
```

The five conditions that trigger the full tier:

| Condition | When it fires | Why |
|-----------|---------------|-----|
| `merge_group` | PR enters the merge queue | Full matrix validates before landing on `develop` |
| `schedule` | Nightly cron (06:00 UTC) | Catch toolchain/environment drift on `main` within 24 hours |
| `workflow_dispatch` | Manual trigger via gh CLI or Actions UI | Ad-hoc full-matrix run without needing to push a commit |
| `push` to `refs/heads/develop` OR `refs/heads/main` | Direct push or squash-merge completes | Safety net on the protected branches |
| `pull_request` with `full-ci` label | Developer opts in | On-demand full matrix for cross-platform investigation |

**Maintainer rule:** when adding a new slow job to `ci.yml`, copy this `if:`
expression verbatim from an existing slow job. Do NOT invent a variation, and do
NOT copy from this playbook comment — transcription drift between the playbook
and the YAML is the failure mode. The YAML copy is canonical.

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
branch protection requires: "all-checks-pass"   ← single required check (API context name)
                                                   PR UI displays as "CI / all-checks-pass"
aggregator: needs all jobs, if: always(), skip == OK
slow jobs:  job-level if: (merge_group || schedule || workflow_dispatch || push-to-develop/main || full-ci-label)
```

### Repo-wide invariant: companion workflows must never be required checks

`pdf-ua1.yml`, `html-wcag.yml`, and `security.yml` use `pull_request.paths:`
filters and do NOT have `merge_group:` triggers. This means:

- On PRs that touch filtered-out files, the entire workflow is skipped.
- On merge-queue entries, the workflow never fires (no `merge_group:` trigger).

**Consequence:** these workflows MUST NEVER be added as required status checks.
Doing so deadlocks every PR whose changed files do not match the path filter, and
deadlocks every merge-queue entry permanently.

If a future decision requires any of these checks to be mandatory:
1. Remove the `paths:` filter from the workflow, OR restructure to the
   job-level `if:` pattern (no workflow-level path filtering).
2. Add `merge_group:` to the workflow's `on:` triggers.
3. Add the job to the `all-checks-pass` aggregator's `needs:` list (the
   aggregator is already the single required check — no change to branch
   protection is needed).

Until those steps are taken, these workflows operate as informational-only
checks. Their failure does not block PRs or merge-queue entries.

---

## 4. Branch protection instruction

The branch protection rule on `develop` MUST be configured as follows:

**Required status checks:**
- `all-checks-pass` — the aggregator job (the ONLY required check)

**IMPORTANT — context name vs. PR UI display:**
The GitHub branch-protection API uses the bare job name as the context string.
The PR check-list displays it as "CI / all-checks-pass" (workflow name / job
name), but the `contexts` field in the PUT payload and the verify output use
`all-checks-pass` alone. Using `"CI / all-checks-pass"` in the API payload will
create a check that never matches the actual reported context — PRs will be
permanently blocked.

To confirm the exact context string reported for a real PR, run:
```bash
gh api repos/drbothen/slideforge/commits/<head-sha>/check-runs \
  --jq '.check_runs[].name'
```
Copy the name exactly as returned. For this workflow the returned name is
`all-checks-pass`.

**Do NOT add:**
- `test (linux-arm64)` — would deadlock on normal PR pushes
- `test (macos-arm64)` — same
- `test (windows-x86_64)` — same
- Any other individual slow-tier job name

**Merge queue:** enable the merge queue via the develop branch protection rule's
"Require merge queue" option (Settings → Branches → Edit rule → "Require merge
queue"). This causes the `merge_group` event to fire when a PR is queued,
triggering the full matrix before the merge commit lands on `develop`. Do NOT
look for a merge-queue toggle in Settings → General — it lives inside the branch
protection rule itself (or in a branch ruleset if using the newer ruleset UI).

### Establishing branch protection for the first time (gh CLI)

**Pre-condition:** `drbothen/slideforge` currently has NO branch protection and
NO rulesets on `develop` (verified 2026-06-10). The procedure below CREATES
protection fresh — it does not modify existing protection.

**Safe ordering:**
1. Merge this story's PR to `develop` first. Nothing currently blocks it — no
   protection exists yet, so there is no deadlock window during migration.
2. After the merge completes, run the commands below to establish protection.

> **WARNING — Security review note (SEC-002 / CWE-284):**
> The payload below sets `required_approving_review_count: 0` and
> `enforce_admins: false`. These are **intentional** for the slideforge project's
> operating model and are justified by the VSDD factory multi-agent review
> pipeline: every PR passes a local adversary 3-CLEAN convergence protocol,
> a dedicated security-reviewer agent scan, and a pr-reviewer final-eyes pass
> before the orchestrator merges under standing human-granted authorisation.
> Review rigour is enforced by the pipeline (documented in CLAUDE.md and
> STATE.md), not by GitHub-native approval gates. `enforce_admins: false`
> preserves the explicit per-request human escape hatch for emergency overrides.
>
> **Any team adopting this playbook WITHOUT an equivalent automated multi-review
> pipeline MUST change these values before applying branch protection:**
> - `required_approving_review_count` → **1 or higher** (human review required)
> - `enforce_admins` → **true** (no admin bypass without deliberate override)
>
> Applying this payload as-is on a repo that lacks the factory pipeline is a
> security misconfiguration that leaves the default branch unprotected from
> direct force-pushes and unapproved merges.

```bash
# Step 1 — Create branch protection with all-checks-pass as the ONLY required
#           status check.
#           NOTE: contexts value is the bare job name, NOT "CI / all-checks-pass".
gh api repos/drbothen/slideforge/branches/develop/protection \
  -X PUT \
  --input - <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["all-checks-pass"]
  },
  "required_pull_request_reviews": {
    "required_approving_review_count": 0
  },
  "enforce_admins": false,
  "restrictions": null
}
EOF

# Step 2 — Enable merge queue.
#   The merge queue is enabled inside the branch protection rule via the UI:
#   Settings → Branches → Edit rule for 'develop' → check "Require merge queue".
#   drbothen/slideforge is a PUBLIC repo — merge queue is FREE.
#   No Team or Enterprise plan is required.

# Step 3 — Verify required checks (context string must be 'all-checks-pass').
gh api repos/drbothen/slideforge/branches/develop/protection \
  | python3 -m json.tool | grep -A 5 "required_status_checks"
# Expected output contains: "all-checks-pass" (NOT "CI / all-checks-pass").

# Step 4 — Cross-check by reading the actual check-run name from a merged PR:
gh api repos/drbothen/slideforge/commits/<head-sha-of-merged-pr>/check-runs \
  --jq '.check_runs[].name'
# Copy the exact string. It must match what you put in contexts above.
```

**Warning for repos that ALREADY have per-leg required checks:**
If `develop` has existing required status checks pointing at individual slow-tier
jobs (e.g. `test (linux-arm64)`), those checks deadlock on every normal PR push
because skipped jobs are never reported as satisfied. Fix procedure:
1. GET the current protection: `gh api repos/drbothen/slideforge/branches/develop/protection`
2. Remove all individual leg entries; keep only `all-checks-pass`.
3. Use PATCH on `required_status_checks` (not PUT on the full protection object)
   to avoid accidentally resetting review/admin/restriction settings:
   ```bash
   gh api repos/drbothen/slideforge/branches/develop/protection/required_status_checks \
     -X PATCH --input - <<'EOF'
   {"strict": true, "contexts": ["all-checks-pass"]}
   EOF
   ```
4. Re-enable merge queue if it was disrupted.

**Note on rulesets vs legacy branch protection:**
GitHub offers two branch protection mechanisms: legacy branch protection (used
above) and repository rulesets (newer). Either can work. If using rulesets,
confirm that: (a) the required check name in the ruleset matches the bare job
name `all-checks-pass`; (b) the ruleset `merge_group` trigger is enabled; and
(c) `skipped` results satisfy the ruleset's required-check gate. Check current
GitHub documentation at time of setup — ruleset semantics are evolving.

---

## 5. Trade-off: platform-specific bugs surface at merge-queue time

Under the tiered model, a bug that manifests only on macOS, Windows, or arm64
is NOT caught until the PR enters the merge queue (or the nightly run catches it
first). This is an intentional trade-off:

**Why it is acceptable:**
- The full matrix still runs before any commit lands on `develop`. No regression
  slips through to the default branch.
- Every squash-merge to `develop` triggers a push-to-develop run with the full
  matrix — `develop` is always validated end-to-end after each story lands.
- The nightly cron (firing from `main`) validates `main` against
  environmental and toolchain drift — new compiler warnings, dependency
  yanks, runner image updates — independent of whether any PR is queued.
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

## 6. `schedule:` default-branch constraint and manual dispatch

**GitHub Actions only fires `schedule:` triggers from the workflow file on the
DEFAULT branch.** The default branch for `drbothen/slideforge` is `main` (not
`develop`).

Consequences:
- The nightly cron defined in `ci.yml` (`0 6 * * *`) does NOT fire while
  `ci.yml` is only on `develop`.
- The nightly leg becomes active once this story's changes are merged from
  `develop` to `main` (the normal release flow).

**Manual full-matrix trigger (workflow_dispatch):**

`ci.yml` exposes `workflow_dispatch:` so any admin can fire the full matrix
without pushing a commit. The `workflow_dispatch` event is included in the
full-tier `if:` condition, so all slow jobs run when it fires.

```bash
# Trigger a manual full-matrix run on develop (or any ref):
gh workflow run ci.yml --repo drbothen/slideforge --ref develop

# Trigger on main (to replicate what nightly does):
gh workflow run ci.yml --repo drbothen/slideforge --ref main

# Watch the run complete:
gh run watch --repo drbothen/slideforge
```

This replaces the previous workaround of triggering the workflow on `main`
before it was merged. You can now fire the full matrix on `develop` directly.

**Manual dispatch target guidance (SEC-003):**
`workflow_dispatch` does not support a `branches:` filter — it always accepts
any ref the caller provides, including unreviewed feature branches and arbitrary
SHAs. To avoid triggering unnecessary cost and CI exposure on unreviewed code:

- **Target `develop` or `main` only** for routine full-matrix validation.
  Dispatching on a feature branch before the PR adversarial/security review
  cycle completes risks wasting slow-tier runner time (including the LibreOffice
  install in `visual-regression` and the full arm64/macOS/Windows matrix) on
  code that has not yet passed the factory review gates.
- **Use the `full-ci` label instead** for per-PR full-matrix runs. The label
  trigger fires via `pull_request.labeled` and targets only the commits already
  in the PR, which have passed the fast tier. This is the preferred path for
  cross-platform investigation on feature branches.
- If you must dispatch on a non-develop/main ref (e.g., to diagnose a
  platform-specific failure before merge), use:
  ```bash
  gh workflow run ci.yml --repo drbothen/slideforge --ref feature/STORY-NNN
  ```
  and be aware that all slow-tier jobs — including `visual-regression`
  (LibreOffice install ~2min) and `test-matrix-slow` (up to 75min per
  platform) — will run on that unreviewed branch.

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
| PR pushed with `full-ci` label | Full tier runs immediately because `on.pull_request.types` includes `labeled` |
| `full-ci` label added to a PR that already has a run in progress | The `labeled` event fires a fresh run with the label in the payload; full tier runs. WARNING: manually RE-RUNNING an existing run does NOT pick up the newly-added label — the re-run replays the original event payload which does not include the label. Only the fresh run triggered by the `labeled` event (or a subsequent push) carries the label. |
| `full-ci` label added, then removed, then re-added | Each `labeled` event triggers a fresh full-tier run |
| Merge queue entry | Full tier runs via `merge_group` event; aggregator fails if any leg fails |
| Nightly cron (from `main`) | Full tier runs via `schedule` event; validates `main` against toolchain/environment drift |
| Nightly cron (before merged to `main`) | Does NOT fire (schedule only runs from default branch); expected behavior |
| Manual dispatch (`workflow_dispatch`) | Full tier runs on the specified ref; see §6 |
| Squash-merge to `develop` | Push-to-develop fires; full tier runs; `develop` is validated end-to-end |
| New slow job added without full-tier `if:` | Runs on every PR — breaks tiered design; see maintainer checklist above |
| Branch protection accidentally adds a slow leg as required | Deadlock; fix: remove it from required checks immediately; use PATCH not PUT to avoid resetting other protection settings |
| Branch protection `contexts` set to `"CI / all-checks-pass"` | Never satisfies — the check is never reported under that name; fix: use `"all-checks-pass"` (bare job name) |

---

## 9. Cache budget (STORY-092)

**Introduced:** STORY-092 (Wave 5, EPIC-19)
**Baseline (2026-06-10, pre-fix):** 9.77 GB / 23 active caches = 97.7% of the ~10 GB per-repo limit.
`v0-rust-test-linux-arm64-*` was ABSENT (LRU-evicted) — confirmed root cause of
arm64 cold-build flakiness.

**Measured (2026-06-11, post-STORY-091 merge, pre-STORY-092 merge):**
`gh api repos/drbothen/slideforge/actions/cache/usage` → 9.66 GB / 22 active caches.
The arm64 test cache remains absent — the budget is still critically high.

### 9.1 Active cache keys and estimated sizes

The table below lists every `shared-key` value currently in `ci.yml`, its workflow source, and the
measured or estimated per-key `target/` size from `gh cache list` output (2026-06-11).

| shared-key | workflow | platform | measured size | notes |
|---|---|---|---|---|
| `clippy` | ci.yml | linux-x86_64 | ~331 MiB | |
| `test-linux-x86_64` | ci.yml | linux-x86_64 | ~772 MiB | |
| `doctest` | ci.yml | linux-x86_64 | ~772 MiB | shares Cargo.lock hash with test-linux-x86_64 — consolidation candidate |
| `supply-chain` | ci.yml | linux-x86_64 | ~98 MiB | small (audit tools only) |
| `test-linux-arm64` | ci.yml | linux-arm64 | 0 (evicted) | absent from cache — primary reliability issue |
| `test-macos-arm64` | ci.yml | macos-arm64 | ~698 MiB | |
| `test-windows-x86_64` | ci.yml | windows-x86_64 | ~759 MiB | |
| `msrv` | ci.yml | linux-x86_64 | ~292 MiB | |
| `docs` | ci.yml | linux-x86_64 | ~276 MiB | |
| `snapshots` | ci.yml | linux-x86_64 | ~744 MiB | |
| `bench` | ci.yml | linux-x86_64 | ~620 MiB | |
| `visual-regression` | ci.yml | linux-x86_64 | ~99 MiB | |
| `perf-smoke` | ci.yml | linux-x86_64 | ~229 MiB | |
| `security-audit` | security.yml | linux-x86_64 | ~98 MiB | |
| `pdf-ua1-structure` | pdf-ua1.yml | linux-x86_64 | ~98 MiB (est.) | |
| `pdf-ua1-verapdf` | pdf-ua1.yml | linux-x86_64 | ~98 MiB (est.) | |
| `html-wcag` | html-wcag.yml | linux-x86_64 | ~98 MiB (est.) | |
| `html-wcag-axe` | html-wcag.yml | linux-x86_64 | ~98 MiB (est.) | |
| `release-verify-*` | release.yml | multi-platform | ~98 MiB (est.) | only on tag push |
| `release-build-*` | release.yml | multi-platform | ~700 MiB (est.) | only on tag push |
| `sbom` | release.yml | linux-x86_64 | ~98 MiB (est.) | only on tag push |

**Active (non-release) cache total estimate: ~6.1 GiB** (excluding release-only keys that only appear on tag push).
**With arm64 cache restored: ~7.0 GiB** — within the ~8 GiB target budget with ~1 GiB headroom.

### 9.2 STORY-092 fixes that reduce budget pressure

1. **rust-cache v2.9.1 hash fix** — eliminates spurious cache misses that caused duplicate
   cache entries with different lockfile-hash suffixes. The 2026-06-11 list shows multiple
   stale entries per shared-key (e.g., two `clippy` entries, two `bench` entries from different
   Cargo.lock hashes). After the SHA bump, rust-cache computes keys correctly and stops
   writing redundant entries.

2. **`cache-on-failure: "true"` on all 11 jobs** — ensures the arm64 test cache is saved
   even when the job times out. Before this fix, a timeout on the cold arm64 build caused
   the cache to not be written, perpetuating the cold-build loop on the next run.

3. **No shared-key consolidation required** — the active budget is ~6.1 GiB without arm64
   and ~7.0 GiB with it. This fits within the ~8 GiB target. The `doctest` key shares
   the same feature flags and target triple as `test-linux-x86_64` and is a future
   consolidation candidate, but the current budget does not require it.

### 9.3 Verification

After STORY-092 merges to develop and a subsequent develop push triggers a full matrix run:

```bash
# Check total budget:
gh api repos/drbothen/slideforge/actions/cache/usage

# Verify arm64 cache is present:
gh cache list --repo drbothen/slideforge --limit 100 | grep arm64

# Expected: v0-rust-test-linux-arm64-* present with a recent created_at timestamp.
# Expected: total active_caches_size_in_bytes < ~8 GiB (~8,589,934,592 bytes).
```

### 9.4 Jobs with `cache-on-failure: "true"` (post-STORY-092)

All 11 jobs that compile Rust code now have `cache-on-failure: "true"`:

| job name | tier | had it before STORY-092 | added by STORY-092 |
|---|---|---|---|
| `test (linux-x86_64)` | fast | yes (PR #79) | — |
| `test-matrix-slow` (all 3 legs) | slow | yes (PR #79) | — |
| `bench` | slow | yes (PR #81) | — |
| `clippy` | fast | no | yes |
| `doctest` | fast | no | yes |
| `supply-chain` | fast | no | yes |
| `msrv` | slow | no | yes |
| `docs` | slow | no | yes |
| `snapshots` | slow | no | yes |
| `visual-regression` | slow | no | yes |
| `perf-smoke` | slow | no | yes |
