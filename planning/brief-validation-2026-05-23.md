---
title: Brief Validation Report — slideforge
date: 2026-05-23
verdict: NEEDS-WORK
analyst: business-analyst
document_type: brief-validation
level: ops
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-23T00:00:00Z
phase: 1a
inputs:
  brief: .factory/specs/product-brief.md
  brief_hash: 9f3f37584767
  brief_size_kb: 38.4
  brief_line_count: 836
traces_to: .factory/specs/product-brief.md
---

# Brief Validation — slideforge

## Verdict: NEEDS-WORK

The brief contains strong substance — the product mission, user personas, competitive
context, and core capability set are all grounded and specific. However it is
OVER_SPECIFIED at ~10,000–12,000 tokens (7–8x the recommended 1,500-token agent
context budget), contains extensive implementation leakage in core sections, and
has two NFR definitions that are not yet precise enough to become measurable
acceptance criteria. No BLOCKING gaps exist in substance; the BLOCKING issues are
context engineering problems that will degrade architect and product-owner agent
performance if not addressed before Phase 1 begins.

---

## Section Assessment

| Section | Status | Finding |
|---------|--------|---------|
| Mission / What Is This? | PASS | §1 mission statement is crisp: 6 numbered capabilities, clear tagline, specific non-goal ("not a WYSIWYG editor"). |
| Who Is It For? | PASS | §1 lists 4 specific personas with implicit pain + workaround (engineering teams generating branded reports, incident response teams, internal tools, python-pptx migrants). Pain is confirmed by market intel §3. |
| Scope — In Scope | PASS | 23 slide types, DSL pipeline, multi-format output (PPTX/PDF/HTML), single-binary distribution, brand-template-first. Well-bounded. |
| Scope — Out of Scope | PASS | §1 explicitly states "not replacing PowerPoint as manual authoring tool." §11.A.1 explicitly defers pyo3 dict API. §6 defers Homebrew/distribution to Phase 5 (pre-amendment). |
| Success Criteria | WEAK | §10 has 5 criteria. Criteria 1, 3, 4, 5 are measurable. Criterion 1 ("indistinguishable to reviewer") is subjective — see Visual Parity section below. Criterion 5 ("basic LSP for autocomplete") is marked as "bonus" with no acceptance threshold defined. |
| Constraints & Integration Points | PASS | §4 (required dependencies with rationale), §8 (code style, error handling, testing, documentation, release conventions), §11.A.1 (quality bar). All constraints are actionable — omitting any would cause implementation failure. |
| Open Questions Resolution | PASS | All 7 open questions resolved in §11.A. See Decision Resolution check below. |
| Risks | PASS | §9 risk register has 8 entries with impact/likelihood/mitigation. Supplemented by market-intel §5 (8 additional risks) and preflight risks (7 entries). Comprehensive. |
| Phase Plan | BLOATED | §6 phased plan is valid context for a human reviewer but is ~800 words of implementation sequencing that belongs in the PRD, not the brief. The scope-expansion note in §11.A correctly warns the PRD must re-scope it, but leaving §6 intact creates a contradiction: agents will read §6 as authoritative and may not notice the §11.A override. |
| Tech Stack Detail | BLOATED | §4 dependency table with "Why not" rejection rationale (~400 words), §3 full crate layout, pipeline ASCII art, and brand-template-loading pseudocode are architecture-level decisions that belong in the architecture document, not the brief. |
| DSL Specification | BLOATED | §5 (~700 words) includes the full reserved-keyword list, color vocabulary, all 23 slide type DSL keywords, validation rules, and three full sample DSL files in Appendix B. This level of specification belongs in the L3 architecture or a dedicated DSL spec file. |
| Agent Factory Instructions | BLOATED | §12 is meta-instructions to the agent factory. This is appropriate content for CLAUDE.md or the pipeline README, not the brief itself. At ~350 words, it adds significant token cost to every brief-read. |
| Visual Parity Test Plan | BLOATED | Appendix A is a 10-step manual QA plan. This belongs in the test specification or the PRD acceptance criteria, not the brief. |

