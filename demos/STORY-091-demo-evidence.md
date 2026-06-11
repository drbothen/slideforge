# Demo Evidence: STORY-091 — CI Tiered Triggers + Merge Queue

**Story:** STORY-091 (Wave 5, EPIC-19)
**Spec version:** 1.4
**Evidence type:** Structural verification (tdd_mode: facade — no Rust unit tests; CI-workflow story)
**ci.yml HEAD:** `30edf4b2` (STORY-091 worktree at time of evidence capture)
**Playbook:** `docs/playbooks/tiered-ci-merge-queue.md`
**Evidence captured:** 2026-06-10
**Overall validity:** actionlint exit 0 — see §Overall section below

---

## AC-001: Per-PR push runs only fast legs (traces to NFR-028)

**Claim:** Exactly 7 ungated (fast-tier) jobs and 7 gated (slow-tier) jobs are defined in
ci.yml. On a normal PR push, only the 7 fast jobs run; the 7 slow jobs are skipped.

### Terminal output — job tier classification script

Script: parse ci.yml, iterate all job IDs under `jobs:`, classify by presence of the
full-tier `if:` gate (keyed on `github.event_name == 'merge_group'` as the leading
discriminator).

```
=== Job tier classification ===
  check-panic-profile            -> FAST (ungated)
  check-pdf-deps                 -> FAST (ungated)
  fmt                            -> FAST (ungated)
  clippy                         -> FAST (ungated)
  test                           -> FAST (ungated)
  doctest                        -> FAST (ungated)
  supply-chain                   -> FAST (ungated)
  test-matrix-slow               -> SLOW (gated)
  msrv                           -> SLOW (gated)
  docs                           -> SLOW (gated)
  snapshots                      -> SLOW (gated)
  bench                          -> SLOW (gated)
  visual-regression              -> SLOW (gated)
  perf-smoke                     -> SLOW (gated)
  all-checks-pass                -> FAST (ungated)
```

**Count analysis:**
- Fast (ungated) jobs: `check-panic-profile`, `check-pdf-deps`, `fmt`, `clippy`, `test`,
  `doctest`, `supply-chain` = **7 fast jobs**
- Slow (gated) jobs: `test-matrix-slow` (3-platform matrix: linux-arm64, macos-arm64,
  windows-x86_64), `msrv`, `docs`, `snapshots`, `bench`, `visual-regression`,
  `perf-smoke` = **7 slow jobs**
- `all-checks-pass` is the aggregator — ungated by design (`if: always()`)

**Result: PASS** — exactly 7 ungated fast jobs and 7 gated slow jobs as specified in AC-001.

### Live wall-clock measurement (deferred-to-PR per AC-001 measure-not-gate clause)

AC-001 explicitly states: "The AC passes if the slow legs are successfully removed from the
per-PR path AND the fast-tier wall-clock is measured and recorded in the PR description —
do NOT gate on a fixed ≤8 min threshold, as that figure is a directional estimate, not a
confirmed per-project measurement."

**Deferred-to-PR:** The actual per-PR wall-clock (target ~6-8 minutes directional) will be
measured when PR #NN triggers on the `ci/bench-timeout-cache-on-failure` branch and recorded
in the PR description. This evidence doc will be updated with the measured value once
available.

---

## AC-002: `all-checks-pass` correctly reports green on PR (traces to NFR-028)

**Claim:** The aggregator treats `skipped` as PASS and only fails when a job that actually
ran reported `failure` or `cancelled`. The Python logic is extracted directly from ci.yml
lines 656–663.

### Terminal output — aggregator Python simulation (3 payloads)

The exact Python embedded in ci.yml (no modifications):

```python
results = json.loads(os.environ['NEEDS_JSON'])
failed = [name for name, info in results.items()
          if info['result'] not in ('success', 'skipped')]
if failed:
    print(' '.join(failed))
```

Run locally against three simulated `NEEDS_JSON` payloads:

```
=== AC-002: Aggregator skipped-as-pass simulation ===

  Payload 1 (all success + slow skipped)
    Result: PASS (exit 0)

  Payload 2 (test/linux-x86_64 failure)
    Result: FAIL (exit 1) -- failed jobs: test

  Payload 3 (clippy cancelled)
    Result: FAIL (exit 1) -- failed jobs: clippy

Aggregator logic source (extracted from ci.yml lines 656-663):

    results = json.loads(os.environ['NEEDS_JSON'])
    failed = [name for name, info in results.items()
              if info['result'] not in ('success', 'skipped')]
    if failed:
        print(' '.join(failed))
```

