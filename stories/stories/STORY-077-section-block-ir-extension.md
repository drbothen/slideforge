---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-077
title: "SectionBlock IR Extension: FieldValue body + section-level register routing"
epic: EPIC-18
wave: 4
points: 8
priority: P0
tdd_mode: strict
status: draft
# BC status: pending PO authorship for BC-3.02.002 clause coverage review
crate: slideforge-types
target_module: slideforge-types, slideforge-syntax, slideforge-eval
subsystems: [SS-01, SS-02, SS-15]
behavioral_contracts: [BC-3.02.002, BC-1.14.003]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
depends_on:
  - STORY-006
  - STORY-007
  - STORY-008
  - STORY-011
  - STORY-012
  - STORY-013
  - STORY-027
blocks:
  - STORY-041
  - STORY-042
estimated_days: 3
---

# STORY-077: SectionBlock IR Extension — FieldValue body + section-level register routing

## Subsystem Anchor Justification

- SS-15 (IR Types) owns the `SectionBlock` struct change (`body: FieldValue` instead of `Value`) per ARCH-INDEX Subsystem Registry. The IR type change is the load-bearing prerequisite.
- SS-01 (DSL Parser) owns the parser change: teaching `slideforge-syntax` to emit `FieldValue::Inlines` for `detail:` / `report:` sub-blocks inside `section <type>:` declarations. Per ARCH-INDEX, SS-01 (slideforge-syntax) is the single parsing authority.
- SS-02 (Evaluator) owns the eval-stage section-level register routing: after `SectionBlock.body` is typed as `FieldValue`, `slideforge-eval` must route `detail:` and `report:` sub-blocks to `RegisteredContent` entries attached to a section node in the output IR.

## Dependency Anchor Justifications

- Depends on STORY-006 (Parser Core): section block parsing builds on the base parser infrastructure from STORY-006.
- Depends on STORY-007 (Parser: @for, @if, {{ expr }}): section content can include `{{ }}` interpolation and control flow; the parser nodes must exist first.
- Depends on STORY-008 (Parser: @include, variants, set): `section <type>:` syntax is adjacent to the vocabulary introduced in STORY-008; reserved keyword enforcement and section-type detection require STORY-008 foundations.
- Depends on STORY-011 (Expression Evaluator Core): `{{ }}` interpolations inside `section` body content must be resolved during evaluation.
- Depends on STORY-012 (Variable Scoping + @for): section content may reference outer-scope variables set before the section block.
- Depends on STORY-013 (@if/@elif/@else): section content may contain conditional blocks.
- Depends on STORY-027 (Layout: Document Section Generation): STORY-027 defines the `SectionBlock` type and establishes how sections appear in the layout IR. STORY-077 extends that type; the extension must be compatible with the existing layout pass.
- Blocks STORY-041 (DOCX Core Serialization): the DOCX exporter must be able to access section-level `RegisteredContent`; this requires STORY-077's IR extension to be present.
- Blocks STORY-042 (DOCX Auto-Generated Document Sections): section-level register routing from `section detail:` and `section report:` blocks must be present before STORY-042 renders section content in DOCX output.

## Summary

This story delivers the IR-extension and eval-stage routing that was descoped from STORY-035
per architect directive F-002 (2026-05-31). The descope reason: `SectionBlock.body` is
`OrderedMap<Arc<str>, Value>` where `Value` is a fully resolved scalar — it cannot carry
`FieldValue::Inlines` (rich inline content for `RegisteredContent.content`) or `Vec<Block>`
(nested block structure for a `detail:` sub-block).

Three coordinated changes are required:

1. **IR extension (`slideforge-types`)**: Change `SectionBlock.body` from
   `OrderedMap<Arc<str>, Value>` to `OrderedMap<Arc<str>, FieldValue>`. `FieldValue` is
   a sum type that includes `FieldValue::Inlines(Vec<InlineNode>)` and (if `Vec<Block>`
   variant is added) nested block structures. This is the load-bearing change that unblocks
   the rest.

2. **Parser changes (`slideforge-syntax`)**: Teach the parser to emit `FieldValue::Inlines`
   for `detail:` and `report:` sub-blocks inside `section <type>:` declarations, instead of
   coercing them to `Value::Str` (which loses inline structure).

3. **Eval-stage routing (`slideforge-eval`)**: After evaluation, section-level `detail:` and
   `report:` sub-blocks must produce `RegisteredContent` entries attached to a section node
   in the output. These entries are routed through the same `Register::Detail` / `Register::Report`
   path as slide-level register fields (established in STORY-035).

