# STORY-077: SectionBlock IR Extension + Inline-Markup Parser — Demo Evidence

## Date: 2026-06-02

## Commit: f7e4d444 (feature/S-077)

## Modality Note

The `slideforge build` CLI command is implemented in STORY-055 (not yet merged).
STORY-077 is a foundational parser/eval/IR story — the demoable behaviors are:

1. The inline markup parser producing structural `TemplateChunk::Bold/Italic/Code/...`
   (not flat literal strings containing `**...**`)
2. The eval stage routing section fields to `RegisteredContent{Register::Detail|Report}`
3. Strict-fatal error diagnostics: unclosed/empty/over-nested markup, and bad-arg builtins

All recordings drive `cargo nextest run` directly against the production crate test suites.
This is the correct modality for a foundational library story with no end-user CLI surface yet.
VHS records real command execution at `f7e4d444` — no fabricated output.

---

## AC Coverage

| AC | Description | Recording | Test Count | Status |
|----|-------------|-----------|------------|--------|
| AC-001 | `SectionBlock.body["detail"]` holds `FieldValue::Inlines` (structural nodes, not `Value::Str`) | `AC-001-002-inline-markup-parser-happy-path.*` | 41 syntax tests | PASS |
| AC-002 | Parser recognizes `**bold**`, `_italic_`, `` `code` ``, `[link](url)`, `^super^`, `~sub~`, `~~strike~~`, `==highlight==` — produces structural `TemplateChunk` variants, NOT literal asterisks | `AC-001-002-inline-markup-parser-happy-path.*` | 41 syntax tests | PASS |
| AC-003 | `detail:` sub-block produces `RegisteredContent{Register::Detail}` on the section node | `AC-003-004-section-register-routing-happy-path.*` | 4 eval tests | PASS |
| AC-004 | `report:` sub-block produces `RegisteredContent{Register::Report}` on the section node | `AC-003-004-section-register-routing-happy-path.*` | 4 eval tests | PASS |
| AC-005 | Standalone `section methodology:` with only `detail:` (no slides) places register content on the SECTION node, NOT on any `LaidOutSlide` | `AC-005-006-standalone-section-no-slides.*` | 3 eval tests | PASS |
| AC-006 | Section with both `detail:` and `report:` produces two `RegisteredContent` entries | `AC-005-006-standalone-section-no-slides.*` | 3 eval tests | PASS |
| Error: E-PAR-019 | Unclosed inline markup delimiter (`**unclosed`) is strict-build-fatal; returns `Err` with code `E-PAR-019` and `SyntaxError::UnclosedInlineMarkup` variant | `AC-ERR-E-PAR-019-unclosed-delimiter.*` | 2 syntax tests | PASS |
| Error: E-PAR-020 | Empty inline markup span (`****`) is strict-build-fatal; returns `Err` with code `E-PAR-020` | `AC-ERR-E-PAR-020-empty-span.*` | 2 syntax tests | PASS |
| Error: E-PAR-021 | Nesting depth exceeded produces `E-PAR-021` fatal error, NOT a stack overflow | `AC-ERR-E-PAR-021-nesting-depth.*` | 1 syntax test | PASS |
| Error: E-EVL-012 | `figref()` with missing/empty arg produces `E-EVL-012` at eval time | `AC-ERR-E-EVL-012-013-014-builtin-bad-args.*` | 15 eval tests (combined) | PASS |
| Error: E-EVL-013 | `ref("")` with empty id produces `E-EVL-013` at eval time | `AC-ERR-E-EVL-012-013-014-builtin-bad-args.*` | 15 eval tests (combined) | PASS |
| Error: E-EVL-014 | `footnote()` with missing/empty arg produces `E-EVL-014` at eval time | `AC-ERR-E-EVL-012-013-014-builtin-bad-args.*` | 15 eval tests (combined) | PASS |

---

## Recordings

All files are in `.factory/demos/STORY-077/`.

### AC-001 / AC-002 — Inline Markup Parser: Structural Node Production (Happy Path)

**Files:**
- `AC-001-002-inline-markup-parser-happy-path.tape`
- `AC-001-002-inline-markup-parser-happy-path.gif`
- `AC-001-002-inline-markup-parser-happy-path.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-syntax -E 'test(template_inline_markup_tests)' --no-fail-fast 2>&1 | tail -20
```

