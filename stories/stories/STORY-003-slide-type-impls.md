---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-003
title: "31 SlideType Implementations"
epic: EPIC-01
wave: 1
points: 8
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-types
subsystems: [SS-15]
target_module: slideforge-types
behavioral_contracts: [BC-3.01.001, BC-3.01.002, BC-3.01.003]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on: [STORY-001, STORY-002]
blocks:
  - STORY-026
  - STORY-049
  - STORY-050
estimated_days: 3
---

# STORY-003: 31 SlideType Implementations

## Summary

Implement all 31 built-in slide types as `SlideType` trait implementations within
`slideforge-types/src/slide_types/`. Each type declares its required fields, optional
fields, and OOXML layout name. This story satisfies BC-3.01.001 (required fields
enforced via trait), BC-3.01.002 (missing required field produces structured error with
field name and type), and BC-3.01.003 (unknown slide type keyword produces E-PAR-007
with closest-match suggestion via edit-distance). The `lay_out` method returns a stub
`LaidOutSlide` in this story — full geometric layout is implemented in STORY-026.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,000 |
| 31 slide type implementation files | ~15,000 |
| `slide_type_registry.rs` with keyword table + edit-distance | ~3,000 |
| Test fixtures (missing field, typo detection) | ~3,000 |
| **Total** | **~25,000** |

Agent context budget: 200k tokens. This story is ~12.5% of budget — within the 20-30% limit.

## Behavioral Contracts

| BC ID | Title | Clauses Covered |
|-------|-------|----------------|
| BC-3.01.001 | Each slide type enforces required fields via SlideType trait | Preconditions 1-3; Postconditions 1-3; Invariants 1-3 |
| BC-3.01.002 | Missing required field on slide type produces compile error with field name | Preconditions 1-2; Postconditions 1-4; Invariants 1-2 |
| BC-3.01.003 | Unknown slide type keyword produces compile error with suggestion | Preconditions 1-2; Postconditions 1-4; Invariants 1-3 |

## Acceptance Criteria

