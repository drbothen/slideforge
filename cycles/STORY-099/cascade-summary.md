---
story: STORY-099
title: "REND-006: DOCX bullet run content + numbering.xml + sectPr + lang"
epic: EPIC-09
points: 8
priority: P0
pr_number: 89
merge_sha: ca412f2a
develop_sha_after: ca412f2a70c49ed710005df7f7ddac2f8bc98595
cascade_passes: 6
findings_closed: 4
convergence: "CONVERGED 3/3 strict-CLEAN (passes 4-5-6)"
date: 2026-06-12
---

# STORY-099 Cascade Summary

## Merge Facts

- PR #89 squash-merged → develop `ca412f2a70c49ed710005df7f7ddac2f8bc98595`
- REND-006 CLOSED. Rendering-fix wave: 5 of 9 complete.
- Workspace tests at tip: 4228 pass / 20 skip / 0 fail. CI green (linux-x86_64, 6m3s).
- Worktree/branches deleted. `.worktrees/` EMPTY.

## Deliverables (What Shipped)

- Bullet paragraphs: `<w:numPr>` (ilvl 0-2 derived from indent) + real `<w:r><w:t>` run content.
- `word/numbering.xml`: populated abstractNum (•/◦/▪ 3 levels: ilvl=0,1,2) + concrete num (numId=1). doc↔numbering cross-resolution verified.
- `sectPr` as final body child: `<w:pgSz w:w="14400" w:h="8100"/>` (EMU/635 exact, remainder=0).
- `<w:lang>` on EVERY `<w:rPr>` including BOTH section serializers (AutoSection + ManualSection paths).
- Shared `DEFAULT_DECK_LANG="en"` constant + shared `validate_xml_lang` validator moved to `slideforge-types`; PPTX delegates to shared constant (no behavior change).
- `docProps/core.xml` default language converged: `<dc:language>en</dc:language>` (not `en-US`).
- SEC-099-001 (CWE-116): fail-fast lang validation at DOCX export entry via `ExportError::InvalidLanguageTag` (mirroring SEC-096-001 pattern from PPTX).

## Pass Map (6 passes)

### Pass 1 — 3 findings

| ID | Severity | Description | Disposition |
|----|----------|-------------|-------------|
| F-099-001 | HIGH | Section serializer runs (AutoSection + ManualSection) emitted runs without `<w:lang>` — lang universality gap | Fixed in commit `a3c2880e` |
| OBS-099-001 | MED | `docProps/core.xml` default language "en-US" instead of "en" for no-lang deck — cross-surface consistency gap | Fixed in commit `a3c2880e` |
| OBS-099-002 | OBS | `ilvl` clamp at 2 for deeply-nested bullets (ilvl≥3 clamped silently) | **Adjudicated acceptable** (see Standing Adjudications) |

Fix commit: `a3c2880e` — section serializers threaded lang; core.xml default converged.

### Pass 2 — 1 finding

| ID | Severity | Description | Disposition |
|----|----------|-------------|-------------|
| F-099-002 | MED | Story spec AC-004 mis-anchored to BC-3.05.001; correct anchor is BC-5.01.005 PC-4 (lang propagation to DOCX run-level `<w:lang>`) | Fixed: story-writer dispatched; story v1.1→v1.2, BC-3.05.001 citation REMOVED, BC-5.01.005 traceability swept; STORY-INDEX + dependency-graph updated; 4 tests renamed in `commit 83f97b94`; traceability table 12↔12 reconciled in commit `7bf5d623` |

Spec fix commits:
- `83f97b94` — 4 tests renamed to trace to BC-5.01.005 (not BC-3.05.001); story v1.1→v1.2.
- `7bf5d623` — STORY-INDEX + dependency-graph depends_on + BC-5.01.005 traceability table 12↔12 sweep.
- `.factory` fix (factory-artifacts) — dependency-graph `depends_on` row updated: STORY-081 → STORY-073 + STORY-085.

### Pass 3 — 1 finding

| ID | Severity | Description | Disposition |
|----|----------|-------------|-------------|
| F-099-003 | LOW | `dependency-graph.md` `depends_on` row for STORY-099 listed STORY-081; correct deps are STORY-073 + STORY-085 | Fixed in `.factory` burst (factory-artifacts) |

