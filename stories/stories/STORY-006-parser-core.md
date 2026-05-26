---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-006
title: "Parser Core: Deck, Slide, Fields, Indentation"
epic: EPIC-02
wave: 1
points: 8
priority: P0
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-syntax
behavioral_contracts:
  - BC-1.01.001
  - BC-1.01.002
verification_properties: [VP-009]
nfr_refs:
  - NFR-021
  - NFR-022
  - NFR-023
  - NFR-024
depends_on:
  - STORY-005
blocks:
  - STORY-007
  - STORY-008
  - STORY-009
  - STORY-010
subsystems:
  - SS-01
target_module: slideforge-syntax
---

# STORY-006: Parser Core: Deck, Slide, Fields, Indentation

## Summary

Implement the core parse tree for `slideforge-syntax`: parse a `.sf` source file
into a fully typed AST representing the deck structure (deck metadata, `vars:` blocks,
`set` rules, and individual `slide <type>:` blocks with their field assignments).
Enforce indentation-significance using chumsky 0.10 — consistent space-only indentation
with file:line:col error spans. Accumulate all errors in a single pass; never halt on
the first error. This story delivers the structural backbone that all other parser
stories (STORY-007 through STORY-010) extend.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-1.01.001 | Parse valid .sf source into typed AST with error accumulation | All 5 postconditions |
| BC-1.01.002 | Reject indentation inconsistency with file:line:col span | All 4 postconditions |

## Acceptance Criteria

- [ ] **AC-001** — A minimal valid `.sf` deck (slideforge_version, lang, brand, one slide) parses
  to an AST with zero diagnostics and exit 0.
  (traces to BC-1.01.001 postcondition 1, 3, 4)

- [ ] **AC-002** — The parsed AST preserves deck metadata (`slideforge_version`, `lang`, `brand`),
  `vars:` blocks, `set` rules, and slide blocks in source order.
  (traces to BC-1.01.001 postcondition 2)

- [ ] **AC-003** — Parsing the same source twice produces byte-identical AST representations
  (determinism invariant).
  (traces to BC-1.01.001 invariant 1)

- [ ] **AC-004** — All source spans in the AST reference valid positions in the input (line ≤
  total line count, col ≤ line byte length + 1).
  (traces to BC-1.01.001 invariant 2)

- [ ] **AC-005** — A slide indented 3 spaces when 2 are expected emits E-PAR-001 with the exact
  message `Unexpected indentation at <file>:<line>:<col>. Expected 2 spaces, found 3.`
  (traces to BC-1.01.002 postcondition 1)

- [ ] **AC-006** — A dedent that lands between two valid indent levels (e.g., 1 space where 0
  or 4 are valid) emits E-PAR-001 identifying the misalignment.
  (traces to BC-1.01.002 postcondition 1)

- [ ] **AC-007** — A source with 2 independent indentation errors emits exactly 2 E-PAR-001
  diagnostics (both accumulated before halting); exit code 1.
  (traces to BC-1.01.002 postcondition 2; BC-1.01.001 invariant 4)

- [ ] **AC-008** — When any parse error exists, no AST is returned to subsequent pipeline
  stages — the `parse` function returns `Err(Vec<SyntaxError>)`.
  (traces to BC-1.01.001 invariant 4)

- [ ] **AC-009** — The parser handles an empty `.sf` file (0 bytes) without panicking; returns
  an empty-deck AST (no slides). The zero-slide validation error is raised by the
  validation stage (STORY-016), not the parser.
  (traces to BC-1.01.001 postcondition 5 — parse stage does not write output)

- [ ] **AC-010** — A `slide <type>:` block with field assignments (`title "..."`, `body "..."`,
  etc.) is parsed into a `SlideNode` struct with all field values captured as
  `FieldValue` variants.
  (traces to BC-1.01.001 postcondition 2)

- [ ] **AC-011** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, and
  `clippy::pedantic` clean (no undocumented `#[allow]` suppressions).
  (traces to NFR-021, NFR-022, NFR-024)

- [ ] **AC-012** — Every public item in `slideforge-syntax` has a rustdoc comment; `cargo doc
  --no-deps` produces 0 warnings.
  (traces to NFR-023)

## Tasks

1. Define the `Span` newtype wrapping `(file_id: u32, byte_start: usize, byte_end: usize)`.
2. Define the `SourceFile` struct holding file path + source text, and a `SourceMap`
   that owns a `Vec<SourceFile>` indexed by `file_id`.
3. Define the `SyntaxError` enum with variants `IndentError`, `UnexpectedToken`,
   `UnexpectedEof`; each carrying a `Span` and `hint: String`.
4. Implement `miette::Diagnostic` on `SyntaxError` — every variant must produce a
   non-None `SourceCode` and non-empty `help` text (BC-1.15.001 invariant 2).