**Payload detail:**

Payload 1 (`all success + slow skipped`) — simulates normal PR push with fast tier passing
and all slow jobs skipped. Aggregator exits 0 = branch protection satisfied.

Payload 2 (`test/linux-x86_64 failure`) — simulates a failing fast-tier test job. The
`test` key has `result: failure`. Aggregator exits 1 = merge blocked.

Payload 3 (`clippy cancelled`) — simulates a cancelled job (e.g., concurrent cancel).
The `clippy` key has `result: cancelled`. Aggregator exits 1 = merge blocked.

**Result: PASS** — `skipped` is correctly treated as success; `failure` and `cancelled`
both block the aggregator; the aggregator itself has `if: always()` ensuring it always
reports regardless of what the other jobs do.

### Aggregator `if: always()` and `needs:` completeness (from ci.yml)

The aggregator job declaration (ci.yml lines 622–643):

```yaml
all-checks-pass:
  name: all-checks-pass
  needs:
    # Fast tier — always runs
    - check-panic-profile
    - check-pdf-deps
    - fmt
    - clippy
    - test
    - doctest
    - supply-chain
    # Slow tier — skipped on normal PR pushes (treated as pass by aggregator)
    - test-matrix-slow
    - msrv
    - docs
    - snapshots
    - bench
    - visual-regression
    - perf-smoke
  runs-on: ubuntu-latest
  timeout-minutes: 5
  if: always()
```

All 14 jobs (7 fast + 7 slow) are present in `needs:`. The `if: always()` ensures the
aggregator runs unconditionally.

---

## AC-003: merge_group runs the full matrix (traces to NFR-026, NFR-029, NFR-030)

**Claim:** The 6-combo full-tier `if:` expression is byte-identical at all 7 slow-job
sites in ci.yml. The expression covers all 6 required event/ref combinations.

### Terminal output — byte-identity scan

```
=== Full-tier if: expression occurrences: 7 ===

  Match 1 at line 294: IDENTICAL
  Match 2 at line 345: IDENTICAL
  Match 3 at line 370: IDENTICAL
  Match 4 at line 398: IDENTICAL
  Match 5 at line 444: IDENTICAL
  Match 6 at line 495: IDENTICAL
  Match 7 at line 575: IDENTICAL

All 7 copies byte-identical: True
```

**Job-to-line mapping:**
- Line 294: `test-matrix-slow` (linux-arm64, macos-arm64, windows-x86_64)
- Line 345: `msrv`
- Line 370: `docs`
- Line 398: `snapshots`
- Line 444: `bench`
- Line 495: `visual-regression`
- Line 575: `perf-smoke`

### The canonical 6-combo expression (verbatim from ci.yml)

```yaml
if: >
  github.event_name == 'merge_group' ||
  github.event_name == 'schedule' ||
  github.event_name == 'workflow_dispatch' ||
  (github.event_name == 'push' && (github.ref == 'refs/heads/develop' || github.ref == 'refs/heads/main')) ||
  (github.event_name == 'pull_request' &&
   contains(github.event.pull_request.labels.*.name, 'full-ci'))
```

### The 6 combos enumerated

```
=== The 6 conditions (combos) ===
  combo-1: github.event_name == 'merge_group'
  combo-2: github.event_name == 'schedule'
  combo-3: github.event_name == 'workflow_dispatch'
  combo-4: push && github.ref == 'refs/heads/develop'
  combo-5: push && github.ref == 'refs/heads/main'
  combo-6: pull_request && contains(labels, 'full-ci')
```

NFR coverage per combo:
- combo-1 (`merge_group`): full matrix validates before commit lands on develop — primary
  NFR-026/NFR-029/NFR-030 gate
- combo-2 (`schedule`): nightly full matrix on main — catches toolchain/env drift
- combo-3 (`workflow_dispatch`): closes ci-workflow-analyzer finding #3 — manual full-matrix
  trigger without needing to push a commit
- combo-4 (push to develop): post-merge full-matrix safety net on develop
- combo-5 (push to main): closes ci-workflow-analyzer finding #5 — release merges to main
  receive full matrix validation
- combo-6 (full-ci label): developer opt-in on-demand full matrix

**Result: PASS** — 7/7 slow job sites carry the byte-identical 6-combo expression.

---

## AC-004: No deadlock — only `all-checks-pass` is a required status check

