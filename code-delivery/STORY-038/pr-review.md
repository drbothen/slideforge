# PR #54 Re-Review (STORY-038 pptx) — Fix-Burst Delta

**Head reviewed:** `47a65364`
**Scope:** Delta since prior APPROVE-with-suggestions (commits `f6f88a22` → `47a65364`).
**Verdict: REQUEST CHANGES (blocking)**

The substance of the fix burst is **sound and correct** — SEC-001, SEC-002, NIT-1, and
the demo-assertion suggestions are all properly closed with load-bearing tests. However,
the burst introduced **CI-failing `fmt` and `clippy` violations** in the same files it
touched, and the PR description's "clippy/fmt/doc clean" claim is **inaccurate** at this
head. CI agrees independently: `fmt` FAIL, `clippy` FAIL, `all-checks-pass` FAIL.

The blockers are mechanical and ~5 minutes to fix (`cargo fmt --all` + inline one
`format!` arg). Re-run `just check` before re-push.

---

## Fix-burst substance — verified GOOD

| Item | Assessment |
|------|-----------|
| **SEC-001** (validate_emu i32-overflow on frame bbox) | SOUND. `validate_emu` now rejects any of x/y/width/height that overflows `i32::MAX`. Mirrors the existing AC-012 slide-size validator in `presentation.rs` (which validates `<p:sldSz>` against `i32::MAX` because `ST_PositiveCoordinate32` is i32). Two new tests (`test_sec001_validate_emu_i32_overflow_width/x`) feed `Emu(i64::MAX)` through the **real** `SlideSerializer::build()` path and assert the exact `InvalidEmu` variant + offending field name in `detail`. Load-bearing — not paper-fixes. |
| **SEC-002** (notes/handout rels error propagation) | SOUND. `build_notes_handout_masters` now returns `Result<(), PptxError>`; both `RelsBuilder::build()` calls are `?`-propagated via `PptxError::OoxmlElement` instead of the prior `unwrap_or_else(\|_\| b"".to_vec())` silent-empty fallback (CWE-755 / CLAUDE.md Forbidden Pattern). Sibling-site sweep (TD-VSDD-060): exactly **one** caller (`lib.rs:143`), correctly `?`-propagated inside `export()` which already returns `Result`. No swallow, no orphaned callers, no regression. Two new tests assert both rels parts are non-empty and contain `<Relationship>` in the produced ZIP. |
| **NIT-1** (ooxmlsdk comment in layout_xml.rs) | SOUND. Corrected comment no longer falsely claims ooxmlsdk "may not produce the opening container tag." Verified: `serialize_master_to_xml` uses `quick_xml::Writer` and emits `<p:sldLayoutIdLst>` explicitly via `Event::Start(BytesStart::new("p:sldLayoutIdLst"))` (layout_xml.rs:678). The corrected claim is accurate. |
| **Demo #2** (ac009/ac010 now assert) | SOUND. Previously these only `println!`'d the expected mapping table and unconditionally returned `true` (no verification). Now they export a real 1-slide deck per keyword, read `slide1.xml.rels`, and assert it references the correct `slideLayoutN.xml` — ac009 additionally asserts it does **not** reference `slideLayout1.xml` (index 0). Result gates via `all_ok`. Genuinely exercises the export path now. |
| **Demo #3** (ac011 tautology) | SOUND. `has_idx_0 = ... \|\| slide1.contains("type=\"title\"")` was a real tautology (RHS == `has_ph_title`), always-true when the branch was entered, masking a missing `idx="0"`. Now checks `idx="0"` independently, and `has_idx_0` is load-bearing in the function's return (`has_ph_title && has_idx_0 && has_body_ph`). |

Tests: **410 pass** locally (incl. the 4 new SEC tests). I reproduced the suite at head.

---

## BLOCKING findings

### [BLOCKING-1] `cargo fmt --all --check` fails — 5 violations introduced by this burst

| Field | Value |
|-------|-------|
| Severity | blocking |
| Category | coherence / CI gate |

CI `fmt` job FAILs, and I reproduced it locally with the pinned stable rustfmt 1.9.0.
All 5 violations are in files this burst touched:

- `crates/slideforge-pptx/examples/demo_layout.rs:509` — `fail(&format!(...))` should collapse to one line
- `crates/slideforge-pptx/examples/demo_layout.rs:585` — same
- `crates/slideforge-pptx/src/slide_serializer.rs:511` — the SEC-001 `for (name, value) in [...]` array literal must break one-element-per-line
- `crates/slideforge-pptx/src/tests/core_tests.rs:464` — `zip_read_entry(&pptx_bytes, "...handoutMaster1.xml.rels")` must wrap
- `crates/slideforge-pptx/src/tests/layout_tests.rs:1704` — the SEC-001 `panic!(...)` must wrap

