---
document_type: resume-playbook
cycle: STORY-086
version: "1.0"
created: 2026-06-06
status: active
purpose: >
  Zero-context mechanical resume procedure for STORY-086. A fresh orchestrator
  session with no memory must be able to pick up from this document alone and
  execute without ambiguity.
---

# STORY-086 Zero-Context Resume Playbook

**IMMEDIATE NEXT ACTION: Dispatch adversary LOCAL pass 13 (streak 0/3, target 3 consecutive strict-CLEAN).**

---

## Ground Truth at Time of Writing (2026-06-06)

| Field | Value |
|-------|-------|
| develop SHA | `030dec6c` (61 merged PRs; 0 open PRs) |
| feature/STORY-086 HEAD | `82f300db` (PUSHED to origin/feature/STORY-086) |
| Workspace tests | 3300/3300 pass, 14 skipped (1 pre-existing cold_budget flake — STORY-080) |
| Canonical exit gate | CLEAN (fmt + pedantic clippy + RUSTDOCFLAGS doc + nextest) |
| Adversary LOCAL streak | 0/3 (pass 11 strict-CLEAN — streak reached 1/3; pass 12 found F-086-P12-MED-001 — REMEDIATED code-doc-only — streak reset 0/3; pass 13 next) |
| BLK-002 | OPEN — closes on STORY-086 merge + Wave 4 re-gate pass |
| Key spec versions | BC-1.16.001 v1.4, BC-3.04.001 v1.6, BC-4.01.001 v1.2, BC-4.02.001 v1.2, BC-5.01.001 v1.3, BC-5.02.001 v1.6, error-taxonomy v2.17, ADR-019 v1.5, STORY-086 v1.5, STORY-088 v1.2 (8 pts) |

---

## Step 0 — Preflight (ALWAYS run first, every session)

### 0a. Factory worktree health

Dispatch `vsdd-factory:factory-worktree-health` via devops-engineer. It verifies:
- `.factory/` is mounted on `factory-artifacts` branch
- `.factory-project/` check (skip — single-repo project)
- Remote branch existence + sync state

### 0b. Verify develop is not drifted (LESSON-12)

```bash
git -C /Users/jmagady/Dev/slideforge rev-parse develop
git -C /Users/jmagady/Dev/slideforge rev-parse origin/develop
```

Both MUST equal `030dec6c`. If local develop has drifted (does not match origin/develop):

```bash
git -C /Users/jmagady/Dev/slideforge fetch origin
git -C /Users/jmagady/Dev/slideforge update-ref refs/heads/develop origin/develop
```

NEVER force-push develop without explicit human approval.

### 0c. Verify or recreate the STORY-086 worktree

```bash
ls /Users/jmagady/Dev/slideforge/.worktrees/STORY-086
```

If MISSING, recreate it:

```bash
git -C /Users/jmagady/Dev/slideforge worktree add .worktrees/STORY-086 feature/STORY-086
```

Then confirm HEAD:

```bash
git -C /Users/jmagady/Dev/slideforge/.worktrees/STORY-086 rev-parse HEAD
```

Must equal `82f300db` (or a later commit if the cascade advanced this session). If it is a DIFFERENT, unexpected commit, stop and investigate before proceeding.

### 0d. Confirm workspace is still CLEAN

```bash
cargo nextest run --workspace --no-fail-fast 2>&1 | tail -5
```

Expected: ~3300 pass, 14 skipped, 0 failures. If failures exist, diagnose before dispatching adversary.

---

## Step 1 — Resume the LOCAL Adversary Cascade

**Current position: streak 0/3. Pass 13 must run next.**

### Discipline rules (inline — all must survive context loss)

**RULE-SEQ (LESSON-7):** Run passes SEQUENTIALLY — one at a time. Never parallelize passes of the same story. Pass N+1 only after pass N is fully remediated and the canonical exit gate is re-run clean.

**RULE-WORKTREE-ABS (LESSON-1, LESSON-16):** Every adversary Read/Grep/Glob call MUST use the WORKTREE-ABSOLUTE path. The adversary's cwd must be set to the worktree:

```
cwd: /Users/jmagady/Dev/slideforge/.worktrees/STORY-086
```

