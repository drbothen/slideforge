---
document_type: reconciliation-assessment
lesson: LESSON-13
story_id: STORY-049
title: "STORY-049 Plugin Registry Assembly — Upstream Availability & Spec Contradiction Analysis"
date: 2026-06-04
producer: architect
status: awaiting-human-decision
---

# STORY-049 Reconciliation Assessment (LESSON-13)

## Executive Summary

STORY-049 cannot be implemented as written. Five independently verifiable gaps exist between
the story spec and the current codebase. These are not ambiguities — they are binary facts
about code that does or does not exist. The story spec contains one internal contradiction
(forbidden-deps clause vs. build() requirement) that cannot be resolved without a human
scope decision.

The recommended path is a **targeted spec amendment + two prerequisite stories**, rather
than a large rewrite of STORY-049. The root crate scope itself is sound; only three
capability gaps and one ambiguity require resolution before implementation is possible.

---

## Ground-Truth Inventory

### What exists today (verified by grep):

| Artifact | Location | Status |
|----------|----------|--------|
| `PluginRegistry` with `register_*` / `lookup_*` API | `crates/slideforge-plugin-api/src/registry.rs:72–272` | EXISTS |
| 10 trait surfaces (trait definitions) | `crates/slideforge-plugin-api/src/traits/*.rs` | EXISTS |
| 31 `SlideType` plugin impls | `crates/slideforge-plugin-api/src/slide_types/` | EXISTS |
| `SlideTypeRegistry` with `register()` / `lookup_by_keyword()` | `crates/slideforge-plugin-api/src/slide_types/registry.rs` | EXISTS |
| DataSource bundled impls (×4) | `crates/slideforge-data/` | EXISTS |
| Exporter bundled impls (PPTX, DOCX, PDF) | `crates/slideforge-pptx/`, `slideforge-docx/`, `slideforge-pdf/` | EXISTS |
| ChartRenderer bundled impl | `crates/slideforge-charts/` | EXISTS |
| DiagramRenderer bundled impl | `crates/slideforge-diagrams/` | EXISTS |
| Validator bundled impls (×5) | `crates/slideforge-validate/` | EXISTS |
| MathRenderer bundled impl | `crates/slideforge-math/` | EXISTS |
| BrandProvider bundled impls (×2) | `crates/slideforge-brand/` | EXISTS |

### What does NOT exist today (verified by grep, confirmed against story ACs):

| Assumed-by-story Artifact | Checked Location | Status |
|--------------------------|-----------------|--------|
| `PluginRegistryBuilder` | All `*.rs` files | MISSING |
| `RegistryError::MissingSurface` | All `*.rs` files | MISSING |
| `surface_count()` on `PluginRegistry` | `registry.rs:72–272` | MISSING |
| `surface_names()` on `PluginRegistry` | `registry.rs:72–272` | MISSING |
| `build()` that fails on empty surfaces | `registry.rs:72–272` | MISSING |
| **Any production `SectionType` plugin impl** | All `*.rs` files | MISSING (stubs only: `StubSectionType` at `registry.rs:425`, `TestSectionType` at `dog_food_test.rs:175`) |
| **Any production `InlineFormat` plugin impl** | All `*.rs` files | MISSING (stubs only: `StubInlineFormat` at `registry.rs:436`, `TestInlineFormat` at `dog_food_test.rs:187`) |
| `BuildOptions` / `BuildOutput` / `BuildError` types | All `*.rs` files | MISSING |
| `catch_unwind` production dispatch boundary | All `*.rs` files (non-test) | MISSING (only in test: `bleed_tests.rs:418–604`; in `normalize.rs:805` inside `#[cfg(test)]` gate comment context) |

---

## Question A: Registry API Reconciliation

### The Conflict

STORY-049 ACs require (`STORY-049.md:119–143`):
- `PluginRegistry::builder()` returning a `PluginRegistryBuilder`
- `RegistryError::MissingSurface` returned from `builder().build()`
- `surface_count() -> usize` on `PluginRegistry`
- `surface_names() -> Vec<&str>` on `PluginRegistry`

The delivered `PluginRegistry` (`registry.rs:72–272`) provides:
- `register_*(&mut self, Box<dyn Trait>)` — push into a `Vec`
- `lookup_*(&self, id: &str) -> Option<&dyn Trait>` — linear scan
- `#[derive(Default)]` on the struct (empty is valid)
- No builder, no error return, no introspection methods

