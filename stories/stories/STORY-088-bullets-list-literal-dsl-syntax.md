---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-088
title: "Bullets list-literal field-value DSL syntax (`bullets: [\"A\",\"B\",\"C\"]`)"
epic: EPIC-02
wave: 5
points: 8
priority: P1
tdd_mode: strict
status: draft
spec_version: "1.2"
last_updated: "2026-06-06"
target_module: slideforge-syntax, slideforge-eval
subsystems: [SS-01, SS-02]
behavioral_contracts: [BC-1.01.002]
# BC-1.16.001 traced via AC footnotes: the list-literal parse result feeds directly
# into the Value::List path that BC-1.16.001 PC-7 specifies for ContentBlock::Bullets.
verification_properties: []
nfr_refs: [NFR-022]
closes_findings: []
depends_on:
  - STORY-006
  - STORY-086
blocks: []
estimated_days: 3
history_note: >
  Created 2026-06-06 to close the parser gap discovered during STORY-086 (Stage 2b
  field-to-block threading, scope expansion authorized 2026-06-06). The human authorized
  "both follow-up stories" (TextTag story + bullets-syntax story). The TextTag follow-up
  story was ABSORBED into STORY-086 — TextTag is fully implemented in-scope there, not
  in a separate story. Only this bullets-syntax story is created as a standalone. AC-007
  in STORY-086 was rescoped (pass-5 adjudication, spec v1.4) to unit-test-only coverage
  after confirming BOTH the direct bullets: [...] literal AND the @var list-binding form
  fail to parse (corrected D5 resolution). This story delivers parser support for both
  forms and is the prerequisite for STORY-086 AC-007 E2E closure. Scope expanded from
  5 points (direct-literal only) to 8 points (both forms) per pass-5.
---

# STORY-088: Bullets list-literal field-value DSL syntax

## Uncertainty Resolution (2026-06-06)

This spec was corrected against the real codebase on `develop` @ 030dec6c. Full resolution:
`.factory/specs/wave4-expanded-scope-uncertainty-resolution.md` (D5).

Key corrections applied:
- **Correct file path:** `crates/slideforge-syntax/src/parser/deck.rs` (under `parser/` subdir),
  NOT `crates/slideforge-syntax/src/deck.rs`. The parser module lives under `src/parser/`.
- **Correct AST representation:** Add `FieldValue::List(Vec<FieldValue>)` as a new variant
  to the `FieldValue` enum in `crates/slideforge-syntax/src/ast.rs`. There is NO
  `FieldValue::Literal(Value::List(...))` path — `Value` is an eval-time type; `FieldValue`
  is the parse-time AST type. The eval layer maps `FieldValue::List(items)` → `Value::List(vals)`.
- **Correct chumsky 0.10 idiom (token-stream, not char-stream):** Use
  `just(Token::LBracket)`, `just(Token::RBracket)`, `just(Token::Comma)` — all confirmed
  present in the token set (used in `crates/slideforge-syntax/src/parser/expr.rs` lines 96–103).
  Do NOT use `just('[')` or any char-level combinator.
- **Also update eval:** `crates/slideforge-eval/src/` must handle `FieldValue::List` and
  produce `Value::List(Vec<Value>)`.

**Pass-5 scope expansion (2026-06-06):** The corrected D5 resolution (per STORY-086 pass-5
adjudication) clarified that BOTH bullets-list syntax forms fail to parse today — the direct
`bullets: [...]` field literal AND the `@var items = [...]` / `bullets: items` variable-binding
form. The `@var` form fails because `deck.rs::value_parser()` has no list-literal arm, so
`@var` list assignments do not produce a parseable `FieldValue::List` at all. This story's
scope is therefore expanded to explicitly cover BOTH forms:

1. **Direct `bullets: ["A","B","C"]` field list-literal** — the `FieldValue::List` AST
   variant + list-literal arm in `value_parser()` + `value_parser_for_vars_block()` (if
   separate) in `deck.rs`, and the corresponding `FieldValue::List → Value::List` eval arm.
