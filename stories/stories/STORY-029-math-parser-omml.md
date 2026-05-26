---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-029
title: "Math Parser: $...$ / $$...$$ + @{var} + OMML Output"
epic: EPIC-10
wave: 3
points: 8
priority: P1
tdd_mode: strict
status: draft
behavioral_contracts: [BC-1.10.001, BC-1.10.002, BC-1.10.003]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-math
target_module: slideforge-math
subsystems: [SS-13]
depends_on: [STORY-005, STORY-009, STORY-001]
blocks:
  - STORY-030
  - STORY-037
  - STORY-038
  - STORY-041
estimated_days: 4
---

# STORY-029: Math Parser — $...$ / $$...$$ + @{var} + OMML Output

## Subsystem Anchor Justification

SS-13 (Math) owns this story's scope because `slideforge-math` is the exclusive
owner of LaTeX parsing and OMML generation per ARCH-INDEX Subsystem Registry. The
parser subsystem (SS-01) handles the `$...$` delimiter lexing (STORY-009) and
passes math AST tokens to SS-13 for semantic processing.

## Dependency Anchor Justifications

- Depends on STORY-005 (Lexer): The mode-based lexer (text / math) produces
  `Token::MathInline` and `Token::MathDisplay` tokens with their inner content
  and source spans. `slideforge-math` parses from these tokens.
- Depends on STORY-009: The `@{var}` interpolation inside math blocks is recognized
  by the parser; `slideforge-math` substitutes variables before LaTeX parsing.
- Depends on STORY-001: `MathAst` is stored in `InlineNode::Math(MathAst)` from
  `slideforge-types`.
- Blocks STORY-030: MathML and PDF path output depend on the `MathAst` IR this
  story produces.
- Blocks STORY-037/038/041: PPTX and DOCX exporters embed OMML produced by this
  story's `MathRenderer` implementation.

## Summary

Implement `slideforge-math` as the `MathRenderer` plugin. The crate provides:

1. **LaTeX parsing**: Parse LaTeX from `$...$` / `$$...$$` tokens into a `MathAst`.
   Supported commands are the KaTeX/pulldown-latex subset. Unsupported commands
   produce `E-EXP-006` with source span and hint (BC-1.10.002).

2. **`@{var}` interpolation**: Before LaTeX parsing, substitute `@{var}` references
   within the math expression string using the resolved variable map.

3. **OMML output**: Serialize `MathAst` to OMML (`<m:oMath>`) XML for PPTX and DOCX
   embedding (BC-1.10.003 postconditions 1–2). OMML must be valid per ECMA-376
   Part 1 §22.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-1.10.001 | $...$ and $$...$$ toggle math mode with @{var} interpolation | AC-001, AC-002, AC-003 |
| BC-1.10.002 | Unsupported LaTeX command produces error with source span and hint | AC-004, AC-005 |
| BC-1.10.003 | Math renders to OMML for PPTX/DOCX (this story); MathML+PDF in STORY-030 | AC-006, AC-007 |

## Acceptance Criteria

### AC-001: $...$ and $$...$$ produce distinct MathAst nodes
(traces to BC-1.10.001 postconditions 1–2)

`$E = mc^2$` produces `MathAst { mode: MathMode::Inline, ... }`.
`$$\sum_{i=0}^{n} i = \frac{n(n+1)}{2}$$` produces `MathAst { mode: MathMode::Display, ... }`.
`MathMode` is an enum with variants `Inline` and `Display`. OMML uses `<m:oMath>` for
inline and `<m:oMathPara>` containing `<m:oMath>` for display mode.

### AC-002: {{ expr }} disabled inside math mode
(traces to BC-1.10.001 postcondition 3)

Inside a math block, `{{ ... }}` sequences are NOT evaluated. They appear as literal
characters in the LaTeX source. The `@{var}` substitution happens FIRST (before LaTeX
parsing); `{{ }}` handling does not run on math content.

### AC-003: @{var} inside math is substituted before parsing
(traces to BC-1.10.001 postcondition 4)

Given `"Mean: $\bar{x} = @{mean_val}$"` with `mean_val = "4.2"`, the substitution
step produces `"Mean: $\bar{x} = 4.2$"` before LaTeX parsing. The substituted value
is always a string. If `@{var}` references an undefined variable,
`E-EVL-002: @{undefined_var} undefined in math context` is emitted with source span.

### AC-004: Unsupported LaTeX command produces E-EXP-006
(traces to BC-1.10.002 postcondition 1)

`$\undefinedcmd{x}$` produces:
- Error code: `E-EXP-006`
- Message: `"Unsupported LaTeX command: \\undefinedcmd"`
- Hint: `"See supported LaTeX subset in DSL reference."`
- Source span: points to the position of `\undefinedcmd` within the math block
  (not just the slide line)
