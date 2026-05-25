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

# BC-3.03.001: Canvas Overflow Produces CanvasOverflow Warning with EMU Estimate

## Description

When the layout engine estimates that a slide's content exceeds the available canvas
height or width (e.g., a `bullets:` list that overflows the body placeholder), it emits
an E-LAY-001 `CanvasOverflow` warning. The warning includes the slide title, field name,
and an EMU estimate of the overflow amount. By default, this is a degraded-severity
warning (not a blocking error); output is still produced. The `--strict-overflow` flag
promotes it to a blocking error. This covers DEC-013.

## Preconditions

1. The layout engine has computed a geometric estimate for all content in a slide.
2. The estimated content height or width exceeds the placeholder's allocated dimensions.
3. The build is NOT in `--strict-overflow` mode (in that mode, see postcondition 5).

## Postconditions

1. E-LAY-001 warning is emitted:
   `CanvasOverflow: slide '<title>' field '<field>' overflows by ~<N> EMU (~<M>pt). Consider reducing content or font size.`
2. The warning includes file:line:col reference to the overflowing field.
3. Output is still written to disk (degraded severity, not blocking in default mode).
4. Exit code is 0 unless other blocking errors are present.
5. If `--strict-overflow` is set: E-LAY-001 is promoted to a blocking error; no output
   is written; exit code 2.

## Invariants

1. Canvas overflow is always a WARNING in default strict mode, never a blocking error
   (unless `--strict-overflow` is explicitly set). (DI-017 covers blocking errors only)
2. The EMU estimate is based on font metrics from the brand's declared fonts (or fallback
   fonts per BC-2.01.006). It is an ESTIMATE — actual rendering may differ.
3. Overflow detection runs at the validation stage (after evaluation, before export).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Multiple slides all overflow (DEC-013 at scale) | All E-LAY-001 warnings accumulated; all output produced; warnings listed at end |
| EC-002 | Overflow on a slide that is @if-excluded | No warning (the slide is not in the evaluated Deck IR) |
| EC-003 | --strict-overflow + --warn-only flags used together | Flags compose: overflow remains fatal (E-LAY-001 as error, exit 2); all other validation errors are demoted to warnings. No E-CFG-003 emitted. |
| EC-004 | Overflow in watch mode | Warning shown in web preview overlay and CLI; rendering continues with truncation |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide content:` with 20 bullet points that exceed the body placeholder | E-LAY-001 warning with slide title, "bullets" field, EMU estimate; output written; exit 0 | happy-path |
| Same deck with `--strict-overflow` | E-LAY-001 as blocking error; no output; exit 2 | edge-case |
| Deck with no overflow | 0 E-LAY-001 warnings; output written; exit 0 | happy-path |
| `--strict-overflow` + `--warn-only` | Overflow is fatal (exit 2); all other validation errors are warnings | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E-LAY-001 warning contains EMU value > 0 when overflow is detected | unit test (mock layout engine with known overflow amount) |
| VP-TBD | Without --strict-overflow, overflow warnings do not prevent output | integration test (check output files exist after build with overflow) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 |
| Capability Anchor Justification | CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 — "Validate canvas overflow" is explicitly listed as a CAP-022 validation; DEC-013 names canvas overflow as a domain edge case |
| L2 Domain Invariants | DI-017 (strict mode: no output on blocking errors — overflow is not blocking by default) |
| Architecture Module | slideforge-layout crate — overflow estimator; slideforge-validate crate — E-LAY-001 emitter (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.03.002 — related to (strict mode blocking errors; overflow is not one by default)
- BC-3.03.003 — related to (warn-only mode also produces output with overflow; error-slide placeholder for breaking errors)

## Architecture Anchors

- `architecture/layout-subsystem.md#overflow-detection` — overflow estimation algorithm

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
