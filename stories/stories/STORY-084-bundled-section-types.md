---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-084
title: "Bundled SectionType Implementations (executive_summary, risk_register, + 5 manual types)"
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
  - STORY-042
  - STORY-077
blocks:
  - STORY-049
estimated_days: 1
---

# STORY-084: Bundled SectionType Implementations

## Subsystem Anchor Justification

SS-14 (Plugin API) owns this story. Per ADR-016 Decision 2, `slideforge-plugin-api` is
the correct home for bundled `SectionType` implementations — mirroring the existing pattern
for `SlideType` (31 implementations in `slideforge-plugin-api/src/slide_types/`). The
`slideforge-types` crate cannot own these implementations because it is a leaf crate with
zero workspace dependencies, and implementing a plugin trait requires depending on
`slideforge-plugin-api` — which already depends on `slideforge-types`. That cycle is
architecturally blocked. `slideforge-plugin-api` is the only dependency-legal owner.

## Dependency Anchor Justifications

- Depends on STORY-002: STORY-002 delivered the `SectionType` trait definition in
  `slideforge-plugin-api/src/traits/section_type.rs` and the `SectionBlock` struct used
  by `generate()`. This story writes concrete implementations of that trait.
- Depends on STORY-042 (DOCX Auto-Generated Document Sections): STORY-042 established the
  auto-section semantics for DOCX — specifically which slide types contribute to
  `executive_summary` and `risk_register` sections. The bundled SectionType impls must be
  consistent with STORY-042's auto-generation rules so the plugin output matches what DOCX
  consumers expect. NOTE: STORY-042's `AutoSectionSerializer` currently bypasses the plugin
  trait (an acknowledged STORY-042 violation of BC-5.02.002). STORY-084 provides the
  production plugin impls; routing DOCX through them is STORY-085's scope.
- Depends on STORY-077 (SectionBlock IR Extension): STORY-077 extended `SectionBlock` in
  `slideforge-types` to carry `FieldValue` body content. The `SectionType::generate()`
  trait method returns `Vec<SectionBlock>` — the impls here must produce `SectionBlock`s
  with the correct fields as defined by STORY-077's extended IR.
- Blocks STORY-049: STORY-049 (Plugin Registry Assembly) calls `register_section_type()`
  for all bundled `SectionType` impls. Without STORY-084, the `SectionType` surface has
  only stub implementations (`StubSectionType` in `#[cfg(test)]`), and AC-001 of STORY-049
  ("all 10 surfaces registered") fails.

## Summary

Create production `SectionType` plugin implementations in a new module
`slideforge-plugin-api/src/section_types/`, following the exact pattern of
`slideforge-plugin-api/src/slide_types/`.

The canonical set of section types is defined in `slideforge-types/src/deck.rs` as
`CANONICAL_MANUAL_SECTION_TYPES` and in BC-3.02.002 invariant 3 and BC-3.02.001 EC-002:

| Section Type | Auto-Generated? | Source |
|---|---|---|
| `executive_summary` | Yes (from `takeaway:` fields) | BC-3.02.001, deck.rs:25 |
| `risk_register` | Yes (from `severity_cards` slides) | BC-3.02.001, deck.rs:26 |
| `methodology` | No | BC-3.02.002, deck.rs:27 |
| `scope` | No | BC-3.02.002, deck.rs:28 |
| `approval` | No | BC-3.02.002, deck.rs:29 |
| `appendix` | No | BC-3.02.002, deck.rs:30 |
| `glossary` | No | BC-3.02.002, deck.rs:31 |

The two auto-generated types (`executive_summary`, `risk_register`) implement
`SectionType::generate(&[Slide])` by scanning slides for the relevant slide types.
The five manual-only types implement `generate()` returning an empty `Vec<SectionBlock>`
(they are document-authoring constructs, not derived from slide data — their content
comes from `section <type>:` DSL blocks, not from slide scanning).

All 7 implementations are registered in the `PluginRegistryBuilder` by STORY-049.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-5.02.001 | All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API | AC-001 through AC-004 |

## Acceptance Criteria

### AC-001: All 7 bundled SectionType impls exist in slideforge-plugin-api/src/section_types/
(traces to BC-5.02.001 postcondition 2 — "SectionType: all ~15 auto-generated section types")

