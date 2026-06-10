---
document_type: adversary-pass-report
story_id: STORY-081
pass: 32
scope: PR-level-diff
branch_head_at_review: be160e2a08bfa01aa0dc208a1192a7854ecb171e
develop_base: 15838de1438f37341e6e4ae234a5c16177510264
verdict_clean_strict: false
verdict_clean_pr_merge: true
streak_after: "1/3 (PR-merge clean — streak requires strict clean for advancement; this pass counts toward merge gate only)"
findings_count: 1
findings_crit: 0
findings_high: 0
findings_med: 0
findings_low: 0
findings_obs: 1
date: 2026-06-10
---

# Adversary Pass 32 — STORY-081 (PR-level gate)

**CLEAN (strict): no**
**CLEAN (PR-merge): yes**
Streak after this pass: 1/3 toward strict convergence (PR-merge gate is satisfied — zero CRIT/HIGH/MED)

## Scope

PR-level diff review. Focus areas per invocation:
- P31-MED-001 fix: caption+description match arms in `thread_fields_to_blocks`
- Consistency of new arms with body/subtitle pattern
- TextTag::Untagged correctness for caption and description fields
- Load-bearing quality of the 4 new tests
- Any remaining inline-content fields lacking match arms
- Thread-local counter fix in `font.rs` (LOAD_SYSTEM_FONTS_THREAD_COUNT)

Commits in scope since pass 31:
- `fix(pdf): thread-local counter fixes test_obs_p09_001 parallel isolation` (99d36267)
- `fix(eval): caption+description inline markup no longer silently dropped (P31-MED-001)` (be160e2a)

---

## Part A — Fix Verification

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| P31-MED-001 | MED | RESOLVED | `thread_fields_to_blocks` now has match arms for `"caption"` and `"description"`. Evidence below. |
| OBS-P09-001 (context) | OBS | RESOLVED | Thread-local counter `LOAD_SYSTEM_FONTS_THREAD_COUNT` isolates test from parallel execution. Evidence below. |

---

## Part B — Fix Verification Evidence

### P31-MED-001: caption+description match arms

**Claim:** `thread_fields_to_blocks()` lacked match arms for `"caption"` and `"description"`. Inlines variant fell through to wildcard and was silently dropped.

**Evidence of resolution (field_to_block.rs lines 171–215):**

```rust
// ── 4a. Caption ──────────────────────────────────────────────────────────
match slide.fields.get("caption") {
    Some(FieldValue::Inlines(nodes)) if !nodes.is_empty() => {
        slide.blocks.push(make_text_block_tagged_inlines(
            nodes.clone(),
            TextTag::Untagged,
        ));
    },
    _ => {
        if let Some(text) = extract_str_field(slide, "caption")
            && !text.trim().is_empty()
        {
            slide.blocks.push(make_text_block_tagged(text, TextTag::Untagged));
        }
    },
}
// ── 4b. Description ──────────────────────────────────────────────────────
match slide.fields.get("description") { /* identical pattern */ }
```

**Consistency with body/subtitle pattern:** PASS. The `body` arm (lines 154–168) and `subtitle` arm (lines 128–144) both use the identical two-arm `match` pattern:
1. `Some(FieldValue::Inlines(nodes)) if !nodes.is_empty()` → `make_text_block_tagged_inlines(nodes.clone(), tag)`
2. `_` → `extract_str_field` fallback

The caption and description arms follow this pattern exactly.

**TextTag::Untagged correctness:** PASS. `caption` and `description` do not correspond to any PPTX semantic placeholder (`type="title"`, `type="subTitle"`, `type="body"`). `TextTag::Untagged` is the correct semantic: it routes through `TextTag::Untagged →` layout `TextRun` arm (layout.rs:427) → `FrameContent::TextRun(inlines)` → all exporters receive the inline structure via their TextRun path. This is consistent with the documented routing: `TextTag::Untagged → FrameContent::TextRun (generic inline run)` (make_text_block_tagged docstring).

### 4 New Tests: Load-Bearing Assessment

All four tests are load-bearing assertions. They are NOT paper fixes.

**test_p31_med_001_caption_inlines_produces_text_block (field_to_block.rs:906)**
- Creates `FieldValue::Inlines([Bold([Plain("bold caption")])])` for `"caption"` field
- Calls `thread_fields_to_blocks`
- Asserts: exactly 1 block, `ContentBlock::Text`, `TextTag::Untagged`, first inline is `InlineNode::Bold`
- Would FAIL on the pre-fix code (no match arm → wildcard → 0 blocks)
- LOAD-BEARING: yes

