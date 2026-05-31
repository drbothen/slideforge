---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-024
title: "Brand Extraction CLI (slideforge extract-brand)"
epic: EPIC-06
wave: 3
points: 3
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-brand
subsystems: [SS-04, SS-18]
target_module: slideforge-brand
behavioral_contracts: [BC-2.01.003]
verification_properties: [VP-051, VP-052]
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-022
  - STORY-023
blocks:
  - STORY-057
estimated_days: 1
---

# STORY-024: Brand Extraction CLI (slideforge extract-brand)

## Summary

Implement the `BrandExtractor` in `slideforge-brand` that reads an existing `.pptx` file
and produces a `brand.toml` file in the output directory. The extractor uses the
`BrandLoader` from STORY-022 to read the OOXML package, then serializes all 12 color
slots, heading/body fonts, detected logo (copying image bytes to `brand.assets/`), and
footer text into `brand.toml` using the schema defined in STORY-023. This story
implements the library-level extraction logic only. The CLI subcommand
(`slideforge extract-brand <template.pptx>`) wiring is in STORY-057.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~2,500 |
| `crates/slideforge-brand/src/extractor.rs` | ~2,500 |
| Test code | ~2,000 |
| BC file consulted (BC-2.01.003) | ~1,000 |
| **Total** | **~8,000** |

Agent context budget: 200k tokens. This story is ~4% of budget — well within limit.

## Acceptance Criteria

- [ ] **AC-001:** `BrandExtractor::extract(source_pptx: &str, output_dir: &Path, force: bool) -> Result<BrandExtractionResult, BrandError>` loads the `.pptx` via `BrandLoader::load()` (STORY-022), serializes the `BrandTemplate` fields to `brand.toml` format, and writes the file to `output_dir/brand.toml`. On success, returns `BrandExtractionResult { brand_toml_path, logo_asset_path: Option<PathBuf> }`.
  (traces to BC-2.01.003 postcondition 1 — `brand.toml` written with `[colors]`, `[fonts]`, `[logo]`, `[footer]`)

- [ ] **AC-002:** If `output_dir/brand.toml` already exists and `force = false`, returns `BrandError::OutputExists { path: Arc<str> }` with message: `"<path>: brand.toml already exists. Use --force to overwrite."` (exit code 4). If `force = true`, the existing file is overwritten.
  (traces to BC-2.01.003 edge case EC-001 — existing brand.toml without --force)

- [ ] **AC-003:** All 12 OOXML color slots are written in the `[colors]` section of `brand.toml` using their semantic field names. The slot-to-field mapping is:
  - `dk1` → `dk1 = "#RRGGBB"`
  - `lt1` → `lt1 = "#RRGGBB"`
  - `dk2` → `dk2 = "#RRGGBB"`
  - `lt2` → `lt2 = "#RRGGBB"`
  - `acc1`–`acc6` → `acc1`–`acc6 = "#RRGGBB"`
  - `hlink` → `hlink = "#RRGGBB"`
  - `folHlink` → `fol_hlink = "#RRGGBB"` (TOML-safe name)
  All 12 slots are extracted in ECMA-376 sequential order per BC-2.01.003 invariant 1.
  (traces to BC-2.01.003 postcondition 2 — all 12 OOXML slots represented)

- [ ] **AC-004:** If a logo image is found in `slideMaster1.xml.rels`, its bytes are written to `output_dir/brand.assets/logo.<ext>` where `<ext>` is the original file extension from the ZIP (`png`, `jpg`, `svg`, etc.). The `brand.toml` `[logo]` section is written with `path = "brand.assets/logo.<ext>"`. If no logo is found, the `[logo]` section is omitted from `brand.toml`.
  (traces to BC-2.01.003 postcondition 3 — logo copied to `brand.assets/logo.<ext>`)

- [ ] **AC-005:** `sysClr` elements (Windows system color references) in `theme1.xml` use their `lastClr` attribute value as the hex color in `brand.toml`. This is the same logic as `BrandLoader` (from STORY-022).
  (traces to BC-2.01.003 edge case EC-002 — `sysClr` → `lastClr`)

- [ ] **AC-006:** Color slots with `lumMod`/`tint`/`shade` transforms (`schemeClr` with modifiers) are written to `brand.toml` as their base hex value — specifically, the referenced slot's resolved hex for the resolvable case; for references that cannot be resolved (self-reference or color-map mnemonics like `tx1`/`bg1`/`phClr`), a best-effort default hex is written instead — with an inline TOML comment in both cases: `# derived via tint/shade; may not match exact color`. This is the best-effort extraction per BC-2.01.003 edge case EC-003.
  (traces to BC-2.01.003 edge case EC-003 — tint/shade transforms noted in comment)

