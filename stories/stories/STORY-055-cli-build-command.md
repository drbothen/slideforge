---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-055
title: "CLI: build command + miette error rendering"
epic: EPIC-15
wave: 5
points: 8
priority: P0
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-cli
behavioral_contracts:
  - BC-1.15.001
  - BC-1.15.002
  - BC-1.15.003
verification_properties: []
nfr_refs:
  - NFR-001
  - NFR-005
  - NFR-021
  - NFR-022
  - NFR-023
  - NFR-024
  - NFR-032
  - NFR-033
depends_on:
  - STORY-010
  - STORY-049
  - STORY-050
blocks:
  - STORY-056
  - STORY-057
  - STORY-058
  - STORY-059
subsystems:
  - SS-18
target_module: slideforge-cli
---

# STORY-055: CLI: build command + miette error rendering

## Summary

Implement the `slideforge build` subcommand in `slideforge-cli` — the primary user-facing
entry point for the full compilation pipeline. This story wires the end-to-end six-stage
pipeline (Parse → Evaluate → Brand → Validate → Layout → Export) through the CLI binary,
renders all diagnostics via miette with colored source pointers, enforces the three-tier
exit code model, and handles output format selection.

Key responsibilities:

1. **Subcommand dispatch** via `clap` 4.5 derive macros. The `Build` subcommand accepts
   a positional `source` argument (path to `.sf` file), `--format` flag, `--output-dir`
   flag, `--warn-only` flag, `--offline` flag, `--variant` flag, and global flags
   (`--json`, `--quiet`, `--verbose`, `--no-color`) inherited from the root CLI struct.
2. **Pipeline orchestration**: invoke `slideforge::compile(source, options)` from the root
   crate (assembled plugin registry), collect a `DiagnosticSink`, and pipe errors through
   `DiagnosticRenderer`.
3. **Exit code enforcement** per BC-1.15.003: parse errors → exit 1; validation errors
   (strict) → exit 2; export errors → exit 3; success → exit 0.
4. **miette rendering**: colored output when stdout is a TTY; plain text in CI/pipe; JSON
   when `--json` is given.
5. **Tracing instrumentation**: emit spans for all 6 pipeline stages per NFR-032.
6. **Format selection**: `--format pptx,docx,pdf,html` selects output formats; default is
   all four formats.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-1.15.001 | All errors carry file:line:col span and correction hint | Postconditions 1–5; Invariants 1–3 |
| BC-1.15.002 | All errors accumulated in single pass — no fail-on-first | Postconditions 1–4; Invariants 1–3 |
| BC-1.15.003 | Parse errors always fatal; validation errors fatal in strict mode only | Postconditions 1–6; Invariants 1–4 |

## Acceptance Criteria

- [ ] **AC-001** — `slideforge build deck.sf` drives the full compile pipeline for `deck.sf`
  and produces output in `dist/` (or `--output-dir` target). On success, exit code is 0.
  (traces to BC-1.15.003 postcondition 5 — lint-only exit 0)

- [ ] **AC-002** — A source with one E-PAR-* error exits with code 1. No output files are
  written to disk. The error is rendered with file:line:col span via miette.
  (traces to BC-1.15.003 postcondition 1; BC-1.15.001 postcondition 1)

- [ ] **AC-003** — A source with one E-EVL-* error in strict mode exits with code 2. No
  output files are written.
  (traces to BC-1.15.003 postcondition 2)

- [ ] **AC-004** — A source with one E-EVL-* error with `--warn-only` exits with code 0.
  Output files are written with error-slide placeholders at affected positions.
  (traces to BC-1.15.003 postcondition 3, invariant 4)

- [ ] **AC-005** — An export failure (E-EXP-*) exits with code 3. No output files are
  written.
  (traces to BC-1.15.003 postcondition 4)

