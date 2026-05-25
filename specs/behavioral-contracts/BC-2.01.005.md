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

# BC-2.01.005: Brand Synthesizer Produces All 31 Layouts (11 Standard + 20 Custom)

## Description

The `BrandSynthesizer` generates a complete OOXML slide master containing exactly 31
slide layouts — 11 standard Office layout types and 20 custom slideforge layouts (CL-01
through CL-20) — as established in Spike S5. Each layout has correct placeholder
structures, semantic names, and relationship back-pointers to the slide master. Two dark
layouts (CL-01 and CL-11) carry `clrMapOvr` overrides.

## Preconditions

1. `BrandConfig` has been populated (from brand.toml synthesis or .pptx loading).
2. All 12 OOXML color slots are populated in the `BrandConfig` (guaranteed by BC-2.01.002
   or BC-2.01.004 prior to this stage).

## Postconditions

1. The synthesized `BrandTemplate` contains exactly 31 `slideLayout` entries indexed
   as `slideLayout1.xml` through `slideLayout31.xml`.
2. All 11 standard layout types are present with their canonical OOXML `type` attribute
   values (`title`, `obj`, `secHead`, `twoObj`, `twoColTx`, `titleOnly`, `blank`, `objTx`,
   `picTx`, `vertTitleAndTx`, `vertTx`).
3. All 20 custom layouts are present with names prefixed `"SF "` (e.g., "SF Section
   Divider", "SF Stat Grid") and correct placeholder structures per Spike S5 §1.4.
4. Layouts CL-01 (SF Section Divider) and CL-11 (SF End Slide) contain `clrMapOvr`
   with `bg1="dk2" tx1="lt1"`. These layouts also carry explicit white text colors on
   placeholder runs (R4 mitigation from Spike S5 §6.1).
5. Every layout has a `_rels` file back-pointing to `slideMaster1.xml`.
6. `[Content_Types].xml` registers all 31 layouts.
7. `notesMaster1.xml` and `handoutMaster1.xml` stubs are present (even if empty content).

## Invariants

1. Layout count is exactly 31 — never 30 or 32. (DI-015, Spike S5 §1.1)
2. Every layout EXCEPT the Blank layout has a `title` or `ctrTitle` placeholder for
   screen reader accessibility. (Spike S5 §2.3)
3. Slide master IDs start at 2^31 (2147483648); layout IDs start at 2^31+1.
4. All `<p:cNvPr name="...">` attributes use semantic labels, not "Shape N". (Spike S5 §2.3)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | brand.toml has a logo PNG > 5 MB | Logo embedded into slide master; lint warning about file size; no error |
| EC-002 | Brand has no logo declared | Logo placeholder on master is empty/absent; layout thumbnails show no logo; no error |
| EC-003 | Synthesized .pptx opened in LibreOffice 7.x (old clrMapOvr support) | Dark layouts (CL-01, CL-11) still show white text because explicit white run colors are written (R4 mitigation) |
| EC-004 | `slideforge extract-brand` run on synthesized .pptx | All 31 layouts detected; round-trip colors match brand.toml (Spike S5 §3.3) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Full brand.toml → synthesize | .pptx contains exactly 31 `<p:sldLayout>` elements | happy-path |
| Synthesized brand → open in PowerPoint | Zero "repair" warnings; all 31 layouts visible in layout picker | happy-path |
| Synthesized brand opened via quick-xml | `<a:clrMapOvr>` present on slideLayout12.xml (CL-01) and slideLayout22.xml (CL-11) | edge-case |
| Synthesized brand, blank layout | `<p:sldLayout type="blank">` has 0 placeholders (not a missing-title error) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Synthesized .pptx has exactly 31 `<p:sldLayout>` elements | snapshot test (count sldLayout elements in package) |
| VP-TBD | Every layout except Blank has title or ctrTitle placeholder | unit test (iterate layouts, check placeholder type) |
| VP-TBD | CL-01 and CL-11 have clrMapOvr and explicit white text runs | unit test (parse layout XML, check elements) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 |
| Capability Anchor Justification | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 — the 31-layout requirement is the structural output of the brand synthesis path; Spike S5 establishes this as the definitive taxonomy |
| L2 Domain Invariants | DI-015 (brand palette must cover all 12 OOXML slots), DI-016 (single master architecture) |
| Architecture Module | slideforge-brand crate — BrandSynthesizer (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-2.01.002 — depends on (this BC is called after the BrandConfig is populated by BC-2.01.002)
- BC-4.01.005 — depends on (PPTX exporter verifies slide IDs start at 256 and master IDs at 2^31)
- BC-4.01.006 — depends on (notesMaster and handoutMaster stubs required by PPTX export correctness)

## Architecture Anchors

- `architecture/brand-architecture.md#layout-taxonomy` — 31-layout taxonomy (Spike S5)
- `architecture/brand-architecture.md#synthesis` — BrandSynthesizer API and two-phase template application

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
