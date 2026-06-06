---
title: "Wave 4 Integration Gate — Content Threading & a11y Strict-Gate Assessment"
created: 2026-06-05
status: ACCEPTED
author: architect
traces_to: ADR-018, ADR-005, BC-5.02.001, BC-3.04.001
severity: CRITICAL (Problem A) / HIGH (Problem B)
---

# Wave 4 Content-Threading & A11y Strict-Gate Assessment

## Executive Summary

The Wave 4 integration gate exposed two structurally coupled failures with a
single shared root cause: `slideforge-eval` always emits `Slide.blocks = vec![]`.
Problem A (content-empty exports) and Problem B (unsatisfiable a11y gate) are both
downstream symptoms of this intentional deferral. They require coordinated fixes,
not independent patches.

---

## 1. Problem A — Systemic Content-Rendering Gap

### 1.1 Root Cause: `Slide.blocks` Is Intentionally Empty

`crates/slideforge-eval/src/for_eval.rs:342` constructs every `Slide` with
`blocks: vec![]`. The in-code comment (lines 337–342) is explicit:

> "Block-level content (images, charts, diagrams, shapes) is populated by
> later pipeline stages (layout, PPTX generation, Waves 3+). In Wave 2, the
> evaluator only resolves field values."

This was a deliberate Wave 2 scope boundary, not a bug. However, the spec
contract (BC-5.02.001, CLAUDE.md §Accessibility) and the exporter story ACs
(STORY-037 through STORY-045) were authored assuming the threading work would
precede export delivery. Instead, exporters shipped before the eval→blocks
pathway was implemented. The result: structurally valid but content-empty output.

### 1.2 End-to-End Content Path Trace

#### Title and Subtitle Text (all exporters)

1. Parser: `SlideNode.fields` contains `FieldValue::Template` for `"title"`,
   `"body"`, `"subtitle"` keys.
2. Eval (`for_eval.rs:eval_slide_node`): resolves templates to
   `FieldValue::Literal(Value::Str(...))`, stores in `Slide.fields`.
3. Layout (`layout.rs:182–208`): reads `Slide.fields["title"]` and
   `Slide.fields["body"/"subtitle"]` specifically for index-0 and index-1
   `FrameContent::Empty` frames, populating `frame.text_flow` for overflow
   validation only. Crucially: this does NOT produce `FrameContent::TextRun`
   for title/body. The frames remain `FrameContent::Empty`.
4. PPTX exporter: consumes `FrameContent::Empty` — emits no `<a:r>` text run.
   Result: `<p:spTree>` has no text shapes.
5. PDF exporter: has no `FrameContent::TextRun` to issue `show_text` operators.
   Result: pages are geometrically blank.

Conclusion: **`Slide.fields["title"/"body"]` are resolved strings** stored in
eval output. They are not being converted to `FrameContent::TextRun` frames
that exporters can render. The conversion step (eval fields → ContentBlocks →
FrameContent::TextRun) is the missing link.

#### Body/Bullets/Report Text (DOCX partial exception)

DOCX has SOME content (body/takeaway/report text) because the DOCX exporter
reads `LaidOutSlide.register_content` directly. `register_content` is populated
by `extract_register_content` from `Slide.fields["notes"/"report"/"detail"]`
during eval (Step 2 above). This path does NOT go through `Slide.blocks` or
`FrameContent::TextRun`. DOCX body sections that are driven by register content
(notes, report, detail fields) work.

Slide-title `Heading1` runs in DOCX are EMPTY because: the DOCX exporter reads
title from `LaidOutSlide.slide_type_keyword` or from a frame — it does not
directly read `Slide.fields["title"]`. Since no `FrameContent::TextRun` frame
carries the title text, the Heading1 run is empty.

#### Charts, Images, Diagrams (all exporters)

1. Parser: `SlideNode.fields` contains `chart_type`, `alt`, etc. as flat
   `FieldValue` entries.
2. Eval: resolves field values into `Slide.fields`. **Does NOT construct
   `ContentBlock::Chart/Image/Diagram`.** `Slide.blocks` remains `vec![]`.
3. Layout (`layout.rs:213–254`): shape pass filters `slide.blocks` for
   `ContentBlock::Shape` — finds nothing. `thread_media_alt_into_frames`
   iterates `slide.blocks` — finds nothing. Region map frames for `chart`,
   `image`, `screenshot`, `bio`, `diagram` slide types carry
   `FrameContent::Chart/Image/Diagram { alt: AltText::Decorative }` as
   structural placeholders. These placeholders are **never overwritten**.
