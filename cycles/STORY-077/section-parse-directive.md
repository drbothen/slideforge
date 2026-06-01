---
document_type: binding-directive
directive_id: DIR-077-001
issued_by: architect
issued_date: 2026-06-01
status: binding
targets: [story-writer, implementer, product-owner]
story: STORY-077
bc: [BC-3.02.002, BC-1.14.003]
supersedes: []
---

# Binding Directive DIR-077-001: Section-Block Parsing Gap — Ownership, Grammar, and Scope Resolution

## Status

BINDING. Resolves a prerequisite gap discovered during STORY-077 delivery. The keyword
`"section"` is currently RESERVED-TO-ERROR in the DSL parser (keywords.rs:74, E-PAR-006),
`SectionNode.fields` is intentionally empty (ast.rs:303-314 — "placeholder for STORY-008+"),
and no parser production builds a `SectionNode` from source. This directive establishes the
authoritative ownership ruling, the canonical grammar, all ancillary decisions, and the
revised delivery plan. All agents execute under this directive.

---

## 1. Ownership Ruling

### The Gap is a STORY-027 Implementation Shortfall, Not a STORY-008 Scope Gap

**STORY-008** (Parser: @include, variants, set rules, aliases) has no section-block
parsing in scope. Its behavioral contracts (BC-1.01.004, BC-1.06.x, BC-1.07.x,
BC-1.08.x, BC-1.09.x) are exclusively about `@include`, `variants:`, `set`, and
`alias`. Its tasks, file list, and ACs make zero mention of `section <type>:` blocks.
STORY-008's classification of `"section"` in RESERVED_KEYWORDS as E-PAR-006 is
CORRECT for STORY-008's scope: the keyword is recognized (not silently ignored) and
rejected with a targeted error message until implemented. This is the standard
reserved-but-unimplemented pattern used throughout the codebase for planned features.
STORY-008 is NOT the gap.

**STORY-027** (Layout: Document Section Generation) IS where the gap lives. STORY-027's
AC-004 ("Manual section blocks appear in LaidOutDeck") states: "`section <type>:` blocks
in the `Deck` IR (top-level blocks, not nested inside slides) are collected into
`GeneratedSection { source: SectionSource::Manual, ... }` entries." STORY-027's tasks
include "Implement manual section collection: iterate `deck.top_level_sections`." This
implies the parser already produces `SectionNode` entries in the AST — but it does not.
STORY-027's scope declaration says it covers `slideforge-layout` (crate) and SS-05
(Layout Engine). It neither delivers nor explicitly defers the SS-01 (parser) work
needed to parse the DSL syntax it consumes. The section-block parsing work is a
prerequisite that STORY-027 assumed was done but never assigned.

**Verdict:** The gap is a missing story — a prerequisite that falls in the gap between
STORY-008 (which correctly reserved the keyword) and STORY-027 (which assumes parseable
section blocks without specifying who would implement parsing). It is NOT a defect in
STORY-008 or STORY-027 individually; it is a story-decomposition gap in the wave
schedule.

### Why STORY-077 Cannot Absorb the Parsing Work