- [ ] **AC-006** — A source with 3 independent errors (e.g., 2 E-EVL-001, 1 E-A11-001) all
  appear in the terminal output in a single build run.
  (traces to BC-1.15.002 postcondition 1)

- [ ] **AC-007** — Errors in the terminal output are printed in source-file order: ascending
  by (file, line, col).
  (traces to BC-1.15.002 postcondition 2)

- [ ] **AC-008** — Every emitted diagnostic carries a non-empty correction hint rendered
  below the source snippet.
  (traces to BC-1.15.001 postcondition 2, invariant 2)

- [ ] **AC-009** — In TTY mode: diagnostics are rendered with ANSI color codes and source
  snippets via `miette::GraphicalReportHandler`. In non-TTY mode (redirected output): plain
  text without ANSI codes.
  (traces to BC-1.15.001 postcondition 4, edge case EC-004)

- [ ] **AC-010** — `--format pptx` produces only `dist/deck.pptx`; `--format pptx,html`
  produces `dist/deck.pptx` and `dist/deck.html`; default (no `--format`) produces all 4
  formats.
  (traces to BC-1.15.003 postcondition 1 — success path implies format selection handled)

- [ ] **AC-011** — Six tracing spans emitted per build: `parse`, `evaluate`, `brand`,
  `validate`, `layout`, `export`. Each carries the stage name and duration.
  (traces to NFR-032 — all 6 pipeline stages emit tracing spans)

- [ ] **AC-012** — Exit code reflects the earliest pipeline-stage failure across all accumulated
  errors (exit code reflects earliest pipeline-stage failure: E-PAR exit 1 takes precedence
  over E-EVL exit 2 and E-EXP exit 3).
  (traces to BC-1.15.003 postcondition 6)

- [ ] **AC-013** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, and
  `clippy::pedantic` clean.
  (traces to NFR-021, NFR-022, NFR-024)

- [ ] **AC-014** — All public items have rustdoc. `cargo doc --no-deps` produces 0 warnings.
  (traces to NFR-023)

- [ ] **AC-015** — `slideforge build` accepts `--otel-endpoint <url>` to enable OpenTelemetry
  export via a `tracing-opentelemetry` subscriber layer. When the flag is set, all tracing
  spans produced during the six pipeline stages are exported as OTel traces to the specified
  OTLP endpoint. When the flag is absent, no OTel dependency is activated — the default
  `tracing-subscriber` fmt layer is used instead. The `tracing-opentelemetry` feature is
  gated behind a Cargo feature flag (`otel`) so that builds without OTel export have no
  additional transitive dependencies. Cite ADR-021 (tokio async runtime required by
  `opentelemetry_sdk` rt-tokio feature).

  OTel initialization uses the current builder API (NOT the deprecated `new_pipeline()`):
  ```rust
  let exporter = opentelemetry_otlp::SpanExporter::builder()
      .with_tonic()
      .with_endpoint(endpoint_url)
      .build()?;
  let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
      .with_batch_exporter(exporter)
      .build();
  let tracer = provider.tracer("slideforge");
  let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
  ```
  Do NOT use the deprecated `opentelemetry::global::set_text_map_propagator()` or
  `new_pipeline().install_batch()` pattern — these are removed in opentelemetry 0.32.
  (traces to NFR-032 — opentelemetry-compatible export hooks required by quality bar)

## Tasks

1. Define the root `Cli` struct with global flags using `clap` derive (`{workspace = true}`, =4.6.1):
   ```rust
   #[derive(Parser)]
   #[command(name = "slideforge", version, about)]
   pub struct Cli {
       #[command(flatten)]
       pub global: GlobalFlags,
       #[command(subcommand)]
       pub command: Command,
   }

   #[derive(Args)]
   pub struct GlobalFlags {
       #[arg(long, global = true)]
       pub json: bool,
       #[arg(long, global = true)]
       pub quiet: bool,
       #[arg(long, short = 'v', global = true, action = ArgAction::Count)]
       pub verbose: u8,
       #[arg(long, global = true, env = "NO_COLOR")]
       pub no_color: bool,
       #[arg(long, global = true)]
       pub warn_only: bool,
       #[arg(long, global = true)]
       pub offline: bool,
   }

   #[derive(Subcommand)]
   pub enum Command {
       Build(BuildArgs),
       Watch(WatchArgs),   // defined in STORY-056
       Init(InitArgs),     // defined in STORY-057
       ExtractBrand(ExtractBrandArgs), // defined in STORY-057
       Config(ConfigArgs), // STORY-065
       Package(PackageArgs), // STORY-060
   }
   ```
