---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-061
title: "Package: @import resolution in evaluator"
epic: EPIC-16
wave: 5
points: 5
priority: P1
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-package
behavioral_contracts:
  - BC-5.03.002
  - BC-1.06.004
verification_properties: []
nfr_refs:
  - NFR-018
depends_on:
  - STORY-060
  - STORY-013
blocks: []
subsystems:
  - SS-16
  - SS-02
target_module: slideforge-package
---

# STORY-061: Package: @import resolution in evaluator

## Summary

Wire `@import "package/item"` resolution between the evaluator (`slideforge-eval`) and
the package system (`slideforge-package`). When the evaluator encounters an `@import`
directive, it queries the package resolver to locate the named package in `sf.lock`,
resolve the cached archive, and return the `.sf` content for inlining. If the package
is not in `sf.lock`, the evaluator emits E-PKG-001 with a concrete `slideforge package
install <url>` hint.

Key responsibilities:

1. **Evaluator hook point**: add a `PackageResolver` trait to `slideforge-package`
   that the evaluator calls. The evaluator depends on the trait (not the concrete
   implementation) to maintain the correct dependency direction.
2. **sf.lock lookup**: given a package name (first path component of the `@import`
   path), find the matching `LockedPackage` entry in `sf.lock`.
3. **Cache validation**: confirm the package archive exists in the local cache at the
   expected SHA-256 path. If the archive is missing but `sf.lock` has an entry,
   trigger a re-fetch (using the `rev` from sf.lock) and re-verify checksum.
4. **Content extraction**: extract the requested item (everything after the first `/`
   in the `@import` path) from the package archive and return it as a `String` of `.sf`
   source for inline parsing.
5. **Error reporting**: E-PKG-001 carries the file:line:col of the `@import` directive,
   the missing package name, and a concrete install command.
6. **Semantic distinction**: `@import` NEVER falls back to local file lookup. A path
   that looks like a relative file path (`"./local.sf"`) in an `@import` directive
   is a parse error pointing the user to use `@include` instead.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-5.03.002 | @import of package not in sf.lock fails with install hint | Postconditions 1–4; Invariants 1–3 |
| BC-1.06.004 | @import resolves to installed package or fails with install hint | Postconditions (happy + not-installed); Invariants 1–2 |

## Acceptance Criteria

- [ ] **AC-001** — `@import "brand-kit/logo"` with `brand-kit` present in `sf.lock`
  and cache resolves to the `logo.sf` content from the package archive, which is
  inlined at the import position and compiled as part of the parent deck.
  (traces to BC-1.06.004 postcondition 1, postcondition 2, postcondition 3)

- [ ] **AC-002** — `@import "brand-kit/logo"` when `sf.lock` does not exist emits
  E-PKG-001: `Package 'brand-kit' not found in sf.lock. Run: slideforge package install
  <repo-url>`. Exit code is 5. The error carries the file:line:col of the `@import`
  directive.
  (traces to BC-5.03.002 postcondition 1, postcondition 2, postcondition 4;
  BC-1.06.004 postcondition not-installed 1)

- [ ] **AC-003** — `@import "brand-kit/logo"` when `sf.lock` exists but has no
  `brand-kit` entry emits E-PKG-001 (same error as AC-002).
  (traces to BC-5.03.002 precondition 2)

- [ ] **AC-004** — Package name lookup in `sf.lock` is case-sensitive: `@import
  "Brand-Kit/logo"` does NOT match a `sf.lock` entry named `brand-kit`. E-PKG-001
  emitted.
  (traces to BC-5.03.002 edge case EC-001)

- [ ] **AC-005** — Multiple `@import` directives for the same missing package produce
  E-PKG-001 emitted exactly once per missing package name (deduplicated), not once per
  `@import` site.
  (traces to BC-5.03.002 edge case EC-002)

- [ ] **AC-006** — If `sf.lock` has a `brand-kit` entry but the cache archive file is
  missing, the resolver re-fetches the package using the `rev` from `sf.lock`, verifies
  the checksum, and populates the cache. The build then continues successfully.
  (traces to BC-5.03.002 edge case EC-004)

- [ ] **AC-007** — `@import "./local.sf"` (path-like argument) produces a parse error:
  "Use @include for local file imports; @import is for installed packages." Exit code 1.
  (traces to BC-1.06.004 edge case EC-003)

