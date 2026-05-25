---
document_type: domain-spec-section
level: L2
section: "invariants"
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
  - .factory/planning/brand-template-patterns.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Domain Invariants

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> DI-NNN are business rules that must ALWAYS hold. Violation = system defect.
> These reflect domain rules, not implementation constraints.

---

## Accessibility Invariants

### DI-001: Alt Text Required on All Visual Elements

Every image, chart, and diagram in every output format must have non-empty alt text, OR
must be explicitly marked `decorative: true`. This is enforced at compile time (validator
produces a blocking error, not a warning). No output is produced for a deck that violates
this rule in strict mode.

**Business invariant because:** accessibility is a compile-time contract (product-brief.md §2,
Q6). A document that silences screen-reader users is not a valid branded document. Deferred
alt text would defeat the entire accessibility value proposition.

### DI-002: Color-Coded Elements Must Have Text Labels

Any slide element that uses color to convey meaning (severity_cards severity levels,
status indicator, progress_bar fill, weighted_composite scores) must also declare
`label "..."` with the meaning in text form. Color alone must never be the sole differentiator.

**Business invariant because:** WCAG 1.4.1 (Use of Color). Enforced at compile time (Q6).

### DI-003: Deck Language Must Be Declared

Every deck must declare `lang "..."` in its metadata (defaults to "en" if absent but
produces a lint warning). Language declaration propagates to all output formats: PPTX
Core Properties, PDF `/Lang` tag, HTML `<html lang="...">`.

**Business invariant because:** screen readers use language to select pronunciation engine;
missing language declaration breaks accessibility for non-English content (Q6, domain-research.md §2.4).

---

## DSL / Computation Invariants

### DI-004: No Implicit Type Coercion

Values in the DSL are never silently coerced. The string "NO" stays "NO" (not `false`).
The string "1.10" stays "1.10" (not `1.1` float). Numeric operations require explicit
formatting functions. YAML-style implicit coercion is a compile error.

**Business invariant because:** implicit coercion was identified as a pain point in competitor
tools (R3 research, dsl-competitor-analysis.md). Silent data mutation corrupts data-driven
documents silently — the opposite of the "deterministic, version-controllable" product promise.

### DI-005: Computation Must Be Provably Terminating

The computation model (rungs 1-9) must guarantee termination: `@for` iterates over finite
collections only, `@if` is a branch (not a loop), no recursion, no user-defined functions.
Any construct that could produce an infinite loop is rejected at parse time or compile time.

**Business invariant because:** the product promise of < 500ms cold builds depends on bounded
computation. Provable termination is also required for formal verification via Kani (CLAUDE.md
quality bar). This is a domain constraint, not a performance optimization.

### DI-006: Undefined Variables Are Compile Errors

Any `{{ var }}` interpolation that references a variable not in scope at evaluation time
is a compile error with file:line:col span. There is no runtime fallback, no empty-string
substitution, no "undefined" sentinel.

**Business invariant because:** silent missing-data propagation is the root cause of corrupted
data-driven slides (docxtemplater nullGetter research, domain-research.md §4.3). A variable
that expands to empty silently produces a broken document — worse than a compile error.

### DI-007: Include Cycles Are Compile Errors

Any `@include` or `@import` chain that creates a cycle must be detected at compile time
and reported as an error with the full cycle path shown.

**Business invariant because:** include cycles produce infinite loops in naive parsers. Cycle
detection is a correctness invariant for the composition system (Q12).

---

## Architecture Invariants

### DI-008: All Bundled Plugins Must Use Plugin Trait APIs

No bundled plugin may access internal data structures or functions outside of its declared
plugin trait interface. If a bundled plugin needs to bypass the API, the API is wrong and
must be fixed. This is the "dog-fooding guarantee" (Q3 decision).

**Business invariant because:** if bundled plugins bypass the trait API, the trait API will
never be correct for third-party use. The extensibility value proposition collapses. This
rule is the direct statement of the Plugin Principle (q3-decision-final.md).

### DI-009: Two-IR Model Integrity

The `Deck` IR (semantic, pre-layout) and `LaidOutDeck` IR (geometric, post-layout) must
remain distinct types. No exporter may add layout/geometry information to the `Deck` IR.
No layout engine output may lose semantic information required by exporters.

**Business invariant because:** this boundary is what makes exporters independently pluggable
(ir-prior-art.md). Mixing semantic and geometric concerns in one IR type is the failure mode
observed in python-pptx (class hierarchy anti-pattern from R6).

### DI-010: Integer EMU for All Coordinates

All coordinate and dimension values in the `LaidOutDeck` IR use integer EMU units (914400
per inch), never `f64`. Conversion to/from user-facing units (inches, cm, pt) happens at
DSL parse/layout boundary only.

**Business invariant because:** floating-point coordinates make `Hash + Eq` impossible
(required for comemo). Integer EMU is also required for OOXML schema correctness (R4
research: element ordering and coordinate types are schema-significant).

### DI-011: All IR Types Must Implement Hash + Eq + Clone

