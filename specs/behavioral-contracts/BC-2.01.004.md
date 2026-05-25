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

# BC-2.01.004: brand.toml Missing Color Slots — Synthesize with Derived Defaults and Warn

## Description

When `brand.toml` declares fewer than 12 OOXML color slots, the brand synthesizer
infers the missing slots from the declared colors using a deterministic derivation
algorithm. Each inferred slot triggers one E-BRD-003 cosmetic warning. The build
continues with the synthesized palette. This contract covers DEC-016 (missing required
color slot edge case).

## Preconditions

1. `brand.toml` is valid TOML and references at least one color in `[colors]`.
2. At least one OOXML color slot is absent (fewer than 12 declared).
3. The synthesis stage is active (invoked from `Brand::synthesize()`).

## Postconditions

1. All 12 OOXML scheme color slots are populated in the resulting `BrandTemplate`.
   No slot is left empty or null. (DI-015)
2. For each inferred slot, exactly one E-BRD-003 cosmetic warning is emitted naming
   the slot and the source color it was derived from.
3. Inferred slots use deterministic logic: dk1 = darkest declared color; lt1 = white
   (#FFFFFF); dk2 = brand_primary if declared, else dk1 lightened 20%; remaining acc
   slots filled from brand accent colors or Material-palette-derived variants.
4. The brand.toml file is NOT modified (inference is in-memory only).
5. Build exits with code 0.

## Invariants

1. The number of E-BRD-003 warnings equals exactly (12 − number of explicitly declared slots). (DI-015)
2. Derivation is deterministic: same brand.toml with same missing slots always produces
   the same inferred colors.
3. All inferred colors are valid 6-digit uppercase hex RGB (no alpha, no CSS names).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | brand.toml has only acc1 and dk1 (DEC-016 canonical) | 10 E-BRD-003 warnings; all 10 remaining slots inferred; brand valid |
| EC-002 | brand.toml is completely empty (no [colors] section at all) | 12 E-BRD-003 warnings; entire palette derived from default 1898 & Co. baseline; build continues |
| EC-003 | brand.toml has all 12 slots declared | 0 warnings; no inference; exact declared values used |
| EC-004 | Inferred color for acc1 conflicts with dk1 (low contrast) | E-A11-004 contrast warning emitted for the affected layout if contrast ratio < 4.5:1 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| brand.toml: `[colors] brand_primary = "#003766"` only | BrandTemplate with 12 colors; 11 E-BRD-003 warnings (all slots except acc1 inferred); exit 0 | happy-path |
| brand.toml: `[colors]` section present but empty | 12 E-BRD-003 warnings; default palette used; exit 0 | edge-case |
| brand.toml: all 12 colors declared | BrandTemplate with exact colors; 0 warnings; exit 0 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Inferred color count = 12 − declared color count (always) | unit test (count E-BRD-003 warnings vs. declared slots) |
| VP-TBD | Derivation is deterministic: same partial brand.toml always produces same Brand struct | proptest (same input → identical BrandTemplate) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 |
| Capability Anchor Justification | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 — synthesizing complete brand from partial brand.toml is a core synthesis path in CAP-018; DEC-016 names this exact edge case |
| L2 Domain Invariants | DI-015 (brand palette must cover all 12 OOXML slots) |
| Architecture Module | slideforge-brand crate — BrandSynthesizer (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-2.01.002 — composes with (BC-2.01.002 covers the full synthesis case; this BC covers the missing-slot sub-path)
- BC-2.01.001 — related to (missing slot inference also applies when extracting from .pptx with incomplete theme)

## Architecture Anchors

- `architecture/brand-architecture.md#synthesis` — color slot derivation algorithm
- `architecture/brand-architecture.md#layout-taxonomy` — 31 layout taxonomy (Spike S5)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
