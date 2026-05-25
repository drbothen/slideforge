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
capability: CAP-008
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

# BC-1.08.002: set Rules Support {{ }} Interpolation and brand.* References

## Description

Set rule values are not limited to string literals. They may contain `{{ expr }}`
interpolation using any variable in the deck scope, and they may reference brand
color/font tokens via `brand.*` dotted paths. This allows set rules to create
semantically meaningful defaults that adapt to the active brand and data context.

## Preconditions

1. A `set <type>: <field> {{ expr }}` or `set <type>: <field> brand.<token>` declaration exists.
2. All variables referenced in the expression are declared in the deck's `vars:` block or loaded via `@data`.
3. Brand tokens referenced via `brand.*` correspond to entries in the active brand configuration.

## Postconditions

1. `{{ expr }}` in a set rule is evaluated exactly as in slide field values — same expression grammar, same scope, same type rules.
2. `brand.<token>` in a set rule resolves to the brand token value after brand loading.
3. The evaluated value is substituted as the default for all matching slides.
4. If the expression references an undefined variable, E-EVL-001 is emitted (same error as in slide fields).
5. If the brand token does not exist, a compile error names the unknown token and lists available tokens.

## Invariants

1. `{{ expr }}` in set rules obeys DI-004 (no implicit type coercion) and DI-006 (undefined variables are compile errors).
2. `brand.*` references are resolved at brand-load time, not at set-rule declaration time.
3. Set rule expression evaluation is deterministic — same inputs produce same output.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `set content: color {{ brand.accent1 }}` — nested brand access via interpolation | Resolves to brand.accent1 hex value; functionally equivalent to `brand.accent1` direct ref |
| EC-002 | `set content: footer {{ undefined_var }}` | E-EVL-001 compile error with scope path |
| EC-003 | `brand.nonexistent_token` in set rule | Compile error naming unknown token; lists valid brand.* tokens |
| EC-004 | Set rule value is a computed expression `{{ base_font_size | add 2 }}` | Expression evaluated with arithmetic; result applied as numeric field default |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `vars: { co: "Acme" }` + `set title: footer "© {{ co }}"` | All title slides show footer "© Acme"; exit 0 | happy-path |
| `set severity_cards: color_high brand.danger` (brand has `danger: "#D00"`) | All severity_cards slides use #D00 for color_high | happy-path |
| `set content: footer {{ no_such_var }}` | E-EVL-001 at set rule location; exit 2 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | {{ }} in set rule uses same evaluator as slide field (property: evaluated value matches direct slide field) | unit test: compare set-rule slide output vs. explicit slide field output |
| VP-TBD | brand.* in set rule resolves after brand loading; if brand changes, set rule output changes | unit test: swap brand, verify set rule outcome changes |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Set Rules for Slide-Type Defaults") per capabilities.md §CAP-008 |
| Capability Anchor Justification | CAP-008 ("Set Rules for Slide-Type Defaults") per capabilities.md §CAP-008 — interpolation and brand refs in set values are explicitly stated in CAP-008 as a supported feature |
| L2 Domain Invariants | DI-004 (no implicit type coercion), DI-006 (undefined variables are compile errors) |
| Architecture Module | slideforge-eval crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.08.001 — composes with (this BC enriches the set rule value language)
- BC-1.08.003 — depends on (brand.* resolution timing is specified in BC-1.08.003)
- BC-1.02.001 — depends on ({{ expr }} evaluation rules are shared)

## Architecture Anchors

- `architecture/system-overview.md#set-rules` — set rule expression evaluation
- `architecture/system-overview.md#brand-load` — brand token resolution order

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
