---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-060
title: "Package: install + sf.lock + SHA-256 integrity"
epic: EPIC-16
wave: 5
points: 8
priority: P1
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-package
behavioral_contracts:
  - BC-5.03.001
  - BC-5.03.003
verification_properties: []
nfr_refs:
  - NFR-018
depends_on:
  - STORY-055
blocks:
  - STORY-061
  - STORY-062
  - STORY-063
subsystems:
  - SS-16
target_module: slideforge-package
---

# STORY-060: Package: install + sf.lock + SHA-256 integrity

## Summary

Implement `slideforge package install <git-url>[@<ref>]` in the `slideforge-package`
crate. The command fetches a content package from a git repository, verifies the
package manifest (`sf-package.toml`), writes/updates `sf.lock` with the resolved commit
SHA and SHA-256 checksum of the package archive, and updates `slideforge.toml`
`[dependencies]`. It also implements the BC-5.03.003 build warning when `sf.lock` is
absent for a project that declares dependencies.

Key responsibilities:

1. **Git clone/fetch**: clone the package repository at the specified URL, resolve the
   optional `@<ref>` to a full 40-character commit SHA.
2. **Manifest validation**: verify that the cloned repository contains `sf-package.toml`
   with required fields (`name`, `version`).
3. **Archive and checksum**: create a deterministic archive of the package source tree
   (tar.gz with sorted file list and normalized timestamps), compute SHA-256 via the
   `sha2` crate.
4. **sf.lock write**: append or update the package entry in `sf.lock` (TOML format,
   deterministic — sorted by name, no timestamps).
5. **slideforge.toml update**: append the package to `[dependencies]` if not already
   present.
6. **Cache write**: store the package archive at `~/.slideforge/cache/<sha256>.tar.gz`.
7. **Build-time warning**: detect at project-load time when `slideforge.toml` has a
   non-empty `[dependencies]` section but `sf.lock` is absent; emit E-PKG-004.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-5.03.001 | slideforge package install adds dep to slideforge.toml and sf.lock with sha256 | Postconditions 1–5; Invariants 1–4 |
| BC-5.03.003 | sf.lock missing for project with deps produces build warning | Postconditions 1–4; Invariants 1–3 |

## Acceptance Criteria

- [ ] **AC-001** — `slideforge package install https://github.com/example/sf-brand-pkg`
  clones the repository, resolves `HEAD` to a 40-character commit SHA, and writes an
  `sf.lock` entry with fields `name`, `url`, `rev` (40 chars), and `sha256`.
  (traces to BC-5.03.001 postcondition 2)

- [ ] **AC-002** — After a successful install, `slideforge.toml` `[dependencies]` contains
  an entry for the package with the resolved version or git ref.
  (traces to BC-5.03.001 postcondition 1)

- [ ] **AC-003** — The package archive is cached at
  `~/.slideforge/cache/<sha256>.tar.gz`. A subsequent build that uses `@import`
  succeeds without re-fetching over the network.
  (traces to BC-5.03.001 postcondition 3, postcondition 4)

- [ ] **AC-004** — Installing the same package at the same version a second time is a
  no-op: `sf.lock` has exactly one entry for the package and exits 0.
  (traces to BC-5.03.001 postcondition 5, edge case EC-004)

- [ ] **AC-005** — Installing with `@v1.2.3` tag ref: `sf.lock` entry `rev` is the
  resolved commit SHA for that tag, NOT the tag name string.
  (traces to BC-5.03.001 invariant 1, edge case EC-005)

- [ ] **AC-006** — If the repository is unreachable, E-PKG-002 is emitted and
  `slideforge.toml` + `sf.lock` are left unchanged. Exit code is 5.
  (traces to BC-5.03.001 edge case EC-001)

- [ ] **AC-007** — If the repository lacks `sf-package.toml`, E-PKG-002 is emitted
  with the message "not a valid slideforge package"; no entry is added. Exit code is 5.
  (traces to BC-5.03.001 edge case EC-003)

- [ ] **AC-008** — `sf.lock` entries are sorted alphabetically by package name and
  contain no timestamp fields. Two installs on the same package produce byte-for-byte
  identical `sf.lock` output.
  (traces to BC-5.03.001 invariant 2, invariant 3 — determinism)

