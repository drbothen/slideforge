---
document_type: architect-adjudication
story: STORY-087
finding: F-087-P2-CONTENT-RENDERING
severity: HIGH
decision: Option T (Threading) — extend Stage 2b to thread label/value/components into ContentBlocks; wire RegionRole-driven slot filling; dynamic-geometry bar fill via a new FrameContent::ColorBar variant
status: DECIDED
author: architect
timestamp: 2026-06-06
version: "1.0"
---

# Architect Adjudication — STORY-087 Pass-2
## Content Rendering for Color-Coded Slide Types (status / progress_bar / weighted_composite)

---

## 1. Finding Statement

Adversary pass-2 identified that all three color-coded slide types pass validation (LabelCheck
E-A11-002 + ValueRangeValidator E-VAL-011 — F-G3-HIGH-003 / F-087-P1-001 closures work) but
render NO VISIBLE CONTENT to output.

The gap has three dimensions:

1. **Stage 2b silence.** `thread_fields_to_blocks` (crates/slideforge-eval/src/field_to_block.rs)
   threads only `title`, `subtitle`, `body`, `bullets`, `chart_type`, `src`, `source`. It does
   NOT thread `label`, `value`, or `components` into `ContentBlock` entries.

2. **Region-slot emptiness.** `region_frames_for` (crates/slideforge-layout/src/regions.rs)
   produces the correct geometry skeleton for all three types but every frame carries
   `FrameContent::Empty`. The `fill_region_slot_or_append` pass in `layout::run` only fills
   slots claimed by `ContentBlock::Text` blocks tagged `TextTag::Title`/`Subtitle`/`Body`.
   Since `label` and `value` produce no `ContentBlock::Text` blocks today, the Body-role and
   Generic-role frames for these types remain `FrameContent::Empty` in the final `LaidOutSlide`.

3. **Progress-bar proportional geometry.** `progress_bar` requires a fourth frame whose
   `width` is proportional to `value/100 * bar_background_width`. This is dynamic geometry
   that depends on a field value — it cannot come from the static `region_frames_for`
   skeleton and cannot be injected by `fill_region_slot_or_append` (which only fills
   pre-allocated slots). The `lay_out()` implementations are geometry-only stubs (pass-1
   adjudication) and `lay_out()` is never called by `layout::run` (ADR-005 boundary preserved).

AC-002/008/015 all assert that label text is VISIBLE in PPTX output (present in an OOXML
element). This requires the content to be in a `FrameContent` variant that exporters
serialise to `<a:t>` text runs.

---

## 2. stat_callout Canonical Pattern — What It Does End-to-End

**Finding: stat_callout is NOT a canonical pattern for content rendering. It relies on
Stage 2b threading via title only.**

Tracing stat_callout through the pipeline:

1. **stat_callout.rs `lay_out()`** — returns a 5-frame skeleton with all `FrameContent::Empty`
   frames. The five slots carry `RegionRole::Title` (slot 0), and `RegionRole::Generic` (slots
   1–4, for stat_1, stat_2, label_1, label_2).

2. **`region_frames_for("stat_callout")`** — same 5-frame skeleton, identical geometry.
   `layout::run` calls `region_frames_for`, not `lay_out()`.

3. **Stage 2b threading** — `thread_fields_to_blocks` threads `"title"` into
   `ContentBlock::Text(TextTag::Title)`. It does NOT thread `stat_1`, `label_1`, `stat_2`,
   `label_2`. Those four Generic-role slots receive NO `ContentBlock::Text` from Stage 2b.

4. **`fill_region_slot_or_append` in `layout::run`** — fills the `RegionRole::Title` slot
   with the title text. The four `RegionRole::Generic` slots receive no fill because no
   `ContentBlock::Text` with any tag was produced for them.

5. **Result:** stat_callout's `source_index`, `source_title`, and speaker notes are
   propagated but `stat_1`, `label_1`, `stat_2`, `label_2` values are ALSO rendered as
   `FrameContent::Empty` in the current codebase. stat_callout suffers the same defect as
   the three color-coded types — it just was not targeted by adversary pass-2 because
   `stat_callout` has no WCAG label enforcement contract driving immediate attention.

**Implication:** stat_callout does NOT provide a solved template for the problem. It
demonstrates the same gap. The correct pattern must be established by this adjudication
for all four affected types.

---

## 3. Option Analysis

### Option A — Route registered SlideTypes through `lay_out()` in `layout::run`

Already REJECTED in pass-1 adjudication (F-087-P1-001). The rationale stands:

- Blast radius across all 34 registered types.
- ADR-005 two-IR boundary violation.
- Kani/VP-011 purity regression.
- Trait signature incompatibility with deck-level pipeline state.

**REJECTED.**

### Option L (lay_out wiring — scoped)

Could we route ONLY the three new types + stat_callout through `lay_out()` and let
`lay_out()` produce a complete `LaidOutSlide` for those types while keeping the
`region_frames_for` path for all others?

This is the "opt-in" variant of Option A. It does not escape the structural problems:

1. `lay_out()` lacks `source_index`, deck-level `deck_warnings`, speaker-note derivation,
   and the `collect_sections` output. If `layout::run` routes selected types through
   `lay_out()`, it must either duplicate all post-region-map passes in each `lay_out()`
   implementation (explosion of per-type responsibility) or merge the partial result
   back into the pipeline (architectural complexity with no precedent).

2. ADR-005 reads: `layout::run` is the SOLE semantic→geometric boundary. Introducing
   per-type alternate entry points dissolves this invariant even for a subset.

3. Kani VP-011: the proof covers `layout::run`'s invariant that slide count is preserved
   and all bboxes are valid. Introducing conditional `lay_out()` dispatch creates an
   untested branch. The opt-in scope does not reduce this concern — any dynamic dispatch
   makes the proof surface irregular.

**REJECTED.** Option T is both safer and more complete.

### Option T — Threading (Selected)

**What it means:**

1. Extend Stage 2b (`thread_fields_to_blocks`) to thread `label`, `value`, and per-component
   data from `slide.fields` into `ContentBlock::Text` entries with the correct `TextTag` and
   a new `TextTag::Label` variant (or reuse `TextTag::Body` with slide-type discrimination —
   see Section 4 for the resolution).

2. Assign the correct `RegionRole` to the Generic-role region frames in `regions.rs` so
   that `fill_region_slot_or_append` routes the label/value blocks into them.

3. For `progress_bar`, introduce a new `FrameContent::ColorBar { filled_width_emu: Emu,
   total_width_emu: Emu, color: Rgb }` variant in the layout IR, and produce it in a new
   dedicated pass in `layout::run` after the standard `fill_region_slot_or_append` loop.
   The color-bar frame replaces (or augments) the Generic bar-background slot.

4. For `weighted_composite`, each component's `name`/`label`/`score`/`weight` row is
   represented as a single `ContentBlock::Text(TextTag::Body)` block per row. Each row
   maps to one of the five pre-allocated `RegionRole::Generic` component slots. The
   threading pass constructs one composed text string per component and emits it as a
   `ContentBlock::Text(TextTag::Body)` (or a dedicated tag — see Section 4).

**Why this is correct:**

1. **Zero ADR-005 violation.** Stage 2b is the semantic pre-processing stage that already
   populates `Slide.blocks` before `layout::run`. Extending Stage 2b to handle additional
   field types is the chartered mechanism — ADR-019 Decision 3 explicitly governs the
   fields-to-blocks mapping.

2. **Zero blast radius on existing types.** `thread_fields_to_blocks` already has
   slide-type-keyed dispatch for media types (chart/image/diagram). Adding
   `"status" | "progress_bar" | "weighted_composite" | "stat_callout"` branches is a
   strictly additive change that does not touch any other slide type's code path.

3. **RegionRole wiring is already the correct routing mechanism.** The existing `fill_region_slot_or_append`
   already routes by `RegionRole`. The only gap is that the Generic slots in the three
   affected types have no role-mapped blocks to claim them. Adding the blocks + tightening
   the role on those slots closes the gap.

4. **Kani/VP-011 purity preserved.** `thread_fields_to_blocks` is a pure function (documented
   in its module). The extension is also pure. `layout::run` is unchanged structurally; only
   the data it receives (more populated `Slide.blocks`) changes.

5. **Solves stat_callout for free.** Once Stage 2b threads `stat_1`/`label_1`/`stat_2`/`label_2`,
   stat_callout's four `RegionRole::Generic` slots will be filled by `fill_region_slot_or_append`
   through the existing generic-fallback path. No `regions.rs` change needed for stat_callout.

---

## 4. Decision and Detailed Mechanism

### 4.1 New TextTag Variant: `TextTag::ColorLabel`

**Rationale:** `label` on a color-coded slide is semantically distinct from `body`. If we
thread it as `TextTag::Body`, the Body-role slot gets claimed, which is correct for `status`
(the Body slot IS the label region) but breaks `progress_bar` and `weighted_composite` where
`Body`-role frames exist for different content (the label-below-bar in progress_bar; the
aggregate-score label in weighted_composite). To avoid order-sensitivity surprises and to
allow the PPTX exporter to render `ColorLabel` frames with semantically correct placeholder
type, a new `TextTag::ColorLabel` variant is introduced.

`TextTag::ColorLabel` maps to `RegionRole::Body` in `fill_region_slot_or_append` (same as
`TextTag::Body`) — the routing logic is the same but the semantic intent is distinct, enabling
exporters to style it differently (e.g., bold, larger font) and future accessibility tooling
to identify it.

**`slideforge-types` change:** Add `ColorLabel` variant to `TextTag` enum. This is additive
and non-breaking to all existing match arms (all have `_` or explicit arms already).

### 4.2 Stage 2b Extension (field_to_block.rs)

Extend the slide-type dispatch section of `thread_fields_to_blocks` to handle the color-coded
types. The extension follows the same structure as the `"chart"` / `"image"` / `"diagram"` arms.

