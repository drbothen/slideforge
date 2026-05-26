---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-058
title: "CLI: --json/--quiet/--verbose flags + NO_COLOR + non-TTY"
epic: EPIC-15
wave: 5
points: 5
priority: P0
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-cli
behavioral_contracts:
  - BC-1.15.001
verification_properties: []
nfr_refs:
  - NFR-021
  - NFR-022
  - NFR-023
  - NFR-024
  - NFR-032
  - NFR-033
depends_on:
  - STORY-055
blocks: []
subsystems:
  - SS-18
target_module: slideforge-cli
---

# STORY-058: CLI: --json/--quiet/--verbose flags + NO_COLOR + non-TTY

## Summary

Complete the global flag behavior for `slideforge-cli`. STORY-055 defined `GlobalFlags`
with the `--json`, `--quiet`, `--verbose`, `--no-color`, `--warn-only`, and `--offline`
fields. This story implements the actual behavior:

- **`--json`**: emit all diagnostics and status output as structured JSON to stderr.
  Normal (non-error) stdout output is suppressed.
- **`--quiet`**: suppress all informational and warning output; only errors are shown.
- **`--verbose`**: enable debug-level tracing (stackable: `-v` = debug, `-vv` = trace).
- **`--no-color`**: disable ANSI color codes in all output. Also respected when the
  `NO_COLOR` env var is set to any non-empty string (per the NO_COLOR spec at no-color.org).
- **Non-TTY detection**: when stdout/stderr is redirected (not a terminal), automatically
  disable color output and progress animations.

The primary BC traced by this story is BC-1.15.001 (spans + hints + rendering mode).
Specifically postconditions 4 and 5 (color in TTY, `--no-color` flag) are the behavioral
anchor; the other flag behaviors (quiet, verbose, json) are observability concerns
(NFR-032, NFR-033).

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-1.15.001 | All errors carry file:line:col span and correction hint | Postcondition 4 (TTY-aware rendering), Postcondition 5 (--no-color), Edge Case EC-004 |

## Acceptance Criteria

- [ ] **AC-001** — When `stderr` is a TTY and `--no-color` is not set and `NO_COLOR` env var
  is unset, diagnostics are rendered with ANSI color codes via `miette::GraphicalReportHandler`.
  (traces to BC-1.15.001 postcondition 4)

- [ ] **AC-002** — When `stderr` is NOT a TTY (piped/redirected), diagnostics are rendered as
  plain text without ANSI escape sequences, regardless of `--no-color`.
  (traces to BC-1.15.001 edge case EC-004)

- [ ] **AC-003** — `--no-color` (CLI flag) disables ANSI color codes unconditionally, even when
  stderr is a TTY.
  (traces to BC-1.15.001 postcondition 5)

- [ ] **AC-004** — `NO_COLOR=1` (or any non-empty value) in the environment disables ANSI color
  codes, equivalent to `--no-color`.
  (traces to BC-1.15.001 postcondition 5 — no-color.org spec)

- [ ] **AC-005** — `--json` flag outputs all diagnostics as a single JSON object to stderr:
  ```json
  {
    "diagnostics": [...],
    "total": N,
    "has_fatal": bool,
    "exit_code": N
  }
  ```
  The JSON schema matches `DiagnosticSink::to_json()` from STORY-010.
  (traces to NFR-033 — structured JSON output complete and valid)

- [ ] **AC-006** — When `--json` is active, no human-readable diagnostic text is written to
  stderr. The file manifest and next-step blocks from `slideforge init` are suppressed from
  stdout (JSON output only).
  (traces to NFR-033 — JSON mode is mutually exclusive with human-readable output)

- [ ] **AC-007** — `--quiet` suppresses all `tracing::info!` and `tracing::warn!` output.
  Only `tracing::error!` (and the final diagnostic summary) are emitted.
  (traces to BC-1.15.001 postcondition 4 — "plain text in CI/pipe contexts" implies
  non-verbose output appropriate for automated environments)

- [ ] **AC-008** — `-v` (one occurrence of `--verbose`) enables `tracing::debug!` level output.
  `-vv` (two occurrences) enables `tracing::trace!` level output.
  (traces to NFR-032 — tracing instrumentation is observable at appropriate verbosity)

