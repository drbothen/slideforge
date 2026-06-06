---
document_type: behavioral-contract
level: L3
version: "1.4"
status: active
producer: product-owner
timestamp: 2026-06-05T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md, specs/architecture/adr/ADR-019-stage-2b-field-to-block-threading.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-02
capability: CAP-001
lifecycle_status: active
introduced: v1.0.0
modified:
  - "v1.1 — STORY-086 pass-5 adjudication (F-086-P5-CRIT-001, F-086-P5-MED-002): EC-004 mechanism corrected. The prior text stated 'the pre-layout validator emits W-A11-002' — that mechanism is unreachable because AltTextValidator::validate() is Shape-only per ADR-018 v1.2 Decision-3; charts are never validated pre-layout. Corrected: when both decorative: true and a non-empty alt are set, resolve_alt returns AltText::Decorative (decorative wins) AND emits W-A11-002 via tracing::warn!(code = \"W-A11-002\") at Stage-2b resolution time. PC-12 alt-resolution rule reworded to make the decorative-first ordering explicit and unambiguous. No behavioral change — decorative-first was already the canonical postcondition per PC-12."
  - "v1.2 — CORRUPTED (commit 608ec7b0): read types from develop-branch main checkout instead of STORY-086 worktree; erroneously stripped TextTag, TextBlock.tag, and AltText::Unspecified which all exist in the worktree. Superseded by v1.3."
  - "v1.3 — STORY-086 pass-7 recovery: revert erroneous v1.2 (608ec7b0) which stripped TextTag/TextBlock.tag/AltText::Unspecified after reading develop-branch types instead of the STORY-086 worktree. Reapply only the two legitimate fixes verified against worktree types: (1) PC-7 BulletItem struct shape corrected from nonexistent {text, level} fields to real {inlines: Vec<InlineNode>, children: Vec<BulletItem>, span: SourceSpan} per crates/slideforge-types/src/block.rs:84-91 (F-086-P7-MED-001 fix). (2) ImageSpec field corrected from src to path per crates/slideforge-types/src/specs.rs:324 — only the Rust struct field name changes; the DSL keyword the user writes remains src:. TextTag (block.rs:36-59), TextBlock.tag (block.rs:74), and AltText::Unspecified (specs.rs:162) are RETAINED as they exist in the worktree."
  - "v1.4 — F-086-P9-MED-001 fix: PC-9 ChartSpec produces-clause removed phantom `data_source: ...` field. Real ChartSpec struct (worktree crates/slideforge-types/src/specs.rs:170-191) has exactly four fields: chart_type, alt, decorative, span — no data_source. The postcondition now reflects the actual struct. Implementation in field_to_block.rs:184-189 already correctly omits data_source; this is a spec-to-code alignment (factual type correction). ImageSpec/DiagramSpec/BulletItem field shapes were already corrected in passes 6-7 and are confirmed correct."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.16.001: Post-Eval Field-to-Block Threading Pass Populates Slide.blocks from Resolved Field Values

## Description

After `eval_deck` produces a `Deck` with resolved `Slide.fields` and empty `Slide.blocks`,
the Stage 2b threading pass (`thread_fields_to_blocks`) reads those resolved field values
and writes typed `ContentBlock` entries into `Slide.blocks`. This pure, deterministic pass
is the single-responsibility bridge between the semantic IR (field-resolved `Deck`) and
the geometric IR (`LaidOutDeck`). It runs after all `{{ }}` expressions are resolved and
before brand load, layout, or any effectful stage.

This contract covers the complete fields→blocks mapping for all v1.0 slide content types:
text (title/body/subtitle), bullets, chart metadata, image metadata, and diagram metadata.
Shape blocks are explicitly excluded — they are handled by the layout shape pass and are
NOT populated by this threading pass.

## Preconditions

1. `eval_deck` has completed successfully and returned a `Deck`.
2. Every `Slide.fields` entry is in resolved form: `FieldValue::Literal(Value::*)`
   or `FieldValue::Inlines(Vec<InlineNode>)`. No `FieldValue::Template` remains.
