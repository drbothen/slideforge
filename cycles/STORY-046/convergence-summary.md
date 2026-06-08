---
document_type: convergence-summary
level: ops
version: "1.0"
status: converged
producer: state-manager
timestamp: 2026-06-08T00:00:00
cycle: STORY-046
story: STORY-046
story_title: Static HTML Exporter (P4 Composite Rendering Model)
feature_branch: feature/STORY-046
feature_branch_head: 93cef9fb
merge_status: NOT_YET_MERGED — demo + PR pending
convergence_result: "3/3 strict-CLEAN (passes 21, 22, 23); total 23 passes"
traces_to: STATE.md
---

# Convergence Summary — STORY-046

## Result

**LOCAL adversarial cascade CONVERGED.** 3/3 strict-CLEAN (passes 21, 22, 23). Total 23 passes.

- Feature branch: `feature/STORY-046`
- HEAD at convergence: `93cef9fb`
- Merge status: NOT YET MERGED — demo recording + PR still pending.

---

## Rendering-Model Pivot During Cascade

The cascade surfaced and resolved a fundamental rendering-model decision for the HTML exporter.

### ADR-008 Amendment — P4 Composite Rendering Model

The final P4 model adopted during the cascade:

- Slide text is real positioned HTML elements (`<div>`, `<p>`, etc.) — NOT buried inside an SVG `<foreignObject>`. This preserves DOM accessibility and avoids the usvg/PDF `foreignObject` breakage identified as a HIGH finding.
- Graphical frames (charts, diagrams, decorative shapes) are rendered in a sibling `<svg role="presentation">` element, isolated from the text layer.
- The outer SVG coordinate wrapper is **NOT** `aria-hidden` — this was a Pass-3 ARIA correction. The graphical sibling SVG carries `role="presentation"` to suppress it from the a11y tree; the outer structural element must remain visible to AT.

ADR-008 amended in-place to record this decision. The `HtmlExporter` was registered in `crates/slideforge/src/registry.rs` via human-authorized decision (2026-06-08 HTML-seam decision).

### BC-4.03.003 → v1.4

Updated to reflect the P4 rendering model:

- Invariant 2 revised.
- PC-6 and PC-7 updated.
- **4-step heading chain** formalized: deck title (`<h1>`, visually hidden fallback for degenerate-bbox Title slides) → section heading (`<h2>`) → slide title (`<h3>`) → content heading (`<h4>`). The synthetic visually-hidden `<h1>` fallback for zero-height/missing title frames was added in response to CRIT-1 (Pass 14) and refined in Pass 20.

### STORY-046 → v1.3

Story spec updated to reflect the P4 model and heading-chain ACs.

### Export Architecture — v1.x update

`export-architecture.md` updated to describe the P4 composite model as the canonical HTML rendering approach.

---

## Key Findings Fixed Across the Cascade

The following are the structurally significant findings (1 CRIT, several HIGH/MED) resolved during the 23-pass cascade:

| Finding ID | Severity | Description | Pass Resolved |
|------------|----------|-------------|---------------|
| HIGH-B2 (role=img-canvas-hides-content) | HIGH | Original P2 model wrapped all slide content in a single `<svg role="img">` canvas — every text element became invisible to AT. Drove pivot to P4. | Pass 2 (model pivot) |
| HIGH-B2 (foreignObject-breaks-usvg-PDF) | HIGH | `<foreignObject>` in SVG is silently stripped by `usvg` during SVG processing; PDF exporter would lose all text. Confirmed P4 text-HTML-layer approach. | Pass 2 |
| F-7.01 (non-functional-axe-core-CI-gate) | HIGH | The axe-core CI gate used the wrong package API (`@axe-core/playwright` invoked as bare import, not as `checkA11y()`). Gate passed even when violations existed. Fixed: correct `injectAxe()` + `checkA11y()` call sequence. | Pass 7 |
| F-P9-001 (body-block-content-inside-h1) | MED | Slide body paragraphs were being nested inside the `<h1>` element. `<h1>` must contain only the slide title text. | Pass 9 |
| CRIT-1 / P14 (degenerate-bbox-Title-zero-h1) | HIGH/CRIT | Title frame with zero height (degenerate bbox) produced an `<h1>` with empty text-content, making the heading useless to AT and leaving the heading chain broken. Fixed: synthetic visually-hidden `<h1>` fallback injected when title text is absent or bbox collapses. | Pass 14 |
| P17 (decorative-alt-chart-diagram-empty-role-img) | HIGH | `chart:` and `diagram:` frames with `decorative: true` were emitting no accessible label and no `role`. Fixed: emit `<figure role="img" aria-label="">` with empty label to mark decorative intent. | Pass 17 |
| CRIT (subtitle-h1-h3-skip-on-title-slide) | CRIT | Title slide had heading structure `h1 → h3` (subtitle), skipping `h2`. Fixed: subtitle on a title slide is `h2`; the 4-step heading chain is slide-type-aware. | Pass 20 |

