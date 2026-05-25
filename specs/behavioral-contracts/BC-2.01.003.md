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

# BC-2.01.003: Extract brand.toml from Existing .pptx via slideforge extract-brand

## Description

The `slideforge extract-brand <template.pptx>` command reads an existing .pptx file
and produces a `brand.toml` file in the current directory. The extracted TOML maps all
12 OOXML theme color slots to human-readable semantic field names (text_primary, background,
brand_primary, etc.), extracts heading/body font names, and copies the logo image to
`brand.assets/logo.<ext>` if one is found in the slide master's media relationships.

## Preconditions

1. The source path resolves to a readable `.pptx` file.
2. The file is a valid OOXML ZIP package.
3. No `brand.toml` already exists at the output path, or `--force` flag is provided.

## Postconditions

1. A `brand.toml` is written containing `[colors]`, `[fonts]`, and (if detected) `[logo]`
   and `[footer]` sections as specified in Spike S5 Part 4.2.
2. All 12 OOXML color slots are represented; `sysClr` elements use their `lastClr`
   attribute as the hex value.
3. If a logo image is found in slideMaster1.xml.rels, it is copied to
   `brand.assets/logo.<ext>` and `[logo] path = "brand.assets/logo.<ext>"` is written.
4. Build exits with code 0.

## Invariants

1. Color extraction follows ECMA-376 sequential order (dk1, lt1, dk2, lt2, acc1-acc6,
   hlink, folHlink) — the extractor does NOT use random-access lookup. (DI-015)
2. The extraction is read-only; the source .pptx is never modified.
3. Semantic field names are fixed per Spike S5 taxonomy — they are not configurable.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | brand.toml already exists and --force not passed | Error: "brand.toml already exists. Use --force to overwrite." Exit 4. |
| EC-002 | Source .pptx has sysClr elements instead of srgbClr | Use lastClr attribute value; no error |
| EC-003 | Source .pptx has lumMod/tint/shade transforms on schemeClr | Write value as-is with inline comment warning: "derived via tint/shade; may not match exact color" |
| EC-004 | Source .pptx has multiple slide masters | Extract from slideMaster1.xml only; lint warning noting multi-master template |
| EC-005 | Logo has an unsupported image format in ppt/media/ | Copy the file anyway; log a warning about potential rendering differences |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Well-formed corporate .pptx (12 colors, heading+body fonts, logo) | brand.toml with all sections; brand.assets/logo.png written; exit 0 | happy-path |
| .pptx with sysClr for dk1 (Windows system color) | brand.toml text_primary = "#<lastClr value>"; exit 0 | edge-case |
| brand.toml already exists, no --force | Error: "brand.toml already exists. Use --force to overwrite."; exit 4 | error |
| .pptx with no logo in slide master | brand.toml written without [logo] section; exit 0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Extracted brand.toml round-trips: load the .pptx, extract to brand.toml, synthesize from brand.toml → verify color values match | integration test |
| VP-TBD | Source .pptx file is unmodified after extraction | integration test (file hash before/after) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 |
| Capability Anchor Justification | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 — bidirectional extraction ("slideforge extract-brand deck.pptx → brand.toml") is explicitly described in CAP-018 |
| L2 Domain Invariants | DI-015 (brand palette must cover all 12 OOXML slots) |
| Architecture Module | slideforge-brand crate — BrandExtractor; slideforge-cli crate — extract-brand subcommand (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-2.01.001 — composes with (same OOXML parsing logic; extraction produces what loading consumes)
- BC-2.01.002 — related to (extraction enables round-trip: load .pptx → extract brand.toml → synthesize)

## Architecture Anchors

- `architecture/brand-architecture.md` — brand extraction algorithm (Spike S5 Part 4)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