STORY-077 was scoped as an 8-point IR-extension story covering three coordinated changes
across three crates (slideforge-types IR change, slideforge-syntax parser change,
slideforge-eval routing change). The parser change assumed section blocks were already
parseable (as described in STORY-077's AC-002: "round-trip test parses `section
methodology:` and asserts..."). The real parser work — un-reserving the keyword,
implementing the sub-block grammar from scratch, wiring span-attributed error recovery —
is an additional 5 points of SS-01 work. Absorbing it into STORY-077 would push the
story to 13 points, violate the wave-4 batch-A parallelism, and require re-scoping
STORY-008's keyword reservation decision.

The correct path is a NEW STORY (STORY-078) that delivers section-block parsing and
becomes a prerequisite for STORY-077.

---

## 2. Recommended Delivery Path

### New Story: STORY-078 — Parser: section block parsing

- **Crate:** `slideforge-syntax`
- **Subsystem:** SS-01 (DSL Parser — single parsing authority per ARCH-INDEX)
- **Wave:** 4, Batch A (must complete before STORY-077 begins delivery)
- **Points:** 5
- **Priority:** P0
- **Dependencies:** STORY-007 (template_value parser, needed for inline content in sub-blocks), STORY-008 (keyword infrastructure, AliasRegistry patterns)
- **Blocks:** STORY-077
- **Estimated days:** 2

**What STORY-078 delivers** (see Section 3 for grammar):

1. Un-reserve `"section"` from `RESERVED_KEYWORDS` as E-PAR-006; re-classify it as a
   recognized deck-level block introducer (same pattern as `"slide"`).
2. Implement `section_block_parser` in `slideforge-syntax/src/parser/section.rs`
   (new file). The parser produces `SectionNode` with a non-empty `fields: Vec<FieldNode>`
   where each field is a recognized or unrecognized sub-block key.
3. Wire `SectionNode` production into `deck_parser` in `parser/deck.rs` (alongside the
   existing `DeckItem::Block` arm).
4. Implement non-fatal lint warning for unrecognized sub-block keys at parse time
   (replaces the `todo!()` stub in `section.rs:66-73`; BC-3.02.002 invariant 4).
5. Implement reserved-name collision check for sub-block keys (replaces `todo!()` stub
   in `section.rs:84-90`; BC-3.02.002 EC-006).
6. Emit `LayoutError::UnknownSectionType` for unrecognized section TYPE names at parse
   time (BC-3.02.002 invariant 3, EC-001).
7. Deliver snapshot tests for the parser's AST output and unit tests for all ACs.

### Revised STORY-077 Scope (unchanged behavioral scope, now implementable)

STORY-077 retains all its current ACs and points (8). The parser change entry in
STORY-077's task list ("Update the parser in `crates/slideforge-syntax/` to emit
`FieldValue::Inlines` for `detail:` and `report:` sub-blocks within `section <type>:`
declarations") now becomes: "Extend the STORY-078 parser to emit `FieldValue::Inlines`
for `detail:` and `report:` sub-block content (instead of `FieldValue::Template`) inside
`section <type>:` declarations." The STORY-078 parser will initially store sub-block
values as `FieldValue::Template` (same as slide fields); STORY-077 upgrades those
specific keys to `FieldValue::Inlines`.

STORY-077's `depends_on` list must add `STORY-078`.

### Wave Impact

STORY-077 currently blocks STORY-041 and STORY-042 (Wave 4, Batch B). Under this plan:

- STORY-078 is added to Wave 4, Batch A. It has no dependency on any other Batch A
  story; it can run in parallel with STORY-035/036/043/044/045/073/075/076.
- STORY-077 begins delivery after STORY-078 merges. If STORY-078 merges mid-Batch-A,
  STORY-077 can still complete before Batch B starts.
- The STORY-041/042 gate is unchanged — they remain blocked on STORY-077.
- Net wave-schedule impact: +2 days in Batch A (the STORY-078 2-day estimate). STORY-077
  itself does not extend beyond its 3-day estimate because the parsing prerequisite is
  now handled by STORY-078.

**story-writer must:** (a) create STORY-078 with the scope defined above and Section 3
grammar; (b) add `STORY-078` to `STORY-077.depends_on`; (c) add `STORY-077` to
`STORY-078.blocks`.

---

## 3. Canonical Grammar for section Block Parsing (Binding)

This grammar is derived from the 25 BINDING DSL decisions (q1-q25-decisions.md),
BC-3.02.002 v1.2, and the existing slide-block grammar. It does NOT invent any new
syntax. All decisions here are consistent with the project conventions.

### 3.1 Keyword Handling

Remove `"section"` from `RESERVED_KEYWORDS` in `keywords.rs`. Replace it with a
recognized block introducer. The keyword `"section"` is NOT a banned identifier — it is
a valid block-level keyword that introduces a section block, exactly as `"slide"` introduces
a slide block and `"variants"` introduces a variants block. The E-PAR-006 entry for
`"section"` must be deleted.

The classification in `classify_keyword("section")` must return `None` after this change
(section is not an error; it's an active keyword). `is_reserved_bare_keyword("section")`
must return `false` after this change.

### 3.2 Top-Level Grammar Production

```text
section_block ::= "section" IDENT ":" NEWLINE INDENT section_body DEDENT
section_body  ::= section_sub_block*
section_sub_block ::= IDENT value NEWLINE         # single-line field (scalar value)
                    | IDENT ":" NEWLINE INDENT inline_content_block DEDENT  # multi-line register key
inline_content_block ::= template_line+
template_line ::= (STRING_CONTENT | INTERP_EXPR | INLINE_FMT)* NEWLINE
```

Constraints:
- Indentation uses spaces only; tabs produce a lex error (existing rule, unchanged).
- Indentation level is consistent within a block (existing rule, unchanged).
- The section type IDENT (immediately after `"section"`) is validated against the known
  section type registry (methodology, scope, approval, appendix, glossary, plus any
  registered via SectionType plugin trait). Unrecognized type → E-PAR-007-class fatal
  error (BC-3.02.002 invariant 3, EC-001): `"Unknown section type '<name>'. Known types:
  [methodology, scope, approval, appendix, glossary]"`.

### 3.3 Sub-block Parsing Rules

There are two classes of sub-blocks:

**Register sub-blocks** (`detail:`, `report:`) — multi-line inline content blocks.
Their values are parsed as inline node sequences using the EXISTING slide-level inline
parser (the same `template_value()` combinator used for slide fields). This is the
single source of truth for inline parsing; no second inline parser is introduced.
Initial parse-time representation: `FieldValue::Template(Vec<TemplateChunk>)`.
STORY-077 then upgrades these to `FieldValue::Inlines(Vec<InlineNode>)` during its
parser extension work.

**Unrecognized sub-blocks** — any key that is not in `REGISTER_SUB_BLOCK_KEYS`
(`["notes", "report", "detail"]`) produces a non-fatal lint warning at parse time:
`"Unrecognized section sub-block key '<key>' — ignored"` (BC-3.02.002 invariant 4,
EC-005). Build continues; the key is silently dropped from the AST.

**Reserved-name collision** — if an unrecognized sub-block key coincidentally matches
a reserved register name (`notes`, `report`, `detail`) but is used in a syntactically
ambiguous way (e.g., without the trailing `:` that signals a register sub-block), the
collision check in `check_reserved_name_collision` fires a FATAL parse error:
`"Key '<name>' is a reserved register name — use '<name>:' register syntax or choose
a different key."` (BC-3.02.002 EC-006).

### 3.4 AST Representation

`SectionNode.fields: Vec<FieldNode>` is used as-is (already declared in ast.rs:309-314).
Each sub-block becomes one `FieldNode { name: Spanned<String>, value: Spanned<FieldValue> }`.
`SectionNode.kind: Spanned<String>` holds the section type name.

No structural AST changes are needed. The `fields` field, declared empty as a placeholder,
now receives real values. This is backward-compatible.

### 3.5 Span Propagation

Every `FieldNode.name` and `FieldNode.value` carries a `Span { file_id, start, end }`.
The `file_id` is the file in which the section block is authored (including included
files via `@include`). This is identical to the span model for slide fields and must
use the same `to_span(ss, file_id)` helper.

### 3.6 Error Recovery

Error recovery follows the existing pattern from `slide_block_parser` and `block_item`:
`recover_with(skip_then_retry_until(...))` — skip the bad token(s) and retry at the next
newline. Error accumulation (never fail-on-first) per Q23 (LOCKED).

### 3.7 Integration into deck_parser

In `deck_parser`, add a `section` arm alongside the existing `block` arm:

```rust
let section = section_block_parser(file_id).map(DeckItem::Section);
```

Add `DeckItem::Section(SectionNode)` to the `DeckItem` enum. In the `validate` closure,
push `SectionNode` items into `deck.items` as `BlockItem::Section(...)`. The existing
`expand_block_item` already handles `BlockItem::Section(_) => item` (returns unchanged) —
no change needed there.

---

## 4. Warning Stage Ruling: Parse Time vs. Eval Time

BC-3.02.002 invariant 4 says the unrecognized-sub-block-key warning is emitted "at parse
time." The current implementation emits it at EVAL (eval.rs, `eval_section_nodes` function,
lines approx. 430-450: the `if !SECTION_REGISTER_KEYS.contains(...)` branch pushes an
`EvalDiagnostic::UnrecognizedSectionSubBlockKey` warning). This is architecturally
defensible — the eval stage has full context about the section type and the known registry.
However, BC-3.02.002 invariant 4 text explicitly says "parse time."