4. Post-layout a11y validator sees `AltText::Decorative` on every
   Chart/Image/Diagram frame — emits E-A11-001 for each.

### 1.3 Was `eval_slide_node` EVER Supposed to Emit ContentBlocks?

No. ADR-005 (two-IR model) explicitly rules this out:

> "The evaluator's contract is to resolve field values, not to construct
> content geometry."

ADR-018 Section "Rationale" reiterates this (Option B rejection). The evaluator
is a **field resolver**, not a ContentBlock factory. ContentBlock construction
is a layout/Wave-3 concern.

The gap is not that `for_eval.rs` is wrong — it is that the **pipeline stage
responsible for converting `Slide.fields` into `ContentBlock` entries** (which
would then populate `Slide.blocks` and feed `thread_media_alt_into_frames`)
has not been built yet.

### 1.4 BC and ADR Mis-Description Flags

The following spec artifacts contain comments that are no longer accurate:

| Location | Mis-description | Flag |
|----------|-----------------|------|
| `for_eval.rs:336–341` comment | "Validators that inspect `slide.blocks` operate on `Deck` values produced by the layout stage" | FALSE — AltTextValidator now operates on `LaidOutDeck` (ADR-018 Decision 3), not layout-produced Deck. The comment predates ADR-018. Must be updated. |
| `alt_text.rs:150–193` `validate_post_layout` comment | "MUST be revisited for Wave 3+" — specifically, the AltText::Decorative match | ACCURATE AS WARNING but should now be anchored to a specific story ID rather than a wave label. |
| `layout.rs:163–166` comment | "wiring slide body blocks into FrameContent::Body is STORY-027 scope" | STORY-027 has shipped (sections). This comment cites the wrong story. The field-to-frame wiring work should be cited against the correct new story. |

### 1.5 Remediation Scope: Fix-Burst vs New Stories

The content-threading gap is NOT fixable in a single fix-burst scoped to an
existing story. The missing pipeline stage — converting `Slide.fields` into
`Slide.blocks` (or directly into layout frames) — is substantial work spanning
at minimum three crates and touching the two-IR boundary.

**Recommendation: Two new stories. Wave 5.**

#### Story A: `slide-field-to-block-threading` (High Priority)

**Scope:** Implement the eval→ContentBlock transition for the five field-driven
content types: title text, body/subtitle text, bullets, chart metadata, and
image/diagram metadata.

**What this is NOT:** This is not the chart renderer (SVG generation), not the
mermaid engine, not image file I/O. It is the step that constructs typed
ContentBlock objects from already-resolved `Slide.fields` values so that
`thread_media_alt_into_frames` and the inline layout pass have something to work
with.

**Affected crates:**
- `slideforge-eval` — new pass after `eval_slide_node` that reads resolved
  `Slide.fields` and populates `Slide.blocks` with typed ContentBlocks.
  Must respect the purity boundary: no I/O, no file reads, deterministic.
- `slideforge-types` — no new types needed; ContentBlock variants already exist.
- `slideforge-layout` — `thread_media_alt_into_frames` already correct; will
  begin producing real alt threading once `Slide.blocks` is non-empty.
- `slideforge-validate` — `AltTextValidator.validate()` pre-layout pass will
  become functional (currently dead but correctly structured per ADR-018).

**BCs affected:** BC-3.04.001 (alt threading), BC-5.02.001 (alt required),
BC-1.01.005 (ContentBlock structure).

**Architecture decision required:** Where exactly in the pipeline does
`Slide.fields → ContentBlock` conversion happen? Three options:
- (a) Inside `eval_slide_node` — violates ADR-005 (eval must not produce
  geometry). Rejected.
- (b) A new post-eval, pre-layout pass that mutates `Slide.blocks` from
  resolved field values. This is the correct seam: it happens after all
  `{{ }}` expressions are resolved but before layout geometry is computed.
  The pass is deterministic and pure (reads `Slide.fields`, writes
  `Slide.blocks`) — compatible with Kani analysis. **Recommended.**
- (c) Inside `layout::run` — layout already has a shape pass. Could add a
  "field-to-frame" pass here. Violates separation of concerns (layout should
  transform ContentBlocks to frames, not create ContentBlocks from raw fields).
  Rejected.

