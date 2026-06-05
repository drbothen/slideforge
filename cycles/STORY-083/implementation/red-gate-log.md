---
story: STORY-083
title: "Plugin Registry Builder + Surface Enforcement"
red_gate_verified: true
red_gate_date: 2026-06-05
producer: vsdd-factory:test-writer
crate: slideforge-plugin-api
---

# Red Gate Log — STORY-083

## Purpose

Documents the 8 failing behavioral tests verified RED (before implementation) per
the TDD Red Gate discipline. Tests were written by the test-writer agent and
confirmed failing before the implementer wrote a single line of production code.

## Red Gate Summary

| Test Name | Crate | Target Symbol | Failure Mode |
|-----------|-------|--------------|--------------|
| `test_registry_builder_all_surfaces_present` | slideforge-plugin-api | `PluginRegistryBuilder` | `PluginRegistryBuilder` did not exist — compile error |
| `test_registry_builder_missing_surface_error` | slideforge-plugin-api | `RegistryError::MissingSurface` | `RegistryError::MissingSurface` variant did not exist — compile error |
| `test_registry_builder_surface_count` | slideforge-plugin-api | `surface_count()` | `surface_count()` method did not exist — compile error |
| `test_registry_builder_surface_names` | slideforge-plugin-api | `surface_names()` | `surface_names()` method did not exist — compile error |
| `test_surface_names_constant_cardinality` | slideforge-plugin-api | `SURFACE_NAMES` | `SURFACE_NAMES` const did not exist — compile error |
| `test_partial_registry_missing_exporter` | slideforge-plugin-api | `RegistryError::MissingSurface` | Variant did not exist — compile error |
| `test_partial_registry_missing_chart_renderer` | slideforge-plugin-api | `RegistryError::MissingSurface` | Variant did not exist — compile error |
| `test_surface_presence_helper` | slideforge-plugin-api | `surface_presence` helper | Helper function did not exist — compile error |

## BC Coverage

All 8 tests collectively verify BC-5.02.001 invariant 3:

> **Invariant 3:** Every mandatory plugin surface (all 10 `PluginSurface` variants)
> MUST be registered before the registry is built. Attempting to build with any
> surface absent MUST return `RegistryError::MissingSurface { surface: PluginSurface }`.

This invariant was present in BC-5.02.001 v1.3 (amended 2026-06-04, LESSON-13
reconciliation) but was NOT implemented in STORY-002's delivered registry code.
STORY-083 closes this gap.

## Red Gate Protocol Compliance

Per SID-1 (No-Ignored-Test Rationalization) and STORY-083 story spec:

- Tests were written BEFORE implementation (strict TDD)
- All 8 tests confirmed failing (compile errors) before implementer dispatch
- No test relies on external dependencies (pure in-memory, no filesystem, no DTU)
- Integration path exercised: `PluginRegistryBuilder::new()` → register surfaces
  → `build()` → `PluginRegistry` or `RegistryError::MissingSurface`

## Post-Implementation Result

After implementation by the implementer agent:

- 133 plugin-api tests pass (including the 8 new Red Gate tests)
- 0 failures
- Workspace-level: all tests continue to pass

## Notes

- CI required fix commit `bc52da1a` (pedantic clippy + rustdoc link) before PR #57 went green. See LESSON-16 in STATE.md.
- Adversary cascade: 6 passes total; strict-CLEAN on passes 4/5/6.
- Findings closed: F-083-01 (doc examples compile), F-083-02 (single-source surface_presence helper), F-083-03 (partial-registry coverage tests), F-083-P2-01 (SURFACE_NAMES rustdoc anchor), L-083-P3-01 (unused_mut in doctests).
