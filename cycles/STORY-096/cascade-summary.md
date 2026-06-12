---
story: STORY-096
title: "REND-007: PPTX slideMaster 16:9 geometry + progress_bar layout + lang on runs"
merge_sha: 2a873a5d65d22a2c1b59e3c2ff8dd440a76066f8
pr: 88
developed_sha: 6e29bce17283c20c75211cad957f2064c67236df
cascade_passes: 9
findings_closed: 11
sec_findings: [SEC-096-001]
converged_at: "passes 7-8-9"
po_adjudications: 2
---

# STORY-096 Cascade Summary

## Merge Facts

- **PR #88** squash-merged to develop `2a873a5d` (88 merged PRs, 0 open PRs).
- **REND-007 CLOSED.**
- EPIC-08, 5 pts, P0. Target crate: `slideforge-pptx`.
- LOCAL adversary cascade: 9 passes, 11 findings + SEC-096-001 closed, **CONVERGED 3/3 strict-CLEAN (passes 7-8-9)**.
- Reviews: security CLEAN (initial + re-verify of `9c8e1e1e`); pr-reviewer APPROVE (3 nits N-1/N-2/N-3 non-blocking).
- Demo evidence: `.factory/demos/STORY-096-demo-evidence.md` (4/4 ACs PASS — 11 tests pass).
- Workspace tests at tip: 4214 pass / 20 skip / 0 fail. CI fast tier green (test linux-x86_64 6m24s; all-checks-pass).

## Scope Delivered

- **Master placeholder geometry** derived from `brand.page_size` (footer/body position + `.max(0)` saturating clamps).
- **Schema-invalid `<p:sldSz>` REMOVED** from `slideMaster1.xml` — element is invalid in `CT_SlideMaster` (ECMA-376 §19.3.1.42). `presentation.xml` parity pinned.
- **`progress_bar` named layout** (CL-20 "SF Progress Bar", idx 30 → `slideLayout31.xml`, rels-verified).
- **`lang` on every `a:rPr`** (title/subtitle/body) + universality test.
- **No-lang default converged to "en"** via `DEFAULT_DECK_LANG`; cross-surface identity confirmed (`<dc:language>` in `docProps/core.xml`).
- **SEC-096-001:** fail-fast `validate_lang_for_xml` at `export_inner` entry point.

## Pass Map

### Pass 1

**Findings:**

| ID | Severity | Description |
|----|----------|-------------|
| F-096-001 | HIGH | Title and subtitle `a:rPr` lang not propagated — only body runs received lang |
| F-096-002 | MED | No-lang default divergence — code used "en-US" in one surface, spec requires "en" |
| F-096-003 | MED | Footer placeholder bottom edge exceeded 16:9 bounds under some page-size inputs |
| F-096-004 | LOW | Missing `.max(0)` clamp on `footer_zone_top` computation |
| F-096-005 | LOW | `progress_bar` idx hard-coded to 30 rather than resolved from layout map |

**Fix commits:** `7d5ec6a7` + `7c81a6a6`

### Pass 2

**Findings:**

| ID | Severity | Description |
|----|----------|-------------|
| F-096-P2-001 | MED | Stale "en-US"-default prose in `serialize_slide_to_xml` doc-comment — post-fix comment still claimed default was "en-US" |
| F-096-P2-002 | MED | Stale "en-US"-default prose in `apply_lang_to_run` — same historical-voice doc narration |

**Fix commits:** `84d4cc0a` + `a3dbc4f8`

### Pass 3

**Result:** strict-CLEAN. Streak: 1/3.

### Pass 4

**Findings:**

| ID | Severity | Description |
|----|----------|-------------|
| F-096-A001 | CRITICAL | `slideMaster1.xml` emits `<p:sldSz>` — element is schema-invalid in `CT_SlideMaster` per ECMA-376 §19.3.1.42; triggers PowerPoint repair dialog |
| OBS-1 | OBS | Minor observation: inline comment cited ECMA section number without full title |

Streak reset to 0/3 by CRITICAL finding. PO adjudication triggered for AC-001 rewrite.

**Fix commit:** `352ea217`

