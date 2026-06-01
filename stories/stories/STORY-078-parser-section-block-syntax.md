---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-078
title: "Parser: section block syntax (section <type>: ... with sub-blocks)"
epic: EPIC-02
wave: 4
points: 5
priority: P0
tdd_mode: strict
status: ready
crate: slideforge-syntax
target_module: slideforge-syntax
subsystems: [SS-01]
behavioral_contracts: [BC-3.02.002]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-007
  - STORY-008
blocks:
  - STORY-077
estimated_days: 2
---

# STORY-078: Parser — section block syntax (section <type>: ... with sub-blocks)

## Subsystem Anchor Justification

SS-01 (DSL Parser) owns this story because `section <type>:` block parsing is a
parse-stage concern. Per ARCH-INDEX, SS-01 (`slideforge-syntax`) is the single parsing
authority for all DSL syntax: slide blocks, @include, variants, control flow, and now
section blocks. No other subsystem may produce `SectionNode` entries in the AST.

This story delivers the parser production identified as missing in DIR-077-001. STORY-008
correctly reserved `"section"` as E-PAR-006 (standard reserved-but-unimplemented pattern)
until this story lifts that reservation and makes the keyword active.

## Dependency Anchor Justifications

- Depends on STORY-007 (Parser: @for, @if, {{ expr }}): the `template_value()` combinator
  and inline content parsing infrastructure — used for `detail:` and `report:` sub-block
  values — is established in STORY-007. Section sub-block content reuses this combinator
  directly (single source of truth; no second inline parser).
- Depends on STORY-008 (Parser: @include, variants, set, aliases): the `RESERVED_KEYWORDS`
  map, `classify_keyword`, `is_reserved_bare_keyword`, and `AliasRegistry` infrastructure
  are all defined in STORY-008. Un-reserving `"section"` requires modifying STORY-008's
  keyword registry. The `check_reserved_name_collision` and `warn_unrecognized_section_sub_block`
  stubs in `section.rs` originate in STORY-008's scaffolding — this story replaces
  those stubs with real implementations.
- Blocks STORY-077 (SectionBlock IR Extension): STORY-077's AC-002 requires the parser to
  already produce `SectionNode` entries from source. STORY-077 then upgrades specific
  sub-block fields from `FieldValue::Template` to `FieldValue::Inlines`; it cannot do
  this without the parser production this story delivers.

## Summary

The `"section"` keyword is currently classified as `RESERVED_KEYWORDS["section"]` →
E-PAR-006 in `keywords.rs`, with `SectionNode.fields` intentionally empty (a placeholder
per `ast.rs:303-314`). No parser production exists to parse `section <type>:` blocks
from source. This blocks STORY-077 (IR extension) and, transitively, STORY-041/042
(DOCX exporters).

Per architect directive DIR-077-001 (2026-06-01), the parsing work is a STORY-027
decomposition gap (not a STORY-008 defect). The fix is a new 5-point SS-01 story that
delivers the full `section_block_parser` combinator and un-reserves the keyword.

This story delivers:

1. **Keyword un-reservation**: Remove `"section"` from `RESERVED_KEYWORDS` as E-PAR-006.
   Re-classify as an active block-level keyword (same tier as `"slide"`).
2. **`section_block_parser` combinator** in new file `parser/section.rs`: parses the
   full grammar from DIR-077-001 §3 → `SectionNode { kind: Spanned<String>, fields: Vec<FieldNode> }`.
3. **Deck-level wiring**: add `DeckItem::Section(SectionNode)` variant; wire
   `section_block_parser` into `deck_parser` alongside the existing `DeckItem::Block` arm.
4. **Sub-block parsing**: recognized keys (`report:`, `detail:`) produce `FieldNode` entries
   with values parsed via the EXISTING `template_value()` combinator (initial representation:
   `FieldValue::Template(Vec<TemplateChunk>)` — STORY-077 upgrades these to `FieldValue::Inlines`).