2. Define `BuildArgs`:
   ```rust
   #[derive(Args)]
   pub struct BuildArgs {
       /// Path to the .sf source file
       pub source: PathBuf,
       /// Output directory (default: dist/)
       #[arg(long, default_value = "dist")]
       pub output_dir: PathBuf,
       /// Comma-separated list of output formats (default: all)
       #[arg(long, value_delimiter = ',', default_values_t = all_formats())]
       pub format: Vec<OutputFormat>,
       /// Select a named deck variant
       #[arg(long)]
       pub variant: Option<String>,
   }

   #[derive(ValueEnum, Clone, Debug)]
   pub enum OutputFormat { Pptx, Docx, Pdf, Html }
   ```
3. Implement `run_build(args: &BuildArgs, global: &GlobalFlags) -> ExitCode` in
   `src/commands/build.rs`:
   - Detect TTY: `use_color = std::io::stderr().is_terminal() && !global.no_color && !global.json`
     (use `std::io::IsTerminal` — do NOT add the `atty` crate; `atty` is unmaintained and the stdlib trait is stable since Rust 1.70)
   - Construct `CompileOptions` from flags; call `slideforge::compile(options)`
   - On `Ok(outputs)`: write files to `output_dir`; print success summary
   - On `Err(sink)`: call `DiagnosticRenderer::render_all()`; return exit code from
     `sink.max_severity()`
4. Implement exit code mapping:
   ```rust
   fn exit_code_for_severity(sev: Option<ParseSeverity>) -> ExitCode {
       match sev {
           None => ExitCode::SUCCESS,
           Some(ParseSeverity::Warning) => ExitCode::SUCCESS,
           Some(ParseSeverity::Error) => ExitCode::from(2),
           Some(ParseSeverity::Fatal) => ExitCode::from(1),  // parse errors
           // Export fatal uses 3; handled by ExportError variant
       }
   }
   ```
5. Add `tracing-subscriber` initialization in `main()`:
   - Use `tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env())` by default;
     structured JSON format when `--json`. Requires `tracing-subscriber` with feature `env-filter`.
   - `RUST_LOG` env var drives `EnvFilter`; CLI verbosity sets the fallback level if `RUST_LOG` is absent.
   - Guard initialization with `std::sync::OnceLock` or call `try_init()` (returns `Err` on second
     init; guard so tests can call it safely without panic).
   - Emit `tracing::info_span!("parse")`, `("evaluate")`, `("brand")`, `("validate")`,
     `("layout")`, `("export")` within the pipeline orchestration in the root crate
     `slideforge::compile()`
6. Add format selection logic: `CompileOptions::formats: Vec<OutputFormat>` — only run
   exporters for selected formats; skip unselected ones
7. Implement output atomicity: write output to a temporary path inside `output_dir`;
   on success, rename to final path. On failure, delete the temporary file.
8. Write unit and integration tests for all ACs

## File List

- `crates/slideforge-cli/src/main.rs` — CLI entry point: parse args, route to subcommand
- `crates/slideforge-cli/src/cli.rs` — `Cli`, `GlobalFlags`, `Command`, `BuildArgs`,
  `OutputFormat` derive structs
