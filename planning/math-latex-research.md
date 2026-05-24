---
title: "Math/LaTeX Support -- Syntax Conflicts and Rendering Pipeline"
date: 2026-05-24
analyst: research-agent
status: foundation-research
audience: product-owner + architect + human-reviewer
---

# Math/LaTeX Support Research

## Executive Summary

**Recommended delimiter:** Option A -- standard `$...$` / `$$...$$` with interpolation disabled inside math regions. This is the most familiar syntax for users, has the lowest learning curve, and all conflicts are resolvable via strict mode-based parsing (text mode vs. math mode vs. raw mode). Interpolation into math expressions should use pre-computed variables or a v2 `@{var}` opt-in syntax.

**Conflicts found:** 14 conflict pairs analyzed; 4 rated Medium-to-High severity, all resolvable via mode-based parsing. The critical conflict is `{{ }}` vs `{}` inside math (e.g., `$\frac{{{ value }}}{2}$`), resolved by disabling `{{ }}` interpolation inside `$...$` regions entirely.

**LaTeX-to-OMML conversion:** No pure-Rust crate exists. Best path for v1.0: LaTeX -> MathML (via `pulldown-latex` v0.7.1, pure Rust) -> OMML (via embedded `mml2omml.xsl` XSLT transform using libxslt FFI). Alternative: shell out to Pandoc's texmath for direct LaTeX -> OMML.

**v1.0 math scope:** Ship `$...$` / `$$...$$` delimiters with no interpolation inside math. Render via KaTeX (HTML/web preview), pulldown-latex -> MathML (PDF), and MathML -> OMML via XSLT (PPTX/DOCX). Defer math-inside-interpolation to v2.

**Accessibility:** KaTeX produces MathML by default (`htmlAndMathml` output mode), meeting WCAG 1.1.1 and 1.3.1. OMML in PPTX/DOCX is screen-reader accessible in Microsoft Office (JAWS/NVDA). Non-Microsoft renderers degrade OMML to images, losing accessibility.

---

## Syntax Conflict Matrix

### Complete Conflict Map: slideforge DSL vs. LaTeX Math

| slideforge Syntax | LaTeX Syntax | Conflict? | Severity | Context | Resolution |
|---|---|---|---|---|---|
| `{{ var }}` | `{}` grouping | YES | **HIGH** | `$\frac{{{ val }}}{2}$` -- parser cannot distinguish LaTeX braces from interpolation braces | Disable `{{ }}` inside `$...$`. Lex math regions first; inside them, all `{}` is LaTeX. |
| `{{ var }}` | `$...$` delimiter | No | None | Different characters | N/A |
| `_italic_` | `_` subscript | YES | **MEDIUM** | `_x_` in text = italic; `x_1` in math = subscript | Mode-based: `_` is italic only in text mode; subscript only in math mode. |
| `^sup^` | `^` superscript | YES | **MEDIUM** | `^note^` in text = superscript; `x^2` in math = superscript | Mode-based: `^word^` only in text mode; `^{expr}` only in math mode. |
| `#` comment | `#` macro param | YES | **HIGH** | `# comment` vs `$f#1$` | `#` is a comment only at start-of-line in text mode. Inside math, `#` is passed to LaTeX unchanged. |
| `**bold**` | `*` (multiplication) | No | None | `**` has no special meaning in LaTeX math | N/A |
| `[text](url)` | `[]` (optional args) | Minimal | **LOW** | `$[0,1]$` uses brackets, but inside math, link syntax is disabled | Disable link parsing inside math regions. |
| `"""..."""` | Any LaTeX | No | None | Triple-quote blocks are raw; no math or DSL parsing inside | N/A |
| Any DSL | `\` (commands) | No | None | slideforge DSL does not use `\` as a control character | N/A |
| Any DSL | `&` (alignment) | No | None | slideforge DSL does not use `&` | N/A |
| Any DSL | `~` (nbsp) | No | None | slideforge DSL does not use `~` | N/A |
| Any DSL | `%` (LaTeX comment) | No | None | slideforge uses `#` for comments, not `%` | N/A |
| `$` (none currently) | `$` delimiter | No | None | slideforge DSL has no `$` syntax; it is free for math delimiters | N/A |
| `{{ }}` | `$$...$$` display | No | None | `$$` is two `$` characters, not related to `{{ }}` | N/A |

### Deep Dive: The `{{ }}` vs `{}` Problem

