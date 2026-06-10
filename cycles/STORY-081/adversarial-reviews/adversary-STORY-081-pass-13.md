---
document_type: adversary-pass-report
story_id: STORY-081
pass: 13
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: 5556aa75
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 2
findings_crit: 0
findings_high: 0
findings_med: 2
findings_low: 0
findings_obs: 2
findings_open: 0
findings_remediated: 2
---

# Adversary Pass 13 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3 (RESET from 1/3).**

Fresh-context independent re-derivation at HEAD 5556aa75 (unchanged since Pass 12 clean, LESSON-7). Re-verified prior-pass-heavy axes (single-form rendering, EC-004 body-link, notes <a:highlight>, PDF measure==draw) sound. New value on the COMBINED/COMPOUND-form axis: surfaced a real cross-format parity defect + the exit-gate gap that hid it.

## Findings

### F-P13-001 [MEDIUM] — DOCX silently drops inline formatting applied to a hyperlink (cross-format parity break)

- `crates/slideforge-docx/src/document_body.rs:602-616` `apply_run_property` `other => other` arm passes `ParagraphChoice::WHyperlink` through unchanged. For `Bold([Link])` (`**[click](https://x)**`, reachable per template_inline_markup_tests.rs:3660), the `<w:b/>` is NEVER applied — styling silently discarded, no warn, no test. PPTX-body/HTML/PDF all preserve bold-on-link; DOCX is the lone divergent exporter. Production-grade no-silent-drop violation.
- REMEDIATED (commit eec4be32): WHyperlink arm now iterates `hyperlink.hyperlink_choice` and applies the run-property closure to each inner WR run; r:id clickability preserved (on the <w:hyperlink> element). apply_run_property signature widened FnOnce→Fn. Red-Gate tests `test_f_p13_001_docx_bold_wrapping_link_preserves_both_wb_and_hyperlink` + italic variant (failed pre-fix, pass post-fix).

### F-P13-002 [MEDIUM][FU-EXIT-GATE-DISTINGUISHING-OUTPUT] — no distinguishing combined-form assertion on PPTX-body/DOCX/HTML/PDF (only notes path, via STORY-085, had combined coverage)

- Every combined cell (bold+italic, bold+link, BoldItalic-collect, nested) was untested on the 4 STORY-081 exporter surfaces; this gate gap hid F-P13-001.
- REMEDIATED (commit eec4be32): added load-bearing combined-form assertions — PPTX `Bold([Italic])`→`b="1"`∧`i="1"`; DOCX→`<w:b/>`∧`<w:i/>` same rPr; HTML→nested `<strong><em>`; PDF→`slide_to_krilla_runs(Bold([Italic]))`→`FontFaceKind::BoldItalic`. Plus e2e fixture extended (slide 3 bold-link `**[click](https://example.com)**`, slide 4 nested `**_bold and italic_**`) with 6 cross-exporter parity assertions.

## Observations

### OBS-P13-001 [OBS — process-gap] (carries OBS-P11-001/P12-001)

PPTX retains two independent OOXML run-generators (typed body make_run vs raw-string notes emit_run); divergence-prone (the P11-HIGH highlight bug + this DOCX-link drop both lived in non-typed/secondary paths). FU-PPTX-DUAL-RUN-GENERATOR. Non-blocking OBS.

### OBS-P13-002 [OBS — within AC-004 scope, NOT a finding]

PDF Strikethrough/Highlight (slide_pdf.rs:282-288) recurse with inherited face, emit no visual strike-line/highlight bg, no warn. AC-004 only requires Bold/Italic/Code + super/sub for PDF; strike/highlight are NOT PDF-required. Spec-compliant; noted only if a future parity tightening promotes it.

## ORCHESTRATOR-FLAGGED for Pass 14 adjudication

Post-fix, the PPTX BODY path omits `<a:hlinkClick>` for NESTED (non-top-level) links (e.g. `Bold([Link])`) — so `**[click](url)**` renders bold but NON-clickable in PPTX, while DOCX (post-F-P13-001) and HTML render it bold AND clickable. Implementer cites F-085-P6-001 "top-level-only" scope (chosen for notes to avoid orphan-rel invariant violation, mirrored into the body EC-004 collection). UNRESOLVED QUESTION for Pass 14: is "top-level-only" a spec-sanctioned limitation, or a cross-format parity defect (PPTX silently dropping a wrapped link's URL)? Pass 14 must adjudicate against the story spec + EC-004 + any F-085 decision record.

## Trajectory

P2(5)->P3(3)->P4(2)->P5(1)->P6(1MED+2OBS)->P7(0,1/3)->P8(1HIGH,reset)->P9(0,1/3)->P10(1HIGH,reset)->P11(1HIGH+1MED+1LOW,0/3)->P12(0,1/3)->P13(2MED,reset 0/3)

## Next Step

All P13 findings remediated at HEAD eec4be32. Restart cascade — adversary Pass 14 fresh, with explicit adjudication of the nested-link PPTX parity question above.