2. **`@var`/vars-block list assignment form** — the `set_rule_value_parser()` (or equivalent
   vars-block value parser) in `deck.rs` must also accept `[...]` tokens, producing
   `FieldValue::List`. Once parsed, the eval layer resolves the variable to `Value::List`,
   which is then resolvable via `FieldValue::Ident` lookup in `thread_fields_to_blocks`
   to produce `ContentBlock::Bullets`.

Both forms must be covered by unit tests. The E2E fixture proving `bullets:` resolves to
`ContentBlock::Bullets` with ≥3 runs (AC-007 in this story) is the end-to-end closure of
STORY-086 AC-007's deferred coverage. This story is the PREREQUISITE for STORY-086 AC-007's
end-to-end DSL coverage — STORY-086 AC-007's E2E `.sf` fixture must remain `#[ignore]`
until this story merges.

**Re-estimation note:** The scope expansion adds the `@var`/vars-block parsing path and its
eval plumbing in addition to the direct field literal. Original estimate was 5 points
(direct-literal path only). The expanded scope covering both forms is estimated at **8
points** — still within a single story's budget (≤13 points). This change is flagged for
state-manager to reconcile STORY-INDEX and sprint-state.

## Subsystem Anchor Justification

- SS-01 (DSL Parser, `slideforge-syntax`) owns all field-value parsing in
  `crates/slideforge-syntax/src/parser/deck.rs`. The `value_parser()` function currently
  handles `FieldValue::Template(Vec<TemplateChunk>)`, `FieldValue::Num(i64)`,
  `FieldValue::Float(f64)`, `FieldValue::Bool(bool)`, `FieldValue::Ident(String)`, and
  `FieldValue::Error`. It does NOT yet handle `[...]` list-literal tokens.
  The new `FieldValue::List(Vec<FieldValue>)` variant is added to `ast.rs`; `value_parser()`
  in `deck.rs` is extended to parse it using token-stream combinators. The vars-block /
  `@var` value parser (`set_rule_value_parser()` or whichever parser handles the RHS of
  `@var ident = <value>`) is also extended with the same list-literal arm so that
  `@var items = ["A","B","C"]` parses to `FieldValue::List` in the vars-block context.
  If the two parsers share the same `value_parser()` combinator, only one arm is needed.
  This change also touches `slideforge-eval` (to map `FieldValue::List` → `Value::List`),
  but no other crate is modified.

- SS-02 (Evaluator, `slideforge-eval`) is modified to handle `FieldValue::List` in its
  eval dispatch, mapping it to `Value::List(Vec<Value>)`. The `thread_fields_to_blocks`
  function in `field_to_block.rs` already resolves `FieldValue::Ident` lookups to their
  bound `Value`, so a `@var items` reference that evaluates to `Value::List` will be
  routed to `ContentBlock::Bullets` by the existing Stage-2b logic (BC-1.16.001 PC-7).
  SS-02 is the anchor for this eval change because the evaluator crate owns
  `FieldValue` → `Value` dispatch.

## Dependency Anchor Justifications

- Depends on STORY-006 (Parser Core): STORY-006 delivered the foundational field parser
  infrastructure in `deck.rs` (slide fields, indentation, field-value pairs). This story
  adds a new arm to the existing field-value combinator; it cannot be implemented without
  that base infrastructure.

