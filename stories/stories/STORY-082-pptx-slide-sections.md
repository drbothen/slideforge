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
spec_version: "1.2"
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
- **SS-06 (slideforge-pptx)**: The `SectionListBuilder` receives the serialized
  `presentation.xml` bytes from `PresentationSerializer`, constructs the
  `p:extLst`/`p14:sectionLst` block via raw-XML injection (quick-xml), and injects it
  immediately before `</p:presentation>`. SS-06 owns the exporter output.

## Dependency Anchor Justifications

- Depends on STORY-040 (Speaker Notes + notesMaster): STORY-040 establishes the
  `presentation.xml` serialization structure (`PresentationSerializer`) and the ZIP
  assembly pipeline. STORY-082 extends `PresentationSerializer` with the `p14:sectionLst` extension block (via `p:extLst` raw-XML injection).
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

4. **PPTX exporter** (`crates/slideforge-pptx`): `SectionListBuilder` receives the
   `presentation.xml` bytes produced by `PresentationSerializer::build()`. If
   `slide_sections` is empty it returns the bytes unchanged (no `p:extLst` injected).
   Otherwise it constructs the `p:extLst` / `p14:sectionLst` block with `quick-xml`,
   injects it immediately before `</p:presentation>`, and patches the
   `<p:presentation` opening tag to declare `xmlns:p14`. This raw-XML injection approach
   is required because `ooxmlsdk =0.6.1` has no typed `p14` structs and silently drops
   unknown extension children (same pattern as the existing W1/W2 post-processing
   workarounds in this crate). GUIDs are generated deterministically from section names
   via the `sha2 =0.11.0` algorithm below — NOT `Uuid::new_v4()`. Section names are
   XML-escaped.

### BC-1.14.003 Non-Interference Constraint

`<p14:sectionLst>` (emitted inside `<p:extLst>/<p:ext>`) is built EXCLUSIVELY from:
- The section name (the quoted string in `section "Name":`)
- Slide membership (slide IDs that fall under that section)

It MUST NOT read or embed:
- Section body content (`report:`, `notes:`, `detail:` sub-fields)
- `RegisteredContent` nodes of any kind from BC-3.02.002

BC-1.14.003 postconditions 3 and 5 remain fully intact: detail register content does not
appear in PPTX under any circumstances, including inside `<p:extLst>`, `<p14:sectionLst>`,
`<p14:section>` attributes, or `<p14:sldIdLst>` elements.

### Multi-Renderer Parity Exception (Human-Accepted 2026-06-04)

`p14:sectionLst` (inside `p:extLst`) is classified as "Fair — PPT only". It renders as
slide navigation sections in PowerPoint 365 only; Keynote, Google Slides, and LibreOffice
ignore the extension block without rendering corruption. This is a documented,
human-accepted departure from the full multi-renderer parity gate. Test coverage for the
sectionLst extension is restricted to PowerPoint 365 rendering verification.
Multi-renderer parity tests for this element are explicitly excluded.

### sectionLst XML Structure

PowerPoint slide sections are stored in the **Microsoft PowerPoint 2010 extension
namespace** (`p14`), not as a bare `p:sectionLst`. The bare `<p:sectionLst>` form does
not exist in the ECMA-376 schema for `p:presentation`; PowerPoint 365 would not
recognize it. The authoritative structure is:

```xml
<p:presentation
    xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
    xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main"
    ...>
  <!-- existing children: p:sldMasterIdLst, p:sldIdLst, p:sldSz, p:notesSz,
       p:defaultTextStyle, etc. -->
  <p:extLst>
    <p:ext uri="{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}">
      <p14:sectionLst xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main">
        <p14:section name="Background" id="{XXXXXXXX-XXXX-5XXX-AXXX-XXXXXXXXXXXX}">
          <p14:sldIdLst>
            <p14:sldId id="256"/>
            <p14:sldId id="257"/>
          </p14:sldIdLst>
        </p14:section>
        <p14:section name="Analysis" id="{YYYYYYYY-YYYY-5YYY-AYYY-YYYYYYYYYYYY}">
          <p14:sldIdLst>
            <p14:sldId id="258"/>
            <p14:sldId id="259"/>
          </p14:sldIdLst>
        </p14:section>
      </p14:sectionLst>
    </p:ext>
  </p:extLst>
</p:presentation>
```

