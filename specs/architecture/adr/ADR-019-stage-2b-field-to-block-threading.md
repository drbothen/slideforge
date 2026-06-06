---
document_type: adr
adr_id: ADR-019
title: "Stage 2b — post-eval field-to-block threading pass in slideforge-eval"
status: accepted
date: 2026-06-05
accepted_date: 2026-06-05
version: "1.4"
subsystems_affected: [SS-02, SS-03, SS-05, SS-15]
supersedes: null
superseded_by: null
amends: ADR-016
context: >
  Wave 4 integration gate (content-threading assessment 2026-06-05) confirmed that
  Slide.blocks is always vec![] after eval. Exporters, the alt-text validator, and
  the layout inline-content pass all silently produce empty output as a result.
  Human authorized Stage 2b as a new pure post-eval pass (2026-06-05). This ADR
  codifies the precise seam contract, the fields→blocks mapping, the AltText::Unspecified
  type addition, and the a11y state machine. Closes BLK-002, F-G3-CRIT-001,
  F-G3-HIGH-001, F-G3-HIGH-002. Bundled a11y fix (AltText::Unspecified) authorized
  simultaneously.
human_gate_required: false
human_gate_resolved: >
  Human authorized Stage 2b seam (option b, post-eval pure pass) on 2026-06-05.
  Human authorized AltText::Unspecified bundled into Story A on 2026-06-05.
  Human authorized status/progress_bar/weighted_composite as v1.0 in-scope types
  on 2026-06-05.
traces_to: ARCH-INDEX.md
---

# ADR-019: Stage 2b — Post-Eval Field-to-Block Threading Pass

## Status

**ACCEPTED.** Human authorized Stage 2b (option b, post-eval pure pass) and the
bundled `AltText::Unspecified` fix on 2026-06-05. Implementation assigned to
Story A (`slide-field-to-block-threading`).

## Amendment Log

| Date | Version | Author | Change |
|------|---------|--------|--------|
| 2026-06-05 | v1.0 | architect | Initial ADR. Stage 2b seam contract, fields→blocks mapping, AltText::Unspecified state machine, decorative handling, purity classification, comemo compatibility. |
| 2026-06-06 | v1.1 | architect | Amendment A: D1 (TextTag→FrameContent layout mapping; exporters unchanged) + D5 (FieldValue::List parser model). See wave4-expanded-scope-uncertainty-resolution.md for full resolution. |
| 2026-06-06 | v1.2 | architect | Amendment B (STORY-086 pass-5 adjudication F-086-P5-CRIT-001): Decision 4.1 conflict path corrected from alt-first (BC-3.04.001 Inv-11) to decorative-first (BC-1.16.001 PC-12). BC-3.04.001 Inv-11 governs shape DSL path only; media threading at Stage 2b is governed exclusively by BC-1.16.001. W-A11-002 emission site corrected from "pre-layout validator" to "resolve_alt via tracing::warn!" — AltTextValidator.validate() is Shape-only per ADR-018 v1.2 and cannot emit W-A11-002 for charts/images/diagrams. |
| 2026-06-06 | v1.3 | architect | Amendment C (STORY-086 pass-6 adjudication F-086-P6-MED-001): §3.1 table and §3.3 alt-rule corrected to TRIMMED storage. `InlineNode::Plain(s)` → `InlineNode::Plain(Arc::from(s.trim()))` for title/subtitle/body; `AltText::Provided(Arc::from(s))` → `AltText::Provided(Arc::from(s.trim()))` for alt. Function-level doc comment (Decision 2 inline example) updated likewise. BC-1.16.001 PC-1/PC-4/PC-12 is the authoritative contract (contract semantics per CLAUDE.md precedence rule 1); ADR-019 is brought into alignment. |
| 2026-06-06 | v1.4 | architect | Amendment D (STORY-086 pass-9 adjudication F-086-P9-MED-001): §3.3 ChartSpec construction example corrected — removed phantom `data_source: ...` field. Real `ChartSpec` in `slideforge-types/src/specs.rs` has exactly four fields: `chart_type`, `alt`, `decorative`, `span`. Sibling-site sweep (TD-VSDD-060) confirmed single occurrence. |

---

## Context

### The Gap

`slideforge-eval` produces a `Deck` whose every `Slide.blocks` is `vec![]`
(for_eval.rs:342). This was an intentional Wave 2 scope boundary: the evaluator's
contract is to resolve field expressions, not to construct content geometry
(ADR-005). The comment at for_eval.rs:336–341 records this deferral.

The consequence is total: title text, body text, bullets, chart metadata, image
metadata, and diagram metadata never reach the layout engine or exporters. Outputs
are structurally valid but content-empty. Three exporters (PPTX, PDF, DOCX) and
the post-layout a11y validator (ADR-018) are all blocked by the same missing step.