3. All `{{ expr }}` interpolations have been evaluated. No unresolved expressions remain.
4. `Slide.blocks` is `vec![]` for all slides (invariant of `eval_deck` per `for_eval.rs:342`).
5. The `Deck` has passed no layout, brand, or export stages — this pass runs pre-brand
   (before Stage 3 in the pipeline ordering defined by ADR-019 Decision 1).

## Postconditions

### Text Content

1. For each `Slide` where `fields["title"]` is `FieldValue::Literal(Value::Str(s))` with
   `s.trim()` non-empty: a `ContentBlock::Text(TextBlock { inlines: vec![InlineNode::Plain(Arc::from(s.trim()))], tag: TextTag::Title, span: SourceSpan::default() })` is prepended to `Slide.blocks`.
2. For each `Slide` where `fields["title"]` is `FieldValue::Inlines(nodes)`: a
   `ContentBlock::Text(TextBlock { inlines: nodes, tag: TextTag::Title, span: SourceSpan::default() })` is prepended.
3. For each `Slide` where `fields["subtitle"]` is `FieldValue::Literal(Value::Str(s))`
   with `s.trim()` non-empty: a `ContentBlock::Text(TextBlock { inlines: vec![InlineNode::Plain(Arc::from(s.trim()))], tag: TextTag::Subtitle, span: SourceSpan::default() })` is appended after the title block (if present).
4. For each `Slide` where `fields["body"]` is `FieldValue::Literal(Value::Str(s))`
   with `s.trim()` non-empty: a `ContentBlock::Text(TextBlock { inlines: vec![InlineNode::Plain(Arc::from(s.trim()))], tag: TextTag::Body, span: SourceSpan::default() })` is appended.
5. For each `Slide` where `fields["body"]` is `FieldValue::Inlines(nodes)`: a
   `ContentBlock::Text(TextBlock { inlines: nodes, tag: TextTag::Body, span: SourceSpan::default() })` is appended.
6. Empty strings (after trim) produce NO ContentBlock. An empty `fields["title"]` is
   silently skipped; no ContentBlock::Text is emitted for it.

### Bullets

7. For each `Slide` where `fields["bullets"]` is `FieldValue::Literal(Value::List(items))`:
   a `ContentBlock::Bullets(vec![...])` is appended, with one
   `BulletItem { inlines: vec![InlineNode::Plain(Arc::from(item_str))], children: vec![], span: SourceSpan::default() }`
   per list entry. (`BulletItem` real fields per `crates/slideforge-types/src/block.rs:84-91`:
   `inlines: Vec<InlineNode>`, `children: Vec<BulletItem>`, `span: SourceSpan`. The fields
   `text` and `level` do NOT exist — F-086-P7-MED-001 fix.)
   (Note: in the parser AST — pre-eval — a bullet list literal is `FieldValue::List(Vec<FieldValue>)`, added by STORY-088 per D5. Eval converts this to `FieldValue::Literal(Value::List(...))` in `Slide.fields`. This threading pass therefore always sees the post-eval `FieldValue::Literal(Value::List(...))` form, consistent with PC-2.)
8. For each `Slide` where `fields["bullets"]` is `FieldValue::Inlines(nodes)`:
   a `ContentBlock::Bullets(...)` is appended with pre-parsed bullet nodes threaded as-is.

### Chart / Image / Diagram (Alt Resolution)

9. For each `Slide` with `slide_type == "chart"` (or any registered SlideType with `has_chart: true`):
   if `fields["chart_type"]` is `FieldValue::Literal(Value::Str(s))`, a `ContentBlock::Chart(ChartSpec { chart_type: Arc::from(s), alt: <alt>, decorative: <decorative>, span })` is appended, where `<alt>` is resolved per the alt-resolution rule (Postcondition 12).
   If `fields["chart_type"]` is absent, emit `tracing::warn!` and skip block construction.
