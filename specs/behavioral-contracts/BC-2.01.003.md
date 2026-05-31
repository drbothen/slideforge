---
document_type: behavioral-contract
level: L3
version: "1.9"
status: draft
producer: product-owner
timestamp: 2026-05-31T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-04
capability: CAP-018
lifecycle_status: active
introduced: v1.0.0
modified:
  - version: "1.9"
    date: 2026-05-31
    reason: "STORY-076 BC widening: widen EC-003 from 'schemeClr with lumMod/tint/shade' to 'schemeClr OR srgbClr with lumMod/lumOff/tint/shade transforms' — same output behavior for both element types: write base value as-is with inline TOML comment '# derived via tint/shade; may not match exact color'. The unified behavior is driven by the is_derived flag on ColorSlot (set by BC-2.01.001 EC-006 for srgbClr, already set for schemeClr by STORY-022). Add Canonical Test Vector for srgbClr transform case."
  - version: "1.8"
    date: 2026-05-30
    reason: "F-024-pass8-OBS-1 spec-parity sync: add <path>: prefix to EC-001 error message example and matching Canonical Test Vectors row so both match the authoritative E-BRD-006 form in error-taxonomy.md and code error.rs:233."
  - version: "1.7"
    date: 2026-05-30
    reason: "OBS-1 remainder: fill Stories row and Story Anchor with STORY-024 (library-level extraction; CLI wiring is STORY-057). OBS-2: clarify EC-005 to state logo format detection is by file extension (png/jpg/jpeg/gif/svg), not content/magic-byte sniffing — consistent with AC-004."
  - version: "1.6"
    date: 2026-05-30
    reason: "Correct VP-051/VP-052 anchoring test file path: loader.rs → extractor.rs (all three test functions live in crates/slideforge-brand/src/extractor.rs, verified by grep). No other content changed."
  - version: "1.5"
    date: 2026-05-30
    reason: "F-024-pass5-OBS-1 (VP portion): resolve VP-TBD placeholders — allocate VP-051 (brand round-trip extraction, integration test) and VP-052 (source file read-only invariant, integration test); populate VP Anchors section with anchoring test names."
  - version: "1.4"
    date: 2026-05-30
    reason: "F-024-pass4-OBS-3: clarify Description prose — library writes brand.toml to caller-supplied output_dir parameter, not hardcoded CWD; CWD default is a CLI-layer concern deferred to STORY-057. Prose-only fix; no AC or postcondition semantics changed."
  - version: "1.3"
    date: 2026-05-30
    reason: "F-024R-OBS-2: resolve SS-TBD placeholder — set subsystem to SS-04 (Brand, slideforge-brand) per ARCH-INDEX Subsystem Registry; STORY-024 frontmatter anchors this BC to subsystems [SS-04, SS-18]; BC implements BrandExtractor in slideforge-brand making SS-04 the primary owner."
  - version: "1.2"
    date: 2026-05-30
    reason: "OBS-3: align Description and Canonical Test Vectors to OOXML slot names (dk1/lt1/dk2/lt2/acc1-acc6/hlink/fol_hlink) per STORY-024 AC-003 and BrandConfig schema; OBS-1: add footer-detection implementation note referencing STORY-075."
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
and writes `brand.toml` (and `brand.assets/`) to a caller-supplied output directory
(the `output_dir: &Path` parameter of `BrandExtractor::extract`); the CLI subcommand
wired in STORY-057 will default that directory to the current working directory.
The extracted TOML maps all 12 OOXML theme color slots to their OOXML slot names in
the `[colors]` section: `dk1` (dark 1 / primary text), `lt1` (light 1 / background),
`dk2` (dark 2 / secondary text), `lt2` (light 2 / secondary background),
`acc1`–`acc6` (accent colors 1–6), `hlink` (hyperlink), and `fol_hlink` (followed
hyperlink — TOML-safe form of OOXML `folHlink`). It also extracts heading/body font
names into `[fonts]` and copies the logo image to `brand.assets/logo.<ext>` if one
is found in the slide master's media relationships.

## Preconditions

1. The source path resolves to a readable `.pptx` file.
2. The file is a valid OOXML ZIP package.
3. No `brand.toml` already exists at the output path, or `--force` flag is provided.

## Postconditions

