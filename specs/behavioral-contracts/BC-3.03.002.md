---
document_type: behavioral-contract
level: L3
version: "1.2"
status: active
producer: product-owner
timestamp: 2026-06-11T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-022
lifecycle_status: active
introduced: v1.0.0
modified: ["v1.2 — rendering-fix wave (REND-005/STORY-098): W-VAL-103 content-drop sub-case promoted to broken/exit-2 in strict mode. Precondition 2 expanded. Postcondition 3 rewritten to cover content-drop promotion. Invariant 4 added. EC-006 added. Route A selected (reclassify sub-case within W-VAL-103 as context-sensitive severity; no new E-VAL-105 code)."]
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
2. The deck has at least one **blocking** diagnostic: either (a) a validation error
   (E-EVL-*, E-DAT-*, E-A11-*, E-LAY-002, or E-LAY-003), or (b) a W-VAL-103
   **content-drop** sub-case (see Invariant 4).
3. The parse stage has succeeded (no E-PAR-* errors — this BC covers the validation phase).

## Postconditions

1. Zero output files are written to the output directory.
2. All blocking diagnostics are reported with their source spans.
3. **Exit code 2** (EXIT_VALIDATION_ERROR) is returned. This includes the W-VAL-103
   content-drop sub-case (see Invariant 4): when W-VAL-103 is emitted because an
   authored content field (`shape:` or `body` on a slide type that does not support it)
   would be silently dropped, it is promoted to a **broken** diagnostic in strict mode
   and triggers exit 2. W-VAL-103 on purely non-content unknown fields (metadata-only
   fields with no content impact) retains its cosmetic/exit-0 classification.
4. The output directory is NOT created if it did not previously exist (no empty directory left behind).
5. If the output directory DID previously exist, it is NOT modified (no partial output).

## Invariants

1. The all-or-nothing invariant: either ALL requested output formats are written, or NONE are. (DI-017)
2. Strict mode is the default — `--warn-only` must be explicitly opt-in.
3. Validation errors accumulate before this check (per DI-018); the no-output decision is made after all errors are collected.
4. **W-VAL-103 content-drop sub-case (Route A — context-sensitive severity):** W-VAL-103
   has TWO severity contexts:
   - **Content-drop context** (broken/exit-2 in strict mode): W-VAL-103 is emitted for an
     unknown field whose presence would cause authored content to be silently dropped
     (specifically: `shape:` on a slide type that does not support custom shapes, or `body`
     on a slide type whose schema rejects `body`). In strict mode, this IS a build failure
     (exit 2, no output). In `--warn-only`, the field is dropped with a warning and the
     build continues. Silent content dropping in default strict mode is prohibited per
     CLAUDE.md "no silent fallback" and the production-grade default principle.
   - **Non-content context** (cosmetic/exit-0): W-VAL-103 is emitted for an unknown field
     that has no content impact (e.g., a metadata annotation not recognized by the slide
     type). This remains cosmetic and does not trigger exit 2.
   The implementer in `validate_fields` distinguishes the two contexts by checking whether
   the unknown field key is in the set `{"shape", "body"}` (content-bearing DSL fields
   on unsupported types). All other unknown fields remain cosmetic. **No new error code
   E-VAL-105 is introduced** — Route A is chosen (reclassification of the sub-case, not
   a new code). The W-VAL-103 message format is UNCHANGED; the diagnostic severity is
   determined at accumulation time by the field-key check.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Multiple validation errors: accessibility + canvas overflow | All errors reported; zero output; exit 2 |
| EC-002 | Only canvas overflow errors (E-LAY-001) | By default, canvas overflow is a WARNING not an error. Zero output is NOT triggered by E-LAY-001 alone unless `--strict-overflow` is set. |
| EC-003 | Mix of warnings and errors | Zero output (errors are present); warnings listed but do not change the no-output decision |
| EC-004 | Zero-slide deck (E-LAY-002) | Zero output; exit 2 (E-LAY-002 is a validation error) |
| EC-005 | --warn-only flag present | This BC does not apply. See BC-3.03.003 for warn-only behavior. |
| EC-006 | W-VAL-103 for `shape:` on a slide type that does not support shapes (e.g., `slide title:` with a `shape:` field) in strict mode | Exit 2; "Unknown field 'shape' for slide type 'title'..."; zero output (content-drop sub-case, broken severity in strict mode) |
| EC-007 | W-VAL-103 for `body` on `content` slide type (schema-invalid combination) in strict mode | Exit 2; "Unknown field 'body' for slide type 'content'..."; zero output (content-drop sub-case per body/content schema drift fix in STORY-098) |
| EC-008 | W-VAL-103 for an unknown metadata annotation (non-content field) in strict mode | Exit 0; cosmetic warning emitted; output produced (non-content sub-case, cosmetic severity) |

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
| L2 Domain Invariants | DI-017 (strict mode produces no output on validation error), DI-018 (error accumulation — all diagnostics collected before gate fires) |
| Architecture Module | slideforge-cli crate — output gate logic; slideforge-plugin-api validate_fields — W-VAL-103 context-sensitive severity |
| Stories | STORY-098 (W-VAL-103 content-drop promotion + body/content schema drift) |

## Related BCs

- BC-3.03.003 — supersedes (warn-only mode overrides this BC's no-output behavior)
- BC-3.03.001 — related to (canvas overflow uses degraded severity; this BC covers broken severity errors)
- BC-5.01.001 — related to (missing alt text is one source of validation errors gated here)
- BC-1.15.002 — depends on (error accumulation ensures all errors are known before this gate applies)

## Architecture Anchors

- `architecture/system-overview.md` — strict mode output gate

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
