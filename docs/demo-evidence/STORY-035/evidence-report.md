# Evidence Report: STORY-035 — Writing Register Routing in Evaluator

**Story ID:** STORY-035
**Epic:** EPIC-18
**Crate:** `slideforge-eval` (routing pass) + `slideforge-layout` (no-bleed gate)
**BC Trace:** BC-1.14.001, BC-1.14.002, BC-1.14.003, BC-1.14.004
**HEAD SHA verified:** `feature/S-035` branch
**Workspace test count (slideforge-eval):** 226 tests run: 226 passed, 0 skipped
**Recording tool:** VHS 0.10.0 (terminal — library story, nextest harness demos)
**Font:** FiraCode Nerd Font Mono

> **Library-only story note:** STORY-035 implements the register routing pass at the
> library level only. There is no CLI, no web surface, and no visual output — exporters
> that consume `register_content` are deferred to Wave 4 (STORY-037, 040, 041, 042).
> All demos invoke the AC-specific tests via `cargo nextest run -p slideforge-eval` or
> `cargo nextest run -p slideforge-layout`. This follows the established pattern for
> library stories (see STORY-024 for precedent).

---

## Coverage Summary

All 8 acceptance criteria have recorded demo evidence. AC-001/003/005/007/008 demo the
extraction and tagging side (`slideforge-eval`, `register_routing.rs`). AC-002/004/006
demo the no-bleed side (`slideforge-layout`, layout frame construction). AC-008 unifies
both concerns in a single end-to-end integration test through `eval_deck`.

| AC | Description | Demo File | Tests Covered | Status |
|----|-------------|-----------|--------------|--------|
| AC-001 | `notes` fields extracted and tagged into `register_content` at eval stage | `AC-001-notes-extracted-and-tagged` | `test_bc_1_14_001_notes_extracted_and_tagged`, `test_bc_1_14_001_notes_content_captured` | PASSED |
| AC-002 | `notes` content excluded from visual frames (no-bleed) | `AC-002-notes-no-bleed-to-frames` | `test_bc_3_06_001_register_tags_notes`, `test_f004_no_bleed_register_text_not_in_frames` | PASSED |
| AC-003 | `report` fields extracted and tagged into `register_content` at eval stage | `AC-003-report-extracted-and-tagged` | `test_bc_1_14_002_report_extracted_and_tagged`, `test_bc_1_14_002_report_content_captured_correctly` | PASSED |
| AC-004 | `report` content excluded from visual frames (no-bleed) | `AC-004-report-no-bleed-to-frames` | `test_f004a_collect_frame_text_positive_control`, `test_f004_no_bleed_register_text_not_in_frames` | PASSED |
| AC-005 | `detail` fields extracted and tagged at eval stage (slide-level only; section-level in STORY-077) | `AC-005-detail-extracted-and-tagged` | `test_bc_1_14_003_detail_extracted_and_tagged`, `test_bc_1_14_003_detail_content_captured_correctly` | PASSED |
| AC-006 | `detail` content excluded from visual frames (no-bleed) | `AC-006-detail-no-bleed-to-frames` | `test_bc_3_06_001_register_tags_detail`, `test_f004_no_bleed_register_text_not_in_frames` | PASSED |
| AC-007 | `{{ expr }}` interpolation resolved BEFORE register tagging | `AC-007-interpolation-resolved-before-tagging` | `test_bc_1_14_001_interpolation_resolved_before_tagging`, `test_f_p3_002_interpolation_resolved_before_register_tagging_eval_deck` | PASSED |
| AC-008 | All three registers on one slide: exactly 3 entries; frames carry only visual content | `AC-008-all-three-registers-one-slide` | `test_bc_1_14_004_all_three_registers_on_one_slide`, `test_f003_eval_deck_populates_register_content_from_all_three_fields` | PASSED |

---

## AC-001: notes Fields Extracted and Tagged

**File:** `AC-001-notes-extracted-and-tagged.{tape,gif,webm}`
**BC trace:** BC-1.14.001 invariant 1 — routing determined at Evaluate stage

`extract_register_content(&slide)` inspects `slide.fields` for the `"notes"` key.
When the key is present and non-null, it produces a `RegisteredContent { register: Register::Notes, content: ... }`
entry. The two tests cover: (1) correct `Register::Notes` tag produced, (2) plain text
content captured verbatim. Both assertions run without any I/O — pure-core function
on a constructed `Slide` IR.

