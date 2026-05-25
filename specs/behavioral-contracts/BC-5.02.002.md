---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-021
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-5.02.002: No Bundled Plugin Bypasses the Registered Trait Interface (Dog-Fooding Guarantee)

## Description

Every bundled plugin (PPTX exporter, DOCX exporter, PDF exporter, chart renderer,
diagram renderer, etc.) accesses other components exclusively through the declared
plugin trait interfaces. No bundled plugin uses internal crate-private APIs, direct
struct field access across crate boundaries, or `use slideforge_internal::*` bypass
paths. If a bundled plugin needs functionality not available through the trait API,
the trait API is enhanced — the bypass is not the solution.

## Preconditions

1. A bundled plugin is implemented (any of the 10 extensibility surfaces).
2. The plugin needs to interact with other pipeline components (e.g., exporter reads Brand, ChartRenderer uses data).

## Postconditions

1. All cross-component interactions occur via the declared trait APIs.
2. No `pub(crate)` or `pub(super)` functions in other crates are called by plugin code.
3. The plugin compiles and passes tests using only the declared trait interfaces.
4. The plugin's trait implementation is architecturally identical to what a third-party plugin would write.

## Invariants

1. "Bundled plugins dog-food the same API as future external plugins." — No internal bypass paths exist. (DI-008)
2. If a bundled plugin attempts to call a private function in another crate, CI must detect this via `cargo clippy` lint or compilation error.
3. This invariant is verified continuously in CI — not just at release time.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | PPTX exporter needs a Brand field not in the Brand trait | The Brand trait/struct is extended with that field. Not: PPTX exporter reads from an internal BrandImpl struct. |
| EC-002 | ChartRenderer plugin needs layout information not in ChartSpec | ChartSpec is extended. Not: plugin reaches into LaidOutDeck internals. |
| EC-003 | A new bundled plugin is added that works for all tests but bypasses the API | CI lint or review catches the violation. This is a DI-008 violation = bug. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| All 10 bundled plugin implementations compile using public trait APIs only | Compilation succeeds; 0 clippy warnings about visibility violations | happy-path |
| Hypothetical test: plugin calls `slideforge_pptx::internal::LayoutHelper::compute()` (private) | Compilation error: private function; CI blocks merge | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | No bundled plugin crate has cross-crate `use` of crate-private symbols | Integration test: compile each plugin in isolation (without access to internals) |
| VP-TBD | Plugin API test: each trait can be implemented by a minimal test plugin that does NOT import slideforge internals | Unit test per trait |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-021 ("Plugin Architecture with 10 Extensibility Surfaces") per capabilities.md §CAP-021 |
| Capability Anchor Justification | CAP-021 ("Plugin Architecture with 10 Extensibility Surfaces") per capabilities.md §CAP-021 — "no bypass paths allowed" is verbatim from CAP-021's dog-fooding requirement |
| L2 Domain Invariants | DI-008 (all bundled plugins must use plugin trait APIs) |
| Architecture Module | slideforge-plugin-api crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.02.001 — composes with (BC-5.02.001 verifies the API exists; this BC verifies it is actually used)
- BC-4.01.001 — depends on (PPTX exporter is a bundled plugin subject to this BC)
- BC-4.03.001 — depends on (PDF exporter is a bundled plugin subject to this BC)

## Architecture Anchors

- `architecture/plugin-architecture.md#dog-fooding` — dog-fooding guarantee specification

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
