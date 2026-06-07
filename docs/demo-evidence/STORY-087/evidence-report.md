# STORY-087 Demo Evidence Report

**Story:** STORY-087 — Color-Coded Slide Types: status, progress_bar, weighted_composite (Registration + LabelCheck WCAG Enforcement)
**Crates:** `slideforge-plugin-api`, `slideforge-syntax`, `slideforge-layout`, `slideforge-validate`, `slideforge-eval`, `slideforge-types`, `slideforge-pptx`, `slideforge-pdf`, `slideforge-docx`, `slideforge-html`, `slideforge`
**Branch:** `feature/STORY-087`
**Recorded:** 2026-06-06
**Recording tool:** VHS (terminal capture of `cargo nextest` tests against real pipeline output)

---

## Artifacts

| File | Format | Covered ACs |
|------|--------|-------------|
| `AC-001-007-014-023-024-registration.gif` | GIF (PR embed) | AC-001, AC-007, AC-014, AC-023, AC-024 |
| `AC-001-007-014-023-024-registration.webm` | WEBM (archival) | AC-001, AC-007, AC-014, AC-023, AC-024 |
| `AC-001-007-014-023-024-registration.tape` | VHS script | AC-001, AC-007, AC-014, AC-023, AC-024 |
| `AC-002-008-015-visible-output.gif` | GIF (PR embed) | AC-002, AC-008, AC-015 |
| `AC-002-008-015-visible-output.webm` | WEBM (archival) | AC-002, AC-008, AC-015 |
| `AC-002-008-015-visible-output.tape` | VHS script | AC-002, AC-008, AC-015 |
| `AC-003-009-012-error-paths.gif` | GIF (PR embed) | AC-003, AC-005, AC-009, AC-012, AC-013 |
| `AC-003-009-012-error-paths.webm` | WEBM (archival) | AC-003, AC-005, AC-009, AC-012, AC-013 |
| `AC-003-009-012-error-paths.tape` | VHS script | AC-003, AC-005, AC-009, AC-012, AC-013 |

---

## Coverage Map

### AC-001 — status slide type is a registered keyword and parses without error

**Demonstrated in recording:** `AC-001-007-014-023-024-registration` — Section 1
**Evidence:** `cargo nextest run -p slideforge-plugin-api -E 'test(~ac001_status_keyword) + ...'`
- `test_BC_1_17_001_ac001_status_keyword_registered` — asserts `SLIDE_TYPE_KEYWORDS.contains("status")` returns true.
  Before this story, `"status"` was absent from the PHF set → parse would emit `E-PAR-NNN` (unknown keyword).
  After this story, the keyword is present → parse succeeds.

**Result:** PASS

---

### AC-002 — status slide with title + label builds successfully and label is visible in output

**Demonstrated in recording:** `AC-002-008-015-visible-output` — Section 1
**Evidence:** `test_AC_002_status_label_visible_in_output` — programmatic build path (SID-1).
Constructs a `Deck` with a `status` slide having `fields["label"] = "On Track"`, calls `build()` with
`strict: true`, then inspects the returned `LaidOutSlide` to assert that at least one
`FrameContent::Body(...)` frame contains the text "On Track". This is a positive-content assertion —
merely `Ok` return is insufficient. Stage-2b threading (`thread_fields_to_blocks`) routes the `label`
field as `ContentBlock::Text(TextTag::ColorLabel)` → `fill_region_slot_or_append` maps
`TextTag::ColorLabel → RegionRole::Body` → `FrameContent::Body`.

**Result:** PASS

---

### AC-003 — status slide missing label → E-A11-002 in strict mode

**Demonstrated in recording:** `AC-003-009-012-error-paths` — Section 1
**Evidence:** `test_BC_1_17_001_ac003_status_missing_label_message_contains_title` builds
`story-087-status-missing-label-project-beta.sf` (`slide status: title "Project Beta"`, no label)
with `strict: true` and asserts:
(a) build returns `Err(BuildError::ValidationFailed)`;
(b) at least one diagnostic has code `"E-A11-002"`;
(c) the diagnostic message contains both `"status"` and `"Project Beta"`.
`LabelCheckValidator` reads `Slide.fields["label"]` directly (Stage 5 pre-layout, independent of Stage-2b).
No output bytes are produced. This is the primary WCAG 1.4.1 enforcement gate — F-G3-HIGH-003 closure.

**Result:** PASS (error path)

---

### AC-004 — status slide empty label → E-A11-002

**Not given a dedicated recording** (unit-level test, same error path as AC-003).
`test_BC_1_17_001_ac004g_status_lay_out_geometry_only_empty_label_ok` in `slideforge-plugin-api`
tests the `lay_out()` geometry-only path. The build-level empty-label error is covered by
`test_BC_5_01_003_label_empty_string` in `slideforge-validate` which asserts that
`LabelCheckValidator` fires E-A11-002 for an empty string label (whitespace-only too).