### Why Now

ADR-018 (post-layout validation pass) is implemented and accepted, but remains
incapable of distinguishing "author supplied alt text that was threaded" from
"structural placeholder that was never overwritten." The root problem is that
`thread_media_alt_into_frames` (layout.rs) has nothing to thread because
`Slide.blocks` is always empty. ADR-018 Decision 3 allocates alt-text checking
to `validate_post_layout` — which is correct — but the data it needs to validate
does not yet exist.

### Three Options Considered

**(a) Inside `eval_slide_node`:** The evaluator constructs `ContentBlock::*` from
resolved `Slide.fields` scalar values during the same pass that produces `Slide`.
Rejected: violates ADR-005. The evaluator must not produce content geometry; its
contract is field resolution only. Constructing typed ContentBlock objects from
scalar field values would blur the eval/layout boundary on which purity,
Kani-amenability, and the two-IR model depend.

**(b) A new post-eval, pre-layout pass — "Stage 2b" — implemented in
`slideforge-eval` as a separate module.** This pass runs after all `{{ }}`
expressions are resolved and `eval_deck` returns its `Deck`. It reads
`Slide.fields`, writes `Slide.blocks`, and is 100% pure (no I/O, no external
state, deterministic). Compatible with Kani bounded-model checking. **Selected.**

**(c) Inside `layout::run`:** The layout stage constructs ContentBlocks from
`Slide.fields` as a preliminary step before its shape pass. Rejected: layout's
contract is to transform ContentBlocks into geometric frames, not to create
ContentBlocks from raw fields. Conflating field-parsing responsibilities with
geometry assignment violates separation of concerns and would force the layout
crate to re-implement field semantics already encoded in the evaluator.

Human selected option (b) on 2026-06-05.

---

## Decisions

### Decision 1: Stage 2b Placement in the Pipeline

Stage 2b is a **new pipeline stage** inserted between Stage 2 (eval) and Stage 3
(brand load) in `build_inner` (`slideforge/src/lib.rs`). The `build_inner` doc
comment (currently enumerates Stages 1–7 per ADR-016/ADR-018) is amended to
include Stage 2b. The pipeline ordering becomes:

```
Stage 1: assemble plugin registry
Stage 2: parse   (slideforge-syntax)           → DeckNode (AST)
Stage 2a: eval   (slideforge-eval::eval_deck)  → Deck (fields resolved, blocks empty)
Stage 2b: field-to-block threading             → Deck (blocks populated)  ← NEW (ADR-019)
Stage 3: brand load (slideforge-brand)
Stage 4: inject lang default (slideforge-validate)
Stage 5: validate pre-layout (all Validators, &Deck)
Stage 6: layout  (slideforge-layout)           → LaidOutDeck
Stage 6b: validate post-layout (all Validators, &LaidOutDeck)  (ADR-018)
Stage 7: export
```

NOTE: ADR-016 Decision 3 originally numbered the pipeline with brand as Stage 2,
parse as Stage 3, eval as Stage 4, validate as Stage 5, layout as Stage 6, and
export as Stage 7 (matching the in-code comment numbering in lib.rs). This ADR
renumbers conceptually to make the eval→threading→layout boundary explicit.
The in-code comment in `build_inner` MUST be updated to match the above sequence.
The old ADR-016 stage numbering is superseded for this file by this ADR's numbering.

#### Placement rationale

Stage 2b runs after eval for two reasons:
1. It requires fully resolved `FieldValue::Literal(Value::Str(_))` values —
   interpolations and `{{ expr }}` expressions must be resolved before
   ContentBlocks can carry them.
2. It does NOT require brand, layout, or any external data. Running it
   before brand load keeps the pass pure and avoids any implicit ordering
   dependency on effectful stages.

### Decision 2: Stage 2b Function Signature and Module Location

Stage 2b is implemented as a public function in a new module:

```
crates/slideforge-eval/src/field_to_block.rs
```

Public signature (exported from `slideforge-eval::lib.rs`):

