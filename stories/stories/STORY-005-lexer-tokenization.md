---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-005
title: "Lexer + Mode-Based Tokenization"
epic: EPIC-02
wave: 1
points: 5
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-syntax
subsystems: [SS-01]
target_module: slideforge-syntax
behavioral_contracts: [BC-1.01.003]
verification_properties: [VP-001, VP-014]
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on: [STORY-001]
blocks:
  - STORY-006
  - STORY-007
  - STORY-008
  - STORY-009
  - STORY-010
estimated_days: 2
---

# STORY-005: Lexer + Mode-Based Tokenization

## Summary

Implement the byte-level lexer for the slideforge DSL in `slideforge-syntax`. The
lexer converts raw `.sf` source text into a `Token` stream that the chumsky 0.10.1
parser (STORY-006) consumes. The lexer handles three modes: text mode (normal DSL),
math mode (`$...$` and `$$...$$` delimiters), and raw mode (internal only; rejected
at the user level in STORY-009). This story specifically satisfies BC-1.01.003 — tab
characters in leading whitespace are detected at the byte level and produce E-PAR-003
errors with exact file:line:col spans, accumulated before halting.

The lexer is separate from the chumsky parser. chumsky operates on the `Token` stream,
not on raw bytes. This separation makes tab detection deterministic at the byte level
before the combinator layer runs.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3,500 |
| `src/lexer.rs` full implementation | ~5,000 |
| `src/token.rs` token definitions | ~2,000 |
| `src/lexer_error.rs` error types | ~800 |
| Unit test fixtures + test module | ~3,000 |
| **Total** | **~14,300** |

Agent context budget: 200k tokens. This story is ~7.2% of budget — well within limit.

## Behavioral Contracts

| BC ID | Title | Clauses Covered |
|-------|-------|----------------|
| BC-1.01.003 | Reject tab indentation with hard error | Preconditions 1-2; Postconditions 1-4; Invariants 1-3 |

## Acceptance Criteria

- [ ] **AC-001:** `lex(source: &str, file: Arc<str>) -> (Vec<Token>, Vec<LexError>)` function exists and returns a token stream plus any accumulated lexer errors. It does NOT fail-fast — it accumulates all errors in a single pass. (traces to BC-1.01.003 postcondition 2 and invariant — DI-018 error accumulation)
- [ ] **AC-002:** Every tab character (`\t`) found in leading whitespace on any line produces a `LexError::TabIndentation { file, line, col, byte_offset }` entry in the error vec. (traces to BC-1.01.003 precondition 2 and postcondition 1)
- [ ] **AC-003:** The `LexError::TabIndentation` error message format is: `Tab character at {file}:{line}:{col}. slideforge requires spaces for indentation.` — with exact file:line:col. (traces to BC-1.01.003 postcondition 1)
- [ ] **AC-004:** A tab character inside a quoted string value (e.g., `title "contains\there"`) does NOT produce a `LexError::TabIndentation`. (traces to BC-1.01.003 invariant 1 and edge case EC-001)
- [ ] **AC-005:** A tab character inside a comment (`# tab\there`) does NOT produce a `LexError::TabIndentation`. (traces to BC-1.01.003 edge case EC-004)
- [ ] **AC-006:** A file with both tab indentation and mixed space-indentation violations produces BOTH `LexError::TabIndentation` errors and `LexError::IndentationInconsistency` errors — all accumulated. (traces to BC-1.01.003 edge case EC-002)
- [ ] **AC-007:** When `lex` returns with `errors.len() > 0`, the returned `tokens` vec may still be non-empty (lexer continues after errors to accumulate more). The parser (STORY-006) decides when to stop based on accumulated errors. (traces to BC-1.01.003 postcondition 2 — error accumulation)
- [ ] **AC-008:** `Token` enum covers all DSL surface tokens: `Ident(Arc<str>)`, `StringLit(Arc<str>)`, `IntLit(i64)`, `FloatLit(ordered_float::OrderedFloat<f64>)`, `BoolLit(bool)`, `At` (for `@`), `DollarSingle` (for `$`), `DollarDouble` (for `$$`), `DoubleBrace` (for `{{`), `CloseBrace` (for `}}`), `AtBrace` (for `@{`), `Colon`, `Pipe`, `Newline`, `Indent(usize)`, `Dedent`, `Eof`, `MathContent(Arc<str>)`. (traces to story decomposition requirement — complete token coverage for STORY-006 parser)
- [ ] **AC-009:** `Token` with its `SourceSpan` wrapping (`Spanned<Token>`) carries exact byte_offset for every token — used by the parser for error reporting with file:line:col. (traces to BC-1.15.001 — all errors carry file:line:col span)
- [ ] **AC-010:** `LexerMode` enum is defined with variants `Text`, `Math`, `MathDisplay` — and the lexer correctly switches between modes when `$` (single) or `$$` (display) delimiters are encountered. (traces to architecture requirement — mode-based parsing per CLAUDE.md DSL Parser section)
- [ ] **AC-011:** In `Math` mode, `@{` produces `AtBrace` token (for math interpolation). In `Text` mode, `{{` produces `DoubleBrace` token. Neither token type is valid in the other mode — the lexer differentiates by current mode. (traces to CLAUDE.md: "`{{ var }}` interpolation in text mode; `@{var}` in math mode")
- [ ] **AC-012:** `#![forbid(unsafe_code)]` is on the crate root (NFR-024).
- [ ] **AC-013:** `#![warn(missing_docs)]` is on the crate root; all public items documented (NFR-023).
- [ ] **AC-014:** `cargo clippy -p slideforge-syntax -- -D warnings` produces zero warnings (NFR-022).
- [ ] **AC-015:** All production deps in `Cargo.toml` use `=` version pinning (NFR-025). Deps: `slideforge-types` (workspace path), `chumsky = "=0.10.1"`, `thiserror = "=2.0.18"`, `ordered-float = "=4.6.0"`.

