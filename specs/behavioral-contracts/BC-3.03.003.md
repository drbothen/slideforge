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
capability: CAP-022
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

# BC-3.03.003: Warn-Only Mode Renders Error-Slide Placeholders and Continues

## Description

When `slideforge build --warn-only` is used (or when running in `slideforge watch`),
validation errors that would normally block output in strict mode instead produce an
error-slide placeholder at the affected position. The build continues and produces
output with all non-erroring slides rendered normally. This enables iterative
authoring workflows where broken slides should not block the rest of the deck.

## Preconditions

1. `--warn-only` flag is present on `slideforge build`, OR the build is running via
   `slideforge watch` (watch mode implies warn-only per Q17 decision).
2. One or more validation errors are present that would be blocking in strict mode
   (E-EVL-*, E-DAT-*, E-A11-001, E-A11-002 categories).

## Postconditions

1. For each error that would block in strict mode: an error-slide placeholder is
   inserted at the slide's position in the output. The placeholder shows the error
   message prominently (red/orange banner, error code, file:line:col).
2. All non-erroring slides are rendered normally and included in the output.
3. All validation errors are still reported in the CLI output (warnings, not errors).
4. Output files ARE written to disk.
5. Exit code is 0 (warn-only: errors demoted to warnings).

## Invariants

1. Error-slide placeholders preserve the deck's slide count and order. A broken slide
   at position N is replaced by an error-slide at position N — not skipped.
2. Canvas overflow (E-LAY-001) is already a warning in default mode and does NOT
   produce an error-slide placeholder; it produces only a CLI warning.
3. Parse errors (E-PAR-*) are ALWAYS fatal — `--warn-only` does not demote parse errors.
   The deck must parse successfully before warn-only can apply.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Multiple slides each have E-EVL errors | Each produces an error-slide placeholder; non-erroring slides render normally |
| EC-002 | All slides have errors | Output file contains N error-slide placeholders; still written; exit 0 |
| EC-003 | --warn-only with a parse error (E-PAR-*) | Parse error is fatal regardless; build halts at parse stage; no output |
| EC-004 | Error-slide placeholder position in PPTX | The error-slide uses the Blank layout (no content overflows); error text as a large centered text shape |
| EC-005 | `--warn-only` + `--strict-overflow` | Flags compose: overflow remains fatal (exit 2); all other validation errors are demoted to warnings. No E-CFG-003 emitted. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 5-slide deck, slide 3 has E-A11-001 (missing alt); `--warn-only` | 5 slides in output: slides 1,2,4,5 normal; slide 3 is error-slide; E-A11-001 shown as warning; exit 0 | happy-path |
| Deck with parse error + `--warn-only` | E-PAR-* is fatal; no output; exit 1 | edge-case |
| Deck with no errors + `--warn-only` | Normal output; 0 warnings; exit 0 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Slide count in output equals slide count in source (broken slides replaced, not removed) | integration test (count slides in .pptx vs. slide blocks in .sf) |
| VP-TBD | Parse errors remain fatal in warn-only mode | integration test (deck with parse error + --warn-only → no output) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 |
| Capability Anchor Justification | CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 — "Warn-only (watch mode): output with error-slide placeholders" is stated verbatim in CAP-022 |
| L2 Domain Invariants | DI-017 (strict mode no-output invariant — this BC is the explicit exception to DI-017) |
| Architecture Module | slideforge-cli crate — warn-only flag; slideforge-eval crate — error-slide placeholder generation (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.03.002 — supersedes (this BC overrides BC-3.03.002's no-output behavior when --warn-only is active)
- BC-3.03.001 — related to (canvas overflow warnings appear in both strict and warn-only modes)
- BC-5.05.001 — depends on (watch mode uses warn-only semantics)

## Architecture Anchors

- `architecture/system-overview.md#warn-only` — warn-only mode output gate logic

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
