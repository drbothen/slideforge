---
title: Composition, Mixins, and Hierarchical Organization — Prior Art
date: 2026-05-23
analyst: research-agent
status: foundation-research
audience: business-analyst + product-owner + architect
prompted_by: "User question — Atmos-style organization for slideforge?"
research_depth: 15 ecosystems surveyed, Atmos studied in depth
perplexity_calls: 4 (3 research + 1 reason); 1 search
context_budget: ~600 lines
---

# Composition / Mixins / Hierarchical Organization — Prior Art

## Executive Summary

**The user asked:** *"What about mixins and organization? Would we benefit from any of that as well? (Thinking like what Atmos does for Terraform.)"*

**Direct answer (TL;DR):** **PARTIAL adoption — yes to lightweight "stacks" (variants), no to Atmos-style components, no to first-class mixin syntax for v1.0.**

The Atmos composition model is exquisitely engineered for **infrastructure-as-code at enterprise scale**, where a single Terraform module gets instantiated across 50+ tenant/region/env combinations with different secrets, CIDRs, and tags. **Slideforge's domain is different in degree and kind**: a deck has 10–50 slides, an organization runs a few brand templates, and the canonical "variant" use case is *"render this deck for execs vs. engineers"* — not "deploy this VPC to 50 accounts."

After studying Atmos plus 14 adjacent ecosystems (Helm, Kustomize, Cue, Dhall, Jsonnet, Nix overlays, OpenAPI, GraphQL, Sass, Tailwind, Pulumi, Jekyll/Hugo, Storybook, Typst, Marp/Slidev), the strong recommendation is:

| Atmos primitive | Slideforge v1.0 verdict | Reasoning |
|---|---|---|
| **Imports / `@include`** | **YES — already decided** | Universal across all 15 ecosystems. Already in `decisions-applied.md` Q3. |
| **Stack variants** (one source, many outputs) | **YES — lightweight form** | Maps directly to "exec vs tech deck" use case. Implement as `variants:` block + tag filters, NOT directory hierarchy. |
| **Mixins** (named reusable fragments) | **NO for v1.0** | Use `@include` of fragment files + theme/brand config. Reserve `mixin` keyword for v2. |
| **Components** (parameterized patterns) | **NO for v1.0** | The 23 opinionated slide types ARE the component library. User-defined components would dilute the opinionated stance. |
| **`metadata.inherits` chain** | **NO** | Multiple inheritance adds enormous complexity; no clear slide use case. |
| **Deep-merge with configurable strategies** | **NO** | Atmos's own community regularly hits regressions here (issue #2376, v1.212 type-mismatch bug fix). High cost, low slide-DSL value. |
| **Go-template variable interpolation** | **MAYBE — simple form only** | `{{ client_name }}` string substitution only. NO conditionals, NO loops, NO functions in v1.0. |
| **Hierarchical config (slideforge.toml per dir)** | **NO** | Single root config. Re-evaluate in v2 if users demand monorepo-style deck collections. |