The existing API is from an already-merged, adversary-converged story (STORY-002).

### The Standing Rule

CLAUDE.md §Source-of-Truth Precedence states: "For code-vs-spec conflicts: the SPEC wins.
Code is brought into alignment via fix-burst or follow-up story, not the other way around."

This rule is stated without exception. However, the situation has an important wrinkle:
the spec in question (STORY-049) is a _story spec_, and the code in question was delivered
by _STORY-002_, which has its own story spec. Applying precedence strictly, STORY-049 is
the more recent document and wins. But STORY-002's spec also stated what `PluginRegistry`
should look like — and neither STORY-002 nor its BC (BC-5.02.001) mandated a Builder or
fail-on-missing behavior. STORY-049 introduced both as new requirements.

### Options

**Option A-i — Amend STORY-049 to match the existing API (story conforms to code)**

Change AC-001 to test `PluginRegistry::default()` using `lookup_*` calls to confirm
coverage rather than `surface_count() == 10`. Change AC-002 to test `lookup_*` returning
`None` for an empty registry rather than `RegistryError::MissingSurface`. Remove AC-004
(`surface_count` / `surface_names`). Remove the Builder from Tasks.

- Effort: amend STORY-049 spec only. 0 new stories. ~0.5 points of spec editing.
- Loss: surface introspection (`surface_count`, `surface_names`) and fail-on-empty-build
  semantics are genuinely useful for runtime initialization auditing.
- BC impact: **BC-5.02.001 invariant 3** says "unregistered surfaces cause an initialization
  error (not a silent no-op)" (`BC-5.02.001.md:68`). The current `PluginRegistry::new()` is
  silent — registering zero plugins raises no error. This is a BC violation in the EXISTING
  code, not just in STORY-049. Amending STORY-049 without fixing the existing code would
  leave a BC violation in production.

**Option A-ii — Extend `slideforge-plugin-api` to add Builder + RegistryError + introspection**

Add to `slideforge-plugin-api/src/registry.rs`:
- `PluginRegistryBuilder` with per-surface `register_*` builder methods
- `RegistryError::MissingSurface { surface: &'static str }`
- `PluginRegistryBuilder::build() -> Result<PluginRegistry, RegistryError>` — fails if any
  of the 10 surfaces has zero registrations
- `PluginRegistry::surface_count() -> usize`
- `PluginRegistry::surface_names() -> Vec<&'static str>`

- Effort: ~3 story-points in `slideforge-plugin-api` (a fix-burst or a new story).
  Does NOT touch the root crate; this is work in plugin-api.
- Gain: satisfies BC-5.02.001 invariant 3 properly; adds runtime auditability; the
  Builder pattern cleanly encodes "assembled registry vs empty working registry".
- Complication: STORY-049's Tasks (`STORY-049.md:187–199`) split the Builder between
  `slideforge/src/registry.rs` and `slideforge-plugin-api`. The Builder belongs entirely
  in `slideforge-plugin-api` (not root), which is the cleaner design.

### Recommendation: Option A-ii, but as a PREREQUISITE story, not in STORY-049 scope

The Builder + RegistryError + introspection belong in `slideforge-plugin-api`, not in the
root crate. Adding them to STORY-049 would violate STORY-049's own assembly-only constraint
(the root crate should not be extending plugin-api). Create `STORY-049-PRE-A`:
**"Extend PluginRegistryBuilder + RegistryError + surface introspection"** in
`slideforge-plugin-api`, ~3 points. STORY-049 then depends on it and calls
`PluginRegistryBuilder::default_bundled()` to assemble the registry.

BC impact flag: **BC-5.02.001 invariant 3 is currently violated by the existing
`PluginRegistry` in STORY-002's delivered code**. This requires a product-owner
acknowledgment. The BC says "unregistered surfaces cause an initialization error";
the current API is silently permissive. STORY-049-PRE-A closes the gap.

---

## Question B: SectionType + InlineFormat Gap

### The Confirmed Gap

No production implementation of `SectionType` or `InlineFormat` exists anywhere in the
codebase. The only implementations are:
- `StubSectionType` / `StubInlineFormat` in `slideforge-plugin-api/src/registry.rs:425–448`
  (inside `#[cfg(test)]`)
- `TestSectionType` / `TestInlineFormat` in `slideforge-plugin-api/tests/dog_food_test.rs:175–204`
  (test file)