**test_p31_med_001_description_inlines_produces_text_block (field_to_block.rs:940)**
- Creates `FieldValue::Inlines([Italic([Plain("italic description")])])` for `"description"` field
- Same structural assertions with `InlineNode::Italic`
- Would FAIL on the pre-fix code
- LOAD-BEARING: yes

**test_p31_med_001_caption_plain_str_produces_text_block (field_to_block.rs:973)**
- Creates `FieldValue::Literal(Value::Str("plain caption text"))` for `"caption"` field
- Asserts: 1 block, `ContentBlock::Text`
- Verifies the `_` fallback arm handles plain strings correctly (backward compatibility)
- Previously caption had NO arms at all, so this test is also new and load-bearing
- LOAD-BEARING: yes

**test_p31_med_001_description_plain_str_produces_text_block (field_to_block.rs:988)**
- Same as above for description
- LOAD-BEARING: yes

### OBS-P09-001: Thread-local counter isolation (font.rs)

**Claim:** Process-global counter delta was unreliable under parallel test execution; other tests calling `resolve_font_set` on other threads incremented the global counter, producing a false delta of 15 instead of 1.

**Evidence of resolution (font.rs lines 49–86):**

```rust
#[cfg(test)]
thread_local! {
    static LOAD_SYSTEM_FONTS_THREAD_COUNT: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
}

fn load_system_fonts_counted(db: &mut fontdb::Database) {
    db.load_system_fonts();
    LOAD_SYSTEM_FONTS_COUNT.fetch_add(1, Ordering::Relaxed);
    #[cfg(test)]
    LOAD_SYSTEM_FONTS_THREAD_COUNT.with(|c| c.set(c.get() + 1));
}
```

The `thread_local!` counter starts at 0 per thread. Parallel tests on other threads do not affect the delta. The test `test_obs_p09_001_resolve_font_set_loads_system_fonts_exactly_once` uses the delta pattern: `count_before = load_system_fonts_thread_call_count()` then `count_after - count_before` — isolating the measurement to exactly one `resolve_font_set` call on the current thread.

**Structural enforcement:** `load_system_fonts_counted` is the ONLY call site of `db.load_system_fonts()` in the entire crate (confirmed by grep — no other direct calls exist). The `LOAD_SYSTEM_FONTS_THREAD_COUNT` increment is structural, not advisory.

**Scope of `#[cfg(test)]`:** The thread-local is `#[cfg(test)]`-gated, so it adds zero overhead in release builds. The `load_system_fonts_thread_call_count()` accessor is also `#[cfg(test)] pub(crate)`. Correct.

---

## INLINE_CONTENT_FIELDS completeness sweep

`INLINE_CONTENT_FIELDS = ["bullets", "body", "caption", "description", "subtitle"]`

All five fields now have correct match arms in `thread_fields_to_blocks`:

| Field | Match arm(s) present | TextTag | Notes |
|-------|---------------------|---------|-------|
| `title` | Not in INLINE_CONTENT_FIELDS | N/A | Title uses plain extract_str_field; AC-006 dual-title shadow is separate |
| `subtitle` | ✓ (lines 128–144) | `TextTag::Subtitle` | Added in STORY-081 C2 fix |
| `body` | ✓ (lines 154–168) | `TextTag::Body` | Added in STORY-081 C2 fix |
| `caption` | ✓ (lines 176–192) | `TextTag::Untagged` | Added in P31-MED-001 fix |
| `description` | ✓ (lines 199–215) | `TextTag::Untagged` | Added in P31-MED-001 fix |
| `bullets` | ✓ (lines 217–275) | N/A (ContentBlock::Bullets) | Handles List + Inlines + InlinesList |

No INLINE_CONTENT_FIELDS member is missing a match arm.

---

## Per-Seam Verification Table (Pass 32 focus)

