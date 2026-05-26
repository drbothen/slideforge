---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-071
title: "Fuzz Harnesses + cargo-mutants Integration"
epic: EPIC-20
wave: 6
points: 8
priority: P0
tdd_mode: facade
status: draft
crate: slideforge-syntax
subsystems: [SS-01, SS-02]
target_module: slideforge-syntax
behavioral_contracts: []
# BC status: pending PO authorship — Wave 6 stories validate existing contracts
# via formal methods. BCs exercised: BC-1.01.001 (VP-014), BC-1.02.001 (VP-015).
# No new BCs introduced here.
verification_properties: [VP-014, VP-015]
nfr_refs: []
assumption_validations: []
risk_mitigations: []
depends_on:
  - STORY-005
  - STORY-006
  - STORY-007
  - STORY-008
  - STORY-009
  - STORY-010
  - STORY-011
  - STORY-012
  - STORY-013
  - STORY-014
  - STORY-066
  - STORY-067
blocks: []
estimated_days: 3
---

# STORY-071: Fuzz Harnesses + cargo-mutants Integration

## Summary

Deliver the complete cargo-fuzz and cargo-mutants integration for slideforge.
This story is the capstone of EPIC-20 (Phase 6 formal verification) and coordinates
with STORY-066 (syntax Kani proofs) and STORY-067 (eval Kani proofs) which each
established preliminary fuzz harness skeletons.

**Deliverables:**

1. **VP-014 fuzz harness (complete):** `fuzz/fuzz_targets/syntax_parse.rs` — the
   full production-ready fuzz target for `slideforge-syntax`. Any arbitrary byte input
   either produces a valid AST or a diagnostic list; never panics.

2. **VP-015 fuzz harness (complete):** `fuzz/fuzz_targets/eval_fuzz.rs` — the full
   production-ready fuzz target for `slideforge-eval`. Any structurally valid (via
   `Arbitrary`) AST either evaluates to a result or accumulates diagnostics; never panics.

3. **cargo-mutants integration:** `cargo-mutants` configured for the workspace with
   documented per-crate kill-rate budgets. CI `just mutants` target runs mutation
   testing with pass/fail threshold.

4. **`kani-all` and `fuzz-smoke` Justfile targets:** Convenience targets that run
   all Kani proofs across all crates (STORY-066, STORY-067, STORY-068, STORY-070)
   and run both fuzz harnesses for the 10-second smoke duration.

5. **`kani.yml` complete CI workflow:** The final, complete `.github/workflows/kani.yml`
   that integrates all Kani proof steps (from STORY-066 through STORY-070) plus
   fuzz smoke, plus nightly fuzz long-run.

**tdd_mode: facade** — combined scaffold+impl delivery. The fuzz harnesses and
mutants configuration are the product.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~5,000 |
| `fuzz/Cargo.toml` (complete) | ~1,000 |
| `fuzz/fuzz_targets/syntax_parse.rs` (complete) | ~1,500 |
| `fuzz/fuzz_targets/eval_fuzz.rs` (complete) | ~1,500 |
| `.cargo/mutants.toml` (cargo-mutants config) | ~1,000 |
| `Justfile` additions (kani-all, fuzz-smoke, mutants) | ~1,000 |
| `.github/workflows/kani.yml` (complete) | ~3,000 |
| `.github/workflows/nightly-fuzz.yml` (new) | ~1,500 |
| Referenced source (`syntax`, `eval`) | ~6,000 |
| **Total** | **~21,500** |

> 21,500 tokens ≈ 21% of a 100k-token context window. At the upper edge of budget.
> The CI YAML files are the primary source of token consumption.

## Acceptance Criteria

### AC-001: VP-014 fuzz harness survives 10-second smoke run without crash
The complete `fuzz/fuzz_targets/syntax_parse.rs` harness runs via
`cargo fuzz run syntax_parse -- -max_total_time=10` for 10 seconds on a Linux CI
runner without any crash, panic, or ASAN violation. Corpus: empty (libfuzzer
generates from scratch); or pre-seeded from `fuzz/corpus/syntax_parse/` if a
prior session accumulated inputs.
(traces to VP-014 — parser fuzz: any input terminates and produces errors or AST)