- [ ] **AC-009** — `sf.lock` format is valid TOML. Each entry is a
  `[[package]]` array-of-tables with exactly the fields `name`, `url`, `rev`, `sha256`
  and no others.
  (traces to BC-5.03.001 postcondition 2)

- [ ] **AC-010** — At build invocation time, when `slideforge.toml` has a non-empty
  `[dependencies]` section and `sf.lock` does not exist, E-PKG-004 is emitted once
  (not per-dependency) and the build continues with exit 0.
  (traces to BC-5.03.003 postcondition 1, postcondition 2, postcondition 3)

- [ ] **AC-011** — E-PKG-004 is NOT emitted when `slideforge.toml` has an empty or
  absent `[dependencies]` section, even if `sf.lock` is absent.
  (traces to BC-5.03.003 invariant 3, edge case EC-001)

- [ ] **AC-012** — E-PKG-004 is suppressed when `--no-lock-check` flag is passed.
  (traces to BC-5.03.003 invariant 2, edge case EC-005)

- [ ] **AC-013** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code,
  `clippy::pedantic` clean. All public items have rustdoc.
  (traces to NFR-021, NFR-022, NFR-024, NFR-023)

## Tasks

1. Create the `slideforge-package` crate in the Cargo workspace:
   ```toml
   # Cargo.toml (workspace)
   members = [
     # ...existing...
     "crates/slideforge-package",
   ]
   ```
   ```toml
   # crates/slideforge-package/Cargo.toml
   [package]
   name = "slideforge-package"
   version = "0.1.0"
   edition = "2024"

   [dependencies]
   git2 = "=0.20"
   sha2 = "=0.10"
   toml = "=0.8"
   serde = { version = "=1.0", features = ["derive"] }
   thiserror = "=2.0"
   tracing = "=0.1"
   tokio = { version = "=1.44", features = ["fs", "process"] }
   dirs = "=5.0"
   ```

2. Define the `sf.lock` data model:
   ```rust
   // src/lock.rs
   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
   pub struct LockFile {
       #[serde(rename = "package")]
       pub packages: Vec<LockedPackage>,
   }

   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
   pub struct LockedPackage {
       pub name: String,
       pub url: String,
       pub rev: String,    // 40-char commit SHA
       pub sha256: String, // hex-encoded SHA-256 of the package archive
   }

   impl LockFile {
       pub fn load(path: &Path) -> Result<Self, PackageError>;
       pub fn save(&self, path: &Path) -> Result<(), PackageError>;
       pub fn find(&self, name: &str) -> Option<&LockedPackage>;
       pub fn upsert(&mut self, pkg: LockedPackage);
       /// Sort by name for deterministic output
       pub fn normalize(&mut self) { self.packages.sort(); }
   }
   ```

3. Implement the install algorithm in `src/install.rs`:
   ```rust
   pub async fn install(url: &str, ref_spec: Option<&str>) -> Result<LockedPackage, PackageError> {
       // Step 1: Clone into a temp dir
       let tmp_dir = tempfile::tempdir()?;
       git_clone(url, tmp_dir.path(), ref_spec).await?;
       // Step 2: Resolve HEAD commit SHA
       let rev = resolve_head_sha(tmp_dir.path())?;
       // Step 3: Validate sf-package.toml
       let manifest = load_manifest(tmp_dir.path())?;
       // Step 4: Create deterministic tar archive
       let archive = create_archive(tmp_dir.path())?;
       // Step 5: Compute SHA-256
       let sha256 = sha256_hex(&archive);
       // Step 6: Write to cache
       let cache_path = cache_dir()?.join(format!("{sha256}.tar.gz"));
       tokio::fs::write(&cache_path, &archive).await?;
       Ok(LockedPackage { name: manifest.name, url: url.to_string(), rev, sha256 })
   }

   fn create_archive(dir: &Path) -> Result<Vec<u8>, PackageError> {
       // Sorted file list, normalized timestamps (0), tar.gz
       // This ensures byte-for-byte identical archives for the same content
   }
   ```