## Previous Story Intelligence

N/A — this is the first EPIC-02 story. EPIC-01 (STORY-001 through STORY-004) defines
the types used in the token stream and error types.

Key dependency: STORY-001 provides `SourceSpan` (used for token spans) and `Value`
(not directly used in lexer, but `Arc<str>` pattern from that crate is used here).

The `slideforge-syntax` crate stub was scaffolded as part of the initial workspace
setup. Before writing, read the existing `crates/slideforge-syntax/Cargo.toml` and
`src/lib.rs` to understand what skeleton already exists.

## Architecture Compliance Rules

Sourced from `architecture/crate-architecture.md`, `architecture/system-overview.md`, and CLAUDE.md:

1. **Pure Core (SS-01):** `slideforge-syntax` is Pure Core — no I/O, no filesystem, no network. The lexer receives a `&str` in memory and produces tokens. File reading is the CLI's responsibility.
2. **Error accumulation (DI-018):** The lexer MUST accumulate all errors in one pass. No `Err(e); return` pattern. Use a `Vec<LexError>` accumulated throughout.
3. **Byte-level tab detection:** Tab detection MUST happen at the byte level before chumsky runs. This makes it deterministic and verifiable by Kani (VP-001 in STORY-066). Do not rely on chumsky's error recovery for tab detection.
4. **`Arc<str>` for string literals:** String literal values in tokens must use `Arc<str>`, consistent with the rest of the IR type system. Do not use `String` in token types.
5. **`chumsky = "=0.10.1"` only:** The pinned version is `0.10.1`. Do not use 0.9.x patterns — chumsky 0.10 has breaking API changes. Specifically, use `chumsky::prelude::*` from 0.10 and `chumsky::error::Rich` for error accumulation.
6. **No eval/layout deps:** `slideforge-syntax` MUST NOT depend on `slideforge-eval`, `slideforge-validate`, `slideforge-layout`, or any effectful crate. It may depend on `slideforge-types` for shared type primitives.
7. **Forbidden dependencies:** `slideforge-syntax` MUST NOT import from: `slideforge-eval`, `slideforge-brand`, `slideforge-pptx`, `slideforge-layout`, `slideforge-validate`, or any effectful crate. Build fails if such a dep appears.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `slideforge-types` | workspace path | `SourceSpan`, `Arc<str>` pattern |
| `chumsky` | `=0.10.1` | Token type structure (lexer produces `Spanned<Token>` compatible with chumsky) |
| `thiserror` | `=2.0.18` | `LexError` derive |
| `ordered-float` | `=4.6.0` | `OrderedFloat<f64>` in `Token::FloatLit` |

Dev-only dependencies (no pinning):
- No dev deps needed for lexer unit tests.

## File Structure Requirements

Files to CREATE:

```
crates/slideforge-syntax/
├── Cargo.toml                       # manifest; add chumsky =0.10.1 + ordered-float =4.2
├── src/
│   ├── lib.rs                       # #![forbid(unsafe_code)]; pub mod token; pub mod lexer; pub mod lexer_error
│   ├── token.rs                     # Token enum + Spanned<T> alias + LexerMode
│   ├── lexer.rs                     # lex() function + LexerState struct
│   └── lexer_error.rs               # LexError enum
```