### AC-002: VP-015 fuzz harness survives 10-second smoke run without crash
The complete `fuzz/fuzz_targets/eval_fuzz.rs` harness runs via
`cargo fuzz run eval_fuzz -- -max_total_time=10` for 10 seconds without crash.
The harness uses `Arbitrary` to generate structurally valid `Deck` ASTs from
raw bytes and calls `evaluate()`.
(traces to VP-015 — eval fuzz: any valid AST terminates eval within time bound)

### AC-003: cargo-mutants configured with documented kill-rate budgets
`.cargo/mutants.toml` exists (or equivalent `mutants.toml` in workspace root) with:
- Per-crate kill-rate budgets documented in comments
- Exclusion rules for generated code, test modules, proof modules
- Timeout per mutant (default 60s, configurable)

The documented minimum kill-rate budgets are:
| Crate | Minimum Kill Rate |
|-------|-------------------|
| slideforge-syntax | ≥ 75% |
| slideforge-eval | ≥ 75% |
| slideforge-validate | ≥ 70% |
| slideforge-layout | ≥ 70% |
| slideforge-pptx | ≥ 65% |
| slideforge-pdf | ≥ 65% |
| slideforge-brand | ≥ 65% |

### AC-004: `just mutants` target runs and reports kill rate
`just mutants` runs `cargo mutants --workspace --jobs 4` and exits 1 if any
crate falls below its documented minimum kill rate. The Justfile target documents
the kill-rate requirements inline as comments.

### AC-005: `just fuzz-smoke` target runs both harnesses for 10 seconds each
`just fuzz-smoke` runs both `syntax_parse` and `eval_fuzz` harnesses sequentially
for 10 seconds each and exits 0 if neither crashes. Documents platform constraint
(Linux/macOS only for cargo-fuzz).

### AC-006: `just kani-all` target runs all proofs from EPIC-20
`just kani-all` calls `just kani-syntax`, `just kani-eval`, `just kani-validate`,
and `just kani-pdf` in sequence. Reports which crates passed/failed. Returns exit 0
only if all four pass.

### AC-007: Complete `.github/workflows/kani.yml` integrates all EPIC-20 Kani steps
The complete `kani.yml` workflow has the following jobs:
- `kani-syntax` (P1, non-blocking) — STORY-066
- `kani-eval` (P1, non-blocking) — STORY-067
- `kani-validate` (P0, blocking) — STORY-068
- `kani-pdf` (P0, blocking) — STORY-070
- `fuzz-smoke` (P1, non-blocking) — STORY-071

### AC-008: Nightly fuzz workflow runs 5-minute long-run
`.github/workflows/nightly-fuzz.yml` triggers on `schedule: "0 2 * * *"` (2am UTC).
Runs both fuzz targets for 300 seconds each. Uploads crash artifacts if found.
Non-blocking (does not block PR merge) but creates a GitHub issue on crash detection.

### AC-009: Fuzz corpus committed to repository
`fuzz/corpus/syntax_parse/` and `fuzz/corpus/eval_fuzz/` exist and contain at
minimum one seed input each (a minimal valid `.sf` document for syntax; a minimal
valid serialized `Deck` for eval). Seed inputs guide libfuzzer coverage from the
start rather than exploring from empty.

## Tasks

- [ ] 1. Read `fuzz/Cargo.toml` as created by STORY-066 and STORY-067 (skeleton)
- [ ] 2. Complete `fuzz/Cargo.toml`: add both binary targets, correct workspace paths,
         ASAN/UBSAN feature flags, correct Rust edition
- [ ] 3. Complete `fuzz/fuzz_targets/syntax_parse.rs` — expand STORY-066 skeleton to
         include UTF-8 handling, corpus hints, and ensure `no_main` attribute is correct
- [ ] 4. Complete `fuzz/fuzz_targets/eval_fuzz.rs` — expand STORY-067 skeleton to use
         `Arbitrary` on the full `Deck` type (not a subset)
- [ ] 5. Add `#[derive(arbitrary::Arbitrary)]` to `Deck`, `Slide`, `ContentBlock`,
         `Value` in `slideforge-types/src/` under `#[cfg(any(test, feature = "arbitrary"))]`
- [ ] 6. Add `arbitrary = { version = "=1.4", features = ["derive"], optional = true }`
         to `slideforge-types/Cargo.toml`; enable in fuzz crate dep