- [ ] **AC-008** — `@import "brand-kit/logo"` with `brand-kit` in `sf.lock` but cache
  checksum mismatch emits E-PKG-003 (checksum mismatch). Build halts. Exit code 5.
  (traces to BC-5.03.002 edge case EC-003)

- [ ] **AC-009** — `--offline` flag with a missing package cache entry emits E-PKG-001
  with an added note: "--offline mode; cannot fetch missing package".
  (traces to BC-5.03.002 edge case EC-005)

- [ ] **AC-010** — `@import` resolution happens at compile time (parse/evaluation phase),
  before any output is produced. No partial output is written if an `@import` fails.
  (traces to BC-1.06.004 invariant 2; BC-5.03.002 invariant 3)

- [ ] **AC-011** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code,
  `clippy::pedantic` clean. All public items have rustdoc.
  (traces to NFR-021, NFR-022, NFR-024, NFR-023)

## Tasks

1. Define the `PackageResolver` trait in `slideforge-package/src/resolver.rs`:
   ```rust
   /// Trait object interface for @import resolution.
   /// The evaluator calls this; the concrete impl lives in slideforge-package.
   pub trait PackageResolver: Send + Sync {
       /// Look up a package by name and return the content of the requested item.
       /// `package_name`: first component of the @import path.
       /// `item_path`: remainder (e.g. "slides/intro" from "brand-kit/slides/intro").
       fn resolve(
           &self,
           package_name: &str,
           item_path: &str,
           span: Span,
           offline: bool,
       ) -> Result<String, PackageResolveError>;

       /// Return the list of all package names currently missing from sf.lock.
       /// Used for deduplication of E-PKG-001 errors.
       fn missing_packages(&self, imports: &[ImportDirective]) -> Vec<String>;
   }
   ```

2. Implement `DefaultPackageResolver` in `slideforge-package/src/resolver.rs`:
   ```rust
   pub struct DefaultPackageResolver {
       pub lock_file: Option<LockFile>,  // None = sf.lock doesn't exist
       pub cache_dir: PathBuf,
   }

   impl PackageResolver for DefaultPackageResolver {
       fn resolve(&self, name, item, span, offline) -> Result<String, PackageResolveError> {
           // 1. Validate @import path is not file-like
           if name.starts_with('.') || name.starts_with('/') {
               return Err(PackageResolveError::FilePathInImport { span });
           }
           // 2. Look up sf.lock
           let lock = self.lock_file.as_ref()
               .ok_or(PackageResolveError::NotInLock { name: name.into(), span })?;
           let entry = lock.find(name)
               .ok_or(PackageResolveError::NotInLock { name: name.into(), span })?;
           // 3. Check cache
           let archive_path = self.cache_dir.join(format!("{}.tar.gz", entry.sha256));
           let archive = if archive_path.exists() {
               std::fs::read(&archive_path)?
           } else if offline {
               return Err(PackageResolveError::OfflineMissingCache { name: name.into(), span });
           } else {
               // Re-fetch using entry.rev
               let archive = re_fetch(entry)?;
               std::fs::write(&archive_path, &archive)?;
               archive
           };
           // 4. Verify checksum
           let actual = sha256_hex(&archive);
           if actual != entry.sha256 {
               return Err(PackageResolveError::ChecksumMismatch { name: name.into(), span });
           }
           // 5. Extract item from archive
           extract_item_from_archive(&archive, item, span)
       }
   }
   ```

3. Implement deduplication in `missing_packages()`:
   ```rust
   fn missing_packages(&self, imports: &[ImportDirective]) -> Vec<String> {
       let mut seen = std::collections::HashSet::new();
       imports.iter()
           .filter_map(|d| {
               let name = first_path_component(&d.path);
               if self.lock_file.as_ref().and_then(|lf| lf.find(name)).is_none() {
                   seen.insert(name.to_string()).then_some(name.to_string())
               } else {
                   None
               }
           })
           .collect()
   }
   ```

4. Wire the resolver into `slideforge-eval`. In `slideforge-eval`, accept a
   `Box<dyn PackageResolver>` as a parameter to the evaluator's `Context`. The evaluator
   calls `resolver.resolve(...)` when it encounters `Expr::Import { package, item, span }`.

5. Register `DefaultPackageResolver` in the root `slideforge` crate's
   `compile(options)` function — construct with the project's `sf.lock` path and cache
   dir, pass into `EvalContext`.

6. Handle the "path-like @import" error at the AST parse level (in `slideforge-syntax`)
   — when `@import` is followed by a string starting with `.` or `/`, emit a parse error
   with the "use @include" hint.

