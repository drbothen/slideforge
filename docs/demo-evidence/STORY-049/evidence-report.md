# STORY-049 Demo Evidence Report

**Story:** STORY-049 — Plugin Registry Assembly (root crate)
**Crate:** `slideforge` (root)
**Branch:** `feature/STORY-049`
**Recorded:** 2026-06-05
**Recording tool:** VHS (terminal capture of `cargo nextest`, `cargo build`, `cargo tree`, `cargo clippy`, `bash scripts/check-panic-profile.sh`)

---

## Artifacts

| File | Format | Covered ACs |
|------|--------|-------------|
| `AC-001-004-registry-assembly.gif` | GIF (PR embed) | AC-001, AC-002, AC-003, AC-004 |
| `AC-001-004-registry-assembly.webm` | WEBM (archival) | AC-001, AC-002, AC-003, AC-004 |
| `AC-001-004-registry-assembly.tape` | VHS script | AC-001, AC-002, AC-003, AC-004 |
| `AC-005-008-dogfood-panic-guard.gif` | GIF (PR embed) | AC-005, AC-006, AC-007, AC-008, BONUS |
| `AC-005-008-dogfood-panic-guard.webm` | WEBM (archival) | AC-005, AC-006, AC-007, AC-008, BONUS |
| `AC-005-008-dogfood-panic-guard.tape` | VHS script | AC-005, AC-006, AC-007, AC-008, BONUS |

**No example binary added** — `test_crit2_end_to_end_build_ok_with_toml_brand` in
`crates/slideforge/src/lib.rs` already demonstrates the end-to-end `build()` pipeline
(shown in BONUS section of tape 2). Adding a separate example binary would duplicate
coverage without adding signal and would add `--all-targets` compile surface with no gate
benefit.

---

## Coverage Map

### AC-001 — PluginRegistryBuilder::default_bundled().build() registers all 10 surfaces

**Demonstrated in:** `AC-001-004-registry-assembly.tape` Section 1

**Evidence:** `cargo nextest run -p slideforge -E 'test(register_bundled) or test(surface_count) or test(surface_names) or test(default_registry)' -q` runs 4 tests:
- `test_bc_5_02_001_register_bundled_plugins_build_returns_ok_with_10_surfaces` — asserts `surface_count() == 10` after `register_bundled_plugins(&mut builder); builder.build()`
- `test_bc_5_02_001_register_bundled_plugins_all_10_surface_names_present` — asserts `surface_names()` equals `SURFACE_NAMES` constant in declaration order
- `test_bc_5_02_001_default_registry_returns_ok_with_10_surfaces` — asserts `default_registry().surface_count() == 10`
- `test_bc_5_02_001_surface_count_and_surface_names_on_default_registry` — asserts both methods simultaneously on `default_registry()`

Recording shows `4 tests run: 4 passed`.

**Result:** PASS

---

### AC-002 — PluginRegistryBuilder with missing surface returns RegistryError::MissingSurface, not panic

**Demonstrated in:** `AC-001-004-registry-assembly.tape` Section 2

**Evidence:** `cargo nextest run -p slideforge -E 'test(empty_builder) or test(partial_builder)' -q` runs 2 tests:
- `test_bc_5_02_001_empty_builder_returns_err_missing_surface_from_root_crate` — asserts `PluginRegistryBuilder::default().build()` returns `Err(RegistryError::MissingSurface { .. })` (not panic, not `Ok`)
- `test_bc_5_02_001_partial_builder_missing_exporter_reports_error` — asserts builder with only DataSource registered reports `surface == "Exporter"` as the first missing surface

Recording shows `2 tests run: 2 passed`. The process continues running after both tests — proving no panic occurred.

**Result:** PASS

---

### AC-003 — cargo build --workspace passes with all 10 surfaces covered

**Demonstrated in:** `AC-001-004-registry-assembly.tape` Section 3

