# PR Review Findings: STORY-043 — PDF Core Backend

**PR:** #41 — feat(pdf): PDF Core backend — krilla + SlideTagEngine
**Story:** STORY-043
**BC:** BC-4.03.002
**Review initiated:** 2026-05-31

---

## Convergence Table

| Cycle | Total Findings | Blocking (CRIT/HIGH/MED) | Fixed | Remaining Blocking |
|-------|---------------|--------------------------|-------|-------------------|
| 1     | 2 (LOW/OBS)   | 0                        | 0     | 0 → APPROVE       |

---

## Cycle 1 Findings

### Finding R1-001
- **Severity:** LOW
- **Category:** coherence
- **Location:** `crates/slideforge-eval/Cargo.toml:32`
- **Description:** `indexmap = "=2.10.0"` is a direct pin, not `{ workspace = true }`. Other crates in this PR correctly use `{ workspace = true }` or `{ version = "=2.10.0", features = ["std"] }`. The slideforge-eval form has no `features = ["std"]` either, though std is likely not needed since eval only uses `IndexMap` from indexmap's `std` context via the workspace.
- **Status:** PRE-EXISTING on develop (not introduced by this PR — was `"=2.9.0"` before; the PR correctly bumped to `"=2.10.0"`). This inconsistency predates this PR. Not a blocker for merge.
- **Route:** OBS — note only. Pre-existing inconsistency not introduced by this PR.

### Finding R1-002
- **Severity:** OBS (observation)
- **Category:** coherence / spec-gap
- **Location:** AC-006 / `.github/workflows/ci.yml`
- **Description:** Story spec AC-006 mentions 5 CI targets including `x86_64-apple-darwin` (macOS Intel). The CI matrix covers only 4 runners (linux-x86_64, linux-arm64, macos-arm64, windows-x86_64). macOS-x86_64 was removed pre-existing on develop due to chronic runner availability issues (documented in the ci.yml comment at line 100).
- **Status:** PRE-EXISTING on develop — not introduced by this PR. This PR does not touch the test matrix. The CI comment explicitly documents the rationale. Not a blocker.
- **Route:** OBS — note only. Pre-existing CI architecture decision.

---

## Verdict: APPROVE

**CLEAN (strict):** yes — zero findings of any severity introduced by this PR
**CLEAN (PR-merge):** yes — zero CRIT/HIGH/MED findings

Both findings (R1-001, R1-002) are pre-existing conditions on `develop` not introduced by this PR. The PR correctly bumps indexmap and does not worsen the macos-x86_64 situation.

### Positive Affirmations

- All 26 slideforge-pdf tests pass; 2412/2412 workspace tests pass
- BC-4.03.002 traceable end-to-end: all 9 ACs have test coverage with BC IDs in test names
- `#![forbid(unsafe_code)]` + zero `.unwrap()` in non-test code verified
- `#![warn(missing_docs)]` present; all public items documented
- `clippy::pedantic` + `clippy::unwrap_used` clean
- Structural tag tree (Part/H1/Figure/Table/Document) confirmed via byte-level assertions in tests
- Dependency perimeter enforced at two layers: Rust integration tests + shell CI script both in `all-checks-pass`
- `krilla =0.6.0` correctly pinned as direct dep; `pdf-writer` and `subsetter` correctly transitive-only
- EMU canonicalization uses `slideforge_types::Emu::to_points()` — no duplicate constant (TD-VSDD-060 compliant)
- Architect-ruled deferrals (044/045/049) correctly cited with concrete story anchors (SID-1 compliant)
- `#[non_exhaustive]` on `PdfExportError` — forward-compatible without breaking downstream
- `Default` impl on `PdfExporter` and `SlideTagEngine` — ergonomic and correctly delegating to `new()`
- LOCAL adversary cascade: 3/3 strict-CLEAN at passes 12/13/14 of 14 (BC-5.39.001 satisfied)