The descoped EC-003 from STORY-035 ("Standalone `section detail:` with no parent slide →
`Detail` entry on section node; not attached to any `LaidOutSlide`") is the canonical
acceptance criterion this story delivers.

### Wave 4 Batch Assignment

STORY-077 is assigned to **Wave 4, Batch A** (parallel with STORY-035/036, STORY-043/044/045,
STORY-073, STORY-075, STORY-076).

**Reasoning:** STORY-077's structural prerequisites are all satisfied by Wave 1-3
(parser foundation Wave 1, evaluator core Wave 2, layout section generation Wave 3 via STORY-027).
It does NOT depend on STORY-035 or STORY-036 (no runtime dependency — they extend different parts
of the eval pipeline). STORY-077 must complete before STORY-042 (Batch B), which renders
section-level content in DOCX output. Since STORY-042 is Batch B of Wave 4, STORY-077 fits
naturally in Batch A. No circular dependency is introduced.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-3.02.002 | Manually Authored Section Blocks appear in DOCX/PDF | AC-001, AC-002, AC-003, AC-004, AC-EC-001 |
| BC-1.14.003 | detail register routes to DOCX/PDF only; excluded from PPTX and web preview | AC-005, AC-006 |

## Acceptance Criteria

### AC-001: SectionBlock.body carries FieldValue (not Value)
(traces to BC-3.02.002 postcondition 1 — section content preserved with inline structure)

After this story, `SectionBlock.body` is typed `OrderedMap<Arc<str>, FieldValue>`. A
`section methodology:` block with a `detail:` sub-block containing rich inline content
(e.g., bold text, xref links) produces a `SectionBlock` whose `body` entry for `"detail"`
is `FieldValue::Inlines(vec![InlineNode::Bold(...), ...])`, not `Value::Str("...")`. The
inline structure is preserved from parse time through to evaluation.

### AC-002: Parser emits FieldValue::Inlines for detail:/report: in section blocks
(traces to BC-3.02.002 postcondition 1 — inline {{ }} resolved; detail/report sub-blocks structured)

The parser produces `FieldValue::Inlines` (not `Value::Str`) for `detail:` and `report:`
sub-block content appearing inside a `section <type>:` declaration. A round-trip test
parses:

```
section methodology:
  detail:
    **Bold claim.** See xref(slide-1).
```

and asserts the resulting `SectionBlock.body["detail"]` is `FieldValue::Inlines` containing
`InlineNode::Bold` and `InlineNode::Xref` nodes — not a flat string.

### AC-003: Eval-stage produces RegisteredContent for section detail: blocks
(traces to BC-1.14.003 invariant 1 — routing determined at Evaluate stage)

After evaluation, a `section <type>:` block whose body contains a `detail:` sub-block
produces at least one `RegisteredContent { register: Register::Detail, content: Vec<InlineNode> }`
attached to the section's output node. The `detail:` content is evaluated (all `{{ expr }}`
interpolations resolved) before being tagged.

A unit test constructs a deck with:
```
section methodology:
  detail: "Methodology detail: {{ client }}"
```
with `client = "Acme"` in scope and asserts the section output carries
`RegisteredContent { register: Register::Detail, content: [InlineNode::Plain("Methodology detail: Acme")] }`.

### AC-004: Eval-stage produces RegisteredContent for section report: blocks
(traces to BC-3.02.002 postcondition 4 — report sub-block in section appears in DOCX)

After evaluation, a `section <type>:` block whose body contains a `report:` sub-block
produces `RegisteredContent { register: Register::Report, content: ... }` attached to
the section's output node. The `report:` content does NOT appear in PPTX or web preview
(enforced via BC-1.14.002 routing rules in the DOCX exporter, which reads from `register_content`).

### AC-005: section detail: content excluded from PPTX and web preview
(traces to BC-1.14.003 postcondition 5 — detail NOT in PPTX or web preview)

Section-level `detail:` register content follows the same exclusion rules as slide-level
`detail:`. It does NOT appear in PPTX slide XML or web preview canvas. A test builds a deck
containing only a `section detail:` block (no slides with detail fields) and asserts the PPTX
ZIP contains no section detail content.

### AC-006: STORY-035 descoped EC-003 is now covered — standalone section detail:
(traces to BC-3.02.002 invariant 1 — section blocks DOCX/PDF only; traces to BC-1.14.003 invariant 1)

A standalone `section <type>:` block with only a `detail:` sub-block (no visual slide body)
produces `RegisteredContent { register: Register::Detail, ... }` on the section's output node.
This entry is NOT attached to any `LaidOutSlide` (because no slide exists). DOCX/PDF exporters
find this entry on the section node and render it in the appropriate section. This is the exact
behavior descoped from STORY-035 EC-003.

### AC-EC-001: Unrecognized section-level register sub-block is a parse warning
(traces to BC-3.02.002 invariant 3 — unrecognized section type → compile error)

