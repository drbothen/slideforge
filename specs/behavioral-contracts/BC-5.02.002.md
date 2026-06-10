---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - version: "1.5"
    date: 2026-06-09
    author: product-owner
    reason: "STORY-081 / ADR-024 inline-run generator unification: Update Postcondition 5, EC-004, Description, and canonical test vectors to reflect the new unified engine. The dispatch mechanism is now render_inline_nodes_to_runs(nodes, hlink_resolver) + serialize_ooxml_run (both in slideforge-plugin-api/src/inline_formats/ooxml_runs.rs), replacing the per-node render_with_context loop and dispatch_inline_nodes_to_ooxml. The dog-fooding contract is MORE strongly satisfied: both the body path (via ooxml_run_to_ooxmlsdk) and the notes path (via serialize_ooxml_run) share one engine in slideforge-plugin-api — no bespoke per-consumer generator exists. Added ADR-024 cross-reference."
  - version: "1.4"
    date: 2026-06-05
    author: product-owner
    reason: "STORY-085 / ADR-017 Option A (human-authorized 2026-06-05): Clarify Postcondition 5 — (a) .rels relationship REGISTRATION is exporter-owned and distinct from run construction; (b) ALL <a:r> run construction including hyperlink runs flows through the InlineFormat trait dispatch (now including render_with_context); (c) the AC-005 grep-zero vector still holds with a single dispatch site. Update EC-004 to reflect that the OOXML bypass is the gap STORY-085 closes via render_with_context. No carve-out exception added — dog-fooding guarantee intact."
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
(`InlineNode` → OOXML `<a:r>` run markup) through internal structs or functions
private to `slideforge-pptx`. All inline serialization — including the OOXML path —
MUST flow through the unified engine `render_inline_nodes_to_runs` + `serialize_ooxml_run`
located in `slideforge-plugin-api/src/inline_formats/ooxml_runs.rs` (per ADR-024).
Both the slide-body path (which converts the resulting `Vec<OoxmlRun>` to typed
ooxmlsdk `Run` objects via `ooxml_run_to_ooxmlsdk`) and the PPTX notes path (which
serializes via `serialize_ooxml_run` directly) consume the same engine. There is no
separate bespoke run generator for either consumer. This satisfies the dog-fooding
guarantee more strongly than the previous per-node `render_with_context` dispatch:
the engine itself lives in `slideforge-plugin-api`, outside of `slideforge-pptx`.

## Preconditions

1. A bundled plugin is implemented (any of the 10 extensibility surfaces).
2. The plugin needs to interact with other pipeline components (e.g., exporter reads Brand, ChartRenderer uses data).

## Postconditions

1. All cross-component interactions (including inline serialization) occur via the declared
   trait APIs and shared engine functions in `slideforge-plugin-api`. No bundled plugin,
   including the PPTX exporter, serializes `InlineNode` values through internal functions
   private to `slideforge-pptx`. All `<a:r>` run markup flows through
   `render_inline_nodes_to_runs` → `serialize_ooxml_run` (notes path) or
   `render_inline_nodes_to_runs` → `ooxml_run_to_ooxmlsdk` (body path), both engine
   functions residing in `slideforge-plugin-api`.
2. No `pub(crate)` or `pub(super)` functions in other crates are called by plugin code.
3. The plugin compiles and passes tests using only the declared trait interfaces.
4. The plugin's trait implementation is architecturally identical to what a third-party plugin
   would write.
