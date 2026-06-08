---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-063
title: "Package: verify (SHA-256 checksum audit)"
epic: EPIC-16
wave: 5
points: 3
priority: P1
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-package
behavioral_contracts:
  - BC-5.03.006
verification_properties: []
nfr_refs:
  - NFR-018
depends_on:
  - STORY-060
blocks: []
subsystems:
  - SS-16
target_module: slideforge-package
---

# STORY-063: Package: verify (SHA-256 checksum audit)

## Summary

Implement `slideforge package verify` — a read-only integrity audit that re-computes
the SHA-256 checksum of every installed package archive in the local cache and
compares it byte-for-byte against the value recorded in `sf.lock`. Any mismatch
indicates a corrupted download or tampered cache, and is reported as E-PKG-003.

The command is deliberately read-only: it never modifies `sf.lock`, re-downloads
packages, or alters any file. Its purpose is to give users and CI pipelines a
supply-chain integrity gate independent of the build process.

Key responsibilities:

1. **Load sf.lock** — parse all `LockedPackage` entries.
2. **For each package**, locate the cache archive at
   `~/.slideforge/cache/<sha256>.tar.gz`.
3. **Re-compute SHA-256** of the archive bytes using `sha2`.
4. **Compare** computed hex against `sf.lock` `sha256` field.
5. **Report**: all-OK or per-package E-PKG-003 with expected vs actual hashes.
6. **Exit code**: 0 if all match; 5 if any mismatch or missing archive.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-5.03.006 | slideforge package verify checks all packages against sf.lock SHA-256 checksums | Postconditions 1–5; Invariants 1–4 |

## Acceptance Criteria

- [ ] **AC-001** — `slideforge package verify` with all packages installed and checksums
  matching prints "All N packages verified." and exits 0.
  (traces to BC-5.03.006 postcondition 3, edge case EC-001)

- [ ] **AC-002** — If any package's cached archive has a SHA-256 that does not match
  the `sf.lock` value, E-PKG-003 is printed for that package showing the expected
  checksum and the actual computed checksum. Exit code is 5.
  (traces to BC-5.03.006 postcondition 4, edge case EC-002)

- [ ] **AC-003** — If a package is in `sf.lock` but its cache archive does not exist,
  E-PKG-001 is emitted for that package with a suggestion to run
  `slideforge package install`. Exit code is 5.
  (traces to BC-5.03.006 invariant 2, edge case EC-003)

- [ ] **AC-004** — `slideforge package verify` with `sf.lock` absent emits an error:
  "Cannot verify without sf.lock. Run: slideforge package install." Exit code is 5.
  (traces to BC-5.03.006 edge case EC-004)

- [ ] **AC-005** — `slideforge package verify` with an empty `[dependencies]` (or no
  packages in sf.lock) prints "No packages to verify." Exit code is 0.
  (traces to BC-5.03.006 edge case EC-005)

- [ ] **AC-006** — When multiple packages are present, a mismatch in ONE package does
  NOT stop verification of the remaining packages. All packages are verified; all
  mismatches are reported in a single run.
  (traces to BC-5.03.006 postcondition 4 — partial mismatch reporting)

- [ ] **AC-007** — `package verify` does NOT modify `sf.lock`, the cache, or any file.
  Running verify twice in succession on the same state produces identical output.
  (traces to BC-5.03.006 invariant 1, invariant 4)

- [ ] **AC-008** — The verification is deterministic: the same cache archive always
  produces the same SHA-256 (sha2 is deterministic; no randomness used).
  (traces to BC-5.03.006 invariant 4)

- [ ] **AC-009** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code,
  `clippy::pedantic` clean. All public items have rustdoc.
  (traces to NFR-021, NFR-022, NFR-024, NFR-023)

## Tasks

1. Implement `src/verify.rs`:
   ```rust
   pub struct VerifyResult {
       pub total: usize,
       pub passed: usize,
       pub failures: Vec<VerifyFailure>,
   }

   pub enum VerifyFailure {
       ChecksumMismatch {
           name: String,
           expected: String,
           actual: String,
       },
       MissingCache {
           name: String,
           expected_sha256: String,
       },
   }

   /// Verify all packages in sf.lock against their cached archives.
   pub fn verify(project_dir: &Path) -> Result<VerifyResult, PackageError>;
   ```
   Algorithm:
   ```
   1. If sf.lock absent → Err(PackageError::NoLockFile)
   2. Load sf.lock → packages: Vec<LockedPackage>
   3. If packages is empty → Ok(VerifyResult { total: 0, passed: 0, failures: [] })
   4. For each LockedPackage:
      a. archive_path = cache_dir / format!("{}.tar.gz", pkg.sha256)
      b. if !archive_path.exists() → failures.push(MissingCache)
      c. else:
         bytes = fs::read(archive_path)?
         actual = sha256_hex(&bytes)
         if actual != pkg.sha256 → failures.push(ChecksumMismatch)
         else → passed += 1
   5. Return VerifyResult { total: packages.len(), passed, failures }
   ```