The module `slideforge-plugin-api/src/section_types/` contains concrete implementations
of `SectionType` for: `executive_summary`, `risk_register`, `methodology`, `scope`,
`approval`, `appendix`, and `glossary`. Each struct implements `SectionType` using only
`slideforge-plugin-api`'s public trait API — no imports from crate-private modules.

### AC-002: ExecutiveSummarySectionType::generate() scans slides for takeaway fields
(traces to BC-5.02.001 postcondition 3)

`ExecutiveSummarySectionType::generate(slides)` produces one `SectionBlock` per slide
that has a `takeaway` field (i.e., slides where the slide IR has a `takeaway` entry in
its fields map). The `SectionBlock.title` is derived from the slide title; `level` is 1;
`include_in_toc` is `true`. Slides without a `takeaway` field are skipped.

Unit test: deck with 3 slides — 2 have `takeaway:` fields, 1 does not → `generate()`
returns 2 `SectionBlock`s.

### AC-003: RiskRegisterSectionType::generate() scans slides for severity_cards type
(traces to BC-5.02.001 postcondition 3)

`RiskRegisterSectionType::generate(slides)` produces one `SectionBlock` per slide
whose `slide_type` field matches the canonical `severity_cards` slide type (from
`slideforge-types` slide type definitions). `SectionBlock.title` is the slide title;
`level` is 1; `include_in_toc` is `true`.

Unit test: deck with 5 slides — 2 are `severity_cards`, 3 are not → returns 2 blocks.

### AC-004: Manual-only SectionType impls (methodology, scope, approval, appendix, glossary) return empty Vec
(traces to BC-5.02.001 postcondition 3)

Each of the 5 manual-only section types implements `generate(slides) -> Vec<SectionBlock>`
returning `vec![]`. These types generate no structural output from slide data — their
content comes exclusively from `section <type>:` DSL blocks, which are authored manually
and processed by the evaluator, not by the plugin's `generate()` call.

Unit test for each: `generate(any_slides)` → `vec![]`.

### AC-005: All 7 impls compile using only slideforge-plugin-api public API
(traces to BC-5.02.001 postcondition 3 — dog-fooding)

No implementation imports `slideforge_types::deck::CANONICAL_MANUAL_SECTION_TYPES` via
a private or `pub(crate)` path, and no implementation imports internals from other
slideforge crates. The only import is `use slideforge_plugin_api::traits::section_type::{SectionType, SectionBlock}`.

Verified by: `cargo build -p slideforge-plugin-api` compiles without warnings; `cargo
tree -p slideforge-plugin-api` shows no new crate dependencies added by this story.

## Tasks

- [ ] Create `crates/slideforge-plugin-api/src/section_types/mod.rs`:
  - Declare submodules for each type
  - Re-export all 7 structs as public from the module
- [ ] Create `crates/slideforge-plugin-api/src/section_types/executive_summary.rs`:
  - `ExecutiveSummarySectionType` struct (unit struct or with config)
  - `id()` returns `"executive_summary"`
  - `generate()` scans `slides` for those with a `takeaway` field (check `Slide` field
    access — use the public API on the `Slide` type from `slideforge-types`)
- [ ] Create `crates/slideforge-plugin-api/src/section_types/risk_register.rs`:
  - `RiskRegisterSectionType` struct
  - `id()` returns `"risk_register"`
  - `generate()` scans `slides` for those whose type is `severity_cards`
- [ ] Create `crates/slideforge-plugin-api/src/section_types/{methodology, scope, approval, appendix, glossary}.rs`:
  - Each is a unit struct implementing `SectionType`
  - `id()` returns the lowercase section type name string
  - `generate()` always returns `vec![]`
  - These 5 can be in a single file `manual_types.rs` to avoid 5 trivial files
- [ ] Add `pub mod section_types;` to `crates/slideforge-plugin-api/src/lib.rs`
- [ ] Export `section_types::*` structs from `lib.rs` as public API
- [ ] Write unit tests:
  - `ExecutiveSummarySectionType::generate()` with and without takeaway slides
  - `RiskRegisterSectionType::generate()` with severity_cards slides
  - All 5 manual types return `vec![]` for any slide slice

## Previous Story Intelligence

N/A — first story in `section_types/` module. Pattern: mirror `src/slide_types/`
structure exactly. `slide_types/mod.rs` declares submodules; each type is its own file
or a logical group. The `SectionType` trait is simpler than `SlideType` (only one method:
`generate`), so implementations are shorter.