---

## Bloat Score

**Estimated tokens:** ~10,500–12,000 / 1,500 recommended max — OVER (7–8x)

The brief encompasses what should be 4–5 separate documents:

| Logical document | Current location | Target location |
|-----------------|-----------------|----------------|
| Core brief (mission, users, scope, success, constraints) | §1, §10, §11 | `product-brief.md` (keep) |
| Architecture overview (pipeline, crate layout, brand loading) | §3, §4 | `architecture-overview.md` |
| DSL specification (syntax, keywords, colors, types) | §5 | `dsl-spec.md` |
| Phase plan / PRD scaffold | §6 | PRD (product-owner to author) |
| Agent factory meta-instructions | §12 | CLAUDE.md or pipeline README |
| Visual parity test plan | Appendix A | Test spec / PRD acceptance criteria |
| Sample DSL files | Appendix B | `examples/` directory (already planned) |

**Recommendation:** Shard into the structure above before Phase 1 begins. The core
brief (§1, §2 summary, §10, §11 decisions) would be ~1,200–1,500 tokens — well within
budget. Link to sibling documents for depth. Each downstream agent (architect,
product-owner, story-writer) loads only what it needs.

---

## Decision Resolution Check

All 7 decisions in §11.A are evaluated below.

| Q | Decision | Concrete? | Testable? | Assessment |
|---|----------|-----------|-----------|------------|
| Q1 | Name: `slideforge` confirmed | YES | YES — crate names match | PASS |
| Q2 | Indentation-significant DSL; chumsky semantic-indentation | YES | YES — grammar produces parse errors for misindented input | PASS |
| Q3 | `@include "path.sf"` directives; affects parser, eval, project config | YES | YES — cycle detection + cross-file span tests | PASS |
| Q4 | Bidirectional bridge: both `.pptx` and `.toml` accepted; `slideforge extract-brand` command; full synthesis from `.toml` only | YES | YES — acceptance: given only `brand.toml`, output opens in PowerPoint/Keynote/Google Slides/LibreOffice | PASS (scope expansion is captured but very dense) |
| Q5 | All 3 exporters (PPTX + PDF + HTML) in v1.0; PDF backend is open ADR-003 | YES | YES — gated on ADR-003 decision | PASS |
| Q6 | DSL-only; Python shims out to CLI binary; pyo3 deferred indefinitely | YES | YES — no pyo3 crate in v1.0 deliverable | PASS |
| Q7 | Typst-style web preview: embedded server + websocket + canvas renderer | YES | PARTIAL — "canvas-based renderer (likely reusing the HTML exporter)" has the word "likely" — this is still an open design question, not a closed decision. The architect needs a firmer direction here before ADR-005 can be written. | WEAK |

**Q7 flagged:** The phrase "likely reusing the HTML exporter from Q5 as the underlying renderer" is
a hypothesis, not a decision. If the HTML exporter produces static HTML that is not
canvas-renderable for real-time preview, the web-preview architecture is
underspecified. ADR-005 must resolve this: is the canvas renderer the HTML exporter
output, a separate renderer, or a hybrid? The brief should be updated to state the
decision or explicitly name it as the ADR-005 open question.

---

## NFR Testability Check

