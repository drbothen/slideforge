---
title: Per-Directory Config + Workspace Models — Research
date: 2026-05-24
analyst: research-agent
status: foundation-research
audience: product-owner + architect + human-reviewer
---

# Workspace Model Research

## Executive Summary

**Recommended model: Explicit workspace manifest (Cargo-style) with a narrowly-constrained EditorConfig-style cascade as an optional convenience layer.** The primary config mechanism should be a `slideforge.toml` at the workspace root declaring members and shared defaults; per-deck overrides live in the deck's own front matter or a sidecar config. An optional per-directory `.sfconfig` file provides a thin "family defaults" layer for subdirectory trees (e.g., all quarterly reports inherit the same output settings), bounded by `root = true` semantics. This hybrid avoids the "Hugo problem" (unpredictable multi-axis cascade) while delivering the convenience of EditorConfig (simple, small-surface, one-axis cascade). The closest existing model is **Cargo workspaces** — explicit membership, opt-in inheritance, single lockfile at root.

---

## Tool Survey (14 tools)

### Cargo Workspaces

**How config inherits:**
- Root `Cargo.toml` declares `[workspace]` with `members = [...]` (explicit, supports globs).
- `[workspace.dependencies]` centralizes dependency specs; per-crate Cargo.toml opts in with `dep = { workspace = true }`.
- `[workspace.package]` shares metadata (version, edition, license, etc.); members opt in field-by-field with `version.workspace = true`.
- Single `Cargo.lock` at workspace root governs all members.

**Pain points:**
- Confusion that `[workspace.dependencies]` doesn't auto-apply — each crate must opt in explicitly.
- `default-features` cannot be overridden per-crate (set once at workspace level).
- Incompatible dependency trees force crates to be excluded from the workspace.
- Path resolution for inherited `readme`/`license-file` is relative to the root, not the member.

**What users like:**
- Unambiguous: "check root Cargo.toml, then my crate's Cargo.toml, done."
- Single lockfile prevents version drift.
- Opt-in inheritance means no surprises.
- Explicit membership means you always know what's in the workspace.

### npm/pnpm/yarn Workspaces

**How config inherits:**
- Root `package.json` declares `"workspaces": ["packages/*"]`.
- Dependencies declared at root are hoisted to root `node_modules`; per-package `package.json` declares package-specific deps.
- Node's module resolution walks up directories, finding hoisted deps implicitly.

**Pain points:**
- **Phantom dependencies**: packages use deps they never declared (only works because of hoisting).
- **Hoisting confusion**: users can't tell which version is actually in use; changing one package's deps can break another.
- **Publish breakage**: works in monorepo, fails when package is installed standalone.
- pnpm's strict mode surfaces these as errors, but migration is painful.

**What users like:**
- Single lockfile at root.
- `workspace:*` protocol (pnpm) for inter-package references.
- Familiar model for JS developers.

### Turborepo

**How config inherits:**
- Root `turbo.json` defines pipeline tasks (build, test, lint) with inputs/outputs/caching.
- Per-package `turbo.json` can `"extends": ["//"]` to inherit from root, then override task config.
- Array fields now support `$TURBO_EXTENDS$` for appending rather than replacing.

**Pain points:**
- Before `extends`, config was repetitive across packages.
- Array replacement semantics (pre-2.7) forced full restating of inputs/outputs.
- Pipeline ordering complexity for complex inter-package dependencies.

**What users like:**
- Clear "root defines shared, package overrides specifics" mental model.
- `extends` makes inheritance explicit and opt-in.

### Nx

**How config inherits:**
- Root `nx.json` defines `targetDefaults` — shared task configuration (inputs, cache, dependsOn).
- Per-project `project.json` defines project-specific targets.
- Nx merges targetDefaults with project-level targets at resolution time.
- Plugin-based target inference can auto-generate targets.

**Pain points:**
- **Implicit inference can be surprising**: plugins generate targets that may ignore or conflict with targetDefaults.
- Hard to opt out of inferred targets for specific projects.
- Users struggle to know whether a target came from inference, targetDefaults, or project.json.
- Unexpected caching behavior when inference and defaults interact.

**What users like:**
- Powerful for large monorepos with many similar projects.
- targetDefaults reduce boilerplate significantly.

### Bazel

