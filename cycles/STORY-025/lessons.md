---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-05-30T00:00:00
cycle: "STORY-025"
inputs: [STATE.md]
input-hash: "[pending]"
traces_to: STATE.md
---

# Lessons Learned — STORY-025

<!-- Durable lessons from the STORY-025 LOCAL adversary cascade (11 passes, 3/3
     strict-CLEAN at passes 9-10-11). Organized by category per content routing
     rules. Two [process-gap] patterns recurred 3x each during the cascade. -->

## Process-Level

1. **[process-gap] Widen-in-taxonomy requires same-burst sweep of #[error] string + variant docs + field docs + sibling doc-tables** — When an error code or message is WIDENED in error-taxonomy.md or a BC (e.g., E-BRD-007 widened from "brand template file" to "brand root directory" to include overlay coverage), the corresponding `#[error("...")]` attribute string, variant doc-comment, field-level docs, and any doc-table rows citing that error must be updated in the SAME fix-burst. This pattern recurred 3 times during the STORY-025 cascade: (a) E-BRD-007 message body update — taxonomy updated but `#[error]` string and 2 doc-tables lagged; (b) LogoRequired context-neutral message change — BC updated but the implementation `#[error]` string retained the context-specific phrasing; (c) doc-tables across BC-2.02.001 and error-taxonomy.md went out of sync when LogoRequired was generalized. **Disposition:** Fix was applied in-scope each time. Warrants a self-improvement follow-up: add a pre-fix-burst checklist step — "grep crates/ for the literal old error message string before committing any taxonomy/BC widen change."
   _Discovered: Passes 3, 5, 7 — 2026-05-30_

2. **[process-gap] Adding a new error return arm requires `# Errors` rustdoc sweep against all reachable return paths** — When a new error return is added to a function (e.g., EC-008 empty-path → `LogoRequired` added to `resolve_overlay`), the `# Errors` doc-section must enumerate ALL reachable error variants, not just the new one. During the cascade, 2 `# Errors` sections were found to omit pre-existing reachable variants after the new arm was added (the new arm was documented but the context was incomplete). **Disposition:** Fixed in-scope. Justified deferral for a self-improvement policy: adversary pass checklist should include "for any function with a new error arm, verify `# Errors` is complete against all match/? arms." No future-story anchor needed — this is a per-burst discipline check.
   _Discovered: Passes 4, 8 — 2026-05-30_

## Agent-Level

3. **Path-traversal containment guard must reuse existing strip_unc_prefix pattern** — The HIGH finding at pass 1 (path-traversal in `resolve_overlay` via `..` components) was remediated by adding a `path.components()` containment guard that strips `Component::ParentDir` and `Component::Prefix`. The canonical strip_unc_prefix helper already existed for UNC path normalization. The guard was co-located with strip_unc_prefix to centralize all path-normalization logic in one place. Pattern to carry forward: whenever a new path-accepting function is added to slideforge-brand, check whether strip_unc_prefix + containment guard apply.
   _Discovered: Pass 1 — 2026-05-30_

4. **media_type unification via shared logo::media_type_from_extension eliminates divergence risk** — `infer_media_type` was initially implemented as a standalone match arm in overlay.rs. Pass 2 found that the logo module (logo.rs) had an independent media-type lookup. The fix unified both callers onto a single `logo::media_type_from_extension` function. Lesson: whenever two functions in the same crate answer the same lookup question (extension → MIME type, color name → slot, etc.), unify immediately — do not leave parallel lookup tables that will diverge under future extension additions.
   _Discovered: Pass 2 — 2026-05-30_

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | Pre-fix-burst checklist: grep crates/ for literal old error message before committing taxonomy/BC widen | Per-story adversary cascade | proposed |
| 2 | Adversary checklist: verify `# Errors` completeness against all reachable arms when a new error return is added | Per-pass adversary review | proposed |