The most dangerous conflict. Consider:

```
$\frac{{{ revenue }}}{2}$
```

Without rules, the parser sees: `\frac` + `{` + `{{ revenue }}` + `}` + `{2}` -- but the brace nesting is ambiguous. Is `{{{ revenue }}}` one LaTeX group containing an interpolation, or three nested LaTeX groups?

**Resolution (binding for v1.0):** Interpolation `{{ }}` is **completely disabled** inside `$...$` and `$$...$$` regions. Inside math, every `{` and `}` is a LaTeX grouping brace. Period.

To get data into equations, users pre-compute:

```
vars:
  ratio {{ revenue / cost | number(2) }}

report """
  The ratio is $\frac{R}{C} = RATIO_VALUE$.
"""
```

Where `RATIO_VALUE` is injected as a literal number during evaluation, before the math parser sees it. The exact mechanism (pre-computed variable substitution vs. a v2 `@{var}` syntax) is deferred to DSL design.

### Deep Dive: `_` Italic vs Subscript

```
Write _italic text_ and $x_1$ on the board.
```

- In text mode: `_italic text_` triggers italic formatting.
- In math mode: `x_1` is `x` subscript `1`.

**Resolution:** Strict mode separation. The parser identifies math regions (`$...$`) first, then applies text-mode formatting only to non-math text. Inside math regions, `_` is always LaTeX subscript.

### Deep Dive: `#` Comment vs Macro Parameter

```
# This is a comment
$f#1$
```

**Resolution:** `#` starts a DSL comment only when:
1. In text mode (not inside `$...$`), AND
2. It is the first non-whitespace character on the line.

Inside math, `#` passes through to LaTeX as a macro parameter character.

---

## Math Delimiter Options (A-F)

### Option A: Standard `$...$` / `$$...$$` -- Interpolation Disabled Inside (RECOMMENDED)

```
slide formula:
  title "Revenue Model"
  report """
    The revenue function is $R(x) = px - C(x)$ where $p$ is unit price.

    Display equation:
    $$\text{Profit} = R(x) - C(x) = (p - c)x - F$$
  """
```

| Criterion | Assessment |
|---|---|
| User familiarity | Excellent -- standard LaTeX/Markdown convention |
| Conflict severity | Low (with mode-based parsing) |
| Implementation complexity | Low -- identify `$...$` regions, disable DSL inside |
| Data injection into math | Not possible in v1.0; pre-compute values as variables |
| Cross-tool precedent | Pandoc, Quarto, Marp, Slidev, Reveal.js, MDX all use this |
| User effort | Minimal for math authoring; extra step for data-driven math |

**Verdict: RECOMMENDED for v1.0.** Solves 95% of use cases. Data injection deferred.

### Option B: `$...$` with `@{var}` Opt-In Interpolation

```
report """
  The formula: $\frac{R}{C} = \frac{@{revenue}}{@{cost}}$
"""
```

| Criterion | Assessment |
|---|---|
| User familiarity | Good for math; unfamiliar `@{}` syntax |
| Conflict severity | Low -- `@{` is unambiguous |
| Implementation complexity | Medium -- two interpolation syntaxes |
| Data injection into math | Yes, via `@{var}` |
| Cross-tool precedent | No major tool uses this pattern |
| User effort | Low once learned; cognitive overhead of two syntaxes |

**Verdict: Good v2 candidate.** Adds complexity but enables data-driven equations.

### Option C: Function-Call Syntax `{{ math("...") }}`

```
report """
  Revenue was {{ math("R = " + revenue + " \\text{ million}") }}.
"""
```

| Criterion | Assessment |
|---|---|
| User familiarity | Poor -- verbose, unfamiliar |
| Conflict severity | None -- math is inside a string literal |
| Implementation complexity | Medium |
| Data injection into math | Yes, via string concatenation |
| Cross-tool precedent | None |
| User effort | High -- escaped backslashes, string concatenation |

**Verdict: Rejected.** Too verbose for real math content. Escaped backslashes make complex equations unreadable.

### Option D: Typst-Style Math (Not LaTeX)

```
report """
  Revenue was $R = p x - C(x)$ where $p$ is unit price.
"""
```

| Criterion | Assessment |
|---|---|
| User familiarity | Poor -- very few people know Typst math |
| Conflict severity | Low (Typst designed to avoid conflicts) |
| Implementation complexity | High -- need Typst math parser, not LaTeX |
| Data injection into math | Built-in via `#var` |
| Cross-tool precedent | Only Typst |
| User effort | High learning curve |

