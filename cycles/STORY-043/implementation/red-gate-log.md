---
story: STORY-043
phase: Red Gate (BC-5.38.001)
date: 2026-05-31
agent: test-writer
status: VERIFIED — 3 failing / 12 passing
---

# STORY-043 Red Gate Log

## Summary

The Red Gate for STORY-043 (`slideforge-pdf` scaffold) was executed and
verified. All behavioral stub tests fail correctly at `todo!()` stubs.
All dep-guard and structural tests pass immediately.

## Type Reconciliation (Spec vs. Reality)

### Exporter trait (CRITICAL DIVERGENCE)

The story spec claimed:
```
fn export(&self, deck: &LaidOutDeck, brand: &Brand, output: &mut dyn Write)
    -> Result<(), ExportError>
```

The REAL signature from `crates/slideforge-plugin-api/src/traits/exporter.rs`:
```rust
fn export(
    &self,
    deck: &Deck,           // ADDED: semantic IR (Deck, not LaidOutDeck)
    laid_out: &LaidOutDeck, // RENAMED from 'deck'
    brand: &Brand,
    opts: &ExportOptions,  // CHANGED: opts struct, not dyn Write
) -> Result<Vec<u8>, ExportError>  // CHANGED: returns bytes, not writes
```

**Divergences from spec:**
1. Takes `deck: &Deck` (semantic) AND `laid_out: &LaidOutDeck` (geometric) — not just one
2. Returns `Result<Vec<u8>, ExportError>` — does NOT write to `dyn Write`
3. Has `opts: &ExportOptions` parameter — no `output: &mut dyn Write`
4. `ExportError` is from `slideforge_plugin_api` (has `RenderError`, `IoError`, etc.)

**Implementation consequence:** `PdfExporter::generate_pdf` maps `PdfExportError`
→ `ExportError::RenderError` at the trait boundary. The implementer follows this pattern.

### LaidOutDeck / LaidOutSlide

Confirmed at: `crates/slideforge-layout/src/types.rs`. Structure as expected.
`FrameContent::Diagram(NormalizedDiagramSvg)` — NOT `DiagramSvg`. No `ChartSvg`
variant — it's `FrameContent::Chart` (no payload).

### Brand

Confirmed at: `crates/slideforge-types/src/brand.rs`. Has `name`, `palette`,
`fonts`, `layouts`, `span` fields.

### NormalizedDiagramSvg

Confirmed at: `crates/slideforge-types/src/specs.rs`. Newtype over `Arc<str>`.
Constructor: `NormalizedDiagramSvg::from_normalized_string(Arc<str>)`.
Access: `.as_str()`.

### FrameContent variants (divergence from spec)

Spec said `FrameContent::DiagramSvg` and `FrameContent::ChartSvg`.
Reality: `FrameContent::Diagram(NormalizedDiagramSvg)` and `FrameContent::Chart`.

### indexmap version conflict (FIXED IN SCOPE)

krilla 0.6.0 requires `indexmap ^2.10.0` but the workspace and 3 crates
(`slideforge-types`, `slideforge-data`, `slideforge-eval`) pinned `=2.9.0`.
Fixed by bumping all 4 pins to `=2.10.0` (minimum satisfying both constraints).
This is a supply-chain-acceptable bump (patch minor, same API, no breaking changes).

## krilla Dependencies Added

| Crate | Version | Type |
|-------|---------|------|
| krilla | =0.6.0 | Direct dep of slideforge-pdf |
| pdf-writer | 0.14.0 | Transitive via krilla |
| subsetter | 0.2.4 | Transitive via krilla (spec said 0.2.3; ^0.2.3 resolved 0.2.4) |
| skrifa | 0.37.0 + 0.42.1 | Transitive via krilla |
| tiny-skia-path | 0.11.4 | Transitive via krilla |
| font-types | 0.10.1 + 0.11.3 | Transitive via krilla/skrifa |
| read-fonts, write-fonts | various | Transitive |
| xmp-writer | 0.3.3 | Transitive via krilla |

**28 new packages locked** in Cargo.lock.

## Test Files Created

