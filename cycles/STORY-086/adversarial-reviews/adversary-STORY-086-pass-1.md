---
document_type: adversary-pass-report
story_id: STORY-086
pass: 1
date: 2026-06-06
branch: feature/STORY-086
branch_head_at_review: ba3bcc93
develop_base: 030dec6c
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 4
findings_open: 0
findings_remediated: 4
adjudication_file: architect-pass-1-adjudication.md
---

# STORY-086 Adversary Pass 1 Report

## Pass Metadata

| Field | Value |
|-------|-------|
| Branch | `feature/STORY-086` |
| Branch HEAD at review | `ba3bcc93` |
| develop base | `030dec6c` |
| Pass date | 2026-06-06 |
| CLEAN (strict) | **no** — 1 CRIT + 3 MED + 5 OBS |
| CLEAN (PR-merge) | **no** — CRIT present |
| Streak | 0/3 (reset) |
| Adjudication | See `architect-pass-1-adjudication.md` — all findings adjudicated same session |

## Findings Summary

| ID | Severity | Status | Title |
|----|----------|--------|-------|
| F-086-P1-CRIT-001 | CRIT | RESOLVED — code-conforms | Stage 2b skips emitting alt=None media ContentBlocks, violating BC-1.16.001 PC-9/10/11 |
| F-086-P1-MED-001 | MED | RESOLVED — scope-bounded (STORY-088) | AC-007 bullets e2e path never exercised; direct list-literal DSL syntax unparseable |
| F-086-P1-MED-002 | MED | RESOLVED — implement-now (TextTag, absorbed into STORY-086) | Title text lands in generic body shape, not PPTX title placeholder `<p:ph type="title">` |
| F-086-P1-MED-003 | MED | RESOLVED — implement-now (TextTag, absorbed into STORY-086) | DOCX "first non-empty TextRun = title" positional fallback is fragile |
| OBS-086-P1-001 | OBS | VERIFIED-CORRECT | AltText state machine (regions/thread/validate) correctly designed |
| OBS-086-P1-002 | OBS | VERIFIED-CORRECT | AC-016 comment fixes correct |
| OBS-086-P1-003 | OBS | VERIFIED-CORRECT | Stage 2b wiring correct (AC-017) |
| OBS-086-P1-004 | OBS | VERIFIED-CORRECT | Pure-core architecture correct (ADR-019 D7) |
| OBS-086-P1-005 | OBS | VERIFIED-CORRECT | AltTextValidator::validate_post_layout hook correct |

---

## Critical Findings

### F-086-P1-CRIT-001 — Stage 2b skips emitting alt=None media ContentBlocks

**Severity:** CRITICAL
**Status:** RESOLVED — code-conforms (architect adjudication)
**BC:** BC-1.16.001 PC-9, PC-10, PC-11

**Finding:**

Stage 2b (`field_to_block.rs`) was intentionally written to SKIP emitting
`ContentBlock::Chart`, `ContentBlock::Image`, and `ContentBlock::Diagram` when
`alt = None`. The implementer rationalized this as prevention of an
E-A11-001 double-fire:

- Pre-layout: `AltTextValidator::validate()` traverses `Slide.blocks` and treats
  `AltText::None` as a missing-alt error (E-A11-001).
- Post-layout: `validate_post_layout()` traverses `LaidOutSlide` frames and fires
  the same E-A11-001 for any `AltText::Unspecified` frame.