**For `"status"`:**
```
// Thread "title" → TextTag::Title (already handled by the generic title block above)
// Thread "label" → TextTag::ColorLabel (Body role slot — the wide right-side frame)
if let Some(text) = extract_str_field(slide, "label")
    && !text.trim().is_empty()
{
    slide.blocks.push(make_text_block_tagged(text, TextTag::ColorLabel));
}
```

**For `"progress_bar"`:**
```
// Thread "title" → TextTag::Title (generic handling above)
// Thread "label" → TextTag::ColorLabel (Body role slot — label below bar)
if let Some(text) = extract_str_field(slide, "label")
    && !text.trim().is_empty()
{
    slide.blocks.push(make_text_block_tagged(text, TextTag::ColorLabel));
}
// Thread "value" → a new ColorBarSpec block (see 4.3)
// (value is an integer; thread it as ContentBlock::ColorBar)
if let Some(FieldValue::Literal(Value::Int(v))) = slide.fields.get("value") {
    // Clamp to [0, 100] defensively — ValueRangeValidator already fires for out-of-range,
    // but the threading pass must not panic.
    let pct: i64 = (*v).clamp(0, 100);
    slide.blocks.push(Block {
        content: ContentBlock::ColorBar(ColorBarSpec { percent: pct as u8 }),
        label: None,
        span: SourceSpan::default(),
    });
}
```

**For `"weighted_composite"`:**
```
// Thread "title" → TextTag::Title (generic handling above)
// Thread "label" (aggregate) → TextTag::ColorLabel (Body role slot — aggregate row)
if let Some(text) = extract_str_field(slide, "label")
    && !text.trim().is_empty()
{
    slide.blocks.push(make_text_block_tagged(text, TextTag::ColorLabel));
}
// Thread each component → TextTag::Body (component row slots, Generic role)
// Compose a single text string per component: "<name>: <score>/100 (wt: <weight>) — <label>"
// This ensures all semantic data is accessible via the text frame.
if let Some(FieldValue::Literal(Value::List(components))) = slide.fields.get("components") {
    for comp_val in components.iter() {
        if let Value::Map(comp) = comp_val {
            let row_text = compose_component_row_text(comp);
            if !row_text.is_empty() {
                slide.blocks.push(make_text_block_tagged(&row_text, TextTag::Body));
            }
        }
    }
}
```

`compose_component_row_text` is a pure helper that reads `name`, `score`, `weight`, `label`
from a component map and formats a single accessible string. Example output:
`"Quality: 85/100 (wt: 0.4) — Excellent"`.

**Stat_callout** also gains threading in the same extension block:
```
// Thread "stat_1", "label_1", "stat_2", "label_2", and optionally "stat_3", "label_3"
// → TextTag::Body (all claim Generic role slots in order of registration)
for field_name in &["stat_1", "label_1", "stat_2", "label_2", "stat_3", "label_3"] {
    if let Some(text) = extract_str_field(slide, field_name)
        && !text.trim().is_empty()
    {
        slide.blocks.push(make_text_block_tagged(text, TextTag::Body));
    }
}
```

### 4.3 New ContentBlock::ColorBar and ColorBarSpec

**Rationale:** The progress bar's proportional fill width is dynamic geometry — it depends on
a numeric field value. It cannot be expressed as a static geometry frame in `region_frames_for`.
The correct mechanism is a new `ContentBlock` variant that carries the numeric proportion, and a
corresponding new `FrameContent` variant that the layout engine computes proportional geometry
for at layout time.

**`slideforge-types` change:** Add to `ContentBlock` enum:
```rust
/// A color-bar fill directive: used by progress_bar to encode the proportional fill.
/// The `percent` field (0–100) drives the fill width at layout time.
/// Accessibility: the label is separately threaded as ColorLabel; this block is
/// geometry-only and does not carry text.
ColorBar(ColorBarSpec),
```

And new type:
```rust
/// Parameters for a color-bar fill frame (progress_bar slide type).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColorBarSpec {
    /// Fill percentage in [0, 100]. Drives the proportional width at layout time.
    pub percent: u8,
}
```

**`slideforge-layout/src/types.rs` change:** Add to `FrameContent` enum:
```rust
/// A progress-bar fill frame with proportional width.
///
/// The `filled_width_emu` is computed by the layout engine as:
///   `(percent as i64 * bar_background_width_emu) / 100`
/// rounded to integer EMU. `total_width_emu` is the bar background frame's width
/// (carried for exporter convenience to compute relative fill without re-reading geometry).
ColorBar {
    /// The computed fill width in EMU.
    filled_width_emu: Emu,
    /// The total bar width in EMU (= bar background frame width).
    total_width_emu: Emu,
    /// The fill color derived from the brand's primary color, or a fixed default.
    color: Rgb,
}
```

All new types derive `Hash + Eq + Clone + Debug` (comemo AC-010 requirement).

### 4.4 layout::run — ColorBar Materialization Pass

