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
capability: CAP-018
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

# BC-2.01.006: Font Unavailable on Build Host Produces Warning and Uses Fallback

## Description

When `brand.toml` references a font (in `[fonts]`) that is not installed on the build
host, the brand synthesizer emits an E-BRD-004 cosmetic warning identifying the
unavailable font and the fallback being substituted. The build continues with the
fallback font embedded in the OOXML theme. The output document uses the declared font
name in the XML (so the correct font renders on the viewing machine if installed), but
text metrics may differ.

## Preconditions

1. `brand.toml` declares `[fonts] heading = "<FontName>"` or `[fonts] body = "<FontName>"`.
2. The declared font is not found in the build host's font directory (platform-specific
   search: `~/.fonts/`, `/usr/share/fonts/`, macOS system fonts, Windows C:\Windows\Fonts\).
3. A fallback font is available (always: Aptos or Calibri as last-resort fallback).

## Postconditions

1. Exactly one E-BRD-004 cosmetic warning is emitted per unavailable font, naming the
   unavailable font and the selected fallback.
2. The OOXML theme XML (`theme1.xml`) still writes the DECLARED font name — not the
   fallback — so the intended font renders on machines where it is installed.
3. The text layout engine uses the fallback font for build-time metric calculations
   (overflow detection, column width estimates). The warning notes this.
4. Build exits with code 0 (font unavailability is cosmetic, not a build failure).

## Invariants

1. Font unavailability NEVER causes a build failure. It is always cosmetic. (DI-015 does
   not require fonts to be installed; it requires color slots to be complete.)
2. The OOXML output always writes the user's declared font name, preserving rendering
   intent on fully-equipped machines.
3. The fallback font chain is deterministic: Aptos → Calibri → Arial → system sans-serif.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Both heading and body fonts unavailable | Two E-BRD-004 warnings (one each); build continues with fallback for both |
| EC-002 | Font is available on macOS but not Linux CI | E-BRD-004 on Linux CI runner; no warning on macOS; OOXML output identical |
| EC-003 | Font name contains a typo (e.g., "Calibr" instead of "Calibri") | E-BRD-004; fallback used; OOXML writes the misspelled name; fonts with that name don't exist anywhere |
| EC-004 | CJK font not found | E-BRD-004 for CJK font; Latin fonts unaffected; build continues |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| brand.toml heading = "Aptos Display" (not installed in CI environment) | E-BRD-004: "Font 'Aptos Display' not available. Using 'Calibri'."; theme1.xml writes "Aptos Display"; exit 0 | happy-path |
| brand.toml heading = "Arial" (universally installed) | 0 E-BRD-004 warnings; exit 0 | happy-path |
| brand.toml heading = "NonExistentFont" | E-BRD-004; fallback chain resolves to system sans-serif; OOXML writes "NonExistentFont" | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | OOXML theme1.xml always writes declared font name regardless of availability | unit test (mock font unavailable; inspect theme XML output) |
| VP-TBD | E-BRD-004 warning count equals number of unavailable fonts declared | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 |
| Capability Anchor Justification | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 — font configuration is part of brand synthesis; the font fallback contract ensures builds succeed across CI environments with different font installations |
| L2 Domain Invariants | DI-015 (brand palette completeness; this BC is the font analogue) |
| Architecture Module | slideforge-brand crate — BrandSynthesizer font resolution (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-2.01.002 — composes with (font resolution is a sub-step of BrandSynthesizer::synthesize)
- BC-2.01.004 — related to (missing color slots are the color analogue of missing fonts)

## Architecture Anchors

- `architecture/brand-architecture.md` — font availability check and fallback chain

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
