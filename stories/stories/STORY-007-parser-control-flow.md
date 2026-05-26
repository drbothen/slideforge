---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-007
title: "Parser: @for, @if/@elif/@else, {{ expr }}"
epic: EPIC-02
wave: 1
points: 8
priority: P0
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-syntax
behavioral_contracts:
  - BC-1.04.001
  - BC-1.05.001
verification_properties: []
nfr_refs:
  - NFR-021
  - NFR-022
  - NFR-023
  - NFR-024
depends_on:
  - STORY-005
  - STORY-006
blocks:
  - STORY-011
  - STORY-012
subsystems:
  - SS-01
target_module: slideforge-syntax
---

# STORY-007: Parser: @for, @if/@elif/@else, {{ expr }}

## Summary

Extend the `slideforge-syntax` parser (built in STORY-006) with control-flow
productions: `@for item in collection:` iteration blocks, `@if/@elif/@else:` conditional
blocks, and `{{ expr }}` interpolation expressions inside field values. The parser
produces AST nodes for all three constructs; evaluation semantics are implemented later
in `slideforge-eval` (STORY-011/012). This story's scope is purely syntactic: correct
node structure, accurate spans, error accumulation, and error recovery per production.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-1.04.001 | @for over finite collection generates slides/blocks per item | Postconditions 1-4 (parse-time: node structure, source order, loop-var binding in AST, no-error on valid input) |
| BC-1.05.001 | @if/@elif/@else at slide, element, field, and section scope | Postconditions 1-5 (parse-time: branch structure, AST node per branch, invariant 3 — same syntax at all scopes) |

Note: BC-1.04.001 postcondition 1 ("one slide/block generated per element") and
BC-1.05.001 postconditions 1-4 (branch selection) are EVALUATION concerns handled
in STORY-012. The parser stories cover the structural preconditions only.

## Acceptance Criteria

- [ ] **AC-001** — `@for item in items:` followed by an indented `slide title:` block parses
  to a `ForNode { binding: "item", collection: Expr::Ident("items"), body: [SlideNode] }`.
  (traces to BC-1.04.001 postcondition 2 — generated slides appear in document order
  at the `@for` position)

- [ ] **AC-002** — `@for x in [1, 2, 3]:` with an inline literal list parses the collection
  as `Expr::List([Expr::Num(1), Expr::Num(2), Expr::Num(3)])`.
  (traces to BC-1.04.001 edge case EC-001)

- [ ] **AC-003** — The loop binding variable `x` is recorded in `ForNode.binding` and is NOT
  present in the outer scope's variable list (loop variable scoping is an eval concern;
  parser records it as a new binding name).
  (traces to BC-1.04.001 invariant 3)

- [ ] **AC-004** — `@if condition:` block with a `slide` body parses to `IfNode { condition:
  Expr, then_body: [SlideNode], elif_branches: [], else_body: None }`.
  (traces to BC-1.05.001 postcondition 1, invariant 1)

- [ ] **AC-005** — `@if cond1: ... @elif cond2: ... @else: ...` parses to a single `IfNode`
  with `elif_branches: [(Expr, body)]` and `else_body: Some(body)`.
  (traces to BC-1.05.001 postcondition 2, invariant 1)

