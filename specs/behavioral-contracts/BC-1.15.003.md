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
capability: CAP-030
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

# BC-1.15.003: Parse Errors Are Always Fatal; Validation Errors Fatal in Strict Mode Only

## Description

The slideforge diagnostic system applies a three-tier severity model to determine
build behavior. Parse errors (E-PAR) are always fatal: they prevent AST construction
and therefore prevent any output, regardless of the `--warn-only` flag. Validation
errors (E-EVL, E-DAT, E-LAY, E-A11) are fatal in strict mode (the default) but
produce error-slide placeholders and continue building in `--warn-only` mode. Lint
warnings (cosmetic severity) are always advisory — they never prevent output.

## Preconditions

1. A .sf source is submitted for building.
2. The build may have zero or more errors across severity categories.

## Postconditions

1. Parse errors (E-PAR-*): always produce exit 1; no output files written; no `--warn-only` override.
2. Validation errors (E-EVL-*, E-DAT-*, E-LAY-*, E-A11-*) in strict mode (default): exit 2; no output files written.
3. Validation errors with `--warn-only`: exit 0; output files written with error-slide placeholders at affected positions; warnings emitted to stderr.
4. Export errors (E-EXP-*): always produce exit 3; no output files written.
5. Lint/cosmetic diagnostics (E-BRD-003, E-BRD-004, E-A11-003): exit 0; output files written; warnings emitted to stderr.
6. Exit code is the HIGHEST severity code across all accumulated errors.

## Invariants

1. `--warn-only` NEVER demotes parse errors (E-PAR) to warnings — they remain fatal.
2. `--warn-only` NEVER demotes export errors (E-EXP) to warnings — they remain fatal.
3. In strict mode, partial output (some slides rendered, some failed) is NEVER written to disk. All-or-nothing per DI-017.
4. In `--warn-only` mode, error-slide placeholders are written AT the position of the failed slide (not appended at the end).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Mix of parse error + validation error in one build | Exit 1 (parse takes precedence); no output |
| EC-002 | `--warn-only` + parse error | Parse error still fatal; exit 1; no output |
| EC-003 | Only cosmetic diagnostics | Exit 0; full output; warnings on stderr |
| EC-004 | `--warn-only` + 3 validation errors | Exit 0; output with 3 error-slide placeholders; 3 warnings on stderr |
| EC-005 | Zero errors of any kind | Exit 0; full output; empty stderr |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Source with E-PAR-001 (tab char) | Exit 1; no output; parse error on stderr | happy-path (strict) |
| Source with E-EVL-001 in strict mode | Exit 2; no output | happy-path (strict) |
| Source with E-EVL-001 + `--warn-only` | Exit 0; output with error-slide placeholder; warning on stderr | happy-path (warn-only) |
| Source with E-PAR-001 + `--warn-only` | Exit 1; no output (parse error overrides warn-only) | edge-case |
| Source with only E-BRD-003 (cosmetic) | Exit 0; full output; warning on stderr | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Parse errors are fatal even with --warn-only (property: E-PAR in strict = E-PAR in warn-only for exit code) | unit test: build with E-PAR + --warn-only; verify exit 1 |
| VP-TBD | No partial output in strict mode with any error severity ≥ degraded | integration test: verify no output file written when validation error present in strict mode |
| VP-TBD | Exit code is max of all error exit codes | unit test: build with E-PAR (1) + E-EVL (2); verify exit 2... wait: parse wins → exit 1 | unit test with known exit-code ordering |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-030 ("Diagnostic Reporting with Source Spans") per capabilities.md §CAP-030 |
| Capability Anchor Justification | CAP-030 ("Diagnostic Reporting with Source Spans") per capabilities.md §CAP-030 — "Three-tier severity: parse errors (always fatal), validation errors (fatal in strict, warnings in warn-only), lint (always warnings)" is verbatim from CAP-030 |
| L2 Domain Invariants | DI-017 (strict mode produces no output on validation error), DI-018 (error accumulation in one pass) |
| Architecture Module | slideforge-cli crate — exit code determination and output-gate logic (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.15.001 — composes with (all errors referenced here carry spans per BC-1.15.001)
- BC-1.15.002 — composes with (all errors here are accumulated per BC-1.15.002)
- BC-3.03.002 — depends on (strict-mode no-output behavior)
- BC-3.03.003 — depends on (warn-only error-slide placeholder behavior)

## Architecture Anchors

- `architecture/system-overview.md` — severity tier definitions, exit code mapping, and output gate in the final pipeline stage

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