5. Define the unspanned AST node types: `DeckNode`, `SlideNode`, `FieldNode`,
   `VarsBlock`, `SetRule`. Add `Spanned<T>` wrapper `(T, Span)` for all node types.
6. Implement the indentation-tracking layer using chumsky 0.10's `custom` combinator or
   `Indented` pattern. The tracker must:
   - Reject tabs at first occurrence with E-PAR-003 (covered fully in STORY-005; reuse
     the token produced by the lexer from STORY-005).
   - Track indent stack as `Vec<usize>` (space count per level).
   - On INDENT: push new level; on DEDENT: pop to matching level or emit E-PAR-001.
7. Implement the `deck` top-level parser: parse optional `slideforge_version`, then
   `lang`, `brand`, `vars:` blocks, `set` rules, and `slide` blocks, accumulating all
   errors via chumsky's `recover_with(skip_then_retry_until(...))`.
8. Implement the `slide_block` parser: `slide <type>:` header followed by indented
   field lines. Each field is `<name> <value>` where value is a quoted string, number,
   or bare identifier.
9. Implement snapshot tests (via `insta`) for:
   - Minimal deck (1 slide) → expected AST JSON snapshot.
   - Deck with vars + set rule + 3 slides → snapshot.
   - 2-indentation-error deck → error list snapshot.
10. Implement unit tests for all ACs listed above.

## File List

- `crates/slideforge-syntax/src/lib.rs` — crate root; re-exports `parse`, `DeckNode`,
  `SyntaxError`, `Span`, `SourceMap`
- `crates/slideforge-syntax/src/span.rs` — `Span`, `SourceFile`, `SourceMap`,
  `Spanned<T>`
- `crates/slideforge-syntax/src/error.rs` — `SyntaxError` enum + `miette::Diagnostic`
  impls
- `crates/slideforge-syntax/src/ast.rs` — `DeckNode`, `SlideNode`, `FieldNode`,
  `VarsBlock`, `SetRule`, `FieldValue`
- `crates/slideforge-syntax/src/indent.rs` — indentation tracker; integrates with
  chumsky via custom combinator
- `crates/slideforge-syntax/src/parser/deck.rs` — top-level `deck()` parser
- `crates/slideforge-syntax/src/parser/slide.rs` — `slide_block()` parser
- `crates/slideforge-syntax/src/parser/mod.rs` — module re-exports; public `parse()`
  entry point
- `crates/slideforge-syntax/tests/snapshots/` — insta snapshot files

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~3 500 |
| chumsky 0.10 docs (key combinator reference) | ~2 000 |
| BC files (2 BCs) | ~1 500 |
| Existing STORY-005 lexer source (input) | ~2 500 |
| Target source files to write | ~5 000 |
| Test files | ~3 000 |
| **Total** | **~17 500** |

Context budget: 17 500 / 200 000 (Sonnet 4.6 context) ≈ 9% — well within limit.

## Test Strategy

**Unit tests** (`crates/slideforge-syntax/src/parser/deck.rs` `#[cfg(test)] mod tests`):

- `test_minimal_deck_parses_ok()`: parse TV-1.1 minimal deck; assert 1 slide, 0 errors.
- `test_ast_deterministic()`: parse same source twice; assert AST equality.
- `test_all_spans_in_bounds()`: parse 10-slide deck; walk all spans; assert line ≤
  source.lines().count() + 1.
- `test_indent_err_single()`: 3-space indent where 2 expected; assert exactly 1
  E-PAR-001 with correct `expected=2, found=3`.
- `test_indent_err_accumulated()`: 2 independent errors; assert 2 E-PAR-001 diagnostics.
- `test_dedent_misalign()`: dedent to 1-space where 0 or 4 are valid; assert E-PAR-001.
- `test_parse_returns_err_on_error()`: source with 1 error; assert `parse()` returns
  `Err(_)`, not `Ok(_)`.
- `test_empty_file_no_panic()`: parse empty string; assert no panic, returns
  `Ok(DeckNode { slides: [], .. })` or `Err([])`.
- `test_slide_fields_captured()`: `slide title:` with `title "Hello"` and `footer
  "Slide 1"`; assert `SlideNode.fields` contains both field assignments.
- `test_vars_block_parsed()`: `vars: { client: "Acme" }`; assert `DeckNode.vars`
  contains `client: FieldValue::Str("Acme")`.
- `test_set_rule_parsed()`: `set content: footer "Default"`; assert `DeckNode.set_rules`
  has one entry.

**Snapshot tests** (insta, `crates/slideforge-syntax/tests/`):

- `test_snapshot_minimal_deck()`: parses `tests/fixtures/minimal.sf`; snapshot AST
  debug output.
