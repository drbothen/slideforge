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
- Blocks STORY-050: E2E integration tests require a working `PluginRegistry::default()`
  that registers all 10 plugin surfaces.

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

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-5.02.001 | All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API | AC-001 through AC-005 |
| BC-5.02.002 | No Bundled Plugin Bypasses the Registered Trait Interface (Dog-Fooding Guarantee) | AC-006 through AC-008 |

## Acceptance Criteria

### AC-001: PluginRegistry::default() registers all 10 trait surfaces
(traces to BC-5.02.001 postcondition 1)

`PluginRegistry::default()` completes without error and has non-empty registration
for all 10 trait surfaces:
- DataSource: JSON, CSV, YAML, TOML, HTTP sources (from `slideforge-data`)
- Exporter: PPTX (`slideforge-pptx`), DOCX (`slideforge-docx`), PDF (`slideforge-pdf`)
- ChartRenderer: plotters-backed renderer (`slideforge-charts`)
- DiagramRenderer: mermaid-rs-renderer (`slideforge-diagrams`)
- Validator: accessibility + overflow validators (`slideforge-validate`)
- MathRenderer: pulldown-latex math renderer (`slideforge-math`)
- BrandProvider: file-based brand provider (`slideforge-brand`)
- SlideType: all 31 slide type implementations (`slideforge-types`)
- SectionType: all auto-generated section types (`slideforge-types`)
- InlineFormat: all 11 inline format types (`slideforge-types`)

Unit test: `PluginRegistry::default()` returns `Ok(registry)` where `registry.
surface_count()` == 10 and each surface has at least 1 implementation.

### AC-002: Unregistered surface returns RegistryError, not panic
(traces to BC-5.02.001 invariant 3)

`PluginRegistry::builder().build()` (without registering all surfaces) returns
`Err(RegistryError::MissingSurface { surface: "DataSource" })` (or similar) when
`DataSource` has no registered implementation. This is verified by unit test:
build an empty registry and call `build()`, assert the error is returned.

### AC-003: cargo build --workspace passes with all 10 surfaces covered
(traces to BC-5.02.001 postcondition 4)

`cargo build --workspace` and `cargo test --workspace` both pass. This is the
CI gate that confirms all 10 surfaces are implemented and compile correctly.

### AC-004: PluginRegistry exposes surface_count() and surface_names()
(traces to BC-5.02.001 postcondition 2)

`PluginRegistry::surface_count() -> usize` returns 10 for the default registry.
`PluginRegistry::surface_names() -> Vec<&str>` returns the canonical names of all
registered surfaces.

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

### AC-008: Plugin registered but panics → E-PLG-001 error, not process crash
(traces to BC-5.02.001 edge case EC-003)

If a registered plugin implementation panics during execution, the panic is caught
at the plugin dispatch boundary (using `std::panic::catch_unwind`) and returned as
`Err(PluginError::PluginPanic { plugin_name, message })`. The `slideforge` process
does not crash. Verified by unit test using a test plugin that panics.

## Tasks

- [ ] Create `crates/slideforge/Cargo.toml`:
  - All plugin crates as workspace dependencies
  - `slideforge-plugin-api` (workspace)
  - `thiserror = "=2.0.18"`
- [ ] Create `crates/slideforge/src/lib.rs`:
  - `pub use slideforge_plugin_api::PluginRegistry;`
  - `pub fn build(source: &str, options: BuildOptions) -> Result<BuildOutput, BuildError>`
  - `#![forbid(unsafe_code)]`
- [ ] Create `crates/slideforge/src/registry.rs`:
  - `PluginRegistryBuilder` (or extend plugin-api's builder)
  - `register_bundled_plugins(builder: &mut Builder)` — registers all 10 surfaces
  - `PluginRegistry::default()` calls `register_bundled_plugins`
- [ ] Create `crates/slideforge/src/error.rs`:
  - `RegistryError`, `PluginError` enums
  - `BuildError` enum wrapping all pipeline errors
- [ ] Create `crates/slideforge/src/dispatch.rs`:
  - `catch_unwind` wrapper for plugin calls → `PluginError::PluginPanic`
- [ ] Write unit tests:
  - `PluginRegistry::default()` → `surface_count() == 10`
  - Empty registry → `RegistryError::MissingSurface`
  - Panic-catching dispatch test
- [ ] Write dog-fooding test in `tests/external_plugin_test.rs`
- [ ] Verify `slideforge` is the root crate in `Cargo.toml` `[workspace]`

## Previous Story Intelligence

N/A — first story in EPIC-21. However, this story is the culmination of all Wave 1
through Wave 4 plugin implementation stories. The root crate is intentionally thin:
it contains no logic of its own, only assembly.

Design principle from ARCH-INDEX: "No code lives in the root crate beyond registry
construction." If the implementer finds themselves adding parsing, evaluation, or
export logic to the root crate, it belongs in a subsystem crate instead.

## Architecture Compliance Rules

1. **Root crate is assembly-only**: No parsing, evaluation, layout, or export logic.
   Any non-trivial logic added here is a violation of the architecture.
2. **Dog-fooding guarantee (BC-5.02.002)**: Each `register_*` call in `registry.rs`
   uses the `Box<dyn PluginTrait>` pattern, not a direct struct reference. The
   compiler enforces this because the registration API accepts `Box<dyn Trait>`.
3. **Panic isolation (BC-5.02.001 edge case EC-003)**: Plugin calls are wrapped in
   `catch_unwind`. This is the ONLY use of `catch_unwind` in the codebase — it is
   the plugin dispatch boundary.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| All plugin workspace crates | workspace | Plugin implementations to register |
| `slideforge-plugin-api` | workspace | PluginRegistry, all trait surfaces |
| `thiserror` | `=2.0.18` | Error enums |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge/Cargo.toml` | Create (or already exists as stub) | All plugin deps |
| `crates/slideforge/src/lib.rs` | Create | Public API + re-exports |
| `crates/slideforge/src/registry.rs` | Create | register_bundled_plugins() |
| `crates/slideforge/src/error.rs` | Create | RegistryError, PluginError, BuildError |
| `crates/slideforge/src/dispatch.rs` | Create | catch_unwind plugin dispatch wrapper |
| `crates/slideforge/tests/external_plugin_test.rs` | Create | Dog-fooding test |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,500 |
| BC-5.02.001 | ~1,200 |
| BC-5.02.002 | ~1,000 |
| Plugin trait definitions (STORY-002) | ~1,500 |
| Each Wave 4 exporter's public API (4 × 400) | ~1,600 |
| Test files to write | ~1,500 |
| **Total** | **~9,300** |

Context budget: ~9% of a 100k-token context window. Within limit.

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

The root `slideforge` crate MUST NOT depend on any crate that is not a direct
plugin implementation. In particular:
- No `slideforge-syntax`, `slideforge-eval`, `slideforge-layout`, `slideforge-validate`
  as direct dependencies (these are accessed via the plugin trait API, not directly)
- Exception: `slideforge-types` for `LaidOutDeck` and `BuildOptions` types
- Exception: `slideforge-plugin-api` for the registry infrastructure

If the root crate gains direct dependencies on subsystem implementation crates,
the plugin architecture is compromised.