- [ ] 7. Create seed corpus files in `fuzz/corpus/syntax_parse/` and `fuzz/corpus/eval_fuzz/`
- [ ] 8. Install `cargo-mutants` and run against workspace to establish baseline kill rates
- [ ] 9. Create `.cargo/mutants.toml` (or `mutants.toml`) with exclusion rules and timeouts
- [ ] 10. Update `Justfile` with `kani-all`, `fuzz-smoke`, and `mutants` targets
- [ ] 11. Write complete `.github/workflows/kani.yml` integrating all four crate proof steps
- [ ] 12. Write `.github/workflows/nightly-fuzz.yml` with schedule trigger + artifact upload
- [ ] 13. Run `cargo fuzz build syntax_parse eval_fuzz` and verify both compile
- [ ] 14. Run `just fuzz-smoke` and verify 0 crashes in 10 seconds each
- [ ] 15. Run `cargo mutants -p slideforge-syntax --timeout 60` to verify kill rate ≥ 75%
- [ ] 16. Run `cargo mutants -p slideforge-eval --timeout 60` to verify kill rate ≥ 75%

## Previous Story Intelligence

STORY-071 is the integration story for EPIC-20. It depends on STORY-066 and STORY-067
having established fuzz target skeletons. Key lessons:

- STORY-066 established `fuzz/Cargo.toml` with a `syntax_parse` target. STORY-071
  completes it by adding `eval_fuzz` and ensuring the workspace configuration is correct.
- STORY-067 established the `eval_fuzz.rs` skeleton with `Arbitrary` usage. STORY-071
  completes the `Arbitrary` derive chain for `Deck` and related types.
- The `Justfile` has accumulated `kani-syntax`, `kani-eval`, `kani-validate`, and
  `kani-pdf` targets from prior stories. STORY-071 adds the `kani-all` wrapper.
- cargo-mutants baseline kill rates are unknown until STORY-071 runs the initial
  assessment. The kill-rate budgets in AC-003 are targets — if the baseline is below
  a target, the story must add tests (not lower the target).

## Architecture Compliance Rules

Derived from `architecture/verification-architecture.md`:

1. **cargo-fuzz is Linux/macOS only:** `cargo fuzz run` requires Linux or macOS.
   The CI step must be gated: `if: runner.os != 'Windows'`. Windows CI skips fuzz.
2. **cargo-mutants is all platforms:** Mutation testing runs on all 5 platforms.
   The `just mutants` target has no platform gate.
3. **`Arbitrary` must be optional:** The `arbitrary` feature in `slideforge-types`
   must be optional (not enabled in production builds). Gate with
   `#[cfg(any(test, feature = "arbitrary"))]`.
4. **Fuzz targets never call `unwrap()` in the harness itself:** The harness must
   treat any result as valid. `unwrap()` in the harness body would cause a false
   crash. Use `let _ = result;` or `match result { ... }` patterns.
5. **Mutants exclusion rules:** Exclude from mutation testing:
   - `src/proofs/` modules (these are verification code, not production code)
   - `#[cfg(test)]` modules
   - Generated code (if any)
   - `fn main()` in CLI binaries (integration tests cover, not unit mutation)

## Library and Framework Requirements

| Library | Pinned Version | Role | Notes |
|---------|---------------|------|-------|
| libfuzzer-sys | =0.4.10 | Fuzz harness runtime | `[dependencies]` in `fuzz/Cargo.toml` |
| arbitrary | =1.4 | Fuzz input generation | Optional dep in slideforge-types; dep in fuzz crate |
| cargo-mutants | latest | Mutation testing tool | Installed as tool: `cargo install cargo-mutants` |
| cargo-fuzz | latest | Fuzz harness management | Installed as tool: `cargo install cargo-fuzz` |

Note: `cargo-mutants` and `cargo-fuzz` are installed as tools on the CI runner, not
as crate dependencies. They are not pinned in `Cargo.toml`.

## File Structure Requirements

Files to CREATE:
- `fuzz/corpus/syntax_parse/seed-001` — minimal valid .sf document byte seed
- `fuzz/corpus/eval_fuzz/seed-001` — minimal valid Deck serialized bytes seed
- `.cargo/mutants.toml` — cargo-mutants configuration
- `.github/workflows/nightly-fuzz.yml` — nightly long-run fuzz workflow

Files to COMPLETE (from skeletons in STORY-066/067):
- `fuzz/Cargo.toml` — add eval_fuzz target, fix any skeleton gaps
- `fuzz/fuzz_targets/syntax_parse.rs` — complete from STORY-066 skeleton
- `fuzz/fuzz_targets/eval_fuzz.rs` — complete from STORY-067 skeleton