| NFR | Current definition | Precise enough? | Issue / Recommended tightening |
|----|-------------------|----------------|-------------------------------|
| Build performance: < 500ms cold, < 50ms incremental | "< 500ms cold build for 25-slide deck enforced in CI as benchmark gate (criterion + bench regression check); incremental rebuild < 50ms" | YES | Measurement context is clear (criterion, 25-slide deck). Wall-clock vs. CPU time should be specified (recommend: wall-clock on CI runner — document baseline runner spec in CI config). |
| Visual parity | "indistinguishable to a reviewer who didn't know it was rebuilt" (§10) + "semantic and brand-consistent output is required" (§6 Phase 2) | NO — conflicting definitions | §10 says "indistinguishable"; §6 says "pixel-perfect parity is not required; semantic and brand-consistent." These two definitions are contradictory. An agent implementing snapshot tests cannot satisfy both simultaneously. Requires a single authoritative definition with explicit tolerance. See Visual Parity section below. |
| Multi-renderer parity | "Synthesized .pptx must render correctly in PowerPoint (Office), Keynote, Google Slides, LibreOffice — verified via automated rendering + visual diff in CI" | WEAK | "Render correctly" and "visual diff" are undefined. What diff threshold triggers a CI failure? Pixel-identical is impossible across renderers. Needs: allowed variance definition (e.g., "no structural corruption, no text truncation, no missing elements; pixel diff < X% of slide area per renderer"). |
| WCAG AA on web preview | "web preview audited against WCAG AA via accessibility-auditor on every PR touching the preview" | WEAK | "accessibility-auditor" is unnamed. Preflight confirms no Rust-native WCAG AA library exists. Tooling must be specified (pa11y, playwright+axe, or lighthouse CLI) before a story can be written. Architect must add this to the ADR set or to an explicit Quality Bar footnote. |
| Holdout eval | "Mean satisfaction >= 0.85, must-pass >= 0.6" | YES | Thresholds are numeric. Method (holdout scenario set definition) is factory-standard — acceptable. |
| Spec convergence | "3 clean adversarial passes on PRD + architecture before Phase 2 starts" | YES | "Clean pass" = adversary finds no new issues. Measurable via adversarial-review skill output. |
| Mutation testing | "cargo-mutants mutation testing in CI with documented score budget" | WEAK | Score budget is not documented in the brief. "Documented score budget" defers the definition. Must be filled before Phase 6 hardening story is written. Recommend: add a specific target (e.g., >= 80% mutation kill rate) to the Quality Bar table. |
| Kani proofs | "Kani proofs for pure-core functions in slideforge-syntax and slideforge-eval" | WEAK | "Pure-core functions" is not enumerated. The architect must identify which function signatures are Kani-amenable and list them in the architecture doc. The brief should state that the architecture doc must include a Kani coverage target. |

---

## Visual Parity Contract Check

**Current state:** Two contradictory definitions exist in the same document.

- §10 (Success Criteria, v1.0 gate): "indistinguishable to a reviewer who didn't know it was rebuilt"
- §6 Phase 2 (Acceptance): "Pixel-perfect parity is not required; semantic and brand-consistent output is required"
- §6 Phase 3 (Acceptance): "indistinguishable from the Python tool's current output"

These three statements cannot all be satisfied simultaneously. "Indistinguishable" requires
pixel equivalence; "semantic and brand-consistent" explicitly permits visual deviation.

The brief does not define:
- Positional tolerance (e.g., element position within ±N pts)
- Font rendering equivalence (identical font metrics, or same font family/size/weight sufficient?)
- Color tolerance (exact RGB match, or visually indistinguishable on screen?)
- Cross-renderer tolerance (PowerPoint vs. Keynote vs. LibreOffice may render identically-formatted XML differently — which renderer is the ground truth?)

**Required addition:** A single authoritative visual parity definition. Recommended wording:

> Visual parity: the Rust output is structurally equivalent to the Python reference output.
> Structural equivalence means: same slide count, same slide type per position, same text content,
> same brand color assignments, same layout regions (header/body/takeaway/footer present or absent
> per type), same speaker notes content. Pixel-level rendering differences caused by renderer
> (e.g., sub-pixel font hinting) are accepted. Element position deviation <= 4pt from reference
> is accepted. Color values must match exactly (same RGB as brand palette definition).
> PowerPoint (Office 365, macOS) is the ground-truth renderer for parity assessment.

Without this definition, snapshot tests will be written against an ambiguous target.

---

## Additional ADRs Identified

The brief and preflight identify ADR-001 through ADR-006. The following additional
architectural decisions are buried in the brief and require formal ADRs:

