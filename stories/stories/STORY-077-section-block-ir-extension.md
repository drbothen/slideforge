---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-077
title: "SectionBlock IR Extension: FieldValue body + section-level register routing + inline-markup parser"
epic: EPIC-18
wave: 4
points: 13
priority: P0
tdd_mode: strict
status: ready
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
  - STORY-078
blocks:
  - STORY-041
  - STORY-042
  - STORY-081
estimated_days: 5
---

# STORY-077: SectionBlock IR Extension — FieldValue body + section-level register routing + inline-markup parser

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
- Depends on STORY-078 (Parser: section block syntax): STORY-077's AC-002 requires the parser to already produce `SectionNode` entries from source. STORY-078 delivers the `section_block_parser` combinator and un-reserves the `"section"` keyword. STORY-077 is BLOCKED until STORY-078 merges. Per DIR-077-001 §2, STORY-078 initially stores sub-block values as `FieldValue::Template`; STORY-077's parser extension work upgrades `detail:` and `report:` keys to `FieldValue::Inlines`.
- Blocks STORY-041 (DOCX Core Serialization): the DOCX exporter must be able to access section-level `RegisteredContent`; this requires STORY-077's IR extension to be present.
- Blocks STORY-042 (DOCX Auto-Generated Document Sections): section-level register routing from `section detail:` and `section report:` blocks must be present before STORY-042 renders section content in DOCX output.

## Summary

This story delivers the IR-extension and eval-stage routing that was descoped from STORY-035
per architect directive F-002 (2026-05-31). The descope reason: `SectionBlock.body` is
`OrderedMap<Arc<str>, Value>` where `Value` is a fully resolved scalar — it cannot carry
`FieldValue::Inlines` (rich inline content for `RegisteredContent.content`) or `Vec<Block>`
(nested block structure for a `detail:` sub-block).

**DIR-077-001 parsing prerequisite:** Section-block parsing is delivered by STORY-078
(a new STORY-027 decomposition gap story). STORY-077 is BLOCKED until STORY-078 merges.
The task entry "Update the parser ... to emit `FieldValue::Inlines` for `detail:` and
`report:` sub-blocks within `section <type>:` declarations" is now scoped as: extend the
STORY-078 parser output — which initially stores sub-block values as `FieldValue::Template` —
to emit `FieldValue::Inlines` for the recognized register keys (`detail:`, `report:`).

**DIR-077-001-A corrections (2026-06-01):**