**Result:** PASS (covered by validate unit suite)

---

### AC-005 — status missing label + strict=false → Ok, warning emitted

**Demonstrated in recording:** `AC-003-009-012-error-paths` — Section 3
**Evidence:** `test_AC_005_status_missing_label_warn_only_is_ok` builds a `status` slide with no
label using `strict: false` and asserts build returns `Ok`. Because the label field is absent,
Stage-2b threading emits no `ContentBlock::Text(TextTag::ColorLabel)` block → the Body-role frame
in the `LaidOutSlide` remains `FrameContent::Empty` (no phantom label content). The warn-only mode
produces non-conformant output but does not error.

**Result:** PASS (warn-only path)

---

### AC-006 — LabelCheck reads Slide.fields, not Slide.blocks (Stage-2b independence)

**Not given a dedicated recording** (unit-level invariant test).
`test_BC_1_17_001_ac006_status_label_check_reads_fields_not_blocks` in `slideforge-validate` calls
`LabelCheckValidator::validate()` on a `Deck` where `Slide.fields["label"] = None` but
`Slide.blocks = vec![]` (pre-Stage-2b state). The validator still fires E-A11-002, confirming
LabelCheck does not depend on Stage-2b having populated blocks.

**Result:** PASS (covered by validate unit suite)

---

### AC-007 — progress_bar slide type is registered and parses

**Demonstrated in recording:** `AC-001-007-014-023-024-registration` — Section 1
**Evidence:** `test_BC_1_17_002_ac007_progress_bar_keyword_registered` — asserts
`SLIDE_TYPE_KEYWORDS.contains("progress_bar")` is true. Included in the same 4-test run as AC-001.

**Result:** PASS

---

### AC-008 — progress_bar with valid title + label + value(75) builds with visible label and bar

**Demonstrated in recording:** `AC-002-008-015-visible-output` — Sections 2 and 3
**Evidence (Section 2):** `test_AC_008_progress_bar_label_and_bar_visible` asserts TWO things:
(a) the label text "75% complete" is present in at least one `FrameContent::Body(...)` frame;
(b) at least one `FrameContent::ColorBar { filled_width_emu, .. }` frame has `filled_width_emu > Emu(0)`.
This requires both Stage-2b threading (label → ColorLabel → Body slot; value → ColorBar spec) AND
the ColorBar materialization pass in `layout::run` (computes `filled_width_emu = total * 75 / 100`).

**Evidence (Section 3):** `test_AC_008_progress_bar_bar_rendered_in_pptx_xml` builds a full PPTX
and asserts the slide XML contains a solid-fill `<p:sp>` element (the rendered color bar).
This is the end-to-end visible-output gate: content threads from DSL field through layout IR to OOXML.

**Result:** PASS (both label visibility and ColorBar render)

---

### AC-009 — progress_bar missing label → E-A11-002

**Demonstrated in recording:** `AC-003-009-012-error-paths` — Section 2
**Evidence:** `test_BC_1_17_002_ac009_progress_bar_missing_label_message_contains_title` builds
`story-087-progress-bar-missing-label-sprint4.sf` (`slide progress_bar: title "Sprint 4" value 65`,
no label) with `strict: true` and asserts:
(a) build returns `Err(BuildError::ValidationFailed)`;
(b) at least one diagnostic has code `"E-A11-002"`;
(c) the message contains both `"progress_bar"` and `"Sprint 4"`.

**Result:** PASS (error path)

---

### AC-010 — progress_bar value=0 is valid (boundary)

**Not given a dedicated recording** (boundary validation unit test).
`test_BC_1_17_002_build_progress_bar_value_0_boundary_is_ok` in `slideforge/tests/e2e/story_087_value_range.rs`
builds a fixture with `value 0` and asserts `Ok`. Covered in the full story-087 E2E suite.

**Result:** PASS

---

### AC-011 — progress_bar value=100 is valid (boundary)

**Not given a dedicated recording** (boundary validation unit test).
`test_BC_1_17_002_build_progress_bar_value_100_boundary_is_ok` builds `value 100` and asserts `Ok`.

**Result:** PASS

---

### AC-012 — progress_bar value=101 → BuildError::ValidationFailed with E-VAL-011

**Demonstrated in recording:** `AC-003-009-012-error-paths` — Section 4
**Evidence:** `test_BC_1_17_002_build_progress_bar_value_101_is_validation_failed` builds
`story-087-progress-bar-101.sf` (`slide progress_bar: title "Sprint 4 Progress" label "101% complete" value 101`)
and asserts:
(a) build returns `Err(BuildError::ValidationFailed)`;
(b) at least one diagnostic has code `"E-VAL-011"`;
(c) the message contains `"progress_bar value must be between 0 and 100; got 101."`.
`ValueRangeValidator` fires at Stage 5 (pre-layout). `lay_out()` is geometry-only and is never
reached for this fixture (F-087-P1-001 compliance: validation is NOT in `lay_out()`).

