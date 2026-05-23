---
title: Slideforge Agent Factory Instructions
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.0
created: 2026-05-23
status: SEED-EXTRACT
---

# Agent Factory Instructions

> Source: §12 (Agent Factory Instructions) and Appendix A (Visual Parity Test Plan)
> from PROJECT-SEED.md. Verbatim extraction.
> NOTE: Visual parity test plan in Appendix A is supplemented and superseded by
> `visual-parity-contract.md` for CI tolerance definitions.

---

## Section 12: Agent Factory Instructions

This section is meta — instructions for how the agent factory should approach this seed.

### Recommended agent assignment

| Phase | Suggested agent team |
|-------|---------------------|
| Phase 0 | 1 scaffolding agent |
| Phase 1 | 1 parser agent + 1 PPTX exporter agent + 1 integration agent |
| Phase 2 | 1 agent per slide type (23 parallel), 1 review agent |
| Phase 3 | 1 brand template agent + 1 validator agent + 1 diagnostic-rendering agent |
| Phase 4 | 1 incremental compilation agent + 1 formatter agent + 1 doc-generation agent |
| Phase 5 | 1 release-engineering agent + 1 FFI agent per language |

Each agent should:
1. Read this seed in full before starting
2. Read the relevant reference files in `./reference/`
3. Produce a self-contained PR that passes CI
4. Include tests with their implementation
5. Update `CHANGELOG.md`

### Things the factory must NOT do

- **Do not invent slide types.** The 23 types are the contract. New types require human approval and design review.
- **Do not redesign the visual style.** Brand colors and font sizes are fixed.
- **Do not change the DSL syntax mid-phase.** If syntax issues emerge, raise to a human and pause.
- **Do not skip tests.** Every public-API change needs a test.
- **Do not modify `./reference/`.** It's the source of truth for visual behavior.
- **Do not pick the project name.** Phase 0 starts with a placeholder until confirmed. (NOTE: Name is now confirmed as `slideforge` — see decisions-applied.md Q1.)

### Things the factory must DO

- Ask for clarification when an Open Question (Section 11) is reached. (NOTE: All 7 are resolved — see decisions-applied.md.)
- Snapshot every visual output decision (each slide type rendered to XML, committed)
- Maintain visual parity with the Python tool (per the contract in visual-parity-contract.md)
- Optimize for clarity in DSL syntax over implementation cleverness
- Document every public API with rustdoc

---

## Appendix A: Visual Parity Test Plan

> NOTE: This is the original seed test plan. The binding CI tolerance spec is in
> `visual-parity-contract.md`. The steps below are preserved as the manual QA process
> that runs at each phase gate in ADDITION to automated snapshot tests.

To verify visual parity with the Python reference, the factory should:

1. **Build the reference deck via Python:** `uv run python scripts/build-incident-brief.py scripts/incident_data/mss_metrics_leadership.py` → `MSS-Metrics-Leadership-Brief.pptx`
2. **Build the equivalent deck via slideforge:** `slideforge build examples/mss-metrics.sf --template templates/1898.pptx` → `slideforge-output.pptx`
3. **Compare:**
   - Open both in PowerPoint side by side
   - Verify slide-by-slide visual equivalence
   - Verify slide count matches (25 slides)
   - Verify takeaway bars render
   - Verify talk tracks appear in speaker notes
   - Verify brand chrome (logo, page numbers, confidential markers) appears correctly

Pixel-perfect equivalence is NOT required. Brand-consistent and semantically-equivalent output IS required.

For precise tolerance definitions (position ±4pt, exact RGB, etc.) see `visual-parity-contract.md`.
