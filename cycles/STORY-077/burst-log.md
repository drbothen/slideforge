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

## STORY-077 Inline-Markup Cascade — Passes 6–21 (archived 2026-06-02)

Cascade ran from 6c72e686 through 9c862fef on feature/S-077 (LOCAL-ONLY).
Full detail in git log of feature/S-077. Condensed narrative below.

- **Pass 6:** F-077-P6-001 (LOW) misleading comment + dead `into_message()` → fixed 48003e86.
- **Pass 7:** F-077-P7-001 (CRIT) UTF-8 char-boundary panic in `scan_template_chunks` + F-077-P7-002 (MED) unbounded recursion → E-PAR-021 depth cap (MAX_INLINE_NESTING=64) + 3 LOW → fixed ed56066d. error-taxonomy advanced: E-PAR-021 registered.
- **Pass 8:** F-077-P8-001 (MED) E-PAR-021 wrong structured code (emitted generic SyntaxError) → dedicated SyntaxError variant + accumulation parity → fixed ad321f52.
- **Pass 9:** F-077-P9-001 (MED) `ref()`/`footnote()` zero-arg silently dropped → E-EVL-013/E-EVL-014 + comment fixes → fixed 5427d854. error-taxonomy advanced: E-EVL-014 registered.
- **Pass 10:** F-077-P10-001 (HIGH) section `register_content` dropped at layout boundary → `GeneratedSection.register_content` propagation + eval coverage → fixed e205bec4. Adjudication: F-077-P10-001 IN-SCOPE (applies existing `LaidOutSlide.register_content` slide-path pattern; no new architecture decision).
- **Pass 11:** F-077-P11-001 (LOW) `figref()` empty-resolved arg → E-EVL-012 raised → fixed 3502d947.
- **Pass 12:** F-077-P12-001 (LOW) empty-arg guard missing on `Pipe` sibling arms → fixed 931d053e.
- **Pass 13:** F-077-P13-001 (LOW) layout `UnknownSectionType` hardcoded type list → SSOT-derived + drift-guard → fixed d433d019.
- **Pass 14:** F-077-P14-001 (HIGH) E-PAR-019/020/021 emitted as non-fatal warnings; spec mandates strict-build-fatal → routed to fatal errors; 14 tests rewritten → fixed 2d94395f. Adjudication: E-PAR-019/020/021 are strict-build-fatal (consistent with E-PAR-017; `--warn-only` demotion is a future global concern).
- **Pass 15:** F-077-P15-001/002 (MED×2) stale fatal-semantics rustdoc + non-load-bearing E-PAR-021 fatality test → fixed 4be94e4b.
- **Pass 16:** strict-CLEAN (streak 1/3). Build verified green.
- **Pass 17:** F-077-P17-001 (HIGH) slide-level inline-markup chunks silently dropped (text data loss) → `flatten_chunks_to_string` flat-text preservation per DIR-077-002 §4 → fixed 99b6262e. Streak reset to 0/3. Adjudication: slide-level flat-text preservation IN-SCOPE (DIR-077-002 §4 mandates `Value::Str` flat text); structural `InlineNode` upgrade deferred to STORY-081.
- **Pass 18:** F-077-P18-001 (MED) markup-wrapped brand-ref regressed to fatal error in set-rules → `brand_ref_field` shared predicate + `preserve_brand_ref` threading (set-rules only) → fixed 2dbce955. Adjudication: markup-wrapped brand-ref preserved in SET-RULES ONLY (slide/vars brand-ref preservation is NOT existing behavior; out of scope, STORY-081/PO).
- **Pass 19:** strict-CLEAN (streak 1/3). Build verified green (2595 passed / 3 skipped; 1 pre-existing perf flake `test_cold_budget_under_200ms` in slideforge-diagrams).
- **Pass 20:** strict-CLEAN (streak 2/3).
- **Pass 21:** F-077-P21-001 (MED) unclosed `[link](url` emitted no error; DIR-077-002 §5 mandates Fatal → strict-fatal E-PAR-019, all 8 delimiters now uniform → fixed 9c862fef. 4 new tests (unclosed-link). Workspace 2598/2599 (same pre-existing flake). Streak reset to 0/3.

**error-taxonomy status after pass 21:** v2.12 — E-PAR-019/020/021 + E-EVL-012/013/014 all registered this cascade.

---

## Post-Merge Burst (2026-06-03)

STORY-077 MERGED to develop as PR #49, squash commit c8913cad ("feat(syntax,eval,types,layout): SectionBlock IR extension + inline-markup parser (STORY-077)"). develop advanced e5d818e7 → c8913cad. develop PR count: 49.

Final cascade summary: 26 adversary passes. Streak achieved at passes 24-25-26 (3/3 strict-CLEAN). CI 16/16 green. Security-reviewer CLEAN (0 crit/0 important). PR-reviewer APPROVE (0 blocking).

Post-merge cleanup: feature/S-077 branch deleted (remote + local). Worktree `.worktrees/STORY-077` removed.