| Proposed ADR | Source | Reason an ADR is needed |
|-------------|--------|------------------------|
| ADR-001 | §11.A Q4, preflight | Brand synthesis: full OOXML from-scratch generation — already proposed |
| ADR-002 | §11.A.1, preflight | Multi-renderer snapshot test matrix — already proposed |
| ADR-003 | §11.A Q5 | PDF backend choice — already proposed |
| ADR-004 | §11.A Q3 | @include resolution semantics — already proposed |
| ADR-005 | §11.A Q7 | Web-preview architecture — already proposed |
| ADR-006 | §11.A Q4/Q5 | IR shape across exporters — already proposed |
| **ADR-007** | §4, preflight §2 | **MSRV and ooxmlsdk version policy** — bump workspace MSRV from 1.85 to 1.88 (required by ooxmlsdk >= 0.4.0); pin chumsky to 0.13; define ongoing dependency version update cadence. Preflight flags this as HIGH severity. Must be decided before any crate implementing ooxmlsdk is started. |
| **ADR-008** | §5 DSL + §11.A Q7 | **Canvas renderer architecture** — is the web-preview canvas renderer the HTML exporter's output piped to a browser canvas, a separate IR-to-canvas renderer, or a SVG-intermediate approach? This is distinct from ADR-005 (server architecture) and determines whether the HTML exporter crate must be designed for dual-use (static export + live preview). |
| **ADR-009** | §8, §4, preflight | **Error recovery semantics** — the brief states chumsky provides error recovery ("partial AST") but does not specify: (a) what IR is produced when the parse is partial, (b) whether the evaluator runs on a partial AST, (c) whether layout/export attempt on a partial IR is always, never, or conditionally attempted. This affects the entire pipeline's failure contract and must be decided before Phase 1 parser design. |
| **ADR-010** | §4 pluggable exporters + §11.A Q4 | **Plugin/extension mechanism** — §3 states "adding a new output format means adding a new exporter crate, not modifying the layout engine" and that the IR is "the stable internal contract." But: (a) is the IR a public API (semver-stable across crate versions)? (b) Is there a first-class plugin interface for third-party exporter crates? (c) Is the `slideforge-ffi` crate the intended external exporter surface, or is there a separate plugin ABI? No ADR exists for this. |
| **ADR-011** | §11.A.1 Quality Bar, preflight | **WCAG AA tooling approach** — preflight confirms no Rust-native WCAG AA library exists. The accessibility gate requires an external tool (pa11y, playwright+axe, lighthouse CLI). Adds a Node.js toolchain dependency to CI. Must be decided before the web-preview story is decomposed, as it affects both CI design and the HTML exporter's output requirements. |

**Total ADRs recommended:** 11 (6 existing + 5 new)

---

## Risks

| Risk | Severity | Impact if not addressed before Phase 1 |
|------|----------|----------------------------------------|
| **Brief is 7–8x over context budget** | HIGH | Architect and product-owner agents will front-load ~10k tokens of brief context, leaving limited budget for reasoning. Errors, missed constraints, and incomplete spec coverage are the predictable failure mode. |
| **§10 vs §6 visual parity contradiction** | HIGH | Snapshot test authors will implement against an ambiguous contract. Phase 2 will produce snapshot tests that either over-constrain (pixel-perfect, causing false failures) or under-constrain (semantic-only, missing real regressions). Both outcomes require expensive rework. |
| **Q7 web-preview "likely" wording** | MEDIUM | ADR-005 cannot be closed until the architect resolves whether the HTML exporter is the canvas renderer. If the HTML exporter is later found unsuitable for live canvas rendering (e.g., it emits static HTML with no real-time redraw mechanism), Phase 4 (watch mode) requires a major scope revision. |
| **ooxmlsdk PPTX coverage unknown** | HIGH | Preflight flags this as HIGH. If ooxmlsdk cannot handle slide masters, theme XML, or notes master at the depth required for brand synthesis (ADR-001), the entire PPTX exporter must fall back to raw OOXML via `zip + quick-xml`. This is a significant implementation risk that should be resolved by a Phase 1 spike, not discovered in Phase 2. |
| **MSRV conflict blocks ooxmlsdk integration** | HIGH | Any story that adds ooxmlsdk as a real dependency will break the MSRV CI job until ADR-007 is closed. If not resolved before Phase 1 stories begin, the CI will be red from the first implementation PR. |
| **Mutation testing score budget undefined** | MEDIUM | The Phase 6 `cargo-mutants` gate cannot be configured in CI until a score target is set. Leaving it undefined allows the gate to be gamed (set low) or arbitrary (set too high). |
| **Plugin/IR stability undefined (ADR-010)** | MEDIUM | If the IR is implicitly treated as stable by third-party crate authors before it is formally declared stable, breaking IR changes post-v1.0 will be painful. Design the IR stability contract in Phase 1, not after. |

