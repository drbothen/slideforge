---
document_type: architect-adjudication
story: STORY-086
pass: 5
date: 2026-06-06
findings_addressed: [F-086-P5-CRIT-001, F-086-P5-MED-002]
verdict: decorative-first
resolution_type: code-conforms
human_auth_required: false
---

# Architect Adjudication — STORY-086 Pass 5

## Conflict Summary

Two behavioral contracts prescribe opposite precedence rules for the case where a
chart/image/diagram slide has BOTH `decorative: true` AND a non-empty `alt "..."`:

**BC-1.16.001 PC-12 / EC-004 (Stage 2b threading pass — this story):**
Decorative-first. When `decorative == true`, `alt = Some(AltText::Decorative)`
regardless of any supplied alt string. A conflict warning W-A11-002 is emitted.

**BC-3.04.001 Invariant 11 (shape DSL — older BC, v1.5.0+):**
Alt-first. Explicit alt text supersedes implicit-decorative inference. When both
are supplied, `AltText::Provided(s)` wins; W-A11-002 is still emitted.

The STORY-086 implementation in `resolve_alt` follows BC-3.04.001 Invariant 11
(alt-first) and cites it in a code comment, producing `AltText::Provided("desc")`
when both fields are set. This contradicts BC-1.16.001 PC-12, which the story's
own PRIMARY anchor BC mandates as decorative-first.

ADR-019 Decision 4.1 (the AltText state machine) also encodes alt-first for the
conflict case, citing BC-3.04.001 Invariant 11. No test covers the both-set case,
so the divergence has been invisible.

---

## Verdict: DECORATIVE-FIRST is Canonical

### Precedence Analysis (CLAUDE.md Source-of-Truth Rules)

Under CLAUDE.md Source-of-Truth Precedence, the relevant rules are:

- Rule 1: Story spec supersedes BC it traces to for implementation scope; BC
  supersedes for contract semantics.
- Rule 2: ADR supersedes earlier ADRs on the same decision.
- Rule 7 (Standing Rule): For code-vs-spec conflicts, the SPEC wins.

BC-1.16.001 is the PRIMARY anchor BC for STORY-086. It defines the Stage 2b
threading pass contract — the exact operation that `resolve_alt` implements.
BC-3.04.001 is the shape DSL contract, covering a completely different domain:
the `shape:` block syntax for custom positioned shapes. Its Invariant 11 was
written for ShapeSpec/ShapeFrame resolution, not for the media alt-resolution
path in the threading pass.

The two BCs cover non-overlapping implementation domains:
- BC-1.16.001 governs `thread_fields_to_blocks` / `resolve_alt` in
  `slideforge-eval/src/field_to_block.rs` (Stage 2b, semantic IR threading).
- BC-3.04.001 governs `layout_shapes` / shape block construction in
  `slideforge-layout` (layout stage, geometric IR).

BC-3.04.001 Invariant 11 was never intended to govern the Stage 2b media
alt-resolution path. Its scope is explicitly "shape: block declares custom shape
with type/position/fill/text/alt" — the title of the BC itself. Extending its
precedence rule to cover ChartSpec/ImageSpec/DiagramSpec construction during Stage 2b
is an overreach that the implementer made by reaching for the nearest precedent
without checking domain scope.

BC-1.16.001 is:
- More specific (Stage 2b threading pass, this exact function)
- More recent (introduced for STORY-086; BC-3.04.001 Invariant 11 written for STORY-028)
- The authoritative contract for the operation in question

**Therefore: BC-1.16.001 PC-12 / EC-004 wins. Decorative-first is canonical for
`resolve_alt`.**

### WCAG and a11y Semantic Justification

Both BC-1.16.001 (decorative-first) and BC-3.04.001 Invariant 11 (alt-first) claim
WCAG support. The question is what an author who writes BOTH `decorative: true` AND
`alt "..."` actually intends.