4. Implement SHA-256 computation using `sha2`:
   ```rust
   use sha2::{Sha256, Digest};

   pub fn sha256_hex(data: &[u8]) -> String {
       let mut hasher = Sha256::new();
       hasher.update(data);
       hex::encode(hasher.finalize())
   }
   ```

5. Implement `slideforge.toml` update — append to `[dependencies]` without disturbing
   other sections. Use `toml_edit` for structure-preserving TOML mutation:
   ```rust
   pub fn add_dependency(toml_path: &Path, name: &str, rev: &str) -> Result<(), PackageError>;
   ```

6. Implement the build-time lock check in `src/check.rs`:
   ```rust
   /// Called at project load time (before build starts).
   /// Returns Some(E_PKG_004_diagnostic) if warning should fire.
   pub fn check_lock_present(
       project_dir: &Path,
       no_lock_check: bool,
   ) -> Option<Diagnostic>;
   ```
   Logic: if `no_lock_check` → None; if `[dependencies]` empty or absent → None; if
   `sf.lock` exists → None; else → Some(E-PKG-004 diagnostic).

7. Wire `slideforge package install` subcommand in the CLI (`STORY-055` `Command::Package`
   arm — this story adds the `PackageArgs` struct and `install` handler).

8. Write all unit and integration tests for the ACs.

## File List

- `crates/slideforge-package/Cargo.toml` — new crate manifest
- `crates/slideforge-package/src/lib.rs` — public API re-exports
- `crates/slideforge-package/src/error.rs` — `PackageError` enum (E-PKG-001 through
  E-PKG-004 codes)
- `crates/slideforge-package/src/lock.rs` — `LockFile`, `LockedPackage` model + I/O
- `crates/slideforge-package/src/manifest.rs` — `SfPackageManifest` (parses `sf-package.toml`)
- `crates/slideforge-package/src/install.rs` — `install()` async function
- `crates/slideforge-package/src/archive.rs` — deterministic tar.gz archive creation
- `crates/slideforge-package/src/hash.rs` — `sha256_hex()` pure function
- `crates/slideforge-package/src/cache.rs` — `cache_dir()`, cache read/write helpers
- `crates/slideforge-package/src/check.rs` — `check_lock_present()` build-time check
- `crates/slideforge-package/src/git.rs` — `git_clone()`, `resolve_head_sha()` wrappers
- `crates/slideforge-package/src/toml_edit.rs` — `add_dependency()` structure-preserving
  TOML update
- `crates/slideforge-package/tests/install_integration.rs` — integration tests with
  a local bare git repository fixture

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~6 000 |
| BC-5.03.001 + BC-5.03.003 | ~4 000 |
| STORY-055 (CLI struct, PackageArgs integration point) | ~2 500 |
| Target source files to write | ~7 000 |
| Test files | ~4 000 |
| **Total** | **~23 500** |

Context budget: 23 500 / 200 000 ≈ 11.8% — within limit.

## Test Strategy

**Unit tests** (`crates/slideforge-package/src/` `#[cfg(test)]`):

- `test_sha256_hex_known_input()`: SHA-256("abc") = known hex; assert correct.
- `test_lock_file_round_trip()`: serialize and deserialize a `LockFile`; assert equal.
- `test_lock_file_normalize_sorts_by_name()`: create lock with ["z-pkg", "a-pkg"];
  call `normalize()`; assert "a-pkg" first.
- `test_lock_file_upsert_no_duplicate()`: upsert same package twice; assert one entry.
- `test_archive_determinism()`: create archive of a fixed directory twice; assert same bytes.
- `test_check_lock_present_no_deps()`: `slideforge.toml` with empty `[dependencies]`,
  no `sf.lock`; assert None returned.
- `test_check_lock_present_with_lock()`: `slideforge.toml` with deps, `sf.lock` present;
  assert None returned.
- `test_check_lock_present_missing_lock()`: `slideforge.toml` with deps, no `sf.lock`;
  assert Some(E-PKG-004) returned.
- `test_check_lock_no_lock_check_flag()`: missing lock but `--no-lock-check`; assert None.
- `test_rev_is_40_chars()`: mock install; assert `rev` in `LockedPackage` has length 40.

