# Red Gate Log — STORY-049

**Story:** Plugin Registry Assembly (root crate)
**Date:** 2026-06-05
**Status:** RED GATE VERIFIED

## Summary

15 tests written across 3 test modules. 6 fail on `todo!()` stub (Red Gate confirmed).
9 pass because they test pre-existing STORY-083 behavior or the `dispatch_plugin` function
which has a correct implementation as part of the stub infrastructure.

## Exit Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Compile | `cargo build --workspace` | PASS |
| Red Gate | `cargo nextest run -p slideforge --no-fail-fast` | 6 FAIL / 9 PASS |
| Format | `cargo fmt --all -- --check` | PASS |

## Failing Tests (Red — STORY-049 behavior not yet implemented)

All 6 fail with: `not yet implemented: register_bundled_plugins — STORY-049 implementation`

| Test | Module | AC |
|------|--------|----|
| `test_bc_5_02_001_register_bundled_plugins_build_returns_ok_with_10_surfaces` | `registry` | AC-001 |
| `test_bc_5_02_001_register_bundled_plugins_all_10_surface_names_present` | `registry` | AC-001/AC-004 |
| `test_bc_5_02_001_default_registry_returns_ok_with_10_surfaces` | `registry` | AC-001 |
| `test_bc_5_02_001_surface_count_and_surface_names_on_default_registry` | `registry` | AC-004 |
| `test_bc_5_02_001_ec004_multiple_brand_providers_registered` | `registry` | EC-004 |
| `test_bc_5_02_002_external_plugin_compiles_and_registers_with_bundled_plugins` | `external_plugin_test` | AC-007 |

Each test fails with a real assertion (`.expect()` on a `todo!()` panic) — not via `#[should_panic]`.
None of the failures are inverted-Red-Gate per LESSON-17.

## Passing Tests (infrastructure/pre-existing behavior)

| Test | Module | Reason |
|------|--------|--------|
| `test_bc_5_02_001_empty_builder_returns_err_missing_surface_from_root_crate` | `registry` | Tests STORY-083 PluginRegistryBuilder; no stub dependency |
| `test_bc_5_02_001_partial_builder_missing_exporter_reports_error` | `registry` | Tests STORY-083 PluginRegistryBuilder; no stub dependency |
| `test_bc_5_02_001_ec003_panic_str_returns_plugin_panic_error` | `dispatch` | `dispatch_plugin` is fully implemented (dispatch infrastructure) |
| `test_bc_5_02_001_ec003_panic_string_returns_plugin_panic_error` | `dispatch` | `dispatch_plugin` is fully implemented |
| `test_bc_5_02_001_ec003_no_panic_returns_ok` | `dispatch` | `dispatch_plugin` is fully implemented |
| `test_bc_5_02_001_ec003_plugin_panic_carries_plugin_name` | `dispatch` | `dispatch_plugin` is fully implemented |
| `test_bc_5_02_001_plugin_error_display_contains_plugin_name_and_message` | `dispatch` | Tests `thiserror`-derived Display |
| `test_bc_5_02_002_external_plugin_id_returns_expected_value` | `external_plugin_test` | Tests `TestDataSource` struct; no stub dependency |
| `test_bc_5_02_002_external_plugin_usable_without_bundled_plugins` | `external_plugin_test` | Uses direct `PluginRegistry::register_data_source`; no stub dependency |

## AC Coverage

| AC | Coverage | Test(s) | Status |
|----|----------|---------|--------|
| AC-001 | register_bundled_plugins → Ok(registry) with surface_count==10 | `test_bc_5_02_001_register_bundled_plugins_*` | RED |
| AC-002 | Empty builder → Err(MissingSurface) | `test_bc_5_02_001_empty_builder_returns_err_*` | GREEN (STORY-083) |
| AC-003 | `cargo build --workspace` passes | Build gate | PASS |
| AC-004 | surface_count()==10 and surface_names() canonical | `test_bc_5_02_001_surface_count_and_surface_names_*` | RED |
| AC-005 | Compilation-only (cargo-tree gate) | No unit test needed | N/A |
| AC-006 | Compilation-only (clippy gate) | No unit test needed | N/A |
| AC-007 | External plugin compiles + registers | `test_bc_5_02_002_external_plugin_*` | RED (bundled reg), GREEN (standalone) |
| AC-008 | Plugin panic → PluginError::PluginPanic | `test_bc_5_02_001_ec003_*` | GREEN (dispatch implemented) |

