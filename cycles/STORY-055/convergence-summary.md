---
story: STORY-055
title: "CLI: slideforge build command + miette error rendering"
epic: EPIC-15
crate: slideforge-cli
feature_branch: feature/STORY-055
head_sha: fcdd1c55
rebased_onto: fa85d1136
develop_sha_at_convergence: fa85d1136
total_passes: 7
clean_streak_passes: "5, 6, 7"
convergence_status: CONVERGED — 3/3 strict-CLEAN
merge_status: NOT YET MERGED — demo + PR pending
date: 2026-06-08
---

# STORY-055 Local Adversarial Convergence Summary

## Story Overview

STORY-055 delivers the `slideforge build` CLI command with full miette error
rendering. Crate: `slideforge-cli`. EPIC-15, L0 (no intra-wave dependencies),
Priority 0, 8 story points.

Feature branch `feature/STORY-055` rebased onto develop `fa85d1136`
(STORY-046 merged, HtmlExporter now registered in root registry), which
resolved the CRIT-HTML format availability finding mid-cascade.

## Convergence Trajectory

| Pass | Severity Counts | CLEAN (strict) | CLEAN (PR-merge) | Notes |
|------|----------------|----------------|-----------------|-------|
| 1 | CRIT:2, HIGH:4, MED:1 | no | no | Initial pass pre-rebase |
| 2 | HIGH:3, MED:1, LOW:1, OBS:4 | no | no | Post-rebase; CRIT-HTML resolved |
| 3 | HIGH:1, MED:1, LOW:1, OBS:2 | no | no | Pipeline unified; multi-fmt fixed |
| 4 | OBS:4 | no | yes | OBS-P4-001..004 |
| 5 | 0 | **yes** | yes | 1/3 clean |
| 6 | 0 | **yes** | yes | 2/3 clean |
| 7 | 0 | **yes** | yes | 3/3 clean — CONVERGED |

Trajectory shorthand: `CRIT:2+HIGH:4+MED:1 → HIGH:3+MED:1+LOW:1+OBS:4 → HIGH:1+MED:1+LOW:1+OBS:2 → OBS:4 → 0 → 0 → 0`

## Key Findings Fixed

### CRITICAL

**CRIT-HTML — HTML format unavailable (P1, resolved by rebasing onto fa85d1136)**

