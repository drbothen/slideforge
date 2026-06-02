# PR Review — PR #47 (STORY-075: Brand Loader Footer Detection)

**Verdict: APPROVE** — 0 blocking findings.

Fresh-eyes merge-gate review against the story ACs, OOXML parsing correctness,
test quality (TD-VSDD-059), and conventions compliance. Reviewed every changed
file in the diff (7 source files + 1 evidence report + 7 demo recording sets).

---

## What I Verified

### OOXML parsing correctness (the core risk surface)

- **Multi-run concatenation (AC-001 / EC-008):** The SAX state machine accumulates
  `run_buf` per `<a:r>`, pushes it into `sp_text_accum` on `</a:r>`, and trims the
  full accumulator only at `</p:sp>`. The `MASTER_MULTIRUN_FOOTER_XML` fixture
  ("Acme Confidential " + "2026") asserts `Some("Acme Confidential 2026")`. This
  assertion fails under any first-run-only implementation — load-bearing. Correct.

- **Footer flag source = `<p:hf>` on slideMaster1.xml, NOT presProps.xml
  (AC-003 / adversary H-1):** Verified by grep of the production source: zero
  references to `presProps`, `showPr`, or `read_pres_props_flags`. The integration
  test `test_bc_2_01_001_load_pptx_footer_flags_populated_from_hf_element` goes
  further — it deliberately includes a `presProps.xml` stub in the ZIP and asserts
  the flags still come from `<p:hf>`. This is the exact correction from BC-2.01.001
  v1.3 and it is implemented correctly. `extract_hf_flags` parses `ftr`/`dt`/`sldNum`
  via `parse_ooxml_bool`, which accepts both `"1"/"0"` and `"true"/"false"`
  (ECMA-376 xsd:boolean). Both forms are exercised
  (`..._hf_boolean_string_form`, `..._v1_3_canonical_vector_ftr1_dt1_sldnum0`).

- **First-placeholder-wins (EC-003):** `footer_ph_done` gates the `<p:sp>` start
  and `<p:ph>` match handlers, so subsequent footer placeholders are skipped.
  `..._multiple_placeholders_first_wins` asserts `Some("First Footer")`. Correct.

- **`<a:fld>` vs `<a:r>` handling (EC-006):** `<a:t>` text is collected only when
  `in_run` is set, and `in_run` is set only on `<a:r>` start. An `<a:t>` nested
  inside `<a:fld>` is never collected. `..._field_element_treated_as_no_text`
  asserts `text = None`. Correct.

- **Layout fallback (AC-002):** When the master footer placeholder resolves to
  `EmptyText`, `detect_footer` re-reads `slideLayout1.xml` via a text-only pass.
  Flags always come from the master `<p:hf>` (per v1.0 scope — layout `<p:hf>`
  override is v2-deferred, matching story v1.2). Covered by three tests
  (fallback-has-text, no-layout, both-empty).

- **DOCX guard (AC-006):** `if !is_pptx` short-circuits to `FooterDetection::default()`
  before any ZIP read. `..._load_docx_footer_always_none` and
  `..._detect_footer_docx_returns_default` confirm — even with a master XML present
  in the DOCX ZIP, it is never read.

- **Absent master (AC-005 / EC-001):** Returns defaults with a `tracing::debug!`,
  no panic. `..._detect_footer_absent_master_xml` covers this.

### Conventions compliance (AC-008)

- No `presProps` parsing in production (grep-confirmed).
- No `.unwrap()`/`.expect()` outside the `#[cfg(test)]` block; production uses
  `.ok()?`, `.unwrap_or("")`, and `filter_map(Result::ok)`. The test module is
  gated with `#[allow(clippy::unwrap_used)]`.
- No `println!` — all logging via `tracing::debug!` with structured fields.
- No `unsafe`. `#[must_use]` on `detect_footer`. `#![warn(missing_docs)]` satisfied
  (every public item documented).
- All 16 CI checks pass: clippy (`-D warnings`), fmt, docs, doctest, supply-chain,
  msrv, snapshots, visual-regression, perf-smoke, and the full 4-platform test
  matrix (linux-x86_64/arm64, macos-arm64, windows-x86_64).

### Test quality (TD-VSDD-059 — no paper-fixes)

25 tests, every one with specific value assertions (`assert_eq!` on exact strings,
per-flag `assert!`). The flag tests are genuinely load-bearing — each documents the
Red Gate rationale showing it would fail against a presProps-reading implementation.
The multi-run test fails under first-run-only. The self-acknowledged structurally-weak
case (`..._hf_partial_attrs`, which passes for both impls) is explicitly flagged as
relying on the other `<p:hf>` tests for the Red Gate signal — honest and the
surrounding tests carry the load. No vacuous assertions found.

### Checklist (all 8)

1. **Diff coherence** — PASS. All production changes relate to footer detection.
   The `footer_flags: FooterFlags::default()` additions in `extractor.rs`,
   `synthesizer.rs`, `srgbclr_transform_tests.rs`, and `template.rs` are the required
   mechanical updates for the additive struct field.
2. **Description accuracy** — PASS. PR body matches the diff exactly.
3. **Test coverage** — PASS. Every changed production line is exercised; all 8 ACs
   and 8 ECs have tests.
4. **Demo evidence** — PASS. `evidence-report.md` present; 7 AC recording sets, each
   with `.gif` + `.webm` + `.tape`. Covers AC-001..AC-008 with both success and
   error/edge paths (dedicated Error-Path Coverage table). Real recordings, not `.txt`.
5. **Commit quality** — PASS. Conventional Commits, story ID present, clear messages
   tracing the TDD red→green→fix→docs sequence.
6. **Diff size** — PASS in context. 2379 additions, but ~1874 lines are the new
   `footer.rs` (dominated by XML fixtures + 25 tests) plus demo evidence. Production
   logic is compact.
7. **Missing changes** — PASS. Every spec deliverable present (footer.rs,
   template.rs field, loader.rs wiring at the former line-214 hardcode, lib.rs export).
8. **Dependency status** — PASS. Depends on STORY-022 (merged); blocks STORY-024
   (still draft — informational).

---

## Non-blocking observations (NIT — for awareness, no action required for merge)

- **[NIT] `local_name_owned` allocates a `String` per element event.** Every start/end
  element across the whole master XML triggers a heap allocation just to match a local
  name. Matching on `e.local_name().as_ref()` byte slices would avoid the allocation.
  Brand loading is not on the hot build path, so this is purely a micro-optimization —
  noting it only so a future perf sweep can pick it up.

- **[NIT] `in_sp` is a boolean, not a depth counter.** If a footer `<p:sp>` ever
  contained a nested shape (e.g., inside a group shape), a premature `</p:sp>` would
  reset `in_sp` early. OOXML footer placeholders are not nested shapes, and the
  `footer_ph_done` short-circuit further bounds the risk, so this is not a real-world
  defect — just a structural note.

---

## Note on flaky test

The `slideforge-diagrams cold_budget` timing test is a documented pre-existing flaky
test, not touched by this PR (not in the changed-files list). Not a blocker, per
review scope.

---

**Conclusion:** This PR correctly closes the BC-2.01.001 footer-detection gap and the
adversary H-1 presProps correction. Parsing logic is sound across all flagged edge
cases, tests are load-bearing, conventions are clean, CI is green, and demo evidence
is complete. Approving for merge.