Key facts:
- `p:ext uri` is the fixed constant `{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}` — the
  sectionLst extension slot identifier. Same value in every file; not per-deck generated.
- `p14` namespace URI: `http://schemas.microsoft.com/office/powerpoint/2010/main`. The
  root `<p:presentation>` opening tag MUST declare `xmlns:p14` (preferred; canonical
  Office form).
- `<p:extLst>` MUST be the LAST child element of `<p:presentation>` (CT_Presentation
  is an ordered `sequence`; `p:extLst` is the terminal optional element). Element
  ordering is schema-significant; inserting `p:extLst` earlier produces a schema-invalid
  document.
- `SectionListBuilder` enforces this by injecting the `p:extLst` block immediately
  before the closing `</p:presentation>` tag in the raw bytes.

#### Deterministic GUID Derivation

`p14:section/@id` is a brace-wrapped, hyphen-separated, UPPERCASE hex GUID derived
deterministically from the section name:

```
raw = sha2::Sha256::digest(section_name.as_bytes())   // 32 bytes
uuid_bytes[0..16] = raw[0..16]
// Set version nibble (byte 6, high nibble) = 5  (UUID v5 convention)
uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x50
// Set variant bits (byte 8, high 2 bits) = 10  (RFC 4122 variant)
uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80
// Format as: {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}  (uppercase)
```

Same section name → same GUID → byte-identical output across builds. `Uuid::new_v4()`
is FORBIDDEN (non-deterministic; violates BC-4.01.003 invariant 3).

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.01.003 v1.4 | PPTX: slide sections (Half B — postcondition 5, postcondition 7, postcondition 8, invariants 3-5, EC-002/004/005/006/010/011) | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007, AC-009, AC-010, AC-011 |
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

### AC-003: Slides produce p14:sectionLst in presentation.xml via p:extLst
(traces to BC-4.01.003 postcondition 5 — sectionLst groups slides by section)

A deck with `section "Background":` (slides 1-2) and `section "Analysis":` (slides 3-4)
produces a `<p:extLst>` containing a `<p:ext uri="{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}">`
containing a `<p14:sectionLst>` in `presentation.xml`, with two `<p14:section>` elements.
Section names match the DSL declarations. The `<p:presentation>` opening tag declares
`xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main"`. A unit test
parses `presentation.xml` from the ZIP and asserts: (a) `p:extLst` is the last child of
`p:presentation`; (b) `p:ext/@uri` equals `{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}`;
(c) `p14:sectionLst` has two `p14:section` children with correct names.

### AC-004: Deck with no section groupings has no extLst/sectionLst
(traces to BC-4.01.003 edge case EC-002 — no sections → no sectionLst)

A deck with no `section "Name":` slide-grouping blocks produces `presentation.xml`
without any `<p:extLst>` or `<p14:sectionLst>` element. `SectionListBuilder` returns
the bytes unchanged. A unit test parses `presentation.xml` and asserts both
`p:extLst` and `p14:sectionLst` are absent.

### AC-005: XML-escaped section names in p14:section
(traces to BC-4.01.003 edge case EC-004 — section name with special chars)

A section named `"Background & Overview"` produces
`<p14:section name="Background &amp; Overview">` in the presentation XML. The XML is
well-formed (parseable without error by a standards-compliant XML parser).

### AC-006: Two single-slide sections both present in p14:sectionLst
(traces to BC-4.01.003 edge case EC-005 — two sections with one slide each)

A deck with two `section "Name":` blocks each containing exactly one slide produces
a `<p14:sectionLst>` with two `<p14:section>` entries. Each section's `<p14:sldIdLst>`
contains exactly one `<p14:sldId>`. A unit test asserts that `p14:sectionLst` has two
`p14:section` children and each has exactly one `p14:sldId` child.

### AC-007: sectionLst skipped for non-PPTX formats
(traces to BC-4.01.003 edge case EC-006 — PPT-only element; DOCX/HTML/PDF unaffected)

