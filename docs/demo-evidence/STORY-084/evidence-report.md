# STORY-084 Demo Evidence Report

**Story:** STORY-084 — Bundled SectionType Implementations  
**Crate:** `slideforge-plugin-api`  
**Branch:** `feature/STORY-084`  
**Recorded:** 2026-06-04  
**Recording tool:** VHS (terminal capture of `cargo run --example`)

---

## Artifacts

| File | Format | Covered ACs |
|------|--------|-------------|
| `AC-001-005-section-types.gif` | GIF (PR embed) | AC-001, AC-002, AC-003, AC-004, AC-005 |
| `AC-001-005-section-types.webm` | WEBM (archival) | AC-001, AC-002, AC-003, AC-004, AC-005 |
| `AC-001-005-section-types.tape` | VHS script | AC-001, AC-002, AC-003, AC-004, AC-005 |

---

## Coverage Map

### AC-001 — All 7 bundled SectionType impls exist; id()s match canonical set

**Demonstrated in recording at:** AC-001 block  
**Evidence:** The demo instantiates all 7 implementations via the public
`slideforge_plugin_api` re-exports, calls `id()` on each, and prints them.
The output confirms all 7 ids: `executive_summary`, `risk_register`,
`methodology`, `scope`, `approval`, `appendix`, `glossary`.  
**Result:** `PASS  all 7 ids match canonical set`

---

### AC-002 — ExecutiveSummarySectionType::generate() scans slides for takeaway fields

**Demonstrated in recording at:** AC-002 happy-path block  
**Evidence (happy path):** Deck of 3 slides: slide 0 has `takeaway`, slide 1
does not, slide 2 has `takeaway`. `generate()` returns exactly 2
`SectionBlock`s: both at `level=1`, `include_in_toc=true`, with correct
`start_slide_index` values (0 and 2, confirming absolute-index ordering).

**Demonstrated in recording at:** AC-002 / BC-3.02.001 inv-4 block  
**Evidence (exclusion path):** Deck of 2 slides that both have `takeaway`;
slide 0 carries `register: Notes`. `generate()` returns exactly 1 block —
only the non-notes slide contributes, confirming BC-3.02.001 invariant 4.  
**Result:** `PASS` on both paths

---

### AC-003 — RiskRegisterSectionType::generate() scans slides for severity_cards type

**Demonstrated in recording at:** AC-003 happy-path block  
**Evidence (happy path):** Deck of 5 slides: 2 are `severity_cards`, 3 are
other types. `generate()` returns exactly 2 `SectionBlock`s at `level=1`,
`include_in_toc=true`, with `start_slide_index` values 1 and 3 (absolute
positions in the 5-slide deck).

**Demonstrated in recording at:** AC-003 / BC-3.02.001 inv-4 block  
**Evidence (exclusion path):** Deck of 2 `severity_cards` slides; slide 0
has `register: Notes`. `generate()` returns exactly 1 block — only the
non-notes `severity_cards` slide is included.  
**Result:** `PASS` on both paths

---

### AC-004 — Manual-only SectionType impls return empty Vec for any slide slice

**Demonstrated in recording at:** AC-004 block  
**Evidence:** All 5 manual types (`methodology`, `scope`, `approval`,
`appendix`, `glossary`) each have `generate()` called with a 3-slide deck
containing mixed slide types. All return 0 blocks.  
**Result:** `PASS  all 5 manual types return vec![] for any slide slice`

---

### AC-005 — All 7 impls compile using only slideforge-plugin-api public API

**Demonstrated in recording at:** AC-005 block  
**Evidence:** The entire example binary (`story_084_section_types.rs`) imports
only `slideforge_plugin_api::*` public re-exports and
`slideforge_types::{FieldValue, OrderedMap, Register, Slide, SourceSpan, Value}`.
No imports from private paths, exporter crates, or other forbidden modules.
`cargo build --example story_084_section_types -p slideforge-plugin-api`
succeeds with zero warnings.  
**Result:** `PASS`

---

## CI Gate Confirmation

All three CI gates ran clean in the worktree before recording:

| Gate | Command | Result |
|------|---------|--------|
| Clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items` | CLEAN |
| Rustdoc | `RUSTDOCFLAGS="-D warnings" cargo doc -p slideforge-plugin-api --no-deps` | CLEAN |
| Fmt | `cargo +nightly fmt --all -- --check` | CLEAN |

---

## Example Binary

**Path:** `crates/slideforge-plugin-api/examples/story_084_section_types.rs`  
**Run command:** `cargo run --example story_084_section_types -p slideforge-plugin-api -q`