- Depends on STORY-086 (Stage 2b threading): The list-literal syntax is only useful
  once Stage 2b routes `Value::List` items into `ContentBlock::Bullets`. STORY-086
  implements the `Value::List → ContentBlock::Bullets` threading (proven via Stage-2b
  unit test; the E2E DSL path is `#[ignore]`'d pending this story). This story adds
  the DSL parse-layer entry point for both the direct field-literal and the `@var`/vars-block
  form. The order matters: STORY-086 must merge first so the Stage-2b threading infrastructure
  exists, then this story adds the parser support that makes the `Value::List` path
  reachable from `.sf` source.

- Does NOT block any other Wave 5 story. It is a quality-of-life improvement to the
  DSL parser.

## Summary

`slideforge-syntax`'s field-value parser in `deck.rs` does not yet support `[...]`
list-literal tokens as a field value. Consequently, BOTH of the following DSL forms fail
to parse today (confirmed via pass-5 adjudication + corrected D5 resolution):

**Form 1 — Direct field list-literal (fails):**
```
slide content:
  bullets: ["Item A", "Item B", "Item C"]
```

**Form 2 — @var/vars-block list assignment (also fails):**
```
@var items = ["Item A", "Item B", "Item C"]
slide content:
  bullets: items
```

Both forms fail because `deck.rs::value_parser()` (and the vars-block value parser) have
no `[...]` list-literal arm. This story fixes both forms.

This story adds `[...]` list-literal parsing to `value_parser()` and the vars-block value
parser in `crates/slideforge-syntax/src/parser/deck.rs` so that `bullets: ["A","B","C"]`
parses to `FieldValue::List(vec![FieldValue::Template(...), ...])`, and
`@var items = ["A","B","C"]` also parses to `FieldValue::List(...)` in the vars-block context.

The new `FieldValue::List(Vec<FieldValue>)` variant is added to the `FieldValue` enum in
`crates/slideforge-syntax/src/ast.rs`. In `slideforge-eval`, `FieldValue::List(items)` is
evaluated to `Value::List(vals)` by evaluating each item to its `Value` equivalent. For the
`@var` form, the existing `FieldValue::Ident` lookup resolves the bound variable to `Value::List`
in the evaluator environment, then `thread_fields_to_blocks` (BC-1.16.001 PC-7) routes it to
`ContentBlock::Bullets`.

Once `Value::List` is produced by eval (via either form), the existing Stage 2b path in
`thread_fields_to_blocks` routes `Value::List` to `ContentBlock::Bullets` without modification.
Layout and exporters are unchanged.

This story is the PREREQUISITE for STORY-086 AC-007's end-to-end DSL coverage. STORY-086
AC-007's E2E `.sf` bullets fixture is `#[ignore]`'d until this story merges.

**Scope (v1.2 — expanded per pass-5 adjudication):**
- New: `FieldValue::List(Vec<FieldValue>)` variant in `ast.rs`
- New: list-literal arm in `value_parser()` in `crates/slideforge-syntax/src/parser/deck.rs`
  using token-stream combinators: `just(Token::LBracket)` / `just(Token::RBracket)` /
  `just(Token::Comma)` — idiom from `expr.rs` lines 96–103 (covers `bullets: ["A","B","C"]`)
- New: list-literal arm in `set_rule_value_parser()` / vars-block value parser in
  `deck.rs` (covers `@var items = ["A","B","C"]` / vars-block form); if the same
  `value_parser()` combinator is shared, only one arm is needed — confirm by reading
  the actual `deck.rs` implementation before implementing
- New: `FieldValue::List` eval handling in `crates/slideforge-eval/src/` → `Value::List`;
  and `FieldValue::Ident` lookup in `thread_fields_to_blocks` already handles `Value::List`
  variables (the threading pass resolves bound variables via the eval environment)
- Nested list-literals are NOT in scope; only flat `[string1, string2, ...]` literals required
- Existing Stage 2b, evaluator, layout, and exporter paths are otherwise unchanged

**Closes:** The AC-007 parser-gap note in STORY-086. STORY-086 AC-007's E2E `.sf` bullets
fixture is `#[ignore]`'d pending this story. After this story merges, both the direct
`bullets: [...]` form and the `@var` binding form parse correctly, and STORY-086 AC-007's
E2E fixture can be un-ignored.

## Narrative

As a slideforge author who wants to write concise `.sf` files,
I want to declare bullet items inline as `bullets: ["A", "B", "C"]`
so that I don't need to introduce a separate `@var` binding for a simple bullet list.

## Behavioral Contracts

| BC | Title | Version | Role in This Story |
|----|-------|---------|-------------------|
| BC-1.01.002 | Parser Core: Slide Field Parsing | v1.x | Primary: field-value parser extension; the `[...]` list-literal is a new `FieldValue::List(Vec<FieldValue>)` production in the field-value grammar. New variant added to `FieldValue` enum in `ast.rs`. |

**Secondary trace (via AC footnote):**

| BC | Title | Version | Role in This Story |
|----|-------|---------|-------------------|
| BC-1.16.001 | Post-Eval Field-to-Block Threading Pass | v1.0 | The parsed `FieldValue::List` feeds through eval to `Value::List`, then PC-7 — `Value::List(items)` → `ContentBlock::Bullets` — which is already implemented in STORY-086. This story provides the parse-side entry point. |

## Acceptance Criteria

### AC-001 — `bullets: ["A","B","C"]` parses to FieldValue::List(Vec<FieldValue>)
A fixture `.sf` source containing:
```
slide content:
  title "My Slide"
  bullets: ["Item A", "Item B", "Item C"]
```
parses without error. The resulting `SlideNode.fields["bullets"]` is
`FieldValue::List(vec![FieldValue::Template(...), FieldValue::Template(...), FieldValue::Template(...)])`
where each inner item is a `FieldValue::Template` wrapping the string literal. When evaluated,
this produces `Value::List(vec![Value::Str("Item A"), Value::Str("Item B"), Value::Str("Item C")])`.
No parse error is emitted. This is the primary positive case.
(traces to BC-1.01.002 — field-value parser; `FieldValue::List` is a new variant in ast.rs)

### AC-002 — Empty list-literal `bullets: []` parses to FieldValue::List([])
A fixture `.sf` source with `bullets: []` parses successfully to `FieldValue::List(vec![])`.
No error is emitted. The empty-list case is valid per BC-1.16.001 EC-008 (empty bullets
produces `ContentBlock::Bullets(vec![])`).
(traces to BC-1.01.002 — field-value parser handles edge case of empty list literal;
secondary trace: BC-1.16.001 EC-008 — empty list is valid)

### AC-003 — Single-item list `bullets: ["Only"]` parses correctly
A fixture `.sf` source with `bullets: ["Only"]` parses to
`FieldValue::List(vec![FieldValue::Template(...)])`. Single-item lists must not be
treated as a bare string. When evaluated: `Value::List(vec![Value::Str("Only")])`.
(traces to BC-1.01.002 — field-value parser; single-element list)

### AC-004 — List-literal parses inside standard slide block indentation
The `[...]` list-literal must be parseable as a field value in the standard
field-position (indent-significant, after `key:` on the same line). Example:
```
slide content:
  title "Agenda"
  bullets: ["Step 1", "Step 2", "Step 3"]
  body "See notes."
```
All four fields parse correctly; no off-by-one indentation error or partial-parse failure.
(traces to BC-1.01.002 — field-value parser integrates with indentation-sensitive slide field grammar)

### AC-005 — Non-string list items produce E-PAR diagnostic, NOT a panic
A fixture `.sf` source with `bullets: [42, true]` (non-string list items) produces a
parser diagnostic (`E-PAR-NNN` or equivalent) describing the type mismatch, not a panic
or silent wrong value. Error accumulation (BC-1.15.001) applies: the parser continues
and reports all errors in the file.
(traces to BC-1.01.002 — parser error path for invalid list-element types;
BC-1.15.001 invariant — error accumulation, no fail-on-first)

### AC-006 — @var binding form continues to work (no regression)
A fixture `.sf` source using the `@var items = [...]` + `bullets: items` pattern
(the existing workaround from STORY-086 AC-007) still parses and builds correctly.
No regression in the `@var` path.
(traces to BC-1.01.002 — regression guard on variable-reference field values)

### AC-007 — E2E: `bullets: ["A","B","C"]` produces ≥3 text runs in PPTX output
A full build using `bullets: ["Item A", "Item B", "Item C"]` (direct list-literal,
no `@var`) produces a PPTX where the content slide contains at least 3 `<a:r>` text
runs with the corresponding bullet strings. This test is the end-to-end closure of the
STORY-086 AC-007 parser-gap note.
(traces to BC-1.01.002 — parse entry point; secondary trace: BC-1.16.001 postcondition 7 —
Value::List → ContentBlock::Bullets → layout frames → PPTX text runs)

## Architecture Mapping

| Component | Crate | File | Change Type | Pure/Effectful |
|-----------|-------|------|-------------|---------------|
| `FieldValue::List` variant | `slideforge-syntax` | `src/ast.rs` | Add `List(Vec<FieldValue>)` variant to `FieldValue` enum | Pure (AST change) |
| `value_parser()` extension (field literal) | `slideforge-syntax` | `src/parser/deck.rs` | Add `FieldValue::List` combinator arm using `just(Token::LBracket)` / `separated_by(just(Token::Comma))` / `just(Token::RBracket)` — idiom from `src/parser/expr.rs` lines 96–103. Covers `bullets: ["A","B","C"]` field-literal form. | Pure |
| vars-block / `@var` value parser extension | `slideforge-syntax` | `src/parser/deck.rs` | Extend `set_rule_value_parser()` or the shared `value_parser()` (whichever handles `@var ident = <value>` RHS) with the same list-literal arm so `@var items = ["A","B","C"]` parses to `FieldValue::List`. If `value_parser()` is shared, this is the SAME arm — no duplicate implementation needed. Confirm by reading `deck.rs` before implementing. | Pure |
| Parser error for non-string list items | `slideforge-syntax` | `src/parser/deck.rs` | Error recovery for non-string items → `E-PAR-NNN` diagnostic | Pure |
| `FieldValue::List` eval handling | `slideforge-eval` | `src/` (eval dispatch) | Map `FieldValue::List(items)` → evaluate each item → `Value::List(vals)`. Also ensures `FieldValue::Ident` lookup resolves to `Value::List` for the `@var` form (already works if the eval environment stores `Value::List`). | Pure |

**Forbidden Dependencies:**
- `slideforge-syntax` MUST NOT import `slideforge-eval`, `slideforge-layout`, or any
  exporter crate. The parser is the leaf of the dependency chain. Adding any downstream
  dependency would create a cycle. Build-time enforcement: `cargo deny`.
- `slideforge-eval` changes are IN SCOPE: `FieldValue::List` eval dispatch is required
  so that the parsed list reaches `Value::List` and Stage 2b can produce `ContentBlock::Bullets`.
- No changes to `slideforge-layout`, `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`,
  or `slideforge-html`. If those crates require changes, the story spec must be updated
  and rescoped.

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec (v1.2, expanded) | ~3,500 |
| BC-1.01.002 (field-value grammar section) | ~1,500 |
| BC-1.16.001 PC-7 (bullets from Value::List) | ~500 |
| BC-1.15.001 (error accumulation) | ~500 |
| `crates/slideforge-syntax/src/ast.rs` (FieldValue enum) | ~1,000 |
| `crates/slideforge-syntax/src/parser/deck.rs` (value_parser + vars-block parser) | ~4,000 |
| `crates/slideforge-syntax/src/parser/expr.rs` lines 96–103 (chumsky list idiom template) | ~300 |
| `crates/slideforge-eval/src/` (FieldValue::List eval dispatch + FieldValue::Ident lookup path) | ~1,500 |
| Unit test files (field-literal + @var forms, new) | ~2,500 |
| E2E integration test update (AC-007 + @var regression AC-006) | ~1,500 |
| Tool outputs (compiler, test results) | ~1,500 |
| **TOTAL ESTIMATED** | **~18,300 tokens** |

18,300 tokens is ~9% of a 200k context window — well within the 20-30% per-story budget.
Scope increased from original ~13,500 (direct-literal only) due to the addition of the
`@var`/vars-block parsing path (pass-5 scope expansion).

## Previous Story Intelligence

- **STORY-086 (parent context, corrected AC-007 per pass-5):** AC-007 in STORY-086 was
  corrected in spec v1.4 (pass-5 adjudication): BOTH the direct `bullets: [...]` literal
  form AND the `@var items = [...]` variable-binding form fail to parse today. The original
  D5 resolution incorrectly claimed the `@var` form was "confirmed to produce Value::List
  from the existing evaluator" — this was wrong. Neither form hits the eval layer because
  `deck.rs::value_parser()` has no list-literal arm for either context. STORY-086 AC-007's
  E2E `.sf` bullets fixture is therefore `#[ignore]`'d and awaits this story. STORY-086
  AC-007's load-bearing test is the Stage-2b unit test only (programmatic `Value::List`
  construction, bypassing the parser). The exact parse failure: when `[` is encountered as
  a field-value start token, the parser does not recognize it and produces a parse error.