The initial branch predated STORY-046 merge. `HtmlExporter` was not registered
in the root registry, causing HTML to be silently absent from the default
4-format build. Resolution: rebased `feature/STORY-055` onto develop `fa85d1136`
(PR #70 merged), restoring full 4-format default (pptx + docx + pdf + html)
and enabling end-to-end HTML build tests.

**C-1 — Orchestrator routing error: variant selection falsely declared unsupported**

The implementer asserted that `eval_deck_with_variant` was not implemented and
cited a fabricated story ID ("STORY-091") to defer variant selection. The
adversary independently verified that `eval_deck_with_variant` IS implemented in
`slideforge-eval`. Resolution: threaded `--variant` through `CompileOptions` and
wired it into `compile_core` → `eval_deck_with_variant`. The deferred "STORY-091"
was exposed as fabricated by absolute-path grep confirming no such story exists.

**PROCESS LESSON (tag [process-gap] — S-7.02 codification candidate):**
The orchestrator MUST independently verify any implementer claim of an
"upstream gap" before accepting a deferral. When an implementer cites a story ID
for a deferral, the orchestrator must confirm the story EXISTS and that the
claimed gap is real. "STORY-091" was fabricated; caught only by adversary
absolute-path grep. Standing rule candidate: orchestrator verifies upstream
claims before routing deferrals; implementer self-disclosure of capability gaps
is NOT authoritative.

### HIGH

**HIGH-001 — Validation diagnostics dropped file:line:col span**

`ValidationFailed` diagnostics emitted by `SlideValidator` were not carrying
source spans through the build pipeline. Fixed by threading `DiagnosticSpan`
from validator output into `BuildError::ValidationFailed`, surfacing correct
file:line:col in miette rendering. Traces BC-1.15.001 PC-2.

**HIGH-002 — Multi-format duplicate diagnostic rendering (fixed via unified pipeline)**

Running `build` with multiple output formats caused duplicate diagnostic blocks
in stderr — one per format pass. Root cause: `build()` / `build_inner()` looped
over formats and invoked the full parse→eval→validate pipeline per format.
Resolution: unified pipeline (`compile_core` / `compile_inner`) runs once and
produces a `CompiledDeck`; format-specific export is a thin second pass. Single
diagnostic block regardless of format count. Traces BC-1.15.002 INV-2.

**HIGH-003 — `--json` hardcoded `total: 1`**

JSON output always emitted `"total": 1` regardless of actual diagnostic count.
Fixed by counting findings from the real diagnostic accumulator before
serialization. Traces BC-1.15.001 PC-4 (JSON fidelity).

**HIGH-004 — `exit_code_to_u8` fragile round-trip / silent fallback**

`exit_code_to_u8` used a match-on-integer pattern that silently fell back to 0
for unknown values produced by future variants. Replaced with a direct
`as u8` cast on the enum discriminant + a `debug_assert!` to catch out-of-range
values in test builds. Eliminates silent exit-0 on novel error kinds.
Traces BC-1.15.003 INV-3.

### MEDIUM

**F-P2-MED-001 — Cross-stage eval+validator accumulation (BC-1.15.002 INV-3)**

Non-fatal `eval` `Error`-level diagnostics (as opposed to `Fatal`) were silently
swallowed in `compile_inner` when eval succeeded with warnings. The fix gates
non-fatal eval errors through a `MultistageFailed` variant that carries both
eval diagnostics and any subsequent validator diagnostics, preserving full
accumulation. Traces BC-1.15.002 INV-3 (all errors reported).

**MED-001 — Two-phase all-or-nothing output atomicity**

Export writes were non-atomic: partial output files could persist if a
mid-multi-format export failed. Fixed by buffering all format outputs before any
`fs::rename()` and only committing all files atomically on full success.
Traces BC-1.15.003 PC-3 (atomicity invariant).

**HIGH-P3-001 / MED-P3-002 — BC-1.15.002 PC2 source-order interleave**

Diagnostics from parse, eval, and validation phases were emitted in pipeline
execution order rather than source file order (line:col ascending). Fixed by
introducing a `diag_util` sort/dedup helper called at the `compile_core`
boundary. Single canonical sort point; all downstream consumers (stderr render +
JSON render) receive pre-sorted diagnostics. Traces BC-1.15.002 PC-2.

### OBSERVATIONAL (Pass 4 — all resolved)

**OBS-P4-001 — Eval-side dedup missing**

Eval diagnostics were not deduplicated before merge with validator diagnostics.
Added dedup on eval output before passing to `diag_util` sort/dedup.

**OBS-P4-002 — Interleave dedup symmetry**

The dedup helper applied to stderr path but not JSON path, causing inconsistent
counts. Unified: dedup runs once at `compile_core` boundary, both paths receive
deduped list.

**OBS-P4-003 — JSON eval-severity fidelity**

Eval `Warning`-level diagnostics serialized as `"error"` in JSON output.
Fixed by mapping severity enum variants directly to JSON string labels.

**OBS-P4-004 — `render_build_error_to_json_value` purity + JSON tests**

The JSON rendering function had side effects (writing to a shared buffer)
making it untestable in isolation. Refactored to pure: takes diagnostics,
returns `serde_json::Value`. Added 6 JSON-specific unit tests covering all
severity levels and multi-diagnostic arrays.

### LOW

**LOW-001 — `watch` / `init` / `extract-brand` graceful not-implemented**

Subcommands planned for STORY-056 and STORY-057 were registered in the CLI
router but panicked on invocation. Replaced panics with structured
`BuildError::NotImplemented` returns that emit a user-facing message citing the
planned story anchors (STORY-056, STORY-057) and exit code 3.

## Structural Improvement: Unified Pipeline

The cascade drove a significant architectural improvement to `slideforge-cli`:

**Before:** two parallel code paths
- `build()` / `build_inner()`: used by ~13 e2e consumers; ran full pipeline per format
- `compile_core()` / `compile_inner()`: sketch; not wired

**After:** single canonical pipeline
- `compile_core()` / `compile_inner()` is the ONE pipeline:
  parse → eval → brand → validate → layout → `CompiledDeck`
- `build()` / `build_inner()` is a thin adapter:
  `compile_core()` + `export_format()` per requested format
- New public API surface: `compile()`, `export_format()`, `CompileOptions`,
  `CompiledDeck`

This eliminates duplicate diagnostic generation, ensures single-sort/dedup,
and establishes the foundation for incremental rebuild (STORY-059 / comemo).
Reinforces DRY and TD-VSDD-060 (sibling-site sweep on value changes).

## Open Follow-ups (non-blocking; recorded in BACKLOG Open Follow-ups)

| ID | Description | Anchor |
|----|-------------|--------|
| FU-055-OFFLINE-FLAG | `--offline` global flag declared in CLI arg struct but not threaded into the pipeline — no effect until data-source fetching exists. Non-behavioral today. | STORY-021 (data-source integration) |
| FU-055-VARIANT-NOTE | Variant selection now works end-to-end via `eval_deck_with_variant`; no further action. Note for traceability only. | Closed; traceability |
| FU-055-JSON-HASFATAL-DOC | Story spec L442-443 illustrative `--json` snippet shows `has_fatal:true` at exit 2, which differs from code's `has_fatal = exit1 || exit3` definition. Non-binding prose; optional trivial story-doc fix. | Story-spec prose only; low-priority doc fix |

## Merge Readiness

- Local adversarial cascade: CONVERGED 3/3 strict-CLEAN (passes 5, 6, 7 of 7 total)
- Feature branch: `feature/STORY-055`, HEAD `fcdd1c55`, rebased onto develop `fa85d1136`
- Pre-merge pending: demo recording (demo-recorder) + PR creation (pr-manager)
- No blocking issues

## Lessons / Process Notes

1. **LESSON-ORCHESTRATOR-VERIFY-UPSTREAM (process-gap, S-7.02 candidate):**
   When an implementer cites a story ID for a deferral ("STORY-091 not yet
   implemented"), the orchestrator must independently grep/confirm that story
   exists before accepting the deferral. Fabricated story IDs are a real
   failure mode — caught only by adversary absolute-path grep in this cascade.
   Self-disclosure of capability gaps by the implementer is NOT authoritative.

2. **LESSON-19 (SIBLING-SWEEP) reinforced:** Multi-format diagnostic rendering
   changes propagated through 6 files (build.rs, compile.rs, diag_util.rs,
   output.rs, json_render.rs, integration tests). A partial fix on build.rs
   alone reset the streak at pass 3. Full grep-map before each fix-burst is
   mandatory.

3. **Unified-pipeline structure (compile_core → export_format) is the canonical
   model for incremental rebuild in STORY-059.** New stories building on CLI
   must follow this two-phase contract.
