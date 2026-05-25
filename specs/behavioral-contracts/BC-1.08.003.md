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

# BC-1.08.003: brand.* References in set Rules Are Evaluated After Brand Loading

## Description

When a `set` rule value references `brand.*` tokens, those tokens must not be resolved
during parse time or before the brand is loaded. The order of declaration in the .sf
file — whether the `set` rule appears before or after the `brand:` metadata field —
must not affect the resolved value. Brand references in set rules are always deferred
to the brand-load phase of the pipeline.

## Preconditions

1. A `set <type>: <field> brand.<token>` declaration exists in the .sf source.
2. A brand configuration is declared (either via `template:` pointing to a .pptx/.docx, or `brand:` pointing to a brand.toml).
3. The referenced `brand.<token>` exists in the loaded brand configuration.

## Postconditions

1. The `brand.<token>` value in the set rule resolves to the correct token value from the loaded brand — regardless of where in the file the set rule is declared.
2. If the brand is loaded after the set rule appears in source order, the evaluation is still correct (deferred resolution).
3. If the brand token does not exist in the loaded brand, a compile error names the missing token and lists available tokens.
4. If no brand is loaded at all, E-BRD-001 is emitted (brand required).

## Invariants

1. Brand token resolution for set rules happens in the Evaluate phase, not the Parse phase.
2. Declaration order in the .sf file does not affect brand token resolution (deferred evaluation).
3. This BC directly covers DEC-005: brand color token used before brand is loaded.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-005) | `set severity_cards: color_high brand.danger` appears before `template: "corp.pptx"` in file | brand.danger resolves correctly; declaration order is irrelevant |
| EC-002 | `brand.nonexistent` in a set rule, valid brand loaded | Compile error: "Unknown brand token 'nonexistent'. Available tokens: [...]" |
| EC-003 | No brand configured at all | E-BRD-001 brand-not-found; set rule evaluation never reached |
| EC-004 | brand.toml has a token that conflicts with a DSL keyword | Token is accessible via `brand.*` dotted path; no conflict (different namespace) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| set rule with `brand.primary` at top of file; brand loaded from brand.toml at bottom | All matching slides use brand.primary value; exit 0 | happy-path (DEC-005) |
| `set content: color brand.undefined_token` with otherwise valid brand | Compile error naming unknown token; exit 4 or 2 | error |
| Brand file missing entirely; set rule with brand ref present | E-BRD-001; exit 4; no output | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Same set rule with brand ref produces same output regardless of brand declaration position in file | unit test: swap declaration order, verify identical AST evaluation |
| VP-TBD | Missing brand token in set rule produces compile error with token name | unit test with fixture |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Set Rules for Slide-Type Defaults") per capabilities.md §CAP-008 |
| Capability Anchor Justification | CAP-008 ("Set Rules for Slide-Type Defaults") per capabilities.md §CAP-008 — "Supports brand.* references" is called out in CAP-008; DEC-005 is the canonical edge case |
| L2 Domain Invariants | (none directly — deferred evaluation is an implementation constraint, not a domain invariant) |
| Architecture Module | slideforge-eval crate — brand load order (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.08.001 — composes with (this BC is a property of set rule value evaluation)
- BC-1.08.002 — composes with (brand.* is one of the value types supported in set rules)

## Architecture Anchors

- `architecture/system-overview.md` — brand token resolution phase; Evaluate phase ordering: brand load precedes set rule evaluation

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
