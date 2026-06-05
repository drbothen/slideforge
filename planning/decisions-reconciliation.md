---
title: "Planning Decisions — Reconciliation Log"
status: LIVING DOCUMENT
purpose: >
  Records human-authorized decisions that resolve same-precedence conflicts between
  planning artifacts (q-decision files, taxonomy files, research docs). Each entry
  identifies the conflict, the ruling, the provenance, and the downstream artifacts
  updated as a result. Do NOT silently edit one doc to match another without recording
  the decision here first.
format_note: >
  Entries are append-only. Never delete or edit a prior entry. Mark superseded entries
  with [SUPERSEDED by DREC-NNN] if a later ruling changes an earlier one.
---

# Planning Decisions Reconciliation Log

---

## DREC-001 — Slide Sections Version: v1.0 Confirmed (Overrides Taxonomy v1.x)

**Date:** 2026-06-04
**Decided by:** Human (Joshua Magady)
**Authority level:** Human ruling — BINDING, overrides same-level planning artifacts

### Conflict Identified

Two same-precedence planning artifacts disagreed on the target version for PPTX slide
sections (`<p:sectionLst>`):

| Artifact | Classification | Version |
|----------|---------------|---------|
| `planning/pptx-element-taxonomy.md` (Row: "Slide sections") | "Fair -- PPT only" | **v1.x** |
| `planning/q1-decision-final.md` Section 7 (PowerPoint Elements) | Listed in "35 v1.0-core" prose | **v1.0** |

Per CLAUDE.md Source-of-Truth Precedence rules: these two files sit at the same level
(both are planning research/decision artifacts). Neither automatically supersedes the
other. The conflict required human adjudication.

### Human Ruling

**Slide sections (PPTX `<p:sectionLst>`) are confirmed v1.0.** The q1-decision-final.md
Section 7 listing stands. The pptx-element-taxonomy.md v1.x classification is
overridden by this ruling.

Rationale stated by the human: the feature is worth delivering in v1.0 despite being
PPT-only. Non-PowerPoint renderers ignore `<p:sectionLst>` without rendering corruption
(graceful degradation). The organizational benefit in PowerPoint 365 justifies the
element.

### Accepted Exceptions

1. **Multi-renderer parity exception:** `<p:sectionLst>` is exempt from the CLAUDE.md
   multi-renderer parity quality gate (4 target renderers: PowerPoint, Keynote, Google
   Slides, LibreOffice). It renders only in PowerPoint 365. This is a DOCUMENTED,
   HUMAN-ACCEPTED departure from full multi-renderer parity, not a gap.

2. **Taxonomy annotation:** The v1.x classification in pptx-element-taxonomy.md is NOT
   changed (the taxonomy file is a research artifact, not a decision document). Instead,
   this reconciliation record is the source of truth for the version decision. Readers
   of pptx-element-taxonomy.md should consult this file when they see a discrepancy
   with q1-decision-final.md.

### Delivery

Delivered via TWO stories (human-authorized scope split, same session):
- **STORY-040** — Speaker notes + notesMaster/handoutMaster (ships immediately)
- **STORY-08x** — Slide grouping sectionLst (follow-up v1.0 story; story-writer assigns
  final ID). Requires: new DSL syntax (`section "Name":` quoted-name slide-grouping
  construct), `slide_sections` field added to `LaidOutDeck` IR, eval-stage membership
  mapping.

### Downstream Artifacts Updated

| Artifact | Change |
|----------|--------|
| `.factory/specs/behavioral-contracts/BC-4.01.003.md` | v1.1 → v1.2; scope split documented; parity exception recorded; BC-1.14.003 non-interference clarified; changelog entry added |
| `planning/decisions-reconciliation.md` (this file) | DREC-001 created |

### What Was NOT Changed

- `planning/pptx-element-taxonomy.md` — left intact as research artifact; the v1.x row
  is a historical research classification, not an override target. This reconciliation
  record is the authoritative override.
- `planning/q1-decision-final.md` — left intact; the Section 7 "35 v1.0-core" prose
  already includes slide sections. No edit needed.
- `BC-1.14.003` — left fully intact. The non-interference constraint (sectionLst must
  not read section body or register_content) is cited in BC-4.01.003 as a cross-reference,
  not a change to BC-1.14.003 itself.

---