10. For each `Slide` with `slide_type` in `["image", "screenshot", "bio"]`:
    if `fields["src"]` is `FieldValue::Literal(Value::Str(p))`, a `ContentBlock::Image(ImageSpec { path: Arc::from(p), alt: <alt>, decorative: <decorative>, span })` is appended.
    (`ImageSpec` real Rust struct field is `path: Arc<str>`, NOT `src` — per `crates/slideforge-types/src/specs.rs:324`. The DSL keyword the user writes in `.sf` files is still `src:`; the Rust struct field name is `path`.)
    If `fields["src"]` is absent, emit `tracing::warn!` and skip.
11. For each `Slide` with `slide_type == "diagram"`:
    if `fields["source"]` is `FieldValue::Literal(Value::Str(s))`, a `ContentBlock::Diagram(DiagramSpec { source: Arc::from(s), alt: <alt>, decorative: <decorative>, span })` is appended.
    If `fields["source"]` is absent, emit `tracing::warn!` and skip.
12. **Alt-resolution rule** (applied identically for Chart, Image, and Diagram):
    Decorative-first: when `decorative: true` is set, it takes unconditional precedence
    over any supplied `alt` string. This is the canonical ordering for Stage-2b media
    alt-resolution (BC-1.16.001 PC-12), which differs from the shape DSL path
    (BC-3.04.001 Invariant 11, alt-first) — those BCs govern non-overlapping domains
    and must NOT be cross-applied.
    - If `fields["decorative"] == FieldValue::Literal(Value::Bool(true))`:
      `alt = Some(AltText::Decorative)`, `decorative = true`.
      If `fields["alt"]` is ALSO set to a non-empty string, `resolve_alt` still returns
      `AltText::Decorative` (decorative wins) AND emits
      `tracing::warn!(code = "W-A11-002", ...)` at resolution time to flag the authoring
      conflict. The alt string is discarded. (See also: EC-004, W-A11-002 taxonomy entry.)
    - Else if `fields["alt"] == FieldValue::Literal(Value::Str(s))` and `s.trim()` is non-empty:
      `alt = Some(AltText::Provided(Arc::from(s.trim())))`, `decorative = false`.
    - Else (neither present, or `alt` is empty/whitespace-only, or `decorative` is false/absent):
      `alt = None`, `decorative = false`.
    `None` on `ContentBlock.alt` is later mapped to `AltText::Unspecified` by
    `thread_media_alt_into_frames` in layout (ADR-019 Decision 5.2). `AltText::Unspecified`
    on a frame triggers E-A11-001 in `validate_post_layout` (ADR-019 Decision 5.3).
    (`AltText` has three variants: `Provided(Arc<str>)`, `Decorative`, `Unspecified` —
    per `crates/slideforge-types/src/specs.rs:150-163`.)

### Block Ordering and Shape Exclusion

13. Within a single slide, `Slide.blocks` is populated in canonical order:
    1. Title block (if applicable)
    2. Subtitle block (if applicable)
    3. Body block (if applicable)
    4. Bullets block (if applicable)
    5. Chart / Image / Diagram block (if applicable, at most one per slide)
14. Shape blocks are NOT populated by this pass. Shape blocks originate from `SlideNode.shapes`
    and are threaded by the existing shape pass in `layout::run`. Stage 2b does NOT read,
    modify, or append shape blocks.

### Purity

15. `thread_fields_to_blocks` performs no I/O, no filesystem access, no network calls,
    and has no global mutable state. Its output is fully determined by its input `Deck`.
    The function is Kani-amenable and property-based testable.

## Invariants

1. **Pure pass.** No side effects beyond mutating `Slide.blocks` in the input `Deck`.
   No I/O; deterministic output. (ADR-019 Decision 7)
2. **Canonical block order.** Within each slide, blocks appear in the order: title →
   subtitle → body → bullets → chart/image/diagram. No other ordering is valid.