**Ruling: retain eval-stage emission for this project. Authorize a BC amendment.**

Rationale: The current eval-stage implementation is correct behavior (warning is
non-fatal, named correctly) and architecturally coherent (the parser stores unrecognized
keys in the AST; the eval stage decides their fate). Moving the check to the parser
requires the parser to know the section type registry — a coupling that makes the
parser depend on plugin registration, violating the pure-core boundary. The eval stage
already has this coupling for other type-validation purposes.

**Product Owner must amend BC-3.02.002 invariant 4** as follows:

Replace:
> "An unrecognized sub-block key inside a RECOGNIZED `section <type>:` block ... is a
> NON-FATAL lint warning naming the key. The warning is emitted at parse time."

With:
> "An unrecognized sub-block key inside a RECOGNIZED `section <type>:` block ... is a
> NON-FATAL lint warning naming the key. The warning is emitted at the Evaluate stage
> (eval_section_nodes). The parser stores unrecognized keys in `SectionNode.fields` for
> the eval stage to dispatch; it does not silently discard them."

Version bump: BC-3.02.002 must be bumped from `"1.2"` to `"1.3"` with a `modified`
entry citing this directive (DIR-077-001, 2026-06-01) and reason: "Corrected warning
stage for unrecognized sub-block keys from parse time to eval stage per architect ruling
(maintains pure-core parser boundary)."

