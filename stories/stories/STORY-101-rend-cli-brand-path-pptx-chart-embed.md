---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-101
title: "REND-009/010b: CLI bare-path brand discovery fix + PPTX chart SVG embedding"
epic: EPIC-15
wave: 5
points: 8
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.0"
created: "2026-06-11"
source_findings: [REND-009, REND-010]
behavioral_contracts: [BC-4.01.001, BC-1.11.001, BC-3.07.003]
# BC-3.07.003 (CLI input path normalized before brand and config discovery) is the
# authoritative contract for the REND-009 bare-path fix. It specifies the 4-case
# normalize_parent() pure function, the forbidden empty-path diagnostic rule, and the
# regression test requirement. Authored 2026-06-11 per product-owner rendering-fix wave burst.
#
# REND-010b: this story IS the anchor story for the "deferred to a later story" code
# comment in slide_serializer.rs line ~653 (FrameContent::Chart). After this story is
# delivered, that comment is removed (chart SVG embedding implemented). The comment at
# line ~1686 (build_image_picture) must be updated to cite STORY-102.
verification_properties: []
nfr_refs: []
closes_findings: [REND-009, REND-010]
depends_on:
  - STORY-055
  - STORY-031
  - STORY-037
  - STORY-038
blocks: []
target_module: slideforge-cli, slideforge-pptx
subsystems: [SS-18, SS-06]
estimated_days: 4
---

# STORY-101: REND-009/010b — CLI Bare-Path Brand Fix + PPTX Chart SVG Embedding

## Subsystem Anchor Justification

SS-18 (CLI Orchestrator) owns REND-009: the bare-path `Path::parent()` bug is in the CLI
brand resolution module. SS-06 (PPTX Export) owns REND-010b: chart SVG and image media
embedding belongs to `slideforge-pptx`. EPIC-15 anchors because the most user-reported
bug (CLI fails on `slideforge build deck.sf`) is a CLI/SS-18 issue.

## Dependency Anchor Justifications

- `depends_on: [STORY-055]` — CLI build command; brand resolution lives in the CLI crate
  or the brand resolution module called from CLI.
- `depends_on: [STORY-031]` — Chart renderer produces SVG output; this story wires that
  SVG into the PPTX media ZIP.
- `depends_on: [STORY-037]` — PPTX core serializer with ZIP assembly; media parts are
  added to the ZIP here.
- `depends_on: [STORY-038]` — Layout compliance; SVG relationship entries follow the
  layout established in STORY-038.

## Narrative

As a slideforge user, I want to run `slideforge build deck.sf` from any working directory
using a bare filename without getting a cryptic "brand I/O error for ''" diagnostic, and
I want chart SVGs to appear in the PPTX output (not silent empty picture placeholders).

## Previous Story Intelligence

STORY-055 built the CLI build command including brand resolution. The bare-path bug was
not discovered until demo review. STORY-037 built the PPTX serializer and intentionally
deferred chart/image media embedding with a code comment "deferred to a later story" —
but no story ID was cited. This story is that later story for chart SVG embedding.
Image binary data embedding is a separate concern scoped to the Expected Gaps section.

## Architecture Compliance Rules

- REND-009 fix: `path.parent()` of a bare filename (e.g., `"deck.sf"`) returns
  `Some(Path::new(""))`. Brand resolution must normalize this to `Path::new(".")`. The fix
  is: `path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."))`.
- REND-010b: chart SVG embedding follows the OOXML media part pattern established in
  BC-4.01.001 EC-004: chart SVG goes in `/ppt/media/chartN.svg`; a relationship entry in
  `slide1.xml.rels` references it via `r:embed`; `[Content_Types].xml` has an SVG Override.
  The `<a:blip r:embed="rId..."/>` in `BlipFill` must reference the actual rId.
- Image binary embedding is deferred to a follow-up story (see below) — this story
  delivers chart SVG only. The code comment for image embedding on line 1686 is
  updated to cite `STORY-102` (see BC Gap Notice below).
- Per ADR-001: no raw XML; use ooxmlsdk types for all relationship and content-type entries.
- Per DI-010: integer EMU coordinates in all position/size attrs.
- A regression test for `slideforge build deck.sf` with a bare filename MUST be added.

## BC Gap Notice (image binary embedding)