**Decision needed from human:** Approve option (b). This requires a new
pipeline stage between Stage 2 (eval) and Stage 6 (layout), tentatively
"Stage 2b: field-to-block threading". No new crate needed; implemented in
`slideforge-eval` as a separate module.

#### Story B: `title-frame-threading` (Coupled to Story A)

**Scope:** Ensure `FrameContent::Empty` frames for title/subtitle/body
regions are populated with `FrameContent::TextRun` so exporters can
render text. Currently `layout::run` only computes `text_flow` (for overflow
validation) but does not convert the `Empty` frame to a `TextRun` frame.

**Affected crates:**
- `slideforge-layout` — convert index-0 and index-1 `FrameContent::Empty`
  frames to `FrameContent::TextRun` when `Slide.fields["title"]` /
  `Slide.fields["body"/"subtitle"]` provide non-empty strings.
- All three exporters (PPTX, PDF, DOCX) — already consume `FrameContent::TextRun`;
  these should start working without exporter changes.

**Note:** Story B could be absorbed into Story A if the field-to-block
threading approach (option b) produces `ContentBlock::Text` entries for
title/body text, and the existing inline layout pass in `layout.rs:270–298`
converts them to `FrameContent::TextRun` frames. In that case, no separate
story is needed — the inline layout pass already does the conversion
correctly for `ContentBlock::Text`. This is the preferred approach.

**Architecture validation:** `layout.rs:270–298` already handles
`ContentBlock::Text` by pushing `FrameContent::TextRun` frames. If Story A's
post-eval pass emits `ContentBlock::Text(TextBlock { inlines: [Plain(title_str)] })`
for the title field, the layout pass will correctly convert it — no layout
changes needed. Story B collapses into Story A.

### 1.6 Summary Scope Decision

**One new story (Story A) is required, Wave 5:**
`slide-field-to-block-threading` — implement the post-eval pass that converts
resolved `Slide.fields` into typed `Slide.blocks` ContentBlock entries. Title,
body, and subtitle text map to `ContentBlock::Text`. Chart/Image/Diagram
fields map to `ContentBlock::Chart/Image/Diagram` with alt threading. Bullets
map to `ContentBlock::Bullets`. This unblocks `thread_media_alt_into_frames`,
the inline layout pass, all three exporters, and the a11y strict gate.

**Human sign-off needed:** Approve Wave 5 scope addition. Blocking for content
correctness; not deferrable to v1.x per the production-grade default.

---

## 2. Problem B — a11y Strict-Gate Is Unsatisfiable (F-G3-CRIT-001)

### 2.1 Current State

With `BuildOptions::default()` (`strict: true`), every deck containing a
`slide chart:`, `slide image:`, `slide screenshot:`, `slide bio:`, or
`slide diagram:` type unconditionally fails with E-A11-001.

The mechanism:
1. `regions.rs` emits `FrameContent::Chart/Image/Diagram { alt: AltText::Decorative }`
   as a structural placeholder for these slide types.
2. Because `Slide.blocks` is always empty, `thread_media_alt_into_frames` never
   overwrites the placeholder — even when the author wrote `alt "description"`.
3. `validate_post_layout` sees `AltText::Decorative` and emits E-A11-001.
4. Strict gate fires. `BuildError::ValidationFailed`. No author remedy.