5. **Relationship registration is exporter-owned; run construction is engine-owned — no
   carve-out exception.**
   The PPTX exporter (`slideforge-pptx`) is responsible for two distinct operations on
   hyperlink `Link` nodes:
   - **Relationship registration (exporter-owned):** The exporter registers the link URL
     in the slide's `.rels` package part to obtain an `rId`. This is a PPTX packaging
     concern — the exporter owns the package state and is the correct site for this call.
     This step does NOT produce any `<a:r>` XML and is NOT subject to the unified engine.
   - **Run construction (engine-owned, per ADR-024):** ALL `<a:r>` OOXML run markup
     construction — including the `<a:rPr>` element and `<a:t>` text for Link nodes —
     MUST be produced by calling `render_inline_nodes_to_runs(nodes, hlink_resolver)`
     (defined in `slideforge-plugin-api/src/inline_formats/ooxml_runs.rs`), where
     `hlink_resolver` is a closure supplied by the consumer that maps a URL to its
     pre-registered `rId`. The body path converts the resulting `Vec<OoxmlRun>` to typed
     ooxmlsdk `Run` objects via `ooxml_run_to_ooxmlsdk`; the notes path serializes via
     `serialize_ooxml_run`. No production code path in `slideforge-pptx` constructs
     `<a:r>` / `<a:rPr>` markup outside of these two conversion functions (which
     themselves derive from `OoxmlRun` produced by the shared engine).
     Note: `render_with_context` (ADR-017 Option A) remains in the `InlineFormat` trait
     for forward compatibility, but it is NOT the dispatch mechanism for PPTX run
     construction after ADR-024. Neither the body path nor the notes path calls
     `render_with_context` for OOXML inline-run generation.
   The invariant holds: the dog-fooding guarantee is intact and strengthened. The split
   is registration (packaging concern, exporter-owned) vs. construction (rendering
   concern, engine-owned in `slideforge-plugin-api`). The AC-005 grep-zero vector
   (`a:r` / `a:rPr` / `serialize_inline` outside the two conversion functions) continues
   to hold — there is no bespoke per-consumer run generator.

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
| EC-004 | `slideforge-pptx` contains any internal function that generates `<a:r>` OOXML outside the unified engine (e.g., a private `serialize_inline_node()`, a bespoke per-node loop calling `render_with_context`, or `dispatch_inline_nodes_to_ooxml`) | This is an explicit contract violation of this BC. The function MUST be removed. The compliant mechanism (per ADR-024, closed by STORY-081) is: call `render_inline_nodes_to_runs(nodes, hlink_resolver)` from `slideforge-plugin-api/src/inline_formats/ooxml_runs.rs`, then convert the resulting `Vec<OoxmlRun>` via `ooxml_run_to_ooxmlsdk` (body path) or `serialize_ooxml_run` (notes path). For Link nodes, the `hlink_resolver` closure supplies the pre-registered `hyperlink_rid` from the `.rels` registration step (which remains exporter-owned and is not part of the engine). CI adversarial review MUST flag any remaining `<a:r>` construction outside the two conversion functions as a blocker. STORY-081 closes this gap. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| All 10 bundled plugin implementations compile using public trait APIs only | Compilation succeeds; 0 clippy warnings about visibility violations | happy-path |
| Hypothetical test: plugin calls `slideforge_pptx::internal::LayoutHelper::compute()` (private) | Compilation error: private function; CI blocks merge | error |
| `render_inline_nodes_to_runs([InlineNode::Bold([InlineNode::Plain("hello")])], &|_| None)` called from `slideforge-plugin-api` | Returns `Ok(vec![OoxmlRun { text: "hello", bold: true, .. }])`; `serialize_ooxml_run` produces `<a:r><a:rPr b="1"/><a:t>hello</a:t></a:r>`; no internal `slideforge-pptx` function is involved | happy-path (OOXML dog-fooding) |
| `grep -r "a:r\|a:rPr\|serialize_inline" crates/slideforge-pptx/src/` excluding the two conversion functions (`ooxml_run_to_ooxmlsdk` in `slide_serializer.rs` and `serialize_ooxml_run` calls in `notes_slide.rs`) | Zero matches outside of those two conversion functions — no bespoke per-consumer run generator remains | architectural-invariant (AC-005) |

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
- `architecture/adr/ADR-024-pptx-inline-run-generator-unification.md` — unified engine design: `render_inline_nodes_to_runs` + `OoxmlRun` + `serialize_ooxml_run` in `slideforge-plugin-api`; supersedes the `render_with_context` per-node dispatch loop for PPTX run construction

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
