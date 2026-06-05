---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-049
title: "Plugin Registry Assembly (root crate)"
epic: EPIC-21
wave: 4
points: 5
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-5.02.001, BC-5.02.002]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge
target_module: slideforge
subsystems: [SS-14, SS-18]
depends_on:
  - STORY-002
  - STORY-003
  - STORY-015
  - STORY-016
  - STORY-017
  - STORY-018
  - STORY-019
  - STORY-020
  - STORY-021
  - STORY-022
  - STORY-023
  - STORY-029
  - STORY-031
  - STORY-033
  - STORY-034
  - STORY-037
  - STORY-038
  - STORY-039
  - STORY-040
  - STORY-041
  - STORY-042
  - STORY-043
  - STORY-044
  - STORY-045
  - STORY-083
  - STORY-084
  - STORY-085
blocks:
  - STORY-050
estimated_days: 2
---

# STORY-049: Plugin Registry Assembly (root crate)

## Subsystem Anchor Justification

SS-14 (Plugin API) and SS-18 (CLI Orchestrator) jointly own this story.
SS-14 owns the `PluginRegistry` type and the 10 trait surface registrations (the
root crate is the assembly point for all plugin implementations). SS-18 owns this
story because the root `slideforge` crate is the entry point for the CLI binary and
the library API. Per ARCH-INDEX note: "the root `slideforge` crate has no SS-ID
because it is a thin re-export facade that assembles the plugin registry."

## Dependency Anchor Justifications

- Depends on STORY-002: The 10 plugin trait surfaces defined in `slideforge-plugin-api`
  are the interfaces that the registry registers. Required before the registry can
  assemble.
- Depends on STORY-003/STORY-015-017: Validator, SlideType, and all Wave 1-3 plugin
  implementations must exist before they can be registered.
- Depends on STORY-037-045: The four Exporter implementations (PPTX, DOCX, PDF,
  HTML — noting HTML/preview is Wave 5) must be registered. STORY-049 registers
  PPTX + DOCX + PDF exporters (Wave 4); HTML registration is deferred to STORY-050
  which assembles the full registry for E2E tests.
- Depends on STORY-083: STORY-083 delivers `PluginRegistryBuilder`, `RegistryError::MissingSurface`,
  `surface_count()`, and `surface_names()` in `slideforge-plugin-api`. STORY-049's AC-002
  (typed error on empty builder) and AC-004 (surface introspection) cannot be satisfied
  without STORY-083 being merged first.
- Depends on STORY-084: STORY-084 delivers production `SectionType` plugin implementations
  (`ExecutiveSummarySectionType`, `RiskRegisterSectionType`, and 5 manual-only types) in
  `slideforge-plugin-api/src/section_types/`. AC-001 requires all 10 surfaces registered;
  without STORY-084, `SectionType` has only stub implementations.
- Depends on STORY-085: STORY-085 delivers `DefaultInlineFormat` (all 12 `InlineNode`
  variants × 3 output formats) in `slideforge-plugin-api/src/inline_formats/`. AC-001
  requires all 10 surfaces registered; without STORY-085, `InlineFormat` has only stubs.
  STORY-085 also completes the BC-5.02.002 dog-fooding refactor in `slideforge-pptx`.
- Blocks STORY-050: E2E integration tests require a working `PluginRegistryBuilder`-assembled
  registry with all 10 surfaces registered.

## Summary

Implement the root `slideforge` crate: a thin assembly layer that constructs a
`PluginRegistry` containing all bundled plugin implementations. This is the
composition root for the entire system.

The root crate:
1. Defines `PluginRegistry` (or re-exports it from `slideforge-plugin-api`).
2. Implements `PluginRegistry::default()` that registers all bundled plugins for
   all 10 trait surfaces.
3. Exposes the public library API: `build(source: &str, options: BuildOptions) ->
   Result<BuildOutput, BuildError>`.
4. Verifies the dog-fooding guarantee (BC-5.02.002): each bundled plugin is
   registered via the public trait API, not via direct struct construction.
5. Provides `PluginRegistry::builder()` for custom registries (third-party plugin
   use case).

The assembly is compile-time verified: if any of the 10 required trait surfaces is
unregistered when `build()` is called, a `RegistryError::MissingSurface { surface }` is
returned (not a panic).