**Tests exercised (41 total, all PASS):**
- `test_BC_3_02_002_bold_chunk_produced` — `"**hello**"` → `TemplateChunk::Bold([Literal("hello")])`
- `test_BC_3_02_002_italic_chunk_produced` — `"_hello_"` → `TemplateChunk::Italic([Literal("hello")])`
- `test_BC_3_02_002_code_chunk_produced` — `` "`fn foo()`" `` → `TemplateChunk::Code("fn foo()")`
- `test_BC_3_02_002_link_chunk_produced` — `"[click here](https://example.com)"` → `TemplateChunk::Link{...}`
- `test_BC_3_02_002_superscript_chunk_produced` — `"^sup^"` → `TemplateChunk::Superscript`
- `test_BC_3_02_002_subscript_chunk_produced` — `"~sub~"` → `TemplateChunk::Subscript`
- `test_BC_3_02_002_strikethrough_before_subscript` — mixed markup
- `test_BC_3_02_002_highlight_chunk_produced` — `"==mark=="` → `TemplateChunk::Highlight`
- `test_BC_3_02_002_bold_with_interpolation` — `"**{{ var }}**"` → Bold wrapping Interpolation
- `test_BC_3_02_002_nested_bold_italic` — `"**_hi_**"` → Bold wrapping Italic
- `test_BC_3_02_002_code_span_no_inner_markup` — code span verbatim (no inner markup)
- `test_BC_3_02_002_math_mode_no_inline_markup` — `$...$` is opaque to inline markup scanner
- + 29 additional tests (error paths, ref/figref/footnote call parsing, multibyte safety, nesting)

**Evidence:** 41/41 PASS. The parser now produces structural `TemplateChunk` variants for all supported inline markup forms. `FieldValue::Inlines` is populated with real `InlineNode` variants, not flat strings containing raw delimiter characters.

---

### AC-003 / AC-004 — Section Register Routing: detail and report (Happy Path)

**Files:**
- `AC-003-004-section-register-routing-happy-path.tape`
- `AC-003-004-section-register-routing-happy-path.gif`
- `AC-003-004-section-register-routing-happy-path.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-eval -E 'test(ac003)|test(ac004)' --no-fail-fast 2>&1 | tail -20
```

**Tests exercised (4 total, all PASS):**
- `test_BC_3_02_002_ac003_extract_section_register_content_produces_detail_entry`
  - Input: `SectionBlock` with `body["detail"] = FieldValue::Inlines([Plain("...")])`
  - Expected: `extract_section_register_content` returns `[RegisteredContent{register: Register::Detail, ...}]`
- `test_BC_3_02_002_ac003_interpolation_resolved_before_detail_tagging`
  - Verifies template interpolation is evaluated before routing to register
- `test_BC_3_02_002_ac004_extract_section_register_content_produces_report_entry`
  - Input: `SectionBlock` with `body["report"] = FieldValue::Inlines([Plain("...")])`
  - Expected: `RegisteredContent{register: Register::Report, ...}`
- `test_BC_1_14_003_ac004_section_report_tagged_for_docx_pdf_only`
  - Verifies `Register::Report` is tagged with the correct output-format sentinel

**Evidence:** 4/4 PASS. `extract_section_register_content` correctly routes `detail:` → `Register::Detail` and `report:` → `Register::Report`.

---

### AC-005 / AC-006 — Standalone Section: Register on Section Node, Not Slide

**Files:**
- `AC-005-006-standalone-section-no-slides.tape`
- `AC-005-006-standalone-section-no-slides.gif`
- `AC-005-006-standalone-section-no-slides.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-eval -E 'test(ac006)|test(ac005)' --no-fail-fast 2>&1 | tail -20
```

**Tests exercised (3 total, all PASS):**
- `test_BC_3_02_002_ac006_standalone_section_detail_no_slides`
  - A deck with only `section methodology:` + `detail:` (no slide blocks) — the register content appears on `SectionBlock.register_content`, NOT on any `LaidOutSlide`
- `test_BC_3_02_002_ac006_section_with_both_registers_produces_two_entries`
  - Both `detail:` and `report:` produce two separate `RegisteredContent` entries
- `test_BC_1_14_003_ac005_detail_tagged_register_detail_excluded_from_pptx_sentinel`
  - `Register::Detail` is excluded from PPTX output (document-only register) — verified via format sentinel

**Evidence:** 3/3 PASS. Descoped STORY-035 EC-003 is fully covered: standalone section register content is on the section node, isolated from slide layout.

---

### Error: E-PAR-019 — Unclosed Inline Markup Delimiter

**Files:**
- `AC-ERR-E-PAR-019-unclosed-delimiter.tape`
- `AC-ERR-E-PAR-019-unclosed-delimiter.gif`
- `AC-ERR-E-PAR-019-unclosed-delimiter.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-syntax -E 'test(E_PAR_019)' --no-fail-fast 2>&1 | tail -20
```

