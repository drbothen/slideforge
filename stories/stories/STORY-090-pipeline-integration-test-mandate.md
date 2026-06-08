---
id: STORY-090
title: "Mandate pipeline-origin integration tests for multi-crate eval→exporter stories"
epic: EPIC-19
wave: TBD
status: draft
priority: P1
points: 3
tdd_mode: strict
created: 2026-06-08
author: state-manager (process-gap codification per S-7.02 cycle-closing checklist)
traces_to:
  - PROC-GAP-PIPELINE-BYPASS-TESTS
  - PROC-GAP-SUBSTRING-RENDER-ASSERT
recurrence_count: 3
anchored_to: "STORY-082, STORY-072, STORY-081 (dead-wiring failures); STORY-088 (correct reference)"
---

# STORY-090 — Mandate pipeline-origin integration tests for multi-crate eval→exporter stories

## Problem Statement

In three separate Wave-5 stories (STORY-082, STORY-072, STORY-081) the implementation
shipped largely non-functional end-to-end — dead exporter code, dropped fields, hardcoded
`vec![]`, unwired seams — yet all per-story Red Gate tests passed. The test-writer wrote
exporter/unit tests against hand-constructed IR inputs that bypass the real
parse→eval→layout→export pipeline. The Red Gate was structurally incapable of catching
dead wiring.

STORY-088 did it correctly: a real `.sf` DSL source → `slideforge::build()` → assert
format-native markup present in the output bytes. It converged cleanly.

STORY-072 also had a related failure (PROC-GAP-SUBSTRING-RENDER-ASSERT): a substring-only
test (`contains("linear-gradient")`) matched the broken output (CSS background on SVG rect
paints nothing) because the substring was present even in the broken code path.

This is the single highest-leverage quality fix from Wave 5. It would have caught all three
failures at Red-Gate time.

## Acceptance Criteria

### AC-001 — Test-writer dispatch rule
The test-writer agent prompt (or orchestrator dispatch template for any story spanning
≥2 crates across the eval→exporter seam) MUST include the following mandatory test
requirement: at least one test in the Red Gate suite must originate from `slideforge::build()`
or equivalent DSL source (not hand-built IR) and assert FORMAT-NATIVE output — the actual
markup element/attribute/structure in the output bytes, not a keyword substring.

### AC-002 — Implementer dispatch rule
The implementer agent prompt for ≥2-crate eval→exporter stories MUST include a
self-check: "Does my test suite include at least one test where input is a real `.sf` source
string fed through `slideforge::build()` or `compile()`, and the assertion checks the actual
format-native markup in the output (not just a substring keyword)?"

### AC-003 — Adversary checklist item
The per-story LOCAL adversary checklist MUST include: "Check for hand-built IR bypass:
are any exporter tests constructed from hand-built `LaidOutDeck`/`LaidOutSlide` directly
rather than from `slideforge::build()` or the real pipeline? If yes, flag as CRIT — the
Red Gate cannot catch dead seams. Also check for substring-only render assertions (e.g.,
`contains("foo")`) where a format-native structure assertion would be more definitive."

### AC-004 — Reference example documented
A brief note in `.factory/specs/conventions.md` (or the story-delivery dispatch template)
cites STORY-088 as the canonical correct example and STORY-082/072/081 as the anti-pattern
examples.

## Notes

- This story does NOT change any production code — it changes agent dispatch templates
  and process documents only.
- Scope is narrow: 3-point story. The deliverable is updated dispatch instructions in
  the orchestrator/story-writer templates or CLAUDE.md conventions section.
- STORY-088 (correct reference implementation) already exists and is in-flight.
- Recurrence count: 3 confirmed occurrences (STORY-082/072/081). 4th would be unacceptable.
- Related: PROC-GAP-SUBSTRING-RENDER-ASSERT (already logged in BACKLOG.md); this story
  covers both process gaps together since they share the same root cause (tests not
  exercising the real pipeline).
