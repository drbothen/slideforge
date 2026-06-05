---
story: STORY-084
title: "Bundled SectionType Implementations"
red_gate_verified: true
red_gate_date: 2026-06-05
producer: vsdd-factory:test-writer
crate: slideforge-plugin-api
---

# Red Gate Log — STORY-084

## Purpose

Documents the 13 failing behavioral tests verified RED (before implementation) per
the TDD Red Gate discipline. Tests were written by the test-writer agent and
confirmed failing before the implementer wrote a single line of production code.

## Red Gate Anti-Pattern Note (LESSON recorded this session)

The initial Red Gate submission contained `#[should_panic]` tests — an inverted Red
Gate anti-pattern. A `#[should_panic]` test wrapping a `todo!()` call PASSES against
stub code (because `todo!()` panics, satisfying `should_panic`) and then BREAKS on
correct implementation (because the real code does not panic). This is the opposite
of the TDD Red Gate requirement: tests must FAIL on stubs and PASS on implementation.

The test-writer step corrected all `#[should_panic]` tests to behavioral assertions
(direct output tests using `assert_eq!` / `assert!`) before the implementer was
dispatched. The corrected tests properly compiled but panicked at `todo!()` in stub
code, confirming genuine RED state.

This pattern is documented as a new lesson (see STATE.md Standing Process Rules).

## Red Gate Summary

| Test Name | Crate | Target Symbol | Failure Mode |
|-----------|-------|--------------|--------------|
| `test_executive_summary_auto_type_returns_sections` | slideforge-plugin-api | `ExecutiveSummaryType::generate_sections` | `todo!()` panic |
| `test_executive_summary_excludes_notes_register` | slideforge-plugin-api | `ExecutiveSummaryType::generate_sections` | `todo!()` panic |
| `test_executive_summary_slide_type_id` | slideforge-plugin-api | `ExecutiveSummaryType` | `todo!()` panic |
| `test_risk_register_auto_type_returns_sections` | slideforge-plugin-api | `RiskRegisterType::generate_sections` | `todo!()` panic |
| `test_risk_register_excludes_notes_register` | slideforge-plugin-api | `RiskRegisterType::generate_sections` | `todo!()` panic (mirrors layout sections.rs HIGH-002) |
| `test_risk_register_slide_type_id` | slideforge-plugin-api | `RiskRegisterType` | `todo!()` panic |
| `test_methodology_manual_type_returns_empty` | slideforge-plugin-api | `MethodologyType::generate_sections` | `todo!()` panic |
| `test_scope_manual_type_returns_empty` | slideforge-plugin-api | `ScopeType::generate_sections` | `todo!()` panic |
| `test_approval_manual_type_returns_empty` | slideforge-plugin-api | `ApprovalType::generate_sections` | `todo!()` panic |
| `test_appendix_manual_type_returns_empty` | slideforge-plugin-api | `AppendixType::generate_sections` | `todo!()` panic |
| `test_glossary_manual_type_returns_empty` | slideforge-plugin-api | `GlossaryType::generate_sections` | `todo!()` panic |
| `test_empty_title_produces_valid_section` | slideforge-plugin-api | all 7 impls | `todo!()` panic |
| `test_auto_type_start_slide_index_relative_zero` | slideforge-plugin-api | `ExecutiveSummaryType`, `RiskRegisterType` | `todo!()` panic |

## BC Coverage

The 13 tests collectively verify:

- **BC-5.02.001 (SectionType surface):** All 7 bundled implementations satisfy the
  `SectionType` plugin trait. The two auto types (`executive_summary`, `risk_register`)
  return populated `Vec<Section>`. The five manual types (`methodology`, `scope`,
  `approval`, `appendix`, `glossary`) return `vec![]`.
- **BC-3.02.001 invariant 4 (notes-register exclusion):** Both auto types
  (`ExecutiveSummaryType` and `RiskRegisterType`) MUST exclude content tagged with the
  `notes` writing register. Tests `test_executive_summary_excludes_notes_register` and
  `test_risk_register_excludes_notes_register` verify this invariant. The
  `risk_register` notes-exclusion was the CRIT-084-001 finding (mirrored from
  `layout/sections.rs` HIGH-002 — the upstream layout code had the same gap, which was
  also closed in scope).

## Red Gate Protocol Compliance

Per SID-1 (No-Ignored-Test Rationalization) and STORY-084 story spec:

- Tests were written BEFORE implementation (strict TDD)
- `#[should_panic]` tests identified and corrected to behavioral assertions before
  implementer dispatch (see anti-pattern note above)
- All 13 tests confirmed failing via `todo!()` panics before implementer dispatch
- No test relies on external dependencies (pure in-memory, no filesystem, no DTU)
- Integration path exercised: construct section input → call `generate_sections()` →
  assert output `Vec<Section>` contents and writing-register filtering

## Post-Implementation Result

After implementation by the implementer agent:

- 171 plugin-api tests pass (including all 13 new Red Gate tests)
- 0 failures
- Workspace-level: all tests continue to pass

## Notes

- CRIT-084-001 (risk_register notes-exclusion): Closed in scope. The same gap existed
  in `layout/sections.rs` (HIGH-002) and was fixed as a parallel drive-by fix.
- MED-084-002 (absolute `start_slide_index` + None field assertions): Closed in scope.
  Tests were updated to assert relative/documented semantics rather than brittle
  absolute integers.
- MED-084-003 (empty-title documented + tested): Closed in scope. The `test_empty_title_produces_valid_section` test was added and behavior documented.
- OBS-084-004 / LOW-084-005 / OBS-084-006 / LOW-084-A (stale comments + docstring
  copy-paste artifact): All closed in scope during implementer fix-burst.
- LOCAL adversary convergence: 7 passes total; strict-CLEAN on passes 5/6/7.
- Security review: APPROVE, CLEAN — DoS/panic/notes-disclosure all clean; no new deps.
- PR #58 also removed 4 stray `.factory/` files (STORY-027/029/034 red-gate-logs +
  STORY-001 demo) that were historically mis-tracked on develop. Intentional hygiene.
- `severity_cards` confirmed as the contributing slide-type id for the auto types via
  slide-types-catalog.md + layout sections.rs cross-reference.