After the `fill_region_slot_or_append` loop (which handles `ContentBlock::Text` and `ContentBlock::Bullets`),
add a targeted pass for `ContentBlock::ColorBar`:

```rust
// ColorBar materialization pass — progress_bar only.
// Finds the bar-background Generic frame (index 1 for progress_bar), computes the
// proportional fill width from the ColorBarSpec.percent, and REPLACES the Generic
// frame's content with FrameContent::ColorBar{...}.
// This pass runs only when the slide contains a ContentBlock::ColorBar.
for block in &slide.blocks {
    if let ContentBlock::ColorBar(spec) = &block.content {
        // Find the first FrameContent::Empty with RegionRole::Generic (bar-background slot).
        if let Some(frame) = all_frames.iter_mut().find(|f| {
            matches!(f.content, FrameContent::Empty)
                && f.region_role == Some(RegionRole::Generic)
        }) {
            let total_width = frame.bbox.width;
            let filled_width = Emu((spec.percent as i64 * total_width.0) / 100);
            // Brand primary color: use brand.colors.primary if available, else #0070C0 (blue).
            let color = brand.colors.as_ref()
                .and_then(|c| c.primary)
                .unwrap_or(Rgb { r: 0, g: 112, b: 192 });
            frame.content = FrameContent::ColorBar {
                filled_width_emu: filled_width,
                total_width_emu: total_width,
                color,
            };
        }
        break; // at most one ColorBar per slide
    }
}
```

This is a pure, contained addition. It only mutates frames for slides that contain a
`ContentBlock::ColorBar`. No existing slide type produces that block. Blast radius = zero.

### 4.5 RegionRole Precision for Color-Coded Slots

Current `regions.rs` assignments for the three types:

| Type | Frame | Current RegionRole |
|------|-------|--------------------|
| status | 0 (color strip) | Generic |
| status | 1 (label + title) | Body |
| progress_bar | 0 (title) | Title |
| progress_bar | 1 (bar bg) | Generic |
| progress_bar | 2 (label) | Body |
| weighted_composite | 0 (title) | Title |
| weighted_composite | 1 (agg label) | Body |
| weighted_composite | 2–6 (rows) | Generic (×5) |

These assignments are already correct. The `TextTag::Title` blocks (produced by the generic
title pass in Stage 2b) will claim the Title-role slots. The `TextTag::ColorLabel` blocks
(produced by the new label-threading pass) will claim the Body-role slots via the updated
`fill_region_slot_or_append` — which now maps `TextTag::ColorLabel → RegionRole::Body`.

The `TextTag::Body` blocks produced for weighted_composite component rows will claim the
five Generic-role row slots in registration order (first Generic slot → first component,
second Generic slot → second component, etc.), which is correct because `fill_region_slot_or_append`'s
generic-fallback path claims the first remaining Generic/None Empty slot sequentially.

**No changes needed to `regions.rs`** — the geometry is already correct. Only Stage 2b and
the `TextTag::ColorLabel` mapping require changes.

### 4.6 fill_region_slot_or_append — TextTag::ColorLabel mapping

In `layout.rs`, `fill_region_slot_or_append` maps `TextTag::Title → RegionRole::Title`,
`TextTag::Subtitle → RegionRole::Subtitle`, `TextTag::Body → RegionRole::Body`. Add:

```rust
TextTag::ColorLabel => crate::types::RegionRole::Body,
```

This routes `ColorLabel`-tagged blocks to the Body-role slot, same as regular Body blocks,
but with distinct semantic intent preserved on the frame for exporters.

In `layout::run`'s `ContentBlock::Text` match arm, add `TextTag::ColorLabel` handling
analogous to `TextTag::Body`:

```rust
TextTag::ColorLabel => {
    let text = extract_inline_text_str(&text_block.inlines);
    let content = crate::types::FrameContent::Body(vec![ContentBlock::Text(
        text_block.clone(),
    )]);
    fill_region_slot_or_append(
        &mut all_frames,
        TextTag::ColorLabel,
        content,
        &text_block.inlines,
        page_size,
        source_index,
    )?;
},
```

This re-uses `FrameContent::Body` as the content variant — the semantic differentiation
is carried by `TextTag::ColorLabel` on the `TextBlock` inside it, which exporters can
inspect to apply label-specific rendering (font size, color, style).

---

## 5. Blast-Radius Analysis and Containment

