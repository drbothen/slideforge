---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-065
title: "Workspace: .sfconfig cascade + config explain provenance"
epic: EPIC-17
wave: 5
points: 5
priority: P1
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-config
behavioral_contracts:
  - BC-5.04.002
  - BC-5.04.003
verification_properties: []
nfr_refs: []
depends_on:
  - STORY-064
blocks: []
subsystems:
  - SS-17
  - SS-18
target_module: slideforge-config
---

# STORY-065: Workspace: .sfconfig cascade + config explain provenance

## Summary

Extend the workspace configuration system with two features:

1. **`.sfconfig` cascade** (BC-5.04.002) — workspace-subdirectory TOML files that
   override root `slideforge.toml` settings at up to 3 cascade levels. The merge rule
   is last-wins scalars / replace lists / deep-merge maps (DI-020). A 4th-level
   `.sfconfig` triggers E-CFG-006 warning and is ignored. A malformed `.sfconfig`
   produces E-CFG-006 with the file path. The `[workspace]` key is not valid inside
   `.sfconfig`.

2. **`slideforge config explain`** (BC-5.04.003) — a diagnostic command that prints
   the resolved value of each configuration key with the file that set it and the
   cascade level (workspace-root / sfconfig-level-2 / sfconfig-level-1 / deck-local /
   cli-flag / default). Supports `--format json` for machine-readable output.

Both features build on the `ProjectSettings`, `SlideForgeTOML`, and `merge_member_settings()`
infrastructure defined in STORY-064.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-5.04.002 | .sfconfig cascade (max 3 levels) applies family-specific overrides | Postconditions 1–5; Invariants 1–3 |
| BC-5.04.003 | slideforge config explain shows configuration provenance for any setting | Postconditions 1–5; Invariants 1–3 |

## Acceptance Criteria

### `.sfconfig` cascade

- [ ] **AC-001** — A deck-local `.sfconfig` in the same directory as the deck's `.sf`
  file overrides the workspace root `slideforge.toml` setting for that deck only. Other
  decks without a `.sfconfig` use the root settings.
  (traces to BC-5.04.002 postcondition 1)

- [ ] **AC-002** — An intermediate `.sfconfig` (one level above the deck, one below
  workspace root) applies as a second override layer:
  deck-local `.sfconfig` > intermediate `.sfconfig` > workspace root.
  (traces to BC-5.04.002 postcondition 2)

- [ ] **AC-003** — The cascade depth is exactly 3 levels (workspace root →
  intermediate → deck-local). A `.sfconfig` at depth 4 triggers E-CFG-006 warning and
  is ignored; cascade stops at depth 3.
  (traces to BC-5.04.002 postcondition 3, invariant 1, edge case EC-003)

- [ ] **AC-004** — Settings absent from a `.sfconfig` fall through to the higher-level
  config. Only the keys present in `.sfconfig` are overridden.
  (traces to BC-5.04.002 postcondition 4, invariant 3)

- [ ] **AC-005** — A malformed `.sfconfig` (invalid TOML) produces E-CFG-006 with the
  file path and parse detail. The deck-level build fails.
  (traces to BC-5.04.002 postcondition 5, edge case — malformed)

- [ ] **AC-006** — A `.sfconfig` containing a `[workspace]` section triggers E-CFG-006
  warning: "workspace section not valid in .sfconfig". That key is ignored; other valid
  keys in the file are applied.
  (traces to BC-5.04.002 invariant 2, edge case EC-002)

- [ ] **AC-007** — The merge rule for config values is: last-wins scalars (deck-local
  value wins), replace lists (deck-local list replaces root list entirely), deep-merge
  maps (deck-local map keys override root keys; absent keys fall through).
  (traces to BC-5.04.002 invariant 3 — DI-020)

### `slideforge config explain`

- [ ] **AC-008** — `slideforge config explain brand` shows the resolved value of the
  `brand` key, the source file path, and the cascade level. The 5 distinct provenance
  levels are: `cli-flag`, `sfconfig-level-1` (deck directory; equivalent to `deck-local`),
  `sfconfig-level-2` (parent directory), `workspace-root`, and `default`.
  (traces to BC-5.04.003 postcondition 1, postcondition 2)

- [ ] **AC-009** — `slideforge config explain` (no key argument) shows ALL configuration
  keys with their resolved values and provenances.
  (traces to BC-5.04.003 postcondition 3)

