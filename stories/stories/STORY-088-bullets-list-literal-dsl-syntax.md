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
spec_version: "1.0"
last_updated: "2026-06-06"
target_module: slideforge-syntax
subsystems: [SS-01]
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

## Subsystem Anchor Justification

- SS-01 (DSL Parser, `slideforge-syntax`) owns all field-value parsing in `deck.rs`. The
  `field_value_parser()` (or equivalent combinator in `deck.rs`) currently handles
  `FieldValue::Literal(Value::Str)`, `FieldValue::Literal(Value::Bool)`,
  `FieldValue::Literal(Value::Number)`, and `FieldValue::Inlines(...)`. It does NOT yet
  handle `[...]` list-literal tokens as `FieldValue::Literal(Value::List(...))`.
  SS-01 is the single-responsibility owner of the parser; this change stays entirely
  within `slideforge-syntax`. No other crate is modified.

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

This story adds `[...]` list-literal parsing to the field-value combinator in
`slideforge-syntax/src/deck.rs` so that `bullets: ["A","B","C"]` parses to
`FieldValue::Literal(Value::List(vec![Value::Str("A"), Value::Str("B"), Value::Str("C")]))`.

Once parsed, the existing Stage 2b path (`thread_fields_to_blocks`, BC-1.16.001 PC-7)
routes `Value::List` to `ContentBlock::Bullets` without modification. The evaluator,
layout, and exporters all handle `Value::List` already — this change is exclusively in
the parser.

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
| BC-1.01.002 | Parser Core: Slide Field Parsing | v1.x | Primary: field-value parser extension; the `[...]` list-literal is a new `FieldValue::Literal(Value::List(...))` production in the field-value grammar |

**Secondary trace (via AC footnote):**

| BC | Title | Version | Role in This Story |
|----|-------|---------|-------------------|
| BC-1.16.001 | Post-Eval Field-to-Block Threading Pass | v1.0 | The parsed `Value::List` feeds PC-7 — `FieldValue::Literal(Value::List(items))` → `ContentBlock::Bullets` — which is already implemented in STORY-086. This story provides the parse-side entry point. |

## Acceptance Criteria

### AC-001 — `bullets: ["A","B","C"]` parses to FieldValue::Literal(Value::List)
A fixture `.sf` source containing:
```
slide content:
  title "My Slide"
  bullets: ["Item A", "Item B", "Item C"]
```
parses without error. The resulting `SlideNode.fields["bullets"]` is
`FieldValue::Literal(Value::List(vec![Value::Str("A"), Value::Str("B"), Value::Str("C")]))`.
No parse error is emitted. This is the primary positive case.
(traces to BC-1.01.002 — field-value parser; the `[...]` production is a new field-value form)

### AC-002 — Empty list-literal `bullets: []` parses to Value::List([])
A fixture `.sf` source with `bullets: []` parses successfully to
`FieldValue::Literal(Value::List(vec![]))`. No error is emitted. The empty-list case
is valid per BC-1.16.001 EC-008 (empty bullets produces `ContentBlock::Bullets(vec![])`).
(traces to BC-1.01.002 — field-value parser handles edge case of empty list literal;
secondary trace: BC-1.16.001 EC-008 — empty list is valid)

### AC-003 — Single-item list `bullets: ["Only"]` parses correctly
A fixture `.sf` source with `bullets: ["Only"]` parses to
`FieldValue::Literal(Value::List(vec![Value::Str("Only")]))`. Single-item lists must
not be treated as a bare string.
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
| `field_value_parser` (or equivalent) | `slideforge-syntax` | `src/deck.rs` | Add `[...]` list-literal combinator arm to field-value parser | Pure |
| Parser error for non-string list items | `slideforge-syntax` | `src/deck.rs` | Error variant or diagnostic for type mismatch in list items | Pure |

**Forbidden Dependencies:**
- `slideforge-syntax` MUST NOT import `slideforge-eval`, `slideforge-layout`, or any
  exporter crate. The parser is the leaf of the dependency chain. Adding any downstream
  dependency would create a cycle. Build-time enforcement: `cargo deny`.
