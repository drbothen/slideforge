---
document_type: adversary-pass-report
story_id: STORY-050
pass: 2
date: 2026-06-05
branch: feature/STORY-050
branch_head_at_review: c4e4b26e
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 3
findings_open: 0
findings_remediated: 3
---

# STORY-050 Adversary Pass 2 Report

## Pass Metadata

| Field | Value |
|-------|-------|
| Branch | `feature/STORY-050` |
| Branch HEAD at review | `c4e4b26e` |
| Pass date | 2026-06-05 |
| CLEAN (strict) | **no** — 3 findings (HIGH + OBS + OBS) |
| CLEAN (PR-merge) | **no** — 1 HIGH present |
| Streak | 0/3 (reset) |

## Findings Summary

| ID | Severity | Title | Status |
|----|----------|-------|--------|
| F-050-P2-HIGH-001 | HIGH | AC-007 regression guards vacuously true — pass-1 fix was a paper-fix | REMEDIATED |
| OBS-050-P2-001 | OBS | Gap-1 title None-path has zero coverage | REMEDIATED |
| OBS-050-P2-002 | OBS | `tempfile` unused in `slideforge` crate (speculative retention) | REMEDIATED |

---

## Finding Details

### F-050-P2-HIGH-001 — AC-007 regression guards vacuously true (HIGH)

**Classification:** HIGH — correctness defect, load-bearing test failure

**Description:**
The pass-1 fix for AC-007 (Gap 3 — observability span markers) was itself a paper-fix. The
regression guard tests checked a `stage="..."` structured field that no production code emitted.
Specifically:

1. `stage="<name>"` structured fields were NOT present in any `info_span!` call in the production
   code after the pass-1 fix. The tests asserted on these fields, but the assertions were vacuously
   true because `tracing-test` captures matched by span name, not by field value — the tests passed
   without the field being emitted.

2. The positive assertions checked literal message text (e.g., `"parse stage"`, `"evaluate stage"`)
   that was decoupled from the actual span name. A span could be renamed without breaking the test.

**Root cause:** Pass-1 remediation renamed spans correctly but did not add `stage="<name>"` as a
structured field to the `info_span!` macro calls. The tests were written asserting on a field that
the instrumentation never emitted.

**Remediation (PROVEN load-bearing):**
Implementer added `stage="<name>"` structured field to ALL 6 canonical `info_span!` calls in
`slideforge/src/lib.rs` `build_inner`:
- `info_span!("parse", stage = "parse")`
- `info_span!("evaluate", stage = "evaluate")`
- `info_span!("brand", stage = "brand")`
- `info_span!("validate", stage = "validate")`
- `info_span!("layout", stage = "layout")`
- `info_span!("export", stage = "export")`

Tests were rewritten to assert on the `stage` structured field directly.

**Load-bearing proof (TD-VSDD-059):** Implementer temporarily renamed `evaluate` span to `eval`
and confirmed that the AC-007 test suite FAILED (span name mismatch detected). Span was then
restored to `evaluate` and all tests passed. This demonstrates the guards are not vacuous — the
test is genuinely coupled to the production span names and field values.

**Remediation commit:** `5d34b5ae`

---

### OBS-050-P2-001 — Gap-1 title None-path zero coverage (OBS)

**Classification:** OBS — coverage gap

**Description:**
Gap-1 fixed `eval_deck_with_variant` to derive document title from the first `slide title:` block's
`title` field. However, the test coverage for this fix had zero direct assertions on the None-path
(i.e., what happens when no title slide is present, or when the title slide has no `title` field).
The existing tests only exercised the Some-path (title present → metadata populated). The None-path
code was untested.

**Remediation:**
Three unit tests added to the eval crate directly targeting `eval_deck_with_variant`:

1. **Some-path test** — deck with a title slide containing a `title` field: asserts
   `DeckMetadata.title == Some("My Deck Title")`.

2. **None-path no-title-slide test** — deck with no `slide title:` block at all: asserts
   `DeckMetadata.title == None`.

3. **None-path title-slide-without-title-field test** — deck with a `slide title:` block but
   no `title` field defined: asserts `DeckMetadata.title == None`.

All three tests call `eval_deck_with_variant` directly (not the full `build()` pipeline), isolating
the title-derivation logic under test without external dependencies.

**Remediation commit:** `10112c18`

---

### OBS-050-P2-002 — `tempfile` unused in `slideforge` crate (OBS)

**Classification:** OBS — dead dependency

**Description:**
`tempfile = "=3.27.0"` was retained in `crates/slideforge/Cargo.toml` [dev-dependencies] after
the E2E test design was finalized. The E2E tests use in-memory output bytes (no temporary files),
so `tempfile` was never imported or used in any `slideforge` crate test. This was a speculative
retention from an earlier draft of the test design.

**Remediation:**
`tempfile` removed from `crates/slideforge/Cargo.toml` dev-dependencies. Real usages confirmed
retained in `slideforge-brand`, `slideforge-data`, and `slideforge-pdf` (all legitimate).

STORY-050 story spec updated to v1.3: Library & Framework Requirements table row for `tempfile`
now explicitly notes removal from the `slideforge` crate with justification; Cargo.toml File
Structure entry updated to reflect dev-dependencies without tempfile.

**Remediation commit:** `b253de47`
**Story spec alignment:** `spec_version: "1.3"` (persisted in `.factory/stories/stories/STORY-050-e2e-integration-tests.md`)

---

## Confirmed-Clean Dimensions

The following areas were probed and confirmed clean in pass 2:

| Dimension | Verdict |
|-----------|---------|
| ADR-018 conformance — post-layout validation pass wiring | CLEAN |
| Decision 5a error-precedence rule (ADR-018 v1.1) | CLEAN |
| Gap-2 post-layout validation pass — AltTextValidator fires on ContentBlock::Chart/Image | CLEAN |
| EC-001 (basic PPTX export end-to-end) | CLEAN |
| 6 exact span names (parse/evaluate/brand/validate/layout/export) | CLEAN — confirmed via load-bearing rename test |
| No forbidden patterns (unwrap outside tests, println in lib, f64 EMU) | CLEAN |
| OBS-1 (CanvasOverflowValidator inert) — not re-raised | ANCHORED (pre-existing; ADR-018 Decision 4 deferral) |

## Implementer Commits (pass-2 era)

| SHA | Description |
|-----|-------------|
| `5d34b5ae` | AC-007 stage= structured field added to all 6 info_span! calls; tests rewritten with load-bearing proof (rename→FAIL→restore→PASS) |
| `10112c18` | Gap-1 title None-path: 3 eval-crate unit tests (Some-path, None-path no-title-slide, None-path title-slide-without-title-field) |
| `b253de47` | tempfile removed from slideforge Cargo.toml dev-dependencies (OBS-050-P2-002) |
| `23481e1e` | Pre-push canonical gate green (fmt + pedantic clippy + nextest workspace 3242/3242) |

## Workspace Gate (post-remediation)

| Gate | Result |
|------|--------|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items` | PASS |
| `cargo nextest run --workspace --no-fail-fast` | PASS (3242/3242) |
| `cargo test --doc --workspace` | PASS |
| `cargo insta test --check --workspace` | PASS |

**Streak after pass 2: 0/3** — pass 2 was NOT clean (HIGH present). Remediation complete.
Next: adversary pass 3 on `feature/STORY-050` HEAD `23481e1e`.