**Input that triggers E-PAR-019:**
```
slide content:
  detail "**unclosed"
```

**Expected behavior verified by tests:**
- `parse()` returns `Err(errors)` in strict mode (NOT `Ok`) — build-fatal
- Errors vec is non-empty (error accumulation preserved)
- First error carries diagnostic code `E-PAR-019`
- Error variant is `SyntaxError::UnclosedInlineMarkup { .. }`
- Rendered message contains NO routing sentinel leak (`SLIDEFORGE_INLINE_ROUTE`)

**Tests exercised (2 total, all PASS):**
- `test_E_PAR_019_unclosed_inline_markup_code_assertion`
- `test_E_PAR_019_warning_is_unclosed_inline_markup_variant`

**Evidence:** 2/2 PASS. Unclosed delimiter is strict-build-fatal with correct diagnostic code.

---

### Error: E-PAR-020 — Empty Inline Markup Span

**Files:**
- `AC-ERR-E-PAR-020-empty-span.tape`
- `AC-ERR-E-PAR-020-empty-span.gif`
- `AC-ERR-E-PAR-020-empty-span.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-syntax -E 'test(E_PAR_020)' --no-fail-fast 2>&1 | tail -20
```

**Input that triggers E-PAR-020:**
```
slide content:
  detail "****"
```

**Expected behavior verified by tests:**
- `parse()` returns `Err(errors)` in strict mode — build-fatal
- First error carries diagnostic code `E-PAR-020`
- Error variant is `SyntaxError::EmptyInlineMarkupSpan { .. }`

**Tests exercised (2 total, all PASS):**
- `test_E_PAR_020_empty_inline_markup_span_code_assertion`
- `test_E_PAR_020_warning_is_empty_inline_markup_span_variant`

**Evidence:** 2/2 PASS.

---

### Error: E-PAR-021 — Nesting Depth Exceeded

**Files:**
- `AC-ERR-E-PAR-021-nesting-depth.tape`
- `AC-ERR-E-PAR-021-nesting-depth.gif`
- `AC-ERR-E-PAR-021-nesting-depth.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-syntax -E 'test(E_PAR_021)' --no-fail-fast 2>&1 | tail -20
```

**Input that triggers E-PAR-021:**
Deep nesting of inline spans beyond the configured depth limit (e.g., `**_**_..._**_**` repeated many times).

**Expected behavior verified by tests:**
- `parse()` returns `Err(errors)` with code `E-PAR-021`
- No stack overflow (the recursion limit is bounded)

**Tests exercised (1 total, PASS):**
- `test_F077_P7_002_deep_nesting_produces_E_PAR_021_no_stack_overflow`

**Evidence:** 1/1 PASS. Bounded recursion enforced; no panic on adversarial input.

---

### Error: E-EVL-012 / E-EVL-013 / E-EVL-014 — Bad-Arg Builtins

**Files:**
- `AC-ERR-E-EVL-012-013-014-builtin-bad-args.tape`
- `AC-ERR-E-EVL-012-013-014-builtin-bad-args.gif`
- `AC-ERR-E-EVL-012-013-014-builtin-bad-args.webm`

**Command recorded:**
```
cargo nextest run -p slideforge-eval -E 'test(e_evl_012)|test(e_evl_013)|test(e_evl_014)|test(figref)|test(footnote_zero)|test(ref_zero)|test(ref_empty)|test(footnote_empty)|test(footnote_ident)|test(ref_ident)' --no-fail-fast 2>&1 | tail -25
```

**Inputs that trigger each code:**

| Error | Input | Trigger |
|-------|-------|---------|
| E-EVL-012 | `figref()` with no arg, or `figref(var)` where var resolves to empty | `figref()` called with missing/non-evaluable arg |
| E-EVL-013 | `ref("")` or `ref(var)` where var resolves to `""` | Cross-reference with empty id |
| E-EVL-014 | `footnote()` with no arg, or `footnote(var)` where var resolves to `""` | Footnote with empty/missing content |

