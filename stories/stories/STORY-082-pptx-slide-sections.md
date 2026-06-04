---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-082
title: "PPTX: Slide-Grouping Sections (sectionLst) — DSL + IR + Eval + Exporter"
epic: EPIC-08
wave: 5
points: 5
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.01.003, BC-1.14.003]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-pptx
target_module: slideforge-pptx
subsystems: [SS-01, SS-02, SS-05, SS-06]
depends_on:
  - STORY-040
  - STORY-078
blocks: []
estimated_days: 2
---

# STORY-082: PPTX: Slide-Grouping Sections (sectionLst) — DSL + IR + Eval + Exporter

## Subsystem Anchor Justifications

- **SS-01 (slideforge-syntax)**: The `section "Name":` slide-grouping construct is a new
  DSL syntax element requiring parser work. SS-01 owns all DSL grammar changes.
- **SS-02 (slideforge-eval)**: The evaluator must map slides to their enclosing
  `section "Name":` group and populate `LaidOutDeck.slide_sections`. SS-02 owns eval-stage
  IR population.
- **SS-05 (slideforge-layout)**: A new `slide_sections` field is added to `LaidOutDeck` in
  `slideforge-layout`. SS-05 owns the IR struct definition.
- **SS-06 (slideforge-pptx)**: The `SectionListBuilder` reads `slide_sections` and emits
  `<p:sectionLst>` in `presentation.xml`. SS-06 owns the exporter output.

## Dependency Anchor Justifications

- Depends on STORY-040 (Speaker Notes + notesMaster): STORY-040 establishes the
  `presentation.xml` serialization structure (`PresentationSerializer`) and the ZIP
  assembly pipeline. STORY-082 extends `PresentationSerializer` with `<p:sectionLst>`.
  Building on STORY-040's exporter base is the correct layering; doing it before would
  require rework.
- Depends on STORY-078 (Parser: section block syntax): STORY-078 established the
  `section <type>:` document-structure block parse path and the disambiguation rules. The
  `section "Name":` slide-grouping syntax is syntactically distinct (quoted-string name,
  slide children) and must not conflict with STORY-078's bare-ident form. Reading STORY-078
  before writing this parser change is required to avoid a grammar collision.
- Blocks nothing (provisional): STORY-049 and STORY-050 do not assert sectionLst
  specifically (confirmed by grep: neither file references `sectionLst` or `slide_sections`).
  If STORY-049/050 are later scoped to include sectionLst integration tests, their
  `depends_on` fields should be updated to include STORY-082.

## Summary

This story delivers the complete cross-crate pipeline for PPTX slide-grouping sections:
a new `section "Name":` DSL construct, a new `slide_sections` IR field, an eval-stage
membership mapping, and a `SectionListBuilder` PPTX exporter. It is the "Half B" delivery
of BC-4.01.003 (human-authorized scope split 2026-06-04).

### Scope Overview

1. **New DSL construct** (`crates/slideforge-syntax`): `section "Name":` — a quoted-string
   slide-grouping block that contains slides as children. This is syntactically DISTINCT
   from the existing `section IDENT:` document-structure block (STORY-078). The parser must
   disambiguate: bare-ident after `section` → document-structure block (existing, STORY-078);
   quoted-string after `section` → slide-grouping block (new, this story).

2. **New IR field** (`crates/slideforge-layout`): `slide_sections: Vec<SlideSectionEntry>`
   on `LaidOutDeck`. `SlideSectionEntry` holds `name: Arc<str>` + `slide_ids: Vec<u32>`.
   This field MUST NOT reuse `LaidOutDeck.sections` (which carries DOCX/PDF
   `GeneratedSection` entries — a different concept at a different abstraction level).

3. **Eval mapping** (`crates/slideforge-eval`): The evaluator maps slides to their enclosing
   `section "Name":` group by position in the parsed deck, then populates
   `LaidOutDeck.slide_sections`. Slides not inside any `section "Name":` group are ungrouped
   (absent from `slide_sections`).

4. **PPTX exporter** (`crates/slideforge-pptx`): `SectionListBuilder` reads
   `LaidOutDeck.slide_sections` and emits `<p:sectionLst>` inside `<p:presentation>`.
   GUIDs are generated deterministically from section names via `sha2 =0.10.9`, NOT
   `Uuid::new_v4()`. Section names are XML-escaped. `<p:sectionLst>` is omitted when
   `slide_sections` is empty.

### BC-1.14.003 Non-Interference Constraint

`<p:sectionLst>` is built EXCLUSIVELY from:
- The section name (the quoted string in `section "Name":`)
- Slide membership (slide IDs that fall under that section)

It MUST NOT read or embed:
- Section body content (`report:`, `notes:`, `detail:` sub-fields)
- `RegisteredContent` nodes of any kind from BC-3.02.002

