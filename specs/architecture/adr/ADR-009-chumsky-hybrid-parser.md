---
document_type: adr
adr_id: ADR-009
title: Parser via chumsky 0.10 hybrid approach
status: accepted
date: 2026-05-24
spike_input: S4-chumsky-indentation-parser.md
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-009: Parser via chumsky 0.10 Hybrid Approach

## Context

The slideforge DSL is indentation-significant (like Python). Error accumulation (DI-018)
is a first-class requirement: users must see all parse errors, not just the first. S4 spike
confirmed chumsky 0.10 is viable with a hybrid architecture. 14/14 spike tests pass.
Parse time for 25 slides: 89µs — four orders of magnitude below the 500ms NFR.

## Decision

Use a hybrid architecture:
1. **Hand-written indentation lexer** producing `Vec<(Token, SimpleSpan)>`.
2. **chumsky 0.10 parser** consuming the token stream via `Stream`.

## Consequences

**Indentation lexer (hand-written):**
- Python-style INDENT/DEDENT token synthesis using an indent stack.
- Spaces-only: tab in indentation produces `LexErrorKind::TabInIndent` with byte-accurate
  span (VP-001 Kani proof candidate).
- Lex errors accumulate independently; the lexer continues producing a best-effort token stream.
- Numeric literals preserved as strings: `Float("1.10")` not `f64(1.1)` (DI-004).

**chumsky parser:**
- `extra::Err<Rich<Token, SimpleSpan>>` for structured error types.
- Span table lookup: `Vec<SimpleSpan>` carried alongside parse to translate token-index
  spans to byte-offset spans in error messages. This is a 2-3 line overhead per parse
  invocation — acceptable.
- `recover_with(skip_then_retry_until(...))` at field level and block level for error accumulation.
- Slide-level recovery: `skip_then_retry_until(any(), one_of([slide_kw, Token::Eof]))` in
  production (not in spike) so a bad slide does not prevent recovery of subsequent slides.

**Span option (future):** The `BorrowInput` newtype approach — implementing the `Input` trait
for `&[(Token, SimpleSpan)]` — eliminates the span table indirection entirely (~50 lines).
Evaluate during Phase 3 `slideforge-syntax` initial implementation story.

**Rejected alternatives (S4):**
- winnow: error accumulation is opt-in add-on, not first-class.
- pest: no built-in error recovery; slideforge needs accumulate-all-errors.
- Hand-written (full): 2× more lines than chumsky for comparable error recovery quality.
