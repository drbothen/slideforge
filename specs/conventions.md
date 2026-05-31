---
title: Slideforge Conventions
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.3
created: 2026-05-23
updated: 2026-05-31
status: SEED-EXTRACT
changelog:
  - version: 1.3
    date: 2026-05-31
    ref: OBS-2 correction (Wave 3 gate fix — stale-checkout reading removed)
    change: >
      Removed erroneous "Code state" column and CODE GAP ALERT blockquote from the
      plugin-api enum table. That column was written from a stale checkout of the develop
      branch (without the wave3-gate-fix commits) and incorrectly reported 10 code gaps.
      All 11 PATH-A enums carry #[non_exhaustive] on the fix branch; DiagnosticSeverity
      correctly lacks it. The table is now policy-only (Enum / Kind / PATH / policy YES/NO);
      point-in-time code conformance is verified by the adversary and clippy, not recorded
      in the spec.
  - version: 1.2
    date: 2026-05-31
    ref: OBS-2 (Wave 3 gate fix pass 7 — DiagnosticSeverity unlisted)
    change: >
      Added exhaustive plugin-api public enum classification table with scope statement.
      Enumerates all 12 public enums in slideforge-plugin-api with PATH A/B, policy, and
      actual code attribute state. Adds scope statement making table robust against future
      "unlisted public enum" findings.
  - version: 1.1
    date: 2026-05-31
    ref: OBS-1 (Wave 3 gate fix F-W3G-P4-001)
    change: Added ColorValue to #[non_exhaustive] classification table (PATH A / YES; STORY-072 Gradient variant planned).
---

# Slideforge Conventions

> Source: §8 (Conventions) from PROJECT-SEED.md. Verbatim extraction.

---

## Section 8: Conventions

### Code style

- **`rustfmt`** with project config (`rustfmt.toml`): `edition = "2024"`, `max_width = 100`, `tab_spaces = 4`
- **`clippy::pedantic`** enabled with documented exceptions
- **Public APIs** documented with rustdoc; `#![warn(missing_docs)]` on public modules
- **No `unsafe`** in any crate (FFI bindings are the only exception, in `slideforge-ffi`)
- **No `unwrap()`** outside of tests and clearly-infallible main paths; use `?` and proper error types

### Error handling

- Crate-level error enums using `thiserror`
- Display impls for human-readable messages
- All parser/eval errors carry source spans (line, column, range)
- CLI errors render via `miette` for nice colored output with source pointers

### Testing

- **Unit tests** in each crate's `src/` (next to the code, in `#[cfg(test)] mod tests`)
- **Snapshot tests** for parser AST output and IR (use `insta` crate)
- **Integration tests** in `tests/` for end-to-end CLI behavior
- **Fixture tests** in `tests/fixtures/` — expected `.pptx` output files for diff comparison
- **CI runs all tests** on push/PR via GitHub Actions

### Documentation

- **`README.md`** — project overview, quick start, link to full docs
- **`docs/`** — multi-file documentation including DSL reference, slide type catalog, architecture deep-dive
- **Rustdoc** — published to `docs.rs/slideforge` automatically on crate publish
- **`CHANGELOG.md`** — Keep-a-Changelog format

### Release process

- **Semver:** `0.x` for pre-stable. `1.0` when DSL syntax is frozen and major slide types are stable.
- **Tags:** `vX.Y.Z` (git tags)
- **GitHub Releases** with auto-built binaries
- **crates.io** publishing for library crates
- **Homebrew tap** for the CLI binary

### Commit conventions

- Conventional Commits format: `<type>(<scope>): <description>`
- Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `perf`, `ci`
- Scopes: `syntax`, `eval`, `layout`, `pptx`, `cli`, `validate`, `docs`, `ci`
- No AI attribution lines in commits (no `Co-Authored-By: Claude` etc.)

### Repository conventions

- Default branch: `main`
- Protected: PRs required for changes to `main`, CI must pass
- Issues use templates for: bug report, feature request, slide type proposal
- Discussions enabled for DSL syntax debates

### `#[non_exhaustive]` Policy (OBS-1)

Codified after finding F-W3G-P3-001 (Wave 3 integration-gate adversarial review).
Revised after finding F-W3G-P4-001 (Wave 3 gate fix) to align policy with the plugin-first
architecture: the extension model determines whether `#[non_exhaustive]` is appropriate,
not merely whether an enum is "plugin-related".

