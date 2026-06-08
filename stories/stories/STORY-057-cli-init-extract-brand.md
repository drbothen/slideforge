---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-057
title: "CLI: init scaffolding + extract-brand command"
epic: EPIC-15
wave: 5
points: 5
priority: P0
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-cli
behavioral_contracts:
  - BC-5.06.001
  - BC-5.06.002
verification_properties: []
nfr_refs:
  - NFR-021
  - NFR-022
  - NFR-023
  - NFR-024
depends_on:
  - STORY-055
  - STORY-024
blocks: []
subsystems:
  - SS-18
target_module: slideforge-cli
---

# STORY-057: CLI: init scaffolding + extract-brand command

## Summary

Implement two subcommands in `slideforge-cli`:

1. **`slideforge init [NAME]`** — scaffolds a complete, immediately buildable slideforge
   project in the target directory. Creates `deck.sf`, `brand.toml`, `slideforge.toml`,
   `assets/`, and `.gitignore`. The generated `deck.sf` must compile successfully with
   `slideforge build deck.sf` with exit 0 immediately after init. Rejects if
   `slideforge.toml` already exists (without `--force`).

2. **`slideforge extract-brand <template.pptx>`** — delegates to the brand extraction
   logic from STORY-024 (`slideforge-brand::extract_brand_toml()`) and writes the
   resulting `brand.toml` to the specified output path (default: `brand.toml` in current
   directory).

Both commands follow the atomicity rule from DI-009: if any file write fails mid-operation,
partially created files are cleaned up (rollback).

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-5.06.001 | slideforge init scaffolds a buildable starter project | All 6 postconditions; Invariants 1–4 |
| BC-5.06.002 | slideforge init rejects existing project without --force (E-CFG-008) | All 4 postconditions; Invariants 1–3 |

## Acceptance Criteria

- [ ] **AC-001** — `slideforge init` in an empty directory creates exactly these artifacts:
  `deck.sf`, `brand.toml`, `slideforge.toml`, `assets/` (empty dir), `.gitignore`.
  Exit code is 0.
  (traces to BC-5.06.001 postcondition 1)

- [ ] **AC-002** — `slideforge build deck.sf` (immediately after `slideforge init`) exits 0
  and produces `dist/deck.pptx`.
  (traces to BC-5.06.001 postcondition 2)

- [ ] **AC-003** — A file manifest listing all created files with their paths is printed to
  stdout after successful init.
  (traces to BC-5.06.001 postcondition 3)

- [ ] **AC-004** — A "get started" next-step block is printed showing how to `build`, `watch`,
  and `config explain`.
  (traces to BC-5.06.001 postcondition 4)

- [ ] **AC-005** — `slideforge init my-deck` creates `my-deck/` directory (if it does not
  exist) and places all scaffold files inside it.
  (traces to BC-5.06.001 postcondition 5)

- [ ] **AC-006** — The generated `deck.sf` includes `slideforge_version "1"`, `lang "en-US"`,
  and at least one slide with all required fields populated and no missing alt text.
  (traces to BC-5.06.001 invariant 1; DI-003 lang required)

- [ ] **AC-007** — The generated `brand.toml` has all 12 OOXML color slots populated with
  safe default values (no missing required keys).
  (traces to BC-5.06.001 postcondition 1 — brand.toml with all 12 slots)

- [ ] **AC-008** — If disk fills mid-scaffold (after 2 of 4 files written), all partially
  written files are deleted and E-EXP-007 is reported. Exit code is 3.
  (traces to BC-5.06.001 invariant 3 — atomic all-or-nothing)

- [ ] **AC-009** — `slideforge init` when `slideforge.toml` exists (without `--force`)
  emits E-CFG-008 with a message including the target directory path and hint to use
  `--force`. Exit code is 4. Zero files are created, overwritten, or deleted.
  (traces to BC-5.06.002 postconditions 1–3)