- [ ] **AC-010** — `slideforge config explain unknown-key` emits E-CFG-006 with a list
  of valid configuration keys. Exit code 4.
  (traces to BC-5.04.003 postcondition 4, edge case EC-002)

- [ ] **AC-011** — `slideforge config explain --format json` produces valid JSON:
  `{"key": {"value": "...", "provenance": "...", "source": "..."}}`. Exit code 0.
  (traces to BC-5.04.003 postcondition 5, edge case EC-003)

- [ ] **AC-012** — `slideforge config explain` run with a `--template path/to/brand.pptx`
  CLI flag shows `template` key with provenance `cli-flag` and the flag value.
  (traces to BC-5.04.003 invariant 2, edge case EC-004)

- [ ] **AC-013** — `slideforge config explain` does NOT modify any config files. Running
  it twice in succession produces identical output.
  (traces to BC-5.04.003 invariant 1)

- [ ] **AC-014** — `slideforge config explain` run outside a project directory (no
  `slideforge.toml` in cwd or parents) emits E-CFG-005: "workspace root not found".
  Exit code 4.
  (traces to BC-5.04.003 edge case EC-001)

- [ ] **AC-015** — Default configuration values (not set in any file) are shown with
  provenance `default` and no source file path.
  (traces to BC-5.04.003 invariant 3)

- [ ] **AC-016** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code,
  `clippy::pedantic` clean. All public items have rustdoc.
  (traces to NFR-021, NFR-022, NFR-024, NFR-023)

## Tasks

1. Implement cascade loading in `crates/slideforge-config/src/cascade.rs`:
   ```rust
   pub struct CascadeLevel {
       pub level: ConfigLevel,
       pub source: Option<PathBuf>,      // None for defaults and cli-flags
       pub settings: ProjectSettings,
   }

   #[derive(Debug, Clone, PartialEq, Eq)]
   pub enum ConfigLevel {
       Default,
       WorkspaceRoot,
       SfconfigLevel2,  // intermediate directory
       SfconfigLevel1,  // deck-local directory (closest to the deck)
       CliFlag,
   }

   /// Walk from `deck_dir` up toward `workspace_root`, collecting .sfconfig files.
   /// Maximum depth: workspace_root + 2 intermediate dirs (total 3 levels).
   pub fn collect_sfconfig_levels(
       workspace_root: &Path,
       deck_dir: &Path,
   ) -> Result<Vec<CascadeLevel>, Vec<ConfigError>>;
   ```
   Algorithm:
   ```
   levels = []
   current = deck_dir
   depth = 0
   while current != workspace_root && depth < 3:
       sfconfig_path = current / ".sfconfig"
       if sfconfig_path.exists():
           parsed = parse_sfconfig(sfconfig_path)?
           level_enum = match depth { 0 => SfconfigLevel1, 1 => SfconfigLevel2, _ => warn+ignore }
           levels.push(CascadeLevel { level: level_enum, source: sfconfig_path, settings: parsed })
       current = current.parent()
       depth += 1
   if depth == 3 and (current / ".sfconfig").exists():
       emit E-CFG-006 warning: "depth 4 .sfconfig ignored"
   levels.reverse()   // workspace-root-first order for merge
   ```

2. Implement `parse_sfconfig(path: &Path) -> Result<ProjectSettings, ConfigError>` in
   `src/cascade.rs`:
   - Parse TOML from path.
   - If `[workspace]` key present in parsed table: emit E-CFG-006 warning; remove key.
   - Deserialize remaining content into `ProjectSettings`.
   - On TOML parse error: `ConfigError::MalformedSfconfig { path, detail }`.

3. Implement `merge_cascade(levels: &[CascadeLevel]) -> ResolvedConfig` in
   `src/cascade.rs`:
   ```rust
   pub struct ResolvedConfig {
       pub settings: ProjectSettings,
       pub provenance: HashMap<String, ProvenanceEntry>,
   }

   pub struct ProvenanceEntry {
       pub value: String,
       pub level: ConfigLevel,
       pub source: Option<PathBuf>,
   }
   ```
   For each field in `ProjectSettings`: the value from the highest-precedence non-None
   level wins. Track which level set each field in `provenance`.

