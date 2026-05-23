---
title: DSL Design Questions — Proposed Defaults
date: 2026-05-23
analyst: business-analyst
status: AWAITING_HUMAN_REVIEW
sources:
  - .factory/planning/python-reference-deep-read.md
  - .factory/planning/brand-template-patterns.md
  - .factory/planning/dsl-competitor-analysis.md
  - .factory/planning/ooxml-foundations.md
  - .factory/planning/wcag-for-slides.md
  - .factory/planning/ir-prior-art.md
  - .factory/planning/composition-prior-art.md
question_count: 25
---

# DSL Design Questions — Proposed Defaults

## How This Document Works

Each question has:
- **Context**: Why this matters and what research informs it
- **Proposed default**: The analyst's recommendation (research-informed)
- **Rationale**: Why this default, citing specific research findings
- **Alternatives considered**: What else was on the table
- **Human decision**: (blank — to be filled by human)

Questions are organized into tiers:
- **Tier 1 (foundational)**: Affects parser/IR/exporter architecture. Must be decided before Phase 1 architecture.
- **Tier 2 (design)**: Affects DSL ergonomics and feature set. Should be decided before Phase 2 stories.
- **Tier 3 (polish)**: Can be deferred to Phase 3-4 without blocking.

---

## Tier 1 — Foundational (decide before Phase 1 architecture)

### Q1: Does the DSL have computation?

**Context**: R3 surveyed nine competitors on variables/loops/conditionals. Slidev's `{{ }}` Vue template syntax collides with LaTeX math, producing "unclear errors" (Slidev #2245, Aug 2025). Marp has no variable syntax at all, which is its single biggest driver of user abandonment toward Slidev. Quarto supports full R/Python code execution — which is powerful but makes the DSL a programming environment, not a presentation tool. R7 (composition prior art) found that Atmos-style Go-template `{{ if }}` / `{{ range }}` is a famously confusing double-pass footgun. All 15 ecosystems surveyed agreed that simple string substitution (`{{ var }}`) is the most-requested lightweight feature. None recommended full Turing-complete computation in a slide DSL.

**Proposed default**: Simple string interpolation ONLY in v1.0. Syntax `{{ var_name }}` (double-brace). Scope: deck-level `vars:` block plus per-variant `vars:` overrides. No conditionals, no loops, no function calls, no expressions. Undefined variables fail the build loudly with line+column diagnostic.

**Rationale**: R3 shows variables are the #1 most-requested DSL feature across competitors (cited as "please let me do `${client_name}` in the title"). R7 proves that going further (conditionals, loops) is a footgun — Atmos's community still files regressions on Go-template double-pass behavior. The product brief's driving use case is "Python analyst generates decks from data" — that data binding happens OUTSIDE the DSL (Python builds the .sf string). The DSL's own computation responsibility is boilerplate substitution only.

Note on syntax: `{{ var_name }}` chosen deliberately over `${}` (shell/JavaScript collision), `@var:name` (R7 alternative — verbose), or `{:var}` (obscure). See Q9 for inline-interpolation collision rules.

**Alternatives considered**: Full Typst-style function system (too complex for v1 target users); Jinja2-compatible syntax (collision with Python template usage in calling code); no variables at all (leaves client-name-in-every-title as painful manual work).

**Human decision**: ___

---

### Q2: Custom / composed slide types?

**Context**: (Extends locked decision Q2/Q3 on includes — this question is about whether users can DEFINE new types, not just include files.) R7 found that competitors' extension systems are rarely used and create maintenance burden. The Atmos study found that user-defined components dilute the opinionated stance that is the product's primary value. R1 confirmed the Python reference's 23 slide types cover 11 distinct patterns in a real leadership deck. R3 found Marp users LEAVE because of Marp's limited type set — they don't ask for "let me define new types", they ask "please add columns/grids as first-class types."

**Proposed default**: No user-defined slide types in v1.0. The 23 opinionated types ARE the component library. Reserve `component`, `extends`, and `inherits` as keywords (parser recognizes, rejects with "reserved for v2" diagnostic). Reconsider in v2 if 3+ enterprise customers demonstrate a real compound-slide-pattern need.

**Rationale**: R7 explicitly: "the 23 types ARE the component library; user-defined components dilute the opinionated stance." R3 confirms competitor extension systems are underused — users ask for better built-in types, not extension APIs. Every type added to the core is better than every type left to user configuration.

**Alternatives considered**: Sass-style `@mixin name(args)` + `@include` instantiation (syntax is right but creates fragmentation — company A's `incident-summary` mixin vs company B's); parametric slide types via `set` rules (achieves 80% of the use case without user-defined types — see Q10).

**Human decision**: ___

---

### Q3: Data binding mechanism

**Context**: R3 found that "data binding — pulling tables/charts from external data files at compile time" is the #2 most-requested feature across competitors (after columns/grids). python-pptx + pandas is the standard workaround; R1 confirmed the Python reference uses inline Python dicts as the data model. R3 also warns against `$...$` (LaTeX/Marp/markdown collision) and `{{ }}` being overloaded for both variable substitution AND data references.

Note: Locked decision Q3 covers `@include "path.sf"` for slide fragments. This question covers data binding for CONTENT inside slides (e.g., loading a table from CSV, binding a metric value from JSON).

**Proposed default**: No compile-time data binding in v1.0. Deck content is fully inline in the .sf file. The canonical pattern is: Python/script generates the .sf string with data already interpolated (using `{{ var }}` for simple strings from the `vars:` block), then shells out to `slideforge build`. Provide `@data` as a RESERVED directive keyword that the parser recognizes but rejects with "data binding planned for v2" — this prevents users from inventing incompatible workarounds.

**Rationale**: R3 verdict: "consider `@chart:from=data.csv` directive for v2." The Python reference's own pipeline (Python generates JSON data → Python builder code creates the slide JSON → python-pptx renders) demonstrates that data binding is naturally an OUTER layer concern. Competing tools that do compile-time data binding (Quarto via R/Python execution, Slidev via Vue props) either require a full programming language runtime or add significant complexity. A Rust binary that also exec()s Python to load data is a security surface and complexity amplifier.