- `crates/slideforge-cli/src/commands/build.rs` — `run_build()` function
- `crates/slideforge-cli/src/commands/mod.rs` — command dispatch
- `crates/slideforge-cli/src/exit_code.rs` — `exit_code_for_severity()`, exit code constants
- `crates/slideforge-cli/src/output.rs` — output file writing + atomicity
- `crates/slideforge-cli/src/tracing_setup.rs` — tracing-subscriber initialization
- `crates/slideforge-cli/Cargo.toml` — add `{workspace = true}` deps: `clap` (=4.6.1),
  `miette` (=7.6.0), `tracing` (=0.1.44), `tracing-subscriber` (=0.3.23 env-filter),
  `thiserror` (=2.0.18), `serde_json` (=1.0.150); add optional `otel` feature deps:
  `opentelemetry =0.32.0`, `opentelemetry_sdk =0.32.0`, `opentelemetry-otlp =0.32.0`,
  `tracing-opentelemetry =0.33.0`.
  TTY detection: use `std::io::IsTerminal` — no `atty` crate.
- `crates/slideforge-cli/tests/build_integration.rs` — integration tests

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~5 500 |
| BC-1.15.001, BC-1.15.002, BC-1.15.003 | ~4 500 |
| STORY-010 (DiagnosticSink, DiagnosticRenderer, ParseSeverity API) | ~3 000 |
| STORY-049 (plugin registry API) | ~2 000 |
| Target source files to write | ~5 000 |
| Test files | ~3 000 |
| **Total** | **~24 000** |

Context budget: 24 000 / 200 000 ≈ 12% — within limit.

## Test Strategy

**Unit tests** (`crates/slideforge-cli/src/commands/build.rs #[cfg(test)]`):

- `test_exit_code_parse_error()`: mock `DiagnosticSink` with 1 E-PAR variant; assert
  `exit_code_for_severity()` returns `ExitCode::from(1)`.
- `test_exit_code_eval_error_strict()`: mock sink with E-EVL variant; strict mode; assert
  exit code 2.
- `test_exit_code_eval_error_warn_only()`: mock sink with E-EVL variant; warn-only; assert
  exit code 0.
- `test_exit_code_success()`: empty sink; assert exit code 0.
- `test_exit_code_highest_severity()`: sink with E-PAR + E-EVL; assert exit code 1 (parse
  takes precedence).
- `test_format_selection_pptx_only()`: build with `--format pptx`; assert only `.pptx`
  written.

**Integration tests** (`crates/slideforge-cli/tests/build_integration.rs`):

- `test_build_success()`: build a valid fixture deck; assert exit 0; assert dist/ files
  created for all 4 formats.
- `test_build_parse_error_exit_1()`: build a fixture with a tab character; assert exit 1;
  assert no output files in dist/.
- `test_build_eval_error_strict_exit_2()`: build a fixture with undefined variable; strict
  mode; assert exit 2; no output files.
- `test_build_eval_error_warn_only_exit_0()`: same fixture; `--warn-only`; assert exit 0;
  assert output files written.
- `test_build_multi_error_all_reported()`: 3-error fixture; assert all 3 errors in stderr.
- `test_build_error_order()`: fixture with errors at different line numbers; assert output
  order matches source order.
- `test_build_no_color_plain_text()`: pipe stdout; assert no ANSI codes in output.
- `test_build_tracing_spans_emitted()`: use `tracing_test` crate; assert all 6 span names
  appear in the span log.

## Dependencies

- **Depends on:** STORY-010 (provides `DiagnosticSink`, `DiagnosticRenderer`, `ParseSeverity`
  — the core error infrastructure consumed by the CLI)
- **Depends on:** STORY-049 (plugin registry assembly — the `slideforge::compile()` entry
  point that the CLI calls)
- **Depends on:** STORY-050 (end-to-end integration test suite — validates the full pipeline
  the CLI orchestrates)
