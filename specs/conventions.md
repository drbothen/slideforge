---
title: Slideforge Conventions
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.0
created: 2026-05-23
status: SEED-EXTRACT
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
