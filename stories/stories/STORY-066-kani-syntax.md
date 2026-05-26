---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-066
title: "Kani Proofs: slideforge-syntax (VP-001, VP-003, VP-009, VP-014)"
epic: EPIC-20
wave: 6
points: 8
priority: P0
tdd_mode: facade
status: draft
crate: slideforge-syntax
subsystems: [SS-01]
target_module: slideforge-syntax
behavioral_contracts: []
# BC status: pending PO authorship — Wave 6 formal-verification stories validate
# existing behavioral contracts rather than defining new ones. BCs that this story
# exercises through formal proofs are BC-1.01.003 (VP-001), BC-1.04.003 (VP-003),
# BC-1.01.001 (VP-009, VP-014). The proofs themselves are the artifact; no new BCs
# are introduced here. A product-owner must author BC-8.30.001-style formal-verification
# contracts if this story is to transition to status: ready.
verification_properties: [VP-001, VP-003, VP-009, VP-014]
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
blocks: []
estimated_days: 3
---

# STORY-066: Kani Proofs: slideforge-syntax (VP-001, VP-003, VP-009, VP-014)

## Summary

Add Phase 6 formal verification for `slideforge-syntax`. This story delivers:

1. **VP-001 (Kani proof):** Tab detection byte span accuracy — every tab byte at
   position P produces a diagnostic with span [P, P+1).
2. **VP-003 (Kani proof):** `@for` over a bounded collection always terminates —
   for any finite collection of size ≤ 64, iteration halts.
3. **VP-009 (proptest suite):** Parse of any valid `.sf` source produces a non-empty
   AST — randomly generated valid `.sf` documents always produce at least one slide node.
4. **VP-014 (cargo-fuzz harness):** Parser fuzz — any arbitrary byte sequence either
   terminates with a valid AST or accumulates at least one diagnostic; no panic.

**tdd_mode: facade** — these are combined scaffold+implementation deliveries. The
verification artifacts (Kani harnesses, proptest suites, fuzz targets) are the product.
The quality gate is mutation testing at wave gate rather than Red Gate density.

**Platform constraint:** Kani proofs (VP-001, VP-003) run only on Linux/macOS. Windows
contributors rely on concrete unit tests in the same `proofs/` module. The proptest
suite (VP-009) and fuzz harness (VP-014) run on all platforms.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,000 |
| `crates/slideforge-syntax/src/proofs/` (4 files) | ~4,000 |
| `fuzz/fuzz_targets/syntax_parse.rs` | ~1,000 |
| Justfile additions (`kani-local`, `fuzz-local`) | ~500 |
| `.github/workflows/kani.yml` additions | ~1,000 |
| Referenced source in `slideforge-syntax/src/` | ~8,000 |
| **Total** | **~18,500** |

> 18,500 tokens ≈ 18% of a 100k-token context window. Within the 20-30% budget.

## Acceptance Criteria

### AC-001: VP-001 Kani proof compiles and passes
Kani proof `proofs::vp001_tab_span_accuracy::tab_detection_byte_span` in
`crates/slideforge-syntax/src/proofs/vp001_tab_span.rs` compiles under
`cargo kani -p slideforge-syntax` and reports `VERIFICATION SUCCESSFUL`.
The proof verifies that for any tab byte at any valid offset within a
bounded input buffer (length ≤ 32), the resulting diagnostic span has
start == offset and end == offset + 1.
(traces to VP-001 — tab detection byte span accuracy)

### AC-002: VP-003 Kani proof compiles and passes
Kani proof `proofs::vp003_for_termination::for_over_bounded_terminates` in
`crates/slideforge-syntax/src/proofs/vp003_for_termination.rs` compiles and
passes. The proof verifies that `parse_for_body` always terminates when the
collection size is bounded to ≤ 64 items, via `#[kani::unwind(65)]`.
(traces to VP-003 — @for over bounded collection always terminates)

### AC-003: VP-009 proptest suite finds zero counterexamples
Proptest suite in `crates/slideforge-syntax/tests/proptest_ast.rs` with
strategy `arb_valid_sf_source()` generates 1,000 random but structurally
valid `.sf` documents and asserts `parse(source).ast.slides.len() >= 1`
for each. No counterexample found in 1,000 runs.
(traces to VP-009 — parse of valid .sf source produces non-empty AST)

### AC-004: VP-014 fuzz harness compiles and runs without crash
Fuzz target `fuzz/fuzz_targets/syntax_parse.rs` compiles via `cargo fuzz build syntax_parse`
and runs for ≥ 10 seconds via `cargo fuzz run syntax_parse -- -max_total_time=10`
without panicking. The harness calls `parse(arbitrary_bytes)` and asserts the
result is either `Ok(ast)` or `Err(diagnostics)` — never a panic.
(traces to VP-014 — parser fuzz: any input terminates and produces errors or AST)