- [ ] **AC-010** — `slideforge init --force` when `slideforge.toml` exists overwrites only
  the four scaffold files (`deck.sf`, `brand.toml`, `slideforge.toml`, `.gitignore`).
  User data files (e.g., `data.json`) are not deleted. Exit code is 0.
  (traces to BC-5.06.002 invariant 2)

- [ ] **AC-011** — `slideforge extract-brand template.pptx` delegates to
  `slideforge_brand::extract_brand_toml(template_path)` and writes `brand.toml` to the
  current directory (or `--output` path). Exit code 0 on success.
  (traces to BC-5.06.001 postcondition 1 — extract-brand is a direct path to getting a
  brand.toml; BC-2.01.003 is implemented in STORY-024 which this story delegates to)

- [ ] **AC-012** — `slideforge extract-brand template.pptx` when `brand.toml` already exists
  requires `--force` or exits with E-CFG-009 and exit code 4.
  (traces to BC-5.06.002 invariant 1 — no filesystem mutation without explicit permission)

- [ ] **AC-013** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, `clippy::pedantic`
  clean.
  (traces to NFR-021, NFR-022, NFR-024)

- [ ] **AC-014** — All public items have rustdoc. `cargo doc --no-deps` produces 0 warnings.
  (traces to NFR-023)

## Tasks

1. Define `InitArgs` in `src/cli.rs`:
   ```rust
   #[derive(Args)]
   pub struct InitArgs {
       /// Optional project name (creates a subdirectory)
       pub name: Option<String>,
       /// Overwrite existing scaffold files
       #[arg(long)]
       pub force: bool,
   }
   ```
2. Define `ExtractBrandArgs` in `src/cli.rs`:
   ```rust
   #[derive(Args)]
   pub struct ExtractBrandArgs {
       /// Path to .pptx template
       pub template: PathBuf,
       /// Output path for brand.toml (default: brand.toml in current dir)
       #[arg(long, short = 'o', default_value = "brand.toml")]
       pub output: PathBuf,
       /// Overwrite existing brand.toml
       #[arg(long)]
       pub force: bool,
   }
   ```
3. Implement `run_init(args: &InitArgs, global: &GlobalFlags) -> ExitCode` in
   `src/commands/init.rs`:
   - Resolve target dir: current dir if `args.name` is None; `PathBuf::from(name)` if Some.
   - Check for existing `slideforge.toml`: if exists and `!args.force` → emit E-CFG-008 and
     return exit 4.
   - Write all scaffold files via `ScaffoldWriter::write_all()` with rollback on error.
   - Print file manifest + next-step block to stdout.
   - Return exit 0.
4. Implement `ScaffoldWriter`:
   ```rust
   struct ScaffoldWriter {
       target_dir: PathBuf,
       written: Vec<PathBuf>, // for rollback
   }
   impl ScaffoldWriter {
       fn write_all(&mut self, force: bool) -> Result<Vec<PathBuf>, ScaffoldError>;
       fn rollback(&self);   // delete all `written` files
       fn write_deck_sf(&mut self) -> Result<(), ScaffoldError>;
       fn write_brand_toml(&mut self) -> Result<(), ScaffoldError>;
       fn write_slideforge_toml(&mut self) -> Result<(), ScaffoldError>;
       fn write_gitignore(&mut self) -> Result<(), ScaffoldError>;
       fn create_assets_dir(&mut self) -> Result<(), ScaffoldError>;
   }
   ```
5. Define scaffold content as `const &str` strings. The generated `deck.sf` must be
   syntactically valid per BC-5.06.001 invariant 1:
   ```
   slideforge_version "1"
   lang "en-US"

   slide title:
     title "Welcome to slideforge"
     subtitle "Built with slideforge — the Rust presentation compiler"
     alt "Title slide: Welcome to slideforge"

   slide bullets:
     title "Getting Started"
     items:
       - "Run `slideforge build deck.sf` to compile"
       - "Run `slideforge watch deck.sf` for live preview"
       - "Edit `brand.toml` to apply your brand"
     alt "Bullets slide: Getting Started steps"

   slide end:
     title "Thank You"
     alt "End slide: Thank You"
   ```