- [ ] **AC-001:** All 31 slide types are implemented as separate structs implementing `SlideType`. Each struct returns a non-empty `&[FieldDef]` from `required_fields()`. (traces to BC-3.01.001 postcondition 1 and invariant 1)
- [ ] **AC-002:** `SlideTypeRegistry` struct provides `lookup_by_keyword(keyword: &str) -> Option<&dyn SlideType>` that returns `Some` for all 31 registered keywords and `None` for unknown keywords. (traces to BC-3.01.003 precondition 1)
- [ ] **AC-003:** `SlideTypeRegistry::suggest(unknown: &str) -> Option<&str>` returns the closest-match keyword using Levenshtein edit-distance when `distance <= 3`, or `None` when no type is within edit-distance 3. (traces to BC-3.01.003 postcondition 1 and invariant 1)
- [ ] **AC-004:** `SlideTypeRegistry::all_keywords() -> &[&str]` returns a slice of all 31 registered type keywords — used in E-PAR-007 fallback message when no suggestion is available. (traces to BC-3.01.003 invariant 2)
- [ ] **AC-005:** The `validate_fields` free function (or method on `SlideTypeRegistry`) accepts a `&Slide` and a `&dyn SlideType` and returns `Vec<Diagnostic>` — one diagnostic per missing or empty required field, with format: "Required field '{field}' missing on {type} slide at {file}:{line}:{col}. Required fields for '{type}': [{list}]." (traces to BC-3.01.002 postcondition 1)
- [ ] **AC-006:** `validate_fields` accumulates ALL missing field diagnostics before returning (does not short-circuit on the first missing field). (traces to BC-3.01.002 invariant 1 and BC-1.15.002)
- [ ] **AC-007:** An empty string value for a required field triggers the same diagnostic as an absent field — "Required field '{field}' is empty on {type} slide..." (traces to BC-3.01.001 invariant 2)
- [ ] **AC-008:** Fields present in the slide DSL but not declared by the `SlideType` produce a `DiagnosticSeverity::Warning` (not error) with message "Unknown field '{field}' for slide type '{type}'. Known fields: [...]." (traces to BC-3.01.001 invariant 3)
- [ ] **AC-009:** `slide content:` type registers `required_fields()` as `[FieldDef { name: "title", required: true }]` and `optional_fields()` as `[bullets, takeaway, notes, ...]`. Verified by unit test. (traces to BC-3.01.001 postcondition 1)
- [ ] **AC-010:** `slide stat_callout:` type registers all 4 stat fields (`stat_1`, `label_1`, `stat_2`, `label_2`) as required. Unit test verifies required field count. (traces to BC-3.01.001 postcondition 1)
- [ ] **AC-011:** `slide title:` type registers `title` as the only required field. Unit test verifies. (traces to BC-3.01.001 postcondition 1)
- [ ] **AC-012:** `SlideType::lay_out` on all 31 types returns `Ok(LaidOutSlide { elements: vec![], .. })` stub in this story — NOT `unimplemented!()`. The stub must compile and return `Ok`. (design choice — unimplemented!() panics at runtime; Ok stub allows test pipelines to complete.) (traces to BC-3.01.001 postcondition 1)
- [ ] **AC-013:** A unit test verifies that `SlideTypeRegistry::suggest("conetnt")` returns `Some("content")` (Levenshtein distance 1). (traces to BC-3.01.003 edge case EC-001)
- [ ] **AC-014:** A unit test verifies that `SlideTypeRegistry::suggest("flibbertigibbet")` returns `None` (distance > 3). (traces to BC-3.01.003 edge case EC-002)
- [ ] **AC-015:** `#![forbid(unsafe_code)]` is present on the crate root (inherited from STORY-001 — verify it's still there) (NFR-024).
- [ ] **AC-016:** `cargo clippy -p slideforge-types -- -D warnings` produces zero warnings after this story's additions (NFR-022).
- [ ] **AC-017:** All 31 slide type keywords are discoverable via `SlideTypeRegistry::all_keywords()` — unit test asserts `all_keywords().len() == 31`. (traces to BC-3.01.003 invariant 2)

## Previous Story Intelligence

- STORY-001 defines `Slide`, `FieldValue`, `ContentBlock`, `SourceSpan`, `LaidOutSlide`, `LaidOutDeck`, `Brand`. This story depends on those types being present.
- STORY-002 defines the `SlideType` trait and `FieldDef`. The `required_fields()` and `optional_fields()` methods return `&[FieldDef]` as defined there.
- `validate_fields` uses `Diagnostic` from `slideforge-plugin-api` — import accordingly.

## Architecture Compliance Rules

Sourced from `architecture/crate-architecture.md` and `architecture/plugin-architecture.md`:

1. **Pure Core (SS-15):** `slideforge-types` is Pure Core. No I/O in any slide type implementation. `lay_out` stubs return `Ok(LaidOutSlide::stub())`.
2. **Trait usage (DI-008):** All 31 slide types implement the `SlideType` trait from `slideforge-plugin-api`. No ad-hoc per-type if-chains anywhere in the codebase. The trait IS the dispatch mechanism.
3. **Registry is single source of truth (BC-3.01.003 invariant 3):** The `SlideTypeRegistry` holds the 31 registered types. Unknown keyword detection uses the registry, not a hard-coded match list.
4. **Edit-distance for suggestions (BC-3.01.003 invariant 1):** Use the `strsim` crate (Levenshtein or Jaro-Winkler). Do not write a custom edit-distance implementation — `strsim` is the approved library.
5. **Forbidden dependencies:** This code lives in `slideforge-types`. It MUST NOT depend on `slideforge-eval`, `slideforge-syntax`, or any effectful crate. It may import from `slideforge-plugin-api` (for `SlideType` trait and `FieldDef`).

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `slideforge-types` | workspace path (self) | All IR types |
| `slideforge-plugin-api` | workspace path | `SlideType` trait, `FieldDef`, `Diagnostic` |
| `strsim` | `=0.11.1` | Levenshtein edit-distance for slide type suggestions |
| `thiserror` | `=2.0.18` | Already in crate from STORY-001 |

## File Structure Requirements

Files to create (all within `crates/slideforge-types/`):

```
src/
├── slide_types/
│   ├── mod.rs                       # pub use all type modules; SLIDE_TYPES const array
│   ├── registry.rs                  # SlideTypeRegistry + validate_fields fn
│   │
│   # === Standard types (11) ===
│   ├── title.rs                     # slide title:
│   ├── section_break.rs             # slide section_break:
│   ├── content.rs                   # slide content:
│   ├── two_col.rs                   # slide two_col:
│   ├── image.rs                     # slide image:
│   ├── blank.rs                     # slide blank:
│   ├── agenda.rs                    # slide agenda:
│   ├── toc.rs                       # slide toc:
│   ├── quote.rs                     # slide quote:
│   ├── team.rs                      # slide team:
│   ├── bio.rs                       # slide bio:
│   │
│   # === Custom types (20: CL-01 through CL-20) ===
│   ├── executive_summary.rs         # slide executive_summary:   [CL-01]
│   ├── problem_statement.rs         # slide problem_statement:   [CL-02]
│   ├── recommendation.rs            # slide recommendation:      [CL-03]
│   ├── risk_register.rs             # slide risk_register:       [CL-04]
│   ├── timeline.rs                  # slide timeline:            [CL-05]
│   ├── stat_callout.rs              # slide stat_callout:        [CL-06]
│   ├── comparison.rs                # slide comparison:          [CL-07]
│   ├── process_flow.rs              # slide process_flow:        [CL-08]
│   ├── matrix.rs                    # slide matrix:              [CL-09]
│   ├── financials.rs                # slide financials:          [CL-10]
│   ├── chart.rs                     # slide chart:               [CL-11]
│   ├── diagram.rs                   # slide diagram:             [CL-12]
│   ├── screenshot.rs                # slide screenshot:          [CL-13]
│   ├── code_sample.rs               # slide code_sample:         [CL-14]
│   ├── video.rs                     # slide video:               [CL-15]
│   ├── survey_results.rs            # slide survey_results:      [CL-16]
│   ├── org_chart.rs                 # slide org_chart:           [CL-17]
│   ├── roadmap.rs                   # slide roadmap:             [CL-18]
│   ├── kpi_dashboard.rs             # slide kpi_dashboard:       [CL-19]
│   └── closing.rs                   # slide closing:             [CL-20]
```

Files to MODIFY:
- `src/lib.rs` — add `pub mod slide_types;` declaration.
- `Cargo.toml` — add `strsim = "=0.11"` to `[dependencies]`.

## 31 Slide Types Reference Table

| Keyword | Category | Required Fields | Optional Fields |
|---------|----------|----------------|----------------|
| `title` | Standard | `title` | `subtitle`, `author`, `date`, `notes` |
| `section_break` | Standard | `title` | `subtitle`, `notes` |
| `content` | Standard | `title` | `bullets`, `takeaway`, `notes`, `report` |
| `two_col` | Standard | `title`, `left`, `right` | `notes`, `report` |
| `image` | Standard | `title`, `image`, `alt` | `caption`, `notes` |
| `blank` | Standard | — | `notes` |
| `agenda` | Standard | `title` | `items`, `notes` |
| `toc` | Standard | `title` | `notes` |
| `quote` | Standard | `quote`, `attribution` | `notes` |
| `team` | Standard | `title` | `members`, `notes` |
| `bio` | Standard | `name`, `title`, `bio` | `image`, `alt`, `notes` |
| `executive_summary` | Custom CL-01 | `title`, `summary` | `bullets`, `notes`, `report` |
| `problem_statement` | Custom CL-02 | `title`, `problem` | `impact`, `notes`, `report` |
| `recommendation` | Custom CL-03 | `title`, `recommendation` | `rationale`, `risk`, `notes`, `report` |
| `risk_register` | Custom CL-04 | `title` | `risks`, `notes`, `report` |
| `timeline` | Custom CL-05 | `title` | `milestones`, `notes`, `report` |
| `stat_callout` | Custom CL-06 | `stat_1`, `label_1`, `stat_2`, `label_2` | `stat_3`, `label_3`, `notes` |
| `comparison` | Custom CL-07 | `title`, `option_a`, `option_b` | `criteria`, `notes`, `report` |
| `process_flow` | Custom CL-08 | `title` | `steps`, `notes`, `report` |
| `matrix` | Custom CL-09 | `title` | `cells`, `notes`, `report` |
| `financials` | Custom CL-10 | `title` | `rows`, `notes`, `report` |
| `chart` | Custom CL-11 | `title`, `chart_type`, `data` | `alt`, `notes`, `report` |
| `diagram` | Custom CL-12 | `title`, `diagram` | `alt`, `notes`, `report` |
| `screenshot` | Custom CL-13 | `title`, `image`, `alt` | `caption`, `notes` |
| `code_sample` | Custom CL-14 | `title`, `code` | `language`, `notes` |
| `video` | Custom CL-15 | `title`, `video_url`, `alt` | `notes` |
| `survey_results` | Custom CL-16 | `title` | `results`, `notes`, `report` |
| `org_chart` | Custom CL-17 | `title` | `nodes`, `notes` |
| `roadmap` | Custom CL-18 | `title` | `phases`, `notes`, `report` |
| `kpi_dashboard` | Custom CL-19 | `title` | `kpis`, `notes`, `report` |
| `closing` | Custom CL-20 | `title` | `call_to_action`, `contact`, `notes` |

## Tasks

1. **Create `src/slide_types/registry.rs`** — `SlideTypeRegistry`, `validate_fields`, `suggest`, `all_keywords`. (45 min)
2. **Create `src/slide_types/mod.rs`** — wire up all 31 types; `SLIDE_TYPE_REGISTRY: LazyLock<SlideTypeRegistry>`. (20 min)
3. **Create standard types (11 files):** `title.rs`, `section_break.rs`, `content.rs`, `two_col.rs`, `image.rs`, `blank.rs`, `agenda.rs`, `toc.rs`, `quote.rs`, `team.rs`, `bio.rs` — each with `required_fields`, `optional_fields`, `layout_name`, and stub `lay_out`. (90 min)
4. **Create custom types CL-01 through CL-10 (10 files):** `executive_summary.rs` through `financials.rs` (90 min)
5. **Create custom types CL-11 through CL-20 (10 files):** `chart.rs` through `closing.rs` (90 min)
6. **Add `strsim = "=0.11.1"` to `Cargo.toml`** and update `src/lib.rs`. (5 min)
7. **Write unit tests in `registry.rs`** — typo detection, all 31 keywords, field validation accumulation. (30 min)
8. **Run `cargo test -p slideforge-types`** — confirm all tests pass. (5 min)
9. **Run `cargo clippy -p slideforge-types -- -D warnings`** — fix any warnings. (15 min)

## Test Strategy

### Unit tests (`src/slide_types/registry.rs` `#[cfg(test)]`)

```rust
#[test]
fn suggest_single_char_typo() {
    let reg = SlideTypeRegistry::default();
    assert_eq!(reg.suggest("conetnt"), Some("content"));
    assert_eq!(reg.suggest("ttle"), Some("title"));
    assert_eq!(reg.suggest("satcallout"), Some("stat_callout"));
}

#[test]
fn suggest_no_match_far_edit_distance() {
    let reg = SlideTypeRegistry::default();
    assert_eq!(reg.suggest("flibbertigibbet"), None);
}

#[test]
fn all_31_keywords_present() {
    let reg = SlideTypeRegistry::default();
    assert_eq!(reg.all_keywords().len(), 31);
}

#[test]
fn validate_fields_accumulates_all_missing() {
    let reg = SlideTypeRegistry::default();
    // stat_callout requires 4 fields; provide none
    let slide = Slide { slide_type: Arc::from("stat_callout"), fields: IndexMap::new(), .. };
    let slide_type = reg.lookup_by_keyword("stat_callout").unwrap();
    let diags = validate_fields(&slide, slide_type.as_ref());
    assert_eq!(diags.len(), 4, "should accumulate all 4 missing required fields");
    assert!(diags.iter().all(|d| d.severity == DiagnosticSeverity::Error));
}

#[test]
fn empty_string_counts_as_missing() {
    let reg = SlideTypeRegistry::default();
    let mut fields = IndexMap::new();
    fields.insert(Arc::from("title"), FieldValue::Literal(Value::Str(Arc::from(""))));
    let slide = Slide { slide_type: Arc::from("title"), fields, .. };
    let slide_type = reg.lookup_by_keyword("title").unwrap();
    let diags = validate_fields(&slide, slide_type.as_ref());
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("is empty"));
}

#[test]
fn unknown_field_is_warning_not_error() {
    let reg = SlideTypeRegistry::default();
    let mut fields = IndexMap::new();
    fields.insert(Arc::from("title"), FieldValue::Literal(Value::Str(Arc::from("My Slide"))));
    fields.insert(Arc::from("bogus_field"), FieldValue::Literal(Value::Str(Arc::from("x"))));
    let slide = Slide { slide_type: Arc::from("title"), fields, .. };
    let slide_type = reg.lookup_by_keyword("title").unwrap();
    let diags = validate_fields(&slide, slide_type.as_ref());
    // No errors (title is present and non-empty)
    assert!(diags.iter().all(|d| d.severity == DiagnosticSeverity::Warning));
}
```

### Snapshot tests

None in this story. Rendered AST snapshot tests come in STORY-006 (parser).

## Dependencies

**Depends on:**
- STORY-001 (IR core types): needs `Slide`, `FieldValue`, `Value`, `LaidOutSlide`, `Brand`, `SourceSpan`. Justification: `SlideType::lay_out` signature references `Slide`, `Brand`, `Canvas`, `LaidOutSlide` — all from STORY-001.
- STORY-002 (plugin trait api): needs `SlideType` trait, `FieldDef`, `Diagnostic`, `DiagnosticSeverity`. Justification: all 31 implementations use the `SlideType` trait as their interface.

**Blocks:**
- STORY-006 (parser core): the parser needs the slide type keyword table to validate `slide <keyword>:` declarations at parse time.
- STORY-049 (plugin registry): needs all 31 `SlideType` implementations to register them.
- STORY-050 (e2e tests): needs slide type implementations to run end-to-end pipeline tests.

## Implementation Notes

### `SlideTypeRegistry` Implementation Pattern

```rust
use std::collections::HashMap;
use std::sync::Arc;

pub struct SlideTypeRegistry {
    types: HashMap<Arc<str>, Box<dyn SlideType>>,
    keywords: Vec<Arc<str>>,  // in registration order for deterministic output
}

impl Default for SlideTypeRegistry {
    fn default() -> Self {
        let mut r = Self::new();
        r.register(Box::new(TitleSlideType));
        r.register(Box::new(ContentSlideType));
        // ... all 31 types ...
        r
    }
}
```

### Stub `lay_out` Implementation Pattern

Every slide type in this story returns a stub `LaidOutSlide`. Do NOT use `unimplemented!()` or `todo!()` — that panics at runtime and blocks test pipelines:

```rust
fn lay_out(&self, slide: &Slide, _brand: &Brand, canvas: Canvas) -> Result<LaidOutSlide, LayoutError> {
    // Stub implementation — full layout computed in STORY-026
    Ok(LaidOutSlide {
        elements: vec![],
        slide_number: 0,
        source_slide: Arc::new(slide.clone()),
    })
}
```

### `required_fields` Pattern

```rust
fn required_fields(&self) -> &[FieldDef] {
    static FIELDS: std::sync::LazyLock<Vec<FieldDef>> = std::sync::LazyLock::new(|| vec![
        FieldDef { name: Arc::from("title"), required: true, description: Arc::from("Slide title text"), default_value: None },
    ]);
    &FIELDS
}
```

Use `std::sync::LazyLock` (stable since Rust 1.80) for static field arrays. Do NOT use `once_cell` — Rust's `LazyLock` is the standard since edition 2024.

### `validate_fields` Error Format

The exact error message format per BC-3.01.002 postcondition 1:
```
Required field 'title' missing on content slide at src/deck.sf:12:1. Required fields for 'content': [title].
```

The hint includes the required fields list. Use `Diagnostic::hint` field for this.

### Edit-Distance Threshold

Per BC-3.01.003 invariant 1: suggest when `strsim::levenshtein(unknown, candidate) <= 3`. If multiple candidates are within distance 3, return the one with the smallest distance. Ties broken by keyword alphabetical order.

### `blank` Slide Type

`slide blank:` has zero required fields. `required_fields()` returns `&[]`. This is valid — blank slides are intentionally empty. `validate_fields` on a blank slide with no fields returns an empty `Vec<Diagnostic>`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slide conetnt:` (typo) | E-PAR-007 with suggestion "content"; edit-distance 1 |
| EC-002 | `slide xyz:` (edit-distance > 3) | E-PAR-007 with no suggestion; message lists all 31 types |
| EC-003 | `slide raw:` (reserved keyword) | E-PAR-006 (reserved keyword), not E-PAR-007 — handled by parser (STORY-009), not registry |
| EC-004 | `slide blank:` with no fields | No missing-field errors — `blank` has zero required fields |
| EC-005 | `slide stat_callout:` with only 2 of 4 required stats | 2 diagnostic errors accumulated; the 2 present fields are not mentioned |
| EC-006 | `slide CONTENT:` (wrong case) | Keyword lookup is case-sensitive (per BC-3.01.003 EC-004); suggests "content" via edit-distance |
