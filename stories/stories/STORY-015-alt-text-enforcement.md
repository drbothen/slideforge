---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-015
title: "Alt Text Enforcement"
epic: EPIC-04
wave: 2
points: 5
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-validate
subsystems: [SS-03]
target_module: slideforge-validate
behavioral_contracts:
  - BC-5.01.001
  - BC-5.01.002
verification_properties: [VP-002, VP-008]
nfr_refs: [NFR-014, NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-001
  - STORY-002
  - STORY-010
blocks:
  - STORY-016
  - STORY-017
  - STORY-037
  - STORY-039
  - STORY-043
  - STORY-045
  - STORY-046
  - STORY-068
estimated_days: 2
---

# STORY-015: Alt Text Enforcement

## Summary

Implement compile-time alt text enforcement in `slideforge-validate`. The validator
scans the evaluated `Deck` IR (from `slideforge-eval`) for visual elements (images,
charts, diagrams, and shapes with visual content) that are missing `alt "..."` text.
Missing alt text produces E-A11-001 (a compile error in strict mode). The `decorative: true`
opt-out marks elements as PDF Artifacts / empty-alt in all output formats. This is an
accessibility invariant (DI-001) — no output format can contain a visual element with
neither alt text nor decorative marker.

The validator runs after evaluation and before layout/export. It consumes the `Deck` IR
and emits diagnostics through `DiagnosticSink`. The `Validator` trait (STORY-002) is
implemented by `AltTextValidator` in this story.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-5.01.001 | Missing alt on visual element is compile error with element location | All 4 postconditions + 4 invariants |
| BC-5.01.002 | decorative: true opts out of alt requirement; emits empty alt in all formats | All 5 postconditions + 3 invariants |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,000 |
| BC files (2 BCs) | ~2,000 |
| STORY-001 IR types reference | ~1,500 |
| STORY-002 Validator trait reference | ~1,000 |
| STORY-010 DiagnosticSink reference | ~500 |
| Target source files to write | ~4,000 |
| Test files | ~3,000 |
| **Total** | **~16,000** |

Agent context budget: 200k tokens. This story is ~8% of budget — within limit.

## Acceptance Criteria

- [ ] **AC-001** — `image: "photo.png"` with no `alt` field and no `decorative: true`
  produces E-A11-001: `Missing alt text on image 'photo.png' at <file>:<line>:<col>.
  Add alt "..." or mark decorative: true`. Build exits 2 in strict mode. No output.
  (traces to BC-5.01.001 postcondition 1, postcondition 2)

- [ ] **AC-002** — `slide diagram:` with no `alt` field produces E-A11-001 for the diagram
  element. `slide chart:` with no `alt` field also produces E-A11-001.
  (traces to BC-5.01.001 postcondition 1)

- [ ] **AC-003** — `alt ""` (empty string) is treated as missing alt text. E-A11-001 emitted.
  `alt " "` (whitespace only) is also treated as missing alt text.
  (traces to BC-5.01.001 edge case EC-001, EC-002)

- [ ] **AC-004** — A deck with 3 images, all missing alt, accumulates 3 E-A11-001 diagnostics
  (all reported before halting).
  (traces to BC-5.01.001 postcondition 2; BC-1.15.002)

- [ ] **AC-005** — In `--warn-only` mode, E-A11-001 is demoted to a warning. Build continues;
  error-slide placeholder replaces the affected slide; exit 0.
  (traces to BC-5.01.001 postcondition 4)

- [ ] **AC-006** — `image: "bg.png"` with `decorative: true` produces no E-A11-001. The
  decorated element's alt text in IR is set to `AltText::Decorative` (not None, not Str).
  (traces to BC-5.01.002 postcondition 1)

- [ ] **AC-007** — `decorative: true` on an image propagates to the `ContentBlock::Image`
  variant in the `Deck` IR with `alt: AltText::Decorative`. Exporters read this field
  to produce: PPTX `descr=""`, PDF Artifact, HTML `alt="" role="presentation"`.
  The validator does NOT write output format metadata — it only validates and sets
  `AltText::Decorative` in the IR.
  (traces to BC-5.01.002 postcondition 2, postcondition 3, postcondition 4)

- [ ] **AC-008** — All slides on which every visual element is `decorative: true` produce
  no E-A11-001 (DEC-008: legitimately all-decorative slide is valid).
  (traces to BC-5.01.002 edge case EC-001)

- [ ] **AC-009** — `alt "..."` and `decorative: true` both set on the same element:
  `decorative` wins; W-A11-001 lint warning is emitted:
  `Alt text ignored for decorative element at <file>:<line>:<col>`.
  (traces to BC-5.01.002 invariant 3)

- [ ] **AC-010** — The validator runs BEFORE layout (not in exporters). Confirmed by test
  structure: `AltTextValidator::validate()` takes `&Deck` (semantic IR), not `&LaidOutDeck`.
  (traces to BC-5.01.001 invariant 2)

- [ ] **AC-011** — `AltTextValidator` implements the `Validator` plugin trait from STORY-002.
  `cargo test -p slideforge-validate` passes.

- [ ] **AC-012** — `#![forbid(unsafe_code)]`, zero `.unwrap()`, `clippy::pedantic` clean,
  all public items documented.
  (traces to NFR-021, NFR-022, NFR-023, NFR-024)

- [ ] **AC-013** — All production deps use `=` version pinning.
  (traces to NFR-025)

## Previous Story Intelligence

N/A — first story in EPIC-04. Key setup:

- **STORY-001** defines `ContentBlock` (with `Image`, `Chart`, `Diagram`, `Shape` variants),
  `Deck`, `Slide`, `SourceSpan`. The validator walks these IR types.
- **STORY-002** defines the `Validator` trait. `AltTextValidator` implements it.
- **STORY-010** defines `DiagnosticSink`. All E-A11-001 / W-A11-001 errors go through it.

The IR type system from STORY-001 must be extended to include the `AltText` enum.
Coordinate with STORY-001's implementer — if `AltText` is not yet in the IR, add it
as part of this story's first task (modifying `slideforge-types`).

## Architecture Compliance Rules

Sourced from `architecture/crate-architecture.md` and `architecture/verification-architecture.md`:

1. **Pure Core classification (SS-03):** `slideforge-validate` is a Pure Core crate.
   No I/O, no filesystem access, no async. The validator reads IR in-memory.
2. **Validation stage position:** The validator runs AFTER evaluation (after `eval_deck()`)
   and BEFORE layout (`slideforge-layout`). This ordering gives it access to fully resolved
   values (slide content, alt text strings after `{{ }}` evaluation). The validator MUST NOT
   be called before evaluation completes.
3. **`Validator` trait boundary:** `AltTextValidator::validate()` MUST use the public
   `Validator` trait from `slideforge-plugin-api`. No internal shortcut that bypasses the
   trait interface (BC-5.02.002 / DI-008).
4. **`AltText` enum in IR:** Add `AltText` to `slideforge-types` (extends STORY-001):
   ```rust
   pub enum AltText {
       Provided(Arc<str>),    // non-empty alt text
       Decorative,            // decorative: true; empty alt in output
   }
   ```
   Visual element fields use `Option<AltText>` — `None` means missing (E-A11-001 territory).
5. **Forbidden dependencies:** `slideforge-validate` MUST NOT depend on: `slideforge-eval`,
   `slideforge-layout`, `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`,
   `slideforge-data`, `slideforge-brand`, `slideforge-cli`.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `slideforge-types` | workspace | `Deck`, `Slide`, `ContentBlock`, `SourceSpan`, `AltText` |
| `slideforge-plugin-api` | workspace | `Validator` trait |
| `thiserror` | `=2.0.18` | `ValidationError` derives |
| `miette` | workspace (`"7"` with `fancy` feature — resolves to latest 7.x, currently 7.6.0) | `Diagnostic` trait impl on `ValidationError` |

Dev dependencies: `insta` (compatible)

## File Structure Requirements

Files to create:

```
crates/slideforge-validate/
├── Cargo.toml                     # workspace deps; thiserror =2.0.18, miette =7.2
├── src/
│   ├── lib.rs                     # crate root; #![forbid(unsafe_code)]; pub use
│   ├── error.rs                   # ValidationError enum; E-A11-001, W-A11-001 error codes
│   ├── alt_text.rs                # AltTextValidator: Validator impl
│   └── utils.rs                   # is_blank(s: &str) -> bool; shared validation helpers
```

Files to extend (in `slideforge-types`):

```
crates/slideforge-types/src/
├── block.rs    # ADD: AltText enum; update ContentBlock::Image, Chart, Diagram, Shape to use Option<AltText>
```

## Tasks

1. **Extend `crates/slideforge-types/src/block.rs`** — add `AltText` enum; update
   `ImageSpec`, `ChartSpec`, `DiagramSpec`, `ShapeSpec` to use `alt: Option<AltText>`.
   Ensure `AltText` implements `Hash + Eq + Clone + Debug`. (30 min)
2. **Verify `crates/slideforge-validate/Cargo.toml`** has workspace deps
   (`thiserror = { workspace = true }`, `miette = { workspace = true }`). The crate
   already exists as a scaffold. (10 min)
3. **Write `src/lib.rs`** with `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, module
   declarations. (10 min)
4. **Write `src/error.rs`** — `ValidationError` enum with `MissingAltText`,
   `DecorativeWithAlt` (lint), and `EmptyAltText` variants. Each carries `span: SourceSpan`
   and `element_type: Arc<str>`. Implement `miette::Diagnostic` with error codes
   `E_A11_001` and `W_A11_001`. (30 min)
5. **Write `src/utils.rs`** — `pub fn is_blank(s: &str) -> bool` (true if empty or
   all-whitespace). Used to detect invalid alt text. (10 min)
6. **Write `src/alt_text.rs`** — `AltTextValidator` struct implementing the `Validator`
   trait. The `validate(&self, deck: &Deck, sink: &mut DiagnosticSink)` method:
   1. Walk all slides in the deck.
   2. For each slide, walk `slide.blocks` and check `ContentBlock::Image`, `Chart`,
      `Diagram`, `Shape` variants.
   3. For each visual element: check `spec.alt`:
      - `None` → push E-A11-001.
      - `Some(AltText::Provided(s))` if `is_blank(s)` → push E-A11-001.
      - `Some(AltText::Decorative)` → no error (valid opt-out).
      - `Some(AltText::Provided(s))` with element also marked decorative → push W-A11-001.
   4. All errors accumulated (never return early). (60 min)
7. **Write unit and integration tests**. See Test Strategy. (45 min)
8. **Run `cargo test -p slideforge-validate`** and confirm pass. (10 min)

## Test Strategy

### Unit tests (`#[cfg(test)] mod tests` in `alt_text.rs`)

- `test_missing_alt_single_image()`: Deck with 1 image, no alt → 1 E-A11-001.
- `test_missing_alt_multiple_images()`: Deck with 3 images, all missing alt → 3 E-A11-001.
- `test_present_alt_no_error()`: image with `alt: Some(AltText::Provided("Team photo"))` → 0 errors.
- `test_empty_string_alt_is_missing()`: `alt: Some(AltText::Provided(""))` → E-A11-001.
- `test_whitespace_only_alt_is_missing()`: `alt: Some(AltText::Provided("  "))` → E-A11-001.
- `test_decorative_no_error()`: `alt: Some(AltText::Decorative)` → 0 errors.
- `test_decorative_all_elements()`: slide with 3 images all decorative → 0 errors.
- `test_alt_and_decorative_together()`: `alt: Some(AltText::Provided("text")), decorative: true` → W-A11-001 lint (decorative wins).
- `test_chart_missing_alt()`: chart slide with no alt → E-A11-001.
- `test_diagram_missing_alt()`: diagram slide with no alt → E-A11-001.
- `test_error_accumulation()`: 2 images missing alt + 1 diagram missing alt → 3 E-A11-001.

### Snapshot tests

- `test_snapshot_alt_error_message()`: missing alt on image; snapshot the rendered diagnostic string (includes file:line:col, element type, hint).

## Dependencies

**Depends on:**
- STORY-001 (IR types: `Deck`, `Slide`, `ContentBlock`, `SourceSpan`; extends `AltText`)
- STORY-002 (plugin-api: `Validator` trait — creates `slideforge-plugin-api` crate)
- STORY-010 (DiagnosticSink)

**Blocks:**
- STORY-016 (canvas overflow validation — shares the `slideforge-validate` crate; STORY-015 establishes the crate structure)
- STORY-017 (WCAG contrast + label validation — same crate)
- STORY-037/039 (PPTX export + accessibility metadata — reads `AltText` from IR)
- STORY-043/045 (PDF export + PDF/UA-1 — reads `AltText::Decorative` for Artifact tagging)
- STORY-046 (HTML export — reads `AltText` for `alt=""` / `role="presentation"`)
- STORY-068 (Kani proofs for validate — includes proofs on alt invariant VP-002, VP-008)

### Dependency Anchor Justifications

- SS-03 owns this story's scope because SS-03 is the Validation subsystem in the
  ARCH-INDEX Subsystem Registry and `slideforge-validate` is its sole crate.
- STORY-015 depends on STORY-001 because the validator walks `Deck` / `Slide` / `ContentBlock`
  IR types from `slideforge-types`.
- STORY-015 depends on STORY-002 because `AltTextValidator` implements the `Validator`
  plugin trait from `slideforge-plugin-api` (crate created in STORY-002).
- STORY-015 depends on STORY-010 because `DiagnosticSink` (from STORY-010) is the error
  accumulation mechanism used by all validators.
- STORY-015 blocks STORY-016/017 because those stories extend the same `slideforge-validate`
  crate structure established here.

**Note:** `slideforge-plugin-api` is not yet in `[workspace.members]` — STORY-002 must
add it. Verify the crate exists before implementing this story.

## Implementation Notes

### `AltText` Enum (Extends `slideforge-types`)

```rust
/// The alt text state of a visual element.
///
/// Every visual element in the IR must carry an explicit alt text decision.
/// `None` means the declaration is missing — E-A11-001 at validation time.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AltText {
    /// Non-empty, non-whitespace alt text provided by the author.
    Provided(Arc<str>),
    /// Element explicitly marked `decorative: true`.
    /// Produces empty alt in all output formats (PDF Artifact, PPTX descr="", HTML alt="").
    Decorative,
}
```

### Visual Element Types Subject to Alt Requirement

The following `ContentBlock` variants require alt text:
- `ContentBlock::Image(ImageSpec)` — always
- `ContentBlock::Chart(ChartSpec)` — always (describes the data, e.g., "Bar chart: Q1-Q4 revenue")
- `ContentBlock::Diagram(DiagramSpec)` — always (describes the diagram content)
- `ContentBlock::Shape(ShapeSpec)` — only if the shape has visual/image content; pure decorative shapes with no content meaning may use `decorative: true`

Non-visual blocks (Text, Bullets, Math, Table) do NOT require alt text — they are text
content that is already readable.

### Strict Mode vs. Warn-Only Mode

The validator itself does NOT know about strict mode vs. warn-only mode. It emits all
E-A11-001 diagnostics into the `DiagnosticSink`. The mode decision (abort build vs.
produce error-slide placeholder) is made by the CLI (STORY-055) based on the
`--warn-only` flag and the severity level of the diagnostics in the sink.

The validator's contract: emit ALL validation errors; never abort early.

### Error Message Format

```
E-A11-001: Missing alt text on <element_type> at <file>:<line>:<col>.
  Add alt "..." or mark decorative: true.
  (DI-001: all visual elements require alt text or decorative: true)
```

`<element_type>` is one of: "image", "chart", "diagram", "shape".
The `miette` Diagnostic impl renders this with colored source pointers.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `alt ""` (empty string alt) | E-A11-001 (empty string is not valid alt text) |
| EC-002 | `alt "   "` (whitespace-only alt) | E-A11-001 (whitespace-only is not valid alt text) |
| EC-003 | All visual elements on a slide are decorative (DEC-008) | No E-A11-001; slide is valid |
| EC-004 | Shape with no image content (pure layout box) | If `decorative: true` → no error. If neither alt nor decorative → E-A11-001 |
| EC-005 | Visual element inside `@for` loop (multiple instances per template) | Each instance evaluated → each instance in Deck IR checked independently |
| EC-006 | `@if`-excluded slide with missing alt | Excluded slides are not in `Deck` IR → no E-A11-001 for excluded content |
| EC-007 | alt and decorative both set (EC-002 from BC-5.01.002) | W-A11-001 warning; decorative wins; alt text not used in output |