5. **Unrecognized sub-block keys**: stored in `SectionNode.fields` as `FieldNode` entries
   (NOT dropped at parse time); a non-fatal `ParseSeverity::Warning` is emitted at parse
   time via the `validate()`/`emitter.emit()` idiom, routed through `push_with_severity(..,
   ParseSeverity::Warning)` into `ParseResult::warnings` (per DIR-077-001-A Ruling 2).
6. **`SECTION_REGISTER_KEYS` fix**: constant in `section.rs` changed from
   `&["notes", "report", "detail"]` to `&["report", "detail"]` per DIR-077-001 §5.
7. **Reserved-name collision check**: parse-time FATAL error via the same
   `validate()`/`emitter.emit()` idiom — if an unrecognized sub-block key matches a
   reserved register name used without the required `:` suffix (EC-006), emit a fatal
   `Rich::custom` error with corrective hint. Replace the `todo!()` stub in
   `check_reserved_name_collision` (BC-3.02.002 EC-006; DIR-077-001-A Ruling 2).
8. **Span propagation**: every `FieldNode.name` and `FieldNode.value` carries a
   `Span { file_id, start, end }` using the existing `to_span(ss, file_id)` helper.
9. **Error recovery**: `recover_with(skip_then_retry_until(...))` pattern — same as
   `slide_block_parser`; error accumulation (never fail-on-first per Q23, LOCKED).
10. **Snapshot and unit tests** for all ACs.

Initial parse-time representation note: Sub-block values are stored as
`FieldValue::Template(Vec<TemplateChunk>)` — the same representation as slide fields.
STORY-077 is responsible for upgrading `detail:` and `report:` sub-blocks to
`FieldValue::Inlines(Vec<InlineNode>)`. This is an intentional two-story decomposition per
DIR-077-001 §2 (Revised STORY-077 Scope).

## Behavioral Contracts

| BC | Title | Version | Covered ACs |
|----|-------|---------|-------------|
| BC-3.02.002 | Manually Authored Section Blocks (section methodology:) Appear in DOCX/PDF | v1.2 | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007, AC-008 |

> **DIR-077-001-A scope notes:** AC-003 traces to BC-3.02.002 precondition 2 (extensible SectionType registry) — TYPE validation against the registry is eval-stage (STORY-077), not parse-stage. AC-004 traces to BC-3.02.002 invariant 4 — the KEY warning is parse-time (STORY-078); eval does NOT re-emit it.

## Acceptance Criteria

### AC-001: `"section"` is no longer E-PAR-006 reserved-to-error
(traces to BC-3.02.002 precondition 1 — .sf file may contain section blocks; precondition 3 — top-level deck position)

After this story, `"section"` is removed from `RESERVED_KEYWORDS`. Writing
`section methodology:` (or any recognized section type) in a `.sf` file does NOT produce
E-PAR-006. `classify_keyword("section")` returns `None`; `is_reserved_bare_keyword("section")`
returns `false`. A parse of a minimal section block completes without any diagnostics:

```
section methodology:
  detail: "Some content."
```

Asserts: `parser.errors.is_empty()` and the resulting AST contains one `DeckItem::Section`.

### AC-002: `section <type>:` with recognized type and indented sub-blocks parses to `SectionNode`
(traces to BC-3.02.002 precondition 2 — recognized SectionType; postcondition 5 — interpolation resolved later; postcondition 8 — sub-block content stored with structural fidelity)

A `section <type>:` block with a recognized type (methodology, scope, approval, appendix,
glossary) and indented sub-block fields produces a `SectionNode` where:

- `kind.value == "<type>"` (the section type string)
- `kind.span` is non-empty (valid source span)
- `fields.len()` equals the number of sub-block entries in the source
- Each `FieldNode` has `name.value` matching the sub-block key and a non-empty `name.span`
- Each `FieldNode.value` is `FieldValue::Template(...)` (populated, not empty)

Canonical fixture:

```
section methodology:
  report: "We applied rigor."
  detail: "Extended methodology detail."
```

Asserts: `node.kind.value == "methodology"`, `node.fields.len() == 2`,
`node.fields[0].name.value == "report"`, `node.fields[1].name.value == "detail"`,
both field values are `FieldValue::Template`.

