---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-088
title: "Bullets list-literal field-value DSL syntax (`bullets: [\"A\",\"B\",\"C\"]`)"
epic: EPIC-02
wave: 5
points: 5
priority: P1
tdd_mode: strict
status: draft
spec_version: "1.1"
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
estimated_days: 2
history_note: >
  Created 2026-06-06 to close the parser gap discovered during STORY-086 (Stage 2b
  field-to-block threading, scope expansion authorized 2026-06-06). The human authorized
  "both follow-up stories" (TextTag story + bullets-syntax story). The TextTag follow-up
  story was ABSORBED into STORY-086 — TextTag is fully implemented in-scope there, not
  in a separate story. Only this bullets-syntax story is created as a standalone. AC-007
  in STORY-086 was fixed to use the @var variable-binding workaround; this story
  implements the direct list-literal syntax that enables `bullets: ["A","B","C"]`
  without the workaround.
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

## Subsystem Anchor Justification

- SS-01 (DSL Parser, `slideforge-syntax`) owns all field-value parsing in
  `crates/slideforge-syntax/src/parser/deck.rs`. The `value_parser()` function currently
  handles `FieldValue::Template(Vec<TemplateChunk>)`, `FieldValue::Num(i64)`,
  `FieldValue::Float(f64)`, `FieldValue::Bool(bool)`, `FieldValue::Ident(String)`, and
  `FieldValue::Error`. It does NOT yet handle `[...]` list-literal tokens.
  The new `FieldValue::List(Vec<FieldValue>)` variant is added to `ast.rs`; `value_parser()`
  in `deck.rs` is extended to parse it using token-stream combinators.
  This change also touches `slideforge-eval` (to map `FieldValue::List` → `Value::List`),
  but no other crate is modified.

## Dependency Anchor Justifications

- Depends on STORY-006 (Parser Core): STORY-006 delivered the foundational field parser
  infrastructure in `deck.rs` (slide fields, indentation, field-value pairs). This story
  adds a new arm to the existing field-value combinator; it cannot be implemented without
  that base infrastructure.

- Depends on STORY-086 (Stage 2b threading): The list-literal syntax is only useful
  once Stage 2b routes `Value::List` items into `ContentBlock::Bullets`. STORY-086
  implements the `Value::List → ContentBlock::Bullets` threading. This story adds the
  DSL entry point. The order matters: STORY-086 must merge first so the E2E test path
  is proven to work via `@var` binding, then this story adds the direct syntax.

- Does NOT block any other Wave 5 story. It is a quality-of-life improvement to the
  DSL parser; existing workaround (`@var` binding) remains valid.

## Summary

`slideforge-syntax`'s field-value parser in `deck.rs` does not yet support
`[...]` list-literal tokens as a field value. Consequently, the DSL snippet:

```
slide content:
  bullets: ["Item A", "Item B", "Item C"]
```

fails to parse, forcing authors to use the `@var` binding workaround:

```
@var items = ["Item A", "Item B", "Item C"]
slide content:
  bullets: items
```

This story adds `[...]` list-literal parsing to the `value_parser()` function in
`crates/slideforge-syntax/src/parser/deck.rs` so that `bullets: ["A","B","C"]` parses to
`FieldValue::List(vec![FieldValue::Template(...), FieldValue::Template(...), FieldValue::Template(...)])`.

The new `FieldValue::List(Vec<FieldValue>)` variant is added to the `FieldValue` enum in
`crates/slideforge-syntax/src/ast.rs`. In `slideforge-eval`, `FieldValue::List(items)` is
evaluated to `Value::List(vals)` by evaluating each item to its `Value` equivalent.

Once `Value::List` is produced by eval, the existing Stage 2b path (`thread_fields_to_blocks`,
BC-1.16.001 PC-7) routes `Value::List` to `ContentBlock::Bullets` without modification.
Layout and exporters handle `Value::List` already.

**Scope:**
- New: `FieldValue::List(Vec<FieldValue>)` variant in `ast.rs`
- New: list-literal arm in `value_parser()` in `crates/slideforge-syntax/src/parser/deck.rs`
  using token-stream combinators: `just(Token::LBracket)` / `just(Token::RBracket)` /
  `just(Token::Comma)` — idiom from `expr.rs` lines 96–103
- New: `FieldValue::List` eval handling in `crates/slideforge-eval/src/` → `Value::List`
- The `@var` binding form continues to work (no regression)
- Nested list-literals are NOT in scope; only flat string item lists required

**Scope:**
- New: `[...]` list-literal parsing in the field-value parser of `deck.rs`
- Existing Stage 2b, evaluator, layout, and exporter paths are unchanged
- The `@var` binding form continues to work (no regression)
- Nested list-literals (lists of lists) are NOT in scope; only flat
  `[string1, string2, ...]` literals are required (bullet items are strings)