- [ ] **AC-009** — The `--json` flag is mutually exclusive with `--quiet` (both suppress
  human-readable output, but combining them makes no sense). If both are passed, `--json`
  takes precedence and a lint warning is printed: "Note: --quiet ignored when --json is
  active."
  (traces to BC-1.15.001 postcondition 4 — correct rendering mode selection)

- [ ] **AC-010** — Color-detection logic is encapsulated in `ColorMode::detect(global: &GlobalFlags)
  -> ColorMode` returning `ColorMode::Color`, `ColorMode::Plain`, or `ColorMode::Json`.
  All commands use this instead of inline TTY detection.
  (traces to BC-1.15.001 postcondition 4 — consistent color mode across all subcommands)

- [ ] **AC-011** — The JSON diagnostic schema is tested by parsing the output with
  `serde_json::from_str()` and asserting all required keys are present.
  (traces to NFR-033 — JSON parses without error; all fields present)

- [ ] **AC-012** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, `clippy::pedantic`
  clean.
  (traces to NFR-021, NFR-022, NFR-024)

- [ ] **AC-013** — All public items have rustdoc. `cargo doc --no-deps` produces 0 warnings.
  (traces to NFR-023)

## Tasks

1. Implement `ColorMode` enum in `src/color.rs`:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum ColorMode {
       /// ANSI colors enabled (TTY detected, no --no-color, no NO_COLOR env)
       Color,
       /// Plain text, no ANSI codes (non-TTY or --no-color or NO_COLOR env)
       Plain,
       /// JSON output to stderr; no human-readable diagnostics
       Json,
   }

   impl ColorMode {
       pub fn detect(global: &GlobalFlags) -> Self {
           if global.json {
               return Self::Json;
           }
           let no_color_env = std::env::var("NO_COLOR").map(|v| !v.is_empty()).unwrap_or(false);
           let is_tty = std::io::stderr().is_terminal();
           if global.no_color || no_color_env || !is_tty {
               Self::Plain
           } else {
               Self::Color
           }
       }
   }
   ```
2. Update `src/tracing_setup.rs` to honor `global.verbose` and `global.quiet`:
   ```rust
   pub fn init_tracing(global: &GlobalFlags) {
       let level = match (global.quiet, global.verbose) {
           (true, _) => Level::ERROR,
           (false, 0) => Level::INFO,
           (false, 1) => Level::DEBUG,
           (false, _) => Level::TRACE,
       };
       let _ = tracing_subscriber::fmt()
           .with_max_level(level)
           .with_ansi(ColorMode::detect(global) == ColorMode::Color)
           .try_init();
   }
   ```
3. Update `DiagnosticRenderer` usage in `src/commands/build.rs` (and future commands):
   - Replace inline `use_color` bool with `ColorMode::detect(global)`.
   - In `ColorMode::Json` mode: call `sink.to_json()` and write to stderr as JSON.
   - In `ColorMode::Plain`/`Color` mode: call `renderer.render_all()` as before.
4. Implement `JsonDiagnosticOutput` in `src/json_output.rs`:
   ```rust
   pub struct JsonDiagnosticOutput {
       pub diagnostics: Vec<JsonDiagnosticEntry>,
       pub total: usize,
       pub has_fatal: bool,
       pub exit_code: u8,
   }
   ```
   Write `emit_json_diagnostics(sink: &DiagnosticSink, exit_code: u8)` that
   serializes and writes to stderr.
5. Update `src/commands/init.rs`: in `--json` mode, suppress manifest/next-step print;
   instead emit JSON with `{"command": "init", "created_files": [...], "exit_code": 0}`.
6. Validate `--json` + `--quiet` combination: detect in `commands::dispatch()` and print
   the lint note before running the command.
7. Write unit and integration tests for all ACs.

## File List

- `crates/slideforge-cli/src/color.rs` — `ColorMode`, `ColorMode::detect()`
- `crates/slideforge-cli/src/json_output.rs` — `JsonDiagnosticOutput`,
  `JsonDiagnosticEntry`, `emit_json_diagnostics()`
- `crates/slideforge-cli/src/tracing_setup.rs` — updated: honor `quiet` and `verbose`
- `crates/slideforge-cli/src/commands/build.rs` — updated: use `ColorMode::detect()`
- `crates/slideforge-cli/src/commands/init.rs` — updated: JSON mode suppresses manifest
- `crates/slideforge-cli/tests/flags_integration.rs` — integration tests

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~4 500 |
| BC-1.15.001 | ~1 500 |
| STORY-055 (GlobalFlags, DiagnosticRenderer usage) | ~2 000 |
| STORY-010 (DiagnosticSink::to_json() schema) | ~1 500 |
| Target source files to write | ~3 500 |
| Test files | ~2 500 |
| **Total** | **~15 500** |

Context budget: 15 500 / 200 000 ≈ 7.75% — within limit.

## Test Strategy

**Unit tests** (`src/color.rs #[cfg(test)]`):