For shapes (BC-3.04.001 domain): An author who writes `shape: type rect alt "Blue
rectangle" decorative: true` has provided a typed, deliberate alt string embedded in
structural DSL syntax. The `alt "..."` is a first-class DSL field requiring explicit
authoring. It is reasonable to honor the alt, since authoring it alongside decorative
is an annotation conflict the author made consciously.

For charts/images/diagrams (BC-1.16.001 domain): The `decorative: true` field on a
chart or image is a stronger, more intentional signal. An author who marks a chart
`decorative: true` is asserting the chart conveys no meaning — it is a background
decoration. If they ALSO wrote `alt "desc"`, the most likely cause is a template or
copy-paste error. Preserving the alt text on a "decorative" chart risks creating an
accessibility contradiction: a screen reader would announce the chart description
despite the author's stated intent to suppress it. The `decorative: true` field on
a media block is architecturally the more specific accessibility opt-out.

More critically: BC-1.16.001 EC-004 explicitly documents this conflict case and
mandates decorative-first with a warning. ADR-019 Decision 4.1's AltText state
machine was authored alongside BC-1.16.001 and should have matched it — the
Decision 4.1 conflict path encoding "alt takes precedence per BC-3.04.001 Invariant
11" is itself an error in ADR-019 (it imported the wrong rule from the wrong domain).

**Decorative-first with W-A11-002 warning is the correct policy for the Stage 2b
media alt-resolution path.**

### Resolution Type: CODE-CONFORMS

The code must be fixed to match BC-1.16.001. No human authorization is required.

BC-3.04.001 Invariant 11 is NOT being changed in its semantics for shapes; it is
being scoped to its proper domain (shape blocks only). A reconciling clarification
will be added to its prose. ADR-019 Decision 4.1 conflict path will be corrected.

**HUMAN AUTHORIZATION REQUIRED: NO.**

---

## F-086-P5-MED-002: W-A11-002 Relocation Decision

### The Problem

EC-004 (BC-1.16.001 and STORY-086) states that the decorative+alt conflict warning
W-A11-002 fires "pre-layout" via `AltTextValidator::validate()`. But ADR-018 v1.2
Decision 3 restricts `AltTextValidator.validate()` pre-layout to
`ContentBlock::Shape` ONLY. Charts/images/diagrams are validated post-layout only.
This makes the EC-004 stated mechanism unreachable as written.

### Decision: Emit W-A11-002 at Stage 2b in resolve_alt

The warning should fire inside `resolve_alt` itself (or the immediate call site),
at the moment the conflict is detected — not in the validator at all. This is
architecturally correct for three reasons:

1. **It is not a validation finding; it is an authoring lint.** W-A11-002 is not
   checking for an error condition (both fields being set does not prevent correct
   output). It is alerting the author to a redundant/contradictory field
   combination so they can clean it up. This is logging-layer work, not
   validator-layer work. The correct mechanism is `tracing::warn!` inside
   `resolve_alt` when both fields are detected, before returning
   `Some(AltText::Decorative)`.

2. **It avoids a double-classification problem.** Emitting W-A11-002 post-layout
   (in `validate_post_layout`) would require `AltTextValidator` to re-derive
   whether the `alt = AltText::Decorative` result came from an "only decorative"
   case or a "decorative wins over alt" case. That information is not preserved in
   the frame. Re-deriving it post-layout would require propagating an additional
   conflict flag through the IR, which is unnecessary complexity.

3. **It is consistent with what BC-3.04.001 Invariant 11 actually does for shapes.**
   For shapes, W-A11-002 is emitted by `slideforge-validate` at the layout stage
   when the alt-text validator sees a shape with both fields set. But for Stage 2b
   media threading, the threading pass is the natural site — it is where the
   conflict is detected and the resolution decision is made.

### Chosen Location: `resolve_alt` via `tracing::warn!`

