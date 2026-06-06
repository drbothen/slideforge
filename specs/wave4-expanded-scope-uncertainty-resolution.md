---
document_type: uncertainty-resolution
title: "Wave-4-expanded + Wave-5 Scope — Design Uncertainty Resolution (D1–D5)"
date: 2026-06-06
resolves: [U-086-01, U-086-02, U-086-03, U-086-04, U-087-01, U-087-02, U-088-01, U-088-02, U-088-03]
stories_affected: [STORY-086, STORY-087, STORY-088]
status: resolved
traces_to: ARCH-INDEX.md
---

# Wave-4-Expanded + Wave-5 Scope — Design Uncertainty Resolution

All five design-level uncertainties resolved by reading the real codebase on
`develop` @ 030dec6c. Findings below are authoritative; story specs must be
corrected to match before implementation begins.

---

## D1 (U-086-01, U-086-02, U-086-03) — KEYSTONE: Tag → FrameContent mapping

### What is already true in the codebase

**FrameContent** (`crates/slideforge-layout/src/types.rs`) already has:
- `FrameContent::Title(Arc<str>)` — primary slide title
- `FrameContent::Subtitle(Arc<str>)` — subtitle or secondary heading
- `FrameContent::Body(Vec<ContentBlock>)` — structured body content
- `FrameContent::TextRun(Vec<InlineNode>)` — rich inline text run

**slide_serializer.rs** (`crates/slideforge-pptx/src/slide_serializer.rs`) already routes:
- `FrameContent::Title` → `<p:ph type="title" idx=0>`
- `FrameContent::Subtitle` → `<p:ph type="subTitle" idx=1>`
- `FrameContent::Body | TextRun` → `<p:ph type="body" idx=1>`

**document_body.rs** (`crates/slideforge-docx/src/document_body.rs`) already routes:
- Finds `FrameContent::Title(t)` from `slide.frames` → emits `Heading1` paragraph
- `Register::Report` content → `Normal` paragraphs under Heading1

Both exporters do NOT need any routing changes.

**layout.rs** (`crates/slideforge-layout/src/layout.rs`) currently:
- Maps `ContentBlock::Text(text_block)` → `FrameContent::TextRun(text_block.inlines.clone())`
- The `TextRun` is a flat inline pass for inline validation. It does NOT produce
  `FrameContent::Title` or `FrameContent::Subtitle`.
- The title frame is currently populated via a field-lookup at frame_idx==0 for
  text_flow computation only (lines 186–206), not for final FrameContent routing.

**TextBlock** (`crates/slideforge-types/src/block.rs`) currently has:
```rust
pub struct TextBlock {
    pub inlines: Vec<InlineNode>,
    pub span: SourceSpan,
}
```
There is **no `tag` field** on `TextBlock`. This is the gap.

### Decision (D1)

**Add `TextTag` enum and `TextBlock.tag` field. Map tagged ContentBlock::Text in
`layout::run` to the correct `FrameContent` variant. Exporters need zero changes.**

#### Step 1 — Add TextTag to slideforge-types

In `crates/slideforge-types/src/block.rs`, add:

```rust
/// Semantic role of a text paragraph within a slide.
///
/// `TextTag` is assigned by Stage 2b (`thread_fields_to_blocks`, ADR-019) when
/// constructing `ContentBlock::Text` blocks from `Slide.fields`. The layout
/// engine reads `TextTag` to route text into the correct `FrameContent` variant.
///
/// This tag is the SEMANTIC source (inline IR side). The `FrameContent::Title`,
/// `::Subtitle`, `::Body` variants are the GEOMETRIC carriers (layout IR side).
/// The layout mapping is the single translation point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextTag {
    /// Primary slide title — maps to `FrameContent::Title`.
    Title,
    /// Subtitle or secondary heading — maps to `FrameContent::Subtitle`.
    Subtitle,
    /// Body paragraph — maps to `FrameContent::Body` (wrapped in ContentBlock::Text).
    Body,
    /// Generic/untagged paragraph — maps to `FrameContent::TextRun`.
    Untagged,
}

pub struct TextBlock {
    pub inlines: Vec<InlineNode>,
    /// Semantic role. Set by Stage 2b. Defaults to `Untagged` for backward compat.
    pub tag: TextTag,
    pub span: SourceSpan,
}
```

