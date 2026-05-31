---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-05-30T00:00:00
cycle: STORY-024
inputs: [STATE.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Lessons Learned — STORY-024

STORY-024 Brand Extraction CLI (BC-2.01.003). LOCAL adversary cascade: 11 passes,
convergence at 3/3 strict-CLEAN (passes 9, 10, 11). 211 slideforge-brand tests passing.
Key hardening: scheme-ref hex resolution, TOML escaping, EC-005 warn, AC-007 string
type, end-to-end round-trip. Spec parity: E-BRD-006, VP-051/VP-052, BC v1.1 → v1.8.
Follow-ups: STORY-075 (footer detection) + STORY-076 (srgbClr transform extraction).

---

## Process-Level

1. **[process-gap] EC-003 round-trip-parse test convention** — The adversary flagged
   that EC-003 (tint/shade transform round-trip for schemeClr) had no failing test
   driving the behavior during Red Gate. The implementer added a unit test during the
   green phase, but the test was written post-hoc rather than as a Red Gate test. This
   violates TDD strict mode (tests should fail before implementation, not be written
   during implementation).
   _Discovered: Pass 3-4, 2026-05-30_

   **Disposition:** Warrants self-improvement follow-up. Proposed codification: "When
   any EC in a BC is not covered by a Red Gate failing test, the test-writer must either
   add a failing test for it, or record an explicit coverage gap in the story spec with a
   justification and a future story anchor (e.g., STORY-NNN). The implementer MAY NOT
   write the EC test themselves during green phase — that short-circuits TDD discipline."
   This is a standing process rule candidate; defer codification to next session review
   (cost: low, impact: medium).

2. **[process-gap] BC anchor-population gating before implementer dispatch** — The
   adversary found in early passes that VP-051 and VP-052 were allocated as "VP-TBD"
   placeholders in BC-2.01.003 v1.1 at Red Gate time. The architect had not yet
   populated the VP Anchors section with the concrete test function names and file paths.
   This meant BC-2.01.003 was technically incomplete when the implementer was dispatched.
   _Discovered: Pass 5 (VP-TBD resolution), 2026-05-30_

   **Disposition:** Justified deferral with observation. The VP-TBD pattern is tolerated
   when VP numbers are allocated and test file paths are known — which they were (VP-051,
   VP-052, in extractor.rs). The gap was the concrete test function names not yet written
   (tests didn't exist yet since implementer had not run). This is a chicken-and-egg
   scenario: VP Anchors require test function names, but test function names don't exist
   until after Red Gate. Proposed clarification to process: "VP Anchors in a BC may cite
   `[test function names TBD — will be populated after Red Gate by state-manager post-impl
   sweep]` if the VP number and file path are already allocated. The state-manager
   post-merge burst MUST backfill the actual function names." No separate story needed —
   add to state-manager operating checklist.

---

## Agent-Level

3. **Scheme-ref hex resolution requires color-table lookup, not passthrough** — The
   initial extractor implementation treated `schemeClr` slots as passthrough hex values.
   The adversary caught that `schemeClr` references (e.g., `accent1`) must be resolved
   against the theme color table to produce a concrete hex value for brand.toml, not
   emitted as the symbolic name. The fix required adding a scheme-ref resolver step in
   `BrandExtractor::extract()`.
   _Discovered: Pass 2, 2026-05-30_

4. **TOML string escaping for special characters** — Brand names and font family strings
   containing characters like `&`, `<`, `>`, and backslashes must be escaped for TOML
   output. The initial implementation did not escape these. Fixed with a TOML-aware
   serialization path using the `toml` crate's string escaping.
   _Discovered: Pass 4, 2026-05-30_

5. **EC-005 extension-only logo format detection emits warn, not error** — Early
   implementation treated an unrecognized logo file extension as E-BRD-006 (fatal). The
   BC-2.01.003 EC-005 specifies a `tracing::warn!` and continuation, not a fatal error.
   Fixed in pass 6 to match BC semantics.
   _Discovered: Pass 6, 2026-05-30_

6. **AC-007 color slot values are strings in TOML** — The extractor initially wrote color
   slot hex values as unquoted TOML values. TOML requires hex strings to be quoted
   (e.g., `dk1 = "#1F1F1F"`). Fixed to use `format!("\"{}\"", hex)` for all color slot
   serialization.
   _Discovered: Pass 7, 2026-05-30_

---

## Infrastructure-Level

7. **footer_text: None hardcoded in loader.rs:214** — The adversary found (F-024A-OBS-1)
   that `loader.rs` hardcodes `footer_text: None`, making the `[footer]` writer in
   `BrandExtractor` permanently unreachable on real PPTX input. This was an existing gap
   in STORY-022's scope, not introduced by STORY-024. The correct resolution was to
   create STORY-075 (footer detection follow-up story) rather than expanding STORY-024's
   scope to include loader modifications.
   _Discovered: Pass 8 (adversary OBS-1), 2026-05-30_

---

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | EC coverage at Red Gate: test-writer must cover all ECs or record gap with story anchor | test-writer agent, Red Gate protocol | proposed |
| 2 | VP Anchor backfill: state-manager post-merge burst must populate VP Anchor test function names | state-manager operating checklist | proposed |