- Exit code: 3 (export error) in strict mode

### AC-005: Multiple unsupported commands accumulated
(traces to BC-1.10.002 EC-001 and DI-018)

Two unsupported commands in one math block produce two `E-EXP-006` diagnostics
accumulated via `DiagnosticSink`. Processing continues after the first error
(DI-018 — no halt on first error).

### AC-006: OMML output for PPTX/DOCX
(traces to BC-1.10.003 postconditions 1–2)

`MathRenderer::render_omml(ast: &MathAst) -> Result<String, MathError>` produces a
valid OMML XML string. For `$x^2$`:
```xml
<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <m:sSup>
    <m:e><m:r><m:t>x</m:t></m:r></m:e>
    <m:sup><m:r><m:t>2</m:t></m:r></m:sup>
  </m:sSup>
</m:oMath>
```
The namespace declaration `xmlns:m="..."` must be present on the root element.
For display math (`$$...$$`), the OMML is wrapped in `<m:oMathPara>`.

### AC-007: MathRenderer plugin trait implementation
(traces to BC-1.10.003 invariant 1 — single source AST for all format renderings)

`slideforge_math::MathRendererImpl` implements `slideforge_plugin_api::MathRenderer`:
```rust
pub trait MathRenderer: Send + Sync {
    fn parse(&self, source: &str, mode: MathMode, vars: &HashMap<Arc<str>, Value>)
        -> Result<MathAst, Vec<MathDiagnostic>>;
    fn render_omml(&self, ast: &MathAst) -> Result<String, MathError>;
    fn render_mathml(&self, ast: &MathAst) -> Result<String, MathError>;
    fn render_pdf_paths(&self, ast: &MathAst) -> Result<SvgPaths, MathError>;
}
```
`render_mathml` and `render_pdf_paths` are defined here (trait established) but
implemented in STORY-030. Their stub implementations return
`Err(MathError::NotYetImplemented)` until STORY-030.

### Supported LaTeX subset (v1.0)

The following commands are supported. All others produce `E-EXP-006`:

**Greek letters**: `\alpha`, `\beta`, `\gamma`, `\delta`, `\epsilon`, `\theta`,
`\lambda`, `\mu`, `\pi`, `\sigma`, `\phi`, `\omega` (and uppercase variants).

**Operators**: `\frac{num}{denom}`, `\sum`, `\prod`, `\int`, `\sqrt`, `\sqrt[n]`,
`^` (superscript), `_` (subscript), `\cdot`, `\times`, `\div`, `\pm`, `\mp`,
`\leq`, `\geq`, `\neq`, `\approx`, `\infty`.

**Delimiters**: `\left(`, `\right)`, `\left[`, `\right]`, `\left\{`, `\right\}`.

**Environments**: `\begin{align}...\end{align}`, `\begin{cases}...\end{cases}`.

**Accents**: `\hat`, `\bar`, `\vec`, `\dot`, `\ddot`.

**Forbidden in v1.0 (produce E-EXP-006 with specific hint)**:
- `\newcommand`: hint "user-defined macros require v2+"
- `\def`: hint "user-defined macros require v2+"
- `\begin{tikzpicture}`: hint "TikZ drawings are not supported in v1.0"

## Tasks

- [ ] Create `crates/slideforge-math/` with `Cargo.toml`
- [ ] Add `slideforge-math` to workspace members
- [ ] Define `MathAst`, `MathMode`, `MathNode`, `MathDiagnostic`, `MathError` types in `src/ast.rs`
- [ ] Implement `@{var}` substitution pass in `src/interpolation.rs`
- [ ] Implement LaTeX parser for supported command subset in `src/parser.rs` (using `pulldown-latex` or hand-recursive descent for the subset)
- [ ] Implement `E-EXP-006` error with source span for unsupported commands
- [ ] Implement OMML serializer in `src/omml.rs`: `MathAst → XML string`
- [ ] Declare `MathRenderer` trait in `slideforge-plugin-api` (if not already defined from STORY-002)
- [ ] Implement `MathRendererImpl` struct in `src/lib.rs`
- [ ] Stub `render_mathml` and `render_pdf_paths` with `Err(NotYetImplemented)`
- [ ] Write unit tests:
  - `$E = mc^2$` → correct OMML with `<m:sSup>`
  - `$$\sum$$` → display mode OMML with `<m:oMathPara>`
  - `@{val}` substitution with `val = "3.14"`
  - `\undefinedcmd` → E-EXP-006 with span pointing inside math block
  - two unsupported commands → two diagnostics accumulated
  - `${{arr | currency}}` → `{{arr | currency}}` treated as literal (not evaluated)
- [ ] Write `insta` snapshot tests for OMML output of representative expressions

## Previous Story Intelligence

N/A — first story in EPIC-10. STORY-009 (parser) established math mode tokenization;
that context feeds in as token stream input. The math crate is a new pure-core crate.

