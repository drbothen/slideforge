---
story_id: STORY-027
phase: red-gate
timestamp: 2026-05-27
agent: test-writer
---

# Red Gate Log — STORY-027

## Result: RED GATE VERIFIED

All 20 new behavioral tests FAIL before any implementation.
All 91 pre-existing tests PASS (no regression).

## Test Run Summary

```
test result: FAILED. 91 passed; 20 failed; 0 ignored; 0 measured; 0 filtered out
```

## Failing Tests (20)

| Test Name | BC | Failure Mode |
|-----------|-----|--------------|
| `sections::tests::test_bc_3_02_001_executive_summary_three_takeaways` | BC-3.02.001 / AC-001 | `todo!()` panic in `collect_executive_summary` |
| `sections::tests::test_bc_3_02_001_executive_summary_heading` | BC-3.02.001 / AC-001 | `todo!()` panic in `collect_executive_summary` |
| `sections::tests::test_bc_3_02_001_executive_summary_target_formats` | BC-3.02.001 / AC-001 | `todo!()` panic in `collect_executive_summary` |
| `sections::tests::test_bc_3_02_001_executive_summary_absent_when_no_takeaways` | BC-3.02.001 / AC-003 | `todo!()` panic in `collect_executive_summary` |
| `sections::tests::test_bc_3_02_001_executive_summary_absent_for_empty_deck` | BC-3.02.001 / AC-003 | `todo!()` panic in `collect_executive_summary` |
| `sections::tests::test_bc_3_02_001_executive_summary_skips_slides_without_takeaway` | BC-3.02.001 / AC-001 | `todo!()` panic in `collect_executive_summary` |
| `sections::tests::test_bc_3_02_001_executive_summary_excludes_pptx_and_html_from_target_formats` | BC-3.02.001 / AC-005 | `todo!()` panic in `collect_executive_summary` |
| `sections::tests::test_bc_3_02_001_risk_register_from_severity_cards` | BC-3.02.001 / AC-002 | `todo!()` panic in `collect_risk_register` |
| `sections::tests::test_bc_3_02_001_risk_register_heading` | BC-3.02.001 / AC-002 | `todo!()` panic in `collect_risk_register` |
| `sections::tests::test_bc_3_02_001_risk_register_absent_when_no_severity_cards` | BC-3.02.001 / AC-003 | `todo!()` panic in `collect_risk_register` |
| `sections::tests::test_bc_3_02_001_risk_register_multiple_slides_no_dedup` | BC-3.02.001 / EC-003 | `todo!()` panic in `collect_risk_register` |
| `sections::tests::test_bc_3_02_001_risk_row_preserves_card_fields` | BC-3.02.001 / AC-002 | `todo!()` panic in `collect_risk_register` |
| `sections::tests::test_bc_3_02_001_risk_register_excludes_pptx_and_html_from_target_formats` | BC-3.02.001 / AC-005 | `todo!()` panic in `collect_risk_register` |
| `sections::tests::test_bc_3_02_001_collect_sections_combines_auto_generated` | BC-3.02.001 / AC-007 | `todo!()` panic in `collect_sections` |
| `sections::tests::test_bc_3_02_001_collect_sections_empty_when_no_contributing_slides` | BC-3.02.001 / AC-003 | `todo!()` panic in `collect_sections` |
| `sections::tests::test_bc_3_02_001_collect_sections_empty_deck_produces_empty_vec` | BC-3.02.001 / AC-003 | `todo!()` panic in `collect_sections` |
| `sections::tests::test_bc_3_02_001_collect_sections_is_deterministic` | BC-3.02.001 / AC-007 | `todo!()` panic in `collect_sections` |
| `sections::tests::test_bc_3_02_002_manual_section_block_passed_through` | BC-3.02.002 / AC-004 | `todo!()` panic in `collect_sections` |
| `sections::tests::test_bc_3_02_002_manual_section_source_is_manually_authored` | BC-3.02.002 / AC-004 | `todo!()` panic in `collect_sections` |
| `tests::test_layout_run_populates_sections_from_takeaway_slides` | BC-3.02.001 / AC-001 | Assertion: `LaidOutDeck.sections` is empty (`layout::run` seeds `Vec::new()`) |

## Passing Structural Tests (compilation probes, not behavior)

| Test Name | Purpose |
|-----------|---------|
| `sections::tests::test_bc_3_02_001_section_kind_variants_exist` | Type compile probe |
| `sections::tests::test_bc_3_02_001_section_source_variants_exist` | Type compile probe |
| `sections::tests::test_bc_3_02_001_section_item_variants_exist` | Type compile probe |
| `sections::tests::test_bc_3_02_001_output_format_variants_exist` | Type compile probe |
| `sections::tests::test_bc_3_02_001_generated_section_implements_hash_eq_clone` | AC-010 trait bounds |
| `sections::tests::test_bc_3_02_002_unknown_section_type_error_variant_exists` | EC-005 error type |

## BC Coverage Map

| BC | ACs Tested | Tests |
|----|-----------|-------|
| BC-3.02.001 | AC-001, AC-002, AC-003, AC-005, AC-007, EC-003 | 17 behavioral |
| BC-3.02.002 | AC-004, AC-005 | 3 behavioral (2 driving Red Gate + 1 type probe) |

## Known Gap: BC-3.02.002 Manual Section Field

`Deck` does not yet have a `section_blocks` / `top_level_sections` field.
The BC-3.02.002 tests (`test_bc_3_02_002_manual_section_block_passed_through`,
`test_bc_3_02_002_manual_section_source_is_manually_authored`) are written to
compile against the current `Deck` struct and drive the `collect_sections`
`todo!()` path. When the implementer adds `Deck::section_blocks`, these tests
must be strengthened with full assertions (construct a deck with a section block,
verify the returned `ManualSection` kind and source).

## Missing ACs (out of scope for test-writer)

- **AC-006** (manual supersedes auto): requires `Deck::section_blocks` field — deferred to implementer.
- **AC-007** (`section_order:` sorting): tested for determinism; explicit ordering test requires `Deck::section_order` field.

## File Paths

- Tests file: `crates/slideforge-layout/src/sections.rs` (test module)
- Integration test: `crates/slideforge-layout/src/lib.rs` (test module)

## Implementer Instructions

Make each of the 20 failing tests pass, one at a time, with minimum code:

1. Implement `collect_executive_summary` — iterate `deck.slides`, collect `takeaway` string fields, return `None` if empty.
2. Implement `collect_risk_register` — iterate `deck.slides` for `slide_type == "severity_cards"`, collect card fields as `RiskRow`, return `None` if empty.
3. Add `section_blocks: Vec<SectionBlock>` (or equivalent) to `slideforge_types::Deck`.
4. Implement `collect_sections` — combine auto-generated sections and manual blocks, apply AC-006 supersession, apply AC-007 ordering.
5. Wire `collect_sections` into `layout::run` (replace `let sections = Vec::new();` placeholder).
