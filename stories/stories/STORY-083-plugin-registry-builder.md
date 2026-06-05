---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-083
title: "Plugin Registry Builder + Surface Enforcement"
epic: EPIC-21
wave: 4
points: 3
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-5.02.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022]
crate: slideforge-plugin-api
target_module: slideforge-plugin-api
subsystems: [SS-14]
depends_on:
  - STORY-002
blocks:
  - STORY-049
estimated_days: 1
---

# STORY-083: Plugin Registry Builder + Surface Enforcement

## Subsystem Anchor Justification

SS-14 (Plugin API) owns this story. `slideforge-plugin-api` is the authoritative home for
the `PluginRegistry`, `PluginRegistryBuilder`, `RegistryError`, and all surface-enforcement
logic per ADR-016 Decision 1. The registry builder and its fail-on-empty semantics are
plugin-api concerns — the root crate is the *caller* (assembler), not the *definer*.

## Dependency Anchor Justifications

- Depends on STORY-002: STORY-002 delivered the `PluginRegistry` struct with the 10 surface
  vectors (`register_*` / `lookup_*` API). STORY-083 extends that struct with the builder
  pattern, RegistryError, and introspection methods. The STORY-002 registry.rs file is the
  primary modification target.
- Blocks STORY-049: STORY-049 (Plugin Registry Assembly) depends on `PluginRegistryBuilder`
  being available in `slideforge-plugin-api` to assemble the production registry. Without
  STORY-083, STORY-049's AC-002 (`RegistryError::MissingSurface`) and AC-004 (`surface_count()`
  / `surface_names()`) cannot be satisfied.

## Summary

Extend `slideforge-plugin-api/src/registry.rs` — delivered by STORY-002 — with:

1. `PluginRegistryBuilder`: a builder that accumulates per-surface registrations and
   finalizes into a `PluginRegistry` via `build() -> Result<PluginRegistry, RegistryError>`.
2. `RegistryError::MissingSurface { surface: &'static str }`: a typed error returned when
   any of the 10 required surfaces has zero registrations at `build()` time.
3. `PluginRegistry::surface_count() -> usize`: introspection — number of surfaces with at
   least one registration in the assembled registry.
4. `PluginRegistry::surface_names() -> Vec<&'static str>`: introspection — names of all
   registered surfaces.

This closes the BC-5.02.001 invariant 3 gap in STORY-002's delivered code: the current
`PluginRegistry::new()` / `#[derive(Default)]` silently accepts zero registrations. After
this story, any caller that calls `PluginRegistryBuilder::build()` without registering all
10 surfaces receives a typed `Err(RegistryError::MissingSurface { .. })` — not a panic,
not a silent no-op.

Backward compatibility: the existing `register_*` / `lookup_*` methods on `PluginRegistry`
are preserved unchanged. The builder provides an alternative construction path; callers
using `PluginRegistry::default()` (if any) are migrated within this story's scope to
`PluginRegistryBuilder`.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-5.02.001 | All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API | AC-001 through AC-005 |

## Acceptance Criteria

### AC-001: PluginRegistryBuilder::default().build() with no registrations returns Err(MissingSurface)
(traces to BC-5.02.001 invariant 3 postcondition on build())

`PluginRegistryBuilder::default()` with zero `register_*` calls returns
`Err(RegistryError::MissingSurface { surface: "DataSource" })` (or the first
missing surface name, alphabetically or by declaration order — either is acceptable,
but the choice must be consistent and documented). The error is a typed `RegistryError`
variant — not a `panic!`, not an `.unwrap()` failure, not an `anyhow::Error`.

Unit test:
```rust
let result = PluginRegistryBuilder::default().build();
assert!(matches!(result, Err(RegistryError::MissingSurface { .. })));
```

### AC-002: PluginRegistryBuilder with all 10 surfaces registered returns Ok(registry)
(traces to BC-5.02.001 invariant 3 postcondition on build())

A `PluginRegistryBuilder` that has had at least one `register_*` call for each of the
10 required surfaces returns `Ok(PluginRegistry)`. The returned registry has
`surface_count() == 10`.

Unit test: register one stub impl per surface, call `build()`, assert `Ok(..)` and
`registry.surface_count() == 10`.

### AC-003: PluginRegistry::surface_count() returns 10 for fully-registered registry
(traces to BC-5.02.001 postcondition 2)

`surface_count()` returns the count of surfaces that have at least one registered
implementation. For the fully-registered registry (all 10 surfaces), returns 10.
For a partial registry (e.g., only DataSource registered via the old mutation API),
returns the actual non-zero count.

### AC-004: PluginRegistry::surface_names() returns all 10 canonical surface names
(traces to BC-5.02.001 invariant 1)

`surface_names()` returns a `Vec<&'static str>` containing exactly the canonical
names: `["DataSource", "Exporter", "ChartRenderer", "DiagramRenderer", "Validator",
"MathRenderer", "BrandProvider", "SlideType", "SectionType", "InlineFormat"]`
(or the subset that is registered, in the same canonical order). For the fully-registered
registry, `surface_names().len() == 10`.

### AC-005: RegistryError implements Debug, Display, Error; is non-exhaustive
(traces to BC-5.02.001 invariant 3)

`RegistryError` derives or implements `Debug`, `Display`, and `std::error::Error`
via `thiserror`. The error message for `MissingSurface { surface }` is:
`"required plugin surface '{surface}' has no registered implementations"` (or equivalent
human-readable form). The enum is `#[non_exhaustive]` to allow adding variants in
future versions without breaking callers.

## Tasks