Files to MODIFY:
- `slideforge-types/Cargo.toml` — add `arbitrary` optional dep
- `slideforge-types/src/lib.rs` (or types module) — add `#[derive(Arbitrary)]`
  under feature gate for `Deck`, `Slide`, `ContentBlock`, `Value`
- `Justfile` — add `kani-all`, `fuzz-smoke`, `mutants` targets
- `.github/workflows/kani.yml` — complete and finalize

Files to NOT touch:
- Any production source files other than the `Arbitrary` derive additions

## Implementation Notes

### Complete `fuzz/Cargo.toml`

```toml
[package]
name = "slideforge-fuzz"
version = "0.0.0"
publish = false
edition = "2024"

[package.metadata]
cargo-fuzz = true

[dependencies]
libfuzzer-sys = "=0.4.10"
arbitrary = { version = "=1.4", features = ["derive"] }

# Workspace members to fuzz
slideforge-syntax = { path = "../crates/slideforge-syntax" }
slideforge-eval = { path = "../crates/slideforge-eval" }
slideforge-types = { path = "../crates/slideforge-types", features = ["arbitrary"] }

[[bin]]
name = "syntax_parse"
path = "fuzz_targets/syntax_parse.rs"
test = false
doc = false

[[bin]]
name = "eval_fuzz"
path = "fuzz_targets/eval_fuzz.rs"
test = false
doc = false
```

### Complete `fuzz/fuzz_targets/syntax_parse.rs`

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use slideforge_syntax::parse;

fuzz_target!(|data: &[u8]| {
    // Invariant: parse must never panic on any input.
    // It must return either Ok(ast) or Err(diagnostics).
    //
    // Strategy A: treat input as UTF-8 text
    if let Ok(s) = std::str::from_utf8(data) {
        let result = parse(s);
        // Both Ok and Err are valid results; panic is the only failure.
        let _ = result;
    }
    // Strategy B: non-UTF-8 input — parse must also not panic when given
    // a byte slice that happens not to be valid UTF-8. The pre-parse UTF-8
    // validation in the lexer must return a graceful error.
    // (The outer harness catches any panic via libFuzzer's ASAN integration.)
});
```

### Complete `fuzz/fuzz_targets/eval_fuzz.rs`

```rust
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slideforge_eval::{evaluate, Scope};
use slideforge_types::Deck;

fuzz_target!(|data: &[u8]| {
    // Generate a structurally valid Deck from arbitrary bytes.
    // The Arbitrary impl constrains the deck to structurally valid shapes.
    let mut unstructured = arbitrary::Unstructured::new(data);
    if let Ok(deck) = Deck::arbitrary_take_rest(&mut unstructured) {
        // Invariant: evaluate must never panic on any structurally valid Deck.
        let _ = evaluate(deck, &Scope::empty());
    }
    // If Arbitrary fails (insufficient bytes), the harness returns without
    // calling evaluate — this is expected for very short inputs.
});
```

### `.cargo/mutants.toml` Configuration

```toml
# cargo-mutants configuration for slideforge workspace
# Run: cargo mutants --workspace --timeout 60 --jobs 4
# Kill-rate budgets (documented minimum per crate):
#   slideforge-syntax:   75%
#   slideforge-eval:     75%
#   slideforge-validate: 70%
#   slideforge-layout:   70%
#   slideforge-pptx:     65%
#   slideforge-pdf:      65%
#   slideforge-brand:    65%

[defaults]
timeout_multiplier = 2.0  # 2× test suite runtime per mutant
jobs = 4

# Exclude proof modules (verification code, not production logic)
exclude_re = [
    "src/proofs/.*",
    "#\\[cfg\\(kani\\)\\]",
    "#\\[cfg\\(test\\)\\]",
]

# Exclude trivially non-killable patterns
exclude_methods = [
    "fmt",
    "clone",
    "default",
]
```

### Justfile Targets (complete additions)

```makefile
# Run ALL Kani proofs across all EPIC-20 crates (Linux/macOS only)
kani-all:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running all Kani proofs..."
    just kani-syntax && echo "  [PASS] syntax" || echo "  [FAIL] syntax"
    just kani-eval   && echo "  [PASS] eval"   || echo "  [FAIL] eval"
    just kani-validate && echo "  [PASS] validate (P0)" || { echo "  [FAIL] validate (P0) — BLOCKING"; exit 1; }
    just kani-pdf    && echo "  [PASS] pdf (P0)"     || { echo "  [FAIL] pdf (P0) — BLOCKING"; exit 1; }
    echo "All P0 proofs passed."

