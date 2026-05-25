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

# BC-1.07.004: Reject --variant Flag Referencing Undefined Variant (E-CFG-001)

## Description

When the `--variant <name>` CLI flag specifies a variant name that is not declared in
the deck's `variants:` block, the build fails with E-CFG-001, showing the undefined
name and the complete list of defined variant names. The build does NOT fall back to
a no-variant build. This covers DEC-020 from the domain edge-case catalog.

Note: E-EVL-005 was previously cited here. It has been retired — `--variant` is a CLI
configuration flag (not an evaluator concern), so undefined-variant is a configuration
error at exit code 4, not an evaluation error at exit code 2.

## Preconditions

1. `slideforge build deck.sf --variant <name>` is invoked.
2. `<name>` does not match any variant declared in the deck's `variants:` block.

## Postconditions

1. E-CFG-001 is emitted: `Variant '<name>' not defined in deck. Defined variants: [<list>]. Did you mean '<closest>'?`
2. The error lists all declared variant names.
3. A "did you mean" suggestion is provided for the closest match (Levenshtein distance ≤ 3).
4. Build exits with code 4 (EXIT_CONFIG_ERROR).
5. No output is produced; the build does NOT fall back to the full (no-variant) deck.

## Invariants

1. `--variant` flag with undefined name is always an error — no silent fallback.
2. Case sensitivity: variant names are case-sensitive; `Exec` ≠ `exec`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-020) | `--variant internal-exec` where only `exec` and `full` are defined | E-CFG-001: variants: [exec, full]. Did you mean 'exec'? |
| EC-002 | `--variant Exec` (wrong case) | E-CFG-001: 'Exec' not declared. Did you mean 'exec'? |
| EC-003 | Deck has no `variants:` block; `--variant` flag used | E-CFG-001: deck declares no variants; exit 4 |
| EC-004 | `--variant exec` where exec is correctly declared | Build proceeds normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slideforge build deck.sf --variant typo-exec` (defined: exec) | E-CFG-001: 'typo-exec' not defined in deck. Variants: [exec]. Did you mean 'exec'?; exit 4 | error (DEC-020) |
| `slideforge build deck.sf --variant exec` (declared) | Build succeeds with exec variant | happy-path |
| No variants: block, `--variant any` | E-CFG-001: deck declares no variants; exit 4 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E-CFG-001 always lists all defined variants | unit test |
| VP-TBD | Build never falls back to no-variant build on invalid --variant | unit test (assert no output written) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Variant-Based Deck Segmentation") per capabilities.md §CAP-007 |
| Capability Anchor Justification | CAP-007 ("Variant-Based Deck Segmentation") per capabilities.md §CAP-007 — the --variant flag is specified as a CLI interface for variant deck segmentation |
| L2 Edge Cases | DEC-020 (deck built with --variant that does not exist) |
| Architecture Module | slideforge-cli crate — variant flag validation (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.07.001 — depends on (filtering uses the validated variant)
- BC-1.07.005 — related to (zero-slide warning from valid variant)

## Architecture Anchors

- `architecture/system-overview.md` — CLI variant flag handling

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