```rust
fn resolve_alt(slide: &slideforge_types::Slide) -> Option<AltText> {
    let has_decorative = is_decorative(slide);
    let alt_str = extract_str_field(slide, "alt");

    if has_decorative {
        if let Some(s) = alt_str {
            if !s.trim().is_empty() {
                // W-A11-002: both decorative: true and alt "..." supplied.
                // decorative takes precedence in Stage 2b media threading.
                // The alt string is discarded. Author should remove one field.
                tracing::warn!(
                    code = "W-A11-002",
                    alt = s,
                    "slide has both alt and decorative: true; \
                     decorative takes precedence for chart/image/diagram in Stage 2b. \
                     Consider removing the alt field or removing decorative: true."
                );
            }
        }
        Some(AltText::Decorative)
    } else if let Some(s) = alt_str {
        if !s.trim().is_empty() {
            Some(AltText::Provided(Arc::from(s)))
        } else {
            None
        }
    } else {
        None
    }
}
```

The warning is emitted via `tracing::warn!` with `code = "W-A11-002"` structured
field so structured log consumers can filter on it. It does not become a
`Diagnostic` in the validator diagnostic list — it is not subject to strict-mode
gating and does not affect exit code. The existing W-A11-002 description in
BC-3.04.001 ("shape has both alt and decorative: true; alt takes precedence") needs
a parallel note for the media threading case.

**Drop option (c) and separate post-layout option (b) are rejected.** Dropping the
warning removes useful authoring feedback. Post-layout emission requires IR
enrichment that adds complexity without benefit.

---

## Change Plan

### Item 1: Fix `resolve_alt` in the STORY-086 worktree

**File:** `crates/slideforge-eval/src/field_to_block.rs` (lines 267-283 in worktree HEAD 2c677d97)

**Owner:** Implementer (STORY-086 in-scope fix)

**Change:** Rewrite `resolve_alt` to decorative-first precedence:
- Check `has_decorative` first. If true AND alt is also non-empty, emit
  `tracing::warn!` with `code = "W-A11-002"` and return `Some(AltText::Decorative)`.
- If `has_decorative` but no alt, return `Some(AltText::Decorative)` silently.
- Else if non-empty alt, return `Some(AltText::Provided(Arc::from(s)))`.
- Else return `None`.

Update the doc comment on `resolve_alt` to cite BC-1.16.001 PC-12 (not
BC-3.04.001 Invariant 11) as the precedence authority for this function.

Remove the comment "alt takes precedence over decorative (BC-3.04.001 Invariant 11)"
and replace with "decorative takes precedence in Stage 2b media threading
(BC-1.16.001 PC-12); W-A11-002 emitted when both are set."

### Item 2: Add a load-bearing unit test for the both-set case

**File:** `crates/slideforge-eval/src/field_to_block.rs` (in `#[cfg(test)] mod tests`)

**Owner:** Implementer (STORY-086 in-scope fix)

**Change:** Add a test (e.g., `test_chart_both_decorative_and_alt_decorative_wins`) that:
- Constructs a deck with a chart slide having both `fields["decorative"] =
  Value::Bool(true)` and `fields["alt"] = Value::Str("desc")`.
- Calls `thread_fields_to_blocks(&mut deck)`.
- Asserts `spec.alt == Some(AltText::Decorative)` and `spec.decorative == true`.
- Does NOT assert that `alt = Some(AltText::Provided("desc"))` (this assertion
  would have passed under the wrong alt-first implementation; its absence here
  confirms the correct decorative-first behavior).

This test is the load-bearing check required by TD-VSDD-059. It must be a concrete
assertion on `AltText::Decorative`, not a doc-comment or rename.

Additionally, add a tracing capture assertion (or use `tracing_test` crate if
available) to confirm W-A11-002 is emitted when both fields are set. If
`tracing_test` is not in scope, at minimum add a comment citing the `tracing::warn!`
call with its `code = "W-A11-002"` field as the emission point.

### Item 3: Correct ADR-019 Decision 4.1 conflict path

**File:** `.factory/specs/architecture/adr/ADR-019-stage-2b-field-to-block-threading.md`

**Owner:** Architect (spec amendment, in-scope for this adjudication)

**Change:** In the AltText State Machine (Decision 4.1), the conflict path currently reads:

```
Author writes both `alt "..."` AND `decorative: true` (conflict):
  → Stage 2b: `alt` takes precedence per BC-3.04.001 Invariant 11.
    ContentBlock.alt = Some(AltText::Provided(s)), decorative = true.
  → validate pre-layout: emits W-A11-002 ("alt takes precedence over decorative").
  → frame.alt = AltText::Provided(s) — correct alt propagated.
```

This must be corrected to:

```
Author writes both `alt "..."` AND `decorative: true` (conflict):
  → Stage 2b: `decorative` takes precedence per BC-1.16.001 PC-12.
    ContentBlock.alt = Some(AltText::Decorative), decorative = true.
    resolve_alt emits tracing::warn!(code = "W-A11-002") at detection time.
  → thread_media_alt_into_frames: frame.alt = AltText::Decorative
  → validate_post_layout: Decorative → VALID (author opt-out; warning already emitted).
```

Note on scope: ADR-019 Decision 4.1's previous text was consistent with BC-3.04.001
Invariant 11 (alt-first) but was written for the Stage 2b media domain where
BC-1.16.001 is the authoritative contract. The correction aligns ADR-019 with the
correct primary BC for this domain.

**Also update:** The doc comment on `thread_fields_to_blocks` in ADR-019
(Section 3.3 / 3.4 / 3.5 alt-resolution sub-rules) must be updated from the current
"decorative → Decorative, else alt → Provided" bullet list (which already shows
decorative-first in the primary path but has no mention of the both-set conflict
case) to include: "If both decorative and non-empty alt: decorative wins; W-A11-002
emitted via tracing::warn!."

### Item 4: Reconcile BC-3.04.001 Invariant 11 domain scope

**File:** `.factory/specs/behavioral-contracts/BC-3.04.001.md`

**Owner:** Product Owner (BC amendment)

**Change:** Add a domain-scoping sentence to Invariant 11 clarifying it applies to
the shape DSL path only and does NOT govern Stage 2b media alt-resolution. The
existing Invariant 11 text and semantics for shapes are CORRECT and UNCHANGED.
Only the scope clarification is new.

Proposed addition at the end of Invariant 11 (after the existing WCAG/F-P20 citations):

> **Domain scope:** This invariant applies exclusively to the `shape:` block DSL
> path (ShapeSpec → ShapeFrame resolution in `layout_shapes`). For chart/image/diagram
> media elements, alt-resolution precedence is governed by BC-1.16.001 PC-12
> (decorative-first in Stage 2b threading), which applies the opposite precedence
> for architecturally justified reasons (see architect adjudication STORY-086
> pass 5, 2026-06-06). The two rules coexist without contradiction because they
> govern non-overlapping IR construction paths.

No changes to EC-018, the existing canonical test vectors for shapes, or any other
part of BC-3.04.001.

### Item 5: Correct BC-1.16.001 EC-004 warning emission mechanism

**File:** `.factory/specs/behavioral-contracts/BC-1.16.001.md`

**Owner:** Product Owner (BC amendment)

**Change:** EC-004 currently reads:

> `alt = Some(AltText::Decorative)`, `decorative = true`. Decorative takes
> precedence in Stage 2b. Note: ADR-019 Decision 4.1 documents this conflict;
> the pre-layout validator emits W-A11-002.

The phrase "the pre-layout validator emits W-A11-002" is inaccurate. The warning is
emitted by `resolve_alt` via `tracing::warn!`, NOT by `AltTextValidator::validate()`
(which is restricted to Shape blocks only per ADR-018 v1.2).

Correct to:

> `alt = Some(AltText::Decorative)`, `decorative = true`. Decorative takes
> precedence in Stage 2b. `resolve_alt` emits `tracing::warn!(code = "W-A11-002")`
> at the detection site in the threading pass. This warning is NOT a validator
> diagnostic — it does not affect exit code or strict-mode gating.

### Item 6: Register W-A11-002 in the error taxonomy

**File:** `.factory/specs/prd-supplements/error-taxonomy.md`

**Owner:** Product Owner (taxonomy amendment)