**Claim:** (1) No workflow-level `paths` or `paths-ignore` filters exist in ci.yml.
(2) The playbook documents the deadlock pattern and the branch protection instruction
requiring only `all-checks-pass`.

### Terminal output — grep for paths/paths-ignore

```
=== AC-004: Workflow-level paths/paths-ignore scan ===

--- grep for 'paths:' in on: block ---
on: block content:
on:
  push:
    branches: [develop, main]
  pull_request:
    branches: [develop, main]
    types: [opened, synchronize, reopened, labeled]
  merge_group:
  workflow_dispatch:
  schedule:
    - cron: "0 6 * * *"

CONFIRMED: No paths/paths-ignore in on: block.

--- grep for paths: anywhere in file ---
  (no matches — CLEAN)
  (no matches — CLEAN)
```

**Result: PASS** — zero occurrences of `paths:` or `paths-ignore:` anywhere in ci.yml.

### Playbook §4 required-check instruction (quoted)

From `docs/playbooks/tiered-ci-merge-queue.md` §4 "Branch protection instruction":

> **Required status checks:**
> - `all-checks-pass` — the aggregator job (the ONLY required check)
>
> **IMPORTANT — context name vs. PR UI display:**
> The GitHub branch-protection API uses the bare job name as the context string.
> The PR check-list displays it as "CI / all-checks-pass" (workflow name / job
> name), but the `contexts` field in the PUT payload and the verify output use
> `all-checks-pass` alone. Using `"CI / all-checks-pass"` in the API payload will
> create a check that never matches the actual reported context — PRs will be
> permanently blocked.

The playbook §3 also documents the deadlock anti-pattern explicitly, covering:
- Workflow-level `paths`/`paths-ignore` on a workflow containing a required check
- Individual slow job listed as a required status check when that job uses job-level `if:`

---

## AC-005: Nightly and develop push still run the full matrix

**Claim:** `push` to `develop` branch and `schedule` nightly both trigger the full matrix
via the 6-combo `if:` condition. The schedule-from-main constraint is explicitly documented.

### Terminal output — on: block and coverage

```
on: block:
on:
  push:
    branches: [develop, main]
  pull_request:
    branches: [develop, main]
    types: [opened, synchronize, reopened, labeled]
  merge_group:
  workflow_dispatch:
  schedule:
    # Nightly full-matrix run at 06:00 UTC.
    # NOTE: only fires from the default branch ('main'). See header comment above.
    - cron: "0 6 * * *"

Schedule cron (line 41): - cron: "0 6 * * *"
```

The `on:` block confirms:
- `push: branches: [develop, main]` — push events to both branches trigger the workflow
- `merge_group:` — present for merge queue support
- `workflow_dispatch:` — present for manual full-matrix trigger
- `schedule: cron: "0 6 * * *"` — nightly at 06:00 UTC

The slow-tier `if:` expression (verified byte-identical at 7 sites in AC-003) includes both:
- `github.event_name == 'schedule'` (combo-2)
- `github.event_name == 'push' && github.ref == 'refs/heads/develop'` (combo-4)
- `github.event_name == 'push' && github.ref == 'refs/heads/main'` (combo-5)

Push to `develop` lines confirmed present at 7 slow-job sites (lines 298, 349, 374, 402,
448, 499, 579 per AC-005 grep output):
```
grep -n "refs/heads/develop" ci.yml (slow-job sites):
  298: (github.event_name == 'push' && (github.ref == 'refs/heads/develop' || ...))
  349: (same)
  374: (same)
  402: (same)
  448: (same)
  499: (same)
  579: (same)
```

### Documented schedule-from-main constraint

From `docs/playbooks/tiered-ci-merge-queue.md` §6 "`schedule:` default-branch constraint":

> **GitHub Actions only fires `schedule:` triggers from the workflow file on the
> DEFAULT branch.** The default branch for `drbothen/slideforge` is `main` (not `develop`).
>
> Consequences:
> - The nightly cron defined in `ci.yml` (`0 6 * * *`) does NOT fire while
>   `ci.yml` is only on `develop`.
> - The nightly leg becomes active once this story's changes are merged from
>   `develop` to `main` (the normal release flow).

This constraint is also documented in the ci.yml header comment (lines 16-21):