---

## Trajectory Shorthand

Passes 1–20 contained findings (exact per-pass counts not individually recorded in this summary).  
Passes 21, 22, 23: **0 findings** (strict-CLEAN).

---

## Process Lessons (Codification Candidates)

The following process gaps were identified during the cascade. Items tagged `[process-gap]` are candidates for Standing Process Rule codification.

### LESSON-20 (ADVERSARY-SPEC-PATHS) [process-gap]

**Pass-4 OBS-1** — the per-story LOCAL adversary dispatch was sent with relative spec paths. The per-story git worktree (`.worktrees/STORY-046`) does not contain the `.factory/` mount. The adversary could not read BC-4.03.003, ADR-008, or the export-architecture spec, and operated from memory / what was visible in the worktree.

**Rule:** Per-story LOCAL adversary dispatches MUST include ABSOLUTE `.factory/` spec paths (story file, BCs, ADRs) in the task. Minimum required: story spec path, traced BC paths, traced ADR paths, export-architecture path.

This is codified as **LESSON-20** in STATE.md Standing Process Rules.

### LESSON-19 (SIBLING-SWEEP) — Reinforced [process-gap]

**Passes 14, 17, and 20** all produced findings that were the same class: "fix was applied to some sibling arms or code paths but not all." Each time, the targeted fix covered the primary path but left sibling match arms, fallback branches, or alternate rendering paths un-updated.

This recurrence reinforces LESSON-19 (already in STATE.md). Specific examples from STORY-046:

- Pass 14: the visually-hidden `<h1>` fallback was added for the normal title path but not the degenerate-bbox path.
- Pass 17: `decorative: true` handling was fixed for `image:` frames but not `chart:` and `diagram:` frames.
- Pass 20: subtitle heading level was corrected on the `Content` slide type but not the `TitleSlide` type.

**Reinforcement prescription:** When the adversary finds a partial-propagation defect, the fix-burst MUST enumerate ALL sibling sites via grep before patching, and the implementer must confirm in their response that every sibling was addressed.

### Heading-order as highest-defect-density area [structural-lesson]

Heading-order accounted for 6 findings across the cascade (more than any other single area). The root cause was an incremental, per-slide-type heading-level assignment strategy that required each slide type to "know" its position in the heading hierarchy.

**Design prescription for future HTML exporters:** Design the heading-level system as a whole-deck computed traversal with a proven no-skip invariant from the start. Assign heading levels via a single pass over the full slide/section structure before rendering any individual slide, rather than per-type constants. This would have prevented the heading-chain findings at Passes 9, 14, and 20.

---

## Open Items — Anchored Forward (Non-Blocking)

These items were observed but are intentionally deferred to later stories. They do NOT block demo/PR/merge.

| Item | Anchor Story | Notes |
|------|-------------|-------|
| `sanitize_svg_for_graphics_layer` direct XSS unit test | STORY-047/048 | The sanitizer entry point needs a targeted test that exercises the production code path with a crafted XSS payload. Real chart/image SVG wiring lands in 047/048. |
| Entity-encoded `javascript:` href hardening in sanitizer | STORY-047/048 | Entity-encoded variants (e.g., `&#106;avascript:`) should be tested and blocked. Anchored to real SVG wiring story. |
| ColorBar body-path accessible-name in HTML exporter | STORY-047/048 | The body path for ColorBar (chart-backed) needs an accessible label when the chart frame has a non-empty title. Anchored to chart SVG wiring. |
| Stale `lib.rs` module-doc summary lines | Trivial doc sweep | Several module-level doc comments still reference the pre-P4 rendering approach (P2 phrasing). Trivial doc sweep — can be fixed in any subsequent burst. |

---

## Commit Authority

This convergence summary was written by the state-manager at human direction. No spec documents, source code, or review reports were written by the state-manager. The ADR-008 amendment, BC-4.03.003 v1.4 update, STORY-046 v1.3 update, and export-architecture update were authored by the appropriate specialist agents during the cascade itself.
