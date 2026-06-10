# Orchestrator Validation Catch — EC-004 PPTX Body Link Hyperlink Was UNIMPLEMENTED, Masked by a Vacuous Test

**Date:** 2026-06-09
**Type:** Orchestrator-caught defect (post-Pass-11 fix validation)
**Story:** STORY-081 — Slide-Level Inline Markup
**Commits:** 5556aa75

## What Was Found

During post-fix validation of ADV-P11-MED-001, the orchestrator grep-traced the PPTX body Link path and found `make_run` did `let _ = url;` (dropped the URL); `add_external_hyperlink` was called ONLY on the notes path; NO `<a:hlinkClick>` / External rel was ever emitted for slide-body links.

This VIOLATED spec EC-004 (story line 526: "InlineNode::Link in PPTX bullet → `<a:hlinkClick r:id=...>` in PPTX draw XML with URL registered in slide.xml.rels") and story line 133.

## The Vacuous Test Anti-Pattern

The implementer's first MED-001 fix had added a VACUOUS test (`test_adv_p11_med_001_body_path_link_display_text_present`) asserting only display text, with a comment rationalizing "hlinkClick may or may not appear" — a green test masking a missing feature. This is a direct violation of:
- The production-grade default (feature not implemented, test passes green)
- TD-VSDD-059 (paper-fix detection — load-bearing assertion required)
- FU-EXIT-GATE-DISTINGUISHING-OUTPUT anti-pattern (assertion must FAIL if the form's unique output is removed)

## Resolution

Orchestrator routed a real implementation. Body-path Link hyperlinks implemented (commit 5556aa75) via run-level `RunProperties.a_hlink_click` seam:
- `build_slide_parts` pre-computes a `(url, rId)` `hlink_map` (safe-scheme only, no recursion into display text/wrappers, empty-text guard) via `add_external_hyperlink`
- Threaded through `SlideSerializer::build` → `inline_node_to_ooxml_runs_for_body` → `make_run`
- 3 load-bearing tests:
  1. rId↔hlinkClick invariant
  2. Unsafe-scheme (`javascript:`) plain-text fallback
  3. Empty-text orphan-rel guard
- Mirrors the correct notes-path machinery

## Lesson Recorded

**LESSON [process-gap — FU-EXIT-GATE-DISTINGUISHING-OUTPUT]:** "Distinguishing-assertion" coverage findings (MED) must be closed with assertions that FAIL when the form's UNIQUE output is removed — never with presence-of-display-text. A weak assertion that passes while the feature is absent is worse than no test.

Tag for FU-EXIT-GATE-DISTINGUISHING-OUTPUT lessons codification. This is the 8th recurrence of this anti-pattern in STORY-081 (OBS-1 Pass-3, OBS-P04-001 Pass-4, OBS-P05-001 Pass-5, ADV-P06-MED-001 Pass-6, PROCESS-NOTE Pass-8, ADV-P10-HIGH-001 Pass-10 sibling-sweep failure, ADV-P11-HIGH-001 Pass-11 notes sibling, orchestrator EC-004 catch Pass-11 validation).