2. Implement `format_verify_output(result: &VerifyResult) -> String`:
   ```rust
   pub fn format_verify_output(result: &VerifyResult) -> (String, bool) {
       // Returns (human_readable_output, should_exit_nonzero)
       if result.failures.is_empty() {
           (format!("All {} packages verified.", result.total), false)
       } else {
           let mut lines = Vec::new();
           for f in &result.failures {
               match f {
                   VerifyFailure::ChecksumMismatch { name, expected, actual } =>
                       lines.push(format!(
                           "E-PKG-003: '{}' checksum mismatch\n  expected: {}\n  actual:   {}",
                           name, expected, actual
                       )),
                   VerifyFailure::MissingCache { name, .. } =>
                       lines.push(format!(
                           "E-PKG-001: '{}' cache missing — run: slideforge package install",
                           name
                       )),
               }
           }
           lines.push(format!("{}/{} packages verified.", result.passed, result.total));
           (lines.join("\n"), true)
       }
   }
   ```

3. Wire CLI subcommand in `slideforge-cli/src/commands/package.rs`:
   ```rust
   pub fn run_verify(args: &VerifyArgs, _global: &GlobalFlags) -> ExitCode {
       match verify(&args.project_dir) {
           Err(PackageError::NoLockFile) => {
               eprintln!("Cannot verify without sf.lock. Run: slideforge package install");
               ExitCode::from(5)
           }
           Err(e) => { eprintln!("{e}"); ExitCode::from(5) }
           Ok(result) => {
               let (output, fail) = format_verify_output(&result);
               println!("{output}");
               if fail { ExitCode::from(5) } else { ExitCode::SUCCESS }
           }
       }
   }
   ```

4. Write all tests.

## File List

- `crates/slideforge-package/src/verify.rs` — `verify()`, `VerifyResult`,
  `VerifyFailure`, `format_verify_output()`
- `crates/slideforge-cli/src/commands/package.rs` — `run_verify()` handler added to
  existing package command file
- `crates/slideforge-package/tests/verify_integration.rs` — integration tests

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~3 500 |
| BC-5.03.006 | ~2 000 |
| STORY-060 (LockFile, sha256_hex, cache layout, PackageError) | ~3 000 |
| Target source files to write | ~2 500 |
| Test files | ~2 000 |
| **Total** | **~13 000** |

Context budget: 13 000 / 200 000 ≈ 6.5% — well within limit.

## Test Strategy

**Unit tests** (`crates/slideforge-package/src/verify.rs #[cfg(test)]`):

- `test_verify_empty_packages()`: empty `sf.lock`; assert `VerifyResult { total: 0,
  passed: 0, failures: [] }`.
- `test_verify_all_match()`: 2 packages with correct archives; assert all passed; exit 0.
- `test_verify_checksum_mismatch()`: corrupt one archive (write wrong bytes); assert
  `VerifyFailure::ChecksumMismatch` for that package; other package still passes.
- `test_verify_missing_cache()`: entry in sf.lock but no archive file; assert
  `VerifyFailure::MissingCache`.
- `test_verify_is_read_only()`: snapshot directory before; run verify; snapshot after;
  assert identical.
- `test_verify_deterministic()`: run verify twice on same state; assert identical
  `VerifyResult`.
- `test_format_all_verified()`: `VerifyResult { total: 2, passed: 2, failures: [] }`;
  assert output contains "All 2 packages verified.".
- `test_format_mismatch_shows_both_hashes()`: mismatch failure; assert output contains
  both `expected:` and `actual:` lines.
- `test_verify_no_lock_file()`: no `sf.lock`; assert `Err(PackageError::NoLockFile)`.

**Integration tests** (`crates/slideforge-package/tests/verify_integration.rs`):

- `test_verify_after_clean_install()`: install package from local bare repo; run
  `verify()`; assert all passed.
- `test_verify_after_cache_corruption()`: install; overwrite archive with garbage bytes;
  run `verify()`; assert `ChecksumMismatch` for that package; exit 5.
- `test_verify_after_cache_deletion()`: install; delete archive file; run `verify()`;
  assert `MissingCache`; exit 5.
- `test_verify_partial_failure()`: 2 packages; corrupt 1; assert both verified; 1
  failure; `passed = 1, total = 2`.

## Dependencies

- **Depends on:** STORY-060 (`LockFile`, `LockedPackage`, `sha256_hex()`, `cache_dir()`,
  and `PackageError` are all defined there — `verify` is a pure consumer of that
  infrastructure with no new writes)
- **Blocks:** (none)

## Dependency Anchor Justifications