**Integration tests** (`crates/slideforge-package/tests/install_integration.rs`):

- `test_install_from_local_bare_repo()`: create a local bare git repo with
  `sf-package.toml`; run `install()`; assert `sf.lock` written with correct rev + sha256.
- `test_install_idempotent()`: install twice; assert sf.lock has exactly 1 entry.
- `test_install_tag_ref()`: create local repo with a `v1.0.0` tag; install with
  `@v1.0.0`; assert `rev` is the commit SHA, not "v1.0.0".
- `test_install_missing_manifest()`: repo without `sf-package.toml`; assert
  E-PKG-002 error.
- `test_install_does_not_modify_sf_files()`: run install; assert no `.sf` file in project
  is modified.
- `test_e_pkg_004_emitted_once()`: project with 2 deps in toml, no sf.lock; call
  `check_lock_present()`; assert exactly 1 diagnostic returned.

## Dependencies

- **Depends on:** STORY-055 (defines the `Command::Package(PackageArgs)` arm in the CLI
  dispatch struct that this story populates with the install subcommand handler)
- **Blocks:** STORY-061 (`@import` resolution depends on the `sf.lock` and cache that
  install creates)
- **Blocks:** STORY-062 (`remove` and `list` operate on the lock file format defined here)
- **Blocks:** STORY-063 (`verify` reads the same `sf.lock` and cache format defined here)

## Dependency Anchor Justifications

- SS-16 owns this story's scope because SS-16 is the Package Management subsystem owning
  `slideforge-package` per ARCH-INDEX Subsystem Registry.
- STORY-060 depends on STORY-055 because the CLI `Command::Package` arm (where
  `PackageArgs` is wired) is defined in STORY-055's `cli.rs`. Without the CLI struct
  existing, the install subcommand has nowhere to dispatch from.
- STORY-060 blocks STORY-061/062/063 because those stories consume the `LockFile`,
  `LockedPackage`, `PackageError`, and cache layout defined here. The lock format
  is the shared contract.

## Architecture Compliance Rules

1. `slideforge-package` is an **effectful** crate — it performs network I/O (git clone),
   filesystem writes (sf.lock, cache), and subprocess calls. All effectful operations must
   be async (`tokio`).
2. The pure functions (`sha256_hex`, `create_archive`, `LockFile` serialization) must be
   factored into separate modules with no async/I/O so they are Kani-amenable (Phase 6).
3. `slideforge-package` must NOT import `slideforge-eval` or any parser crate — package
   resolution at build time is the evaluator's responsibility (STORY-061 wires that
   direction).
4. All writes to `slideforge.toml` and `sf.lock` must be atomic: write to a `.tmp` file
   adjacent to the target, then `fs::rename()`. Never partial-write.
5. `git2` is allowed for git operations. Do NOT shell out to `git` binary — use `libgit2`
   via the `git2` crate for portability across all 5 target platforms.
6. `#![forbid(unsafe_code)]` at the crate root (git2 uses unsafe internally but behind
   a safe API boundary — that is acceptable).

**Forbidden dependencies for `slideforge-package`:**
- Must NOT import `slideforge-eval` — the evaluator calls the package resolver, not
  vice versa.
- Must NOT import `chumsky` or `slideforge-syntax` — package resolution is independent
  of parsing.
- Must NOT use `std::process::Command` for git — use `git2` crate only.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `git2` | `=0.20` | Git clone + SHA resolution (libgit2 bindings) |
| `sha2` | `=0.10` | SHA-256 checksum computation |
| `toml` | `=0.8` | sf.lock and sf-package.toml parsing |
| `toml_edit` | `=0.22` | Structure-preserving slideforge.toml mutation |
| `serde` | `=1.0` | Serialize/Deserialize for LockFile |
| `thiserror` | `=2.0` | PackageError derive |
| `tokio` | `=1.44` | Async runtime for I/O operations |
| `dirs` | `=5.0` | `~/.slideforge/cache/` home dir resolution |
| `tempfile` | `=3.10` | Temp dir for git clone staging |
| `hex` | `=0.4` | Hex encoding of SHA-256 digest |

