---
title: Slideforge Product Brief
version: 2.0
created: 2026-05-23
status: ACTIVE — sharded 2026-05-23 from PROJECT-SEED.md
license: MIT OR Apache-2.0 (dual)
links:
  seed: .factory/seed/PROJECT-SEED.md
  architecture: .factory/specs/architecture-overview.md
  dsl-spec: .factory/specs/dsl-spec.md
  slide-types: .factory/specs/slide-types-catalog.md
  conventions: .factory/specs/conventions.md
  risks: .factory/specs/risks.md
  success-metrics: .factory/specs/success-metrics.md
  decisions: .factory/specs/decisions-applied.md
  agent-instructions: .factory/specs/agent-factory-instructions.md
  samples: .factory/specs/sample-dsl-snippets.md
  visual-parity: .factory/specs/visual-parity-contract.md
---

# Slideforge Product Brief

> Sharded 2026-05-23. This slim brief is the entry point for all downstream agents.
> Each section below is a summary only — follow the "See:" links for the full content.
> Do NOT load this file and the linked files simultaneously; load only what your task requires.

---

## Section 1: Project Identity

**Name:** `slideforge` (confirmed 2026-05-23 — see decisions-applied.md Q1)

**Mission:** Build a presentation generator that reads structured slide specifications in a purpose-built DSL, renders branded `.pptx` output using a corporate brand template, supports 23 composable opinionated slide types, compiles fast and incrementally, reports errors helpfully, and distributes as a single binary.

**Tagline:** *"Branded presentations from a DSL, not a deck of dicts."*

**Target users:**
- Engineering teams generating regular branded reports and briefs from data
- Incident response teams producing executive briefs from structured event data
- Internal tools needing deterministic, version-controllable presentation output
- `python-pptx` users who want better tooling

**Non-goal:** Replacing PowerPoint as a manual authoring tool.

→ See: `/Users/jmagady/Dev/slideforge/.factory/seed/PROJECT-SEED.md` §1 for full name-candidate table and rationale.

---

## Section 2: Why This Exists

A working Python implementation exists in `.factory/seed/reference/` (23 slide types, ~1900 lines). We are rewriting it in Rust because: no error recovery, Python syntax is the wrong DSL, no incremental compilation, no type safety, single output target (PPTX only), distribution requires Python runtime, and no editor support.

We keep: the 23 slide types and their visual designs, brand template integration, data-driven philosophy, and opinionated slide-type vocabulary.

We change: DSL replaces Python dicts; compilation pipeline replaces direct rendering; incremental support; multi-target output (PPTX + PDF + HTML); single-binary distribution.

→ See: `/Users/jmagady/Dev/slideforge/.factory/seed/PROJECT-SEED.md` §2 for full rationale.

---

## Section 3: Architecture Overview (Abstract)

Four-stage pipeline (Parse → Evaluate → Layout → Export) borrowed from Typst. Each stage is a separate crate. Exporters are pluggable via a stable IR. Brand template handling mirrors the Python tool's approach. Crate workspace layout defined.

→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/architecture-overview.md`

---

## Section 4: Tech Stack (Abstract)

Key dependencies: `chumsky ^0.10` (parser), `ooxmlsdk ^0.6` (OOXML), `miette`/`ariadne` (diagnostics), `clap ^4` (CLI), `tracing` (logging). Rust edition 2024, MSRV 1.85 (subject to ADR-007). Full rejection rationale for alternatives documented.

→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/architecture-overview.md` §4

---

## Section 5: DSL Specification (Abstract)

Indentation-significant syntax (YAML/Python-like). 23 slide type keywords. Color vocabulary of 11 named brand colors. Validation rules with source spans. File extension `.sf`. Multi-file via `@include`.

→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/dsl-spec.md`
→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/slide-types-catalog.md` (the 23-type table)
→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/sample-dsl-snippets.md` (Appendix B samples)

---

## Section 6: Implementation Plan (Abstract)

Original seed §6 phases are preserved for context in architecture-overview.md but are SUPERSEDED by the scope expansion in decisions-applied.md §11.A. The Q4/Q5/Q7 decisions add ~2x scope vs. the seed's original estimates. The PRD (product-owner, Phase 1) is the authoritative phase plan.

→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/architecture-overview.md` §6 (superseded seed plan)
→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/decisions-applied.md` §11.A (scope expansion note)

---

## Section 7: Reference Materials

→ See: `/Users/jmagady/Dev/slideforge/.factory/seed/reference/` for Python implementation
→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/agent-factory-instructions.md` for how to use the reference

---

## Section 8: Conventions

→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/conventions.md`

---

## Section 9: Risks

→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/risks.md`

---

## Section 10: Success Metrics

→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/success-metrics.md`
→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/visual-parity-contract.md` (binding parity tolerance spec supersedes §10 criterion 1)

---

## Section 11: Open Questions

All 7 Open Questions resolved 2026-05-23. See decisions-applied.md for binding decisions and consequences.

→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/decisions-applied.md`

---

## Section 12: Agent Factory Instructions

→ See: `/Users/jmagady/Dev/slideforge/.factory/specs/agent-factory-instructions.md`