- **Blocks:** STORY-056 (watch mode extends the build command infrastructure)
- **Blocks:** STORY-057 (init/extract-brand share CLI struct and global flags)
- **Blocks:** STORY-058 (flags implementation depends on GlobalFlags struct here)
- **Blocks:** STORY-059 (benchmarks run the same build pipeline defined here)

## Dependency Anchor Justifications

- SS-18 owns this story's scope because SS-18 is the CLI Orchestrator subsystem owning
  `slideforge-cli` per ARCH-INDEX Subsystem Registry. All user-facing command dispatch
  and pipeline orchestration lives here.
- STORY-055 depends on STORY-010 because the exit code logic and error rendering directly
  consume `ParseSeverity`, `DiagnosticSink`, and `DiagnosticRenderer` defined in STORY-010.
- STORY-055 depends on STORY-049 because the build command invokes `slideforge::compile()`
  which is the assembled plugin registry entry point defined in STORY-049.
- STORY-055 blocks STORY-056/057/058/059 because all later CLI stories share the `Cli`
  struct, `GlobalFlags`, and command dispatch infrastructure defined here.

## Architecture Compliance Rules

1. `slideforge-cli` is an **effectful shell** crate — it performs I/O (stdout, stderr, file
   writes, TTY detection) per the purity boundary map.
2. The CLI crate must NOT contain pipeline logic — all compile logic lives in the root
   `slideforge` crate or specialist crates. The CLI is an orchestration façade only.
3. TTY detection uses `std::io::IsTerminal` (stable since Rust 1.70) — do NOT add the
   `atty` crate. Check `std::io::stderr().is_terminal()`.
4. Output atomicity: write to `<output_dir>/<filename>.tmp` then `fs::rename()` to final
   path. Never write partial output.
5. `#![forbid(unsafe_code)]` must be present at the crate root.
6. `tracing_subscriber` must NOT be initialized more than once per process — guard with
   `std::sync::OnceLock` or `tracing_subscriber::fmt().try_init()`. Note: `try_init()` returns
   `Err(SetLoggerError)` on a second call; in production wrap in `OnceLock`; in tests call
   `try_init()` and discard the error so concurrent test threads do not panic.

**Forbidden dependencies for `slideforge-cli`:**
- Must NOT import `chumsky` directly — parsing is SS-01's domain.
- Must NOT import `ooxmlsdk`, `pdf-writer`, or `krilla` directly — export is SS-06/07/08/09.
- Must NOT import internal crates below `slideforge` (the root) — the CLI talks to the
  root crate only; the root crate assembles the registry.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `clap` | `{workspace = true}` (=4.6.1) | CLI arg parsing, derive macros. Centralized in `[workspace.dependencies]` per ADR-022. |
| `miette` | `{workspace = true}` (=7.6.0) | Diagnostic rendering with source snippets. Centralized per ADR-022. |
| `tracing` | `{workspace = true}` (=0.1.44) | Span instrumentation. Centralized per ADR-022. |
| `tracing-subscriber` | `{workspace = true}` (=0.3.23, feature `env-filter`) | Tracing setup and `EnvFilter` for `RUST_LOG`. Centralized per ADR-022. |
| `thiserror` | `{workspace = true}` (=2.0.18) | Error derives for CLI-layer errors. Centralized per ADR-022. |
| `serde_json` | `{workspace = true}` (=1.0.150) | JSON diagnostic output for `--json` mode. Centralized per ADR-022. |
| `opentelemetry` | `=0.32.0` (feature `otel` only) | OTel API — gated behind `otel` Cargo feature. |
| `opentelemetry_sdk` | `=0.32.0` (feature `rt-tokio`; `otel` only) | OTel SDK — gated behind `otel` Cargo feature. Cite ADR-021 (tokio async runtime). |
| `opentelemetry-otlp` | `=0.32.0` (features `trace,grpc-tonic`; `otel` only) | OTLP gRPC exporter for `--otel-endpoint`. Gated behind `otel` feature. |
| `tracing-opentelemetry` | `=0.33.0` (intentionally one minor ahead of opentelemetry; `otel` only) | OTel subscriber layer. Gated behind `otel` feature. |

