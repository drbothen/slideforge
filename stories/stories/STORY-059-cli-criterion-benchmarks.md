---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-059
title: "CLI: Criterion performance benchmarks"
epic: EPIC-15
wave: 5
points: 5
priority: P0
tdd_mode: facade
status: draft
# BC status: no BCs reference this story directly — benchmarks are NFR/observability
# concerns. Status remains draft until wave-gate sign-off.
crate: slideforge-cli
behavioral_contracts: []
verification_properties: []
nfr_refs:
  - NFR-001
  - NFR-002
  - NFR-005
  - NFR-006
  - NFR-021
  - NFR-022
  - NFR-023
  - NFR-024
depends_on:
  - STORY-055
  - STORY-056
blocks: []
subsystems:
  - SS-18
target_module: slideforge-cli
---

# STORY-059: CLI: Criterion performance benchmarks

## Summary

Implement Criterion 0.8 performance benchmarks for the slideforge build pipeline in
`crates/slideforge-cli/benches/build_bench.rs`. The benchmarks enforce the CLAUDE.md
quality bar performance gates and are run in CI to detect regressions.

**`tdd_mode: facade`** — This story delivers a combined scaffold + implementation in one
step. There are no stubs to stub out; the benchmark harness and the functions under test
both exist (STORY-055/056 built them). Red Gate TDD does not apply. Mutation testing at
the wave gate replaces the Red Gate density check.

Three benchmark groups:

1. **`cold_build`** — compile a 25-slide fixture deck from scratch (parse + eval + brand
   synthesis + validate + layout + all 4 exporters). Target: < 500ms wall-clock on a
   GitHub Actions Linux x86_64 runner (NFR-001).

2. **`incremental_rebuild`** — watch-mode re-evaluation after a single `.sf` field change
   (no data re-fetch, no brand re-synthesis). Target: < 50ms wall-clock (NFR-002).

3. **`serialization_only`** — PPTX/DOCX serialization for the same 25-slide fixture from
   a pre-built `LaidOutDeck` (no parse/eval/layout). Target: < 200ms wall-clock (NFR-005).

Additionally: isolated benchmarks for `parse_only` and `eval_only` stages for diagnostic
purposes (not CI-gated, but helpful for locating regressions).

The CI job runs `cargo bench --bench build_bench -- --save-baseline current` and compares
against a stored baseline via `critcmp =0.1.8`. Absolute threshold enforcement parses
`target/criterion/.../new/estimates.json` directly with `jq`.

## Note on Behavioral Contracts

`behavioral_contracts: []` — benchmarks do not implement behavioral contracts. Per the
Spec-First Gate (S-7.01), the status remains `draft` until wave-gate review (the gate
for facade-mode stories is mutation testing at wave gate, not BC authorship). The
`tdd_mode: facade` annotation is the explicit signal.

## Acceptance Criteria

- [ ] **AC-001** — `cargo bench --bench build_bench` runs to completion without panics on the
  developer's local machine and on all 5 CI platforms.
  (traces to NFR-001 — validation method is CI criterion benchmark)

- [ ] **AC-002** — `cold_build/25_slides_all_formats` benchmark mean time is < 500ms on a
  GitHub Actions Linux x86_64 runner (`ubuntu-latest`).
  (traces to NFR-001 — cold build < 500ms)

- [ ] **AC-003** — **DEFERRED to v1.x** — The `incremental_rebuild/single_field_change`
  benchmark is implemented and records timing, but the < 50ms gate is NOT enforced in v1.0 CI.
  NFR-002 (incremental rebuild < 50ms) is deferred to v1.x (nfr-catalog v1.3) because it
  requires the comemo incremental cache which is a post-v1.0 feature. The v1.0 benchmark calls
  `slideforge::compile()` on a modified source string and records the result; the 50ms threshold
  enforcement will be added when comemo ships.
  (traces to NFR-002 — DEFERRED to v1.x; v1.0 keeps the cold-build gate only)

