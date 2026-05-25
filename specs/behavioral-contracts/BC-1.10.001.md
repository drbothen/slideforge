---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-012
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.10.001: $...$ and $$...$$ Toggle Math Mode with @{var} Interpolation

## Description

The slideforge DSL uses `$...$` (inline) and `$$...$$` (display) delimiters to switch
from text mode to math mode within string fields. Inside math mode, `{{ expr }}`
text interpolation is disabled. Instead, `@{var}` is the interpolation syntax for
math-mode variable substitution. Math mode produces LaTeX-formatted expressions that
are rendered by the MathRenderer plugin to the appropriate output format.

## Preconditions

1. A string field in a slide block contains `$...$` or `$$...$$` delimiters.
2. For `@{var}` interpolation inside math: the variable `var` is declared in scope (vars: block or @data).
3. The LaTeX expression inside the delimiters is syntactically valid (handled by BC-1.10.002 for errors).

## Postconditions

1. Content inside `$...$` is parsed as inline math mode; content outside remains in text mode.
2. Content inside `$$...$$` is parsed as display (block) math mode.
3. Inside math mode, `{{ ... }}` sequences are NOT evaluated as text interpolation — they are literal characters in the LaTeX source.
4. `@{var}` inside math mode resolves to the variable's string/numeric value and splices it into the LaTeX expression before rendering.
5. Nested `$...$` inside `$$...$$` (or vice versa) is a parse error.
6. The sequence `${{` (dollar followed immediately by double-brace) is treated as text interpolation in text mode, not a math delimiter (DEC-009).

## Invariants

1. Math mode and text mode are mutually exclusive at any point in parsing — the parser maintains a mode flag.
2. `@{var}` is only valid inside math mode; in text mode `@{...}` is a parse error (or reserved syntax if used elsewhere).
3. DI-004 applies in math mode: `@{var}` must resolve to a declared variable; no implicit coercion.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-009) | `"Revenue: ${{ kpis.arr \| currency }}"` — `${{` sequence | Treated as text-mode interpolation (currency value), not math delimiter |
| EC-002 | Unclosed `$...` (no closing `$`) | E-PAR-002 style error: unclosed math delimiter at file:line:col |
| EC-003 | `@{undefined_var}` inside math block | E-EVL-002: @{undefined_var} undefined in math context |
| EC-004 | `$$...$$` with nested `$...$` inside | Parse error: nested math delimiters not allowed |
| EC-005 | `$...$` spanning multiple lines | Supported for inline math (single-line expressions preferred; multi-line display math uses `$$...\n...$$`) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `"The formula $E = mc^2$ is Einstein's"` | Text with inline math expression parsed; exit 0 | happy-path |
| `"Mean: $\bar{x} = @{mean_val}$"` with `mean_val = 4.2` | Math with @{mean_val} substituted as `4.2` before rendering | happy-path |
| `"Revenue: ${{ arr \| currency }}"` | Text interpolation applied (dollar + brace = text mode); exit 0 | edge-case (DEC-009) |
| `"Price: ${{ val }` (unclosed)` | E-PAR-002 unclosed delimiter; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `${{` is treated as text interpolation, not math delimiter | unit test: verify parse tree treats `${{ }}` as Inline::Interpolation |
| VP-TBD | @{var} in math mode resolves to variable value; {{ var }} in math mode is literal | unit test with fixture |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-012 ("Math and LaTeX Rendering") per capabilities.md §CAP-012 |
| Capability Anchor Justification | CAP-012 ("Math and LaTeX Rendering") per capabilities.md §CAP-012 — mode-based parsing with @{var} interpolation in math blocks is verbatim from CAP-012 |
| L2 Domain Invariants | DI-004 (no implicit type coercion), DI-006 (undefined variables are compile errors) |
| Architecture Module | slideforge-syntax crate — MathModeParser (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.10.002 — related to (error path for unsupported LaTeX commands)
- BC-1.10.003 — depends on (this BC produces math nodes; BC-1.10.003 renders them per format)
- BC-1.02.001 — related to ({{ expr }} evaluation is disabled in math mode)

## Architecture Anchors

- `architecture/system-overview.md` — mode-based parser design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