Key note from reconciliation assessment (LESSON-13): `STORY-042` already delivered
auto-section behavior via `AutoSectionSerializer`, which bypasses the plugin trait.
STORY-084 creates the plugin-correct versions. STORY-085 is responsible for routing DOCX
through them (the BC-5.02.002 dog-fooding refactor). Do NOT refactor STORY-042 code
in this story — that is STORY-085 scope.

## Architecture Compliance Rules

1. **Module placement: `src/section_types/`, NOT `src/traits/`.** Trait definitions live
   in `src/traits/section_type.rs`; bundled impls live in `src/section_types/`. This mirrors
   the SlideType pattern (`traits/slide_type.rs` vs `slide_types/`).
2. **Only slideforge-plugin-api public API.** The `generate()` implementations access `Slide`
   fields via the public `slideforge_types::Slide` API (or via `slideforge_plugin_api`'s
   re-export of `slideforge_types`). No `pub(crate)` or private field access.
3. **No new crate dependencies.** `slideforge-plugin-api` already depends on `slideforge-types`.
   No additional workspace deps may be added.
4. **`#![forbid(unsafe_code)]`** already in force; no unsafe additions.
5. **ADR-016 Decision 2 governs.** Owner is `slideforge-plugin-api/src/section_types/`.
   `slideforge-types` remains a leaf with zero workspace deps.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` | workspace (already dep) | `Slide`, `SectionBlock` types |
| `slideforge-plugin-api` | workspace | `SectionType` trait (self-reference within crate) |

No new library dependencies required.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-plugin-api/src/section_types/mod.rs` | Create | Module root, re-exports |
| `crates/slideforge-plugin-api/src/section_types/executive_summary.rs` | Create | ExecutiveSummarySectionType |
| `crates/slideforge-plugin-api/src/section_types/risk_register.rs` | Create | RiskRegisterSectionType |
| `crates/slideforge-plugin-api/src/section_types/manual_types.rs` | Create | 5 manual-only impls |
| `crates/slideforge-plugin-api/src/lib.rs` | Modify | Add `pub mod section_types;` + re-exports |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,200 |
| BC-5.02.001 (postcondition 2-3) | ~800 |
| ADR-016 Decision 2 | ~400 |
| `SectionType` trait + `SectionBlock` (section_type.rs) | ~1,500 |
| `Slide` struct public API (for generate() scanning) | ~800 |
| Test code to write | ~600 |
| **Total** | **~6,300** |

Context budget: ~6% of a 100k-token context window. Within limit.

## Test Strategy

- **Unit tests in each implementation file's `#[cfg(test)] mod tests`** block:
  - `ExecutiveSummarySectionType`: 3 test cases (no takeaway slides, all takeaway slides,
    mixed; verify count and `SectionBlock` field values)
  - `RiskRegisterSectionType`: 3 test cases (no severity_cards, some, all)
  - Each of the 5 manual types: 1 test case each — `generate(any)` → `vec![]`
- **Compilation test**: `cargo build -p slideforge-plugin-api` succeeds with no new warnings

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `generate()` called with empty slide slice | Returns `vec![]` (auto-gen types have nothing to scan; manual types already return `vec![]`) |
| EC-002 | Slide has both a `takeaway:` field AND is a `severity_cards` type | Both `ExecutiveSummarySectionType` and `RiskRegisterSectionType` include it (each plugin is independent) |
| EC-003 | New 8th section type added to CANONICAL_MANUAL_SECTION_TYPES | New type needs its own impl in this module + registration in STORY-049 |

## Forbidden Dependencies

`slideforge-plugin-api/src/section_types/` MUST NOT:
- Import from `slideforge-docx`, `slideforge-pptx`, `slideforge-pdf`, or any exporter crate
- Import from `slideforge-eval`, `slideforge-syntax`, `slideforge-layout`
- Import `pub(crate)` or private symbols from `slideforge-types`
- Add any new workspace crate to `slideforge-plugin-api/Cargo.toml`

Build-time enforcement: `cargo build -p slideforge-plugin-api --all-features` must
succeed with zero clippy warnings. Any import from a forbidden crate produces a compile
error (the forbidden crates are not in `slideforge-plugin-api`'s dep tree).