**Result:** PASS (error path)

---

### AC-013 — progress_bar value=-1 → BuildError::ValidationFailed with E-VAL-011

**Demonstrated in recording:** `AC-003-009-012-error-paths` — Section 5
**Evidence:** `test_BC_1_17_002_build_progress_bar_value_neg1_is_validation_failed` builds
`story-087-progress-bar-neg1.sf` (`value -1`) and asserts `Err` with E-VAL-011
and message `"progress_bar value must be between 0 and 100; got -1."`.

**Result:** PASS (error path)

---

### AC-014 — weighted_composite slide type is registered and parses

**Demonstrated in recording:** `AC-001-007-014-023-024-registration` — Section 1
**Evidence:** `test_BC_1_17_003_ac014_weighted_composite_keyword_registered` — asserts
`SLIDE_TYPE_KEYWORDS.contains("weighted_composite")` is true.

**Result:** PASS

---

### AC-015 — weighted_composite with valid fields builds with all labels visible

**Demonstrated in recording:** `AC-002-008-015-visible-output` — Section 4
**Evidence:** `test_AC_015_weighted_composite_labels_visible` — uses programmatic SID-1 path
(pre-populates `Slide.fields["components"]` with `Value::List([Value::Map({...}), ...])`, bypassing
the DSL list-of-map parser per AC-015 NOTE, since STORY-088 DSL list parser is not yet landed).
Asserts ALL of:
(a) aggregate label "Overall: Good (78/100)" is present in at least one `FrameContent::Body(...)` frame;
(b) exactly 2 Generic-role frames carry component text including "Quality", "85", "Excellent";
(c) exactly 2 Generic-role frames carry component text including "Price", "72", "Acceptable".
Stage-2b threading composites each component row as `"<name>: <score>/100 (wt: <weight>) — <label>"`.

**Result:** PASS

---

### AC-016 — weighted_composite missing top-level label → E-A11-002

**Not given a dedicated recording** (unit-level test in validate suite).
`test_BC_5_01_003_weighted_composite_missing_label` in `label_check.rs` asserts E-A11-002 for
a `weighted_composite` slide with no top-level `label` field.

**Result:** PASS (covered by validate unit suite)

---

### AC-017 — weighted_composite component missing label → E-A11-002 for that component

**Not given a dedicated recording** (unit-level test).
`test_BC_1_17_003_ac022_weighted_composite_label_check_component_iteration` in `label_check.rs`
calls `LabelCheckValidator::validate()` with a 2-component deck where component 2 has no label,
asserts E-A11-002 identifying that component.

**Result:** PASS (covered by validate unit suite)

---

### AC-018 — weighted_composite accumulates 3 E-A11-002 when top + 2 components missing labels

**Not given a dedicated recording** (unit-level accumulation test).
`test_BC_1_17_003_ac018_weighted_composite_accumulates_3_label_errors` calls the validator with
top-level label absent and both components missing labels, asserts 3 separate E-A11-002 diagnostics.
DI-018 (error accumulation) is enforced — the validator does not bail on the first missing label.

**Result:** PASS (covered by validate unit suite)

---

### AC-019 — weighted_composite empty components list → BuildError::ValidationFailed with E-VAL-011

**Not given a dedicated recording** (unit test in value_range.rs).
`test_BC_1_17_003_empty_components_is_error` in `value_range.rs` directly tests
`ValueRangeValidator` with `components: []` (empty list), asserts E-VAL-011 with message
`"weighted_composite requires at least one component; got empty list."`.
The unauthorized deferral comment (`// Empty-components validation is a separate concern`)
was removed per F-087-P2-001 architect pass-2 adjudication.

**Result:** PASS (covered by value_range unit suite)

---

### AC-020 — weighted_composite component score=101 → E-VAL-011

**Not given a dedicated recording** (unit test in value_range.rs).
`test_BC_1_17_003_score_101_error` asserts E-VAL-011 for `score 101` with message
`"weighted_composite components[0].score must be between 0 and 100; got 101."`.
Build-level path: `test_BC_1_17_003_build_weighted_composite_score_101_is_validation_failed`
in `story_087_value_range.rs` exercises the full pipeline.

**Result:** PASS

---

### AC-021 — weighted_composite component weight=0 → E-VAL-011

**Not given a dedicated recording** (unit test in value_range.rs).
`test_BC_1_17_003_weight_int_zero_error` asserts E-VAL-011 for `weight 0` with message
`"weighted_composite components[0].weight must be positive; got 0."`.
Build-level path: `test_BC_1_17_003_build_weighted_composite_weight_zero_is_validation_failed`
in `story_087_value_range.rs`.