`TextTag` derives `Debug + Clone + Copy + PartialEq + Eq + Hash` — comemo-compatible.

#### Step 2 — Stage 2b (ADR-019) sets TextTag

In `crates/slideforge-eval/src/field_to_block.rs` (the new Stage 2b module),
`thread_fields_to_blocks` constructs:

```rust
// For slide.fields["title"] = Value::Str(s):
ContentBlock::Text(TextBlock {
    inlines: vec![InlineNode::Plain(Arc::from(s))],
    tag: TextTag::Title,
    span: ...,
})

// For slide.fields["subtitle"] = Value::Str(s):
ContentBlock::Text(TextBlock {
    inlines: vec![InlineNode::Plain(Arc::from(s))],
    tag: TextTag::Subtitle,
    span: ...,
})

// For slide.fields["body"] = Value::Str(s):
ContentBlock::Text(TextBlock {
    inlines: vec![InlineNode::Plain(Arc::from(s))],
    tag: TextTag::Body,
    span: ...,
})
```

#### Step 3 — layout.rs maps TextTag → FrameContent

In `crates/slideforge-layout/src/layout.rs`, the inline text pass (currently at
lines ~270–314) processes `ContentBlock::Text(text_block)`. The mapping becomes:

```rust
ContentBlock::Text(text_block) => {
    match text_block.tag {
        TextTag::Title => {
            // Produce FrameContent::Title carrying the plain text.
            // Replaces the Empty frame at index 0 (title region) OR
            // appends a new Title frame if no Empty frame exists.
            let text = extract_inline_text_to_str(&text_block.inlines);
            // Update the region frame at index 0 if it is Empty:
            if let Some(frame) = all_frames.iter_mut().find(|f| matches!(f.content, FrameContent::Empty) && /* frame_idx==0 heuristic */) {
                frame.content = FrameContent::Title(Arc::from(text.as_str()));
                frame.text_flow = Some(compute_text_flow(&text, frame.bbox));
            }
            // Do NOT push a separate TextRun frame for Title-tagged blocks.
        }
        TextTag::Subtitle => {
            // Same pattern: find the subtitle region frame (index 1 for title/closing/section_break)
            // and set FrameContent::Subtitle.
        }
        TextTag::Body => {
            // Find the body region frame (index 1 for content/agenda/etc.)
            // and set FrameContent::Body(vec![ContentBlock::Text(text_block.clone())]).
        }
        TextTag::Untagged => {
            // Existing path: push FrameContent::TextRun.
            let bbox = /* clamped full-width placeholder */;
            all_frames.push(Frame {
                bbox,
                content: FrameContent::TextRun(text_block.inlines.clone()),
                text_flow: None,
            });
        }
    }
}
```

**The exact frame-slot assignment uses the region frame index, not a positional
guess.** The title region is always index 0 for slide types that have one (see
`regions.rs`). The body/subtitle region is always index 1. `layout.rs` already has
the frame vector at this point; match by frame_idx (0 = title, 1 = subtitle/body)
and `FrameContent::Empty` to find the slot.

#### Exporter impact

PPTX `slide_serializer.rs`: **zero changes.** It already handles `FrameContent::Title`,
`Subtitle`, `Body`, and `TextRun`. Once layout produces `FrameContent::Title` from
`TextTag::Title`, the serializer routes it correctly with no modification.

DOCX `document_body.rs`: **zero changes.** It already finds `FrameContent::Title(t)`
from `slide.frames` at line 146 for Heading1. Once layout populates that frame,
DOCX output is correct.

#### Why TextRun is not eliminated

`FrameContent::TextRun` remains for:
- Inline validation pass (scanning `InlineNode` trees for xref depth, depth violations)
- Bullet item frames (already produce TextRun per `push_bullet_frames`)
- `TextTag::Untagged` paragraphs from `ContentBlock::Text` that are not title/body/subtitle

The `TextTag::Title/Subtitle/Body` path does NOT produce a separate TextRun frame —
the tagged block goes into the region slot and is done.

#### Clarification: no double-encoding