- **STORY-006 (Parser Core):** STORY-006 established the indentation-sensitive field
  parser in `crates/slideforge-syntax/src/parser/deck.rs`. The implementer must read
  the existing `value_parser()` function in that file to understand the current chumsky
  0.10 token-stream combinator pattern before adding the new arm.

  The chumsky 0.10 token-stream list idiom is demonstrated at
  `crates/slideforge-syntax/src/parser/expr.rs` lines 96–103:
  ```rust
  let list = e
      .clone()
      .separated_by(just(Token::Comma))
      .allow_trailing()
      .collect::<Vec<_>>()
      .delimited_by(just(Token::LBracket), just(Token::RBracket))
      .map(Expr::List);
  ```
  The `FieldValue::List` parser in `deck.rs` follows the same pattern, using `value_parser_ref`
  (recursive reference) in place of `e`, and mapping to `FieldValue::List(items)` instead
  of `Expr::List`. `Token::LBracket`, `Token::RBracket`, and `Token::Comma` are confirmed
  present in the token set (used in `expr.rs`). Do NOT use `just('[')` — this is a
  char-stream idiom and will not work in chumsky 0.10's token-stream parser.

- **STORY-010 (Error Accumulation):** Error recovery pattern: use `recover_with` or
  `map_err` to turn list-literal parse failures into `E-PAR-NNN` diagnostics without
  aborting the entire file parse.