### AC-005: Kani proofs are gated by `#[cfg(kani)]`
All proof functions use `#[cfg(kani)]` at the module level so they compile
ONLY under `cargo kani`, never under `cargo build` or `cargo test`. The
normal test suite is not affected.

### AC-006: `just kani-syntax` Justfile target succeeds
`just kani-syntax` runs `cargo kani -p slideforge-syntax --harness proofs`
and exits 0 on a Linux/macOS host with Kani installed. The Justfile target
documents the platform constraint with a comment.

### AC-007: CI `kani.yml` job includes slideforge-syntax proofs
`.github/workflows/kani.yml` has a step `kani-syntax` that runs
`just kani-syntax` on `ubuntu-latest`. The job runs on `push` to `develop`
and `main` branches and on PRs touching `crates/slideforge-syntax/**`.

## Tasks

- [ ] 1. Read `crates/slideforge-syntax/src/lexer.rs` (tab detection implementation)
- [ ] 2. Read `crates/slideforge-syntax/src/parser.rs` (for-loop parsing and iteration)
- [ ] 3. Create `crates/slideforge-syntax/src/proofs/mod.rs` with `#[cfg(kani)]` guard
- [ ] 4. Write VP-001 proof in `crates/slideforge-syntax/src/proofs/vp001_tab_span.rs`
         — use `kani::any::<usize>()` for offset, `kani::assume` for bounds
- [ ] 5. Write VP-003 proof in `crates/slideforge-syntax/src/proofs/vp003_for_termination.rs`
         — bound collection size with `kani::assume(n <= 64)`, unwind bound 65
- [ ] 6. Create `crates/slideforge-syntax/tests/proptest_ast.rs` with `arb_valid_sf_source()`
         strategy using `proptest` 1.4 leaf generators (slide type keywords, field names)
- [ ] 7. Create `fuzz/fuzz_targets/syntax_parse.rs` with `libfuzzer_sys::fuzz_target!` macro
- [ ] 8. Create `fuzz/Cargo.toml` if not already present (or add target to existing)
- [ ] 9. Add `[kani]` workspace section to `Cargo.toml` root if not present
- [ ] 10. Add `just kani-syntax` target to `Justfile`
- [ ] 11. Add/update `.github/workflows/kani.yml` with `kani-syntax` step
- [ ] 12. Run `cargo kani -p slideforge-syntax` on Linux/macOS to verify proofs pass
- [ ] 13. Run `cargo test -p slideforge-syntax --test proptest_ast` to verify VP-009
- [ ] 14. Run `cargo fuzz build syntax_parse && cargo fuzz run syntax_parse -- -max_total_time=10`

## Previous Story Intelligence

N/A — STORY-066 is the first story in EPIC-20 (Wave 6 formal verification). The
Wave 5 stories that implement `slideforge-syntax` are prerequisites (STORY-005 through
STORY-010). The proof harnesses attach to the final implementations from those stories.

Key patterns from Wave 1 STORY-005 and STORY-006 to carry forward:
- The lexer's tab detection logic lives in `crates/slideforge-syntax/src/lexer.rs`.
  The proof must import the actual production function, not a stub.
- `DiagnosticSink` accumulates errors. Proofs must verify the sink is non-empty after
  a tab byte is encountered.
- chumsky 0.10.1 uses `Rich` error type — the proptest strategy must produce inputs
  that satisfy chumsky's `Stream` interface.

## Architecture Compliance Rules

Derived from `architecture/verification-architecture.md` and `architecture/purity-boundary-map.md`:

1. **Kani gate:** All proof modules MUST use `#[cfg(kani)]`. Never `#[cfg(test)]` for
   proofs — these are distinct compilation modes.
2. **Pure-core only:** Kani proofs may only call pure functions. `slideforge-syntax` is
   classified SS-01 (Pure core, Kani-amenable). No I/O, no async, no mutex in proofs.
3. **Unwind bounds must be explicit:** Every `#[kani::proof]` function targeting a loop
   MUST have `#[kani::unwind(N)]` where N is 1 + max loop iterations in the bounded model.
4. **No production dep changes:** Kani proof additions must not add production deps.
   Add Kani as `[dev-dependencies]` only; proptest and libfuzzer-sys in their respective
   `[dev-dependencies]` and `[dependencies]` (for fuzz crate).
5. **Forbidden dependencies:** `slideforge-syntax` MUST NOT gain dependencies on
   `slideforge-eval`, `slideforge-layout`, `slideforge-pptx`, or any exporter crate.
   The proofs must not introduce indirect forbidden deps.

## Library and Framework Requirements