### AC-003: Parser stores section TYPE verbatim — no built-in-list rejection at parse time
(traces to BC-3.02.002 precondition 2 — SectionType plugin registry is open and extensible; BC-3.02.002 invariant 3 note — TYPE validation is eval-stage only per DIR-077-001-A Ruling 3)

The parser stores the identifier immediately following `"section"` verbatim in
`SectionNode.kind: Spanned<String>`. It performs NO built-in-list check and emits NO
error for any type name at parse time. `section foobar:` with a valid body MUST parse
successfully with zero errors and zero warnings.

Rationale (DIR-077-001-A Ruling 3): the valid section type set is open and
plugin-extensible. A plugin-registered type (e.g., `section timeline:`) submitted with
the `timeline` plugin loaded must NOT be falsely rejected at parse time. The eval stage
owns section TYPE validation against the full `SectionType` plugin registry (implemented
in STORY-077).

Asserts: given input
```
section foobar:
  detail: "Some content."
```
`parser.errors.is_empty() == true`, `parser.warnings.is_empty() == true`, the resulting
AST contains one `DeckItem::Section`, and `node.kind.value == "foobar"`.

### AC-004: Unrecognized sub-block KEY produces a parse-time warning AND is retained in `SectionNode.fields`
(traces to BC-3.02.002 invariant 4 — unrecognized sub-block key → non-fatal lint warning at parse time; BC-3.02.002 EC-005)

An unrecognized sub-block key (e.g., `foo:`) inside a section block produces:

(a) A `FieldNode` with `name.value == "foo"` retained in `SectionNode.fields` (the key
    is NOT dropped from the AST), AND
(b) A non-fatal `ParseSeverity::Warning` diagnostic in `ParseResult::warnings` emitted
    at parse time via the `validate()`/`emitter.emit(Rich::custom(span, msg))` pattern
    inside `section_block_parser`, routed through `push_with_severity(..,
    ParseSeverity::Warning)` — the same channel used by the missing-`slideforge_version`
    advisory at `parser/mod.rs:293-305`.

The warning message names the unrecognized key: `"W-PAR-NNN: Unrecognized section
sub-block key 'foo' — ignored"`. The parse SUCCEEDS (zero errors). The warning is
visible to the caller BEFORE eval runs.

This is NOT new infrastructure: every required piece (the `validate()` combinator,
`Emitter::emit`, `ParseSeverity::Warning`, `push_with_severity`, `ParseResult::warnings`)
already exists in the slideforge codebase. STORY-078 wires them into `parser/section.rs`.
Dir-077-001-A Ruling 2 cancels the earlier eval-deferral instruction.

A parse of:

```
section methodology:
  foo: "unrecognized"
  detail: "real content"
```

Asserts: `parser.errors.is_empty()` (parse succeeds), `result.warnings.len() == 1`,
the warning message contains `"foo"`, the warning span points to the `foo` token,
`node.fields.len() == 2`, and `node.fields[0].name.value == "foo"` is present
alongside `"detail"`.

### AC-005: `section <type>:` block inside a slide block is a parse error
(traces to BC-3.02.002 precondition 3 — section blocks must be top-level; BC-3.02.002 EC-002)