REND-010b covers two separate deferrals in `slide_serializer.rs`:
1. Line 653: `FrameContent::Chart` — chart SVG embedding → **implemented in this story**.
2. Line 1686: `build_image_picture` — image binary embedding → **deferred to STORY-102**
   (to be created in a follow-up burst if needed, or folded into an existing Wave-5 image
   story if the orchestrator confirms one exists).

After this story lands, the code comment at line 1686 must read:
```rust
// image media embedding is deferred to STORY-102 (image-binary-pptx-embedding)
```

PO action: confirm STORY-102 scope or whether image embedding is in scope for this story.
If the human authorizes folding image embedding into this story, remove this notice and
implement AC-006 below.

## Library & Framework Requirements

- `ooxmlsdk` 0.6.1 — `Relationship`, `ContentTypeOverride` types for media part registration.
- `slideforge-pptx` ZIP assembler (from STORY-037).
- `std::path::Path` — for CLI bare-path fix.

## File Structure Requirements

Files to modify:
- `crates/slideforge-cli/src/main.rs` or brand resolution module — normalize
  `path.parent()` for bare filenames.
- `crates/slideforge-pptx/src/slide_serializer.rs` — implement chart SVG media embedding
  in `FrameContent::Chart` branch (line ~653); update `build_image_picture` comment
  to cite STORY-102.
- `crates/slideforge-pptx/src/lib.rs` — register SVG media parts in ZIP assembly;
  add relationship entries; add content-type overrides.
- `crates/slideforge-cli/src/tests/` — regression test for bare-path invocation.
- `crates/slideforge-pptx/src/tests/core_tests.rs` — add failing test for chart SVG embed.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| This story spec | ~2,500 |
| `crates/slideforge-cli/src/main.rs` or brand module | ~2,000 |
| `crates/slideforge-pptx/src/slide_serializer.rs` (lines 640-700, 1680-1750) | ~3,000 |
| `crates/slideforge-pptx/src/lib.rs` (ZIP assembly path) | ~3,000 |
| Test files | ~2,500 |
| **Total** | **~13,000** |

## Acceptance Criteria

### AC-001: slideforge build deck.sf succeeds with bare filename
(traces to BC-3.07.003 postcondition 3 — brand discovery succeeds for bare-filename invocations; BC-3.07.003 postcondition 2 — no diagnostic cites empty path)

`slideforge build deck.sf` invoked from the directory containing `deck.sf` succeeds
(exit 0) and produces output. No "brand I/O error for ''" diagnostic is emitted. The
brand resolution correctly searches the current working directory when the normalized
parent is `"."`. Any E-BRD-* or I/O diagnostic cites the normalized path (`"."` or
`"./brand.pptx"`) and NEVER an empty string `""` (BC-3.07.003 postcondition 2 —
empty path in a diagnostic is a contract violation).

Verified by: integration test invoking the CLI with `path = "deck.sf"` from a temp dir
containing the file; assert exit 0 and no "I/O error for ''" in stderr.

### AC-002: normalize_parent() 4-case pure-function contract
(traces to BC-3.07.003 postcondition 1 + invariant 1 — pure function with no side effects, all four input forms)

`normalize_parent` is a pure function (BC-3.07.003 Invariant 1: same input always yields
same output, no side effects). A unit test verifies all four canonical cases from
BC-3.07.003 postcondition 1:
```
normalize_parent("deck.sf")          == Path::new(".")          // bare filename
normalize_parent("./deck.sf")        == Path::new(".")          // dot-relative
normalize_parent("/abs/path/deck.sf") == Path::new("/abs/path") // absolute
normalize_parent("subdir/deck.sf")   == Path::new("subdir")     // relative-with-dir
```
Per BC-3.07.003 Invariant 2, normalization is applied ONCE at the CLI entry point. No
downstream crate (brand, config, layout, export) performs its own bare-filename fix.
Per BC-3.07.003 Invariant 4, normalization does NOT change the source file path itself
(only the parent used for brand/config discovery).

Verified by: unit test in `slideforge-cli` asserting these four cases exactly as specified
in BC-3.07.003 postcondition 1.

### AC-003: Chart SVG embedded as media part in .pptx
(traces to BC-4.01.001 EC-004 — chart SVG in /ppt/media/chart1.svg with relationship)

