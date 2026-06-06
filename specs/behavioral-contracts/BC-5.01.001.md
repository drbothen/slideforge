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
capability: CAP-020
lifecycle_status: active
introduced: v1.0.0
modified:
  - version: "1.3"
    date: 2026-06-05
    author: product-owner
    reason: "ADR-019 (Stage 2b, human-authorized 2026-06-05): Clarify that E-A11-001 fires on AltText::Unspecified (pipeline gap — no author alt-text threaded), NOT on AltText::Decorative (author opt-out — valid). AltText::Unspecified is the new third variant (ADR-019 Decision 4) that distinguishes 'structural placeholder never overwritten' from 'author chose decorative'. Update Description, Postcondition 1, Invariant 2, EC-004, and EC-005 to reference AltText::Unspecified. Add EC-007 for the Decorative-valid path. Bump version."
  - version: "1.2"
    date: 2026-06-05
    author: product-owner
    reason: "STORY-050 Gap-2 / ADR-018 (human-authorized 2026-06-05): Correct Invariant 2 — AltTextValidator now runs POST-layout (Stage 6b) via validate_post_layout(&LaidOutDeck), not pre-layout. Deck.slides[*].blocks is always vec![] after eval (for_eval.rs:342); ContentBlock::Chart/Image/Diagram only exist in LaidOutDeck.slides[*].frames post-layout. The compile-error guarantee is upheld by the post-layout pass, not the pre-layout pass. Description and Postcondition 1 note updated to reflect post-layout enforcement."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-5.01.001: Missing alt on Visual Element Is Compile Error with Element Location (Post-Layout Enforcement via Stage 6b)

## Description

Every image, chart, and diagram declared in a .sf file must have a non-empty `alt "..."`
field, OR must be explicitly marked `decorative: true`. If either condition is absent,
the build produces E-A11-001 with the element type, identifier, and source location.
This is a compile-time contract — accessibility is never deferred to runtime or
post-processing. The error is fatal in strict mode.

**Pipeline placement (ADR-018 + ADR-019):** E-A11-001 is emitted by `AltTextValidator`
during the **post-layout validation pass** (Stage 6b), not the pre-layout pass (Stage 5).
`ContentBlock::Chart`, `ContentBlock::Image`, and `ContentBlock::Diagram` are created
during `layout::run` and exist only in `LaidOutDeck.slides[*].frames` — they are not
present in the pre-layout `Deck`. The enforcement guarantee is preserved: `build()`
returns `Err(BuildError::ValidationFailed)` before any output is written.

**AltText::Unspecified (ADR-019 Decision 4):** E-A11-001 fires specifically when a
frame's `alt` field is `AltText::Unspecified` — the pipeline placeholder variant that
means "no author alt-text data was threaded into this frame." `AltText::Unspecified`
is produced by `thread_media_alt_into_frames` (layout) when the upstream `ContentBlock`
has `alt = None` (author supplied neither `alt "..."` nor `decorative: true`), and
by `regions.rs` structural placeholders that were never overwritten.
`AltText::Decorative` (author wrote `decorative: true`) is a valid opt-out; it does
NOT trigger E-A11-001. This distinction is enforced by `validate_post_layout` match
arms (ADR-019 Decision 5.3).

## Preconditions

1. A visual element (image, chart slide, diagram slide, or shape with visual content) is declared in the source.
2. The element does NOT have a non-empty `alt "..."` field.
3. The element does NOT have `decorative: true`.

## Postconditions

1. E-A11-001 is emitted: `Missing alt text on <element-type> '<identifier>' at <file>:<line>:<col>. Add alt "..." or mark decorative: true`. Emitted by `AltTextValidator.validate_post_layout()` during Stage 6b, after `layout::run` produces the `LaidOutDeck`. The triggering condition is `FrameContent::Chart/Image/Diagram { alt: AltText::Unspecified }` — the pipeline placeholder state indicating no author alt-text was threaded. `AltText::Decorative` frames are valid and do NOT trigger this error.
2. Build returns `Err(BuildError::ValidationFailed(diagnostics))` in strict mode, where `diagnostics` contains at least one `Diagnostic { code: "E-A11-001", .. }`. No output is produced.
3. CLI exits with code 2 (validation error) in strict mode.
4. In `--warn-only` mode: warning emitted via `tracing::warn!`, build continues, output produced.

## Invariants