- **LESSON-16 (STORY-083):** Run `cargo clippy --workspace --all-targets -- -D warnings`
  before declaring convergence. New parser combinators with public error types require
  rustdoc (`#![warn(missing_docs)]`).

## Architecture Compliance Rules

These rules are extracted from the DSL spec and chumsky usage in `slideforge-syntax`.
Violating any constitutes a finding in adversarial review.

1. **No tabs, only spaces** (CLAUDE.md DSL convention). The parser already rejects tabs
   at the indentation level; the list-literal combinator must also reject tab characters
   within list items (or treat them as a parse error, not silently include them).

2. **Error accumulation — no fail-on-first** (BC-1.15.001). The list-literal combinator
   must use chumsky's `recover_with` or equivalent so that a malformed list does not
   abort parsing of the rest of the slide. All errors in the file must be reported.

3. **No implicit type coercion** (CLAUDE.md Forbidden Patterns). `NO` stays string "NO",
   `1.10` stays "1.10". When parsing list items, do not coerce numeric strings to numbers
   or boolean strings to bools. All list items for `bullets:` are expected to be
   `Value::Str`. Non-string items produce a diagnostic (AC-005).

4. **`#![forbid(unsafe_code)]`** — No `unsafe` blocks permitted in parser additions.

5. **Indentation invariant** — The `[...]` list literal must be parseable on a single
   line immediately after `bullets:`. Multi-line list literals (with items on separate
   lines) are NOT required for v1.0; single-line form only. The `Token::LBracket` token
   must appear on the same line as the field key. Multi-line parsing produces a descriptive
   diagnostic (see EC-006), not a cryptic failure.

