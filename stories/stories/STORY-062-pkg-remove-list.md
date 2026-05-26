---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-062
title: "Package: remove + list"
epic: EPIC-16
wave: 5
points: 3
priority: P1
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-package
behavioral_contracts:
  - BC-5.03.004
  - BC-5.03.005
verification_properties: []
nfr_refs: []
depends_on:
  - STORY-060
blocks: []
subsystems:
  - SS-16
target_module: slideforge-package
---

# STORY-062: Package: remove + list

## Summary

Implement two `slideforge package` subcommands in `slideforge-package`:

1. **`slideforge package remove <name>`** — removes the named package from
   `slideforge.toml` `[dependencies]`, deletes its entry from `sf.lock`, and
   removes the local cache archive. If any `.sf` file uses `@import` for the
   removed package, a warning is printed (but removal completes).

2. **`slideforge package list`** — reads `slideforge.toml` `[dependencies]` and
   `sf.lock`, prints a formatted table showing each package with its lock status
   (LOCKED / UNLOCKED / MISMATCH). Pure read-only operation.

Both commands reuse the `LockFile`, `LockedPackage`, and `PackageError` types
defined in STORY-060. No new crate structure is needed.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-5.03.004 | slideforge package list shows all installed packages with versions and lock status | Postconditions 1–5; Invariants 1–3 |
| BC-5.03.005 | slideforge package remove removes package from slideforge.toml and sf.lock | Postconditions 1–5; Invariants 1–4 |

## Acceptance Criteria

### `package list`

- [ ] **AC-001** — `slideforge package list` prints a table to stdout with columns:
  Name, Version, SHA (first 8 chars of rev), Status. Packages are sorted
  alphabetically by name. Exit code is 0.
  (traces to BC-5.03.004 postcondition 1, postcondition 2, invariant 2)

- [ ] **AC-002** — If `sf.lock` is absent, all packages from `[dependencies]` are
  shown with status `UNLOCKED` and a warning is printed: "sf.lock not found — run
  slideforge package install". Exit code is 0.
  (traces to BC-5.03.004 postcondition 4, edge case EC-002)

- [ ] **AC-003** — If `[dependencies]` is empty or absent, the output is
  "No packages installed." Exit code is 0.
  (traces to BC-5.03.004 postcondition 3, edge case EC-001)

- [ ] **AC-004** — If `slideforge.toml` is not found, E-CFG-007 is emitted and exit
  code is 4.
  (traces to BC-5.03.004 edge case EC-004)

- [ ] **AC-005** — `package list` does NOT write or modify any file.
  (traces to BC-5.03.004 invariant 1)

- [ ] **AC-006** — A package in `slideforge.toml` but absent from `sf.lock` is shown
  as `UNLOCKED` (not silently omitted).
  (traces to BC-5.03.004 invariant 3)

- [ ] **AC-007** — A package with a declared version in `slideforge.toml` that differs
  from the resolved SHA in `sf.lock` is shown as `MISMATCH` with both values displayed.
  (traces to BC-5.03.004 edge case EC-005)

### `package remove`

- [ ] **AC-008** — `slideforge package remove brand-kit` removes the `brand-kit` entry
  from `slideforge.toml` `[dependencies]` and from `sf.lock`. The local cache archive is
  deleted. Exit code is 0.
  (traces to BC-5.03.005 postcondition 1, postcondition 2, postcondition 3)

- [ ] **AC-009** — All other packages in `slideforge.toml` and `sf.lock` are unchanged
  after a `remove` of an unrelated package.
  (traces to BC-5.03.005 invariant 1)

- [ ] **AC-010** — No `.sf` source files are modified by `remove`.
  (traces to BC-5.03.005 invariant 2)

- [ ] **AC-011** — If any `.sf` file contains `@import "brand-kit/..."`, a warning is
  printed listing the affected files. Remove still completes; exit code is 0.
  (traces to BC-5.03.005 postcondition 4, postcondition 5, edge case EC-002)

- [ ] **AC-012** — Removing a package not in `[dependencies]` emits E-CFG-007
  ("package '<name>' not found") with no changes to any file. Exit code is 4.
  (traces to BC-5.03.005 invariant 3, edge case EC-001)