Key constraint from CLAUDE.md: pulldown-latex or equivalent must NOT use Node.js or
any subprocess. The entire math pipeline is a Rust library call chain.

## Architecture Compliance Rules

1. **SS-13 is Pure core**: No file I/O, no process spawning, no network. All math
   transformation is pure Rust library calls.
2. **Plugin trait compliance (DI-008)**: `MathRendererImpl` must use only the
   `slideforge-plugin-api::MathRenderer` public trait. No internal type bypasses.
3. **Single MathAst source (BC-1.10.003 invariant 1)**: OMML, MathML, and PDF paths
   are all produced from the same `MathAst`. The AST is parsed once; format rendering
   is a separate pass.
4. **ECMA-376 §22 OMML compliance**: OMML elements must be correctly namespaced
   (`xmlns:m`). Element order is schema-significant.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` | workspace | `MathAst`, `InlineNode`, `Value` |
| `slideforge-plugin-api` | workspace | `MathRenderer` trait |
| `pulldown-latex` | `=0.7.1` | LaTeX parsing — event-based (pull) parser API (or hand-rolled recursive descent if pulldown-latex API is incompatible — document the choice) |
| `quick-xml` | `=0.36.0` | OMML XML serialization |
| `thiserror` | `=2.0.18` | `MathError` |
| `insta` | `=1.39.0` | snapshot tests for OMML output |

NOTE: If `pulldown-latex 0.7.1` does not compile on all 5 platforms, fall back to
a hand-rolled recursive descent parser for the supported subset. Document the
decision in this story's implementation notes and flag for architect review.

pulldown-latex 0.7.x uses an EVENT-BASED (pull) parser API, not a tree-structured AST.
The library emits `Result<Event, ParserError>` items. To build the internal `MathAst`,
collect events into a tree structure in `src/parser.rs`. The built-in `push_mathml`/`write_mathml`
function produces MathML directly from events. For OMML output, the custom `src/omml.rs`
module walks the `MathAst` (not pulldown-latex events directly).

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-math/Cargo.toml` | Create | Crate manifest |
| `crates/slideforge-math/src/lib.rs` | Create | `MathRendererImpl` + public API |
| `crates/slideforge-math/src/ast.rs` | Create | `MathAst`, `MathNode`, `MathMode` |
| `crates/slideforge-math/src/parser.rs` | Create | LaTeX → MathAst parser |
| `crates/slideforge-math/src/interpolation.rs` | Create | `@{var}` substitution pass |
| `crates/slideforge-math/src/omml.rs` | Create | `MathAst → OMML XML` serializer |
| `crates/slideforge-math/src/error.rs` | Create | `MathError`, `MathDiagnostic` |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,800 |
| BC-1.10.001 + BC-1.10.002 + BC-1.10.003 | ~4,000 |
| `slideforge-types` InlineNode, Value | ~1,000 |
| `slideforge-plugin-api` MathRenderer trait | ~500 |
| OMML reference (ECMA-376 §22 relevant subset) | ~1,500 |
| Test files to write | ~3,000 |
| **Total** | **~12,800** |

## Test Strategy

- **Unit tests**: OMML roundtrip for each supported LaTeX structure (`^`, `_`,
  `\frac`, `\sum`, `\sqrt`, Greek letters); `@{var}` substitution; E-EXP-006
  for unknown commands with span validation; `${{` sequence treated as literal.
- **Snapshot tests (insta)**: OMML XML output for 10 representative expressions
  covering all supported command families.
- **Property test (light)**: `parse(source)` never panics for any ASCII input
  (returns error or AST, never panic).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `${{ arr \| currency }}` (dollar + brace = text mode) | DEC-009: treated as text interpolation at parse/eval layer; never reaches math parser |
| EC-002 | Unclosed `$..` (no closing `$`) | E-PAR-002: unclosed math delimiter at file:line:col |
| EC-003 | `@{undefined_var}` inside math | E-EVL-002 accumulated; `@{undefined_var}` substitution fails |
| EC-004 | `$$...$$` with nested `$...$` | Parse error: nested math delimiters not allowed |
| EC-005 | `\newcommand` | E-EXP-006 with hint "user-defined macros require v2+" |
| EC-006 | Display math in inline context | `MathMode::Display` in `MathAst`; OMML uses `<m:oMathPara>` |

## Forbidden Dependencies

`slideforge-math` MUST NOT depend on:
- Any exporter crate (`slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`,
  `slideforge-html`, `slideforge-preview`)
- `slideforge-data`, `slideforge-brand`, `slideforge-layout`, `slideforge-cli`
- Any Node.js or subprocess-based LaTeX renderer

If OMML output needs a Node.js call to MathJax, reject that approach entirely.
The OMML must be produced from a pure Rust library.
