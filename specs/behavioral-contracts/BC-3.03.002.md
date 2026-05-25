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

# BC-3.03.002: Strict Mode Produces No Output on Validation Error

## Description

In strict mode (the default build mode, without `--warn-only`), any validation error
prevents all output files from being written. No partial output is ever produced when
the deck has validation errors. This is an all-or-nothing contract that preserves
document integrity — a partially rendered deck with unknown errors could be mistakenly
shared as if it were correct.

## Preconditions

1. The `slideforge build` command is run WITHOUT the `--warn-only` flag.
2. The deck has at least one validation error (E-EVL-*, E-DAT-*, E-A11-*, E-LAY-002, or E-LAY-003).
3. The parse stage has succeeded (no E-PAR-* errors — this BC covers the validation phase).

## Postconditions

1. Zero output files are written to the output directory.
2. All validation errors are reported with their source spans.
3. Build exits with code 2 (EXIT_VALIDATION_ERROR).
4. The output directory is NOT created if it did not previously exist (no empty directory left behind).
5. If the output directory DID previously exist, it is NOT modified (no partial output).

## Invariants

1. The all-or-nothing invariant: either ALL requested output formats are written, or NONE are. (DI-017)
2. Strict mode is the default — `--warn-only` must be explicitly opt-in.
3. Validation errors accumulate before this check (per DI-018); the no-output decision is made after all errors are collected.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Multiple validation errors: accessibility + canvas overflow | All errors reported; zero output; exit 2 |
| EC-002 | Only canvas overflow errors (E-LAY-001) | By default, canvas overflow is a WARNING not an error. Zero output is NOT triggered by E-LAY-001 alone unless `--strict-overflow` is set. |
| EC-003 | Mix of warnings and errors | Zero output (errors are present); warnings listed but do not change the no-output decision |
| EC-004 | Zero-slide deck (E-LAY-002) | Zero output; exit 2 (E-LAY-002 is a validation error) |
| EC-005 | --warn-only flag present | This BC does not apply. See BC-3.03.003 for warn-only behavior. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck missing alt text on 1 image; `slideforge build deck.sf` | E-A11-001 reported; no .pptx written; exit 2 | happy-path (strict mode) |
| Valid deck; `slideforge build deck.sf` | Output written; exit 0 | happy-path |
| Deck with E-A11-001; `slideforge build deck.sf --warn-only` | E-A11-001 as warning; .pptx written with error-slide; exit 0 | edge-case (warn-only, BC-3.03.003) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | No output file exists in output dir after a strict-mode build with validation errors | integration test: check output dir contents |
| VP-TBD | All-or-nothing: cannot produce pptx but not docx when both are requested and there is a validation error | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 |
| Capability Anchor Justification | CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 — "Strict mode (default build): validation errors produce no output" is stated verbatim in CAP-022 |
| L2 Domain Invariants | DI-017 (strict mode produces no output on validation error) |
| Architecture Module | slideforge-cli crate — output gate logic (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.03.003 — supersedes (warn-only mode overrides this BC's no-output behavior)
- BC-3.03.001 — related to (canvas overflow uses degraded severity; this BC covers broken severity errors)
- BC-5.01.001 — related to (missing alt text is one source of validation errors gated here)
- BC-1.15.002 — depends on (error accumulation ensures all errors are known before this gate applies)

## Architecture Anchors

- `architecture/pipeline.md#output-gate` — strict mode output gate

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
