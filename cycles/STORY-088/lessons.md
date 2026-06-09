# STORY-088 Lessons Learned

_Captured 2026-06-09 after PR #78 merge (develop 15838de1, ADMIN OVERRIDE). Bullets list-literal DSL — 10-pass LOCAL cascade, 3/3 strict-CLEAN (passes 8-9-10)._

---

## [process-gap] Value-position parser quad-duplication

**Category:** Process gap (caught during scope expansion + adversary cascade)
**Date:** 2026-06-09
**Severity:** HIGH — caused a self-introduced CRITICAL recursion DoS + message divergence across 4 hand-maintained sites; required shared-combinator refactor mid-cascade

### Root Cause

STORY-088's scope expansion (human-adjudicated AC-012/AC-013) required adding
list-literal support to 4 parser sites that handled FieldValue value positions:
direct field-value, @var assignment, set-rule defaults, and variant vars overrides.

The initial implementation copied the list-literal element validation logic to
all 4 sites with "keep in sync" comments. This immediately produced two defects:

1. **E-PAR-024 message divergence** — the 4 copies drifted in their error message
   text, violating the error-taxonomy contract for E-PAR-024 message uniformity.
2. **CRITICAL recursion DoS** — one of the 4 copies introduced a self-referential
   parser combinator (the list-literal combinator called itself via a shared
   reference, creating unbounded recursion on deeply nested input). This was
   caught by the adversary in the same pass and flagged CRITICAL; the O(1)
   non-recursive depth tracker was designed and implemented to close it.

The adversary finding was: 4 duplicated implementations with hand-maintained
"keep N sites in sync by comment" annotations are a defect generator because
(a) comments do not enforce correctness, (b) each independent copy can
independently diverge, and (c) any refactor must be applied N times.

### Resolution

A single shared `list_literal_elements` combinator was extracted. All 4 value
positions call the shared combinator. The non-recursive O(1) depth tracker
(a u8 counter passed by value, not a recursive descent) replaced the
self-referential approach. E-PAR-024 message text is now defined once.

### Rule

Value-position parser features MUST route through ONE shared combinator from
the start. When a new value type needs to be supported across multiple value
positions, the correct implementation sequence is:

1. Implement the shared combinator once (with its test suite).
2. Wire all value positions to call the shared combinator.
3. Never copy-paste the validation logic — even with "keep in sync" comments.

**Exception:** If two value positions genuinely have different validation
semantics (e.g., different element types, different recursion rules), they
MAY have separate combinators, but must share the common validation primitive
(e.g., the depth tracker, the error code constant).

### Disposition

Codified lesson (no follow-up story needed). The shared-combinator pattern is
now established in the codebase (`list_literal_elements` in slideforge-syntax).
Future stories adding new value types to parser should reference this lesson.

_Anchor: slideforge-syntax parser; applicable to any story adding a new value
form that must appear in multiple value-position contexts._

---

## Follow-ups Registered

**FU-088-BC10102-ANCHOR** (spec-steward; non-blocking): Pre-existing
BC-1.01.002 H1 title vs field-value anchor mismatch detected during Pass-1.
Not introduced by STORY-088. Tracked for spec-steward cleanup; no code change
required.

**FU-CI-ARM64-TEST-FAILURE** (HIGH — investigate on next PR): The
`test (linux-arm64)` CI leg on PR #78 had a genuine nextest test failure
(passes on linux-x86_64, macos-arm64, windows-x86_64, and local 3916-test
suite). Exact test name unknown — the arm64 runner hung in GitHub's "Complete
job" finalization phase, preventing log retrieval before the job was cancelled.
Human directed admin-override merge; investigation deferred to next PR.
STORY-088's own tests run <0.04s on all platforms. Strongly suspected to be a
pre-existing perf/timing flake on the slow emulated arm64 runner (e.g.,
test_cold_budget_under_200ms or http_4xx retry timing), NOT a STORY-088 defect.
On the next PR: immediately capture the failed test name from the arm64 log
before any finalization hang. If confirmed as a timing flake, gate it with
`#[cfg_attr(slow_runner, ignore)]` or relax the time budget.

---

_Full cascade detail: 10 passes, converged 3/3 strict-CLEAN (passes 8-9-10). Scope expanded Pass-4: story v1.1→v1.2→v1.3 (AC-012 set-rule list default, AC-013 variant vars list override). dsl-spec v1.1, error-taxonomy v2.28. Shared list_literal_elements combinator; non-recursive O(1) depth tracker for E-PAR-024._