No story in the wave schedule was explicitly assigned to deliver bundled plugin
implementations for these two surfaces. STORY-042 ("DOCX Auto-Generated Document Sections")
works _around_ the SectionType plugin — it uses `AutoSectionSerializer` as an internal
struct (`slideforge-docx/src/auto_sections.rs:46`) that dispatches on string-matching
section type names, bypassing the `SectionType` plugin trait entirely. This is an
architectural violation of BC-5.02.002 (dog-fooding guarantee), but it is STORY-042's
existing delivered behavior and is out of scope here.

STORY-049 AC-001 requires `PluginRegistry::default()` to register implementations of all
10 surfaces including `SectionType` and `InlineFormat` (`STORY-049.md:115–117`).

### The Correct Owning Crate

`plugin-architecture.md:32–33` assigns:
- `SectionType` owner: `slideforge-types`
- `InlineFormat` owner: `slideforge-types`

This assignment is correct per the purity-boundary-map. `slideforge-types` is a pure-core
leaf crate (no workspace dependencies). A bundled `SectionType` impl would implement
`generate(&[Slide]) -> Vec<SectionBlock>` — pure logic on IR types, no I/O. A bundled
`InlineFormat` impl would implement `render(&InlineNode, InlineOutputFormat) -> Result<String,
InlineError>` — pure serialization, no I/O. Both belong in `slideforge-types` and are
Kani-amenable.

However, `slideforge-types` currently has ZERO dependency on `slideforge-plugin-api`
(it is a leaf crate). Adding a bundled trait implementation would require `slideforge-types`
to depend on `slideforge-plugin-api` for the trait definitions. This creates a
dependency loop if `slideforge-plugin-api` depends on `slideforge-types` (which it does —
see `slideforge-plugin-api/src/traits/section_type.rs:10: use slideforge_types::Slide`).

**The loop:** `slideforge-types` → `slideforge-plugin-api` → `slideforge-types` = CYCLE.
This is architecturally blocked.

### Options

**Option B-i — Create bundled impls inside STORY-049 scope (rejected)**

Adding a new `TocSectionType` struct and a `DefaultInlineFormat` struct inside
`crates/slideforge/src/` violates the assembly-only rule stated in `STORY-049.md:214`:
"No parsing, evaluation, layout, or export logic." Inline format rendering is logic.
Section generation is logic. This option is architecturally non-compliant.

**Option B-ii — Create bundled impls in `slideforge-plugin-api` (correct)**

The trait definitions live in `slideforge-plugin-api`. The bundled impls for surfaces that
cannot live in `slideforge-types` (due to the cycle above) should live alongside their
trait definitions, in `slideforge-plugin-api` itself. This mirrors what `SlideType` already
does: the 31 built-in SlideType impls live in `slideforge-plugin-api/src/slide_types/`
(`crates/slideforge-plugin-api/src/slide_types/*.rs`, 4,191 lines total). Following this
exact pattern:
- `slideforge-plugin-api/src/section_types/` — bundled `TocSectionType`, `ChapterSectionType`,
  `AppendixSectionType` implementing `SectionType`
- `slideforge-plugin-api/src/inline_formats/` — `DefaultInlineFormat` implementing
  `InlineFormat` for all 12 `InlineNode` variants × 3 output formats (OOXML/HTML/Markdown)

This introduces no cycle. `slideforge-plugin-api` depends on `slideforge-types` already
(`traits/section_type.rs:10`, `traits/inline_format.rs:14`). Adding implementations
alongside the traits is clean.

**Option B-iii — Register only 8 ready surfaces; defer SectionType + InlineFormat**

Relax AC-001's "all 10" to "all 8 surfaces with bundled implementations; SectionType and
InlineFormat deferred to STORY-NNN". This would require a BC amendment because
BC-5.02.001 postcondition 2 explicitly lists "SectionType: all ~15 auto-generated section
types" and "InlineFormat: all 11 inline formatting types" as required.