## Files Created

| File | Purpose |
|------|---------|
| `crates/slideforge/Cargo.toml` | Updated with all 10 plugin crate deps + dev-dependencies |
| `crates/slideforge/src/lib.rs` | Public API + re-exports; `build()` stub |
| `crates/slideforge/src/error.rs` | `PluginError`, `BuildError` enums; `RegistryError` re-export |
| `crates/slideforge/src/dispatch.rs` | `dispatch_plugin` catch_unwind wrapper + AC-008 tests |
| `crates/slideforge/src/registry.rs` | `register_bundled_plugins` stub; `default_registry` stub; tests |
| `crates/slideforge/tests/external_plugin_test.rs` | AC-007 dog-fooding test |
| `.factory/cycles/STORY-049/implementation/red-gate-log.md` | This file |

## Constructor/API Findings for Implementer

### PluginRegistryBuilder API (from STORY-083)
- `register_data_source(&mut self, plugin: Box<dyn DataSource + Send + Sync>) -> &mut Self`
- `register_exporter(&mut self, plugin: Box<dyn Exporter + Send + Sync>) -> &mut Self`
- `register_chart_renderer(&mut self, plugin: Box<dyn ChartRenderer + Send + Sync>) -> &mut Self`
- `register_diagram_renderer(&mut self, plugin: Box<dyn DiagramRenderer + Send + Sync>) -> &mut Self`
- `register_validator(&mut self, plugin: Box<dyn Validator + Send + Sync>) -> &mut Self`
- `register_math_renderer(&mut self, plugin: Box<dyn MathRenderer + Send + Sync>) -> &mut Self`
- `register_brand_provider(&mut self, plugin: Box<dyn BrandProvider + Send + Sync>) -> &mut Self`
- `register_slide_type(&mut self, plugin: Box<dyn SlideType + Send + Sync>) -> &mut Self`
- `register_section_type(&mut self, plugin: Box<dyn SectionType + Send + Sync>) -> &mut Self`
- `register_inline_format(&mut self, plugin: Box<dyn InlineFormat + Send + Sync>) -> &mut Self`
- `build(self) -> Result<PluginRegistry, RegistryError>` (consumes builder)

### Bundled Constructor Names Per Surface

| Surface | Crate | Constructor(s) |
|---------|-------|----------------|
| DataSource | `slideforge_data` | `FileDataSource::new(path)`, `HttpDataSource::new(url)`, `XlsxDataSource::new(path)`, `SqliteDataSource::new(path, query)` |
| Exporter | `slideforge_pptx/docx/pdf` | `PptxExporter::new()`, `DocxExporter` (unit struct), `PdfExporter::new()` |
| ChartRenderer | `slideforge_charts` | `ChartRendererImpl::new()` |
| DiagramRenderer | `slideforge_diagrams` | `DiagramRendererImpl::new()` |
| Validator | `slideforge_validate` | `AltTextValidator` (unit), `ZeroSlideValidator` (unit), `CanvasOverflowValidator::from_config(&ValidationConfig::default())`, `LabelCheckValidator` (unit), `LangValidator` (unit) |
| MathRenderer | `slideforge_math` | `MathRendererImpl::new()` |
| BrandProvider | `slideforge_brand` | `BrandLoader::new()`, `BrandSynthesizer` (unit struct, `#[derive(Default)]`) |
| SlideType | `slideforge_plugin_api::slide_types` | All 31 types via `<Type>SlideType::new()` |
| SectionType | `slideforge_plugin_api::section_types` | All 7 types are unit structs with `#[derive(Default)]` — NO `::new()`. Use struct literal. |
| InlineFormat | `slideforge_plugin_api::inline_formats` | `DefaultInlineFormat` (unit struct, `#[derive(Default, Copy)]`) — NO `::new()`. Use struct literal. |

## Notes / Concerns

None. All BCs covered. No BC ambiguities found. The implementation is straightforward:
replace the `todo!()` in `register_bundled_plugins` with the 44-line blueprint already in
the comments.