A deck with `section "Name":` slide-grouping blocks, when built to DOCX or HTML, does
not include sectionLst logic. The DOCX and HTML output is unaffected. A unit test builds
the same deck to DOCX and asserts no `sectionLst` or `SlideSectionEntry` processing
occurs in those exporters.

### AC-008: p14:sectionLst carries no register content
(traces to BC-1.14.003 postcondition 3 + postcondition 5 — non-interference)

A deck with `section "Background":` that also contains `detail:` and `report:` register
content in the section body produces a `<p14:sectionLst>` that contains ONLY the section
name and slide IDs. A unit test parses the `p14:sectionLst` XML and asserts no text from
the `detail:` or `report:` register appears in any attribute or element within
`<p:extLst>`, `<p14:sectionLst>`, `<p14:section>`, or `<p14:sldIdLst>`.

### AC-009: Deterministic section GUIDs in p14:section
(traces to BC-4.01.003 invariant 3 — section data not fabricated; determinism required for reproducible builds)

Two identical builds of the same deck produce byte-identical `<p14:section id="...">` GUID
attributes. The GUID is derived from the section name via the SHA-256 + UUID v5-like
derivation algorithm (version nibble = 5, RFC 4122 variant bits). `Uuid::new_v4()` is
FORBIDDEN. A unit test builds the same deck twice and asserts the `p14:sectionLst` XML
is identical. A second test asserts the GUID for `section "Background":` is the same
brace-wrapped uppercase hex value on both runs.

### AC-010: Empty section name is rejected with E-PAR-023
(traces to BC-4.01.003 postcondition 7 + invariant 5 — empty name rejected at parse time)

A source file containing `section "":` (empty quoted name) produces a parse error
`E-PAR-023` (`ParseError::EmptySectionGroupName`), exits with code 1 in strict mode,
and includes the message `section group name must be non-empty at <file>:<line>:<col>.
Provide a quoted, non-empty name, e.g. section "Background":`. The error is accumulated
(parsing continues to find additional errors). No `SectionGroupNode` is produced for
the rejected block. The span points at the opening `"` of the empty name token. A unit
test in `slideforge-syntax` asserts: (a) the error variant is
`ParseError::EmptySectionGroupName`; (b) the span is correct; (c) no `SectionGroupNode`
appears in the resulting AST for that block.

### AC-011: Duplicate section names emit W-PAR-002 and produce same GUID
(traces to BC-4.01.003 postcondition 8 + invariant 5 — duplicate name yields warning + identical GUID; no crash)

A source file containing two `section "Background":` blocks produces a warning
`W-PAR-002` (`ParseWarning::DuplicateSectionGroupName`) with message
`warning: [W-PAR-002] Duplicate section group name 'Background' at <file>:<line>:<col>.
Both sections are emitted with the same GUID. Consider using distinct names.` The build
exits with code 0 (cosmetic; warning does not block the build). Both `SectionGroupNode`
entries are produced; both appear in `LaidOutDeck.slide_sections` with the same
deterministic GUID (identical name → identical SHA-256 derivation). A unit test asserts:
(a) warning variant is `ParseWarning::DuplicateSectionGroupName`; (b) two sections
present in output IR; (c) both `p14:section/@id` attributes are identical strings.

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
- [ ] **slideforge-pptx**: Promote `quick-xml =0.36.0` from dev-dependency to production
  dependency in `slideforge-pptx/Cargo.toml` (required for raw-XML injection in
  `SectionListBuilder`; previously dev-only)
- [ ] **slideforge-pptx**: Add `sha2 = { workspace = true }` as a production dependency
  in `slideforge-pptx/Cargo.toml` (workspace pin `=0.11.0`, shared with slideforge-math;
  deferred from STORY-040 scope split — already consumed via `workspace = true`, no
  version override needed at the crate level)
- [ ] **slideforge-syntax**: Implement empty-name detection in the `section "Name":`
  parser combinator; emit `ParseError::EmptySectionGroupName` (E-PAR-023) and produce
  no `SectionGroupNode` for the rejected block
- [ ] **slideforge-syntax**: Implement duplicate-name detection (`seen_names: HashSet<Arc<str>>`
  across slide-grouping blocks); emit `ParseWarning::DuplicateSectionGroupName` (W-PAR-002)
  on each collision; continue emitting both sections to AST