If both fire for the same element, AC-005 ("exactly one E-A11-001 per missing-alt
element") would be violated. Skipping the block emission prevented double-fire
by never creating the block, but violated BC-1.16.001 PC-9/10/11 which require
those ContentBlocks to be emitted regardless of alt status so exporters can
produce the visual content.

**Root cause:** The pre-layout `AltTextValidator::validate()` owned too broad a
scope — it claimed Chart/Image/Diagram in addition to Shape.

**Resolution:** See `architect-pass-1-adjudication.md` Issue 1 (CODE-CONFORMS).
Emit the alt=None blocks per BC-1.16.001. Fix
`AltTextValidator::validate()` to own ONLY `ContentBlock::Shape`.
Chart/Image/Diagram are post-layout-only per ADR-018 v1.2 Decision-3 amendment
→ single-fire → AC-005 satisfied. No BC amendment required.

**Scope impact for implementer:**
- Invert `test_chart_no_alt_skips_block` (currently asserts skip → must assert emit)
- Move or remove `alt_text.rs` tests that assert `validate()` fires on
  Chart/Image/Diagram
- Implementer exit-gate must confirm single-fire with a load-bearing test

---

## Medium Findings

### F-086-P1-MED-001 — AC-007 bullets e2e path untested; direct list-literal DSL syntax unparseable

**Severity:** MED
**Status:** RESOLVED — scope-bounded (STORY-088, Wave 5, 5 pts)
**BC:** BC-1.16.001 PC-8 (bullets ContentBlock)

**Finding:**

The AC-007 fixture in the test suite uses a slide with `title`, `subtitle`, and
`body` fields — it DOES NOT exercise the `bullets: [...]` path. The
`ContentBlock::Bullets` production path is therefore not exercised end-to-end.

Root cause: the direct `bullets: [...]` DSL list-literal syntax DOES NOT PARSE.
`slideforge-syntax/src/field_value/deck.rs` has no list-literal arm in its
field-value parser. The `bullets:` field is only reachable via the variable-binding
form:

```sf
@var items = ["item1", "item2"]
bullets: items
```

The AC-007 test fixture therefore cannot exercise the direct-literal path because
it does not exist yet in the parser.

**Resolution:** See `architect-pass-1-adjudication.md` Issue 3.
For STORY-086: AC-007 acceptance criterion must use the variable-binding form
(`@var items=[...]; bullets: items`) + add a `Value::List` unit test.
List-literal DSL syntax (`bullets: [...]` inline) is deferred to STORY-088
(Wave 5, 5 pts) where the parser will gain the list-literal arm.

---

### F-086-P1-MED-002 — Title text lands in generic body shape, not PPTX title placeholder

**Severity:** MED
**Status:** RESOLVED — implement-now (TextTag mechanism, absorbed into STORY-086 scope 13→21 pts)
**BC:** BC-1.16.001 PC-1; BC-4.01.001 (pre-v1.2)

**Finding:**

When Stage 2b emits a `ContentBlock::Text` for the slide title field, the block
carries no semantic tag indicating it is a title. The layout engine routes all
`ContentBlock::Text` blocks to generic body-text frames. The PPTX exporter then
serializes these frames as generic `<p:sp>` shapes without `<p:ph type="title">`.

This means:
- PowerPoint/Keynote/Google Slides will NOT recognize the slide title
- Title placeholder animations and master-layout inheritance are broken
- Accessibility: title placeholder is a required PPTX structural landmark

Root cause: `TextBlock` has no `tag` field. The `TextTag` enum (Title/Subtitle/Body)
specified in BC-1.16.001 PC-1 and expected by the layout engine's title-frame
routing was never built.

**Resolution:** See `architect-pass-1-adjudication.md` Issue 2 (IMPLEMENT NOW,
human-authorized). TextTag enum + `TextBlock.tag` field across ~7 crates;
Stage 2b tags blocks; layout routes `TextTag::Title` → title-placeholder frame;
PPTX `<p:ph type="title">`/`subTitle`/`body`; DOCX Heading1/Heading2/Normal
(replaces positional fallback). BC-4.01.001 → v1.2, BC-4.02.001 → v1.2.

---

### F-086-P1-MED-003 — DOCX "first non-empty TextRun = title" positional fallback is fragile

**Severity:** MED
**Status:** RESOLVED — implement-now (TextTag mechanism, absorbed into STORY-086 scope 13→21 pts)
**BC:** BC-4.02.001 (pre-v1.2)

**Finding:**

The DOCX exporter uses a positional heuristic: "the first non-empty
`TextRun` in a slide's content becomes the Heading1 (title)." This breaks when:
- A slide has no title field (body-only slide → first body paragraph promoted to
  Heading1 incorrectly)
- A slide has a subtitle before the body (subtitle becomes Heading1)
- Any content reordering occurs in Stage 2b

Root cause: same as F-086-P1-MED-002 — `TextBlock` has no `tag` field; the
`TextTag::Title` → DOCX `Heading1` routing path was never built.

**Resolution:** Same as F-086-P1-MED-002. `TextTag` enum fix eliminates the
positional fallback entirely; DOCX exporter uses `TextTag::Title` → `Heading1`,
`TextTag::Subtitle` → `Heading2`, `TextTag::Body` → `Normal` (or paragraph-style
per content type). BC-4.02.001 → v1.2 specifies this routing.

---

## Observations (Verified Correct)

### OBS-086-P1-001 — AltText state machine correctly designed

**Severity:** OBS
**Status:** VERIFIED-CORRECT

The `AltText` enum's three-region state machine
(`AltText::None` / `AltText::Unspecified` / `AltText::Text(s)`) is
correctly designed. `None` is the pre-Stage-2b sentinel; `Unspecified` is the
post-Stage-2b sentinel for frames whose alt has not been threaded yet;
`Text(s)` is the resolved value. The `thread_media_alt_into_frames` pass
correctly transitions `Unspecified → Text` or leaves `Unspecified` for the
post-layout validator to catch. No defect.

### OBS-086-P1-002 — AC-016 comment fixes correct

**Severity:** OBS
**Status:** VERIFIED-CORRECT

The comment corrections introduced for AC-016 (misleading `for_eval.rs:336-341`
comment and related prose) are accurate and match the actual semantics of the
post-Stage-2b pipeline. No regression.

### OBS-086-P1-003 — Stage 2b wiring correct for AC-017

**Severity:** OBS
**Status:** VERIFIED-CORRECT

The wiring of Stage 2b output into the pipeline (passing `ContentBlock` vec
from `field_to_block.rs` into `Slide.blocks` for the downstream layout pass)
is correctly implemented per AC-017. The defect is in what blocks are emitted
(CRIT-001), not in how they are wired.

### OBS-086-P1-004 — Purity architecture correct (ADR-019 D7)

**Severity:** OBS
**Status:** VERIFIED-CORRECT

Stage 2b (`field_to_block.rs`) is correctly a pure function with no I/O side
effects, as required by ADR-019 Decision 7. All file I/O for media is deferred
to the export stage. No violation.

### OBS-086-P1-005 — AltTextValidator::validate_post_layout hook correct

**Severity:** OBS
**Status:** VERIFIED-CORRECT

The post-layout hook `validate_post_layout()` correctly traverses
`LaidOutSlide` frames (not `Slide.blocks`) and fires E-A11-001 for any frame
with `AltText::Unspecified`. This is the correct single-fire point for
Chart/Image/Diagram alt enforcement per the intended ADR-018 v1.2 Decision-3
amendment. No defect in the hook design itself.

---

## Pass Summary

**CLEAN (strict):** no — 1 CRIT + 3 MED found. Streak reset to 0/3.
**CLEAN (PR-merge):** no — CRIT present.

All 4 active findings were adjudicated within the same session:
- F-086-P1-CRIT-001: code-conforms resolution. Implementer must invert
  `test_chart_no_alt_skips_block` and remove/move pre-layout Chart/Image/Diagram
  tests from `alt_text.rs`.
- F-086-P1-MED-001: scoped to STORY-088 (Wave 5). AC-007 uses variable-binding
  form + Value::List unit test.
- F-086-P1-MED-002 + F-086-P1-MED-003: TextTag mechanism implemented in STORY-086
  (scope expanded 13→21 pts, human-authorized).

Next action: test-writer adds Red Gate tests for TextTag ACs + AC-007
variable-binding bullets + Value::List unit test. All must FAIL on current
HEAD ba3bcc93.
