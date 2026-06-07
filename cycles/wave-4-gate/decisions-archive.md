# Wave 4 Gate — Decisions Archive

Entries migrated from STATE.md Decisions Log to keep STATE.md under 200 lines.
Archived: 2026-06-07. Covers STORY-087 per-pass entries, STORY-086 per-pass entries,
STORY-050/049/085/084/083 entries, and earlier Wave 4 decisions.

---

## Archived Entries (chronological, most recent first)

| Date | ID | Decision |
|------|-----|---------|
| 2026-06-06 | STORY-087-CONVERGED | STORY-087 LOCAL adversary cascade CONVERGED 3/3 strict-CLEAN (passes 8/9/10 per BC-5.39.001). Demo recorded: 3 VHS recordings at docs/demo-evidence/STORY-087/ covering F-G3-HIGH-003 closure, visible bar+label output, E-A11-002 + E-VAL-011 error paths. Worktree HEAD 96c2af83 → 15297d49. |
| 2026-06-06 | STORY-087-PASS10 | Adversary LOCAL pass 10 strict-CLEAN — streak 3/3. CASCADE CONVERGED. |
| 2026-06-06 | STORY-087-PASS9 | Adversary LOCAL pass 9 strict-CLEAN — streak 2/3. |
| 2026-06-06 | STORY-087-PASS8 | Adversary LOCAL pass 8 strict-CLEAN — streak 1/3. F-087-P7-001 (two-layer component cap) confirmed load-bearing. |
| 2026-06-06 | STORY-087-PASS7 | Adversary LOCAL pass 7 found F-087-P7-001 (MEDIUM): weighted_composite 6+ components emitted stray frame beyond 5-row maximum (BC-1.17.003 PC-9). REMEDIATED: two-layer cap in field_to_block.rs + defense-in-depth guard in layout.rs. Streak reset 0/3. |
| 2026-06-06 | STORY-087-PASS6 | Adversary LOCAL pass 6 strict-CLEAN — streak 1/3. OBS-P6-001 (status title geometry) + OBS-P6-002 (DOCX percent rounding) noted NON-blocking; deferred to VISUAL-REVIEW/Phase-4. |
| 2026-06-06 | STORY-087-PASS5 | Adversary LOCAL pass 5 found ADV-STORY087-P05-HIGH-001: make_missing_label_error emitted wrong BC-1.17.003 PC2 message format (no `<title-value>` token); AC-003/009 tests were non-load-bearing. Also LOW-001: make_missing_component_label_error had extra sentence not in PC4. REMEDIATED: message format corrected; load-bearing content assertions added. Streak reset 0/3. |
| 2026-06-06 | STORY-087-PASS4 | Adversary LOCAL pass 4 strict-CLEAN — streak 1/3. P4-OBS (value_range wording phrasing inconsistency with BC-1.17.003 PC3) corrected in-scope. |
| 2026-06-06 | STORY-087-PASS3 | Adversary LOCAL pass 3 found F-087-P3-001 (HIGH, TD-VSDD-059 paper-fix): ColorBar IR materialized correctly but DROPPED at serialization — PPTX/PDF/DOCX exporter arms were warn/no-op stubs; register_bundled_plugins registered 31 vs 34 (3 color-coded types missing). REMEDIATED: all exporters emit ColorBar; 3 color-coded types added to registry (count 31→34). BC-1.17.001/002/003 → v1.2.1. Streak reset 0/3. |
| 2026-06-06 | STORY-087-PASS2 | Adversary LOCAL pass 2 found F-087-P2-001 (empty components on weighted_composite undefined behavior). REMEDIATED. Streak reset 0/3. |
| 2026-06-06 | STORY-087-PASS1 | Adversary LOCAL pass 1 found F-087-P1-001 (CRITICAL): value-range validation in progress_bar missing bounds check. REMEDIATED. Streak 0/3. |
| 2026-06-06 | STORY-087-PULLED-INTO-WAVE4 | STORY-087 (Color-Coded Slide Types — status/progress_bar/weighted_composite) pulled into Wave 4 as remediation pull-in (13 pts, P1). Human-authorized 2026-06-06. Closes F-G3-HIGH-003. |
| 2026-06-06 | PROCESS-GAP-WORKTREE-TYPES | [process-gap] PG-WORKTREE-TYPES: spec-correction agents for IN-FLIGHT worktree stories MUST read types from the worktree (.worktrees/STORY-NNN/), NOT main checkout. PO commit 608ec7b0 read develop types, stripped TextTag/AltText::Unspecified from BC-1.16.001; reverted by f2261592. Second process-gap this cycle (first: PG-TD060-SCOPE pass-4). Lessons file: .factory/cycles/STORY-086/lessons.md. |
| 2026-06-06 | STORY-086-CONVERGED | STORY-086 LOCAL adversary cascade CONVERGED 3/3 (passes 14/15/16-rerun all strict-CLEAN). Full 16-pass cascade with pass-16-original VOIDED (reviewer factual error) and re-run. |
| 2026-06-06 | STORY-086-PASS16-RERUN | Adversary LOCAL pass 16 RE-RUN: strict-CLEAN. Streak 3/3. CASCADE CONVERGED. |
| 2026-06-06 | STORY-086-PASS16-VOIDED | Adversary LOCAL pass 16 ORIGINAL VOIDED — reviewer factual error. Finding retracted; re-run dispatched. |
| 2026-06-06 | STORY-086-PASS15 | Adversary LOCAL pass 15 strict-CLEAN — streak 2/3. |
| 2026-06-06 | STORY-086-PASS14 | Adversary LOCAL pass 14 strict-CLEAN — streak 1/3. |
| 2026-06-06 | STORY-086-PASS13 | Adversary LOCAL pass 13 NOT strict-CLEAN (PR-merge CLEAN) — streak reset. F-086-P13-OBS-001 LOW: string conversion of None variant. Streak reset 0/3. |
| 2026-06-06 | STORY-086-PASS12 | Adversary LOCAL pass 12 NOT strict-CLEAN — streak reset. F-086-P12-MED-001: TextTag display impl emitted raw enum variant name. REMEDIATED. |
| 2026-06-06 | STORY-086-PASS11 | Adversary LOCAL pass 11 strict-CLEAN — streak 1/3. |
| 2026-06-06 | STORY-086-PASS10 | Adversary LOCAL pass 10 NOT strict-CLEAN — still 0/3. All 3 findings are the same underlying issue differently framed. |
| 2026-06-06 | STORY-086-PASS9 | Adversary LOCAL pass 9 NOT strict-CLEAN — streak reset. F-086-P9-MED-001 (span field omitted in TextTag display). REMEDIATED. |
| 2026-06-06 | STORY-086-PASS8 | Adversary LOCAL pass 8 strict-CLEAN — streak 1/3. |
| 2026-06-06 | STORY-086-PASS7 | Adversary LOCAL pass 7 CLEAN (PR-merge) but NOT strict-CLEAN — code correct at BC level; spec and doc-comment inconsistency only. Streak 0/3. |
| 2026-06-06 | STORY-086-PASS6 | Adversary LOCAL pass 6 found F-086-P6-MED-001: implementation stored UNTRIMMED title strings. REMEDIATED. Streak reset 0/3. |
| 2026-06-06 | STORY-086-PASS5 | Adversary LOCAL pass 5 found F-086-P5-CRIT-001 + F-086-P5-MED-001/002 + OBS-1 + D5 correction (STORY-088 scope +3 pts). STORY-088 expanded 5→8 pts. |
| 2026-06-06 | STORY-088-SCOPE-EXPAND | STORY-088 (Bullets list-literal DSL) scope expanded per adversary pass-5 D5 correction (+3 pts, 5→8). Human-authorized. |
| 2026-06-06 | PROCESS-GAP-TD060-SCOPE | [process-gap] Adversary pass-4 F-086-P4-MED-001 was in-scope fix misrouted as potential story-split. TD-VSDD-060 sibling-site sweep discipline reaffirmed. |
| 2026-06-06 | STORY-086-UNCERTAINTY-REMOVED | Before session restart, durability + uncertainty-removal on the eval/layout/types pipeline ran to confirm threading approach. |
| 2026-06-06 | STORY-086-PASS3 | Adversary LOCAL pass 3 found F-086-P3-HIGH-001 + F-086-P3-MED-001. Streak reset 0/3. |
| 2026-06-06 | STORY-086-PASS2 | Adversary LOCAL pass 2 found 3 MED + OBS. Streak reset 0/3. MED-001/002: placement of content threading hooks. |
| 2026-06-06 | STORY-086-PASS1 | Adversary LOCAL pass 1 found F-086-P1-CRIT-001 (Stage 2b skipped emitting alt=None for images). Streak 0/3. |
| 2026-06-06 | WAVE4-GATE-FAIL | Wave 4 integration gate RAN and FAILED. Gate 1 PASS (3244/3246; 2 perf-timing flakes tolerated). Gate 2 SKIP (no DTU). Gate 3 FAIL (F-G3-CRIT-001 content-block threading; F-G3-HIGH-001/002 validators inert; F-G3-HIGH-003 color-coded types unregistered; F-G3-HIGH-004 comment). Gate 5 FAIL (mean 0.56 < 0.85). |
| 2026-06-05 | STORY-050-MERGE | STORY-050 MERGED PR #61 (030dec6c). E2E integration suite + PDF/a11y/observability pipeline fixes. ADR-018 v1.1; BC-5.02.001 v1.5; BC-5.01.001 v1.2; error-taxonomy v2.15. DRIFT-CRITICAL-1 + DRIFT-CRITICAL-2 RESOLVED. 5-pass LOCAL cascade, 3/3 strict-CLEAN (passes 3-4-5). |
| 2026-06-05 | STORY-050-SEC | PR #61 security review: SEC-050-001 (IMPORTANT, CWE-116 control-char injection in PDF metadata) FIXED IN-SCOPE + re-reviewed CLEAN. |
| 2026-06-05 | STORY-050-CONV | STORY-050 LOCAL adversarial cascade CONVERGED. 5 passes total; passes 3-4-5 strict-CLEAN. |
| 2026-06-05 | STORY-050-PASS2 | STORY-050 adversary pass 2 NOT clean (3 findings). REMEDIATED. |
| 2026-06-05 | STORY-050-PASS1 | STORY-050 adversary pass 1 NOT clean (6 findings). REMEDIATED. |
| 2026-06-05 | STORY-050-GAP2-SPEC-BURST | Gap-2 spec burst landed (factory-artifacts). ADR-018 (post-layout validation pass), BC-5.02.001 v1.5. |
| 2026-06-05 | STORY-050-GAP2-AUTHORIZED | Human AUTHORIZED Gap-2 = Option A: implement a POST-LAYOUT validation pass for alt-text enforcement. |
| 2026-06-05 | STORY-050-RED-GATE | STORY-050 E2E Red Gate delivered (55 pass / 7 fail; branch `feature/STORY-050`). |
| 2026-06-05 | STORY-049-MERGE | STORY-049 MERGED PR #60 (e6f7832d). Plugin Registry Assembly. 21/21 CI checks green. 12-pass cascade, 3/3 strict-CLEAN (passes 10-11-12). |
| 2026-06-05 | STORY-049-CONV | STORY-049 LOCAL adversary cascade CONVERGED. 12 passes total; passes 10-11-12 strict-CLEAN. |
| 2026-06-05 | STORY-049-P6-8 | STORY-049 adversary passes 6-8 (multiple doc/type-alias findings fixed). |
| 2026-06-05 | STORY-049-R4 | STORY-049 adversary round 4 — ALL FIXED. HIGH-A: strict=false default in PluginRegistry::new corrected. |
| 2026-06-05 | STORY-049-R3 | STORY-049 adversary round 3 — ALL FIXED. IMP-1: strict defaulted false; BC-5.02.001 invariant 3 gap. |
| 2026-06-05 | STORY-049-R2 | STORY-049 adversary round 2 — ALL FIXED. C1: brand routing inverted. |
| 2026-06-05 | STORY-049-R1 | STORY-049 adversary round 1 — ALL FIXED. C1: build() returned Ok(()) instead of Ok(Deck). |
| 2026-06-05 | STORY-085 | STORY-085 MERGED PR #59 (e704e700). Bundled DefaultInlineFormat + PPTX dog-fooding refactor. 20/20 CI green. 9-pass cascade, 3/3 strict-CLEAN (passes 7-8-9). |
| 2026-06-05 | STORY-085-CONV | STORY-085 LOCAL adversary cascade CONVERGED. Passes 7-8-9 strict-CLEAN. |
| 2026-06-05 | STORY-085-R5 | STORY-085 adversary round 5 — all prior findings re-verified closed. |
| 2026-06-05 | STORY-085-R4 | STORY-085 adversary round 4 — all prior findings re-verified closed. |
| 2026-06-05 | STORY-085-R3 | STORY-085 adversary round 3 — 3 MED footnote-variant findings fixed. |
| 2026-06-05 | STORY-085-R2 | STORY-085 adversary round 2 — 6 prior findings closed; combined run-property accumulator (no silent drop of nested formatting). Spec deltas: AC-006 prose split; VP-053 harness corrected. |
| 2026-06-05 | STORY-085-R1 | STORY-085 adversary round 1 — 6 findings fixed. ADR-017 Option A human-approved. BC-5.02.001→v1.4; BC-3.05.001→v1.3.6; BC-5.02.002→v1.4; VP-053 created. STORY-085 status: 0/3 strict-CLEAN (pass 2 pending). |
| 2026-06-05 | STORY-084 | STORY-084 MERGED PR #58 (801f351b). 7 bundled SectionType impls. BC-3.02.001 invariant 4 enforced. 7-pass cascade, 3/3 strict-CLEAN (passes 5/6/7). Inverted-Red-Gate anti-pattern caught → LESSON-17. |
| 2026-06-05 | STORY-083 | STORY-083 MERGED PR #57 (5aaa27d2). PluginRegistryBuilder + RegistryError::MissingSurface. 6-pass cascade, 3/3 strict-CLEAN (passes 4/5/6). CI required fix commit bc52da1a → LESSON-16. |
| 2026-06-04 | LESSON-13-STORY-049 | STORY-049 LESSON-13 reconciliation (human-authorized). Three decisions: (A) RegistryBuilder → STORY-083; (B) SectionType (7) + InlineFormat (12) → STORY-084 + STORY-085; (C) root crate = pipeline driver. ADR-016 authored. Wave 4: 18→21 stories, 115→129 pts. Total: 81→85 stories, 497→511 pts. |
| 2026-06-04 | STORY-040 | STORY-040 MERGED PR #56 (869fb401). Batch B pptx chain complete. Slide-grouping split to STORY-082 (human-authorized). SafeUrl guard shipped. |
| 2026-06-04 | SEC-039 | SEC-039-001 (CWE-116, MED) + SEC-039-002 (CWE-754, LOW) FIXED IN-SCOPE during STORY-039 PR review. |
| 2026-06-04 | BC-5.01.005-v1.2 | Invariant 2 amended: lang SoT corrected to `deck.metadata.lang` (human-authorized). No code change — impl was already correct. 5 stale story-spec refs corrected. |
