---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-079
title: "slideforge-diagrams: SVG DoS hardening — byte-size cap + nesting-depth guard"
epic: EPIC-12
wave: 5
points: 3
priority: P2
tdd_mode: strict
status: draft
behavioral_contracts: [BC-1.12.003]
verification_properties: []
nfr_refs: [NFR-016, NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
crate: slideforge-diagrams
target_module: slideforge-diagrams
subsystems: [SS-11]
depends_on:
  - STORY-034
blocks: []
estimated_days: 1
# BC status: BC-1.12.003 v1.2 active — PO review complete (2026-06-08). SEC-001 (CWE-674, nesting-depth cap) and SEC-002 (CWE-400, byte-size cap) formalized as postconditions 8–10 and invariants 4–7. Story ACs and implementation scope unchanged.
---

# STORY-079: slideforge-diagrams — SVG DoS Hardening (byte-size cap + nesting-depth guard)

## Subsystem Anchor Justification

SS-11 (Diagrams) owns this story because `usvg_normalize` in `slideforge-diagrams` is
the sole normalization path for externally-produced mermaid SVG. Per ARCH-INDEX, SS-11
(`slideforge-diagrams`) is "Effectful shell (font DB scan). Mostly pure." The DoS guard
must live at the SS-11 boundary — before the allocation-heavy `usvg::Tree::from_str`
parse — mirroring the same architectural decision made in SS-07 (PDF Export, STORY-043
SEC-001/SEC-002).

## Dependency Anchor Justifications

- Depends on STORY-034: STORY-034 delivered `usvg_normalize` and established the
  `RawDiagramSvg` type. This story adds size/depth guards to the existing function —
  it cannot exist without the function body from STORY-034.
- Blocks nothing: diagrams hardening is orthogonal to all downstream exporters; the
  existing `NormalizedDiagramSvg` contract is unchanged.

## Summary

STATE.md drift item (2026-05-31, row DI-4): `crates/slideforge-diagrams/src/normalize.rs`
calls `usvg::Tree::from_str(raw.as_str(), &opt)` on externally-sourced mermaid SVG
WITHOUT a size or nesting-depth guard. A pathologically large SVG payload (or deeply
nested `<g>` tree) supplied by a compromised diagram renderer or a malicious `@data`
source could exhaust heap/stack before usvg returns an error — a CWE-400 resource
exhaustion vector.

STORY-043 (PDF Export) already fixed this class of vulnerability in `slideforge-pdf`
via `MAX_SVG_BYTES = 50 * 1024 * 1024` (SEC-002, CWE-400) and
`MAX_SVG_NESTING_DEPTH = 64` (SEC-001, CWE-674). This story applies the identical
pattern to `slideforge-diagrams` for consistency and defense-in-depth.

### Deliverables

1. **`MAX_SVG_BYTES` constant** in `normalize.rs`: `50 * 1024 * 1024` (50 MiB).
   Pre-parse byte-length check on `raw.as_str().len()`. On exceed:
   `Err(DiagramError::SvgNormalizationFailed { slide_title, cause: "SVG input too large: N bytes exceeds the 52428800-byte limit", span: SourceSpan::from(0..0) })`.
   Error code `E-EXP-004` (existing taxonomy entry for SVG normalization failures).

2. **`MAX_SVG_NESTING_DEPTH` constant** in `normalize.rs`: `64`.
   Post-parse XML traversal counting `<g>` depth via a linear scan (no recursion).
   On exceed: `Err(DiagramError::SvgNormalizationFailed { ... cause: "SVG group nesting depth N exceeds the 64-level limit; possible DoS input" ... })`.
   The scan must be iterative (not recursive) to avoid replacing one stack-exhaustion
   vector with another.

3. **Tracing** on rejection: `tracing::warn!(bytes = raw.as_str().len(), "usvg_normalize: SVG rejected — exceeds size cap")` and equivalent for depth.

4. **Three new tests** (non-vacuous, load-bearing):
   - `test_sec_dos_svg_oversize_rejected`: SVG of `MAX_SVG_BYTES + 1` bytes returns `Err(SvgNormalizationFailed)` containing "E-EXP-004".
   - `test_sec_dos_svg_deep_nesting_rejected`: SVG with `MAX_SVG_NESTING_DEPTH + 1` nested `<g>` elements returns `Err(SvgNormalizationFailed)`.
   - `test_sec_dos_svg_valid_input_unaffected`: A real-world Mermaid SVG well within both limits returns `Ok(NormalizedDiagramSvg)`.

5. **No new Cargo dependencies** — all guards use the existing standard library only.

### Design Note: Error Code Reuse

`E-EXP-004` ("SVG normalization failed for diagram '<title>': <usvg-error>") is the
correct error code for both CWE-400 and the existing malformed-SVG path. The error
taxonomy does not require distinct codes for sub-causes within the same normalization
failure category. The `cause` field in `DiagramError::SvgNormalizationFailed` carries
the distinguishing detail. This matches the STORY-043 pattern where `PdfExportError::SvgEmbed`
wraps both SEC-001 and SEC-002 failures.

### Constants Must Match slideforge-pdf

`MAX_SVG_BYTES = 50 * 1024 * 1024` and `MAX_SVG_NESTING_DEPTH = 64` MUST equal the
constants in `crates/slideforge-pdf/src/svg_embed.rs`. If a future story changes one,
it must change both (see Forbidden Patterns below). Consistent limits prevent
split-brain security posture.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-1.12.003 | SVG Normalized via usvg Before PPTX Embedding (no foreignObject, absolute dims) | AC-001 through AC-003 |

Note: BC-1.12.003 postcondition 7 ("If usvg fails to normalize the SVG, E-EXP-004 is
emitted") is the anchor clause. The DoS guards are a security invariant extending that
postcondition: the guard MUST fire before `usvg::Tree::from_str` to prevent resource
exhaustion even when usvg would eventually return an error.

## Acceptance Criteria

### AC-001: Oversized SVG rejected before parse (traces to BC-1.12.003 postcondition 7)

When `usvg_normalize` is called with a `RawDiagramSvg` whose byte length exceeds
`MAX_SVG_BYTES` (50 MiB), it MUST return `Err(DiagramError::SvgNormalizationFailed)`
WITHOUT calling `usvg::Tree::from_str`. The error `cause` string MUST contain the
actual byte count and the limit. The error display MUST contain "E-EXP-004". The
rejection MUST occur before any heap allocation proportional to the SVG size.

### AC-002: Deeply nested SVG rejected after parse, before re-serialization (traces to BC-1.12.003 postcondition 7)

When `usvg_normalize` is called with a `RawDiagramSvg` containing more than
`MAX_SVG_NESTING_DEPTH` (64) nested `<g>` elements (measured as the maximum depth
of the `Group` nesting tree), it MUST return `Err(DiagramError::SvgNormalizationFailed)`
after the usvg parse step but before `tree.to_string()` is called. The error `cause`
MUST contain the detected depth and the limit. The depth scan MUST be iterative
(stack-based), not recursive — no unbounded call stack depth.

**usvg 0.47.0 API (verified):** usvg 0.47.0 preserves source `<g>` nesting 1:1 in the
parsed `Tree` (dummy-group removal was dropped in usvg 0.30.0). The depth scan MUST
operate on the parsed `usvg::Tree`, NOT on the raw SVG string. Traverse using
`Group::children()` — each child is a `usvg::Node` enum; match `usvg::Node::Group(g)`
to descend. The root `Tree::root()` is itself a `Group` counted as depth 1. Use an
iterative stack-based DFS: push `(group_ref, current_depth)` tuples; track
`max_depth: usize`; return error if `max_depth > MAX_SVG_NESTING_DEPTH`. (per
export-architecture v1.2)

### AC-003: Valid SVG within limits passes through unaffected (traces to BC-1.12.003 postcondition 7)

When `usvg_normalize` is called with a `RawDiagramSvg` that is:
- Fewer than `MAX_SVG_BYTES` bytes in length, AND
- Contains `<g>` nesting no deeper than `MAX_SVG_NESTING_DEPTH` levels,

it MUST return `Ok(NormalizedDiagramSvg)` with behavior identical to the pre-guard
implementation. The guards MUST NOT alter the normalization output for legitimate SVG.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `usvg_normalize` (extended) | `crates/slideforge-diagrams/src/normalize.rs` | Mostly pure (reads system fonts once via OnceLock) |
| Tests | `crates/slideforge-diagrams/src/normalize.rs` `#[cfg(test)] mod tests` | Pure (no I/O in new tests) |

Architecture section files:
- `architecture/module-decomposition.md` (SS-11 boundary)
- `architecture/dependency-graph.md` (no new deps added)

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~2,500 |
| `crates/slideforge-diagrams/src/normalize.rs` (existing, ~1,400 lines) | ~18,000 |
| `crates/slideforge-pdf/src/svg_embed.rs` (reference pattern) | ~4,000 |
| BC-1.12.003 | ~800 |
| Error taxonomy excerpt (E-EXP-004) | ~200 |
| Test output / tool feedback | ~2,000 |
| **Total** | **~27,500** |

27,500 tokens is well within 20% of a 200k-token agent context window. No split needed.

## Tasks

- [ ] Read `crates/slideforge-pdf/src/svg_embed.rs` SEC-001/SEC-002 constants and test pattern
- [ ] Add `MAX_SVG_BYTES: usize = 50 * 1024 * 1024` constant to `normalize.rs` with SEC-002 doc comment
- [ ] Add `MAX_SVG_NESTING_DEPTH: usize = 64` constant to `normalize.rs` with SEC-001 doc comment
- [ ] Insert byte-size guard at top of `usvg_normalize` body (before `font_db()` call)
- [ ] Implement iterative `<g>`-depth counter scanning the parsed `usvg::Tree` (NOT the raw string): use an explicit stack of `(&Group, usize)` tuples; call `group.children()` and match `usvg::Node::Group(child_group)` to recurse; root group (`tree.root()`) counts as depth 1; track `max_depth`; usvg 0.47.0 preserves source `<g>` nesting 1:1 (per export-architecture v1.2)
- [ ] Insert depth guard after `usvg::Tree::from_str` succeeds, before `tree.to_string()`
- [ ] Add `tracing::warn!` events for both rejection paths
- [ ] Write `test_sec_dos_svg_oversize_rejected` (size cap test)
- [ ] Write `test_sec_dos_svg_deep_nesting_rejected` (nesting depth test)
- [ ] Write `test_sec_dos_svg_valid_input_unaffected` (happy path non-vacuous test)
- [ ] `cargo nextest run -p slideforge-diagrams --no-fail-fast` — all 3 new tests pass, existing tests unaffected
- [ ] `cargo clippy -p slideforge-diagrams --all-targets -- -D warnings` clean
- [ ] `cargo fmt -p slideforge-diagrams -- --check` clean

## Previous Story Intelligence

STORY-043 (PDF Core: pdf-writer + krilla + SlideTagEngine) established the
SEC-001/SEC-002 pattern used here. Verbatim key implementation notes:

1. **Size guard fires BEFORE `usvg::Tree::from_str`** — the entire point is to avoid
   the allocation-heavy parse. Check `svg_str.len() > MAX_SVG_BYTES` and return
   `Err(...)` immediately. Do NOT parse first and check size later.

2. **Depth guard is iterative, not recursive, and scans the parsed Tree** — the
   STORY-043 implementation uses a stack-based DFS over the usvg node tree via
   `Group::children()` matching `usvg::Node::Group`. Use the same approach here.
   usvg 0.47.0 preserves `<g>` nesting 1:1 in the Tree (confirmed; dummy-group
   removal was dropped in usvg 0.30.0), so counting `Group` depth in the parsed
   Tree is equivalent to counting raw `<g>` depth. Do NOT scan the raw string —
   scan the parsed Tree. A recursive depth counter would be ironic (replacing one
   stack-exhaustion path with another). (per export-architecture v1.2)

3. **The test for oversize does NOT allocate 50 MiB** — in STORY-043 the test uses
   a technique of wrapping a minimal SVG with enough whitespace padding to cross the
   threshold. For `MAX_SVG_BYTES + 1 = 52,428,801`, use:
   ```
   let prefix = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\">";
   let suffix = b"</svg>";
   let padding = MAX_SVG_BYTES + 1 - prefix.len() - suffix.len();
   let svg = format!("{}{}{}", str::from_utf8(prefix).unwrap(), " ".repeat(padding), str::from_utf8(suffix).unwrap());
   ```
   The SVG is valid XML but oversize — the guard fires before any expensive parse.

4. **E-EXP-004 is the correct error code** — do NOT invent a new code. The error
   taxonomy entry "SVG normalization failed for diagram '<title>': <usvg-error>" is
   general enough to encompass both malformed-SVG and policy-rejection failures.

5. **Constants must match slideforge-pdf exactly** — `MAX_SVG_BYTES = 50 * 1024 * 1024`
   and `MAX_SVG_NESTING_DEPTH = 64`. Do not use different values.

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and project ADRs:

1. **No new production dependencies** (NFR-025): the guards use only the Rust standard
   library and the already-imported `usvg` crate. `Cargo.toml` for `slideforge-diagrams`
   MUST NOT gain new dependencies.
2. **Zero `.unwrap()` in non-test code** (NFR-021): error mapping uses `?` with
   structured `DiagramError` variants. No `.expect()` in production paths.
3. **`#![forbid(unsafe_code)]`** (NFR-024): the iterative depth scan MUST be implemented
   with safe Rust only.
4. **`tracing` for all security-relevant rejections** (CLAUDE.md conventions): both
   size-cap and depth-cap rejections MUST emit `tracing::warn!` with structured fields.
5. **Guard ordering invariant**: size guard MUST precede depth guard MUST precede
   `font_db()` call. The font DB load (50–300ms) must never be paid for an oversized input.

## Library and Framework Requirements

All version pins from `Cargo.lock` / workspace `Cargo.toml`:

| Dependency | Version | Usage |
|------------|---------|-------|
| `usvg` | `{workspace = true}` (=0.47.0 — centralized in [workspace.dependencies] per ADR-022) | existing dependency — `usvg::Tree` / `Group::children()` / `usvg::Node::Group` iteration for depth check |
| `tracing` | `{workspace = true}` (=0.1.44 — centralized in [workspace.dependencies] per ADR-022) | `tracing::warn!` for rejection events |
| `miette` | `{workspace = true}` (=7.6.0 — centralized in [workspace.dependencies] per ADR-022) | `SourceSpan::from(0..0)` for position-less errors |
| `thiserror` | `{workspace = true}` (=2.0.18 — centralized in [workspace.dependencies] per ADR-022) | `DiagramError` is already `#[derive(thiserror::Error)]` |

No new dependencies. All listed crates are already in `slideforge-diagrams/Cargo.toml`.

## File Structure Requirements

| Action | File | Change |
|--------|------|--------|
| Modify | `crates/slideforge-diagrams/src/normalize.rs` | Add 2 constants + size guard + depth guard + tracing + 3 tests |
| No change | `crates/slideforge-diagrams/Cargo.toml` | No new deps |
| No change | `crates/slideforge-diagrams/src/lib.rs` | Public API unchanged |
| No change | `crates/slideforge-diagrams/tests/cold_budget.rs` | Timing test not affected |

The **only file modified** is `normalize.rs`. All new tests are in the existing
`#[cfg(test)] mod tests` block at the bottom of `normalize.rs`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SVG exactly at `MAX_SVG_BYTES` | PASS — boundary is exclusive (`> MAX_SVG_BYTES` rejects) |
| EC-002 | SVG at `MAX_SVG_BYTES + 1` | FAIL — `Err(SvgNormalizationFailed)` with E-EXP-004 |
| EC-003 | SVG with exactly `MAX_SVG_NESTING_DEPTH` levels | PASS — boundary is exclusive (`> MAX_SVG_NESTING_DEPTH` rejects) |
| EC-004 | SVG with `MAX_SVG_NESTING_DEPTH + 1` levels | FAIL — `Err(SvgNormalizationFailed)` with E-EXP-004 |
| EC-005 | Already-handled malformed SVG (usvg parse failure) | Unchanged — existing `map_err` path still fires; no interaction with new guards |
| EC-006 | Valid Mermaid flowchart SVG (real-world sample) | PASS — `Ok(NormalizedDiagramSvg)`, no behavioral change |

## Forbidden Dependencies

The following modules MUST NOT appear as new production dependencies in
`crates/slideforge-diagrams/Cargo.toml`. If any of the below are added, the build
MUST fail (CI `cargo deny` / dependency grep enforcement):

- Any additional XML parsing crate (only `usvg` / `roxmltree` transitively)
- Any HTTP client crate (this is a pure normalization module)
- Any async runtime crate (`tokio`, `async-std`, `smol`)

## Test Strategy

All tests are unit tests in `#[cfg(test)] mod tests` within `normalize.rs`, consistent
with the project convention (tests next to the code). No new integration test binaries.

### Non-Vacuity Requirements

- `test_sec_dos_svg_oversize_rejected`: the test MUST construct an SVG that is
  `MAX_SVG_BYTES + 1` bytes and verify the specific `SvgNormalizationFailed` variant
  is returned. Asserting only `is_err()` is insufficient — the test must prove the
  size-cap path, not any other error path.
- `test_sec_dos_svg_deep_nesting_rejected`: the test MUST construct an SVG with exactly
  `MAX_SVG_NESTING_DEPTH + 1` `<g>` levels and verify `SvgNormalizationFailed` is
  returned. The `cause` string MUST be inspected to confirm the depth-cap message
  (not the size-cap or usvg-parse-failure message).
- `test_sec_dos_svg_valid_input_unaffected`: the test MUST verify the existing
  `simple_geometry_svg()` fixture still returns `Ok(NormalizedDiagramSvg)` after
  the guards are added. This proves the guard does not accidentally reject legitimate input.

## Complexity Estimate

3 story points. The algorithm is fully specified (copy the SEC-001/SEC-002 pattern from
`slideforge-pdf`). The implementer must read one source file (STORY-043's `svg_embed.rs`),
adapt the constants and error type to `slideforge-diagrams`, and write three tests. No
architectural design work required.

Compared to STORY-034 (5 pts, full `usvg_normalize` implementation + OnceLock + font
resolver + re-injection pipeline): this story adds ~30 lines of production code and
~80 lines of test code to an already-implemented function. Estimated 1 day including
adversarial cascade.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-31 | story-writer | Initial creation from STATE.md drift item DI-4; applies SEC-001/SEC-002 pattern from STORY-043 to slideforge-diagrams. |
| 1.1 | 2026-06-07 | story-writer | Wave-5 remove-uncertainty propagation: confirmed usvg 0.47.0 preserves source `<g>` nesting 1:1 (dummy-group removal dropped in usvg 0.30.0); updated AC-002 and depth-guard task to use `Group::children()` matching `usvg::Node::Group` on the parsed Tree (not raw string); removed raw-string ambiguity from Previous Story Intelligence; fixed three stale version pins: tracing =0.1.41→{workspace=true} (=0.1.44), thiserror =2.0.12→{workspace=true} (=2.0.18), miette =7.2.0→{workspace=true} (=7.6.0) per ADR-022. Cited export-architecture v1.2. |