| Focus area | Result | Notes |
|---|---|---|
| `caption` new arm: FieldValue::Inlines → ContentBlock::Text(Untagged) | PASS | Match arm at lines 176–192 |
| `description` new arm: FieldValue::Inlines → ContentBlock::Text(Untagged) | PASS | Match arm at lines 199–215 |
| Caption arm consistent with body pattern | PASS | Identical two-arm match structure |
| Description arm consistent with body pattern | PASS | Identical two-arm match structure |
| TextTag::Untagged correct for caption | PASS | No PPTX semantic placeholder for caption |
| TextTag::Untagged correct for description | PASS | No PPTX semantic placeholder for description |
| test_p31_med_001_caption_inlines_produces_text_block — load-bearing | PASS | Would fail pre-fix; asserts Bold inline node present |
| test_p31_med_001_description_inlines_produces_text_block — load-bearing | PASS | Would fail pre-fix; asserts Italic inline node present |
| test_p31_med_001_caption_plain_str_produces_text_block — load-bearing | PASS | Backward compat; asserts ContentBlock::Text produced |
| test_p31_med_001_description_plain_str_produces_text_block — load-bearing | PASS | Backward compat; asserts ContentBlock::Text produced |
| All INLINE_CONTENT_FIELDS have match arms | PASS | All 5 covered |
| Thread-local counter: LOAD_SYSTEM_FONTS_THREAD_COUNT | PASS | Isolates OBS-P09-001 test from parallel tests |
| Thread-local counter: #[cfg(test)] gated | PASS | Zero release overhead |
| load_system_fonts only via counted wrapper | PASS | No other direct calls in crate |
| Workspace: pass/skip/fail | PASS | Commit message: 4089 pass / 20 skip / 0 fail; clippy+fmt clean |

---

## New Finding (OBS)

### OBS-P32-001 — OBS-P31-001 e2e requirement not met in fix burst

**Severity:** OBS (observation; non-blocking for PR-merge)

**Location:** `crates/slideforge/tests/e2e/story_081_inline_markup_e2e.rs` (gap — file not modified)

**Description:**

Pass 31 report (OBS-P31-001) stated explicitly:

> "Not blocking PR-merge pending MED fix, **but must be added in the same fix burst.**"

The OBS-P31-001 requirement was: add an e2e test that exercises the full pipeline (DSL → eval → layout → any exporter) for a slide with an italic `caption` field (or bold `description` field) and confirms the inline markup reaches exporter-native output (e.g., `<strong>` in HTML, or `b="1"` in PPTX, or the absence of literal `**` characters).

The P31-MED-001 fix burst (`fix(eval): caption+description inline markup no longer silently dropped`) changed only `crates/slideforge-eval/src/field_to_block.rs`. No e2e test was added. No fixture file for caption inline markup was created.

**Scope of current coverage gap:**

The 4 new unit tests in `field_to_block.rs` verify Stage 2b only (field threading to ContentBlock). They do NOT cover:
- Layout stage: `TextTag::Untagged` → `FrameContent::TextRun(inlines)` for caption content
- Any exporter receiving the TextRun frames for caption/description

Given that the layout routing for `TextTag::Untagged` (layout.rs:427–456) is pre-existing and tested by other paths, the risk of a regression in this specific routing is low but not zero. The stage-2b tests are the primary load-bearing guard; the e2e test would provide belt-and-suspenders protection at the full-pipeline level as required by OBS-P31-001.

**Action required (non-blocking):**

Add to `crates/slideforge/tests/e2e/story_081_inline_markup_e2e.rs` (or equivalent):
1. A fixture file `story-081-caption-inline.sf` with a slide containing `caption: "_italic caption_"` and `description: "**bold description**"`.
2. An e2e test that builds the deck and asserts format-native markup in at least one exporter (HTML is cheapest: assert `<em>italic caption</em>` in output bytes and absence of `_italic caption_` literal).

This is OBS-only (non-blocking for merge) because: (a) the PR-merge gate is CRIT+HIGH+MED only; (b) the 4 load-bearing unit tests in field_to_block.rs DO guard the critical failure path; (c) the layout/exporter paths for TextTag::Untagged are shared infrastructure already exercised by the body/subtitle pipeline.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |
| OBS | 1 (OBS-P32-001: missing e2e test for caption/description pipeline, required in same burst by OBS-P31-001) |

**Overall Assessment:** pass-with-obs
**Convergence:** Findings remain (OBS) — but PR-merge gate is satisfied
**Readiness:** READY FOR MERGE under BC-5.39.001 PR-merge gate (zero CRIT+HIGH+MED)

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 32 |
| **New findings** | 1 (OBS-P32-001) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (single OBS observation) |
| **Median severity** | OBS (below LOW; non-blocking) |
| **Trajectory** | ...→1 MED (P31)→0 CRIT/HIGH/MED (P32) |
| **Verdict** | FINDINGS_REMAIN (OBS only) — PR-merge gate SATISFIED |

**CLEAN (strict): no** (OBS finding present — strict requires zero findings of any severity)
**CLEAN (PR-merge): yes** (zero CRIT+HIGH+MED — BC-5.39.001 PR-merge gate satisfied)