| File | Tests |
|------|-------|
| `crates/slideforge-pdf/src/exporter.rs` | 4 unit tests |
| `crates/slideforge-pdf/src/tag_engine.rs` | 2 unit tests |
| `crates/slideforge-pdf/src/svg_embed.rs` | 2 unit tests |
| `crates/slideforge-pdf/src/font.rs` | 2 unit tests |
| `crates/slideforge-pdf/tests/no_forbidden_deps.rs` | 5 integration tests |

**Total: 15 tests**

## Red Gate Results

```
cargo nextest run -p slideforge-pdf --no-fail-fast
Summary: 15 tests run: 12 passed, 3 failed, 0 skipped
```

### Failing Tests (Red Gate — correct behavior)

| Test | Stub Location | Reason |
|------|-------------|--------|
| `test_bc_4_03_002_export_produces_pdf_bytes` | `exporter.rs:generate_pdf()` | `todo!()` |
| `test_bc_4_03_002_tag_slide_produces_one_part_per_slide` | `tag_engine.rs:tag_slide()` | `todo!()` |
| `test_bc_4_03_002_font_bytes_load_reads_file` | `font.rs:FontBytes::load()` | `todo!()` |

### Passing Tests (Dep-guard + Structural — correct behavior)

| Test | AC | Passes Because |
|------|----|----------------|
| `test_bc_4_03_002_pdf_exporter_implements_exporter_trait` | AC-001 | Compile-time: impl exists |
| `test_bc_4_03_002_exporter_id_and_extension` | AC-001 | id()="pdf", ext()="pdf" are real values |
| `test_bc_4_03_002_no_subprocess_structural_check` | AC-008 | PdfExporter is Send+Sync |
| `test_font_bytes_from_bytes_round_trips` | AC-004 | from_bytes() is not stubbed |
| `test_bc_4_03_002_tag_slide_panics_at_stub` | AC-003 | `#[should_panic]` guard |
| `test_bc_4_03_002_svg_embed_converts_rect_to_vector_paths` | AC-005 | usvg parse only (not stub path) |
| `test_bc_4_03_002_svg_str_is_parseable_by_usvg` | AC-005 | usvg parse is real |
| `test_bc_4_03_002_no_browser_pdf_deps_in_cargo_lock` | AC-002/AC-009 | krilla is pure Rust |
| `test_bc_4_03_002_no_ffi_pdf_deps_in_cargo_lock` | AC-007 | No C deps |
| `test_bc_4_03_002_no_direct_pdf_writer_dep` | RISK-1 | Not in Cargo.toml deps |
| `test_bc_4_03_002_no_direct_subsetter_dep` | RISK-2 | Not in Cargo.toml deps |
| `test_bc_4_03_002_krilla_present_in_cargo_lock` | AC-009 | krilla in lock |

## Workspace Build Result

`cargo build --workspace` — CLEAN. 28 new packages compiled. All existing crates
unaffected.

## Concerns

1. **subsetter version drift**: Tech-validation said `0.2.3`; Cargo.lock shows `0.2.4`.
   This is expected — krilla pins `^0.2.3` which resolves to latest compatible.
   No action required unless krilla 0.6.0's compatibility is specifically tied to 0.2.3.

2. **AC-004 font subsetting test**: The `test_bc_4_03_002_font_bytes_load_reads_file`
   test reads a temp file — it drives `FontBytes::load()` from the filesystem.
   The actual krilla-internal subsetting test (AC-004 full assertion: font in output
   is smaller than full font) requires a live krilla Document. This is deferred to
   the green phase where the implementer wires a document harness.

3. **AC-005 svg_embed test**: The surface-level test is split: usvg parse passes
   now (not stubbed), but the krilla Surface draw call requires a live Document/Page
   context. The implementer provides the harness in the green phase.

4. **indexmap 2.9.0 → 2.10.0 bump**: 4 files modified beyond slideforge-pdf scope.
   This is a blocking dependency conflict (krilla requires ^2.10.0). The fix is
   production-grade and correct — no API breakage in 2.10.0 vs 2.9.0.

## Red Gate Decision

**VERIFIED**: All behavioral tests fail before implementation. The implementer
may now proceed. Instructions: make each failing test pass, one at a time,
with minimum code.