## File Structure Requirements

```
crates/slideforge-package/
  src/
    lib.rs              # pub use: PackageError, LockFile, LockedPackage, install, check_lock_present
    error.rs            # PackageError enum: NetworkError, InvalidManifest, ChecksumMismatch, ...
    lock.rs             # LockFile, LockedPackage: serialize/deserialize, upsert, normalize
    manifest.rs         # SfPackageManifest: name, version, description fields
    install.rs          # install(url, ref_spec) -> Result<LockedPackage, PackageError>
    archive.rs          # create_archive(dir) -> Result<Vec<u8>, PackageError> (deterministic tar.gz)
    hash.rs             # sha256_hex(data: &[u8]) -> String (pure, no I/O)
    cache.rs            # cache_dir(), write_cache(), cache_path_for(sha256)
    check.rs            # check_lock_present(project_dir, no_lock_check) -> Option<Diagnostic>
    git.rs              # git_clone(url, dst, ref_spec), resolve_head_sha(repo_path)
    toml_edit.rs        # add_dependency(toml_path, name, rev) — structure-preserving mutation
  tests/
    install_integration.rs  # integration tests using local bare repo fixtures
  Cargo.toml
```

## Previous Story Intelligence

N/A — STORY-060 is the first package management story. However, STORY-055 established
the `Command::Package` stub in the CLI struct. Read the `cli.rs` from STORY-055 before
implementing to understand where `PackageArgs` slots in.

Key architectural constraint from epics.md: packages contain `.sf` files and assets
ONLY — no executable code, no plugins. The package install MUST NOT run any code from
the package during installation.

## Implementation Notes

### sf.lock TOML format

```toml
[[package]]
name = "sf-brand-kit"
url = "https://github.com/acme/sf-brand-kit"
rev = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

[[package]]
name = "sf-icon-set"
url = "https://github.com/acme/sf-icon-set"
rev = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
sha256 = "ba7816bf8f01cfea414140de5dae2ec73b3a2d5f6f4b35e0a3c3e7b7a7a0000"
```

Note: entries are always sorted alphabetically by `name`. No `[metadata]` header,
no timestamps, no tool-version comments. Same format as Cargo.lock arrays-of-tables.

### sf-package.toml format (package manifest)

```toml
[package]
name = "sf-brand-kit"
version = "1.0.0"
description = "Brand assets for Acme Corp decks"
authors = ["design@acme.com"]
license = "MIT"
```

Required fields: `name`, `version`. All others optional.

### Error codes for this story

| Code | Meaning | Exit |
|------|---------|------|
| E-PKG-001 | Package not in sf.lock (missing/not installed) | 5 |
| E-PKG-002 | Network error or invalid git URL or no sf-package.toml | 5 |
| E-PKG-004 | sf.lock absent for project with declared dependencies (degraded — warning only) | 0 |

E-PKG-003 (checksum mismatch) is defined here but emitted by STORY-063.

### Atomic write pattern

```rust
async fn atomic_write(path: &Path, content: &[u8]) -> Result<(), PackageError> {
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, content).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}
```

This ensures that `sf.lock` is never in a partially-written state visible to other
processes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Network unreachable during install | E-PKG-002; slideforge.toml and sf.lock unchanged |
| EC-002 | Invalid git URL (malformed) | E-PKG-002 with malformed URL message; no partial state written |
| EC-003 | Repository lacks sf-package.toml | E-PKG-002: "not a valid slideforge package"; no entry added |
| EC-004 | Re-install same package at same version | No-op; sf.lock unchanged; success exit 0 |
| EC-005 | Install with @v1.2.3 tag ref | sf.lock entry rev is resolved commit SHA of that tag, not the tag name |
| EC-006 | slideforge.toml has [dependencies] but no @import in .sf files | E-PKG-004 still emitted (deps declared but lock missing) |
| EC-007 | sf.lock exists but is empty | No E-PKG-004 (file exists; content check not performed at load time) |
| EC-008 | Two concurrent installs of different packages | Atomic writes prevent corruption; last writer wins on sf.lock (single-threaded CLI) |