**RULE-PG-WORKTREE-TYPES (CRITICAL — learned from pass-7 corruption):**
Any agent correcting a spec against code for an IN-FLIGHT worktree story MUST read type and trait definitions from the WORKTREE path:

```
.worktrees/STORY-086/crates/...
```

NEVER from the main checkout `/Users/jmagady/Dev/slideforge/crates/...` (that is the `develop` branch, which does NOT contain STORY-086's additions: `TextTag`, `TextBlock.tag`, `AltText::Unspecified`).

Precedent: PO commit `608ec7b0` read develop types and wrongly stripped `TextTag`/`TextBlock.tag`/`AltText::Unspecified` from BC-1.16.001. Recovered via `f2261592`. This is process-gap PG-WORKTREE-TYPES in `cycles/STORY-086/lessons.md`.

**RULE-DIFF:** The adversary reviews `git diff origin/develop..HEAD` (the full story diff). Do NOT limit review to only the most recent commit.

**RULE-CLEAN-STRICT:** CLEAN (strict) = ZERO findings of ANY severity (advances streak toward 3/3). CLEAN (PR-merge) = zero CRIT+HIGH+MED only (does NOT advance streak). The adversary MUST report both explicitly:

```
CLEAN (strict): yes/no
CLEAN (PR-merge): yes/no
```

Any finding of any severity = streak resets to 0/3.

**RULE-REMEDIATION:** After any findings, route via the VSDD Feedback table before redispatching adversary:

| Finding type | Route to agent |
|---|---|
| Spec gap / BC text stale | product-owner (MUST use worktree-absolute paths for type reading) |
| Architecture decision / inter-BC conflict | architect |
| Failing test / test gap | test-writer |
| Implementation defect | implementer |
| Story spec narrative stale | story-writer |

After every fix-burst by any agent: re-run the FULL canonical exit gate before dispatching the next adversary pass.

**RULE-GATE:** Canonical exit gate for STORY-086:

```bash
# In /Users/jmagady/Dev/slideforge/.worktrees/STORY-086
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo nextest run --workspace --no-fail-fast
```

All four must pass with zero failures before the next adversary pass.

### Adversary pass dispatch template (copy-paste ready)

Replace `<HEAD>` with the current worktree HEAD SHA before dispatching. Next pass number is 13.

```
ADVERSARY LOCAL PASS [N] — STORY-086

You are the adversary for STORY-086 (slide-field-to-block content threading + TextTag).
This is pass [N] of the LOCAL convergence cascade (streak [X]/3, target 3 consecutive
strict-CLEAN per BC-5.39.001).

WORKING DIRECTORY (mandatory): /Users/jmagady/Dev/slideforge/.worktrees/STORY-086
ALL Read/Grep/Glob calls MUST use this absolute worktree path.

STORY-086 feature branch HEAD: 82f300db
Review target: git diff origin/develop..82f300db
  (the complete story diff — not just the latest commit)

CONTRACT VERSIONS IN EFFECT (read from worktree .factory/ paths):
  BC-1.16.001  v1.4   (.factory/specs/behavioral-contracts/BC-1.16.001-alt-text-enforcement.md)
  BC-3.04.001  v1.6   (.factory/specs/behavioral-contracts/BC-3.04.001-exporter-contracts.md)
  BC-4.01.001  v1.2   (.factory/specs/behavioral-contracts/BC-4.01.001-pptx-slide-types.md)
  BC-4.02.001  v1.2   (.factory/specs/behavioral-contracts/BC-4.02.001-docx-slide-types.md)
  BC-5.01.001  v1.3   (.factory/specs/behavioral-contracts/BC-5.01.001-validate-contracts.md)
  BC-5.02.001  v1.6   (.factory/specs/behavioral-contracts/BC-5.02.001-plugin-registry.md)
  error-taxonomy v2.17 (.factory/specs/prd-supplements/error-taxonomy.md)
  ADR-019 v1.5        (.factory/specs/architecture/adr/ADR-019-stage2b-field-threading.md)
  STORY-086 v1.5      (.factory/stories/STORY-086-slide-field-to-block-threading.md)

FOCUS AREAS (check all):
  1. BC-1.16.001 PC-7/PC-8/PC-9/PC-10/PC-11/PC-12: TextTag assignment, AltText state machine,
     decorative-first ordering, trimmed content invariant
  2. BC-4.01.001 / BC-4.02.001: tag-driven FrameContent slot selection (RegionRole enum)
     — tag-aware slot fills MUST use role match, not position-first-empty-wins
  3. ADR-019 §3.1/§3.3: trim on store (Arc::from(text.trim()), Arc::from(s.trim()))
  4. BC-5.01.001 PC-3/PC-12 + BC-5.02.001 Inv-11 post-layout AltTextValidator path
  5. AC-007 #[ignore] SID-1 citation (must name STORY-088 + test name)
  6. BulletItem struct fields: {inlines: Vec<InlineNode>, children: Vec<BulletItem>, span: Span}
     (NOT {text, level, span} — that was the stale form corrected in BC-1.16.001 v1.3)
  7. ImageSpec field names: Rust field `path` (DSL keyword `src`) — confirmed PC-10
  8. W-A11-002 tracing::warn! emission on both-set (decorative+alt) — single-fire post-layout

CRITICAL — WORKTREE TYPE PATHS:
  TextTag, TextBlock.tag, AltText::Unspecified exist ONLY in the STORY-086 worktree
  (.worktrees/STORY-086/crates/slideforge-types/src/block.rs).
  They are NOT on develop/main checkout. If you read types from the main checkout
  you will produce a corrupted finding — use ONLY the worktree path.

PREVIOUSLY FIXED (verified closed in passes 1-10 — do NOT reopen unless new evidence):
  F-086-P1-CRIT-001 (emit alt=None blocks), P2-MED-001/002, P2-geometry, P3-HIGH-001
  (RegionRole tag-aware slot), P3-MED-001 (FrameContent doc), P4 findings,
  P5-CRIT-001 (decorative-first), P5-MED-001/002, P5-OBS-1, P6-MED-001 (trim storage),
  P7-MED-001 (BulletItem fields stale BC), P8 (strict-CLEAN — all prior verified closed),
  P9-MED-001 (phantom ChartSpec.data_source in BC-1.16.001 PC-9 + ADR-019 §3.3 — removed v1.4),
  P9-LOW-001 (extract_str_field doc comment inaccurate — corrected),
  P10-MED-001 (ADR-019 §3.2 BulletItem phantom {text,level} — replaced with real fields {inlines,children} per v1.5),
  P10-MED-002 (ADR-019 §3.4 ImageSpec phantom DSL keyword src — replaced with real Rust field path per v1.5),
  P10-MED-003 (field_to_block.rs:60+:268 rustdoc showed untrimmed Arc::from(s) — corrected to Arc::from(s.trim())),
  P11 (strict-CLEAN — all prior findings verified closed),
  P12-MED-001 (build_inner body stage-comments line 459 misstated brand-vs-threading order, contradicting ADR-019 Decision 1 mandate — body comments renumbered + reconciled with real physical execution order; code-doc-only fix)

MANDATORY OUTPUT LINES (include verbatim at end of report):
  CLEAN (strict): yes/no
  CLEAN (PR-merge): yes/no
  Streak after this pass: [X]/3
```

### Pass result handling

| Result | Action |
|---|---|
| CLEAN (strict) = yes | Streak advances. If now 3/3 → proceed to Step 2. If 1/3 or 2/3 → dispatch next pass immediately. |
| CLEAN (strict) = no | Streak resets to 0/3. Remediate all findings. Re-run canonical exit gate. Dispatch next pass. |
| API overload / timeout | Re-dispatch the same pass number. Do NOT increment the pass number. |

---

## Step 2 — Post-Convergence Chain (after 3 strict-CLEAN)

Execute in this exact order. Do NOT parallelize within a step.

### Step 2.1 — Demo recording

Dispatch `vsdd-factory:demo-recorder` for STORY-086.

**LESSON-15 (mandatory):** After recording, run the FULL canonical clippy on any example binary added:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items
```

The `-W clippy::missing_docs_in_private_items` flag is required — example private items need doc comments. CI has failed twice when this flag was omitted (STORY-039, STORY-040).

### Step 2.2 — Push feature branch

```bash
git -C /Users/jmagady/Dev/slideforge/.worktrees/STORY-086 push origin feature/STORY-086
```

Confirm `origin/feature/STORY-086` matches the current worktree HEAD (will be later than `a055345e` if cascade produced fix commits).

### Step 2.3 — PR creation

Dispatch `vsdd-factory:pr-manager` for the 9-step PR cycle. PR targets `develop`. Title follows Conventional Commits format.

**LESSON-5 (mandatory):** pr-manager CANNOT spawn sub-agents. The orchestrator dispatches security-reviewer and pr-reviewer INDEPENDENTLY (see Step 2.4).

### Step 2.4 — Security review + PR review (orchestrator dispatches both)

Dispatch in parallel (they are independent):
- `vsdd-factory:security-reviewer` on the STORY-086 PR diff
- `vsdd-factory:pr-reviewer` on the STORY-086 PR diff

Wait for BOTH to complete before proceeding.

**LESSON-9 (mandatory):** If any security or pr-reviewer finding is fixed AFTER convergence (code diff changes), RE-RUN both security-reviewer + pr-reviewer and wait for CI before merge. Do NOT assume prior convergence still holds after diff changes.

### Step 2.5 — CI gate

Wait for all CI checks green on the PR. CI matrix covers: fmt, pedantic clippy, nextest, doctest, insta snapshots, bench, perf-smoke, visual-regression, pdf-ua1, supply-chain, audit, msrv, semgrep, panic-profile, check-pdf-deps, docs.

### Step 2.6 — Merge (STANDING MERGE AUTH)

Squash-merge when ALL of:
- CI green (all checks)
- security-reviewer: APPROVE/CLEAN
- pr-reviewer: APPROVE

Execute: `gh pr merge --squash <PR-number>`

**Post-merge discipline (LESSON-12):** Immediately verify develop is not drifted:

```bash
git -C /Users/jmagady/Dev/slideforge fetch origin
git -C /Users/jmagady/Dev/slideforge rev-parse develop
git -C /Users/jmagady/Dev/slideforge rev-parse origin/develop
```

Both must match. If drifted, reset: `git -C /Users/jmagady/Dev/slideforge update-ref refs/heads/develop origin/develop`.

Record the new develop SHA in STATE.md (state-manager dispatch).

---

## Step 3 — Re-run Wave 4 Gate

After STORY-086 merges, re-run the failed Wave 4 gate checks on patched develop.

**What carries over (do NOT re-run):**
- Gate 1: PASS (carry over)
- Gate 2: SKIP/no DTU (carry over)

**What must re-run:**
- Gate 3: adversary review on the patched develop diff
- Gate 5: holdout evaluator

Dispatch: `vsdd-factory:wave-gate` on develop after merge.

Root causes being fixed by STORY-086 (verify each is closed at gate re-run):
- F-G3-CRIT-001: eval sets slide.blocks=vec![] (for_eval.rs:342) → FIXED by Stage 2b field-to-block threading
- F-G3-HIGH-001/002: inert validators (AltText pre-layout, CanvasOverflowValidator, LabelCheck) → unblocked by content threading
- Holdout mean 0.56 / min_critical 0.30 below threshold → content-EMPTY output was root cause; threading fix unblocks

BLK-002 closes ONLY when STORY-086 is merged AND Wave 4 Gate 3 + Gate 5 re-pass.

---

## Step 4 — Advance to Wave 5

Only after ALL Wave 4 gates pass (including re-run Gate 3 + Gate 5).

**Wave 5 story queue (in dependency order):**
1. STORY-087 — Color-coded slide types (closes F-G3-HIGH-003; Wave 5, P0)
2. STORY-082 — PPTX Slide-Grouping Sections (Wave 5, 5 pts, P0; BC-4.01.003 Half B, BC-1.14.003; split from STORY-040, human-authorized 2026-06-04; EPIC-08)
3. STORY-081 — Slide-level inline markup (P0 for v1.0, Wave 5, EPIC-18, depends STORY-077)
4. STORY-088 — Bullets list-literal DSL syntax (Wave 5, 8 pts; covers BOTH `bullets: ["A","B","C"]` AND `bullets: var_name` @var/vars-block binding)

---

## Step 5 — Cycle-Close Codification (S-7.02)

Before declaring the STORY-086 sub-cycle CLOSED, resolve both process-gaps in `cycles/STORY-086/lessons.md`:

### PG-TD060-SCOPE
TD-VSDD-060 sibling-sweep scope during STORY-086 did not include narrative/doc/test-name sites — only match-arm compile sites.

Required action: create a follow-up improvement story anchored to the self-improvement epic, OR produce a justified deferral with a specific anchor story/wave. Cannot be left unanchored.

### PG-WORKTREE-TYPES
Spec-correction agents for IN-FLIGHT worktree stories MUST read types from the worktree (.worktrees/STORY-NNN/), NOT main checkout (develop). PO commit `608ec7b0` violated this and corrupted BC-1.16.001; recovered via `f2261592`.

Required action: create a follow-up improvement story OR justified deferral. The orchestrator dispatch template for spec-correction agents must be codified to EXPLICITLY pin the worktree type path for in-flight stories.

---

## Durable Artifact Index

All paths are under `/Users/jmagady/Dev/slideforge/.factory/` (factory-artifacts branch).

| Artifact | Path | Notes |
|---|---|---|
| Adversary pass 1 | `cycles/STORY-086/adversarial-reviews/adversary-STORY-086-pass-1.md` | D1-D5 found; architect adjudication pass-1 |
| Adversary passes 2-7 | `cycles/STORY-086/adversarial-reviews/adversary-STORY-086-pass-{2..7}.md` | Note: only pass-1 file currently committed; passes 2-7 recorded in STATE.md Decisions Log |
| Architect adjudication pass-1 | `cycles/STORY-086/architect-pass-1-adjudication.md` | D1 TextTag→FrameContent routing (layout-side), D2-D5 |
| Architect adjudication pass-5 | `cycles/STORY-086/architect-pass-5-adjudication.md` | Decorative-first canonical; ADR-019 v1.2; W-A11-002 mechanism |
| Architect adjudication pass-6 | `cycles/STORY-086/architect-pass-6-adjudication.md` | TRIMMED canonical; BC supersedes ADR; ADR-019 v1.3 |
| ADR-019 v1.5 | `specs/architecture/adr/ADR-019-stage2b-field-threading.md` | Stage 2b post-eval field-to-block threading; exhaustive struct-field sweep — BulletItem phantom {text,level} + ImageSpec phantom {src} removed; all other struct examples confirmed correct |
| BC-1.16.001 v1.4 | `specs/behavioral-contracts/BC-1.16.001-alt-text-enforcement.md` | TextTag, AltText state machine, decorative-first, trim; phantom data_source removed from PC-9 |
| BC-3.04.001 v1.6 | `specs/behavioral-contracts/BC-3.04.001-exporter-contracts.md` | Inv-11 scoped to shape-DSL path only |
| BC-4.01.001 v1.2 | `specs/behavioral-contracts/BC-4.01.001-pptx-slide-types.md` | Tag-routing for FrameContent |
| BC-4.02.001 v1.2 | `specs/behavioral-contracts/BC-4.02.001-docx-slide-types.md` | Tag-routing for FrameContent |
| BC-5.01.001 v1.3 | `specs/behavioral-contracts/BC-5.01.001-validate-contracts.md` | Post-layout AltTextValidator path |
| BC-5.02.001 v1.6 | `specs/behavioral-contracts/BC-5.02.001-plugin-registry.md` | |
| Error taxonomy v2.17 | `specs/prd-supplements/error-taxonomy.md` | W-A11-002 registered |
| STORY-086 v1.5 | `stories/STORY-086-slide-field-to-block-threading.md` | Corrected e2e test path |
| STORY-088 v1.2 | `stories/STORY-088-bullets-list-literal-dsl.md` | 8 pts; covers BOTH bullet forms |
| Wave4 uncertainty resolution | `specs/wave4-expanded-scope-uncertainty-resolution.md` | D1-D5 architect decisions pre-Red Gate |
| Lessons | `cycles/STORY-086/lessons.md` | PG-TD060-SCOPE + PG-WORKTREE-TYPES |
| Session checkpoints (archived) | `cycles/STORY-086/session-checkpoints.md` | All prior session checkpoints |