7. Write all tests.

## File List

- `crates/slideforge-package/src/resolver.rs` — `PackageResolver` trait +
  `DefaultPackageResolver` impl + `PackageResolveError` enum
- `crates/slideforge-package/src/extract.rs` — `extract_item_from_archive()` — reads a
  `.tar.gz` archive, finds the file at the requested path, returns contents as String
- `crates/slideforge-eval/src/context.rs` — add `resolver: Box<dyn PackageResolver>` to
  `EvalContext`; call resolver on `Import` AST nodes
- `crates/slideforge-syntax/src/parser.rs` — add parse-time guard: `@import` with
  file-like path emits parse error with `@include` suggestion
- `crates/slideforge/src/lib.rs` — construct `DefaultPackageResolver` and pass to
  `EvalContext`
- `crates/slideforge-package/tests/resolver_integration.rs` — integration tests

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~5 000 |
| BC-5.03.002 + BC-1.06.004 | ~3 500 |
| STORY-060 (LockFile, LockedPackage, cache layout) | ~3 500 |
| STORY-013 (@include/cycle detection — context for how eval handles file inlining) | ~2 500 |
| slideforge-eval context.rs (existing EvalContext type) | ~2 000 |
| Target source files to write | ~5 500 |
| Test files | ~3 000 |
| **Total** | **~25 000** |

Context budget: 25 000 / 200 000 ≈ 12.5% — within limit.

## Test Strategy

**Unit tests** (`crates/slideforge-package/src/resolver.rs #[cfg(test)]`):

- `test_resolve_missing_lock_file()`: `DefaultPackageResolver` with `lock_file: None`;
  call `resolve("brand-kit", "logo", span, false)`; assert `NotInLock` error.
- `test_resolve_package_not_in_lock()`: lock file exists but no entry for "brand-kit";
  assert `NotInLock`.
- `test_resolve_case_sensitive()`: lock has "brand-kit"; resolve "Brand-Kit"; assert
  `NotInLock`.
- `test_resolve_file_like_path()`: `@import "./local.sf"` → assert `FilePathInImport`.
- `test_missing_packages_deduplication()`: 3 imports of "brand-kit" (missing); assert
  `missing_packages()` returns exactly 1 entry.
- `test_resolve_offline_missing_cache()`: lock has "brand-kit", cache absent, offline=true;
  assert `OfflineMissingCache`.
- `test_resolve_checksum_mismatch()`: lock has "brand-kit" with sha256 X; cache has
  archive with sha256 Y; assert `ChecksumMismatch`.

**Integration tests** (`crates/slideforge-package/tests/resolver_integration.rs`):

- `test_resolve_happy_path()`: create a local package with a `logo.sf` item; install;
  resolve via `DefaultPackageResolver`; assert returned content matches `logo.sf`.
- `test_resolve_missing_item_path()`: package installed; request `brand-kit/nonexistent`;
  assert appropriate "item not found" error.
- `test_resolve_cache_refetch()`: install package; delete cache file; resolve again
  (online mode); assert cache re-populated and content returned.
- `test_import_integration_eval()`: end-to-end: `.sf` source with `@import` → evaluator
  inlines the package content → AST contains the inlined slides.

## Dependencies

- **Depends on:** STORY-060 (provides `LockFile`, `LockedPackage`, cache layout, and
  `sha256_hex` that the resolver reads)
- **Depends on:** STORY-013 (`@include` / cycle detection — the evaluator's existing
  mechanism for inlining file content; `@import` hooks into the same AST inline path
  but with a different source: package cache vs local filesystem)
- **Blocks:** (none — the remaining package stories are independent of import resolution)

## Dependency Anchor Justifications

- SS-16 owns the `PackageResolver` trait and `DefaultPackageResolver` impl because SS-16
  is the Package Management subsystem. SS-02 (Evaluator) is the consumer — it calls the
  trait but does not implement it.
- STORY-061 depends on STORY-060 because `DefaultPackageResolver` reads the `LockFile`
  and cache layout defined and written in STORY-060. Without an installed package, there
  is nothing to resolve.
- STORY-061 depends on STORY-013 because the evaluator's `@include` inlining mechanism
  is the existing code path that `@import` resolution hooks into. The evaluator calls
  `resolver.resolve()` then feeds the returned source string back through the same
  parse-and-inline flow as `@include`.

## Architecture Compliance Rules

