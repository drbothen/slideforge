---
document_type: architect-adjudication
story: STORY-086
pass: 6
date: 2026-06-06
findings_addressed: [F-086-P6-MED-001]
verdict: trimmed-is-canonical
resolution_type: adr-corrected
human_auth_required: false
---

# Architect Adjudication — STORY-086 Pass 6

## Conflict Summary (F-086-P6-MED-001)

**BC-1.16.001 v1.1** (the primary anchor BC for this story's threading pass) specifies
TRIMMED storage in three postconditions:

- PC-1 (title): `inlines: [InlineNode::Plain(Arc::from(s.trim()))]`
- PC-4 (body): `Arc::from(s.trim())`
- PC-12 (alt Provided case): `Some(AltText::Provided(Arc::from(s.trim())))`

Invariant 3 reinforces: "A field that resolves to an empty string (or whitespace-only
string after trim) produces no ContentBlock."

**ADR-019 v1.2** §3.1 table and §3.3 alt-rule specified UNTRIMMED storage:

- §3.1: `InlineNode::Plain(s)` for title, body, and subtitle rows
- Function-level doc comment (Decision 2): `AltText::Provided(Arc::from(s))`
- §3.3 alt-rule: `AltText::Provided(Arc::from(s))` for non-empty-after-trim alt

**The implementation** (`crates/slideforge-eval/src/field_to_block.rs`) conforms to
the ADR's untrimmed form, not to the BC's trimmed form.

Both artifacts are dated 2026-06-05. At the same precedence level within CLAUDE.md
rules, the tiebreaker applies.

---

## Verdict: TRIMMED IS CANONICAL

**Human authorization required: NO.**

### Precedence Rationale (CLAUDE.md Source-of-Truth Rule 1)

The conflict concerns **contract semantics** — specifically, what content value is
stored in a produced `ContentBlock` / `AltText`. CLAUDE.md Rule 1 states:

> "The BC supersedes when the conflict is about contract semantics."

BC-1.16.001 is the authoritative contract for Stage 2b threading pass semantics.
ADR-019 is an architectural decision record that describes HOW to implement the
contract — it is the specification of the implementation approach, not the behavioral
contract itself. When the two diverge on contract semantics, the BC wins.

### No Architectural Reason for Untrimmed

Architectural review finds no downstream stage that depends on raw (untrimmed)
whitespace in title/body/subtitle/alt strings. The pipeline's whitespace semantics are:

1. The DSL parser already strips leading/trailing blank lines from block text.
2. Exporters (PPTX, DOCX, PDF) do not round-trip raw whitespace from ContentBlock
   strings back to the layout engine — they render trimmed display text.
3. The empty-string skip rule (Invariant 3 / EC-001 / EC-002) already requires that
   whitespace-only strings produce no block, which implies `.trim()` is always applied
   before the non-empty check. Storing the untrimmed form after that check passes is
   inconsistent — if the trim result is "non-empty", the trimmed form is what should
   be stored.
4. No Kani proof harness, property-based test, or downstream consumer relies on
   receiving leading/trailing whitespace in `InlineNode::Plain` or `AltText::Provided`.

There is no compelling architectural reason for untrimmed to win. Trimmed is correct.

---

## ADR-019 Corrections Applied

### Version Bump

Frontmatter version bumped from `1.1` to `1.3`. The amendment log already recorded
v1.2 (Amendment B, pass-5 adjudication) but the frontmatter was not updated at that
time. This amendment corrects both: the frontmatter reflects the true current version
after three amendments.

### Amendment Log Entry (v1.3)

Added to Amendment Log table:

> Amendment C (STORY-086 pass-6 adjudication F-086-P6-MED-001): §3.1 table and §3.3
> alt-rule corrected to TRIMMED storage. `InlineNode::Plain(s)` →
> `InlineNode::Plain(Arc::from(s.trim()))` for title/subtitle/body;
> `AltText::Provided(Arc::from(s))` → `AltText::Provided(Arc::from(s.trim()))` for
> alt. Function-level doc comment (Decision 2 inline example) updated likewise.
> BC-1.16.001 PC-1/PC-4/PC-12 is the authoritative contract (contract semantics per
> CLAUDE.md precedence rule 1); ADR-019 is brought into alignment.

### §3.1 Table (Text Content — title, body, subtitle)

Three table rows updated. Before (all three rows):

```
InlineNode::Plain(s)
```

After (all three rows):

```
InlineNode::Plain(Arc::from(s.trim()))
```

Notes column updated to cite the relevant BC postcondition (PC-1, PC-4) and
confirm that whitespace is stripped.

### §3.3 Alt-Rule (Chart) — and by reference §3.4/§3.5

The Provided branch of the alt-resolution rule corrected. Before:

```
`fields["alt"] == Value::Str(s)` (non-empty after trim) → `alt = Some(AltText::Provided(Arc::from(s)))`, `decorative = false`
```

After:

```
`fields["alt"] == Value::Str(s)` (non-empty after trim) → `alt = Some(AltText::Provided(Arc::from(s.trim())))`, `decorative = false`
```

§3.4 (Image) and §3.5 (Diagram) both reference "identical to chart (3.3.alt)" — no
additional changes needed in those sections; the reference propagates the trimmed
form automatically.

### Decision 2 Doc Comment (Function-Level Rust Docstring)

Line in the `thread_fields_to_blocks` docstring corrected. Before:

```
/// 2. `Slide.fields["alt"] == Value::Str(s)` (non-empty, non-whitespace) → `ContentBlock.alt = Some(AltText::Provided(Arc::from(s)))`
```

After:

```
/// 2. `Slide.fields["alt"] == Value::Str(s)` (non-empty, non-whitespace) → `ContentBlock.alt = Some(AltText::Provided(Arc::from(s.trim())))`
```

---

## Change Plan for Downstream Owners

### Implementer (in-scope for current STORY-086 worktree)

**Files:** `crates/slideforge-eval/src/field_to_block.rs`

Three fix sites:

1. `make_text_block_tagged` (~line 324-333): wherever a `FieldValue::Literal(Value::Str(s))`
   is converted to `InlineNode::Plain(Arc::from(s))`, change to
   `InlineNode::Plain(Arc::from(s.trim()))`. The whitespace-only guard (the `.trim()`
   non-empty check that skips block emission) must be retained; the stored value must
   also be trimmed.

2. `resolve_alt` (~line 297-300): where `AltText::Provided(Arc::from(s))` is produced
   for the non-empty alt case, change to `AltText::Provided(Arc::from(s.trim()))`.

3. `extract_str_field` (~line 254-258): if this helper returns the raw string and the
   callers trim independently — confirm `resolve_alt` applies `.trim()` at the
   `AltText::Provided` construction site. If `extract_str_field` is the single source
   of string extraction, the trim should happen at the stored-value construction point,
   not in the predicate check.

**Test requirement (load-bearing — TD-VSDD-059):**

Add a unit test (e.g., `test_title_with_surrounding_whitespace_is_stored_trimmed`) that:
- Constructs a Deck slide with `fields["title"] = Value::Str("  My Title  ")` (padded).
- Calls `thread_fields_to_blocks(&mut deck)`.
- Asserts `ContentBlock::Text(TextBlock { inlines: [InlineNode::Plain(s)], .. })` where
  `s == "My Title"` (trimmed), NOT `"  My Title  "`.

A parallel test for the alt Provided case:
- `fields["alt"] = Value::Str("  Team photo  ")` on an image slide.
- Asserts `AltText::Provided(s)` where `s == "Team photo"` (trimmed).

The whitespace-only skip test (EC-002 canonical vector) is already specified in the
BC and must be present: `fields["title"] = Value::Str("  ")` produces no block.

### Test-Writer (trim Red Gate)

The above tests constitute the trim Red Gate. They must be written as failing tests
BEFORE the implementer applies the trim fix, to confirm the current untrimmed behavior
is captured as a Red test. After the fix they turn Green.

---

## STORY-086 Story File — No Contradictions Found

A scan of the STORY-086 story spec for trim-semantics claims is not possible in this
scope (implementer owns the worktree; the story file is in the worktree). The
implementer should confirm that the story's acceptance criteria do not assert
untrimmed storage. If any AC asserts `Plain(s)` with raw whitespace, the story-writer
should be notified for correction — the architect does NOT edit story files.

---

## Summary

| Item | Action | Owner | Gate |
|------|--------|-------|------|
| ADR-019 §3.1 table (title/body/subtitle) | Corrected to `Arc::from(s.trim())` | Architect (done) | Applied in this burst |
| ADR-019 §3.3 alt-rule Provided branch | Corrected to `Arc::from(s.trim())` | Architect (done) | Applied in this burst |
| ADR-019 Decision 2 doc comment | Corrected to `Arc::from(s.trim())` | Architect (done) | Applied in this burst |
| ADR-019 frontmatter version | Bumped to v1.3 | Architect (done) | Applied in this burst |
| `make_text_block_tagged` trim fix | Add `.trim()` at `Plain(Arc::from(...))` | Implementer | Blocking — before next adversary pass |
| `resolve_alt` trim fix | Add `.trim()` at `AltText::Provided(Arc::from(...))` | Implementer | Blocking — before next adversary pass |
| Trim unit tests (title + alt) | Write as Red Gate before implementer fix | Test-writer | Blocking |
| STORY-086 story ACs | Confirm no untrimmed assertion; correct if found | Story-writer / Implementer | Non-blocking for spec commit |

**Human authorization required: NO.** This is a spec-alignment correction bringing
ADR-019 into conformance with BC-1.16.001 on a pure contract-semantics question
where the BC is the authoritative source per CLAUDE.md Rule 1. No product decisions
are being changed; the behavioral contract has always mandated trimmed storage.