4. Implement `slideforge config explain` command in `slideforge-cli/src/commands/config.rs`:
   ```rust
   pub fn run_config_explain(
       key: Option<&str>,
       format: ExplainFormat,
       global: &GlobalFlags,
   ) -> ExitCode;

   pub enum ExplainFormat { Text, Json }
   ```
   Algorithm:
   - Find workspace root: walk up from cwd looking for `slideforge.toml` with `[workspace]`.
   - Load cascade levels for cwd as "deck dir".
   - Apply CLI flags to provenance map with level `CliFlag`.
   - If `key` is Some: look up; if not found → E-CFG-006 with valid-key list.
   - If `key` is None: show all keys.
   - Format and print per `ExplainFormat`.

5. Implement text and JSON output formatters in `src/explain_format.rs`:
   ```rust
   pub fn format_text(key: &str, entry: &ProvenanceEntry) -> String {
       let source_str = entry.source.as_deref()
           .map(|p| p.display().to_string())
           .unwrap_or_default();
       format!(
           "{} = {:?}\n  (from: {}, level: {:?})",
           key, entry.value, source_str, entry.level
       )
   }

   pub fn format_json(resolved: &ResolvedConfig) -> String {
       // serde_json::to_string_pretty of a HashMap<String, JsonProvenanceEntry>
   }
   ```

6. Wire `Command::Config(ConfigArgs)` in `slideforge-cli/src/cli.rs`:
   ```rust
   #[derive(Args)]
   pub struct ConfigArgs {
       #[command(subcommand)]
       pub subcommand: ConfigSubcommand,
   }

   #[derive(Subcommand)]
   pub enum ConfigSubcommand {
       Explain(ExplainArgs),
   }

   #[derive(Args)]
   pub struct ExplainArgs {
       pub key: Option<String>,
       #[arg(long, value_enum, default_value = "text")]
       pub format: ExplainFormat,
   }
   ```

7. Write all tests.

## File List

- `crates/slideforge-config/src/cascade.rs` — `CascadeLevel`, `ConfigLevel`,
  `collect_sfconfig_levels()`, `parse_sfconfig()`, `merge_cascade()`, `ResolvedConfig`,
  `ProvenanceEntry`
- `crates/slideforge-config/src/explain_format.rs` — `format_text()`, `format_json()`
- `crates/slideforge-cli/src/commands/config.rs` — `run_config_explain()`
- `crates/slideforge-cli/src/cli.rs` — `Command::Config(ConfigArgs)`, `ExplainArgs`,
  `ExplainFormat` (adds to existing file from STORY-055)
- `crates/slideforge-config/tests/cascade_integration.rs` — integration tests
- `crates/slideforge-cli/tests/config_explain_integration.rs` — CLI integration tests

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~5 500 |
| BC-5.04.002 + BC-5.04.003 | ~4 500 |
| STORY-064 (ProjectSettings, SlideForgeTOML, merge_member_settings infrastructure) | ~3 500 |
| STORY-055 (GlobalFlags, Command enum — where Config is wired) | ~2 000 |
| Target source files to write | ~5 500 |
| Test files | ~3 000 |
| **Total** | **~24 000** |

Context budget: 24 000 / 200 000 ≈ 12% — within limit.

## Test Strategy

**Unit tests** (`crates/slideforge-config/src/cascade.rs #[cfg(test)]`):

- `test_collect_no_sfconfig()`: deck_dir with no `.sfconfig` at any level; assert empty
  levels list.
- `test_collect_deck_local_only()`: `.sfconfig` only in deck_dir; assert 1 level at
  `SfconfigLevel1`.
- `test_collect_intermediate_and_deck()`: `.sfconfig` in both intermediate and deck dirs;
  assert 2 levels in workspace-root-first order.
- `test_collect_depth_4_warns()`: `.sfconfig` at 4 levels; assert E-CFG-006 warning for
  the 4th; only 3 levels in result.
- `test_parse_sfconfig_workspace_key_warned()`: `.sfconfig` with `[workspace]` section;
  assert E-CFG-006 warning; other keys parsed successfully.
- `test_parse_sfconfig_malformed_toml()`: invalid TOML; assert `MalformedSfconfig` error.
- `test_merge_deck_overrides_root()`: root `brand = "corp"`, deck-local `brand = "client"`;
  merged → "client"; provenance level = `SfconfigLevel1`.