# Fuzz smoke: 10 seconds each harness (Linux/macOS only)
fuzz-smoke:
    # Platform: Linux/macOS only. cargo-fuzz requires nightly for full ASAN.
    cargo fuzz run syntax_parse -- -max_total_time=10 -print_final_stats=1
    cargo fuzz run eval_fuzz    -- -max_total_time=10 -print_final_stats=1

# Mutation testing (all platforms)
# Kill-rate budgets:
#   syntax:   >=75%  eval:     >=75%
#   validate: >=70%  layout:   >=70%
#   pptx:     >=65%  pdf:      >=65%
#   brand:    >=65%
mutants:
    cargo mutants --workspace --timeout 60 --jobs 4 \
        --output .mutants-out/
    @echo "Review .mutants-out/outcomes.json for per-crate kill rates"
    @# CI threshold check script:
    python3 scripts/check-mutants-threshold.py .mutants-out/outcomes.json

# Per-crate fuzz convenience targets (use for targeted investigation)
fuzz-syntax time="10":
    cargo fuzz run syntax_parse -- -max_total_time={{time}}

fuzz-eval time="10":
    cargo fuzz run eval_fuzz -- -max_total_time={{time}}
```

### Complete `.github/workflows/kani.yml`

```yaml
name: Formal Verification (Kani + Fuzz)

on:
  push:
    branches: [main, develop]
  pull_request:
    paths:
      - 'crates/**'
      - 'fuzz/**'
      - '.github/workflows/kani.yml'

jobs:
  kani-validate:
    name: "Kani P0 — slideforge-validate"
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust (stable)
        uses: dtolnay/rust-toolchain@stable
      - name: Install Kani
        run: |
          cargo install kani-verifier --locked
          cargo kani setup
      - name: Run kani-validate (P0 gate)
        run: just kani-validate

  kani-pdf:
    name: "Kani P0 — slideforge-pdf"
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust (stable)
        uses: dtolnay/rust-toolchain@stable
      - name: Install Kani
        run: |
          cargo install kani-verifier --locked
          cargo kani setup
      - name: Run kani-pdf (P0 gate)
        run: just kani-pdf

  kani-syntax:
    name: "Kani P1 — slideforge-syntax"
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust (stable)
        uses: dtolnay/rust-toolchain@stable
      - name: Install Kani
        run: |
          cargo install kani-verifier --locked
          cargo kani setup
      - name: Run kani-syntax (P1 non-blocking)
        run: just kani-syntax

  kani-eval:
    name: "Kani P1 — slideforge-eval"
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust (stable)
        uses: dtolnay/rust-toolchain@stable
      - name: Install Kani
        run: |
          cargo install kani-verifier --locked
          cargo kani setup
      - name: Run kani-eval (P1 non-blocking)
        run: just kani-eval

  fuzz-smoke:
    name: "Fuzz Smoke (10s each)"
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust (nightly — required for cargo-fuzz)
        uses: dtolnay/rust-toolchain@nightly
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      - name: Run fuzz smoke
        run: just fuzz-smoke
```

### Nightly Fuzz Workflow

```yaml
# .github/workflows/nightly-fuzz.yml
name: Nightly Fuzz (5min)

on:
  schedule:
    - cron: '0 2 * * *'  # 2am UTC nightly
  workflow_dispatch:      # allow manual trigger

jobs:
  fuzz-nightly:
    name: "Fuzz Long-Run (300s)"
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust (nightly)
        uses: dtolnay/rust-toolchain@nightly
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      - name: Run fuzz long-run — syntax
        run: cargo fuzz run syntax_parse -- -max_total_time=300 -print_final_stats=1
      - name: Run fuzz long-run — eval
        run: cargo fuzz run eval_fuzz -- -max_total_time=300 -print_final_stats=1
      - name: Upload crash artifacts (if any)
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: fuzz-crashes-${{ github.run_id }}
          path: |
            fuzz/artifacts/
            fuzz/corpus/
```

### Kill-Rate Threshold Check Script

```python
# scripts/check-mutants-threshold.py
"""Check cargo-mutants outcomes against documented kill-rate budgets."""
import json
import sys