- [ ] **slideforge-pptx**: Implement `SectionListBuilder` in `src/sections.rs`
  - Accept `presentation_xml_bytes: Vec<u8>` + `sections: &[SlideSectionEntry]`
  - If `sections` is empty, return bytes unchanged (no `p:extLst` injection)
  - Build `p:extLst` / `p14:sectionLst` block using `quick-xml` Writer:
    - Fixed `p:ext uri="{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}"`
    - `xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main"` on `p14:sectionLst`
    - One `p14:section` per `SlideSectionEntry` with deterministic GUID and XML-escaped name
    - One `p14:sldId` per slide ID inside `p14:sldIdLst`
  - Locate closing `</p:presentation>` in bytes; inject `p:extLst` block immediately before it
  - Patch `<p:presentation` opening tag to add `xmlns:p14="..."` declaration
  - Deterministic GUID: SHA-256 → first 16 bytes → set version nibble=5, variant=RFC4122 → uppercase braced format
- [ ] Update `PresentationSerializer` to pass its serialized `presentation.xml` bytes
  through `SectionListBuilder` when building the ZIP
- [ ] Write unit tests for all eleven ACs (AC-001 through AC-011)

## Previous Story Intelligence

STORY-040 established the `PresentationSerializer` and `ZipAssembler` infrastructure,
including the W1/W2 post-processing pattern (raw-XML injection into ooxmlsdk-serialized
bytes for features `ooxmlsdk` cannot type). This story adds `SectionListBuilder` as a new
module following the same W2 pattern: receive the `presentation.xml` bytes from
`PresentationSerializer::build()`, inject the `p:extLst`/`p14:sectionLst` block via
`quick-xml`, and return the modified bytes. The `SectionListBuilder` is the sole consumer
of `LaidOutDeck.slide_sections`. Review STORY-040's `opc_postprocess.rs` before
implementing to stay consistent with the injection approach.

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
   appears in `<p14:sectionLst>` attributes or child elements. `BleedChecker` should be
   extended to assert absence from `<p:extLst>` / `<p14:sectionLst>` nodes.
3. **p14 extension namespace required (no bare p:sectionLst)**: The sectionLst MUST be
   emitted under `p:extLst` / `p:ext` with the fixed URI `{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}`
   and using the `p14` namespace (`http://schemas.microsoft.com/office/powerpoint/2010/main`).
   The bare `<p:sectionLst>` form is FORBIDDEN — it does not exist in the ECMA-376 schema
   and PowerPoint 365 would not recognize it.
4. **p:extLst must be last child of p:presentation (element ordering)**: CT_Presentation is
   an ordered sequence. `p:extLst` is the terminal optional element. Injecting it before
   `p:sldIdLst`, `p:sldSz`, etc. produces a schema-invalid document. `SectionListBuilder`
   enforces this by injecting immediately before `</p:presentation>`.
5. **Raw-XML injection via quick-xml (ooxmlsdk cannot emit p14)**: `ooxmlsdk =0.6.1` has no
   typed `p14` structs and silently drops unknown ext children. `SectionListBuilder` MUST
   use `quick-xml =0.36.0` for constructing the `p:extLst` block and inject it into the
   serialized bytes — consistent with the W1/W2 post-processing pattern in this crate.
   Attempting to emit `p14:sectionLst` through the `ooxmlsdk` typed API is FORBIDDEN.
6. **sectionLst absent when no groupings (BC-4.01.003 invariant 4)**: `SectionListBuilder`
   returns the bytes unchanged when `slide_sections` is empty; no `p:extLst` or
   `p14:sectionLst` is emitted in that case.
7. **Deterministic GUIDs (BC-4.01.003 invariant 3)**: `Uuid::new_v4()` is FORBIDDEN. Use
   the SHA-256 + UUID v5-like derivation (version nibble=5, RFC 4122 variant bits). Same
   name → same GUID → reproducible build. Output in uppercase brace-wrapped format.
8. **Grammar disambiguation (no regression to STORY-078)**: The bare-ident `section` form
   must continue to parse into `SectionBlock` exactly as it did before this story. Unit
   tests from STORY-078 must all pass without modification.