**Closes:** The AC-007 parser-gap note in STORY-086. STORY-086 AC-007 uses the `@var`
workaround; after this story merges, future authors can use either form.

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
| `value_parser()` extension | `slideforge-syntax` | `src/parser/deck.rs` | Add `FieldValue::List` combinator arm using `just(Token::LBracket)` / `separated_by(just(Token::Comma))` / `just(Token::RBracket)` — idiom from `src/parser/expr.rs` lines 96–103 | Pure |
| Parser error for non-string list items | `slideforge-syntax` | `src/parser/deck.rs` | Error recovery for non-string items → `E-PAR-NNN` diagnostic | Pure |
| `FieldValue::List` eval handling | `slideforge-eval` | `src/` (eval dispatch) | Map `FieldValue::List(items)` → evaluate each item → `Value::List(vals)` | Pure |

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
| This story spec | ~2,500 |
| BC-1.01.002 (field-value grammar section) | ~1,500 |
| BC-1.16.001 PC-7 (bullets from Value::List) | ~500 |
| BC-1.15.001 (error accumulation) | ~500 |
| `crates/slideforge-syntax/src/ast.rs` (FieldValue enum) | ~1,000 |
| `crates/slideforge-syntax/src/parser/deck.rs` (value_parser function) | ~3,000 |
| `crates/slideforge-syntax/src/parser/expr.rs` lines 96–103 (chumsky list idiom template) | ~300 |
| `crates/slideforge-eval/src/` (FieldValue::List eval dispatch) | ~1,000 |
| Unit test file (new) | ~1,500 |
| E2E integration test update (AC-007) | ~1,000 |
| Tool outputs (compiler, test results) | ~1,500 |
| **TOTAL ESTIMATED** | **~13,500 tokens** |

13,500 tokens is ~7% of a 200k context window — well within the 20-30% per-story budget.

## Previous Story Intelligence

- **STORY-086 (parent context):** AC-007 in STORY-086 documented the parser gap and
  used the `@var` workaround. The exact error behavior when `bullets: ["A","B","C"]`
  is attempted was documented there: the field-value parser does not recognize `[` as a
  valid field-value start token. This story fixes that.

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
crates/slideforge-syntax/src/ast.rs             [add FieldValue::List(Vec<FieldValue>) variant to FieldValue enum]
crates/slideforge-syntax/src/parser/deck.rs     [extend value_parser() with list-literal arm using Token::LBracket/Comma/RBracket;
                                                  add error recovery for non-string list items → E-PAR diagnostic]
crates/slideforge-eval/src/                     [handle FieldValue::List(items) in eval dispatch → Value::List(vals);
                                                  grep for the FieldValue match/dispatch location and add the new arm]
```
NOTE: The file `crates/slideforge-syntax/src/deck.rs` does NOT exist at the top level —
the real path is `crates/slideforge-syntax/src/parser/deck.rs`. Confirm before starting.

Files that RECEIVE new tests:
```
crates/slideforge-syntax/src/parser/deck.rs    [cfg(test) block: add AC-001 through AC-006 unit tests]
tests/integration/e2e_build_tests.rs           [update: add AC-007 E2E test for direct list-literal bullets]
```

Forbidden: Do NOT modify `slideforge-layout`, `slideforge-pptx`, `slideforge-docx`,
`slideforge-pdf`, or `slideforge-html`. The only production code changes are in
`crates/slideforge-syntax/src/ast.rs`, `crates/slideforge-syntax/src/parser/deck.rs`,
and `crates/slideforge-eval/src/`.

## Tasks

- [ ] **T1 — Red Gate: write all failing tests first**
  - [ ] T1.1: Write unit tests for the list-literal parser arm (AC-001, AC-002, AC-003, AC-004, AC-005, AC-006)
    in `crates/slideforge-syntax/src/parser/deck.rs` `#[cfg(test)] mod tests` block.
    Tests must FAIL (FieldValue::List variant not yet defined).
  - [ ] T1.2: Update E2E integration test for AC-007 (direct `bullets: ["A","B","C"]` without @var)
    in `tests/integration/e2e_build_tests.rs`. Test must FAIL.
  - [ ] T1.3: Verify Red Gate density ≥0.5 before implementation starts.

- [ ] **T2 — Add FieldValue::List variant and implement value_parser() extension**
  - [ ] T2.1: In `crates/slideforge-syntax/src/ast.rs`, add `List(Vec<FieldValue>)` variant to
    the `FieldValue` enum. Ensure all existing `match` arms on `FieldValue` compile
    (add the new arm everywhere — `non_exhaustive_patterns` will catch missing arms).
  - [ ] T2.2: In `crates/slideforge-syntax/src/parser/deck.rs`, locate `value_parser()` and add a
    new `list_val` combinator following the `expr.rs` lines 96–103 template:
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
  - [ ] T2.3: Add error recovery: when a list item is not a string literal, produce `E-PAR-NNN`
    diagnostic and continue. Do NOT abort the file parse.
  - [ ] T2.4: In `crates/slideforge-eval/src/`, locate the `FieldValue` match/dispatch (grep for
    `FieldValue::Template` or `FieldValue::Num` to find the eval function). Add arm:
    `FieldValue::List(items) => Value::List(items.into_iter().map(|item| eval_field_value(item, ctx)).collect())`.
  - [ ] T2.5: Ensure the new arm integrates with indentation-sensitive field context (list starts
    on same line as `bullets:`). Confirm with a parse test at T3.1.

- [ ] **T3 — Green pass: all ACs passing**
  - [ ] T3.1: Run `cargo nextest run -p slideforge-syntax --no-fail-fast` — all AC-001..AC-006 unit tests pass.
    (Tests are in `crates/slideforge-syntax/src/parser/deck.rs` cfg(test) block.)
  - [ ] T3.2: Run `cargo nextest run -p slideforge-eval --no-fail-fast` — FieldValue::List eval produces Value::List.
  - [ ] T3.3: Run E2E integration test for AC-007 — direct list-literal bullets produce PPTX text runs.
  - [ ] T3.4: Run `just check` (full workspace pre-push gate: fmt + clippy pedantic + nextest + doctests).

- [ ] **T4 — Regression check: @var binding form unchanged**
  - [ ] T4.1: Run the `@var` binding test from STORY-086 AC-007 — must still pass.

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
