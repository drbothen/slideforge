# STORY-098 Demo Evidence

**Story:** REND-005/010a — Strict-mode W-VAL-103 content-drop exit + chart no-data enforcement
**Branch:** feature/STORY-098
**HEAD:** 567f00a0
**Adversarial cascade:** CONVERGED 3/3 strict-CLEAN (passes 17-18-19)
**Evidence date:** 2026-06-11

---

## Test suite baseline

All 232 slideforge-validate unit tests pass (0 skipped):

```
$ cargo nextest run -p slideforge-validate --no-fail-fast
Summary [0.334s] 232 tests run: 232 passed, 0 skipped
```

All 419 slideforge-eval unit tests pass (0 skipped):

```
$ cargo nextest run -p slideforge-eval --no-fail-fast
Summary [0.591s] 419 tests run: 419 passed, 0 skipped
```

All 6 STORY-098 integration exit-code tests pass:

```
$ cargo nextest run -p slideforge -E 'test(story_098)' --no-fail-fast
Summary [0.059s] 6 tests run: 6 passed, 141 skipped
```

---

## AC-001: shape/body on unsupporting type — strict mode exits non-zero (W-VAL-103 content-drop sub-case)

**BC:** BC-3.03.002 v1.3 postcondition 3 + Invariant 4 — Route A: W-VAL-103 `{shape, body}`
sub-case = broken/exit-2 in strict mode.

### Integration test run

```
$ cargo nextest run -p slideforge \
    -E 'test(test_f098_p1_004_ac001_body_on_unsupporting_type_strict_exits_2)'
PASS [0.022s] (1/1) slideforge::e2e_tests
  e2e_story_098_exit_codes::test_f098_p1_004_ac001_body_on_unsupporting_type_strict_exits_2
Summary [0.022s] 1 test run: 1 passed
```

### What the test proves

Input: a `chart` slide containing a `body:` field that is NOT in `chart`'s `known_fields()`.
The `data:` field is provided (non-empty) to isolate the W-VAL-103 signal from E-LAY-003.

- Assertion 1: `matches!(result, Err(BuildError::ValidationFailed { .. }))` — strict build
  returns `Err` (BC-3.03.002 DI-017: all-or-nothing). PASS.
- Assertion 2: the `ValidationFailed` diagnostics contain a `W-VAL-103` entry whose
  `message` includes `'body'`. PASS.
- Assertion 3: that `W-VAL-103` diagnostic has `severity == DiagnosticSeverity::Error`
  (content-drop key `"body"` in `{"shape", "body"}` — BC-3.03.002 v1.3 Invariant 4). PASS.

### Warn-only counterpart (same deck, non-blocking)

```
$ cargo nextest run -p slideforge \
    -E 'test(test_f098_p1_004_ac001_body_on_unsupporting_type_warn_only_exits_0)'
PASS [0.059s] (1/1) slideforge::e2e_tests
  e2e_story_098_exit_codes::test_f098_p1_004_ac001_body_on_unsupporting_type_warn_only_exits_0
Summary [0.059s] 1 test run: 1 passed
```

Same `chart + body` DSL source in `--warn-only` mode returns `Ok` — W-VAL-103 is
non-blocking in warn-only mode regardless of field key (BC-3.03.002 v1.3 Invariant 4
warn-only branch).

### Unit test coverage (FieldSchemaValidator + registry.rs)

AC-003's consistency test (run below) also exercises the AC-001 path:
`test_BC_3_03_002_body_content_schema_consistency` Part (b) confirms `body` on `chart`
emits W-VAL-103 at Error severity at the unit level.

**VERDICT: PASS**

---

## AC-002: body field on content slide renders prose content — exit 0, no W-VAL-103

**BC:** BC-3.03.002 v1.3 EC-007 (REVERSED) + BC-4.01.001 v1.2 PC-11 — body placeholder
rendering contract.

### Integration test run