```rust
/// Post-eval field-to-block threading pass (Stage 2b, ADR-019).
///
/// Reads resolved field values from every `Slide.fields` in `deck` and
/// populates `Slide.blocks` with typed [`ContentBlock`] entries derived
/// from those fields. This pass runs AFTER `eval_deck` completes and BEFORE
/// `layout::run` — it is the single-responsibility bridge between the
/// semantic IR (field-resolved `Deck`) and the geometric IR (`LaidOutDeck`).
///
/// # Purity contract
///
/// This function is **pure** in the architectural sense: it performs no I/O,
/// no filesystem access, no network calls, and has no global mutable state.
/// Its output is fully determined by its input. This makes it amenable to
/// Kani bounded-model checking and property-based testing.
///
/// # AltText contract
///
/// When constructing `ContentBlock::Chart`, `ContentBlock::Image`, or
/// `ContentBlock::Diagram`, the threading pass uses the following alt-resolution
/// precedence (see Decision 4, AltText state machine):
///
/// 1. `Slide.fields["decorative"] == Value::Bool(true)` → `ContentBlock.alt = Some(AltText::Decorative)`
/// 2. `Slide.fields["alt"] == Value::Str(s)` (non-empty, non-whitespace) → `ContentBlock.alt = Some(AltText::Provided(Arc::from(s.trim())))`
/// 3. Neither present → `ContentBlock.alt = None`
///
/// The layout `thread_media_alt_into_frames` function then maps `None` to
/// `AltText::Unspecified` on the resulting frame (Decision 5).
///
/// # Idempotency
///
/// If `Slide.blocks` is already non-empty for a slide, this function
/// appends to it rather than replacing it. In practice, `eval_deck`
/// always produces `Slide.blocks = vec![]`, so this is a no-op guard.
pub fn thread_fields_to_blocks(deck: &mut Deck) {
    // ...
}
```

`thread_fields_to_blocks` is the ONLY public API exported by `field_to_block.rs`.
The `build_inner` function in `slideforge/src/lib.rs` calls it directly after
`eval_deck` returns, passing `&mut deck`.

### Decision 3: Fields→Blocks Mapping Contract

The following table defines the complete mapping for v1.0. Every slide type in
the registered slide type set MUST be covered by at least one rule. Rules are
evaluated in the order listed; the first matching rule applies.

#### 3.1 Text Content (title, body, subtitle)

| Slide.fields key | Value type | Produces | Notes |
|-----------------|------------|---------|-------|
| `"title"` | `FieldValue::Literal(Value::Str(s))` | `ContentBlock::Text(TextBlock { inlines: [InlineNode::Plain(Arc::from(s.trim()))], tag: TextTag::Title })` | Prepended to blocks before body; leading/trailing whitespace stripped per BC-1.16.001 PC-1 |
| `"body"` | `FieldValue::Literal(Value::Str(s))` | `ContentBlock::Text(TextBlock { inlines: [InlineNode::Plain(Arc::from(s.trim()))], tag: TextTag::Body })` | Leading/trailing whitespace stripped per BC-1.16.001 PC-4 |
| `"subtitle"` | `FieldValue::Literal(Value::Str(s))` | `ContentBlock::Text(TextBlock { inlines: [InlineNode::Plain(Arc::from(s.trim()))], tag: TextTag::Subtitle })` | Only for slide types that use subtitle (e.g., `title`); whitespace stripped |
| `"body"` | `FieldValue::Inlines(nodes)` | `ContentBlock::Text(TextBlock { inlines: nodes, tag: TextTag::Body })` | Rich-text body |
| `"title"` | `FieldValue::Inlines(nodes)` | `ContentBlock::Text(TextBlock { inlines: nodes, tag: TextTag::Title })` | Rich-text title |

Empty strings (after trim): skip — do not emit a ContentBlock for an empty-string title or body.

#### 3.2 Bullets

| Slide.fields key | Value type | Produces | Notes |
|-----------------|------------|---------|-------|
| `"bullets"` | `FieldValue::Literal(Value::List(items))` | `ContentBlock::Bullets(vec![BulletItem { ... }])` | One `BulletItem` per list entry |
| `"bullets"` | `FieldValue::Inlines(nodes)` | `ContentBlock::Bullets(...)` | Pre-parsed bullet list from parser |

Bullet items are constructed as `BulletItem { text: [InlineNode::Plain(item_str)], level: 0, span: SourceSpan::default() }` from string list items. Rich-text bullets from `Inlines` are threaded as-is.

#### 3.3 Chart

Applies when `slide.slide_type == "chart"` (or any slide type registered in the
`ChartRenderer` plugin surface that declares `has_chart: true` in its `SlideTypeSpec`).

| Slide.fields key | Value type | Produces | Notes |
|-----------------|------------|---------|-------|
| `"chart_type"` | `FieldValue::Literal(Value::Str(s))` | `ChartSpec { chart_type: Arc::from(s), alt: <resolved per 3.3.alt>, decorative: <resolved per 3.3.alt>, span }` | chart_type is required for chart slides |

Alt resolution for chart (section 3.3.alt):
- `fields["decorative"] == Value::Bool(true)` → `alt = Some(AltText::Decorative)`, `decorative = true`
- `fields["alt"] == Value::Str(s)` (non-empty after trim) → `alt = Some(AltText::Provided(Arc::from(s.trim())))`, `decorative = false`
- Neither → `alt = None`, `decorative = false`