```yaml
# IMPORTANT — schedule: and the default branch:
#   GitHub Actions only fires schedule: triggers from the workflow file on the
#   DEFAULT branch (currently 'main'). A nightly cron defined here on 'develop'
#   will NOT fire until this file is merged to 'main'. The nightly leg activates
#   automatically once this story is merged to main via the normal develop→main
#   release flow.
```

**Result: PASS** — develop push and schedule triggers present in `on:` block; covered by
slow-tier `if:` combos 2, 4, and 5 at all 7 slow-job sites; schedule default-branch
constraint documented in both ci.yml header and playbook §6.

---

## AC-006: `docs/playbooks/tiered-ci-merge-queue.md` created

**Claim:** File exists and contains the 5 required sections (a)-(e) plus the schedule
constraint.

### Terminal output — ls -la

```
-rw-r--r--  1 jmagady  staff  17869 Jun 10 19:33 docs/playbooks/tiered-ci-merge-queue.md
```

### Terminal output — section headings (grep '^##')

```
10:## Summary
24:## 1. Tiered trigger design rationale
61:## 2. Full-tier `if:` condition (exact expression)
93:## 3. Deadlock anti-pattern — do not reintroduce
98:### What causes the deadlock
116:### How this design prevents the deadlock
126:### Forbidden patterns
142:### Safe pattern (correct)
151:### Repo-wide invariant: companion workflows must never be required checks
176:## 4. Branch protection instruction
212:### Establishing branch protection for the first time (gh CLI)
286:## 5. Trade-off: platform-specific bugs surface at merge-queue time
316:## 6. `schedule:` default-branch constraint and manual dispatch
355:## 7. Adding a new CI job — maintainer checklist
378:## 8. Edge cases reference
```

### AC-006 requirement mapping

| Requirement | Section | Status |
|-------------|---------|--------|
| (a) Tiered trigger design rationale | §1 "Tiered trigger design rationale" (line 24) | PRESENT |
| (b) Job-level `if:` condition expression for full-tier gate | §2 "Full-tier `if:` condition (exact expression)" (line 61) | PRESENT |
| (c) Deadlock anti-pattern explanation | §3 "Deadlock anti-pattern — do not reintroduce" (line 93) | PRESENT |
| (d) Branch protection instruction (only `all-checks-pass` required) | §4 "Branch protection instruction" (line 176) | PRESENT |
| (e) Trade-off: platform-specific bugs surface at merge-queue time | §5 "Trade-off: platform-specific bugs surface at merge-queue time" (line 286) | PRESENT |
| `schedule:` default-branch constraint | §6 "`schedule:` default-branch constraint and manual dispatch" (line 316) | PRESENT |

File size: 17,869 bytes. All 5 required AC-006 sections (a)-(e) present plus the schedule
constraint documented in its own section (§6). Additionally: §7 maintainer checklist and
§8 edge cases reference (8 scenarios, including EC-002 correction per spec v1.2).

**Result: PASS** — all required sections present.

---

## Overall: actionlint validation

```
$ actionlint .github/workflows/ci.yml
EXIT CODE: 0 (clean)
```

actionlint 1.7.12 reports zero errors or warnings on ci.yml as shipped in the STORY-091
worktree (HEAD 30edf4b2). This confirms the YAML is structurally valid as a GitHub Actions
workflow.

---

## Evidence Summary

| AC | Description | Local verification | Live-CI gate |
|----|-------------|-------------------|--------------|
| AC-001 | 7 fast / 7 slow jobs; fast-only on PR push | PASS — job tier classification script | Wall-clock measurement deferred to PR #NN (measure-not-gate per AC-001 framing) |
| AC-002 | Aggregator skipped-as-pass | PASS — 3-payload Python simulation: success+skipped→0, failure→1, cancelled→1 | Live green/red behavior confirmed at PR time |
| AC-003 | 6-combo gate byte-identical at 7 sites | PASS — pattern match: 7/7 IDENTICAL | merge_group run confirmed at PR merge-queue time |
| AC-004 | No paths/paths-ignore; only all-checks-pass required | PASS — zero grep matches; playbook §4 quoted | Branch protection config (external) deferred to PR merge-prep |
| AC-005 | develop push + nightly full matrix | PASS — on: block + 7 slow-job if: sites cover combo-2/4/5; schedule constraint documented | Nightly activates after merge to main |
| AC-006 | Playbook created with required sections | PASS — file exists (17,869 bytes); all 5 sections (a)-(e) + schedule constraint present | N/A (static artifact) |
| Overall | actionlint clean | PASS — exit code 0 | N/A |