**Alternatives considered**: `@data "file.csv" as rows` directive (R7 Q18 sketch — useful, but ties the DSL to a runtime data-loading concern that belongs in the calling layer); `${metric.value}` notation for variable interpolation into slide fields (covered by Q9's `{{ var }}` syntax for simple string substitution).

**Human decision**: ___

---

### Q4: Per-slide vs. deck-level template binding

**Context**: R2 found that `<p:clrMapOvr>` allows a layout to override color semantics per-layout (dark section dividers, end slides). FULL per-slide template switching (different brand entirely mid-deck) would require multiple slide masters and is OOXML-hard to synthesize correctly. R1 shows the Python reference uses exactly two layout families (blue divider, purple divider) — not arbitrary per-slide templates. R2's recommendation is to generate 11 standard layouts + ~20 custom layouts from a SINGLE master/theme.

**Proposed default**: Template binding is deck-level only in v1.0. One brand per deck. Color-variant layouts (dark dividers, end slides) are handled via `clrMapOvr` at the layout level within the single template — they are baked into the brand template, not selected per-slide by the DSL author. Per-slide `color:` hints (e.g., `title: color: purple`) map to pre-defined layout variants within the template, not to a different brand.

**Rationale**: R2: "full template-switch mid-deck is OOXML-hard." The `clrMapOvr` mechanism (already used by the Python reference for blue/purple dividers and end slides) provides the right level of per-slide customization within a single brand. R1 confirms the Python reference's actual production use: two color variants of the divider layout, period. Multi-brand decks are the v2 use case (different master per variant — maps to R7's variant system).