**PO Adjudication (a) — AC-001 rewrite:** AC-001 was amended from "sldSz derived from brand.page_size" to "sldSz in presentation.xml ONLY; master ASSERTS ABSENCE." BC-5.01.005 v1.2 → v1.3 (no-lang default "en" on ALL surfaces). Story spec v1.0 → v1.1 (then v1.2). STORY-099 v1.1 + STORY-100 v1.1 swept; STORY-100 gained AC-006/EC-005 HTML lang + T-008/T-009, BC-5.01.005 added to its frontmatter.

### Pass 5

**Findings:**

| ID | Severity | Description |
|----|----------|-------------|
| F-096-A005 | MED | Test naming: `test_master_no_sldsz` did not reflect TIMELESS AC assertion in name |
| F-096-A006 | MED | Doc claim in `master_serializer.rs` stated "sldSz derived from brand" — post-fix, the master does NOT emit sldSz at all; doc was factually wrong |
| F-096-A007 | LOW | Test module comment still used historical narration voice ("previously hardcoded") |

**Fix commit:** `13d7e338`

### Pass 6

**Findings:**

| ID | Severity | Description |
|----|----------|-------------|
| F-096-P6-001 | HIGH | Module traceability table in `master_serializer.rs` listed 9 functions but test matrix had 11 rows — count and name mismatch introduced during pass-4/pass-5 fix-burst |
| OBS-P6-1 | OBS | One table row had a stale function name from a rename |

Streak reset to 0/3. Traceability reconciliation required.

**Fix commit:** `6e29bce1`

### Pass 7

**Result:** strict-CLEAN. Streak: 1/3.

### Pass 8

**Result:** strict-CLEAN. Streak: 2/3.

### Pass 9

**Result:** strict-CLEAN. Streak: 3/3. **CONVERGED.**

## Standing Adjudications (Carried Forward Through Cascade)

| ID | Ruling |
|----|--------|
| CL-20 slot reuse | `progress_bar` at idx 30 reuses the existing CL-20 layout slot — intentional per the layout index table; not a conflict |
| EC-003 guard-by-design | EC-003 (empty slide no rPr error) is satisfied by guard-by-design in the XML builder; no additional test required |
| HTML default → STORY-100 | HTML `lang` default propagation deferred to STORY-100 (scoped to HTML exporter) — not in-scope for STORY-096 |
| "Red Gate" assertion labels acceptable | Test names containing "Red Gate" / "RED" in assertion comments are acceptable; timeless prose in the function body is what matters |
| rPr lang escaping non-issue | BCP-47 tag values (e.g. "en-US", "fr-FR") do not require XML attribute escaping; hyphen is valid in attribute values |

## PO Adjudications

**Adjudication (a) — no-lang default "en" on ALL surfaces:**
- BC-5.01.005 v1.2 → v1.3: default when no lang declared is "en" (not "en-US") across all export surfaces.
- Story v1.0 → v1.1 (initial adjudication) → v1.2 (LESSON-19 sweep complete).
- LESSON-19 sweep: STORY-099 v1.1 updated; STORY-100 v1.1 updated + gained AC-006/EC-005 HTML lang + T-008/T-009 + BC-5.01.005 in frontmatter.

**Adjudication (b) — AC-001 sldSz placement:**
- AC-001 rewritten: `<p:sldSz>` belongs in `presentation.xml` ONLY. `slideMaster1.xml` ASSERTS ABSENCE. Schema-invalid element removed from master.
- Story spec changelog: v1.2 (product-owner — CRITICAL schema fix, ECMA-376 §19.3.1.42).

## Security Review

- Initial pass: CLEAN.
- Re-verify at `9c8e1e1e` (post-pass-6 fix): CLEAN. SEC-096-001 validated as resolved.
- **SEC-096-001:** `validate_lang_for_xml` fail-fast guard at `export_inner` entry — catches malformed BCP-47 tags that would embed literal `lang="` sequences in output XML without escaping. Closed in-scope.

## PR Review

pr-reviewer APPROVE. 3 non-blocking nits:
- **N-1:** Minor doc phrasing in `layout_xml.rs` — "4:3 same height" comment imprecise. Logged as FU-096-43-DOC-NOTE.
- **N-2:** Dead static `cy` value for `MASTER_PLACEHOLDER_DEFS` body entry idx==1 — no compile-time signal it is unused. Logged as FU-096-DEAD-STATIC-CY.
- **N-3:** Observation on test module organization (non-blocking). Accepted as-is.