BC-1.14.003 postconditions 3 and 5 remain fully intact: detail register content does not
appear in PPTX under any circumstances, including inside `<p:sectionLst>` attributes or
extension data.

### Multi-Renderer Parity Exception (Human-Accepted 2026-06-04)

`<p:sectionLst>` is classified as "Fair — PPT only". It renders as slide navigation
sections in PowerPoint 365 only; Keynote, Google Slides, and LibreOffice ignore it without
rendering corruption. This is a documented, human-accepted departure from the full
multi-renderer parity gate. Test coverage for sectionLst is restricted to
PowerPoint 365 rendering verification. Multi-renderer parity tests for this element are
explicitly excluded.

### sectionLst XML Structure

```xml
<p:sectionLst>
  <p:section name="Background" id="{DETERMINISTIC-GUID}">
    <p:sldIdLst>
      <p:sldId id="256"/>
      <p:sldId id="257"/>
    </p:sldIdLst>
  </p:section>
  <p:section name="Analysis" id="{DETERMINISTIC-GUID}">
    <p:sldIdLst>
      <p:sldId id="258"/>
      <p:sldId id="259"/>
    </p:sldIdLst>
  </p:section>
</p:sectionLst>
```

GUIDs are derived deterministically: `sha2::Sha256::digest(section_name_bytes)`, then
formatted as a UUID v5-style hex string. Same section name → same GUID → byte-identical
output across builds.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.01.003 | PPTX: slide sections (Half B — postcondition 5, invariants 3-4, EC-002/004/005/006) | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007 |
| BC-1.14.003 | Register routing non-interference — sectionLst must not carry register content | AC-008 |

## Acceptance Criteria

### AC-001: section "Name": construct parses successfully
(traces to BC-4.01.003 precondition 5 — quoted-name DSL construct parsed; distinct from bare-ident form)

A source file containing `section "Background":` followed by slide children is parsed
without error. The resulting AST contains a `SectionGroupNode { name: "Background",
slides: [...] }` (or equivalent named type). A unit test in `slideforge-syntax` asserts
the AST node type and name string.

### AC-002: section "Name": is distinct from section IDENT:
(traces to BC-4.01.003 precondition 5 — no grammar collision with STORY-078 document-structure blocks)

