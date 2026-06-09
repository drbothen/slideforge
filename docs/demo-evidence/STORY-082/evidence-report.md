---
story_id: STORY-082
title: "PPTX: Slide-Grouping Sections (sectionLst) — DSL + IR + Eval + Exporter"
recorded: 2026-06-09
toolchain: VHS 0.10.0 (terminal recording)
branch: feature/STORY-082
---

# Evidence Report — STORY-082

## Coverage Summary

| AC | Title | Recording | XML Snippet | Path |
|----|-------|-----------|-------------|------|
| AC-001 | `section "Name":` construct parses successfully | AC-001-002-dsl-parsing-happy-path | — | [tape](AC-001-002-dsl-parsing-happy-path.tape) / [gif](AC-001-002-dsl-parsing-happy-path.gif) |
| AC-002 | `section "Name":` distinct from `section IDENT:` | AC-001-002-dsl-parsing-happy-path | — | [tape](AC-001-002-dsl-parsing-happy-path.tape) / [gif](AC-001-002-dsl-parsing-happy-path.gif) |
| AC-003 | Slides produce `<p14:sectionLst>` via `<p:extLst>` | AC-003-006-009-pptx-section-lst-emission + AC-003-e2e-pipeline-section-lst-in-xml | [snippet](presentation-xml-section-lst-snippet.xml) | [tape](AC-003-006-009-pptx-section-lst-emission.tape) / [gif](AC-003-006-009-pptx-section-lst-emission.gif) |
| AC-004 | No `extLst`/`sectionLst` when no groupings | AC-003-006-009-pptx-section-lst-emission | — | [tape](AC-003-006-009-pptx-section-lst-emission.tape) / [gif](AC-003-006-009-pptx-section-lst-emission.gif) |
| AC-005 | XML-escaped section names in `p14:section` | AC-003-006-009-pptx-section-lst-emission | — | [tape](AC-003-006-009-pptx-section-lst-emission.tape) / [gif](AC-003-006-009-pptx-section-lst-emission.gif) |
| AC-006 | Two single-slide sections both in `p14:sectionLst` | AC-003-006-009-pptx-section-lst-emission | — | [tape](AC-003-006-009-pptx-section-lst-emission.tape) / [gif](AC-003-006-009-pptx-section-lst-emission.gif) |
| AC-007 | `sectionLst` skipped for non-PPTX formats | AC-007-008-non-pptx-skip-and-non-interference | — | [tape](AC-007-008-non-pptx-skip-and-non-interference.tape) / [gif](AC-007-008-non-pptx-skip-and-non-interference.gif) |
| AC-008 | `p14:sectionLst` carries no register content (BC-1.14.003) | AC-007-008-non-pptx-skip-and-non-interference | — | [tape](AC-007-008-non-pptx-skip-and-non-interference.tape) / [gif](AC-007-008-non-pptx-skip-and-non-interference.gif) |
| AC-009 | Deterministic section GUIDs in `p14:section` | AC-003-006-009-pptx-section-lst-emission | [snippet](presentation-xml-section-lst-snippet.xml) | [tape](AC-003-006-009-pptx-section-lst-emission.tape) / [gif](AC-003-006-009-pptx-section-lst-emission.gif) |
| AC-010 (error) | Empty section name rejected with E-PAR-023, exit 1 | AC-010-empty-name-error-path | — | [tape](AC-010-empty-name-error-path.tape) / [gif](AC-010-empty-name-error-path.gif) |
| AC-011 (warn) | Duplicate section names emit W-PAR-002, exit 0, same GUID | AC-011-duplicate-name-warning-path | — | [tape](AC-011-duplicate-name-warning-path.tape) / [gif](AC-011-duplicate-name-warning-path.gif) |

## Recording Details

### AC-001 & AC-002 — DSL parsing happy path

**Recording:** `AC-001-002-dsl-parsing-happy-path.{gif,webm,tape}`

Demonstrates the `section "Name":` slide-grouping block accepted by the parser
(`AC-001`) and the disambiguation rule that a quoted-string after `section` produces
a `SectionGroupNode` while a bare-ident produces the STORY-078 `SectionBlock`
(`AC-002`). No grammar collision between the two forms.

Tests shown: `test_BC_4_01_003_ac001_section_group_node_parses_with_name`,
`test_BC_4_01_003_ac002_section_group_distinct_from_section_block`.
All pass.

### AC-003, AC-004, AC-005, AC-006, AC-009 — PPTX sectionLst emission (happy path)

**Recording:** `AC-003-006-009-pptx-section-lst-emission.{gif,webm,tape}`

Demonstrates `SectionListBuilder::inject()` at unit-test level. Covers:
- `AC-003`: `<p:extLst>`/`<p14:sectionLst>` produced with correct `p:ext/@uri`
  `{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}`, `xmlns:p14` declared on
  `<p:presentation>`, and `<p:extLst>` positioned as the last child.
- `AC-004`: empty sections input returns bytes unchanged — no `<p:extLst>` emitted.
- `AC-005`: section name `"Background & Overview"` produces
  `name="Background &amp; Overview"` (XML-escaped). `<` and `>` tested separately
  via `test_BC_4_01_003_ec004_lt_gt_in_section_name_are_xml_escaped`.
- `AC-006`: two single-slide sections → two `<p14:section>` each with one `<p14:sldId>`.
- `AC-009`: `derive_section_guid` is deterministic (same input → same SHA-256-derived
  GUID); brace-wrapped uppercase hex format; version nibble=5; RFC 4122 variant bits.