A `section methodology:` block with an unrecognized sub-block key (e.g., `foo:`) produces
a lint warning (not a fatal error) naming the unrecognized key, consistent with the general
policy that unknown keys in section bodies are non-fatal in strict mode unless the key
collides with a reserved register name.

## Tasks

- [ ] Change `SectionBlock.body` type in `crates/slideforge-types/src/deck.rs` from `OrderedMap<Arc<str>, Value>` to `OrderedMap<Arc<str>, FieldValue>`
- [ ] Update all construction sites for `SectionBlock { body: ... }` in tests and parser to use `FieldValue` variants
- [ ] Update the parser in `crates/slideforge-syntax/` to emit `FieldValue::Inlines` for `detail:` and `report:` sub-blocks within `section <type>:` declarations (instead of coercing to `Value::Str`)
- [ ] Verify all existing `section` parsing tests still pass after the IR change (no regressions)
- [ ] Add `extract_section_register_content(section: &SectionBlock) -> Vec<RegisteredContent>` in `crates/slideforge-eval/src/register_routing.rs`
  - Extract `detail:` and `report:` entries from `section.body` as `FieldValue::Inlines`
  - Convert inline nodes to `RegisteredContent` with correct `Register` variant
  - Return `Vec<RegisteredContent>`
- [ ] Call `extract_section_register_content` in the eval pipeline for each `SectionBlock` in the `Deck`
- [ ] Attach resulting `Vec<RegisteredContent>` to a section output node in the evaluated IR (coordinate with layout IR for how section-level `register_content` is represented)
- [ ] Write unit tests:
  - `section detail:` with plain text → `Register::Detail` entry on section node
  - `section report:` with plain text → `Register::Report` entry on section node
  - `section detail: "{{ var }}"` with scoped variable → interpolated before tagging
  - Standalone `section detail:` with no slides → section node has `Detail` entry; no `LaidOutSlide` entry
  - `section` with both `detail:` and `report:` sub-blocks → two entries on section node
- [ ] Write integration test: `layout::run()` end-to-end with a deck containing both slides (with register fields) and a `section` block (with `detail:` sub-block); assert correct `register_content` on both slide nodes and section nodes
- [ ] Verify no PPTX slide XML contains section-level `detail:` content (no bleed)

## Previous Story Intelligence

STORY-035 is the direct predecessor for register routing. It establishes:
- `Register` enum (`Notes`, `Report`, `Detail`)
- `RegisteredContent { register: Register, content: Vec<InlineSpan> }` struct
- `extract_register_content(slide: &Slide) -> Vec<RegisteredContent>` in `register_routing.rs`
- `LaidOutSlide.register_content: Vec<RegisteredContent>` field

STORY-077 extends this pattern to section nodes, adding:
- `extract_section_register_content(section: &SectionBlock) -> Vec<RegisteredContent>`
- Section-level `register_content` on section IR nodes

STORY-027 (Layout: Document Section Generation) is the other predecessor — it defines how
sections appear in the layout IR. The implementer must coordinate with the section IR node
type from STORY-027 to add the `register_content` field to the section output node. If the
section IR node type does not yet have a `register_content` field, add it here.

The key lesson from the STORY-035 adversarial review (F-002): do NOT store section `detail:`
as a plain `Value::Str` and re-parse inline nodes at export time. That pattern violates
BC-1.14.003 invariant 1 (routing at Evaluate stage) and creates a second source of truth.

## Architecture Compliance Rules

1. **FieldValue over Value for structured content (architect directive F-002)**: Section body
   sub-blocks that contain rich inline content (`detail:`, `report:`) MUST be stored as
   `FieldValue::Inlines`, not `Value::Str`. No plain-string workarounds for inline structure.
2. **Register routing is an evaluate-stage concern (BC-1.14.003 invariant 1)**: Section-level
   register routing produces `RegisteredContent` at eval time, same as slide-level routing. No
   exporter may re-derive register membership from raw `SectionBlock.body` entries.
3. **No circular Cargo dependencies**: `slideforge-types` → zero workspace deps. `slideforge-syntax`
   → depends on `slideforge-types`. `slideforge-eval` → depends on `slideforge-types` and
   `slideforge-syntax`. This story does not introduce any new Cargo edges.
4. **No exporter crates in slideforge-eval**: `slideforge-eval` must NOT import `slideforge-pptx`,
   `slideforge-docx`, `slideforge-pdf`, or `slideforge-html`. Forbidden — build fails if added.