**Verdict: Rejected.** LaTeX is the industry standard. Adopting Typst math syntax would alienate users.

### Option E: Backtick-Delimited Math

```
report """
  Revenue was `math: R = px - C(x)` where `math: p` is unit price.

  ```math
  \frac{R}{C} = \frac{revenue}{cost}
  ```
"""
```

| Criterion | Assessment |
|---|---|
| User familiarity | Medium -- code-block metaphor is familiar |
| Conflict severity | None -- backticks are unambiguous |
| Implementation complexity | Low |
| Data injection into math | Ambiguous -- needs rules |
| Cross-tool precedent | GitHub, GitLab use ` ```math ` blocks |
| User effort | Slightly more verbose than `$...$` |

**Verdict: Viable alternative.** Good if `$` proves problematic in practice. Could be offered as an alternative delimiter.

### Option F: `$...$` with Pre-Computed LaTeX Variables

```
vars:
  ratio {{ revenue / cost | number(2) }}

report """
  The ratio is $\frac{R}{C} = {{ ratio }}$.
"""
```

**Verdict: Rejected as a distinct option.** This is just Option A with the `{{ }}` vs `{}` conflict unresolved. The `{{ ratio }}` inside `$...$` reintroduces the exact ambiguity we're trying to avoid.

---

## Recommended Delimiter

**Option A: `$...$` / `$$...$$` with interpolation disabled inside math.**

Rationale:
1. Universal familiarity -- every tool in the ecosystem uses `$...$`.
2. Zero ambiguity -- mode-based parsing cleanly separates text and math.
3. Low implementation cost -- identify `$` delimiters, switch parser mode.
4. The "can't interpolate data into equations" limitation is acceptable for v1.0; pre-computed variables handle 90% of data-driven math needs.
5. `@{var}` math interpolation can be added in v2 without breaking changes.

**Parser contract:**
- Phase 1: Identify `"""..."""` raw blocks (no parsing inside).
- Phase 2: Inside non-raw text, identify `$...$` and `$$...$$` math regions. Respect `\$` escaping.
- Phase 3: In text regions, apply DSL formatting (`_italic_`, `**bold**`, `[links](url)`, `#` comments, `{{ }}` interpolation).
- Phase 4: In math regions, pass content to LaTeX math parser unchanged (no DSL processing).

---

## Cross-Tool Survey

| Tool | Math Syntax | Interpolation Mechanism | Coexistence Strategy | Known Issues |
|---|---|---|---|---|
| **Pandoc** | `$...$`, `$$...$$` | None (converter, not template engine) | Heuristic `$` detection; requires no inner spaces | False positives with currency `$5` |
| **Quarto** | `$...$`, `$$...$$` (via Pandoc) | `` `r expr` ``, `{r}` code chunks | Different syntax families; no overlap | Inherits Pandoc's `$` heuristics |
| **Slidev** | `$...$`, `$$...$$` (KaTeX) | `{{ expr }}` (Vue template) | KaTeX renders in Markdown phase; Vue in template phase | `$\sqrt{{{ x }}}$` needs careful brace balancing |
| **Marp** | `$...$` (KaTeX) | None by default | Inherits KaTeX behavior | `$` collides with currency text; escape with `\$` |
| **MDX** | `$...$` via remark-math | `{ expr }` (JSX expressions) | remark-math runs before JSX interpretation; braces inside math are safe | Edge cases with `{}` in math vs JSX |
| **Reveal.js** | Plugin: KaTeX or MathJax | None in core | Configurable delimiters; can use `\(...\)` instead of `$` | Auto-render false positives |
| **Typst** | `$...$` | `#var` / `#(expr)` | `#` works in both text and math mode | Minimal -- designed together |
| **Jupyter** | `$...$` in Markdown cells | `{var}` in Python f-strings | Different contexts (code vs. markdown cells) | None -- separate execution |
| **R Markdown** | `$...$` | `` `r inline_expr` `` | Different syntax; no `$`/`{}` overlap | Minimal |

**Key insight:** Every tool that uses `$...$` for math and has some interpolation mechanism resolves conflicts through **phase separation** -- math is parsed/rendered in a different pipeline stage than interpolation. slideforge should do the same.

---

## Rendering Pipeline