**How config inherits:**
- `WORKSPACE` (or `MODULE.bazel` in Bzlmod) at root declares external dependencies and toolchains.
- Per-package `BUILD` files declare targets with fully explicit `srcs`, `deps`, `data`.
- **No inheritance between BUILD files.** Shared patterns are encoded in Starlark macros/rules.
- Strict dependency enforcement: fails if an import isn't in declared `deps`.

**Pain points:**
- Extreme verbosity — every dependency must be explicit.
- Steep learning curve for Starlark rules.
- Requires code generation tools (Gazelle) to remain practical.

**What users like:**
- **Unambiguous**: "look at this BUILD file, that's what this target uses."
- Reproducible and hermetic builds.
- No "spooky action at a distance" from parent configs.
- Scales to enormous codebases (Google's monorepo).

### Go Modules + go.work

**How config inherits:**
- Each module has its own `go.mod` — fully self-contained dependency contract.
- `go.work` is a thin overlay: declares which local modules are "in the workspace" via `use ./path`.
- Workspace-wide `replace` directives override per-module replaces during local dev.
- **No config inheritance by design.** `go.work` affects build behavior, not `go.mod` contents.
- `go work sync` propagates resolved versions back into individual `go.mod` files explicitly.

**Design principles:**
- Preserve module isolation and reproducibility.
- Workspace is a local-dev convenience, not a published contract.
- "No inheritance" avoids hidden coupling — a module's `go.mod` is always the full description of its requirements.
- Minimal surface area: only `go`, `use`, `replace` directives.

**Pain points:**
- Very minimal — some users want more workspace features (shared tooling config, etc.).
- `go.work` is intentionally not committed in many workflows (local-only dev tool).

**What users like:**
- Dead-simple mental model: "these modules work together locally."
- Each module remains independently publishable/buildable.
- `GOWORK=off` to escape the workspace instantly.

### Pants

**How config inherits:**
- `pants.toml` at repo root sets global defaults (interpreter constraints, backends, tool config).
- Per-directory `BUILD` files declare targets (not defaults for subdirectories).
- BUILD files are NOT hierarchical — no parent-child relationship between BUILD files.
- Global defaults from `pants.toml` apply unless a specific target overrides a field.

**Pain points:**
- Users expect BUILD files to be hierarchical (parent sets defaults for children) — they aren't.
- Dependency inference is powerful but opaque when it fails.
- Steep learning curve for initial configuration.
- Deciding what goes in `pants.toml` vs tool-native config (pyproject.toml, etc.).

**What users like:**
- Clear separation: `pants.toml` = global config, BUILD = per-target declarations.
- Dependency inference eliminates most explicit dep declarations.

### Gradle

**How config inherits:**
- `settings.gradle(.kts)` declares subprojects via `include(...)`.
- Root `build.gradle(.kts)` historically used `allprojects {}` / `subprojects {}` to inject config.
- Modern pattern: **convention plugins** — shared config extracted into a plugin that projects opt into.
- `gradle.properties` cascades (root > subproject, each level can override).

**Pain points:**
- `allprojects`/`subprojects` creates cross-project coupling; breaks configuration cache.
- New users confused about what goes in settings.gradle vs build.gradle.
- Kotlin DSL is type-safe but slow to reimport and has cryptic errors.
- Multiple "right ways" to share config (buildSrc vs includedBuild vs precompiled script plugins).

**What users like:**
- Convention plugins: explicit opt-in, composable, local-first.
- `gradle.properties` hierarchy is simple and predictable.
- Very powerful for complex multi-project builds.

### Maven

**How config inherits:**
- Parent-child POM inheritance: child declares `<parent>` with GAV coordinates.
- Parent provides `<dependencyManagement>` (version constraints, not actual deps), `<pluginManagement>`, properties.
- Child inherits everything from parent unless explicitly overridden.
- Resolution: local filesystem first (`../pom.xml`), then repository.

**Pain points:**
- Deep inheritance trees make it hard to trace where a plugin config originates.
- "God parent" POMs accumulate bloat and become risky to change.
- `<relativePath>` vs repository resolution causes "works on my machine" issues.
- `mvn help:effective-pom` is necessary but newcomers rarely know it.
- XML verbosity.

**What users like:**
- `<dependencyManagement>` pattern (declare version once, use everywhere without version) is well-understood.
- BOMs (bill of materials) for version coordination across organizations.
- Clear single-parent inheritance model.

### ESLint Cascading (.eslintrc)

**How config inherits:**
- Per-directory `.eslintrc.*` files; ESLint searches up the directory tree.
- Configuration cascade: closest file to the linted file has highest priority.
- `root: true` stops upward search.
- Rules are merged: child extends parent, child overrides win.

**Pain points:**
- Users accidentally inherit from ancestor directories outside their project (home directory `.eslintrc`).
- `root: true` is a best practice but wasn't obvious — many projects shipped without it.
- In monorepos, per-package cascading interacts with workspace tooling (editor integration, CI).
- ESLint moved to "flat config" in v9 (2024) partly because cascading was too confusing.

**What users like:**
- Per-directory overrides for specific patterns (tests get different rules).
- Simple for small projects.
- `root: true` provides a clear boundary once you know about it.

**Key lesson:** ESLint's cascade worked *okay* but was confusing enough that the project **deprecated it** in favor of flat config. The cascade model has a ceiling.

### EditorConfig Cascading (.editorconfig)

**How config inherits:**
- Per-directory `.editorconfig` files; editor plugins search up to filesystem root or `root = true`.
- Properties in closer files take precedence.
- Simple INI format with glob-based section headers.

**Pain points:**
- Very few. The model is universally praised.
- No `extends` mechanism (feature request #236, still open since 2015).
- Limited to editor/formatting settings — intentionally small scope.

**What users like:**
- **Universally praised** as the gold standard of cascading config.
- Tiny surface area (indent style, size, charset, EOL, trim whitespace).
- Low-risk: wrong config = funny indentation, not broken builds.
- `root = true` provides clear boundary.
- Single file format, single file name, no alternative sources.

**Why it works:** EditorConfig controls only cosmetic editor behavior. The domain is small, impact is low-risk, and there's exactly one axis of resolution (directory tree). This is the key insight.

### Docker Compose Override

**How config inherits:**
- Base `docker-compose.yml` + `docker-compose.override.yml` (auto-loaded if present).
- Additional files via `-f` flag or `COMPOSE_FILE` env var; later files override earlier.
- Maps merge recursively (key-by-key); scalars are replaced; **lists are replaced entirely** (not appended).

**Pain points:**
- **List replacement is the #1 surprise**: adding a `ports` entry in override replaces ALL ports from base.
- Path resolution relative to first file, not the override file's location.
- Users forget the implicit `override.yml` is loaded; conflicts with explicit `-f` stacks.
- Complex stacks (base + dev + local + CI) are hard to reason about.
- `docker compose config` needed to see final merged result, but few know this.

**What users like:**
- Simple two-file model (base + override) for dev vs prod.
- `docker compose config` as a "show me the final result" tool.

---

## Per-Directory Cascade vs. Explicit Workspace

| Dimension | Per-directory cascade (EditorConfig model) | Explicit workspace (Cargo model) |
|-----------|-------------------------------------------|----------------------------------|
| Config discovery | Walk up directory tree | Declared in root manifest |
| Inheritance | Implicit (file existence = it applies) | Explicit (opt-in with `workspace = true`) |
| Membership | Implicit (any .sf file found) | Declared (`members = [...]`) |
| Ambiguity | "Which config applies here?" | Unambiguous — two places to look |
| Boilerplate | Low (inherit everything by default) | Medium (must declare what to inherit) |
| Predictability | Depends on surface area | High by construction |
| Debuggability | Needs "explain" tool to trace provenance | Check root manifest + deck config |
| Build command | Walk tree, find .sf files | `slideforge build --workspace` |
| Lockfile scope | Per-directory? Per-root? Unclear | Single lockfile at workspace root |
| Scale ceiling | ~50 items before it gets confusing | Hundreds of members (proven at scale) |

### When cascade works (EditorConfig pattern):
- **Small surface area** — few properties, all cosmetic/behavioral
- **Low-risk impact** — wrong value = minor visual difference, not broken output
- **Single axis** — only directory tree, no other dimensions
- **Clear boundary** — `root = true` stops search
- **Single source type** — only `.editorconfig` files, no alternatives

### When cascade fails (Hugo pattern):
- **Large surface area** — many properties controlling core behavior
- **High-risk impact** — wrong value = broken output, wrong URLs, missing content
- **Multiple axes** — directory + theme + language + environment
- **Multiple source types** — TOML + YAML + JSON + front matter + CLI flags
- **Complex merge rules** — different fields merge differently
- **No "explain" command** — users can't trace provenance

### The slideforge question:
slideforge config controls output behavior (brand, output_dir, variants, packages) — this is **high-impact, medium-surface-area**. It sits between EditorConfig (low impact) and Hugo (enormous impact). A pure cascade would need careful constraints to avoid the Hugo problem.

---

## The slideforge Use Case

### Concrete scenario: 1898 & Co. multi-deck organization

```
1898-slides/
├── slideforge.toml              # WORKSPACE ROOT
├── brand.toml                   # corporate brand (shared)
├── sf.lock                      # single lockfile for workspace
├── catalog/                     # shared slide library (installed packages + local)
├── quarterly-reviews/           # deck family 1
│   ├── .sfconfig                # (optional) family-level defaults
│   ├── q1-2026.sf
│   ├── q2-2026.sf
│   └── q3-2026.sf
├── incident-briefs/             # deck family 2
��   ├── .sfconfig                # different output_dir, different variant set
│   ├── inc-2026-0320.sf
│   └── inc-2026-0415.sf
├── client-reports/              # deck family 3 (per-client sub-families)
│   ├── .sfconfig                # all client reports share DOCX + PDF output
│   ├── acme/
│   │   ├── .sfconfig            # Acme-specific brand overlay
│   │   └── monthly-2026-03.sf
│   └── globex/
│       └── monthly-2026-03.sf
└── training/                    # deck family 4
    ├── onboarding.sf
    └── security-awareness.sf
```

### Questions answered:

**Do all decks share one brand.toml?**
Yes, by default. The workspace manifest declares `brand = "brand.toml"` in `[workspace.defaults]`. A family (or individual deck) can override with `brand_overlay = "acme-brand.toml"` in `.sfconfig` or deck front matter.

**Do all decks share one output_dir?**
Default: `output_dir = "dist/"` at workspace level. Families override: incident-briefs might use `output_dir = "dist/incidents/"`. The effective path is relative to the workspace root.

**Can `slideforge build` build ALL decks?**
Yes. `slideforge build --workspace` builds all declared members. `slideforge build quarterly-reviews/` builds one family. `slideforge build quarterly-reviews/q1-2026.sf` builds one deck.

**How do packages interact with workspaces?**
Single `sf.lock` at workspace root. Workspace-level `[packages]` declares shared package deps. Per-deck can add additional packages (merged). Cache is workspace-level (one `.sf-cache/` dir).

---

## Recommended Model

### Cargo-style workspace with EditorConfig-style family defaults

The recommended model has **three layers** with fully predictable merge order:

```
Layer 1: Built-in defaults (hardcoded in slideforge binary)
Layer 2: [workspace.defaults] in slideforge.toml (explicit, root-level)
Layer 3: .sfconfig in directory tree (optional, bounded cascade)
Layer 4: Deck-level config (front matter in .sf file)
```

Merge semantics (already locked from Q1): last-wins scalars, replace lists, deep-merge maps.

### Root manifest: `slideforge.toml`

```toml
[workspace]
members = [
  "quarterly-reviews/*.sf",
  "incident-briefs/*.sf",
  "client-reports/**/*.sf",
  "training/*.sf",
]

# Explicit shared defaults — Cargo's [workspace.package] equivalent
[workspace.defaults]
brand = "brand.toml"
output_dir = "dist/"
output_formats = ["pptx", "pdf"]
slideforge_version = "1.0"

# Workspace-level shared variables
[workspace.defaults.vars]
company_name = "1898 & Co."
fiscal_year = "FY2026"
confidentiality = "Company Confidential"

# Workspace-level shared set rules
[workspace.defaults.set]
title.footer = "{{ company_name }} | {{ confidentiality }}"
title.logo = "assets/logo.svg"

# Workspace-level shared variants
[workspace.defaults.variants]
exec = { set = { "*.detail" = "hidden" } }
tech = { set = { "*.notes" = "visible" } }

# Workspace-level package dependencies
[workspace.packages]
"github.com/1898co/slide-catalog" = "^2.1"
"github.com/slideforge/charts-extra" = "^1.0"
```

### Family-level defaults: `.sfconfig` (optional cascade)

```toml
# incident-briefs/.sfconfig
# Applies to all .sf files in this directory and subdirectories
output_dir = "dist/incidents/"
output_formats = ["pptx", "pdf", "docx"]

[vars]
report_type = "Incident Brief"

[set]
title.accent_color = "alert_red"
```

### Cascade rules (avoiding the Hugo problem):

1. **Single file name**: only `.sfconfig` — no alternative names, no YAML/JSON alternatives.
2. **Bounded search**: cascade walks up ONLY within the workspace root (never above `slideforge.toml`).
3. **Shallow by default**: maximum 3 levels of `.sfconfig` nesting (configurable, default 3).
4. **`root = true`**: any `.sfconfig` with `root = true` stops the upward search.
5. **Small surface area**: `.sfconfig` can ONLY contain: `output_dir`, `output_formats`, `brand`, `brand_overlay`, `[vars]`, `[set]`, `[variants]`, `[packages]`. It CANNOT contain workspace membership, build configuration, or tool settings.
6. **Provenance command**: `slideforge config explain path/to/deck.sf` prints the resolved config with provenance annotations showing which layer contributed each value.
7. **Apply order**: workspace root `.sfconfig` (if any) -> intermediate directory `.sfconfig` files -> leaf directory `.sfconfig` -> deck front matter.

### Why this avoids the Hugo problem:

| Hugo failure mode | slideforge mitigation |
|---|---|
| Multiple config formats (TOML/YAML/JSON) | Single format: TOML only, single filename |
| Multiple axes (theme + lang + env + section) | Single axis: directory tree only |
| High-impact settings in cascade | High-impact settings restricted to workspace manifest |
| No explain command | `slideforge config explain` shows provenance |
| Unbounded search (to filesystem root) | Bounded: stops at workspace root |
| Complex merge rules vary by field | Uniform merge: last-wins / replace / deep-merge |

---

## Workspace Behavior Specifics

### Build behavior

| Command | Behavior |
|---------|----------|
| `slideforge build` | Build the .sf file in current directory (or error if multiple) |
| `slideforge build deck.sf` | Build one specific deck |
| `slideforge build --workspace` | Build ALL declared workspace members |
| `slideforge build --workspace quarterly-reviews/` | Build all members matching the path prefix |
| `slideforge build --workspace --parallel` | Build all members in parallel |
| `slideforge build --workspace --changed` | Build only members with changed inputs since last build |

### Shared vars

```toml
# Workspace-level vars are available to all decks
[workspace.defaults.vars]
company = "1898 & Co."

# Per-family .sfconfig can add/override vars
[vars]
team = "Engineering"

# Deck front matter can add/override further
# deck:
#   vars:
#     author: "Jane Smith"
```

Effective vars for a deck: workspace.defaults.vars deep-merged with .sfconfig chain vars, deep-merged with deck vars.

### Shared set rules

```toml
# Workspace-level set rules apply to all decks
[workspace.defaults.set]
title.footer = "{{ company }}"

# Family .sfconfig can add/override
[set]
title.accent_color = "navy"

# Deck can override further in its set: block
```

### Shared variants

```toml
# Workspace-level variants available to all decks
[workspace.defaults.variants]
exec = { set = { "*.detail" = "hidden" } }
tech = { set = { "*.notes" = "visible" } }

# Family can add variants (deep-merge)
[variants]
client = { set = { "*.internal_notes" = "hidden" } }

# Deck can add/override variants in front matter
```

### Lockfile scope

- **Single `sf.lock` at workspace root.** All decks in the workspace share one lockfile.
- Rationale: same as Cargo — one lockfile prevents version drift, ensures reproducible builds.
- `slideforge package install` operates at workspace level (installs to workspace cache).
- `slideforge package install --deck deck.sf` adds a package only to that deck's dependency list.

### Package cache scope

- **Single `.sf-cache/` at workspace root** — shared across all decks.
- Packages are content-addressed (like pnpm's store) — no duplication even if multiple decks use different versions.
- `slideforge package list --workspace` shows all packages used across the workspace.

---

## How It Interacts with Packages (Q19)

The package model (git-based, with lockfile) maps cleanly onto the workspace model:

| Concern | Workspace-level | Per-deck |
|---------|----------------|----------|
| Declare dependencies | `[workspace.packages]` | `packages:` in deck front matter |
| Lockfile | `sf.lock` at workspace root | N/A (single lockfile) |
| Cache | `.sf-cache/` at workspace root | N/A (shared cache) |
| Install command | `slideforge package install` | `slideforge package install --deck x.sf` |
| Resolution | Workspace-level packages + per-deck packages merged | Per-deck sees union of both |

**Interaction semantics:**
1. Workspace-level packages are available to ALL decks (like Cargo's `[workspace.dependencies]`).
2. Per-deck packages are additional (additive merge — union of workspace + deck packages).
3. Version conflicts: if workspace declares `catalog ^2.0` and a deck declares `catalog ^3.0`, this is an error (like Cargo). The deck must explicitly override with a comment explaining why.
4. Per-deck packages still use the workspace lockfile — they add entries to `sf.lock`.

---

## How It Interacts with Variants (Q11)

Variant inheritance already decided in Q11 (multiple inheritance). The workspace adds one more layer:

```
Workspace variants (available to all decks)
  └── Family variants (.sfconfig — additive)
       └── Deck variants (deck front matter — additive + can override)
```

| Behavior | Workspace variants | Family variants | Deck variants |
|----------|-------------------|-----------------|---------------|
| Availability | All decks see them | Decks in that family see them | Only that deck |
| Override | Cannot be overridden by default | Can override workspace variants | Can override all |
| Opt-out | Deck can set `variants.exec = false` to disable | Same | N/A |

**Example flow:**
```toml
# slideforge.toml
[workspace.defaults.variants]
exec = { set = { "*.detail" = "hidden" } }

# incident-briefs/.sfconfig
[variants]
urgent = { set = { title.accent_color = "red" } }

# incident-briefs/inc-2026-0320.sf (deck front matter)
# variants:
#   exec: false          # opt out of workspace variant
#   classified:          # add deck-specific variant
#     set:
#       title.footer: "CLASSIFIED"
```

---

## Sources

### Primary research (MCP tools)
1. Cargo workspaces — doc.rust-lang.org/cargo/reference/workspaces.html (via Context7)
2. Cargo workspace inheritance RFC — rust-lang.github.io/rfcs/1525-cargo-workspace.html
3. Go workspaces — go.dev/blog/get-familiar-with-workspaces, go.dev/doc/tutorial/workspaces
4. ESLint config cascade — eslint.org/blog/2022/08/new-config-system-part-1/
5. ESLint flat config discussion — github.com/eslint/eslint/discussions/16960
6. EditorConfig spec — editorconfig.org
7. Docker Compose merge — docs.docker.com/compose/how-tos/multiple-compose-files/merge/
8. Pants initial configuration — pantsbuild.org/dev/docs/getting-started/initial-configuration
9. Turborepo extends — turborepo.dev/blog/turbo-2-7
10. Nx targetDefaults — github.com/nrwl/nx/issues/26708, github.com/nrwl/nx/discussions/22630
11. npm hoisting issues — github.com/npm/cli/issues/4512, github.com/npm/cli/issues/4056
12. pnpm workspace protocol — divriots.com/blog/switching-to-pnpm/
13. Gradle convention plugins — tech.bedrockstreaming.com/2023/07/07/gradle-convention-plugins.html
14. Maven parent POM — baeldung.com/maven-relativepath
15. Bazel deps model — bazel.build/basics/dependencies
16. Deno workspaces — docs.deno.com/runtime/fundamentals/workspaces
17. Cargo autoinherit — mainmatter.com/blog/2024/03/18/cargo-autoinherit/

### Design pattern references
18. Rush monorepo docs on phantom dependencies — rushjs.io/pages/maintainer/package_managers/
19. ESLint "new config system" blog (deprecating cascade) — eslint.org/blog/2022/08/new-config-system-part-1/
20. Hugo config directory docs — gohugo.io/getting-started/configuration/

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 1 | Deep comparison of workspace/monorepo config models across 9 build tools |
| Perplexity perplexity_ask | 5 | Cargo workspace details, Go go.work details, npm/pnpm hoisting, Turborepo/Nx inheritance, Docker Compose/Pants/Hugo |
| Perplexity perplexity_search | 2 | ESLint cascading problems, EditorConfig cascade design |
| Perplexity perplexity_reason | 1 | Synthesis: which model fits slideforge's use case |
| Context7 | 2 | Cargo workspace documentation (resolve-library-id + query-docs) |
| Tavily tavily_search | 1 | Monorepo config inheritance best practices |
| Training data | 2 areas | Hugo-specific failure modes (general knowledge), Docker Compose merge semantics |

**Total MCP tool calls:** 12
**Training data reliance:** low — all major claims verified against web sources; Hugo analysis synthesized from multiple ESLint/EditorConfig comparisons and the user's own R7 findings.
