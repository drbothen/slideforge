---
document_type: domain-spec-index
level: L2
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/specs/product-brief.md
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q2-decision-final.md
  - .factory/planning/q3-decision-final.md
  - .factory/planning/q4-q15-decisions.md
  - .factory/planning/q16-q25-decisions.md
  - .factory/planning/domain-research.md
  - .factory/specs/architecture-overview.md
input-hash: "[pending compute-input-hash]"
traces_to: .factory/specs/product-brief.md
sections:
  - capabilities.md
  - entities.md
  - invariants.md
  - events.md
  - edge-cases.md
  - assumptions.md
  - risks.md
  - failure-modes.md
  - differentiators.md
  - ubiquitous-language.md
---

# L2 Domain Specification: slideforge

> **Sharded artifact (DF-021).** This index provides navigation and summary.
> Detail lives in per-section files listed below. Each section targets
> 800-1,200 tokens for optimal LLM consumption.

## Domain Summary

slideforge is a data-reactive branded document platform that compiles a single
indentation-significant DSL source (.sf files) into multiple output formats
(PPTX, DOCX, PDF, HTML, web preview) through a six-stage pipeline
(Parse → Evaluate → Validate → Brand → Layout → Export) with a plugin-first
architecture and compile-time accessibility enforcement.

---

## Document Map

| Section | File | Tokens | Primary Consumer | Purpose |
|---------|------|--------|-----------------|---------|
| Domain Capabilities | capabilities.md | ~1,100 | product-owner, architect, story-writer | CAP-001 through CAP-030 capability catalog |
| Domain Entities | entities.md | ~1,100 | architect, product-owner | Entity model with bounded contexts and relationships |
| Domain Invariants | invariants.md | ~900 | product-owner, architect | DI-001 through DI-022 business rules |
| Domain Events | events.md | ~850 | architect | Event triggers, preconditions, and outcomes |
| Edge Cases | edge-cases.md | ~1,000 | story-writer, test-writer | DEC-001 through DEC-020 domain-level edge cases |
| Assumptions | assumptions.md | ~950 | product-owner, test-writer | ASM-001 through ASM-014 with validation methods |
| Risks | risks.md | ~1,000 | product-owner, architect | R-001 through R-016 risk register |
| Failure Modes | failure-modes.md | ~950 | architect, test-writer | FM-001 through FM-018 runtime failure catalog |
| Differentiators | differentiators.md | ~800 | product-owner | Competitive differentiator-to-CAP-NNN mapping |
| Ubiquitous Language | ubiquitous-language.md | ~900 | all agents | Glossary of 40+ domain terms with precise definitions |

---

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| BC creation input | capabilities.md + invariants.md + edge-cases.md + assumptions.md + risks.md + differentiators.md |
| Architecture design input | capabilities.md + entities.md + invariants.md + events.md + risks.md + failure-modes.md |
| Story decomposition input | capabilities.md + edge-cases.md |
| Holdout scenario generation | assumptions.md + risks.md + failure-modes.md |
| NFR derivation | risks.md + failure-modes.md |
| Full domain review (adversary/spec-reviewer) | ALL sections |

---

## ID Registry Summary

| ID Format | Count | Range | Section |
|-----------|-------|-------|---------|
| CAP-NNN | 30 | CAP-001 to CAP-030 | capabilities.md |
| DI-NNN | 22 | DI-001 to DI-022 | invariants.md |
| DEC-NNN | 20 | DEC-001 to DEC-020 | edge-cases.md |
| ASM-NNN | 14 | ASM-001 to ASM-014 | assumptions.md |
| R-NNN | 16 | R-001 to R-016 | risks.md |
| FM-NNN | 18 | FM-001 to FM-018 | failure-modes.md |

---

## Priority Distribution

| Priority | Count | CAP Items |
|----------|-------|-----------|
| P0 (must-have) | 20 | CAP-001 through CAP-018 + CAP-029 + CAP-030 |
| P1 (should-have) | 10 | CAP-019 through CAP-028 |
| P2 (nice-to-have) | 0 | (none — all capabilities are at least P1) |

---

## Bounded Contexts (Domain Map)

Four bounded contexts integrate through two stable IR contracts:

```
[Authoring BC]  --Deck IR-->  [Layout BC]  --LaidOutDeck IR-->  [Export BC]
      |                                                               |
      +-----------> [Branding BC] --(Brand)------------------------>+
```

| Bounded Context | Core Language | Crates | Stable Contract |
|----------------|--------------|--------|----------------|
| **Authoring** | Deck, Slide, ContentBlock, Variable, Register | slideforge-syntax, slideforge-eval | Deck IR |
| **Branding** | Brand, Theme, Palette, Typography, Master | slideforge-brand | Brand struct |
| **Layout** | Frame, BoundingBox, EMU, TextFlow, Constraint | slideforge-layout | LaidOutDeck IR |
| **Export** | RenderTarget, Part, Package, ContentType | slideforge-pptx/docx/pdf/html/preview | Output bytes |

---

## Anchor Justifications (creators_justify_anchors)

All capabilities in capabilities.md are grounded in the following product brief sections and decision documents:

- **CAP-001 to CAP-009** (DSL + computation): grounded in product-brief.md §5, q1-decision-final.md §1 (computation rungs 1-9)
- **CAP-010 to CAP-014** (slide types + aliases): grounded in q1-decision-final.md §4, q2-decision-final.md Layers A+B
- **CAP-015 to CAP-017** (output formats): grounded in q1-decision-final.md §2 (5 output formats), product-brief.md §2
- **CAP-018 to CAP-019** (brand/template): grounded in q1-decision-final.md §10 (brand bridge), q4-q15-decisions.md Q4
- **CAP-020** (accessibility validation): grounded in q4-q15-decisions.md Q6, product-brief.md §2
- **CAP-021 to CAP-024** (plugin + validation): grounded in q3-decision-final.md §2 (10 surfaces), q4-q15-decisions.md Q6, Q7, Q8
- **CAP-025 to CAP-027** (workspace + packages + watch mode): grounded in q16-q25-decisions.md Q19, Q20, Q21, Q17
- **CAP-028** (DSL versioning): grounded in q16-q25-decisions.md Q16
- **CAP-029** (writing registers): grounded in q1-decision-final.md §3
- **CAP-030** (diagnostic reporting): grounded in q16-q25-decisions.md Q17, Q23

All invariants in invariants.md are grounded as domain rules, not implementation constraints, as documented per invariant entry.