6. Define `brand.toml` template content with all 12 OOXML color slots as comments
   showing their purpose and safe RGB defaults.
7. Implement rollback: iterate `self.written` in reverse; `fs::remove_file()` for files,
   `fs::remove_dir()` for `assets/`. Swallow errors in rollback (best-effort).
8. Implement `run_extract_brand(args: &ExtractBrandArgs, global: &GlobalFlags) -> ExitCode`
   in `src/commands/extract_brand.rs`:
   - Check target output path: if exists and `!args.force` → E-CFG-009; exit 4.
   - Call `slideforge_brand::extract_brand_toml(args.template.clone())?`
   - Write result to `args.output` atomically (temp file + rename).
   - Print success message. Return exit 0.
9. Write tests for all ACs.

## File List

- `crates/slideforge-cli/src/commands/init.rs` — `run_init()`, `ScaffoldWriter`, scaffold
  content constants
- `crates/slideforge-cli/src/commands/extract_brand.rs` — `run_extract_brand()`
- `crates/slideforge-cli/src/cli.rs` — add `InitArgs`, `ExtractBrandArgs` (extends
  existing)
- `crates/slideforge-cli/src/scaffold_content.rs` — `DECK_SF_TEMPLATE`,
  `BRAND_TOML_TEMPLATE`, `SLIDEFORGE_TOML_TEMPLATE`, `GITIGNORE_TEMPLATE` constants
- `crates/slideforge-cli/tests/init_integration.rs` — integration tests

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~5 000 |
| BC-5.06.001, BC-5.06.002 | ~3 500 |
| STORY-055 (CLI struct — InitArgs, ExtractBrandArgs hooks) | ~1 500 |
| STORY-024 (extract_brand_toml() API signature) | ~1 500 |
| Target source files to write | ~4 000 |
| Test files | ~2 500 |
| **Total** | **~18 000** |

Context budget: 18 000 / 200 000 ≈ 9.0% — within limit.

## Test Strategy

**Unit tests** (`src/commands/init.rs #[cfg(test)]`):

- `test_scaffold_writer_creates_all_files()`: run `write_all()` in a temp dir; assert all
  5 artifacts (4 files + assets/) exist.
- `test_scaffold_writer_rollback()`: mock disk full on 3rd write; call `write_all()`; assert
  rollback removes the 2 successfully written files.
- `test_deck_sf_template_parses()`: parse `DECK_SF_TEMPLATE` via `slideforge_syntax::parse()`
  in a unit test; assert zero errors.
- `test_brand_toml_has_all_12_slots()`: parse `BRAND_TOML_TEMPLATE` as TOML; assert all 12
  expected color keys are present.
- `test_ecfg008_without_force()`: create a temp dir with `slideforge.toml`; run `run_init()`
  without `--force`; assert exit code 4 and zero new files.
- `test_force_overwrites_scaffold_only()`: create temp dir with `slideforge.toml` + user
  `data.json`; run `--force`; assert `data.json` still exists.

**Integration tests** (`tests/init_integration.rs`):

- `test_init_empty_dir()`: run `slideforge init` in temp dir; assert all 5 artifacts
  created; assert manifest printed to stdout.
- `test_init_named()`: run `slideforge init my-deck`; assert `my-deck/` created with all
  artifacts.
- `test_init_then_build()`: init in temp dir; then invoke `run_build()` on the generated
  `deck.sf`; assert exit 0 and `dist/deck.pptx` created.
- `test_extract_brand_produces_valid_toml()`: run `extract-brand` on a test `.pptx` fixture;
  assert the output `brand.toml` is valid TOML with at least 1 color key.
- `test_extract_brand_refuses_without_force()`: existing `brand.toml`; run without `--force`;
  assert exit 4 and no overwrite.

## Dependencies