THRESHOLDS = {
    "slideforge-syntax":   0.75,
    "slideforge-eval":     0.75,
    "slideforge-validate": 0.70,
    "slideforge-layout":   0.70,
    "slideforge-pptx":     0.65,
    "slideforge-pdf":      0.65,
    "slideforge-brand":    0.65,
}

outcomes_path = sys.argv[1]
with open(outcomes_path) as f:
    outcomes = json.load(f)

failures = []
for crate, threshold in THRESHOLDS.items():
    crate_data = outcomes.get(crate, {})
    killed = crate_data.get("killed", 0)
    total = crate_data.get("total", 0)
    if total == 0:
        continue  # no mutants generated (crate may be excluded)
    rate = killed / total
    if rate < threshold:
        failures.append(f"  {crate}: {rate:.1%} (minimum {threshold:.0%})")

if failures:
    print("FAIL — Kill rate below threshold for:")
    for f in failures:
        print(f)
    sys.exit(1)
else:
    print("PASS — All crate kill rates meet budget.")
    sys.exit(0)
```

## Dependencies

### Dependency Justification

- STORY-071 depends on STORY-005 through STORY-010 because VP-014 fuzzes the
  complete `slideforge-syntax` parser. All six Wave 1 parser stories must be complete
  for the harness to have meaningful coverage.
- STORY-071 depends on STORY-011 through STORY-014 because VP-015 fuzzes the
  complete `slideforge-eval` evaluator. All four Wave 2 evaluator stories must
  be complete.
- STORY-071 depends on STORY-066 because that story established the `fuzz/Cargo.toml`
  skeleton and `syntax_parse.rs` skeleton that STORY-071 completes.
- STORY-071 depends on STORY-067 because that story established the `eval_fuzz.rs`
  skeleton that STORY-071 completes.
- STORY-071 does not block any other story — it is the terminal deliverable of EPIC-20.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty byte slice to syntax_parse fuzz — VP-014 | `std::str::from_utf8(b"")` succeeds; `parse("")` returns empty AST or diagnostic |
| EC-002 | Single null byte `\x00` to syntax_parse — VP-014 | UTF-8 conversion fails; harness skips the call; no panic |
| EC-003 | Valid UTF-8 slideforge source to syntax_parse — VP-014 | Returns `Ok(ast)` with at least the metadata node |
| EC-004 | 65535-byte garbage to eval_fuzz — VP-015 | `Arbitrary::arbitrary_take_rest` exhausts input; harness returns without eval call |
| EC-005 | Deck with 1000 slides from Arbitrary — VP-015 | evaluate() applies @for termination guard; returns bounded output or Err |
| EC-006 | cargo-mutants generates mutant that changes SLIDE_HEIGHT_EMU | Kill: coordinate mapping tests detect the wrong constant |
| EC-007 | cargo-mutants generates mutant removing `kani::assert` in proof | Excluded by `exclude_re = ["src/proofs/.*"]` — proofs are not mutated |
| EC-008 | Fuzz corpus seed file is invalid for the harness | libfuzzer handles gracefully — skips or reports minimal repro without crash |
| EC-009 | nightly-fuzz workflow finds a crash | Artifacts uploaded; GitHub issue created via `actions/github-script`; next merge gate blocked until fixed |
| EC-010 | kill-rate threshold script receives crate with 0 mutants | Script skips crate (no total mutants); threshold check passes by default |

## Test Strategy

- **VP-014, VP-015:** cargo-fuzz harnesses. Smoke: 10s per PR. Long-run: 300s nightly.
  Zero crashes is the pass criterion.
- **cargo-mutants:** Runs on all platforms. Kill rates reported per crate. Threshold
  check script exits 1 if any crate falls below its documented budget.
- **Regression:** Any new crash found by fuzz is a blocker. Crashes are saved as
  artifacts, added to the corpus as regression inputs, and must be fixed before
  the next merge.
- **Wave 6 gate:** The full wave gate requires: all Kani P0 proofs pass + fuzz 24h
  run without crash + cargo-mutants kill rates meet budget.

---

*Subsystem anchor justifications:*
*SS-01 (DSL Parser): owns VP-014 — slideforge-syntax is the fuzz target, per ARCH-INDEX.*
*SS-02 (Evaluator): owns VP-015 — slideforge-eval is the fuzz target, per ARCH-INDEX.*
*STORY-071 spans both subsystems because it delivers the integrated fuzz+mutants*
*infrastructure for the entire EPIC-20 formal verification suite.*
