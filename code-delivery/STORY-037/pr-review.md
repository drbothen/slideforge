## PR Reviewer — Fresh-Eyes Review: PR #52 (STORY-037, PPTX Core Serialization)

**Verdict: APPROVE WITH SUGGESTIONS (no blocking findings)**

Independent review by a different model family. I reviewed every changed source
file at head `e7699d9a` against the 10 ACs, the project conventions, and
regression risk to `develop`. CI is fully green (21/21 checks including audit,
supply-chain, pinning-audit, semgrep, the 4-platform test matrix, snapshots, and
visual-regression).

---

### Checklist outcome

| # | Item | Result |
|---|------|--------|
| 1 | Diff coherence | PASS — all changes scoped to `slideforge-pptx` (new crate) + additive `slideforge-brand` extensions. No unrelated churn. |
| 2 | Description accuracy | PASS — PR body claims match the diff (verified ooxmlsdk usage, two-phase layout mapping, dual-zip note, determinism, clrMapOvr, typed `<p:pic>`). |
| 3 | Test coverage | PASS (one minor gap, see S1) — 42 load-bearing tests across all 10 ACs + F-037 finding fixes. Determinism and bleed tests are genuinely end-to-end, not tautological. |
| 4 | Demo evidence | PASS — real GIF (440 KB) + WebM (852 KB) + tape + evidence-report.md. All 10 ACs mapped; success and register-exclusion paths shown; AC-008 correctly deferred to STORY-052 CI gate. |
| 5 | Commit quality | PASS — conventional `feat(pptx):` with story ID. |
| 6 | Diff size | NOTE — ~6.0k additions, but it is a new crate + a pre-existing brand module extension. Appropriate for a 13-point story; not a true reviewability concern. |
| 7 | Missing changes | PASS — every claimed AC is implemented and tested. Deferrals are genuine (see below). |
| 8 | Dependency status | PASS — all upstream stories merged; both new deps pinned with `=`. |

---

### Verification of deferral genuineness (ADR-015 Addendum A)

The deferrals are real scope boundaries, not dodges:

- **Plain-text flattening of inline markup** (`extract_inline_text` collapses
  bold/italic/code/link/math to plain text) is consistent with "core
  serialization" scope; rich-run formatting is downstream work. The IR is read
  correctly; nothing is corrupted, only un-styled.
- **9 unmapped DSL keywords → index-1 fallback** is implemented with a
  `tracing::warn!` and an explicit no-silent-fallback rationale; STORY-038 owns the
  new layout slots. `find_layout_index` is a real two-phase lookup (keyword, then
  ooxml_type) with dedicated unit tests, not a stub.
- **clrMapOvr-all-31, SVG raster parity, slide-shape alt-text, notes content** are
  each anchored to a specific later story (038/039/040). The dark-layout
  `clrMapOvr` path that IS in scope is implemented and tested (EC-005).

### Regression check on `slideforge-brand` (merged crate extended)

No regression. The only public-surface change is the new
`SlideLayoutDef.slide_type_keyword` field. All construction of `SlideLayoutDef`
is funnelled through the crate-private `light_layout` / `custom_layout` /
`dark_layout` helpers (signatures changed internally only) plus brand-crate
tests; there are NO external direct constructors of `SlideLayoutDef` in the
workspace, so adding the field compiles cleanly everywhere. The new
`serialize_master_to_xml` / `serialize_theme_to_xml` follow the SAME
`quick_xml`-event pattern already established in this module by STORY-023's
`serialize_layout_to_xml` — so this is consistent with the brand crate's existing
convention (and the ooxmlsdk-only rule correctly applies to `slideforge-pptx`,
which does use ooxmlsdk typed builders throughout).

Schema-significant ordering is handled correctly: the master emits
`cSld → clrMap → sldLayoutIdLst → hf → txStyles` with an explicit comment citing
the F-PASS2-H1 fix (`<p:hf>` before `<p:txStyles>`). The `sldLayoutId` rId
convention (rId2..rId32, theme=rId1) matches the master `.rels` built in
`lib.rs`. Theme `clrScheme` element order matches ECMA-376.

---

### Non-blocking suggestions

**[S1 — suggestion / coverage] `validate_emu` Err path is untested.**
`slide_serializer.rs:275` returns `PptxError::InvalidEmu` on negative width/height,
and the path is wired into every frame branch, but no test injects a negative-size
bbox to exercise the `Err` branch. Under the production-grade testing bar, add a
small unit test that builds a frame with `width < 0` and asserts `InvalidEmu`.

**[S2 — suggestion / robustness] Silent i32 clamp on slide size.**
`presentation.rs:159-160` uses `i32::try_from(emu).unwrap_or(9_144_000)` /
`unwrap_or(5_143_500)`. For any realistic deck EMU fits in i32, so this never
triggers, but it is a silent fallback on out-of-range input rather than a
structured error — slightly at odds with the no-silent-fallback convention applied
elsewhere in this very PR. Consider returning `PptxError::InvalidEmu` (or
documenting the clamp as a deliberate, logged saturation).

**[S3 — nit / fidelity] `Subtitle` frames map to `PlaceholderValues::Title`
(idx 0).** `slide_serializer.rs:124-143`/`366-370` treat Title and Subtitle
identically. Real PPTX subtitles use a `subTitle` placeholder. Harmless for
"core serialization," but worth confirming this is intended to land in STORY-038
rather than silently persisting.

**[S4 — nit] `build_notes_handout_masters` swallows rels-build failure** with
`unwrap_or_else(|_| b"".to_vec())` (`lib.rs:632,650`), which would emit an empty
(invalid) `.rels` part instead of surfacing an error. The path is documented as
infallible (one entry always added), so this is cosmetic — but an empty `.rels`
would be a silent corruption if the invariant ever broke. Prefer propagating the
error like the sibling `build_*` functions do.

None of S1–S4 block merge.

---

### What I verified directly (anti-rubber-stamp)

- ooxmlsdk typed builders used throughout `slideforge-pptx` (Content_Types, rels,
  presentation, slide, `<p:pic>` blip-fill) — no XML string concatenation except
  the documented OPC `docProps/core.xml` exception, which XML-escapes the
  user-supplied `lang` value (F-037-006, with a test).
- Determinism: real two-build SHA-256 equality test (1-slide and 5-slide), plus an
  epoch-timestamp assertion reading actual ZIP entries. Sort-by-path + epoch
  DateTime + alphabetical Content_Types overrides — no HashMap iteration leakage.
- AC-010 register bleed: tests inject `REPORT_SENTINEL_037` / `DETAIL_SENTINEL_037`
  into `register_content` and assert absence via the real `BleedChecker`
  (entity-decoding aware). Genuinely load-bearing.
- Slide IDs ≥ 256 and master ID = 2^31 asserted against parsed `presentation.xml`.
- Dual `zip` crate versions (4.2.0 in pptx, 2.3.0 in brand/eval) — passed `audit`,
  `supply-chain`, and `pinning-audit` CI gates; rationale documented in Cargo.toml.

**Recommendation:** APPROVE WITH SUGGESTIONS. Safe to squash-merge. S1–S4 can be
folded into STORY-038 or a follow-up; none are merge blockers.