3. **Empty strings skipped.** A field that resolves to an empty string (or whitespace-only
   string after trim) produces no `ContentBlock`. Zero-content slides remain empty.
4. **Shape blocks not touched.** The function MUST NOT read `Slide.shapes`, produce
   `ContentBlock::Shape`, or mutate any pre-existing blocks. Shape block population is
   exclusively owned by `layout::run`. (ADR-019 Decision 3.6)
5. **At most one media block per slide.** A slide MUST NOT have more than one
   `ContentBlock::Chart`, `ContentBlock::Image`, or `ContentBlock::Diagram` in its blocks
   list (slide type semantics guarantee a slide is one type). If field values are
   inconsistent (e.g., a chart slide also has a `src` field), only the chart block is
   produced.
6. **Alt-resolution is total and exhaustive.** Every Chart/Image/Diagram ContentBlock
   produced by this pass has `alt` set to exactly one of: `Some(AltText::Provided(_))`,
   `Some(AltText::Decorative)`, or `None`. The value `Some(AltText::Unspecified)` is
   NEVER produced by this pass — `Unspecified` is exclusively assigned by layout's
   `thread_media_alt_into_frames` (ADR-019 Decision 4). (`AltText` has three variants:
   `Provided`, `Decorative`, `Unspecified` — per specs.rs:150-163.)
7. **Idempotency guard.** If `Slide.blocks` is already non-empty (e.g., from a future
   pipeline change), the function appends to it rather than replacing. In the current
   pipeline, `eval_deck` always produces `Slide.blocks = vec![]`, so this guard is a
   no-op.
8. **All `AltText`, `ContentBlock`, `TextBlock`, `BulletItem` types implement
   `Hash + Eq + Clone`.** Comemo compatibility is maintained. No `f64` coordinates.
   (CLAUDE.md hash+eq+clone rule; ADR-019 Decision 7)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `fields["title"]` is `Value::Str("")` (empty string) | No `ContentBlock::Text` for title. Empty strings are silently skipped per Postcondition 6. |