- [ ] **AC-004** — `serialization/pptx_25_slides` + `serialization/docx_25_slides` benchmark
  mean times are each < 200ms.
  (traces to NFR-005 — PPTX/DOCX serialization < 200ms for 25-slide deck)

- [ ] **AC-005** — The 25-slide fixture (`benches/fixtures/bench_deck.sf`) uses at least 10
  distinct slide types, 2 file-based `@data` sources, and brand.toml synthesis (per the
  NFR-001 validation workflow definition in the NFR catalog).
  (traces to NFR-001 validation workflow — "25 slides using at least 10 slide types, brand.toml
  synthesis, 2 @data sources (file-based)")

- [ ] **AC-006** — Criterion saves baseline JSON files under `target/criterion/`. After the
  bench job runs, use `critcmp =0.1.8` for relative-to-baseline regression detection in CI.
  (traces to NFR-001/002 — "CI regression detection")

- [ ] **AC-007** — The CI workflow step for benchmarks (`jobs.bench` in `ci.yml`) fails the PR
  if `cold_build` mean exceeds 500ms. Regression gating uses two methods:
  (a) Relative: `critcmp` compares current run against saved baseline and reports regressions.
  (b) Absolute: parse `target/criterion/cold_build__25_slides_all_formats/new/estimates.json`
      and assert `mean.point_estimate` (in ns) ÷ 1_000_000 < 500ms using `jq` + shell exit 1.
  Note: `criterion-compare` is not a real crate — use `critcmp =0.1.8` for baseline comparison.
  (traces to NFR-001 — "Gate: blocking — PR cannot merge if NFR-001 is violated")

- [ ] **AC-008** — Benchmarks are in a separate binary target (`[[bench]]` in `Cargo.toml`) and
  do NOT run under `cargo test`. They run only under `cargo bench`.
  (traces to NFR-021 — no accidental slow test suite; benchmarks excluded from `cargo test`)

- [ ] **AC-009** — `#![forbid(unsafe_code)]` and `clippy::pedantic` clean on the bench binary.
  (traces to NFR-021, NFR-022, NFR-024)

- [ ] **AC-010** — All public benchmark helper functions and fixture helpers have rustdoc.
  (traces to NFR-023)

## Tasks

1. Create bench fixture: `benches/fixtures/bench_deck.sf` — 25-slide deck using at least
   10 distinct slide types. Include 2 `@data` directives pointing to
   `benches/fixtures/data1.json` and `benches/fixtures/data2.json`. Include `brand.toml`
   reference.
2. Create `benches/fixtures/brand.toml` with all 12 OOXML color slots (same defaults as
   STORY-057's scaffold template).
3. Create `benches/fixtures/data1.json` and `data2.json` — small JSON arrays (10 items
   each) used by `@data` directives in `bench_deck.sf`.
4. Implement `benches/build_bench.rs` using Criterion 0.8.2:
   ```rust
   use std::hint::black_box;  // ALWAYS use std::hint::black_box — criterion::black_box is deprecated and fails -D warnings
   use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

   fn bench_cold_build(c: &mut Criterion) {
       let source = include_str!("fixtures/bench_deck.sf");
       let brand = include_str!("fixtures/brand.toml");

       c.bench_function("cold_build/25_slides_all_formats", |b| {
           b.iter(|| {
               let opts = CompileOptions {
                   source: black_box(source.to_string()),
                   brand_toml: black_box(Some(brand.to_string())),
                   formats: all_formats(),
                   warn_only: false,
                   offline: true,  // use file-based data sources only
                   ..Default::default()
               };
               black_box(slideforge::compile(opts).expect("fixture must compile"))
           })
       });
   }

   fn bench_incremental_rebuild(c: &mut Criterion) {
       // NFR-002 (<50ms) is DEFERRED to v1.x (nfr-catalog v1.3); true incremental requires
       // comemo which is post-v1.0. This benchmark records timing for regression tracking only.
       // The v1.0 implementation calls compile() on the modified source string (no warm cache).
       // When comemo ships, replace compile() with compile_incremental() and enable the CI gate.
       let source = include_str!("fixtures/bench_deck.sf");

       c.bench_function("incremental_rebuild/single_field_change", |b| {
           b.iter(|| {
               let modified = black_box(source).replacen(
                   r#"title "Welcome to slideforge""#,
                   r#"title "Updated title""#,
                   1,
               );
               let opts = CompileOptions {
                   source: black_box(modified),
                   formats: vec![OutputFormat::Pptx],
                   offline: true,
                   ..Default::default()
               };
               // v1.0: full compile; comemo incremental available in v1.x
               black_box(slideforge::compile(opts).expect("incremental must succeed"))
           })
       });
   }

   fn bench_serialization(c: &mut Criterion) {
       let laid_out = precompile_deck();   // build LaidOutDeck once; reuse across iters

       let mut group = c.benchmark_group("serialization");
       group.bench_function("pptx_25_slides", |b| {
           b.iter(|| {
               black_box(slideforge_pptx::export(black_box(&laid_out))
                   .expect("pptx export must succeed"))
           });
       });
       group.bench_function("docx_25_slides", |b| {
           b.iter(|| {
               black_box(slideforge_docx::export(black_box(&laid_out))
                   .expect("docx export must succeed"))
           });
       });
       group.finish();
   }

   fn bench_parse_only(c: &mut Criterion) {
       let source = include_str!("fixtures/bench_deck.sf");
       c.bench_function("parse_only/25_slides", |b| {
           b.iter(|| {
               black_box(slideforge_syntax::parse(black_box(source), "bench_deck.sf")
                   .expect("fixture must parse"))
           })
       });
   }

   criterion_group!(benches, bench_cold_build, bench_incremental_rebuild,
                    bench_serialization, bench_parse_only);
   criterion_main!(benches);
   ```
5. Add `[[bench]]` section to `crates/slideforge-cli/Cargo.toml`:
   ```toml
   [[bench]]
   name = "build_bench"
   harness = false

   [dev-dependencies]
   # criterion =0.8.2: MSRV 1.88. html_reports is OPT-IN (not default).
   # Default features: cargo_bench_support, plotters, rayon.
   criterion = { version = "=0.8.2", features = ["html_reports"] }
   # critcmp for baseline regression comparison in CI
   critcmp = "=0.1.8"
   ```
   If the workspace centralizes `criterion`, use `{workspace = true}`; otherwise pin at `=0.8.2`.
6. Add `jobs.bench` to `.github/workflows/ci.yml`:
   ```yaml
   bench:
     runs-on: ubuntu-latest
     steps:
       - uses: actions/checkout@v4
       - name: Run benchmarks
         run: cargo bench --bench build_bench -- --save-baseline current
       - name: Check cold_build absolute threshold (< 500ms, NFR-001)
         run: |
           # Parse Criterion's estimates.json for the cold_build mean (nanoseconds).
           # Criterion 0.8 writes to target/criterion/<group>/<bench>/new/estimates.json.
           ESTIMATES="target/criterion/cold_build/25_slides_all_formats/new/estimates.json"
           NS=$(jq '.mean.point_estimate' "$ESTIMATES")
           MS=$(echo "scale=2; $NS / 1000000" | bc)
           echo "cold_build mean: ${MS}ms"
           python3 -c "import sys; ms=float('${MS}'); sys.exit(0 if ms < 500 else (print(f'FAIL: cold_build {ms:.1f}ms > 500ms gate', file=sys.stderr) or 1))"
       # NOTE: incremental_rebuild <50ms gate is DEFERRED to v1.x (NFR-002, nfr-catalog v1.3).
       # The benchmark runs but the threshold is not enforced in v1.0 CI.
       - name: Baseline regression check (critcmp)
         run: |
           # Compare current run against the committed baseline (main branch).
           # critcmp exits 0 if no regressions; exits 1 if any bench regressed > noise threshold.
           cargo critcmp main current --threshold 10
         continue-on-error: true  # non-blocking in v1.0; tighten at release
   ```
7. Save baseline: run `cargo bench -- --save-baseline main` on first setup; commit
   `benches/baselines/` to the repository.
8. Document benchmark purpose and threshold rationale in `benches/build_bench.rs` as a
   module-level doc comment.

## File List

- `crates/slideforge-cli/benches/build_bench.rs` — Criterion benchmark harness
- `crates/slideforge-cli/benches/fixtures/bench_deck.sf` — 25-slide fixture deck
- `crates/slideforge-cli/benches/fixtures/brand.toml` — brand config for fixture
- `crates/slideforge-cli/benches/fixtures/data1.json` — data source 1
- `crates/slideforge-cli/benches/fixtures/data2.json` — data source 2
- `crates/slideforge-cli/Cargo.toml` — add `[[bench]]` section + `criterion =0.8.2` dev-dep
  (feature `html_reports` opt-in) + `critcmp =0.1.8` dev-dep
- `.github/workflows/ci.yml` — add `jobs.bench` with threshold checks

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~4 500 |
| STORY-055 (CompileOptions API) | ~1 500 |
| STORY-056 (compile_incremental() API) | ~1 500 |
| Criterion 0.8 API reference | ~1 000 |
| Benchmark harness code to write | ~3 000 |
| Fixture files to write | ~1 500 |
| CI workflow update | ~1 000 |
| **Total** | **~14 000** |

Context budget: 14 000 / 200 000 ≈ 7.0% — within limit.

## Test Strategy

Benchmarks are not unit-tested in the TDD sense. Validation strategy:

**Functional correctness** (that benchmarks run without panicking):
- `cargo bench --bench build_bench -- --test` runs benchmarks as unit tests (Criterion
  supports `--test` mode). Add to `ci.yml`'s `test` job as a sanity check.

**Performance gate** (CI threshold check):
- `jobs.bench` workflow step as described in Tasks item 6.
- Threshold is checked via a simple Python one-liner against the Criterion bencher output.

**Baseline regression detection**:
- `critcmp =0.1.8` compares two Criterion baseline JSON files and outputs a report.
  Use `cargo critcmp main current --threshold 10` in the PR workflow to surface regressions
  exceeding 10% even if below the absolute threshold.
  Note: `criterion-compare` is NOT a real crate — it does not exist on crates.io. Use `critcmp`.

**Wave-gate mutation testing** (facade-mode requirement):
- Mutation testing at the EPIC-15 wave gate via `cargo-mutants` runs against the
  benchmark fixture helpers (e.g., `precompile_deck()`, `all_formats()`) to ensure the
  helpers are not trivially wrong.

## Dependencies

- **Depends on:** STORY-055 (provides `CompileOptions`, `slideforge::compile()`, and
  `OutputFormat` used in benchmark setup)
- **Depends on:** STORY-056 (provides `slideforge::compile_incremental()` used in the
  incremental rebuild benchmark)
- **Blocks:** (none)

## Dependency Anchor Justifications

- SS-18 owns this story's scope because benchmarks live in `slideforge-cli/benches/` —
  the CLI crate is the integration point for the full pipeline being benchmarked, per
  ARCH-INDEX SS-18 description.
- STORY-059 depends on STORY-055 because `CompileOptions` and `slideforge::compile()` are
  defined there; the benchmark calls them directly.
- STORY-059 depends on STORY-056 because `slideforge::compile_incremental()` (required by
  the incremental rebuild benchmark) is implemented in the context of the watch mode work
  in STORY-056.

## Architecture Compliance Rules

1. Benchmarks are `[[bench]]` binary targets — they are compiled separately from the main
   library and CLI binary. They do NOT affect the production binary size.
2. All fixture data is embedded via `include_str!()` — no runtime file I/O in the hot
   benchmark loop. (This ensures benchmark reproducibility across CI environments.)
3. `black_box()` must wrap EVERY benchmark input and output to prevent the compiler from
   optimizing away the benchmarked code.
4. The `precompile_deck()` and similar helpers that build state once before the benchmark
   loop MUST NOT be included in the measured time. Use Criterion's `b.iter_with_setup()`
   or run setup in the outer scope of `b.iter()`.
5. The benchmarks must compile with `#![forbid(unsafe_code)]` — even bench binaries
   respect this rule.

**Forbidden in benchmarks:**
- Do NOT perform network I/O in benchmarks (`--offline: true` always set in fixture opts).
- Do NOT write output files to disk in the hot loop — measure compute only, not I/O.
- Do NOT use `std::thread::sleep()` in benchmarks.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `criterion` | `=0.8.2` (dev-dep; feature `html_reports` opt-in) | Benchmark harness. MSRV 1.88. Default features: cargo_bench_support, plotters, rayon. `html_reports` is NOT default — must be explicitly enabled. Use `{workspace = true}` if centralized per ADR-022; else pin `=0.8.2`. |
| `critcmp` | `=0.1.8` (dev-dep) | CLI tool for Criterion baseline comparison in CI (`cargo critcmp`). |

**CRITICAL:** Always import `use std::hint::black_box;` — NEVER `use criterion::black_box`
(deprecated in Criterion 0.5+, removed/empty in 0.8, causes compile error under `-D warnings`).

Note: `criterion` and `critcmp` are dev-dependencies. Do NOT add them as production dependencies.

## File Structure Requirements

```
crates/slideforge-cli/
  benches/
    build_bench.rs          # Criterion benchmark harness (main file)
    fixtures/
      bench_deck.sf         # 25-slide fixture deck (10+ slide types, 2 @data sources)
      brand.toml            # Brand config for fixture
      data1.json            # Data source 1 (10 items)
      data2.json            # Data source 2 (10 items)
    baselines/              # Criterion baseline JSON files (committed)
      .gitkeep              # Keep directory in git even when empty
  Cargo.toml                # Add [[bench]] + criterion dev-dep
```

## Previous Story Intelligence

From STORY-055: `CompileOptions` struct. Confirm its exact fields before using in
benchmarks — particularly `warm_cache`, `offline`, and `formats`. If `warm_cache` does
not exist in the `CompileOptions` API after STORY-055 is implemented, the incremental
rebuild benchmark must call `slideforge::compile_incremental(modified_source, prev_cache)`
with a different signature. Read the STORY-055 implementation before writing the bench.

From STORY-056: `slideforge::compile_incremental()` may be named differently (e.g.,
`slideforge::compile_with_cache()`). Check the actual symbol name after STORY-056 ships.
If the function does not exist (e.g., incremental is not yet exposed at the root crate
level), the `incremental_rebuild` benchmark should call `slideforge::compile()` with the
modified source and note in a comment that true incremental benchmarking requires
STORY-056's cache API.

Key from NFR-001 validation workflow (nfr-catalog.md): the fixture specification is
binding — "25 slides using at least 10 slide types, brand.toml synthesis, 2 @data sources
(file-based)". The `bench_deck.sf` fixture must satisfy all three criteria or the
benchmark does not correctly validate NFR-001.

## Implementation Notes

### Font loading in the cold-build benchmark

For the `cold_build` benchmark, the font loading step must NOT call `load_system_fonts()` in
the timed region — system font discovery is non-deterministic in timing (varies by OS, number
of fonts installed, Windows registry scan) and can dominate the measurement on CI.

Instead, load a bundled/pinned font set in the benchmark setup (outside `b.iter()`):
```rust
// Setup (outside the hot loop):
let font_db = {
    let mut db = fontdb::Database::new();
    db.load_fonts_dir("benches/fixtures/fonts/");  // bundled subset, deterministic
    // OR: db.load_font_data(include_bytes!("fixtures/fonts/Calibri.ttf").to_vec());
    db
};

// Hot loop:
b.iter(|| {
    let opts = CompileOptions { font_db: black_box(font_db.clone()), ... };
    black_box(slideforge::compile(opts).expect("fixture must compile"))
})
```

Commit a minimal set of fonts (e.g., Calibri subset, 2-3 files) under `benches/fixtures/fonts/`.
This ensures the cold-build benchmark measures actual pipeline work, not OS font scanning.

### bench_deck.sf fixture slide type coverage

The fixture must use at least 10 of the 31 slide types. Suggested set (covers diverse
layout complexity):
1. `slide title:` — title slide
2. `slide bullets:` — bulleted list
3. `slide two-column:` — two-column layout
4. `slide chart:` — bar chart with `@data` binding
5. `slide table:` — data table with `@data` binding
6. `slide quote:` — pull quote
7. `slide image-full:` — full-bleed image
8. `slide section-divider:` — section break
9. `slide agenda:` — agenda slide
10. `slide end:` — end slide

Use `@data sales from "data1.json"` and `@data metrics from "data2.json"`.

### Criterion configuration

```rust
use criterion::Criterion;

pub fn criterion_config() -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::from_secs(10))  // 10s per bench
        .warm_up_time(std::time::Duration::from_secs(3))       // 3s warmup
        .sample_size(50)                                        // 50 samples
        .noise_threshold(0.05)                                  // 5% noise tolerance
}
```

### CI threshold enforcement

Criterion 0.8 does NOT support `--output-format bencher` reliably for threshold checking.
Instead, parse the JSON estimates file written by `--save-baseline`:

```
target/criterion/<group>/<benchmark_name>/new/estimates.json
```

The JSON structure:
```json
{
  "mean": { "point_estimate": 312483521.0, "standard_error": 1234567.0, ... },
  ...
}
```

Extract and check with `jq` + Python:
```bash
NS=$(jq '.mean.point_estimate' target/criterion/cold_build/25_slides_all_formats/new/estimates.json)
MS=$(echo "scale=2; $NS / 1000000" | bc)
python3 -c "import sys; ms=float('${MS}'); sys.exit(0 if ms < 500 else 1)"
```

For relative regression detection: `cargo critcmp <baseline> <current> --threshold 10`
(critcmp reads the Criterion baseline JSON files and reports benches that regressed > 10%).

### Wall-clock vs CPU time

Criterion measures wall-clock time by default. For CI reproducibility, add:
```rust
.with_measurement(criterion::WallTime)
```
(This is the default, but explicit is better.)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Benchmark fixture fails to compile | `expect("fixture must compile")` panics during benchmark setup; Criterion reports the benchmark as failed; CI job fails |
| EC-002 | CI runner is slower than threshold (flaky gate) | Threshold is set with 20% headroom: NFR says < 500ms; CI gate checks < 500ms; if the runner is consistently > 400ms, investigate before tightening |
| EC-003 | `compile_incremental()` not yet available | Fall back to `compile()` with a comment; note that true incremental measurement requires STORY-056's cache API |
| EC-004 | Criterion html_reports feature availability | `html_reports` is OPT-IN (not the default); it is explicitly enabled in `Cargo.toml`. In CI, benchmarks are run with `--save-baseline`; HTML reports are generated locally only. Criterion 0.8 does not panic if the feature is enabled but reports are not viewed. |
| EC-005 | Platform variance: macOS benchmarks are not gated | Only `ubuntu-latest` runs are gated. macOS/Windows bench jobs run but threshold failures are non-blocking (different CPU characteristics) |