- **Depends on:** STORY-055 (provides `Cli` struct, `GlobalFlags`, and command dispatch into
  which `InitArgs` and `ExtractBrandArgs` are added)
- **Depends on:** STORY-024 (`slideforge_brand::extract_brand_toml()` — the brand extraction
  function that `extract-brand` delegates to; the CLI only wires the I/O)
- **Blocks:** (none — both commands are leaf features in the CLI graph)

## Dependency Anchor Justifications

- SS-18 owns this story's scope because SS-18 is the CLI Orchestrator subsystem owning
  `slideforge-cli` per ARCH-INDEX Subsystem Registry. Project scaffolding is user-facing
  lifecycle I/O owned by the CLI.
- STORY-057 depends on STORY-055 because `InitArgs` and `ExtractBrandArgs` are arms of
  `Command` defined in the `Cli` struct in STORY-055, and `run_init()`/`run_extract_brand()`
  must be added to the dispatch table in `commands/mod.rs`.
- STORY-057 depends on STORY-024 because `run_extract_brand()` calls
  `slideforge_brand::extract_brand_toml()` — the STORY-057 CLI wrapper performs no
  brand-extraction logic itself; it only handles I/O wrapping.

## Architecture Compliance Rules

1. `slideforge-cli` is an **effectful shell** — init writes files, extract-brand reads
   binary PPTX and writes TOML. No pure-core logic belongs here.
2. The `ScaffoldWriter` uses a **two-phase commit**: all writes are tracked in `written`;
   on error, rollback by deleting them. This implements DI-009 atomicity.
3. Brand extraction logic MUST NOT be re-implemented here — call
   `slideforge_brand::extract_brand_toml()`. The CLI wrapper only adds the file I/O
   wrapping (source path, output path, force flag).
4. Scaffold content (deck.sf template, brand.toml template, etc.) are `const &str` values
   in `scaffold_content.rs`. They MUST NOT be generated dynamically at runtime; static
   template strings are sufficient and simpler to test.
5. The generated `deck.sf` template MUST be kept in sync with the parser's current syntax.
   If any syntax-breaking change occurs in EPIC-02, the template in `scaffold_content.rs`
   must be updated in the same story/fix.

**Forbidden dependencies for `slideforge-cli`:**
- `init.rs` must NOT import `chumsky` or `slideforge-syntax` directly.
- `extract_brand.rs` must NOT re-implement OOXML parsing — delegate to `slideforge-brand`.
- No dynamic heap allocations for scaffold template content (const strings preferred).

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `clap` | `{workspace = true}` (=4.6.1) | `InitArgs`, `ExtractBrandArgs` derive. Centralized per ADR-022. |
| `toml` | `{workspace = true}` (=1.1.2) | Validate generated `brand.toml` template in tests. Two-major bump from 0.8; centralized per ADR-022. See note below. |
| `thiserror` | `{workspace = true}` (=2.0.18) | `ScaffoldError` error type. Centralized per ADR-022. |

**toml 1.x migration note:** `toml =1.1.2` is a two-major-version bump from `toml =0.8`.
The primary behavior change affecting this story: when generating `brand.toml` content via
struct serialization (preferred for determinism), use `toml::to_string_pretty()` from
`toml =1.1.2`. If using `toml::Value` deserialization in tests, the API is compatible but
flatten (`#[serde(flatten)]`) layouts must be test-verified — serialize → deserialize round-trip
in `test_brand_toml_has_all_12_slots()` to catch any flatten edge cases introduced by the
toml 1.x internal representation changes.

No new production dependencies are needed for this story beyond what STORY-055 already
added to `Cargo.toml`.

## File Structure Requirements

```
crates/slideforge-cli/
  src/
    commands/
      init.rs               # run_init(), ScaffoldWriter
      extract_brand.rs      # run_extract_brand()
    scaffold_content.rs     # DECK_SF_TEMPLATE, BRAND_TOML_TEMPLATE, etc. (const strings)
  tests/
    init_integration.rs     # integration tests
```