6. **Token-stream combinators only** — `chumsky` 0.10 in this project operates on a
   token stream, not a character stream. Use `just(Token::LBracket)`, `just(Token::Comma)`,
   `just(Token::RBracket)`. Do NOT use `just('[')`, `just(',')`, `just(']')` — those are
   char-stream combinators and will not compile against the `Token` type. Template:
   `crates/slideforge-syntax/src/parser/expr.rs` lines 96–103.

## Library and Framework Requirements

All versions from the project dependency graph (authoritative source:
`crates/slideforge-syntax/Cargo.toml`):

| Library | Version | Purpose |
|---------|---------|---------|
| `chumsky` | 0.10+ (workspace) | Parser combinator; use `just(Token::LBracket)` / `just(Token::RBracket)` / `just(Token::Comma)` — token-stream not char-stream |
| `miette` (via `slideforge-syntax`) | workspace | Diagnostic span attachment for AC-005 error case |
| `slideforge-syntax` internal | n/a | `FieldValue` enum in `ast.rs` receives new `List(Vec<FieldValue>)` variant; `Token` enum in `token.rs` provides `LBracket`/`RBracket`/`Comma` |
| `slideforge-eval` | workspace path | Must handle `FieldValue::List` in eval dispatch, mapping to `Value::List(Vec<Value>)` |

