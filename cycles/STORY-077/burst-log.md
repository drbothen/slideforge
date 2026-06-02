# STORY-077 Burst Log

Narrative history for STORY-077 (SectionBlock IR Extension + Inline-Markup Parser).
Archived from STATE.md Decisions Log + supplemental sections per content-routing rules.

---

## Decisions Log — Historical Entries (archived 2026-06-02)

- 2026-05-31 — WAVE 4 STARTED. STORY-077 created (P0, 8pts, blocks 041/042). Total: 80 stories/473 pts. 16 crates.
- 2026-06-01 — STORY-077 BLOCKED: parser gap (STORY-027 decomp gap). STORY-078 created (5pts, P0, EPIC-02). DIR-077-001 + DIR-077-001-A issued.
- 2026-06-01 — STORY-077 fresh delivery from develop. Red Gate done (16 RED). Impl green (fd539c4e). Adversary pass 1: 1 CRIT (F-077-P1-001 eval/layout section-type list divergence rejecting executive_summary/risk_register), 2 HIGH (paper-tested interpolation; untested EC-002 undefined-var), 2 OBS (EC-006 spec contradiction; missing layout regression test). Fix-burst in progress. 0/3 streak.
- 2026-06-01 — Session lessons LESSON-1..LESSON-6 codified.
- 2026-06-02 — STORY-077 pass-3 CLEAN (strict: yes, PR-merge: yes). Streak 1/3. Pass-1 fix-burst (CRIT F-077-P1-001 section-type SSOT) + pass-2 fix-burst (register-key SSOT alias, BC corrections, OBS test gaps) both complete. BC-3.02.002 bumped v1.3→v1.4 (recognized-type list 5→7: +executive_summary, +risk_register). BC-1.14.003 bumped v1.2→v1.3 (subsystem SS-TBD→SS-02). STORY-077 bumped v1.3→v1.4 (EC-006 re-scoped top-level-only directive). All committed this burst. Pass 4 NEXT.
- 2026-06-02 — STORY-077 scope expansion (human decision 2026-06-02). DIR-077-002 issued: inline-markup parser (TemplateChunk variants + parser + chunks_to_inline_nodes in slideforge-eval) added to STORY-077 scope. Story v1.4→v1.5, 8→13 pts, est_days 3→5. BC-3.02.002 bumped v1.4→v1.5 (PC8 inline-markup clarification). Pass-5 finding F-077-P5-001 now properly resolved by expansion scope; streak reset 0/3. Worktree HEAD c7c1ae6f (section IR + register routing complete).
- 2026-06-02 — STORY-077 inline-markup cascade rounds: pass-1 (F-077-P5-001 Expr::Call unreachable fixed), pass-2 (F-077-P2-001 genuine parse→eval test; F-077-P2-002 delimiter spans; underscore regression tests), pass-3 CLEAN (1/3 streak). E-EVL diagnostic-code collision resolved → E-EVL-012 (FigrefInvalidArg) + E-EVL-013 (InlineXrefEmptyId). E-PAR collision resolved → E-PAR-019 (unclosed inline markup) + E-PAR-020 (empty inline markup span). error-taxonomy bumped v2.8→v2.10. Pass-4 fix-burst (kind-based sentinel routing). Pass-5 found F-077-P5-001 [HIGH]: sentinel leaks into SyntaxError message field. HEAD b5a161ac (not pushed). Streak 0/3.
- 2026-06-02 — F-077-P5-001 FIXED at 6c72e686. Commit: "fix(syntax): eliminate routing sentinel leak in inline markup diagnostics (F-077-P5-001)". strip_custom_wrapper() strips the chumsky `Custom("…")` wrapper from ALL error reason strings; InlineMarkupRoute carries original_msg; mod.rs passes original_msg not the tagged blob. Bonus: fixes same leak on E-PAR-012/013/014. HEAD 6c72e686 (not pushed). Streak 0/3. Pass 6 NEXT.

---

## STORY-045 Forward-Obligations (archived 2026-06-02 — STORY-045 MERGED)

STORY-045 merged at PR #48, develop e5d818e7. All obligations below are resolved.

- **From STORY-036:** contiguous-run sentinels for register content — DONE (STORY-045 wired).
- **From STORY-043:** real frame-level Diagram/Chart alt text wired through SlideTagEngine — DONE (STORY-045 wired real alt text from IR frames).
- **From STORY-044 (F-P19 cascade):**
  1. `decorative_frame_indices` → emit `ContentTag::Artifact(ArtifactType::Other)` for decorative Image/Shape — DONE.
  2. `draw_body_blocks` single shared baseline → per-block/per-line baselines — DONE (F-P2-001 fixed 2026-06-01: per-block/bullet baseline stacking with BODY_LINE_LEADING=1.2).
  3. `text_fill_black()` → text color from brand/theme palette — RE-COUPLED to future bg-fill story per orchestrator ruling 2026-06-01. STORY-045 draws no background fills, so black-on-dark contrast defect cannot occur; PDF/UA-1-compliant. Future adversary passes on STORY-045 must not re-flag this.

---

## Process Wins (archived 2026-06-02)

- Pre-implementation tech-validation (research-agent) + architect coordinate-model directive for new-dependency stories catches library-vs-spec coordinate bugs before implementation.
- Test-utils feature-gating pattern (STORY-036 BleedChecker) avoids test-only deps leaking into production builds.
- SVG paint-state isolation pattern (STORY-044): explicitly set fill + clear stroke before every text draw.
- Security re-review after targeted MED fix (STORY-076): fix-then-re-review pattern confirms no new surface introduced.
- STORY-045 orchestrator adjudication pattern: when adversary obligation text conditions its own fix on another concurrent change, route the obligation to the story that triggers that change — not a deferral, faithful to the obligation's own predicate.
