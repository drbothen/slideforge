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
capability: CAP-020
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

# BC-5.01.004: Missing Deck lang Declaration Produces Lint Warning; Default Is "en"

## Description

Every deck should declare `lang "..."` in its metadata (e.g., `lang "en-US"`). If the
`lang` field is absent, the build produces E-A11-003 as a cosmetic lint warning
(severity: cosmetic, exit 0) and defaults to "en" for all output formats. The warning
is emitted in both strict and warn-only modes. Non-English content in a deck with no
`lang` declaration risks incorrect screen-reader pronunciation but does not block output.

## Preconditions

1. A .sf deck file is being parsed and validated.
2. The deck metadata block does NOT contain a `lang "..."` declaration.

## Postconditions

1. E-A11-003 is emitted: `Missing lang declaration in deck metadata. Defaulting to "en".
   Screen readers may mispronounce non-English content. Add lang "en-US" (or appropriate
   BCP-47 tag).`
2. Build continues regardless of mode (cosmetic severity, exit 0).
3. Output is produced with the default lang "en" embedded in all formats (PPTX, PDF, HTML).
4. The warning does NOT count toward strict-mode blocking errors.

## Invariants

1. E-A11-003 is always cosmetic (exit 0) — unlike E-A11-001 and E-A11-002 which are
   blocking in strict mode. (DI-003 — missing lang is a warning, not a compile error)
2. The default language "en" is used when lang is absent — it is NEVER null in IR or
   output formats.
3. A `lang` value of empty string is treated as missing (E-A11-003 emitted).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Deck with `lang "en-US"` declared | No E-A11-003; "en-US" propagates to all output formats |
| EC-002 | Deck with `lang ""` (empty string) | E-A11-003 emitted; defaults to "en" |
| EC-003 | Deck with `lang "zh-Hant-TW"` (4-part BCP-47 tag) | No E-A11-003; "zh-Hant-TW" propagates correctly |
| EC-004 | Deck with other validation errors AND missing lang | All errors accumulated; E-A11-003 among them; exit code is highest severity error |
| EC-005 | Variant that overrides lang | Variant may declare `lang "fr-FR"` to override deck-level lang |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck with no `lang` field | E-A11-003 warning; output produced; PPTX dc:language = "en" | happy-path (warn) |
| Deck with `lang "de"` | No E-A11-003; PPTX dc:language = "de"; HTML `<html lang="de">` | happy-path |
| Deck with `lang ""` | E-A11-003; defaults to "en" | edge-case |
| Deck with missing lang + validation errors | All errors in output; exit code = max severity (2 if blocking errors present) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E-A11-003 emitted exactly once for deck with no lang | unit test |
| VP-TBD | Output produced (not blocked) when only E-A11-003 is present | unit test |
| VP-TBD | Default "en" embedded in PPTX dc:language when lang absent | unit test: parse PPTX core.xml |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 |
| Capability Anchor Justification | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 — "lang 'en-US' at deck level" with lint warning if absent is the exact language of CAP-020 |
| L2 Domain Invariants | DI-003 (deck language must be declared; propagates to all output formats) |
| Architecture Module | slideforge-validate crate (SS-03) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.01.005 — composes with (this BC establishes the lang value; BC-5.01.005 specifies propagation)
- BC-5.01.001 — related to (same compile-time accessibility enforcement pattern; different severity)

## Architecture Anchors

- `architecture/plugin-architecture.md` — lang declaration handling

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