```
$ cargo nextest run -p slideforge \
    -E 'test(test_body_on_content_renders_and_exits_0)'
PASS [0.023s] (1/1) slideforge::e2e_tests
  e2e_story_098_exit_codes::test_body_on_content_renders_and_exits_0
Summary [0.023s] 1 test run: 1 passed
```

### What the test proves

Input: a `content:` slide with `title: "My Content Slide"` and `body: "Prose text"`.
Build mode: strict.

- Assertion 1: `result.is_ok()` — strict mode returns `Ok` (no W-VAL-103 emitted, no
  error diagnostics). `body` is a declared optional field in `known_fields("content")`
  (content.rs STORY-098 F-098-P1-002 + PO adjudication F-098-ADJ-BODY-CONTENT). PASS.
- Assertion 2 (load-bearing, TD-VSDD-059): the compiled `LaidOutDeck` contains a
  `FrameContent::Body` frame where at least one `ContentBlock::Text` has an `InlineNode::Plain`
  containing the string `"Prose text"`. This is the distinguishing assertion: mere exit-0
  is insufficient; the prose must land in the output IR (BC-4.01.001 v1.2 PC-11:
  `TextTag::Body → FrameContent::Body`). PASS.

Strict `Ok` semantically proves no W-VAL-103 was accumulated (any `Error`-severity
diagnostic in strict mode produces `Err(ValidationFailed)`).

**VERDICT: PASS**

---

## AC-003: body on content consistent across validator and eval — resolution (a) confirmed

**BC:** BC-3.03.002 v1.3 Invariant 4 — `known_fields()` is the authority; schema
consistency enforced across both validator and eval codepaths.

### Validator unit test

```
$ cargo nextest run -p slideforge-validate \
    -E 'test(test_BC_3_03_002_body_content_schema_consistency)'
PASS [0.015s] (1/1) slideforge-validate
  field_schema::tests::test_BC_3_03_002_body_content_schema_consistency
Summary [0.016s] 1 test run: 1 passed
```

### What the consistency test proves

Two-part test covering both sides of the `known_fields()` authority:

**Part (a) — `body` on `content` (acceptance):** builds a `Deck` with a `content` slide
carrying `body: "Body text is valid here."` and runs `FieldSchemaValidator`. Asserts no
`W-VAL-103` with `message.contains("'body'")` is emitted. PASS. `known_fields("content")`
includes `"body"` (line 110, known_fields.rs) — the validator sees it as a declared
optional field and does not fire W-VAL-103.

**Part (b) — `body` on `chart` (rejection at Error severity):** builds a `Deck` with a
`chart` slide carrying `body: "..."`. Asserts `W-VAL-103` is emitted with
`severity == Error`. PASS. `known_fields("chart")` does NOT include `"body"` — the
content-drop key check fires and promotes W-VAL-103 to Error (BC-3.03.002 Invariant 4).
Message format verified: contains `"Unknown field 'body'"` and slide type `"chart"`;
code remains `"W-VAL-103"` (Route A: no new code introduced).

### Eval threading regression guard

```
$ cargo nextest run -p slideforge-eval \
    -E 'test(test_f098_p3_004_progress_bar_body_field_not_threaded)'
PASS [0.010s] (1/1) slideforge-eval
  field_to_block::tests::test_f098_p3_004_progress_bar_body_field_not_threaded
Summary [0.011s] 1 test run: 1 passed
```

Guards that `thread_one_slide` does NOT thread `body` content into frames for slide types
whose `known_fields()` does NOT include `"body"`. The `slide_type_supports_body` gate uses
`known_fields()` as the authority — same source of truth as the validator.

### Quote regression guard (AC-001 boundary integrity)

```
$ cargo nextest run -p slideforge \
    -E 'test(test_body_on_quote_strict_exits_2)'
PASS [0.022s] (1/1) slideforge::e2e_tests
  e2e_story_098_exit_codes::test_body_on_quote_strict_exits_2
Summary [0.022s] 1 test run: 1 passed
```

