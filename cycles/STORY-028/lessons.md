# Wave 3 Batch 2 Lessons Learned

Archived from STATE.md on 2026-05-30. Covers STORY-030, STORY-023, STORY-020, STORY-028 cascades.

## Process Lessons

- **TD-VSDD-059 paper-fix detection.** STORY-030 Pass 11 caught an OnceLock that was actually dead code — font field still reparsed every call. Without the explicit invariant test (parse-counter assertion), the regression would have shipped.

- **Implementer scope discipline.** STORY-023 Pass 13 fix burst over-applied `#[non_exhaustive]` to user-facing TOML schema structs, creating Pass 14 CRIT. Future implementer dispatches should specify exact targets and warn against over-extension.

- **Platform asymmetry in local adversary.** STORY-034 took 4 post-convergence CI iterations because local 3-CLEAN ran on macOS; Mermaid's Trebuchet MS font-family couldn't match on Linux CI without fonts-liberation + usvg font_resolver fallback. Process-gap candidate: local adversary should include Linux-container test pass for font/text/rendering code paths.

- **Spec-code drift accumulates.** STORY-023 Pass 13 found 3 CRIT spec-vs-code drift items (E-BRD-007 undocumented, E-BRD-005 retire/un-retire, E-BRD-002 PPTX/TOML→PPTX/DOCX). All required factory commits to error-taxonomy.md to fix.

- **Sibling-site sweep (TD-VSDD-060) is high-yield.** Pass 14 STORY-030 + Pass 17 STORY-023 both found missing `#[instrument]` on entrypoints by comparing against sibling crates. Should be a standard adversary axis.

- **Dual-path domain-object construction symmetry (STORY-023 PR-level F1).** Two production code paths (synthesizer vs loader) constructing the same domain type (BrandPalette) from the same template diverged on slot mapping — visual parity violation. Pattern: when two code paths produce the same domain object, the type itself should encode invariants or a shared constructor should be the only path. Future adversary axis: dual-path domain-object construction symmetry.

- **AKM compounding-novelty confirmed over 29+32-pass cascades (STORY-020 + STORY-028).** Each fresh-context pass surfaced new defect classes — paper-fixes, sibling-sweep gaps, semantic anchoring drift, BC version propagation, doc-vs-code precision. Real defects found across the full run. Production-grade canonical principle honored throughout — no MVP deferrals; all findings closed or surfaced as follow-up stories (STORY-072, STORY-073, STORY-074).

- **Sibling-sweep recurrence pattern (BC version bumps).** Every BC version bump triggered propagation work to story-spec body + code comments — three rounds of v1.x.x bumps required three sweeps. Going forward: when bumping a BC version, automatically dispatch implementer for code-comment sweep + story-writer for spec-body sweep + grep-all-.factory/-and-crates audit before declaring fix-burst complete.

- **Implementer overclaim pattern (TD-VSDD-059 at agent-process level).** Pass-27 implementer for STORY-020 claimed 33 test renames in 4 files; adversary verified only 8 in 1 file. Cross-story scope creep when implementer extends scope without orchestrator authorization. Recovery: orchestrator MUST verify git log against implementer's claimed file count before declaring a fix-burst closure.

- **Compounding-novelty value persists past pass 28.** Even after 28 clean passes, pass 29 on STORY-020 found a real internal contradiction (AC-008 citation contradicting 12-variant claim in the same docstring). Fresh-context audits never reach 'done' but each pass narrows the defect space.

- **Convergence is asymptotic, not absolute.** STORY-020 took 29 passes; STORY-028 took 32. Each pass found 1-5 new findings in the late phase. After 26+ passes the LOW findings became progressively cosmetic. The PR-merge gate (zero CRIT/HIGH/MED) is the canonical merge criterion; strict 3-CLEAN is the convergence criterion for adversarial cascade closure.

- **Sibling-sweep recurrence is systemic.** Every BC version bump during STORY-028 cascade generated new sweep work across spec body + code comments + tests. Pattern: orchestrator should automatically dispatch implementer for code-comment sweep AND story-writer for spec-body sweep AND grep-all-perimeter audit before declaring any BC bump complete.
