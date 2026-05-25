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
capability: CAP-002
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

# BC-1.02.004: Handle ${{ seq }} as Text Interpolation, Not Math Delimiter

## Description

The sequence `${{` in a field value is a special-case token. `$` immediately followed
by `{{` is parsed as the start of a text-mode variable interpolation, not as the start
of a math mode block. This prevents a currency or dollar-sign character before an
expression from accidentally toggling math mode. This covers DEC-009 from the domain
edge-case catalog.

## Preconditions

1. A field value contains the character sequence `${{` in text mode.
2. The lexer is in text mode (not inside `$...$` or `$$...$$`).

## Postconditions

1. `${{ expr }}` is parsed as a text-mode interpolation with the literal `$` prefix prepended to the result.
2. Math mode is NOT activated.
3. The expression inside `{{ }}` is evaluated normally as a text expression.
4. The resulting string value is `$` + evaluated expression result.

## Invariants

1. `${{` is always text-mode interpolation; it cannot activate math mode.
2. `$` as a standalone character followed by a space or non-`{` is not an interpolation trigger.
3. `$${{` (two dollars then brace) is NOT a math display block followed by interpolation; it is `$$` (math display open) followed by literal `{{`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-009) | `title "Revenue: ${{ kpis.arr | currency }}"` | `$` is a literal prefix; result = "Revenue: $1,234,567" |
| EC-002 | `title "Cost: $ {{ price }}"` (space between $ and {{) | `$` is literal text; `{{ price }}` is normal interpolation; result = "Cost: $ <value>" |
| EC-003 | `title "${{ amount }} USD"` | Parsed as `$` + interpolated amount + " USD" |
| EC-004 | `$${{ foo }}` | `$$` opens math display mode; `{{ foo }}` inside is NOT text interpolation — use `@{foo}` in math context |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `vars: { arr: 1000000 }` / `stat "${{ arr | currency }}"` | stat="$1,000,000" (string, text mode) | happy-path (DEC-009) |
| `title "${{ count }} items"` with count=5 | title="$5 items" | happy-path |
| `stat "$${{ n }}"` | E-EVL-002 if n undefined in math context; math mode activated by `$$` | boundary |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `${{` never activates math mode in text context | unit test with lexer state inspection |
| VP-TBD | Output of `${{ expr }}` equals "$" + to_string(eval(expr)) | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Variable Interpolation and Expression Evaluation") per capabilities.md §CAP-002 |
| Capability Anchor Justification | CAP-002 ("Variable Interpolation and Expression Evaluation") per capabilities.md §CAP-002 — the `${{` ambiguity is a specified corner case of the expression evaluation grammar |
| L2 Edge Cases | DEC-009 (math mode delimiters inside string fields) |
| Architecture Module | slideforge-syntax crate — lexer mode switching (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.02.001 — depends on (normal expression evaluation is BC-1.02.001; this is the `${{` special case)
- BC-1.10.001 — related to (math mode `$...$` delimiter rules)

## Architecture Anchors

- `architecture/system-overview.md` — text mode vs math mode lexer state machine

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