`build()` is the pipeline driver per ADR-016 Decision 3. It runs the stages in order:
parse (`slideforge-syntax`) → eval (`slideforge-eval`) → **validate** → layout
(`slideforge-layout`) → export (via the registered `Exporter`). The validate stage
iterates all registered `Validator` implementations on the semantic `Deck` (pre-layout)
via `registry.iter_validators()` on the public API (see `crates/slideforge-plugin-api/src/registry.rs`).
`BuildOptions.strict` controls error disposition: `strict=true` (default) causes
`BuildError::ValidationFailed { diagnostics }` on any validation errors; `strict=false`
(`--warn-only` CLI flag) logs warnings and proceeds. Plugin-surface calls
(`BrandProvider::load`, `Validator::validate`, `Exporter::export`) are all routed through
the `catch_unwind` dispatch boundary in `crates/slideforge/src/dispatch.rs` (H1 — see
AC-008).

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-5.02.001 | All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API | AC-001 through AC-005 |
| BC-5.02.002 | No Bundled Plugin Bypasses the Registered Trait Interface (Dog-Fooding Guarantee) | AC-006 through AC-008 |

## Acceptance Criteria

### AC-001: PluginRegistryBuilder::default_bundled().build() registers all 10 trait surfaces
(traces to BC-5.02.001 postcondition 1)

A fully-assembled `PluginRegistry` (built via `PluginRegistryBuilder` with all bundled
plugins registered) returns `Ok(registry)` from `build()` where `registry.surface_count()`
== 10 and each surface has at least 1 implementation:
- DataSource: JSON, CSV, YAML, TOML, HTTP sources (from `slideforge-data`)
- Exporter: PPTX (`slideforge-pptx`), DOCX (`slideforge-docx`), PDF (`slideforge-pdf`)
- ChartRenderer: plotters-backed renderer (`slideforge-charts`)
- DiagramRenderer: mermaid-rs-renderer (`slideforge-diagrams`)
- Validator: accessibility + overflow validators (`slideforge-validate`)
- MathRenderer: pulldown-latex math renderer (`slideforge-math`)
- BrandProvider: file-based brand provider (`slideforge-brand`)
- SlideType: all 31 slide type implementations (`slideforge-plugin-api/src/slide_types/`)
- SectionType: all 7 bundled section type implementations (`slideforge-plugin-api/src/section_types/` — delivered by STORY-084)
- InlineFormat: `DefaultInlineFormat` covering all 12 inline format types (`slideforge-plugin-api/src/inline_formats/` — delivered by STORY-085)

Unit test: call `register_bundled_plugins(&mut builder)` then `builder.build()`, assert
`Ok(registry)` where `registry.surface_count() == 10` and each surface has at least 1
implementation.

### AC-002: PluginRegistryBuilder with missing surface returns RegistryError::MissingSurface, not panic
(traces to BC-5.02.001 invariant 3)

`PluginRegistryBuilder::default().build()` (with zero registrations) returns
`Err(RegistryError::MissingSurface { surface: "<first-missing-surface>" })`. The
`RegistryError::MissingSurface` variant is a typed error (delivered by STORY-083 in
`slideforge-plugin-api`). This is verified by unit test: create an empty builder,
call `build()`, assert `Err(RegistryError::MissingSurface { .. })` is returned — not
a panic, not an `Ok(registry)` with an empty surface vec.

### AC-003: cargo build --workspace passes with all 10 surfaces covered
(traces to BC-5.02.001 postcondition 4)

`cargo build --workspace` and `cargo test --workspace` both pass. This is the
CI gate that confirms all 10 surfaces are implemented and compile correctly.

### AC-004: PluginRegistry exposes surface_count() and surface_names() (delivered by STORY-083)
(traces to BC-5.02.001 postcondition 2)

`PluginRegistry::surface_count() -> usize` returns 10 for the fully-assembled default
registry (all 10 surfaces registered). `PluginRegistry::surface_names() -> Vec<&'static str>`
returns the 10 canonical surface names: `["DataSource", "Exporter", "ChartRenderer",
"DiagramRenderer", "Validator", "MathRenderer", "BrandProvider", "SlideType",
"SectionType", "InlineFormat"]`. These methods are defined in `slideforge-plugin-api`
(STORY-083); STORY-049 exercises them in the assembly integration test.

### AC-005: Each bundled plugin compiles using only slideforge-plugin-api imports
(traces to BC-5.02.001 postcondition 3)

