---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-035
title: "Writing Register Routing in Evaluator"
epic: EPIC-18
wave: 4
points: 5
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-1.14.001, BC-1.14.002, BC-1.14.003, BC-1.14.004]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-eval
target_module: slideforge-eval
subsystems: [SS-02]
depends_on:
  - STORY-011
  - STORY-012
  - STORY-013
blocks:
  - STORY-036
  - STORY-037
  - STORY-040
  - STORY-041
  - STORY-042
estimated_days: 2
---

# STORY-035: Writing Register Routing in Evaluator

## Subsystem Anchor Justification

SS-02 (Evaluator) owns this story's scope because register routing is an evaluate-stage
concern per BC-1.14.001 invariant 1, BC-1.14.002 invariant 1, and BC-1.14.003 invariant 1.
The ARCH-INDEX Subsystem Registry lists `slideforge-eval` under SS-02 as the pure-core
crate responsible for producing `LaidOutDeck` annotated with semantic information.
Register tagging must happen here, before exporters, so that no exporter can accidentally
include the wrong register content (DI-012).

## Dependency Anchor Justifications

- Depends on STORY-011 (Expression Evaluator Core): register fields (`notes:`, `report:`,
  `detail:`) contain `{{ expr }}` interpolations that must be resolved by the evaluator
  before register tagging.
- Depends on STORY-012 (Variable Scoping + @for): registers inside `@for` loops produce
  per-iteration content blocks, requiring scoping to be complete first.
- Depends on STORY-013 (@if/@elif/@else Evaluation): conditional registers must evaluate
  to a concrete value before being tagged to a register.
- Blocks STORY-036 (No Content Bleed Invariant): the bleed-invariant tests require the
  routing infrastructure built in this story to exist first.
- Blocks STORY-037, 040, 041, 042: PPTX and DOCX exporters consume register-tagged
  content from `LaidOutDeck`; that tagging is established here.

## Summary

Implement the writing register routing pass in `slideforge-eval`. After the main
evaluation pass produces a `Deck` IR with all field values resolved, this pass traverses
every `Slide` and every `Section` looking for content in the three writing registers:
`notes`, `report`, and `detail`. It tags each `ContentBlock` with its `Register` variant
and stores the result in the `LaidOutSlide.register_content` map. Exporters read only
their allowed registers from this map; they never inspect raw slide fields for register
content.

### Three Registers

| Register | DSL Fields | Routed To |
|----------|-----------|-----------|
| `notes` | `notes "..."` / `notes:` block | PPTX `<p:notes>`, DOCX notes section, HTML `<aside data-notes>` |
| `report` | `report "..."` / `report:` block | DOCX body paragraphs, PDF body text |
| `detail` | `detail "..."` / `detail:` block | DOCX extended sections, PDF appendix |

### ContentBlock Structure

Each register content block is stored as `RegisteredContent`:

```rust
pub enum Register {
    Notes,
    Report,
    Detail,
}

pub struct RegisteredContent {
    pub register: Register,
    pub content: Vec<InlineSpan>,  // already-evaluated inline content
}
```

`LaidOutSlide` gains a field:

```rust
pub register_content: Vec<RegisteredContent>,
```

This field is populated by the routing pass and consumed by exporters. Visual slide
content (title, bullets, etc.) is stored separately in `LaidOutSlide.frames` and is
never tagged with a register.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-1.14.001 | notes register routes to presenter notes in PPTX/DOCX/HTML only | AC-001, AC-002 |
| BC-1.14.002 | report register routes to DOCX body; excluded from PPTX slide content | AC-003, AC-004 |
| BC-1.14.003 | detail register routes to DOCX/PDF only; excluded from PPTX and web preview | AC-005, AC-006 |
| BC-1.14.004 | No Register Content Bleeds to Wrong Format | AC-008 |

## Acceptance Criteria

### AC-001: notes fields extracted and tagged at evaluate stage
(traces to BC-1.14.001 invariant 1 — routing determined at Evaluate stage)

After evaluation, every `LaidOutSlide` that has a non-empty `notes` field or `notes:`
block has at least one `RegisteredContent { register: Register::Notes, content: ... }`
entry in `register_content`. The `notes` content is evaluated (all `{{ expr }}`
interpolations resolved) before tagging. The `notes` content does NOT appear in
`LaidOutSlide.frames` (visual content).

### AC-002: notes content excluded from visual frames
(traces to BC-1.14.001 postcondition 1 — notes does NOT appear in slide body shapes)

Frame construction is ALLOWLIST-based: the layout pass reads only `title`, `subtitle`,
and `body` from `slide.fields`; register keys remain in `slide.fields` but are never
consulted by the frame builder. No `FrameContent` variant in `LaidOutSlide.frames`
contains notes register text (per architecture directive F-035-P2-002 / Option D).
A unit test builds a slide with `notes "Test notes"` and confirms `frames` does not
contain "Test notes" while `register_content` does (tagged as `Notes`).