- SS-16 owns this story's scope because SS-16 is the Package Management subsystem
  owning `slideforge-package` per ARCH-INDEX Subsystem Registry.
- STORY-063 depends on STORY-060 because `verify()` reads the `LockFile` and computes
  SHA-256 of archives using `sha256_hex()` — both defined in STORY-060. The cache path
  formula (`~/.slideforge/cache/<sha256>.tar.gz`) is also defined there.

## Architecture Compliance Rules

1. `package verify` is strictly **read-only**. The implementation MUST NOT call any
   function that writes to the filesystem. This is enforced by test: snapshot before/after.
2. The SHA-256 computation reuses `sha256_hex()` from `crates/slideforge-package/src/hash.rs`
   (defined in STORY-060). Do NOT inline a second implementation.
3. All packages in `sf.lock` are verified in a single pass — do NOT short-circuit on
   first failure. All failures are collected and returned.
4. `verify()` returns `Ok(VerifyResult)` even when there are failures. `Err` is reserved
   for I/O errors (can't read sf.lock, etc.) — not for checksum mismatches.
5. `#![forbid(unsafe_code)]` at the crate root.

**Forbidden dependencies for `slideforge-package`:**
- Must NOT import `slideforge-eval` or `slideforge-syntax`.
- Must NOT use any network I/O in `verify()` — the verification is against the local
  cache only (no re-fetch on mismatch).

## Library and Framework Requirements

All versions centralized in `[workspace.dependencies]` per ADR-022; crate uses `{ workspace = true }`.
No new dependencies beyond what STORY-060 already established.

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `sha2` | `=0.11.0` | Re-compute SHA-256 of archive — reuses `sha256_hex()` from STORY-060 `src/hash.rs` (finalize() → hybrid_array::Array → [u8;32] via .into()) |
| `hex` | `=0.4.3` | Hex encoding (already present) |
| `toml` | `=1.1.2` | sf.lock parsing (already present from STORY-060) |
| `thiserror` | `=2.0.18` | PackageError (already present) |

No new dependencies are needed. This story only adds code within the existing crate.

**Determinism note for AC-008**: The verification is deterministic because sha2 is
deterministic and the archive bytes in the cache are fixed after install. However,
whether a re-fetch (on a different OS) reproduces identical bytes couples to STORY-060's
reproducible-archive risk (flate2 OS-byte caveat). The cross-platform CI byte-diff test
from STORY-060 must pass before AC-008 can be considered fully validated.

## File Structure Requirements

```
crates/slideforge-package/
  src/
    verify.rs   # verify(), VerifyResult, VerifyFailure, format_verify_output()
    lib.rs      # add: pub use verify::{verify, VerifyResult, VerifyFailure};
  tests/
    verify_integration.rs
crates/slideforge-cli/
  src/
    commands/
      package.rs  # run_verify() added to existing package.rs
```

## Previous Story Intelligence

STORY-060 defined:
- `sha256_hex(data: &[u8]) -> String` in `src/hash.rs`
- `cache_dir() -> PathBuf` in `src/cache.rs`
- `LockFile::load(path)` in `src/lock.rs`
- `PackageError` enum in `src/error.rs`

STORY-063 uses all four. Read STORY-060's implementation before writing `verify.rs`
to get the exact function signatures and error variant names.

## Implementation Notes

### Why verify doesn't re-fetch on mismatch

`verify` is intentionally non-destructive. A checksum mismatch on a package that was
previously installed might indicate:
- Supply chain attack (archive tampered in cache)
- Disk corruption
- Incomplete download

In all these cases, re-downloading silently would mask the problem. The correct
remediation is human-reviewed: `slideforge package remove <pkg> && slideforge package
install <url>` to get a fresh install.

### Exit code 5 for all package errors

All package command errors use exit code 5. This is distinct from:
- Exit 1 (parse error)
- Exit 2 (validation error)
- Exit 3 (export error)
- Exit 4 (configuration error / E-CFG-*)
- Exit 0 (success)

### NFR-018 reference

NFR-018 ("SHA-256 integrity verified on use, 100% verified") is the supply-chain
requirement that `verify` satisfies. This story implements the explicit audit command;
STORY-061 implements the implicit verification-on-load (re-verify on cache miss).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All packages match sf.lock | "All N packages verified."; exit 0 |
| EC-002 | One package archive checksum mismatches | E-PKG-003 for that package; exit 5; other packages still verified and reported |
| EC-003 | Package in [dependencies] but local cache missing | E-PKG-001 for missing package; suggest re-running install; exit 5 |
| EC-004 | sf.lock absent | Error: cannot verify without sf.lock; exit 5 |
| EC-005 | [dependencies] is empty / no packages in sf.lock | "No packages to verify."; exit 0 |
| EC-006 | Two packages: one missing cache, one checksum mismatch | Both errors reported; exit 5; passed count shows how many clean |