`cargo fmt --all -- --check` is a non-negotiable CI gate (CLAUDE.md Build & Test / Quality Bar).

**Fix:** `cargo fmt --all` and commit. ~1 minute.

### [BLOCKING-2] `cargo clippy --all-targets --all-features` fails — pedantic `uninlined_format_args` in the new ac009 demo code

| Field | Value |
|-------|-------|
| Severity | blocking |
| Category | coherence / CI gate |

CI `clippy` job FAILs at `crates/slideforge-pptx/examples/demo_layout.rs:522`:

```
error: variables can be used directly in the `format!` string
   --> crates/slideforge-pptx/examples/demo_layout.rs:522:9
522 |         println!(
523 |             "    {kw:25} → {} (uses_index_1={}  uses_index_0={})",
524 |             expected_layout_file, uses_correct_layout, uses_wrong_layout
```

This is in the **ac009 demo fix itself** (Demo #2). `clippy::pedantic` with `-D warnings`
is a non-negotiable CI gate.

**Fix:** inline the args:
```rust
println!(
    "    {kw:25} → {expected_layout_file} (uses_index_1={uses_correct_layout}  uses_index_0={uses_wrong_layout})"
);
```

### [BLOCKING-3] PR description claims "clippy/fmt/doc clean" — inaccurate at head 47a65364

| Field | Value |
|-------|-------|
| Severity | blocking |
| Category | description |

The description states "Tests went to 410 pass; clippy/fmt/doc clean." Tests and doc are
clean, but `fmt` and `clippy` are NOT (BLOCKING-1, BLOCKING-2; CI confirms). Update the
claim once the gates are green.

---

## NIT (non-blocking)

### [NIT-1] SEC-001 comment cites "ST_Coordinate32" for frame `<a:off>`/`<a:ext>`

`slide_serializer.rs` justifies the i32 bound as "ECMA-376 ST_Coordinate32." In ECMA-376,
frame-level `<a:off>`/`<a:ext>` are `ST_Coordinate` (xsd:long / i64), not the i32
`ST_Coordinate32` (that type governs slide-size `<p:sldSz>`, which is what `presentation.rs`
AC-012 validates). The i32 bound chosen here is **defensible** — it's conservative
(PowerPoint rejects offsets beyond i32 in practice), it mirrors the slide-size validator
for consistency, and it only rejects absurd >2.1-billion-EMU (>2.3 km) coordinates. So the
behavior is fine; only the cited schema type is imprecise. Consider rewording to "matches
the i32 slide-size bound enforced in presentation.rs (AC-012) for cross-validator
consistency" rather than asserting frame offsets are `ST_Coordinate32`. Non-blocking.

### [NIT-2] ac011 demo prints `ok(...)` for the idx chain even when `has_idx_0` is false

In `check_ac011`, the `if has_ph_title { ... ok("...idx chain present") }` line prints a
success message regardless of `has_idx_0`. The function-level return still gates correctly
(`&& has_idx_0`), so this is purely a per-line console-message cosmetic. Consider gating the
`ok`/`fail` print on `has_idx_0` too for honest demo output. Non-blocking.

---

## What I verified (no rubber-stamp)

- Re-ran `slideforge-pptx` + `slideforge-brand` suites at head: 410 pass, 1 skipped.
- Inspected the SEC-001/SEC-002 tests — confirmed they drive the real production code paths
  (`SlideSerializer::build`, `export` ZIP output), assert exact error variants, and would
  fail if the production checks were removed (load-bearing, TD-VSDD-059 clear).
- TD-VSDD-060 sibling-site sweep on the `build_notes_handout_masters` signature change:
  one definition, one correctly-propagated caller.
- Confirmed the NIT-1 comment's new claim against `serialize_master_to_xml` (quick_xml).
- Reproduced CI fmt + clippy failures locally with pinned stable rustfmt/clippy 1.9.0.

---

**Verdict: REQUEST CHANGES.** The engineering is correct and the security findings are
properly closed with load-bearing tests — this is close. The blockers are mechanical CI-gate
violations (`cargo fmt --all` + one inlined `format!` arg) plus a 1-line description
correction. Run `just check` before re-push; should go green and flip to APPROVE.