The parser modules (`parser.rs`, `ast.rs`) are created in STORY-006 through STORY-009.

Files to MODIFY:
- `Cargo.toml` — add `chumsky = "=0.10.1"`, `ordered-float = "=4.6.0"` to `[dependencies]`.
- `src/lib.rs` — update with `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, module declarations.

## Tasks

1. **Update `Cargo.toml`** — add `chumsky = "=0.10.1"`, `ordered-float = "=4.6.0"` to `[dependencies]`. Verify existing deps. (10 min)
2. **Write `src/lexer_error.rs`** — `LexError` enum with `TabIndentation`, `IndentationInconsistency`, `UnterminatedString`, `UnterminatedMath`, `InvalidCharacter` variants. Each carries `SourceSpan`. Implement `thiserror::Error` with appropriate `#[error]` messages. (20 min)
3. **Write `src/token.rs`** — `Token` enum with all variants per AC-008. `LexerMode` enum. `type Spanned<T> = (T, std::ops::Range<usize>)` alias (chumsky 0.10 convention). (25 min)
4. **Write `src/lexer.rs`** — `lex(source: &str, file: Arc<str>) -> (Vec<Spanned<Token>>, Vec<LexError>)` function. Implement `LexerState` struct with `mode: LexerMode`, `indent_stack: Vec<usize>`, `errors: Vec<LexError>`. (60 min)
5. **Implement tab detection in `lexer.rs`** — scan leading whitespace of each logical line; emit `LexError::TabIndentation` for each `\t` found. Do NOT stop; continue scanning rest of line. (15 min)
6. **Implement indentation token generation** — produce `Indent(n)` when indentation increases, `Dedent` when it decreases, relative to `indent_stack`. (20 min)
7. **Implement mode switching** — when `$` is encountered outside a string, switch to `Math` mode; `$$` to `MathDisplay` mode; matching delimiter switches back to `Text`. (20 min)
8. **Write `src/lib.rs`** — `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, module declarations, pub re-exports of `lex`, `Token`, `LexError`, `LexerMode`. (10 min)
9. **Write unit tests** — tab detection, string literals, math mode, comment handling, error accumulation (see Test Strategy). (40 min)
10. **Run `cargo test -p slideforge-syntax`** — confirm all tests pass. (5 min)
11. **Run `cargo clippy -p slideforge-syntax -- -D warnings`** — fix all. (15 min)

## Test Strategy

### Unit tests (`src/lexer.rs` `#[cfg(test)]`)

