---
document_type: adversary-pass-report
story_id: STORY-081
pass: 31
scope: PR-level-diff
branch_head_at_review: 99d362674261c7f19e3d9e2259d2b42ef804dc88
develop_base: 15838de1438f37341e6e4ae234a5c16177510264
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3 (reset — CRIT/HIGH/MED finding present)"
findings_count: 1
findings_crit: 0
findings_high: 0
findings_med: 1
findings_low: 0
findings_obs: 2
date: 2026-06-10
---

# Adversary Pass 31 — STORY-081 (PR-level gate)

**CLEAN (strict): no**
**CLEAN (PR-merge): no**
Streak after this pass: 0/3 (reset — MED finding)

## Scope

PR-level diff review. Focus areas per invocation:
- FieldValue::InlinesList variant (eval List→InlinesList conversion, register_routing None+warn arm, layout/sections UnresolvedTakeaway arm, field_to_block bullets)
- EC-007 in list path
- Hash+Eq+Clone on InlinesList
- STORY-081×STORY-088 seam
- Whether other inline-content fields have DSL test gaps
- `#[allow(clippy::too_many_lines)]` on DOCX serialize()

Commits in scope: `feat(eval,types,pptx,docx,pdf,html)`, `fix(docx,test,docs)`, `fix(eval,types,docx)`, `docs(demo)`, `fix(pdf)`.

---

## Finding P31-MED-001 — caption/description inline markup silently dropped end-to-end

**Severity:** MED
**Files:**
- `crates/slideforge-eval/src/for_eval.rs` (INLINE_CONTENT_FIELDS constant, line 202)
- `crates/slideforge-eval/src/field_to_block.rs` (thread_fields_to_blocks, no arms for caption/description)

### Evidence

`INLINE_CONTENT_FIELDS` in `for_eval.rs` includes `"caption"` and `"description"`:

```rust
const INLINE_CONTENT_FIELDS: &[&str] = &["bullets", "body", "caption", "description", "subtitle"];
```

This means `eval_slide_node` will produce `FieldValue::Inlines(nodes)` for a `caption` or `description` field containing markup. Tests `test_BC_3_05_001_ac001_caption_inline_markup_produces_inlines` and `test_BC_3_05_001_ac001_description_inline_markup_produces_inlines` verify this eval-stage behavior.

However, `thread_fields_to_blocks()` in `field_to_block.rs` has NO match arms for `"caption"` or `"description"`. The function only threads `"title"`, `"subtitle"`, `"body"`, `"bullets"`, `"label"`, media blocks, shape blocks, and color-coded slide fields. A `FieldValue::Inlines` for `"caption"` or `"description"` falls through to no arm and is silently dropped — the inlines never reach any exporter.

### Spec claim being violated

BC-3.05.001 field scope table (line 284-285):
```
| `caption`     | Yes | Full inline markup; all exporters |
| `description` | Yes | Full inline markup; all exporters |
```

The story spec (STORY-081-slide-level-inline-markup.md line 118-119) explicitly lists caption and description:
> Extend `eval_slide_node` to call `chunks_to_inline_nodes` ... for slide fields that semantically carry inline content: `bullets` list items, `body`, `caption`, `description`, `subtitle`.

### Impact

Any slide author who writes `caption: "_italic caption_"` or `description: "**key metric**"` will:
1. Get `FieldValue::Inlines` at eval time (correct)
2. Get silently dropped inlines in `thread_fields_to_blocks` (defect)
3. Get NO caption/description in any exporter output (defect)

This is not a corner case — image slides routinely carry `caption` fields, and the spec explicitly promises full inline markup support for them.

### Required fix

Add match arms for `"caption"` and `"description"` in `thread_fields_to_blocks()` analogous to the existing `"body"` arms (lines 154-168 in `field_to_block.rs`). Both fields use `TextTag::Untagged` (they have no dedicated placeholder routing), so the pattern is:

```rust
// ── caption / description ────────────────────────────────────────────
match slide.fields.get("caption") {
    Some(FieldValue::Inlines(nodes)) if !nodes.is_empty() => {
        slide.blocks.push(make_text_block_tagged_inlines(nodes.clone(), TextTag::Untagged));
    },
    _ => {
        if let Some(text) = extract_str_field(slide, "caption")
            && !text.trim().is_empty()
        {
            slide.blocks.push(make_text_block_tagged(text, TextTag::Untagged));
        }
    },
}
// ... same pattern for "description"
```

Additionally, load-bearing tests must verify that:
1. A slide with `caption: "_italic_"` produces a `ContentBlock::Text` block with `InlineNode::Italic`
2. A slide with `description: "**bold**"` produces a `ContentBlock::Text` block with `InlineNode::Bold`

The existing eval-level tests (`test_BC_3_05_001_ac001_caption_inline_markup_produces_inlines`) do NOT cover the threading-to-block step. New tests in `field_to_block.rs` are required.