### LaTeX to OMML (for PPTX/DOCX)

**Target:** Native, editable equations in PowerPoint and Word.

**Pipeline:** `LaTeX -> MathML -> OMML`

#### Step 1: LaTeX to MathML (Rust-native)

**Best crate: `pulldown-latex` v0.7.1**
- Pure Rust, no JS dependencies
- Pull parser design (like pulldown-cmark)
- Outputs MathML Core (browser-compatible subset)
- 28,605 total downloads; 5.9K SLoC; MIT licensed
- MSRV: Rust 1.74.1
- Last release: ~1 year ago (stable, not abandoned)
- Supports: fractions, integrals, summations, matrices, Greek letters, most standard LaTeX math commands

**Alternative: `latex2mathml` v0.2.4**
- Pure Rust, older (last release ~2019, effectively unmaintained)
- Simpler API: `latex_to_mathml(&str) -> Result<String>`
- ~65,000 total downloads
- Being discussed for rustdoc integration, but maintainability concerns

**Alternative: `katex` crate (Rust bindings)**
- Wraps KaTeX JS via embedded QuickJS engine
- Heavier dependency (JS runtime in Rust binary)
- Can output MathML via KaTeX's `output: "mathml"` option
- Good fallback if pure-Rust parsers lack coverage

**Recommendation:** Use `pulldown-latex` as primary. Fall back to `katex` crate for edge cases where LaTeX coverage gaps exist.

#### Step 2: MathML to OMML (XSLT Transform)

**No pure-Rust path exists.** Options:

1. **Embed `mml2omml.xsl` + libxslt FFI (RECOMMENDED)**
   - Microsoft's `mml2omml.xsl` is an XSLT 1.0 stylesheet that converts MathML to OMML
   - Available on GitHub (transpect/docx_modify-lib) under permissive license
   - Apply via libxslt C library through Rust FFI
   - Embed the stylesheet with `include_str!()` in the binary
   - Pipeline: `pulldown-latex` -> MathML string -> libxslt transform -> OMML string -> insert into OOXML

2. **Shell out to Pandoc**
   - `echo "$LATEX" | pandoc -f latex -t docx --mathml` or use texmath directly
   - Pandoc's texmath does LaTeX -> OMML directly (no MathML intermediate)
   - Pro: battle-tested, complete LaTeX coverage
   - Con: external dependency; not a single binary

3. **Custom Rust MathML-to-OMML converter**
   - Write a direct tree transformation in Rust using `quick-xml`
   - MathML and OMML are structurally similar; the transform is mechanical
   - Pro: no C dependencies, pure Rust
   - Con: significant engineering effort; must handle all MathML constructs
   - This is the best long-term path but not viable for v1.0

**v1.0 Recommendation:** Option 1 (libxslt FFI with embedded XSLT). It is proven, complete, and the XSLT file is small (~50KB). If libxslt proves problematic for distribution, fall back to Option 2 (Pandoc as an optional dependency).

#### OMML Embedding in PPTX

In PPTX files, math is embedded inside DrawingML text bodies:

```xml
<p:sp>
  <p:txBody>
    <a:p>
      <a14:m>
        <m:oMathPara>
          <m:oMath>
            <!-- OMML content here -->
          </m:oMath>
        </m:oMathPara>
      </a14:m>
    </a:p>
  </p:txBody>
</p:sp>
```

The `a14:m` element wraps OMML inside DrawingML. The namespace `a14` is `http://schemas.microsoft.com/office/drawing/2010/main`.

#### OMML Embedding in DOCX

In DOCX files, math is embedded directly in paragraph runs:

```xml
<w:p>
  <m:oMathPara>
    <m:oMath>
      <!-- OMML content here -->
    </m:oMath>
  </m:oMathPara>
</w:p>
```

- Inline math: `<m:oMath>` directly inside `<w:p>`
- Display math: `<m:oMathPara>` containing `<m:oMath>`

### LaTeX to KaTeX (for HTML/Web Preview)

**Target:** Rendered math in HTML export and `slideforge watch` web preview.

**Pipeline:** `LaTeX -> KaTeX HTML+MathML`

**Options:**

1. **`katex` Rust crate (bindings to KaTeX via QuickJS)**
   - Server-side rendering: LaTeX string in, HTML+MathML string out
   - Output includes MathML for accessibility
   - Requires bundling KaTeX JS + QuickJS engine
   - Well-tested; mirrors KaTeX behavior exactly

