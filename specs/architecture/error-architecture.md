---
document_type: architecture-section
section: error-architecture
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# Error Architecture

## Core Design (ADR-012, DI-018)

All errors accumulate in a single pass. The first error NEVER terminates processing
unless a parse error prevents further meaningful parsing. Users see all errors from
a single build, not one at a time.

## Error Taxonomy

Errors are classified by the PRD Section 5 taxonomy. Each error code has a severity:
- **Fatal (strict mode):** Build produces NO output. All accessibility errors (E-A11-NNN),
  evaluation errors (E-EVL-NNN), and data-source errors (E-DAT-NNN) are fatal.
- **Warning (warn-only mode):** Build produces output with error-slide placeholders
  for slides that could not be parsed or validated.

## Span Threading

Every error carries a `Span { file: Arc<str>, start: u32, end: u32 }` (byte offsets).
Spans flow from lexer through parser through evaluator through validator.

Span recovery in the chumsky pipeline (S4 finding):
- Lexer: byte-accurate `SimpleSpan` on every token (hand-written lexer).
- Parser: token-index spans from chumsky `Stream`; translated to byte spans via
  a `Vec<SimpleSpan>` span table carried alongside the parse.
- Evaluator: spans from AST nodes (preserved from parser output).
- Validator: spans from Deck IR field positions.

## Error Types

Each crate owns a `thiserror`-derived error enum:

| Crate | Error Enum | Category Prefix |
|-------|-----------|----------------|
| slideforge-syntax | `ParseError` | E-PAR-NNN |
| slideforge-eval | `EvalError` | E-EVL-NNN, E-DAT-NNN |
| slideforge-validate | `ValidationError` | E-A11-NNN, E-LAY-NNN, E-CFG-NNN |
| slideforge-brand | `BrandError` | E-BRD-NNN |
| slideforge-pptx | `PptxError` | E-EXP-NNN |
| slideforge-pdf | `PdfError` | E-EXP-NNN |

## CLI Rendering (miette)

The CLI (`slideforge-cli`) collects all errors and renders them via `miette` with:
- Colored source pointers (`fancy` feature)
- File:line:col attribution
- Correction hints per error
- Aggregated display: all errors shown before exit

No `println!` in library crates. All diagnostic output uses `tracing::*!` with
structured fields or the miette error chain. The CLI is the sole output point.

## Strict Mode vs Warn-Only Mode

- **Strict (default):** `DI-017` — no output files written on any validation error.
  The process exits non-zero with all errors displayed.
- **Warn-only (`--warn-only` flag):** Fatal errors become warnings; error-slide
  placeholders are inserted for slides that fail validation. Output is written.
  The process exits with a warning summary but exit code 0 (configurable).

## No `unwrap()` Rule (NFR-021)

Zero `.unwrap()` or `.expect()` in non-test code. All `Result`s propagate via `?`
with structured error variants. The `clippy::unwrap_used` lint is enabled on
non-test targets in CI (`-D clippy::unwrap_used`).