### AC-003: report fields extracted and tagged at evaluate stage
(traces to BC-1.14.002 invariant 1 — report routing determined at Evaluate stage)

After evaluation, every `LaidOutSlide` with a non-empty `report` field or `report:`
block has at least one `RegisteredContent { register: Register::Report, content: ... }`
entry in `register_content`. The `report` content is evaluated before tagging. The
`report` content does NOT appear in `LaidOutSlide.frames`.

### AC-004: report content excluded from visual frames
(traces to BC-1.14.002 postcondition 3 — report NOT in PPTX slide shape)

A unit test builds a slide with `report "Analysis text"` and confirms: (1) `frames`
does not contain "Analysis text"; (2) `register_content` contains one entry with
`Register::Report` containing "Analysis text".

### AC-005: detail fields extracted and tagged at evaluate stage
(traces to BC-1.14.003 invariant 1 — detail routing determined at Evaluate stage)

After evaluation, every `LaidOutSlide` with a non-empty `detail` field or `detail:`
block has at least one `RegisteredContent { register: Register::Detail, content: ... }`
in `register_content`. The `detail` content is evaluated before tagging. The `detail`
content does NOT appear in `LaidOutSlide.frames`.

Note: section-level `detail` routing (standalone `section detail:` blocks attaching
`RegisteredContent` to a section node) is out of scope for this story — see Scope
Boundary section below for the descope record.

### AC-006: detail content excluded from visual frames
(traces to BC-1.14.003 postcondition 3 — detail NOT in PPTX or web preview)

A unit test builds a slide with `detail "Technical appendix text"` and confirms:
(1) `frames` does not contain "Technical appendix text"; (2) `register_content` contains
`Register::Detail` entry with the text.

### AC-007: interpolation evaluated before tagging
(traces to BC-1.14.001 postcondition 1 — notes content uses {{ expr }} interpolation)

A unit test declares `notes "Quarter: {{ quarter }}"` with `quarter = "Q1"` in scope.
The `RegisteredContent` in `register_content` contains "Quarter: Q1" (interpolated),
not the raw template string.

### AC-008: all three registers present on same slide
(traces to BC-1.14.004 invariant 3 — LaidOutSlide carries register-tagged content)

A unit test builds a slide with all three registers populated. `register_content`
contains exactly three entries: one `Notes`, one `Report`, one `Detail`. `frames`
contains only the visual content (title, bullets, etc.) — none of the register text.

## Tasks