---

## Recommended Actions

### BLOCKING (must resolve before Phase 1 begins)

1. **Shard the brief.** Extract §3, §4, §5, §6, §12, and Appendix A/B into separate companion
   documents (`architecture-overview.md`, `dsl-spec.md`, `examples/`). Slim the brief to its
   core: §1 mission/users/scope, §10 success criteria, §11 decisions, §11.A.1 quality bar.
   Target: < 1,500 tokens.

2. **Resolve the visual parity contradiction.** Remove the "indistinguishable" language from §10
   or rewrite it to match §6's "semantic and brand-consistent" definition. Add a precise
   tolerance spec (position ±4pt, color exact RGB, same slide count and type, same text content,
   PowerPoint as ground truth renderer). This must be in the brief before the architect writes
   snapshot test contracts.

3. **Close Q7 web-preview wording.** Remove "likely" from §11.A Q7. Either declare the HTML
   exporter is the canvas renderer source and update ADR-005 scope accordingly, or explicitly
   list "canvas renderer architecture" as ADR-008 open. The brief cannot leave a v1.0 scope item
   in a probabilistic state.

4. **Add ADR-007 (MSRV/ooxmlsdk) to the ADR candidate list.** This is a Phase 1
   blocker identified by preflight. The brief currently omits it from the ADR list.

### SUGGESTED (improves quality but not blocking)

5. **Add mutation testing score budget to the Quality Bar table.** Recommend >= 80% mutation
   kill rate for `slideforge-syntax` and `slideforge-eval` crates. Without a number, the gate
   is unconfigurable.

6. **Add ADR-008 (canvas renderer), ADR-009 (error recovery semantics), ADR-010 (plugin/IR
   stability), ADR-011 (WCAG AA tooling) to the ADR candidates list.** These are genuine
   Phase 1 architect decisions that will surface as conflicts if not pre-named.

7. **Define the "basic LSP" bonus criterion in §10.** Either promote it to a v1.0 gate with a
   measurable threshold (e.g., "autocomplete for slide type keywords in VS Code extension") or
   formally mark it as v1.x deferred. "Bonus" is not a decision.

8. **Specify §10 Criterion 4 more precisely.** "A developer with no prior context can author a
   working `.sf` file using only `docs/dsl-reference.md`" is a usability criterion but has no
   measurement method. Add: "Validated by a usability test with one internal developer who has
   not seen the DSL; success = produces a syntactically valid 3-slide `.sf` file within 30
   minutes."

9. **Confirm the §6 Phase plan is explicitly superseded.** Add a banner to §6 (or remove it)
   stating: "This phased plan is superseded by the scope expansion in §11.A. The PRD (authored
   by product-owner in Phase 1) is the authoritative phase plan." Without this, agents reading
   §6 may treat it as current.

---

## Sources / Cross-References

- `.factory/specs/product-brief.md` (836 lines, 38.4 KB, hash `9f3f37584767`)
- `.factory/STATE.md` (Quality Bar table, ADR-001..ADR-006 list)
- `.factory/planning/market-intel-2026-05-23.md` (GO verdict, competitive analysis)
- `.factory/preflight/preflight-2026-05-23.md` (PASS-WITH-NOTES, 3 findings: MSRV conflict, notify version, chumsky version gap)
