---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-095
title: "REND-002/008-pdf: PDF line-wrapping engine + progress_bar /Figure tag + bold font subset"
epic: EPIC-13
wave: 5
points: 8
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.0"
created: "2026-06-11"
source_findings: [REND-002, REND-008]
behavioral_contracts: [BC-4.03.001, BC-4.03.002]
# BC status: BC-4.03.001 (PDF/UA-1 — progress_bar tagged /Artifact violates structure
# requirements; missing bold font subset means bold text renders wrong). BC-4.03.002
# (PDF produced via pdf-writer + krilla — text clipping past page edge is a rendering
# defect in the krilla emission path). Both BCs are authored.
# REND-002 root cause: slideforge-pdf has no word-wrap engine; text runs past page edge.
# REND-008-pdf: progress_bar tagged /Artifact (must be /Figure with Alt);
#               only single font subset embedded (bold weight missing).
verification_properties: [VP-006]
nfr_refs: []
closes_findings: [REND-002, REND-008]
depends_on:
  - STORY-043
  - STORY-044
  - STORY-045
blocks: []
target_module: slideforge-pdf
subsystems: [SS-07]
estimated_days: 4
---

# STORY-095: REND-002/008-pdf — PDF Line-Wrapping + /Figure Tag + Bold Font Subset

## Subsystem Anchor Justification

SS-07 (PDF Export) owns all three defects: the line-wrapping engine, the structure-tag
assignment for progress_bar, and the font-subset embedding. All fixes live in
`slideforge-pdf`. EPIC-13 is the owning epic.

## Dependency Anchor Justifications

- `depends_on: [STORY-043]` — PDF core backend (pdf-writer + krilla + SlideTagEngine).
- `depends_on: [STORY-044]` — EMU-to-PDF coordinate mapping; line-wrapping depends on
  correct page-margin coordinates.
- `depends_on: [STORY-045]` — PDF/UA-1 tagging; this story fixes two UA-1 violations
  (structure tag and font) that undermine the veraPDF pass established in STORY-045.

## Narrative

As a slideforge user, I want PDF output where text wraps correctly within slide margins,
progress_bar information is accessible to screen reader users, and bold text renders with
the correct weight, so that the PDF is readable and PDF/UA-1 compliant.

## Previous Story Intelligence

STORY-043 built the PDF core with pdf-writer + krilla and the SlideTagEngine. STORY-045
achieved PDF/UA-1 compliance (veraPDF clean). This story fixes three post-STORY-045
defects discovered in demo output:

1. Text runs past the page edge — no word-wrap or character-wrap boundary.
2. `progress_bar` slide type tags its visual bar as `/Artifact` — AT loses the data.
3. Single font file embedded — the regular-weight subset only; bold characters render
   as regular weight.

## Architecture Compliance Rules

- Per BC-4.03.001: veraPDF `--flavour ua1` must pass in CI after this fix. The
  progress_bar `/Figure` tag with Alt attribute is required by ISO 14289-1 §7.3.
- Per BC-4.03.002: text emission uses pdf-writer + krilla; no Chrome/headless.
- DI-010: coordinates are integer EMUs converted to PDF user units via the established
  `pdf_coord()` mapping function from STORY-044.
- No f64 intermediate arithmetic in the EMU-to-PDF coordinate path.
- `#![forbid(unsafe_code)]` in `slideforge-pdf`.
- The word-wrap implementation MUST be a pure function (no side effects) for Kani
  amenability per VP-006.

## Library & Framework Requirements

- `krilla` (current version per `Cargo.lock`) — `TextShaper` or equivalent text-layout
  primitive for measuring glyph widths.
- `pdf-writer` (current version per `Cargo.lock`) — Structure element tagging APIs.
- Font data: both regular and bold variants of the deck's configured font must be loaded.
  Read `Cargo.lock` for exact dependency versions before writing.

## File Structure Requirements

Files to create / modify:
- `crates/slideforge-pdf/src/text_layout.rs` — NEW: word-wrap engine (pure function);
  `pub fn wrap_text(text: &str, max_width_pts: f64, font_metrics: &FontMetrics) -> Vec<&str>`
- `crates/slideforge-pdf/src/exporter.rs` — call `wrap_text` before emitting text spans;
  fix progress_bar tag from `/Artifact` to `/Figure` with Alt; load bold font subset.