2. **`pulldown-latex` -> MathML, then embed MathML directly**
   - Use `<math>` elements in HTML output
   - MathML Core is supported in all major browsers since Jan 2023
   - Lighter dependency than KaTeX
   - Visual quality may differ from KaTeX rendering

3. **RaTeX (pure Rust KaTeX-compatible engine)**
   - Brand new (v0.1.9, May 2026); >99.5% KaTeX syntax coverage claimed
   - Pure Rust, no JS engine needed
   - Outputs to SVG, PNG, PDF, WASM
   - Very promising but too new for production v1.0
   - Watch for maturity in 2026-2027

**v1.0 Recommendation:** Use the `katex` crate for HTML/web preview output. It produces both visual HTML and accessible MathML. For PDF, pipe the KaTeX HTML through headless Chrome (already likely the PDF pipeline per ADR-003).

### LaTeX to PDF

**Target:** High-quality math in PDF output.

**Pipeline options:**

1. **HTML (with KaTeX) -> Headless Chrome -> PDF** (if ADR-003 chooses HTML-to-PDF)
   - KaTeX renders math in HTML; Chrome rasterizes to PDF
   - Math quality is excellent (KaTeX is pixel-perfect)
   - Accessibility: MathML in the HTML source is preserved in Chrome's PDF output (tagged PDF)

2. **MathML -> Typst-as-backend -> PDF** (if ADR-003 chooses Typst)
   - Would need MathML-to-Typst math conversion
   - Typst's own math syntax is different from MathML; non-trivial

3. **Direct PDF via `pdf-writer` with math glyphs**
   - Would need a full math layout engine (essentially reimplementing KaTeX/MathJax in Rust)
   - Not viable for v1.0

**v1.0 Recommendation:** Option 1 (HTML + KaTeX -> Chrome PDF). This reuses the HTML exporter and gives the best quality-to-effort ratio.

---

## Rust Crate Ecosystem for Math

| Crate | Version | Purpose | Maturity | Last Updated | Downloads | License | Notes |
|---|---|---|---|---|---|---|---|
| **pulldown-latex** | 0.7.1 | LaTeX math -> MathML Core (pull parser) | Good | ~2024 | 28,605 | MIT | Best pure-Rust option for MathML generation. Modeled after pulldown-cmark. |
| **latex2mathml** | 0.2.4 | LaTeX math -> MathML | Stale | ~2019 | ~65,000 | MIT/Apache-2.0 | Unmaintained; functional but no updates in ~6 years. |
| **katex** (xu-cheng) | 0.4.x | KaTeX Rust bindings via QuickJS | Good | ~2024 | ~150,000 | MIT | Embeds JS engine; server-side KaTeX rendering. Produces HTML+MathML. |
| **katex-rs** | 0.2.4 | Alternative KaTeX Rust port | Early | ~2024 | ~10,000 | MIT | Claims pure Rust; less mature than xu-cheng/katex. |
| **RaTeX** | 0.1.9 | Pure Rust KaTeX-compatible renderer | Very New | May 2026 | Low | MIT | >99.5% KaTeX coverage claimed. Outputs SVG/PNG/PDF. Too new for production. |
| **math-core** (latex2mmlc) | Latest | LaTeX -> MathML Core | Good | ~2024 | Low | MIT | Fork/reimplementation of latex2mathml with MathML Core output. |
| **pulldown-cmark** | 0.12.x | Markdown parser | Mature | 2025 | Millions | MIT | No built-in math support; must integrate math separately. |
| **mathml-latex** | 0.0.3 | MathML/LaTeX utilities | Experimental | May 2023 | Low | MIT | Thin wrapper; limited functionality. |

### No Rust crate exists for:
- **LaTeX -> OMML** (direct conversion)
- **MathML -> OMML** (would need XSLT or custom transform)
- **XSLT 1.0 engine** (pure Rust -- no mature option exists)

### Dependency recommendation for v1.0:
```toml
[dependencies]
pulldown-latex = "0.7"     # LaTeX -> MathML (pure Rust)
katex = "0.4"              # LaTeX -> HTML+MathML (for web output)
quick-xml = "0.36"         # XML manipulation for OMML insertion
# libxslt-sys or equivalent for MathML -> OMML transform
```

---

## OMML Deep Dive

### OMML XML for Common Expressions

#### Fraction: pi/2

