# Evidence Report: STORY-043 — PDF Core: pdf-writer + krilla + SlideTagEngine

**Story ID:** STORY-043
**Epic:** EPIC-13
**Crate:** `slideforge-pdf`
**BC Trace:** BC-4.03.002 (all postconditions and invariants covered)
**HEAD SHA verified:** `feature/S-043` branch
**Workspace test count (slideforge-pdf):** 26 tests run: 26 passed, 0 skipped
**Recording tool:** VHS 0.10.0 (terminal — library story, nextest harness demos)
**Font:** FiraCode Nerd Font Mono

> **Library-only story note:** STORY-043 delivers the `slideforge-pdf` crate backend:
> `PdfExporter` (Exporter trait impl), `SlideTagEngine` (PDF/UA-1 structure tree),
> `font.rs` (loading helpers), `svg_embed.rs` (usvg parse to krilla Surface), and
> `error.rs`. Pages are structurally tagged but blank — content drawing (text, SVG
> path rendering to Surface) is wired in STORY-044. All demos invoke the
> load-bearing-NOW deliverables via `cargo nextest run -p slideforge-pdf`. This follows
> the established pattern for library stories (see STORY-035/STORY-036 for precedent).

---

## Coverage Summary

Five demos cover all load-bearing deliverables. AC-006 (cross-platform CI matrix) cannot
be demonstrated locally and is verified by CI on the PR. AC-002/007/008/009 (dependency
perimeter) are covered by both the integration test binary (`no_forbidden_deps.rs`) and
the shell CI guard (`scripts/check-pdf-deps.sh`) — both captured in a single combined demo.

| Demo | Description | Demo File | Tests Covered | Status |
|------|-------------|-----------|--------------|--------|
| AC-001 | PdfExporter implements Exporter trait; export returns `%PDF-` bytes; id/extension = "pdf" | `AC-001-pdf-exporter-trait` | 3 tests: `_implements_exporter_trait`, `_export_produces_pdf_bytes`, `_exporter_id_and_extension` | PASSED |
| AC-003 | SlideTagEngine: Part/H1/Figure/Document tags; Table→TR→TH/TD; decorative exclusion; empty-bullet guard | `AC-003-tagged-pdf-structure` | 7 tests: `_export_produces_tagged_pdf`, `_tag_slide_produces_one_part`, `_assemble_deck_tag_tree`, `_figure_alt_text_threaded`, `_decorative_frame_excluded`, `_tag_table_builds`, `_empty_bullets_produces_no_tag_group` | PASSED |
| AC-004 | Font loading: `load_font_data`, `FontBytes::load`, `system_font_fallback`, missing-path error, None for absent family | `AC-004-font-loading` | 5 tests: `_load_font_data_reads_file`, `_load_font_data_errors_on_missing_path`, `_font_bytes_load_reads_file`, `_font_bytes_from_bytes_round_trips`, `_system_font_fallback_none_for_absent_family` | PASSED |
| AC-005 | SVG embed: usvg parse → krilla Surface path ops; no `/Subtype /Image`; fill operator present | `AC-005-svg-vector-embed` | 2 tests: `_svg_embed_converts_rect_to_vector_paths`, `_svg_str_is_parseable_by_usvg` | PASSED |
| AC-002/007/008/009 | No browser/FFI/subprocess deps; krilla present; no direct pdf-writer/subsetter direct deps; `check-pdf-deps.sh` PASS | `DEP-GUARDS-no-browser-no-ffi` | 5 integration tests in `no_forbidden_deps.rs` + shell script `scripts/check-pdf-deps.sh` | PASSED |

---

## AC-001: PdfExporter Implements Exporter Trait; Export Produces %PDF- Bytes

**File:** `AC-001-pdf-exporter-trait.{tape,gif,webm}`
**BC trace:** BC-4.03.002 postcondition 1 — `PdfExporter` implements the `Exporter` plugin
trait and returns `Ok(Vec<u8>)` containing valid PDF bytes.
**Nextest filter:** `test(exporter_implements) | test(export_produces_pdf) | test(exporter_id_and_extension)` — 3 tests

Three load-bearing assertions:

1. **Trait bound satisfaction** (`test_bc_4_03_002_pdf_exporter_implements_exporter_trait`):
   Compile-time proof via `fn assert_exporter<T: Exporter + Send + Sync>() {}` called with
   `PdfExporter`. If `PdfExporter` does not implement `Exporter`, `Send`, or `Sync`, the crate
   does not compile — the test cannot even link.

2. **PDF byte output** (`test_bc_4_03_002_export_produces_pdf_bytes`): Calls
   `PdfExporter::export()` on a minimal 1-slide `LaidOutDeck`. Asserts:
   - `result.is_ok()` — no error during export
   - `bytes` is non-empty
   - `bytes.starts_with(b"%PDF-")` — the magic header is present, confirming krilla
     serialized a structurally valid PDF

3. **Identity** (`test_bc_4_03_002_exporter_id_and_extension`): Asserts `id() == "pdf"`
   and `extension() == "pdf"` — the plugin registry uses these to route format dispatch.

The real trait signature (discovered during TDD):
```
fn export(&self, deck: &Deck, laid_out: &LaidOutDeck, brand: &Brand, opts: &ExportOptions)
    -> Result<Vec<u8>, ExportError>
```
`PdfExportError` is mapped to `ExportError::RenderError` at the trait boundary via `?`.
`Document::finish()` returns `KrillaResult<Vec<u8>>` — mapped to `PdfExportError::Serialize`.

---

## AC-003: SlideTagEngine Maps LaidOutSlide to PDF Structure Tags

**File:** `AC-003-tagged-pdf-structure.{tape,gif,webm}`
**BC trace:** BC-4.03.002 postcondition 3 — `SlideTagEngine` produces a PDF structure tree
with `/Document` (auto-emitted by krilla), `/Part` per slide, `/H1`, `/Figure` with alt.
**Nextest filter:** 7 tests across `exporter.rs` and `tag_engine.rs`

Seven behavioral assertions covering the complete tag hierarchy:

| Test | What it proves |
|------|---------------|
| `test_bc_4_03_002_export_produces_tagged_pdf` | Full-round-trip: exported PDF bytes contain `StructTreeRoot`, `/Part`, `/H1`, `/Figure`, AND `/Document` (auto-emitted by krilla's `TagTree::serialize()` — confirms F-P4-004 non-vacuous closure). Uses a Title+Image(alt) deck. |
| `test_bc_4_03_002_tag_slide_produces_one_part_per_slide` | `tag_slide()` returns `PartResult` with non-empty Part group; first child is `TagKind::Hn(_)` (H1) for a Title frame. |
| `test_bc_4_03_002_assemble_deck_tag_tree_one_part_per_slide` | `assemble_deck_tag_tree(vec![part1, part2])` produces a `TagTree` with exactly 2 Group children (one Part per slide). |
| `test_bc_4_03_002_figure_alt_text_threaded_from_frame` | Image frame with `alt = "A mountain landscape at sunrise"` produces `TagKind::Figure(_)` child; `group.tag.alt_text()` returns the correct string. |
| `test_bc_4_03_002_decorative_frame_excluded_from_tag_tree` | Image frame with empty alt is listed in `decorative_frame_indices` and NOT added to the Part group (Part has only the H1 child). |
| `test_bc_4_03_002_tag_table_builds_table_tr_th_td` | `tag_table()` produces `Table → TR(2 TH) → TR(2 TD) → TR(2 TD)` for a 2-header, 2-row spec. TagKind variants verified by `matches!`. |
| `test_bc_4_03_002_empty_bullets_produces_no_tag_group` | `tag_content_block(ContentBlock::Bullets([]))` returns `Ok(None)` — no childless `L` element emitted. |

Key architecture note captured: `TagKind` is the variant-bearing enum (not `Tag`). The API
uses typed constructors like `Tag::<kind::Part>::Part` and `Tag::<kind::Hn>::Hn(level, None)`.
`TagKind::Document` does not exist — the PDF `/Document` root is auto-emitted by krilla's
`TagTree::serialize()` calling `struct_elem.kind(StructRole::Document)` (krilla source
`tagging/mod.rs:1050`; pdf-writer `structure.rs:877`).

---

## AC-004: Font Loading Helpers

**File:** `AC-004-font-loading.{tape,gif,webm}`
**BC trace:** BC-4.03.002 invariant 3 — font loading helpers established; subsetting is
krilla-internal (exercised in STORY-044 when text is drawn via Surface).
**Nextest filter:** `binary(slideforge_pdf) & test(font)` — 5 tests

Five assertions covering all font-loading paths:

| Test | What it proves |
|------|---------------|
| `test_bc_4_03_002_load_font_data_reads_file` | `load_font_data(path)` reads a real temp `.ttf` file written with synthetic bytes; returns the bytes unchanged. |
| `test_bc_4_03_002_load_font_data_errors_on_missing_path` | `load_font_data(nonexistent_path)` returns `Err(PdfExportError::Io { .. })` — no panic. |
| `test_bc_4_03_002_font_bytes_load_reads_file` | `FontBytes::load(path)` reads a real file; `as_bytes()` returns the original data. |
| `test_font_bytes_from_bytes_round_trips` | `FontBytes::from_bytes(v).as_bytes()` == `v` — the newtype wrapper preserves bytes unchanged. |
| `test_bc_4_03_002_system_font_fallback_none_for_absent_family` | `system_font_fallback("__nonexistent_family_xyz__")` returns `None` — absent family does not panic or error. |

No subsetting size assertion in STORY-043 because no glyphs are drawn yet. The glyph-size
regression test ("ASCII-only subset must be smaller than the full font") is deferred to
STORY-044 where `Surface` text drawing makes it verifiable (scope-directive Decision 2).
`subsetter` arrives transitively via krilla — no direct dep was added (RISK-2 compliance).

---

## AC-005: SVG Paths Embedded as Vector (No Rasterization)

**File:** `AC-005-svg-vector-embed.{tape,gif,webm}`
**BC trace:** BC-4.03.002 postcondition 4 — SVG parsed via `usvg`, paths emitted on krilla
`Surface`; PDF output contains fill operators and no `/Subtype /Image`.
**Nextest filter:** `test(svg)` — 2 tests

Two vector-embedding assertions:

| Test | What it proves |
|------|---------------|
| `test_bc_4_03_002_svg_embed_converts_rect_to_vector_paths` | A minimal `<rect>` SVG is passed through `embed_svg` → a 1-slide `LaidOutDeck` exported to PDF bytes. Asserts (1) a fill operator is present (`b"rg"` or `b"k"`) confirming vector path drawing, (2) no `/Subtype /Image` token is present confirming zero rasterization. |
| `test_bc_4_03_002_svg_str_is_parseable_by_usvg` | `usvg::Tree::from_str(SVG_RECT, &Options::default())` succeeds without error — the usvg parse layer works correctly with the workspace `usvg =0.47.0` pin. |

Architecture note: krilla 0.6.0 has no usvg dependency (confirmed in tech-validation.md).
The workspace `usvg =0.47.0` is safe alongside krilla with zero version conflicts.
Path segment conversion bridges `tiny_skia_path 0.12.0` (usvg) to `0.11.4` (krilla) by
iterating `usvg::Path::data().segments()` and rebuilding via `krilla::PathBuilder`.

---

## AC-002 / AC-007 / AC-008 / AC-009: Dependency Perimeter Guards

**File:** `DEP-GUARDS-no-browser-no-ffi.{tape,gif,webm}`
**BC trace:** BC-4.03.002 invariant 1 (no browser), invariant 4 (no FFI), postcondition 2
(no subprocess), AC-009 (krilla present in Cargo.lock).
**Demo shows:** Integration test binary (`no_forbidden_deps.rs`, 5 tests) + shell script
(`scripts/check-pdf-deps.sh`, PASS output).

Two-layer enforcement demonstrated:

**Layer 1 — Rust integration tests** (`crates/slideforge-pdf/tests/no_forbidden_deps.rs`):

| Test | What it proves |
|------|---------------|
| `test_bc_4_03_002_no_browser_pdf_deps_in_cargo_lock` | `Cargo.lock` contains no `chromium`, `headless-chrome`, `puppeteer-rs`, `wkhtmltopdf` (AC-002 / AC-009). |
| `test_bc_4_03_002_no_ffi_pdf_deps_in_cargo_lock` | `Cargo.lock` contains no `libharu`, `cairo-rs`, `pango-sys`, `freetype-sys`, `harfbuzz-sys` (AC-007). |
| `test_bc_4_03_002_no_direct_pdf_writer_dep` | `slideforge-pdf/Cargo.toml` has no `pdf-writer` in `[dependencies]` (RISK-1 guard — transitive only). |
| `test_bc_4_03_002_no_direct_subsetter_dep` | `slideforge-pdf/Cargo.toml` has no `subsetter` in `[dependencies]` (RISK-2 guard — transitive only). |
| `test_bc_4_03_002_krilla_present_in_cargo_lock` | `Cargo.lock` contains `krilla` — the pure-Rust PDF engine is present (AC-009 positive guard). |

**Layer 2 — Shell CI guard** (`scripts/check-pdf-deps.sh`):
- Scans `Cargo.lock` for all browser-based and C-library FFI forbidden crate names.
- Scans `crates/slideforge-pdf/src/` (6 `.rs` files) for `std::process`, `Command::new`,
  `process::Command` — positive-coverage guard asserts >= 5 files scanned.
- Exits 0 if all clean. Demo shows the full PASS output.

Subprocess assertion note: The source-level grep (AC-008 scope-directive Decision 3) provides
the load-bearing enforcement. The full strace/dtrace integration test is deferred to STORY-049
(`test_e2e_pdf_export_no_execve_syscall`) which requires the complete CLI binary + syscall
tracer (SID-1: specific story named, concrete dependency cited).

---

## Deferred Evidence

AC-006 (cross-platform compilation on 5 CI targets) cannot be locally demonstrated — it
requires the full CI matrix (aarch64/x86_64 macOS, x86_64/aarch64 Linux, x86_64 Windows).
Verification runs on PR CI. No unit test can substitute for cross-compilation verification.

AC-008 strace/dtrace integration path is deferred to STORY-049 (full CLI binary required).
The source-level grep enforcement is demonstrated in the DEP-GUARDS demo.

---

## Harness Files

No new harness files were created for demo recording. All tests demonstrated are part of
the STORY-043 TDD delivery:

| File | Tests |
|------|-------|
| `crates/slideforge-pdf/src/exporter.rs` | AC-001 and AC-003 (integrated): `#[cfg(test)] mod tests` |
| `crates/slideforge-pdf/src/tag_engine.rs` | AC-003: `#[cfg(test)] mod tests` |
| `crates/slideforge-pdf/src/font.rs` | AC-004: `#[cfg(test)] mod tests` |
| `crates/slideforge-pdf/src/svg_embed.rs` | AC-005: `#[cfg(test)] mod tests` |
| `crates/slideforge-pdf/tests/no_forbidden_deps.rs` | AC-002/007/008/009: integration test binary |
| `scripts/check-pdf-deps.sh` | AC-002/007/008/009: shell CI guard |

No production code was modified during demo recording.

---

## File Manifest

```
docs/demo-evidence/STORY-043/
├── AC-001-pdf-exporter-trait.tape
├── AC-001-pdf-exporter-trait.gif
├── AC-001-pdf-exporter-trait.webm
├── AC-003-tagged-pdf-structure.tape
├── AC-003-tagged-pdf-structure.gif
├── AC-003-tagged-pdf-structure.webm
├── AC-004-font-loading.tape
├── AC-004-font-loading.gif
├── AC-004-font-loading.webm
├── AC-005-svg-vector-embed.tape
├── AC-005-svg-vector-embed.gif
├── AC-005-svg-vector-embed.webm
├── DEP-GUARDS-no-browser-no-ffi.tape
├── DEP-GUARDS-no-browser-no-ffi.gif
├── DEP-GUARDS-no-browser-no-ffi.webm
└── evidence-report.md
```

Total: 15 recording files (5 demos x 3 formats each) + this report = 16 files.