| Change | Files Affected | Blast Radius |
|--------|----------------|--------------|
| `TextTag::ColorLabel` new variant | `slideforge-types/src/lib.rs` or `text.rs` | Low: additive; existing match arms with `_` unaffected; explicit-arm match in `layout.rs` needs 1 new arm |
| `ContentBlock::ColorBar(ColorBarSpec)` + `ColorBarSpec` | `slideforge-types/src/lib.rs` | Low: additive; existing match arms in `field_to_block.rs`, `layout.rs`, exporters all have `_ => {}` catch-alls |
| `FrameContent::ColorBar{...}` | `slideforge-layout/src/types.rs` | Low: additive; existing match arms in exporters have `_ => {}` or explicit exhaustive arms; each exporter must handle new variant (see Section 6) |
| `thread_fields_to_blocks` extension | `slideforge-eval/src/field_to_block.rs` | Zero on existing types: new match arms for `"status" \| "progress_bar" \| "weighted_composite" \| "stat_callout"` only; existing arms unchanged |
| ColorBar materialization pass in `layout::run` | `slideforge-layout/src/layout.rs` | Zero on existing types: the pass searches `ContentBlock::ColorBar` which no existing slide produces |
| `fill_region_slot_or_append` new arm | `slideforge-layout/src/layout.rs` | Zero on existing types: no existing `ContentBlock` produces `TextTag::ColorLabel` |

**Overall blast radius: LOW.** All changes are strictly additive. No existing code path is
modified; only new match arms and new conditional passes are added.

---

## 6. Exporter Handling for FrameContent::ColorBar

Exporters that do not yet handle `FrameContent::ColorBar` must have a stub added. The
stub must render visibly (not silently skip) to prevent silent content loss:

- **PPTX exporter (`slideforge-pptx`):** Render a `<p:sp>` shape element with `<a:solidFill>`
  using `color.r/g/b`, positioned at the bar-background frame's `bbox.x, bbox.y`, with
  `width = filled_width_emu` and `height = frame.bbox.height`. The unfilled portion requires
  no element (slide background shows through). This is a single solid-fill rectangle — a
  well-understood OOXML primitive.

- **PDF and HTML exporters:** Render a filled rectangle at the proportional width. These are
  also standard primitives.

- **DOCX exporter:** Render the percentage value as text (e.g., "75%") since DOCX does not
  have a native progress bar element. The `filled_width_emu` is not meaningful in a flow
  document; exporters should fall back gracefully to the label text already present in the
  adjacent `ColorLabel` frame.

The exporter stubs can be marked `// TODO(STORY-NNN): full ColorBar rendering` with a
tracked story reference for refinement, but must NOT silently skip — that would violate the
visible-output requirement of AC-002/008/015. At minimum, a `tracing::warn!` must be emitted
if the variant is skipped.

---

## 7. Ruling on F-087-P2-001 — Empty Components for weighted_composite

**Finding:** Should `weighted_composite` with `components: []` (empty list) emit E-VAL-011?
Is `ValueRangeValidator` the right home? What is the exact message and behavior?

**Ruling: YES — `ValueRangeValidator` IS the correct home. The empty-list check must be
added to `ValueRangeValidator::validate_weighted_composite_components`.**

**Rationale:**

The current `ValueRangeValidator.validate_weighted_composite_components` implementation
silently returns when `components` is an empty list:
```rust
// Absent or wrong type — not a range error; no E-VAL-011.
// (Empty-components validation is a separate concern; range validator
// only checks numeric field ranges when components are present and valid.)
return;
```

This is an error — an empty list is not "absent or wrong type." A `Value::List([])` is a
valid DSL value that passes Stage 2a evaluation but violates BC-1.17.003 postcondition 3:
"The `components` list MUST be non-empty."

AC-019 explicitly requires: `components: []` → `Err(BuildError::ValidationFailed)` with
`Diagnostic { code: "E-VAL-011", .. }` and message indicating "at least one component".

The validator is the correct enforcement point (same rationale as weight/score range checks:
pre-layout, reads Slide.fields, emits structured diagnostic, no change to lay_out()).

**Implementation directive:**

In `validate_weighted_composite_components` in `value_range.rs`, change the early-return
arm for `Value::List(list)` to emit E-VAL-011 when the list is empty:

```rust
Some(FieldValue::Literal(Value::List(list))) if list.is_empty() => {
    diagnostics.push(make_range_error(
        "weighted_composite requires at least one component; got empty list.",
        span,
    ));
    return;
},
Some(FieldValue::Literal(Value::List(list))) => list.as_slice(),
```

**Exact error message:** `"weighted_composite requires at least one component; got empty list."`

**Error code:** `E-VAL-011` (same as other value-range violations — this is a structural
value constraint, not an accessibility violation).

**Severity:** `DiagnosticSeverity::Error` (strict-mode fatal, same as other E-VAL-011).

The comment in the current code that reads "Empty-components validation is a separate concern"
was written as a deferral placeholder without authorization. Under the canonical principle
(no-MVP rule), it must be fixed in scope. This adjudication authorizes and directs that fix.

---

## 8. Implementation Directive Summary

### Files to Create

None. All changes are extensions to existing files.

### Files to Modify

#### `slideforge-types/src/` — TextTag + ContentBlock additions

**Add `ColorLabel` to `TextTag` enum:**
```rust
/// A color-coded slide label — the accessible text co-encoding required by WCAG 1.4.1.
/// Maps to RegionRole::Body in fill_region_slot_or_append. Used by status,
/// progress_bar, and weighted_composite slide types. Routed to FrameContent::Body
/// at layout time; exporters may apply label-specific font styling.
ColorLabel,
```