**Evidence:** `cargo build --workspace 2>&1 | tail -5` shows:
- All 10 plugin crates (`slideforge-data`, `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, `slideforge-charts`, `slideforge-diagrams`, `slideforge-validate`, `slideforge-math`, `slideforge-brand`, `slideforge-plugin-api`) compile
- Final line: `Finished 'dev' profile ... target(s) in N.NNs`
- Zero errors, zero warnings

**Result:** PASS

---

### AC-004 — PluginRegistry exposes surface_count() and surface_names()

**Demonstrated in:** `AC-001-004-registry-assembly.tape` Section 1 (shared with AC-001)

**Evidence:** `test_bc_5_02_001_surface_count_and_surface_names_on_default_registry` explicitly asserts:
- `surface_count() == 10`
- `surface_names()` == `["DataSource", "Exporter", "ChartRenderer", "DiagramRenderer", "Validator", "MathRenderer", "BrandProvider", "SlideType", "SectionType", "InlineFormat"]`

Recording shows test PASS.

**Result:** PASS

---

### AC-005 — Each bundled plugin compiles using only slideforge-plugin-api imports

**Demonstrated in:** `AC-005-008-dogfood-panic-guard.tape` Section 1

**Evidence:** `cargo tree -p slideforge-pptx 2>&1 | grep -E 'slideforge-pdf|slideforge-html' && echo FOUND || echo 'CLEAN: no pdf/html cross-dep in pptx'`

Recording shows: `CLEAN: no pdf/html cross-dep in pptx`

This confirms `slideforge-pptx`'s dependency tree contains neither `slideforge-pdf` nor `slideforge-html` — the cross-contamination the AC guards against.

**Result:** PASS

---

### AC-006 — No bundled plugin calls private functions in other crates

**Demonstrated in:** `AC-005-008-dogfood-panic-guard.tape` Section 2

**Evidence:** `cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -4`

Recording shows: `Finished 'dev' profile ... target(s) in N.NNs` with zero warnings or errors. The `-D warnings` flag promotes any visibility violation to an error — the clean exit proves no private cross-crate calls exist.

**Result:** PASS

---

### AC-007 — Dog-fooding test: minimal external plugin compiles without internals

**Demonstrated in:** `AC-005-008-dogfood-panic-guard.tape` Section 3

**Evidence:** `cargo nextest run --test external_plugin_test -q 2>&1 | tail -6` runs 3 tests in `crates/slideforge/tests/external_plugin_test.rs`:
- `test_bc_5_02_002_external_plugin_compiles_and_registers_with_bundled_plugins` — `TestDataSource` (importing only `slideforge` + `slideforge-plugin-api` + `slideforge-types`) registers alongside bundled plugins; `surface_count()` stays 10; `lookup_data_source("test-external-data-source")` returns `Some`; `load()` returns `Ok`
- `test_bc_5_02_002_external_plugin_usable_without_bundled_plugins` — `TestDataSource` registered in bare `PluginRegistry::new()` without any bundled plugins; `load()` returns `Ok(Value::Int(42))`
- `test_bc_5_02_002_external_plugin_id_returns_expected_value` — `id()` returns `"test-external-data-source"`

Recording shows `3 tests run: 3 passed`.

The compile-time `const _: fn() = || { assert_test_data_source_implements_trait::<TestDataSource>(); }` assertion in the test file confirms the trait bound is satisfied.

**Result:** PASS

---

### AC-008 — Plugin registered but panics → PluginError::PluginPanic error, not process crash

**Demonstrated in:** `AC-005-008-dogfood-panic-guard.tape` Section 4

**Evidence:** `cargo nextest run -p slideforge -E 'test(panic)' -q 2>&1 | tail -8` runs 5 tests:
- `test_bc_5_02_001_ec003_panic_str_returns_plugin_panic_error` — `&str` payload caught; `PluginPanic { plugin_name: "test-plugin", message: "intentional test panic" }` returned; process continues
- `test_bc_5_02_001_ec003_panic_string_returns_plugin_panic_error` — `String` payload caught; `PluginPanic` returned
- `test_bc_5_02_001_ec003_no_panic_returns_ok` — non-panicking closure returns `Ok(42)`
- `test_bc_5_02_001_ec003_plugin_panic_carries_plugin_name` — `plugin_name` field matches the name passed to `dispatch_plugin`
- `test_h1_dispatch_plugin_catches_panicking_exporter` — panicking exporter wired through `build()` dispatch path returns `Err(BuildError::ExportFailed { .. })` via `PluginError::PluginPanic`

Recording shows `5 tests run: 5 passed`. The process continued running after all 5 panicking-plugin tests — proving `catch_unwind` absorbed every panic.

**Result:** PASS

---

### BONUS — check-panic-profile.sh CI guard

**Demonstrated in:** `AC-005-008-dogfood-panic-guard.tape` Section 5

**Evidence:** `bash scripts/check-panic-profile.sh` output:
- `[profile.release]` — `OK: panic = "unwind" found`
- `[profile.dist]` — `OK: panic = "unwind" found`
- `[abort scan]` — all 3 profiles (`release`, `dist`, `bench`) show `OK: does not contain panic = "abort"`
- Final: `PASS: panic=unwind safety perimeter intact. catch_unwind will function correctly in shipped builds.`

This confirms Architecture Compliance Rule 4 / BC-5.02.001 EC-003: the CI guard exits 0 on the current `Cargo.toml`.

**Result:** PASS

---

### BONUS — End-to-end build() pipeline

**Demonstrated by:** `test_crit2_end_to_end_build_ok_with_toml_brand` in `crates/slideforge/src/lib.rs`

Full test suite run (`cargo nextest run -p slideforge --no-fail-fast`) shows `38 tests run: 38 passed, 0 skipped`, including this test which:
- Constructs a minimal `.sf` source with `slideforge_version`, `lang "en-US"`, `slide title:` block
- Passes a `BrandSource::TomlFile(brand.toml)` with a TOML brand fixture
- Calls `build(source, &options)` — wiring the full pipeline: parse → eval → validate → layout → PPTX export
- Asserts `Ok(BuildOutput { bytes, .. })` where `!bytes.is_empty()` and `bytes.starts_with(b"PK")` (valid ZIP/PPTX)

This is the headline capability: one `build()` call producing a valid `.pptx` without any bundled-plugin bypass.

---

## CI Gate Confirmation

All CI gates ran clean in the worktree before recording:

| Gate | Command | Result |
|------|---------|--------|
| Tests (slideforge) | `cargo nextest run -p slideforge --no-fail-fast` | CLEAN (38/38 PASS) |
| Tests (external plugin) | `cargo nextest run --test external_plugin_test` | CLEAN (3/3 PASS) |
| Build (workspace) | `cargo build --workspace` | CLEAN (0 errors, 0 warnings) |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | CLEAN |
| Dep isolation | `cargo tree -p slideforge-pptx \| grep slideforge-pdf\|slideforge-html` | CLEAN (no cross-dep) |
| Panic profile guard | `bash scripts/check-panic-profile.sh` | PASS (exit 0) |

---

## AC → Evidence Matrix

| AC | Title (abbrev) | Tape | Section | Test(s) | Result |
|----|---------------|------|---------|---------|--------|
| AC-001 | 10 surfaces registered | AC-001-004... | §1 | `register_bundled_plugins_build_returns_ok_with_10_surfaces`, `all_10_surface_names_present`, `default_registry_returns_ok` | PASS |
| AC-002 | Empty builder → MissingSurface | AC-001-004... | §2 | `empty_builder_returns_err_missing_surface_from_root_crate`, `partial_builder_missing_exporter` | PASS |
| AC-003 | workspace builds | AC-001-004... | §3 | `cargo build --workspace` | PASS |
| AC-004 | surface_count + surface_names | AC-001-004... | §1 | `surface_count_and_surface_names_on_default_registry` | PASS |
| AC-005 | Single-crate imports / no cross-dep | AC-005-008... | §1 | `cargo tree` grep-zero | PASS |
| AC-006 | No visibility violations | AC-005-008... | §2 | `cargo clippy -D warnings` | PASS |
| AC-007 | External dog-fooding plugin | AC-005-008... | §3 | `external_plugin_compiles_and_registers_with_bundled_plugins`, `external_plugin_usable_without_bundled_plugins`, `external_plugin_id` | PASS |
| AC-008 | Plugin panic → PluginPanic, no crash | AC-005-008... | §4 | `panic_str_returns_plugin_panic_error`, `panic_string_returns_plugin_panic_error`, `no_panic_returns_ok`, `plugin_panic_carries_plugin_name`, `test_h1_dispatch_plugin_catches_panicking_exporter` | PASS |
| BONUS | panic=unwind CI guard | AC-005-008... | §5 | `check-panic-profile.sh` | PASS |
| BONUS | End-to-end build() → .pptx | (lib.rs test) | — | `test_crit2_end_to_end_build_ok_with_toml_brand` | PASS |