1. A `brand.toml` is written containing `[colors]`, `[fonts]`, and (if detected) `[logo]`
   and `[footer]` sections as specified in Spike S5 Part 4.2.
   > **Implementation note — footer section:** The `[footer]` writer in `BrandExtractor`
   > is complete and emits the section when `BrandTemplate.footer_text` is `Some(...)`.
   > However, footer *detection* during brand loading (reading `<p:ph type="ftr"/>` from
   > the slide master XML and extracting footer visibility flags from `presProps.xml`) is
   > tracked as STORY-075 ("Brand Loader: Footer Detection from .pptx Slide Master/Layout
   > Placeholders", anchored to BC-2.01.001). Until STORY-075 is delivered,
   > `BrandLoader::load()` hardcodes `footer_text: None`, making the `[footer]` writer
   > permanently unreachable on real input. The "if detected" hedge above refers to this
   > runtime condition — the writer activates automatically once STORY-075 populates
   > `BrandTemplate.footer_text`.
2. All 12 OOXML color slots are represented; `sysClr` elements use their `lastClr`
   attribute as the hex value.
3. If a logo image is found in slideMaster1.xml.rels, it is copied to
   `brand.assets/logo.<ext>` and `[logo] path = "brand.assets/logo.<ext>"` is written.
4. Build exits with code 0.

## Invariants

1. Color extraction follows ECMA-376 sequential order (dk1, lt1, dk2, lt2, acc1–acc6,
   hlink, folHlink in XML; `fol_hlink` in TOML) — the extractor does NOT use random-access
   lookup. (DI-015)
2. The extraction is read-only; the source .pptx is never modified.
3. OOXML slot names in `[colors]` are fixed (dk1, lt1, dk2, lt2, acc1–acc6, hlink, fol_hlink)
   and written in ECMA-376 sequential order — they are not configurable and the output order
   is stable (IndexMap-backed, not HashMap iteration order).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | brand.toml already exists and --force not passed | Error: "<path>: brand.toml already exists. Use --force to overwrite." Exit 4. |
| EC-002 | Source .pptx has sysClr elements instead of srgbClr | Use lastClr attribute value; no error |
| EC-003 | Source .pptx has `lumMod`, `lumOff`, `tint`, or `shade` transform child elements on either `schemeClr` OR `srgbClr` color elements | For both element types: write the base value as-is to `brand.toml` with inline TOML comment `# derived via tint/shade; may not match exact color`. The `is_derived` flag on `ColorSlot` (set to `true` by the loader — see BC-2.01.001 EC-006 for srgbClr, and the schemeClr path in STORY-022) drives this unified extractor behavior. The extractor branch conditions on `ColorSlot.is_derived` regardless of the originating element type. No E-BRD-NNN error is emitted for this case — it is an observability comment only. |
| EC-004 | Source .pptx has multiple slide masters | Extract from slideMaster1.xml only; lint warning noting multi-master template |
| EC-005 | Logo file extension in ppt/media/ is not one of the renderable set (png, jpg/jpeg, gif, svg) — detection is by file extension only, not by content/magic-byte sniffing | Copy the file anyway; log a warning about potential rendering differences |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Well-formed corporate .pptx (12 colors, heading+body fonts, logo) | brand.toml with `[colors]` containing all 12 OOXML slots (dk1–fol_hlink), `[fonts]`, `[logo]`; brand.assets/logo.png written; exit 0 | happy-path |
| .pptx with sysClr for dk1 (Windows system color) | brand.toml `[colors]` section contains `dk1 = "#<lastClr value>"`; exit 0 | edge-case |
| brand.toml already exists, no --force | Error: "<path>: brand.toml already exists. Use --force to overwrite."; exit 4 | error |
| .pptx with no logo in slide master | brand.toml written with `[colors]` and `[fonts]` but no `[logo]` section; exit 0 | edge-case |
| .pptx with srgbClr slot (dk2 = `#003087`) having `<a:lumMod val="75000"/>` child | brand.toml `[colors]` section contains `dk2 = "#003087"` with trailing inline comment `# derived via tint/shade; may not match exact color`; exit 0 | edge-case (EC-003 widened) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-051 | Extracted brand.toml round-trips: load the .pptx, extract to brand.toml, synthesize from brand.toml → verify color values match | integration test — anchored by `test_bc_2_01_003_round_trip_pptx_to_brand_toml_and_back` and `test_bc_2_01_003_effectful_extract_load_from_toml_round_trip` in `crates/slideforge-brand/src/extractor.rs` |
| VP-052 | Source .pptx file is unmodified after extraction | integration test (file hash before/after) — anchored by `test_bc_2_01_003_invariant_source_file_unmodified` in `crates/slideforge-brand/src/extractor.rs` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 |
| Capability Anchor Justification | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 — bidirectional extraction ("slideforge extract-brand deck.pptx → brand.toml") is explicitly described in CAP-018 |
| L2 Domain Invariants | DI-015 (brand palette must cover all 12 OOXML slots) |
| Architecture Module | slideforge-brand crate — BrandExtractor; slideforge-cli crate — extract-brand subcommand (filled by architect) |
| Stories | STORY-024 (library-level extraction — `BrandExtractor::extract`); CLI wiring via STORY-057 |

## Related BCs

- BC-2.01.001 — composes with (same OOXML parsing logic; extraction produces what loading consumes)
- BC-2.01.002 — related to (extraction enables round-trip: load .pptx → extract brand.toml → synthesize)

## Architecture Anchors

- `architecture/brand-architecture.md` — brand extraction algorithm (Spike S5 Part 4)

## Story Anchor

STORY-024 (`brand-extract-cli`) — implements the library-level `BrandExtractor::extract`
function in `crates/slideforge-brand`. The CLI subcommand wiring that maps
`slideforge extract-brand <template.pptx>` to this library call is STORY-057.

## VP Anchors

| VP ID | Description | Proof Method | Anchoring Tests |
|-------|-------------|-------------|----------------|
| VP-051 | Brand round-trip extraction: .pptx → extract → brand.toml → synthesize → color values match | integration test | `test_bc_2_01_003_round_trip_pptx_to_brand_toml_and_back`, `test_bc_2_01_003_effectful_extract_load_from_toml_round_trip` (`crates/slideforge-brand/src/extractor.rs`) |
| VP-052 | Source .pptx read-only: byte-identical before and after `BrandExtractor::extract` | integration test (file hash before/after) | `test_bc_2_01_003_invariant_source_file_unmodified` (`crates/slideforge-brand/src/extractor.rs`) |

**Rationale — tool choice:** Both properties exercise the effectful I/O boundary (`BrandExtractor::extract` reads a ZIP archive and writes TOML to disk). Kani cannot be applied here — Kani proves properties of pure, side-effect-free functions in bounded model checking; effectful I/O (file reads/writes, ZIP decompression) falls outside its scope. The correct vehicle is named integration tests that run in CI with real fixture `.pptx` files. VP-051 is distinct from VP-012 (which covers the synthesis direction: brand.toml → synthesize → extract → round-trip via proptest on pure-core functions in `slideforge-brand`); VP-051 covers the extraction direction starting from an actual `.pptx` fixture.