Verified by compilation: each plugin crate is listed in the workspace but imports
ONLY from `slideforge-plugin-api` and `slideforge-types`. This is already enforced
by the individual story implementations (STORY-037 through STORY-045) but the
registry assembly test confirms it: a `cargo tree -p slideforge-pptx` does not
include `slideforge-pdf` or `slideforge-html`.

### AC-006: No bundled plugin calls private functions in other crates
(traces to BC-5.02.002 invariant 1)

`cargo clippy --workspace --all-targets` produces zero warnings about visibility
violations (private function calls across crate boundaries). This is the automated
CI enforcement of the dog-fooding guarantee.

### AC-007: Dog-fooding test: minimal external plugin compiles without internals
(traces to BC-5.02.002 postcondition 4)

A test fixture in `crates/slideforge/tests/external_plugin_test.rs` defines a
minimal `TestDataSource` that implements the `DataSource` trait using ONLY
`slideforge-plugin-api` imports (no `slideforge-data` or other internal crates).
The test verifies this compiles and the minimal plugin can be registered in a
`PluginRegistry` alongside the bundled plugins.

### AC-008: Plugin registered but panics → PluginError::PluginPanic error, not process crash
(traces to BC-5.02.001 edge case EC-003)

If a registered plugin implementation panics during execution, the panic is caught
at the plugin dispatch boundary (using `std::panic::catch_unwind`) and returned as
`Err(PluginError::PluginPanic { plugin_name, message })`. The `slideforge` process
does not crash. Verified by unit test using a test plugin that panics.

`catch_unwind` is genuinely load-bearing: it is wired into `build()`'s plugin dispatch
path in `crates/slideforge/src/dispatch.rs` (not only into unit tests). Every
`BrandProvider::load`, `Validator::validate`, and `Exporter::export` call in `build()`
goes through this boundary. `catch_unwind` is only effective when the process's panic
strategy is `unwind`; `panic = "abort"` silently disables it and causes process
termination on plugin panic. Therefore `[profile.release]` and `[profile.dist]` in the
workspace root `Cargo.toml` MUST set `panic = "unwind"` (see Architecture Compliance
Rules and Forbidden Dependencies for the binding enforcement rule). Note: existing
test-only uses of `catch_unwind` in `slideforge-eval/tests/bleed_tests.rs` and
`slideforge-diagrams/src/normalize.rs` (inside `#[cfg(test)]` contexts) are not
production code and are not affected by this constraint.

## Tasks

- [ ] Update `crates/slideforge/Cargo.toml` (already exists as stub):
  - Confirm all plugin crates as workspace dependencies (slideforge-data, slideforge-pptx,
    slideforge-docx, slideforge-pdf, slideforge-charts, slideforge-diagrams,
    slideforge-validate, slideforge-math, slideforge-brand, slideforge-plugin-api)
  - Confirm pipeline crates: slideforge-syntax, slideforge-eval, slideforge-layout,
    slideforge-validate (per ADR-016 Decision 3 — these are correct deps)
  - `thiserror = "=2.0.18"`
- [ ] Create `crates/slideforge/src/lib.rs`:
  - `pub use slideforge_plugin_api::{PluginRegistry, PluginRegistryBuilder, RegistryError};`
  - `pub fn build(source: &str, options: BuildOptions) -> Result<BuildOutput, BuildError>`
  - `#![forbid(unsafe_code)]`
- [ ] Create `crates/slideforge/src/registry.rs`:
  - `register_bundled_plugins(builder: &mut PluginRegistryBuilder)` — calls all 10
    `builder.register_*()` methods with the production bundled implementations
  - `pub fn default_registry() -> Result<PluginRegistry, RegistryError>` — calls
    `register_bundled_plugins` then `builder.build()`
  - NOTE: `PluginRegistryBuilder` is defined in `slideforge-plugin-api` (STORY-083);
    this file only CALLS it — no builder type is defined in the root crate
- [ ] Create `crates/slideforge/src/error.rs`:
  - `PluginError` enum (root-crate-specific error types)
  - `BuildError` enum wrapping all pipeline errors (RegistryError, parse, eval, layout,
    export); variants carry structured diagnostics with file:line:col spans + hints:
    `ParseFailed { diagnostics: Vec<Diagnostic>, count: usize }`,
    `EvalFailed { diagnostics: Vec<Diagnostic>, count: usize }`,
    `ValidationFailed { diagnostics: Vec<Diagnostic> }` — not raw counts alone
  - Note: `RegistryError` is defined in `slideforge-plugin-api` and re-exported here;
    not redefined