**Change:** W-A11-002 is referenced in BC-3.04.001 v1.5.1 and BC-1.16.001 EC-004
but is NOT registered in the error taxonomy (the taxonomy has no W-A11 section — only
E-A11-001 through E-A11-004). Add a "Accessibility Warnings (W-A11)" subsection after
the "Accessibility Errors (E-A11)" section:

```markdown
## Accessibility Warnings (W-A11)

Non-fatal lint warnings emitted at Stage 2b threading time. Build continues; output
is produced. Exit code 0. Routed via `tracing::warn!` with a `code` structured field;
not subject to strict-mode gating.

| Code | Severity | Exit | Description | Traces To |
|------|---------|------|-------------|-----------|
| W-A11-002 | cosmetic | 0 | Emitted when a chart/image/diagram slide has both `decorative: true` and a non-empty `alt "..."`. Decorative takes precedence (BC-1.16.001 PC-12); the alt string is discarded. Message: "slide has both alt and decorative: true; decorative takes precedence for chart/image/diagram in Stage 2b. Consider removing the alt field or removing decorative: true." Also emitted for `shape:` blocks (BC-3.04.001 Invariant 11) when alt wins over decorative in that domain — same code, opposite resolution direction. | DI-001, BC-1.16.001 PC-12, BC-3.04.001 Inv-11 |
```

Note the one-code, two-contexts nature of W-A11-002: for shapes (BC-3.04.001), alt
wins; for media (BC-1.16.001), decorative wins. The warning message should be
context-specific (the Stage 2b message should say "decorative takes precedence";
the shape message should say "alt takes precedence"). Both use code W-A11-002 as
registered in BC-3.04.001 v1.5.1. If this dual-context usage is considered
confusing, the PO may allocate a distinct code (W-A11-003) for the Stage 2b
decorative-wins case — that decision is within PO authority and does not require
human escalation.

---

## Summary of Owners and Sequencing

| Item | Change | Owner | Gate |
|------|--------|-------|------|
| 1 | Fix `resolve_alt` to decorative-first | Implementer | Blocking — must ship in STORY-086 before merge |
| 2 | Add both-set unit test (decorative wins) | Implementer | Blocking — must ship in STORY-086 before merge |
| 3 | Correct ADR-019 Decision 4.1 conflict path | Architect | Blocking — apply before state-manager commit |
| 4 | BC-3.04.001 Inv-11 domain scope clarification | Product Owner | Non-blocking for merge, but must ship in same burst |
| 5 | BC-1.16.001 EC-004 warning mechanism correction | Product Owner | Non-blocking for merge, but must ship in same burst |
| 6 | Register W-A11-002 in error-taxonomy.md | Product Owner | Non-blocking for merge, but must ship in same burst |

Items 1 and 2 are implementer-owned and block STORY-086 merge. They must be done
before the next adversary pass can declare CLEAN.

Items 3 is architect-owned and can be applied directly; it amends an ADR that is in
the architect's authority to maintain.

Items 4, 5, and 6 are product-owner-owned spec amendments. They do not block the
code merge but must be committed in the same factory-artifacts burst with Items 3
to maintain consistency (per TD-VSDD-053 single-commit-per-burst; the PO and
architect amendments should be bundled into one state-manager commit).

---

## Human Authorization

**NOT REQUIRED.** This is a code-conforms adjudication: the code is wrong and
must be brought into alignment with the authoritative spec (BC-1.16.001 PC-12).
The spec amendment to BC-3.04.001 is a domain-scope clarification, not a semantic
change to that BC's existing invariant. The ADR-019 correction fixes an internal
consistency error in the state machine documentation. No new product decisions are
being made; no previously authorized decisions are being reversed.

The only scenario requiring human escalation would be if the human wishes to adopt
alt-first precedence for charts/images/diagrams (overriding BC-1.16.001 PC-12).
If that is desired, the human must authorize amendment of BC-1.16.001 PC-12 and
EC-004, which would flip the canonical rule and require re-verification of
AltText behavior across the pipeline. This adjudication does not assume that
outcome — it implements the spec as written.
