# PR #60 Review — STORY-049 Plugin Registry Assembly / composition root

**Verdict: APPROVE**

Independent fresh-eyes review (different reviewer context). I reviewed every changed
file in `develop...HEAD`, re-ran the canonical gates locally in the worktree, and
verified AC coverage, contract fidelity, and the panic-safety perimeter against the
actual code rather than relying on the prior 12-pass LOCAL adversary cascade.

## What I verified independently (not rubber-stamped)

| Check | Method | Result |
|-------|--------|--------|
| Workspace/crate build | `cargo build -p slideforge` | Clean |
| Clippy pedantic | `cargo clippy -p slideforge --all-targets -- -D warnings` | Clean (0 warnings) |
| Crate tests | `cargo nextest run -p slideforge --no-fail-fast` | 38/38 PASS (incl. 3 external_plugin_test) |
| Headline E2E | `test_crit2_end_to_end_build_ok_with_toml_brand` | PASS — produces non-empty `PK`-prefixed PPTX |
| Panic perimeter | `bash scripts/check-panic-profile.sh` | PASS (release + dist = unwind; no abort in any shipped profile) |
| Dependency PRs merged | `git log develop` grep | All of STORY-002/037-045/083/084/085 merged (#41-#59) |
| PR mergeability | `gh pr view 60` | OPEN, base develop, MERGEABLE |

## AC coverage (all 8 satisfied with load-bearing tests)

- **AC-001 / AC-004** — `register_bundled_plugins` + `default_registry()` → `surface_count()==10`; `surface_names()` equals canonical `SURFACE_NAMES`. Real assertions, not paper-fixes. PASS.
- **AC-002** — empty `PluginRegistryBuilder::default().build()` → `Err(RegistryError::MissingSurface)`; partial builder reports `"Exporter"` as first missing. PASS.
- **AC-003** — `cargo build --workspace` gate; verified clean locally. PASS.
- **AC-005 / AC-006** — dep-isolation via `cargo tree` + clippy `-D warnings` (visibility violations promoted to errors). Clean. PASS.
- **AC-007** — `external_plugin_test.rs` `TestDataSource` imports ONLY `slideforge` (re-export facade) + `slideforge-plugin-api` + `slideforge-types`, registers alongside bundled plugins, and is callable. Compile-time trait-bound assertion present. This is genuine dog-fooding via `Box<dyn Trait>`. PASS.
- **AC-008** — `dispatch.rs` `catch_unwind` boundary is load-bearing: wired into every `BrandProvider::load`, `Validator::validate`, and `Exporter::export` call in `build_inner` (lib.rs lines 378, 451, 529). 5 panic tests cover `&str`/`String`/no-panic/plugin_name/exporter-through-build paths. PASS.

## Contract fidelity

- **BC-5.02.001 v1.4** — all 10 surfaces assembled via the public `register_*` API; `MissingSurface` is a typed error, not a panic. Confirmed.
- **BC-5.02.002 v1.4** — every registration uses `Box::new(...)` into a `Box<dyn Trait>`-accepting builder; the compiler enforces the dog-fooding guarantee. No private cross-crate calls (clippy clean). Confirmed.
- **ADR-016 Decision 3** — root crate IS the pipeline driver: `build()` wires parse → eval → validate → inject_lang_default → layout → export. Validators iterated via the public `iter_validators()` (new method added to `slideforge-plugin-api/src/registry.rs`), not private field access. No plugin logic lives in the root crate — registry.rs is registration-only, lib.rs is orchestration-only. Confirmed.

## Panic-safety perimeter + CI guard

`scripts/check-panic-profile.sh` is a section-aware TOML parser (correctly skips comments and scopes to the target `[profile.NAME]` table), enforces explicit `panic = "unwind"` on `release` and `dist`, scans `release`/`dist`/`bench` for `abort`, and has a positive-coverage floor (`EXPECTED_VALIDATED=2`). It is wired into `.github/workflows/ci.yml` as the `check-panic-profile` job and added to the `all-checks-pass` `needs:` aggregate, so a future PR flipping a shipped profile to `abort` is blocked before merge. The `[profile.dist]` block redundantly declares `panic = "unwind"` (belt-and-suspenders vs. `inherits = "release"`) — correct defensive choice. `set -euo pipefail`, reads `Cargo.toml` only, no network/writes. Solid.

## Code quality notes

- Production `build_inner` path has zero `.unwrap()`/`.expect()` — all error handling via `?`, `.ok_or`, `.ok_or_else`, `.map_err`. (All `expect`/`unwrap` hits are inside `#[cfg(test)]` modules.)
- Validator-loop panic semantics are correct: a single bundled-validator panic short-circuits the build into `BuildError::Plugin(PluginPanic)` — fail-safe, no crash.
- Ownership around `dispatch_plugin` closures is handled correctly: `validator_id`/`exporter_id` are `.to_owned()` and `exporter.extension()` captured before the closure borrows the plugin.
- `BuildOutput.extension` derived from `exporter.extension()` (trait method) not the format lookup key — proven by `test_med_d`.
- Diagnostic fidelity (`OwnedDiag`) preserves `code`/`help`/`labels` — satisfies the CLAUDE.md span+hint mandate.
- `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, `#![warn(clippy::pedantic)]` all present on the root crate.

## Scope / deferrals

No scope creep. HTML exporter correctly excluded (slideforge-html not in Wave 4 workspace); the deferral to STORY-050 is anchored in code comments (registry.rs line 31, lib.rs Cargo.toml comment) and the story spec. Full E2E (OBS-E) deferred to STORY-050. Acceptable anchored deferrals.

## Findings

### NIT-1 (non-blocking) — evidence-report.md describes a non-existent error variant

`docs/demo-evidence/STORY-049/evidence-report.md` line 141 states that
`test_h1_dispatch_plugin_catches_panicking_exporter` returns
`Err(BuildError::ExportFailed { .. })`. There is no `ExportFailed` variant on
`BuildError` (the actual variant is `Export(#[source] ExportError)`), and the
referenced test actually asserts `PluginError::PluginPanic` directly — it calls
`dispatch_plugin` and never constructs a `BuildError`. This is a documentation
inaccuracy in the evidence narrative only; the code and the test are correct.
Suggestion: reword to "returns `Err(PluginError::PluginPanic { .. })` through the
`dispatch_plugin` boundary" to match the test. Does not block merge.

### OBS-1 (informational) — stale pre-merge checklist box

PR description line 229 ("All dependency PRs merged") is left unchecked. I verified
independently that all predecessors (STORY-002/037-045/083/084/085, PRs #41-#59) are
merged into develop and the PR is MERGEABLE. The unchecked box is stale-at-write-time
only; the actual state satisfies the dependency gate.

## PR description accuracy

Mermaid architecture/dependency/traceability diagrams match the diff. AC→test→code
table is accurate. Convergence claims (12-pass cascade, 3/3 strict-CLEAN) are consistent
with the commit history (pass-2/3/4/6/9 fix commits visible). Test-evidence table matches
my local re-run (38/38). One label nit: the diagram node "BuildError::ValidationFailed"
and the evidence-report `ExportFailed` mismatch (NIT-1) are the only inaccuracies found.

## Conclusion

This is a well-constructed composition root. The dog-fooding guarantee is compiler-enforced,
the panic-safety perimeter is genuinely load-bearing and CI-guarded, the end-to-end
`build()` lynchpin produces a valid PPTX, and all 8 ACs are backed by real assertions.
The two findings are documentation-only and do not affect correctness or merge safety.

**PR REVIEW: APPROVE**