`TextBlock.tag` is the **semantic source** (lives in the `Deck` / semantic IR).
`FrameContent::Title` is the **geometric carrier** (lives in `LaidOutDeck`).
The layout pass translates one to the other. There is no double-encoding: the tag
lives in the semantic IR until the layout stage reads it and discards it when
constructing the geometric IR frame.

---

## D2 (U-086-04) — Sweep blast radius

### AltText::Decorative occurrences

The ADR-019 sibling-site sweep (Decision 4, AltText::Unspecified) requires updating
ALL match arms on `AltText` to add a `Unspecified` arm. The scanner measured 168
occurrences across 15 files, but most are in tests and doc-comment examples.

**Crates with production-code AltText match sites that MUST be updated:**

| Crate | File | Count (approx) | Change |
|-------|------|----------------|--------|
| `slideforge-layout` | `src/layout.rs` | 3 | fallback arms in `thread_media_alt_into_frames` |
| `slideforge-layout` | `src/regions.rs` | 5 | structural placeholder change |
| `slideforge-validate` | `src/alt_text.rs` | ~4 | `validate_post_layout` match arms |
| `slideforge-pptx` | `src/slide_serializer.rs` | 0 production; ~8 in tests | add `Unspecified` arm to test matches |
| `slideforge-docx` | `src/document_body.rs` | 0 | no direct AltText match in this file |
| `slideforge-pdf` | any | TBD | grep required |
| `slideforge-types` | `src/block.rs` | 2 | `produces_structure_group()` matches |

**The corrected `target_module` list for STORY-086** (not just pptx/docx):
- `crates/slideforge-types/src/block.rs` — add TextTag enum + TextBlock.tag
- `crates/slideforge-types/src/specs.rs` — add AltText::Unspecified
- `crates/slideforge-eval/src/field_to_block.rs` — new file (Stage 2b)
- `crates/slideforge-eval/src/lib.rs` — export thread_fields_to_blocks
- `crates/slideforge/src/lib.rs` — wire Stage 2b into build_inner
- `crates/slideforge-layout/src/layout.rs` — TextTag routing + fallback Unspecified
- `crates/slideforge-layout/src/regions.rs` — 5 Decorative → Unspecified
- `crates/slideforge-validate/src/alt_text.rs` — Unspecified → E-A11-001 match arms
- All crates grepped for AltText::Decorative structural placeholders (sibling sweep)

**TextBlock construction sites (STORY-086 AC-007 — add tag field):**
The 18 TextBlock construction sites across 8 files currently use `TextBlock { inlines, span }`.
After adding `tag: TextTag`, all construction sites must add `tag: TextTag::Untagged`
as the default. Stage 2b constructs TextBlock with semantic tags; all other
construction sites (test helpers, parser output) use `TextTag::Untagged`.

Crates with TextBlock construction that must add `tag: TextTag::Untagged`:
- `crates/slideforge-syntax/src/` — parser output (any TextBlock construction in AST lowering)
- `crates/slideforge-eval/src/` — any TextBlock construction in eval
- Test helpers in all crates

---

## D3 (U-087-01) — SlideType trait surface + field validation

### Real trait surface (from `crates/slideforge-plugin-api/src/traits/slide_type.rs`)

The trait is:
```rust
pub trait SlideType: Send + Sync {
    fn id(&self) -> &'static str;
    fn required_fields(&self) -> &[FieldDef];
    fn optional_fields(&self) -> &[FieldDef];
    fn layout_name(&self) -> &'static str;
    fn lay_out(&self, slide: &Slide, brand: &Brand, canvas: Canvas) -> Result<LaidOutSlide, LayoutError>;
}
```

There is **no `keyword()`, no `region_spec()`, no `validate_fields()` method** on
the trait. The stories were wrong about the trait surface.

`validate_fields` is a **free function** in
`crates/slideforge-plugin-api/src/slide_types/registry.rs`:
```rust
pub fn validate_fields(slide: &Slide, slide_type: &dyn SlideType) -> Vec<Diagnostic>
```

### Decision (D3)

**Where custom validation for the 3 new types goes:**

Custom field validation (progress_bar value∈[0,100]; weighted_composite component
weight>0/score∈[0,100]/non-empty) belongs in `SlideType::lay_out()`, returning
`Err(LayoutError::MissingRequiredField)` or `Err(LayoutError::FieldTypeMismatch)`.