**Alternatives considered**: `template:` field on each slide block (would require multiple slide masters per deck, full `sldMasterIdLst` with N entries, expensive to synthesize correctly); brand-per-variant (supported via R7's variant system where each variant specifies `brand: acme-exec` — this is deck-level binding per variant, not per-slide).

**Human decision**: ___

---

### Q5: Output target declaration

**Context**: Locked decision Q5 confirmed all three exporters (PPTX + PDF + HTML) ship in v1.0. This question asks: does the `.sf` file DECLARE which outputs it targets, or is that entirely CLI-driven?

**Proposed default**: CLI-driven. The `.sf` file does NOT declare output targets. `slideforge build deck.sf` builds all three by default. `slideforge build deck.sf --pptx-only` or `--pdf-only` restricts. A deck-level `outputs:` field is RESERVED but not parsed in v1.0 (parser accepts, emits deprecation warning: "outputs field is planned for v2 — use CLI flags").

**Rationale**: R3 found that "per-format style overrides without duplicating content" is a top-3 Quarto user request. The RIGHT place to express "I only want HTML today" is the CLI invocation, not baked into the source file — the same .sf file should be renderable to all formats without editing. R3's competitor analysis confirms Quarto users regularly complain that pptx and revealjs diverge because format intent is baked into the source.

**Alternatives considered**: `outputs: [pptx, pdf]` in deck frontmatter (locks the file to those formats — reduces portability); format-specific blocks in DSL (Quarto's approach — creates duplication; Quarto's own users complain about this).

**Human decision**: ___

---

### Q6: `alt "..."` and accessibility DSL surface

**Context**: R5 (WCAG) found 23 WCAG criteria that apply to slideforge's four surfaces. The single most impactful DSL-level requirement: `alt` text on non-decorative visual elements is mandatory for WCAG 1.1.1 (Non-text Content), PDF/UA-1 `/Alt` on `<Figure>` tags, PPTX `cNvPr@descr`, and HTML `<img alt>`. R5 explicitly: "without this, the DSL cannot ship a11y-compliant decks." R3 found Reveal.js has had an open accessibility issue (#3637) for 18+ months because alt text is not first-class. ADR-011 (tooling) is locked per decisions-applied to `@axe-core/playwright` — that tooling requires alt text to be in the HTML output, which means it must flow through the IR from the DSL.

**Proposed default**: `alt "..."` is a first-class DSL field on every image/icon/chart/diagram block. It is REQUIRED (compile error if absent) unless `decorative: true` is set (which emits empty alt + PDF Artifact tag). Top-level deck field `lang: "en-US"` (default `"en"`) propagates to HTML root `<html lang>`, PPTX run-level `rPr/@lang`, and PDF `/Lang`. Color-coded slide types (`severity_cards`, `status`, `progress_bar`, `weighted_composite`, `formula`) require a `label:` field co-encoding meaning in text alongside color — lint hard error if absent.

**Rationale**: R5 NFRs a11y-NFR-1 through a11y-NFR-10 are non-negotiable per the quality bar. R2 found that the PPTX Accessibility Checker flags missing alt text for EVERY slide in a deck if the slide master logo lacks alt. R3 found Marp's SVG viewport-scale breaks Ctrl-+ zoom (Marp #636) — a WCAG 2.1.1 failure that slideforge avoids by design. Accessibility cannot be added as an afterthought to a DSL — it must be structural.

**Alternatives considered**: Optional `alt:` field with lint warning (R5 explicitly argues against this — the compile error is the differentiator); separate accessibility pass at export time (fails because alt text is semantic, not geometric — it must be in the IR before layout, not injected by the PPTX emitter alone).

**Human decision**: ___

---

### Q7: IR stability contract — RawBlock escape hatch

**Context**: R6 recommends a `Raw { exporter: ExporterId, content: String }` variant on both `Slide` and `Inline` IR types (mirroring Pandoc's `RawBlock Format Text` + `RawInline Format Text`). This allows PPTX-specific OOXML, HTML-specific markup, or PDF-specific ops to be passed through opaquely. The question is: should the DSL EXPOSE this to users, or is it IR-internal?

Locked decision Q7 covers the live preview architecture. This question is distinct: it is about whether a power-user can write `raw pptx: "..."` in a .sf file.

**Proposed default**: IR-internal only in v1.0. The `Raw` IR variant exists for internal exporter use (e.g., the PPTX emitter generating a custom XML island for a complex shape) but is NOT exposed as a DSL keyword. Reserve `raw` as a keyword (parser rejects with "raw escape hatch planned for v2"). Rationale: exposing raw OOXML to users is a footgun — it breaks PDF/HTML export (they see a `Raw { exporter: "pptx", content: "..." }` node and must skip it, silently dropping content). It also breaks the WCAG accessibility contract (raw OOXML bypasses the a11y lint).

**Rationale**: R6: "Plan the escape hatch from day one" — the IR needs `Raw`, but R3's competitor analysis shows that every tool with an HTML escape hatch ("just use `<style scoped>`") sees it become the PRIMARY usage pattern for complex layouts, undermining the DSL's opinionated value. The 23 opinionated types exist precisely to prevent "raw OOXML in the .sf file."

**Alternatives considered**: `raw pptx: |` block in DSL for power users (valid v2 feature for custom XML islands, but v1.0 has no use case that can't be handled by the 23 slide types); IR-only with no DSL keyword reserved (not reserving it lets users write `raw:` as a field name on slides, causing future syntax collisions).

**Human decision**: ___

---

## Tier 2 — Design (decide before Phase 2 stories)

### Q8: Inline formatting surface

**Context**: R1 found the Python reference uses `**bold**` string prefix notation for headers inside bullet lists — a fragile pattern where `str.strip("*")` strips leading/trailing asterisks. R3 found math-in-markdown collision risk: Marp uses `$...$` for math (LaTeX-style), which collides with shell variables and CSS custom properties in user content. R6's IR sketch defines `Inline::Bold`, `Inline::Italic`, `Inline::Underline`, `Inline::Code`, `Inline::Link`, `Inline::Math` as the confirmed set.

**Proposed default**: Markdown-inspired inline syntax for v1.0: `**bold**`, `_italic_` (underscore only, not `*italic*` to avoid ambiguity with bold), `` `code` `` (backtick code spans). Links: `[text](url)`. Math: `$math$` is explicitly NOT supported in v1.0 — reserve `$` as a reserved character that emits "math support planned for v2" — this avoids the LaTeX/shell collision. Superscript/subscript: `^sup^` / `~sub~` reserved but not implemented. Emoji by Unicode literal (no `:shortcode:` syntax). No footnotes in v1.0.

**Rationale**: R1 confirms the Python reference's inline needs are: bold headers in bullets, plain bullets, colored text via status fields. No math in the reference deck at all. R3: "`$...$` collision with LaTeX/Marp/markdown" is a known ergonomics trap. Restricting to the confirmed set (bold/italic/code/link) and reserving math for v2 avoids the #5 ergonomics trap from R3 (inline interpolation collision).

**Alternatives considered**: Full CommonMark inline set (overkill for corporate slide decks; brings in footnotes, HTML inline, hard-coded link reference definitions that slides don't need); Typst-style function calls for formatting (`#bold[text]`) (harder to type, unfamiliar for the target user who likely knows Markdown).

**Human decision**: ___

---

### Q9: Variable interpolation syntax

**Context**: R3 explicitly lists the three traps: `{{ }}` (Vue/Slidev collision with LaTeX), `$...$` (LaTeX/Marp), `<%...%>` (ERB). R7 recommends `@var:name` or `${var}` with strict scoping. Q1 above proposed `{{ var_name }}` as the interpolation syntax.

**Proposed default**: `{{ var_name }}` double-brace syntax for variable interpolation (decided in Q1). Additional rules: (a) `{{` outside a string-valued field is a hard parse error; (b) nested braces `{{{ }}}` are a parse error; (c) literal `{{` in text is escaped as `\{{`; (d) variable names are restricted to `[a-z][a-z0-9_]*` (snake_case identifiers only — prevents ambiguity with math-like expressions); (e) interpolation is only valid in string fields (title, subtitle, bullet text, notes, label fields) — it is NOT valid in numeric fields, color fields, or structural fields.

**Rationale**: R3: `{{ }}` is Vue-specific and only collides if the user writes Vue components inside .sf files, which is impossible by design. The real collision risk is `$...$` (blocked by Q8 decision to not support math). `{{ }}` is familiar to anyone who has used Jinja2, Ansible, or Helm — the target user (Python analyst) almost certainly knows it. Restricting to snake_case variable names eliminates expression-language creep.

**Alternatives considered**: `@{var_name}` (less familiar, harder to type); `${var}` (shell collision in strings passed via CLI args — common in CI/CD scripts); `@var:name` (verbose, unusual for the target audience).

**Human decision**: ___

---

### Q10: `set` rules for deck-level type defaults

**Context**: R7's strongest recommendation — directly ported from Typst's `set` rule pattern. Typst: "`set` rules establish defaults for an element type within a scope." The slide use case: a deck with 8 `severity_cards` slides should not repeat `color_high: red` on every one. R7 found this is the cleanest solution to the "copy-paste boilerplate per slide" problem without inventing mixins.

**Proposed default**: Yes, implement `set` rules in v1.0. Syntax within the `deck:` block:
```
deck:
  set severity_cards:
    color_high: red
    color_medium: orange
    color_low: green
```
Each slide of that type inherits the `set` defaults unless the slide explicitly overrides a field. Precedence: slide-level field wins over `set` default wins over brand-template default. `set` rules are lexically scoped to the deck block (no cross-file `set` in v1.0 — that would require cascade semantics like CSS; defer to v2).

**Rationale**: R7: "The `set` rule is the closest analog to what slideforge actually needs." This costs one new keyword and one new syntax rule in the parser, but eliminates the #1 boilerplate complaint for power users (Tailwind's flat-specificity guarantee applied to slide defaults). R3: Marp/Slidev's branding pain comes directly from having no equivalent — "minor brand variations require entire new CSS files." `set` rules at deck level are the lightweight fix.

**Alternatives considered**: Slide-type-specific config files in `defaults/` directory (see Q21 — that is the organization-wide variant of this; `set` rules serve the per-deck use case); mixins (heavier syntax, v2 candidate per Q2/Q24).

**Human decision**: ___

---

### Q11: Variants — one source, multiple output decks

**Context**: R7 identified "one source, exec vs. tech deck" as the flagship variant use case. R3 found no competitor handles this well — Quarto has `params:` in frontmatter for simple substitution, but cross-format fidelity breaks (revealjs→pptx is famously lossy). The Python reference has no variant concept — the analyst copy-pastes and manually edits, which is the exact problem to solve.

**Proposed default**: Yes, implement lightweight variants in v1.0. In-deck `variants:` block with `include_tags:` / `exclude_tags:` filters and per-variant `vars:` overrides. Each slide block can carry `tags: [list]`. Build invocation: `slideforge build deck.sf --variant exec-external`. If no `--variant` flag, all slides are included. Precedence: slide-level vars > variant vars > deck-level vars. List replacement (no append). No directory-hierarchy stacks in v1.0.

**Rationale**: R7: "Without variants, users will copy-paste decks and lose the single-source guarantee." R3: Quarto's params approach is one-dimensional (no slide-level tag filtering). The tag-filter model (include/exclude by tag) is simpler than Quarto's conditional blocks and avoids embedding conditional logic in slide definitions. Cost is a small parser feature + a pre-layout filtering step in the compiler.

**Alternatives considered**: Directory-hierarchy stacks (Atmos-style — R7 explicitly recommends against for v1.0: "maps to a much simpler problem domain"); `@if audience == "exec"` conditional blocks in slides (control flow in DSL — see Q1 decision against computation); multiple separate .sf files with `@include` (loses single-source guarantee — the exec deck and tech deck diverge independently).

**Human decision**: ___

---

### Q12: `@include` semantics depth

**Context**: Locked decision Q3 confirmed `@include "path.sf"` directives. This question is about WHAT can be included: slides only? Fragments (partial slides)? Shared `set` definitions? Data? R7 found Hugo's partials + Typst's imports are the most ergonomic. R3 found Slidev #2241 "imported snippets don't hot-reload automatically" — slideforge must fix this. R3 also confirmed that `@include` must resolve via documented paths (relative to source file, then `--include-path` flags).

**Proposed default**: `@include` includes one or more complete SLIDES (slide blocks with their full type + fields). It does NOT include fragments (partial slide blocks), raw data files, or shared `set` definitions (those go in the deck block, not an include file). Resolution: relative to the source file first, then `--include-path`. Cycle detection is required (A includes B includes A → compile error with the cycle shown). Hot-reload in `slideforge watch`: any change to an included file triggers a full re-parse. `catalog/` and `mixins/` are RESERVED directory names as conventions (see Q21/Q24).

**Rationale**: Fragment includes (partial slide blocks) require the parser to handle "open" slide blocks that span file boundaries — complex to implement and to understand. Complete slide includes (`@include "standard_disclaimer_slide.sf"`) are unambiguous. R7: "full slide include is the 80% use case and avoids the variable-scope-leakage problems of fragment includes." R3: hot-reload is a table-stakes requirement vs. Slidev (which broke hot-reload in 2025).

**Alternatives considered**: Fragment includes for shared boilerplate like `notes:` blocks (legitimate use case, but deferred to v2 — the workaround is copy-paste the notes block, which is acceptable for v1); `@include data.csv` for data binding (blocked by Q3 decision).

**Human decision**: ___

---

### Q13: Block comments / doc-comments

**Context**: R3: "Pick one [comment style] and enforce. Don't permit multiple comment styles." The Python reference uses Python's `#` single-line comments. YAML uses `#`. The indentation-significant DSL (locked Q2) is closest to Python/YAML in syntax model.

**Proposed default**: `#` line comments only in v1.0. No block comments (`/* */`, `""" """`). No doc-comments that flow into output. Rationale: single consistent comment style eliminates "which comment syntax is active here?" confusion. `"""` block comments are reserved as a keyword (parser rejects with "block comment syntax planned for v2"). NOTE: if the `#` character needs to appear in slide content (e.g., a slide about git or shell commands), it must appear as the text value of a field — `title: "Use # for shell comments"` — not as standalone content. The parser only interprets `#` as a comment at the START of a line (after optional whitespace), not inside string values.

**Rationale**: R3's ergonomics trap #3: "tab vs. space indentation surprises" — the same principle applies to comment syntax. One style, documented, enforced. R2 shows the OOXML/brand domain is all-XML where `<!-- -->` comments don't exist in the DSL surface. R7 shows all surveyed config languages that allowed multiple comment styles (YAML's `#` + TOML's `#` being the same, but tools like Jsonnet having `//`, `#`, and `/* */`) created documentation fragmentation.

**Alternatives considered**: `//` line comments (C-style, familiar to Rust/Java users, but odd for an indentation-significant DSL); `#` + `"""..."""` block comments (block comments are useful for temporarily commenting out slides — but the correct tool is `# skip: true` on a slide block or tag-based variant filtering, not block comments).

**Human decision**: ___

---

### Q14: Conditional rendering

**Context**: R7 found "show slide X only if condition Y" is partially covered by the variants tag-filter system (Q11). R3: "conditional rendering — `show this slide only for audience=execs`" is the canonical use case. R3 verdict: "likely non-goal for v1 — tag-based slide filtering at compile time is a simpler win."

**Proposed default**: No conditional rendering in v1.0 beyond tag-based variant filtering (Q11). Tags are the condition: `tags: [exec]` on a slide + `--variant exec-external` on the build command is the conditional rendering mechanism. Reserve `if:`, `when:`, and `unless:` as keywords (parser rejects with "conditional rendering via @if planned for v2 — use tags and variants for v1").

**Rationale**: R3: "explicit non-goal for v1 — keep DSL declarative." R7: the variant system already covers the primary use case. Conditionals in a DSL are a slippery slope toward a template language (the Python-as-DSL regression we're designing away from). Tag-based filtering is declarative (the slide exists; the variant decides whether to include it) — semantically cleaner than `@if audience == "exec"` which is procedural.

**Alternatives considered**: `@if variant == "exec"` directive on slide blocks (functional, but bleeds variant logic into slide definitions — the tag model keeps variant logic in the `variants:` section); feature flags via deck-level `vars:` + interpolation (doesn't help with slide-inclusion decisions, only field-value substitution).

**Human decision**: ___

---

### Q15: Asset references

**Context**: R1 found the Python reference uses `add_picture()` with local filesystem paths — `add_picture(slide, 'logo.png', ...)`. R3: "python-pptx's `add_picture()` requires local filesystem path — no URLs." R2 confirmed that image content type registration is required in `[Content_Types].xml` for PPTX. R4 confirmed image path handling: `ppt/media/imageN.png` convention. R3: "external images (r:link with TargetMode=External) are strongly discouraged — Keynote, Google Slides, LibreOffice frequently fail to fetch."

**Proposed default**: Relative filesystem paths only for image/asset references. Syntax: `image: "assets/logo.png"` or `logo: "logo.png"`. Path resolution: relative to the .sf source file. No URL-based assets in v1.0. No `@asset "logo.png" as company_logo` named assets in v1.0 (reserve the syntax). Asset references in `@include`-d files resolve relative to the INCLUDED file's directory, not the root deck.

**Rationale**: R4: external image links break in 3 of 4 renderers (Keynote, Google Slides, LibreOffice). R3: python-pptx's URL limitation is a known pain point but the CORRECT fix is pre-fetching assets at build time, not runtime HTTP in the renderer. The Python reference's production usage confirms local paths are sufficient for the primary use case.

**Alternatives considered**: `asset:` block with name binding (`@asset "logo.png" as logo`) for deck-global asset registry (useful for shared logos in multi-slide decks — deferred to v2 since `@include` can reference a slide that has the asset); URL support with at-build-time fetch (adds network dependency to build — R3 documents how Pandoc's image embedding regressions caused by HTTP fetching broke user workflows).

**Human decision**: ___

---

## Tier 3 — Polish (can defer to Phase 3-4)

### Q16: DSL versioning

**Context**: R3 found MDX-deck v2 broke decks because of `---` separator changes without a versioning scheme. R7: "the `.sf` file should be a self-describing artifact — frontmatter declares grammar version." R3's lock-in lessons: make source files self-describing so migration is detectable.

**Proposed default**: Deck-level `slideforge_version: "1.0"` field in the deck metadata block. Required in v1.0. Parser emits a hard error if `slideforge_version` is absent (guidance: "add `slideforge_version: \"1.0\"` as the first field of your deck block"). When a v2 parser reads a v1.0 file, it emits a migration warning listing breaking changes. When a v1.0 parser reads a future-version file, it emits an error ("this file requires slideforge v2.0 or newer").

**Rationale**: R3: MDX-deck v2's separator change broke thousands of decks because files had no version marker. MDX-deck v4's deprecations similarly had no forward-incompatibility warning. Making `slideforge_version` required from day 1 costs one line per deck and provides forward-compatibility signaling for free. R7 recommends this explicitly.

**Alternatives considered**: Version in a `slideforge.toml` project file only (doesn't work for single-file decks or `@include`-d fragment files that float independently); SemVer in the file path convention (fragile, not self-describing).

**Human decision**: ___

---

### Q17: Strict vs. forgiving parsing modes

**Context**: R3: "No validation, no overflow warnings, no canvas-bounds checks until rendered" is the #4 universal complaint across ALL competitors. Typst "warns on layout non-convergence." The Python reference only warns (never blocks). R3: "the Rust implementation should error or warn loudly" (for color typos, missing fields, overflow, etc.).

**Proposed default**: Default mode is STRICT. `slideforge build` fails on any validation error. `slideforge build --warn-only` prints all validation errors as warnings and continues to produce output (for iterative editing). `slideforge watch` runs in warn-only mode by default (hard errors would stop the preview loop). The distinction:
- Parse errors (syntax) → always fatal, both modes
- Validation errors (canvas overflow, missing alt text, color contrast failure) → fatal in strict mode, warnings in warn-only
- Lint warnings (bullet count, consecutive content slides) → warnings only, both modes

**Rationale**: R3: slideforge differentiating on "compile error tells you about canvas overflow before you open PowerPoint" is the most-cited unmet need. Making strict mode the DEFAULT enforces the quality bar without users having to opt in. The Python reference's warning-only validator ("Never blocks the build" per R1) is explicitly listed in R1 as a behavior "NOT to carry forward." R3: Touying warns on non-convergence; slideforge should go further.

**Alternatives considered**: Warn-only as default (familiar from python-pptx; but this is a step backward from the quality bar and would normalize the "looks fine in preview, broken in export" cycle); separate `slideforge lint` command (useful addition, but doesn't substitute for build-time validation since many users won't run it).

**Human decision**: ___

---

### Q18: Internationalization / RTL

**Context**: R5 confirmed WCAG 3.1.1 (Language of Page) as a required criterion — `lang: "en-US"` in deck metadata propagates to HTML `<html lang>`, PPTX `<a:rPr lang="en-US"/>`, PDF `/Lang`. R3: Slidev has an open BiDi (right-to-left) issue; no major competitor handles RTL well. R2 confirmed the OOXML font scheme supports CJK/RTL fallback fonts via script-specific overrides in `<a:majorFont>`.

**Proposed default**: `lang: "en-US"` as a required top-level deck field (default value if absent: `"en"`). Page-level language only in v1.0 — no per-run language tagging. RTL text direction: `dir: rtl` on deck or slide block is a RESERVED field (parser accepts, emits: "RTL layout is planned for v2 — slides will render LTR in v1.0"). CJK fonts: the brand TOML can declare `[fonts] cjk = "Yu Gothic"` which populates the script-specific overrides in theme XML. No full i18n localization of multiple-language decks in v1.0.

**Rationale**: R5 NFR a11y-NFR-7: "all emitted output MUST carry a language tag." `lang:` is mandatory, just not defaultless. RTL is explicitly flagged as a v2 candidate in R3 (Slidev open issue, no major competitor handles it). CJK font declaration in brand TOML costs one config field and correctly encodes the font scheme — zero DSL complexity.

**Alternatives considered**: Omitting `lang:` entirely and hardcoding `en-US` (fails WCAG 3.1.1 for non-English decks); per-run `{:lang "fr"}text{/}` inline annotation (R5 explicitly defers this to post-v1.0; page-level is sufficient for v1.0 conformance).

**Human decision**: ___

---

### Q19: Library / package model

**Context**: R7 studied 15 ecosystems and found package registries add significant ecosystem ramifications. R3: "Marp's `no Node, just CLI binary` is repeatedly cited as why people stay with Marp despite limitations." R3: "A teacher with 200 lectures shouldn't need 200 node_modules folders" (Slidev #63). The product brief targets "Python analysts who build decks programmatically" — not framework ecosystem builders.

**Proposed default**: No package registry or installable slide kits in v1.0. `@include` from local filesystem paths is the reuse mechanism. Organizations build their own `catalog/` directory convention (Q24). Reserve `@package`, `@import from`, and `@registry` as keywords. Revisit in v2 only if 5+ users explicitly request it with concrete use cases.

**Rationale**: R7: "Git-based `@include` from local paths is enough. A registry is a v2+ feature with significant ecosystem ramifications." R3: the #5 universal pain point across competitors is HEAVY INSTALL FOOTPRINT — a package registry makes this worse. The product's single-binary story (R3: "slideforge ships as a single binary or wheel") is incompatible with a per-deck registry-pull model.

**Alternatives considered**: Git-URL `@include` syntax (`@include "github.com/acme/slides/catalog/disclaimer.sf"`) — useful, but adds network dependency and authentication complexity to the build; deferred to v2 as a defined extension point.

**Human decision**: ___

---

### Q20: Hierarchical project config

**Context**: R7 explicitly recommends against `slideforge.toml` per-directory with hierarchical merge for v1.0. R7: "Hugo lookup-order is notoriously hard to predict; the doc has long flowcharts. Variable scope leaks between includes are common." Atmos's own community regularly trips on multi-level config merge regressions.

**Proposed default**: Single `slideforge.toml` at the project root only. No per-directory config files. No config merging or inheritance. The `slideforge.toml` declares: `include_path`, `default_brand`, `output_dir`, `slideforge_version`. Deck-level configuration is in the `deck:` block of each `.sf` file. R7's verdict: "Single root config. Re-evaluate in v2 if monorepo-style deck collections become a real use case."

**Rationale**: R7 found this directly — "No hierarchical config for v1.0." R3: config-language confusions (YAML tab/coerce gotchas) are amplified in hierarchical configs where the user can't tell which level a value came from. The product brief's primary user (Python analyst with a single deck per script run) has no use for directory-hierarchy config.

**Alternatives considered**: Per-deck `slideforge.toml` sidecar (solves the "I want different output dirs per deck" use case — but `slideforge build deck.sf --output-dir ./out/` CLI flags do the same without a config file proliferation).

**Human decision**: ___

---

### Q21: Defaults directory convention

**Context**: R7 found Hugo's `_default` cascade ("files in `_default/` apply globally unless explicitly overridden") is one of the best composition patterns across 15 ecosystems. The slideforge analog: a `defaults/` directory at the deck root holds per-slide-type default configs. The engine auto-applies them as `set` rules (Q10) without the author writing them in the deck block.

**Proposed default**: Yes — adopt `defaults/` directory convention with zero new syntax cost. If `defaults/severity_cards.sf` exists at the deck root (or in any `--include-path` directory), its content is treated as a `set severity_cards:` block applied to the deck. File format: a single `set <type>:` block per file, same syntax as Q10. Precedence: deck-level `set` block wins over `defaults/` file. This is a CONVENTION documented in the slideforge user guide, not a new DSL keyword.

**Rationale**: R7: "Files in `defaults/<slide-type>.sf` are auto-applied as `set` blocks. Costs nothing; benefits organizations." This gives enterprise users (with company-wide severity badge colors, standard takeaway text, standard footer wording) a clean mechanism to distribute organization defaults without requiring all deck authors to copy-paste `set` blocks. Zero engine cost beyond a directory scan during project init.

**Alternatives considered**: `import defaults` directive in slideforge.toml (explicit but adds config surface area); command-line `--defaults-dir ./shared/defaults/` flag (useful addition on top of the convention, but not a replacement — the convention is cheaper).

**Human decision**: ___

---

### Q22: Reserved keyword expansion

**Context**: R3 found that built-in TOC generation is a top-5 cross-tool feature request (Reveal.js, Marp, Quarto all missing). Mermaid/diagrams is a top-3 request. R1's 23 slide types do not include `toc`, `agenda`, `chart`, `diagram`, or `mermaid`. R3: "Mermaid/diagrams as native slide type. Even if it costs install footprint, this is a top-3 cross-tool request."

**Proposed default**: Reserve the following slide type keywords for future use — parser recognizes them, rejects with "planned for v2" diagnostic: `toc`, `agenda`, `chart`, `diagram`, `mermaid`, `code`, `quote`, `bio`, `team`, `case_study`. Reserve the following directive keywords: `@mixin`, `@component`, `@if`, `@when`, `@unless`, `@for`, `@data`, `@asset`, `@package`, `@registry`. This is a one-time investment in the parser's keyword table to prevent user-defined identifiers from colliding with future built-ins.

**Rationale**: R3: Marp's absence of Mermaid drives users to Slidev. If slideforge ships v1.0 without Mermaid but without reserving `mermaid` as a keyword, users will work around it by naming a slide type `mermaid` via some future extension mechanism — and that workaround will break when v2 adds native Mermaid support. R3 also found TOC is the #4 cross-tool feature request — `toc` and `agenda` should be reserved now.

**Alternatives considered**: Reserve nothing (cleaner v1.0 grammar, but creates forward-compat debt); implement `chart` + `toc` in v1.0 (would delay v1.0 significantly; Phase 3 stories should not block on these).

**Human decision**: ___

---

### Q23: Error recovery semantics

**Context**: R3 found "Marp/Slidev just crash; Typst recovers well." R1: "the Rust implementation should have a validator that blocks" (recommending the Python warning-only behavior NOT be carried forward). R4's IR design (ADR-009) is explicitly listed as an open question: "what does the parser do with malformed slides? Partial AST? Error continuation? Recovery hints?"

**Proposed default**: Error accumulation with partial output inhibition. The parser collects ALL errors in a single pass (no fail-fast on first error — Typst's model) and reports them together with line+column+span diagnostics. On validation errors: if `--warn-only` mode, emit partial output (slides with errors are replaced with an "error slide" placeholder that renders as a red "build error" slide in the output). In strict mode (default), no output is emitted, but ALL errors are reported. Recovery hints are mandatory per error: "slide title is empty — add `title: \"Your title\"`" style messages.

**Rationale**: R3: "Clear error messages at compile-time pointing to line+column beats Repair? [Yes/No] by orders of magnitude." The "accumulate all errors, report together" model (vs. fail-fast) is better developer UX — especially in `slideforge watch` where the user wants to see all their errors at once, not fix one then discover another. R4 (ADR-009) is blocked on this decision.

**Alternatives considered**: Fail-fast on first error (familiar from many compilers, but bad for iterative editing where the user wants to see all 5 errors at once); always emit partial output even in strict mode (produces corrupt PPTX files — defeats the contract of "PPTX export fidelity is a guarantee, not best-effort" from R3).

**Human decision**: ___

---

### Q24: Mixin syntax (v2 candidate)

**Context**: R7 recommends deferring user-defined mixins to v2. R7: "Use `@include` of fragment files + theme/brand config. Reserve `mixin` keyword for v2." The Sass `@mixin name(args)` + `@include name(args)` pattern is the most familiar mixin model for the target user. R7 found GraphQL fragment syntax (`fragment FooterFields on Slide { ... }` + `...footer_fields`) as the cleanest named-fragment model.

**Proposed default**: No mixin syntax in v1.0. `mixin` is a reserved keyword (parser rejects: "mixin syntax planned for v2 — use `@include` of fragment files"). When v2 adds mixins, adopt Sass-inspired syntax adapted to indentation-significant DSL:
```
mixin standard_footer(company_name):
  notes:
    "{{ company_name }} Confidential"
  takeaway: "Questions? Contact {{ company_name }} team"
```
instanced via `@include mixin:standard_footer(company_name="Acme")`. This syntax sketch is NORMATIVE for forward-compatibility planning — the parser's keyword table should reserve `mixin` with this future form in mind.

**Rationale**: R7: "No mixin syntax for v1.0 — `@include` is enough." Recording the intended v2 syntax here prevents the keyword from being used for a different purpose in any interim extension. R7's Sass lesson: `@mixin`/`@include` is textbook reusable fragment syntax; `@extend`-style global cascade is the anti-pattern to avoid.

**Alternatives considered**: Named `@fragment` blocks instead of `mixin` (less familiar to target audience); parameterized slide includes via `@include "frag.sf" with (name=value)` (simpler but less composable for multi-param cases).

**Human decision**: ___

---

### Q25: Merge semantics if/when mixins land

**Context**: R7 found that Atmos's deep-merge is "the recurring footgun" (issue #2376: regression in v1.212.0). Helm's list-merge is "the canonical pain point" (issue #3486 still open after years). R7 recommendation: "pick ONE merge rule and document it ferociously."

**Proposed default**: When mixins/variants/`@include` produce overlapping fields, the merge rule is: **last-wins for scalars, REPLACE (not append) for lists, deep-merge for maps.** This is identical to Atmos's default merge behavior. Explicitly NOT configurable — no `merge_strategy:` per-field option in v1.0 or v2. If a list must be extended, the author re-declares the full list. If a map must be deep-merged, it happens automatically. Scalar overrides replace the parent value.

Document this rule in the user guide with three worked examples showing what happens when: (a) a slide's `items:` list overrides a `set`-default `items:` list (full replacement); (b) a variant's `vars:` map adds keys to the deck's `vars:` map (deep merge — both sets of keys survive); (c) a slide overrides one field of a slide with a `set`-default (last-wins — the slide's value wins).

**Rationale**: R7: "Configurable merge strategies are footguns. Pick ONE rule." Atmos, Helm, and Kustomize all have regression histories traceable to merge-strategy configuration options. R7: "Do NOT add configurable merge strategies in v1.0 — Atmos's own community trips on these regularly." The "replace lists" default is correct for slide content (an `items:` list is the whole list, not a delta to be appended to a default list).

**Alternatives considered**: Append-for-lists default (more intuitive for "add to the default items" case — but leads to accidental list growth when users forget they have a `set` default; replace-is-safer because it's explicit); explicit merge operators like Dhall's `/\` vs `//` (elegant but unknown to the target audience; adds DSL surface area for a rare-in-practice problem).

**Human decision**: ___

---

## Cross-Research Synthesis

### Top-10 DSL design implications from research (aggregated across R1-R7)

1. **Color names must fail loudly.** R1 found `resolve_color()` silently falls back to TEAL for unknown color strings (line 88). The production deck has a color typo → teal cards. The Rust implementation must error on unknown color names. All 11 brand color names must be statically validated at parse time.

2. **Canvas-overflow is a compile error, not a runtime surprise.** R3 found this is the #4 universal pain point. The Python reference has no overflow detection at all (R1: "python-pptx places text past the 13.33" right edge without warning"). slideforge's IR layout pass MUST compute bounding boxes and reject slides that overflow the 13.33" × 7.5" canvas.

3. **Bullet lists are structural, not string-prefixed.** R1 found `"**header**"` string prefixes as the Python reference's bold-header detection — fragile, surprising, R1 explicitly lists it as a quirk NOT to port. The DSL must use structural elements: `header:` for bold headers, `bullet:` for bullet items.

4. **The PPTX package graph is fragile.** R2 and R4 documented 13 pitfalls in from-scratch PPTX generation: strict element ordering in `theme1.xml`, `notesMaster1.xml` expected even if unused, `[Content_Types].xml` must register every part, `standalone="yes"` on every XML declaration. ADR-001 must produce a formal OOXML synthesis specification before any PPTX emitter code is written.

5. **The two-IR split is load-bearing.** R6: `Deck` (semantic, pre-layout) and `LaidOutDeck` (geometric, post-layout). PPTX exporters need BOTH: semantic IR for placeholder types and accessibility tags, geometric IR for non-placeholder shape positions. Mixing them (as python-pptx does) is the root cause of python-pptx's "no semantic structure" problem.

6. **Accessibility is a DSL contract, not an export-time add-on.** R5: "the accessibility bar cannot be satisfied by export-time hacks alone — it must be a first-class concept in the IR." `alt:` on images, `lang:` on the deck, and text-label co-encoding on color-coded slides must be DSL-level fields validated at parse time, not injected by emitters.

7. **Enterprise template compatibility requires 11 standard layouts PLUS custom layouts.** R2: "slideforge cannot fit its visual taxonomy into Microsoft's 11 standard layouts alone — approximately 10-12 custom layouts are needed." ADR-001 must map all 23 slide types to specific layout XML, with `clrMapOvr` for dark-themed variants (dividers, end slides).

8. **Single binary, no runtime dependencies.** R3: Marp's "no Node, just CLI binary" is the primary reason users stay despite Marp's limitations. Slidev's per-project node_modules is the #5 pain point (1 teacher, 200 node_modules folders). slideforge must ship as a single statically-linked binary with no external runtime deps at build time.

9. **Talk-track / speaker notes deserve markdown formatting.** R1: the Python reference uses plain string assignment to notes — "no structured formatting." R3: "support markdown formatting in notes from day one" — Reveal.js supports HTML notes; Marp supports HTML notes. The DSL `notes:` field should support the same inline formatting as slide body text (bold/italic/code — Q8), not just plain text.

10. **The `weighted_composite` raw-weight bug is a template for spec clarity.** R1: the Python reference displays raw `weight` labels even when weights are normalized (e.g., weights sum to 200 → segments are each 25% but labels say 50%). The DSL must make normalization explicit: either require weights sum to 100 (hard error if not), or always display normalized percentages. This is one of 18 "quirks NOT to carry forward" in R1.

---

### Architectural steer for Phase 1

The Phase 1 architect MUST know the following from this document before writing ADRs:

1. **ADR-001 (OOXML synthesis):** R2 + R4 together establish the complete PPTX package requirements. Generate 11 standard layouts + ~20 custom layouts from one slide master + one theme. Use `clrMapOvr` for dark-themed layouts (dividers, end). Element ordering is strict. `notesMaster1.xml` is required even if empty. `standalone="yes"` on every XML declaration. The `sldId` counter starts at 256; `sldMasterId` starts at 2147483648.

2. **ADR-006 / ADR-010 (IR shape):** The IR is a two-layer design: `Deck` (semantic) + `LaidOutDeck` (geometric). `Deck` uses `Vec<Arc<Slide>>` where `Slide` is a sum type over 23 variants plus `Raw { exporter, content }`. All IR types implement `Hash + Eq + Clone` for comemo compatibility. The `Attr` bag (id, classes, kv-pairs) provides Pandoc-style extensibility on every node. `Raw` variant is IR-internal only (not a DSL keyword in v1.0).

3. **ADR-008 (canvas renderer):** R5 establishes that `<canvas>`-only slide rendering cannot be WCAG AA compliant without a parallel DOM accessibility tree. SVG-based rendering (with `<title>`, `<desc>`, `role="img"`) is the lower-friction path to AA compliance. This is a key input to the canvas-vs-SVG decision.

4. **ADR-011 (WCAG tooling):** R5 recommends `@axe-core/playwright` for web surfaces, `veraPDF` + PAC 2024 for PDF, custom OOXML linter (Rust, in-tree) + Microsoft Accessibility Checker (manual) for PPTX. Lighthouse is explicitly rejected (runs only ~50 of axe-core's ~96 rules).

5. **Parser (chumsky):** The indentation-significant DSL (locked Q2) must: reject tabs hard (spaces only), reject YAML-style implicit type coercion (`NO` stays the string "NO", `1.10` stays "1.10"), support `@include` with cycle detection, parse `{{ var_name }}` interpolation in string fields only. The keyword table must reserve all future keywords listed in Q22-Q25.

---

### NFRs identified from research

These flow into the PRD as binding non-functional requirements:

| NFR ID | Description | Source | Tier |
|--------|-------------|--------|------|
| **perf-NFR-1** | Cold build of 25-slide deck: < 500ms. Incremental rebuild on single-slide change: < 50ms. | decisions-applied Q7 + R6 (comemo incremental model) | P0 |
| **perf-NFR-2** | PPTX output for 25-slide deck: < 5MB before compression. | R2 (typical blank deck ZIPs to ~15-18 KB; 25 real slides add media) | P1 |
| **a11y-NFR-1** | Web preview: zero axe-core violations at `serious`/`critical` severity per PR. | R5 | P0 |
| **a11y-NFR-2** | HTML export: zero axe-core violations on canonical 23-type sample deck. | R5 | P0 |
| **a11y-NFR-3** | PDF export: passes veraPDF for PDF/UA-1 on canonical sample deck. | R5 | P0 |
| **a11y-NFR-4** | PPTX export: zero errors from custom OOXML linter; zero errors from PowerPoint Accessibility Checker (manual, per-release). | R5 | P0 |
| **a11y-NFR-5** | All brand color pairs satisfy WCAG 1.4.3/1.4.11 contrast (4.5:1 normal, 3.0:1 large/non-text). | R5 | P0 |
| **compat-NFR-1** | PPTX output renders correctly in PowerPoint, Keynote, Google Slides, LibreOffice (all four). Verified via automated rendering + visual diff in CI. | R2 + decisions-applied ADR-002 | P0 |
| **compat-NFR-2** | PPTX synthesis from `brand.toml` alone (no base .pptx) produces a valid package that opens without "repair" dialog in all four renderers. | R2 + R4 + decisions-applied Q4 | P0 |
| **ergo-NFR-1** | Build diagnostics include line+column+span for every error. Error messages include a correction hint ("did you mean X?"). | R3 + R1 (quirks section) | P1 |
| **ergo-NFR-2** | Unknown color names are compile errors (not silent fallback). | R1 (resolve_color silent TEAL fallback) | P0 |
| **ergo-NFR-3** | Canvas overflow (text/shapes outside 13.33" × 7.5") is a compile error in strict mode. | R3 (universal pain point #4) | P0 |
| **sec-NFR-1** | The slideforge binary performs no network I/O at build time. All assets (images, fonts, brand templates) are read from local filesystem. | R3 (external image link failures) | P0 |