- [ ] Define `Register` enum (`Notes`, `Report`, `Detail`) in `slideforge-types` (or `slideforge-eval` if it doesn't cross crate boundaries)
- [ ] Define `RegisteredContent { register: Register, content: Vec<InlineSpan> }` struct in `slideforge-types`
- [ ] Add `register_content: Vec<RegisteredContent>` field to `LaidOutSlide`
- [ ] Implement `extract_register_content(slide: &EvalSlide) -> Vec<RegisteredContent>` in `src/register_routing.rs`
  - Extract `notes`, `report`, `detail` fields from evaluated slide
  - Evaluate inline spans (interpolation already resolved at this point)
  - Tag each block with the correct `Register` variant
  - Return `Vec<RegisteredContent>`
- [ ] Call `extract_register_content` in the evaluation pipeline before layout pass
- [ ] Ensure register fields never reach `frames` — frame construction is ALLOWLIST-based (reads only `title`/`subtitle`/`body` from `slide.fields`); register keys intentionally remain in `slide.fields` and are surfaced only via `register_content` (per architecture directive F-035-P2-002 / Option D)
- [ ] Write unit tests:
  - `notes "..."` → `register_content` has `Notes`, `frames` does not contain text
  - `report "..."` → `register_content` has `Report`, `frames` does not contain text
  - `detail "..."` → `register_content` has `Detail`, `frames` does not contain text
  - `notes "Quarter: {{ quarter }}"` with scoped var → interpolated before tagging
  - Slide with all three registers → three entries, correct types, visual frames unaffected

## Previous Story Intelligence

N/A — first story in EPIC-18. Writing registers are a new concern. STORY-011, 012, 013
(expression evaluator core, scoping, conditional evaluation) establish the evaluation
pipeline that this story extends. The `LaidOutSlide` type is extended from STORY-026
(layout core) with the `register_content` field — coordinate with STORY-026's type
definitions to add the field there.

## Scope Boundary — Section-Level Register Routing Descoped

**Architect directive dated 2026-05-31 (F-002 [HIGH]):**

Section-level register routing — specifically the attachment of `RegisteredContent`
to a section node from a standalone `section detail:` or `section report:` block — is
**OUT OF SCOPE** for this story. It has been descoped because `SectionBlock.body` is
currently typed `OrderedMap<Arc<str>, Value>`, where `Value` is a fully resolved scalar.
`Value` cannot carry `FieldValue::Inlines` (rich inline content for `RegisteredContent.content`)
or `Vec<Block>` (nested block structure for a `detail:` sub-block). A plain-string
workaround is not acceptable under the production-grade default.

The concrete IR-extension work required (changing `SectionBlock.body` from `Value` to
`FieldValue`, teaching the parser to emit `FieldValue::Inlines` for `detail:` within
section blocks, adding eval-stage routing of section-level `detail`/`report` to
`RegisteredContent`) is tracked in **STORY-077** (anchor: BC-3.02.002 postcondition 1 /
EC-004 / descoped EC-003). STORY-077 blocks any DOCX story that renders section-level
detail or report content (cross-ref STORY-041, STORY-042).

Slide-level `detail` field extraction (AC-005) **remains in scope** — it is exercised
by the existing `extract_register_content` function for the `Slide` type.

## Architecture Compliance Rules

1. **Register routing is an evaluate-stage concern (BC-1.14.001/002/003 invariants)**:
   All three BC invariants state "routing is determined at the Evaluate stage". No
   exporter may re-derive register membership from raw DSL fields.
2. **LaidOutDeck IR carries register-tagged content (BC-1.14.004 invariant 3)**:
   `LaidOutSlide.register_content` is the authoritative source for register content;
   exporters read only this field.
3. **SS-02 purity boundary**: `slideforge-eval` is a pure core crate. The register
   routing pass is a pure transformation (no I/O, no file reads). This is compatible
   with SS-02's pure-core classification in ARCH-INDEX.
4. **No dependency on exporter crates**: `slideforge-eval` must NOT import
   `slideforge-pptx`, `slideforge-docx`, or any exporter. Forbidden dependency —
   build fails if added.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` (workspace) | workspace | `Register`, `RegisteredContent`, `LaidOutSlide` |
| `slideforge-eval` (self) | workspace | Register routing pass |

No new external dependencies required. This story extends existing types and the
evaluation pipeline.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-types/src/register.rs` | Modify | Add `RegisteredContent` struct (`Register` enum pre-exists) |
| `crates/slideforge-types/src/slide.rs` | Modify | Add `register_content: Vec<RegisteredContent>` to semantic `Slide` |
| `crates/slideforge-layout/src/types.rs` | Modify | Add `register_content: Vec<RegisteredContent>` to `LaidOutSlide` |
| `crates/slideforge-eval/src/register_routing.rs` | Create | `extract_register_content()` pass |
| `crates/slideforge-eval/src/for_eval.rs` | Modify | Call register routing pass inside `eval_slide_node` before layout |
| `crates/slideforge-layout/src/layout.rs` | Modify | Copy `register_content` from evaluated `Slide` into `LaidOutSlide` |
| `crates/slideforge-eval/src/register_routing.rs` | Modify | Co-located `#[cfg(test)] mod tests` for all AC/EC cases (inline per CLAUDE.md convention) |
| `crates/slideforge-eval/src/eval.rs` | Modify | Co-located `#[cfg(test)] mod tests` for `eval_deck` integration cases |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,800 |
| BC-1.14.001 | ~1,500 |
| BC-1.14.002 | ~1,200 |
| BC-1.14.003 | ~1,200 |
| BC-1.14.004 | ~1,000 |
| `slideforge-types` IR definitions (from STORY-001/026) | ~1,500 |
| Test files to write | ~2,000 |
| **Total** | **~11,200** |

Well within the 20-30% context budget.

## Test Strategy

- **Unit tests**: All six register types extracted correctly; interpolation resolved
  before tagging; visual frames uncontaminated; all-three-registers-on-same-slide
  case (most important invariant test).
- **Snapshot test**: `insta` snapshot of `register_content` for a 3-register slide —
  captures the exact content tagging for regression detection.
- **No integration tests in this story**: Integration (exporter-level bleed checking)
  is covered by STORY-036.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with only `notes` field (no visual content) | `frames` empty; `register_content` has one `Notes` entry |
| EC-002 | `notes` field with `{{ expr }}` interpolation | Interpolation evaluated; resolved value stored in `Notes` entry |
| EC-003 | ~~Standalone `section detail:` with no parent slide~~ | **DESCOPED** — moved to STORY-077 (BC-3.02.002). Section-level register routing requires `SectionBlock.body: FieldValue` IR extension not yet available. |
| EC-004 | `detail` and `report` both present on same slide | Both entries in `register_content`; both absent from `frames` |
| EC-005 | `@for` loop with `notes` field | Each iteration produces its own `Notes` entry in its slide's `register_content` |

## Forbidden Dependencies

`slideforge-eval` must NOT depend on:
- `slideforge-pptx` — exporter crate; would create a downward cycle
- `slideforge-docx` — exporter crate
- `slideforge-pdf` — exporter crate
- `slideforge-html` — exporter crate
- `slideforge-preview` — effectful shell

Build fails if any of the above appear in `slideforge-eval/Cargo.toml`.