#### The architectural question to ask first

> **"Does a new `X` get added by adding a VARIANT to the built-in enum, or by
> registering a PLUGIN through the trait system?"**

This question determines the correct path:

**PATH A — variant-growth:** The enum's variant set grows when new capabilities ship.
External code that pattern-matches the enum must handle new variants. → **Apply `#[non_exhaustive]`.**

**PATH B — plugin-extension (closed catalog):** New capability is added via the plugin
trait system (implementing a trait, calling `register_X`). The built-in enum is a fixed
catalog of the bundled set. Extension is orthogonal to the enum. → **Do NOT apply
`#[non_exhaustive]`**; exhaustive match arms are a correctness feature, not a burden.

#### Classification for this codebase

**Scope statement:** The table below exhaustively enumerates every public enum in
`slideforge-plugin-api` — the external plugin-author SemVer surface that external
crate authors pattern-match against. Every public plugin-api enum is classified
here; a reviewer who finds an unlisted plugin-api enum has found a table gap and
should file a finding. For public enums in OTHER crates (e.g., `slideforge-data`,
`slideforge-brand`, `slideforge-charts`), the classification RULES above are
authoritative — they need not be individually listed here, though notable
cross-crate examples are included below for illustrative purposes. This table
states POLICY only; point-in-time code conformance is verified by the adversary
and clippy, not recorded here.

##### `slideforge-plugin-api` — exhaustive public enum catalog

| Enum | Source file | Kind | PATH | `#[non_exhaustive]` per policy | Rationale |
|------|-------------|------|------|--------------------------------|-----------|
| `BrandSource` | `traits/brand_provider.rs` | Data | PATH A | YES | External `BrandProvider` plugins match this in their `load()` impl; new source types (e.g., `ApiEndpoint`, `GitRepo`) require new variants; external plugins need the wildcard-arm obligation |
| `BrandError` | `traits/brand_provider.rs` | Error | PATH A | YES | Error enum grows with new diagnostic variants in minor releases; standard SemVer hygiene |
| `ChartError` | `traits/chart_renderer.rs` | Error | PATH A | YES | Error enum grows with new diagnostic variants; standard SemVer hygiene |
| `DataSourceError` | `traits/data_source.rs` | Error | PATH A | YES | Error enum grows with new diagnostic variants; doc-comment explains forward-compat obligation |
| `DiagramError` | `traits/diagram_renderer.rs` | Error | PATH A | YES | Error enum grows with new diagnostic variants; standard SemVer hygiene |
| `ExportError` | `traits/exporter.rs` | Error | PATH A | YES | Error enum grows with new diagnostic variants; standard SemVer hygiene |
| `InlineOutputFormat` | `traits/inline_format.rs` | Data | PATH A | YES | External `InlineFormat` plugins match this in their `render()` impl; new inline formats require new variants; external plugins need the wildcard-arm obligation |
| `InlineError` | `traits/inline_format.rs` | Error | PATH A | YES | Error enum grows with new diagnostic variants; standard SemVer hygiene |
| `MathOutputFormat` | `traits/math_renderer.rs` | Data | PATH A | YES | External `MathRenderer` plugins match this in their `render()` impl; new output formats (e.g., Epub) require new variants; external plugins need the wildcard-arm obligation |
| `MathError` | `traits/math_renderer.rs` | Error | PATH A | YES | Error enum grows with new diagnostic variants; standard SemVer hygiene |
| `LayoutError` | `traits/slide_type.rs` | Error | PATH A | YES | Error enum grows with new diagnostic variants; standard SemVer hygiene |
| `DiagnosticSeverity` | `traits/validator.rs` | Data | PATH B | NO | Closed 3-variant severity domain (Error/Warning/Info); the validator plugin trait uses `DiagnosticSeverity` in `Diagnostic` records but the severity set is a fixed taxonomy; new severity levels are deliberate breaking changes; exhaustive match is a correctness feature for this closed domain |

##### Cross-crate examples (not exhaustive — governed by rules above)