Add `ColorLabel` to all `match` arms in `slideforge-types` that are exhaustive over `TextTag`.

**Add `ContentBlock::ColorBar(ColorBarSpec)` and `ColorBarSpec`:**
See Section 4.3 for the full definitions. `ColorBarSpec` carries only `percent: u8`.
Add to all exhaustive `match` arms in `slideforge-types`.

#### `slideforge-eval/src/field_to_block.rs` — Stage 2b extension

Add a new dispatch section after the media block (chart/image/diagram section, around line 266):

```rust
// ── 6. Color-coded slide types (status / progress_bar / weighted_composite / stat_callout)
//
// These types have fields beyond title/subtitle/body that carry WCAG co-encoding
// text (label) and numeric state (value, components). Stage 2b threads them into
// ContentBlocks so that layout::run can fill the pre-allocated region slots.
//
// Threading contract:
//   "label"       → ContentBlock::Text(TextTag::ColorLabel) — Body-role slot
//   "value"       → ContentBlock::ColorBar(ColorBarSpec)    — bar fill geometry
//   "components"  → ContentBlock::Text(TextTag::Body) × N  — per-component Generic slots
//   stat fields   → ContentBlock::Text(TextTag::Body) × N  — Generic slots (stat_callout)
```

See Section 4.2 for the per-type threading code. The `compose_component_row_text` helper
is a new pure private function in the same file.

#### `slideforge-layout/src/types.rs` — FrameContent::ColorBar

Add `ColorBar` variant per Section 4.3.

#### `slideforge-layout/src/layout.rs` — ColorBar pass + TextTag::ColorLabel routing

Add `TextTag::ColorLabel` arm in the `ContentBlock::Text` match per Section 4.6.
Add ColorBar materialization pass per Section 4.4.
Update `fill_region_slot_or_append`'s `expected_role` match to include `TextTag::ColorLabel → RegionRole::Body`.

#### `slideforge-validate/src/value_range.rs` — empty-components check

Add the empty-list arm per Section 7. Update the module doc comment to reflect that empty
components is also checked (E-VAL-011).

#### Exporter stubs

Each exporter crate (`slideforge-pptx`, `slideforge-pdf`, `slideforge-html`, `slideforge-docx`)
must handle `FrameContent::ColorBar` — at minimum with a `tracing::warn!` and graceful
fallback; at best with a real filled-rectangle render per Section 6.

---

## 9. Spec Amendments

### 9.1 STORY-087 v1.3 → v1.4

**File:** `.factory/stories/stories/STORY-087-color-coded-slide-types.md`

Bump `spec_version` from `"1.3"` to `"1.4"`. Add changelog entry:

```yaml
- version: "1.4"
  date: "2026-06-06"
  note: "Architect pass-2 adjudication: content rendering mechanism decided (Option T —
         Threading). Stage 2b extended to thread label/value/components into ContentBlocks.
         New TextTag::ColorLabel, ContentBlock::ColorBar(ColorBarSpec), FrameContent::ColorBar.
         AC-002/008/015 now have a load-bearing mechanism for visible output. F-087-P2-001:
         ValueRangeValidator empty-list check added for weighted_composite. Architecture Mapping
         table updated."
```

Update the Architecture Mapping table to add:
- `thread_fields_to_blocks` extension in `slideforge-eval/src/field_to_block.rs`
- New `TextTag::ColorLabel` in `slideforge-types`
- New `ContentBlock::ColorBar(ColorBarSpec)` in `slideforge-types`
- New `FrameContent::ColorBar` in `slideforge-layout/src/types.rs`
- ColorBar materialization pass in `slideforge-layout/src/layout.rs`
- `ValueRangeValidator` empty-list check (AC-019 fix) in `slideforge-validate/src/value_range.rs`

### 9.2 BC-1.17.001/002/003 — Rendering Mechanism Addenda

**Files:** `.factory/specs/behavioral-contracts/BC-1.17.001.md`, `BC-1.17.002.md`, `BC-1.17.003.md`

Add a new postcondition to each BC:

**BC-1.17.001 — add PC-8 (v1.1 → v1.2):**
"PC-8: The `label` field is threaded into `Slide.blocks` as `ContentBlock::Text(TextTag::ColorLabel)`
by `thread_fields_to_blocks` (Stage 2b, ADR-019). At layout time, `fill_region_slot_or_append`
routes it into the `RegionRole::Body` frame. Exporters render the frame as visible text."

**BC-1.17.002 — add PC-9 (v1.1 → v1.2):**
"PC-9: The `label` field is threaded as `ContentBlock::Text(TextTag::ColorLabel)` into the
Body-role frame. The `value` field is threaded as `ContentBlock::ColorBar(ColorBarSpec { percent })`.
The layout engine materialises `ColorBar` into `FrameContent::ColorBar { filled_width_emu, total_width_emu, color }`
by proportional computation at layout time."