1. **Dependency direction**: `slideforge-eval` → `slideforge-package` (evaluator calls
   resolver). `slideforge-package` must NOT import `slideforge-eval`. Trait is defined
   in `slideforge-package` and consumed by `slideforge-eval`.
2. `@import` NEVER touches the local filesystem outside the package cache directory.
   All file accesses go through `extract_item_from_archive()`.
3. The resolver is passed as a `Box<dyn PackageResolver>` (trait object) into
   `EvalContext`. Do NOT use generics here — the evaluator is not generic over the
   resolver to keep compile times manageable.
4. Error deduplication happens at the resolver level, not in the CLI. The CLI should
   pass through whatever the resolver returns.
5. `#![forbid(unsafe_code)]` at the crate root.

**Forbidden dependencies for `slideforge-package`:**
- Must NOT import `slideforge-eval` (circular — evaluator imports package, not reverse).
- Must NOT import `chumsky` or `slideforge-syntax`.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `sha2` | `=0.10` | Re-verify cache checksum on re-fetch |
| `hex` | `=0.4` | Hex encoding |
| `tar` | `=0.4` | Extract .tar.gz archive contents |
| `flate2` | `=1.0` | Gzip decompression for .tar.gz |
| `thiserror` | `=2.0` | `PackageResolveError` derive |
| `git2` | `=0.20` | Re-fetch via git when cache is missing |

## File Structure Requirements

```
crates/slideforge-package/
  src/
    resolver.rs     # PackageResolver trait, DefaultPackageResolver, PackageResolveError
    extract.rs      # extract_item_from_archive(archive: &[u8], item: &str) -> Result<String>
  tests/
    resolver_integration.rs
crates/slideforge-eval/
  src/
    context.rs      # add resolver: Box<dyn PackageResolver> field to EvalContext
    eval.rs         # call resolver.resolve() on Import AST nodes
crates/slideforge-syntax/
  src/
    parser.rs       # add guard: @import with ./ or / prefix is a parse error
crates/slideforge/
  src/
    lib.rs          # construct DefaultPackageResolver, pass to EvalContext
```

## Previous Story Intelligence

STORY-060 established the `LockFile`, `LockedPackage`, and cache directory layout.
Before implementing this story, read STORY-060's implementation to understand:
- The exact TOML structure of `sf.lock` (array-of-tables `[[package]]`)
- The cache path formula: `~/.slideforge/cache/<sha256>.tar.gz`
- The `sha256_hex()` function signature (already implemented in STORY-060)

STORY-013 established how `@include` inlines `.sf` content. The `@import` flow is
identical AFTER the package resolver returns the source string — both paths call
`parse_source_string()` and merge the resulting AST into the parent.

## Implementation Notes

### @import path parsing rule

The first path component (before the first `/`) is the package name. Everything
after the first `/` is the item path within the package:

```
@import "brand-kit/slides/intro"
         ^^^^^^^^^ ^^^^^^^^^^^
         pkg name  item path
```

For a package where `sf-package.toml` declares `name = "brand-kit"`, the item
path `slides/intro` maps to the file `slides/intro.sf` inside the package archive.

### Re-fetch algorithm (cache miss, online mode)

```
1. Find LockedPackage entry in sf.lock (has url + rev)
2. git clone <url> into temp dir
3. git checkout <rev> in temp dir
4. create_archive(temp dir) → Vec<u8>
5. verify sha256_hex(archive) == entry.sha256  (if mismatch → E-PKG-003)
6. write archive to cache path
7. return archive
```

### Error codes

| Code | Meaning | Exit |
|------|---------|------|
| E-PKG-001 | Package not in sf.lock | 5 |
| E-PKG-003 | Cache checksum mismatch (corrupt/tampered) | 5 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | sf.lock exists but package entry has wrong name case | E-PKG-001 (case-sensitive lookup) |
| EC-002 | Multiple @import of same missing package | E-PKG-001 emitted once per missing package (deduplicated) |
| EC-003 | sf.lock entry exists but cache SHA-256 checksum mismatch | E-PKG-003 (checksum mismatch), not E-PKG-001 |
| EC-004 | Package in sf.lock but cache missing | Re-fetch using sf.lock rev; verify; no error if successful |
| EC-005 | --offline flag with missing package | E-PKG-001 with "--offline mode" note |
| EC-006 | @import with path-like string (@import "./local.sf") | Parse error: use @include for local files |
| EC-007 | @import item path not found in package archive | PackageResolveError::ItemNotFound with item path |
