---
document_type: domain-spec-section
level: L2
section: "edge-cases"
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q4-q15-decisions.md
  - .factory/planning/q16-q25-decisions.md
  - .factory/planning/domain-research.md
  - .factory/planning/dsl-competitor-analysis.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Domain Edge Cases

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.

---

## DEC-001: Empty @for Loop

`@for item in collection:` where `collection` evaluates to an empty list or empty object.
Expected: zero slides or elements generated for that block. No error. Document section
types that derive from @for data (e.g., risk_register) should render as "no items" or
be omitted from the document structure depending on configuration.

**Why it matters:** An empty data source is a legitimate production state (e.g., "no active
incidents today"). Silent omission vs. empty-section rendering must be specified.
**Traces to:** CAP-004, DI-005.

---

## DEC-002: Data Source Returns Null or Missing Field

`{{ item.field }}` where `item` lacks the `field` key in a JSON/CSV row.
Expected: compile error with scope path and source span. No silent empty-string substitution.
**Why it matters:** Missing fields in data-driven slides silently produce blank content areas
that look like bugs but are actually data errors (domain-research.md §4.3, nullGetter research).
**Traces to:** CAP-003, DI-006.

---

## DEC-003: Circular Include Chain

`deck.sf` includes `a.sf` which includes `b.sf` which includes `deck.sf`.
Expected: compile error showing the full cycle path: `deck.sf → a.sf → b.sf → deck.sf`.
**Traces to:** CAP-006, DI-007.

---

## DEC-004: Variable Shadowing Across Variant Inheritance

A deck declares `vars: { color: "blue" }`. A variant `exec` declares `vars: { color: "red" }`.
A child variant inherits exec and also declares `vars: { color: "green" }`.
Expected: child's own vars win (highest precedence per Q25 11-level chain). Result: `"green"`.
**Why it matters:** Multi-level variant inheritance with name collisions is the primary source
of unexpected output in composable document systems.
**Traces to:** CAP-007, DI-020.

---

## DEC-005: Brand Color Token Used Before Brand Is Loaded

A `set severity_cards: color_high brand.danger` where `brand.danger` is referenced
before the brand file is resolved.
Expected: brand references in set rules are evaluated after brand loading. Order of
declaration in the .sf file must not affect the result. Build error if the brand token
does not exist.
**Traces to:** CAP-008, CAP-018.

---

## DEC-006: @include Path with Variable That Resolves to Nonexistent File

`@include "{{ client_name }}/template.sf"` where `client_name` resolves to a name whose
corresponding file does not exist.
Expected: compile error after variable resolution, showing the resolved path and span.
**Why it matters:** Dynamic include paths (Q9) create a class of errors where the source
is a variable value, not a literal path. The error message must show the resolved path.
**Traces to:** CAP-006, DI-006.

---

## DEC-007: @for Over HTTP Data Source That Fails During Watch Mode

An HTTP data source returns a 404 or network error during a watch-mode re-build cycle.
Expected: error-slide placeholder rendered for affected slides; last successfully fetched
data is not stale-served (no implicit caching of last-good data in v1.0); diagnostic
shown in CLI.
**Traces to:** CAP-027, FM-006.

---

## DEC-008: Slide with Only Decorative Images

A slide where every image has `decorative: true` and no other meaningful visual content.
Expected: no compile error (decorative flag is the explicit opt-out from alt requirement).
All images emit empty alt attributes and PDF Artifact tags in PDF. Accessibility validators
do not flag this as an error.
**Traces to:** CAP-020, DI-001.

---

## DEC-009: Math Mode Delimiters Inside String Fields

`title "Revenue: ${{ kpis.arr | currency }}"` — a literal `$` in a string field that
does not open math mode because it is immediately followed by `{{`.
Expected: The `${{` sequence is a special case. `$` followed by `{{` starts variable
interpolation in text mode, not math mode. The result is the currency value, not a math
expression. Parser must handle this ambiguity explicitly.
**Traces to:** CAP-002, CAP-012.

---

## DEC-010: Slide Type Keyword Used as Variable Name

`vars: { chart: "my-chart-data" }` — user names a variable the same as a slide type keyword.
Expected: compile error. Variable names that collide with reserved slide type keywords or
reserved keywords (Q22) are rejected.
**Traces to:** CAP-001, DI-021.

---

## DEC-011: Zero-Slide Deck

A .sf file that defines `vars:` and `set` rules but contains no `slide` blocks.
Expected: validation error ("Deck must contain at least one slide"). No output.
**Traces to:** entities.md (Deck invariant: at least one slide required), DI-017.

---

## DEC-012: Variant That Excludes All Slides

A variant where `exclude_tags` is broad enough to exclude every slide in the deck.
Expected: validation warning ("Variant 'exec' produces a zero-slide deck"). Build
continues in warn-only; no output file written for the variant in strict mode.
**Traces to:** CAP-007, DI-017.

---

## DEC-013: Text Overflow in Content Slide

A `slide content:` with a `bullets:` list that, when rendered at the brand's font size
and margins, exceeds the slide canvas height.
Expected: `CanvasOverflow` validation warning with the slide title, field name, and
estimated overflow amount. Not a compile error in default build (user may intentionally
have long content). In watch mode: warning shown in web preview overlay.
**Traces to:** CAP-022, FM-005.

---

## DEC-014: Chart Data Source Has Zero Rows

`slide chart:` where `data {{ kpis.monthly }}` evaluates to an empty list.
Expected: validation warning ("Chart data is empty"). Error slide placeholder rendered.
Chart exporter must not crash on empty data.
**Traces to:** CAP-013, DI-006.

---

## DEC-015: Mermaid Diagram Syntax Error

`slide diagram:` with invalid Mermaid syntax in the `source:` field.
Expected: compile-time error from the DiagramRenderer plugin with line number within
the diagram source block and a hint. Error slide placeholder in watch mode.
**Traces to:** CAP-014.

---

## DEC-016: brand.toml Missing a Required Color Slot

A brand.toml that defines `acc1` and `acc2` but not `dk1`, `lt1`, etc.
Expected: Brand synthesizer fills in all 12 OOXML scheme color slots using sensible
defaults derived from the declared colors (e.g., `dk1` = darkest declared color).
A warning lists the inferred slots so the user can review them.
**Traces to:** CAP-018, DI-015.

---

## DEC-017: Nested @for with Outer Scope Variable Shadowing

```
@for project in projects:
  @for item in project.items:
    title "{{ project.name }}: {{ item.name }}"
```
Inner `@for` loop must have access to outer loop variable `project`.
Expected: lexical scoping; outer variables remain in scope inside inner loops. Variable
`item` does not shadow outer loop variables.
**Traces to:** CAP-004, CAP-002.

---

## DEC-018: HTTP Data Source Returns Different Schema on Re-fetch

Watch mode re-fetches an HTTP data source that previously returned `{ "count": 5 }` but
now returns `{ "total": 5 }` (field rename). Slides referencing `{{ data.count }}` now
have undefined variable references.
Expected: Undefined variable errors shown as error-slide placeholders in watch mode.
The schema change is not silently tolerated.
**Traces to:** DEC-002, CAP-027.

---

## DEC-019: @import Resolves to Package Not in sf.lock

`@import "1898-slides/incident-card"` where `1898-slides` is not in `sf.lock` (package
was not installed).
Expected: compile error ("Package '1898-slides' not found in sf.lock. Run: slideforge
package install github.com/1898/slides").
**Traces to:** CAP-025, DI-019.

---

## DEC-020: Deck Built with --variant That Does Not Exist

`slideforge build deck.sf --variant internal-exec` where `internal-exec` is not declared
in the deck's `variants:` block.
Expected: compile error with the undefined variant name. Build does not fall back to
the default (no-variant) build. Error lists defined variants to help the user.
**Traces to:** CAP-007, DI-017.