Tests:
- `test_bc_1_14_001_notes_extracted_and_tagged` — `register_content[0].register == Register::Notes`
- `test_bc_1_14_001_notes_content_captured` — `content` text equals `"Emphasize the growth story"`

---

## AC-002: notes Content Excluded from Visual Frames

**File:** `AC-002-notes-no-bleed-to-frames.{tape,gif,webm}`
**BC trace:** BC-1.14.001 postcondition 1 — notes does NOT appear in slide body shapes

The layout engine uses an ALLOWLIST approach: `layout_run` reads only `title`, `subtitle`,
and `body` from `slide.fields` when constructing `LaidOutSlide.frames`. Register keys are
retained in `slide.fields` but never consulted by the frame builder. Two layout tests prove
this invariant:

- `test_bc_3_06_001_register_tags_notes` — builds a slide with a `notes` field; confirms
  `LaidOutSlide.register_content` is tagged `Notes`; confirms `frames` does NOT contain
  the notes text.
- `test_f004_no_bleed_register_text_not_in_frames` — cross-register no-bleed gate:
  all three registers on one slide; scans every `FrameContent` variant in `frames` for
  register text; asserts none found.

Tests: `test_bc_3_06_001_register_tags_notes`, `test_f004_no_bleed_register_text_not_in_frames`

---

## AC-003: report Fields Extracted and Tagged

**File:** `AC-003-report-extracted-and-tagged.{tape,gif,webm}`
**BC trace:** BC-1.14.002 invariant 1 — report routing determined at Evaluate stage

`extract_register_content` handles `"report"` identically to `"notes"` with
`Register::Report` tagging. The two tests verify:

- `test_bc_1_14_002_report_extracted_and_tagged` — `register_content[0].register == Register::Report`
- `test_bc_1_14_002_report_content_captured_correctly` — content text equals `"Analysis text"`;
  `register == Register::Report`

Tests: `test_bc_1_14_002_report_extracted_and_tagged`, `test_bc_1_14_002_report_content_captured_correctly`

---

## AC-004: report Content Excluded from Visual Frames

**File:** `AC-004-report-no-bleed-to-frames.{tape,gif,webm}`
**BC trace:** BC-1.14.002 postcondition 3 — report NOT in PPTX slide shape

The demo shows the positive control first (`test_f004a_collect_frame_text_positive_control`:
a slide whose visual title text IS reachable via frame scanning, proving the scanner works),
then the no-bleed gate (`test_f004_no_bleed_register_text_not_in_frames`: report text is
NOT reachable via frame scanning). Together these rule out a false negative.

Tests: `test_f004a_collect_frame_text_positive_control`, `test_f004_no_bleed_register_text_not_in_frames`

---

## AC-005: detail Fields Extracted and Tagged (Slide-Level)

**File:** `AC-005-detail-extracted-and-tagged.{tape,gif,webm}`
**BC trace:** BC-1.14.003 invariant 1 — detail routing determined at Evaluate stage

Slide-level `detail` field extraction is in scope. Section-level `detail` (standalone
`section detail:` blocks) is descoped to STORY-077 — the IR type `SectionBlock.body`
needs extension to carry `FieldValue::Inlines` before section-level routing is possible.

Tests:
- `test_bc_1_14_003_detail_extracted_and_tagged` — `register_content[0].register == Register::Detail`
- `test_bc_1_14_003_detail_content_captured_correctly` — content text equals `"Technical appendix text"`

---

## AC-006: detail Content Excluded from Visual Frames

**File:** `AC-006-detail-no-bleed-to-frames.{tape,gif,webm}`
**BC trace:** BC-1.14.003 postcondition 3 — detail NOT in PPTX or web preview

`test_bc_3_06_001_register_tags_detail` directly asserts that `detail` text appears in
`register_content` (tagged `Detail`) and NOT in any `FrameContent` variant. The shared
`test_f004_no_bleed_register_text_not_in_frames` additionally verifies the bleed gate
for all three registers simultaneously.

Tests: `test_bc_3_06_001_register_tags_detail`, `test_f004_no_bleed_register_text_not_in_frames`

---

## AC-007: Interpolation Resolved Before Tagging

**File:** `AC-007-interpolation-resolved-before-tagging.{tape,gif,webm}`
**BC trace:** BC-1.14.001 postcondition 1 — notes content uses `{{ expr }}` interpolation

This demo exercises two complementary test levels:

1. **Unit level** (`test_bc_1_14_001_interpolation_resolved_before_tagging`): verifies
   the precondition that `extract_register_content` receives a fully-evaluated
   `FieldValue::Literal(Value::Str("Quarter: Q1"))` — not a raw `FieldValue::Interpolated`
   or `FieldValue::Expr`. The raw `{{` token must NOT appear in the result.

2. **Integration level** (`test_f_p3_002_interpolation_resolved_before_register_tagging_eval_deck`):
   drives the REAL `eval_deck` pipeline with `notes "Quarter: {{ quarter }}"` and
   `quarter = "Q1"` in scope. Proves that `eval_deck` resolves the template BEFORE
   `extract_register_content` is called. The resulting `register_content[0].content`
   contains `"Quarter: Q1"` — not the raw template string. The test also asserts that
   neither `{{` nor the literal string `quarter` appear in the content.

Tests: `test_bc_1_14_001_interpolation_resolved_before_tagging`, `test_f_p3_002_interpolation_resolved_before_register_tagging_eval_deck`

---

## AC-008: All Three Registers on One Slide

**File:** `AC-008-all-three-registers-one-slide.{tape,gif,webm}`
**BC trace:** BC-1.14.004 invariant 3 — `LaidOutSlide` carries register-tagged content

This is the most important invariant test. Two tests collaborate:

1. **Unit** (`test_bc_1_14_004_all_three_registers_on_one_slide`): a slide with
   `title`, `notes`, `report`, and `detail` fields all populated. After
   `extract_register_content`, `register_content.len() == 3`. Each `Register` variant
   is present. No cross-talk: notes contains `"Speaker: emphasize growth"`, report
   contains `"Detailed narrative for readers"`, detail contains `"Technical appendix"`.

2. **Integration via `eval_deck`** (`test_f003_eval_deck_populates_register_content_from_all_three_fields`):
   proves that `eval_deck` wires `extract_register_content` correctly — the result
   flows through `deck.slides[0].register_content`. This rules out a test-only path
   that bypasses the production wiring. The ordering invariant (Notes < Report < Detail)
   is also asserted at the integration level.

Tests: `test_bc_1_14_004_all_three_registers_on_one_slide`, `test_f003_eval_deck_populates_register_content_from_all_three_fields`

---

## Harness Files Added

No additional harness files were created. All tests used for demos are co-located
unit/integration tests already implemented in STORY-035 as part of the TDD delivery:

| File | Tests |
|------|-------|
| `crates/slideforge-eval/src/register_routing.rs` | AC-001, AC-003, AC-005, AC-007 (unit), AC-008 (unit) |
| `crates/slideforge-eval/src/eval.rs` | AC-007 (integration), AC-008 (integration) |
| `crates/slideforge-layout/src/lib.rs` | AC-002, AC-004, AC-006 |

No production code was modified. No new harness files were added to the worktree.

---

## File Manifest

```
docs/demo-evidence/STORY-035/
├── AC-001-notes-extracted-and-tagged.tape
├── AC-001-notes-extracted-and-tagged.gif
├── AC-001-notes-extracted-and-tagged.webm
├── AC-002-notes-no-bleed-to-frames.tape
├── AC-002-notes-no-bleed-to-frames.gif
├── AC-002-notes-no-bleed-to-frames.webm
├── AC-003-report-extracted-and-tagged.tape
├── AC-003-report-extracted-and-tagged.gif
├── AC-003-report-extracted-and-tagged.webm
├── AC-004-report-no-bleed-to-frames.tape
├── AC-004-report-no-bleed-to-frames.gif
├── AC-004-report-no-bleed-to-frames.webm
├── AC-005-detail-extracted-and-tagged.tape
├── AC-005-detail-extracted-and-tagged.gif
├── AC-005-detail-extracted-and-tagged.webm
├── AC-006-detail-no-bleed-to-frames.tape
├── AC-006-detail-no-bleed-to-frames.gif
├── AC-006-detail-no-bleed-to-frames.webm
├── AC-007-interpolation-resolved-before-tagging.tape
├── AC-007-interpolation-resolved-before-tagging.gif
├── AC-007-interpolation-resolved-before-tagging.webm
├── AC-008-all-three-registers-one-slide.tape
├── AC-008-all-three-registers-one-slide.gif
├── AC-008-all-three-registers-one-slide.webm
└── evidence-report.md
```

Total: 24 recording files (8 AC x 3 formats each) + this report = 25 files.