- [ ] **AC-007:** If the source `.pptx` has multiple slide masters, only `slideMaster1.xml` is processed. A lint warning is emitted: `tracing::warn!("Source .pptx has multiple slide masters; extracting from slideMaster1.xml only.")`.
  (traces to BC-2.01.003 edge case EC-004)

- [ ] **AC-008:** The source `.pptx` file is NEVER modified. Read-only file access only. An integration test verifies the SHA-256 hash of the source file before and after extraction is identical.
  (traces to BC-2.01.003 invariant 2 — extraction is read-only; exercises VP-052)

- [ ] **AC-009:** The `brand.toml` TOML output is human-readable with section headers (`[colors]`, `[fonts]`, `[logo]`, `[footer]`), one field per line, and fields in a stable order (not `HashMap` iteration order — use `IndexMap` or explicit ordering).
  (traces to BC-2.01.003 invariant 3 — semantic field names are fixed; implies stable ordering)

- [ ] **AC-010:** The extraction enables round-trip: load `.pptx` → extract `brand.toml` → synthesize from `brand.toml` → produced `BrandTemplate` has same color hex values as original `.pptx` template. Integration test verifies this round-trip.
  (traces to BC-2.01.003 verification property — round-trip integration test; exercises VP-051)

- [ ] **AC-011:** `#![forbid(unsafe_code)]` (NFR-024), clippy clean (NFR-022), `=` version pinning (NFR-025) maintained.

## Previous Story Intelligence

Continues from STORY-022 and STORY-023. Key context:
- `BrandLoader::load()` from STORY-022 already parses the `.pptx` and produces a `BrandTemplate` with 12 `ColorSlot` values. The extractor's job is to serialize that `BrandTemplate` back to `brand.toml` format.
- `BrandConfig` (the `brand.toml` schema struct) from STORY-023 defines the serde layout. Use it with `toml::to_string()` for serialization.
- `LogoAsset` from STORY-022 carries `bytes`, `media_type`, and `original_path`. Use `original_path` to determine the output file extension.

## Architecture Compliance Rules

1. **Extraction = Load + Serialize:** `BrandExtractor` is a thin wrapper. It calls `BrandLoader::load()` → gets `BrandTemplate` → converts to `BrandConfig` → serializes to TOML. The heavy lifting is in STORY-022's loader.
2. **Read-only access to source:** The source `.pptx` is opened via `BrandLoader::load()` which is already read-only.
3. **`brand.assets/` directory creation:** Create the directory if it does not exist. Do not error if it already exists. Use `std::fs::create_dir_all`.
4. **Forbidden dependencies:** Same as STORY-022.

## Library and Framework Requirements

No new libraries required — `toml = "=0.8"` (for serialization) and `std::fs` are sufficient. All other deps from STORY-022 are reused.

For TOML serialization, use `toml::to_string()` on a `BrandConfig` struct. Since `BrandConfig` already has `serde::Serialize` derive from STORY-023, this is straightforward.

## File Structure Requirements

Files to create:

```
crates/slideforge-brand/src/
└── extractor.rs          # BrandExtractor struct + extract() function
```

Files to modify:

```
crates/slideforge-brand/src/
├── error.rs              # Add: OutputExists variant to BrandError
└── lib.rs                # Add: pub mod extractor;
```

## Tasks

1. **Add `BrandError::OutputExists`** to `error.rs`. (5 min)
2. **Write `src/extractor.rs`:**
   - `BrandExtractionResult { brand_toml_path: PathBuf, logo_asset_path: Option<PathBuf> }`.
   - `BrandExtractor::extract(source: &str, out_dir: &Path, force: bool) -> Result<BrandExtractionResult, BrandError>`:
     - Call `BrandLoader::load(source, ctx)`.
     - Convert `BrandTemplate` → `BrandConfig` (map `ColorSlot` values back to field names).
     - Handle `force` / existing file check.
     - Serialize `BrandConfig` → TOML string via `toml::to_string()`.
     - Write TOML string to `out_dir/brand.toml`.
     - If `template.logo.is_some()`: create `out_dir/brand.assets/`; write logo bytes; set `[logo] path`.
     - Return `BrandExtractionResult`.
   (40 min)
3. **Write integration tests** — see Test Strategy. (25 min)
4. **Run `cargo clippy -p slideforge-brand -- -D warnings`** — fix. (10 min)
5. **Run `cargo test -p slideforge-brand`** — all tests pass. (10 min)

## Test Strategy

### Integration tests in `extractor.rs`