If `chart_type` is absent for a chart slide, emit `tracing::warn!` and skip block
construction (layout will produce an `Unspecified`-alt frame via `regions.rs`).

#### 3.4 Image

Applies when `slide.slide_type` is in: `image`, `screenshot`, `bio`.

| Slide.fields key | Value type | Produces | Notes |
|-----------------|------------|---------|-------|
| `"src"` | `FieldValue::Literal(Value::Str(path))` | `ImageSpec { src: Arc::from(path), alt: <resolved per 3.4.alt>, decorative: <resolved>, span }` | |

Alt resolution: identical to chart (3.3.alt) but reads `fields["alt"]` and `fields["decorative"]`.

If `src` is absent for an image slide, emit `tracing::warn!` and skip (layout
produces `Unspecified`-alt frame via `regions.rs`).

#### 3.5 Diagram

Applies when `slide.slide_type == "diagram"`.

| Slide.fields key | Value type | Produces | Notes |
|-----------------|------------|---------|-------|
| `"source"` | `FieldValue::Literal(Value::Str(s))` | `DiagramSpec { source: Arc::from(s), alt: <resolved per 3.4.alt>, decorative: <resolved>, span }` | |

Alt resolution: same pattern as 3.3.alt.

#### 3.6 Block Ordering

Within a single slide, `Slide.blocks` is populated in this canonical order:
1. Title block (if applicable)
2. Subtitle block (if applicable)
3. Body block (if applicable)
4. Bullets block (if applicable)
5. Chart / Image / Diagram block (if applicable, one per slide)
6. Shape blocks (existing — already threaded by layout shape pass; not touched here)

Shape blocks are NOT populated by Stage 2b — shape blocks come from `SlideNode.shapes`
via the existing shape pass in `layout::run`. Stage 2b does not modify or append
shape blocks.

### Decision 4: AltText::Unspecified — New Variant in slideforge-types

A third variant is added to the `AltText` enum in
`crates/slideforge-types/src/specs.rs`:

```rust
/// The alt text state of a visual element.
///
/// - `Provided(s)` — author supplied `alt "..."` with non-empty, non-whitespace text.
/// - `Decorative` — author explicitly wrote `decorative: true`; element has no
///   accessible description by design.
/// - `Unspecified` — no author alt-text data has been threaded into this frame yet.
///   Used exclusively by `regions.rs` for structural placeholders and by
///   `thread_media_alt_into_frames` when a ContentBlock has `alt = None`.
///   MUST NOT be used to mean "the author chose no alt text."
///   The post-layout validator (`AltTextValidator::validate_post_layout`) treats
///   `Unspecified` as a missing-alt error (E-A11-001) in strict mode.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AltText {
    /// Non-empty, non-whitespace alt text provided by the author.
    Provided(Arc<str>),
    /// Element explicitly marked `decorative: true`.
    Decorative,
    /// Structural pipeline placeholder: no author alt-text data has been threaded.
    /// Produced by `regions.rs` frame construction and by `thread_media_alt_into_frames`
    /// when the corresponding ContentBlock has `alt = None`.
    Unspecified,
}
```

All existing `AltText` match sites across the workspace MUST be updated to handle
the new variant. The sibling-site sweep (TD-VSDD-060) is MANDATORY before committing.

Derive list unchanged: `Debug + Clone + PartialEq + Eq + Hash` — comemo-compatible.

#### 4.1 AltText State Machine

The following state machine documents all transitions for a visual element's alt
text as it moves through the pipeline. Precisely one of these paths applies to
every `FrameContent::Chart/Image/Diagram` frame in `LaidOutDeck`:

```
Author writes `alt "..."` (non-empty)
  → Stage 2b: ContentBlock.alt = Some(AltText::Provided(s))
  → thread_media_alt_into_frames: frame.alt = AltText::Provided(s)
  → validate_post_layout: Provided(_) → VALID, no diagnostic

Author writes `decorative: true`
  → Stage 2b: ContentBlock.alt = Some(AltText::Decorative)
  → thread_media_alt_into_frames: frame.alt = AltText::Decorative
  → validate_post_layout: Decorative → VALID (author opt-out)

Author writes nothing (no alt, no decorative)
  → Stage 2b: ContentBlock.alt = None
  → thread_media_alt_into_frames: alt=None fallback → frame.alt = AltText::Unspecified
  → validate_post_layout: Unspecified → E-A11-001 (strict: error; non-strict: warning)

No ContentBlock produced (field absent for slide type)
  → thread_media_alt_into_frames finds no block → no frame overwrite
  → regions.rs structural placeholder carries AltText::Unspecified (Decision 5)
  → validate_post_layout: Unspecified → E-A11-001
```