Per the SPEC-wins rule, the human must authorize this spec amendment before the Product
Owner applies it. This directive documents the reasoning; human sign-off is the gate.

---

## 5. `notes:` on a Section — Canonical Ruling

Three-way inconsistency identified:

- `SECTION_REGISTER_KEYS` in `section.rs:37` includes `"notes"`.
- `extract_section_register_content` in `register_routing.rs:269-294` explicitly excludes
  `"notes"` (processes only `["report", "detail"]`, with comment: "Notes is not a valid
  section-level register sub-block; sections carry document-oriented content only").
- `E-EVL-010` help text (not yet verified) reportedly excludes `"notes"`.

**Ruling: `notes:` is NOT a valid register sub-block on a section block. Remove it from
`SECTION_REGISTER_KEYS`.**

**Justification grounded in the BINDING Q1 decision (q1-decision-final.md):**

The `notes` register is the PRESENTER register — "what the speaker says." Sections
(`section methodology:`, `section scope:`, etc.) are document-mode constructs that appear
exclusively in DOCX/PDF output. They have no PPTX rendering path and no slide canvas.
"Presenter notes on a document section" has no semantic meaning — there is no slide to
present, no speaker view to display them in. The `report` and `detail` registers are the
document-mode registers: `report` (reader narrative), `detail` (extended analysis). These
are the correct registers for section content.

**Implementation consequences:**

1. `SECTION_REGISTER_KEYS` in `section.rs` must be changed from `&["notes", "report",
   "detail"]` to `&["report", "detail"]`.

2. `REGISTER_SUB_BLOCK_KEYS` in `section.rs` (public constant, used by the warn logic)
   must match.

3. After this change, `notes:` inside a section block is an UNRECOGNIZED sub-block key.
   It will reach the eval stage's unrecognized-key branch and emit a non-fatal warning:
   `"Unrecognized section sub-block key 'notes' — ignored"`. This is the correct behavior:
   the user gets a helpful diagnostic that `notes:` is not meaningful on a section.

4. `extract_section_register_content` requires no change — it already excludes `notes`.

5. `E-EVL-010` (if it exists with the wrong help text) must be updated to match: `notes`
   is not listed as a valid section register key.

**BC-1.14.001** (notes register routing) does NOT need amendment. BC-1.14.001 governs
notes on SLIDES. Sections are out of scope for BC-1.14.001 and this ruling does not
affect it.

**No BC amendment required for this ruling.** It resolves an inconsistency within the
existing invariant text: BC-3.02.002 EC-004/EC-006 already implies `notes` is not a
recognized register sub-block (only `report` and `detail` appear in EC-004's description).
This is a code-correction, not a spec change.

---

## 6. Points and Wave Schedule Impact

| Item | Action | Points | Wave | Batch |
|------|--------|--------|------|-------|
| STORY-078 (new) | Create — section block parsing | 5 | 4 | A |
| STORY-077 (existing) | Add `STORY-078` to `depends_on`; task list adjusted | 8 (unchanged) | 4 | A (after 078 merges) |
| STORY-041, STORY-042 | No change — still blocked on STORY-077 | — | 4 | B |

Net wave-4 point increase: +5 (STORY-078). Net wave-4 calendar impact: +2 days in Batch
A. STORY-077 and STORY-078 are independent enough to overlap if parallelism is available.

---

## 7. STORY-078 Story Definition (for story-writer)

story-writer must create STORY-078 with at minimum the following structure:

```
story_id: STORY-078
title: "Parser: section block syntax (section <type>: ... with sub-blocks)"
epic: EPIC-02  # (same as STORY-008 — Parser epic)
wave: 4
points: 5
priority: P0
tdd_mode: strict
status: ready
crate: slideforge-syntax
target_module: slideforge-syntax
subsystems: [SS-01]
behavioral_contracts: [BC-3.02.002]
depends_on: [STORY-007, STORY-008]
blocks: [STORY-077]
estimated_days: 2
```

Mandatory ACs for STORY-078:

- **AC-001**: `"section"` is no longer E-PAR-006 reserved-to-error. Writing `section
  methodology:` in a `.sf` file does not error.
- **AC-002**: `section <type>:` with recognized type and indented sub-blocks parses to a
  `SectionNode` with `kind = <type>` and `fields` populated with `FieldNode` entries.
- **AC-003**: Unrecognized section TYPE produces a fatal parse error (E-PAR-007-class)
  naming the unknown type and listing known types.
- **AC-004**: Unrecognized sub-block KEY (e.g., `foo:`) inside a recognized section
  produces a `FieldNode` in the AST (not dropped). Eval stage will emit the warning.
- **AC-005**: `section <type>:` block appearing inside a slide block produces a parse
  error: "section blocks must be top-level" (BC-3.02.002 EC-002).
- **AC-006**: `SECTION_REGISTER_KEYS` constant excludes `"notes"` per Section 5 of this
  directive (contains only `["report", "detail"]`).
- **AC-007**: Snapshot test for a 2-section deck AST (regression baseline for STORY-077).
- **AC-008**: `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, `clippy::pedantic`
  clean, `#![warn(missing_docs)]` on all public items.

Mandatory file changes for STORY-078:

| File | Action |
|------|--------|
| `crates/slideforge-syntax/src/keywords.rs` | Remove `"section"` from `RESERVED_KEYWORDS` |
| `crates/slideforge-syntax/src/parser/section.rs` | NEW — `section_block_parser` combinator |
| `crates/slideforge-syntax/src/parser/deck.rs` | Add `DeckItem::Section`; wire `section_block_parser` |
| `crates/slideforge-syntax/src/section.rs` | Replace `REGISTER_SUB_BLOCK_KEYS` with `["report", "detail"]`; implement `warn_unrecognized_section_sub_block` and `check_reserved_name_collision` stubs (remove `todo!()`) |
| `crates/slideforge-syntax/src/ast.rs` | No structural change needed; document that `SectionNode.fields` is now populated |

---

## 8. Implementer Checklist (STORY-078)

- [ ] Remove `"section" => ("E-PAR-006", "section block keyword")` from `RESERVED_KEYWORDS` in `keywords.rs`
- [ ] Verify `classify_keyword("section")` returns `None` and `is_reserved_bare_keyword("section")` returns `false`
- [ ] Create `crates/slideforge-syntax/src/parser/section.rs` with `section_block_parser` function implementing the grammar in Section 3
- [ ] Add `DeckItem::Section(SectionNode)` variant to the `DeckItem` enum in `parser/deck.rs`
- [ ] Wire `section_block_parser(file_id).map(DeckItem::Section)` into `deck_parser`'s item alternatives
- [ ] In the `validate` closure, push `BlockItem::Section(...)` for `DeckItem::Section` items
- [ ] Update `SECTION_REGISTER_KEYS` / `REGISTER_SUB_BLOCK_KEYS` in `section.rs` to `["report", "detail"]` (remove `"notes"`)
- [ ] Implement `warn_unrecognized_section_sub_block` in `section.rs` — remove `todo!()`, emit `ParseSeverity::Warning`
- [ ] Implement `check_reserved_name_collision` in `section.rs` — remove `todo!()`, return correct bool
- [ ] Add `test_section_keyword_no_longer_reserved` — parses `section methodology:` with 0 errors
- [ ] Add `test_section_unknown_type_fatal` — parses `section foobar:` and asserts fatal error
- [ ] Add `test_section_sub_blocks_parsed` — parses section with `report:` and `detail:` sub-blocks; asserts `fields.len() == 2`
- [ ] Add snapshot test for 2-section deck AST
- [ ] Confirm `is_register_sub_block_key("notes")` returns `false` after the `SECTION_REGISTER_KEYS` fix
- [ ] Run `cargo nextest run -p slideforge-syntax --no-fail-fast` — all tests pass including the two currently-`should_panic` stub tests (which must now not panic)

## 9. Product Owner Checklist

- [ ] Amend BC-3.02.002 invariant 4 per Section 4 (change "parse time" to "Evaluate stage") — requires human sign-off per SPEC-wins rule
- [ ] Bump BC-3.02.002 from version `"1.2"` to `"1.3"` with `modified` entry for DIR-077-001
- [ ] Verify BC-3.02.002 EC-005 and EC-006 text remain consistent with revised invariant 4
- [ ] No change required to BC-1.14.001, BC-1.14.003 — this ruling does not alter their scope

## 10. Architect Self-Audit (DIR-077-001)

- [x] Did I rationalize with "for now" or "good enough"? NO — the ruling mandates a new
  story with full production-grade parsing, not a partial shortcut.
- [x] Did I add a tech-debt-register entry? NO — directed a new story and fixed the
  `SECTION_REGISTER_KEYS` inconsistency as required corrections.
- [x] Did I leave a "pending architect review" TODO? NO — all five questions are decided.
- [x] Did I find a bug and surface it as an advisory? NO — fixed in scope (STORY-078
  creation directive, notes exclusion, BC amendment authorization).
- [x] Did I default to the cheapest mechanism? NO — the correct path is a new story
  that delivers the full parser, not expanding STORY-077's scope beyond its budget.

---

## Addendum DIR-077-001-A — Parse-Stage Correction (2026-06-01)

```
addendum_id: DIR-077-001-A
issued_by: architect
issued_date: 2026-06-01
trigger: chumsky-parse-warning-research.md (produced 2026-06-01)
status: binding
supersedes_clauses:
  - §4 (full section — WITHDRAWN and replaced below)
  - §7 AC-003 (partial — see Ruling 3 below)
  - §7 AC-004 (full — WITHDRAWN and replaced below)
  - §8 checklist items that reference eval-stage deferral (see below)
  - §9 Product Owner Checklist items 1-3 (WITHDRAWN — no BC amendment needed)
unchanged:
  - §1 Ownership Ruling (unchanged)
  - §2 Delivery Path / STORY-077/078 split and point estimates (unchanged)
  - §3.1–§3.7 Grammar (unchanged, except §3.3 sub-block key list per §5 — already corrected)
  - §5 notes: exclusion ruling (unchanged)
  - §6 Points and Wave Schedule Impact (unchanged)
targets: [story-writer, implementer, product-owner]
```

### Context

After DIR-077-001 was issued, the research agent investigated whether chumsky 0.10.1
supports non-fatal parse-time diagnostics (triggered by §4's proposed BC amendment).
The research produced `chumsky-parse-warning-research.md` (2026-06-01) which is
authoritative on chumsky capability and on the slideforge codebase's existing plumbing.

Its key verified findings are:

1. chumsky 0.10.1 (pinned `=0.10.1`) supports non-fatal diagnostics via
   `validate()` + `Emitter::emit(Rich::custom(span, msg))` — parsing continues and
   can return `Ok(output)`. This is documented as "non-terminal errors" in the
   official chumsky docs.

2. slideforge ALREADY uses this exact pattern at parse time for E-PAR-007 (unknown
   slide type in `set` rules) and E-PAR-008 (var-name collision), both in
   `parser/deck.rs`. The pattern is established codebase idiom, not hypothetical.

3. slideforge ALREADY has a complete end-to-end warning channel: `ParseSeverity::Warning`
   (`error.rs`), `DiagnosticSink::push_with_severity` (`sink.rs`), `ParseResult::warnings`
   (`parser/mod.rs`), and a live precedent (missing-`slideforge_version` advisory) that
   routes a parse-time non-fatal diagnostic into `ParseResult::warnings` while the parse
   succeeds.

4. The `REGISTER_SUB_BLOCK_KEYS = ["report", "detail"]` set is a compile-time constant —
   not plugin-registered. Checking a parsed identifier against a fixed compile-time set
   introduces **zero** registry coupling and **zero** pure-core violation.

5. The architect's §4 rationale ("Moving the check to the parser requires the parser to
   know the section type registry — a coupling that makes the parser depend on plugin
   registration, violating the pure-core boundary") is correct for the section-**TYPE**
   check (invariant 3) and **false** for the sub-block-**KEY** check (invariant 4). The
   two checks were conflated in DIR-077-001 §4. The research is authoritative on this
   distinction.

These findings are accepted. DIR-077-001 §4's ruling rested on a factual conflation and
is corrected below.

---

### Ruling 1 — §4 Withdrawn: No BC Amendment Required

**DIR-077-001 §4 is WITHDRAWN in its entirety.**

BC-3.02.002 invariant 4 as written ("warning emitted at parse time") is **correct**,
**achievable**, and **architecturally sound** for the sub-block KEY check. No amendment
to BC-3.02.002 is authorized. BC-3.02.002 remains at version `"1.2"` (no `"1.3"` bump
for this reason). The Product Owner must NOT apply the §4 BC amendment — it is cancelled.

**The §9 Product Owner Checklist items 1, 2, and 3 are WITHDRAWN.** No human sign-off
on a BC amendment is required. The product owner has no action from this directive.

The earlier self-audit entry in §10 ("Did I find a bug and surface it as an advisory?
NO — fixed in scope (STORY-078 creation directive, notes exclusion, BC amendment
authorization)") remains accurate in its outcome — the gap is fixed in scope — but
the BC amendment component of that fix is hereby cancelled. The correct fix is the
revised STORY-078 ACs in Ruling 2 below.

---

### Ruling 2 — Ownership Reassignment: Sub-Block KEY Warning Belongs at Parse Time (STORY-078)

**STORY-078's parser OWNS the sub-block KEY warning (invariant 4 / EC-005) and the
reserved-name collision error (invariant 4 / EC-006). Both are parse-time. Neither
defers to eval.**

The implementation pattern is Pattern A from the research document: inside
`section_block_parser` in `parser/section.rs`, parse each sub-block key as an IDENT,
then in a `.validate()` closure:

- If the key is NOT in `REGISTER_SUB_BLOCK_KEYS = ["report", "detail"]`:
  - If it matches a reserved register name used without the required `:` suffix (EC-006):
    emit a **FATAL** `Rich::custom(span, "E-PAR-NNN: Key '<name>' is a reserved register
    name — use '<name>:' register syntax or choose a different key.")`. This is a hard
    parse error; parsing of the sub-block continues at the next key (error recovery).
  - Otherwise (EC-005): emit a **non-fatal warning** `Rich::custom(span,
    "W-PAR-NNN: Unrecognized section sub-block key '<key>' — ignored")`. The key is
    stored in the AST as a `FieldNode` (not silently dropped). The parse continues and
    succeeds.
- At the `Rich → SyntaxError` conversion boundary (`parser/mod.rs`), the warning-prefix
  diagnostic (`W-PAR-` coded or otherwise distinguished) is routed into
  `ParseResult::warnings` (not `errors`), using the same `push_with_severity(..,
  ParseSeverity::Warning)` path as the missing-version advisory at `parser/mod.rs:293-305`.

**This is NOT new infrastructure.** Every required piece already exists in the codebase.
STORY-078 wires existing plumbing into `parser/section.rs`; it does not build new
warning machinery.

**Consequence for STORY-077 AC-004 `"defer to eval"` instruction:** The story-writer
must NOT carry forward the eval-deferral instruction from DIR-077-001 §4 into
STORY-078's AC-004. See Ruling 2's corrected AC-004 below. Eval retains no ownership
of the sub-block KEY warning.

**Eval (`eval_section_nodes`) retains ownership of:** section TYPE validation (invariant 3)
against the full plugin-populated `SectionType` registry. See Ruling 3.

---

### Ruling 3 — TYPE Check Placement: Eval is the Authority; Parser Parses Verbatim

**DIR-077-001 §3.2 and §7 AC-003 are partially corrected.**

The original §3.2 mandate ("Unrecognized type → E-PAR-007-class fatal error... against
the known section type registry [methodology, scope, approval, appendix, glossary, plus
any registered via SectionType plugin trait]") is internally inconsistent: it instructs
the parser to validate against a fixed built-in list, while the valid TYPE set is
open and plugin-extensible. A plugin-registered section type (e.g., `section timeline:`)
submitted to a build with the `timeline` plugin loaded would be falsely rejected at parse
time under the original §3.2. This contradicts BC-3.02.002 precondition 2 and the
architect's own pure-core-boundary rationale (which is correct for the TYPE check).

**Corrected ruling for section TYPE validation:**

The parser's job is **syntactic only** for the type name. It must:
1. Parse the IDENT immediately following `"section"` as the type name verbatim, storing
   it in `SectionNode.kind: Spanned<String>`.
2. Perform NO built-in-list rejection of the type name at parse time.
3. Emit NO parse-time error for an unrecognized section type.

The eval stage (`eval_section_nodes`, or its successor in STORY-077) is the authority for
type validation. Eval has access to the full `SectionType` plugin registry (including
built-ins AND plugin-registered types). When eval encounters a `SectionNode.kind` value
that does not match any registered type, it emits a fatal eval-stage error (error code to
be assigned by the error taxonomy; semantically equivalent to `LayoutError::UnknownSectionType`
cited in the original §3.2 — the error message remains "Unknown section type '<name>'.
Known types: [...]").

**BC-3.02.002 invariant 3** ("An unrecognized section `<type>` → error") is correct in
its behavioral requirement. This ruling clarifies that the **stage** for that error is
eval, not parse. This is a stage clarification, not a semantic change to the invariant.
No BC amendment is required — the invariant does not specify the stage. The story-writer
must NOT add a parse-time type-rejection step to STORY-078.

**Consequence for §6 item "Emit `LayoutError::UnknownSectionType` for unrecognized section
TYPE names at parse time (BC-3.02.002 invariant 3, EC-001)":** This task is REMOVED from
STORY-078's scope. It belongs to the eval story that owns `eval_section_nodes` (either
STORY-077 or a subsequent story that has eval scope). STORY-078 does not emit any error
for unrecognized section types.

**Corrected STORY-078 Mandatory ACs** (supersede the §7 text for AC-003 and AC-004):

- **AC-003 (CORRECTED):** `section <type>:` with ANY identifier as the type name parses
  to a `SectionNode` with `kind = <type>` verbatim. The parser does NOT reject unknown
  type names. `section foobar:` with a valid body parses successfully (0 errors); eval
  is the stage that validates the type against the registry. The §7 AC-003 text
  ("Unrecognized section TYPE produces a fatal parse error") is **WITHDRAWN**.

- **AC-004 (CORRECTED):** Unrecognized sub-block KEY (e.g., `foo:`) inside a recognized
  section produces (a) a `FieldNode` in the AST for that key, AND (b) a non-fatal
  `ParseSeverity::Warning` diagnostic in `ParseResult::warnings` — emitted at parse time
  via the `validate()`/`emit` pattern, routed through `push_with_severity(..,
  ParseSeverity::Warning)`. The warning is visible to the caller before eval runs. The
  §7 AC-004 text ("Eval stage will emit the warning") is **WITHDRAWN**.

**Corrected §8 Implementer Checklist items** (supersede the corresponding bullets):

- [x] ~~Add `test_section_unknown_type_fatal` — parses `section foobar:` and asserts
  fatal error~~ **WITHDRAWN.** Replace with: `test_section_unknown_type_parsed_verbatim`
  — parses `section foobar:` with a valid body and asserts `SectionNode.kind == "foobar"`,
  zero errors, zero warnings.

- [x] ~~Implement `warn_unrecognized_section_sub_block` in `section.rs` — remove
  `todo!()`, emit `ParseSeverity::Warning`~~ **REVISED.** Implement the warning via the
  `.validate()`/`emitter.emit()` idiom inside `section_block_parser` (not as a standalone
  function called post-parse). The `warn_unrecognized_section_sub_block` stub (if it
  exists) is replaced by the inline `validate` closure. Route the emitted diagnostic
  through the existing warning channel (`push_with_severity(.., ParseSeverity::Warning)`)
  into `ParseResult::warnings`, mirroring `parser/mod.rs:293-305`.

- [x] Add `test_section_unrecognized_key_warning_at_parse_time` — parses a section block
  with key `foo: bar` and asserts: (1) parse succeeds (no errors), (2)
  `result.warnings.len() == 1`, (3) the warning message names the key `"foo"`, (4) the
  warning's span points to the `foo` token.

---

### Ruling 4 — Unchanged Elements Confirmed

The following elements of DIR-077-001 are **confirmed unchanged** by this addendum:

- **Grammar (§3.1–§3.6):** All grammar productions unchanged. The section sub-block
  parser parses the type name verbatim (now explicit per Ruling 3) and sub-block keys
  as IDENTs. `template_value()` reuse for inline content parsing is unchanged. Span
  propagation, error recovery, indentation rules: unchanged.

- **`REGISTER_SUB_BLOCK_KEYS = ["report", "detail"]`:** Confirmed. `"notes"` excluded
  per §5 (unchanged). This is the fixed compile-time constant against which the
  parse-time KEY warning fires.

- **STORY-077 / STORY-078 split:** Confirmed. Points unchanged (STORY-077: 8,
  STORY-078: 5). Wave 4, Batch A for STORY-078; STORY-077 after STORY-078 merges.
  The split is correct and is not affected by the stage reassignments above.

- **§5 `notes:` exclusion ruling:** Confirmed unchanged. `notes:` on a section block
  is an unrecognized key; it fires the EC-005 parse-time warning (now at parse time per
  Ruling 2, not eval).

- **§1 Ownership Ruling:** Confirmed unchanged.

- **§6 Wave/Points table:** Confirmed unchanged.

---

### Addendum Self-Audit (DIR-077-001-A)

- [x] Did I rationalize with "for now"? NO — the correction tightens ownership; no
  work is deferred.
- [x] Did I add a tech-debt entry? NO.
- [x] Did I leave a "pending architect review" TODO? NO — all four rulings are
  decisive.
- [x] Did I find a conflict and surface it as an advisory? NO — ruled on all four
  required points.
- [x] Did I default to the cheapest mechanism? NO — Pattern A is used because it is
  the architecturally correct idiomatic path, not because it is cheapest. Pattern A
  happens to also be minimal because the slideforge plumbing already exists.