| EC-002 | `fields["title"]` is `Value::Str("  ")` (whitespace-only) | No `ContentBlock::Text` for title. Whitespace-only after trim is treated as empty. |
| EC-003 | `fields["alt"]` is `Value::Str("")` on a chart slide | `alt = None`. The empty-string alt is NOT treated as `AltText::Provided`. Layout will produce `AltText::Unspecified`; E-A11-001 fires in strict mode. |
| EC-004 | `fields["decorative"] = Value::Bool(true)` AND `fields["alt"] = Value::Str("desc")` on same chart slide | `alt = Some(AltText::Decorative)`, `decorative = true`. Decorative takes precedence in Stage 2b (PC-12). `resolve_alt` returns `AltText::Decorative` and emits `tracing::warn!(code = "W-A11-002")` at resolution time; the alt string "desc" is discarded. The pre-layout `AltTextValidator::validate()` does NOT emit W-A11-002 for charts/images/diagrams — that validator is Shape-only per ADR-018 v1.2 Decision-3. W-A11-002 is a Stage-2b resolution-time warning, not a validation-stage finding. (F-086-P5-MED-002 mechanism correction, 2026-06-06) |
| EC-005 | Chart slide with no `fields["chart_type"]` | `tracing::warn!` emitted. No `ContentBlock::Chart` produced. Layout region for the chart slide will carry `AltText::Unspecified` structural placeholder; E-A11-001 fires in strict mode if no alt was provided. |
| EC-006 | Image slide with no `fields["src"]` | `tracing::warn!` emitted. No `ContentBlock::Image` produced. Same Unspecified outcome as EC-005. (`fields["src"]` is the DSL field name; the Rust struct field is `ImageSpec::path`.) |
| EC-007 | Slide with all three of title, body, and bullets fields | Three blocks produced: `[ContentBlock::Text(Title), ContentBlock::Text(Body), ContentBlock::Bullets(...)]` in canonical order. |
| EC-008 | `fields["bullets"]` is `Value::List([])` (empty list) | `ContentBlock::Bullets(vec![])` is still produced. An empty bullets block is valid — it is the layout engine's job to handle zero-item bullet lists. |
| EC-009 | Slide type not matching any known media type (e.g., `title`, `content`, `toc`) with no `fields["src"]`, `fields["chart_type"]`, or `fields["source"]` | Only text/bullet blocks are produced (per the slide's declared fields). No media block is produced. No warn emitted. |
| EC-010 | `thread_fields_to_blocks` called on a deck with zero slides | Function is a no-op; `Deck.slides` remains empty. No error. |
| EC-011 | Alt provided as `FieldValue::Inlines(...)` (rich-text alt field) | The threading pass reads only `FieldValue::Literal(Value::Str(_))` for alt resolution. A `FieldValue::Inlines` alt field is treated as absent; `alt = None`. Authors must supply plain-text alt strings. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck with one `title` slide: `fields["title"] = FieldValue::Literal(Value::Str("My Title"))` | `Slide.blocks == [ContentBlock::Text(TextBlock { inlines: vec![InlineNode::Plain(Arc::from("My Title"))], tag: TextTag::Title, span: SourceSpan::default() })]` | happy-path (text) |
| Deck with one `content` slide: `fields["title"] = Str("H1"), fields["body"] = Str("Body text")` | `Slide.blocks == [ContentBlock::Text(TextBlock { inlines: vec![Plain("H1")], tag: TextTag::Title, span }), ContentBlock::Text(TextBlock { inlines: vec![Plain("Body text")], tag: TextTag::Body, span })]` — title first, body second | happy-path (canonical order) |
| Deck with `chart` slide: `fields["chart_type"] = Str("bar"), fields["alt"] = Str("Q3 revenue")` | `Slide.blocks == [ContentBlock::Chart(ChartSpec { chart_type: Arc::from("bar"), alt: Some(AltText::Provided(Arc::from("Q3 revenue"))), decorative: false, span })]` | happy-path (chart + alt) |
| Deck with `chart` slide: `fields["chart_type"] = Str("bar"), fields["decorative"] = Bool(true)` | `Slide.blocks == [ContentBlock::Chart(ChartSpec { chart_type: Arc::from("bar"), alt: Some(AltText::Decorative), decorative: true, span })]` | edge-case (decorative chart) |
| Deck with `chart` slide: `fields["chart_type"] = Str("pie")`, no `fields["alt"]`, no `fields["decorative"]` | `Slide.blocks == [ContentBlock::Chart(ChartSpec { chart_type: Arc::from("pie"), alt: None, decorative: false, span })]`; layout maps `None → AltText::Unspecified`; strict mode yields E-A11-001 | error path (missing alt) |
| Deck with `image` slide: `fields["src"] = Str("photo.png"), fields["alt"] = Str("Team photo")` | `Slide.blocks == [ContentBlock::Image(ImageSpec { path: Arc::from("photo.png"), alt: Some(AltText::Provided(Arc::from("Team photo"))), decorative: false, span })]` (`ImageSpec::path`, not `src`) | happy-path (image + alt) |
| Deck with `diagram` slide: `fields["source"] = Str("graph TD; A-->B")`, no alt | `Slide.blocks == [ContentBlock::Diagram(DiagramSpec { source: Arc::from("graph TD; A-->B"), alt: None, decorative: false, span })]`; layout maps `None → AltText::Unspecified`; E-A11-001 in strict mode | error path (missing alt) |
| Deck with slide: `fields["title"] = Str("")` | `Slide.blocks == []` — empty title string produces no block | edge-case (empty string) |
| Deck with `title` slide: `fields["title"] = Str("T"), fields["bullets"] = Value::List([Str("A"), Str("B")])` | `Slide.blocks == [ContentBlock::Text(TextBlock { inlines: vec![Plain("T")], tag: TextTag::Title, span }), ContentBlock::Bullets(vec![BulletItem { inlines: vec![InlineNode::Plain(Arc::from("A"))], children: vec![], span: SourceSpan::default() }, BulletItem { inlines: vec![InlineNode::Plain(Arc::from("B"))], children: vec![], span: SourceSpan::default() }])]` | happy-path (title + bullets, concrete BulletItem struct shape) |
| `thread_fields_to_blocks(&mut deck)` called with shape block already in `slide.shapes` | `Slide.blocks` contains only text/media ContentBlocks; no `ContentBlock::Shape` in result; `slide.shapes` unchanged | invariant (shape exclusion) |
| Deck with `chart` slide: `fields["alt"] = Str("  ")` (whitespace-only) | `alt = None` — whitespace-only treated as absent; `AltText::Unspecified` at layout; E-A11-001 in strict mode | edge-case (whitespace alt) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | For any `Deck` output of `eval_deck`, after `thread_fields_to_blocks`, every slide with `slide_type == "chart"` and non-empty `fields["alt"]` has exactly one `ContentBlock::Chart` with `alt == Some(AltText::Provided(_))` | unit test + Kani (ADR-019 Decision 7 — designated Kani-amenable) |
| VP-TBD | `thread_fields_to_blocks` is pure: calling it twice on the same `Deck` (resetting `blocks` between calls) produces identical results | unit test (determinism) |
| VP-TBD | Block ordering invariant: title always precedes subtitle, subtitle precedes body, body precedes bullets, bullets precede media blocks | unit test |
| VP-TBD | No `ContentBlock::Shape` in `Slide.blocks` after `thread_fields_to_blocks` | unit test |
| VP-TBD | `alt = None` when both `fields["alt"]` is absent and `fields["decorative"]` is absent | unit test + Kani (alt-resolution rule exhaustiveness) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 |
| Capability Anchor Justification | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 — the field-to-block threading pass is the final step of the semantic compilation phase that begins with DSL parsing; it converts parsed-and-evaluated field values into typed ContentBlock semantic nodes, completing the parse-evaluate-thread pipeline that produces the semantic IR (Deck with populated blocks). This is a late stage of CAP-001's "parse .sf source into typed AST" responsibility, not layout or brand. |
| L2 Domain Invariants | DI-001 (alt text required on visual elements — this pass establishes the `alt` field on ContentBlocks that the validator checks), DI-009 (Deck→LaidOutDeck pipeline — Stage 2b is a new pipeline stage in this chain) |
| Architecture Module | slideforge-eval crate (SS-02) — new module `field_to_block.rs`; public API: `thread_fields_to_blocks(deck: &mut Deck)` |
| Architecture Decision | ADR-019 (Stage 2b — Post-Eval Field-to-Block Threading Pass) |
| Stories | (filled by story-writer — Story A: `slide-field-to-block-threading`) |

## Related BCs

- BC-5.01.001 — depends on (this pass establishes `ContentBlock.alt` values that AltTextValidator ultimately checks)
- BC-5.02.001 — depends on (the AltTextValidator's post-layout pass relies on `Slide.blocks` being populated by this threading pass)
- BC-3.04.001 — related to (shape blocks are explicitly NOT in this pass's scope; shape threading remains in layout::run)
- BC-4.01.001 — downstream dependency (PPTX exporter's TextTag title-placeholder routing contract requires `TextTag::Title` from this threading pass as upstream input; BC-4.01.001 v1.2 Postconditions 9–12)
- BC-4.02.001 — downstream dependency (DOCX exporter's TextTag→Heading1 routing contract requires `TextTag::Title` from this threading pass as upstream input; BC-4.02.001 v1.2 Postconditions 8–11)

## Architecture Anchors

- `specs/architecture/adr/ADR-019-stage-2b-field-to-block-threading.md` — definitive contract for Stage 2b
- `specs/architecture/ARCH-INDEX.md` SS-02 (Evaluator, slideforge-eval)

## Story Anchor

Story A: `slide-field-to-block-threading` (to be filled by story-writer)

## VP Anchors

(filled after VP creation)