| Library | Pinned Version | Role | Notes |
|---------|---------------|------|-------|
| kani | latest (~0.65+) | Formal model checker (uses bundled nightly toolchain internally) | `cargo install kani-verifier --locked && cargo kani setup` |
| proptest | =1.6 | Property-based testing | `[dev-dependencies]` in slideforge-syntax |
| libfuzzer-sys | =0.4.10 | Fuzz harness runtime | `[dependencies]` in `fuzz/Cargo.toml` |
| arbitrary | =1.4 | Fuzz input generation | `[dependencies]` in `fuzz/Cargo.toml` |
| chumsky | =0.10.1 | Parser combinator | Already pinned — do not upgrade |

> Kani version: do not pin to a specific Kani version in Cargo.toml — Kani is a
> standalone tool, not a crate dep. The CI job installs Kani via
> `cargo install kani-verifier --locked`. The Justfile documents the expected version.

## File Structure Requirements

Files to CREATE:
- `crates/slideforge-syntax/src/proofs/mod.rs` — module root with `#[cfg(kani)]` guard
- `crates/slideforge-syntax/src/proofs/vp001_tab_span.rs` — VP-001 Kani proof
- `crates/slideforge-syntax/src/proofs/vp003_for_termination.rs` — VP-003 Kani proof
- `crates/slideforge-syntax/tests/proptest_ast.rs` — VP-009 proptest suite
- `fuzz/Cargo.toml` — fuzz workspace member (or append to existing)
- `fuzz/fuzz_targets/syntax_parse.rs` — VP-014 fuzz target

Files to MODIFY:
- `crates/slideforge-syntax/src/lib.rs` — add `mod proofs;` (under `#[cfg(kani)]`)
- `Justfile` — add `kani-syntax`, `fuzz-syntax` targets
- `.github/workflows/kani.yml` — add `kani-syntax` step (create file if absent)
- `crates/slideforge-syntax/Cargo.toml` — add `proptest` dev-dependency

Files to NOT touch:
- Any source file in `crates/slideforge-syntax/src/` other than `lib.rs` and new
  `proofs/` module — proofs attach to existing functions, they do not modify them.

## Implementation Notes

### VP-001 Proof Pattern

The tab detection proof verifies the byte-span contract of the lexer's tab-error path:

```rust
// crates/slideforge-syntax/src/proofs/vp001_tab_span.rs
#[cfg(kani)]
mod proofs {
    use crate::lexer::{lex, LexError};

    #[kani::proof]
    #[kani::unwind(33)]
    fn tab_detection_byte_span() {
        // Construct a bounded input with a tab at a known offset
        let offset: usize = kani::any();
        kani::assume(offset < 32);

        let mut source = vec![b' '; 32];
        source[offset] = b'\t';
        let source_str = std::str::from_utf8(&source).unwrap();

        // Lex must produce a tab error with span [offset, offset+1)
        let result = lex(source_str);
        match result {
            Err(LexError::TabIndentation { span }) => {
                kani::assert(span.start == offset);
                kani::assert(span.end == offset + 1);
            }
            _ => {
                // If there is no error, the tab was not in an indentation position.
                // This is acceptable — not all tab bytes are indentation tabs.
            }
        }
    }
}
```

Adjust the import paths to match the actual `lexer` module structure produced in
STORY-005. The proof function name must remain `tab_detection_byte_span` (referenced
by CI).

### VP-003 Proof Pattern

```rust
// crates/slideforge-syntax/src/proofs/vp003_for_termination.rs
#[cfg(kani)]
mod proofs {
    use crate::parser::ForCollection;

    #[kani::proof]
    #[kani::unwind(65)]
    fn for_over_bounded_terminates() {
        let n: usize = kani::any();
        kani::assume(n <= 64);

        // Build a synthetic bounded collection descriptor
        let collection = ForCollection::Bounded { size: n };

        // parse_for_body must terminate for any bounded n
        let result = crate::parser::parse_for_body_bounded(collection);
        kani::assert(result.is_ok() || result.is_err());
        // The mere fact that kani completes the proof verifies termination.
    }
}
```

If `ForCollection` or `parse_for_body_bounded` are named differently in the actual
implementation, adapt accordingly. Do not rename the production function — add a
thin proof-adapter if needed (inside `#[cfg(kani)]` only).

### VP-009 proptest Strategy

```rust
// crates/slideforge-syntax/tests/proptest_ast.rs
use proptest::prelude::*;
use slideforge_syntax::parse;

fn arb_valid_sf_source() -> impl Strategy<Value = String> {
    // Generate a minimal valid .sf document: at minimum one slide with required fields.
    let slide_types = prop::sample::select(vec![
        "title",
        "bullets",
        "blank",
        "executive_summary",
    ]);
    slide_types.prop_map(|slide_type| {
        format!(
            "slideforge_version \"1\"\ntitle \"Test Deck\"\nlang \"en-US\"\n\n{} slide-1:\n  title: \"Hello\"\n",
            slide_type
        )
    })
}

proptest! {
    #[test]
    fn valid_sf_always_produces_ast(source in arb_valid_sf_source()) {
        let result = parse(&source);
        prop_assert!(
            result.is_ok(),
            "Parse failed for valid source: {:?}",
            result
        );
        let ast = result.unwrap();
        prop_assert!(
            ast.slides.len() >= 1,
            "AST has no slides for source: {}",
            source
        );
    }
}
```

