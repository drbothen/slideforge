---
document_type: adversary-pass-report
story_id: STORY-081
pass: 33
scope: PR-level-diff
branch_head_at_review: a490aacb04a419e0c2fe783df5ef31784bc5f57c
develop_base: 15838de1438f37341e6e4ae234a5c16177510264
verdict_clean_strict: false
verdict_clean_pr_merge: true
streak_after: "2/3 toward strict convergence (PR-merge gate satisfied; OBS-only findings remain)"
findings_count: 2
findings_crit: 0
findings_high: 0
findings_med: 0
findings_low: 0
findings_obs: 2
date: 2026-06-10
---

# Adversary Pass 33 — STORY-081 (PR-level gate)

**CLEAN (strict): no**
**CLEAN (PR-merge): yes**
Streak after this pass: 2/3 toward strict convergence (PR-merge gate is satisfied — zero CRIT/HIGH/MED)

## Scope

PR-level diff review. Focus areas per invocation:
- OBS-P32-001 closure: e2e tests for caption/description inline markup reaching exporters
- story-081-caption-markup.sf fixture quality and completeness
- `test_p31_med_001_caption_bold_reaches_pptx_exporter` — load-bearing check
- `test_p31_med_001_description_italic_reaches_html_exporter` — load-bearing check
- Any new CRIT/HIGH/MED findings introduced by the fix commit

Commit in scope since pass 32:
- `test(eval,e2e): e2e tests for caption inline markup reaching exporters (OBS-P32-001)` (a490aacb)

Changed files:
- `crates/slideforge/tests/e2e/story_081_inline_markup_e2e.rs` (+74 lines)
- `crates/slideforge/tests/fixtures/story-081-caption-markup.sf` (+25 lines)

---

## Part A — Fix Verification

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| OBS-P32-001 | OBS | PARTIALLY_RESOLVED | Two e2e tests added; however the second test name misrepresents what it exercises, and the description field pipeline has no coverage. See OBS-P33-001 and OBS-P33-002. |

---

## Part B — Fix Verification Evidence

### OBS-P32-001 e2e tests: Load-Bearing Assessment

**Claim:** Two new e2e tests were added to `story_081_inline_markup_e2e.rs` to close OBS-P32-001 (missing full-pipeline coverage for caption/description inline markup).

#### test_p31_med_001_caption_bold_reaches_pptx_exporter

**Pipeline:** `story-081-caption-markup.sf` → `slideforge::build(_, "pptx")` → open ZIP → `ppt/slides/slide2.xml` → assert `b="1"` present and `**` absent.

**Fixture analysis:** `story-081-caption-markup.sf` uses `slide image:` type with `caption "See **bold caption** for details."`. The `image` slide type lists `"caption"` in `known_fields.rs` (line 140). The caption contains `**bold caption**`, which the parser converts to `TemplateChunk::Bold([Literal("bold caption")])`, `eval_slide_node` stores as `FieldValue::Inlines([Bold([Plain("bold caption")])])`, and `thread_fields_to_blocks` (P31-MED-001 fix) converts to `ContentBlock::Text(TextTag::Untagged)`. The PPTX exporter then routes this through the ADR-024 unified engine producing `b="1"` in `<a:rPr>`.

**RED GATE validity:** PASS. Would have FAILED before P31-MED-001 fix (no match arm for `"caption"` → wildcard → 0 blocks → no `<a:r>` → no `b="1"`).

**Load-bearing:** yes.

#### test_p31_med_001_description_italic_reaches_html_exporter

**Pipeline:** same `story-081-caption-markup.sf` → `slideforge::build(_, "html")` → assert `<strong>` present and `**` absent.

**Fixture analysis:** `story-081-caption-markup.sf` contains `caption "See **bold caption** for details."` and NO `description` field. This test builds the HTML output and asserts `<strong>` — which is produced by the `caption` field's bold text, NOT by any `description` field.

**Name/content mismatch:** The test is named `test_p31_med_001_description_italic_reaches_html_exporter` but:
1. It does NOT test the `description` field at all.
2. The assertion checks for `<strong>` (bold), not `<em>` (italic).
3. The error message says "for **bold caption**" — the caption field, not description.

The test verifies that caption bold markup reaches the HTML exporter, which is load-bearing for the caption field. However, it provides ZERO coverage for the description field pipeline.

**Load-bearing for caption HTML path:** yes.
**Load-bearing for description field pipeline:** no (zero coverage).

---

## Per-Seam Verification Table (Pass 33 focus)

| Focus area | Result | Notes |
|---|---|---|
| Fixture uses `image:` type where `caption` is a defined known field | PASS | `known_fields.rs` line 140 lists `"caption"` for `"image" \| "screenshot"` |
| Caption `**bold caption**` → `FieldValue::Inlines([Bold([Plain(...)])])` in eval | PASS | `INLINE_CONTENT_FIELDS` includes `"caption"` in `for_eval.rs` line 202 |
| P31-MED-001 match arm routes `FieldValue::Inlines` → `ContentBlock::Text(Untagged)` | PASS | `field_to_block.rs` lines 176-192 |
| PPTX e2e test asserts `b="1"` and no `**` in slide2.xml | PASS (load-bearing) | `test_p31_med_001_caption_bold_reaches_pptx_exporter` |
| HTML e2e test asserts `<strong>` and no `**` | PASS (load-bearing for caption) | `test_p31_med_001_description_italic_reaches_html_exporter` |
| Test name matches test content | FAIL (OBS) | See OBS-P33-001 below |
| Description field pipeline has e2e coverage | FAIL (OBS) | See OBS-P33-002 below |
| Workspace: pass/skip/fail | PASS | Commit message: 4091 pass / 20 skip / 0 fail; clippy+fmt clean |
| No new CRIT/HIGH/MED findings | PASS | No new issues of these severities found |