A file containing both `section methodology:` (document-structure, bare-ident) and
`section "Background":` (slide-grouping, quoted string) parses successfully and produces
two distinct AST node types. A unit test asserts that the bare-ident form produces a
`SectionBlock` (STORY-078 type) and the quoted-string form produces a `SectionGroupNode`
(this story's type), with no cross-contamination.

### AC-003: Slides produce sectionLst in presentation.xml
(traces to BC-4.01.003 postcondition 5 — sectionLst groups slides by section)

A deck with `section "Background":` (slides 1-2) and `section "Analysis":` (slides 3-4)
produces `<p:sectionLst>` in `presentation.xml` with two `<p:section>` elements. The
section names match the DSL declarations. A unit test parses `presentation.xml` from the
ZIP and asserts `sectionLst.sections.len() == 2` and section names are correct.

### AC-004: Deck with no section groupings has no sectionLst
(traces to BC-4.01.003 edge case EC-002 — no sections → no sectionLst)

A deck with no `section "Name":` slide-grouping blocks produces `presentation.xml`
without a `<p:sectionLst>` element. A unit test parses `presentation.xml` and asserts
`sectionLst` is absent.

### AC-005: XML-escaped section names
(traces to BC-4.01.003 edge case EC-004 — section name with special chars)

A section named `"Background & Overview"` produces
`<p:section name="Background &amp; Overview">` in the presentation XML. The XML is
well-formed (parseable without error).

### AC-006: Two single-slide sections both present
(traces to BC-4.01.003 edge case EC-005 — two sections with one slide each)

A deck with two `section "Name":` blocks each containing exactly one slide produces
a `<p:sectionLst>` with two `<p:section>` entries. Each section's `<p:sldIdLst>` contains
exactly one `<p:sldId>`. A unit test asserts `sections.len() == 2` and each section has
exactly one slide reference.

### AC-007: sectionLst skipped for non-PPTX formats
(traces to BC-4.01.003 edge case EC-006 — PPT-only element; DOCX/HTML/PDF unaffected)

A deck with `section "Name":` slide-grouping blocks, when built to DOCX or HTML, does
not include sectionLst logic. The DOCX and HTML output is unaffected. A unit test builds
the same deck to DOCX and asserts no `sectionLst` or `SlideSectionEntry` processing
occurs in those exporters.

### AC-008: sectionLst carries no register content
(traces to BC-1.14.003 postcondition 3 + postcondition 5 — non-interference)

A deck with `section "Background":` that also contains `detail:` and `report:` register
content in the section body produces a `<p:sectionLst>` that contains ONLY the section
name and slide IDs. A unit test parses the sectionLst XML and asserts no text from the
`detail:` or `report:` register appears in any attribute or element within `<p:sectionLst>`.

### AC-009: Deterministic section GUIDs
(traces to BC-4.01.003 invariant 3 — section data not fabricated; determinism required for reproducible builds)

Two identical builds of the same deck produce byte-identical `<p:section id="...">` GUID
attributes. The GUID is derived from the section name via `sha2` hash — not `Uuid::new_v4()`.
A unit test builds the same deck twice and asserts the sectionLst XML is identical. A
second test asserts the GUID for `section "Background":` is the same value on both runs.

## Tasks

- [ ] **slideforge-syntax**: Add `SectionGroupNode` AST node for `section "Name":` construct
  - Parser disambiguates: quoted-string after `section` → `SectionGroupNode`; bare-ident → existing `SectionBlock` (STORY-078)
  - `SectionGroupNode` carries `name: Arc<str>` + `Vec<SlideNode>` children
  - Update error recovery: unknown content under `section "Name":` yields a structured error
- [ ] **slideforge-layout**: Add `SlideSectionEntry` struct and `slide_sections: Vec<SlideSectionEntry>` field to `LaidOutDeck`
  - `SlideSectionEntry { name: Arc<str>, slide_ids: Vec<u32> }` (must derive `Hash + Eq + Clone`)
  - Do NOT repurpose or reuse `LaidOutDeck.sections` — that field is for DOCX `GeneratedSection` entries
- [ ] **slideforge-eval**: Add eval-stage mapping from `SectionGroupNode` to `LaidOutDeck.slide_sections`
  - Walk deck's parsed nodes; for each `SectionGroupNode`, collect its slide children and their assigned slide IDs
  - Populate `LaidOutDeck.slide_sections` in declaration order
  - Slides not inside any `section "Name":` block remain ungrouped (absent from `slide_sections`)
- [ ] **slideforge-pptx**: Implement `SectionListBuilder` in `src/sections.rs`
  - Accept `&[SlideSectionEntry]`
  - Generate `<p:sectionLst>` XML with one `<p:section>` per entry
  - Deterministic GUID derivation from section name: `sha2::Sha256` hash, formatted as UUID-like hex string
  - XML-escape section names
  - Return `None` when input is empty (caller omits `<p:sectionLst>`)
- [ ] Update `PresentationSerializer` to call `SectionListBuilder` and include `<p:sectionLst>` when sections exist
- [ ] Write unit tests for all nine ACs

## Previous Story Intelligence

STORY-040 established the `PresentationSerializer` and `ZipAssembler` infrastructure.
This story adds `SectionListBuilder` as a new module called from `PresentationSerializer`.
The `SectionListBuilder` is the sole consumer of `LaidOutDeck.slide_sections`.

STORY-078 established the `section <type>:` bare-ident parse path. The parser modification
here must preserve all STORY-078 behavior exactly — only the quoted-string branch is new.
Review STORY-078's `section_block()` combinator before writing the disambiguation logic.

STORY-035/036 established register routing. The `slide_sections` field carries ONLY
structural information (name + slide IDs), never `RegisteredContent` entries. The eval
mapping for `slide_sections` is intentionally decoupled from the `extract_section_register_content`
function from STORY-077.

## Architecture Compliance Rules

1. **sectionLst from slide_sections only (BC-4.01.003 invariant 3)**: `SectionListBuilder`
   reads ONLY `LaidOutDeck.slide_sections`. It MUST NOT access `LaidOutDeck.sections`
   (DOCX GeneratedSection), `RegisteredContent`, or any content block fields.
2. **Non-interference (BC-1.14.003 postconditions 3 + 5)**: No detail/report/notes text
   appears in `<p:sectionLst>` attributes or child elements. `BleedChecker` should be
   extended to assert absence from `<p:sectionLst>` nodes.
3. **sectionLst absent when no groupings (BC-4.01.003 invariant 4)**: `SectionListBuilder`
   returns `None` (or equivalent) when `slide_sections` is empty; `PresentationSerializer`
   does not emit `<p:sectionLst>` in that case.
4. **Deterministic GUIDs (BC-4.01.003 invariant 3)**: `Uuid::new_v4()` is FORBIDDEN. Use
   `sha2` hash of section name. Same name → same GUID → reproducible build.
5. **Grammar disambiguation (no regression to STORY-078)**: The bare-ident `section` form
   must continue to parse into `SectionBlock` exactly as it did before this story. Unit
   tests from STORY-078 must all pass without modification.
6. **slide_sections vs sections separation**: `LaidOutDeck.slide_sections` is a new field.
   Do NOT alias, merge, or cast from `LaidOutDeck.sections`. These are different concepts at
   different abstraction levels (slide groupings vs. DOCX document sections).
7. **PPT-only element scope**: `SectionListBuilder` is called ONLY from `slideforge-pptx`.
   The `slideforge-docx`, `slideforge-html`, and `slideforge-pdf` exporters MUST NOT import
   or reference it. A forbidden-dependency build test should verify this.

## Forbidden Dependencies

- `slideforge-pptx` must not gain deps on `slideforge-docx`, `slideforge-html`, or
  `slideforge-pdf` (these are sibling crates; exporter cross-deps are forbidden).
- `SectionListBuilder` must not be imported from any crate other than `slideforge-pptx`.
- `slideforge-layout`'s new `SlideSectionEntry` type is permitted in `slideforge-eval`
  (eval populates it) and `slideforge-pptx` (exporter reads it).

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `ooxmlsdk` | `=0.6.1` | sectionLst XML construction (already in slideforge-pptx) |
| `zip` | `=4.2.0` | ZIP assembly (already in slideforge-pptx) |
| `sha2` | `=0.10.9` | Deterministic GUID derivation from section names |
| `chumsky` | `=0.10.1` | Parser extension (already in slideforge-syntax) |
| `slideforge-types` | workspace | `SlideSectionEntry`, `LaidOutDeck` |

Note: `sha2 =0.10.9` is added to `slideforge-pptx/Cargo.toml` in this story (NOT in
STORY-040). It was intentionally deferred from STORY-040 per the scope split.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-syntax/src/parser/section_group.rs` | Create | `SectionGroupNode` AST node + `section "Name":` parser combinator |
| `crates/slideforge-syntax/src/parser/section_block.rs` | Modify | Disambiguation: quoted-string → SectionGroupNode; bare-ident → existing SectionBlock |
| `crates/slideforge-layout/src/lib.rs` | Modify | Add `SlideSectionEntry` struct; add `slide_sections: Vec<SlideSectionEntry>` to `LaidOutDeck` |
| `crates/slideforge-eval/src/section_groups.rs` | Create | Eval-stage mapping: `SectionGroupNode` → `LaidOutDeck.slide_sections` population |
| `crates/slideforge-pptx/src/sections.rs` | Create | `SectionListBuilder` |
| `crates/slideforge-pptx/src/presentation.rs` | Modify | Call `SectionListBuilder`, emit `<p:sectionLst>` when sections exist |
| `crates/slideforge-pptx/src/tests/sections_tests.rs` | Create | AC-001 through AC-009 tests |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,000 |
| BC-4.01.003 (Half B postconditions + invariants 3-4 + EC-002/004/005/006) | ~1,200 |
| BC-1.14.003 (postconditions 3 + 5) | ~800 |
| STORY-040 notes_master.rs + presentation.rs reference | ~1,000 |
| STORY-078 section_block.rs reference (grammar disambiguation) | ~1,200 |
| STORY-035 RegisteredContent types (non-interference verification) | ~600 |
| New files: section_group.rs + section_groups.rs + sections.rs | ~2,500 |
| Test files | ~2,000 |
| **Total** | **~12,300** |

## Test Strategy

- **Unit tests (slideforge-syntax)**: AC-001 (parse `section "Name":`), AC-002 (disambiguation
  from bare-ident form; regression for all STORY-078 section-block tests).
- **Unit tests (slideforge-layout)**: `SlideSectionEntry` hash/eq/clone derivation; field
  presence on `LaidOutDeck`.
- **Unit tests (slideforge-eval)**: Eval correctly maps slides to section groups; ungrouped
  slides are absent from `slide_sections`.
- **Unit tests (slideforge-pptx)**: AC-003 (sectionLst present with 2 sections), AC-004 (no
  sectionLst when empty), AC-005 (XML-escaped names), AC-006 (two single-slide sections),
  AC-007 (non-PPTX formats unaffected), AC-008 (no register content in sectionLst), AC-009
  (deterministic GUIDs — build twice, assert identical).
- **Snapshot test**: `presentation.xml` for a 4-slide deck with 2 named sections.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-002 | Deck with no `section "Name":` slide-grouping blocks | No `<p:sectionLst>` in presentation.xml |
| EC-004 | Section name with XML special characters (`&`, `<`, `>`) | XML-escaped in `<p:section name>` attribute |
| EC-005 | Two sections each containing one slide | Both sections in sectionLst; each with one sldId |
| EC-006 | `section "Name":` present but deck built for DOCX/HTML/PDF | sectionLst logic skipped; non-PPTX output unaffected |
| EC-010 | Section name that is an empty string `""` | Validation error: E-PAR-NNN "section group name must be non-empty" (new error code — add to error taxonomy) |
| EC-011 | Two `section "Name":` blocks with identical names | Validation warning: duplicate section name; both emitted with same GUID (deterministic per-name); no crash |
