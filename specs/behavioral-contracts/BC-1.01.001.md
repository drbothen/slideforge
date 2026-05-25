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
capability: CAP-001
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

# BC-1.01.001: Parse Valid .sf Source into Typed AST with Error Accumulation

## Description

The slideforge parser accepts a syntactically correct .sf source file and produces
a fully typed AST. All errors encountered during parsing are accumulated in a single
pass — the parser never fails on the first error. The resulting AST faithfully
represents the deck structure including all slide blocks, vars, set rules, includes,
and metadata.

## Preconditions

1. A .sf file exists at the specified source path and is readable.
2. `slideforge_version "N"` is declared in the deck metadata.
3. The file uses spaces (not tabs) for indentation.
4. Indentation is consistent (multiples of the declared indent width).
5. All referenced `@include` paths resolve to readable files with no cycles.

## Postconditions

1. A typed AST is produced representing the complete deck structure.
2. The AST preserves: deck metadata, vars blocks, set rules, slide blocks (in order), @for blocks, @if/@elif/@else blocks, @include/import directives.
3. Zero diagnostics are emitted for a valid source.
4. The build exits with code 0.
5. No output files are written yet (this is the parse stage only; layout/export follow).

## Invariants

1. The AST is a deterministic function of the source text — same source always produces same AST.
2. All source spans in the AST reference valid positions in the input file(s).
3. `@include` directives are fully inlined before the AST is returned (include graph resolved).
4. Parser error accumulation: if ANY parse error exists, no AST is returned to subsequent stages.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Source file is empty | E-LAY-002: zero slides detected at validation stage (not parse stage) |
| EC-002 | Source file contains only metadata (vars, set rules) but no slide blocks | AST produced; zero-slide error raised at validation stage, not parse stage |
| EC-003 | Multi-file deck where one @include file is missing | E-PAR-005 with resolved path; all other errors accumulated before halting |
| EC-004 | @include file itself contains parse errors | Parse errors from included file reported with the included file's path, not the entry file |
| EC-005 | Deck declares slideforge_version "2" but binary only supports "1" | E-PAR-010; exit 1 immediately (forward-incompatible version is always fatal) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Minimal valid deck (TV-1.1: slideforge_version, lang, brand, 1 title slide) | AST with 1 slide node, 0 diagnostics, exit 0 | happy-path |
| Valid 25-slide deck with all 31 slide types | AST with 25 slide nodes, 0 diagnostics, exit 0 | happy-path |
| Deck with 2 independent indentation errors (TV-1.2) | 2 E-PAR-001 errors accumulated, exit 1, no AST | error |
| Deck with tab character at line 3 | E-PAR-003 at line 3, plus any other accumulated errors | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Parser is deterministic: same source → same AST | proptest (round-trip property) |
| VP-TBD | All error spans reference valid file positions (line ≤ file line count) | proptest |
| VP-TBD | Error accumulation: parse of N-error input reports exactly N errors | unit test with fixtures |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 |
| Capability Anchor Justification | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 — this BC defines the exact parse-to-AST contract that CAP-001 specifies as the entry point for all system value |
| L2 Domain Invariants | DI-018 (error accumulation in one pass) |
| Architecture Module | slideforge-syntax crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.01.002 — depends on (indentation error reporting uses same accumulation)
- BC-1.01.004 — composes with (include resolution is part of parse stage)
- BC-1.15.002 — depends on (this BC demonstrates the DI-018 error accumulation invariant)

## Architecture Anchors

- `architecture/system-overview.md` — chumsky 0.10 parser combinator design
- `architecture/system-overview.md` — Parse stage in the six-stage pipeline

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