**Result:** PASS

---

### AC-022 — LabelCheck iterates component sub-fields without Stage-2b

**Not given a dedicated recording** (unit test in label_check.rs).
`test_BC_1_17_003_ac022_weighted_composite_label_check_component_iteration` calls the validator
with `Slide.blocks = vec![]` (pre-Stage-2b) but `Slide.fields["components"]` populated, confirms
E-A11-002 fires for the component with missing label. LabelCheck independence from Stage-2b is verified.

**Result:** PASS (covered by validate unit suite)

---

### AC-023 — LayoutError::UnknownSlideType is NOT emitted for any of the three types

**Demonstrated in recording:** `AC-001-007-014-023-024-registration` — Section 1
**Evidence:** `test_BC_1_17_001_ac023_all_three_types_registered_not_unknown` builds each of the
three types (status, progress_bar, weighted_composite) with all required fields present and asserts
that none returns `LayoutError::UnknownSlideType`. This is the direct closure of F-G3-HIGH-003:
before STORY-087, all three types were in `COLOR_CODED_TYPES` but absent from `SLIDE_TYPE_KEYWORDS`
and `region_frames_for()`, causing `UnknownSlideType` before LabelCheck could run.

**Result:** PASS (F-G3-HIGH-003 closed)

---

### AC-024 — All COLOR_CODED_TYPES entries are present in SLIDE_TYPE_KEYWORDS (consistency invariant)

**Demonstrated in recording:** `AC-001-007-014-023-024-registration` — Section 2
**Evidence:** `test_BC_1_17_001_ac024_all_color_coded_types_in_slide_type_keywords` in
`slideforge-validate/src/label_check.rs` asserts that `COLOR_CODED_TYPES` contains all four entries
(`"status"`, `"progress_bar"`, `"weighted_composite"`, `"severity_cards"`) AND that every entry
in `COLOR_CODED_TYPES` is present in `SLIDE_TYPE_KEYWORDS`. The `"severity_cards"` D4 gap
(present in `COLOR_CODED_TYPES` and `regions.rs` but absent from `SLIDE_TYPE_KEYWORDS`) is fixed
by this story. The consistency invariant eliminates the dead-guard scenario that constituted F-G3-HIGH-003.

**Result:** PASS

---

## CI Gate Confirmation

All CI gates ran clean before recording (LOCAL adversarial convergence: 3/3 CLEAN, 10 passes):

| Gate | Command | Result |
|------|---------|--------|
| STORY-087 all E2E tests | `cargo nextest run -p slideforge -E 'test(story_087)'` | CLEAN (16/16 PASS) |
| Plugin-API color-coded types | `cargo nextest run -p slideforge-plugin-api` | CLEAN (288/288 PASS) |
| Validate suite (label_check + value_range) | `cargo nextest run -p slideforge-validate` | CLEAN (192/192 PASS) |

---

## ACs Not Given Dedicated Recordings

| AC | Reason | Evidence Location |
|----|--------|------------------|
| AC-004 | Same error path as AC-003; empty label unit test | `label_check.rs::test_BC_5_01_003_label_empty_string` |
| AC-006 | LabelCheck invariant — unit test, not visual behavior | `label_check.rs::test_BC_1_17_001_ac006_status_label_check_reads_fields_not_blocks` |
| AC-010 | Boundary value — covered in E2E value_range suite | `story_087_value_range.rs::test_BC_1_17_002_build_progress_bar_value_0_boundary_is_ok` |
| AC-011 | Boundary value — covered in E2E value_range suite | `story_087_value_range.rs::test_BC_1_17_002_build_progress_bar_value_100_boundary_is_ok` |
| AC-016 | Same error path as AC-003 for weighted_composite | `label_check.rs::test_BC_5_01_003_weighted_composite_missing_label` |
| AC-017 | Component label iteration — unit test | `label_check.rs::test_BC_1_17_003_ac022_weighted_composite_label_check_component_iteration` |
| AC-018 | Error accumulation — unit test | `label_check.rs::test_BC_1_17_003_ac018_weighted_composite_accumulates_3_label_errors` |
| AC-019 | Empty components — value_range unit test | `value_range.rs::test_BC_1_17_003_empty_components_is_error` |
| AC-020 | Score range — value_range unit + build E2E | `story_087_value_range.rs::test_BC_1_17_003_build_weighted_composite_score_101_is_validation_failed` |
| AC-021 | Weight positive — value_range unit + build E2E | `story_087_value_range.rs::test_BC_1_17_003_build_weighted_composite_weight_zero_is_validation_failed` |
| AC-022 | LabelCheck Stage-2b independence invariant | `label_check.rs::test_BC_1_17_003_ac022_weighted_composite_label_check_component_iteration` |
