---
document_type: architecture-directive
story_id: STORY-035
issued_by: vsdd-factory:architect
status: binding
date: 2026-05-31
findings_resolved: [F-001, F-002]
---

# Architecture Directive: STORY-035 Register Routing

This directive resolves two adversary findings (F-001 HIGH, F-002 HIGH) on
STORY-035. It is binding on the implementer, product-owner, and any
subsequent adversary pass. Source-of-truth precedence (CLAUDE.md rule 7):
spec wins on code-vs-spec conflicts.

---

## F-001: Canonical Architecture Decision — Option D

### Decision

**Option D is canonical for this codebase. The `slideforge-types` method
`Slide::extract_register_content()` is the duplicate that must be deleted.
The `slideforge-eval::register_routing::extract_register_content()` function
is the single source of truth and must be integrated into the `eval_deck`
pipeline.**

### Reasoning

The adversary offered Option A (layout adds a dep on slideforge-eval) and
Option D (register content added to semantic `Slide` IR; eval populates it;
layout copies). Option A is architecturally inferior here for one concrete
reason: the root `slideforge` crate sequences the pipeline as
`eval_deck → layout::run`. The eval stage produces a `Deck`; the layout
stage consumes that `Deck`. If layout calls back into eval, it inverts the
data flow direction established by the pipeline, even if no circular Cargo
dependency exists. It is a semantic cycle even when the Cargo DAG is
acyclic. Option D respects the pipeline direction.

However, pure Option D (add `register_content: Vec<RegisteredContent>` to
the semantic `Slide` IR in `slideforge-types`) introduces a complication:
`Slide` is a parse-time type. Populating `register_content` at parse time is
wrong — register content requires evaluation (interpolation resolution,
`@for` expansion, `@if` pruning). The cleanest resolution is a
**variant of Option D** that does not mutate `Slide` itself but instead
keeps `register_content` as a field of `LaidOutSlide` (where it already
exists) and has the `eval_deck` pass compute it as a parallel output
alongside the semantic `Deck` — or more precisely, has `eval_deck` or a
post-eval pass produce a `RegisterMap` that the layout engine receives as
input and copies into `LaidOutSlide.register_content` without re-deriving
any routing logic.

The simplest correct implementation given what already exists:

The `extract_register_content` function in `slideforge-eval::register_routing`
is already correct. The problem is that it is never called by `eval_deck`;
instead `Slide::extract_register_content()` (the duplicate in
`slideforge-types`) is called by `slideforge-layout::layout.rs:295`.

The canonical fix is:

1. Call `slideforge_eval::extract_register_content(&slide)` inside
   `slideforge-eval::eval.rs::eval_deck` (or in a post-eval coordination
   layer) for each evaluated slide, and store the result on the `Slide` IR
   or return it as a parallel `Vec<Vec<RegisteredContent>>` alongside `Deck`.

   **Concrete path of least disruption**: add a `register_content:
   Vec<RegisteredContent>` field to `slideforge_types::Slide`. This is
   semantically a post-evaluation annotation, not a parse-time field. Set it
   to `vec![]` in the parser; `eval_deck` populates it by calling
   `extract_register_content(&slide)` after resolving each slide's fields.
   Layout reads `slide.register_content` and copies it into
   `LaidOutSlide.register_content` verbatim — zero routing logic in layout.

2. Delete `Slide::extract_register_content()` from
   `crates/slideforge-types/src/slide.rs` (lines 123–185 in the current
   worktree file, including the private helper `field_value_to_register_inlines`).

3. Delete `fn field_value_to_register_inlines` from
   `crates/slideforge-types/src/slide.rs` (lines 25–46).

4. In `crates/slideforge-layout/src/layout.rs` line 295, replace:
   ```
   let register_content = slide.extract_register_content();
   ```
   with:
   ```
   let register_content = slide.register_content.clone();
   ```
   No import of slideforge-eval in slideforge-layout. No new Cargo dep.

5. The `extract_register_content` function in
   `crates/slideforge-eval/src/register_routing.rs` becomes the only
   implementation. It is already correct and comprehensive. No changes needed
   to the function body.

6. `eval_deck` in `crates/slideforge-eval/src/eval.rs` must call
   `crate::register_routing::extract_register_content(&slide)` after
   resolving each slide's fields and store the result in
   `slide.register_content` before adding the slide to the `Deck`.

### What Gets Deleted

