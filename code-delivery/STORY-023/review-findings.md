---
story_id: STORY-023
pr: 32
branch: feature/S-023
last_updated: 2026-05-29
---

# Review Findings — STORY-023 / PR #32

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining | Status |
|-------|----------|----------|-------|-----------|--------|
| 1     | 2        | 1 (F1)   | 0     | 2         | in-review |
| 2     | 1 (OBS)  | 0        | 2     | 0         | APPROVED — CLEAN (PR-merge) — MERGED dd6054c1 |

## Findings

### F1 — BrandPalette mapping inconsistency (HIGH / blocks merge)

- **Severity:** HIGH (IMPORTANT → blocks merge per BC-5.39.001)
- **Category:** coherence / missing
- **Files:** `crates/slideforge-brand/src/synthesizer.rs:399-402`, `crates/slideforge-brand/src/loader.rs:268-273`
- **Finding:** `BrandProvider::load` for `BrandSource::TomlFile` maps `BrandPalette` using positional slot indices (slot[0]=primary=dk1, slot[1]=secondary=lt1=always white). `brand_from_template` (PPTX/DOCX path) maps by slot name: primary=dk2, secondary=acc1, accent=acc2, neutral=lt2. Same brand loaded from different sources produces different `BrandPalette` — visual parity violation.
- **Route:** implementer
- **Status:** open

### F2 — fol_hlink empty-string fallback (SUGGESTION / non-blocking)

- **Severity:** SUGGESTION
- **Category:** coherence (defensive gap)
- **File:** `crates/slideforge-brand/src/inference.rs:262`
- **Finding:** `map_or("", ...)` produces empty string if `result[10]` is somehow None; `darken_hex("")` silently returns black with no warning or assertion.
- **Route:** implementer (fix in same burst as F1)
- **Status:** open

## Triage Routing Table

| Finding | Routed To | Task Status |
|---------|-----------|-------------|
| F1      | implementer | RESOLVED — commit 4f78aa1c: color_by_name + regression test |
| F2      | implementer | RESOLVED — commit 4f78aa1c: debug_assert added |

## Cycle 2 OBS Finding (non-blocking)

### OBS-1 — slot_hex return type cosmetic inconsistency

- **Severity:** OBS (non-blocking)
- **File:** `crates/slideforge-brand/src/synthesizer.rs`
- **Finding:** `slot_hex` closure returns `String` (then converted to `Arc<str>`), while `loader.rs` `slot_hex` returns `Arc<str>` directly. Functionally equivalent; minor allocation difference.
- **Status:** non-blocking, no action required for merge
