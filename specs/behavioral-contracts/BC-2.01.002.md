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

# BC-2.01.002: Synthesize Complete Brand Template from brand.toml (All 12 OOXML Slots)

## Description

When a deck declares `brand "brand.toml"`, the brand synthesizer reads the TOML
configuration and produces a complete PPTX brand template in memory. The template
must populate all 12 standard OOXML theme color slots (dk1, lt1, dk2, lt2, acc1-acc6,
hlink, folHlink). The synthesized template includes exactly 31 layouts (11 standard +
20 custom slideforge layouts) per Spike S5 taxonomy.

## Preconditions

1. `brand.toml` exists at the referenced path and is valid TOML.
2. `brand.toml` declares at minimum one color (additional slots are synthesized if absent).
3. Font files referenced in `[typography]` are either installed on the build host or a fallback is available.

## Postconditions

1. A `Brand` struct is returned containing: 12 OOXML color slots (all populated), typography settings, logo reference, footer text.
2. The synthesized brand contains exactly 31 slide layouts (11 standard + 20 custom per Spike S5 taxonomy).
3. All 11 standard Office layout types are present with their canonical `type` attribute values.
4. Slide IDs in the master start at 2^31; slide IDs in the synthesized deck start at 256.
5. `notesMaster1.xml` and `handoutMaster1.xml` stubs are present in the synthesized PPTX package.
6. If any color slot was absent from brand.toml, each inferred slot emits one E-BRD-003 warning.

## Invariants

1. All 12 OOXML scheme color slots are ALWAYS populated — never partially synthesized. (DI-015)
2. The synthesized brand is structurally identical to what a human would produce via PowerPoint's "Edit Theme Colors" dialog.
3. Colors are represented as hex RGB (no alpha); never as named color references.
4. The synthesis is deterministic: same brand.toml → same Brand struct.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | brand.toml with only acc1 and dk1 defined (DEC-016) | 10 E-BRD-003 warnings listing inferred colors; synthesized brand is valid; build continues |
| EC-002 | brand.toml completely empty (no [colors] section) | 12 E-BRD-003 warnings; all colors inferred from default palette; build continues |
| EC-003 | brand.toml with all 12 slots defined | 0 warnings; synthesis produces the exact specified colors |
| EC-004 | Font referenced in [typography] not installed on build host | E-BRD-004 warning; fallback font used; text metrics may differ from target platform |
| EC-005 | Logo path referenced in [logo] does not exist | E-BRD-001 (fatal): logo path not found. Logo is required for synthesized brand. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Full brand.toml with all 12 colors declared | Brand struct with exact colors; 0 warnings; 31 layouts | happy-path |
| brand.toml with acc1="3B82F6" and dk1="1F2937" only | Brand struct; 10 E-BRD-003 warnings; 31 layouts | edge-case |
| brand.toml with acc1 declared; no logo | E-BRD-001: logo path required in synthesized brand | error |
| Synthesized PPTX written to disk | XML structure test: 31 `<p:sldLayout>` elements present | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Synthesized brand always has exactly 12 OOXML color entries | unit test (count dk1, lt1, dk2, lt2, acc1-6, hlink, folHlink) |
| VP-TBD | Synthesized PPTX has exactly 31 layouts | snapshot test (count sldLayout elements) |
| VP-TBD | Synthesis is deterministic (same input → same output) | proptest (same brand.toml → Brand structs are equal) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 |
| Capability Anchor Justification | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 — synthesize from brand.toml is the second synthesis path explicitly described in CAP-018 |
| L2 Domain Invariants | DI-015 (brand palette must cover all 12 OOXML slots), DI-016 (single master architecture) |
| Architecture Module | slideforge-brand crate — BrandSynthesizer (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-2.01.001 — related to (this BC is synthesis path; BC-2.01.001 is loading from existing .pptx)
- BC-2.01.004 — composes with (inferred color slot warnings produced by this BC)
- BC-2.01.005 — depends on (the 31-layout requirement is part of the synthesis output)
- BC-4.01.005 — depends on (PPTX export uses the brand produced here)

## Architecture Anchors

- `architecture/brand-architecture.md#synthesis` — brand.toml to OOXML Brand struct
- `architecture/brand-architecture.md#layout-taxonomy` — 31 layout taxonomy (Spike S5)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