| File | What to delete |
|------|----------------|
| `crates/slideforge-types/src/slide.rs` | `fn field_value_to_register_inlines` (private helper, lines 25-46) |
| `crates/slideforge-types/src/slide.rs` | `impl Slide { fn extract_register_content(&self) }` (lines 123-185) |
| `crates/slideforge-types/src/slide.rs` | The `use crate::register::Register;` import if it becomes unused after the above deletions (verify with compiler) |

### What Gets Added

| File | What to add |
|------|-------------|
| `crates/slideforge-types/src/slide.rs` | `pub register_content: Vec<RegisteredContent>` field on `Slide` struct |
| `crates/slideforge-types/src/slide.rs` | Initialize to `vec![]` in all `Slide { ... }` construction sites in tests |
| `crates/slideforge-eval/src/eval.rs` | Call `crate::register_routing::extract_register_content(&slide)` and assign result to `slide.register_content` after field resolution for each slide |

### What Gets Changed

| File | Change |
|------|--------|
| `crates/slideforge-layout/src/layout.rs` line 295 | `slide.extract_register_content()` → `slide.register_content.clone()` |
| All test helpers that construct `Slide { ... }` exhaustively | Add `register_content: vec![]` field |

### No New Cargo Dependencies

Slideforge-layout MUST NOT add slideforge-eval as a dependency. The
`slide.register_content.clone()` approach requires no new Cargo edge. The
DAG remains: `slideforge-eval → slideforge-types`, `slideforge-layout →
slideforge-types`. Layout does not import eval.

### Test Target Mandate (closes adversary F-003)

The existing tests in `crates/slideforge-eval/src/register_routing.rs` call
`extract_register_content(&slide)` directly. These are unit tests of the
function in isolation — they do not exercise the production path (that
`eval_deck` populates `slide.register_content`).

The implementer MUST add an integration-level test in
`crates/slideforge-eval/src/` (or `tests/`) that:

1. Constructs a `DeckNode` AST with a slide containing `notes`, `report`,
   and `detail` fields.
2. Calls `eval_deck(...)` (the production function, not `extract_register_content` directly).
3. Asserts that the resulting `Deck`'s slides have non-empty `register_content`
   populated with the correct `RegisteredContent` entries.

This test exercises the production pipeline path. The existing module-level
tests in `register_routing.rs` remain as unit tests of the pure function;
the new test is the integration proof that `eval_deck` wires it correctly.

---

## F-002: AC-005 / EC-003 (standalone `section detail:`) — DESCOPE DECISION

### Decision

**AC-005 and EC-003 are BLOCKED on deferred IR-extension work. They must be
DESCOPED from STORY-035.**

### Reasoning from IR Types

AC-005 requires: "Standalone `section detail:` blocks (with no parent slide)
also produce `RegisteredContent` entries attached to a synthetic section
node."

EC-003 requires: "Standalone `section detail:` with no parent slide —
`Detail` entry on section node; not attached to any `LaidOutSlide`."

The type is `SectionBlock` in `crates/slideforge-types/src/deck.rs`:

```rust
pub struct SectionBlock {
    pub name: Arc<str>,
    pub body: OrderedMap<Arc<str>, Value>,   // <-- Value, not FieldValue
    pub span: SourceSpan,
}
```

The `body` field is `OrderedMap<Arc<str>, Value>`. `Value` is a fully
resolved scalar (Str, Int, Bool, Float, Null, List, Map). It cannot carry:
- `FieldValue::Inlines` (rich inline content for `RegisteredContent.content`)
- `Vec<Block>` (nested block structure for a `detail:` sub-block)

The `deck.rs` NOTE comment is explicit: "full support is deferred to a
future IR-extension story (cross-crate work needed to introduce
`FieldValue::Inlines` and nested `Vec<Block>` in `SectionBlock`)."

To produce `RegisteredContent { register: Register::Detail, content:
Vec<InlineNode> }` from a `SectionBlock`, the `detail:` sub-block must
be reachable as a typed `FieldValue::Inlines` or `Vec<Block>` from
`SectionBlock.body`. Currently it cannot be — `Value::Str` would be the
only representation, losing the inline structure required by
`RegisteredContent.content: Vec<InlineNode>`. A plain-string workaround
is not acceptable under the production-grade default (CLAUDE.md rule 1).

### Descope Scope

Remove from STORY-035 spec:

- AC-005 (the full acceptance criterion — both slide AND section cases)
- EC-003 (standalone section detail edge case)