- No changes to `slideforge-eval`, `slideforge-layout`, `slideforge-pptx`, `slideforge-docx`,
  or `slideforge-pdf`. If those crates require changes, the story spec must be updated
  and rescoped.

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| BC-1.01.002 (field-value grammar section) | ~1,500 |
| BC-1.16.001 PC-7 (bullets from Value::List) | ~500 |
| BC-1.15.001 (error accumulation) | ~500 |
| `crates/slideforge-syntax/src/deck.rs` (field-value parser section) | ~3,000 |
| `crates/slideforge-syntax/src/lib.rs` (public API) | ~500 |
| Existing list-literal parsing in evaluator (for reference) | ~1,000 |
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
  parser in `deck.rs`. The implementer should read the existing `field_value_parser`
  combinator to understand the chumsky combinator pattern before adding the new arm.
  Expected pattern: `just('[').then(string_literal().separated_by(just(',')).collect::<Vec<_>>()).then(just(']'))`.

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
   lines) are NOT required for v1.0; single-line form only.

## Library and Framework Requirements

All versions from the project dependency graph (authoritative source:
`crates/slideforge-syntax/Cargo.toml`):

| Library | Version | Purpose |
|---------|---------|---------|
| `chumsky` | 0.10+ (workspace) | Parser combinator for the new list-literal arm |
| `miette` (via `slideforge-syntax`) | workspace | Diagnostic span attachment for AC-005 error case |
| `slideforge-types` | workspace path | `Value::List`, `FieldValue::Literal` |

No new external dependencies are added by this story.

## File Structure Requirements

Files to MODIFY:
```
crates/slideforge-syntax/src/deck.rs    [add list-literal arm to field_value_parser; add E-PAR error for non-string items]
```

Files that RECEIVE new tests:
```
crates/slideforge-syntax/src/deck.rs   [cfg(test) block: add AC-001 through AC-006 unit tests]
tests/integration/e2e_build_tests.rs   [update: add AC-007 E2E test for direct list-literal bullets]
```

Forbidden: Do NOT modify `slideforge-eval`, `slideforge-layout`, `slideforge-pptx`,
`slideforge-docx`, `slideforge-pdf`, or `slideforge-html`. The only production code
change is in `crates/slideforge-syntax/src/deck.rs`.

## Tasks

- [ ] **T1 — Red Gate: write all failing tests first**
  - [ ] T1.1: Write unit tests for the list-literal parser arm (AC-001, AC-002, AC-003, AC-004, AC-005, AC-006) in `deck.rs` `#[cfg(test)] mod tests` block. Tests must FAIL (parser arm not yet implemented).
  - [ ] T1.2: Update E2E integration test for AC-007 (direct `bullets: ["A","B","C"]` without @var). Test must FAIL.
  - [ ] T1.3: Verify Red Gate density ≥0.5 before implementation starts.

- [ ] **T2 — Implement list-literal arm in field_value_parser**
  - [ ] T2.1: In `deck.rs`, locate the `field_value_parser` combinator (or equivalent) and add a new arm for `just('[') ... just(']')` that collects comma-separated string literals into `Value::List(...)`.
  - [ ] T2.2: Add error recovery: when a list item is not a string literal, produce `E-PAR-NNN` diagnostic and continue. Do NOT abort the file parse.
  - [ ] T2.3: Ensure the new arm integrates with the indentation-sensitive field context (the `[...]` list starts on the same line as `bullets:`).

- [ ] **T3 — Green pass: all ACs passing**
  - [ ] T3.1: Run `cargo nextest run -p slideforge-syntax --no-fail-fast` — all AC-001..AC-006 unit tests pass.
  - [ ] T3.2: Run E2E integration test for AC-007 — direct list-literal bullets produce PPTX text runs.
  - [ ] T3.3: Run `just check` (full workspace pre-push gate: fmt + clippy pedantic + nextest + doctests).

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