9. **slide_sections vs sections separation**: `LaidOutDeck.slide_sections` is a new field.
   Do NOT alias, merge, or cast from `LaidOutDeck.sections`. These are different concepts at
   different abstraction levels (slide groupings vs. DOCX document sections).
10. **PPT-only element scope**: `SectionListBuilder` is called ONLY from `slideforge-pptx`.
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
| `ooxmlsdk` | `=0.6.1` | Base `p:presentation` serialization (already in slideforge-pptx). Does NOT handle p14 ext — raw injection required. |
| `quick-xml` | `=0.36.0` | Build `p:extLst`/`p14:sectionLst` block and inject into presentation bytes. **Promote from dev-dep to production dep** of `slideforge-pptx` in this story. |
| `sha2` | `=0.11.0` | Deterministic GUID derivation from section names. **Add as production dep** of `slideforge-pptx` via `sha2 = { workspace = true }` (workspace pin `=0.11.0`, shared with slideforge-math; deferred from STORY-040 scope split). |
| `zip` | `=4.2.0` | ZIP assembly (already in slideforge-pptx) |
| `chumsky` | `=0.10.1` | Parser extension (already in slideforge-syntax) |
| `slideforge-types` | workspace | `SlideSectionEntry`, `LaidOutDeck` |

Notes:
- `sha2 =0.11.0`: canonical workspace pin (line 73 of root Cargo.toml), shared with
  slideforge-math (STORY-030). Was intentionally deferred from STORY-040 per the scope
  split; added as `sha2 = { workspace = true }` production dep in this story — resolves
  to `=0.11.0` via workspace inheritance. No per-crate version override is used.
- `quick-xml =0.36.0`: was a dev-dep; promoted to production dep in this story because
  `SectionListBuilder`'s raw-XML injection runs in production code, not only tests.