Rationale: `validate_fields` only checks required/optional field presence (E-VAL-101,
E-VAL-102, W-VAL-103). It does not check value ranges. Value-range validation
happens at layout time when `lay_out()` extracts the field and finds an out-of-range
value. `LayoutError::FieldTypeMismatch` carries an `actual_type` string that can
describe the range violation (e.g., `actual_type: "42.5 — value must be integer 0–100"`).

**Template pattern for new SlideType implementations:**

Follow `crates/slideforge-plugin-api/src/slide_types/stat_callout.rs` (which has 4
required fields and a custom `lay_out()` implementation). The pattern is:
1. `required_fields()` returns a `&[FieldDef]` static slice with the required fields
2. `optional_fields()` returns `&[]` or a static slice
3. `layout_name()` returns the PPTX layout name string
4. `lay_out()` extracts fields from `slide.fields`, validates ranges, constructs
   `LaidOutSlide` using the same region-frame pattern as existing types

For `progress_bar`:
```rust
fn lay_out(&self, slide: &Slide, brand: &Brand, canvas: Canvas) -> Result<LaidOutSlide, LayoutError> {
    let value = match slide.fields.get("value") {
        Some(FieldValue::Literal(Value::Num(n))) if *n >= 0 && *n <= 100 => *n,
        Some(FieldValue::Literal(Value::Num(n))) => return Err(LayoutError::FieldTypeMismatch {
            slide_type: self.id().to_owned(),
            field: "value".to_owned(),
            expected_type: "integer 0–100".to_owned(),
            actual_type: format!("{n} — out of range"),
        }),
        _ => return Err(LayoutError::MissingRequiredField {
            slide_type: self.id().to_owned(),
            field: "value".to_owned(),
        }),
    };
    // ... produce frames
}
```

---

## D4 (U-087-02) — Region registration + progress_bar value-proportional geometry

### Real region contract (from `crates/slideforge-layout/src/regions.rs`)

```rust
pub fn region_frames_for(
    slide_type_keyword: &str,
    page_width: Emu,
    page_height: Emu,
) -> Option<Vec<Frame>>
```

This function has **no access to field values**. It only gets the keyword and
page dimensions. All frames start as `FrameContent::Empty` (or structural media
placeholders). **Value-proportional geometry is impossible in `region_frames_for`.**

### Decision (D4)

**Value-proportional geometry for `progress_bar` lives in `SlideType::lay_out()`.**

The design is:
1. `region_frames_for("progress_bar", w, h)` returns a fixed set of Empty frames:
   a title frame and a bar background frame (full width, fixed height). This is the
   static geometry skeleton.
2. `ProgressBarSlideType::lay_out()` reads `slide.fields["value"]`, computes the
   filled bar width as `bar_bg_width * value / 100`, and constructs a `LaidOutSlide`
   with:
   - Frame 0: `FrameContent::Title(title)` — from the title region frame bbox
   - Frame 1: Bar background — `FrameContent::Empty` (the exporter renders the
     background fill via shape styling, not content)
   - Frame 2: Bar fill — a Shape frame with width proportional to value, constructed
     in `lay_out()` by calling `layout_shapes()` or by direct Frame construction
   - Frame 3: Label — `FrameContent::TextRun` with the label text
   - Alternatively, the entire progress bar is rendered as a Shape/SVG/diagram block
     that `lay_out()` creates.

The critical insight: `SlideType::lay_out()` receives `slide: &Slide` which
has full access to `slide.fields`. It can compute the proportional geometry there.
The region frames from `regions.rs` are only the static skeleton; `lay_out()`
can construct additional frames or modify frame content beyond what `regions.rs`
provides.

**Region registration requirement:** `progress_bar` must be added to `regions.rs`
so that `layout::run`'s `region_frames_for` call does not return `None` (which
triggers `LayoutError::UnknownSlideType`). The registered frames are the static
skeleton. `lay_out()` supplements them.