- `test_merge_absent_key_falls_through()`: root `brand = "corp"`, deck `.sfconfig` has no
  `brand` key; merged → "corp"; provenance level = `WorkspaceRoot`.
- `test_merge_list_replace()`: root `formats = ["pptx"]`, deck `formats = ["pdf"]`; merged
  → `["pdf"]` (replace, not append).
- `test_explain_format_text()`: known `ProvenanceEntry`; assert text output contains
  `from:` and `level:` strings.
- `test_explain_format_json()`: serialize `ResolvedConfig`; parse JSON; assert valid.

**Integration tests** (`crates/slideforge-cli/tests/config_explain_integration.rs`):

- `test_config_explain_single_key()`: workspace with deck-local `.sfconfig` overriding
  `brand`; run `config explain brand`; assert output shows "client.pptx" + "deck-local".
- `test_config_explain_workspace_root()`: no `.sfconfig`; run `config explain brand`;
  assert output shows root value + "workspace-root".
- `test_config_explain_all_keys()`: no key arg; assert ALL valid keys appear in output.
- `test_config_explain_unknown_key()`: `config explain nonexistent`; assert E-CFG-006
  with valid key list; exit 4.
- `test_config_explain_json_format()`: `--format json`; parse output as JSON; assert
  each key has `value`, `provenance`, `source` fields.
- `test_config_explain_cli_flag_provenance()`: run with `--template brand.pptx` flag;
  `config explain template`; assert provenance = "cli-flag".
- `test_config_explain_no_project()`: run in temp dir with no `slideforge.toml`;
  assert E-CFG-005; exit 4.
- `test_config_explain_read_only()`: snapshot dir; run explain; assert no changes.

## Dependencies

- **Depends on:** STORY-064 (`ProjectSettings`, `SlideForgeTOML`, `ConfigError`, and
  `merge_member_settings()` infrastructure — the cascade system extends these types
  with provenance tracking and multi-level resolution)
- **Blocks:** (none — STORY-065 is the last workspace story)

## Dependency Anchor Justifications

- SS-17 owns the cascade loading and `config explain` logic in `slideforge-config` per
  ARCH-INDEX Subsystem Registry. SS-18 (CLI) owns the `Command::Config` dispatch.