- [ ] **AC-006** — `@if` at element scope (inside a slide's indented body, wrapping a field)
  parses identically to `@if` at slide scope — same `IfNode` structure.
  (traces to BC-1.05.001 invariant 3 — same syntax at all four scopes)

- [ ] **AC-007** — `title "Hello {{ name }}"` parses the field value as `FieldValue::Template(
  [TemplateChunk::Literal("Hello "), TemplateChunk::Expr(Expr::Ident("name"))])`.
  (traces to BC-1.04.001 postcondition 3 — `{{ item.field }}` inside loop body
  references current element's fields; this AC verifies the parse structure)

- [ ] **AC-008** — `{{ x + 1 }}` inside a field value parses the expression as
  `Expr::BinOp { op: Add, lhs: Expr::Ident("x"), rhs: Expr::Num(1) }`.
  (traces to BC-1.04.001 postcondition 3)

- [ ] **AC-009** — A source with an `@for` block and a separate `@if` block, both containing
  parse errors, accumulates both error sets before halting.
  (traces to BC-1.01.001 invariant 4 — error accumulation; BC-1.04.001 postcondition 4)

- [ ] **AC-010** — `@while true:` (reserved keyword) produces E-PAR-006 naming the keyword
  and the planned v2+ feature; no `@for` or `@if` node is produced for it.
  (traces to BC-1.04.001 invariant 4 — iteration is always finite; BC-1.01.005)

- [ ] **AC-011** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, and
  `clippy::pedantic` clean. (traces to NFR-021, NFR-022, NFR-024)

- [ ] **AC-012** — Every public item in the new modules has a rustdoc comment; `cargo doc
  --no-deps` produces 0 warnings. (traces to NFR-023)

## Tasks

1. Define AST node types in `ast.rs`:
   - `ForNode { binding: Spanned<String>, collection: Spanned<Expr>, body: Vec<BlockItem> }`
   - `IfNode { condition: Spanned<Expr>, then_body: Vec<BlockItem>, elif_branches: Vec<(Spanned<Expr>, Vec<BlockItem>)>, else_body: Option<Vec<BlockItem>> }`
   - `BlockItem` enum: `Slide(Spanned<SlideNode>)`, `For(Spanned<ForNode>)`, `If(Spanned<IfNode>)`, `Section(Spanned<SectionNode>)` — future variants can be added.
   - Update `DeckNode.slides` from `Vec<Spanned<SlideNode>>` to `Vec<BlockItem>`.
2. Define the `Expr` enum in `expr.rs`:
   - `Ident(String)`, `Num(i64)`, `Str(String)`, `Bool(bool)`, `Null`
   - `List(Vec<Expr>)`, `Map(Vec<(String, Expr)>)`, `FieldAccess { base: Box<Expr>, field: String }`
   - `BinOp { op: BinOpKind, lhs: Box<Expr>, rhs: Box<Expr> }`
   - `UnaryOp { op: UnaryOpKind, operand: Box<Expr> }`
   - `Pipe { lhs: Box<Expr>, filter: String, args: Vec<Expr> }` (for `| upper`, `| pad(10)`)
   - `Error` sentinel for recovery
3. Define `TemplateChunk` in `template.rs`: `Literal(String)` | `Expr(Expr)`.
   Update `FieldValue::Str` to `FieldValue::Template(Vec<TemplateChunk>)` — all string
   values are treated as templates (plain strings have a single `Literal` chunk).
4. Implement `for_block()` parser combinator in `parser/control_flow.rs`:
   - Matches `@for IDENT in expr ":"` followed by indented `block_items`.
   - Emits E-PAR-002 on malformed header (missing `in`, missing `:`).
   - Uses `skip_then_retry_until([Token::Dedent])` for recovery.
5. Implement `if_block()` parser combinator in `parser/control_flow.rs`:
   - Matches `@if expr ":"` indented body, then optional `@elif expr ":"` indented body
     chains, then optional `@else ":"` indented body.
   - Emits E-PAR-002 on malformed condition.
6. Implement `expr()` combinator in `parser/expr.rs`:
   - Precedence: comparison > arithmetic > pipe — use manual precedence climbing
     via `foldl`/`foldr` (chumsky 0.10.1 pattern; `chumsky::pratt` is NOT available).
   - List literals: `"[" (expr ("," expr)*)? "]"`.
   - Field access: `ident ("." ident)*`.
7. Implement `template_value()` combinator in `parser/template.rs`:
   - Split a quoted string on `{{ ... }}` boundaries.
   - Inner content is parsed with `expr()`.
   - Malformed `{{` without matching `}}` emits E-PAR-004 and treats the chunk as
     literal text.
8. Update `deck()` and `slide_block()` parsers to use `block_item()` combinator
   (replacing the `slide_block()` calls).
9. Write snapshot tests for: `@for` block, `@if` with elif + else, nested `@for`
   inside `@if`, `{{ expr }}` field value.
10. Write unit tests for all ACs.

## File List

- `crates/slideforge-syntax/src/expr.rs` — `Expr`, `BinOpKind`, `UnaryOpKind` (new)
- `crates/slideforge-syntax/src/template.rs` — `TemplateChunk` (new)
- `crates/slideforge-syntax/src/ast.rs` — updated: `ForNode`, `IfNode`, `BlockItem`
  added; `DeckNode.slides` type updated; `FieldValue::Template` replaces `Str`
- `crates/slideforge-syntax/src/parser/control_flow.rs` — `for_block()`, `if_block()`
  (new)
- `crates/slideforge-syntax/src/parser/expr.rs` — `expr()` combinator (new)
- `crates/slideforge-syntax/src/parser/template.rs` — `template_value()` combinator
  (new)
- `crates/slideforge-syntax/src/parser/deck.rs` — updated: calls `block_item()`
- `crates/slideforge-syntax/src/parser/slide.rs` — updated: field values use
  `template_value()`
- `crates/slideforge-syntax/tests/fixtures/for_block.sf` — test fixture
- `crates/slideforge-syntax/tests/fixtures/if_elif_else.sf` — test fixture
- `crates/slideforge-syntax/tests/fixtures/template_expr.sf` — test fixture

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~4 000 |
| BC-1.04.001 + BC-1.05.001 | ~3 000 |
| STORY-006 source (ast.rs, parser/deck.rs, parser/slide.rs) | ~4 000 |
| chumsky 0.10 pratt_parser docs | ~1 500 |
| Target source files to write | ~6 000 |
| Test files | ~3 500 |
| **Total** | **~22 000** |

Context budget: 22 000 / 200 000 ≈ 11% — within limit.

## Test Strategy

**Unit tests** (`crates/slideforge-syntax/src/parser/control_flow.rs #[cfg(test)]`):

- `test_for_over_ident_collection()`: parse `@for item in items:` + slide; assert
  `ForNode { binding: "item", collection: Expr::Ident("items") }`.
- `test_for_over_literal_list()`: parse `@for x in [1, 2, 3]:` + slide; assert
  collection is `Expr::List` with 3 elements.
- `test_if_no_else()`: parse `@if env == "prod":` + slide; assert `IfNode` with no
  elif/else.
- `test_if_elif_else()`: parse full chain; assert `elif_branches.len() == 1`,
  `else_body.is_some()`.
- `test_if_at_element_scope()`: parse `@if cond:` inside slide body wrapping a field;
  assert `IfNode` is a child of `SlideNode.body`.
- `test_while_keyword_rejected()`: parse `@while true:` + slide; assert E-PAR-006.
- `test_template_plain_string()`: parse `title "Hello"` (no `{{ }}`); assert
  `TemplateChunk::Literal("Hello")` only.
- `test_template_with_interpolation()`: parse `title "Hello {{ name }}"`;  assert
  `[Literal("Hello "), Expr(Expr::Ident("name"))]`.
- `test_template_binop()`: parse `stat "{{ x + 1 }}"`;  assert inner `Expr::BinOp`.
- `test_malformed_template_no_close()`: parse `title "Hello {{ name"`;  assert
  E-PAR-004 accumulated.
- `test_errors_accumulated_for_and_if()`: both malformed; assert 2+ errors.

**Snapshot tests** (`crates/slideforge-syntax/tests/`):

- `test_snapshot_for_block()`: parses `tests/fixtures/for_block.sf`; snapshot.
- `test_snapshot_if_elif_else()`: parses `tests/fixtures/if_elif_else.sf`; snapshot.
- `test_snapshot_template_expr()`: parses `tests/fixtures/template_expr.sf`; snapshot.

## Dependencies

- **Depends on:** STORY-005 (lexer token types including `@for`, `@if`, `@elif`,
  `@else` as keyword tokens and `{{`/`}}` delimiter tokens)
- **Depends on:** STORY-006 (base AST types `DeckNode`, `SlideNode`, `FieldNode`,
  `Span`, `SyntaxError`; this story extends rather than replaces them)
- **Blocks:** STORY-011 (expression evaluator core — consumes `Expr` AST nodes),
  STORY-012 (variable scoping + @for evaluation — consumes `ForNode`)

## Dependency Anchor Justifications

- SS-01 owns this story's scope because SS-01 is the DSL Parser subsystem owning
  `slideforge-syntax` per ARCH-INDEX Subsystem Registry.
- STORY-007 depends on STORY-005 because `@for`, `@if`, `{{`, `}}` must be emitted as
  distinct `Token` variants by the lexer before the parser can match them.
- STORY-007 depends on STORY-006 because this story modifies `DeckNode.slides` from
  `Vec<Spanned<SlideNode>>` to `Vec<BlockItem>` — that type must be established first.
- STORY-007 blocks STORY-011/012 because the evaluator consumes `Expr` and `ForNode`
  AST nodes that this story defines and produces.

## Architecture Compliance Rules

1. `slideforge-syntax` is **pure core** (SS-01). The `Expr` enum and `ForNode`/`IfNode`
   types must not embed any I/O or runtime state.
2. All new node types must implement `Hash + Eq + Clone + Debug`.
3. `Expr::Error` sentinel is required for error recovery — never use `Option<Expr>`
   as the recovery mechanism (that conflates "absent" with "parse failed").
4. `miette::Diagnostic` on new error variants: non-None `SourceCode`, non-empty `help`.
5. `#![forbid(unsafe_code)]` — already set at crate root; verify not accidentally
   removed when touching `lib.rs`.

**Forbidden dependencies for `slideforge-syntax`:** (same list as STORY-006)
- `slideforge-eval`, `slideforge-validate`, `slideforge-layout`, all exporters,
  `slideforge-data`, `slideforge-brand`, `slideforge-cli`.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `chumsky` | `=0.10.1` | `recursive()` for nested block parsing; manual precedence climbing via `foldl`/`foldr` for expressions (NOTE: `chumsky::pratt` module is NOT available in 0.10.1 — it was added in the 1.0-alpha series only) |
| `thiserror` | `=2.0.18` | Error derives |
| `miette` | `=7.6.0` | `Diagnostic` trait |

chumsky 0.10.1 expression precedence pattern (manual — NO `chumsky::pratt` in 0.10.1):
```rust
// Manual precedence climbing using foldl (chumsky 0.10.1 pattern):
// Highest precedence (most eagerly binding) first:
let atom = int_lit.or(ident_expr).or(expr.delimited_by(just(Token::LParen), just(Token::RParen)));

let unary = just(Token::Bang).repeated().foldr(atom, |_op, rhs| Expr::UnaryOp { op: UnaryOpKind::Not, operand: Box::new(rhs) });

let product = unary.clone().foldl(
    choice((just(Token::Star), just(Token::Slash), just(Token::Percent)))
        .then(unary)
        .repeated(),
    |lhs, (op_tok, rhs)| Expr::BinOp { op: tok_to_binop(op_tok), lhs: Box::new(lhs), rhs: Box::new(rhs) },
);

let sum = product.clone().foldl(
    choice((just(Token::Plus), just(Token::Minus)))
        .then(product)
        .repeated(),
    |lhs, (op_tok, rhs)| Expr::BinOp { op: tok_to_binop(op_tok), lhs: Box::new(lhs), rhs: Box::new(rhs) },
);

let comparison = sum.clone().foldl(
    choice((just(Token::EqEq), just(Token::BangEq), just(Token::Lt), just(Token::Gt)))
        .then(sum)
        .repeated(),
    |lhs, (op_tok, rhs)| Expr::BinOp { op: tok_to_binop(op_tok), lhs: Box::new(lhs), rhs: Box::new(rhs) },
);
// comparison is the top-level expression parser
```

## File Structure Requirements

Control-flow parsers live in `parser/control_flow.rs` (new file). Expression parser
in `parser/expr.rs` (new file). Template parser in `parser/template.rs` (new file).
These modules are declared in `parser/mod.rs`. Do NOT add a new `lib.rs` entry for
each module — use `pub(crate) mod` inside `parser/mod.rs`.

## Previous Story Intelligence

From STORY-006: the chumsky `Rich` error type carries the span natively; map it to
`Span` via `Rich::span()` at the point of error construction, not in a post-processing
step. Apply the same pattern for all new error variants introduced here.

## Implementation Notes

### Grammar productions (new in this story)

```
block_item  ::= slide_block | for_block | if_block
for_block   ::= "@for" IDENT "in" expr ":" INDENT block_item+ DEDENT
if_block    ::= "@if" expr ":" INDENT block_item+ DEDENT
                ("@elif" expr ":" INDENT block_item+ DEDENT)*
                ("@else" ":" INDENT block_item+ DEDENT)?
expr        ::= comparison
comparison  ::= addition (("==" | "!=" | "<" | ">" | "<=" | ">=") addition)*
addition    ::= multiplication (("+" | "-") multiplication)*
multiplication ::= unary (("*" | "/" | "%") unary)*
unary       ::= "!" unary | primary
primary     ::= "(" expr ")" | list | field_access | NUMBER | STRING | IDENT | "true" | "false" | "null"
list        ::= "[" (expr ("," expr)*)? "]"
field_access ::= IDENT ("." IDENT)*
pipe_expr   ::= expr ("|" IDENT ("(" (expr ("," expr)*)? ")")?)*
template    ::= QUOTE (literal_chunk | "{{" expr "}}")* QUOTE
```

### Error recovery per production

| Production | Recovery |
|-----------|---------|
| `for_block` header | Skip tokens to `Token::Colon`; emit E-PAR-002 for malformed header |
| `if_block` condition | Skip to `Token::Colon`; emit E-PAR-002 |
| `expr` | Skip to `Token::Newline`; return `Expr::Error` sentinel |
| `template` `{{ ... }}` | On missing `}}`: read to end of string; emit E-PAR-004 |

### `BlockItem` replaces `slides: Vec<Spanned<SlideNode>>`

The `DeckNode.slides` field must be renamed to `DeckNode.items: Vec<BlockItem>` to
accommodate `@for` and `@if` nodes at deck level. This is a breaking change to the
type defined in STORY-006. Coordinate with the implementer: all code using
`DeckNode.slides` must be updated to `DeckNode.items`.

### `@while` rejection

The reserved keyword `@while` must be rejected with E-PAR-006. The lexer (STORY-005)
emits it as `Token::ReservedKeyword("@while")`. The parser checks for this token in
`block_item()` and emits the error before attempting recovery.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `@for` with empty literal list `[]` | Parsed as `Expr::List([])`; `ForNode` produced; empty collection detection is EVAL concern (STORY-012) |
| EC-002 | `@if` with undefined variable in condition | Parsed as `Expr::Ident("undefined_var")` — evaluation error is EVAL concern (STORY-013) |
| EC-003 | `@elif` without preceding `@if` | E-PAR-002: "unexpected @elif — must follow @if"; accumulated |
| EC-004 | `@else` followed by another `@else` | E-PAR-002: "duplicate @else branch"; second `@else` skipped |
| EC-005 | Nested `@for` inside `@if` inside another `@for` | Parsed recursively via `block_item()` recursion; all three nodes nested in AST |
| EC-006 | `{{ }}` (empty interpolation) | E-PAR-004: "empty expression in `{{ }}`"; `Expr::Error` returned |