**BC-1.17.003 — add PC-9 (v1.1 → v1.2):**
"PC-9: The top-level `label` is threaded as `ContentBlock::Text(TextTag::ColorLabel)` → Body-role frame.
Each component in `components[]` is threaded as `ContentBlock::Text(TextTag::Body)` with composed
text `'<name>: <score>/100 (wt: <weight>) — <label>'` → one Generic-role row frame per component.
A maximum of 5 component rows are rendered; additional components produce trailing `FrameContent::Empty`
Generic slots."

**Version bumps:** BC-1.17.001: `"1.1"` → `"1.2"`. BC-1.17.002: `"1.1"` → `"1.2"`. BC-1.17.003: `"1.1"` → `"1.2"`.
(The `"1.1"` versions are the pass-1 adjudication amendments. This is the pass-2 rendering amendment.)

### 9.3 Error Taxonomy — E-VAL-011 message variant for empty components

**File:** `.factory/specs/prd-supplements/error-taxonomy.md`

The E-VAL-011 entry added in pass-1 adjudication covers "numeric field out of required range
or wrong type." Add a note that E-VAL-011 also covers the structural constraint
"non-empty list required":

> Note: E-VAL-011 is also emitted when `weighted_composite.components` is present but empty
> (`[]`). Message: `"weighted_composite requires at least one component; got empty list."`.
> This is a structural value contract violation, not a type error.

No version bump beyond what pass-1 already specified (`"2.18"`) — this is an addendum
to the existing E-VAL-011 row, not a new code.

---

## 10. Tests the Test-Writer Must Add

### 10.1 Stage 2b Threading Tests (slideforge-eval/src/field_to_block.rs, #[cfg(test)])

| Test name | Setup | Expected |
|-----------|-------|----------|
| `test_status_label_threads_as_color_label` | `status` slide with `label "On Track"` | `ContentBlock::Text(TextTag::ColorLabel)` with text "On Track" in `slide.blocks` |
| `test_status_no_label_no_block` | `status` slide without `label` field | No `ColorLabel` block emitted |
| `test_progress_bar_label_threads_as_color_label` | `progress_bar` slide with `label "75% done"` | `ContentBlock::Text(TextTag::ColorLabel)` in blocks |
| `test_progress_bar_value_threads_as_color_bar` | `progress_bar` slide with `value 75` | `ContentBlock::ColorBar(ColorBarSpec { percent: 75 })` in blocks |
| `test_progress_bar_value_zero_threads_as_color_bar` | `progress_bar` slide with `value 0` | `ContentBlock::ColorBar(ColorBarSpec { percent: 0 })` |
| `test_progress_bar_value_100_threads_as_color_bar` | `progress_bar` slide with `value 100` | `ContentBlock::ColorBar(ColorBarSpec { percent: 100 })` |
| `test_weighted_composite_label_threads_as_color_label` | `weighted_composite` with top-level `label "Overall: Good"` | `ContentBlock::Text(TextTag::ColorLabel)` in blocks |
| `test_weighted_composite_2_components_produce_2_body_blocks` | `weighted_composite` with 2 components | Exactly 2 `ContentBlock::Text(TextTag::Body)` blocks containing component text |
| `test_weighted_composite_component_text_contains_name_score_label` | component with name "Quality" score 85 label "Excellent" | The body text block contains "Quality", "85", and "Excellent" |
| `test_stat_callout_stat1_threads_as_body` | `stat_callout` slide with `stat_1 "$4.2B"` | `ContentBlock::Text(TextTag::Body)` with text "$4.2B" |

### 10.2 Layout Routing Tests (slideforge-layout, unit tests in layout.rs or a new test file)

| Test name | Setup | Expected |
|-----------|-------|----------|
| `test_status_label_fills_body_slot` | `layout::run` with `status` slide having `label` threaded | Body-role frame content is `FrameContent::Body(...)` containing "On Track"; no `FrameContent::Empty` remains in Body slot |
| `test_progress_bar_label_fills_body_slot` | `layout::run` with `progress_bar` having `label` threaded | Body-role frame (frame 2) content is `FrameContent::Body(...)` |
| `test_progress_bar_color_bar_width_proportional` | `layout::run` with `progress_bar` `value 75` | Generic-role frame (frame 1) content is `FrameContent::ColorBar { filled_width_emu, total_width_emu, .. }` where `filled_width_emu == total_width_emu * 75 / 100` |
| `test_progress_bar_color_bar_value_0` | `progress_bar` `value 0` | `filled_width_emu == Emu(0)` |
| `test_progress_bar_color_bar_value_100` | `progress_bar` `value 100` | `filled_width_emu == total_width_emu` |
| `test_weighted_composite_agg_label_fills_body_slot` | `weighted_composite` with aggregate label | Body-role frame content is `FrameContent::Body(...)` containing aggregate label text |
| `test_weighted_composite_2_components_fill_2_generic_slots` | `weighted_composite` with 2 components | Exactly 2 Generic-role slots filled with `FrameContent::Body(...)`; 3 remaining Generic slots are `FrameContent::Empty` |
| `test_weighted_composite_5_components_fill_5_generic_slots` | `weighted_composite` with 5 components | All 5 Generic-role slots filled |