Author writes both `alt "..."` AND `decorative: true` (conflict):
  → Stage 2b: `decorative` takes precedence per BC-1.16.001 PC-12.
    ContentBlock.alt = Some(AltText::Decorative), decorative = true.
    resolve_alt emits tracing::warn!(code = "W-A11-002") at detection time in
    the threading pass (NOT via the pre-layout validator — AltTextValidator.validate()
    is restricted to ContentBlock::Shape per ADR-018 v1.2 Decision 3 amendment).
  → thread_media_alt_into_frames: frame.alt = AltText::Decorative
  → validate_post_layout: Decorative → VALID (author opt-out; W-A11-002 already emitted).
  Note: BC-3.04.001 Invariant 11 (alt-wins for shapes) governs the shape DSL path only
  and does NOT apply to this Stage 2b media threading path. See architect adjudication
  STORY-086 pass 5 (2026-06-06) for full domain-scoping analysis.

### Decision 5: regions.rs and thread_media_alt_into_frames Updates

#### 5.1 regions.rs — Structural Placeholder Change

All five occurrences in `crates/slideforge-layout/src/regions.rs` where
`FrameContent::Chart/Image/Diagram { alt: AltText::Decorative }` is used as a
structural placeholder MUST be changed to `alt: AltText::Unspecified`.

This is the precise change:

```rust
// Before (current — 5 sites in regions.rs):
content: FrameContent::Chart { alt: AltText::Decorative }
content: FrameContent::Image { alt: AltText::Decorative }
content: FrameContent::Diagram { alt: ..., alt: AltText::Decorative }

// After (ADR-019):
content: FrameContent::Chart { alt: AltText::Unspecified }
content: FrameContent::Image { alt: AltText::Unspecified }
content: FrameContent::Diagram { alt: ..., alt: AltText::Unspecified }
```

Comments at those sites are updated from "safe default" language to "structural
placeholder — overwritten by thread_media_alt_into_frames when Stage 2b populates
Slide.blocks; if never overwritten, validate_post_layout emits E-A11-001."

#### 5.2 thread_media_alt_into_frames — Fallback Change

In `crates/slideforge-layout/src/layout.rs`, the three fallback arms in
`thread_media_alt_into_frames` that currently produce `AltText::Decorative` when
`ContentBlock.alt == None` MUST be changed to produce `AltText::Unspecified`:

```rust
// Before (current — 3 arms in thread_media_alt_into_frames):
let resolved_alt = block.spec.alt.clone().unwrap_or_else(|| {
    tracing::warn!("... falling back to AltText::Decorative at layout time");
    AltText::Decorative
});

// After (ADR-019):
let resolved_alt = block.spec.alt.clone().unwrap_or_else(|| {
    tracing::warn!("... falling back to AltText::Unspecified (no author alt threaded)");
    AltText::Unspecified
});
```

The tracing::warn message is updated to accurately describe the state: "no author
alt text was threaded into this ContentBlock — emitting AltText::Unspecified;
validate_post_layout will emit E-A11-001 in strict mode."

#### 5.3 validate_post_layout — Match Arms Update

In `crates/slideforge-validate/src/alt_text.rs`, the `validate_post_layout` match
arms on `FrameContent::Chart`, `FrameContent::Image`, `FrameContent::Diagram` are
updated:

```rust
// Before (current — fires E-A11-001 on Decorative):
AltText::Decorative => { emit E-A11-001 }

// After (ADR-019):
AltText::Unspecified => { emit E-A11-001 }   // missing alt — no author data
AltText::Decorative  => { /* valid — author opt-out */ }
AltText::Provided(_) => { /* valid */ }
```

This change is the fix for F-G3-CRIT-001: the strict gate now distinguishes between
"author said decorative" (valid) and "pipeline never received author alt data"
(error).

#### 5.4 block.rs — produces_structure_group Update

In `crates/slideforge-types/src/block.rs`, `produces_structure_group()` currently
returns `false` for `Image(alt: Decorative | None)` and similar. The new
`Unspecified` variant must be handled:

```rust
// Unspecified has the same structural semantics as Decorative for the
// purposes of PDF structure group generation: neither produces a structure group
// (because both represent "no authored alt text data").
ContentBlock::Image(spec) => matches!(&spec.alt, Some(AltText::Provided(_))),
ContentBlock::Chart(spec) => matches!(spec.alt.as_ref(), Some(AltText::Provided(_))),
ContentBlock::Diagram(spec) => matches!(spec.alt.as_ref(), Some(AltText::Provided(_))),
```

