---
document_type: architect-adjudication
story_id: STORY-086
adversary_pass: 1
date: 2026-06-06
adjudicator: vsdd-factory:architect
status: COMPLETE
issues_count: 3
issues_resolved: 3
scope_change: "13 pts → 21 pts (human-authorized)"
bc_amendments: "BC-4.01.001 → v1.2, BC-4.02.001 → v1.2 (TextTag routing)"
---

# STORY-086 Architect Pass 1 Adjudication

## Summary

Three issues from adversary pass 1 adjudicated. All resolved without BC
amendment except BC-4.01.001/4.02.001 which were amended to v1.2 to specify
TextTag routing. Human authorized scope expansion from 13 → 21 pts for
Issue 2 (TextTag).

---

## Issue 1 — F-086-P1-CRIT-001: Media Block Emission + AltText Validator Scope

**Adversary finding:** Stage 2b (`field_to_block.rs`) skips emitting
`ContentBlock::Chart/Image/Diagram` when `alt = None`, violating BC-1.16.001
PC-9/10/11. Implementer rationale: prevents E-A11-001 double-fire (pre-layout
`AltTextValidator::validate()` over `Slide.blocks` treats `AltText::None` as
missing, AND post-layout `validate_post_layout()` over `AltText::Unspecified`
frames both fire → 2× error, violating AC-005 "exactly one").

**Verdict: CODE-CONFORMS**

The implementer identified a real structural conflict between two validators. The
resolution is not to suppress the block — it is to restrict the pre-layout
validator's scope.

**Resolution (no BC amendment):**

1. **Emit alt=None media blocks per BC-1.16.001 PC-9/10/11.** Stage 2b MUST
   emit `ContentBlock::Chart`, `ContentBlock::Image`, and `ContentBlock::Diagram`
   even when `alt = None`. These blocks carry `AltText::Unspecified` to signal
   the post-layout pass that resolution is pending.

2. **Fix `AltTextValidator::validate()` to own ONLY `ContentBlock::Shape`.**
   Per ADR-018 v1.2 Decision-3 (amendment effective this session):
   Chart/Image/Diagram alt validation is post-layout-only. The pre-layout
   validator fires for Shape blocks (which ARE fully resolved at Stage 2b time)
   and MUST NOT fire for Chart/Image/Diagram (which are resolved by
   `thread_media_alt_into_frames` at Stage 3).

3. **Single-fire guarantee for AC-005:**
   - Shape: pre-layout validates → post-layout skips (Shape blocks have no frame
     counterpart of the same type in `LaidOutSlide`).
   - Chart/Image/Diagram: pre-layout skips → post-layout validates via
     `validate_post_layout()` on `AltText::Unspecified` frames.
   - No element fires twice. AC-005 is satisfied.

**Implementer actions:**
- Invert `test_chart_no_alt_skips_block`: currently asserts the block is NOT
  emitted; must now assert the block IS emitted with `alt = AltText::Unspecified`.
- Move or remove tests in `alt_text.rs` that assert `validate()` fires on
  `ContentBlock::Chart`, `ContentBlock::Image`, `ContentBlock::Diagram` inputs —
  these are now out-of-scope for the pre-layout validator.
- Add or verify a test confirming post-layout `validate_post_layout()` fires
  E-A11-001 for an `AltText::Unspecified` Chart/Image/Diagram frame.
- **No BC amendment required.** BC-1.16.001 already specifies emit; ADR-018 v1.2
  Decision-3 amendment governs validator scope.

---

## Issue 2 — F-086-P1-MED-002 + F-086-P1-MED-003: TextTag Mechanism (IMPLEMENT NOW)

**Adversary finding (MED-002):** Title text lands in a generic body shape, not
the PPTX title placeholder `<p:ph type="title">`. BC-1.16.001 PC-1 assumed a
`TextTag` mechanism that was never built. `TextBlock` has no `tag` field.

**Adversary finding (MED-003):** DOCX "first non-empty TextRun = title"
positional fallback is fragile (title-less slide promotes body to Heading1).
Same root cause: `TextTag` never built.

**Human decision:** IMPLEMENT NOW in STORY-086 (not defer).

**Verdict: IMPLEMENT — scope expanded 13 → 21 pts (human-authorized 2026-06-06)**

**Resolution:**

Implement the `TextTag` enum and `TextBlock.tag` field across the pipeline.
Scope covers ~7 crates and the following sub-tasks:

### TextTag Enum (new, in `slideforge-types` or `slideforge-eval`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextTag {
    Title,
    Subtitle,
    Body,
}
```

### TextBlock.tag field (add to `slideforge-types::TextBlock`)

```rust
pub struct TextBlock {
    pub tag: Option<TextTag>,
    // ... existing fields ...
}
```

`None` means untagged (renderer uses body-default). All existing TextBlock
construction sites that do not set `tag` are valid — they produce `None` and
route to body.

### Stage 2b tagging (in `field_to_block.rs`)

When emitting `ContentBlock::Text` from a slide field:
- Field named `title` → `tag: Some(TextTag::Title)`
- Field named `subtitle` → `tag: Some(TextTag::Subtitle)`
- Field named `body` or unrecognized text field → `tag: Some(TextTag::Body)`
  (or `None` for full generics)

### Layout routing (in `slideforge-layout`)

Layout title-frame routing: when `ContentBlock::Text` has `tag = Some(TextTag::Title)`,
route it to the title-placeholder frame (identified by layout frame type
`FrameType::TitlePlaceholder`). Body and subtitle route to their respective
placeholder frame types.

### PPTX serialization (in `slideforge-pptx`)

Title-tagged frame → `<p:ph type="title">` on the `<p:sp>`.
Subtitle-tagged frame → `<p:ph type="subTitle">`.
Body-tagged frame → `<p:ph type="body">` (or no `type` attr for default body ph).

### DOCX serialization (in `slideforge-docx`)

Replace the positional fallback heuristic entirely:
- `TextTag::Title` → `Heading1` paragraph style
- `TextTag::Subtitle` → `Heading2` paragraph style
- `TextTag::Body` (or untagged) → `Normal` paragraph style

This removes the brittleness of "first non-empty TextRun = title."

### BC amendments (no behavioral change, spec catch-up)

- **BC-4.01.001 → v1.2:** Add explicit routing rule: `TextTag::Title` →
  title-placeholder frame. `TextTag::Subtitle` → subtitle-placeholder.
  `TextTag::Body` / untagged → body-placeholder. Previous v1.1 had no tag
  concept; this is additive, not contradictory.
- **BC-4.02.001 → v1.2:** Replace positional fallback with tag-based routing.
  `TextTag::Title` → Heading1; `TextTag::Subtitle` → Heading2;
  `TextTag::Body` → Normal. Mark positional fallback as **deprecated** (must
  not appear in new code).

**No amendment to BC-1.16.001** — it already specifies `tag` on TextBlock in
PC-1; the spec was ahead of the implementation.

---

## Issue 3 — F-086-P1-MED-001: Bullets List-Literal DSL Syntax (STORY-088)

**Adversary finding:** AC-007 bullets fixture used title/subtitle/body — the
`bullets: [...]` direct list-literal DSL syntax does NOT parse.
`slideforge-syntax` field-value parser (`deck.rs`) lacks a list-literal arm.
`ContentBlock::Bullets` path is never exercised e2e.

**Verdict: SCOPE-BOUNDED — AC-007 variable-binding form + STORY-088 for parser**

**Resolution:**

1. **Within STORY-086 (AC-007):** Test fixture must use the variable-binding form:
   ```sf
   @var items = ["First bullet", "Second bullet", "Third bullet"]
   bullets: items
   ```
   Add a `Value::List` unit test confirming `@var` with a list literal is parsed
   and passed correctly through eval.

2. **STORY-088 (Wave 5, 5 pts, new story):** Add a list-literal arm to the
   `slideforge-syntax` field-value parser so `bullets: ["a", "b", "c"]` parses
   inline without requiring `@var`. This is a parser feature, not a blocker for
   STORY-086's behavioral correctness.

**Rationale for deferral of list-literal syntax:** The variable-binding form
already works and covers the AC-007 behavioral requirement (ContentBlock::Bullets
is emitted, layout processes it, exporters serialize it). The inline list-literal
is a DSL usability improvement, not a correctness gate. Per production-grade
principle, the AC-007 test MUST exercise the real path — use `@var` form. The
list-literal parser work is a self-contained parser story.

---

## Streak Reset

Adversary pass 1 resulted in 4 active findings (1 CRIT + 3 MED). All findings
adjudicated and resolved within the same session. Streak is **0/3**.

Next action for the cascade:

1. test-writer adds Red Gate tests for TextTag ACs:
   - AC-019: title field → `<p:ph type="title">` in PPTX
   - AC-020: body field → body ph in PPTX
   - AC-021: DOCX title → Heading1 via tag (not positional)
   - AC-022: subtitle field → `<p:ph type="subTitle">` in PPTX / Heading2 in DOCX
   - AC-023: tag-over-position (body-only slide does NOT promote body to Heading1)
   - AC-007 (updated): variable-binding bullets fixture + Value::List unit test
   - All must FAIL on current HEAD ba3bcc93 (tag-less implementation)

2. implementer fix-burst:
   - Issue 1: emit alt=None media blocks + restrict validate() to Shape + invert
     `test_chart_no_alt_skips_block` + remove/move pre-layout Chart/Image/Diagram
     tests from `alt_text.rs`
   - Issue 2: TextTag enum + TextBlock.tag across ~7 crates + Stage 2b tagging +
     layout title-frame routing + PPTX `<p:ph>` + DOCX Heading1/Heading2/Normal
     (replace positional fallback). BC-4.01.001 → v1.2, BC-4.02.001 → v1.2.
   - Issue 3: AC-007 variable-binding fixture + Value::List unit test

3. Resume adversary LOCAL cascade from pass 2 (streak 0/3 → target 3/3).

4. demo → PR → security-reviewer + pr-reviewer → merge.

5. Re-run Wave 4 gates (Gate 3 + Gate 5) on updated develop.