A BC amendment requires a product-owner action (the BC is the product-owner's artifact).
This is the longest path and produces a spec with known holes entering Wave 4's
integration gate.

### Recommendation: Option B-ii, as two separate PREREQUISITE stories

Create:
- `STORY-049-PRE-B1`: **"Bundled SectionType plugin implementations"** in
  `slideforge-plugin-api` (new `src/section_types/` module). Implement at minimum:
  `TocSectionType` (generates TOC entries from slide sequence), `ChapterSectionType`
  (groups slides by title-slide boundaries), `AppendixSectionType` (marks trailing slides).
  ~3 points.
- `STORY-049-PRE-B2`: **"Bundled DefaultInlineFormat plugin implementation"** in
  `slideforge-plugin-api` (new `src/inline_formats/` module). Implement `DefaultInlineFormat`
  handling all 12 `InlineNode` variants for OOXML, HTML, and Markdown output formats.
  ~5 points.

STORY-049 then depends on both and registers them from `slideforge-plugin-api`.

Note: `DefaultInlineFormat` for OOXML is non-trivial — it must emit valid `<a:r>` run
markup for PPTX/DOCX. The existing PPTX exporter handles inline nodes internally (in
`slideforge-pptx`). Extracting that into a plugin impl is real work. 5 points is
conservative; budget 8 if the OOXML path requires significant refactoring.

**BC flag:** BC-5.02.001 postcondition 2 (`BC-5.02.001.md:56–57`) requires these bundled
impls to exist. The BC is correct; the gap is that no story was ever assigned to create
them. This is a planning gap, not a BC error. No BC amendment required; only new stories.

---

## Question C: build() Pipeline Wiring vs. Forbidden-Deps Contradiction

### The Contradiction

The spec says two contradictory things in the same document:

1. `STORY-049.md:278–283` (Forbidden Dependencies): "The root `slideforge` crate MUST NOT
   depend on any crate that is not a direct plugin implementation. In particular: No
   `slideforge-syntax`, `slideforge-eval`, `slideforge-layout`, `slideforge-validate` as
   direct dependencies."

2. `STORY-049.md:83–84` + `STORY-049.md:184` (Summary + Tasks): "Exposes the public library
   API: `pub fn build(source: &str, options: BuildOptions) -> Result<BuildOutput, BuildError>`"

A `build()` function that takes `source: &str` and returns a rendered `BuildOutput` must
invoke the parse → eval → validate → layout → export pipeline. That pipeline lives in
`slideforge-syntax`, `slideforge-eval`, `slideforge-layout`, and the exporter crates.
The root crate **cannot implement build() without depending on those crates**.

The existing `crates/slideforge/Cargo.toml` already has `slideforge-syntax`, `slideforge-eval`,
`slideforge-layout`, `slideforge-pptx`, `slideforge-validate` as dependencies
(`crates/slideforge/Cargo.toml:17–23`), meaning the scaffolding was created with the
pipeline-driver interpretation in mind — and contradicts the forbidden-deps clause in its
own story spec.

### What the Architecture Actually Says

`ARCH-INDEX.md:71–74`:
> "the root `slideforge` crate has no SS-ID because it is a thin re-export facade that
> assembles the plugin registry from the subsystem crates and exposes the public library
> API. No code lives in the root crate beyond registry construction."

`system-overview.md:44` (purity table):
> "| slideforge | Effectful shell | Registry assembly, pipeline orchestration — effectful |"

`plugin-architecture.md:37–50` (code example):
```rust
// slideforge/src/registry.rs
pub fn default_registry() -> PluginRegistry {
    let mut r = PluginRegistry::new();
    r.register_data_source(Box::new(JsonDataSource));
    r.register_exporter(Box::new(PptxExporter));
    // ...
    r
}
```

The architecture documentation uses the word "orchestration" and "pipeline orchestration"
to describe the root crate's role. It also says "exposes the public library API." But it
says "No code lives in the root crate beyond registry construction."

These are contradictory in the architecture documents themselves, not just in the story spec.

### The Two Interpretations

**Interpretation C-1 — Root crate is ASSEMBLY-ONLY (thin facade)**

- Root crate: `pub use slideforge_plugin_api::PluginRegistry; pub fn default_registry() -> PluginRegistry`
- No `build()` in root crate
- `build()` lives in `slideforge-cli` (the CLI orchestrator)
- Root crate's Cargo.toml: remove `slideforge-syntax`, `slideforge-eval`, `slideforge-layout`,
  `slideforge-validate`; keep only plugin implementation crates as deps
- Library API surface: consumers get a `PluginRegistry`; they call pipeline stages directly

Under this interpretation, STORY-049 is strictly an assembly story. The `build()` public API
is a future story, assigned to either `slideforge-cli` or a new `slideforge-pipeline` crate.

**Interpretation C-2 — Root crate is PIPELINE DRIVER (assembly + convenience API)**

- Root crate: assembles registry AND exposes `build(source, opts) -> Result<Output, Error>`
- Depends on all pipeline crates (syntax, eval, layout, validate) — same as current Cargo.toml
- Root crate is a convenience layer so library consumers don't need to assemble the pipeline
- Forbidden-deps clause in STORY-049 is wrong and should be deleted
- `slideforge-cli` depends on `slideforge` (root) and calls `slideforge::build()`

This matches the existing `crates/slideforge/Cargo.toml` dependency list and the
`system-overview.md` "pipeline orchestration" language.

### What the Architecture Authorizes

The architecture documents support Interpretation C-2 more strongly:
- `system-overview.md` explicitly says "pipeline orchestration" as the root crate's role
- `crate-architecture.md:38`: "slideforge/ — Root library — assembles PluginRegistry — EFFECTFUL"
- The existing `Cargo.toml` was created with full pipeline deps
- `slideforge-cli/Cargo.toml` depends on `slideforge = { workspace = true }` — meaning CLI
  was always intended to delegate to the root crate

However, the architecture also says "No code lives in the root crate beyond registry
construction" — which would exclude `build()`. This is an architecture document
self-contradiction.

### Recommendation: Interpretation C-2 with explicit spec amendment

Delete the Forbidden Dependencies clause from STORY-049 as written. Replace with:

> "The root crate MAY depend on pipeline crates (`slideforge-syntax`, `slideforge-eval`,
> `slideforge-layout`, `slideforge-validate`) to expose `build()`. It MUST NOT depend
> on individual plugin implementation crates for any purpose other than assembly —
> no direct calls to `PptxExporter::internal_method()`, only `Box<dyn Exporter>` dispatch."

