---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-064
title: "Workspace: slideforge.toml [workspace] + build --workspace"
epic: EPIC-17
wave: 5
points: 5
priority: P1
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-config
behavioral_contracts:
  - BC-5.04.001
verification_properties: []
nfr_refs: []
depends_on:
  - STORY-055
blocks:
  - STORY-065
subsystems:
  - SS-17
  - SS-18
target_module: slideforge-config
---

# STORY-064: Workspace: slideforge.toml [workspace] + build --workspace

## Summary

Implement the workspace build system in `slideforge-config`. A `slideforge.toml` with
a `[workspace]` section declares a multi-deck workspace (Cargo-style). `slideforge
build --workspace` discovers all member decks matching the `members` glob list, builds
each independently, error-isolates failures between members, and produces a summary
report.

Key responsibilities:

1. **Workspace discovery**: parse `[workspace] members = [...]` from `slideforge.toml`.
   Members can be explicit paths or glob patterns (e.g., `"decks/*"`). Discovery is
   deterministic: alphabetical order within each glob expansion.
2. **Per-member build**: each member deck is built with the root `slideforge.toml`
   settings as baseline, overridden by any member-local `slideforge.toml` (if present in
   the member's directory). One member failing does NOT prevent other members from
   building.
3. **Summary report**: after all members finish, print a table: N built OK, M failed,
   with the per-member status and output path.
4. **Flag exclusivity**: `--workspace` and a positional source file argument are mutually
   exclusive (E-CFG-002).
5. **Empty workspace**: `members = []` produces E-CFG-005 (no members found).

This story lives primarily in `slideforge-config` (workspace parsing + member discovery)
and `slideforge-cli` (the `--workspace` flag and build loop).

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-5.04.001 | slideforge.toml [workspace] declares multi-deck members; build --workspace builds all | Postconditions 1–7; Invariants 1–3 |

## Acceptance Criteria

- [ ] **AC-001** — `slideforge build --workspace` in a directory with a valid
  `slideforge.toml [workspace]` section discovers all member decks, builds each, and
  writes output to `<member-dir>/dist/` (configurable via member's `output_dir`). Exit
  code is 0 if all succeed.
  (traces to BC-5.04.001 postcondition 1, postcondition 2, postcondition 3)

- [ ] **AC-002** — When one member deck has a parse error, the other members still
  build successfully. The summary report shows which member failed and which succeeded.
  Exit code is non-zero.
  (traces to BC-5.04.001 postcondition 4, postcondition 5, postcondition 6, edge case EC-002)

- [ ] **AC-003** — Summary report format: `"N/M decks built successfully."` followed by
  per-deck status lines with the member path and either `OK` or `FAILED: <error-summary>`.
  (traces to BC-5.04.001 postcondition 5)

- [ ] **AC-004** — `slideforge build deck.sf --workspace` (source file + workspace flag)
  emits E-CFG-002: "Cannot combine source file with --workspace flag." Exit code 64
  (usage error).
  (traces to BC-5.04.001 invariant 1, edge case EC-003)

- [ ] **AC-005** — `slideforge build --workspace` when `slideforge.toml` has no
  `[workspace]` section emits E-CFG-005: "No [workspace] section in slideforge.toml."
  Exit code 4.
  (traces to BC-5.04.001 postcondition 7, edge case — no workspace section)

- [ ] **AC-006** — `[workspace] members = []` (empty) emits E-CFG-005 or warning:
  "No workspace members found." Exit code non-zero.
  (traces to BC-5.04.001 edge case EC-001)

- [ ] **AC-007** — Member discovery order is alphabetical within each glob expansion.
  Two runs of `build --workspace` on the same project produce the same member order.
  (traces to BC-5.04.001 invariant 2)

- [ ] **AC-008** — Workspace root `slideforge.toml` settings apply as baseline to all
  members. A member-local `slideforge.toml` in the member directory (if present) overrides
  root settings for that member only.
  (traces to BC-5.04.001 invariant 3)

- [ ] **AC-009** — `slideforge build --workspace` with `members = ["decks/*"]` where
  the glob matches 50 directories builds all 50; summary shows 50 entries.
  (traces to BC-5.04.001 edge case EC-005)

- [ ] **AC-010** — A nested `[workspace]` inside a member deck (if one existed) is
  IGNORED by the outer workspace build — no recursion.
  (traces to BC-5.04.001 edge case EC-004)

- [ ] **AC-011** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code,
  `clippy::pedantic` clean. All public items have rustdoc.
  (traces to NFR-021, NFR-022, NFR-024, NFR-023)

## Tasks

1. Create the `slideforge-config` crate in the Cargo workspace:
   All dependencies use `{ workspace = true }` — centralized in `[workspace.dependencies]`
   per ADR-022. New crate added to workspace table per ADR-022.
   ```toml
   # crates/slideforge-config/Cargo.toml
   [package]
   name = "slideforge-config"
   version = "0.1.0"
   edition = "2024"

   [dependencies]
   toml = { workspace = true }
   serde = { workspace = true }
   globset = { workspace = true }
   walkdir = { workspace = true }
   thiserror = { workspace = true }
   tracing = { workspace = true }
   ```

2. Define the `WorkspaceConfig` model in `src/workspace.rs`:
   ```rust
   // HAZARD: #[serde(flatten)] with Option<WorkspaceSection> under toml 1.x is
   // INCONCLUSIVE — behavior may differ from serde_json. REQUIRED: write a
   // serialize/deserialize unit test against toml =1.1.2 for SlideForgeTOML before
   // committing to this layout. If the flatten + Option<table> combination errors,
   // fall back to explicit nesting (i.e., keep workspace as a named field, not flattened).
   #[derive(Debug, Clone, Deserialize)]
   pub struct SlideForgeTOML {
       #[serde(rename = "workspace")]
       pub workspace: Option<WorkspaceSection>,
       // NOTE: Do NOT use #[serde(flatten)] for ProjectSettings without first passing
       // the unit test described above. Use explicit named fields or a named sub-table
       // if flatten proves unreliable with toml =1.1.2.
       pub settings: Option<ProjectSettings>,
   }

   #[derive(Debug, Clone, Deserialize)]
   pub struct WorkspaceSection {
       pub members: Vec<String>,  // glob patterns or explicit paths
   }

   #[derive(Debug, Clone, Deserialize, Default)]
   pub struct ProjectSettings {
       pub brand: Option<String>,
       pub output_dir: Option<String>,
       // additional settings fields from .sfconfig / slideforge.toml
   }
   ```

3. Implement `discover_members(workspace_root: &Path, members_globs: &[String]) -> Vec<PathBuf>`
   in `src/discovery.rs`:

   Use `globset =0.4.18` paired with `walkdir =2.5.0` for member discovery (preferred
   over `glob =0.3` which lacks brace expansion and has cross-platform quirks).

   DEDUP FIX: `Vec::dedup()` only removes CONSECUTIVE duplicates. The full path list
   must be SORTED before dedup to ensure all duplicates from overlapping globs are
   removed. Sort the full accumulated `paths` vec, then dedup (EC-007).

   ```rust
   use globset::{Glob, GlobSetBuilder};
   use walkdir::WalkDir;

   pub fn discover_members(
       workspace_root: &Path,
       members_globs: &[String],
   ) -> Result<Vec<PathBuf>, ConfigError> {
       let mut builder = GlobSetBuilder::new();
       for pattern in members_globs {
           let glob = Glob::new(pattern).map_err(|_| ConfigError::InvalidGlob)?;
           builder.add(glob);
       }
       let glob_set = builder.build().map_err(|_| ConfigError::InvalidGlob)?;

       let mut paths: Vec<PathBuf> = WalkDir::new(workspace_root)
           .min_depth(1)
           .max_depth(2)  // adjust depth based on expected workspace layout
           .into_iter()
           .filter_map(|e| e.ok())
           .filter(|e| e.file_type().is_dir())
           .filter(|e| {
               // Match relative path from workspace_root against glob patterns
               let rel = e.path().strip_prefix(workspace_root).ok();
               rel.map(|r| glob_set.is_match(r)).unwrap_or(false)
           })
           .map(|e| e.into_path())
           .collect();

       // CRITICAL: sort the FULL list before dedup (Vec::dedup removes consecutive dups
       // only — sorting first ensures all cross-glob duplicates are adjacent).
       paths.sort();
       paths.dedup();

       if paths.is_empty() {
           return Err(ConfigError::NoMembersFound);
       }
       Ok(paths)
   }
   ```

4. Implement `load_workspace_config(root: &Path) -> Result<SlideForgeTOML, ConfigError>`
   in `src/loader.rs`:
   - Read and parse `slideforge.toml` from `root`.
   - If `workspace` section absent → `ConfigError::NoWorkspaceSection`.
   - Return the parsed `SlideForgeTOML`.

5. Implement `merge_member_settings(root_settings: &ProjectSettings, member_dir: &Path) -> ProjectSettings`
   in `src/merge.rs`:
   - If `member_dir/slideforge.toml` exists, parse its `ProjectSettings` section.
   - Apply 11-level merge: last-wins scalars, replace lists, deep-merge maps per DI-020.
   - Ignore `[workspace]` section in member-local `slideforge.toml` (no recursion).
   - Return merged settings.

6. Implement `build_workspace(workspace_root: &Path, global: &GlobalFlags) -> WorkspaceBuildResult`
   in `slideforge-cli/src/commands/workspace.rs`:
   ```rust
   pub struct WorkspaceBuildResult {
       pub total: usize,
       pub succeeded: usize,
       pub failures: Vec<MemberFailure>,
   }

   pub struct MemberFailure {
       pub member_path: PathBuf,
       pub error_summary: String,
   }
   ```
   Algorithm:
   ```
   1. load_workspace_config(workspace_root)
   2. discover_members(workspace_root, globs)
   3. For each member in sorted order:
      a. merged = merge_member_settings(root_settings, member_dir)
      b. result = run_single_build(member_dir / "deck.sf", merged, global)
         (using the same build pipeline as STORY-055 run_build())
      c. if Ok → succeeded += 1
      d. if Err → failures.push(MemberFailure { member_path, error_summary })
   4. Return WorkspaceBuildResult
   ```

7. Add `--workspace` flag to `BuildArgs` in `slideforge-cli/src/cli.rs`:
   ```rust
   #[derive(Args)]
   pub struct BuildArgs {
       /// Path to the .sf source file (mutually exclusive with --workspace)
       pub source: Option<PathBuf>,
       #[arg(long)]
       pub workspace: bool,
       // ...other fields...
   }
   ```
   Validate mutual exclusivity: if `source.is_some() && workspace` → E-CFG-002.

8. Wire the workspace build path in `run_build()`:
   ```rust
   if args.workspace {
       if args.source.is_some() {
           eprintln!("E-CFG-002: Cannot combine source file with --workspace");
           return ExitCode::from(64);
       }
       let cwd = std::env::current_dir()?;
       let result = build_workspace(&cwd, global);
       print_workspace_summary(&result);
       return if result.failures.is_empty() { ExitCode::SUCCESS } else { ExitCode::from(1) };
   }
   ```

9. Write all tests.

## File List

- `crates/slideforge-config/Cargo.toml` — new crate manifest
- `crates/slideforge-config/src/lib.rs` — pub API re-exports
- `crates/slideforge-config/src/error.rs` — `ConfigError` enum (E-CFG-002, E-CFG-005)
- `crates/slideforge-config/src/workspace.rs` — `SlideForgeTOML`, `WorkspaceSection`,
  `ProjectSettings`
- `crates/slideforge-config/src/discovery.rs` — `discover_members()`
- `crates/slideforge-config/src/loader.rs` — `load_workspace_config()`
- `crates/slideforge-config/src/merge.rs` — `merge_member_settings()`
- `crates/slideforge-cli/src/cli.rs` — add `--workspace` flag to `BuildArgs`; make
  `source` optional
- `crates/slideforge-cli/src/commands/workspace.rs` — `build_workspace()`,
  `WorkspaceBuildResult`, `MemberFailure`, `print_workspace_summary()`
- `crates/slideforge-config/tests/discovery_integration.rs` — integration tests
- `crates/slideforge-cli/tests/workspace_integration.rs` — CLI integration tests

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~5 000 |
| BC-5.04.001 | ~2 500 |
| STORY-055 (BuildArgs, GlobalFlags, run_build() — the single-deck path this extends) | ~3 500 |
| Target source files to write | ~5 000 |
| Test files | ~3 000 |
| **Total** | **~19 000** |

Context budget: 19 000 / 200 000 ≈ 9.5% — within limit.

## Test Strategy

**Unit tests** (`crates/slideforge-config/src/discovery.rs #[cfg(test)]`):

- `test_discover_explicit_paths()`: workspace with `members = ["decks/a", "decks/b"]`;
  both dirs exist; assert discovered in alphabetical order.
- `test_discover_glob()`: workspace with `members = ["decks/*"]`; 3 matching dirs;
  assert all 3 discovered, sorted.
- `test_discover_empty_glob()`: glob matches nothing; assert `ConfigError::NoMembersFound`.
- `test_discover_deduplication()`: glob overlaps produce same dir twice; assert only
  1 entry in result.
- `test_discover_alphabetical_order()`: dirs named "z", "a", "m"; assert order a < m < z.
- `test_merge_member_overrides_brand()`: root sets `brand = "corp.pptx"`, member-local
  sets `brand = "client.pptx"`; merged → "client.pptx".
- `test_merge_member_absent()`: no member-local `slideforge.toml`; merged → root settings unchanged.
- `test_merge_ignores_workspace_section()`: member-local `slideforge.toml` has `[workspace]`;
  merged settings do NOT contain workspace section.
- `test_load_no_workspace_section()`: `slideforge.toml` without `[workspace]`; assert
  `ConfigError::NoWorkspaceSection`.
- `test_slideforge_toml_serde_flatten_roundtrip()`: serialize a `SlideForgeTOML` with
  both `workspace` and `settings` populated to a TOML string using toml =1.1.2, then
  deserialize back; assert round-trip equality. REQUIRED before committing the
  `#[serde(flatten)]` layout — if this test errors, switch to explicit field nesting.

**Integration tests** (`crates/slideforge-cli/tests/workspace_integration.rs`):

- `test_workspace_builds_all_members()`: workspace with 3 valid member decks; run
  `build --workspace`; assert 3 output files; exit 0.
- `test_workspace_error_isolation()`: workspace with 3 members; 1 has deliberate parse
  error; assert other 2 build; exit non-zero; error-member in failures list.
- `test_workspace_source_and_flag_exclusive()`: `build deck.sf --workspace`; assert
  E-CFG-002; exit 64.
- `test_workspace_no_workspace_section()`: `slideforge.toml` without `[workspace]`;
  assert E-CFG-005; exit 4.
- `test_workspace_empty_members()`: `members = []`; assert E-CFG-005 or warning; exit
  non-zero.
- `test_workspace_summary_format()`: 2 success + 1 failure; assert summary shows
  "2/3" and failure path.

## Dependencies

- **Depends on:** STORY-055 (defines `BuildArgs`, `GlobalFlags`, and `run_build()` for
  single-deck builds — the workspace build loop calls `run_build()` for each member)
- **Blocks:** STORY-065 (`.sfconfig` cascade depends on `ProjectSettings` and the
  `merge_member_settings()` infrastructure defined here; `config explain` also reads
  from the workspace config model)

## Dependency Anchor Justifications

- SS-17 owns the workspace parsing, discovery, and merge logic in `slideforge-config`
  per ARCH-INDEX Subsystem Registry. SS-18 (CLI) owns the `--workspace` flag and the
  `build_workspace()` command orchestration.
- STORY-064 depends on STORY-055 because the workspace build loop calls
  `run_single_build()` which is the same build pipeline established in STORY-055. The
  workspace command is an orchestration layer on top of the single-deck build path.
- STORY-064 blocks STORY-065 because `ProjectSettings`, `SlideForgeTOML`, and
  `merge_member_settings()` are the data structures that STORY-065's `.sfconfig` cascade
  extends. STORY-065 can't add cascade levels without the merge infrastructure.

## Architecture Compliance Rules

1. `slideforge-config` is a **pure configuration-parsing** crate — no pipeline
   execution, no I/O beyond reading TOML files.
2. The workspace build loop in `slideforge-cli/src/commands/workspace.rs` does NOT
   re-implement build logic — it calls the same `run_single_build()` used by the single
   `build` command.
3. Nested workspace sections (`[workspace]` inside a member's `slideforge.toml`) are
   silently ignored — no error, no recursion. The workspace build is single-level.
4. Error isolation between members is mandatory: one member panicking (via `?` / `Err`)
   must be caught at the member-loop boundary and recorded as a failure, not propagated
   up to terminate the entire workspace build.
5. `#![forbid(unsafe_code)]` at the crate root for `slideforge-config`.
6. **Dedup correctness (EC-007)**: `Vec::dedup()` only removes CONSECUTIVE duplicates.
   The full path list from all glob expansions must be SORTED before calling `dedup()`,
   otherwise paths matched by multiple overlapping globs will not be deduplicated.
   Always: `paths.sort(); paths.dedup();` — never `paths.dedup()` alone.
7. **serde(flatten) + Option<table> (toml 1.x)**: Write a serialize/deserialize round-trip
   unit test for `SlideForgeTOML` against toml =1.1.2 before committing. If
   `#[serde(flatten)]` + `Option<WorkspaceSection>` produces an error, use explicit
   named field layout instead.

**Forbidden dependencies for `slideforge-config`:**
- Must NOT import `slideforge-eval`, `slideforge-syntax`, or any exporter crate.
- Must NOT perform network I/O or git operations.
- Must NOT call `slideforge::compile()` — that is the CLI's responsibility.

## Library and Framework Requirements

All versions centralized in `[workspace.dependencies]` per ADR-022; crate uses `{ workspace = true }`.

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `toml` | `=1.1.2` | Parse `slideforge.toml` and member-local overrides |
| `serde` | `=1.0.228` | Deserialize `SlideForgeTOML`, `ProjectSettings` (NOTE: test #[serde(flatten)] + Option<table> against =1.1.2 before committing; fall back to explicit nesting if it errors) |
| `globset` | `=0.4.18` | Glob pattern expansion for `members` list (preferred over glob =0.3 — supports brace expansion and cross-platform patterns; pair with walkdir) |
| `walkdir` | `=2.5.0` | Directory traversal for globset-based member discovery |
| `thiserror` | `=2.0.18` | `ConfigError` enum |
| `tracing` | `=0.1.44` | Span per member-build in workspace loop |

## File Structure Requirements

```
crates/slideforge-config/
  src/
    lib.rs           # pub use: WorkspaceConfig, ProjectSettings, discover_members, ConfigError
    error.rs         # ConfigError: NoWorkspaceSection, NoMembersFound, InvalidGlob, ...
    workspace.rs     # SlideForgeTOML, WorkspaceSection, ProjectSettings structs
    discovery.rs     # discover_members(workspace_root, globs) -> Vec<PathBuf>
    loader.rs        # load_workspace_config(path) -> Result<SlideForgeTOML, ConfigError>
    merge.rs         # merge_member_settings(root, member_dir) -> ProjectSettings
  tests/
    discovery_integration.rs
  Cargo.toml
crates/slideforge-cli/
  src/
    cli.rs              # add: --workspace flag; source becomes Option<PathBuf>
    commands/
      workspace.rs      # build_workspace(), WorkspaceBuildResult, print_workspace_summary()
  tests/
    workspace_integration.rs
```

## Previous Story Intelligence

N/A — STORY-064 is the first workspace story. However, STORY-055 defined the single-deck
`run_build()` function and `BuildArgs`. Read STORY-055's `commands/build.rs` before
implementing to understand the exact signature of the single-deck build call that the
workspace loop will invoke per member.

Key constraint: `source` in `BuildArgs` changes from `PathBuf` (required positional) to
`Option<PathBuf>` (optional) in this story to support `--workspace`. Ensure backward
compatibility: if `--workspace` is false and `source` is None, emit a "source file
required" error to the user.

## Implementation Notes

### Member discovery: what counts as a "member"

A member path from `members = [...]` is expected to point to a DIRECTORY containing
a `.sf` file (typically `deck.sf`). The workspace builder looks for:
1. `<member-dir>/deck.sf` (default main source file name)
2. If not found: `<member-dir>/*.sf` (first .sf file alphabetically)
3. If still not found: member is skipped with a warning.

Future: allow member paths to point directly to `.sf` files (v1.x enhancement).

### 11-level merge precedence for member settings

Per DI-020 and the q16-q25-decisions.md:

```
last-wins scalars:   member value wins over root value
replace lists:       member list replaces (not appends to) root list
deep-merge maps:     member map keys override root keys; absent member keys keep root value
```

For v1.0, `ProjectSettings` contains only scalar fields (`brand: Option<String>`,
`output_dir: Option<String>`) — no lists or maps. Deep-merge rules will be exercised
by STORY-065's `.sfconfig` cascade.

### Error codes

| Code | Meaning | Exit |
|------|---------|------|
| E-CFG-002 | Mutually exclusive flags (source + --workspace) | 64 (usage error) |
| E-CFG-005 | Configuration resource not found (no [workspace] section, or no members) | 4 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `members = []` (empty list) | E-CFG-005 or warning; exit non-zero |
| EC-002 | One member has a parse error | That member fails; others build successfully |
| EC-003 | `slideforge build deck.sf --workspace` | E-CFG-002: mutually exclusive flags; exit 64 |
| EC-004 | Nested workspace (member contains [workspace]) | Inner [workspace] ignored; no recursion |
| EC-005 | Glob `decks/*` matches 50 members | All 50 built; summary shows 50 entries |
| EC-006 | Member directory has no .sf file | Member skipped with warning; does not count as failure |
| EC-007 | Same path matched by two overlapping globs | Deduplicated; built once |