A `section` keyword appearing inside the body of a `slide` block (i.e., indented under
a slide's scope) produces a parse error: `"section blocks must be top-level — found inside
slide block"`. The block is not parsed as a `SectionNode`.

Asserts: `parser.errors` contains an error with correct message; no `SectionNode` is
produced for the mis-placed keyword.

### AC-006: `SECTION_REGISTER_KEYS` excludes `"notes"`
(traces to BC-3.02.002 EC-004 — section `report:` sub-block; EC-006 — reserved-name collision detection; DIR-077-001 §5 ruling)

The `SECTION_REGISTER_KEYS` constant in `section.rs` contains exactly `["report", "detail"]`.
It does NOT contain `"notes"`. `is_register_sub_block_key("notes")` returns `false`.
`is_register_sub_block_key("report")` and `is_register_sub_block_key("detail")` each
return `true`.

This reflects DIR-077-001 §5: the `notes` register is a presenter register for slide
contexts only; document sections have no slide canvas or speaker view.

Asserts: unit test verifying all three `is_register_sub_block_key` calls return the
expected booleans.

### AC-007: Snapshot test for a 2-section deck AST (regression baseline for STORY-077)
(traces to BC-3.02.002 precondition 1 — deck may contain multiple section blocks; postcondition 6 — source order preserved)

An `insta` snapshot test parses a 2-section deck:

```
section methodology:
  report: "Methodology content."

section scope:
  detail: "Scope detail."
  report: "Scope report."
```

The snapshot captures the full AST with both `SectionNode` entries in source order.
This test serves as a regression baseline for STORY-077: any parser change that alters
the AST representation for section blocks will break this snapshot and require explicit
review.

Asserts: snapshot matches; `ast.items` contains 2 `DeckItem::Section` entries in the
order `methodology` then `scope`.

### AC-008: Code quality invariants maintained
(traces to BC-3.02.002 precondition 1 — production parser must be reliable; NFR-021, NFR-022, NFR-024)

The delivered code meets all project-wide code quality invariants:
- `#![forbid(unsafe_code)]` — no unsafe in `slideforge-syntax`
- Zero `.unwrap()` / `.expect()` on `Result` or `Option` in non-test code — all
  fallible paths use `?` + typed error variants
- `clippy::pedantic` clean with no `#[allow(...)]` added without a documented justification
- `#![warn(missing_docs)]` — every new public item has a doc comment

Asserts: `cargo clippy -p slideforge-syntax -- -D warnings` exits 0;
`cargo nextest run -p slideforge-syntax --no-fail-fast` exits 0 with all tests passing
(including the two currently-`should_panic` stub tests, which must not panic after this
story removes the underlying `todo!()` implementations).

## Tasks

- [ ] Remove `"section" => ("E-PAR-006", "section block keyword")` from `RESERVED_KEYWORDS`
      in `crates/slideforge-syntax/src/keywords.rs`
- [ ] Verify `classify_keyword("section")` returns `None` and
      `is_reserved_bare_keyword("section")` returns `false` (add assertions to the
      existing `keywords.rs` unit tests)
- [ ] Create `crates/slideforge-syntax/src/parser/section.rs` with `section_block_parser`
      combinator implementing the grammar in DIR-077-001 §3
- [ ] Parse the section TYPE IDENT verbatim — store in `SectionNode.kind: Spanned<String>`
      with NO built-in-list check and NO parse-time error for unrecognized type names
      (DIR-077-001-A Ruling 3: TYPE validation belongs to eval stage, STORY-077)
- [ ] Parse sub-block fields using the existing `template_value()` combinator (no second
      inline parser); produce `FieldNode { name: Spanned<String>, value: Spanned<FieldValue> }`
      for each sub-block
- [ ] In `section_block_parser`, after parsing each sub-block key IDENT, add a `.validate()`
      closure that checks the key against `REGISTER_SUB_BLOCK_KEYS = ["report", "detail"]`:
      - If the key is NOT in `REGISTER_SUB_BLOCK_KEYS` AND does NOT match a reserved
        register name: emit a non-fatal `Rich::custom(span, "W-PAR-NNN: Unrecognized
        section sub-block key '<key>' — ignored")` via `emitter.emit(...)` (EC-005); the
        key is stored in the AST as a `FieldNode` (NOT dropped)
      - If the key matches a reserved register name used without the required `:` suffix
        (EC-006): emit a FATAL `Rich::custom` error with corrective hint
      - Route warnings via `push_with_severity(.., ParseSeverity::Warning)` into
        `ParseResult::warnings`, mirroring `parser/mod.rs:293-305`
- [ ] Replace the `todo!()` stub in `check_reserved_name_collision` in
      `crates/slideforge-syntax/src/section.rs` with the real reserved-name check
      (called from the `.validate()` closure above); return correct `bool`
- [ ] Remove or repurpose the `warn_unrecognized_section_sub_block` `todo!()` stub in
      `section.rs` — the warning is emitted inline in `section_block_parser`'s `.validate()`
      closure, not via a standalone post-parse function (DIR-077-001-A Ruling 2)
- [ ] Update `SECTION_REGISTER_KEYS` / `REGISTER_SUB_BLOCK_KEYS` in `section.rs` from
      `&["notes", "report", "detail"]` to `&["report", "detail"]`
- [ ] Add `DeckItem::Section(SectionNode)` variant to the `DeckItem` enum in
      `crates/slideforge-syntax/src/parser/deck.rs`
- [ ] Wire `section_block_parser(file_id).map(DeckItem::Section)` into `deck_parser`'s
      item alternatives (alongside existing `DeckItem::Block` arm)
- [ ] In the `validate` closure of `deck_parser`, push `BlockItem::Section(...)` for
      `DeckItem::Section` items into `deck.items`
- [ ] Apply span propagation via `to_span(ss, file_id)` on all `FieldNode.name` and
      `FieldNode.value` spans
- [ ] Apply `recover_with(skip_then_retry_until(...))` error recovery in
      `section_block_parser` — same pattern as `slide_block_parser`
- [ ] Add unit test `test_section_keyword_no_longer_reserved`: parse `section methodology:`
      with empty body → 0 errors, 1 `DeckItem::Section`
- [ ] Add unit test `test_section_recognized_type_with_sub_blocks`: canonical AC-002
      fixture → `fields.len() == 2`
- [ ] Add unit test `test_section_unknown_type_parsed_verbatim`: `section foobar:` with
      valid body → 0 errors, 0 warnings, `node.kind.value == "foobar"` (DIR-077-001-A
      Ruling 3: no fatal error for unknown TYPE at parse time)
- [ ] Add unit test `test_section_unrecognized_key_warning_at_parse_time`: `foo: bar`
      sub-block → parse succeeds (0 errors), `result.warnings.len() == 1`, warning
      message names `"foo"`, warning span points to `foo` token, `node.fields[0].name.value
      == "foo"` is present in AST (DIR-077-001-A Ruling 2)
- [ ] Add unit test `test_section_nested_in_slide_error`: section keyword inside slide
      body → parse error
- [ ] Add unit test `test_register_keys_exclude_notes`: `is_register_sub_block_key("notes")`
      returns `false`; `is_register_sub_block_key("report")` and `("detail")` return `true`
- [ ] Add `insta` snapshot test for 2-section deck AST (AC-007)
- [ ] Run `cargo nextest run -p slideforge-syntax --no-fail-fast` — all tests pass,
      including the two previously-`should_panic` stub tests

## Previous Story Intelligence

N/A — first story in the `section` parser sub-domain. Nearest predecessors:

**STORY-008** (Parser: @include, variants, set, aliases) established the keyword
infrastructure this story modifies. Key patterns to reuse:
- `RESERVED_KEYWORDS` map mutation pattern (add/remove entries)
- `classify_keyword` / `is_reserved_bare_keyword` helper contract
- `AliasRegistry` patterns (note: section TYPE is NOT validated against a parser-owned
  registry — see DIR-077-001-A Ruling 3; only sub-block KEYS use the compile-time
  `REGISTER_SUB_BLOCK_KEYS` constant)
- Error accumulation via the standard multi-error accumulator

**STORY-006** (Parser Core) established the `slide_block_parser` combinator that serves
as the structural template for `section_block_parser`. Reuse:
- `recover_with(skip_then_retry_until(...))` error recovery pattern
- `to_span(ss, file_id)` span propagation helper
- `DeckItem` enum extension pattern

**STORY-007** (Parser: @for, @if, {{ expr }}) established `template_value()` — the
combinator this story reuses for sub-block values. Do NOT introduce a second inline
combinator; call `template_value()` directly.

The two `todo!()` stubs that currently exist in `section.rs` (`warn_unrecognized_section_sub_block`
and `check_reserved_name_collision`) will cause the two `should_panic` tests to panic
until this story replaces them. After this story, those tests must not panic.

## Architecture Compliance Rules

1. **SS-01 is the single parsing authority (ARCH-INDEX Policy 1)**: `section_block_parser`
   lives exclusively in `crates/slideforge-syntax/`. No other crate produces `SectionNode`
   entries. The layout, eval, and exporter crates consume `SectionNode`; they do not parse.
2. **No second inline parser (DIR-077-001 §3.3)**: Sub-block content MUST be parsed via
   the existing `template_value()` combinator. Any deviation creates a second source of
   truth for inline parsing and is an architectural violation.
3. **Pure-core parser boundary (DIR-077-001-A Ruling 3)**: The parser must NOT validate
   section TYPE names against any list (built-in or plugin). The parser stores the type
   IDENT verbatim in `SectionNode.kind`. Type validation against the plugin-extensible
   `SectionType` registry is exclusively an eval-stage concern (STORY-077). This is the
   correct application of the pure-core boundary: the TYPE set is open-ended; only the
   sub-block KEY set (`REGISTER_SUB_BLOCK_KEYS`) is a fixed compile-time constant and
   may be checked at parse time.
4. **No circular Cargo dependencies**: `slideforge-syntax` depends on `slideforge-types`.
   This story adds no new Cargo edges.
5. **`slideforge-syntax` must NOT depend on**: `slideforge-eval`, `slideforge-layout`,
   `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, `slideforge-html`. Build
   fails if any of these appear in `slideforge-syntax/Cargo.toml`.
6. **Error recovery is non-negotiable (Q23, LOCKED)**: `section_block_parser` must use
   `recover_with(skip_then_retry_until(...))` to accumulate errors; it must never bail
   on the first malformed field. Every error must be accumulated before the parse result
   is returned.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `chumsky` | `=0.10.1` | Parser combinator for `section_block_parser` |
| `slideforge-types` | workspace | `SectionNode`, `FieldNode`, `FieldValue`, `DeckItem`, `Span`, `Spanned` types |
| `insta` | `=1.42.1` | Snapshot test for AC-007 |

No new external dependencies introduced. This story uses `chumsky` and `slideforge-types`
which are already present in `slideforge-syntax/Cargo.toml` from STORY-006/007/008.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-syntax/src/keywords.rs` | Modify | Remove `"section"` from `RESERVED_KEYWORDS`; update unit tests |
| `crates/slideforge-syntax/src/parser/section.rs` | Create | `section_block_parser` combinator — full implementation |
| `crates/slideforge-syntax/src/parser/deck.rs` | Modify | Add `DeckItem::Section(SectionNode)` variant; wire `section_block_parser`; update `validate` closure |
| `crates/slideforge-syntax/src/section.rs` | Modify | Set `SECTION_REGISTER_KEYS = &["report", "detail"]`; replace `warn_unrecognized_section_sub_block` `todo!()`; replace `check_reserved_name_collision` `todo!()` |
| `crates/slideforge-syntax/src/ast.rs` | Modify (doc only) | Add doc comment to `SectionNode.fields` documenting that it is now populated (no structural change) |

## Forbidden Dependencies

`slideforge-syntax` must NOT depend on:
- `slideforge-eval` — evaluator crate; would create a circular dependency
- `slideforge-layout` — layout crate; out of parser scope
- `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, `slideforge-html` — exporter crates
- `slideforge-preview` — effectful shell

Build fails if any of the above appear in `crates/slideforge-syntax/Cargo.toml`.

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,800 |
| BC-3.02.002 (single BC, all clauses) | ~1,500 |
| DIR-077-001 directive (grammar spec §3, checklist §8) | ~1,200 |
| `keywords.rs` context (for removal) | ~600 |
| `section.rs` existing stubs (for replacement) | ~800 |
| `ast.rs` SectionNode declaration | ~400 |
| `parser/deck.rs` context (for wiring) | ~800 |
| STORY-006/007/008 patterns (template_value, slide_block_parser) | ~1,500 |
| New `parser/section.rs` to write | ~2,000 |
| Test files to write | ~2,500 |
| **Total** | **~15,100** |

Agent context budget: 200k tokens. This story is ~7.6% of budget — well within limit.

## Test Strategy

- **Unit tests** (in `crates/slideforge-syntax/src/parser/section.rs` or adjacent `tests` module):
  - `test_section_keyword_no_longer_reserved`: minimal `section methodology:` → 0 errors
  - `test_section_recognized_type_with_sub_blocks`: AC-002 canonical fixture → fields populated
  - `test_section_unknown_type_parsed_verbatim`: `section foobar:` with valid body → 0 errors, 0 warnings, `kind == "foobar"` (DIR-077-001-A Ruling 3)
  - `test_section_unrecognized_key_warning_at_parse_time`: `foo: bar` → parse succeeds, `warnings.len() == 1`, warning names `"foo"`, span correct (DIR-077-001-A Ruling 2)
  - `test_section_nested_in_slide_error`: `section` inside slide body → parse error
  - `test_register_keys_exclude_notes`: `is_register_sub_block_key` correctness
  - `test_reserved_name_collision`: key matching reserved name without register syntax → fatal error
  - `test_section_spans_present`: all `FieldNode.name.span` and `FieldNode.value.span` are non-empty
  - `test_section_error_accumulation`: two malformed fields → two errors accumulated (not bail-on-first)
- **Snapshot test** (`insta`): 2-section deck AST — `section methodology:` + `section scope:` in order

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `section foobar:` — unrecognized type | Parser stores `kind = "foobar"` verbatim; 0 errors, 0 warnings at parse time. Eval stage validates the type against the SectionType plugin registry and emits the fatal error (STORY-077). |
| EC-002 | `section` keyword inside a slide block | Parse error: "section blocks must be top-level — found inside slide block"; no SectionNode produced |
| EC-003 | Section block with no sub-blocks (empty body) | `SectionNode.fields` is empty (`vec![]`); no error; section heading will still appear in DOCX (empty section behavior owned by STORY-027) |
| EC-004 | Indentation uses tabs | Lex error (existing rule, unchanged); consistent with all other indentation in the DSL |
| EC-005 | Sub-block key collides with reserved register name without register syntax | Fatal parse-time error (via `.validate()` in `section_block_parser`): "Key '<name>' is a reserved register name — use `<name>:` register syntax or choose a different key" per BC-3.02.002 EC-006 and DIR-077-001-A Ruling 2 |
| EC-006 | `section methodology:` appearing in an `@include`d file | Parser processes via the same file_id mechanism as slides; spans carry the included file's `file_id` |
| EC-007 | Two `section` blocks with the same type (e.g., two `section methodology:`) | Both parse successfully (no uniqueness constraint at parse time); duplicate type warning deferred to eval stage |
| EC-008 | `section` keyword alone on a line (no type identifier) | Parse error: expected IDENT after `section` keyword |
| EC-009 | `section` followed by recognized type but missing colon | Parse error: expected `:` after section type identifier |
| EC-010 | `is_register_sub_block_key("notes")` | Returns `false` — `notes` is NOT in `SECTION_REGISTER_KEYS` per DIR-077-001 §5 |

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-06-01 | story-writer | Initial creation per architect directive DIR-077-001 — resolves STORY-027 decomposition gap; section-block parser prerequisite for STORY-077 |
| 1.1 | 2026-06-01 | story-writer | Per DIR-077-001-A: (a) AC-003 REVISED — parser stores section TYPE verbatim with NO built-in-list check and NO fatal error for unknown types; TYPE validation moved to eval stage (STORY-077) per Ruling 3; (b) AC-004 REVISED — unrecognized sub-block KEY warning is parse-time via `validate()`/`emit` + `push_with_severity(.., ParseSeverity::Warning)` per Ruling 2 (eval-deferral instruction cancelled); reserved-name collision (EC-005/EC-006) also parse-time; (c) architecture compliance rule 3 updated to reflect verbatim TYPE storage; (d) task list, test names, edge case EC-001, and test strategy updated to match. |