```rust
use std::sync::Arc;

fn lex_str(s: &str) -> (Vec<Spanned<Token>>, Vec<LexError>) {
    lex(s, Arc::from("test.sf"))
}

#[test]
fn tab_leading_whitespace_produces_error() {
    let src = "\tfield: value\n";
    let (_, errors) = lex_str(src);
    assert_eq!(errors.len(), 1);
    let LexError::TabIndentation { line, col, .. } = &errors[0] else {
        panic!("expected TabIndentation");
    };
    assert_eq!(*line, 1);
    assert_eq!(*col, 1);
}

#[test]
fn tab_inside_string_no_error() {
    let src = "title \"contains\ttab\"\n";
    let (tokens, errors) = lex_str(src);
    assert!(errors.is_empty(), "tab in string should not produce error");
    let has_string_lit = tokens.iter().any(|(t, _)| matches!(t, Token::StringLit(s) if s.contains('\t')));
    assert!(has_string_lit);
}

#[test]
fn tab_inside_comment_no_error() {
    let src = "# comment\twith tab\n";
    let (_, errors) = lex_str(src);
    assert!(errors.is_empty(), "tab in comment should not produce error");
}

#[test]
fn multiple_tab_lines_all_accumulated() {
    let src = "\tfield1: value\n\tfield2: value\n";
    let (_, errors) = lex_str(src);
    assert_eq!(errors.len(), 2, "should accumulate both tab errors");
    assert!(errors.iter().all(|e| matches!(e, LexError::TabIndentation { .. })));
}

#[test]
fn mixed_space_and_tab_errors_both_accumulated() {
    // Line 1: tab indentation (TabIndentation error)
    // Line 2: inconsistent space indentation (IndentationInconsistency error)
    let src = "\tfield1: val\n  field2: val\n   field3: val\n";
    let (_, errors) = lex_str(src);
    assert!(errors.len() >= 2, "should have both tab and indentation errors");
    let has_tab = errors.iter().any(|e| matches!(e, LexError::TabIndentation { .. }));
    let has_indent = errors.iter().any(|e| matches!(e, LexError::IndentationInconsistency { .. }));
    assert!(has_tab, "should have tab error");
    assert!(has_indent, "should have indentation inconsistency error");
}

#[test]
fn math_mode_switch_dollar_single() {
    let src = "title \"$x^2$\"\n";
    let (tokens, errors) = lex_str(src);
    assert!(errors.is_empty());
    // $x^2$ should be inside a string literal — the mode switch happens
    // when $ appears outside a string literal. Inside a string, $ is just text.
    let has_string = tokens.iter().any(|(t, _)| matches!(t, Token::StringLit(_)));
    assert!(has_string);
}

#[test]
fn math_mode_standalone_dollar() {
    // In DSL prose context (field value without quotes), $...$ switches to math mode
    let src = "body $x^2$\n";
    let (tokens, errors) = lex_str(src);
    // Should produce DollarSingle, MathContent, DollarSingle tokens
    let has_dollar_single = tokens.iter().any(|(t, _)| matches!(t, Token::DollarSingle));
    assert!(has_dollar_single, "should produce DollarSingle token for math mode delimiter");
    assert!(errors.is_empty());
}

#[test]
fn double_brace_text_mode() {
    let src = "title \"{{ name }}\"\n";
    let (tokens, errors) = lex_str(src);
    // {{ within a string literal — depends on implementation:
    // Option A: tokenize as StringLit containing "{{ name }}" (deferred to parser)
    // Option B: produce DoubleBrace token inside string (more complex)
    // Preferred: Option A — parser handles {{ inside string literals
    // The key requirement is no error
    assert!(errors.is_empty());
}

#[test]
fn at_brace_vs_double_brace_mode_distinction() {
    let src = "body @{ var }\n";
    let (tokens, errors) = lex_str(src);
    assert!(errors.is_empty());
    let has_at_brace = tokens.iter().any(|(t, _)| matches!(t, Token::AtBrace));
    assert!(has_at_brace, "@ followed by {{ should produce AtBrace token in text mode");
}

#[test]
fn spanned_tokens_carry_position() {
    let src = "title \"My Deck\"\n";
    let (tokens, _) = lex_str(src);
    // Each token should have a non-degenerate span
    for (_, span) in &tokens {
        assert!(span.end > span.start || span.start == span.end,
            "span should be valid (end >= start)");
    }
}
```

### Integration tests

None in this story — integration tests that run the full parse pipeline come in
STORY-006 (parser core).

### Snapshot tests (STORY-005 contribution)

One snapshot test for the token stream of a canonical minimal `.sf` file, to catch
regressions in tokenization:

```rust
#[test]
fn snapshot_minimal_deck() {
    let src = r#"
slideforge_version "1"
lang "en-US"
title "Test"

slide title:
  title "Hello"
"#;
    let (tokens, errors) = lex_str(src);
    assert!(errors.is_empty());
    insta::assert_debug_snapshot!("minimal_deck_tokens", tokens);
}
```

Note: `insta` is a dev dependency; add `insta = "1.45"` to `[dev-dependencies]` (latest stable as of May 2026 is 1.45.x; dev deps do not require `=` pinning per NFR-025).

## Dependencies

**Depends on:** STORY-001 (IR core types — specifically `SourceSpan` for span information in tokens and errors).

Dependency justification: The `LexError::TabIndentation` and similar errors carry a `SourceSpan` struct which is defined in `slideforge-types`. The lexer needs this type to populate error locations.

**Blocks:**
- STORY-006 (parser core): the chumsky parser is written as a combinator on the `Vec<Spanned<Token>>` output of this story's `lex()` function.
- STORY-007 (parser control flow): same parser dependency.
- STORY-008 (parser includes/variants): same parser dependency.
- STORY-009 (parser math/shape): imports `LexerMode` from this story for math mode context.
- STORY-010 (error accumulation): imports `LexError` from this story.

## Implementation Notes

### Lexer Architecture Choice: Hand-Written over chumsky

The lexer is hand-written (not a chumsky combinator) because:
1. Tab detection must happen at the byte level before combinator parsing begins.
2. Mode switching (`$...$` math mode) is cleaner to implement as explicit state.
3. Indentation tracking requires imperative stack management that is awkward in combinators.