---

## New Findings (OBS)

### OBS-P33-001 — Test name `test_p31_med_001_description_italic_reaches_html_exporter` misrepresents its coverage

**Severity:** OBS (observation; non-blocking for PR-merge)

**Location:** `crates/slideforge/tests/e2e/story_081_inline_markup_e2e.rs` lines 921-948

**Description:**

The test is named `test_p31_med_001_description_italic_reaches_html_exporter` but it:
1. Uses the same fixture as the PPTX test (`story-081-caption-markup.sf`), which has no `description` field.
2. Asserts `<strong>` (bold) is present — not `<em>` (italic), as the name implies.
3. The assertion error message explicitly says "for **bold caption**" — the caption field.

The test is semantically equivalent to an HTML-path complement of `test_p31_med_001_caption_bold_reaches_pptx_exporter`. It provides genuine value for the caption HTML pipeline, but the name misleads reviewers into believing the description field is covered.

**Evidence:**
```rust
fn test_p31_med_001_description_italic_reaches_html_exporter() {
    let source = fixture_source("story-081-caption-markup.sf");  // caption only, no description
    ...
    assert!(html.contains("<strong>"),   // bold, not italic
        "... for **bold caption** ..."   // caption field, not description
    );
}
```

**Impact:** Not blocking. The caption HTML path is covered. The misleading name creates maintenance confusion (future readers may think description is e2e-tested when it is not).

**Recommendation:** Rename to `test_p31_med_001_caption_bold_reaches_html_exporter` to match what it actually tests. This is a one-line rename.

---

### OBS-P33-002 — Description field has no e2e pipeline coverage (OBS-P32-001 requirement partially unmet)

**Severity:** OBS (observation; non-blocking for PR-merge)

**Location:** `crates/slideforge/tests/e2e/story_081_inline_markup_e2e.rs` and `crates/slideforge/tests/fixtures/`

**Description:**

Pass 32 (OBS-P32-001) explicitly required:

> "A fixture file `story-081-caption-inline.sf` with a slide containing `caption: "_italic caption_"` and `description: "**bold description**"`."

The implementation provides `story-081-caption-markup.sf` with only `caption "See **bold caption** for details."` — no `description` field. Neither of the two new e2e tests exercises the `description` field's pipeline at all.

**Coverage gap:**

The `description` field has:
- Unit test coverage in `field_to_block.rs` (Stage 2b only — `thread_fields_to_blocks` → `ContentBlock::Text`)
- NO e2e coverage from DSL input through eval → layout → any exporter

The risk profile is low for the same reasons stated in P32:
- The `"description"` match arm in `thread_fields_to_blocks` is structurally identical to the `"caption"` arm
- `TextTag::Untagged` routing is tested by caption (same code path)
- Layout and exporter paths for `TextTag::Untagged` are exercised by the caption tests

However, the `description` field has a second-order concern: it is not listed in `known_fields.rs` for ANY slide type. This means:
1. There is no DSL-level slide type that natively declares `description` as a valid field.
2. A user writing `description: "text"` on a `content` slide today will have it silently accepted by the parser (field-level validation against `known_fields` is only applied to set-rules, not slide blocks), processed through `INLINE_CONTENT_FIELDS` and `thread_fields_to_blocks`, and emitted to exporters — but the parser does NOT validate this as a known field.
3. This is an existing inconsistency in the design (description exists in `INLINE_CONTENT_FIELDS` and `thread_fields_to_blocks` but not in `known_fields` for any type) and is not introduced by this fix burst.

**What was delivered:**
- 1 PPTX e2e test for caption bold (load-bearing)
- 1 HTML e2e test for caption bold (load-bearing, despite misleading name)

**What OBS-P32-001 requested but was not delivered:**
- description field in fixture
- italic variant (not just bold) for caption

**Action required (non-blocking):** Add a description field to either the existing caption fixture or a new fixture, and add an e2e test that exercises the description field end-to-end. Also consider whether `description` should be added to `known_fields` for relevant slide types (content, bio, etc.) — this may require a separate follow-up story.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |
| OBS | 2 (OBS-P33-001: test name mismatch; OBS-P33-002: description field still not e2e covered) |

**Overall Assessment:** pass-with-obs
**Convergence:** Findings remain (OBS only) — PR-merge gate is satisfied
**Readiness:** READY FOR MERGE under BC-5.39.001 PR-merge gate (zero CRIT+HIGH+MED)

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 33 |
| **New findings** | 2 (OBS-P33-001: test name mismatch; OBS-P33-002: description no e2e) |
| **Duplicate/variant findings** | 0 (OBS-P32-001 closed for caption; description gap is a refinement of the unfulfilled part of P32 requirement) |
| **Novelty score** | 1.0 (two OBS observations) |
| **Median severity** | OBS (below LOW; non-blocking) |
| **Trajectory** | ...→1 MED (P31)→1 OBS (P32)→2 OBS (P33: caption closed, description gap persists + test name finding) |
| **Verdict** | FINDINGS_REMAIN (OBS only) — PR-merge gate SATISFIED |

**CLEAN (strict): no** (OBS findings present — strict requires zero findings of any severity)
**CLEAN (PR-merge): yes** (zero CRIT+HIGH+MED — BC-5.39.001 PR-merge gate satisfied)
