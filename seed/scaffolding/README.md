# Scaffolding Templates

Starter files for the slideforge project. The agent factory copies these into the new project root during **Phase 0: Project scaffolding**.

## Files

| File | Purpose | Customization required |
|------|---------|----------------------|
| `.editorconfig` | Universal editor settings (charset, line endings, indent) | None |
| `rustfmt.toml` | Rust formatter config (stable-only options) | None |
| `clippy.toml` | Clippy lint thresholds (slightly relaxed for parser code) | None |
| `rust-toolchain.toml` | Pin to `stable` channel with rustfmt + clippy components | None |
| `Cargo.toml` | Workspace root config with all crates and shared dependencies | Replace `REPLACE-ME` author and repository placeholders |
| `.gitignore` | Standard Rust ignores plus dist artifacts | None |
| `.github/workflows/ci.yml` | CI: fmt, clippy, tests (Linux/macOS/Windows), MSRV check, docs, insta snapshots | None |
| `CHANGELOG.md` | Keep-a-Changelog format with initial entry | Replace `REPLACE-ME` repository URL |
| `LICENSE-MIT` | MIT license boilerplate | Replace `[YEAR]` and `[COPYRIGHT HOLDER]` |
| `LICENSE-APACHE` | Apache 2.0 license text | Replace `[YEAR]` and `[COPYRIGHT HOLDER]` |

## Application steps (Phase 0)

1. **Copy all files** from this directory into the new project root, preserving the `.github/` subdirectory structure.
2. **Replace placeholder values** in the three files marked above.
3. **Initialize git** in the new project root and create the initial commit.
4. **Verify** that `cargo build --workspace` succeeds (will be a no-op with empty crate stubs but should not fail).
5. **Verify** that `cargo fmt --check`, `cargo clippy`, and `cargo test --workspace` all run cleanly.
6. **Push the initial scaffold** and confirm CI passes on a fresh PR.

## Customization notes

### Cargo.toml workspace dependencies

The `[workspace.dependencies]` section pins shared dependencies to compatible major versions. Individual crates inherit these via:

```toml
[dependencies]
chumsky = { workspace = true }
```

This keeps version drift impossible across the workspace.

### CI matrix

The CI matrix runs tests on Linux, macOS, and Windows. If the project will only support a subset of platforms, trim the matrix in `.github/workflows/ci.yml`.

### MSRV (Minimum Supported Rust Version)

Pinned at 1.85 in three places:
- `clippy.toml` (`msrv = "1.85"`)
- `Cargo.toml` workspace package (`rust-version = "1.85"`)
- `.github/workflows/ci.yml` (MSRV job)

Update all three together if MSRV changes.

### Snapshot tests

The CI workflow includes an `insta` snapshot test job that fails if any `.snap.new` file exists (i.e., a snapshot diff was not reviewed and committed). This is enforced via `cargo insta test --check`. Developers must run `cargo insta review` locally and commit accepted snapshots.

### Profile.dist

A custom `dist` profile inherits from `release` with `lto = "fat"` for distribution binaries. Use `cargo build --profile dist` when producing release binaries for GitHub Releases.

## Notes for the factory

- **Do not modify these scaffolding files** when copying. They are designed to work out-of-the-box.
- **Do replace placeholders** before the initial commit.
- **Do verify CI passes** before any further phase work begins.
- If a customization is needed (e.g., disabling a specific clippy lint that turns out to be too noisy), make the change in a separate commit with a justifying message — don't bundle it with feature work.