1. **Unrecognized sub-block KEY warning is parse-time (Ruling 2, STORY-078's ownership):**
   STORY-078's `section_block_parser` emits the non-fatal `ParseSeverity::Warning` for
   unrecognized sub-block keys via the `validate()`/`emit` + `push_with_severity` pattern.
   STORY-077's eval stage (`eval_section_nodes`) must NOT re-emit this warning. AC-EC-001
   has been updated accordingly.

2. **Section TYPE validation is eval-stage (Ruling 3, STORY-077's obligation):**
   STORY-078 stores the section type IDENT verbatim with no parser-level rejection.
   STORY-077's `eval_section_nodes` function is the authority for type validation: when
   `SectionNode.kind` does not match any entry in the `SectionType` plugin registry
   (built-ins: methodology, scope, approval, appendix, glossary, plus any plugin-registered
   types), emit a fatal eval-stage error equivalent to `LayoutError::UnknownSectionType`
   with message: `"Unknown section type '<name>'. Known types: [...]"`. This satisfies
   BC-3.02.002 invariant 3 at the correct stage.

3. **Eval still routes RegisteredContent from FieldValue::Inlines** for `detail:` and
   `report:` sub-blocks as originally scoped.

**`SECTION_REGISTER_KEYS` correction:** Per DIR-077-001 §5, `SECTION_REGISTER_KEYS` in
`section.rs` must be `["report", "detail"]` — NOT `["notes", "report", "detail"]`. The
`notes` register is the PRESENTER register (speaker view on a slide canvas). Document
sections have no PPTX rendering path and no slide canvas; `notes:` on a section block is
semantically meaningless. `extract_section_register_content` already excludes `notes`
correctly; the `SECTION_REGISTER_KEYS` constant must be brought into alignment. This is a
code correction, not a BC change.

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
(traces to BC-3.02.002 postcondition 8 — inline-structure preservation; FieldValue::Inlines not Value::Str)

After this story, `SectionBlock.body` is typed `OrderedMap<Arc<str>, FieldValue>`. A
`section methodology:` block with a `detail:` sub-block containing rich inline content
(e.g., bold text, xref links) produces a `SectionBlock` whose `body` entry for `"detail"`
is `FieldValue::Inlines(vec![InlineNode::Bold(...), ...])`, not `Value::Str("...")`. The
inline structure is preserved from parse time through to evaluation.

### AC-002: Inline-markup parser produces structural InlineNode variants for section detail:/report: values
(traces to BC-3.02.002 postcondition 8 — inline-structure preservation; FieldValue::Inlines containing structural Bold/Italic/Code/etc. nodes, not literal-asterisk Plain text)

**SCOPE BOUNDARY:** This AC covers section sub-block (`detail:` / `report:`) field values ONLY.
Slide-level field values (`title`, `bullets`, `body`) are addressed in the follow-up STORY-081.

The inline-markup parser is delivered in two phases:

**Phase 1 — Parse time (`slideforge-syntax`):** `template_value()` is extended with new
`TemplateChunk` variants — `Bold(Vec<TemplateChunk>)`, `Italic(Vec<TemplateChunk>)`,
`Code(String)`, `Link { text: Vec<TemplateChunk>, url: String }`, `Superscript(Vec<TemplateChunk>)`,
`Subscript(Vec<TemplateChunk>)`, `Strikethrough(Vec<TemplateChunk>)`,
`Highlight(Vec<TemplateChunk>)` — that mirror `InlineNode` structurally without introducing
a cross-crate dependency from `slideforge-syntax` to `slideforge-types`. An inline markup
scanning pass is added to `template_value()` recognizing `**bold**`, `_italic_`, `` `code` ``,
`[text](url)`, `^sup^`, `~sub~`, `~~del~~`, `==highlight==`, and `{{ ref("id") }}` /
`{{ footnote("...") }}` expression forms (per DIR-077-002 §1 canonical v1 syntax table).

**Phase 2 — Eval time (`slideforge-eval`):** A new pure function
`chunks_to_inline_nodes(chunks: &[TemplateChunk], env: &Env, sink: &mut DiagnosticSink) -> Vec<InlineNode>`
converts `TemplateChunk` sequences into `Vec<InlineNode>` per the mapping in DIR-077-002 §3.
It is called for `detail:` and `report:` sub-block values during section evaluation.
The result is stored as `slideforge_types::FieldValue::Inlines(nodes)` on `SectionBlock.body[key]`.

**Acceptance test (load-bearing — drives the production parser+eval path):** A round-trip
integration test parses and evaluates:

```
section methodology:
  detail: "**Bold claim.** See {{ ref(\"slide-1\") }}."
```

and asserts:
1. `SectionBlock.body["detail"]` is `FieldValue::Inlines(vec![...])`, NOT `FieldValue::Template(...)` or `Value::Str("**Bold claim.** See {{ ref(\"slide-1\") }}.")`.
2. The `Vec<InlineNode>` contains `InlineNode::Bold([InlineNode::Plain(Arc::from("Bold claim."))])` as the first node.
3. The `Vec<InlineNode>` contains `InlineNode::Xref(Arc::from("slide-1"))` (not `InlineNode::Plain(Arc::from("ref(\"slide-1\")"))`).
4. NO `InlineNode::Plain` node contains literal `**` or `*` characters. The forbidden pattern from CLAUDE.md is explicitly verified: `InlineNode::Plain(Arc::from("**Bold claim.**"))` must NOT appear in the output.

**Additional required tests per DIR-077-002 §8 (Red Gate — all must fail before implementation starts):**

In `crates/slideforge-syntax/` (template chunk parsing):
- `test_bold_chunk_produced` — `"**hello**"` → `[TemplateChunk::Bold([TemplateChunk::Literal("hello")])]`
- `test_italic_chunk_produced` — `"_hello_"` → `[TemplateChunk::Italic([...])]`
- `test_code_chunk_produced` — `` "`fn foo()`" `` → `[TemplateChunk::Code("fn foo()")]`
- `test_link_chunk_produced` — `"[click here](https://example.com)"` → `TemplateChunk::Link { text: [...], url: "https://example.com" }`
- `test_superscript_chunk_produced` — `"^2^"` → `[TemplateChunk::Superscript([...])]`
- `test_subscript_chunk_produced` — `"~n~"` → `[TemplateChunk::Subscript([...])]`
- `test_strikethrough_before_subscript` — `"~~del~~ ~sub~"` → `[Strikethrough([...]), Literal(" "), Subscript([...])]` (disambiguation: `~~` consumed before `~`)
- `test_highlight_chunk_produced` — `"==highlight me=="` → `[TemplateChunk::Highlight([...])]`
- `test_bold_with_interpolation` — `"**{{ client }}**"` → `[TemplateChunk::Bold([TemplateChunk::Expr(Expr::Ident("client"))])]`
- `test_nested_bold_italic` — `"**_bold italic_**"` → `[TemplateChunk::Bold([TemplateChunk::Italic([...])])]`
- `test_code_span_no_inner_markup` — `` "`**not bold**`" `` → `[TemplateChunk::Code("**not bold**")]` (verbatim; no inner markup)
- `test_math_mode_no_inline_markup` — `"$**not bold**$"` → `[TemplateChunk::MathInline("**not bold**")]` (math mode disables inline markup)
- `test_unclosed_bold_error_accumulated` — `"**unclosed"` → error accumulated (non-fatal), error count == 1, span points to opening `**`
- `test_empty_bold_error_accumulated` — `"****"` → error accumulated, error count == 1
- `test_template_chunk_bold_derives_hash_eq_clone_debug` — `#[derive(Hash, Eq, Clone, Debug)]` on `TemplateChunk::Bold` (comemo compatibility, ADR-013)

In `crates/slideforge-eval/` (chunks_to_inline_nodes conversion + AC-002 end-to-end):
- `test_bold_chunk_to_inline_node` — `TemplateChunk::Bold([Literal("hi")])` → `InlineNode::Bold([InlineNode::Plain(Arc::from("hi"))])`
- `test_italic_chunk_to_inline_node`
- `test_code_chunk_to_inline_node` — `TemplateChunk::Code("fn x() {}")` → `InlineNode::Code(Arc::from("fn x() {}"))`
- `test_link_chunk_to_inline_node`
- `test_math_inline_chunk_to_inline_node` — → `InlineNode::Math(MathNode { display: false, ... })`
- `test_math_display_chunk_to_inline_node` — → `InlineNode::Math(MathNode { display: true, ... })`
- `test_expr_ref_call_to_xref` — `TemplateChunk::Expr(Expr::Call { func: "ref", args: [Expr::Str("slide-1")] })` → `InlineNode::Xref(Arc::from("slide-1"))`
- `test_expr_footnote_call_to_footnote_node` — → `InlineNode::Footnote([InlineNode::Plain(Arc::from("see appendix"))])`
- `test_ac002_bold_in_section_detail_produces_inlines` — full pipeline: source `section methodology:\n  detail:\n    **Bold claim.**` → `SectionBlock.body["detail"]` is `FieldValue::Inlines([InlineNode::Bold([InlineNode::Plain(Arc::from("Bold claim."))])])`
- `test_ac002_xref_in_section_detail_produces_xref_node` — `{{ ref("slide-1") }}` → `InlineNode::Xref(Arc::from("slide-1"))`
- `test_ac002_plain_text_not_literal_asterisks` — `**Bold** text.` → contains `InlineNode::Bold(...)` and `InlineNode::Plain(Arc::from(" text."))`, NOT a single `InlineNode::Plain` with literal `**`
- `test_ac002_interpolation_with_bold_context` — `**{{ client }}**` with `client = "Acme"` → `InlineNode::Bold([InlineNode::Plain(Arc::from("Acme"))])`
- `test_ac002_report_sub_block_produces_inlines` — same as bold test but for `report:` key

### AC-003: Eval-stage produces RegisteredContent for section detail: blocks
(traces to BC-3.02.002 postcondition 7 — eval-stage RegisteredContent{Register::Detail} on section node; BC-1.14.003 invariant 1 — routing determined at Evaluate stage)

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
(traces to BC-3.02.002 EC-004 — eval-stage RegisteredContent{Register::Report} on section node)

After evaluation, a `section <type>:` block whose body contains a `report:` sub-block
produces `RegisteredContent { register: Register::Report, content: ... }` attached to
the section's output node. The `report:` content does NOT appear in PPTX or web preview
(enforced via BC-1.14.002 routing rules in the DOCX exporter, which reads from `register_content`).

### AC-005: section detail: content excluded from PPTX and web preview
(traces to BC-1.14.003 postcondition 3 — detail NOT in PPTX; BC-1.14.003 postcondition 5 — detail NOT in web preview)

Section-level `detail:` register content follows the same exclusion rules as slide-level
`detail:`. It does NOT appear in PPTX slide XML or web preview canvas. A test builds a deck
containing only a `section detail:` block (no slides with detail fields) and asserts the PPTX
ZIP contains no section detail content.

### AC-006: STORY-035 descoped EC-003 is now covered — standalone section detail:
(traces to BC-3.02.002 postcondition 7 — eval-stage RegisteredContent{Register::Detail} not attached to any LaidOutSlide; BC-1.14.003 EC-001 — standalone section detail: and section <type>:/detail: sub-block both valid, not attached to any slide)

A standalone `section <type>:` block with only a `detail:` sub-block (no visual slide body)
produces `RegisteredContent { register: Register::Detail, ... }` on the section's output node.
This entry is NOT attached to any `LaidOutSlide` (because no slide exists). DOCX/PDF exporters
find this entry on the section node and render it in the appropriate section. This is the exact
behavior descoped from STORY-035 EC-003.

### AC-EC-001: Eval-stage does NOT re-emit the unrecognized-sub-block-key warning
(traces to BC-3.02.002 invariant 4 — unrecognized sub-block key warning is parse-time, owned by STORY-078; BC-3.02.002 EC-005)

Per DIR-077-001-A Ruling 2, the non-fatal lint warning for an unrecognized sub-block key
(e.g., `foo:`) is emitted at parse time by STORY-078's `section_block_parser`. The eval
stage (`eval_section_nodes`) must NOT emit a second copy of this warning. When eval
encounters a `FieldNode` whose key is not in `REGISTER_SUB_BLOCK_KEYS`, it silently
skips it (the user has already received the parse-time warning). A test verifies:

Given a deck with `section methodology: / foo: "x"`, after full `eval_deck()`, assert
that `eval_diagnostics` contains no `UnrecognizedSectionSubBlockKey` entry (the
warning was already emitted during parsing, not duplicated at eval).

## Tasks

### Phase 0: IR Extension (slideforge-types)
- [ ] Change `SectionBlock.body` type in `crates/slideforge-types/src/deck.rs` from `OrderedMap<Arc<str>, Value>` to `OrderedMap<Arc<str>, FieldValue>`
- [ ] Update all construction sites for `SectionBlock { body: ... }` in tests and parser to use `FieldValue` variants
- [ ] Verify all existing `section` parsing tests still pass after the IR change (no regressions)

### Phase 1: Inline-Markup Parser — TemplateChunk Extension (slideforge-syntax) [NEW per DIR-077-002]
- [ ] Add new `TemplateChunk` variants to `crates/slideforge-syntax/src/template.rs`:
  `Bold(Vec<TemplateChunk>)`, `Italic(Vec<TemplateChunk>)`, `Code(String)`,
  `Link { text: Vec<TemplateChunk>, url: String }`, `Superscript(Vec<TemplateChunk>)`,
  `Subscript(Vec<TemplateChunk>)`, `Strikethrough(Vec<TemplateChunk>)`,
  `Highlight(Vec<TemplateChunk>)`. All must derive `Debug + Clone + PartialEq + Eq + Hash`
  (comemo requirement per ADR-013). `Footnote` and `Xref` are NOT new variants — they arrive
  via `TemplateChunk::Expr` function-call expressions.
- [ ] Extend `template_value()` in `crates/slideforge-syntax/src/parser/template.rs` with an
  inline markup scanning pass recognizing all 8 delimiter forms (per DIR-077-002 §1):
  - `**text**` → `TemplateChunk::Bold`; `_text_` → `TemplateChunk::Italic`;
    `` `text` `` → `TemplateChunk::Code` (verbatim — no inner markup, no `{{ }}` inside);
    `[text](url)` → `TemplateChunk::Link`; `^text^` → `TemplateChunk::Superscript`;
    `~~text~~` → `TemplateChunk::Strikethrough` (must be checked BEFORE `~text~`);
    `~text~` → `TemplateChunk::Subscript`; `==text==` → `TemplateChunk::Highlight`
  - Compose with existing `{{ expr }}` interpolation at the same level; `{{ }}` fires inside
    Bold/Italic/Link text children but NOT inside Code or math spans
  - Math regions (`$...$`, `$$...$$`) disable all inline markup — existing rule unchanged
  - Disambiguation: `~~` consumed greedily before `~`; single `*` NOT treated as italic
    (only `_` for italic per DIR-077-002 §1 disambiguation rule 2)
- [ ] Implement error accumulation for malformed markup (per DIR-077-002 §5):
  - Unclosed span (`**unclosed`) → non-fatal error with span at opening delimiter + correction hint; treat content through end-of-string as the span's content; parsing continues
  - Empty span (`****`) → non-fatal error; skip; parsing continues
  - Every error carries a `Span` pointing to the opening delimiter and a correction hint message
- [ ] Ensure `template_value()` is already called by the STORY-078 section-block parser for
  sub-block values — no call-site change needed there; the enhanced parser automatically
  produces new `TemplateChunk` variants for section sub-block content

### Phase 2: Eval-Stage chunks_to_inline_nodes (slideforge-eval) [NEW per DIR-077-002]
- [ ] Add `pub fn chunks_to_inline_nodes(chunks: &[TemplateChunk], env: &Env, sink: &mut DiagnosticSink) -> Vec<InlineNode>`
  to `crates/slideforge-eval/src/register_routing.rs` implementing the full mapping per
  DIR-077-002 §3 (Literal→Plain, Bold→Bold, Italic→Italic, Code→Code, Link→Link,
  MathInline→Math{display:false}, MathDisplay→Math{display:true}, Superscript→Superscript,
  Subscript→Subscript, Strikethrough→Strikethrough, Highlight→Highlight,
  Expr::Call{func:"ref",...}→Xref, Expr::Call{func:"footnote",...}→Footnote,
  Expr::Call{func:"figref",...}→Xref(format!("fig-{n}")),
  Expr(other)→eval_expr_to_string→Plain, MathInterp→Plain after eval)
- [ ] Call `chunks_to_inline_nodes` for `detail:` and `report:` sub-block values when building
  `FieldValue::Inlines` — this is the integration point: `FieldValue::Template(chunks)` →
  `chunks_to_inline_nodes(chunks, env, sink)` → `FieldValue::Inlines(nodes)` on `SectionBlock.body[key]`

### Phase 3: Section Register Routing (slideforge-eval)
- [ ] Add `extract_section_register_content(section: &SectionBlock) -> Vec<RegisteredContent>` in `crates/slideforge-eval/src/register_routing.rs`
  - Extract `detail:` and `report:` entries from `section.body` as `FieldValue::Inlines`
  - Convert inline nodes to `RegisteredContent` with correct `Register` variant
  - Return `Vec<RegisteredContent>`; `SECTION_REGISTER_KEYS` = `["report", "detail"]` (NOT "notes" per DIR-077-001 §5)
- [ ] In `eval_section_nodes`, validate `SectionNode.kind` against the `SectionType` plugin
      registry (built-ins: methodology, scope, approval, appendix, glossary, plus any
      plugin-registered types); emit a fatal eval-stage error for unrecognized types:
      `"Unknown section type '<name>'. Known types: [...]"` (BC-3.02.002 invariant 3;
      DIR-077-001-A Ruling 3 — this check is NOT in STORY-078's parser)
- [ ] In `eval_section_nodes`, when a `FieldNode` key is NOT in `REGISTER_SUB_BLOCK_KEYS`,
      silently skip it — do NOT emit a duplicate warning (the parse-time warning was already
      emitted by STORY-078's parser; DIR-077-001-A Ruling 2)
- [ ] Call `extract_section_register_content` in the eval pipeline for each `SectionBlock` in the `Deck`
- [ ] Attach resulting `Vec<RegisteredContent>` to a section output node in the evaluated IR (coordinate with layout IR for how section-level `register_content` is represented)

### Phase 4: Tests
- [ ] Write all 15 Red Gate tests in `crates/slideforge-syntax/src/parser/` for `TemplateChunk`
  variants and error accumulation (full list in AC-002 above; per DIR-077-002 §8)
- [ ] Write all 13 Red Gate tests in `crates/slideforge-eval/` for `chunks_to_inline_nodes`
  conversion and AC-002 end-to-end pipeline (full list in AC-002 above; per DIR-077-002 §8)
- [ ] Write existing-scope unit tests:
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
| `crates/slideforge-syntax/src/template.rs` | Modify | Add 8 new `TemplateChunk` variants: Bold, Italic, Code, Link, Superscript, Subscript, Strikethrough, Highlight (per DIR-077-002 §3) |
| `crates/slideforge-syntax/src/parser/template.rs` | Modify | Extend `template_value()` with inline markup scanning pass; error accumulation for malformed markup |
| `crates/slideforge-syntax/src/section.rs` | No change needed | STORY-078's section parser already calls `template_value()`; the enhanced parser produces correct chunks automatically |
| `crates/slideforge-eval/src/register_routing.rs` | Modify | Add `chunks_to_inline_nodes()` + `extract_section_register_content(section: &SectionBlock) -> Vec<RegisteredContent>` |
| `crates/slideforge-eval/src/eval.rs` | Modify | Call `extract_section_register_content` for each `SectionBlock` in eval pipeline |
| `crates/slideforge-syntax/src/parser/tests/template_inline_markup_tests.rs` | Create | 15 Red Gate tests for TemplateChunk variants and error accumulation (per DIR-077-002 §8) |
| `crates/slideforge-eval/src/tests/section_register_routing_tests.rs` | Create | 13 Red Gate tests for chunks_to_inline_nodes + AC-002 end-to-end cases (per DIR-077-002 §8) + existing-scope register routing unit tests |
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
| This story spec | ~5,500 |
| BC-3.02.002 | ~1,500 |
| BC-1.14.003 | ~1,200 |
| DIR-077-002 (inline-markup parser directive) | ~3,500 |
| STORY-035 context (register routing patterns) | ~2,500 |
| STORY-027 context (section IR node type) | ~1,500 |
| `slideforge-types` deck.rs + FieldValue types | ~2,000 |
| `slideforge-syntax` template.rs + parser/template.rs context | ~2,500 |
| `slideforge-eval` register_routing.rs context | ~1,500 |
| Test files to write (28 Red Gate + existing-scope + integration) | ~6,000 |
| **Total** | **~27,700** |

Within 20-30% of a 128k-token agent context window (27,700 / 128,000 ≈ 22%). The expanded
scope (inline-markup parser across two crates + 28 Red Gate tests) is substantial but fits.
If the `TemplateChunk` enum has significant existing content requiring deep context, the
implementer may split the test writing into a separate burst without requesting a story split.

## Test Strategy

- **Red Gate tests — template chunk parsing** (in `crates/slideforge-syntax/src/parser/tests/template_inline_markup_tests.rs`):
  All 15 tests listed in AC-002 above. Must fail before implementation starts (TDD Red Gate).
  Key assertions: correct `TemplateChunk` variant produced; error accumulation for unclosed/empty;
  `#[derive(Hash, Eq, Clone, Debug)]` on all new variants; math mode and code span disable inline markup.

- **Red Gate tests — chunks_to_inline_nodes + AC-002 end-to-end** (in `crates/slideforge-eval/src/tests/section_register_routing_tests.rs`):
  All 13 tests listed in AC-002 above. Must fail before implementation starts.
  Key assertions: each TemplateChunk variant maps to correct InlineNode; `ref()` and `footnote()`
  expression calls map to Xref/Footnote; full pipeline produces `FieldValue::Inlines` NOT
  `Value::Str` with literal asterisks.

- **Existing-scope unit tests** (in `crates/slideforge-eval/src/tests/section_register_routing_tests.rs`):
  - `section detail:` with plain text → one `Register::Detail` entry on section node
  - `section report:` with plain text → one `Register::Report` entry
  - `section detail: "{{ client }}"` with scoped `client = "Acme"` → interpolated
  - Standalone `section detail:` (no slides in deck) → section node has `Detail` entry; no `LaidOutSlide`
  - `section` with both `detail:` and `report:` sub-blocks → two entries on section node
  - `section foobar: / detail: "x"` → fatal eval error naming `foobar`, known types listed
    (DIR-077-001-A Ruling 3: parser stored type verbatim; eval validates against registry)
  - `section methodology: / foo: "x"` → eval diagnostics do NOT contain
    `UnrecognizedSectionSubBlockKey` (warning was emitted at parse time by STORY-078;
    eval skips silently per DIR-077-001-A Ruling 2)

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
| EC-006 | `section <type>:` block written inside a `@for` loop body | Parse error (or silently not collected): section blocks are top-level deck items only (DIR-077-001 §3.2, §3.7; BC-3.02.002 EC-002). The `deck_parser` production for `section` is wired at deck level alongside `slide`; it is NOT available inside `@for`/`@if` bodies. At eval, `eval_section_nodes` iterates only `BlockItem::Section` entries in `deck.items` (top-level); any `section`-like token inside a for-loop body either fails to parse or produces a different `BlockItem` variant that is not collected as a section node. No per-iteration section nodes are produced. |
| EC-007 | Unclosed inline markup in `detail:` value: `detail: "**unclosed"` | Non-fatal parse error accumulated (E-PAR-NNN inline-bold-unclosed); span points to opening `**`; correction hint emitted. Content through end-of-string treated as Bold content. Parsing continues — other sub-blocks are processed. `SectionBlock.body["detail"]` is `FieldValue::Inlines([InlineNode::Bold([InlineNode::Plain(Arc::from("unclosed"))])])`. Error surfaces in diagnostic sink; in strict mode (default) the build fails with the accumulated error. |
| EC-008 | Empty inline markup span: `detail: "****"` | Non-fatal parse error accumulated (E-PAR-NNN inline-empty-span); span points to `****` location. Empty span skipped. Parsing continues. The `detail:` value produces an empty `FieldValue::Inlines([])`. Error surfaces in diagnostic sink. |
| EC-009 | `~~del~~` adjacent to `~sub~`: `detail: "~~del~~ ~sub~"` | `~~` consumed greedily before `~` (disambiguation rule 1, DIR-077-002 §1). Result: `FieldValue::Inlines([InlineNode::Strikethrough([InlineNode::Plain(Arc::from("del"))]), InlineNode::Plain(Arc::from(" ")), InlineNode::Subscript([InlineNode::Plain(Arc::from("sub"))])])`. No ambiguity or error. |
| EC-010 | `{{ ref("") }}` (empty xref id) in `detail:` | Non-fatal parse error accumulated (E-PAR-NNN inline-xref-empty-id). No `InlineNode::Xref` produced for this expression. Parsing continues. Error carries span and hint. |
| EC-011 | Inline markup in a `{{ }}` resolved value: `detail: "{{ bold_text }}"` where `bold_text = "**foo**"` | The resolved string `"**foo**"` is treated as `InlineNode::Plain(Arc::from("**foo**"))` — NOT further parsed for inline markup. Inline markup is parsed from DSL source, not from dynamically resolved values (DIR-077-002 §3). No Bold node produced. |
| EC-012 | Inline markup inside `$...$` math span: `detail: "$**not bold**$"` | Math mode disables all inline markup. Result: `FieldValue::Inlines([InlineNode::Math(MathNode { latex: "**not bold**", display: false, ... })])`. The `**` characters are treated as LaTeX content, not markup delimiters. |
| EC-013 | SLIDE-LEVEL field value containing inline markup: `title: "**Bold Title**"` | OUT OF SCOPE for STORY-077. The parser produces `TemplateChunk::Bold(...)` (correct), but the eval stage does NOT call `chunks_to_inline_nodes` for slide-level fields in this story. The title field is evaluated through the existing path producing `Value::Str`. The temporary inconsistency is documented and resolved in STORY-081. |

## References and Intelligence

- **DIR-077-001** — parsing ownership split: STORY-078 owns the section-block parser; STORY-077 owns FieldValue::Inlines upgrade and eval routing.
- **DIR-077-001-A** — unrecognized-key warning is parse-time (STORY-078 owns); section TYPE validation is eval-time (STORY-077 owns).
- **DIR-077-002** (`.factory/cycles/STORY-077/inline-markup-directive.md`) — **BINDING**. Governs the inline-markup parser design, TemplateChunk extension, chunks_to_inline_nodes function, nesting rules, disambiguation rules, error recovery, and plugin-surface alignment. The canonical v1 syntax table (11 markup forms), two-phase architecture decision, and all 28 required Red Gate tests are specified there. This directive was the trigger for scope expansion (2026-06-02, human-authorized).
- **STORY-081** — follow-up story covering slide-level inline markup. Depends on this story. Blocks v1.0.
- **BC-3.02.002** — postcondition 8 (FieldValue::Inlines for section sub-blocks) is the primary behavioral contract for AC-001 and AC-002.
- **BC-1.14.003** — register routing (detail excluded from PPTX/preview) governs AC-005.
- **ADR-013** — comemo Hash compatibility requirement on all IR and TemplateChunk types (`#[derive(Hash, Eq, Clone, Debug)]` mandatory).

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-31 | story-writer | Initial story creation — spun out from STORY-035 per architect directive F-002 (section-level register routing requires SectionBlock IR extension not available in v1.0 Wave 4 without this story) |
| 1.1 | 2026-06-01 | story-writer | BC clause coverage closed by PO (BC-3.02.002 v1.2, BC-1.14.003 v1.2); AC→BC traces corrected (AC-001/AC-002 → PC8; AC-003 → PC7+inv1; AC-004 → EC-004; AC-005 → PC3+PC5; AC-006 → PC7+EC-001; AC-EC-001 → inv4+EC-005); BC-status comment removed; story marked implementation-ready (status: ready). |
| 1.2 | 2026-06-01 | story-writer | Per DIR-077-001 (architect directive): section-block PARSING is delivered by STORY-078 (STORY-027 decomposition gap). STORY-077 is BLOCKED until STORY-078 merges. Added STORY-078 to `depends_on`. STORY-077's parser extension task now reads: "Extend the STORY-078 parser to emit `FieldValue::Inlines` for `detail:` and `report:` sub-block content (instead of `FieldValue::Template`) inside `section <type>:` declarations." Additionally, `SECTION_REGISTER_KEYS` in `section.rs` must be `["report", "detail"]` — `"notes"` is dropped per DIR-077-001 §5 (notes register is presenter-only; sections have no slide canvas). This is a code correction resolving an inconsistency between `SECTION_REGISTER_KEYS` and `extract_section_register_content`; no BC amendment required. STORY-077's existing ACs and points (8) are unchanged. |
| 1.3 | 2026-06-01 | story-writer | Per DIR-077-001-A: (a) Unrecognized sub-block KEY warning is parse-time (STORY-078 ownership, Ruling 2) — STORY-077's eval must NOT re-emit it; AC-EC-001 updated to assert no duplicate eval diagnostic; eval task added to silently skip unrecognized keys. (b) Section TYPE validation is STORY-077's explicit eval-stage obligation (Ruling 3) — `eval_section_nodes` validates `SectionNode.kind` against SectionType plugin registry and emits fatal error for unknown types; added as explicit task and test case. Points unchanged (8). |
| 1.4 | 2026-06-01 | story-writer | EC-006 corrected per adversarial finding F-077-P1-004 (OBS). Previous text asserted per-iteration section nodes inside `@for` loops — behavior that contradicts DIR-077-001 §3.2/§3.7 (section is a top-level deck_parser production, not available in for/if bodies) and BC-3.02.002 EC-002 (section inside slide block is a parse error; by the same constraint section blocks are top-level only). EC-006 re-scoped to state the correct behavior: a `section <type>:` block inside a `@for` body either fails to parse or is not collected as a section node at eval; no per-iteration section nodes are produced. No other content changed. |
| 1.5 | 2026-06-02 | story-writer | **Scope expansion per DIR-077-002 + human authorization (2026-06-02).** AC-002 replaced with load-bearing test specification for the inline-markup parser (TemplateChunk extension + chunks_to_inline_nodes). Tasks reorganized into 4 phases: Phase 0 (IR extension), Phase 1 (TemplateChunk extension in slideforge-syntax — NEW), Phase 2 (chunks_to_inline_nodes in slideforge-eval — NEW), Phase 3 (register routing — unchanged), Phase 4 (28 Red Gate tests). File Structure updated with 2 new files for inline markup tests. Token budget bumped from ~16,700 to ~27,700 tokens. Points bumped from 8 to 13 (rationale: Phase 1 parser work adds 8 TemplateChunk variants + scanning pass + error accumulation across 2 files in slideforge-syntax; Phase 2 adds chunks_to_inline_nodes with 14 mapping cases + recursive traversal; 28 Red Gate tests are new obligation; the parser alone is ~5 points of new work on top of the original 8). Scope boundary added to AC-002: section sub-blocks ONLY; slide-level deferral documented as EC-013 and STORY-081. STORY-077 now blocks STORY-081. References section added citing DIR-077-002. EC-007 through EC-013 added covering inline-markup error cases. estimated_days bumped from 3 to 5. |