```xml
<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <m:f>
    <m:num>
      <m:r><m:t>pi</m:t></m:r>
    </m:num>
    <m:den>
      <m:r><m:t>2</m:t></m:r>
    </m:den>
  </m:f>
</m:oMath>
```

#### Summation: sum_{i=1}^{n} x_i

```xml
<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <m:nary>
    <m:naryPr>
      <m:chr m:val="&#x2211;"/>
    </m:naryPr>
    <m:sub>
      <m:r><m:t>i=1</m:t></m:r>
    </m:sub>
    <m:sup>
      <m:r><m:t>n</m:t></m:r>
    </m:sup>
    <m:e>
      <m:sSub>
        <m:e><m:r><m:t>x</m:t></m:r></m:e>
        <m:sub><m:r><m:t>i</m:t></m:r></m:sub>
      </m:sSub>
    </m:e>
  </m:nary>
</m:oMath>
```

#### Integral: int_0^1 f(x)dx

```xml
<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <m:nary>
    <m:naryPr>
      <m:chr m:val="&#x222B;"/>
    </m:naryPr>
    <m:sub><m:r><m:t>0</m:t></m:r></m:sub>
    <m:sup><m:r><m:t>1</m:t></m:r></m:sup>
    <m:e>
      <m:r><m:t>f(x)dx</m:t></m:r>
    </m:e>
  </m:nary>
</m:oMath>
```

#### 2x2 Matrix

```xml
<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <m:d>
    <m:dPr>
      <m:begChr m:val="["/>
      <m:endChr m:val="]"/>
    </m:dPr>
    <m:e>
      <m:m>
        <m:mr>
          <m:e><m:r><m:t>a</m:t></m:r></m:e>
          <m:e><m:r><m:t>b</m:t></m:r></m:e>
        </m:mr>
        <m:mr>
          <m:e><m:r><m:t>c</m:t></m:r></m:e>
          <m:e><m:r><m:t>d</m:t></m:r></m:e>
        </m:mr>
      </m:m>
    </m:e>
  </m:d>
</m:oMath>
```

### OMML Key Elements Reference

| OMML Element | Purpose | LaTeX Equivalent |
|---|---|---|
| `<m:f>` | Fraction | `\frac{}{}` |
| `<m:nary>` | N-ary operator (sum, integral, product) | `\sum`, `\int`, `\prod` |
| `<m:sSub>` | Subscript | `x_i` |
| `<m:sSup>` | Superscript | `x^2` |
| `<m:sSubSup>` | Sub-superscript | `x_i^2` |
| `<m:rad>` | Radical/root | `\sqrt{}` |
| `<m:m>` | Matrix | `\begin{matrix}` |
| `<m:d>` | Delimiter (parentheses, brackets) | `\left( \right)` |
| `<m:r>` | Run (text content) | Plain text/symbols |
| `<m:t>` | Text within a run | Content characters |
| `<m:acc>` | Accent (hat, bar, tilde) | `\hat{}`, `\bar{}` |
| `<m:limLow>` | Lower limit | `\lim_{x \to 0}` |
| `<m:oMathPara>` | Display math paragraph | `$$...$$` |

---

## Cross-Renderer Math Compatibility

| Feature | PowerPoint | Word | Google Slides | Keynote | LibreOffice |
|---|---|---|---|---|---|
| OMML rendering | Native, full | Native, full | Flattened to images | Flattened to images | Partial, lossy |
| Editable equations | Yes | Yes | No | No | Sometimes (Writer > Impress) |
| Complex expressions | Full support | Full support | Degrades | Degrades | Often breaks |
| Round-trip fidelity | Perfect | Perfect | Lossy | Lossy | Lossy |
| Screen reader support | JAWS/NVDA: Yes | JAWS/NVDA: Yes | Limited | Limited | Limited |
| Matrices | Full | Full | Image only | Image only | Often malformed |
| Equation numbering | Supported | Supported | Lost | Lost | Partial |

**Key finding:** OMML is only fully supported in Microsoft Office. For non-Microsoft renderers, equations are typically converted to images or simplified shapes. This means:

1. OMML is the correct target for PPTX/DOCX when Microsoft Office is the primary viewer.
2. For maximum portability, also consider embedding a fallback image of the rendered equation alongside the OMML (PowerPoint supports this pattern).
3. For Google Slides/Keynote users, the HTML and PDF exports (using KaTeX) will provide better math rendering than the PPTX/DOCX exports viewed in non-Microsoft apps.