- `test_snapshot_vars_set_3slides()`: parses `tests/fixtures/vars_set_3slides.sf`;
  snapshot AST.
- `test_snapshot_2_indent_errors()`: parses `tests/fixtures/2_indent_errors.sf`;
  snapshot error list.

## Dependencies

- **Depends on:** STORY-005 (lexer tokens + mode-based tokenizer; parser consumes
  `Token` stream produced by the lexer)
- **Blocks:** STORY-007 (control flow parser builds on the `DeckNode` + `SlideNode`
  types defined here), STORY-008 (includes/variants extend the same parse tree),
  STORY-009 (math/shape/version parsers extend the grammar), STORY-010 (error
  accumulation infra builds on `SyntaxError` + `Span` defined here)

## Dependency Anchor Justifications

- SS-01 owns this story's scope because SS-01 is the DSL Parser subsystem in the
  ARCH-INDEX Subsystem Registry, and `slideforge-syntax` is its sole crate.
- STORY-006 depends on STORY-005 because the parser consumes `Token` values produced
  by the lexer — the lexer must exist and be correct before the parser can be built.
- STORY-006 blocks STORY-007/008/009/010 because all those stories extend the parser
  grammar or error infrastructure defined here; their implementations reference
  `DeckNode`, `SlideNode`, `Span`, and `SyntaxError` from this story.

## Architecture Compliance Rules

These rules are extracted from `architecture/crate-architecture.md` and apply to all
code written for this story:

1. `slideforge-syntax` is a **pure core** crate (SS-01). It must NOT depend on any
   effectful workspace crate (`slideforge-data`, `slideforge-brand`, etc.).
2. `slideforge-syntax` must NOT depend on `slideforge-eval`, `slideforge-validate`,
   `slideforge-layout`, or any exporter crate. It is a leaf in the dependency graph.
3. All AST node types must implement `Hash + Eq + Clone + Debug` (comemo and Kani
   compatibility per ADR-013 and the IR design rules).
4. All error types must implement `miette::Diagnostic` with a non-None `SourceCode`
   and non-empty `#[help]` annotation. Plain `thiserror` errors without `miette` are
   not acceptable (ADR-012).
5. `#![forbid(unsafe_code)]` at crate root. No exceptions for v1.0.
6. No `f64` in coordinate or span types — use `usize` for byte positions (ADR-013).

**Forbidden dependencies for `slideforge-syntax`:**
- `slideforge-eval` (build MUST fail if this appears in Cargo.toml)
- `slideforge-validate`
- `slideforge-layout`
- `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`
- `slideforge-data`, `slideforge-brand`
- `slideforge-cli`

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `chumsky` | `=0.10.1` | Parser combinators; use `Parser::recover_with(via_parser(...))` or `recover_with(skip_then_retry_until(...))` for error recovery |
| `thiserror` | `=2.0.18` | Error derives on `SyntaxError` |
| `miette` | `=7.6.0` | `Diagnostic` trait impls on `SyntaxError` |
| `insta` | dev-dep, compatible | Snapshot testing |

chumsky 0.10 API notes (do NOT use chumsky 0.9 API — breaking change):
- `Parser<'src, I, O, Extra>` trait is the main abstraction (lifetime-parameterized in 0.10).
- Use `just(Token::Indent)` for single-token matchers.
- Use `.recover_with(via_parser(nested_parser))` or `recover_with(skip_then_retry_until([...]))` from `chumsky::recovery` to accumulate errors.
- Error type: `chumsky::error::Rich<'src, Token>` for source-span information.
- DO NOT use `Parser::map_err_with_state` — this does not exist in 0.10.1. Use `Parser::map_with` for accessing span/state metadata.
- DO NOT use chumsky 0.9's `Parser::parse_recovery()` — in 0.10, use `Parser::parse(stream).into_output_errors()` to get both partial output and errors.
- DO NOT use `chumsky::pratt` module — Pratt parsing is NOT available in 0.10.1 (added in 1.0-alpha series only). Use manual precedence climbing via `foldl`/`foldr` patterns instead.

## File Structure Requirements

```
crates/slideforge-syntax/
  Cargo.toml           # already exists (stub from workspace scaffold)
  src/
    lib.rs             # crate root: #![forbid(unsafe_code)], pub use
    span.rs            # Span, SourceFile, SourceMap, Spanned<T>
    error.rs           # SyntaxError enum + miette::Diagnostic impls
    ast.rs             # DeckNode, SlideNode, FieldNode, VarsBlock, SetRule, FieldValue
    indent.rs          # IndentTracker combinator
    parser/
      mod.rs           # pub fn parse(src: &str, file_id: u32, source_map: &SourceMap)
                       #   -> Result<DeckNode, Vec<SyntaxError>>
      deck.rs          # deck() parser combinator
      slide.rs         # slide_block() parser combinator
  tests/
    fixtures/
      minimal.sf
      vars_set_3slides.sf
      2_indent_errors.sf
    snapshots/         # managed by insta
    integration_tests.rs
```

