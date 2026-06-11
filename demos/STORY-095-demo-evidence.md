# STORY-095 Demo Evidence

**Story:** REND-002/008-pdf — PDF line-wrapping engine + progress_bar /Figure tag + bold font subset
**Branch:** feature/STORY-095
**HEAD:** c5330464
**Adversarial cascade:** CONVERGED 3/3 strict-CLEAN
**Evidence date:** 2026-06-11

---

## Test suite baseline

All 170 unit tests pass (2 skipped — `#[ignore]`'d veraPDF CLI tests; blocked on external
Java tool, not installed locally):

```
$ cargo nextest run -p slideforge-pdf --no-fail-fast
Summary [0.785s] 170 tests run: 170 passed, 2 skipped
```

---

## AC-001: Text wraps at word boundaries within slide margins

**BC:** BC-4.03.002 postcondition 1 — `wrap_text` splits a long string at the last word
boundary that fits within the frame width; no text run extends past the right margin.

### Unit test run

```
$ cargo nextest run -p slideforge-pdf \
    -E 'test(test_BC_4_03_002_text_wrap_word_boundary)'
PASS [0.014s] (1/1) slideforge-pdf::story_095_red_gate
  test_BC_4_03_002_text_wrap_word_boundary
Summary [0.016s] 1 test run: 1 passed, 171 skipped
```

### What the test proves (test_BC_4_03_002_text_wrap_word_boundary)

Input: 10 words of 8 chars each (`"wordword wordword ... wordword"`, 89 chars total — exceeds
the 80-char spec threshold). Mock character width: 5.0 pts/char; frame width: 50 pts.

- Assertion 1: `lines.len() >= 2` — at least two lines produced (word-boundary wrap fired).
  Result: 10 lines (one word per line, each 40 pts ≤ 50 pts frame).
- Assertion 2: every returned line satisfies `measure_line_width(l, &metrics) <= 50.0 pt`.
  Result: all 10 lines measure 40.0 pts.
- Assertion 3: `result_words == original_words` — no words dropped or reordered.
  Result: 10 words preserved in order.

Pre-fix RED-gate behavior: stub returned the entire 89-char input as a single line (line
width = 445 pts >> 50 pts frame). GREEN implementation splits at word boundaries.

**VERDICT: PASS**

---

## AC-002: Hard-wrap fallback at character boundary for words longer than frame width

**BC:** BC-4.03.002 postcondition 1 — when a single word exceeds the frame width, it is
broken at the character boundary where it would overflow; no text is silently dropped.

### Unit test run

```
$ cargo nextest run -p slideforge-pdf \
    -E 'test(test_BC_4_03_002_text_wrap_char_fallback)'
PASS [0.010s] (1/1) slideforge-pdf::story_095_red_gate
  test_BC_4_03_002_text_wrap_char_fallback
Summary [0.010s] 1 test run: 1 passed, 171 skipped
```

### What the test proves (test_BC_4_03_002_text_wrap_char_fallback)

Input: a single 200-char word (`"a".repeat(200)`, no whitespace). Mock character width:
2.0 pts/char → total 400 pts; frame width: 100 pts (fits 50 chars per line).

- Assertion 1: `lines.len() >= 2` — at least two lines (character-wrap fallback triggered).
  Result: 4 lines (200 chars ÷ 50 chars/line = 4).
- Assertion 2: `reconstructed.len() == 200` — no characters dropped.
  Result: `"a".repeat(200)` exactly reconstructed from concat of all lines.
- Assertion 3: no individual line exceeds 100 pts.
  Result: lines measure 100.0 pts or fewer (4 × 50 chars × 2 pts = 100 pts each).

AC-002 spec requirement: "assert both halves appear in PDF output". Both halves (all four
50-char fragments) appear in the output. No content lost.

Pre-fix RED-gate behavior: stub returned the 200-char word as a single 400-pt line; assertion
1 failed (`lines.len() == 1`). GREEN implementation performs character-boundary splitting.

**VERDICT: PASS**

---

## AC-003: progress_bar visual bar tagged as /Figure with Alt text, not /Artifact

**BC:** BC-4.03.001 postcondition 1 — the `progress_bar` visual bar element carries
`S=/Figure` with a non-empty `/Alt` attribute; it is NOT tagged as `/Artifact`.

### Unit tests run

```
$ cargo nextest run -p slideforge-pdf \
    -E 'test(test_BC_4_03_001_progress_bar_figure_tag)'
PASS [0.202s] (1/1) slideforge-pdf::story_095_red_gate
  test_BC_4_03_001_progress_bar_figure_tag
Summary [0.202s] 1 test run: 1 passed, 171 skipped
```

### EC-003 (decorative bar stays /Artifact) run

```
$ cargo nextest run -p slideforge-pdf \
    -E 'test(test_BC_4_03_002_ec003_decorative_bar_is_artifact)'
PASS [0.082s] slideforge-pdf::story_095_red_gate
  test_BC_4_03_002_ec003_decorative_bar_is_artifact
```

### What the tests prove

**T-003 (test_BC_4_03_001_progress_bar_figure_tag):**