1. Every visual element in the output has EITHER non-empty alt text OR is marked as a PDF Artifact / empty-alt in all output formats. (DI-001)
2. **The alt text requirement is enforced POST-layout (Stage 6b) via `validate_post_layout`, firing on AltText::Unspecified frames.**
   `AltTextValidator` runs as part of the post-layout validation pass (Stage 6b per ADR-018),
   iterating `LaidOutDeck.slides[*].frames` for `FrameContent::Chart`, `FrameContent::Image`,
   and `FrameContent::Diagram`. The match logic (ADR-019 Decision 5.3):
   - `AltText::Unspecified` → E-A11-001 (missing alt; structural pipeline placeholder;
     no author alt-text data was threaded into this frame).
   - `AltText::Decorative` → valid (author explicitly wrote `decorative: true`; no error).
   - `AltText::Provided(_)` → valid (author supplied non-empty alt text; no error).
   `AltText::Unspecified` is the pipeline placeholder variant introduced in ADR-019 Decision 4
   to distinguish "structural placeholder never overwritten by author data" from
   "author chose decorative." Before ADR-019, `AltText::Decorative` was misused as a
   structural placeholder in `regions.rs`, causing false-positive E-A11-001 on decorative
   elements. That bug is fixed: `regions.rs` now uses `AltText::Unspecified` for all
   structural placeholders.
   The pre-layout `validate()` call on `AltTextValidator` is a no-op stub (correct —
   `Deck.slides[*].blocks` is always `vec![]` after eval per `for_eval.rs:342`). The
   enforcement guarantee — `build()` returns `Err(BuildError::ValidationFailed)` before
   any output is written — is upheld by Stage 6b, which runs before Stage 7 (export).
3. `decorative: true` produces empty alt attributes in all output formats (empty string in PPTX `<p:ph altText="">`, empty `/Alt` in PDF Artifact, `alt=""` in HTML).
4. No output format can contain a visual element with neither alt text nor decorative marker.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `alt ""` (empty string) | Treated as missing alt text. E-A11-001 emitted. An empty string is not a valid alt text. |
| EC-002 | `alt " "` (whitespace only) | Treated as missing alt text. Whitespace-only alt text is rejected. |
| EC-003 | All images on a slide have `decorative: true` (DEC-008) | No E-A11-001. All images produce empty alt in output. No accessibility error. |
| EC-004 | Chart slide with no `alt` field and no `decorative: true` | Stage 2b produces `ContentBlock::Chart { alt: None }`; `thread_media_alt_into_frames` maps `None` → `AltText::Unspecified` on the frame; `validate_post_layout` matches `Unspecified` → E-A11-001. Chart alt text should describe the data shown. |
| EC-005 | Diagram slide with no `alt` field and no `decorative: true` | Same pipeline path as EC-004: frame carries `AltText::Unspecified`; E-A11-001 emitted. |
| EC-006 | Image inside a shape: block with no alt | E-A11-001 for the shape's visual content. (Shape alt is set at parse time; this follows the BC-3.04.001 Invariant 1 path, not the Stage 2b path.) |
| EC-007 | Chart slide with `decorative: true` (no `alt "..."`) | Stage 2b produces `ContentBlock::Chart { alt: Some(AltText::Decorative) }`; `thread_media_alt_into_frames` writes `AltText::Decorative` to frame; `validate_post_layout` matches `Decorative` → **valid; no E-A11-001**. This is the correct behavior: author explicitly opted out. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `image: "photo.png"` with no alt (strict mode) | `Err(BuildError::ValidationFailed)` with `diagnostics[i].code == "E-A11-001"` — no output written | error (TV-7.1) |
| `image: "photo.png"` with `alt "Team photo from Q3 offsite"` | `Ok(BuildOutput)` — no error; alt text preserved in all outputs | happy-path |
| `image: "background.png"` with `decorative: true` | `Ok(BuildOutput)` — no error; empty alt in all outputs | edge-case (TV-7.3) |
| `image: "photo.png"` with `alt ""` (strict mode) | `Err(BuildError::ValidationFailed)` with `code == "E-A11-001"` (empty string is not valid alt) | error |
| `slide diagram:` with no `alt` field (strict mode) | `Err(BuildError::ValidationFailed)` with `code == "E-A11-001"` on the diagram element | error |
| `slide chart:` with no `alt` field, `strict = false` | `Ok(BuildOutput)` — warning emitted via `tracing::warn!`, output produced | warn-only |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Every image in output has non-empty alt XOR is Artifact/empty | integration test: parse output PPTX/PDF/HTML, verify alt presence |
| VP-TBD | Error count = number of visual elements missing alt (or decorative) | unit test with N-image fixture |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 |
| Capability Anchor Justification | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 — "alt '...' on all visual elements (images, charts, diagrams) ... compile error if absent" is the exact language of CAP-020 |
| L2 Domain Invariants | DI-001 (alt text required on all visual elements) |
| Architecture Module | slideforge-validate crate (SS-03) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.01.002 — composes with (decorative: true opt-out is the complement of this BC)
- BC-5.01.003 — related to (same pattern for color-coded elements)
- BC-3.03.002 — depends on (this validation error triggers the no-output gate)
- BC-4.03.001 — depends on (PDF/UA-1 requires alt text; this BC ensures alt is present at PDF export time)

## Architecture Anchors

- `architecture/plugin-architecture.md` — compile-time accessibility enforcement

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
