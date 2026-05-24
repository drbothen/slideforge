---
title: "Q16-Q25 Decisions — Tier 3 (Polish)"
date: 2026-05-24
status: LOCKED
decided_by: human
---

# Q16-Q25 Decisions

## Q16: DSL versioning — LOCKED
Required `slideforge_version "1"` in deck metadata. Major-version grammar gating. Forward-incompatible error on mismatch. Backward-compatible within major. Enables future `slideforge migrate`.

## Q17: Strict vs. forgiving parsing — LOCKED
Three-tier severity: parse errors always fatal; validation errors fatal in strict (default build), warnings in warn-only (default watch); lint warnings always warnings. `--strict` and `--warn-only` CLI flags for explicit override.

## Q18: i18n / RTL — LOCKED
`lang "en-US"` required at deck level (defaults to "en"). Per-slide `lang:` override for multilingual. `dir: rtl` reserved for v2. CJK font declaration in brand.toml `[fonts] cjk = "..."`.

## Q19: Library / package model — LOCKED
Full Level 4 package model in v1.0:
- `slideforge package install github.com/org/package`
- `slideforge.toml` `[dependencies]` section
- `slideforge-package.toml` manifest per package (name, version, exports)
- `sf.lock` lockfile at workspace root
- Git-based primary (no hosted registry in v1.0)
- `@import "package-name/item"` for package content (distinct from @include)
- Unified with plugin system: packages can contain .sf content AND/OR plugin WASM modules
- New crate: `slideforge-package`
- New keyword: `@import` (resolves from installed packages)
- Cache: `~/.slideforge/packages/` (global) or `.slideforge/packages/` (project-local)

### Package manifest format (slideforge-package.toml)
```toml
[package]
name = "1898-slides"
version = "1.2.0"
description = "1898 & Co. standard slide catalog"
authors = ["Joshua Magady <josh.magady@1898.com>"]
license = "MIT"
repository = "https://github.com/1898/slides"
slideforge_version = "1"

[exports]
slides = ["catalog/*.sf"]
aliases = ["aliases/*.sf"]
defaults = ["defaults/*.sf"]
brand = "brand.toml"
assets = ["assets/*"]
sections = ["sections/*.sf"]

[dependencies]
# packages can depend on other packages
mssp-common = { git = "https://github.com/mssp-tools/common", tag = "v1.0.0" }
```

### Lockfile format (sf.lock)
```toml
[[package]]
name = "1898-slides"
version = "1.2.0"
source = "git+https://github.com/1898/slides?tag=v1.2.0"
commit = "abc123def456789"
checksum = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

[[package]]
name = "mssp-common"
version = "1.0.0"
source = "git+https://github.com/mssp-tools/common?tag=v1.0.0"
commit = "789def012345678"
checksum = "sha256:..."
```

### @import vs @include
| Directive | Resolves from | Use case |
|-----------|--------------|----------|
| `@include "path.sf"` | Local filesystem (relative to source, then include_paths) | Local files, project-internal sharing |
| `@import "package/item"` | Installed packages (sf.lock resolution) | Cross-project sharing, versioned packages |

## Q20: Hierarchical project config — LOCKED
Cargo-style explicit workspace + thin .sfconfig cascade:
- `slideforge.toml` at workspace root: `[workspace]` with `members`, `defaults`, `packages`, `variants`
- `.sfconfig` files at directory levels for family-specific overrides (bounded, max 3 levels)
- `slideforge config explain deck.sf` shows provenance of every resolved value
- Single `sf.lock` at workspace root
- `slideforge build --workspace` builds all members
- `slideforge build --workspace --changed` for CI optimization
- Research: R14 (planning/workspace-model-research.md)

### .sfconfig allowed fields
```toml
# Only these fields are allowed in .sfconfig (small surface area):
[defaults]
output_dir = "output/incidents/"
template = "templates/incident-template.pptx"
brand = "brands/incident-brand.toml"
lang = "en-US"

[set.severity_cards]
color_high = "red"
color_medium = "orange"
color_low = "green"

[set.chart]
color = "brand.primary"

[vars]
department = "Security Operations"
classification = "CONFIDENTIAL"

[variants.exec]
exclude_tags = ["internal"]
```

### slideforge config explain output
```
$ slideforge config explain incident-briefs/inc-2026-0320.sf

  brand = "brands/incident-brand.toml"    # from: .sfconfig (incident-briefs/)
  lang = "en-US"                          # from: workspace (slideforge.toml)
  output_dir = "output/incidents/"        # from: .sfconfig (incident-briefs/)
  set.severity_cards.color_high = "red"   # from: .sfconfig (incident-briefs/)
  set.severity_cards.color_medium = "orange" # from: .sfconfig (incident-briefs/)
  vars.department = "Security Operations" # from: .sfconfig (incident-briefs/)
  variants.exec.exclude_tags = ["internal"] # from: .sfconfig (incident-briefs/)
  packages.1898-slides = "v1.2.0"         # from: workspace (slideforge.toml)
```

## Q21: Defaults directory — LOCKED
Auto-apply (Hugo-style). Files in `defaults/` named `<slide_type>.sf` auto-apply as set rules to all workspace members. Precedence: bottom of the chain (any more-specific set wins). Opt-out: `ignore_defaults: [type]` in deck metadata. `slideforge config explain` shows auto-applied defaults with provenance.

## Q22: Reserved keywords — LOCKED
Full reserved keyword list (~30 keywords). Parser recognizes them and rejects with descriptive error. Includes: component, extends, inherits, raw, @fn, @mixin, @extend, @use, @let, @match, @loop, @while, code, video, audio, animation, transition, morph, zoom, presenter, theme, plugin, export, test. All v1.0 slide types (chart, toc, agenda, quote, grid, bio, diagram, team) are active keywords, not reserved.

## Q23: Error recovery — LOCKED
Accumulate ALL errors in one pass (chumsky's design). Never fail-on-first. Every error has file:line:col span + correction hint ("did you mean...?"). Watch mode renders error-slide placeholders (red slides with error text). Strict mode: no output. Warn-only: output with placeholders. Diagnostics rendered via miette/ariadne.

## Q24: Mixin syntax (v2 direction) — LOCKED
Recorded as normative for forward-compatibility:
- `mixin name(param, param = default):` for definition
- `@mixin name(param: value)` for invocation inside slide blocks
- Named parameters with defaults
- Mixins inject fields into calling slide (notes, takeaway, report, detail)
- Mixins CANNOT create new slide types (that's `component`)
- Keywords already reserved (Q22)

## Q25: Merge semantics — LOCKED
Last-wins scalars, REPLACE lists, deep-merge maps. NOT configurable.

Full 11-level precedence chain (highest wins first):
1. Slide-level field value
2. @mixin injected value (v2)
3. Alias set-rule value
4. Deck-level set-rule value
5. .sfconfig set-rule value
6. defaults/ auto-applied value
7. Variant vars override
8. Variant inherited vars (left-to-right, later wins)
9. Deck-level vars
10. Workspace defaults
11. Brand template default

One rule. Documented. Enforced. No per-field merge-strategy option.