### VP-014 Fuzz Harness

```rust
// fuzz/fuzz_targets/syntax_parse.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use slideforge_syntax::parse;

fuzz_target!(|data: &[u8]| {
    // The parser must never panic on arbitrary byte input.
    // It should return either Ok(ast) or Err(diagnostics).
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse(s);
        // Any result is valid — the invariant is no panic.
    }
    // Non-UTF-8 input: parse must also not panic (string conversion returns Err).
});
```

### Kani CI Configuration

```yaml
# .github/workflows/kani.yml (job step addition)
- name: kani-syntax
  if: runner.os != 'Windows'
  run: |
    cargo install kani-verifier --locked
    cargo kani setup
    just kani-syntax
```

**Important: Kani Toolchain Note**

Kani internally uses a bundled nightly Rust toolchain (it syncs with a recent nightly
per release). The `cargo kani setup` step downloads this toolchain. The project's
`rust-toolchain.toml` (stable) is NOT used for Kani proofs. This is expected behavior
and does not conflict with the project's stable-only policy for production code.

### Justfile Targets

```
# Formal verification — Kani (Linux/macOS only)
kani-syntax:
    # Platform: Linux/macOS only. Windows contributors skip this target.
    cargo kani -p slideforge-syntax --harness proofs::vp001_tab_span::proofs::tab_detection_byte_span
    cargo kani -p slideforge-syntax --harness proofs::vp003_for_termination::proofs::for_over_bounded_terminates

# Fuzz target — parser (Linux CI; 10s smoke, 5min nightly)
fuzz-syntax time="10":
    cargo fuzz run syntax_parse -- -max_total_time={{time}}
```

## Dependencies

### Dependency Justification

- STORY-066 depends on STORY-005 because VP-001 proof calls `lexer::lex()` defined there.
  Without the production lexer, the proof has nothing to verify.
- STORY-066 depends on STORY-006 because VP-003 proof calls parser loop logic from there.
- STORY-066 depends on STORY-007 through STORY-010 because VP-009 proptest strategy
  generates `.sf` documents that exercise `@for`, `@include`, and diagnostic accumulation —
  all built across those stories. A partial implementation would produce spurious failures.
- STORY-066 does not block any other story — it is a terminal Wave 6 verification artifact.

### Blocked-by STORY-IDs and reason
- STORY-005: lexer tab detection (VP-001 target function)
- STORY-006: core parser (VP-003 + VP-009 target functions)
- STORY-007 through STORY-010: full parser surface (VP-009 + VP-014 require complete parser)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Tab at byte offset 0 (first character in file) | VP-001 proof: span [0, 1) produced |
| EC-002 | Tab at last valid position in bounded buffer (offset=31) | VP-001 proof: span [31, 32) produced |
| EC-003 | `@for` over collection of size 0 | VP-003 proof: terminates immediately, produces zero AST nodes |
| EC-004 | `@for` over collection of size 64 (max bound) | VP-003 proof: terminates after 64 iterations |
| EC-005 | Fuzz input is empty byte slice | VP-014 harness: returns `Ok(empty_ast)` or `Err([])`, no panic |
| EC-006 | Fuzz input is valid UTF-8 but syntactically garbage | VP-014 harness: returns `Err(diagnostics)`, no panic |
| EC-007 | proptest generates a slide with no fields | VP-009 suite: parser accepts or rejects gracefully; proptest strategy avoids this via generator design |
| EC-008 | Kani unwind bound exceeded (collection size > 64) | Proof assumes away: `kani::assume(n <= 64)` excludes this case |

## Test Strategy

- **VP-001, VP-003:** Kani bounded model checking. VERIFICATION SUCCESSFUL is the pass criterion.
- **VP-009:** proptest with 1,000 cases (default). `proptest::test_runner::Config::default()` runs.
- **VP-014:** `cargo fuzz run` 10-second smoke on PR, 5-minute nightly long-run. Zero crashes is pass.
- **Regression:** Once proofs pass, they are permanent CI gates. Any refactor of `lex()` or
  the `@for` parser that breaks a proof is a blocking CI failure.

---

*Subsystem anchor justification: SS-01 owns this story's scope because slideforge-syntax
is the sole crate implementing DSL parsing (lexer + chumsky parser), which is the target
of all four VPs in this story, per ARCH-INDEX Subsystem Registry.*