**The one big idea worth stealing from Atmos:** the **catalog pattern**. A convention for `catalog/<fragment-name>.sf` files that organizations use as their reusable slide-fragment library. This costs nothing to support (it's just `@include` on disk-organized files) but provides huge documentation/discoverability value.

**Top-3 patterns to copy** (across all 15 ecosystems):
1. **Tailwind's flat-specificity guarantee** — utility layer ALWAYS beats component layer. No specificity wars. Map this to: deck-level vars > variant vars > slide-level vars, with explicit, documented precedence.
2. **Hugo's `_default` cascade** — directory-based defaults that apply unless explicitly overridden. Map this to: a `defaults.sf` file at the deck root applies to every slide unless overridden inline.
3. **Typst's `set` rule pattern** — "set the default for X within this scope." Map this to: deck-level `set` blocks that establish defaults for all slides of a given type (e.g., `set severity_cards: color_high red`).

**Top-3 anti-patterns to avoid:**
1. **Helm's `valuesFiles` ordering ambiguity** — never let the import-order semantics be implicit. Slideforge should have ONE merge rule and document it ferociously.
2. **Atmos's deep-merge regression history** — the v1.212.0 type-mismatch bug shows that deep-merge is a footgun even for the experts who invented it. Avoid configurable merge strategies in v1.0; pick ONE (last-wins + replace-lists) and stick to it.
3. **Sass `@extend`'s global cascade** — never let a mixin's effects "leak" outside the slide that uses it. Mixin expansion must be lexically scoped.

---

## Atmos Deep-Dive (Primary Subject)

### The Atmos Model

**Stacks** are hierarchical YAML configuration files that define a logical grouping of infrastructure components to deploy together. A "stack" might be `prod-us-west-2`, with separate stacks for `prod-us-east-1`, `staging-us-west-2`, etc. Stacks compose via the `import:` directive:

```yaml
# stacks/prod/us-west-2/app.yaml
import:
  - /stacks/_defaults/global.yaml
  - /stacks/_defaults/networking.yaml
  - /stacks/_defaults/security-prod.yaml
  - /stacks/catalog/database.yaml
vars:
  cidr_block: "10.10.0.0/16"
components:
  terraform:
    database:
      source: "github.com/cloudposse/terraform-aws-rds-cluster?ref=0.83.0"
```

Source: [atmos.tools/learn/imports-basics](https://atmos.tools/learn/imports-basics) (2026).

**Components** are Terraform modules wrapped with stack-driven config. One Terraform code → many instances per stack. Each stack file specifies the component's `vars`, `backend`, `providers`, etc. Source: [atmos.tools/projects/layout](https://atmos.tools/projects/layout) (2026).

**Imports / mixins** are processed in order, top-to-bottom, with **later imports taking precedence**. The deep-merge happens recursively across the entire YAML tree. Two conventions split file purposes:

- `stacks/catalog/<component-name>/` — component-specific default configurations (e.g., `catalog/vpc/default.yaml`).
- `stacks/mixins/<concern>/` — cross-cutting defaults like security hardening, observability defaults.

Source: [atmos.tools/howto/catalogs](https://atmos.tools/howto/catalogs) (2026).

**Inheritance chain** is a *separate* mechanism via `metadata.inherits` on a component. Multiple inheritance is supported; later entries in the inherits list take precedence:

```yaml
# components/database/prod.yaml
metadata:
  name: prod-database
  inherits:
    - base-database
    - security-hardened
vars:
  instance_type: "db.r6g.xlarge"
```

Source: [github.com/cloudposse/atmos agent-skills](https://github.com/cloudposse/atmos/blob/main/agent-skills/skills/atmos-components/SKILL.md) (2026).

**Variable interpolation** uses Go templates with `{{ .vars.foo }}` and context variables auto-derived from the stack path (e.g., `.env`, `.region`). Conditionals and template functions are supported. There is a **template-escape mechanism** using ``{{`{{` }}`` to pass templates through to external systems unprocessed. Source: [atmos.tools/templates](https://atmos.tools/templates) (2026).

**Deep-merge algorithm** is the heart of Atmos. The documented rules:

| Type | Default behavior |
|---|---|
| **Scalars** (string, number, bool) | Last-wins replacement |
| **Maps** | Recursive deep-merge, key-by-key |
| **Lists** | **Replace** by default (not append). Configurable via `merge_strategies` settings to `replace`, `append`, or `merge`. |

Sources: [atmos.tools/stacks/vars](https://atmos.tools/stacks/vars), [atmos.tools/cli/configuration/settings](https://atmos.tools/cli/configuration/settings), [atmos.tools/changelog/faster-deep-merge](https://atmos.tools/changelog/faster-deep-merge) (2026).

**Resolution precedence** (full chain): global vars → component-type vars → base-component inheritance → component-specific vars → file-scoped `overrides:`. Each layer can override the previous.

Source: [atmos.tools/stacks/overrides](https://atmos.tools/stacks/overrides) (2026).

### What Atmos Gets Right

1. **Convention over configuration for organization.** The `catalog/` and `mixins/` directory split is a clean separation between "component defaults" and "cross-cutting concerns." Organizations don't have to invent their own structure. Source: [atmos.tools/howto/catalogs](https://atmos.tools/howto/catalogs).

2. **One import keyword.** Unlike Terragrunt which has `include`, `dependency`, `terraform_remote_state`, etc., Atmos has just `import:` for composition. Simple mental model.

3. **Layered precedence is documented and stable.** The resolution order (global → component-type → inherits → component-specific → overrides) is explicitly published and reasonably intuitive.

4. **Context variables from directory structure.** Auto-providing `.env`, `.region` from the stack path is genuinely useful; it eliminates a class of bugs where users forget to set the env.

5. **Recent doc improvements.** As of March 2026, Cloud Posse explicitly recognized that *"developers commonly struggle with variable resolution"* and added a Blueprint Configuration guide and a Variable Merge Order guide. Source: [Atmos PR #2146, v1.210.0 release notes](https://newreleases.io/project/github/cloudposse/atmos/release/v1.210.0) (March 2026).

### What Atmos Gets Wrong

1. **Deep-merge is the recurring footgun.** Multiple GitHub issues track regressions:
   - **Issue #2376** (2025): "native deep merge replaces nested map with integer YAML key instead of recursively merging" — a regression in v1.212.0.
   - **PR #2248** (March 2026): had to add backward-compatibility for users overriding lists with `{}` instead of `[]` — described as *"this pattern exists in production configs and worked with the previous mergo-based merge."*
   Source: [Atmos CHANGELOG v1.212.0 fix notes](https://newreleases.io/project/github/cloudposse/atmos/release/v1.212.0) (March 2026).

2. **List-replace is a default that surprises users.** Atmos chose `replace` (not `append`) for lists by default, which means newcomers writing `vars.allowed_cidrs:` in a child stack silently *replace* the parent's list rather than extending it. The documented workaround is to configure `merge_strategies` explicitly per-key, but this requires sophistication. Source: [atmos.tools/stacks/vars](https://atmos.tools/stacks/vars).

3. **Two parallel composition systems is confusing.** Imports compose at file level; `metadata.inherits` composes at component level. Both are mandatory to learn. Issue threads regularly show users confusing the two. Source: [github.com/cloudposse/atmos/issues](https://github.com/cloudposse/atmos/issues).

4. **Go template double-pass for imports.** When templates appear in both imports and stacks, Atmos processes them TWICE, requiring users to know the `{{` `{{` `}}` `}}` escape trick. This is a niche feature that bites everyone who hits it. Source: [Atmos v1.70.0 release notes](https://newreleases.io/project/github/cloudposse/atmos/release/v1.70.0) (2024).

5. **Command-merging regressions.** The `commands:` field had a separate merging regression (PR #1533, October 2025) where imported commands were being *replaced* instead of *merged* — fixed by adding name-based override semantics. This is the kind of bug that emerges only with multi-level org/team/project hierarchies.

### Mapping Atmos Concepts → Potential Slideforge Concepts

| Atmos | Slideforge analog | Useful for v1.0? | Reasoning |
|---|---|---|---|
| **Stack** | Variant deck (same source, different audience/redaction/brand) | **YES — lightweight** | Maps directly to flagship "exec vs tech deck" use case. Implement as in-deck `variants:` block, not a directory hierarchy. |
| **Component** | Parameterized slide pattern | **NO** | The 23 opinionated slide types ARE the component library. User-defined components dilute the opinionated stance. |
| **Mixin (import)** | Named reusable slide fragment | **NO syntax — YES via `@include`** | `@include` of fragment files is enough. No `mixin` keyword in v1.0. |
| **`metadata.inherits`** | Slide-pattern inheritance | **NO** | Multiple inheritance is a complexity bomb. Single-source `@include` is enough. |
| **Variable interpolation** | Data binding | **MAYBE — simple form** | `{{ client_name }}` substitution. NO conditionals, NO loops in v1.0. |
| **Catalog directory** | `catalog/` for shared fragments | **YES — convention only** | Free benefit; just a directory convention organizations adopt. |
| **`mixins/` directory** | Brand/footer/disclaimer fragments | **YES — convention only** | Same as above. No new syntax. |
| **`merge_strategies` config** | List-merge tuning | **NO** | The Atmos community regularly trips on this. Pick ONE rule (last-wins, replace lists) and document it. |
| **Go-template double-pass** | — | **HELL NO** | Famous source of confusion. |
| **Workflow / orchestration** | Build pipeline | **NO** | Out of scope; CLI handles single-deck builds. |

---

## Comparative Survey — Other Composition Models

### Helm (Kubernetes values inheritance)

- **Model:** Layered values with strict precedence — `chart values.yaml < parent chart values < user values (-f file1 -f file2) < --set`. Recursive map merging; **list replacement** (no key-aware merging).
- **Strengths:** Precedence rules are documented and consistent; `--set` always wins.
- **Weaknesses:** List-merge is the *canonical* Helm pain point. Adding one entry to a list requires re-specifying the whole list. GitHub issue #3486 is still open after years. The v3.8.2 release broke users with a subtle merge behavior change (issues #10998, #10899).
- **Lessons for slideforge:** Document the precedence order obsessively. If we replace lists by default (Atmos's choice), users will be surprised; if we append, that breaks the "smaller is more specific" mental model. Probably: replace lists, but provide explicit `+= [...]` syntax for append. Or: avoid lists in user-facing config entirely.
- **Citations:** [helm.sh/docs/chart_template_guide/values_files](https://helm.sh/docs/chart_template_guide/values_files) (2026), [github.com/helm/helm/issues/3486](https://github.com/helm/helm/issues/3486), [github.com/helm/helm/issues/12677](https://github.com/helm/helm/issues/12677), [OneUptime blog Feb 2026](https://oneuptime.com/blog/post/2026-02-09-kustomize-strategic-merge-patches/view).

### Kustomize (patch composition)

- **Model:** Pure composition with no inheritance. Strategic-merge patches use Kubernetes API schema knowledge (the `name:` field is treated as a merge key for `env:` lists). JSON patches operate at path level.
- **Strengths:** Schema-aware list merging is genuinely better than Helm. Layered overlays (`bases/` + `overlays/dev/` + `overlays/prod/`) is a clean directory pattern.
- **Weaknesses:** Strategic-merge fails badly for CRDs (no schema known). Patch ordering matters; same patch applied at different positions can produce different results. Cannot remove list elements via strategic-merge.
- **Lessons for slideforge:** Schema-aware merging is great but requires the engine to know slide-type-specific keys. Slideforge has 23 opinionated types — could in theory hardcode merge keys per slide type (e.g., `severity_cards.items[].name` as a merge key). **But** this is v2 territory.
- **Citations:** [kubernetes.io/docs/tasks/manage-kubernetes-objects/kustomization](https://kubernetes.io/docs/tasks/manage-kubernetes-objects/kustomization/), [Argo blog May 2024](https://blog.argoproj.io/argo-crds-and-kustomize-the-problem-of-patching-lists-5cfc43da288c).

### Cue

- **Model:** Constraint-based unification. Not inheritance — values are *unified* via `&`. Contradictions are validation errors, not silent overrides. Order-independent.
- **Strengths:** Order-independence eliminates an entire class of bugs. Strong types embedded in the values.
- **Weaknesses:** Steep learning curve. Strict module system. Less flexible for "I want to override this field" semantics.
- **Lessons for slideforge:** The constraint-based approach is intellectually appealing but the user base is corporate report writers, not type theorists. **Don't go here.**
- **Citations:** [cuelang.org/docs/concept/the-logic-of-cue](https://cuelang.org/docs/concept/the-logic-of-cue/) (2026), [github.com/cue-lang/cue/issues/813](https://github.com/cue-lang/cue/issues/813).

### Dhall

- **Model:** Pure functional with totality. Composition via record merge operators: `/\` (recursive merge), `//` (shallow merge), explicit prefer-left/prefer-right.
- **Strengths:** Total functions guarantee no runtime errors. Explicit merge operators eliminate ambiguity.
- **Weaknesses:** Functional thinking is a barrier for non-programmer users. Verbose.
- **Lessons for slideforge:** The *idea* of explicit merge operators is good — but in YAML/indentation DSLs, the syntax would be terrible. **Skip.**
- **Citations:** [docs.dhall-lang.org/tutorials/Language-Tour.html](https://docs.dhall-lang.org/tutorials/Language-Tour.html), [github.com/dhall-lang/dhall-lang/issues/114](https://github.com/dhall-lang/dhall-lang/issues/114).

### Jsonnet

- **Model:** Prototype-based with `self`, `super`, hidden fields (`::`), and the `+:` deep-merge field operator. Multiple inheritance via `+`.
- **Strengths:** Most powerful object-merging primitive of any config language. `+:` is genuinely elegant.
- **Weaknesses:** `self`/`super` resolution is famously confusing. Array composition defaults to concatenation, which surprises users expecting deep-merge. `std.mergePatch` exists for sophisticated needs but adds complexity.
- **Lessons for slideforge:** If we ever add a v2 mixin system, study Jsonnet's `+:` carefully — but its complexity is well above what slide-deck authors should encounter.
- **Citations:** [jsonnet.org/learning/tutorial.html](https://jsonnet.org/learning/tutorial.html), [groups.google.com/g/jsonnet](https://groups.google.com/g/jsonnet/c/6OkYunT5iXI) (April 2025).

### Nix overlays

- **Model:** `self: super: { ... }` fixed-point functions. Multiple overlays compose; later overlays see earlier overlays' modifications via `super`.
- **Strengths:** Mathematically elegant. Composable in arbitrary depth.
- **Weaknesses:** Infinite-recursion footgun if `self` is referenced outside the returned attribute set. Evaluation order is non-obvious.
- **Lessons for slideforge:** Fixed-point composition is overkill for slides. **Skip.**
- **Citations:** [discourse.nixos.org/t/what-are-overlays/14680](https://discourse.nixos.org/t/what-are-overlays/14680), [discourse.nixos.org/t/infinite-recursion-when-composing-overlays/7594](https://discourse.nixos.org/t/infinite-recursion-when-composing-overlays/7594) (March 2025).

### OpenAPI 3 (allOf, oneOf, anyOf)

- **Model:** Schema composition via `allOf` (intersection), `oneOf` (XOR), `anyOf` (OR). `$ref` for reuse.
- **Strengths:** Well-understood schema algebra.
- **Weaknesses:** `allOf` semantics for property overriding are *ambiguous* in the spec — many tools disagree on whether later schemas in `allOf` override earlier ones (issue #3069, July 2025). `discriminator` is widely seen as deprecated.
- **Lessons for slideforge:** Schema composition is a different problem from value composition. The lesson is: be *explicit* about overriding semantics, even at the cost of brevity. Don't repeat OpenAPI's mistake.
- **Citations:** [swagger.io/docs/specification/v3_0/data-models/oneof-anyof-allof-not](https://swagger.io/docs/specification/v3_0/data-models/oneof-anyof-allof-not/), [github.com/OAI/OpenAPI-Specification/issues/3069](https://github.com/OAI/OpenAPI-Specification/issues/3069).

### GraphQL fragments

- **Model:** Reusable selection sets (`fragment FooFields on Foo { ... }`), interfaces, unions.
- **Strengths:** Lexically scoped, composable, type-safe.
- **Weaknesses:** Tooling-heavy.
- **Lessons for slideforge:** GraphQL fragments are conceptually similar to "named slide fragments." If we ever add first-class mixins, the GraphQL fragment syntax (`fragment standard_footer on Slide { ... }` + `...standard_footer`) is one of the cleaner models.
- **Citations:** [graphql.org/learn/queries/#fragments](https://graphql.org/learn/queries/#fragments).

### Sass / SCSS (`@mixin`, `@include`, `@extend`)

- **Model:** `@mixin name($args) { ... }` defines reusable rule blocks; `@include name($args)` instantiates. `@extend` creates selector-level inheritance with global cascade effects.
- **Strengths:** `@mixin`/`@include` is the textbook example of named reusable fragments with parameters.
- **Weaknesses:** `@extend` has a famous "specificity cascade" footgun — when you extend a selector, all rules targeting that selector now also target the extender, with global side effects. Sass issue #324 (August 2025) is still debating selector specificity fixes.
- **Lessons for slideforge:** Sass's `@mixin name(args)` + `@include name(args)` is the *exact* mental model many slide authors would expect for a mixin system. If we add mixins in v2, copy this syntax (adapted to indentation-significant form). **Avoid `@extend`-style global cascade** — keep mixin effects lexically scoped.
- **Citations:** [sass-lang.com/documentation/at-rules/extend](https://sass-lang.com/documentation/at-rules/extend/), [github.com/sass/sass/issues/324](https://github.com/sass/sass/issues/324).

### Tailwind CSS

- **Model:** Three explicit layers (`@layer base`, `@layer components`, `@layer utilities`) with *architectural* precedence — utilities ALWAYS beat components, regardless of source-order. `@apply` consolidates utility classes into named patterns. `theme.extend` adds tokens.
- **Strengths:** **Flat-specificity guarantee.** No specificity wars. Layer precedence is enforced by the framework, not by author discipline.
- **Weaknesses:** `@apply` is famously controversial. It creates implicit dependencies (changing the base `btn` class breaks all `btn-*` derivatives silently). Debug stories at runtime show only compiled CSS classes, not source.
- **Lessons for slideforge:** **The flat-specificity guarantee is the single best idea in this entire survey.** Map it to slideforge as: explicit, documented, layered precedence (`deck vars > variant vars > slide vars`), with the engine enforcing the order, not the author. **Avoid `@apply`-style hidden-dependency patterns** — every override should be visible at the call site.
- **Citations:** [tailwindcss.com/docs/adding-custom-styles](https://tailwindcss.com/docs/adding-custom-styles), Tailwind documentation.

### Pulumi (stacks and config)

- **Model:** A "stack" is a named instance of a Pulumi program (`Pulumi.yaml` + `Pulumi.<stack>.yaml`). Config inheritance is shallow — each stack file independently sets its own values; no implicit cross-stack merging.
- **Strengths:** Simple and explicit. No deep-merge magic.
- **Weaknesses:** No reuse of config across stacks; teams hand-roll their own templating.
- **Lessons for slideforge:** Pulumi's approach is the *minimal* viable variant system. It's what slideforge should aspire to in v1.0: a `variants:` block that simply lists per-variant overrides, with no inheritance machinery.
- **Citations:** [pulumi.com/docs/concepts/stack](https://www.pulumi.com/docs/concepts/stack/).

### Jekyll / Hugo (layouts, includes, partials)

- **Model:** Layouts inherit via `{% extends %}` (Jekyll) or `define`/`block` (Hugo). Partials/includes are reusable fragments. Hugo introduces a sophisticated lookup-order system for templates.
- **Strengths:** Hugo's `_default` cascade is genuinely useful — files in `_default/` apply globally unless overridden.
- **Weaknesses:** Hugo lookup-order is notoriously hard to predict; the doc has long flowcharts. Variable scope leaks between includes are common.
- **Lessons for slideforge:** **The `_default` cascade is worth copying** — a `defaults.sf` file at the deck root that applies to every slide unless overridden inline. This is the lightweight version of Atmos's `_defaults/` directory convention.
- **Citations:** [gohugo.io/templates/lookup-order](https://gohugo.io/templates/lookup-order/), [jekyllrb.com/docs/layouts](https://jekyllrb.com/docs/layouts/).

### Storybook (stories, decorators, args)

- **Model:** Stories are component instances with `args` (parameters) and `parameters` (configuration). Decorators wrap stories with additional context.
- **Strengths:** Clear separation between component (the reusable thing), args (per-instance data), and parameters (per-instance config).
- **Weaknesses:** The `args` vs `parameters` distinction confuses newcomers. Decorator composition can be hard to debug.
- **Lessons for slideforge:** The args/parameters split is conceptually parallel to "slide content" vs "slide rendering options." Slideforge already has this implicitly (the slide body is content; tags/sensitivity/audience are parameters). **Worth being explicit about the distinction.**
- **Citations:** [storybook.js.org/docs/writing-stories](https://storybook.js.org/docs/writing-stories).

### Typst (functions, `show` rules, `set` rules, templates)

- **Model:** `set` rules establish defaults for an element type within a scope (`set heading(numbering: "1.")`); `show` rules transform elements based on pattern matching; functions/templates are first-class for parameterization. All lexically scoped — closest rule wins.
- **Strengths:** The `set` rule is the cleanest mental model for "default unless overridden" of any system surveyed. Lexical scoping means no spooky action at a distance.
- **Weaknesses:** No mechanism for "sealed" templates that prevent downstream overrides. `show` rules with overlapping matchers have last-rule-wins behavior that can surprise.
- **Lessons for slideforge:** **The `set` rule is the closest analog to what slideforge actually needs.** A deck-level `set severity_cards: color_high red` block establishes a default that every `severity_cards` slide inherits, unless the slide overrides it. **Strongly recommend adopting this concept.**
- **Citations:** [typst.app/docs/reference/styling](https://typst.app/docs/reference/styling/), [typst.app/docs/tutorial/making-a-template](https://typst.app/docs/tutorial/making-a-template/).

### Marp / Slidev (themes, layouts)

- **Model (Marp):** Themes are CSS files; directives in frontmatter control layout. CSS cascade governs everything.
- **Model (Slidev):** Vue components for layouts. Frontmatter precedence: global config < theme < frontmatter < inline props.
- **Strengths:** Marp's simplicity. Slidev's Vue power.
- **Weaknesses:** **Both are weak for corporate branding.** Marp has no theme-parameter mechanism — minor brand variations require entire new CSS files. Slidev's auto-importing components create namespace collisions when corporate addons conflict. Both struggle with "lock down certain styles, allow overrides for others."
- **Lessons for slideforge:** This is slideforge's *exact* opportunity. Marp/Slidev's branding pain is well-documented. Slideforge should ship with: (a) a brand template selector, (b) explicit "this is locked by brand" vs "this is author-overridable" semantics, (c) variant support for one-source-many-decks. **All three are slideforge's distinguishing features.**
- **Citations:** Synthesized from Slidev and Marp documentation review (2026); Marp issue trackers and Slidev addon collisions are recurring themes.

---

## Pattern Distillation

### Five Canonical Composition Primitives (across all 15 ecosystems)

| Primitive | Description | Found in | Slideforge v1.0? |
|---|---|---|---|
| **1. Imports / `@include`** | Pull-in by path | All 15 ecosystems | **YES** (confirmed) |
| **2. Mixins** | Named reusable fragments | Sass, GraphQL fragments, Atmos catalog, Helm partials | **NO syntax** (use `@include`) |
| **3. Inheritance** | Base + override (single or multiple) | Jsonnet, Atmos, Helm subcharts, Hugo layouts, OO languages | **NO** (avoid) |
| **4. Composition (patch)** | Merge multiple sources | Kustomize, Helm values, Atmos imports, Jsonnet `+`, Nix overlays | **YES** (via variants only) |
| **5. Parameterization** | Instantiate with arguments | Typst functions, Sass mixin args, Dhall, Storybook args, GraphQL variables | **MAYBE** (deck-level vars only) |

### Merge Semantics — The Design Space

| Strategy | Scalars | Lists | Maps | Used by |
|---|---|---|---|---|
| **Last-wins (Atmos default)** | replace | **replace** (or append, configurable) | deep merge | Atmos, Helm, Pulumi |
| **First-wins** | first wins | first wins | first wins | (Rare — Hugo for some cases) |
| **Explicit per-key** | author chooses | author chooses | author chooses | Dhall (`/\` vs `//`), Jsonnet (`+` vs `+:`) |
| **Constraint unification** | exact equality | element constraints | constraint propagation | Cue |
| **Schema-aware** | replace | merge by key field | recursive | Kustomize (strategic-merge) |
| **Architectural layering** | last-wins within layer | replace | deep merge | Tailwind (utility > component > base) |

**Recommendation for slideforge:** **Last-wins with replace-for-lists**, identical to Atmos's default. The reasoning:
- It's the most common model across the surveyed systems.
- Users coming from Helm/Atmos will have the right mental model.
- "Replace lists" is the safer default (no accidental list growth across imports).
- If users want to extend a list, they re-declare the full list explicitly. This is verbose but unambiguous.
- **Do NOT add configurable merge strategies in v1.0** — Atmos's own community trips on these regularly.

---

## Recommendations for Slideforge

### Recommendation 1: Adopt LIGHTWEIGHT variants ("stacks-lite")

**Why:** Maps directly to the flagship use case ("same source, exec vs tech deck, internal vs external"). Without variants, users will copy-paste decks and lose the single-source guarantee.

**Cost:** A `variants:` block in deck metadata; tag-based filtering at slide level. Small parser surface; small runtime surface. Big user value.

**Proposed syntax sketch** (within indentation-significant DSL):

```
deck:
  id: q3-performance
  title: "Q3 Performance Review"
  brand: acme-default

  vars:
    client_name: "Globex Corp"
    project_code: "GX-API-23"

  variants:
    exec-external:
      audience: executive
      sensitivity: external
      include_tags: [core, exec]
      exclude_tags: [deep_tech, internal_only]
      brand: acme-exec
      vars:
        footer_text: "For external distribution with {{ client_name }} approval"

    tech-internal:
      audience: technical
      sensitivity: internal
      include_tags: [core, tech]
      brand: acme-tech

slide cover:
  tags: [core]
  title: "Q3 Performance Review – {{ client_name }}"
  subtitle: "Project {{ project_code }}"

slide severity_cards:
  tags: [core, exec]
  title: "Key Risks This Quarter"
  items:
    - severity: high
      title: "API Latency"
      detail: "SLO violations in 3 regions"

slide timeline:
  tags: [core, tech, deep_tech]
  title: "Incident Timeline – Q3"
  events:
    - date: "2026-07-12"
      label: "Latency spike detected"
```

Build invocation:
```
slideforge build q3.sf --variant exec-external
```

**Precedence rules (explicit, documented):**
1. Slide-level field (highest)
2. Variant `vars`
3. Deck-level `vars` (lowest)

All scalars/maps use last-wins. Lists are replaced wholesale (no append).

### Recommendation 2: Adopt Typst-style `set` rules for slide-type defaults

**Why:** The cleanest mental model for "establish a default for all slides of type X within this scope." Solves the "I want all my `severity_cards` slides to use my company's color scheme without copying it 10 times" problem without inventing mixins.

**Cost:** A `set` keyword that takes a slide-type identifier + key-value defaults. Lexically scoped.

**Proposed syntax sketch:**
```
deck:
  id: q3-performance

  # Establish defaults for all severity_cards slides in this deck
  set severity_cards:
    color_high: red
    color_medium: orange
    color_low: green
    sort_by: severity

  set timeline:
    show_dates: true
    date_format: "YYYY-MM-DD"

slide severity_cards:
  # Inherits color_high=red, color_medium=orange, color_low=green, sort_by=severity
  items:
    - severity: high
      title: "..."

slide severity_cards:
  # Override sort_by just for this slide
  sort_by: title
  items:
    - ...
```

**Why not just call this "mixins"?** Because `set` is type-scoped (you set defaults for a slide type), while mixins are arbitrary reusable fragments. `set` is a cleaner, more constrained concept.

### Recommendation 3: Adopt Hugo-style `_default/` cascade convention

**Why:** A `defaults/` directory at the deck root (or globally) holds `<slide-type>.sf` files with default settings. The engine auto-merges these as if they were `set` blocks. This gives organizations a way to ship "company-wide slide defaults" without ceremony.

**Cost:** A directory-scanning convention; no new syntax.

**Proposed layout:**
```
my-deck/
├── deck.sf
├── slides/
│   ├── 01-cover.sf
│   ├── 02-agenda.sf
│   └── ...
├── defaults/                    # ← Auto-applied as set rules
│   ├── severity_cards.sf
│   └── timeline.sf
├── catalog/                     # ← Reusable fragments (via @include)
│   ├── standard_disclaimer.sf
│   └── about_this_report.sf
└── slideforge.toml
```

### Recommendation 4: Simple variable interpolation, NO control flow

**Why:** Users need `{{ client_name }}` for branded boilerplate. They do NOT need conditionals, loops, or function calls in the DSL — that's a regression toward Python-as-DSL, which is what we're rewriting away from.

**Cost:** A tiny parser feature: `{{ identifier }}` (or `{{ namespace.identifier }}`) string substitution. No expressions.

**Rules (explicit):**
- `{{ var }}` substitutes a deck-level or variant-level `vars` entry.
- No conditionals (`{{ if }}`), no loops, no function calls.
- If a variable is undefined, the build fails with a diagnostic (no silent empty-string substitution — that's a class of bugs).
- Escape with `\{{ }}` if a literal is needed.

### Recommendation 5: Reserve `catalog/` and `mixins/` as conventions

**Why:** Same as Atmos. Zero engine cost — these are just directory conventions documented in the guidelines. Organizations adopt them, get free organization.

**Cost:** Documentation only.

### Anti-Recommendations (Patterns Slideforge Should Explicitly NOT Adopt)

1. **NO user-defined `component` syntax in v1.0.** The 23 opinionated slide types ARE the component library. User-defined components would dilute the opinionated stance and create maintenance burden. Reconsider in v2 if real demand emerges for "compound slide patterns" (e.g., "our standard 3-slide incident report").

2. **NO `mixin`/`include` keyword distinction.** Stick with `@include` for everything. Sass's `@mixin` vs `@include` distinction is unnecessary when fragments are file-scoped.

3. **NO `metadata.inherits` chain.** Multiple inheritance is a complexity bomb. Variants + `set` rules + `@include` cover every realistic slide-DSL need.

4. **NO configurable merge strategies.** Pick last-wins + list-replacement. Document it. Move on. Atmos's regression history proves configurable merge is a footgun.

5. **NO Go-template-style logic in interpolation.** No `{{ if }}`, no `{{ range }}`, no functions. We're rewriting Python-as-DSL, not reinventing it.

6. **NO `slideforge.toml` per directory with hierarchical merge in v1.0.** One config file at the deck root. Re-evaluate in v2 if monorepo-style deck collections become a real use case.

7. **NO third-party "slide kit" package registry in v1.0.** Git-based `@include` from local paths is enough. A registry is a v2+ feature with significant ecosystem ramifications.

8. **NO `@extend`-style global cascade.** All composition effects must be lexically scoped to the slide that uses them.

---

## DSL Design Questions Added by This Research

These are new questions for the synthesis doc to address:

- **Q14:** Should slideforge have first-class mixin syntax in v1.0, or rely on `@include` of fragment files?
  - **Proposed default:** `@include` only. No `mixin` keyword. Reserve `mixin` as a reserved identifier for v2.
- **Q15:** Should slideforge have first-class user-defined components (parameterized slide patterns)?
  - **Proposed default:** **No.** The 23 opinionated slide types are the component library. Revisit in v2.
- **Q16:** Should slideforge have a "stack" / "variant" concept for one-source-many-decks?
  - **Proposed default:** **Yes — lightweight form.** In-deck `variants:` block + tag filters + per-variant `vars`. No directory-hierarchy stacks.
- **Q17:** Deep-merge semantics for slide-property maps and lists — last-wins, append, or explicit?
  - **Proposed default:** Last-wins. Lists are replaced wholesale, not appended. No configurable merge strategies in v1.0.
- **Q18:** Variable interpolation — scope rules? Pure-functional or impure?
  - **Proposed default:** Simple `{{ name }}` string substitution. Scope: deck-level vars + variant vars (variant wins). No conditionals, no loops, no functions. Undefined variables fail the build.
- **Q19:** Library / package model — should slideforge support installable third-party slide kits in v1.0?
  - **Proposed default:** **No.** `@include` from local paths or git URLs (relative to deck root) only. No registry. Re-evaluate in v2.
- **Q20:** Hierarchical project config — `slideforge.toml` per directory with merge, or single root config?
  - **Proposed default:** Single root config. No directory-scoped merge in v1.0.
- **Q21:** Inheritance precedence — explicit or implicit?
  - **Proposed default:** **Explicit and documented.** Three layers only: deck > variant > slide. No metadata.inherits chain.
- **Q22:** Should slideforge support Typst-style `set` rules for slide-type defaults?
  - **Proposed default:** **Yes.** `set <slide-type>:` blocks at deck scope set defaults for all slides of that type. Lexically scoped.
- **Q23:** Should slideforge support a Hugo-style `defaults/` directory convention?
  - **Proposed default:** **Yes (as a convention, not new syntax).** Files in `defaults/<slide-type>.sf` are auto-applied as `set` blocks. Costs nothing; benefits organizations.
- **Q24:** Should slideforge reserve `catalog/` and `mixins/` directory names as conventions?
  - **Proposed default:** **Yes (documentation only).** Standard organization for shared fragment libraries. No engine code required.
- **Q25:** Should the parser have a reserved `mixin` keyword now (for v2 future-proofing)?
  - **Proposed default:** **Yes.** Reserve `mixin`, `component`, `inherits`, `extends` as identifiers that the parser recognizes but rejects with a "reserved for future use" diagnostic.

---

## Sources

### Atmos (primary subject)
- [atmos.tools/learn/imports-basics](https://atmos.tools/learn/imports-basics) — 2026
- [atmos.tools/howto/catalogs](https://atmos.tools/howto/catalogs) — 2026
- [atmos.tools/projects/layout](https://atmos.tools/projects/layout) — 2026
- [atmos.tools/stacks/vars](https://atmos.tools/stacks/vars) — 2026
- [atmos.tools/stacks/overrides](https://atmos.tools/stacks/overrides) — 2026
- [atmos.tools/templates](https://atmos.tools/templates) — 2026
- [atmos.tools/cli/configuration/settings](https://atmos.tools/cli/configuration/settings) — 2026
- [atmos.tools/cli/commands/describe/locals](https://atmos.tools/cli/commands/describe/locals) — 2026
- [atmos.tools/changelog/faster-deep-merge](https://atmos.tools/changelog/faster-deep-merge) — 2026
- [atmos.tools/migration/terragrunt](https://atmos.tools/migration/terragrunt) — 2026
- [github.com/cloudposse/atmos/blob/main/agent-skills/skills/atmos-components/SKILL.md](https://github.com/cloudposse/atmos/blob/main/agent-skills/skills/atmos-components/SKILL.md) — 2026
- [github.com/cloudposse/atmos/issues](https://github.com/cloudposse/atmos/issues) — accessed May 2026
- [github.com/cloudposse/atmos/issues/2376](https://github.com/cloudposse/atmos/issues/2376) — 2025 (deep-merge integer-key regression)
- [newreleases.io/project/github/cloudposse/atmos/release/v1.210.0](https://newreleases.io/project/github/cloudposse/atmos/release/v1.210.0) — March 2026 (Blueprint Configuration docs)
- [newreleases.io/project/github/cloudposse/atmos/release/v1.212.0](https://newreleases.io/project/github/cloudposse/atmos/release/v1.212.0) — March 2026 (deep-merge type-mismatch backward-compat fix)
- [newreleases.io/project/github/cloudposse/atmos/release/v1.70.0](https://newreleases.io/project/github/cloudposse/atmos/release/v1.70.0) — 2024 (Go-template double-pass escape)
- [newreleases.io/project/github/cloudposse/atmos/release/v1.193.0](https://newreleases.io/project/github/cloudposse/atmos/release/v1.193.0) — October 2025 (command-merging regression fix)

### Helm
- [helm.sh/docs/chart_template_guide/values_files](https://helm.sh/docs/chart_template_guide/values_files) — 2026
- [github.com/helm/helm/issues/3486](https://github.com/helm/helm/issues/3486) — list-merging pain
- [github.com/helm/helm/issues/12677](https://github.com/helm/helm/issues/12677) — precedence confusion

### Kustomize
- [kubernetes.io/docs/tasks/manage-kubernetes-objects/kustomization](https://kubernetes.io/docs/tasks/manage-kubernetes-objects/kustomization/) — 2026
- [blog.argoproj.io/argo-crds-and-kustomize-the-problem-of-patching-lists](https://blog.argoproj.io/argo-crds-and-kustomize-the-problem-of-patching-lists-5cfc43da288c) — May 2024
- [oneuptime.com/blog/post/2026-02-09-kustomize-strategic-merge-patches/view](https://oneuptime.com/blog/post/2026-02-09-kustomize-strategic-merge-patches/view) — February 2026

### Cue
- [cuelang.org/docs/concept/the-logic-of-cue](https://cuelang.org/docs/concept/the-logic-of-cue/) — 2026
- [cuelang.org/docs/reference/spec](https://cuelang.org/docs/reference/spec/) — 2026
- [github.com/cue-lang/cue/issues/813](https://github.com/cue-lang/cue/issues/813) — March 2025 (module system friction)

### Dhall
- [docs.dhall-lang.org/tutorials/Language-Tour.html](https://docs.dhall-lang.org/tutorials/Language-Tour.html)
- [github.com/dhall-lang/dhall-lang/issues/114](https://github.com/dhall-lang/dhall-lang/issues/114) — record-merge type-error pain
- [www.fbrs.io/dhall](https://www.fbrs.io/dhall/) — February 2025 review

### Jsonnet
- [jsonnet.org/learning/tutorial.html](https://jsonnet.org/learning/tutorial.html) — 2026
- [jsonnet.org/ref/language.html](https://jsonnet.org/ref/language.html) — 2026
- [jsonnet.org/ref/stdlib.html](https://jsonnet.org/ref/stdlib.html) — 2026
- [groups.google.com/g/jsonnet](https://groups.google.com/g/jsonnet/c/6OkYunT5iXI) — April 2025 (self/super confusion)

### Nix overlays
- [discourse.nixos.org/t/what-are-overlays/14680](https://discourse.nixos.org/t/what-are-overlays/14680) — 2026
- [discourse.nixos.org/t/infinite-recursion-when-composing-overlays/7594](https://discourse.nixos.org/t/infinite-recursion-when-composing-overlays/7594) — March 2025
- [discourse.nixos.org/t/in-overlays-when-to-use-self-vs-super/2968](https://discourse.nixos.org/t/in-overlays-when-to-use-self-vs-super/2968) — February 2026
- [discourse.nixos.org/t/reuse-attributes-sets-partly-in-different-parts-of-the-config/52339](https://discourse.nixos.org/t/reuse-attributes-sets-partly-in-different-parts-of-the-config/52339) — April 2026

### OpenAPI 3
- [spec.openapis.org/oas/v3.2.0.html](https://spec.openapis.org/oas/v3.2.0.html) — 2026
- [swagger.io/docs/specification/v3_0/data-models/oneof-anyof-allof-not](https://swagger.io/docs/specification/v3_0/data-models/oneof-anyof-allof-not/) — 2026
- [github.com/OAI/OpenAPI-Specification/issues/3069](https://github.com/OAI/OpenAPI-Specification/issues/3069) — July 2025 (allOf overriding ambiguity)
- [github.com/OAI/OpenAPI-Specification/issues/2143](https://github.com/OAI/OpenAPI-Specification/issues/2143) — January 2025 (discriminator deprecation discussion)

### GraphQL fragments
- [graphql.org/learn/queries/#fragments](https://graphql.org/learn/queries/#fragments) — 2026

### Sass
- [sass-lang.com/documentation/at-rules/extend](https://sass-lang.com/documentation/at-rules/extend/) — 2026
- [sass-lang.com/documentation/at-rules/mixin](https://sass-lang.com/documentation/at-rules/mixin/) — 2026
- [github.com/sass/sass/issues/324](https://github.com/sass/sass/issues/324) — August 2025 (selector-rewriting specificity)

### Tailwind
- [tailwindcss.com/docs/adding-custom-styles](https://tailwindcss.com/docs/adding-custom-styles) — 2026
- [tailwindcss.com/docs/theme](https://tailwindcss.com/docs/theme) — 2026

### Pulumi
- [pulumi.com/docs/concepts/stack](https://www.pulumi.com/docs/concepts/stack/) — 2026

### Jekyll / Hugo
- [gohugo.io/templates/lookup-order](https://gohugo.io/templates/lookup-order/) — 2026
- [jekyllrb.com/docs/layouts](https://jekyllrb.com/docs/layouts/) — 2026
- [gohugo.io/content-management/page-bundles](https://gohugo.io/content-management/page-bundles/) — 2026

### Storybook
- [storybook.js.org/docs/writing-stories](https://storybook.js.org/docs/writing-stories) — 2026
- [storybook.js.org/docs/writing-stories/parameters](https://storybook.js.org/docs/writing-stories/parameters) — 2026

### Typst
- [typst.app/docs/reference/styling](https://typst.app/docs/reference/styling/) — 2026
- [typst.app/docs/tutorial/making-a-template](https://typst.app/docs/tutorial/making-a-template/) — 2026
- [typst.app/docs/reference/scripting](https://typst.app/docs/reference/scripting/) — 2026

### Marp / Slidev
- [marpit.marp.app](https://marpit.marp.app/) — 2026
- [sli.dev/guide/syntax](https://sli.dev/guide/syntax) — 2026
- [sli.dev/themes/use](https://sli.dev/themes/use) — 2026
- [sli.dev/custom/index](https://sli.dev/custom/) — 2026

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity `perplexity_research` | 3 | (1) 8-ecosystem composition comparison (Helm/Kustomize/Cue/Dhall/Jsonnet/Nix/OpenAPI/Sass); (2) Atmos deep-dive (imports/catalogs/inherits/merge/templating); (3) 7-ecosystem slide & web composition (Typst/Marp/Slidev/Tailwind/Hugo/Pulumi/Storybook) |
| Perplexity `perplexity_reason` | 1 | Synthesis: "should slideforge adopt Atmos primitives?" with sketched syntax |
| Perplexity `perplexity_search` | 1 | Atmos GitHub recent issues / community pain points (March 2026 deep-merge regression confirmed) |
| Context7 | 0 | Atmos has no Rust SDK; no library docs needed |
| Tavily | 0 | Coverage from Perplexity was sufficient |
| WebFetch | 0 | Citations sourced from Perplexity research |
| WebSearch | 0 | Not needed |
| Training data | ~10% | General DSL design principles, indentation-significant parser concerns, basic Rust ecosystem knowledge — flagged explicitly |

**Total MCP tool calls:** 5

**Training data reliance:** Low. All non-trivial claims (Atmos behaviors, ecosystem comparisons, issue references) are cited to specific URLs and dated. Synthesis judgments (e.g., "this maps cleanly to slideforge") are explicitly framed as judgments, not facts.

**Inconclusive areas flagged:**
- Marp/Slidev pain-point citations rely on aggregated documentation and community-thread synthesis rather than specific issue numbers. The patterns (theme parameterization gaps, branding rigidity) are consistent across multiple secondary sources but a slide-DSL competitor analysis would benefit from primary GitHub issue lookups in a follow-up pass.
- The proposed slideforge syntax sketches are *illustrative*, not normative. The architect should refine them based on the chosen parser (`chumsky`) capabilities and the overall DSL grammar in `dsl-spec.md`.

---

*End of report.*