The `None` case in the existing `Option<AltText>` fields already covers the
"Stage 2b emits no alt" state correctly — `None` on ContentBlock.alt maps to
`Unspecified` on the frame (Decision 5.2), which maps to `false` here. No change
to this logic is needed; the match against `Some(AltText::Provided(_))` naturally
excludes `Some(AltText::Unspecified)`, `Some(AltText::Decorative)`, and `None`.

### Decision 6: Comment Corrections (Part of Story A)

The following stale comments MUST be updated as part of Story A:

| File | Lines | Stale Content | Replacement |
|------|-------|---------------|-------------|
| `crates/slideforge-eval/src/for_eval.rs` | 336–342 | "Validators that inspect `slide.blocks` operate on `Deck` values produced by the layout stage" — FALSE per ADR-018 Decision 3 | "Block-level content is populated by the post-eval field-to-block threading pass (Stage 2b, ADR-019). AltTextValidator now runs post-layout via ADR-018 Decision 3 — not on this Deck output." |
| `crates/slideforge-layout/src/layout.rs` | 163–166 | "wiring slide body blocks into FrameContent::Body is STORY-027 scope" — STORY-027 has shipped | "Wiring of body/title/chart/image blocks into FrameContent is Stage 2b (ADR-019). See `slideforge-eval::field_to_block::thread_fields_to_blocks`." |
| `crates/slideforge-validate/src/alt_text.rs` | 175–193 | "MUST be revisited for Wave 3+" — wave label without story anchor | "Threading lands in Story A (`slide-field-to-block-threading`). See ADR-019." |

### Decision 7: Purity and Comemo Compatibility

Stage 2b (`thread_fields_to_blocks`) is classified as **Pure Core** (SS-02 scope).

- No I/O, no filesystem, no network, no global mutable state.
- Input: `&mut Deck` (mutates `Slide.blocks` in place).
- Output: unit (mutation is the effect, but the mutation is a pure function of the input).
- All types involved (`Deck`, `Slide`, `ContentBlock`, `AltText`) implement
  `Hash + Eq + Clone` — comemo-compatible from day one.
- Kani-amenable: the function body is bounded, terminating, and deterministic.
  Proof harness scope: for any `Deck` produced by `eval_deck`, after calling
  `thread_fields_to_blocks`, every slide with `slide_type == "chart"` and
  non-empty `fields["alt"]` must have `Slide.blocks` contain exactly one
  `ContentBlock::Chart` with `alt == Some(AltText::Provided(_))`. This property
  is VP-amenable and should be registered in the VP-INDEX when Story A ships.

### Decision 8: Relationship to ADR-005 (Two-IR Model)

Stage 2b does NOT violate ADR-005. The two-IR model defines:
- `Deck` as the **semantic** IR (pre-layout, field-resolved).
- `LaidOutDeck` as the **geometric** IR (post-layout, coordinates computed).

Stage 2b populates `Slide.blocks` — which is a `Vec<Block>` of `ContentBlock`
wrappers — within the `Deck`. This is a **semantic operation**: it converts
field-value semantics (e.g., `chart_type = "bar"`) into typed semantic content
nodes (e.g., `ContentBlock::Chart(ChartSpec { chart_type: "bar", ... })`).
No coordinates, no EMU values, no geometric layout is produced.

The `Deck` was always intended to carry `Slide.blocks`. The `vec![]` initialization
was a deferral, not a design intent. ADR-005's boundary is between semantic IR
(`Deck`) and geometric IR (`LaidOutDeck`) — Stage 2b operates entirely within the
semantic IR and is therefore fully ADR-005-compliant.

### Decision 9: Relationship to ADR-016 (Pipeline Driver)

ADR-016 Decision 3 enumerates the `build_inner` pipeline stages. This ADR
**amends** that enumeration by inserting Stage 2b between eval and brand load.
The in-code doc comment in `slideforge/src/lib.rs:build_inner` must be updated.
The `build_inner` function's dependency list (`use` statements) gains
`slideforge_eval::thread_fields_to_blocks`.

ADR-016 is marked as amended by ADR-019 (see ADR-016 Amendment Log, to be
added by state-manager post-acceptance).

### Decision 10: Relationship to ADR-018 (Post-Layout Validation)

ADR-018 Decision 3 migrates `AltTextValidator` to `validate_post_layout` operating
on `LaidOutDeck.slides[*].frames`. ADR-018's design is correct and unchanged.

Stage 2b provides the upstream data that ADR-018 needs: once `Slide.blocks` is
populated by Stage 2b, `thread_media_alt_into_frames` (layout.rs) will receive
actual `ContentBlock::Chart/Image/Diagram` entries and overwrite the
`AltText::Unspecified` structural placeholders with `AltText::Provided(s)` or
`AltText::Decorative`. `validate_post_layout` then sees the correct values.

