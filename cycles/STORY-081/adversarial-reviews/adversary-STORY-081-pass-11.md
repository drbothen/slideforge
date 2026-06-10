---
document_type: adversary-pass-report
story_id: STORY-081
pass: 11
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: f7c26fba
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 3
findings_crit: 0
findings_high: 1
findings_med: 1
findings_low: 1
findings_obs: 1
findings_open: 0
findings_remediated: 3
---

# Adversary Pass 11 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3.**

Fresh-context re-derivation at HEAD f7c26fba (restart of cascade after Pass-10 reset). Verified the 3 prior fix-changes (PPTX body a:highlight, font.rs double-load, diagrams deterministic gates) are each independently SOUND and load-bearing; OBS-P09/P10-001 fully remediated (NOT carried). Surfaced 3 new findings + 1 process-gap OBS.

## Findings

### ADV-P11-HIGH-001 [HIGH] — incomplete sibling fix: PPTX notes path emits schema-invalid `highlight="yellow"` ATTRIBUTE
- The Pass-10 body highlight fix missed the NOTES sibling. `slideforge-plugin-api/src/inline_formats/default_formatter.rs:~260` (`emit_run`, shared by notes via notes_slide.rs:309) emitted `highlight="yellow"` as an ATTRIBUTE on `<a:rPr>`. DrawingML CT_TextCharacterProperties has NO such attribute (verified ooxmlsdk schema). Highlight silently fails to render in notes (BC-3.02.002 PC8 violation). F-003 test + committed `.snap` locked in invalid bytes (false-green, TD-VSDD-059). Direct TD-VSDD-060 sibling-sweep failure of the fix under review.
- REMEDIATED (commit 34c128b2): emit_run now emits `<a:highlight><a:srgbClr val="FFFF00"/></a:highlight>` CHILD element in correct child-order; docstring matrix corrected; Red-Gate test `test_bc_5_02_001_ooxml_highlight_child_element_not_attribute` + updated `test_f003_ac006_notes_highlight_variant_in_ooxml` (assert child present + attribute absent); snapshot regenerated schema-correct.

### ADV-P11-MED-001 [MED] — PPTX body path missing distinguishing assertions for Code/Superscript/Subscript/Link; spec-planned inline_markup_pptx_snapshot.rs never created
- Body make_run code for Code/Super/Sub/Link was present (Code/Super/Sub correct) but had no load-bearing distinguishing assertion; notes-path tests exercise DefaultInlineFormat not make_run. Per-form FU-EXIT-GATE gap class (same as the P10 Highlight miss).
- REMEDIATED (commit 34c128b2 + 5556aa75): added body-path load-bearing assertions Code→`<a:latin typeface="Courier New"`, Super→`baseline="30000"`, Sub→`baseline="-25000"`. For Link, see ORCHESTRATOR-CAUGHT EC-004 note below.

### ADV-P11-LOW-001 [LOW] — font.rs LOAD_SYSTEM_FONTS_COUNT docstring overstated its invariant
- REMEDIATED (commit 34c128b2): wrapped load in private `load_system_fonts_counted` helper that always increments; docstring now structurally accurate.

## Observations

### OBS-P11-001 [OBS — process-gap]
PPTX inline-markup has TWO independent OOXML run-generators for the same InlineNode set: `slide_serializer::make_run` (typed ooxmlsdk, body) and `DefaultInlineFormat::emit_run` (raw-string, notes). They already diverged on Highlight (P11-HIGH-001) and are at structural risk of diverging on every future inline-form change; the raw-string path bypasses ooxmlsdk schema-enforced child ordering. Recommend architectural unification (route notes through the typed builder) or a cross-path equivalence test. Process/architecture gap → follow-up FU-PPTX-DUAL-RUN-GENERATOR. Does NOT block convergence streak (OBS).

## Trajectory
P2(5)->P3(3)->P4(2)->P5(1)->P6(1MED+2OBS)->P7(0,1/3)->P8(1HIGH,reset)->P9(0,1/3)->P10(1HIGH,reset)->P11(1HIGH+1MED+1LOW,0/3)

## Next Step
All P11 findings remediated. Orchestrator validation of the Link fix surfaced a deeper EC-004 violation (see below). Restart cascade — adversary Pass 12 fresh at HEAD 5556aa75.