- [ ] **AC-013** — `remove` is atomic: both `slideforge.toml` and `sf.lock` are updated
  in the same operation (write-to-tmp then rename), or neither is. A process crash after
  the first file is written but before the second does not leave an inconsistent state
  on next invocation.
  (traces to BC-5.03.005 invariant 4)

- [ ] **AC-014** — `remove` with `sf.lock` absent removes from `slideforge.toml` only;
  no error for missing sf.lock. Exit code is 0.
  (traces to BC-5.03.005 edge case EC-003)

- [ ] **AC-015** — `remove` with an already-deleted local cache does not error; it
  removes from `slideforge.toml` and `sf.lock` successfully.
  (traces to BC-5.03.005 edge case EC-004)

- [ ] **AC-016** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code,
  `clippy::pedantic` clean. All public items have rustdoc.
  (traces to NFR-021, NFR-022, NFR-024, NFR-023)

## Tasks

1. Implement `src/list.rs` — the `list()` function:
   ```rust
   pub struct PackageListEntry {
       pub name: String,
       pub declared_ref: String,   // from slideforge.toml [dependencies]
       pub rev_short: String,      // first 8 chars of rev from sf.lock, or ""
       pub status: LockStatus,
   }

   pub enum LockStatus { Locked, Unlocked, Mismatch }

   pub fn list(project_dir: &Path) -> Result<Vec<PackageListEntry>, PackageError>;
   ```
   Algorithm:
   - Load `slideforge.toml` → E-CFG-007 if absent.
   - Load `sf.lock` if present; if absent, all entries are `Unlocked`.
   - For each dependency in `[dependencies]`: find matching `LockedPackage` in
     `sf.lock` → if not found, `Unlocked`; if found but version mismatch, `Mismatch`;
     else `Locked`.
   - Return sorted by name.

2. Implement `src/list.rs` — `format_list_table(entries: &[PackageListEntry]) -> String`:
   Produces a fixed-width table for stdout output:
   ```
   Name         Version     SHA       Status
   --------     --------    --------  --------
   brand-kit    0.1.0       a1b2c3d4  LOCKED
   icon-set     main        UNLOCKED
   ```

3. Implement `src/remove.rs` — the `remove()` function:
   ```rust
   pub fn remove(project_dir: &Path, package_name: &str) -> Result<RemoveResult, PackageError>;

   pub struct RemoveResult {
       pub affected_sf_files: Vec<PathBuf>, // .sf files that @import the removed package
   }
   ```
   Algorithm:
   - Load `slideforge.toml` → check `[dependencies]` contains `package_name`; if
     absent → E-CFG-007.
   - Scan all `.sf` files in the project dir (recursive glob `**/*.sf`) for
     `@import "<package_name>/..."` patterns → collect into `affected_sf_files`.
   - Remove `package_name` entry from `[dependencies]` using `toml_edit`.
   - Write updated `slideforge.toml` atomically.
   - If `sf.lock` exists: remove the `[[package]]` entry for `package_name`;
     write updated `sf.lock` atomically.
   - Delete cache archive: `~/.slideforge/cache/<sha256>.tar.gz` if it exists
     (best-effort: `fs::remove_file()` ignoring `NotFound` error).
   - Return `RemoveResult`.

4. Add `scan_sf_imports()` helper in `src/scan.rs`:
   ```rust
   /// Find all .sf files under `project_dir` that contain `@import "pkg_name/..."`
   pub fn scan_sf_imports(project_dir: &Path, package_name: &str) -> Vec<PathBuf>;
   ```
   Uses `walkdir` for recursive `.sf` glob and `str::contains()` search within each
   file's content. Does NOT parse the AST — text search is sufficient for a warning.

