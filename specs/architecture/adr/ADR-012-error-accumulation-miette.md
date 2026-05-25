---
document_type: adr
adr_id: ADR-012
title: Error accumulation and miette rendering
status: accepted
date: 2026-05-24
spike_input: S4-chumsky-indentation-parser.md
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-012: Error Accumulation and miette Rendering

## Context

Iterative fix-one-error-at-a-time development cycles are identified as a major pain
point in competitor tools (R3 research). DI-018 mandates that ALL errors accumulate
in a single pass. The CLI must render errors with source pointers and correction hints.

## Decision

- **Lex phase:** hand-written lexer accumulates all errors; produces best-effort token stream.
- **Parse phase:** chumsky `recover_with` at field level and slide level; `Rich<Token>` errors.
- **Eval phase:** `slideforge-eval` accumulates all type errors and undefined variable errors
  without stopping at the first.
- **Validate phase:** `slideforge-validate` validators each return `Vec<Diagnostic>` (never fail-fast).
- **CLI rendering:** `miette` (v7, `fancy` feature) renders all accumulated errors with colored
  source pointers, byte-accurate spans, and correction hints.

## Consequences

**All errors = one miette report:** The CLI collects all `Diagnostic` items from all phases
and renders them together before exiting. Users see every error in one build.

**Strict mode DI-017:** When errors are present and `--warn-only` is NOT set, NO output
files are written. The process exits non-zero after displaying all errors.

**Warn-only mode:** Fatal errors are demoted to warnings; error-slide placeholders are
inserted; output is written. Exit code remains 0 (configurable via `--strict-exit`).

**Correction hints:** Each error type in the error taxonomy (PRD §5) must carry a correction
hint string. This is enforced at the error definition level (not in the renderer) — every
`thiserror` variant includes `#[error("... hint: ...")]` or a separate `.hint()` method.

**No `println!` in libraries (NFR-021):** Library crates emit structured `tracing::warn!/error!`
events. Only `slideforge-cli` converts them to terminal output. This enables programmatic
consumers of the library to receive errors as structured data without screen-scraping.

**chumsky error types:** `extra::Err<Rich<Token, SimpleSpan>>` provides structured error
context (expected/found, span, label) that maps cleanly to miette's `LabeledSpan` API.