- `ooxmlsdk =0.6.1`: retained, but does NOT handle the `p14` extension namespace. The
  typed API silently drops unknown extension elements. Raw injection via `quick-xml` is
  the correct implementation path (consistent with W1/W2 post-processing pattern).

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-syntax/src/parser/section_group.rs` | Create | `SectionGroupNode` AST node + `section "Name":` parser combinator |
| `crates/slideforge-syntax/src/parser/section_block.rs` | Modify | Disambiguation: quoted-string → SectionGroupNode; bare-ident → existing SectionBlock |
| `crates/slideforge-layout/src/lib.rs` | Modify | Add `SlideSectionEntry` struct; add `slide_sections: Vec<SlideSectionEntry>` to `LaidOutDeck` |
| `crates/slideforge-eval/src/section_groups.rs` | Create | Eval-stage mapping: `SectionGroupNode` → `LaidOutDeck.slide_sections` population |
| `crates/slideforge-pptx/src/sections.rs` | Create | `SectionListBuilder` — builds `p:extLst`/`p14:sectionLst` XML block via `quick-xml`; deterministic GUID derivation via `sha2`; injects block + `xmlns:p14` into presentation bytes |
| `crates/slideforge-pptx/src/presentation.rs` | Modify | Pass serialized `presentation.xml` bytes through `SectionListBuilder` post-processing step; `xmlns:p14` is injected here when sections exist |
| `crates/slideforge-pptx/Cargo.toml` | Modify | Add `quick-xml = { workspace = true }` and `sha2 = { workspace = true }` to `[dependencies]` (workspace pins `=0.36.0` and `=0.11.0` respectively) |
| `crates/slideforge-pptx/src/tests/sections_tests.rs` | Create | AC-001 through AC-011 tests |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,500 |
| BC-4.01.003 v1.4 (Half B postconditions 5/7/8 + invariants 3-5 + EC-002/004/005/006/010/011) | ~1,400 |
| BC-1.14.003 (postconditions 3 + 5) | ~800 |
| export-architecture.md §PPTX Slide Sections (raw-injection impl decision) | ~800 |
| error-taxonomy E-PAR-023 + W-PAR-002 notes | ~400 |
| STORY-040 notes_master.rs + presentation.rs reference | ~1,000 |
| STORY-078 section_block.rs reference (grammar disambiguation) | ~1,200 |
| STORY-035 RegisteredContent types (non-interference verification) | ~600 |
| New files: section_group.rs + section_groups.rs + sections.rs | ~2,500 |
| Test files (AC-001 through AC-011) | ~2,200 |
| **Total** | **~14,400** |

## Test Strategy

- **Unit tests (slideforge-syntax)**: AC-001 (parse `section "Name":`), AC-002
  (disambiguation from bare-ident form; regression for all STORY-078 section-block tests),
  AC-010 (empty name `""` → E-PAR-023, no SectionGroupNode produced, correct span),
  AC-011 (duplicate name → W-PAR-002 warning, both sections in AST, same GUID).
- **Unit tests (slideforge-layout)**: `SlideSectionEntry` hash/eq/clone derivation; field
  presence on `LaidOutDeck`.
- **Unit tests (slideforge-eval)**: Eval correctly maps slides to section groups; ungrouped
  slides are absent from `slide_sections`.
- **Unit tests (slideforge-pptx)**: AC-003 (`p14:sectionLst` present under `p:extLst` with
  correct uri, `xmlns:p14` on root, 2 `p14:section` children), AC-004 (no `p:extLst` when
  empty; bytes unchanged), AC-005 (XML-escaped names in `p14:section/@name`), AC-006 (two
  single-slide sections in `p14:sectionLst`), AC-007 (non-PPTX formats unaffected; no
  `SectionListBuilder` call from docx/html/pdf), AC-008 (no register content in
  `p:extLst`/`p14:sectionLst`), AC-009 (deterministic GUIDs — build twice, assert identical
  byte output from `SectionListBuilder`).
- **Snapshot test**: `presentation.xml` for a 4-slide deck with 2 named sections — verify
  `p:extLst` placement is last child, `p:ext/@uri`, `p14:sectionLst` structure, and
  `xmlns:p14` declaration on `p:presentation`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-002 | Deck with no `section "Name":` slide-grouping blocks | No `<p:extLst>` or `<p14:sectionLst>` in presentation.xml; bytes returned unchanged |
| EC-004 | Section name with XML special characters (`&`, `<`, `>`) | XML-escaped in `<p14:section name="...">` attribute |
| EC-005 | Two sections each containing one slide | Both sections in sectionLst; each with one sldId |
| EC-006 | `section "Name":` present but deck built for DOCX/HTML/PDF | sectionLst logic skipped; non-PPTX output unaffected |
| EC-010 | Section name that is an empty string `""` | Parse error **E-PAR-023** (`ParseError::EmptySectionGroupName`): `section group name must be non-empty at <file>:<line>:<col>. Provide a quoted, non-empty name, e.g. section "Background":`. Severity: broken, exit 1. Error accumulated; no `SectionGroupNode` produced. |
| EC-011 | Two `section "Name":` blocks with identical names | Warning **W-PAR-002** (`ParseWarning::DuplicateSectionGroupName`): `warning: [W-PAR-002] Duplicate section group name '<name>' at <file>:<line>:<col>. Both sections are emitted with the same GUID. Consider using distinct names.` Severity: cosmetic, exit 0. Both sections emitted; identical GUID (same name → same SHA-256 derivation). |

## Revision History

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-04 | story-writer | Initial story decomposition |
| 1.1 | 2026-06-08 | story-writer | Pass-5 IMP-1: AC-010 E-PAR-023 exit code corrected 2→1 to match BC-4.01.003 v1.4 + error-taxonomy v2.28; EC-010 table exit code corrected 2→1; BC version references updated v1.3→v1.4 |
| 1.2 | 2026-06-08 | story-writer | Pass-8 F-P8-MED-1: corrected sha2 pin =0.10.9→=0.11.0 (canonical workspace pin line 73 of root Cargo.toml, shared with slideforge-math via STORY-030) in 4 locations: Scope Overview §4 prose, Tasks sha2 task, Library table sha2 row, Library Notes sha2 note, File Structure table Cargo.toml row; framing updated from "promote from dev-dep" to "add as workspace = true production dep" to match actual crate Cargo.toml; full version-pin sweep performed — quick-xml =0.36.0, ooxmlsdk =0.6.1, chumsky =0.10.1, zip =4.2.0 all confirmed MATCH. |