### Pass 4 — CLEAN (strict)

Zero findings of any severity.

```
CLEAN (strict): yes
CLEAN (PR-merge): yes
```

Streak: 1/3.

### Pass 5 — CLEAN (strict)

Zero findings of any severity.

```
CLEAN (strict): yes
CLEAN (PR-merge): yes
```

Streak: 2/3.

### Pass 6 — CLEAN (strict) — CONVERGENCE

Zero findings of any severity.

```
CLEAN (strict): yes
CLEAN (PR-merge): yes
```

Streak: 3/3. **CONVERGED.**

## Standing Adjudications

- **OBS-099-002 (ilvl≥3 clamp acceptable):** Deeply nested bullets beyond ilvl=2 are silently clamped to ilvl=2. The three-level bullet schema (•/◦/▪) is the DSL design decision from q2-decision-final.md. Word/LibreOffice both render 3 levels correctly. Deeper nesting is not a specified requirement in any AC or BC. Adjudicated as acceptable behavior at pass 1.
- **"Red Gate" assertion labels:** Timeless-voice pass requires that test assertions read as invariant statements. Test names using the prefix `test_BC_` and assertion messages describing postconditions (not implementation steps) are compliant.
- **AC-005 structural schema proxies acceptable for unit scope:** Unit tests that assert namespace declarations, sectPr final-child position, and numbering completeness via substring checks on raw XML are accepted as valid structural schema proxies. Full schema validation (veraPDF/OOXML validator) is a Phase 6 gate item.
- **sectPr twips defensive fallback:** When page dimensions are unavailable (e.g., headless test without full brand config), a defensive fallback to DEFAULT_PAGE_WIDTH/HEIGHT constants is acceptable provided the constants are the canonical EMU values (9_144_000 × 5_143_500) and the twips conversion uses integer division by 635.

## Security Review Flow

- **SEC-099-001 [IMPORTANT, CWE-116]:** DOCX export entry point lacked fail-fast validation of the language tag before writing it into XML attributes. An attacker-controlled language tag could embed XML-special characters (`<`, `>`, `"`, `&`) into `<w:lang w:val="..."/>`, producing malformed XML. Fix: `validate_xml_lang` function added to `slideforge-types`; called at DOCX export entry → returns `ExportError::InvalidLanguageTag` on invalid input. Commit `3e6664ba`.
- **SEC-099-002 [suggestion]:** Language tag regex pattern could be tightened beyond basic XML-char rejection to enforce BCP-47 structure. Adjudicated acceptable as suggestion; fix deferred — logged as potential follow-up if language tags from untrusted sources become a threat vector. No current story vehicle; not blocking.

Security re-verification after `3e6664ba`: CLEAN. Both findings resolved before merge.

## PR Review Findings (pr-reviewer, non-blocking)

Three nits received; none blocking merge:

1. **F1 [nit]:** Two `insta` snapshot assertions include `assertion_line` metadata in the snapshot header — strip for cleaner diffs. Logged as FU-099-INSTA-ASSERTION-LINE.
2. **F2 [nit]:** EC-005 (empty-bullet) test assertion checks for `<w:r>` presence without isolating the specific empty-bullet run from adjacent runs — more precise isolation recommended. Logged as FU-099-EC005-TEST-PRECISION.
3. **F3 [nit]:** PR body test count stated 10 tests; actual module has 12. Stale count in PR prose only (no code impact). No vehicle needed; logged as informational.

## Demo Evidence

File: `.factory/demos/STORY-099-demo-evidence.md`

| AC | Result |
|----|--------|
| AC-001 (bullet run content + numPr) | PASS |
| AC-002 (numbering.xml abstract + concrete) | PASS |
| AC-003 (sectPr twips dimensions) | PASS |
| AC-004 (lang universality + default "en") | PASS |
| AC-005 (structural schema proxies) | PASS |

12 Red Gate tests: 12 pass, 0 fail (0.057s).

## Workspace Tests at Merge

- develop `ca412f2a`: 4228 pass / 20 skip / 0 fail. CI green.