ADR-018 requires no amendment. The `AltText::Unspecified` addition (Decision 4
of this ADR) is purely additive to the types crate and consistent with ADR-018's
Decision 3 logic.

---

## Amendment A (v1.1, 2026-06-06): TextTag + FieldValue::List

This amendment records two design decisions resolved by reading the codebase at
`develop` @ 030dec6c. Full resolution with per-story corrected file paths is in
`.factory/specs/wave4-expanded-scope-uncertainty-resolution.md`.

### A1: TextTag → FrameContent Layout Mapping (D1)

Decision 3 Table 3.1 references `TextBlock { inlines, tag: TextTag::Title }` but
**the `tag` field does not yet exist on `TextBlock`**. This amendment makes it explicit:

- `TextBlock` gains `tag: TextTag` (new enum in `slideforge-types::block`).
- `TextTag` variants: `Title`, `Subtitle`, `Body`, `Untagged` (default for all
  existing construction sites outside Stage 2b).
- Stage 2b (`thread_fields_to_blocks`) sets `TextTag::Title/Subtitle/Body` when
  constructing TextBlocks from `Slide.fields["title"]`, `["subtitle"]`, `["body"]`.
- `layout::run`'s inline pass maps tagged `ContentBlock::Text` to the correct
  region frame slot:
  - `TextTag::Title` → find Empty frame at index 0 → set `FrameContent::Title(Arc<str>)`
  - `TextTag::Subtitle` → find Empty frame at index 1 (for types with subtitle) → `FrameContent::Subtitle(Arc<str>)`
  - `TextTag::Body` → find Empty frame at index 1 (for content/agenda/etc.) → `FrameContent::Body(vec![block])`
  - `TextTag::Untagged` → existing path: push `FrameContent::TextRun` frame (unchanged)

**PPTX and DOCX exporters need zero changes.** Both already route FrameContent
variants correctly. The routing that Decision 3 implied was "new exporter work" is
in fact already implemented in `slide_serializer.rs` and `document_body.rs`.

### A2: FieldValue::List Parser Model (D5)

Decision 3 Table 3.2 references `FieldValue::Literal(Value::List(items))`. This form
**does not exist in the parser**. `FieldValue` in `deck.rs` has no `List` variant and
`value_parser()` does not parse list literals. Amendment:

- `FieldValue::List(Vec<FieldValue>)` is added to the AST (`slideforge-syntax::ast`).
- `value_parser()` in `deck.rs` is extended with a list arm using `Token::LBracket`
  / `Token::RBracket` / `Token::Comma` (confirmed present in the token set).
- Eval maps `FieldValue::List` → `Value::List(Vec<Value>)`.
- The chumsky 0.10 idiom for this is confirmed in `expr.rs` lines 96–103.

**Decision 3 Table 3.2 remains correct in intent.** The `FieldValue::Inlines` path
for pre-parsed bullet lists is unchanged. The new `FieldValue::List(items)` path is
what STORY-088 delivers, making `bullets: ["A","B","C"]` parseable at the field level.

---

## Consequences

### Positive

- Title, body, subtitle, bullets, chart metadata, image metadata, and diagram
  metadata flow from the eval output through to all three exporters (PPTX, PDF,
  DOCX) without exporter changes. The exporters already consume `FrameContent::TextRun`
  and `FrameContent::Chart/Image/Diagram` — Stage 2b provides the upstream data
  that causes layout to populate those frames correctly.
- The a11y strict gate (ADR-018 / BC-5.02.001) becomes satisfiable for all slide
  types once authors supply `alt "..."` or `decorative: true`. Before this ADR, the
  gate was unconditionally unsatisfiable for any chart/image/diagram slide because
  the alt text written by the author was never read by any pipeline stage.
- `AltText::Unspecified` makes the validator's discrimination unambiguous.
  `Decorative` now exclusively means "author-chosen opt-out"; `Unspecified` means
  "pipeline gap, E-A11-001 appropriate."
- Decks without visual media types (title, content, bullets, agenda, toc, quote,
  two_col, etc.) build cleanly under strict mode immediately after the regions.rs
  Unspecified change, even before Story A is merged — the false positive
  `AltText::Decorative`→E-A11-001 emission is eliminated.
- The fix for for_eval.rs:336–341 and layout.rs:163–166 comments removes misleading
  documentation that has blocked future contributors from understanding the pipeline.

### Negative / Trade-offs

- `AltText` gains a third variant. All existing match sites must be updated. The
  sibling-site sweep (TD-VSDD-060) covers this; estimated at 8–12 match sites
  across slideforge-layout, slideforge-validate, slideforge-pptx, slideforge-pdf,
  slideforge-docx, and slideforge-types test code.