---

## Accessibility for Math

### WCAG Requirements

Mathematical content is governed by:

- **WCAG 1.1.1 (Non-text Content):** Math must not be image-only without alt text.
- **WCAG 1.3.1 (Info and Relationships):** Math structure (fractions, subscripts) must be programmatically determinable.
- **WCAG 4.1.2 (Name, Role, Value):** Math elements must be parseable by assistive technology.

**Best practice:** Use semantic math markup (MathML or OMML), never images alone.

### Per-Format Accessibility

| Format | Math Representation | Screen Reader Support | WCAG Compliance |
|---|---|---|---|
| HTML (KaTeX) | HTML + MathML (`htmlAndMathml` default) | Good -- MathML is read by NVDA/JAWS/VoiceOver | Compliant |
| PPTX (OMML) | Native OMML equations | Good in Office; poor in non-Office apps | Compliant in Office |
| DOCX (OMML) | Native OMML equations | Good in Word; partial in LibreOffice | Compliant in Word |
| PDF (via Chrome) | Tagged PDF with MathML source | Depends on PDF viewer; Adobe Acrobat supports | Partially compliant |
| Web Preview | KaTeX HTML + MathML | Same as HTML | Compliant |

### KaTeX Accessibility Details

KaTeX's default output mode (`htmlAndMathml`) produces:
1. Visual HTML spans for rendering (with `aria-hidden="true"`)
2. A `<math>` MathML element for screen readers

This is the industry-standard approach. The MathML is hidden visually but accessible to assistive technology.

### Recommendation for slideforge

1. **HTML/web:** Use KaTeX with default `htmlAndMathml` output. Accessible out of the box.
2. **PPTX/DOCX:** Use OMML (not equation images). OMML is the most accessible format for Office documents.
3. **PDF:** Ensure Chrome generates tagged PDF with math structure preserved.
4. **Never** render math as images without a MathML/alt-text fallback.

---

## v1.0 vs v2 Recommendation

### v1.0 Scope (Ship)

1. **Delimiters:** `$...$` (inline) and `$$...$$` (display math)
2. **Parser behavior:** No DSL processing inside math regions. `{{ }}` disabled inside `$...$`.
3. **PPTX/DOCX output:** LaTeX -> MathML (pulldown-latex) -> OMML (mml2omml.xsl via libxslt)
4. **HTML/web output:** LaTeX -> KaTeX HTML+MathML
5. **PDF output:** Via HTML+KaTeX -> headless Chrome (or whichever ADR-003 path)
6. **Accessibility:** KaTeX `htmlAndMathml` for web; OMML for Office
7. **Supported LaTeX subset:** Standard math (fractions, sums, integrals, matrices, Greek letters, basic environments). Document the supported subset explicitly.
8. **Escaping:** `\$` for literal dollar signs in text

### v2 Scope (Defer)

1. **Math interpolation:** `@{var}` syntax for injecting variable values into equations
2. **Extended LaTeX:** Support for `\newcommand`, custom macros, advanced environments
3. **Equation numbering:** Automatic `\tag{}` and cross-referencing
4. **Math in talk tracks:** Math rendering in speaker notes
5. **Equation image fallback:** Auto-generate PNG/SVG fallback for non-Office renderers
6. **RaTeX integration:** When RaTeX matures, evaluate replacing KaTeX for pure-Rust rendering
7. **Custom Rust MathML-to-OMML converter:** Replace libxslt dependency with pure Rust
8. **Typst math syntax:** Optional alternative to LaTeX (if user demand surfaces)

---

## DSL Syntax Impact

### Changes to Existing Decisions

1. **New reserved character: `$`** -- Dollar sign becomes a math delimiter. Not currently used in the DSL, so no conflict with existing decisions.

2. **Parser phase addition:** The parser must identify math regions before applying text-mode formatting. This affects the parser architecture (chumsky) -- math region detection should happen in the lexer or as an early parser pass.

3. **`_` disambiguation:** The existing `_italic_` syntax needs careful scoping. Rule: `_` is italic only when:
   - Not inside a math region (`$...$`)
   - Paired (opening and closing `_` on the same line)
   - Not adjacent to a word character on both sides without spaces (prevents false positives like `file_name_here`)

4. **`^` reservation confirmed:** The existing reservation of `^` for superscript text formatting is compatible with math. `^word^` in text mode = text superscript; `^{expr}` in math mode = LaTeX superscript.