- [ ] Create `crates/slideforge/src/dispatch.rs`:
  - `catch_unwind` wrapper for plugin calls → `PluginError::PluginPanic`
  - This is the only PRODUCTION use of `catch_unwind` in the workspace
- [ ] Write unit tests:
  - `default_registry()` → `Ok(registry)` where `registry.surface_count() == 10`
  - `PluginRegistryBuilder::default().build()` → `Err(RegistryError::MissingSurface { .. })`
  - Panic-catching dispatch test (test plugin that panics → `PluginError::PluginPanic`)
- [ ] Write dog-fooding test in `tests/external_plugin_test.rs`
- [ ] Verify `slideforge` is the root crate in workspace `Cargo.toml` `[workspace]`

## Previous Story Intelligence

Three prerequisite stories (STORY-083, STORY-084, STORY-085) were created by the
LESSON-13 reconciliation (2026-06-04) to close capability gaps that blocked STORY-049:

- **STORY-083** delivered `PluginRegistryBuilder`, `RegistryError::MissingSurface`,
  `surface_count()`, and `surface_names()` in `slideforge-plugin-api/src/registry.rs`.
  Before STORY-083, the `PluginRegistry` was silently permissive (no fail-on-empty
  semantics), violating BC-5.02.001 invariant 3. STORY-049 calls `PluginRegistryBuilder`
  to assemble the registry; STORY-083 defines that builder.

- **STORY-084** delivered 7 bundled `SectionType` implementations in
  `slideforge-plugin-api/src/section_types/`. Before STORY-084, no production `SectionType`
  implementation existed — only test stubs. AC-001 of this story requires the `SectionType`
  surface to be populated with production implementations.

- **STORY-085** delivered `DefaultInlineFormat` (12 `InlineNode` variants × 3 output
  formats) in `slideforge-plugin-api/src/inline_formats/`, and refactored
  `slideforge-pptx`'s inline serialization to dispatch through the plugin trait
  (satisfying BC-5.02.002 EC-004). Before STORY-085, no production `InlineFormat`
  implementation existed.

Architecture authority: per ADR-016 Decision 3, the root crate IS the pipeline driver
(not assembly-only). `build(source, options) -> Result<BuildOutput, BuildError>` belongs
here. The previous "Forbidden Dependencies" clause that excluded `slideforge-syntax`,
`slideforge-eval`, `slideforge-layout`, `slideforge-validate` was incorrect and has been
replaced by the ADR-016 formulation (see Forbidden Dependencies section).

## Architecture Compliance Rules

1. **Root crate is assembly-only**: No parsing, evaluation, layout, or export logic.
   Any non-trivial logic added here is a violation of the architecture.
2. **Dog-fooding guarantee (BC-5.02.002)**: Each `register_*` call in `registry.rs`
   uses the `Box<dyn PluginTrait>` pattern, not a direct struct reference. The
   compiler enforces this because the registration API accepts `Box<dyn Trait>`.
3. **Panic isolation (BC-5.02.001 edge case EC-003)**: Plugin calls are wrapped in
   `catch_unwind`. This is the only PRODUCTION use of `catch_unwind` in the codebase
   — it is the plugin dispatch boundary. Test-only uses of `catch_unwind` exist in
   `slideforge-eval/tests/bleed_tests.rs` and inside `#[cfg(test)]` gates in
   `slideforge-diagrams/src/normalize.rs`; those are test code and do not conflict.
4. **`panic = "unwind"` in shipped profiles (BC-5.02.001 EC-003, binding)**:
   `[profile.release]` and `[profile.dist]` in the workspace root `Cargo.toml` MUST
   declare `panic = "unwind"`. Setting either profile to `panic = "abort"` silently
   disables `catch_unwind` and causes plugin panics to kill the process — a direct
   violation of AC-008 / BC-5.02.001 EC-003. This is a CI-enforced build-time
   constraint: any PR that changes these profiles to `abort` MUST be blocked.
5. **`iter_validators()` on public API only**: The validate stage in `build()` must
   iterate registered `Validator` implementations via `registry.iter_validators()` on
   the `PluginRegistry` public API (defined in `slideforge-plugin-api/src/registry.rs`).
   Direct field access or `pub(crate)` shortcuts into `PluginRegistry` internals from
   the root crate are forbidden.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| All plugin workspace crates | workspace | Plugin implementations to register |