| Test Name | Setup | Expected |
|-----------|-------|----------|
| `test_extract_writes_brand_toml` | Minimal PPTX fixture (from STORY-022 test helpers) | `brand.toml` created at output path; contains `[colors]` with all 12 slots |
| `test_extract_existing_without_force` | Pre-create `brand.toml` in output dir; `force=false` | `BrandError::OutputExists`; existing file unmodified |
| `test_extract_existing_with_force` | Pre-create `brand.toml`; `force=true` | File overwritten; `BrandExtractionResult` returned |
| `test_extract_copies_logo` | PPTX fixture with embedded logo PNG | `brand.assets/logo.png` created; `brand.toml` has `[logo] path = "brand.assets/logo.png"` |
| `test_extract_no_logo_omits_section` | PPTX fixture with no logo | `brand.toml` has no `[logo]` section |
| `test_extract_source_file_unchanged` | Compute SHA-256 of source `.pptx` before and after | SHA-256 hashes are identical |
| `test_round_trip` | Extract from `.pptx` → `brand.toml`; load back via `BrandSynthesizer::load_from_toml()` | `BrandTemplate.colors` hex values match original `.pptx` template's colors |

## Dependencies

**Depends on:**
- STORY-022 (Brand Loading) — `BrandLoader::load()`, `BrandTemplate`, `ColorSlot`, `LogoAsset`.
- STORY-023 (Brand Synthesis) — `BrandConfig` struct (for TOML serialization schema).

Dependency justification: STORY-024 depends on STORY-022 because extraction uses `BrandLoader` for reading. It depends on STORY-023 because `BrandConfig` (the `brand.toml` serde schema) is defined there and used as the serialization target.

**Blocks:**
- STORY-057 (CLI: init scaffolding + extract-brand command) — the CLI wires `slideforge extract-brand <path>` to `BrandExtractor::extract()`.

## Implementation Notes

### BrandTemplate → BrandConfig Conversion

The key mapping is from `[ColorSlot; 12]` (loaded in ECMA-376 order) to `BrandConfig.colors`:

```rust
fn brand_template_to_config(template: &BrandTemplate) -> BrandConfig {
    let colors = &template.colors;
    BrandConfig {
        colors: Some(ColorsConfig {
            dk1: Some(colors[0].hex.to_string()),
            lt1: Some(colors[1].hex.to_string()),
            dk2: Some(colors[2].hex.to_string()),
            lt2: Some(colors[3].hex.to_string()),
            acc1: Some(colors[4].hex.to_string()),
            acc2: Some(colors[5].hex.to_string()),
            acc3: Some(colors[6].hex.to_string()),
            acc4: Some(colors[7].hex.to_string()),
            acc5: Some(colors[8].hex.to_string()),
            acc6: Some(colors[9].hex.to_string()),
            hlink: Some(colors[10].hex.to_string()),
            fol_hlink: Some(colors[11].hex.to_string()),
        }),
        fonts: Some(FontsConfig {
            heading: Some(template.fonts.heading.to_string()),
            body: Some(template.fonts.body.to_string()),
        }),
        logo: template.logo.as_ref().map(|_| LogoConfig {
            path: "brand.assets/logo.<ext>".to_string(), // actual ext determined during write
        }),
        footer: template.footer_text.as_ref().map(|f| FooterConfig {
            text: Some(f.to_string()),
            ..Default::default()
        }),
    }
}
```

### TOML Output Order

`toml::to_string()` does not guarantee field order when using `HashMap`. Use `IndexMap`
in the serde structs (STORY-023's `BrandConfig`) to preserve insertion order. Alternatively,
build the TOML string manually section by section for predictable output order.

Preferred approach: use `toml = "=0.8"` with `serde_serialize` on `BrandConfig` where
all fields are `IndexMap`-backed — the `toml` crate respects `IndexMap` insertion order.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `brand.toml` already exists; `force=false` | `BrandError::OutputExists`; existing file untouched |
| EC-002 | `sysClr` in source `.pptx` | `lastClr` attribute used as hex in `brand.toml` |
| EC-003 | Color slot with `lumMod`/`tint` | Written with TOML inline comment noting derivation |
| EC-004 | Multiple slide masters in source | Only `slideMaster1.xml` extracted; lint warning emitted |
| EC-005 | Logo has unsupported format (e.g., `.emf`) | Logo bytes copied anyway; lint warning about potential rendering differences |
| EC-006 | Output directory does not exist | `std::fs::create_dir_all` creates it; no error |
| EC-007 | Source `.pptx` not found | `BrandError::FileNotFound` from `BrandLoader` |