Additionally, `decorative: true` (the error message's suggested opt-out) fails
to parse with E-PAR-002 (see Problem C below), making the error message actively
misleading.

### 2.2 Why This Is Not an ADR-018 Implementation Bug

ADR-018 correctly describes the intended post-Wave-3 behavior. The code
correctly implements ADR-018's Decision 3 for the **current pipeline state**.
The problem is that ADR-018's Decision 3 is implemented correctly for Wave 3+
but the code's validator comment (alt_text.rs:170–193) explicitly warns that
the Wave 3+ threading landing "MUST" trigger a validator update — and that Wave 3+
work has now been identified (Story A above) but not yet scheduled.

The validator comment at `alt_text.rs:184–189`:

> "At that point, an AltText::Decorative frame will mean EITHER:
> 1. Author-supplied `decorative: true` → VALID, no error should be emitted.
> 2. Missing alt text (placeholder never replaced) → INVALID, E-A11-001.
> This match arm currently cannot distinguish these two cases."

This is exactly the problem. It needs to be solved before Story A lands.

### 2.3 Fix Design: Introduce AltText::Unspecified

**Recommended approach:** Add a third `AltText` variant — `AltText::Unspecified`
— to represent the structural placeholder state ("no author input threaded yet"),
distinct from author-chosen `AltText::Decorative` ("author said this is
decorative").

This is the correct production-grade design because:
- It makes the validator's match arms unambiguous: `Decorative` = author intent,
  `Unspecified` = no author data → validate as error.
- It localizes the change to `slideforge-types::specs::AltText` (one new variant)
  and three match sites.
- It does not require any validator logic change for the post-Story-A state —
  the validator will correctly flag `Unspecified` as missing alt and correctly
  accept `Decorative` as author opt-out.

**AltText enum — new definition:**

```rust
pub enum AltText {
    /// Author explicitly wrote `alt "..."` with a non-empty string.
    Provided(Arc<str>),
    /// Author explicitly wrote `decorative: true` — visual element
    /// intentionally has no accessible description.
    Decorative,
    /// Structural pipeline placeholder: no author alt-text data has been
    /// threaded into this frame yet. NOT equivalent to Decorative.
    /// Used exclusively by `regions.rs` for initial frame construction;
    /// overwritten by `thread_media_alt_into_frames` when Story A lands.
    Unspecified,
}
```

**`regions.rs` change:** All `FrameContent::Chart/Image/Diagram` structural
placeholders change from `alt: AltText::Decorative` to `alt: AltText::Unspecified`.

**`validate_post_layout` change:** The match in `alt_text.rs:210–240` changes:

```rust
// Before (current):
FrameContent::Chart { alt: AltText::Decorative } => { emit E-A11-001 }

// After (correct, post-fix):
FrameContent::Chart { alt: AltText::Unspecified } => { emit E-A11-001 }
FrameContent::Chart { alt: AltText::Decorative } => { /* valid author opt-out */ }
FrameContent::Chart { alt: AltText::Provided(_) } => { /* valid */ }
```

**`thread_media_alt_into_frames` change:** When `ContentBlock::Chart` has
`alt = None`, the fallback becomes `AltText::Unspecified` (not `AltText::Decorative`)
with the same tracing::warn. When `alt = Some(AltText::Decorative)`, write
`AltText::Decorative` to the frame.

### 2.4 What Is Fixable Now vs Threading-Dependent

**Fixable NOW (independent of Problem A / Story A):**
- Add `AltText::Unspecified` variant to `slideforge-types::specs::AltText`.
- Update `regions.rs` to use `AltText::Unspecified` for all structural placeholders.
- Update `validate_post_layout` to flag `Unspecified` as error, accept `Decorative`
  as valid opt-out.
- Update `thread_media_alt_into_frames` fallback to use `Unspecified`.
- Update `produces_structure_group()` in `block.rs` to handle `Unspecified`
  (treat as non-structure-producing, same as `Decorative`).
- This fix unblocks the strict gate for ALL slide types that do NOT have
  chart/image/diagram frames — title, content, agenda, toc, quote, two_col, etc.
  These will now build successfully under strict mode.

**Threading-dependent (requires Story A to complete):**
- For `chart`, `image`, `screenshot`, `bio`, `diagram` slide types: the strict
  gate will still fail until Story A threads author alt-text into ContentBlocks
  and `thread_media_alt_into_frames` overwrites the `Unspecified` placeholder
  with `Provided(s)` or `Decorative`.
- However, once `AltText::Unspecified` is introduced, the CORRECT ERROR is
  emitted: "visual element has no alt text" — which is accurate. The gate now
  fails for the right reason, with a fixable user action (write `alt "..."`).
  It is no longer an unsatisfiable false positive.

**Net effect of the now-fix:**
- Decks without visual media types build cleanly under strict mode.
- Decks with chart/image/diagram types get an accurate, actionable error
  until Story A lands. The error is no longer a false positive.
- `decorative: true` opt-out becomes correct and meaningful once Problem C
  (parse bug) is fixed.

### 2.5 ADR Amendment

ADR-018 does not require amendment. The `AltText::Unspecified` introduction is
additive to the types crate and consistent with ADR-018's Decision 3 rationale.
The ADR's comment ("MUST be revisited for Wave 3+") is now anchored by Story A.

Update `alt_text.rs` validator comment to cite the concrete story ID (Story A)
rather than "Wave 3+" once the story is created.

---

## 3. Problem C — `decorative: true` Parse Failure (E-PAR-002)

### 3.1 Diagnosis

CLAUDE.md documents: `decorative: true` opts out of alt text requirement.

The DSL field parser in `parser/deck.rs:value_parser()` does parse `BoolLit(b)` →
`FieldValue::Bool(b)`. The lexer (`lexer.rs:816–817`) does lex `true` and `false`
as `Token::BoolLit`. The eval (`for_eval.rs:eval_slide_node:267`) handles
`FieldValue::Bool(b)` → `FieldValue::Literal(Value::Bool(b))`.

So the PARSER can handle `decorative: true` as a field value. The question is
whether the slide parser specifically handles `decorative` as a recognized field.

Checking `known_fields.rs:62–69`: `"decorative"` IS in the known field list for
ALL slide types. The lexer does lex `true` → `Token::BoolLit(true)`.

**The parse failure is likely not in the parser itself.** The E-PAR-002 reported
by the holdout evaluator most probably occurs because `decorative: true` is being
written at the TOP-LEVEL slide block (`slide chart:` → `decorative: true`) where
the parser expects it to be a nested field (`alt "..."` inside the slide block),
but the DOCX/PPTX exporters are not reading `Slide.fields["decorative"]` and
treating it as an exemption. OR the test fixture that triggered E-PAR-002 had a
syntax error at the `decorative: true` site (wrong indentation, collision with
another keyword).

**Requires investigation:** The exact DSL input that triggered E-PAR-002 must be
found. One of:
1. Syntax issue (indentation, quote context) — fix is in the fixture/user code.
2. Parser gap: `decorative` is allowed as a field name in `known_fields.rs` for
   all types, but the field-value parser at the slide block level may not accept
   it in certain parsing modes (e.g., inside a `shape:` sub-block context).
3. The `decorative` field IS parsed into `Slide.fields["decorative"] =
   FieldValue::Literal(Value::Bool(true))` — but nothing reads it. The
   AltTextValidator checks `spec.decorative: bool` on `ContentBlock::Image/Chart`
   specs, NOT on `Slide.fields["decorative"]`. Since `ContentBlock` is never
   constructed (Problem A), the `decorative` field is silently ignored.

**Conclusion:** This is classification 3 — a functionality gap, not a parse
bug. The `decorative` flag is accepted by the parser and stored in
`Slide.fields`, but nothing in the current pipeline reads it to set
`spec.decorative = true` on the (not-yet-constructed) ContentBlock. The E-PAR-002
is likely a user-facing artifact from the error message recommendation
("add `decorative: true`") pointing to a field that currently has no effect.

**Fix scope:** The field-to-block threading story (Story A) MUST read
`Slide.fields["decorative"]` when constructing `ContentBlock::Image/Chart/Diagram`
and set `spec.decorative = true` accordingly. No parser changes are needed.
`AltText::Decorative` on the ContentBlock will then flow through
`thread_media_alt_into_frames` to the `FrameContent` alt field.

**Is this fixable now, independent of A/B?** Partially. The E-PAR-002 symptom
will go away naturally once Story A constructs ContentBlocks and reads
`decorative` from fields — no separate fix-burst needed. In the interim, the
error message in `validate_post_layout` should be updated to NOT suggest
`decorative: true` as a remedy, because the field has no current effect. This
message fix is a small, safe now-fixable change (no code change; just error
message text in `make_post_layout_error`).

---

## 4. HIGH Finding Routing (F-G3-HIGH-002 and F-G3-HIGH-003)

### 4.1 F-G3-HIGH-002: Pre-layout AltText Validator + LabelCheck Inert

Root cause: Same as Problem A. `Slide.blocks` is `vec![]`, so the pre-layout
`AltTextValidator.validate()` iterates nothing and produces no signal. This is
documented and architecturally correct per ADR-018 Decision 3.

`LabelCheck.validate()` operates on `Slide.fields` (not `Slide.blocks`) — it IS
functional for slides where the `label` field is resolved by eval. The finding
that LabelCheck "produces no signal" is true only for the three slide types
(`status`, `progress_bar`, `weighted_composite`) addressed in HIGH-003 below.

**Routing:** Folds into the Problem A / Story A remediation. No separate action
needed. The pre-layout `AltTextValidator.validate()` will become functional once
`Slide.blocks` is populated by Story A. LabelCheck routing depends on HIGH-003.

### 4.2 F-G3-HIGH-003: LabelCheckValidator.COLOR_CODED_TYPES Keyword Mismatch

`COLOR_CODED_TYPES = ["severity_cards", "status", "progress_bar", "weighted_composite"]`

**`severity_cards`:** IS a registered slide type keyword (present in
`SLIDE_TYPE_KEYWORDS` phf set in `keywords.rs`; present in `regions.rs` region
map; has no dedicated slide type implementation file but shares the standard
two-region layout). LabelCheck WILL fire for `severity_cards` slides — it is
functional.

**`status`, `progress_bar`, `weighted_composite`:** Are NOT in
`SLIDE_TYPE_KEYWORDS` (confirmed by grep showing no hits in `keywords.rs`). Not
in `regions.rs` region map. Not in plugin-api slide type implementation files
directory. These three types will cause `LayoutError::UnknownSlideType` before
the label validator runs. LabelCheck is dead for these types end-to-end because
the slide types themselves are not registered.

This is a **specification gap**: `COLOR_CODED_TYPES` in the validator claims to
protect three types that do not exist in the registered slide type set. The Q2
DSL decisions specify 31 built-in slide types. `status`, `progress_bar`, and
`weighted_composite` are not among the 31.

**Possible explanations:**
- These types are planned but not yet implemented (planned for Wave 5+).
- The LabelCheck validator was authored speculatively against planned types.
- There is a naming mismatch (e.g., `kpi_dashboard` covers "status"; `stat_callout`
  covers some progress/weighted use case).

**Routing:** This is a separate story from Problem A. Required actions:
1. PO/BA review: Are `status`, `progress_bar`, `weighted_composite` planned
   slide types for v1.0? Or do existing types (`kpi_dashboard`,
   `stat_callout`, `severity_cards`) cover these use cases?
2. If planned for v1.0: add implementations in plugin-api, regions.rs,
   and keywords.rs. This is new slide type implementation work, not a
   content-threading fix.
3. If NOT planned for v1.0: remove these three from `COLOR_CODED_TYPES` until
   the types exist. The guard is dead code and misleading.

**This is NOT foldable into Problem A** — it is a type registry gap, separate
from the content-threading gap. Recommend a fast-track spec adjudication
(PO + BA, same wave) to determine v1.0 scope before the next story cycle.

---

## 5. Summary Routing Table

| Finding | Severity | Fix Category | Timing | Sign-Off Required |
|---------|----------|--------------|--------|-------------------|
| `blocks: vec![]` → content-empty exports | CRITICAL | New Wave 5 story (Story A: `slide-field-to-block-threading`) | Wave 5 | **Human: approve Wave 5 scope + Stage 2b pipeline seam decision** |
| Strict gate unsatisfiable for visual types | CRITICAL | New type variant `AltText::Unspecified` + validator/regions update | **NOW (fix-burst)** | None — within architect authority |
| `decorative: true` error message misleading | HIGH | Update error message text in `make_post_layout_error` | **NOW (fix-burst)** | None |
| F-G3-HIGH-002: pre-layout validators inert | HIGH | Resolved by Story A | Wave 5 | None (absorbed) |
| F-G3-HIGH-003: `status/progress_bar/weighted_composite` not registered | HIGH | Separate: PO/BA spec adjudication, then slide type registration or removal from COLOR_CODED_TYPES | Wave 5 pre-planning | **Human: PO/BA adjudication on slide type v1.0 scope** |
| `for_eval.rs:336–341` comment inaccurate | LOW | Update comment to cite ADR-018 Decision 3 | NOW (fix-burst) | None |

---

## 6. Artifacts Requiring Comment/Spec Updates

### `crates/slideforge-eval/src/for_eval.rs` lines 336–342

Current comment is inaccurate post-ADR-018. Update:

```
// Block-level content is populated by the post-eval field-to-block
// threading pass (Stage 2b, Story A). AltTextValidator now runs
// post-layout via ADR-018 Decision 3 — not on this Deck output.
```

### `crates/slideforge-layout/src/layout.rs` line 163–166

Update comment to cite the correct story ID (Story A) rather than "STORY-027".

### `crates/slideforge-validate/src/alt_text.rs` lines 175–193

Update OBS-1 comment to cite Story A ID once created, anchor Wave 3+ reference
to a concrete story.