### 10.3 ValueRangeValidator — Empty Components Test

| Test name | Setup | Expected |
|-----------|-------|----------|
| `test_BC_1_17_003_empty_components_is_error` | `weighted_composite` slide with `components: []` (empty `Value::List`) | Exactly 1 `E-VAL-011` diagnostic with message containing "at least one component" |

This test goes in `slideforge-validate/src/value_range.rs` `#[cfg(test)]` block.

### 10.4 Build-Level Tests (AC-002, AC-008, AC-015 visible output gate)

The following build-level tests are the load-bearing proof for AC-002/008/015. They MUST
call `slideforge::build()` and inspect the output for visible text, not merely check
`Ok(BuildOutput)`.

| Test name | DSL input | Expected observable |
|-----------|-----------|---------------------|
| `test_AC_002_status_label_visible_in_output` | `slide status: title "Project Alpha" label "On Track"` | Build returns `Ok`; label text "On Track" is present in at least one `FrameContent::Body(...)` frame in the `LaidOutSlide` (inspect `build_output.laid_out_deck` if exposed, or use a rendered-PPTX XML scan) |
| `test_AC_008_progress_bar_label_and_bar_visible` | `slide progress_bar: title "Sprint 4" label "75% complete" value 75` | Build returns `Ok`; label "75% complete" in a Body frame; `FrameContent::ColorBar { filled_width_emu, .. }` frame exists with `filled_width_emu > Emu(0)` |
| `test_AC_015_weighted_composite_labels_visible` | `slide weighted_composite:` with 2 components | Build returns `Ok`; aggregate label and both component labels visible in frames |

These tests may be in `crates/slideforge/tests/e2e/` or in a new `story_087_content_rendering.rs`
file in that directory. Fixtures may require stub/mock brand and format context.
Use `#[ignore]` with a blocking dependency comment only if the DSL list-of-map syntax
(STORY-088) is required to parse the `weighted_composite components:` block end-to-end.
If so, provide a unit-level equivalent in `field_to_block.rs` tests that directly constructs
a `Slide` with the `components` field pre-populated (bypassing the parser) — per SID-1.

---

## 11. Architecture Compliance Verification

| Requirement | Met? | Notes |
|-------------|------|-------|
| ADR-005 two-IR model (layout::run is sole boundary) | YES | No change to layout::run boundary; Stage 2b is pre-layout |
| ADR-006 plugin-first | YES | SlideType impls unchanged; new behavior is in Stage 2b and layout::run passes |
| ADR-019 Decision 3 (Stage 2b threading) | YES | Extension follows the exact chartered mechanism |
| Comemo Hash+Eq+Clone on all new IR types | YES | ColorBarSpec, FrameContent::ColorBar both derive required traits |
| Integer EMU (no f64) | YES | ColorBar geometry uses Emu(i64) integer arithmetic |
| Kani/VP-011 purity preserved | YES | No dynamic dispatch in layout::run; all additions are data-driven |
| BC-1.17.001/002/003 AC-002/008/015 visible output | YES | Threading + fill_region_slot + ColorBar materialization ensures visible frames |
| Zero regression on 34 existing types | YES | All changes are additive; existing type code paths unchanged |
| DI-018 error accumulation | YES | Empty-components check is a single-error emit (list is empty); component iteration already accumulates |
| TD-VSDD-060 sibling-site sweep | REQUIRED | Implementer must grep `TextTag` and `ContentBlock` match arms in all crates before committing |

---

## 12. Decision Log

| Date | Agent | Action |
|------|-------|--------|
| 2026-06-06 | architect | Confirmed stat_callout does NOT solve the pattern — it has the same gap |
| 2026-06-06 | architect | Option A (lay_out routing) re-rejected for pass-2 context — same reasons as pass-1 |
| 2026-06-06 | architect | Option L (scoped lay_out) rejected — structural incompatibility with pipeline state, ADR-005 violation even for subset |
| 2026-06-06 | architect | Option T (threading) selected — additive, pure, zero blast radius, architecturally correct per ADR-019 |
| 2026-06-06 | architect | TextTag::ColorLabel new variant decided — semantic distinction from Body; maps to RegionRole::Body |
| 2026-06-06 | architect | ContentBlock::ColorBar + ColorBarSpec decided — pure carrier for proportional geometry data |
| 2026-06-06 | architect | FrameContent::ColorBar decided — layout-time materialized geometry; brand color wired |
| 2026-06-06 | architect | stat_callout gains threading as a free side-effect (no separate change needed) |
| 2026-06-06 | architect | F-087-P2-001: empty-components → E-VAL-011 via ValueRangeValidator confirmed; deferral comment in existing code overridden under canonical principle no-MVP rule |
| 2026-06-06 | architect | BC-1.17.001/002/003 v1.2 amendments + STORY-087 v1.4 spec bump specified |