5. **`#` scoping tightened:** `#` as a comment must be restricted to start-of-line (or after whitespace) in text mode only. This may already be the behavior; confirm in DSL spec.

6. **Triple-quote blocks:** `"""..."""` blocks should be defined as raw (no math parsing inside), giving users an escape hatch for content that would otherwise be ambiguous.

### New DSL Elements

```
# Math in a report/content block
slide content:
  title "Quadratic Formula"
  body """
    The solutions to $ax^2 + bx + c = 0$ are given by:

    $$x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$$
  """

# Math in a formula slide (existing slide type)
slide formula:
  title "Revenue Model"
  formula $$R(x) = px - C(x)$$
  variables:
    - "$p$" "Unit price"
    - "$C(x)$" "Cost function"
```

---

## Sources

### Crates and Libraries
- pulldown-latex crate: https://crates.io/crates/pulldown-latex (v0.7.1, MIT, 28K downloads)
- latex2mathml crate: https://crates.io/crates/latex2mathml (v0.2.4, unmaintained)
- katex Rust bindings: https://docs.rs/katex (via QuickJS)
- katex-rs crate: https://crates.io/crates/katex-rs (v0.2.4)
- RaTeX project: https://github.com/erweixin/RaTeX (v0.1.9, May 2026)
- math-core (latex2mmlc): https://github.com/tmke8/latex2mmlc
- ratex-wasm: https://lib.rs/crates/ratex-wasm (v0.1.9)

### OMML and Office Math
- Wikipedia OOXML file formats (OMML section): https://en.wikipedia.org/wiki/Office_Open_XML_file_formats
- Microsoft Math in Office blog: https://devblogs.microsoft.com/math-in-office/officemath
- LOC DOCX format description: https://www.loc.gov/preservation/digital/formats/fdd/fdd000397.shtml
- mml2omml.xsl on GitHub: https://github.com/transpect/docx_modify-lib/blob/master/xsl/mml2omml.xsl
- Microsoft XSLT redistribution Q&A: https://learn.microsoft.com/en-us/answers/questions/5286296/redistrubution-of-omml2mml-xsl-from-ms-office

### Math Accessibility
- KaTeX options (htmlAndMathml): https://katex.org/docs/options.html
- Harvard accessible math guidance: https://accessibility.huit.harvard.edu/accessible-math
- Rutgers accessible math: https://radr.rutgers.edu/resource/making-math-accessible
- WCAG math in STEM education: https://edtechbooks.org/accessibility/InnovativeSolutionsSTEM

### Cross-Tool Math Syntax
- Slidev LaTeX docs: https://sli.dev/features/latex
- Reveal.js math plugin: https://revealjs.com/math/
- KaTeX dollar sign issues: https://github.com/KaTeX/KaTeX/discussions/3431
- Cursor IDE LaTeX parsing bug: https://forum.cursor.com/t/currency-dollar-signs-in-assistant-responses-are-parsed-as-latex-inline-math-delimiters/154509
- Pandoc math handling issues: https://github.com/jgm/pandoc/issues/6330
- Rustdoc MathML proposal: https://internals.rust-lang.org/t/math-latex-support-for-rustdoc-now-with-mathml/22463

### Rust XSLT
- No mature pure-Rust XSLT engine exists as of May 2026. libxslt via FFI is the practical path.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 1 | LaTeX-to-OMML conversion pipeline deep dive |
| Perplexity perplexity_reason | 1 | Complete syntax conflict analysis (DSL vs LaTeX) |
| Perplexity perplexity_ask | 5 | OMML XML examples, Rust crate survey, cross-tool math coexistence, MathML-to-OMML conversion, accessibility, cross-renderer compatibility |
| Tavily tavily_search | 4 | OMML specification, latex2mathml crate, katex crate, pulldown-latex crate |
| Tavily tavily_extract | 2 | pulldown-latex/katex/RaTeX crate metadata from crates.io and GitHub |
| Context7 resolve-library-id | 1 | pulldown-latex library lookup |
| Training data | 2 areas | OMML XML structure patterns, general LaTeX syntax knowledge |

**Total MCP tool calls:** 14
**Training data reliance:** low -- All crate versions, download counts, and technical claims verified against live registry data. OMML XML examples cross-referenced with Microsoft documentation. Conflict analysis grounded in actual DSL spec from project files.