5. **SS-15 purity (types crate)**: `slideforge-types` contains only type definitions. The
   `FieldValue` type must already exist in `slideforge-types` (introduced with STORY-001 or
   later types stories). If not, add it here. No business logic in the types crate.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` (workspace) | workspace | `FieldValue`, `SectionBlock`, `RegisteredContent`, `Register` types |
| `slideforge-syntax` (workspace) | workspace | Parser changes for `FieldValue::Inlines` emission |
| `slideforge-eval` (workspace) | workspace | `extract_section_register_content` and eval pipeline integration |

No new external dependencies required. This story extends existing types and the evaluation pipeline.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-types/src/deck.rs` | Modify | Change `SectionBlock.body` field type from `OrderedMap<Arc<str>, Value>` to `OrderedMap<Arc<str>, FieldValue>` |
| `crates/slideforge-types/src/field_value.rs` | Create or Modify | Add/extend `FieldValue` enum to include `FieldValue::Inlines(Vec<InlineNode>)` if not present |
| `crates/slideforge-syntax/src/section.rs` | Modify | Emit `FieldValue::Inlines` for `detail:` and `report:` sub-blocks in section parsing |
| `crates/slideforge-eval/src/register_routing.rs` | Modify | Add `extract_section_register_content(section: &SectionBlock) -> Vec<RegisteredContent>` |
| `crates/slideforge-eval/src/eval.rs` | Modify | Call `extract_section_register_content` for each `SectionBlock` in eval pipeline |
| `crates/slideforge-eval/src/tests/section_register_routing_tests.rs` | Create | Unit tests for all AC cases |
| `crates/slideforge-eval/tests/section_register_integration.rs` | Create | Integration test for full pipeline with section + slide register content |

## Forbidden Dependencies

`slideforge-types` must NOT depend on:
- Any other workspace crate (it is the foundation layer)

`slideforge-eval` must NOT depend on:
- `slideforge-pptx` — exporter crate
- `slideforge-docx` — exporter crate
- `slideforge-pdf` — exporter crate
- `slideforge-html` — exporter crate
- `slideforge-preview` — effectful shell

Build fails if any of the above appear in the respective `Cargo.toml`.

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,500 |
| BC-3.02.002 | ~1,500 |
| BC-1.14.003 | ~1,200 |
| STORY-035 context (register routing patterns) | ~2,500 |
| STORY-027 context (section IR node type) | ~1,500 |
| `slideforge-types` deck.rs + FieldValue types | ~2,000 |
| Parser section.rs context | ~1,500 |
| Test files to write | ~3,000 |
| **Total** | **~16,700** |

Within 20-30% of the agent context window. If the FieldValue type requires significant new
definition work (not yet in the codebase), the implementer should request a story split.

## Test Strategy

- **Unit tests** (in `crates/slideforge-eval/src/tests/section_register_routing_tests.rs`):
  - `section detail:` with plain text → one `Register::Detail` entry on section node
  - `section report:` with plain text → one `Register::Report` entry
  - `section detail: "{{ client }}"` with scoped `client = "Acme"` → interpolated
  - Standalone `section detail:` (no slides in deck) → section node has `Detail` entry; no `LaidOutSlide`
  - `section` with both `detail:` and `report:` sub-blocks → two entries on section node
- **Integration test** (in `crates/slideforge-eval/tests/section_register_integration.rs`):
  - Full `eval_deck()` call on a deck with slides (each having `notes`/`report`/`detail` fields)
    AND a `section methodology:` block (with `detail:` sub-block)
  - Assert: slide `register_content` populated correctly (delegates to STORY-035 path)
  - Assert: section node `register_content` populated with `Register::Detail` entry
  - Assert: section `detail:` content does NOT appear in any `LaidOutSlide.register_content`
- **Snapshot test** (`insta`): snapshot of section node `register_content` for a 2-section
  deck with mixed register sub-blocks — regression detection.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `section <type>:` with no `detail:` or `report:` sub-block | Section node `register_content` is empty (`vec![]`); no error |
| EC-002 | `section <type>:` with `detail: "{{ undefined_var }}"` | Eval error: undefined variable with full scope path; same error as slide-level interpolation failure |
| EC-003 | Section-level `detail:` content appearing in PPTX output | **Must not happen** — PPTX exporter reads only `LaidOutSlide.register_content`, never section nodes (invariant test) |
| EC-004 | `section report:` sub-block content | `RegisteredContent { register: Register::Report, ... }` on section node; appears in DOCX section via STORY-042 |
| EC-005 | Multiple `section` blocks each with `detail:` sub-blocks | Each section node gets its own `register_content`; no cross-contamination |
| EC-006 | `section detail:` sub-block inside a `@for` loop | Each loop iteration's section gets its own section node with `register_content` from that iteration |

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-31 | story-writer | Initial story creation — spun out from STORY-035 per architect directive F-002 (section-level register routing requires SectionBlock IR extension not available in v1.0 Wave 4 without this story) |