No new external dependencies are added by this story.

## File Structure Requirements

Files to MODIFY:
```
crates/slideforge-syntax/src/ast.rs             [add FieldValue::List(Vec<FieldValue>) variant to FieldValue enum;
                                                  update all match arms on FieldValue to add the new variant]
crates/slideforge-syntax/src/parser/deck.rs     [extend value_parser() with list-literal arm using Token::LBracket/Comma/RBracket;
                                                  extend vars-block value parser (set_rule_value_parser() or shared value_parser())
                                                  with the same list-literal arm so @var items = [...] parses to FieldValue::List;
                                                  add error recovery for non-string list items → E-PAR diagnostic]
crates/slideforge-eval/src/                     [handle FieldValue::List(items) in eval dispatch → Value::List(vals);
                                                  grep for the FieldValue match/dispatch location and add the new arm;
                                                  confirm FieldValue::Ident lookup resolves @var-bound Value::List correctly
                                                  (may require no change if environment stores Value)]
```
NOTE: The file `crates/slideforge-syntax/src/deck.rs` does NOT exist at the top level —
the real path is `crates/slideforge-syntax/src/parser/deck.rs`. Confirm before starting.
Read `deck.rs` in full before implementing to determine whether `value_parser()` is shared
between field-literal and vars-block contexts (one arm) or separate (two arms).

Files that RECEIVE new tests:
```
crates/slideforge-syntax/src/parser/deck.rs    [cfg(test) block: add AC-001 through AC-006 unit tests,
                                                  plus @var/vars-block list-form unit test]
tests/integration/e2e_build_tests.rs           [un-ignore STORY-086 AC-007 #[ignore]'d fixture;
                                                  add AC-007 E2E test for direct list-literal bullets]
```

Forbidden: Do NOT modify `slideforge-layout`, `slideforge-pptx`, `slideforge-docx`,
`slideforge-pdf`, or `slideforge-html`. The only production code changes are in
`crates/slideforge-syntax/src/ast.rs`, `crates/slideforge-syntax/src/parser/deck.rs`,
and `crates/slideforge-eval/src/`.

## Tasks

- [ ] **T1 — Red Gate: write all failing tests first**
  - [ ] T1.1: Write unit tests for the list-literal parser arm — direct field-literal form (AC-001, AC-002, AC-003, AC-004, AC-005) in `crates/slideforge-syntax/src/parser/deck.rs` `#[cfg(test)] mod tests` block. Tests must FAIL (FieldValue::List variant not yet defined).
  - [ ] T1.2: Write unit test for the `@var`/vars-block list assignment form (AC-006-ext: `@var items = ["A","B","C"]` parses to FieldValue::List in the vars-block context). Test must FAIL.
  - [ ] T1.3: Write regression test for the existing `@var` + `bullets: items` ident-reference path (original AC-006 regression guard). Must FAIL because the @var list form now requires parsing support.
  - [ ] T1.4: Update E2E integration test for AC-007 (direct `bullets: ["A","B","C"]` without @var) in `tests/integration/e2e_build_tests.rs`. Un-ignore the `#[ignore]`'d STORY-086 AC-007 fixture. Test must FAIL until implementation is complete.
  - [ ] T1.5: Verify Red Gate density ≥0.5 before implementation starts.