The slide-level `detail` field extraction (AC-005 partial) was already
implemented in `extract_register_content` for the `Slide` type (see
`Register::Detail` in register_routing.rs). That slide-level path is
correct and stays. Only the section-node attachment is descoped.

Rewrite AC-005 to cover only slide-level `detail` fields (not standalone
section nodes). The section-node case becomes a separate AC in the
target story below.

### Anchor Story

The descoped behavior anchors to **BC-3.02.002** (section collection and
content authoring). The future story that decomposes BC-3.02.002 fully (the
"IR-extension story" cited in `deck.rs`) must:

1. Change `SectionBlock.body` from `OrderedMap<Arc<str>, Value>` to
   `OrderedMap<Arc<str>, FieldValue>` (or add `Vec<Block>` for nested blocks).
2. Teach the parser to produce `FieldValue::Inlines` for `detail:` sub-blocks
   inside `section <type>:` declarations.
3. Add an AC: "Standalone `section detail:` block produces `RegisteredContent
   { register: Register::Detail, ... }` attached to the section's output in
   `LaidOutDeck`."

That story does not yet exist in the story backlog as of this writing. The
adversary finding creates a MUST-CREATE obligation: a follow-on story to
BC-3.02.002 that scopes the IR extension and section-level register routing.

### Product-Owner Action Required

The product-owner must:

1. Strike AC-005's section-node clause from STORY-035 spec. Rewrite AC-005
   to: "After evaluation, every `LaidOutSlide` with a non-empty `detail`
   field or `detail:` block has at least one `RegisteredContent { register:
   Register::Detail, content: ... }` in `register_content`." (Remove the
   sentence "Standalone `section detail:` blocks (with no parent slide) also
   produce RegisteredContent entries attached to a synthetic section node.")

2. Strike EC-003 entirely from STORY-035 edge cases table.

3. Create a new story (suggest STORY-035b or STORY-03X) scoped to:
   - Extending `SectionBlock.body` to support `FieldValue` (not `Value`)
   - Parser changes to emit `FieldValue::Inlines` for `detail:` within `section`
   - Eval-stage routing of section-level `detail` to `RegisteredContent`
   - Traceability: BC-3.02.002 postcondition 1 + EC-004 + descoped EC-003
   - Blocking dependency: this story must complete before any DOCX exporter
     story attempts to render section-level detail content

4. The product-owner must NOT accept a workaround where section `detail:` is
   stored as a plain `Value::Str` and then re-parsed into inline nodes at
   export time — that pattern violates the evaluate-stage routing invariant
   (BC-1.14.003 invariant 1) and creates a second source of truth.

---

## Summary of Required Actions by Role

### Implementer

1. Add `register_content: Vec<RegisteredContent>` field to `slideforge_types::Slide`.
2. Initialize it to `vec![]` everywhere `Slide { ... }` is constructed
   (parser, test helpers in all crates).
3. In `slideforge_eval::eval.rs::eval_deck`, after resolving each slide's
   fields, call `crate::register_routing::extract_register_content(&slide)`
   and assign the result to `slide.register_content`.
4. In `slideforge_layout::layout.rs:295`, replace the method call with
   `slide.register_content.clone()`.
5. Delete `Slide::extract_register_content()` and the private
   `field_value_to_register_inlines` helper from `slideforge-types/src/slide.rs`.
6. Add the integration test in slideforge-eval that exercises `eval_deck`
   end-to-end and asserts `register_content` is populated (not just the
   unit tests of the isolated function).

### Product-Owner

1. Amend STORY-035 spec: narrow AC-005 to slide-level `detail` only;
   delete EC-003.
2. Create the IR-extension follow-on story anchoring to BC-3.02.002.
3. Add a `blocks:` dependency from the new IR-extension story to any DOCX
   story that renders section-level detail content.

### Adversary (next pass)

Verify:
- `slideforge-types/src/slide.rs` contains NO `extract_register_content`
  method and NO `field_value_to_register_inlines` private helper.
- `slideforge-layout/Cargo.toml` does NOT list `slideforge-eval` as a
  dependency.
- `slideforge-layout/src/layout.rs` calls `slide.register_content.clone()`
  (not a method dispatch to any extraction function).
- `slideforge-eval/src/eval.rs` (or the post-eval slide-building site) calls
  `crate::register_routing::extract_register_content` and assigns to
  `slide.register_content`.
- The integration test exists and is green.
- STORY-035 spec no longer contains section-node language in AC-005 or EC-003.
