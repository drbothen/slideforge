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

# BC-1.10.002: Unsupported LaTeX Command Produces Error with Source Span and Hint

## Description

Not all LaTeX commands are supported by the slideforge MathRenderer (pulldown-latex +
KaTeX subset). When a math expression contains an unsupported LaTeX command, the system
produces a compile error with the source file:line:col span pointing into the math block,
the name of the unsupported command, and a hint pointing to the supported LaTeX subset
in the DSL reference documentation.

## Preconditions

1. A `$...$` or `$$...$$` math block exists in the source with a syntactically well-formed LaTeX expression.
2. The expression contains a LaTeX command (`\something`) that is not in the supported subset.

## Postconditions

1. E-EXP-006 is emitted with: the file:line:col span within the math block, the unsupported command name, a hint: "See supported LaTeX subset in DSL reference."
2. The build exits with code 3 (export error) in strict mode.
3. In `--warn-only` mode: an error-slide placeholder is rendered for the slide containing the math expression.
4. All other errors in the file continue to be accumulated (DI-018).

## Invariants

1. The error always names the specific unsupported command — never a generic "math error."
2. The source span references the position within the math block, not just the slide block.
3. DI-018: this error is accumulated alongside other errors; it does not halt further error collection.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two unsupported commands in same math block | Both reported; accumulated per DI-018 |
| EC-002 | `\newcommand` (user-defined macro) — not supported in v1.0 | E-EXP-006 with hint that user-defined macros require v2+ |
| EC-003 | Unsupported command in display math (`$$...$$`) | Same error format; span points inside display block |
| EC-004 | Supported command used with wrong number of arguments | E-EXP-006 variant: "command \frac requires 2 arguments, got 1" |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `$\alpha + \beta$` (both supported) | Renders successfully; exit 0 | happy-path |
| `$\undefinedcmd{x}$` | E-EXP-006 at math block location; "unsupported command: \undefinedcmd"; exit 3 | error |
| `$\alpha$ and $\badcmd$` in same slide | Both the good block renders and E-EXP-006 for bad block | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Unsupported command error names the command exactly | unit test with fixture |
| VP-TBD | Span in E-EXP-006 points within the math block, not at the slide keyword | unit test: verify line number is inside the $...$ delimiters |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-012 ("Math and LaTeX Rendering") per capabilities.md §CAP-012 |
| Capability Anchor Justification | CAP-012 ("Math and LaTeX Rendering") per capabilities.md §CAP-012 — "every error carries file:line:col span and correction hint" is explicitly required for math rendering errors |
| L2 Domain Invariants | DI-018 (error accumulation in one pass) |
| Architecture Module | slideforge-math crate — MathRenderer error path (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.10.001 — depends on (this BC is the error path for math expressions)
- BC-1.15.001 — composes with (error carries file:line:col span per BC-1.15.001)
- BC-1.15.002 — depends on (error accumulation applies here)

## Architecture Anchors

- `architecture/authoring-subsystem.md#math-mode` — MathRenderer supported command list

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