**Note:** `SlideType::lay_out()` is currently called by the plugin registry tests
but is NOT yet wired into `layout::run` (which calls `region_frames_for` directly).
STORY-087 needs to clarify whether `lay_out()` is called instead of or in addition
to `layout::run`'s region-frame pass. Based on the registry source, `lay_out()`
produces the complete `LaidOutSlide` independently. The pipeline should route to
`SlideType::lay_out()` for new types OR ensure `layout::run` calls `lay_out()` for
slide types that are registered in the `SlideTypeRegistry`. This is an existing
pipeline integration gap that STORY-087 must close.

### D4 — severity_cards keyword gap

`severity_cards` appears in `COLOR_CODED_TYPES` in `label_check.rs` and in
`regions.rs` (standard two-region layout), but does NOT appear in `SLIDE_TYPE_KEYWORDS`
in `keywords.rs`.

This is a real inconsistency: `severity_cards` is registered in `SlideTypeRegistry`
(it appears in `registry.rs` default registration) and in `regions.rs`, but is
absent from the `SLIDE_TYPE_KEYWORDS` PHF set. AC-024 (consistency assertion that
all registered types appear in SLIDE_TYPE_KEYWORDS) will fail.

**Fix required in STORY-087 (or STORY-086 sweep):** Add `severity_cards` to
`SLIDE_TYPE_KEYWORDS` in `crates/slideforge-syntax/src/keywords.rs`. Also add
`status`, `progress_bar`, and `weighted_composite` when STORY-087 introduces them.

---

## D5 (U-088-01, U-088-02, U-088-03) — Parser FieldValue list model

> **CORRECTION — 2026-06-06 (STORY-086 pass-5 / test-writer parser verification):**
> An earlier version of this resolution (pre-correction) claimed: "The `@var` binding
> form is confirmed to produce `Value::List` from the existing evaluator … The path
> `@var items = [...] → eval → Value::List → Stage 2b → ContentBlock::Bullets` is
> fully exercisable with STORY-086 alone." That claim was FALSE. Verified against
> `develop@030dec6c` and the STORY-086 worktree HEAD `77092e2a`:
>
> - `value_parser()` in `crates/slideforge-syntax/src/parser/deck.rs` (line 66)
>   handles only scalar field values (Template/Num/Float/Bool/Ident). There is no
>   list-literal arm; therefore a `vars:` block entry cannot hold a `Value::List`.
> - The `FieldValue::Inlines` / bullets path in `thread_fields_to_blocks` is
>   unreachable from any current DSL input.
> - NO current DSL syntax — neither direct `bullets: ["a","b","c"]` literals NOR
>   any `@var items = [...]` / vars-block form — produces a `Value::List`. The
>   `Value::List → ContentBlock::Bullets` Stage-2b mapping is implemented and
>   load-bearing-tested in STORY-086 via a programmatic unit test
>   (`field_to_block_unit.rs::test_..._ac007_value_list_produces_content_block_bullets`),
>   but is NOT reachable from DSL surface syntax until STORY-088 ships.
> - ALL DSL-level bullets-list syntax (literal and variable-binding) is deferred to
>   STORY-088 (Wave 5), whose scope MUST cover the `value_parser()` list-literal arm
>   + eval propagation so that `bullets:` resolves to a `Value::List`. STORY-088
>   scope is being expanded accordingly (story-writer will update STORY-088 itself).
> - STORY-086's AC-007 load-bearing coverage is the Stage-2b unit test (programmatic
>   path only). The e2e bullets DSL fixture is a placeholder pending STORY-088.
>
> The decision text below is corrected to reflect this. The per-story summary in
> the "Threading to eval's Value::List" sub-section has been rewritten accordingly.

### Real FieldValue (from `crates/slideforge-syntax/src/parser/deck.rs`)

```rust
// value_parser() produces:
FieldValue::Template(Vec<TemplateChunk>)   // string literal with {{ }} interpolation
FieldValue::Num(i64)                        // integer literal
FieldValue::Float(f64)                      // float literal
FieldValue::Bool(bool)                      // bool literal
FieldValue::Ident(String)                   // bare identifier
// On error:
FieldValue::Error
```

There is **no `FieldValue::List` variant** and no `FieldValue::Literal(Value::List(_))`
path. The parser's `value_parser()` in `deck.rs` does not parse list literals.

### Real Expr::List (from `crates/slideforge-syntax/src/parser/expr.rs`)

