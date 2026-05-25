---
document_type: holdout-scenario-index
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
traces_to: .factory/specs/prd.md
---

# Holdout Scenario Index — slideforge v1.0

> **HIDDEN FROM IMPLEMENTER.** These scenarios are used exclusively by the
> holdout-evaluator agent during Phase 4 evaluation. They test non-obvious
> edge cases, integration boundaries, and assumption validation.
> Scenarios are not referenced in story specs or PRD supplements.

---

## Summary

| HS-ID | Title | Type | ASM/R/FM Source | Must-Pass |
|-------|-------|------|----------------|----------|
| HS-001 | DSL user acceptance — engineering team produces Q3 report deck from scratch | acceptance | ASM-001 | YES |
| HS-002 | PPTX multi-renderer fidelity — Python reference deck re-authored in .sf | multi-renderer | ASM-002 | YES |
| HS-003 | HTTP data source schema change during watch mode | watch-mode resilience | FM-005, DEC-018 | YES |
| HS-004 | Empty @for collection — incident report with no incidents | data edge case | DEC-001 | YES |
| HS-005 | Circular @include detection — report with template library | error detection | DEC-003, FM-002 | YES |
| HS-006 | Brand synthesis from sparse brand.toml — only 2 colors declared | brand resilience | DEC-016 | NO |
| HS-007 | Mermaid diagram in PPTX — architecture diagram slide | diagram integration | ASM-003 | YES |
| HS-008 | PDF/UA-1 compliance — complex deck with math, charts, diagrams | accessibility | ASM-007, R-003 | YES |
| HS-009 | known-good corpus — seed reference deck translated to .sf | false positive rate | ASM-001 | YES |
| HS-010 | Known-problematic corpus — deck with multiple accessibility violations | error detection rate | DI-001, DI-002 | YES |
| HS-011 | Performance gate — 25-slide deck with 5 @data sources < 500ms | performance | ASM-011, R-005 | YES |
| HS-012 | Variant-based export — exec vs technical audience | multi-variant | CAP-007 | NO |
| HS-013 | SSRF mitigation — HTTP data source with allowlist | security | R-011 | YES |
| HS-014 | python-pptx migration — existing Python dict deck rewritten in .sf | migration UX | ASM-008 | NO |
| HS-015 | Watch mode HTTP failure — API goes down mid-session | resilience | FM-005, DEC-007 | NO |

Must-Pass count: 11 of 15 scenarios.
Target: ≥ 0.60 must-pass pass rate = at least 7 of 11 must-pass scenarios pass.

---

## Evaluation Protocol

All holdout scenarios are evaluated by the holdout-evaluator agent, which:
1. Receives the scenario input without seeing the implementation details.
2. Runs `slideforge build` (or `slideforge watch`) with the scenario inputs.
3. Verifies the postconditions listed in each HS file.
4. Scores: PASS (all postconditions met), FAIL (any postcondition not met), or PARTIAL.

Must-pass scenarios that FAIL constitute a v1.0 release blocker.
