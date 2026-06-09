---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-015
lifecycle_status: active
introduced: v1.0.0
modified:
  - "2026-06-04: v1.2 — Human-authorized scope split (2026-06-04). Slide-sections
    sub-requirement (sectionLst from `section \"Name\":` groupings) carved to follow-up
    story (STORY-08x / slide-grouping follow-up; story-writer assigns final ID). Speaker
    notes and master/layout/theme postconditions unchanged; owned by STORY-040, delivered
    now. Added BC-1.14.003 non-interference clarification: sectionLst uses section name +
    slide membership ONLY (never section body or register_content). Documented
    multi-renderer parity exception for <p:sectionLst> (PPT-only, human-accepted
    2026-06-04). Reaffirmed slide sections as v1.0 per q1-decision-final.md Section 7
    (overriding pptx-element-taxonomy.md v1.x suggestion; planning reconciliation
    recorded in planning/decisions-reconciliation.md)."
  - "2026-06-07: v1.3 — STORY-082 spec reconciliation burst. Added EC-010 (empty section
    group name → E-PAR-023 fatal) and EC-011 (duplicate section group names → W-PAR-002
    warning, both emitted with same deterministic GUID) to the slide-sections edge-case
    table. Added postcondition 7 (non-empty name validation) and invariant 5
    (duplicate-name behavior: warning + deterministic identical GUID). Error codes
    E-PAR-023 and W-PAR-002 allocated in error-taxonomy.md v2.24 in the same burst."
  - "2026-06-08: v1.4 — Pass-5 IMP-1 spec correction. Corrected E-PAR-023 exit code
    2→1 in PC-7 and EC-010. E-PAR-023 is a parse error; per BC-1.15.003 three-tier model
    (parse errors → exit 1), consistent with every other E-PAR row in the taxonomy and
    with exit_code.rs mapping E-PAR → EXIT_PARSE_ERROR = 1."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-4.01.003: PPTX Contains Speaker Notes and Master/Layout/Theme System; Slide Sections Deferred to Follow-Up Story

## Description

The PPTX exporter must embed speaker notes from the `notes` writing register into
`notesSlide` parts and generate a complete master/layout/theme hierarchy from the brand.
These structural components are required for professional presentation use. A third
sub-requirement — slide section groupings (`<p:sectionLst>`) from `section "Name":` DSL
constructs — is a confirmed v1.0 requirement but is owned by a separate follow-up story
(STORY-08x / slide-grouping follow-up; story-writer assigns the final ID) because it
requires a DSL feature not yet implemented.

## Scope Boundary (Human-Authorized Split 2026-06-04)

This BC covers TWO delivery halves with separate story ownership:

**Half A — Delivered by STORY-040 (now):**
- Speaker notes embedding (`notesSlide` parts from `notes` writing register)
- notesMaster1.xml + handoutMaster1.xml generation
- Master/layout/theme hierarchy (postconditions 1-4, 6 below)

**Half B — Delivered by STORY-08x / slide-grouping follow-up (v1.0, separate story):**
- PPTX `<p:sectionLst>` generation from `section "Name":` slide-grouping constructs
- Requires: new DSL syntax (quoted-name section with slide-children blocks, distinct from
  existing `section IDENT:` document-structure blocks), a `slide_sections` field added to
  `LaidOutDeck` IR, and an eval-stage slide-membership mapping
- Story-writer will anchor the follow-up story to postcondition 5 and the sectionLst
  invariants/edge-cases below

Do NOT merge these halves into a single story. The DSL dependency makes slide-grouping
undeliverable in STORY-040's scope.

## BC-1.14.003 Non-Interference Clarification

The `<p:sectionLst>` element (Half B) is built EXCLUSIVELY from:
1. The **section name** — the quoted string in `section "Name":` (DSL surface)
2. **Slide membership** — the set of slide IDs that fall under that section in the deck

It MUST NOT read or embed:
- Section body content (the `report:`, `notes:`, `detail:` sub-fields)
- `RegisteredContent` nodes of any kind from BC-3.02.002

BC-1.14.003 postconditions 3 and 5 remain fully intact: detail register content does not
appear in PPTX under any circumstances, including inside `<p:sectionLst>` attributes or
extension data. The follow-up story must cite this constraint explicitly.

## Multi-Renderer Parity Exception (Human-Accepted 2026-06-04)

`<p:sectionLst>` is classified as "Fair — PPT only" in pptx-element-taxonomy.md:
it is a PowerPoint-extension-namespace element that does not render as slide navigation
sections in Keynote, Google Slides, or LibreOffice. The CLAUDE.md multi-renderer parity
quality gate applies across the 4 target renderers for all other PPTX elements.