5. Wire CLI subcommands in `slideforge-cli/src/commands/package.rs`:
   ```rust
   pub enum PackageSubcommand {
       Install(InstallArgs),  // STORY-060
       Remove(RemoveArgs),
       List(ListArgs),
       Verify(VerifyArgs),    // STORY-063
   }

   pub fn run_list(args: &ListArgs, global: &GlobalFlags) -> ExitCode;
   pub fn run_remove(args: &RemoveArgs, global: &GlobalFlags) -> ExitCode;
   ```
   `run_list`: call `list()`, `format_list_table()`, print to stdout; handle E-CFG-007
   → stderr + exit 4.
   `run_remove`: call `remove()`; if `affected_sf_files` non-empty, print warning; exit 0.

6. Write tests.

## File List

- `crates/slideforge-package/src/list.rs` — `list()`, `PackageListEntry`, `LockStatus`,
  `format_list_table()`
- `crates/slideforge-package/src/remove.rs` — `remove()`, `RemoveResult`
- `crates/slideforge-package/src/scan.rs` — `scan_sf_imports()` text-search helper
- `crates/slideforge-cli/src/commands/package.rs` — `run_list()`, `run_remove()` handlers
  (the `PackageSubcommand` enum stub was created in STORY-060 install handler)
- `crates/slideforge-package/tests/remove_list_integration.rs` — integration tests

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~4 500 |
| BC-5.03.004 + BC-5.03.005 | ~4 000 |
| STORY-060 (LockFile, PackageError, toml_edit, cache layout) | ~3 500 |
| Target source files to write | ~4 500 |
| Test files | ~2 500 |
| **Total** | **~19 000** |

Context budget: 19 000 / 200 000 ≈ 9.5% — well within limit.

## Test Strategy

**Unit tests** (`crates/slideforge-package/src/list.rs`, `remove.rs` `#[cfg(test)]`):

- `test_list_empty_dependencies()`: toml with empty `[dependencies]`; assert
  `list()` returns empty vec.
- `test_list_unlocked_when_no_lock_file()`: toml with 1 dep, no sf.lock; assert
  status `Unlocked`.
- `test_list_locked_when_present()`: toml + sf.lock both have matching entry; assert
  status `Locked`.
- `test_list_sorted_alphabetically()`: 3 deps in non-sorted order; assert result sorted.
- `test_list_read_only()`: run `list()`; assert no files in project dir modified.
- `test_remove_removes_from_toml()`: install mock package; remove; parse toml; assert
  `[dependencies]` no longer contains it.
- `test_remove_removes_from_lock()`: install; remove; parse sf.lock; assert entry absent.
- `test_remove_preserves_other_packages()`: 2 packages; remove 1; assert other unchanged
  in both toml and sf.lock.
- `test_remove_not_found()`: call `remove("nonexistent")`; assert E-CFG-007; no files
  changed.
- `test_remove_no_sf_lock_ok()`: remove when sf.lock absent; assert toml updated; exit 0.
- `test_remove_missing_cache_ok()`: remove when cache file absent; assert no error; toml
  and sf.lock updated.
- `test_scan_sf_imports_finds_uses()`: create project with 2 .sf files; one has
  `@import "brand-kit/logo"`; assert `scan_sf_imports("brand-kit")` returns that file.
- `test_scan_sf_imports_case_sensitive()`: scan for "Brand-Kit" does NOT match
  `@import "brand-kit/..."`.

**Integration tests** (`crates/slideforge-package/tests/remove_list_integration.rs`):

- `test_list_two_packages()`: set up project with 2 packages in toml + sf.lock; run
  `list()`; assert 2 rows, both LOCKED.
- `test_remove_with_import_warning()`: install package; create `deck.sf` with
  `@import "pkg/item"`; remove; assert warning in `affected_sf_files`.
- `test_remove_atomicity()`: verify that after remove, the resulting toml and sf.lock
  are valid TOML (not partial/corrupt).

## Dependencies

- **Depends on:** STORY-060 (`LockFile`, `LockedPackage`, `PackageError`, `toml_edit`
  atomic write helpers, and cache path formula are all defined there — `remove` and
  `list` are consumers of that infrastructure)
- **Blocks:** (none — STORY-063 verify is independent; it reads the same lock file)

## Dependency Anchor Justifications

- SS-16 owns this story's scope because SS-16 is the Package Management subsystem
  owning `slideforge-package` per ARCH-INDEX Subsystem Registry.