`body` on `quote` slide type in strict mode: `Err(ValidationFailed)` + W-VAL-103 at
Error severity with message naming `"quote"`. Guards that the fix does NOT accidentally
add `"body"` to a non-declaring type's `known_fields()` via `common_optional_fields`
or similar drift. `quote` uses a second non-body-declaring type (separate from `chart`)
to prevent the content-drop logic from relying on a hardcoded type list instead of
`known_fields()`.

**VERDICT: PASS**

---

## AC-004: Chart with no data emits E-LAY-003 and non-zero exit in strict mode

**BC:** BC-1.11.002 v1.2 postcondition 2 (PO adjudication F-098-P1-007) — missing `data:`
and empty-evaluating `data:` treated identically.

### Integration test — chart with no data: field (strict)

```
$ cargo nextest run -p slideforge \
    -E 'test(test_f098_p1_004_ac004_chart_no_data_strict_exits_2)'
PASS [0.022s] (1/1) slideforge::e2e_tests
  e2e_story_098_exit_codes::test_f098_p1_004_ac004_chart_no_data_strict_exits_2
Summary [0.022s] 1 test run: 1 passed
```

### What the test proves

Input: `chart` slide with NO `data:` field at all (missing data source).

- Assertion 1: `matches!(result, Err(BuildError::ValidationFailed { .. }))` — strict build
  exits non-zero. PASS.
- Assertion 2: diagnostics contain `E-LAY-003` entry. PASS.
- Assertion 3: `E-LAY-003` message contains `"[E-LAY-003] Chart data is empty"` — the
  self-prefix is present per error-taxonomy v2.31 (F-098-P1-003). PASS.

### Unit tests — ChartEmptyDataValidator (both empty data and missing data)

```
$ cargo nextest run -p slideforge-validate -E 'test(chart_empty_data)' --no-fail-fast
PASS [0.013s] (1/7) slideforge-validate chart_empty_data::tests::test_chart_missing_data_strict_exits_2
PASS [0.013s] (2/7) slideforge-validate chart_empty_data::tests::test_BC_1_11_002_nonempty_chart_data_no_e_lay_003
PASS [0.013s] (3/7) slideforge-validate chart_empty_data::tests::test_BC_1_11_002_chart_empty_data_strict_exits_2
PASS [0.013s] (4/7) slideforge-validate chart_empty_data::tests::test_BC_1_11_002_chart_empty_data_warn_only_placeholder
PASS [0.013s] (5/7) slideforge-validate chart_empty_data::tests::test_BC_1_11_002_only_empty_data_chart_emits_e_lay_003
PASS [0.013s] (6/7) slideforge-validate chart_empty_data::tests::test_chart_missing_data_warn_only_placeholder
PASS [0.013s] (7/7) slideforge-validate chart_empty_data::tests::test_BC_1_11_002_invariant_2_validator_prevents_renderer_call
Summary [0.014s] 7 tests run: 7 passed, 225 skipped
```

### What the unit tests prove

**test_BC_1_11_002_chart_empty_data_strict_exits_2 (empty-evaluating binding):**
`chart` slide with `data: Value::List([])`. Asserts: E-LAY-003 emitted, exactly 1
diagnostic, `severity == Error`, message contains `"[E-LAY-003]"` + `"Revenue Chart"`
+ `"Rendering error-slide placeholder"`, code pinned to `"E-LAY-003"`, hint present. PASS.

**test_chart_missing_data_strict_exits_2 (missing data: field — F-098-P1-007):**
`chart` slide with no `data:` key at all. Identical assertions. BC-1.11.002 v1.2 EC-005
(F-098-P1-007 PO-adjudicated): absent `data:` field is equally broken as empty data —
both trigger E-LAY-003 with the same message format. PASS.

**test_BC_1_11_002_invariant_2_validator_prevents_renderer_call:**
Confirms the validator side of BC-1.11.002 invariant 2: E-LAY-003 at Error severity is
emitted. In strict mode the pipeline gate aborts before the export stage where
`ChartRenderer::render` would be invoked — `ChartRenderer` is never called. PASS.