The chumsky crate is used in STORY-006+ for the parser layer (which operates on tokens,
not bytes). The lexer produces `Vec<Spanned<Token>>` — the same type chumsky expects
as its input stream.

### Indentation Handling

The DSL is indentation-significant. The lexer produces `Indent(n)` and `Dedent` tokens:
- When indentation level increases (next line has more spaces than stack top): push to stack, emit `Indent(n)`.
- When indentation level decreases: pop from stack, emit `Dedent` (one per level popped).
- Consistency rule: if current line's indentation doesn't match any level in the stack, emit `LexError::IndentationInconsistency`.
- Spaces only: tabs emit `LexError::TabIndentation` and are treated as zero-width for continuation (to accumulate more errors without corrupting the indent stack).

### Math Mode Implementation

Math mode is triggered by `$` or `$$` at the start of a field value (outside quotes):

```
text: This is $x^2$ inline and $$E = mc^2$$ display.
```

When the lexer encounters `$` outside a string literal and outside math mode → emit
`DollarSingle` and switch to `Math` mode. Collect bytes until the matching `$` →
emit `MathContent(Arc<str>)` → emit `DollarSingle` → return to `Text` mode.

`$$` follows same pattern but uses `DollarDouble` tokens and switches to `MathDisplay` mode.

Inside quoted strings (`"..."`), `$` is part of the string literal — do NOT switch mode.

### Spanned Token Format

```rust
/// Token with source byte range for error reporting.
/// Compatible with chumsky 0.10's `Stream<char>` approach but token-based.
pub type Spanned<T> = (T, std::ops::Range<usize>);
```

The `usize` range is a byte offset into the source string. The lexer tracks
`byte_offset: usize` as it advances. Error reporting converts byte_offset to
line:col using a simple line-count scan (pre-compute line starts once).

### Line:Col Computation

Pre-compute a `Vec<usize>` of byte offsets for each line start before tokenizing:
```rust
fn compute_line_starts(src: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(src.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

fn offset_to_line_col(offset: usize, line_starts: &[usize]) -> (u32, u32) {
    let line = line_starts.partition_point(|&s| s <= offset).saturating_sub(1);
    let col = offset - line_starts[line];
    (line as u32 + 1, col as u32 + 1)  // 1-indexed
}
```

### `@` Directives

`@` at the start of a word produces:
- `Token::At` — the `@` prefix.
- `Token::Ident(Arc<str>)` — the directive name (`for`, `if`, `elif`, `else`, `include`, `import`, `data`).

The parser (STORY-006+) handles the combination `At + Ident("for")` as `@for`.

### `{{ }}` vs `@{ }` Token Distinction

- `{{` in text mode (outside math): produces `Token::DoubleBrace`.
- `}}` in text mode: produces `Token::CloseBrace` (same token used for both `}}` and `}`).
- `@{` in math mode: produces `Token::AtBrace`.
- `}` after `@{`: produces `Token::CloseBrace`.

The parser enforces which is valid where. The lexer just produces the correct token
based on context (mode).

### `Token::Indent(usize)` Carrying Level vs. Delta

`Indent(usize)` carries the ABSOLUTE indentation level in spaces (not a delta from
the previous line). This makes the parser's context-free grammar simpler — it doesn't
need to track state to know if it's entering a nested block. `Dedent` carries no data
(the parser pops its own context stack when it sees a `Dedent`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Tab in string `title "has\ttab"` | No `TabIndentation` error; tab preserved in `Token::StringLit` |
| EC-002 | Tab in comment `# tab\there` | No `TabIndentation` error |
| EC-003 | Mixed `\t  field:` (tab then spaces) | `TabIndentation` error at the tab position; spaces after tab are ignored for indentation purposes |
| EC-004 | `$$` followed by unterminated math content (no closing `$$`) | `LexError::UnterminatedMath` accumulated; tokenization continues on next line |
| EC-005 | Empty source file | Returns `([Token::Eof], [])` — empty token stream with EOF token, no errors |
| EC-006 | Source with only comments | Returns tokens for `Newline` and `Eof`; no semantic tokens; no errors |
| EC-007 | `@{` in text mode (not math mode) | Produces `AtBrace` token — valid; the parser/evaluator determines if this is an error in context |
| EC-008 | `{{ }}` in math mode | Produces `DoubleBrace` token — valid token; parser rejects as invalid in math mode context |
| EC-009 | 10,000-line `.sf` file | Lexer completes without stack overflow (iterative, not recursive) — 10k lines is well within stack limits for iterative scanners |