- `crates/slideforge-pdf/src/tests/` — add failing tests before fixes.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| This story spec | ~2,500 |
| `crates/slideforge-pdf/src/exporter.rs` | ~5,000 |
| `crates/slideforge-pdf/src/text_layout.rs` (new) | ~2,000 |
| krilla / pdf-writer docs references | ~1,500 |
| Test files | ~2,000 |
| **Total** | **~13,000** |

## Acceptance Criteria

### AC-001: Text wraps at word boundaries within slide margins
(traces to BC-4.03.002 postcondition 1)

For any text span in a PDF slide that exceeds the frame width (in PDF user units), the
text is split at the last word boundary that fits within the frame width before being
emitted as a new line. No text run extends past the right margin of its containing frame.

Verified by: unit test with a long string (>80 chars) in a narrow frame; assert the
resulting PDF has multiple text span emissions for that content, none exceeding the frame
width. Integration test: build a sample deck with long body text; verify no visual clipping
in rendered output.

### AC-002: Hard-wrap fallback at character boundary for words longer than frame width
(traces to BC-4.03.002 postcondition 1)

When a single word exceeds the frame width, the word is broken at the character boundary
where it would overflow (character-wrap fallback). No text is silently dropped.

Verified by: unit test with a single 200-char word in a 100-char-wide frame; assert both
halves appear in PDF output.

### AC-003: progress_bar visual bar tagged as /Figure with Alt text, not /Artifact
(traces to BC-4.03.001 postcondition 1)

The `progress_bar` slide type's visual bar element is tagged with a `/Figure` structure
element carrying an `/Alt` attribute derived from the slide's `label` field. It is NOT
tagged as `/Artifact`. Assistive technology can reach the progress_bar information.

Verified by: unit test building a progress_bar slide; parse the output PDF structure tree
and assert the bar element carries `S=/Figure` with non-empty `/Alt`. Add to the
veraPDF CI integration test to confirm isCompliant remains true.

### AC-004: Bold font subset embedded in PDF output
(traces to BC-4.03.001 postcondition 1)

When the deck contains bold text (`InlineNode::Bold`), both the regular and bold font
subsets are embedded in the output PDF. Bold characters are rendered at their correct
weight (not degraded to regular weight).

Verified by: unit test building a slide with bold text; parse the PDF font resources and
assert two font subsets are present (regular + bold). Visual snapshot test comparing bold
text rendering in PDF.

### AC-005: veraPDF CI gate remains clean after fixes
(traces to BC-4.03.001 postcondition 2)

After implementing AC-001 through AC-004, `veraPDF --flavour ua1` on the CI integration
test PDF still reports `isCompliant: true` with zero violations.

Verified by: existing CI integration test in STORY-045 continues to pass.

## Tasks

- [ ] **T-001 (RED):** Write `test_text_wrap_word_boundary()` in `slideforge-pdf`.
- [ ] **T-002 (RED):** Write `test_text_wrap_char_fallback()`.
- [ ] **T-003 (RED):** Write `test_progress_bar_figure_tag()`.
- [ ] **T-004 (RED):** Write `test_bold_font_subset_embedded()`.
- [ ] **T-005 (GREEN):** Create `text_layout.rs` with pure word-wrap function.
- [ ] **T-006 (GREEN):** Wire `wrap_text` into the text emission path in `exporter.rs`.
- [ ] **T-007 (GREEN):** Fix progress_bar tag assignment from `/Artifact` to `/Figure`
  with Alt attribute derived from `label` field.
- [ ] **T-008 (GREEN):** Load bold font variant alongside regular font in the font-loading
  path; embed bold subset when bold runs are present.
- [ ] **T-009:** Run `cargo nextest run -p slideforge-pdf --no-fail-fast`.
- [ ] **T-010:** Run `just check` before declaring done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Text with no bold runs | Only regular font subset embedded; no crash |
| EC-002 | Empty text frame | No wrap attempted; empty frame emitted without error |
| EC-003 | progress_bar with decorative: true | Tagged /Artifact (correct — decorative opt-out per BC-5.01.002) |
| EC-004 | progress_bar with non-empty label | /Figure with Alt = label value |
| EC-005 | Text exactly equal to frame width | Single line emitted; no spurious wrap |

## Behavioral Contracts Table

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| BC-4.03.001 | PDF/UA-1 compliant, tagged | AC-003, AC-004, AC-005 |
| BC-4.03.002 | PDF via pdf-writer + krilla + SlideTagEngine | AC-001, AC-002 |

## Test Strategy

TDD strict mode. Write 4 failing tests first. `text_layout.rs` is a pure function module
and should be Kani-amenable per VP-006 (proptest bounds + Kani proof in Phase 6 for
wrap termination).