```rust
// In expr_inner(), the list parser:
let list = e
    .clone()
    .separated_by(just(Token::Comma))
    .allow_trailing()
    .collect::<Vec<_>>()
    .delimited_by(just(Token::LBracket), just(Token::RBracket))
    .map(Expr::List);
```

**`Token::LBracket` and `Token::RBracket` already exist in the token set.** The
lexer emits these tokens. `Expr::List` exists and is parsed in the expression
context (`@for`, `@if` conditions, template interpolations).

The list token infrastructure (`Token::LBracket`, `Token::RBracket`, `Token::Comma`)
is already present. What is missing is a `FieldValue::List` variant in the field
value parser in `deck.rs`.

### Decision (D5)

**STORY-088 adds `FieldValue::List(Vec<FieldValue>)` as a new variant and extends
`value_parser()` to parse list literals.**

The implementation in `deck.rs` follows the exact same pattern as `expr.rs`:

```rust
// Add to value_parser() in deck.rs:
let list_val = value_parser_ref  // recursive reference to value_parser
    .separated_by(just(Token::Comma))
    .allow_trailing()
    .collect::<Vec<_>>()
    .delimited_by(just(Token::LBracket), just(Token::RBracket))
    .map(|(items, span)| (FieldValue::List(items), span));
```

Since `value_parser()` in `deck.rs` currently uses `template_val.or(other_val)`, the
list extension adds a third alternative: `template_val.or(other_val).or(list_val)`.

**`FieldValue::List` does not need to be recursive in v1.0** (no nested lists
in slide fields). The items are flat `FieldValue` scalars. The Expr::List parser
handles recursion for expressions; `FieldValue::List` holds `Vec<FieldValue>` of
scalar types.

### Threading to eval's Value::List

In `slideforge-eval`, `FieldValue::List(items)` will map to `Value::List(...)` during
eval once STORY-088 delivers the `value_parser()` list-literal arm. This follows the
same pattern as `FieldValue::Literal(Value::*)` → direct passthrough: the eval for
`FieldValue::List` converts each item to its `Value` equivalent and produces
`Value::List(values)`.

**Current state (pre-STORY-088):** `Value::List` is reachable ONLY programmatically.
The Stage-2b mapping `Value::List → ContentBlock::Bullets` is implemented and verified
by the STORY-086 unit test, but NO DSL input can produce a `Value::List` until
STORY-088 adds the `FieldValue::List` parser arm. BC-1.16.001 PC-7 and STORY-086
AC-007 are therefore partially covered: the Stage-2b unit test proves the mapping is
correct; full DSL-to-bullets end-to-end coverage is deferred to STORY-088.

**STORY-088 is the prerequisite** that makes `Value::List` reachable from DSL field
assignments. Its scope must include: `FieldValue::List(Vec<FieldValue>)` AST variant,
`value_parser()` list-literal arm in `deck.rs`, eval handling producing `Value::List`,
and at least one end-to-end DSL fixture exercising `bullets: ["a", "b", "c"]`.

### No external research needed

`Token::LBracket` and `Token::RBracket` are confirmed present in the codebase.
The `Expr::List` parser in `expr.rs` shows the exact chumsky 0.10 token-stream list
idiom (using `just(Token::LBracket)` / `just(Token::RBracket)`). No Perplexity
research is needed for the chumsky list-parsing pattern.

---

## Per-Story Corrected File Paths and APIs

### STORY-086 — AltText::Unspecified + TextBlock.tag + Stage 2b threading

**Correct target_module list (not pptx/docx serialize_slide.rs — those files do
not exist and exporters need zero routing changes):**

| File | Change |
|------|--------|
| `crates/slideforge-types/src/specs.rs` | Add `AltText::Unspecified` variant |
| `crates/slideforge-types/src/block.rs` | Add `TextTag` enum; add `tag: TextTag` to `TextBlock` |
| `crates/slideforge-eval/src/field_to_block.rs` | New: `thread_fields_to_blocks(deck: &mut Deck)` |
| `crates/slideforge-eval/src/lib.rs` | Export `thread_fields_to_blocks` |
| `crates/slideforge/src/lib.rs` | Wire Stage 2b into `build_inner` |
| `crates/slideforge-layout/src/layout.rs` | TextTag routing in inline pass; fallback Unspecified |
| `crates/slideforge-layout/src/regions.rs` | 5 sites: `Decorative` → `Unspecified` |
| `crates/slideforge-validate/src/alt_text.rs` | Match arms: `Unspecified` → E-A11-001; `Decorative` → valid |
| All AltText match sites (sibling sweep) | Add `Unspecified` arm |
| All TextBlock construction sites | Add `tag: TextTag::Untagged` |