| `slideforge-plugin-api` | workspace | PluginRegistry, all trait surfaces |
| `thiserror` | `=2.0.18` | Error enums |
| `tracing` | workspace | Structured pipeline logging (info/warn/error spans in build()) |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge/Cargo.toml` | Create (or already exists as stub) | All plugin deps; add `tracing` dep |
| `crates/slideforge/src/lib.rs` | Create | Public API + re-exports |
| `crates/slideforge/src/registry.rs` | Create | register_bundled_plugins() |
| `crates/slideforge/src/error.rs` | Create | RegistryError, PluginError, BuildError |
| `crates/slideforge/src/dispatch.rs` | Create | catch_unwind plugin dispatch wrapper |
| `crates/slideforge/tests/external_plugin_test.rs` | Create | Dog-fooding test |
| `crates/slideforge-plugin-api/src/registry.rs` | Modify | Add `iter_validators()` introspection method so the root crate's `build()` validate stage can iterate all registered `Validator` implementations via the public API — no private cross-crate access permitted (Architecture Compliance Rule 5) |
| `Cargo.toml` (workspace root) | Modify | Set `[profile.release]` and `[profile.dist]` `panic = "unwind"` so `catch_unwind` plugin-panic isolation (BC-5.02.001 EC-003 / AC-008) is effective in all shipped builds; `panic = "abort"` in any shipped profile silently disables catch_unwind and is forbidden (Architecture Compliance Rule 4) |
| `crates/slideforge/Cargo.toml` | Modify | Add `tracing` dep (structured pipeline logging in `build()`) |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,500 |
| BC-5.02.001 | ~1,200 |
| BC-5.02.002 | ~1,000 |
| ADR-016 (Decisions 1-3) | ~700 |
| Plugin trait definitions (STORY-002) | ~1,500 |
| STORY-083: PluginRegistryBuilder API | ~800 |
| STORY-084: SectionType impls (public API surface) | ~600 |
| STORY-085: DefaultInlineFormat (public API surface) | ~600 |
| Each Wave 4 exporter's public API (4 × 400) | ~1,600 |
| BC files (2 BCs) | ~2,200 |
| Test files to write | ~1,500 |
| **Total** | **~15,200** |

Context budget: ~15% of a 100k-token context window. Within limit for a 5-point story.

## Test Strategy

- **Unit tests**: `PluginRegistry::default()` has 10 surfaces. Empty builder returns
  correct error. Panic dispatch wrapper catches panics without crashing.
- **Dog-fooding test**: External test plugin compiles with only `slideforge-plugin-api`
  imports. Confirms the API is self-contained.
- **Compilation test**: `cargo build --workspace` is the ultimate assertion that all
  10 surfaces compile with their implementations.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | New 32nd slide type added | `SlideType` surface still covered; AC-001 still passes |
| EC-002 | Plugin API trait method signature change | All implementations fail to compile; CI blocks |
| EC-003 | Plugin panics during execution | `catch_unwind` returns `PluginError::PluginPanic`; no crash |
| EC-004 | Two bundled plugins for same surface | Both registered; caller selects by name/priority |

## Forbidden Dependencies

Per ADR-016 Decision 3, the root `slideforge` crate IS the pipeline driver — it MAY
depend on pipeline crates (`slideforge-syntax`, `slideforge-eval`, `slideforge-layout`,
`slideforge-validate`) to wire the parse → eval → layout → export pipeline and expose
`build()`. The existing `crates/slideforge/Cargo.toml` entries for these crates are
CORRECT and intentional.

The constraint is: **no plugin logic lives in the root crate**. Specifically:
- The root crate MUST NOT call internal/private functions across crate boundaries.
  All cross-crate plugin interaction goes through `Box<dyn Trait>` dispatch (e.g.,
  `Box<dyn Exporter>::export()` — NOT `PptxExporter::internal_method()` directly).
- The root crate MUST NOT define new plugin trait implementations. Plugin implementations
  live in their respective crates (`slideforge-plugin-api/src/{slide_types,section_types,
  inline_formats}/`, `slideforge-data/`, `slideforge-pptx/`, etc.).
- The `build()` function in `crates/slideforge/src/lib.rs` is permitted and required.
  It wires the pipeline stages using the assembled `PluginRegistry`. This is the public
  library API for library consumers.

If the implementer finds `crates/slideforge/src/` accumulating plugin logic (rendering,
serialization, format-specific code), that is a violation and must be moved to the
appropriate subsystem crate.