Wave 4 Batch A = 10/10 COMPLETE. All 10 stories merged (PRs #39–#49, develop c8913cad).

STATE.md updated: develop c8913cad, pr_count 49, wave_4_batch_a_complete 10/10, STORY-077 DONE, stale "RESUME HERE" turnkey section removed, Session Resume Checkpoint replaced.

STORY-INDEX.md: STORY-045 ready→merged, STORY-075 ready→merged, STORY-077 ready→merged.

Cycle-closing items recorded in cycles/STORY-077/lessons.md:
- SEC-002 (link URL no scheme validation): deferred to STORY-046 with concrete dependency [codified]
- OBS-077-P24-A (underscore italic flanking): pending-intent for PO + STORY-081 [recorded]
- OBS-077-P25-A (parse→eval round-trip test): optional follow-up [recorded]
- E-EVL-007..011 + E-PAR-012 taxonomy debt: follow-up story to be created (3+ recurrences → codified) [codified]
- LESSON-P077-A (fresh-context pass validates protocol): no process change, confirmation [codified]
- LESSON-P077-B (sibling enumeration sweep scope): refinement to TD-VSDD-060 sweep discipline [codified]

---

## STORY-077 Follow-Ups Burst (2026-06-03)

Follow-up cycle dispatched after STORY-077 post-merge burst. Four approved follow-ups shipped as PR #50.

**Scope of PR #50 (squash commit f2573bb1, "STORY-077 follow-ups"):**
- E-PAR-022 link URL scheme allowlist (http/https/mailto) — parse-boundary enforcement; closed SEC-002 earlier than the STORY-046 deferral.
- Italic bilateral flanking: DIR-077-002 §1 amended to add right-flanking guard parity with `*`; closed OBS-077-P24-A.
- Absolute error offsets in inline-markup scanner — corrected embedded byte-offset threading through closer-prediction machinery.
- Parse→eval round-trip integration test consolidating AC-002 seam coverage; closed OBS-077-P25-A.

**Spec artifacts shipped to factory-artifacts at 7f2e52d7 (PR-independent):**
- error-taxonomy v2.13: E-PAR-022 registered; E-EVL-007–011 registered; E-PAR-012 slot marked RETIRED + new code allocated; closed taxonomy debt item.
- DIR-077-002 §1 bilateral-flanking amendment committed to inline-markup-directive.md.

**Follow-up LOCAL adversarial cascade:**
- Pass 1: F-FU-P1-001 (MED) closer-prediction desync after offset threading change + F-FU-P1-B (minor) absolute-offset edge at EOF + OBS-FU-P1-A (lexer inner-quote) + OBS-FU-P1-C (link URL `)` truncation). Fixed.
- Pass 2: F-FU-P2-001 (MED) EOF unclosed-delimiter detection missed new absolute-offset invariant. Fixed.
- Pass 3: F-FU-P3-001 (LOW) stale rustdoc comment on offset parameter. Fixed. Streak 1/3.
- Pass 4: strict-CLEAN. Streak 2/3.
- Pass 5: strict-CLEAN. Streak 3/3.
- Pass 6 (PR-level, after push): strict-CLEAN.
- Pass 7 (post-CI verify): strict-CLEAN.

**CI:** 16/16 green. **Security-reviewer:** CLEAN (0 crit/0 important). **PR-reviewer:** APPROVE (0 blocking).

**Post-merge cleanup:** fix/story-077-followups branch deleted (remote + local). Worktree removed. Only main (develop @ f2573bb1) + .factory worktrees remain.

**develop advanced:** c8913cad → f2573bb1. develop PR count: 50.

**4 follow-ups CLOSED:** SEC-002, E-EVL-007..011/E-PAR-012 taxonomy debt, OBS-077-P24-A, OBS-077-P25-A.
**4 follow-ups TRACKED (new):** E-PAR-021 message cosmetic, OBS-FU-HTML-REDIR (CWE-601, deferred to STORY-046), OBS-FU-P1-A (inner-quote round-trip), OBS-FU-P1-C (URL `)` truncation).

**Lesson LESSON-FU-A recorded** (scanner cascade invariant analysis discipline — see lessons.md).

---

## Process Wins (archived 2026-06-02)

- Pre-implementation tech-validation (research-agent) + architect coordinate-model directive for new-dependency stories catches library-vs-spec coordinate bugs before implementation.
- Test-utils feature-gating pattern (STORY-036 BleedChecker) avoids test-only deps leaking into production builds.
- SVG paint-state isolation pattern (STORY-044): explicitly set fill + clear stroke before every text draw.
- Security re-review after targeted MED fix (STORY-076): fix-then-re-review pattern confirms no new surface introduced.
- STORY-045 orchestrator adjudication pattern: when adversary obligation text conditions its own fix on another concurrent change, route the obligation to the story that triggers that change — not a deferral, faithful to the obligation's own predicate.