**The core routing insight:** PPTX and DOCX exporters already correctly route
`FrameContent::Title → <p:ph type="title">` and `FrameContent::Title(t) → Heading1`.
The only missing piece is layout.rs populating those frame variants from the
TextTag-tagged ContentBlocks that Stage 2b produces.

### STORY-087 — Three new slide types

**Correct trait methods:**
- `id() -> &'static str` — the DSL keyword
- `required_fields() -> &[FieldDef]`
- `optional_fields() -> &[FieldDef]`
- `layout_name() -> &'static str`
- `lay_out(&self, slide: &Slide, brand: &Brand, canvas: Canvas) -> Result<LaidOutSlide, LayoutError>`

**No `keyword()`, `region_spec()`, or `validate_fields()` method on the trait.**
Custom value-range validation goes in `lay_out()` via `LayoutError::FieldTypeMismatch`.

**Registration:** Each new type is registered via:
```rust
r.register(Box::new(ProgressBarSlideType::new()));
// etc.
```
in `crates/slideforge-plugin-api/src/slide_types/registry.rs::Default::default()`.

**regions.rs additions:** Add `"progress_bar"`, `"status"`, `"weighted_composite"`
match arms in `region_frames_for()` returning their static frame skeletons.

**keywords.rs additions:** Add `"progress_bar"`, `"status"`, `"weighted_composite"`,
and `"severity_cards"` (the existing gap) to `SLIDE_TYPE_KEYWORDS`.

**progress_bar value-proportional geometry:** Computed in `ProgressBarSlideType::lay_out()`,
not in `region_frames_for()`. The `lay_out()` method receives `slide: &Slide` with
full field access.

### STORY-088 — FieldValue::List parser

**Correct file:** `crates/slideforge-syntax/src/parser/deck.rs` — the `value_parser()`
function. NOT `expr.rs` (that already works for expressions).

**Correct approach:** Add `FieldValue::List(Vec<FieldValue>)` variant to the AST
(`crates/slideforge-syntax/src/ast.rs`) and extend `value_parser()` to parse
`[ item, item, ... ]` using `Token::LBracket` / `Token::RBracket` / `Token::Comma`
(all confirmed present in the token set).

**Also update eval:** `crates/slideforge-eval/src/` must handle `FieldValue::List`
and produce `Value::List(Vec<Value>)`.

**No Perplexity research needed** — the chumsky 0.10 list-parsing idiom is
demonstrated in `crates/slideforge-syntax/src/parser/expr.rs` lines 96–103.

---

## severity_cards Keyword Gap — Summary

| Location | Status |
|----------|--------|
| `regions.rs` | Present (standard two-region layout) |
| `registry.rs` | Present (registered in Default) |
| `label_check.rs` `COLOR_CODED_TYPES` | Present |
| `keywords.rs` `SLIDE_TYPE_KEYWORDS` | **ABSENT — gap** |

Fix: add `"severity_cards"` to `SLIDE_TYPE_KEYWORDS` in `keywords.rs`. This should
be included in STORY-086 or STORY-087 scope as it is a consistency sweep item.
The `test_bc_1_09_008_is_slide_type_keyword_all_31_types` test does not include
`severity_cards` (consistent with the gap), so adding it requires updating that test too.

---

## External Research Status

No external research is needed. All decisions resolved from codebase reading:

| Item | Research needed? | Resolution |
|------|-----------------|------------|
| chumsky 0.10 list literal idiom | No | Confirmed in `expr.rs` lines 96–103 |
| Token::LBracket/RBracket existence | No | Confirmed used in `expr.rs` |
| OOXML placeholder routing | No | Confirmed working in `slide_serializer.rs` |
| SlideType trait surface | No | Confirmed in `slide_type.rs` |
| region_frames_for signature | No | Confirmed in `regions.rs` |