---

## OBS-P31-001 — No e2e test for caption/description inline markup reaching exporters

**Severity:** OBS (observation)

The eval-level tests verify that `caption`/`description` produce `FieldValue::Inlines`. But there are no e2e tests (in `story_081_inline_markup_e2e.rs` or `crates/slideforge/tests/`) confirming that a slide with an italic caption produces `<em>italic</em>` in HTML or `<a:rPr i="1"/>` in PPTX. Once P31-MED-001 is fixed, an e2e test should be added to guard the full pipeline for these fields. (Not blocking PR-merge pending MED fix, but must be added in the same fix burst.)

---

## OBS-P31-002 — `#[allow(clippy::too_many_lines)]` on DOCX serialize() is correctly justified

**Severity:** OBS (non-finding — confirming compliance)

The `serialize` function in `document_body.rs` (lines 139-697, ~558 lines) has `#[allow(clippy::too_many_lines)]` with comment:
> `serialize` is a single-pass accumulator over slides; splitting it would obscure the sequential semantics of the body/detail paragraph routing (BC-4.02.001 invariant 1) without improving readability.

This is a valid CLAUDE.md exception: "clippy::pedantic enabled with documented exceptions only." The justification is accurate — the function is a sequential accumulation pipeline where the ordering of body/subtitle/TextRun/detail sections is a contract boundary (BC-4.02.001 invariant 1). No finding.

---

## Per-Seam Verification Table (PR-level diff focus areas)

| Focus area | Result | Notes |
|---|---|---|
| `FieldValue::InlinesList` variant — Hash+Eq+Clone | PASS | Derived on `FieldValue` enum; `InlineNode` already derives these |
| `InlinesList` eval: List→InlinesList conversion | PASS | `eval_list_field` correct; mixed plain+markup items handled; plain-only stays `Literal(List)` |
| `InlinesList` eval: EC-007 `{{ var }}` stays Plain | PASS | `chunks_to_inline_nodes` wraps resolved Expr as `Plain(s)`, not re-parsed |
| `InlinesList` register_routing: None+warn arm | PASS | `FieldValue::InlinesList` → `tracing::warn!` + `None`; observable, not silent |
| `InlinesList` layout/sections: UnresolvedTakeaway arm | PASS | `FieldValue::InlinesList` added to exhaustive match in `sections.rs` |
| `InlinesList` field_to_block: bullets arm | PASS | `Some(FieldValue::InlinesList(items))` → per-item `BulletItem` with preserved nodes |
| STORY-081×STORY-088 seam: list-literal bullets with markup | PASS | `eval_list_field` invoked from `FieldValue::List` arm; any_item_has_markup gates correctly |
| `caption`/`description` field eval → `FieldValue::Inlines` | PASS | Eval tests confirm |
| `caption`/`description` field threading → blocks | **FAIL** | No match arms in `thread_fields_to_blocks`; inlines silently dropped — P31-MED-001 |
| `#[allow(clippy::too_many_lines)]` on DOCX serialize | PASS | Justified; documented exception per CLAUDE.md |
| `apply_run_property` FnOnce→Fn change | PASS | Required because `f` is called ≥1 times in WHyperlink arm |
| `source_index` bounds safety (DOCX dual-title path) | PASS | `semantic_deck.slides.get(slide.source_index)` is safe (Option, not index); 1:1 mapping from layout |
| `SubtitleInlines` FrameContent — Hash+Eq+Clone | PASS | FrameContent derives these; InlineNode already satisfies bounds |
| `SubtitleInlines` handled in: PPTX, DOCX, HTML, PDF, a11y | PASS | All 5 callsites present |
| `title_inlines` shadow field: DOCX, PDF, HTML | PASS | All 3 non-PPTX exporters check and use it |
| PPTX ignores `title_inlines` shadow field | PASS | No reference in pptx slide_serializer.rs |
| `EvalError::InlineMarkupInTitle` is Error (not Warning) severity | PASS | `push_with_severity(..., ParseSeverity::Error)` confirmed |
| Snapshot hygiene | NOT CHECKED in this pass (out of scope for MED fix) | — |

---

## Verdict

**CLEAN (strict): no**
**CLEAN (PR-merge): no**

One MED finding (P31-MED-001): `caption` and `description` fields with inline markup are silently dropped in `thread_fields_to_blocks()`. The eval stage correctly produces `FieldValue::Inlines` for these fields, but the threading pass has no arms to carry them to `ContentBlock`, so no exporter receives the inline structure.

This violates BC-3.05.001's "Full inline markup; all exporters" claim for caption and description, and violates the story's Three-Area Work Plan which explicitly includes these fields.

BC-5.39.001 PR-merge gate is blocked (MED finding present). Fix required before merge.
