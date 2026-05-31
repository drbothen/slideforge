---
document_type: scope-directive
story_id: STORY-043
issued_by: architect
date: 2026-05-31
status: binding
supersedes: []
---

# STORY-043 Scope Directive — Adversary Pass-1 Adjudication

This document resolves the three scope questions surfaced by the Pass-1 adversary
review (findings F-001, F-002, F-003, F-004, F-006, F-008). It is binding for the
implementer's fix burst. Amendments to story specs or BCs required as a consequence
are noted explicitly; the story-writer and product-owner execute those amendments.

---

## Decision 1: Structural Tag Tree (F-001, F-002, F-003, F-008)

### Ruling: 043 must wire the complete structural tag tree NOW.

The adversary finding is correct. The structural tag tree — the `TagTree` containing
`Document → Part(per slide) → {H1/P/LI/Figure(+Alt)/Table→TR→TH/TD}` groups — is
STORY-043 scope, not STORY-045 scope. This conclusion follows directly from AC-003
of STORY-043:

> "SlideTagEngine::tag_slide(slide: &LaidOutSlide, doc: &mut Document) -> TagTree
> builds the structure tree using the krilla::tagging module and attaches it via
> Document::set_tag_tree(tag_tree)."
> — STORY-043 AC-003, verbatim

The AC-003 behavioral assertions are unambiguous:
- One TagKind::Part group per slide in the tag tree.
- Figure elements use TagKind::Figure with /Alt set from LaidOutElement::alt.
- Decorative elements are marked as PDF Artifacts — NOT wrapped in a Tag.
- Table elements: Table → TR → TH/TD.
- The complete TagTree is attached via Document::set_tag_tree before document.finish().

### What the implementer deferred that must NOT be deferred:

The shipped `exporter.rs` comments out `set_tag_tree` with: "The tag tree is
attached only when tagging is fully wired (STORY-045)." This is a rationalized
deferral that violates the Canonical Principle. Building the structural tag tree
and attaching it via `set_tag_tree` requires ZERO pixel drawing — it is pure
IR→tag-tree mapping from `LaidOutSlide`. There is no technical blocker in STORY-043.

### Exact 043/045 split:

**STORY-043 delivers (now — no exception):**
- `SlideTagEngine::tag_slide` builds the complete per-slide Part group tree:
  - One `TagKind::Part` per slide.
  - Per frame: `TagKind::Hn(1)` for titles, `TagKind::Hn(2)` for subtitles,
    `TagKind::P` for body paragraphs, `TagKind::L(ListNumbering::Disc)` for
    list bodies, `TagKind::Figure` with `alt: Some(text)` for non-decorative
    figures/charts/diagrams, `TagKind::Table → TR → TH/TD` for tables.
  - Decorative frames (empty alt + `decorative: true`): NOT tagged — the engine
    returns a sentinel so the exporter can mark them as PDF Artifacts.
- `PdfExporter::export` calls `SlideTagEngine::tag_slide` for every slide,
  assembles the per-slide trees under a `TagKind::Document` root group, and calls
  `document.set_tag_tree(tag_tree)` before `document.finish()`.
- Tests in `tag_engine.rs` assert: one Part per slide; Figure gets alt text
  from the frame; decorative frames are excluded from tagged groups.

The shipped `tag_engine.rs` has the tag-group construction code essentially correct
(it already uses the real krilla API per tech-validation findings). The gap is that
`exporter.rs` never calls `tag_slide` or `set_tag_tree`. That wiring is the fix.

Note: the figure `alt=None` hardcode in `exporter.rs` is also a 043 defect — the
frame's alt text field must be passed through to `tag_figure`.

**STORY-045 adds (correctly deferred):**
- `Validator::UA1` configuration in `SerializeSettings` (krilla UA-1 mode activation).
- `document.set_metadata(Metadata { language: "en-US", .. })` for /Lang.
- `/MarkInfo << /Marked true >>` (this is emitted by krilla automatically when
  `set_tag_tree` is called AND `Validator::UA1` is active — 045 activates UA-1 mode).
- Font ToUnicode CMaps (requires text drawing — legitimately blocked by content drawing).
- veraPDF CI gate (`pdf-ua1.yml` workflow).
- The integration test that actually runs `verapdf --flavour ua1`.
- Full reading-order constraint on tag tree iteration.

In short: 043 builds the structural skeleton of the tree and wires it. 045 activates
UA-1 validator mode and adds the document-level metadata required for veraPDF to pass.
The structural tree built in 043 is a necessary precondition for 045 to have anything
to validate.

### Correction Note (2026-05-31 — verified against krilla 0.6.0 source)

The phrase "assembles the per-slide trees under a `TagKind::Document` root group" in
the Execution Order below (step 2) and in the AC-003 code snippet as originally written
is **factually wrong for krilla 0.6.0**. Source inspection of
`~/.cargo/registry/src/.../krilla-0.6.0/src/interchange/tagging/` confirms:

- `TagKind` has **NO `Document` variant** in krilla 0.6.0.
- The PDF `/Document` (StructRole::Document) root structure element is emitted
  **automatically** by krilla's `TagTree::serialize()` as the implicit top-level
  container.
- User code builds: `TagTree → Part*` (one `TagKind::Part` per slide).
- The serialized PDF structure is: `/Document (auto) → /Part* → ...`
- Attempting to create a `TagGroup::new(Tag::with(TagKind::Document))` **will not
  compile** (variant does not exist) and would double the `/Document` element if it
  somehow could.

The CORRECT implementation (TagTree → Part*) is already what the shipped `tag_engine.rs`
code does. The gap identified in Decision 1 is NOT about the Document wrapper — it is
about `tag_slide` never being called and `set_tag_tree` never being invoked from
`exporter.rs`. Those gaps are real and must be fixed.

**Superseded wording** (Execution Order step 2 below): "Assemble the per-slide trees
under a `Document`-level root group" — this is wrong. The correct action is: push each
per-slide `TagKind::Part` group directly onto the `TagTree`, then call
`document.set_tag_tree(tag_tree)`. No Document TagGroup is needed or possible.

STORY-043 AC-003 has been corrected to reflect this behavior.

### No further spec amendment needed for Decision 1.

AC-003 of STORY-043 already requires the correct behavior (as corrected above). The
implementer must implement what AC-003 specifies. No product-owner action needed.

---

## Decision 2: Content Drawing and AC-004 Font Subsetting (F-004)

### Ruling: Text drawing is STORY-044 scope. AC-004 as written is MIS-SCOPED in STORY-043.

The analysis is as follows.

STORY-044 is titled "EMU-to-PDF Coordinate Mapping + Y-Axis Flip" and its task list
states: "Update PdfExporter::export() (from STORY-043) to use emu_to_pt() and
ir_y_to_pdf_y() for all element placement." The word "all element placement" presupposes
that element placement — i.e., text and content drawing onto the krilla Surface — exists
in STORY-044. STORY-044 task 2 of "Previous Story Intelligence" reads:

> "Key invariant from STORY-043: all element placement in PdfExporter::export() was
> written with TODO comments marking where coordinate conversion goes. This story fills
> those TODOs by introducing emu_to_pt() and ir_y_to_pdf_y() and calling them from the
> element-placement loop."

This means STORY-043 is expected to leave TODO stubs for coordinate-dependent placement,
and STORY-044 fills them in. The `element-placement loop` — including text drawing — is
therefore part of STORY-044's mandate, not STORY-043's.

Reading STORY-044's AC-006 confirms: the integration test verifies all element bounding
boxes are within the slide canvas after rendering, which presupposes elements are drawn.

STORY-043's AC-004 reads: "a PDF produced for a deck using only ASCII glyphs from a
large Unicode font must embed a font program significantly smaller than the full font
file." This behavioral assertion is only verifiable if text glyphs are actually drawn
onto the Surface. Since text drawing belongs to STORY-044, the subsetting size assertion
cannot be verified in STORY-043.

### Required spec amendment (story-writer executes):

**STORY-043 AC-004 must be amended** from its current form to:

> **AC-004 (amended): Font loading into krilla — subsetting is krilla-internal.**
>
> The `font.rs` module provides font file loading helpers that make font data available
> for use with krilla's Surface API. Subsetting happens automatically when text is drawn
> via krilla's Surface in STORY-044. This story (043) establishes:
> - `font::load_font_data(path: &Path) -> Result<Vec<u8>, PdfExportError>` — reads font
>   bytes from disk.
> - `font::system_font_fallback(family: &str) -> Option<PathBuf>` — locates a font by
>   family name.
>
> No subsetting size assertion is required in STORY-043 because no glyphs are drawn in
> 043. The glyph-size regression test ("ASCII-only subset must be smaller than the full
> font") is re-scoped to STORY-044 where it becomes feasible.

**STORY-044 gains one AC** (product-owner adds to BC-4.03.005 or creates a new AC in
STORY-044):

> **STORY-044 AC-009 (new): Font subsetting exercised by text drawing.**
>
> When `PdfExporter::export()` draws text elements via krilla's Surface/text API in
> STORY-044, krilla internally subsets fonts via its `subsetter 0.2.3` dependency.
> A unit test verifies: for a deck using only ASCII glyphs from a large Unicode font,
> the embedded font program in the output PDF is smaller than the unsubsetted font file.
> No `subsetter::subset(...)` call is made in `slideforge-pdf` source code; the
> subsetting is entirely internal to krilla.

This re-scoping removes an unverifiable assertion from STORY-043 and creates a
verifiable one in STORY-044 where it belongs. No BC-4.03.002 change is needed for
contract semantics — the invariant ("font subsetting via krilla, no system tooling")
remains; only the test vehicle moves.

### What 043 must NOT defer about font.rs:

The `font.rs` module structure (loading helpers) remains a STORY-043 deliverable.
The implementer ships a working `font::load_font_data` function. What is deferred to
044 is the subsetting regression test, because that test requires text to be drawn.

---

## Decision 3: AC-008 No-Subprocess (F-006)

### Ruling: Add a source-level "no std::process" check in 043. Strace/dtrace test is legitimately 049's.

The current `test_bc_4_03_002_no_subprocess_structural_check` test is tautological:
asserting `Send + Sync` does not prove absence of subprocess spawning. A `Send + Sync`
type can trivially wrap `std::process::Command`. The adversary finding (F-006) is correct.

However, the strace/dtrace integration test in AC-008 as written requires running
`slideforge build --format pdf fixture.sf` under a syscall tracer — this requires a
complete CLI binary, a fixture file, and platform-specific tracing tools. Those
dependencies legitimately belong to STORY-049 (E2E tests).

### What 043 must add (fix in scope, no story amendment needed):

Add a compile-time `#[forbid(process)]` or equivalent structural assertion in
`slideforge-pdf`. The correct mechanism is a `grep`-equivalent CI assertion:

1. In `scripts/check-pdf-deps.sh` (already required by AC-002/AC-009), add:

   ```bash
   # Assert no std::process usage in the export path
   if grep -r "std::process\|Command::new\|process::Command" \
       crates/slideforge-pdf/src/; then
       echo "FAIL: subprocess usage found in slideforge-pdf/src/" >&2
       exit 1
   fi
   ```

2. Replace the `test_bc_4_03_002_no_subprocess_structural_check` unit test body
   with a comment explaining why structural checks are insufficient and citing
   that the source-level grep in `check-pdf-deps.sh` is the real assertion,
   plus the deferred strace test:

   ```rust
   // Source-level check: `scripts/check-pdf-deps.sh` asserts zero
   // `std::process::Command` / `std::process::Command::new` in
   // crates/slideforge-pdf/src/. That is the load-bearing assertion for AC-008.
   //
   // Full strace/dtrace integration test: deferred to STORY-049
   // test name: test_e2e_pdf_export_no_execve_syscall
   // Reason: requires full CLI binary + syscall tracer (strace/dtrace/procmon).
   ```

3. The `test_bc_4_03_002_no_subprocess_structural_check` test MAY keep its
   Send+Sync assertions as supplementary coverage, but the doc comment must not
   claim they prove absence of subprocess spawning.

### No story or BC amendment needed for Decision 3.

The strace test in AC-008 is correctly anchored to STORY-049 by the existing AC-008
text ("Integration test: run slideforge build ... under strace"). The only gap was
missing a source-level assertion in scope. The `check-pdf-deps.sh` extension covers it.

---

## Summary Table

| Finding | Decision | 043 must fix | Spec change required |
|---------|----------|--------------|----------------------|
| F-001: tag_slide never called | Fix in 043 | Wire SlideTagEngine in PdfExporter::export | None — AC-003 already requires it |
| F-002: set_tag_tree never called | Fix in 043 | Call document.set_tag_tree(tree) before finish() | None — AC-003 requires it |
| F-003: Figure alt=None hardcode | Fix in 043 | Pass frame.alt text through to tag_figure | None — AC-003 requires alt threading |
| F-008: decorative→Artifact not wired | Fix in 043 | Mark decorative frames as Artifacts in export | None — AC-003 requires it |
| F-004: AC-004 font subsetting not exercisable | Re-scope | Amend AC-004 to font loading only; move subsetting test to 044 | STORY-043 AC-004 amended; STORY-044 gains AC-009 |
| F-006: AC-008 tautological test | Fix in 043 | Add subprocess grep to check-pdf-deps.sh; update test comment citing STORY-049 | None — strace test already anchored to 049 |

---

## Execution Order for Implementer Fix Burst

1. `exporter.rs`: Wire `SlideTagEngine::tag_slide` call for each slide in the loop.
   Pass the frame's alt text (not `None`) to figure tag construction.
2. `exporter.rs`: Assemble the per-slide trees under a `Document`-level root group
   and call `document.set_tag_tree(assembled_tree)` before `document.finish()`.
3. `exporter.rs`: Add Artifact-marking logic for decorative frames (frames where
   `frame.decorative == true` or where alt is empty per the frame definition).
4. `tag_engine.rs`: If the `tag_slide` signature does not yet accept the full frame
   (to pass alt text), adjust it. The shipped code already has `tag_figure(alt: &str)`
   with `Some(alt.to_owned())` — that method is correct. The gap is calling it with
   real alt text in `tag_slide`.
5. `scripts/check-pdf-deps.sh`: Add the `std::process` grep assertion.
6. `exporter.rs` test: Update `test_bc_4_03_002_no_subprocess_structural_check`
   doc comment per Decision 3.
7. STORY-043 story file: Amend AC-004 (story-writer executes).
8. STORY-044 story file: Add AC-009 (product-owner executes).

Items 7 and 8 are spec amendments — implementer does NOT edit story files. Implementer
completes items 1–6, then notifies orchestrator to dispatch story-writer for AC-004
amendment and product-owner for STORY-044 AC-009 addition.
