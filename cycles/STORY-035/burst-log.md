---
document_type: burst-log
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-05-31T00:00:00
cycle: "STORY-035"
inputs: [STATE.md]
traces_to: STATE.md
---

# Burst Log — STORY-035

Historical narrative archived from STATE.md on 2026-05-31 during prose-accuracy
refresh (content routing: verbose historical narrative moves out of STATE.md into
cycle burst-log per content routing rules, target <250 lines).

---

## Burst: STORY-035 Delivery (2026-05-31)

**Agents dispatched:** test-writer, implementer, adversary (10 passes), pr-manager, pr-reviewer, security-reviewer, state-manager
**Files touched:** crates/slideforge-eval/src/*, crates/slideforge-types/src/*, .factory/stories/STORY-035-writing-register-routing.md, .factory/stories/STORY-077-sectionblock-ir-extension.md, .factory/STATE.md
**Versions bumped:** BC-1.14.004 added; STORY-INDEX 76→77; project total 77 stories / 462 pts

### Summary

STORY-035 (Writing Register Routing) delivered through the full per-story-delivery
flow. Red Gate (3392b598) + Green (4f8a9487) + adversary cascade (10 passes,
3/3 strict-CLEAN at passes 8/9/10) + PR #39 squash-merged to develop @ 0e7d9fde.

Architect directive: F-001 → Option D (eval populates Slide.register_content at
eval stage; layout clones; slideforge-types duplicate method deleted, single source
of truth). F-002 → AC-005 section-node SectionBlock descoped to STORY-077 (EC-003
DESCOPED from STORY-035 scope entirely). BC-1.14.004 added to spec.

STORY-077 (SectionBlock IR Extension) created as architect-directed spin-out. EPIC-18,
P0, 8pts, Wave 4 Batch A. Anchors BC-3.02.002 + BC-1.14.003. Blocks STORY-041/042.

### Details

| Agent | Task | Output |
|-------|------|--------|
| test-writer | Red Gate — 3392b598 on feature/S-035 | 743 failing tests covering all ACs |
| implementer | TDD green — 4f8a9487 | 743/743 GREEN, clippy/fmt clean |
| adversary | Pass 1 | 2 HIGH / 2 MED / 1 LOW findings |
| implementer | Fix-burst 1b4f654e | F-001 Option D impl; F-002 descope; BC-1.14.004 added |
| adversary | Passes 2–7 | Progressive convergence; streak reset at passes 2-7 |
| adversary | Pass 8 | strict-CLEAN — streak 1/3 |
| adversary | Pass 9 | strict-CLEAN — streak 2/3 |
| adversary | Pass 10 | strict-CLEAN — streak 3/3 — CASCADE CONVERGED |
| pr-manager | 9-step PR cycle | PR #39 created, CI 17/17 passed |
| pr-reviewer | Fresh-eyes diff review | APPROVE, 0 findings |
| security-reviewer | Security scan | CLEAN |
| state-manager | Post-merge burst | STATE.md updated, develop_sha → 0e7d9fde, PR count → 39 |

---

## Historical narrative: Orchestrator Playbook Improvements (captured 2026-05-28/30)

From the STORY-020 (29 passes) + STORY-028 (32 passes) + STORY-021 (23 passes) marathon cascades:

1. **BC version bump auto-sweep**: When PO bumps a BC version, orchestrator should dispatch
   implementer for code-comment sweep + story-writer for story-spec body sweep + architect
   for VP-INDEX propagation before declaring BC bump fix-burst complete.

2. **Implementer commit-SHA verification**: Orchestrator should verify claimed N file changes
   against `git log --stat HEAD` before accepting closure. Pass-27 had implementer claim 33
   renames in 4 files; only 8 in 1 file actually shipped (TD-VSDD-059).

3. **Scope creep detection**: When implementer touches files outside story perimeter, surface
   for orchestrator authorization BEFORE the fix-burst.

4. **Convergence asymptote awareness**: After ~10 passes with only LOW findings, check in
   with human about strict-CLEAN vs PR-merge-CLEAN gate selection.

5. **Documentation sweep on green-phase**: When implementer closes a Red Gate, same commit
   should remove stale "/// Red Gate: panics with todo!()" doc lines.

6. **Display-vs-protocol coupling (STORY-021 Pass 15 structural fix)**: Source layer emits
   canonical machine-friendly format; dispatcher/consumer constructs rich Display.

7. **CI clippy version drift (STORY-021)**: `just check` must mirror CI clippy flags exactly.
   Before any pre-push lint claim: `rustup update stable && cargo clippy --workspace
   --all-targets --all-features -- -D clippy::pedantic -D clippy::unwrap_used`.

8. **Spec-entity retirement parity (STORY-021 Pass 13→14)**: When code retires a spec entity,
   the matching spec update must be in the same fix burst.

9. **#[non_exhaustive] default for public enums**: All public enums should default to
   #[non_exhaustive] to prevent SemVer breakage on variant additions.

---

## Historical narrative: Task state from 2026-05-28/30 session

Completed in 2026-05-28/30 session:
- Fix PR #30 STORY-034 conflict + merge (4 CI iterations: rebase + Linux font_resolver + Trebuchet MS via freefont/usvg fallback)
- STORY-030 Pass 10-17 adversary cycles + 4 fix bursts + PR #31 merge (1 CI iteration)
- STORY-023 Pass 11-20 adversary cycles + 5 fix bursts + PR #32 MERGED (dd6054c1)
- STORY-020 29-pass LOCAL adversary cascade, 3-CLEAN passes 27-28-29; PR #33 MERGED (143f1b78)
- STORY-028 32-pass LOCAL adversary cascade, 3-CLEAN passes 30-31-32; PR #34 MERGED (066d625f); follow-ups STORY-072/073/074 created
- STORY-021 23-pass LOCAL adversary cascade, 3-CLEAN passes 21-22-23; ~80 findings; 10 ACs demo'd (30 files); PR #35 MERGED (362c4a1f)
- STORY-024 Red Gate b61b0d04; implementer 211 tests; LOCAL adversary 11 passes 3/3 strict-CLEAN (P9-10-11); 11 ACs demo'd; PR #36 MERGED (924cdc04)
- STORY-025 Red Gate 27d67613; implementer + sibling sweep 42 Slide constructions; LOCAL adversary 11 passes 3/3 strict-CLEAN (P9-10-11); 10 ACs demo'd; PR #37 MERGED (584cbc6f)
- Wave 3 Gate: 2584 tests GREEN; holdout must-pass 5/5; adversary 8 passes strict-CLEAN 6/7/8; fix-PR #38 MERGED (7d266ad7)
- Wave 4 planning: 17 stories / 104 pts approved; STORY-073/075/076 pulled in P1; STORY-072/074 deferred Wave 5; STORY-077 added P0

---

## Historical narrative: Per-Wave Decisions Log detail (archived from STATE.md)

This section preserves the verbose per-merge decision log entries that were in STATE.md.
They remain available here as audit trail. The STATE.md Decisions Log now shows milestone
rows only (one per significant event).

### Wave 3 Batch 2 — detailed merge records

- 2026-05-28 — STORY-019 MERGED (PR #27, 7bc71f9c) — HTTP/HTTPS DataSource + SSRF allowlist (9-pass adversary, 49 findings fixed, defense-in-depth: allowlist before DNS + redirects(0) + body cap + scheme normalization)
- 2026-05-28 — STORY-032 MERGED (PR #28, 5ad267be) — Chart empty-data placeholder, 7-pass adversary convergence
- 2026-05-28 — STORY-027 MERGED (PR #29, 7641d4ea) — DOCX section generation (executive_summary + risk_register), 8-pass adversary convergence
- 2026-05-28 — STORY-030 font engine refactor: ab_glyph 0.2.31 + embedded Latin Modern Math 733KB OTF (GFL/LPPL) replacing synthetic glyph match — authorized by user to fix Pass 9 HIGH findings; now at pass 10 threshold
- 2026-05-28 — STORY-034 adversary 3/3 CONVERGED (9 passes); branch rebased onto develop; PR #30 OPEN (CONFLICTING, needs rebase)
- 2026-05-28 — STORY-034 PR #30 force-push (local rebase to 92221f3d had never reached remote) + 4 CI iterations:
  - iter-1: implementer fontdb-empty hypothesis (apt-get fonts-dejavu-core + fonts-noto-core + fc-cache + Linux fontdb fallback dir scan) — INSUFFICIENT
  - iter-2: clippy len_zero fix + cold_budget budget 200→300ms + diagnostic eprintlns — DIAGNOSTIC REVEALED fontdb=324 but SVG <text>=false
  - iter-3: fonts-liberation added to apt-get + diagnostic removed + cargo fmt — INSUFFICIENT (Liberation Sans Arial-compatible but not Trebuchet-compatible)
  - iter-4: ROOT CAUSE — Mermaid emits font-family="'trebuchet ms', verdana, arial, sans-serif"; usvg 0.47 lookup is case-sensitive byte-compare; doesn't implement CSS fallback chain. Fix: custom usvg::Options::font_resolver with fallback chain (Liberation Sans → DejaVu Sans → Noto Sans → FreeSans). + fonts-freefont-ttf for defense-in-depth.
- 2026-05-28 — STORY-034 MERGED (PR #30 → squash → 0cb4b982). slideforge-diagrams now has post-rendering usvg normalization with PPTX-safe text preservation across all 4 platforms.
- 2026-05-28 — STORY-030 MERGED (PR #31 → squash → 19e79696). slideforge-math: LaTeX → MathML + SvgPaths newtype; GUST Font License + MANIFEST.toml SHA-256 audit + LICENSE-LatinModernMath.txt; @{var} math-mode interpolation; tracing #[instrument]. 170 tests + 2 traced_test. Convergence: 9 adversary iterations, 2 paper-fix corrections (TD-VSDD-059). 3/3 CLEAN Pass 15/16/17.
- 2026-05-28 — STORY-023 CONVERGED 3/3 CLEAN (Pass 18, 19, 20) + demos done (15 ACs at d771099e). 20-pass adversary trail, 5 fix bursts, 1 scope-discipline correction, 3 spec-code drift corrections (E-BRD-002/005/007).
- 2026-05-29 — STORY-023 MERGED (PR #32, dd6054c1) — Brand Synthesis: brand.toml → 31 Layouts. 20 LOCAL adversary passes, 3/3 CLEAN (P18-20). PR-level 2 cycles, cycle 2 CLEAN. F1/F2 fixed in 4f78aa1c.
- 2026-05-30 — STORY-020 MERGED (PR #33, 143f1b78) — DataSource: Excel + SQLite. 29 LOCAL adversary passes, 3/3 CLEAN (P27-28-29). AKM compounding-novelty. 23 ACs demo'd, 251 unit tests + 2 perf_smoke.
- 2026-05-30 — STORY-028 MERGED (PR #34, 066d625f) — Layout: shape: Block + Rich Inline. 32 passes, 3/3 CLEAN (P30-31-32). 18 ACs demo'd, 309 slideforge-layout + 1764 workspace tests. Follow-ups: STORY-072/073/074.
- 2026-05-30 — STORY-021 MERGED (PR #35, 362c4a1f) — DataSource HTTP cache + E-DAT policy errors. 23 LOCAL adversary passes, 3/3 CLEAN (P21-22-23). ~80 findings, 20 fix-burst commits. 10 ACs demo'd (30 evidence files). Key structural: PolicyRejected variant; label-driven IoError vs FileNotFound; #[non_exhaustive]; UTF-8 panic fix; parse_e_dat_code; E-DAT-014 retired; STRUCTURAL FIX Display decoupled from inter-layer protocol (P15). Workspace tests: 2282.
- 2026-05-30 — STORY-024 RED GATE (b61b0d04); implementer 211 tests; adversary 11 passes 3/3 strict-CLEAN (P9-10-11). BC-2.01.003 v1.1→v1.8 (8 incremental versions). Follow-ups: STORY-075 (footer detection) + STORY-076 (srgbClr transform-aware extraction). VP-051+052 added, VP-INDEX 50→52.
- 2026-05-30 — STORY-024 MERGED (PR #36, 924cdc04) — Brand Extraction library. library-only (CLI wiring → STORY-057); 11 ACs demo'd. AI PR-diff review PR-merge-CLEAN (2 LOW/OBS). Security CLEAN.
- 2026-05-30 — STORY-025 RED GATE (27d67613); adversary 11 passes 3/3 strict-CLEAN (P9-10-11). HIGH path-traversal containment guard (E-BRD-007). BC-2.02.001 v1.1→v1.4; BC-2.02.002 v1.1→v1.2; error-taxonomy v2.2. 455 tests. AC-007/009/012 deferred to STORY-008/009 per Architecture Compliance Rule 4.
- 2026-05-30 — STORY-025 MERGED (PR #37, 584cbc6f) — Per-Slide brand_overlay invariant. AI PR-diff review PR-merge-CLEAN (1 LOW + 4 OBS). Security CLEAN. Wave 3 Batch 3 COMPLETE.

### Wave 3 Gate + Wave 4 Start — detailed records

- 2026-05-31 — WAVE 3 GATE: 2584 tests GREEN @ 584cbc6f. Holdout: must-pass 5/5 (SSRF/HS-013 0.90; mean 0.767 over 6 evaluable; 9 blocked structural mid-build). Adversary CONVERGED: 8 passes, strict-CLEAN 6/7/8. Integration findings on fix/wave3-gate: slideforge-brand → [workspace.dependencies]; #[non_exhaustive] hardening; conventions.md v1.3.
- 2026-05-31 — CONVENTIONS.md v1.3: #[non_exhaustive] Policy codified (PATH A variant-growth / PATH B closed-domain; exhaustive plugin-api classification table; wildcard-arm rule). Stale-checkout code-state column removed (OBS-2 correction).
- 2026-05-31 — WAVE 3 GATE FIX-PR #38 MERGED (squash 7d266ad7). AI strict-CLEAN; security CLEAN; 17/17 CI. Adversary pass 8 strict-CLEAN.
- 2026-05-31 — WAVE 3 COMPLETE. All 22 stories merged. develop @ 7d266ad7 (38 PRs, ~2584 tests, 0 failures). 0 active worktrees.
- 2026-05-31 — WAVE 4 STARTED. 16 stories / 96 pts (expanded: STORY-073/075/076 pulled in P1; STORY-072/074 deferred Wave 5 P2). BC-2.01.001 v1.2, BC-2.01.003 v1.9, error-taxonomy v2.3 landed.
- 2026-05-31 — STORY-035 IN PROGRESS (feature/S-035 @ 1b4f654e): Red Gate + Green + LOCAL adversary Pass 1 (2 HIGH/2 MED/1 LOW) + fix-burst. 743/743 GREEN, clippy/fmt clean. Adversary Pass 2 in flight (streak 0/3). Architect directive: F-001 Option D; F-002 AC-005/EC-003 descoped → STORY-077. BC-1.14.004 added.
- 2026-05-31 — STORY-077 CREATED: SectionBlock IR Extension (spin-out from STORY-035 F-002). EPIC-18, P0, 8pts, Wave 4 Batch A. Anchors BC-3.02.002 + BC-1.14.003. Blocks STORY-041/042. Wave 4 now 17 stories / 104 pts; project total 77 stories / 462 pts.
- 2026-05-31 — STORY-035 MERGED (PR #39, 0e7d9fde). 10-pass LOCAL adversary cascade, 3/3 strict-CLEAN at passes 8/9/10. Option D single-source eval-stage routing. pr-reviewer APPROVE 0 findings; security CLEAN; CI 17/17. AC-005 descoped → STORY-077. Wave 4 Batch A: 1/8 complete.