**Tests exercised (15 total, all PASS):**
- `test_f077_p3_001_figref_no_arg_emits_e_evl_012`
- `test_f077_p11_001_figref_ident_resolves_empty_emits_e_evl_012`
- `test_f077_p12_001_pipe_figref_ident_resolves_empty_emits_e_evl_012`
- `test_f077_p3_001_ref_empty_id_call_emits_e_evl_013`
- `test_f077_p3_001_ref_empty_id_pipe_emits_e_evl_013`
- `test_f077_ref_ident_resolves_empty_emits_e_evl_013`
- `test_f077_p12_001_pipe_ref_ident_resolves_empty_emits_e_evl_013`
- `test_f077_p12_001_pipe_ref_empty_literal_emits_e_evl_013`
- `test_f077_p9_001_ref_zero_arg_emits_e_evl_013`
- `test_f077_p9_001_footnote_zero_arg_emits_e_evl_014`
- `test_f077_p9_001_footnote_empty_string_emits_e_evl_014`
- `test_f077_footnote_ident_resolves_empty_emits_e_evl_014`
- `test_f077_p12_001_pipe_footnote_ident_resolves_empty_emits_e_evl_014`
- `test_f077_p12_001_pipe_footnote_empty_literal_emits_e_evl_014`
- `test_f077_p5_001_expr_call_figref_to_xref` (happy-path figref round-trip)

**Evidence:** 15/15 PASS. All three eval-time bad-arg error codes are strict-build-fatal with correct `EvalError` variant and `miette` diagnostic code.

---

## Test Suite Totals (at f7e4d444)

| Crate | Tests Run | Passed | Failed |
|-------|-----------|--------|--------|
| `slideforge-syntax` (inline markup module) | 41 | 41 | 0 |
| `slideforge-syntax` (section parser module) | 20 | 20 | 0 |
| `slideforge-eval` (section register routing module) | 68 | 68 | 0 |
| **Total** | **129** | **129** | **0** |

---

## Artifacts Summary

| File | Type | ACs Covered |
|------|------|-------------|
| `AC-001-002-inline-markup-parser-happy-path.tape` | VHS tape | AC-001, AC-002 |
| `AC-001-002-inline-markup-parser-happy-path.gif` | GIF recording | AC-001, AC-002 |
| `AC-001-002-inline-markup-parser-happy-path.webm` | WebM recording | AC-001, AC-002 |
| `AC-003-004-section-register-routing-happy-path.tape` | VHS tape | AC-003, AC-004 |
| `AC-003-004-section-register-routing-happy-path.gif` | GIF recording | AC-003, AC-004 |
| `AC-003-004-section-register-routing-happy-path.webm` | WebM recording | AC-003, AC-004 |
| `AC-005-006-standalone-section-no-slides.tape` | VHS tape | AC-005, AC-006 |
| `AC-005-006-standalone-section-no-slides.gif` | GIF recording | AC-005, AC-006 |
| `AC-005-006-standalone-section-no-slides.webm` | WebM recording | AC-005, AC-006 |
| `AC-ERR-E-PAR-019-unclosed-delimiter.tape` | VHS tape | E-PAR-019 |
| `AC-ERR-E-PAR-019-unclosed-delimiter.gif` | GIF recording | E-PAR-019 |
| `AC-ERR-E-PAR-019-unclosed-delimiter.webm` | WebM recording | E-PAR-019 |
| `AC-ERR-E-PAR-020-empty-span.tape` | VHS tape | E-PAR-020 |
| `AC-ERR-E-PAR-020-empty-span.gif` | GIF recording | E-PAR-020 |
| `AC-ERR-E-PAR-020-empty-span.webm` | WebM recording | E-PAR-020 |
| `AC-ERR-E-PAR-021-nesting-depth.tape` | VHS tape | E-PAR-021 |
| `AC-ERR-E-PAR-021-nesting-depth.gif` | GIF recording | E-PAR-021 |
| `AC-ERR-E-PAR-021-nesting-depth.webm` | WebM recording | E-PAR-021 |
| `AC-ERR-E-EVL-012-013-014-builtin-bad-args.tape` | VHS tape | E-EVL-012, E-EVL-013, E-EVL-014 |
| `AC-ERR-E-EVL-012-013-014-builtin-bad-args.gif` | GIF recording | E-EVL-012, E-EVL-013, E-EVL-014 |
| `AC-ERR-E-EVL-012-013-014-builtin-bad-args.webm` | WebM recording | E-EVL-012, E-EVL-013, E-EVL-014 |

---

## ACs Not Demoed

None. All acceptance criteria from STORY-077 have direct recording coverage.

**Note on CLI demo absence:** The `slideforge build <file.sf>` command (which would display a
miette-rendered colored source pointer for E-PAR-019/020/021) is implemented in STORY-055
(not yet merged to develop). The error-path recordings here demonstrate the identical behavior
at the library boundary — the `parse()` function returning `Err` with the correct diagnostic
code and variant. The miette rendering path is tested via `rendered.contains(...)` assertions
in the same test suite. A CLI-level recording will be produced in STORY-055 demo evidence.