**XML snippet:** `presentation-xml-section-lst-snippet.xml` — annotated reference of
the `ppt/presentation.xml` structure produced for a 2-section, 4-slide deck.

### AC-003 end-to-end — Full pipeline wiring

**Recording:** `AC-003-e2e-pipeline-section-lst-in-xml.{gif,webm,tape}`

Demonstrates the complete parse→eval→layout→PPTX pipeline (HIGH-3 keystone test)
producing a real PPTX ZIP whose `ppt/presentation.xml` contains `<p14:sectionLst>`.
Assertions verified via `quick-xml` re-parsing of the extracted XML:
- `<p:extLst>` is the LAST child of `<p:presentation>` (nothing between
  `</p:extLst>` and `</p:presentation>`).
- `<p:ext uri="{BB962C8B-...}">` present.
- `<p14:sectionLst>` contains exactly 2 `<p14:section>` children
  (Background + Analysis).
- `xmlns:p14` declared on `<p:presentation>`.

CRIT-A test also shown: `@for` expansion before a `section "Name":` group correctly
advances the flat slide-index counter so section slide IDs are `[258, 259, 260]` not
`[256, ...]`.

### AC-007 & AC-008 — Non-PPTX skip and BC-1.14.003 non-interference

**Recording:** `AC-007-008-non-pptx-skip-and-non-interference.{gif,webm,tape}`

- `AC-007`: `SectionListBuilder` is PPTX-only. Full-pipeline DOCX export
  (`test_BC_4_01_003_med2_docx_has_no_section_lst`) asserts no `sectionLst` string
  in any XML part of the produced `.docx` ZIP.
- `AC-008`: `detail:` and `report:` register content does NOT bleed into
  `<p:extLst>`/`<p14:sectionLst>`. Two sentinel strings verified absent from the
  `p:extLst` block — both at unit level and full-pipeline level (OBS-1 strengthening).
  Traces to BC-1.14.003 postconditions 3 + 5.

### AC-010 — Error path: empty section name (E-PAR-023)

**Recording:** `AC-010-empty-name-error-path.{gif,webm,tape}`

Demonstrates the error path for `section "":`. `parse()` returns `Err` containing
`E-PAR-023`. Exit code 1 in strict mode. No `SectionGroupNode` produced in the
recovered AST (sentinel-discard logic in `deck.rs`).

Tests: `test_BC_4_01_003_ac010_empty_section_name_rejected_e_par_023`,
`test_BC_4_01_003_ac010_empty_section_name_no_section_group_node`. Both pass.

### AC-011 — Warning path: duplicate section names (W-PAR-002)

**Recording:** `AC-011-duplicate-name-warning-path.{gif,webm,tape}`

Demonstrates the warning path for two `section "Background":` blocks.
`parse()` returns `Ok` (cosmetic warning; exit 0). `W-PAR-002` present in warnings.
Both `SectionGroupNode` entries appear in the AST. Both `p14:section/@id` attributes
are identical (same name → same SHA-256-derived GUID).

Tests: `test_BC_4_01_003_ac011_duplicate_section_name_warning_w_par_002`,
`test_BC_4_01_003_ac011_duplicate_section_name_both_sections_present`,
`test_BC_4_01_003_ac011_duplicate_section_name_same_guid`. All pass.

## Quality Gate

- `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items`: **CLEAN** (no output)
- `cargo fmt --all -- --check`: **CLEAN** (no output)
- `cargo test --workspace --no-fail-fast`: **all pass**

## Artifact List

| File | Type | Size |
|------|------|------|
| `AC-001-002-dsl-parsing-happy-path.gif` | GIF recording | 278 KB |
| `AC-001-002-dsl-parsing-happy-path.webm` | WebM recording | 705 KB |
| `AC-001-002-dsl-parsing-happy-path.tape` | VHS script | 1.5 KB |
| `AC-003-006-009-pptx-section-lst-emission.gif` | GIF recording | 801 KB |
| `AC-003-006-009-pptx-section-lst-emission.webm` | WebM recording | 781 KB |
| `AC-003-006-009-pptx-section-lst-emission.tape` | VHS script | 2.5 KB |
| `AC-003-e2e-pipeline-section-lst-in-xml.gif` | GIF recording | 257 KB |
| `AC-003-e2e-pipeline-section-lst-in-xml.webm` | WebM recording | 711 KB |
| `AC-003-e2e-pipeline-section-lst-in-xml.tape` | VHS script | 1.8 KB |
| `AC-007-008-non-pptx-skip-and-non-interference.gif` | GIF recording | 402 KB |
| `AC-007-008-non-pptx-skip-and-non-interference.webm` | WebM recording | 997 KB |
| `AC-007-008-non-pptx-skip-and-non-interference.tape` | VHS script | 2.0 KB |
| `AC-010-empty-name-error-path.gif` | GIF recording | 309 KB |
| `AC-010-empty-name-error-path.webm` | WebM recording | 761 KB |
| `AC-010-empty-name-error-path.tape` | VHS script | 1.5 KB |
| `AC-011-duplicate-name-warning-path.gif` | GIF recording | 326 KB |
| `AC-011-duplicate-name-warning-path.webm` | WebM recording | 452 KB |
| `AC-011-duplicate-name-warning-path.tape` | VHS script | 1.5 KB |
| `presentation-xml-section-lst-snippet.xml` | XML artifact | 2.5 KB |