- STORY-065 depends on STORY-064 because `CascadeLevel` extends `ProjectSettings`
  (defined in STORY-064's `workspace.rs`) with provenance metadata. The cascade logic
  builds on the `merge_member_settings()` foundation but adds the provenance-tracking
  layer and the multi-level `.sfconfig` walk that STORY-064 did not implement.

## Architecture Compliance Rules

1. `slideforge-config` is a **pure configuration-parsing** crate. `cascade.rs` MUST NOT
   perform network I/O, git operations, or shell execution.
2. The cascade walk NEVER traverses above `workspace_root`. Finding `workspace_root` is
   the responsibility of the caller (CLI layer). `collect_sfconfig_levels()` takes
   `workspace_root` as an explicit parameter — it does NOT search for it.
3. `config explain` is strictly read-only. The CLI implementation MUST pass this through
   a test: snapshot directory before/after; assert identical.
4. JSON output uses `serde_json`. Do NOT hand-format JSON strings.
5. `#![forbid(unsafe_code)]` at the crate root.
6. The `config explain` provenance chain covers exactly 5 levels in order of precedence
   (highest to lowest): `cli-flag` > `sfconfig-level-1` (deck directory, i.e., `deck-local`) >
   `sfconfig-level-2` (parent directory) > `workspace-root` > `default`.
   Note: `deck-local` IS `sfconfig-level-1`; they are the same level, not two distinct levels.

**Forbidden dependencies for `slideforge-config`:**
- Must NOT import `slideforge-eval`, `slideforge-syntax`, or any exporter crate.
- Must NOT perform network I/O or git operations.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `toml` | `=0.8` | Parse `.sfconfig` files (already in crate from STORY-064) |
| `serde` | `=1.0` | Deserialize `ProjectSettings` (already present) |
| `serde_json` | `=1.0` | JSON output for `config explain --format json` |
| `thiserror` | `=2.0` | `ConfigError` (already present) |

No new dependencies beyond `serde_json` (not yet in `slideforge-config`).

## File Structure Requirements

```
crates/slideforge-config/
  src/
    cascade.rs          # CascadeLevel, ConfigLevel, collect_sfconfig_levels(),
                        # parse_sfconfig(), merge_cascade(), ResolvedConfig, ProvenanceEntry
    explain_format.rs   # format_text(), format_json()
    lib.rs              # add: pub use cascade::{..}; pub use explain_format::{..};
  tests/
    cascade_integration.rs
  Cargo.toml            # add serde_json = "=1.0"
crates/slideforge-cli/
  src/
    cli.rs              # Command::Config(ConfigArgs), ExplainArgs, ExplainFormat added
    commands/
      config.rs         # run_config_explain()
  tests/
    config_explain_integration.rs
```

## Previous Story Intelligence

STORY-064 defined `ProjectSettings`, `SlideForgeTOML`, `ConfigError`, and
`merge_member_settings()`. Read STORY-064's implementation before writing cascade code
to understand:
- The exact field names in `ProjectSettings` (these become the valid keys for
  `config explain`)
- The merge semantics already implemented in `merge_member_settings()` — the cascade
  merge is the same algorithm applied at multiple levels, not a reimplementation

Key constraint from the DSL decisions (q16-q25-decisions.md): the merge semantics are
NOT configurable. last-wins / replace-list / deep-merge-map is the fixed rule. No
alternate merge strategies.

## Implementation Notes

### `.sfconfig` vs `slideforge.toml` distinction

- `slideforge.toml` is the workspace root config. It may contain `[workspace]`.
- `.sfconfig` is a subdirectory override file. It MUST NOT contain `[workspace]`.
- Both files use the same TOML schema for `ProjectSettings` keys.
- The cascade walk starts from the DECK's directory and walks UP toward workspace root —
  NOT from workspace root walking down.

### Valid configuration keys for `config explain`

The valid key set is derived from `ProjectSettings` struct field names:
- `brand` — path to brand template (.pptx / .docx) or brand.toml
- `output_dir` — output directory (default: `dist/`)
- `formats` — list of output formats (default: all 4)
- `warn_only` — boolean (default: false)
- `offline` — boolean (default: false)
- `lang` — language code (default: "en")

This list expands as `ProjectSettings` gains more fields in future stories. The
`config explain` key list is dynamically derived from the struct, not hardcoded.

### Provenance chain precedence (highest to lowest)

```
6. cli-flag       (e.g., --warn-only flag overrides everything)
5. deck-local     (deck's own directory .sfconfig — SfconfigLevel1)
4. sfconfig-level-2 (intermediate directory .sfconfig — SfconfigLevel2)
3. workspace-root (slideforge.toml [workspace] default settings)
2. default        (built-in defaults, not user-specified)
```

For the first story cut, CLI flags are not tracked in the cascade (that requires
threading GlobalFlags through to the config system). Mark CLI-flag provenance as
a stretch goal if time-boxed.

### JSON output schema

```json
{
  "brand": {
    "value": "client.pptx",
    "provenance": "deck-local",
    "source": "/workspace/decks/client/.sfconfig"
  },
  "output_dir": {
    "value": "dist/",
    "provenance": "default",
    "source": null
  }
}
```

### Error codes for this story

| Code | Meaning | Exit |
|------|---------|------|
| E-CFG-005 | Workspace root not found (no slideforge.toml with [workspace]) | 4 |
| E-CFG-006 | Config file warning: workspace key in .sfconfig, depth-4 file ignored, unknown key, malformed .sfconfig, malformed cascade | 0 (warning) or 4 (error) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No .sfconfig at any level | Workspace root slideforge.toml settings apply directly |
| EC-002 | .sfconfig with [workspace] section | E-CFG-006: workspace section not valid; that key ignored |
| EC-003 | .sfconfig at depth 4 | E-CFG-006 warning; file ignored; cascade stops at depth 3 |
| EC-004 | .sfconfig with unknown key | E-CFG-006 warning (unknown key ignored); build continues |
| EC-005 | .sfconfig overrides brand template path | Brand loaded from .sfconfig-specified path for that deck family |
| EC-006 | config explain with no slideforge.toml | E-CFG-005; exit 4 |
| EC-007 | config explain unknown-key | E-CFG-006 with valid key list; exit 4 |
| EC-008 | config explain --format json | Valid JSON output; exit 0 |
| EC-009 | config explain with cli flag active | Provenance shows cli-flag for that key |
