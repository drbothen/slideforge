# Demo Evidence Report — STORY-030

**Story:** STORY-030 — Math: MathML (HTML) + Path-Based (PDF) Output
**Crate:** `slideforge-math`
**Branch:** `feature/S-030`
**HEAD at recording:** `080e91c3`
**Recorded:** 2026-05-28
**Total tests in crate:** 170 passed, 0 failed, 0 skipped

---

## Coverage Summary

| AC | Title | Recording | Tests | Status |
|----|-------|-----------|-------|--------|
| AC-001 | MathML output for HTML export | AC-001-mathml.gif / .webm | 5 tests — simple expression, superscript, fraction, inline/block display | PASS |
| AC-002 | MathML accessible to screen readers (aria-label) | AC-002-aria-label.gif / .webm | 4 tests — aria-label present, pythagorean label, sum label, empty fallback | PASS |
| AC-003 | PDF path output — vector SVG, no text characters | AC-003-pdf-paths.gif / .webm | 5 tests — no `<text>`, no `<image>`, `<path>` present, well-formed XML, newtype API | PASS |
| AC-004 | Single MathAst source for all formats | AC-004-ast-reusability.gif / .webm | 2 tests — MathML reuse, PDF paths reuse | PASS |
| AC-005 | KaTeX stylesheet deferred to STORY-047 | N/A — no demo required per spec | — | DEFERRED (by spec) |

---

## AC-001: MathML Output for HTML Export

**Traces to:** BC-1.10.003 postcondition 3

**Recording:** `AC-001-mathml.gif` / `AC-001-mathml.webm`

**Tape source:** `AC-001-mathml.tape`

**Tests demonstrated:**

| Test | Assertion |
|------|-----------|
| `test_bc_1_10_003_mathml_simple_expression` | Output contains `<math xmlns="http://www.w3.org/1998/Math/MathML"`, `<mi>x</mi>`, `<mi>y</mi>`, `<mo>+</mo>` |
| `test_bc_1_10_003_mathml_superscript` | `$x^2$` produces `<msup>` containing `<mi>x</mi>` and `<mn>2</mn>` |
| `test_bc_1_10_003_mathml_fraction` | `\frac{1}{2}` produces `<mfrac>` |
| `test_bc_1_10_003_mathml_inline_uses_display_inline` | Inline math has `display="inline"`, never `display="block"` |
| `test_bc_1_10_003_mathml_display_uses_display_block` | Display math (`$$...$$`) has `display="block"`, never `display="inline"` |

**Sample MathML output** (for `$x + y$`, inline):

```xml
<math xmlns="http://www.w3.org/1998/Math/MathML" display="inline" aria-label="x plus y">
  <mrow>
    <mi>x</mi>
    <mo>+</mo>
    <mi>y</mi>
  </mrow>
</math>
```

**Error path:** `test_bc_1_10_003_mathml_empty_greek_returns_error` and `test_bc_1_10_003_mathml_empty_operator_returns_error` verify that malformed AST nodes return `Err(MathError)` rather than silently producing invalid MathML.

---

## AC-002: MathML Accessible to Screen Readers

**Traces to:** BC-1.10.003 postcondition 3 ("accessible to screen readers")

**Recording:** `AC-002-aria-label.gif` / `AC-002-aria-label.webm`

**Tape source:** `AC-002-aria-label.tape`

**Tests demonstrated:**

| Test | Assertion |
|------|-----------|
| `test_bc_1_10_003_mathml_has_aria_label` | Every `<math>` element contains `aria-label=` with a non-empty value |
| `test_bc_1_10_003_aria_label_for_pythagorean` | `E = mc^2` produces label containing "equals" and "squared" |
| `test_bc_1_10_003_aria_label_for_simple_sum` | `x + y` produces label containing "plus" |
| `test_bc_1_10_003_aria_label_empty_ast_returns_fallback` | Empty AST produces `"empty math expression"` (not `aria-label=""`) |

**Sample ARIA output** (for `$E = mc^2$`):

```
aria-label="E equals m times c squared"
```

**Error path:** The fallback `"empty math expression"` is itself the error-path demo — an empty AST receives a meaningful ARIA label rather than an empty string that would confuse screen readers (Finding I4 addressed).

---

## AC-003: PDF Path Output — Vector SVG, No Text Characters

**Traces to:** BC-1.10.003 postcondition 5

**Recording:** `AC-003-pdf-paths.gif` / `AC-003-pdf-paths.webm`

**Tape source:** `AC-003-pdf-paths.tape`

**Tests demonstrated:**

| Test | Assertion |
|------|-----------|
| `test_bc_1_10_003_pdf_paths_svg_has_no_text_elements` | SVG output does not contain `<text` (PDF/UA-1 compliance) |
| `test_bc_1_10_003_pdf_paths_svg_has_no_image_elements` | SVG output does not contain `<image` (no embedded rasters) |
| `test_bc_1_10_003_pdf_paths_svg_contains_path_elements` | SVG output contains `<path d="` elements with glyph outlines |
| `test_bc_1_10_003_pdf_paths_wellformed_xml` | SVG is well-formed XML (verified with `quick-xml` parser) |
| `test_bc_1_10_003_svg_paths_struct_fields_accessible` | `SvgPaths(String)` newtype: `.0` accessor, `Clone`, `PartialEq`, `Hash` all work |

**SvgPaths newtype:** `SvgPaths(String)` — inner SVG string accessed via `.0`. Implements `Clone + PartialEq + Hash` (comemo/HashMap compatible). The SVG contains only `<path d="...">` elements with absolute coordinates; no `<text>` or `<image>` survive usvg normalization.

**Error path:** `test_bc_1_10_003_pdf_unknown_command_errors` — an unknown LaTeX command returns `Err(MathError::UnknownCommand { name })` rather than silent fallback. EC-005 (usvg normalization failure) would return `Err(MathError::TextRemainsInPathOutput)`.

---

## AC-004: Single MathAst Source for All Formats

**Traces to:** BC-1.10.003 invariant 1 ("parse once, render multiple times")

**Recording:** `AC-004-ast-reusability.gif` / `AC-004-ast-reusability.webm`

**Tape source:** `AC-004-ast-reusability.tape`

**Tests demonstrated:**

| Test | Assertion |
|------|-----------|
| `test_bc_1_10_003_mathml_ast_reusable_across_calls` | The same `&MathAst` can be passed to `render_mathml` twice — second call succeeds and produces identical output |
| `test_bc_1_10_003_pdf_paths_ast_reusable_across_calls` | The same `&MathAst` can be passed to `render_pdf_paths` twice without error |

**Architecture compliance:** All three renderers (`render_omml`, `render_mathml`, `render_pdf_paths`) take `&MathAst` — a shared reference. No re-parsing occurs. The AST is an immutable value type. Tests also confirm that calling `render_mathml` followed by `render_pdf_paths` on the same `&MathAst` both succeed (cross-renderer reuse).

---

## AC-005: Web Preview Uses KaTeX HTML+CSS

**Status:** DEFERRED to STORY-047 per spec (AC-005 explicitly scopes KaTeX stylesheet bundling to STORY-047).

`render_mathml` returns MathML for both static HTML export and web preview. No separate demo is required for this AC in STORY-030; the MathML output demonstrated in AC-001 is the web preview representation.

---

## Full Suite Verification

```
cargo nextest run -p slideforge-math --no-fail-fast
Summary [0.237s] 170 tests run: 170 passed, 0 skipped
```

All 170 tests pass at HEAD `080e91c3`, confirming no regressions against STORY-029 tests after adding the STORY-030 implementations.