- `test_color_mode_json_flag()`: construct `GlobalFlags { json: true, .. }`;
  assert `ColorMode::detect() == ColorMode::Json`.
- `test_color_mode_no_color_flag()`: `global.no_color = true`; assert `ColorMode::Plain`.
- `test_color_mode_no_color_env()`: `env::set_var("NO_COLOR", "1")`; assert `ColorMode::Plain`.
  Clear env var after test.
- `test_color_mode_plain_wins_over_color_when_no_tty()`: simulate non-TTY by testing on
  a non-TTY context; `global.no_color = false`; `global.json = false`; assert
  `ColorMode::Plain` (because non-TTY).
- `test_json_and_quiet_produces_note()`: pass `--json --quiet`; assert warning note
  in stderr output (integration test variant).

**Unit tests** (`src/json_output.rs #[cfg(test)]`):

- `test_json_diagnostic_output_parseable()`: construct `JsonDiagnosticOutput` with 2
  entries; serialize to string; parse with `serde_json::from_str()`; assert no error.
- `test_json_diagnostic_all_fields()`: assert output JSON has keys: `diagnostics`,
  `total`, `has_fatal`, `exit_code`.
- `test_json_diagnostic_entry_fields()`: assert each entry has: `error_code`, `severity`,
  `file`, `line`, `col`, `message`, `hint`.

**Integration tests** (`tests/flags_integration.rs`):

- `test_json_flag_stderr_is_json()`: run `build` with `--json` on error fixture;
  capture stderr; parse as JSON; assert all required keys present.
- `test_json_flag_no_ansi()`: assert JSON output contains no ANSI escape sequences.
- `test_quiet_suppresses_info()`: run `build` with `--quiet` on success fixture; assert
  no `INFO` lines in stderr.
- `test_verbose_enables_debug()`: run `build` with `-v` on success fixture; assert at
  least one `DEBUG` or `TRACE` line in stderr.
- `test_no_color_no_ansi()`: run `build` with `--no-color` on error fixture; capture
  stderr; assert no `\x1b[` sequences.
- `test_no_color_env_var()`: set `NO_COLOR=1`; run `build` on error fixture; assert no
  ANSI codes.
- `test_piped_output_no_ansi()`: redirect stderr to a pipe (not TTY); assert no ANSI codes
  without any flags.

## Dependencies

- **Depends on:** STORY-055 (provides `GlobalFlags` struct with all flag fields; provides
  `DiagnosticRenderer` usage patterns that this story updates)
- **Blocks:** (none — this is a refinement/completion story for the flags defined in STORY-055)

## Dependency Anchor Justifications

- SS-18 owns this story's scope because SS-18 is the CLI Orchestrator subsystem and all
  global output-mode behavior is a CLI lifecycle concern per ARCH-INDEX.
- STORY-058 depends on STORY-055 because `GlobalFlags` is defined in STORY-055 and this
  story implements the behavioral semantics of those flags. The `ColorMode` type introduced
  here replaces the inline `use_color: bool` in `run_build()` from STORY-055.

## Architecture Compliance Rules

1. `ColorMode::detect()` is the SINGLE source of truth for color/JSON/plain mode. No
   code anywhere in `slideforge-cli` should independently check `NO_COLOR` env var or TTY
   state — all must call `ColorMode::detect(global)`.
2. `NO_COLOR` env var detection uses `std::env::var("NO_COLOR")` — do NOT use any
   third-party env-var library. The spec (no-color.org) requires "any non-empty value"
   to disable color.
3. `--verbose` uses `clap`'s `action = ArgAction::Count` to allow stacking (`-v`, `-vv`).
   The `u8` count maps to `Level::DEBUG` (1) and `Level::TRACE` (2+). Do NOT invent a
   separate `--debug` or `--trace` flag.
4. `tracing_subscriber` must be initialized exactly once. `try_init()` silently succeeds
   even if called multiple times — acceptable in tests. In production code, wrap in
   `OnceLock` if needed.