The `build()` function belongs in the root crate as the library API entry point. This is
confirmed by `slideforge-cli/Cargo.toml` depending on `slideforge` (the root) and CLI's
stub main calling nothing (yet). The pipeline wiring is the root crate's job; the CLI
transforms user input (paths, flags) into `BuildOptions` and calls `slideforge::build()`.

The architecture-level self-contradiction ("pipeline orchestration" vs. "No code beyond
registry construction") should be resolved in `crate-architecture.md` by the architect —
specifically, replacing "No code lives in the root crate beyond registry construction" with
"No _plugin logic_ lives in the root crate; pipeline wiring via `build()` is permitted."
This is a **minor architecture doc fix**, not a redesign.

---

## Question D: catch_unwind / PluginPanic Boundary (AC-008)

### The Situation

AC-008 (`STORY-049.md:169–175`) requires:
> "If a registered plugin implementation panics during execution, the panic is caught at
> the plugin dispatch boundary (using `std::panic::catch_unwind`) and returned as
> `Err(PluginError::PluginPanic { plugin_name, message })`."

The story's Architecture Compliance Rules say:
> "This is the ONLY use of `catch_unwind` in the codebase — it is the plugin dispatch
> boundary." (`STORY-049.md:221`)

The verified state of the codebase:
- `catch_unwind` is used in `slideforge-diagrams/src/normalize.rs:805` inside a test guard,
  and in `slideforge-eval/tests/bleed_tests.rs:418–604` in test code.
- No production `catch_unwind` or `PluginError::PluginPanic` exists.

### Where This Belongs

**Option D-1 — In the root crate (STORY-049 scope)**

The root crate creates a dispatch wrapper in `src/dispatch.rs` that calls plugin trait
methods inside `catch_unwind`. Any call site in `build()` that invokes a plugin method
goes through this wrapper. `PluginError` is defined in `src/error.rs`.

This is architecturally sound: the root crate is the only caller that dispatches plugins
at runtime (the subsystem crates receive typed plugin implementations directly). The
panic boundary is exactly at the boundary between the application and its plugins.

**Option D-2 — In `slideforge-plugin-api`**

The registry's `lookup_*` methods could return a wrapper type that intercepts calls via
`catch_unwind`. This is more invasive — it changes `lookup_*` return types from
`Option<&dyn Trait>` to a custom dispatch wrapper — and affects STORY-002's delivered API.

Option D-2 is heavier-weight and affects the already-merged STORY-002 registry API.

### Recommendation: AC-008 is correctly scoped to STORY-049, in the root crate

The dispatch wrapper (`crates/slideforge/src/dispatch.rs`) is root-crate-only code with
no public surface. It wraps calls to plugin trait methods inside `build()`. `PluginError`
is a root-crate error type (`crates/slideforge/src/error.rs`). This is clean and does not
require amending the plugin-api registry.

One clarification: STORY-049.md:221 says "ONLY use of `catch_unwind` in the codebase" —
this is already incorrect given the test-code usage in `slideforge-diagrams` and
`slideforge-eval`. The compliance rule should say "only PRODUCTION use of `catch_unwind`".
Amend the story spec accordingly before implementation.

---

## Question E: Net Recommendation

### Diagnosis Summary

| Gap | Severity | Path |
|-----|----------|------|
| A. Registry API missing Builder/introspection | HIGH — BC-5.02.001 inv.3 is violated today | Prerequisite story in `slideforge-plugin-api` |
| B1. No production `SectionType` bundled impl | HIGH — AC-001 fails | Prerequisite story in `slideforge-plugin-api` |
| B2. No production `InlineFormat` bundled impl | HIGH — AC-001 fails | Prerequisite story in `slideforge-plugin-api` |
| C. Forbidden-deps vs. build() contradiction | MEDIUM — internal spec contradiction | Spec amendment; delete bad clause |
| D. catch_unwind scope wording | LOW — minor wording error in spec | Spec amendment; one-line fix |

### Recommended Story Map

```
STORY-049-PRE-A  (3 pts, slideforge-plugin-api)
  └── PluginRegistryBuilder + RegistryError::MissingSurface + surface_count/surface_names
       Closes BC-5.02.001 invariant 3 violation in delivered STORY-002 code.
       All changes in slideforge-plugin-api/src/registry.rs.

STORY-049-PRE-B1  (3 pts, slideforge-plugin-api)
  └── Bundled SectionType plugin implementations (TocSectionType, ChapterSectionType,
       AppendixSectionType) in new slideforge-plugin-api/src/section_types/ module.

STORY-049-PRE-B2  (5 pts, slideforge-plugin-api)
  └── Bundled DefaultInlineFormat plugin implementation in new
       slideforge-plugin-api/src/inline_formats/ module.
       All 12 InlineNode variants × 3 output formats (OOXML, HTML, Markdown).
       Note: OOXML path may require refactoring the inline serialization
       already inside slideforge-pptx. Budget 8 points if that path is complex.

STORY-049 (amended, 5 pts, slideforge root crate)
  └── Depends on: STORY-049-PRE-A, STORY-049-PRE-B1, STORY-049-PRE-B2
      Spec changes needed BEFORE implementation:
      1. Delete Forbidden Dependencies clause; replace with correct formulation (see §C)
      2. AC-001: change test assertion from surface_count()==10 to use PluginRegistryBuilder
      3. AC-002: restate using RegistryError::MissingSurface from PRE-A
      4. AC-003/Tasks: BuildOptions/BuildOutput/BuildError defined in this story (no change)
      5. AC-008 compliance rule: change "ONLY use" to "only PRODUCTION use" (wording fix)
```

### Expand vs. Split Decision

**Recommendation: SPLIT — insert three prerequisite stories before STORY-049**

Rationale: The root of every gap is that `slideforge-plugin-api` is incomplete — it delivered
trait definitions and a registry container (STORY-002) but was never given a story to deliver
the Builder pattern, the initialization error semantics, or the two missing bundled
implementations. These are all `slideforge-plugin-api` scope, not root-crate scope.
STORY-049's original 5-point estimate assumed all prerequisites were in place; with three
prerequisite stories added (3 + 3 + 5 = 11 new points), STORY-049 itself stays at 5 points
but moves to a later slot in Wave 4 after PRE-A/B1/B2 pass. Total wave cost is +11 points.

Expanding STORY-049 to absorb all the work (pushing it to ~16 points) is possible but
violates the production-grade principle (Rule 2): "it is NOT acceptable to ship the current
story partially or with shortcuts." A 16-point story is too large for clean TDD delivery and
would conflate plugin-api extension work with assembly work, making adversarial review harder.

The split is also architecturally honest: PRE-A/B1/B2 are plugin-api stories owned by the
same crate as STORY-002, and they fill genuinely missing capability that BC-5.02.001
postcondition 2 already requires but no story was assigned to deliver.

### Required Human Decisions Before Stories Are Created

1. **BC-5.02.001 invariant 3 product-owner sign-off**: The BC says "unregistered surfaces
   cause an initialization error." The delivered STORY-002 code does not do this. The
   architect recommends Option A-ii (add Builder + RegistryError). The product-owner must
   confirm the BC intent is enforced at runtime and not weakened. If the PO wants to weaken
   invariant 3 (allow empty registry without error), the BC must be amended before
   STORY-049 proceeds.

2. **Root crate as pipeline driver vs. assembly-only**: The architect recommends Interpretation
   C-2 (root crate exposes `build()`; keep pipeline deps). The human must confirm this is
   the intended design for the public library API. If the human prefers C-1 (root crate is
   assembly-only; `build()` lives in CLI), then STORY-049 loses the `build()` requirement
   and the CLI story (STORY-055) gains it, and the root crate's Cargo.toml dependencies
   must be slimmed to plugin-impl crates only.

3. **InlineFormat OOXML refactoring scope**: If `DefaultInlineFormat` for OOXML requires
   extracting inline serialization already inside `slideforge-pptx`, the human must decide
   whether to (a) refactor `slideforge-pptx` as part of STORY-049-PRE-B2 (correct but
   expensive), or (b) have `DefaultInlineFormat` skip OOXML initially (inconsistent with
   BC-5.02.001) and add a follow-up. Option (a) is required by the production-grade default.

---

## Architecture Document Corrections Required

| Document | Change | Severity |
|----------|--------|----------|
| `crate-architecture.md:38` | Change "assembles PluginRegistry" to "assembles PluginRegistry and exposes public build() API" | Minor |
| `crate-architecture.md:38` (doc comment below)| Delete "No code lives in the root crate beyond registry construction" or qualify: "No plugin logic lives in the root crate; pipeline wiring is permitted." | Minor |
| `plugin-architecture.md:23` table | Add `section_types/` and `inline_formats/` modules to `slideforge-plugin-api` owner column rows for SectionType and InlineFormat | Minor |

These are architect-scope amendments. They do not require BC changes.

---

## File:Line Citation Index

| Claim | Source |
|-------|--------|
| `PluginRegistry` has no Builder/introspection | `crates/slideforge-plugin-api/src/registry.rs:72–272` |
| Only stub SectionType impls exist | `crates/slideforge-plugin-api/src/registry.rs:425–433` |
| Only test SectionType impls exist | `crates/slideforge-plugin-api/tests/dog_food_test.rs:175–185` |
| Only stub InlineFormat impls exist | `crates/slideforge-plugin-api/src/registry.rs:436–447` |
| Only test InlineFormat impls exist | `crates/slideforge-plugin-api/tests/dog_food_test.rs:187–204` |
| No BuildOptions/BuildOutput/BuildError exist | confirmed by grep across all `*.rs` |
| catch_unwind only in tests/diagrams normalize | `crates/slideforge-diagrams/src/normalize.rs:805`, `crates/slideforge-eval/tests/bleed_tests.rs:418` |
| Root crate existing deps include syntax/eval/layout | `crates/slideforge/Cargo.toml:17–23` |
| Root crate doc says "pipeline orchestration" | `crates/slideforge-plugin-api/src/slide_types/mod.rs` + `system-overview.md:44` |
| ARCH-INDEX: root crate "thin re-export facade" | `.factory/specs/architecture/ARCH-INDEX.md:71–74` |
| Forbidden deps clause | `STORY-049.md:278–283` |
| build() task requirement | `STORY-049.md:184` |
| BC invariant 3 "unregistered surfaces cause initialization error" | `.factory/specs/behavioral-contracts/BC-5.02.001.md:68` |
| SectionType owner: slideforge-types | `.factory/specs/architecture/plugin-architecture.md:32` |
| InlineFormat owner: slideforge-types | `.factory/specs/architecture/plugin-architecture.md:33` |
| slideforge-plugin-api depends on slideforge-types | `crates/slideforge-plugin-api/src/traits/section_type.rs:10` |
| 31 SlideType impls live in plugin-api | `crates/slideforge-plugin-api/src/slide_types/*.rs` |
| ADR-006: registry assembled in slideforge/src/registry.rs | `.factory/specs/architecture/adr/ADR-006-plugin-first-architecture.md:24` |
| slideforge-cli deps on slideforge root | `crates/slideforge-cli/Cargo.toml:17` |
| STORY-049 "assembly-only" rule | `STORY-049.md:214` |
| Two SectionBlock name collision | `crates/slideforge-plugin-api/src/traits/section_type.rs:20` and `crates/slideforge-types/src/deck.rs:72` |