Every type in the `Deck` and `LaidOutDeck` IRs must implement `Hash + Eq + Clone` from
initial declaration. Adding these traits retroactively after widespread use would require
API-breaking changes.

**Business invariant because:** `comemo` incremental compilation (planned for v1.x) requires
structurally hashable IR types. The constraint must be designed in from day one (domain-research.md
§1.4, ir-prior-art.md).

---

## Output Format Invariants

### DI-012: Single .sf Source Produces All Formats Consistently

The same .sf source file, built with the same brand and variant, must produce semantically
equivalent content across all five output formats. A fact that appears in the PPTX slides
must appear in the DOCX report (via the `report` register) and vice versa.

**Business invariant because:** the product promise is "one source, every format." Divergent
content across formats would require users to maintain multiple sources — defeating the value.

### DI-013: PPTX Output Must Pass Multi-Renderer Fidelity Check

PPTX output must render correctly in PowerPoint (Office), Keynote, Google Slides, and
LibreOffice. No broken layouts, missing content, or corrupt files across these four renderers.

**Business invariant because:** enterprise presentation workflows include all four renderers.
"Works in PowerPoint" but breaks in LibreOffice is not production-grade (CLAUDE.md quality bar,
multi-renderer parity row).

### DI-014: PDF Output Must Be Tagged PDF (PDF/UA-1 Compliant)

The PDF exporter must produce tagged PDF with real text, structure tags, and accessibility
metadata. Rasterizing slides to images is not acceptable — it would break text search, copy/paste,
screen readers, and accessibility compliance.

**Business invariant because:** veraPDF is in the CI quality bar (CLAUDE.md). Tagged PDF is
required for PDF/UA-1 compliance, which domain-research.md §2.4 identified as a gap in R1-R14.

---

## Brand Invariants

### DI-015: Brand Palette Must Cover All 12 OOXML Scheme Color Slots

A synthesized or extracted brand must populate all 12 standard OOXML theme color slots
(dk1, lt1, dk2, lt2, acc1-acc6, hlink, folHlink). A brand missing any slot will produce
OOXML schema errors in some renderers.

**Business invariant because:** element ordering and completeness in OOXML are schema-significant
(R4 research, brand-template-patterns.md R2). Partial palettes cause silent color substitution
in LibreOffice and Keynote.

### DI-016: Single Master Architecture in v1.0

All slides in a deck share a single slide master. `brand_overlay:` swaps placeholder content
(logo, footer text) but does not switch masters. Multi-master is deferred to v2.

**Business invariant because:** multi-master degrades in Google Slides (Q4 decision). Single
master is the only cross-renderer-safe architecture in v1.0.

---

## Validation Invariants

### DI-017: Strict Mode Produces No Output on Validation Error

When a deck has validation errors (not just lint warnings) and `--warn-only` is not specified,
the build produces NO output files. Partial output from an invalid deck is never written to disk.

**Business invariant because:** a partially rendered deck with unknown validation errors could
be mistakenly shared as if it were correct. All-or-nothing output preserves document integrity.

### DI-018: Error Accumulation in One Pass

The parser and validator must accumulate ALL errors in a single pass. The first error must
not terminate processing (unless a parse error prevents further meaningful parsing). Users
must see all errors from a build, not fix one and discover the next.

**Business invariant because:** iterative fix-one-error-at-a-time development cycles are
identified as a major pain point in competitor tools (R3 research). Error accumulation is
the stated chumsky design goal (Q23).

---

## Package / Workspace Invariants

### DI-019: sf.lock Must Be Committed to Version Control

The `sf.lock` lockfile is not optional. Projects with declared dependencies that lack
a committed lockfile produce a build warning. The lockfile ensures reproducible builds
across machines and over time.

**Business invariant because:** version-controllable deterministic output is a stated product
value (product-brief.md §0, data-reactive use cases). Non-reproducible builds violate this.

### DI-020: Merge Semantics Are Not Configurable

The 11-level precedence chain and the merge rules (last-wins scalars, replace lists, deep-merge
maps) are fixed. No per-field merge-strategy option exists. The behavior is documented,
consistent, and predictable.

**Business invariant because:** configurable merge semantics create unbounded complexity and
user-support burden. Q25 explicitly locked this as "one rule, documented, enforced."

---

## DSL Surface Invariants

### DI-021: Reserved Keywords Must Be Rejected with Descriptive Errors

All ~30 reserved keywords (component, extends, raw, @fn, @mixin, etc.) must be rejected
at parse time with a message that names the planned v2+ feature. Users must never silently
have reserved keywords interpreted as something else.

**Business invariant because:** silent misinterpretation of reserved words would produce wrong
slide content without error — a data-integrity failure. Q22 locks the full reserved keyword list.

### DI-022: Variant Inheritance Graph Must Be Acyclic

Variant `inherits:` declarations must form a directed acyclic graph. A cycle in variant
inheritance (A inherits B inherits A) is a compile error with the cycle shown.

**Business invariant because:** cyclic variant inheritance produces undefined merge results
and potential infinite evaluation loops. Q11 restricts to flat namespace with no nested
inheritance chains.