## Previous Story Intelligence

From STORY-055: the `Command` enum must have `Init(InitArgs)` and `ExtractBrand(ExtractBrandArgs)`
arms added. These were stubbed in STORY-055's `commands/mod.rs` with `todo!()`. This story
replaces the stubs with real implementations.

Brand.toml template note: the brand synthesizer (STORY-023) requires all 12 OOXML color
slots to be non-empty strings. The `brand.toml` template must include ALL of:
`dk1`, `lt1`, `dk2`, `lt2`, `accent1`, `accent2`, `accent3`, `accent4`, `accent5`,
`accent6`, `hlink`, `folHlink`. Using hex values like `"#1F2937"` for dark colors and
`"#FFFFFF"` for light colors as safe defaults avoids any synthesis-failure on first build.

## Implementation Notes

### Scaffold content — deck.sf template

The generated `deck.sf` must satisfy:
- `slideforge_version "1"` at the top (BC-1.13.001)
- `lang "en-US"` (DI-003)
- At least 1 slide with all required fields (BC-3.01.001)
- `alt "..."` on every visual element (DI-001)
- No `@data` directives (would require external files, breaking the "immediately buildable"
  guarantee)

### Scaffold content — brand.toml template

```toml
# slideforge brand configuration
# Generated by `slideforge init`. Edit these values to match your brand.

[colors]
dk1   = "#1F2937"  # Dark 1: primary text / dark background
lt1   = "#FFFFFF"  # Light 1: primary background / light text
dk2   = "#374151"  # Dark 2: secondary text
lt2   = "#F9FAFB"  # Light 2: secondary background
accent1 = "#3B82F6"  # Accent 1: primary brand color (blue)
accent2 = "#10B981"  # Accent 2: secondary brand color (green)
accent3 = "#F59E0B"  # Accent 3: tertiary brand color (amber)
accent4 = "#EF4444"  # Accent 4: error / danger (red)
accent5 = "#8B5CF6"  # Accent 5: alternate brand color (violet)
accent6 = "#06B6D4"  # Accent 6: alternate brand color (cyan)
hlink   = "#2563EB"  # Hyperlink color
folHlink = "#7C3AED"  # Followed hyperlink color

[fonts]
heading = "Calibri"
body    = "Calibri"
```

### File manifest output format

```
Created files in /path/to/project:
  deck.sf
  brand.toml
  slideforge.toml
  assets/
  .gitignore

Get started:
  slideforge build deck.sf         # compile to dist/
  slideforge watch deck.sf         # live preview at http://localhost:7070
  slideforge config explain brand   # show brand config provenance
```

### E-CFG-008 and E-CFG-009 error codes

These error codes are NEW — add them to the error taxonomy in
`.factory/specs/prd-supplements/error-taxonomy.md` if not already present:
- `E-CFG-008` — slideforge.toml already exists (init without --force)
- `E-CFG-009` — brand.toml already exists (extract-brand without --force)

Both should carry exit code 4 (EXIT_CONFIG_ERROR).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slideforge init` in non-writable directory | E-EXP-007; exit 3; no partial files |
| EC-002 | `slideforge init my-deck` where `my-deck/` already exists | Scaffold inside existing dir; error only if `slideforge.toml` exists inside |
| EC-003 | `slideforge init --force` with existing user files (not scaffold) | User files preserved; only scaffold files overwritten |
| EC-004 | `slideforge init` in subdir of existing slideforge project | Allowed — only checks the target dir itself, not parent dirs |
| EC-005 | Disk full on 3rd of 5 scaffold writes | Rollback: deletes the 2 already-written files; reports E-EXP-007; exit 3 |
| EC-006 | `slideforge extract-brand` with non-existent template.pptx | E-EXP-006 (file not found); exit 3 |
| EC-007 | `slideforge extract-brand` with a corrupt .pptx | E-BRD-001 (malformed template); exit 3; no partial brand.toml |