| Enum | Location | Extension model | `#[non_exhaustive]`? | Rationale |
|------|----------|-----------------|----------------------|-----------|
| `DataFormat` | `slideforge-data` | PATH A — format-detection registry; new file formats add new variants; `Unknown` sentinel for "no extension" case | YES | Legitimate extension with `Unknown` sentinel; wildcard error arms in all consumers |
| `ChartType` | `slideforge-charts` | PATH B — internal to the bundled Plotters renderer; new chart plugins implement `ChartRenderer` trait with `id()` string, NOT by adding variants; `ChartRenderer::render(spec, brand)` receives `ChartSpec`, not `ChartType` | NO | Closed 7-type catalog for the built-in renderer; exhaustive match is correct |
| `DiagramLang` | `slideforge-diagrams` | PATH B — internal to the bundled Mermaid renderer; new diagram plugins implement `DiagramRenderer` trait with `id()` string; `DiagramRenderer::render(source, opts)` receives raw `&str`, not `DiagramLang` | NO | Closed 1-type catalog (Mermaid v1.0); exhaustive match is correct |
| `OutputFormat` | `slideforge-layout` | PATH B — section target-format bitset; 5 output formats are fixed (Docx, Html, Pdf, Pptx, Preview); new exporters register with `id()`/`extension()` strings, not new `OutputFormat` variants | NO | Closed 5-format catalog; exhaustive match is correct |
| `LogoAsset` | `slideforge-brand` | PATH B — closed domain; adding a variant is a deliberate breaking change | NO | Closed domain; exhaustive match is a correctness feature |
| `ColorValue` | `slideforge-brand` | PATH A — variant-growth; STORY-072 (Gradient Fills) adds a `Gradient` variant; all current consumers are within `slideforge-brand` but external crates (exporters, validators) will match this enum | YES | Future `Gradient` variant is planned; wildcard arms in external consumers MUST return `Err(...)` — never a silent color fallback (R1 forbidden pattern, CLAUDE.md) |

#### Apply `#[non_exhaustive]` to

| Category | Rationale |
|----------|-----------|
| All public error enums | New diagnostic variants are routine minor-release additions; downstream callers should not need to update match arms |
| Plugin-API data enums whose variants appear as PARAMETERS in trait method signatures that external plugins must match | External plugin authors are forced to handle new variants; adding a variant is a minor-release addition |
| Non-error data enums that are open registries with an `Unknown`/sentinel variant | Open detection registries where new entries are expected (example: `DataFormat` with `Unknown` sentinel) |

#### Do NOT apply `#[non_exhaustive]` to

| Category | Rationale |
|----------|-----------|
| Internal bundled-catalog enums where extension is via the plugin trait system | The enum is a fixed catalog of bundled implementations; new plugins register via `id()` string, not by adding variants (examples: `ChartType`, `DiagramLang`, `OutputFormat`) |
| Closed-domain data enums with no planned extension | Adding a variant is a deliberate breaking change; exhaustive match is a feature (example: `LogoAsset`) |
| Internal enums not in the public API | No SemVer benefit inside a single crate |

#### Wildcard arm rule — applies whenever `#[non_exhaustive]` is on a non-error data enum

Wildcard arms MUST propagate an explicit `Err(...)` — **never** supply a silent fallback
value. Returning a default color, default path, or any fabricated value from a wildcard
arm is defect-equivalent to the Python reference's silent TEAL color fallback (R1 finding,
CLAUDE.md Forbidden Patterns table). If no valid fallback exists, return a descriptive error
variant (e.g., `MathError::UnsupportedFormat`, `BrandError::SourceNotFound`).

**Within-crate `Display` impls:** When the `Display` impl is in the same crate that declares
the `#[non_exhaustive]` enum, the compiler can see all variants and the match is exhaustive.
Do NOT add a wildcard arm to `Display` impls in the defining crate — it creates an unreachable
pattern and a compiler warning. The wildcard-arm obligation applies only to EXTERNAL crates.

#### Checklist for every new public enum

- [ ] Error enum? → Apply `#[non_exhaustive]`; brief extensibility justification in doc-comment.
- [ ] Plugin-API data enum used as a trait method parameter that external plugins must match?
  → Apply `#[non_exhaustive]`; explain the extension model in the doc-comment.
- [ ] Internal bundled-catalog enum where extension is via plugin `id()` string?
  → Do NOT apply; state "closed catalog — extension via plugin trait" in the doc-comment.
- [ ] Closed-domain data enum, no planned extension?
  → Do NOT apply; state "closed domain" in doc-comment.
- [ ] Applied `#[non_exhaustive]` to a non-error enum? → Add wildcard `Err(...)` arm in
  EXTERNAL crate match sites; document the obligation in the enum's doc-comment.