Exception: `<p:sectionLst>` is a DOCUMENTED, HUMAN-ACCEPTED departure from full
multi-renderer parity. The human ruled on 2026-06-04 (when authorizing the scope split)
that slide sections ship in v1.0 despite being PPT-only. Rationale: the PowerPoint
organizational benefit justifies the element; non-PowerPoint renderers simply ignore it
without rendering corruption. The feature gracefully degrades (invisible in
Keynote/Google Slides/LibreOffice; present and functional in PowerPoint 365).

Test coverage for sectionLst is restricted to PowerPoint 365 rendering verification.
Multi-renderer parity tests for this element are EXPLICITLY EXCLUDED.

## Preconditions

1. A valid `LaidOutDeck` IR exists with at least one slide.
2. A `Brand` struct is available with a complete master/layout hierarchy (BC-2.01.005).
3. Speaker notes content from the `notes` register has been resolved (BC-1.14.001).

**Preconditions for Half B (slide-grouping follow-up story only):**

4. The `LaidOutDeck` IR has a `slide_sections` field populated by the evaluator with
   section name + ordered slide-ID membership (added by the follow-up story's DSL work).
5. The DSL source uses `section "Name":` quoted-name constructs with slide children
   (distinct from the existing `section IDENT:` document-structure blocks).

## Postconditions

**Postconditions 1-4, 6 (owned by STORY-040):**

1. Every slide that has non-empty `notes` content has a corresponding `notesSlide` part
   in the PPTX ZIP at `ppt/notesSlides/notesSlide<N>.xml`.
2. Slides without notes content have no `notesSlide` part (or an empty one — either is valid).
3. The PPTX contains a slide master (`ppt/slideMasters/slideMaster1.xml`) and at minimum
   11 standard + 20 custom slide layouts (per BC-2.01.005).
4. A `theme1.xml` is present in `ppt/theme/` and references all 12 OOXML color slots (DI-015).
6. The PPTX opens in PowerPoint 365 with notes visible in the Notes pane.

**Postcondition 5 (owned by STORY-08x / slide-grouping follow-up):**

5. When `section "Name":` slide-grouping constructs are declared, the PPTX
   `<p:sectionLst>` element groups slides accordingly, using section name and slide
   membership (slide IDs) only — no section body content or register fields are read.
   The element is present in `<p:presentation><p:ext>` (extension namespace). The
   element is silently absent when no slide-grouping sections are declared.

**Postconditions 7-8 (owned by STORY-082 / slide-grouping follow-up):**

7. A `section "":` construct (empty quoted name) is rejected at parse time with
   `E-PAR-023` (`ParseError::EmptySectionGroupName`), exit 1. No `SectionGroupNode`
   is produced for the rejected block. The error is accumulated; the parser continues
   past the rejected block to find additional errors. (EC-010)

8. Two or more `section "Name":` blocks sharing an identical quoted name are both
   emitted to the IR without error. A non-fatal `W-PAR-002` warning
   (`ParseWarning::DuplicateSectionGroupName`) is emitted for the second and subsequent
   occurrences. Each section receives the same GUID value (derived deterministically
   from the identical name via sha2 hash). The build exits 0 unless a separate fatal
   error is also present. (EC-011)

## Invariants

1. Speaker notes are NEVER placed in slide body content — they route exclusively to
   `notesSlide` parts. (DI-012 — register routing fidelity) [STORY-040]
2. The master/layout hierarchy is generated from the brand, not hardcoded. [STORY-040]
3. Slide section memberships for `<p:sectionLst>` are derived from `section "Name":`
   slide-grouping DSL blocks and the resulting `LaidOutDeck.slide_sections` field; no
   section data is fabricated. Register content (report/notes/detail) is NEVER sourced
   for the `<p:sectionLst>` structure (BC-1.14.003 non-interference). [STORY-08x]
4. `<p:sectionLst>` is omitted entirely when no `section "Name":` groupings exist in
   the deck. [STORY-08x]
5. Section group names MUST be non-empty. An empty-string quoted name (`""`) triggers
   `E-PAR-023` at parse time; no `SectionGroupNode` is constructed for the offending
   block. Duplicate names (two blocks sharing the same non-empty name) are permitted
   with a `W-PAR-002` warning; the deterministic GUID derivation (sha2 hash of name)
   guarantees identical names always produce identical GUIDs — this is intentional and
   does not constitute a collision that must be rejected. [STORY-082]

## Edge Cases

**Speaker notes / master (STORY-040):**

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with empty notes register | No `notesSlide` part generated for that slide (or empty notesSlide) |
| EC-003 | Notes content contains {{ }} interpolation | Interpolation evaluated before embedding in notesSlide XML |

**Slide sections (STORY-08x / slide-grouping follow-up):**

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-002 | Deck with no `section "Name":` slide-grouping blocks | No `<p:sectionLst>` element; silently omitted |
| EC-004 | Section name contains XML special characters (<, >, &) | Section name XML-escaped in `<p:sectionLst>` attribute |
| EC-005 | Two sections containing a single slide each | Both sections present; each slide assigned to its section; no overlap |
| EC-006 | `section "Name":` construct present but deck built for non-PPTX format | sectionLst logic skipped; DOCX/HTML/PDF unaffected (PPT-only element) |
| EC-010 | Section name that is an empty string `""` — e.g. `section "":` | Fatal parse error E-PAR-023 (`ParseError::EmptySectionGroupName`); exit 1 (strict). Error accumulated; no `SectionGroupNode` produced; build halts after error reporting. Span on the opening `"` of the empty name. [STORY-082] |
| EC-011 | Two `section "Name":` blocks with identical quoted names | Non-fatal parse warning W-PAR-002 (`ParseWarning::DuplicateSectionGroupName`) emitted for the second (and subsequent) occurrence(s); exit 0. Both sections are emitted to the IR and included in `LaidOutDeck.slide_sections`. Each receives the same deterministic GUID (sha2 hash of the name — identical names → identical GUIDs). No crash; no section is dropped. The `<name>` and `<file>:<line>:<col>` placeholders identify the duplicate occurrence. [STORY-082] |

## Canonical Test Vectors

**Speaker notes / master (STORY-040):**

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck with 3 slides, each having `notes "Speaker text"` | PPTX has 3 notesSlide parts; PowerPoint notes pane shows text | happy-path |
| Deck with no notes fields | PPTX has 0 notesSlide parts (or empty); no error | edge-case |
| Deck with notes containing `< & >` characters | notesSlide XML: `&lt; &amp; &gt;` — no malformed XML | edge-case |

**Slide sections (STORY-08x / slide-grouping follow-up):**

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck with `section "Background":` grouping 2 slides | PPTX `<p:sectionLst>` has 1 section entry with 2 slide refs; PowerPoint shows section | happy-path |
| Deck with no `section "Name":` blocks | No `<p:sectionLst>` in PPTX; Keynote/Google Slides render normally | edge-case |
| Same deck built to DOCX | No sectionLst logic executed; DOCX unaffected | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method | Owner Story |
|--------|----------|-------------|-------------|
| VP-TBD | notesSlide count = count of slides with non-empty notes | unit test: unzip PPTX, count notesSlide parts | STORY-040 |
| VP-TBD | theme1.xml contains all 12 OOXML color slot elements | snapshot test | STORY-040 |
| VP-TBD | Section slide refs are correct and non-overlapping | integration test: parse sectionLst, verify slide ID sets are disjoint and complete | STORY-08x |
| VP-TBD | sectionLst contains no register content (BC-1.14.003 non-interference) | integration test: assert no detail/report/notes text in sectionLst XML | STORY-08x |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 |
| Capability Anchor Justification | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 — "Embed speaker notes, master/layout/theme system, placeholder inheritance, slide sections" is verbatim from CAP-015 |
| L2 Domain Invariants | DI-012 (single source → all formats consistent; register routing), DI-015 (brand palette covers all 12 color slots) |
| Architecture Module | slideforge-pptx crate (filled by architect) |
| Stories | STORY-040 (Half A: speaker notes + master); STORY-08x / slide-grouping follow-up (Half B: sectionLst — story-writer assigns final ID) |
| Multi-Renderer Parity Exception | `<p:sectionLst>` exempt from multi-renderer parity gate; PPT-only element; human-accepted 2026-06-04 |

## Related BCs

- BC-4.01.001 — composes with (this BC specifies structural components of the PPTX produced by BC-4.01.001)
- BC-1.14.001 — depends on (notes register routing is a prerequisite)
- BC-1.14.003 — non-interference (sectionLst MUST NOT read section body or register_content; BC-1.14.003 postconditions 3 and 5 remain fully intact)
- BC-2.01.005 — depends on (31 layouts must exist in the master for this BC to pass)
- BC-4.01.006 — composes with (notesMaster is a related structural requirement)

## Architecture Anchors

- `architecture/export-architecture.md` — PPTX structural requirements

## Story Anchor

- STORY-040 — Half A (speaker notes + notesMaster/handoutMaster + master/layout/theme)
- STORY-08x — Half B (slide-grouping sectionLst; story-writer assigns final ID)

## VP Anchors

(filled after VP creation)