## File Structure Requirements

```
crates/slideforge-cli/
  src/
    main.rs                 # entry: Cli::parse(); commands::dispatch()
    cli.rs                  # Cli, GlobalFlags, Command, BuildArgs, OutputFormat
    commands/
      mod.rs                # dispatch(cli: Cli) -> ExitCode
      build.rs              # run_build(args: &BuildArgs, global: &GlobalFlags) -> ExitCode
      watch.rs              # stub (STORY-056)
      init.rs               # stub (STORY-057)
      extract_brand.rs      # stub (STORY-057)
    exit_code.rs            # exit_code_for_severity(), constants
    output.rs               # OutputWriter: atomic file writing
    tracing_setup.rs        # init_tracing(global: &GlobalFlags)
    lib.rs                  # (optional) public API for integration tests
  tests/
    build_integration.rs    # integration tests
  Cargo.toml
```

## Previous Story Intelligence

N/A — STORY-055 is the first CLI story in EPIC-15. However, STORY-010 established the
`DiagnosticSink`, `DiagnosticRenderer`, and `ParseSeverity` API. Read STORY-010 before
implementing to understand the exact method signatures and JSON schema.

Key lesson from STORY-010: `DiagnosticRenderer` does NOT detect TTY internally — the CLI
is responsible for determining `use_color: bool` and passing it in. Detect TTY at the
start of `run_build()` using `std::io::stderr().is_terminal()`.

## Implementation Notes

### Exit code mapping (three-tier model)

```
ParseSeverity::Fatal (E-PAR-*) → exit 1
ParseSeverity::Fatal (E-EXP-*) → exit 3   (export errors)
ParseSeverity::Error (strict)  → exit 2
ParseSeverity::Error (warn-only) → exit 0
ParseSeverity::Warning         → exit 0
None (success)                 → exit 0
```

The `DiagnosticSink::max_severity()` returns the highest severity. But E-PAR and E-EXP
are both `Fatal` — they need separate codes. Solution: add an `ExitCode` method to
`DiagnosticSink` that maps the error code categories directly:
- If any E-PAR-* present → 1
- If any E-EXP-* present → 3 (only if no E-PAR-*)
- If any Error-severity in strict mode → 2
- Otherwise → 0

### `--json` flag diagnostic output

When `--json` is active, write JSON to stderr:
```json
{
  "diagnostics": [...],   // from DiagnosticSink::to_json()
  "total": 3,
  "has_fatal": true,
  "exit_code": 2
}
```

### Format selection default

```rust
fn all_formats() -> Vec<OutputFormat> {
    vec![OutputFormat::Pptx, OutputFormat::Docx, OutputFormat::Pdf, OutputFormat::Html]
}
```

### tracing span structure

```rust
let _parse_span = tracing::info_span!("parse", source = %source.display()).entered();
// ... call parse
drop(_parse_span);

let _eval_span = tracing::info_span!("evaluate").entered();
// ... call evaluate
// etc.
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--format` specified with invalid value | `clap` reports error before compilation starts; exit 2 (usage error) |
| EC-002 | `source` file does not exist | E-PAR-005 (file not found) with span = None; formatted as "Error: source file not found: <path>"; exit 1 |
| EC-003 | `--output-dir` not writable | E-EXP-007; exit 3; no partial files |
| EC-004 | `--warn-only` + E-PAR-* | E-PAR-* still exits 1; warn-only does NOT demote parse errors (BC-1.15.003 invariant 1) |
| EC-005 | Only cosmetic diagnostics (E-BRD-003) | Exit 0; full output written; warnings printed to stderr |
| EC-006 | Build with `--variant` flag referencing undefined variant | E-VAR-004 (undefined variant); exit 2 |