- STORY-062 depends on STORY-060 because the `LockFile` serialize/deserialize and atomic
  write utilities are defined in STORY-060. Reimplementing them here would violate DRY
  and create drift.

## Architecture Compliance Rules

1. `package list` is a **pure read-only** command. It MUST NOT write any file. The
   implementation must be verifiable: snapshot the directory before + after, assert no
   changes.
2. `package remove` uses the same atomic write pattern from STORY-060: write to `.tmp`,
   then `fs::rename()`.
3. `scan_sf_imports()` is a text-search only — NOT a parse. Using `str::contains()` is
   intentional and sufficient for a warning (false positives acceptable; false negatives
   not critical since the next build will catch broken imports).
4. `slideforge-package` must NOT import `slideforge-eval` or `slideforge-syntax`.
5. `#![forbid(unsafe_code)]` at the crate root.

**Forbidden dependencies for `slideforge-package`:**
- Must NOT import `slideforge-eval`.
- Must NOT import `chumsky` or `slideforge-syntax`.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `toml_edit` | `=0.22` | Structure-preserving TOML mutation for remove |
| `toml` | `=0.8` | TOML parsing for list (read-only) |
| `walkdir` | `=2.5` | Recursive `.sf` file scan in `scan_sf_imports()` |
| `thiserror` | `=2.0` | PackageError (already present from STORY-060) |

## File Structure Requirements

```
crates/slideforge-package/
  src/
    list.rs     # list(), PackageListEntry, LockStatus, format_list_table()
    remove.rs   # remove(), RemoveResult
    scan.rs     # scan_sf_imports(project_dir, package_name) -> Vec<PathBuf>
  tests/
    remove_list_integration.rs
crates/slideforge-cli/
  src/
    commands/
      package.rs  # run_list(), run_remove() added to existing package command handler
```

## Previous Story Intelligence

STORY-060 established the `LockFile`, `LockedPackage`, atomic write pattern, and
`toml_edit` usage for `slideforge.toml` mutation. Before implementing this story,
read STORY-060's `lock.rs` and `toml_edit.rs` to understand the exact types and
function signatures already available.

Key lesson from STORY-060's design: `remove_dependency()` in `toml_edit.rs` is the
mirror of `add_dependency()`. Both must preserve all other TOML structure (comments,
formatting, other sections).

## Implementation Notes

### `package list` table output format

```
Name         Version     SHA       Status
brand-kit    ^0.1.0      a1b2c3d4  LOCKED
icon-set     main                  UNLOCKED
```

Column widths: auto-sized to the longest entry + 2 spaces padding. Status column
right-aligned. `SHA` column shows first 8 hex characters of the `rev` field (or empty
for UNLOCKED).

### `remove` and `@import` scan heuristic

Text-search pattern for scanning: `@import "` + package_name + `/`. This catches
the common form without a full parse. It does NOT catch `@import` with dynamic
variable paths like `@import {{ pkg_name ~ "/logo" }}` — those would be a later
enhancement. For v1.0, static imports only.

### Error codes

| Code | Meaning | Exit |
|------|---------|------|
| E-CFG-007 | Resource not found (package not in [dependencies]; or slideforge.toml not found) | 4 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No packages in [dependencies] | list: "No packages installed."; exit 0 / remove: E-CFG-007 |
| EC-002 | sf.lock absent | list: all UNLOCKED + warning / remove: remove from toml only |
| EC-003 | Package in sf.lock but not in slideforge.toml [dependencies] | list: ignored (sf.lock entries not in toml are not shown) |
| EC-004 | slideforge.toml not found | list: E-CFG-007; exit 4 |
| EC-005 | sf.lock SHA differs from declared version in slideforge.toml | list: MISMATCH with both values shown |
| EC-006 | remove package with @import in .sf file | Warning listing affected files; remove completes; exit 0 |
| EC-007 | remove with sf.lock absent | Remove from toml only; no error; exit 0 |
| EC-008 | remove with cache already deleted | Best-effort delete (ignore NotFound); exit 0 |