**test_BC_1_11_002_only_empty_data_chart_emits_e_lay_003:**
Mixed deck: one empty-data chart + one non-empty chart. Only the empty-data slide emits
E-LAY-003 (DI-018: all slides checked). PASS.

**EC-003 guard (test_BC_1_11_002_nonempty_chart_data_no_e_lay_003):**
Non-empty chart data does NOT produce E-LAY-003. PASS.

**VERDICT: PASS**

---

## AC-005: Chart with no data shows error-slide placeholder in warn-only mode — exit 0

**BC:** BC-1.11.002 v1.2 postcondition 3 (PO adjudication F-098-P1-007) — missing `data:`
and empty-evaluating `data:` treated identically; E-LAY-003 non-blocking in warn-only.

### Integration test — chart with no data: field (warn-only)

```
$ cargo nextest run -p slideforge \
    -E 'test(test_f098_p1_004_ac005_chart_no_data_warn_only_exits_0)'
PASS [0.055s] (1/1) slideforge::e2e_tests
  e2e_story_098_exit_codes::test_f098_p1_004_ac005_chart_no_data_warn_only_exits_0
Summary [0.055s] 1 test run: 1 passed
```

### What the test proves

Input: same `chart` slide with NO `data:` field, build mode `--warn-only` (strict=false).

- Assertion 1: `result.is_ok()` — `--warn-only` build exits 0. E-LAY-003 is non-blocking
  in warn-only mode; the pipeline gate does not abort. PASS.

Architecture note: the validator (`ChartEmptyDataValidator`) always emits `E-LAY-003` at
`Error` severity regardless of mode. The pipeline gate is the demotion point: in
`--warn-only`, it does not treat `Error` diagnostics as fatal. In strict mode it does.
This is the DI-017 all-or-nothing invariant boundary.

### Unit tests — warn-only E-LAY-003 emission (same 7-test suite)

Both warn-only unit tests from AC-004's suite are directly relevant here:

**test_BC_1_11_002_chart_empty_data_warn_only_placeholder:**
Confirms `ChartEmptyDataValidator` emits E-LAY-003 even in conceptual warn-only context
(the validator fires regardless of mode; Error severity always). PASS.

**test_chart_missing_data_warn_only_placeholder:**
Confirms E-LAY-003 emitted for missing-data chart in warn-only context; message carries
`"[E-LAY-003]"` self-prefix. PASS.

The integration test (`test_f098_p1_004_ac005_chart_no_data_warn_only_exits_0`) proves
the full pipeline: validator fires + gate allows through + build returns `Ok` with output
produced — matching AC-005's "exit 0; output written" requirement.

**VERDICT: PASS**

---

## Verdict table

| AC | Description | Evidence type | Result |
|----|-------------|--------------|--------|
| AC-001 | body on chart (non-declaring type) strict → Err(ValidationFailed) + W-VAL-103 Error; warn-only → Ok | Integration test (exit-code assertion + W-VAL-103 severity pinned); warn-only counterpart exits 0 | PASS |
| AC-002 | body on content strict → Ok + FrameContent::Body carries "Prose text" (load-bearing IR assertion) | Integration test (exit-0 + distinguishing prose content in LaidOutDeck; TD-VSDD-059) | PASS |
| AC-003 | validator + eval both accept body on content (no W-VAL-103); body on chart rejects at Error severity; quote regression guard passes | Unit test (FieldSchemaValidator consistency, 2-part); eval threading guard; quote regression guard | PASS |
| AC-004 | Chart no-data strict → Err(ValidationFailed) + E-LAY-003 self-prefixed; both empty-binding and missing-data: field trigger same path | Integration test (strict exit 2, E-LAY-003 message format); 7 unit tests (empty data, missing data, invariant 2, accumulation, EC-003) | PASS |
| AC-005 | Chart no-data warn-only → Ok (exit 0); E-LAY-003 emitted by validator but non-blocking | Integration test (warn-only exit 0); unit warn-only tests for empty-data and missing-data variants | PASS |