A .pptx built from a deck with a chart slide contains the chart SVG at
`ppt/media/chart1.svg` (or `chartN.svg` for multi-chart decks). The corresponding
`slide1.xml.rels` has a relationship entry with `Type="..." Target="../media/chart1.svg"`.
`[Content_Types].xml` has an Override for `application/svg+xml` (or `image/svg+xml`).

Verified by: integration test building a chart deck; unzip .pptx; assert file exists
at `ppt/media/chart1.svg`; assert relationship entry in slide rels; assert content-type
override.

### AC-004: PPTX chart shape references embedded SVG via r:embed blip
(traces to BC-4.01.001 EC-004 and BC-1.11.001 — chart renders in PPTX output)

The `<a:blip>` element within the chart `<p:pic>` carries a populated `r:embed` attribute
referencing the chart's media relationship ID. The placeholder `<a:blip/>` with no embed
attribute is replaced with `<a:blip r:embed="rId_chart1"/>`. The chart is visible when
the .pptx is opened in PowerPoint.

Verified by: unit test parsing `slide1.xml`; assert `<a:blip r:embed="..."/>` is non-empty
for chart shapes.

### AC-005: Code comment at line 1686 updated to cite STORY-102
(REND-010b discipline requirement — unanchored deferrals are forbidden per CLAUDE.md)

The code comment in `slide_serializer.rs` at the `build_image_picture` function reads:
```rust
// image media embedding is deferred to STORY-102
```
No "deferred to a later story" without a story ID appears anywhere in the codebase.

Verified by: grep the codebase for "deferred to a later story" returns zero results after
this story is merged.

### AC-006 (conditional — PO confirmation required): Image binary embedding in .pptx
(traces to BC-4.01.001 EC-004 — images embedded as media parts)

If the human authorizes folding image embedding into this story: image frames produce
a .pptx with the image binary at `ppt/media/imageN.png` (or appropriate format) with
a correct relationship and content-type entry, and the `<a:blip r:embed="..."/>` element
populated. This AC is OPTIONAL pending PO confirmation.

## Tasks

- [ ] **T-001 (RED):** Write `test_bare_path_brand_resolution()` in `slideforge-cli`.
- [ ] **T-002 (RED):** Write `test_normalize_parent_paths()`.
- [ ] **T-003 (RED):** Write `test_chart_svg_in_pptx_media_zip()` in `slideforge-pptx`.
- [ ] **T-004 (RED):** Write `test_chart_blip_has_r_embed()`.
- [ ] **T-005 (GREEN):** Fix `path.parent()` normalization in CLI brand resolution.
- [ ] **T-006 (GREEN):** Implement chart SVG media embedding in `FrameContent::Chart` branch.
- [ ] **T-007 (GREEN):** Register chart SVG in ZIP assembly (media part + rels + content-type).
- [ ] **T-008 (GREEN):** Wire `r:embed` rId into `<a:blip>` element.
- [ ] **T-009 (GREEN):** Update code comment at line 1686 to cite STORY-102.
- [ ] **T-010:** Run `cargo nextest run -p slideforge-cli -p slideforge-pptx --no-fail-fast`.
- [ ] **T-011:** `grep -rn "deferred to a later story" crates/` — must return zero results.
- [ ] **T-012:** Run `just check` before declaring done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slideforge build ./deck.sf` (explicit dot prefix) | Also works; no brand error |
| EC-002 | `slideforge build /abs/path/to/deck.sf` (absolute path) | Brand searched in /abs/path/to/; works |
| EC-003 | Deck with multiple chart slides | Each chart gets its own media part (chart1.svg, chart2.svg) |
| EC-004 | Deck with no charts | No chart media parts; no crash; clean .pptx |
| EC-005 | Chart SVG with foreignObject (invalid for PPTX per BC-1.12.003) | SVG has no foreignObject (normalized by STORY-034's usvg pass) |

## Behavioral Contracts Table

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| BC-3.07.003 | CLI Input Path Normalized Before Brand and Config Discovery | AC-001, AC-002 |
| BC-4.01.001 | Serialize LaidOutDeck to Valid .pptx | AC-003, AC-004 |
| BC-1.11.001 | Chart renderer via plotters | AC-003, AC-004 |

## Test Strategy

TDD strict mode. Four failing tests first (two per defect area). The CLI bare-path fix is
a one-liner; the chart embedding is more involved (ZIP assembly + rels + content-type).
Run the "deferred to a later story" grep (T-011) as an explicit CI regression gate.