- [ ] **T2 — Add FieldValue::List variant and implement value_parser() extension**
  - [ ] T2.1: In `crates/slideforge-syntax/src/ast.rs`, add `List(Vec<FieldValue>)` variant to the `FieldValue` enum. Ensure all existing `match` arms on `FieldValue` compile (add the new arm everywhere — `non_exhaustive_patterns` will catch missing arms).
  - [ ] T2.2: READ `crates/slideforge-syntax/src/parser/deck.rs` in full before implementing. Determine whether `value_parser()` is shared between the field-literal context and the vars-block/`@var` context, or whether there is a separate `set_rule_value_parser()`. If shared: one arm covers both forms. If separate: add the list-literal arm to BOTH parsers.
  - [ ] T2.3: In `value_parser()`, add a new `list_val` combinator following the `expr.rs` lines 96–103 template:
    ```rust
    let list_val = value_parser_ref
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
        .map(|items| FieldValue::List(items));
    ```
    Add as a new alternative: `.or(list_val)`.
    Use `Token::LBracket` / `Token::RBracket` / `Token::Comma` — NOT char literals.
  - [ ] T2.4: If a separate vars-block value parser exists, apply the same list-literal arm to it (or confirm the shared combinator covers it). Cite the relevant `deck.rs` line numbers in the PR.
  - [ ] T2.5: Add error recovery: when a list item is not a string literal, produce `E-PAR-NNN` diagnostic and continue. Do NOT abort the file parse.
  - [ ] T2.6: In `crates/slideforge-eval/src/`, locate the `FieldValue` match/dispatch (grep for `FieldValue::Template` or `FieldValue::Num`). Add arm: `FieldValue::List(items) => Value::List(items.into_iter().map(|item| eval_field_value(item, ctx)).collect())`.
  - [ ] T2.7: Verify `FieldValue::Ident` lookup in the eval environment: when `@var items` was parsed as `FieldValue::List`, it should be stored in the environment as `Value::List`. The `bullets: items` field is `FieldValue::Ident("items")`; the eval layer looks up `"items"` and finds `Value::List(...)` — no code change needed if the environment already stores `Value` (not `FieldValue`). Confirm by grepping for variable storage in the eval layer.
  - [ ] T2.8: Ensure the new arm integrates with indentation-sensitive field context (list starts on same line as `bullets:` or `@var`). Confirm with parse tests at T3.1.

- [ ] **T3 — Green pass: all ACs passing**
  - [ ] T3.1: Run `cargo nextest run -p slideforge-syntax --no-fail-fast` — all AC-001..AC-006 unit tests pass, including the `@var`/vars-block list form test.
  - [ ] T3.2: Run `cargo nextest run -p slideforge-eval --no-fail-fast` — FieldValue::List eval produces Value::List; @var → FieldValue::Ident lookup resolves to Value::List.
  - [ ] T3.3: Run E2E integration test for AC-007 (direct list-literal bullets produce PPTX text runs with ≥3 `<a:r>` text runs).
  - [ ] T3.4: Run `just check` (full workspace pre-push gate: fmt + clippy pedantic + nextest + doctests).

- [ ] **T4 — Regression check: all bullets paths still green**
  - [ ] T4.1: Confirm the `@var items = [...]` + `bullets: items` E2E path works end-to-end (un-ignored STORY-086 AC-007 fixture passes).
  - [ ] T4.2: Confirm no regression in non-bullets field-value parsing (string templates, numeric fields, bool fields, ident references).

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | AC-002 | `bullets: []` (empty list) | `FieldValue::Literal(Value::List(vec![]))` — valid, produces `ContentBlock::Bullets(vec![])` |
| EC-002 | AC-005 | `bullets: [42, true]` (non-string items) | E-PAR diagnostic, not a panic; error accumulation continues parsing |
| EC-003 | CLAUDE.md | `bullets: ["A","B",]` trailing comma | Parser must either accept trailing comma (preferred) or produce a useful diagnostic — NOT a silent wrong parse |
| EC-004 | CLAUDE.md | `bullets: [""]` single empty-string item | `Value::List([Value::Str("")])` parsed; Stage 2b BC-1.16.001 EC-001 governs whether empty-string bullets are threaded |
| EC-005 | BC-1.15.001 | `bullets: ["A", 42, "C"]` mixed types | All errors collected; at minimum "42" causes E-PAR; parser continues to parse "C" |
| EC-006 | CLAUDE.md | Multi-line `bullets:` with `[` on next line (indented) | Not required in v1.0 (single-line only). If attempted, parser produces a descriptive E-PAR explaining single-line requirement, not a cryptic failure. |

## Dependency Graph

```
STORY-006 (Parser Core: deck.rs field parser infrastructure)
  └─ STORY-086 (Stage 2b: Value::List → ContentBlock::Bullets path established)
       └─ [STORY-088 — THIS STORY — list-literal field-value DSL syntax] (Wave 5)
```

This story has no `blocks:` entries — it is a quality-of-life improvement that no
other Wave 5 story depends on.