5. The `--json` and human-readable modes are **mutually exclusive** at the output level.
   The `ColorMode::Json` variant signals this to all command handlers.

**Forbidden dependencies:**
- Do NOT add an `atty` crate — use `std::io::IsTerminal` (stable since Rust 1.70).
- Do NOT add a `colored` or `termcolor` crate — miette handles color; use
  `GraphicalReportHandler` for color and plain text.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `miette` | `=7.2` | `GraphicalReportHandler` for color/plain rendering |
| `tracing-subscriber` | `=0.3` | Level filtering from `--verbose`/`--quiet` |
| `serde` | `=1.0` | `JsonDiagnosticOutput` serialization |
| `serde_json` | `=1.0` | JSON serialization and test parsing |

## File Structure Requirements

```
crates/slideforge-cli/
  src/
    color.rs                # ColorMode enum, ColorMode::detect()
    json_output.rs          # JsonDiagnosticOutput, emit_json_diagnostics()
    tracing_setup.rs        # updated: verbose/quiet level, color mode
    commands/
      build.rs              # updated: ColorMode::detect() replaces inline use_color
      init.rs               # updated: JSON mode for manifest output
  tests/
    flags_integration.rs    # integration tests for all global flags
```

## Previous Story Intelligence

From STORY-055: `GlobalFlags` has `verbose: u8` using `action = ArgAction::Count`.
Confirm the clap derive uses:
```rust
#[arg(long, short = 'v', global = true, action = ArgAction::Count)]
pub verbose: u8,
```
This is already defined in STORY-055. Do NOT redefine — only implement the tracing setup
that reads the `u8` count.

Key from BC-1.15.001 edge case EC-004: "CLI invoked with redirected stdout (no TTY)" →
plain text without ANSI. The story explicitly says "stdout" but the diagnostic output goes
to STDERR. Apply the non-TTY detection to `std::io::stderr().is_terminal()`, not stdout.
This is the correct implementation per miette's rendering behavior.

## Implementation Notes

### NO_COLOR env var detection

Per the no-color.org specification:
- `NO_COLOR` set to any non-empty string → disable color
- `NO_COLOR` unset or empty → no effect
- The `--color` flag is NOT part of the slideforge CLI (no force-enable override)

```rust
let no_color_env = std::env::var_os("NO_COLOR")
    .map(|v| !v.is_empty())
    .unwrap_or(false);
```

### JSON output schema for diagnostics

The JSON output from `--json` mode must be parseable by `serde_json::from_str()`:

```json
{
  "diagnostics": [
    {
      "error_code": "E-PAR-001",
      "severity": "fatal",
      "file": "deck.sf",
      "line": 5,
      "col": 3,
      "message": "Unexpected tab character at deck.sf:5:3",
      "hint": "Use spaces for indentation"
    }
  ],
  "total": 1,
  "has_fatal": true,
  "exit_code": 1
}
```

This schema is fully defined by `DiagnosticSink::to_json()` from STORY-010. The CLI
only adds `exit_code` to the top-level object.

### `--quiet` + `--json` interaction

When both are passed, `--json` takes precedence. The note "Note: --quiet ignored when
--json is active" is printed to stderr BEFORE the JSON output. Since it is not JSON
itself, it should go to stdout (or be omitted from tests that capture stderr). Simpler:
omit the note if it complicates parsing; just silently let `--json` win.

Design decision: omit the note. `--json` mode is always silent-except-for-JSON. The
`--quiet` flag is a no-op when `--json` is active. Document this in the CLI help text
via `clap`'s `conflicts_with` or simply in the argument documentation string.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `NO_COLOR=` (empty string) | No effect — empty value does not disable color per no-color.org |
| EC-002 | `NO_COLOR=1 --no-color` together | Redundant; both disable color; no error |
| EC-003 | `--json` + diagnostic with multi-line span | JSON `message` field contains the full message string; no embedded ANSI; `line`/`col` reflect the span start |
| EC-004 | `-vvv` (3 verbose flags) | Treated same as `-vv` (trace level); no error; no special behavior beyond trace |
| EC-005 | `--quiet` with 0 errors | Exit 0; nothing printed to stderr (success is silent in quiet mode) |
| EC-006 | `--json` on a successful build | JSON with `{"diagnostics": [], "total": 0, "has_fatal": false, "exit_code": 0}` |