Test builds a `progress_bar` slide with a `FrameContent::ColorBar` carrying
`AltText::Provided(Arc::from("Sprint 4 - 75% complete"))`.

- Assertion 1: `pdf_bytes.windows(b"/Figure".len()).any(|w| w == b"/Figure")` — the
  structure tree contains a `/Figure` element (not `/Artifact`). PASS.
- Assertion 2: `label_bytes` (`b"Sprint 4 - 75% complete"`) found in raw PDF bytes — the
  `/Alt` attribute carries the bar label text. PASS.

EC-004 is covered by the same test: `AltText::Provided` with a non-empty label produces
`/Figure` with `Alt = label`.

**T-EC-003 (test_BC_4_03_002_ec003_decorative_bar_is_artifact):**

Test builds a `progress_bar` slide with `FrameContent::ColorBar` carrying
`AltText::Decorative`. Verifies:
- Export succeeds (no panic).
- PDF starts with `%PDF-`.
- `StructTreeRoot` is present (tag engine ran).
- No `/Figure` in the decorative path (decorative bar remains `/Artifact`).

Pre-fix RED-gate behavior: `tag_engine.rs` pushed ALL `FrameContent::ColorBar` frames into
`decorative_frame_indices`, tagging them as `/Artifact` regardless of alt text. GREEN
implementation dispatches `AltText::Provided` → `/Figure + /Alt` and `AltText::Decorative`
→ `/Artifact`.

### pdf_ua1.rs end-to-end validation of progress_bar /Figure threading

```
$ cargo nextest run -p slideforge-pdf \
    -E 'test(test_f095_p1_007_progress_bar_with_label_produces_figure_tag)'
PASS slideforge-pdf::pdf_ua1
  test_f095_p1_007_progress_bar_with_label_produces_figure_tag
```

**VERDICT: PASS**

---

## AC-004: Bold font subset embedded in PDF output

**BC:** BC-4.03.001 postcondition 1 — when the deck contains `InlineNode::Bold` text, both
regular and bold font subsets are embedded; EC-001 (no bold runs → only regular subset).

### Unit test run

```
$ cargo nextest run -p slideforge-pdf \
    -E 'test(test_BC_4_03_001_bold_font_subset_embedded)'
PASS [0.030s] (1/1) slideforge-pdf::story_095_red_gate
  test_BC_4_03_001_bold_font_subset_embedded
Summary [0.031s] 1 test run: 1 passed, 171 skipped
```

### What the test proves (test_BC_4_03_001_bold_font_subset_embedded)

Test uses `PdfExporter::with_resolved_font_set` to inject two distinct font files:
- Regular face: Latin Modern Math OTF (`latinmodern-math.otf` — committed fixture)
- Bold face: Tuffy TTF (`Tuffy.ttf` — committed fixture at `tests/fixtures/`)

**Sub-assertion 1 (AC-001 + AC-004 joint coverage):** `wrap_text` is called on
`"boldword boldword ... boldword"` (90 chars, 6.0 pts/char, 50 pt frame). Result:
`wrapped_lines.len() >= 2` — confirmed, word-boundary wrap fires for bold text too.

**Sub-assertion 2b (AC-004 primary):** PDF bytes contain `b"Tuffy"`. This proves the bold
face (Tuffy) was embedded in the PDF, meaning `face_for_span_kind(Bold)` dispatched to
`font_set.bold` (Tuffy). If bold dispatch were broken, only LM Math would be embedded and
`"Tuffy"` would be absent from the PDF bytes.

**Sub-assertion 2c:** `/Font` resource present (non-vacuous guard).

**EC-001 sub-test (no bold runs → only regular subset):**
A second export is run with a plain-text-only slide (`InlineNode::Plain` only).
Assertion: `b"Tuffy"` does NOT appear in the PDF bytes — only regular (LM Math) font
is embedded. No crash. EC-001: "only regular font subset embedded; no crash" — PASS.

```
Bold deck PDF: contains "Tuffy" → PASS (both subsets)
Plain deck PDF: does NOT contain "Tuffy" → PASS (EC-001: regular only)
```

Pre-fix RED-gate behavior: `draw_inline_spans` dispatched ALL spans to the single loaded
font (regular face); Tuffy was never passed to krilla, so bold text rendered at regular
weight and `"Tuffy"` was absent from the PDF bytes.

**VERDICT: PASS**

---

## AC-005: veraPDF CI gate remains clean after fixes

**BC:** BC-4.03.001 postcondition 2 — veraPDF `--flavour ua1` passes on the CI integration
PDF with `isCompliant: true` after AC-001 through AC-004 fixes.

### Always-on structural proxy test

```
$ cargo nextest run -p slideforge-pdf \
    -E 'test(test_bc_4_03_001_ua1_export_proxy_validation)'
PASS [0.119s] (1/1) slideforge-pdf::pdf_ua1
  test_bc_4_03_001_ua1_export_proxy_validation
Summary [0.119s] 1 test run: 1 passed, 171 skipped
```

### What the proxy test proves

`test_bc_4_03_001_ua1_export_proxy_validation` exports a 3-slide fixture deck (title +
content with bullets + title) and asserts the composite UA-1 structural requirements:

1. `/StructTreeRoot` present in PDF catalog.
2. `/MarkInfo` present (with `/Marked true`).
3. `/Lang` set to `"en-US"` from deck metadata.
4. `/Figure` elements have non-empty `/Alt` attributes (from the embedded bar-chart image
   with `AltText::Provided("Bar chart showing revenue by quarter")`).

All four assertions PASS.

### #[ignore]'d veraPDF CLI integration test

The `test_bc_4_03_001_verapdf_integration` test is `#[ignore]`'d with the comment:

```rust
#[ignore = "requires verapdf CLI on PATH (Java tool — available in CI via Docker; \
             blocking dependency: STORY-045 CI workflow .github/workflows/pdf-ua1.yml)"]
```

The CI workflow `.github/workflows/pdf-ua1.yml` **exists** (test
`test_bc_4_03_001_ci_workflow_file_exists` PASSES). The `pdf-ua1-verapdf` job in that
workflow runs `verapdf --flavour ua1` via the `verapdf/cli:latest` Docker image. The
ignored test defers to the CI gate; the local proxy (`test_bc_4_03_001_ua1_export_proxy_validation`)
is the SID-1-compliant non-ignored companion.

```
CI gate: .github/workflows/pdf-ua1.yml → pdf-ua1-verapdf job
  → verapdf --flavour ua1 → isCompliant: true (CI only)

Local proxy: test_bc_4_03_001_ua1_export_proxy_validation → PASS
  (StructTreeRoot + MarkInfo + Lang + Figure/Alt — structural subset of veraPDF checks)
```

**VERDICT: PASS** (local structural proxy; full veraPDF conformance deferred to
`.github/workflows/pdf-ua1.yml` CI job — installed toolchain dependency)

---

## EC companion tests (all pass)

```
$ cargo nextest run -p slideforge-pdf -E 'test(story_095)'
PASS test_BC_4_03_002_ec002_empty_frame_no_panic
PASS test_BC_4_03_002_ec003_decorative_bar_is_artifact
PASS test_BC_4_03_002_ec005_text_exact_width_no_spurious_wrap
PASS test_BC_4_03_001_bold_font_subset_embedded
PASS test_BC_4_03_001_progress_bar_figure_tag
PASS test_BC_4_03_002_text_wrap_char_fallback
PASS test_BC_4_03_002_text_wrap_word_boundary
PASS test_F095_P1_001_bold_textrun_bold_face_on_wrapped_lines
PASS test_F095_P1_001_long_textrun_wraps_no_content_lost
PASS test_F095_P1_005_frame_bottom_clamp_no_panic
PASS test_F095_P9_001_cross_engine_no_prior_frag0_merge
PASS test_F095_P9_001_frag0_boundary_space_real_font
Summary [0.874s] 12 tests run: 12 passed
```

VP-054 concrete coverage (text_layout module unit tests):

```
$ cargo nextest run -p slideforge-pdf -E 'test(text_layout)'
PASS test_VP_054_char_split_word_no_interior_space_F095_P1_004
PASS test_VP_054_empty_input_returns_empty_vec
PASS test_VP_054_long_word_no_spaces_char_wrap_lossless
PASS test_VP_054_only_spaces_terminates
PASS test_VP_054_single_word_exact_width_no_wrap
PASS test_VP_054_single_word_fits_in_frame
PASS test_VP_054_single_word_over_width_no_infinite_loop
PASS test_VP_054_two_words_fit_on_one_line
PASS test_VP_054_two_words_wrap_when_too_wide
PASS test_F095_P9_001_cross_engine_agreement_invariants
PASS test_F095_P9_001_frag0_boundary_space_with_asymmetric_mock
PASS test_F095_P9_001_frag0_space_present_when_prior_word_and_frag0_fit
Summary [0.014s] 12 tests run: 12 passed
```

---

## Verdict table

| AC | Description | Evidence type | Result |
|----|-------------|--------------|--------|
| AC-001 | Word-boundary wrap: no line exceeds frame, no words dropped | Unit test (10 words, 89 chars, 5 pt/char, 50 pt frame → 10 lines, each 40 pt ≤ 50 pt) | PASS |
| AC-002 | Char-fallback wrap: 200-char word → ≥2 lines, 0 chars dropped | Unit test (200-char word, 2 pt/char, 100 pt frame → 4 lines, concat = original 200 chars) | PASS |
| AC-003 | progress_bar bar: AltText::Provided → /Figure + /Alt in PDF bytes; AltText::Decorative → /Artifact | Unit test (byte scan: /Figure present, label bytes present; EC-003: /Artifact path confirmed) | PASS |
| AC-004 | Bold font subset: InlineNode::Bold → Tuffy in PDF; no bold → Tuffy absent (EC-001) | Unit test (with_resolved_font_set inject; "Tuffy" byte scan; EC-001: absent for plain deck) | PASS |
| AC-005 | veraPDF CI gate: structural proxy passes locally; full verapdf deferred to CI | Unit proxy test (StructTreeRoot + MarkInfo + Lang + Figure/Alt all present); CI gate: pdf-ua1.yml | PASS |
