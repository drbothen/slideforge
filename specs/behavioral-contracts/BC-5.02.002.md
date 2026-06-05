---
document_type: behavioral-contract
level: L3
version: "1.3"
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
modified:
  - version: "1.3"
    date: 2026-06-04
    author: product-owner
    reason: "M-2 staleness fix (STORY-049 audit): replace stale working label STORY-049-PRE-B2 with canonical story ID STORY-085 in EC-004."
  - version: "1.2"
    date: 2026-06-04
    author: product-owner
    reason: "LESSON-13 reconciliation (STORY-049): Strengthen dog-fooding guarantee to explicitly cover the InlineFormat/OOXML path — the PPTX exporter's internal inline serialization MUST route through the registered InlineFormat plugin trait, not bypass it. Added named OOXML bypass as EC-004 and canonical test vector. (Decision 3 of 3 approved 2026-06-04)."
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

This guarantee explicitly includes the OOXML/PPTX inline serialization path.
`slideforge-pptx` (the PPTX exporter) MUST NOT perform inline node serialization
(`InlineNode` → OOXML `<a:r>` run markup) through internal structs or functions within
`slideforge-pptx`. All inline serialization — including the OOXML path — MUST be
dispatched through the registered `InlineFormat` plugin trait. Where `slideforge-pptx`
currently performs this serialization internally, it MUST be refactored to call the
registered `InlineFormat` implementation for the OOXML output format.

## Preconditions

1. A bundled plugin is implemented (any of the 10 extensibility surfaces).
2. The plugin needs to interact with other pipeline components (e.g., exporter reads Brand, ChartRenderer uses data).

## Postconditions

1. All cross-component interactions (including inline serialization) occur via the declared
   trait APIs. No bundled plugin, including the PPTX exporter, serializes `InlineNode` values
   through internal functions — all such calls are dispatched through the registered
   `InlineFormat` plugin trait.
2. No `pub(crate)` or `pub(super)` functions in other crates are called by plugin code.
3. The plugin compiles and passes tests using only the declared trait interfaces.
4. The plugin's trait implementation is architecturally identical to what a third-party plugin
   would write.
5. The PPTX exporter (`slideforge-pptx`) calls `InlineFormat::render(node, InlineOutputFormat::Ooxml)`
   through the registry for every `InlineNode` it serializes. No production code path in
   `slideforge-pptx` performs `<a:r>` run construction outside of the `InlineFormat` trait
   dispatch.

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
| EC-004 | `slideforge-pptx` contains a private `serialize_inline_node()` function that generates `<a:r>` OOXML without going through the `InlineFormat` trait | This is an explicit contract violation of this BC. The function MUST be removed and replaced with a call to the registered `InlineFormat` implementation for `InlineOutputFormat::Ooxml`. CI adversarial review MUST flag any such function as a blocker. The existing internal inline serialization in `slideforge-pptx` at the time of STORY-085 is a known gap; STORY-085 closes it. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| All 10 bundled plugin implementations compile using public trait APIs only | Compilation succeeds; 0 clippy warnings about visibility violations | happy-path |
| Hypothetical test: plugin calls `slideforge_pptx::internal::LayoutHelper::compute()` (private) | Compilation error: private function; CI blocks merge | error |
| `slideforge-pptx` is compiled in isolation without access to `slideforge-pptx` internals, and an `InlineNode::Bold("hello")` is rendered via the `InlineFormat` trait (OOXML format) | Returns `Ok("<a:r><a:rPr b=\"1\"/><a:t>hello</a:t></a:r>")` or equivalent valid OOXML run markup; no call to any internal `slideforge-pptx` serialization function | happy-path (OOXML dog-fooding) |
| `grep -r "a:r\|a:rPr\|serialize_inline" crates/slideforge-pptx/src/` excluding the exporter's dispatch call site | Zero matches outside of the single dispatch call and its immediate OOXML assembly wrapper in the `InlineFormat` trait impl | architectural-invariant |

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

- `architecture/plugin-architecture.md` — dog-fooding guarantee specification

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