- `thread_fields_to_blocks` is a new module in `slideforge-eval`. Its public API
  must be documented (missing_docs gate applies) and unit-tested. This is
  implementation work, not architectural risk.
- Until Story A lands, the strict gate for chart/image/diagram slides still fails
  with E-A11-001 (because Stage 2b does not exist yet, so `Slide.blocks` is still
  empty). The error message is now accurate ("visual element has no alt text") rather
  than misleading ("decorative element emitted — please add alt text"). The error is
  no longer a false positive; it is a true positive reflecting a genuine gap.

### Status as of 2026-06-05

ACCEPTED, awaiting implementation in Story A (`slide-field-to-block-threading`).
The `AltText::Unspecified` type change plus `regions.rs` + `validate_post_layout`
match arm updates can be landed as a bundled fix within Story A or as a prerequisite
fix-burst immediately preceding Story A — both paths are valid.

---

## Alternatives Considered

**Option (a) — eval-time ContentBlock construction:** Rejected. Violates ADR-005.
See Context section.

**Option (c) — layout-time field-to-ContentBlock construction:** Rejected.
Violates separation of concerns. Layout must transform ContentBlocks to frames, not
create ContentBlocks from raw field values.

**Keeping AltText as two variants (Provided | Decorative) with a None-on-ContentBlock
as the "unspecified" signal:** Rejected. The frame-level `FrameContent::Chart { alt: AltText }`
field cannot carry `None` — it is not `Option<AltText>`. The existing `AltText::Decorative`
was misused as a "not yet filled" placeholder in `regions.rs`. Introducing
`AltText::Unspecified` is the minimal, precise fix that makes the structural
placeholder semantically distinct from the author opt-out. Fewer match arms, clearer
semantics, lower risk of future confusion.

---

## Source / Origin

- `.factory/specs/wave4-content-threading-assessment.md` — root cause analysis and
  remediation scope that motivated this ADR.
- `crates/slideforge-eval/src/for_eval.rs:336–342` — `blocks: vec![]` with deferral comment.
- `crates/slideforge-layout/src/regions.rs:174–321` — five `AltText::Decorative` structural placeholders.
- `crates/slideforge-layout/src/layout.rs:393–509` — `thread_media_alt_into_frames` fallback logic.
- `crates/slideforge-validate/src/alt_text.rs:150–193` — `validate_post_layout` match arms.
- ADR-005 — Two-IR model (boundary between semantic Deck and geometric LaidOutDeck).
- ADR-016 Decision 3 — Pipeline driver stage enumeration (amended by this ADR).
- ADR-018 Decision 3 — AltTextValidator migration to `validate_post_layout`.
- Human authorization of Stage 2b (option b): 2026-06-05.

---

## Implementation Checklist (Story A Scope)

| File | Change | Type |
|------|--------|------|
| `crates/slideforge-types/src/specs.rs` | Add `AltText::Unspecified` variant; update enum doc | Type change |
| `crates/slideforge-eval/src/field_to_block.rs` | New module: `thread_fields_to_blocks(deck: &mut Deck)` | New |
| `crates/slideforge-eval/src/lib.rs` | `pub mod field_to_block; pub use field_to_block::thread_fields_to_blocks;` | Export |
| `crates/slideforge/src/lib.rs` | Insert Stage 2b call after `eval_deck`; update `build_inner` doc comment | Pipeline wiring |
| `crates/slideforge-layout/src/regions.rs` | 5 sites: `AltText::Decorative` → `AltText::Unspecified` for structural placeholders | Bug fix |
| `crates/slideforge-layout/src/layout.rs` | 3 sites: `thread_media_alt_into_frames` fallback `Decorative` → `Unspecified`; update comment at lines 163–166 | Bug fix + comment |
| `crates/slideforge-validate/src/alt_text.rs` | Match arms: `Unspecified → E-A11-001`, `Decorative → valid`; update comment lines 175–193 | Bug fix + comment |
| `crates/slideforge-types/src/block.rs` | `Unspecified` arm in any match on `AltText` (sibling-site sweep) | Sweep |
| `crates/slideforge-eval/src/for_eval.rs` | Update comment lines 336–342 | Comment fix |
| All other AltText match sites | Sibling-site sweep (TD-VSDD-060): grep `AltText::Decorative` and add `Unspecified` arm | Sweep |
| Tests | Unit tests for `thread_fields_to_blocks`; positive and negative vectors per Decision 3; E2E tests asserting rendered text/shapes in PPTX/PDF/DOCX output (LESSON-13/14 vectors) | New tests |
