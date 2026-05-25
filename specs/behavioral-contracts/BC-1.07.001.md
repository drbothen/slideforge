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
capability: CAP-007
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

# BC-1.07.001: variants: Block with include_tags/exclude_tags Produces Filtered Deck

## Description

A `variants:` block declares named audience variants, each with `include_tags` and/or
`exclude_tags` lists. When a build targets a specific variant (via `--variant <name>`
or the default variant), slides are filtered: only slides with a tag in `include_tags`
are included (if specified), and slides with any tag in `exclude_tags` are excluded.
The filtered subset forms the output deck.

## Preconditions

1. A `variants:` block exists in the deck with one or more named variant entries.
2. At least one variant declares `include_tags` and/or `exclude_tags`.
3. The build is invoked with `--variant <name>` or a default variant is configured.

## Postconditions

1. The output deck contains only slides matching the filter criteria: slides whose tags intersect with `include_tags` (when specified) and do not intersect with `exclude_tags`.
2. Slide order is preserved from the source deck.
3. Build exits with code 0 when at least one slide survives filtering. See BC-1.07.005 for zero-slide handling.

## Invariants

1. A slide with no tags is included unless `include_tags` is specified (explicit opt-in required).
2. `include_tags` and `exclude_tags` use list union semantics: a slide is included if any of its tags are in `include_tags`.
3. `exclude_tags` takes precedence: if a slide's tag is in both include and exclude lists, it is excluded.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Variant with only `include_tags: ["exec"]`; slides without tags | Untagged slides excluded (include_tags means explicit opt-in) |
| EC-002 | Variant with only `exclude_tags: ["internal"]`; slides without tags | Untagged slides included (exclude is opt-out only) |
| EC-003 | Slide tagged both `exec` and `internal`; variant includes exec, excludes internal | Slide excluded — exclude takes precedence |
| EC-004 | `--variant` flag without a variants: block | E-CFG-001: variant not declared; exit 4 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 10 slides, 5 tagged `exec`; exec variant with `include_tags: ["exec"]` | 5-slide output | happy-path |
| 10 slides, 3 tagged `internal`; exec variant with `exclude_tags: ["internal"]` | 7-slide output | happy-path |
| All slides excluded by variant filters | Warning per BC-1.07.005; exit 2 in strict mode | boundary |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Filtered output count equals slides matching include/exclude criteria | proptest (arbitrary tag sets) |
| VP-TBD | Slide order is preserved in filtered output | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Variant-Based Deck Segmentation") per capabilities.md §CAP-007 |
| Capability Anchor Justification | CAP-007 ("Variant-Based Deck Segmentation") per capabilities.md §CAP-007 — tag-based filtering is the core mechanism of variant deck segmentation |
| L2 Domain Invariants | DI-017 (strict mode produces no output on validation error) |
| Architecture Module | slideforge-eval crate — variant filter (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.07.002 — composes with (variant vars: override builds on the filtered deck)
- BC-1.07.003 — depends on (inheritance graph must be acyclic before filtering)
- BC-1.07.004 — related to (--variant flag with undefined variant)
- BC-1.07.005 — composes with (empty-result warning)

## Architecture Anchors

- `architecture/system-overview.md` — variant filter algorithm

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