- [ ] Add `PluginRegistryBuilder` struct to `crates/slideforge-plugin-api/src/registry.rs`:
  - Fields: one `Vec<Box<dyn T + Send + Sync>>` per surface (10 total)
  - Per-surface `register_*` builder methods returning `&mut Self`
  - `build(self) -> Result<PluginRegistry, RegistryError>` — checks all 10 vecs are
    non-empty; returns `Err(RegistryError::MissingSurface { surface: "<name>" })` on
    first empty surface; returns `Ok(PluginRegistry)` when all non-empty
- [ ] Add `RegistryError` enum to `crates/slideforge-plugin-api/src/registry.rs` (or a new
  `src/error.rs` re-exported from `lib.rs`):
  - `#[non_exhaustive]` — future variants possible
  - `MissingSurface { surface: &'static str }` — `thiserror`-derived Display
- [ ] Add `surface_count() -> usize` to `PluginRegistry`:
  - Returns the number of surfaces with at least one registration
  - Count is based on the 10 canonical surfaces, not arbitrary vecs
- [ ] Add `surface_names() -> Vec<&'static str>` to `PluginRegistry`:
  - Returns names of all registered surfaces (non-empty vec per surface)
  - The 10 canonical names in declaration order
- [ ] Ensure `PluginRegistryBuilder` is exported from `lib.rs` as part of the public API
- [ ] Write unit tests:
  - `PluginRegistryBuilder::default().build()` → `Err(MissingSurface)`
  - Partially filled builder (8/10 surfaces) → `Err(MissingSurface { surface: "<missing>" })`
  - Fully filled builder (10/10 surfaces) → `Ok(registry)` with `surface_count() == 10`
  - `surface_names()` returns all 10 canonical names
  - `RegistryError` Display message is human-readable

## Previous Story Intelligence

N/A — first story extending STORY-002's registry module. STORY-002 delivered the 10 surface
vecs, `register_*`, and `lookup_*`. This story adds the builder layer above those primitives.

Key constraint from STORY-002: the `register_*` / `lookup_*` mutation API was designed for
internal assembly (not as the primary user-facing API). STORY-083 formalizes the intended
usage by exposing a type-safe builder as the recommended construction path. Both APIs
coexist — the builder delegates to the same internal vecs.

## Architecture Compliance Rules

1. **All work in slideforge-plugin-api only.** No root-crate changes in this story. The builder
   and error types are plugin-api concerns per ADR-016 Decision 1.
2. **Typed error, not panic.** `RegistryError::MissingSurface` must be a `thiserror`-derived
   struct variant. `panic!`, `unwrap()`, `expect()`, or `anyhow` are not acceptable for this
   error path.
3. **Backward compat preserved.** The existing `register_*` / `lookup_*` on `PluginRegistry`
   remain unchanged. No test that calls `PluginRegistry::default()` and then `register_*`
   should be broken by this story.
4. **`#![forbid(unsafe_code)]`** already in force for the crate; no unsafe additions.
5. **`clippy::pedantic` clean.** Builder methods must have `#[must_use]` where appropriate
   (the `build()` method returns a `Result` that must not be ignored).

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `thiserror` | `=2.0.18` | `RegistryError` derive macro |
| `slideforge-plugin-api` | workspace | Existing registry.rs modification target |

No new dependencies required — `thiserror` is already a dependency of `slideforge-plugin-api`
(confirmed: `Cargo.toml:22: thiserror = { workspace = true }`).

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-plugin-api/src/registry.rs` | Modify | Add `PluginRegistryBuilder`, `RegistryError`, `surface_count()`, `surface_names()` |
| `crates/slideforge-plugin-api/src/lib.rs` | Modify | Export `PluginRegistryBuilder`, `RegistryError` from crate root |

No new files required. All additions go into the existing `registry.rs` (or a companion
`error.rs` if the file grows unwieldy, re-exported from `lib.rs`).

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,000 |
| BC-5.02.001 (invariant 3 focus) | ~1,200 |
| ADR-016 Decision 1 | ~600 |
| Existing `registry.rs` (lines 72–272) | ~2,000 |
| New test code to write | ~800 |
| **Total** | **~6,600** |

Context budget: ~7% of a 100k-token context window. Within limit.

## Test Strategy

- **Unit tests in `registry.rs` `#[cfg(test)] mod tests`**:
  - Empty builder → `Err(MissingSurface)` (AC-001)
  - Fully registered builder → `Ok(..)` + `surface_count() == 10` (AC-002, AC-003)
  - `surface_names()` returns 10 names (AC-004)
  - `RegistryError` Display is human-readable (AC-005)
  - Partial builder (missing 1 of 10) → `Err(MissingSurface { surface: "<correct>" })`
- **Existing tests must not regress**: the full test suite in `registry.rs` and
  `dog_food_test.rs` must pass without modification (backward compat of `register_*` /
  `lookup_*` API is required).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Builder called with duplicate surface registrations (multiple `register_data_source` calls) | All registrations are stored; `surface_count()` counts surfaces, not total plugins; `build()` succeeds if all 10 surfaces have at least 1 |
| EC-002 | `surface_names()` called on a partially-assembled registry (not built via builder) | Returns only names of surfaces with at least 1 registration |
| EC-003 | `RegistryError` is pattern-matched by caller | `#[non_exhaustive]` forces a wildcard arm; existing match arms continue to work |

## Forbidden Dependencies

`slideforge-plugin-api` MUST NOT gain new workspace crate dependencies as a result of this
story. The builder and error types use only `thiserror` (already a dep) and the types
already in `registry.rs`. No `slideforge-syntax`, `slideforge-eval`, `slideforge-layout`,
or `slideforge-validate` dependencies are introduced.

If the implementation requires a type not available in `slideforge-plugin-api`, that type
must either already exist there or be defined here — not imported from another workspace crate.