Do NOT create a `main.rs` — this is a library crate only.

## Previous Story Intelligence

N/A — first story in EPIC-02 parser group (STORY-005 is the lexer, which is a
separate concern; this story starts fresh on the parser combinator layer).
Lesson from STORY-005: the chumsky 0.10 `Rich` error type carries the span natively —
extract it via `Rich::span()` and map into the project `Span` type rather than
recomputing spans manually.

## Implementation Notes

### Grammar (key productions)

```
deck         ::= version_decl? lang_decl? brand_decl? (vars_block | set_rule | slide_block)*
version_decl ::= "slideforge_version" STRING NEWLINE
lang_decl    ::= "lang" STRING NEWLINE
brand_decl   ::= "brand" STRING NEWLINE
vars_block   ::= "vars:" INDENT (IDENT ":" value NEWLINE)+ DEDENT
set_rule     ::= "set" IDENT ":" IDENT value NEWLINE
slide_block  ::= "slide" IDENT ":" INDENT (field_line)+ DEDENT
field_line   ::= IDENT value NEWLINE
value        ::= STRING | NUMBER | IDENT
```

### Indentation model

- Declared indent width: 2 spaces (default; spec does not allow user configuration
  of this value in v1.0).
- `IndentTracker` wraps a `Vec<usize>` (stack of space counts at each open level).
- INDENT event: current line's leading spaces > top of stack → push.
- DEDENT event: current line's leading spaces < top of stack → pop until match or emit
  E-PAR-001.
- Blank lines: ignored (do not affect indent tracking).

### Error recovery strategy per production

| Production | Recovery action |
|-----------|----------------|
| `slide_block` header | Skip to next un-indented line; emit E-PAR-002 |
| `field_line` | Skip to end of line (NEWLINE); continue next field |
| `vars_block` value | Skip value token; use `FieldValue::Error` sentinel |
| Indentation error | Emit E-PAR-001; re-align stack to best-guess level; continue |

### AST node types (key signatures)

```rust
// span.rs
pub struct Span {
    pub file_id: u32,
    pub start: usize,  // byte offset
    pub end: usize,    // byte offset (exclusive)
}
pub struct Spanned<T>(pub T, pub Span);

// ast.rs
pub struct DeckNode {
    pub version: Option<Spanned<String>>,
    pub lang: Option<Spanned<String>>,
    pub brand: Option<Spanned<String>>,
    pub vars: Vec<VarsBlock>,
    pub set_rules: Vec<SetRule>,
    pub slides: Vec<Spanned<SlideNode>>,
}

pub struct SlideNode {
    pub kind: Spanned<String>,  // slide type keyword
    pub tags: Vec<Spanned<String>>,
    pub fields: Vec<FieldNode>,
}

pub struct FieldNode {
    pub name: Spanned<String>,
    pub value: Spanned<FieldValue>,
}

pub enum FieldValue {
    Str(String),
    Num(i64),
    Ident(String),
    Error,  // sentinel for recovery
}

pub struct VarsBlock {
    pub entries: Vec<(Spanned<String>, Spanned<FieldValue>)>,
}

pub struct SetRule {
    pub slide_type: Spanned<String>,
    pub field: Spanned<String>,
    pub value: Spanned<FieldValue>,
}
```

### Public parse API

```rust
/// Parse a .sf source string into a typed AST.
///
/// Returns Ok(DeckNode) if parsing succeeds with zero errors.
/// Returns Err(errors) if one or more parse errors were accumulated.
/// Never panics on valid UTF-8 input.
pub fn parse(
    src: &str,
    file_id: u32,
    source_map: &SourceMap,
) -> Result<DeckNode, Vec<SyntaxError>>;
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty source (0 bytes) | `Ok(DeckNode { slides: [], .. })` — zero-slide error is a VALIDATION concern (STORY-016), not a parse error |
| EC-002 | Deck metadata only (no slides) | `Ok(DeckNode { slides: [] })` — same as EC-001 |
| EC-003 | Slide with no fields | Parsed as `SlideNode { fields: [] }` — missing required field is a VALIDATION concern |
| EC-004 | Two indentation errors + one valid slide | 2 E-PAR-001 accumulated; parse still extracts the valid slide if recovery succeeds |
| EC-005 | `vars:` block with deeply nested map value | Parsing fails with "unexpected token"; nested maps are not supported in v1.0 |
| EC-006 | Non-UTF-8 source file | Pre-condition violation; CLI rejects before calling `parse()` |
