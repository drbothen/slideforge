# PR Review — STORY-041 DOCX Core Serialization (PR #51)

**Reviewer:** pr-reviewer (fresh-eyes, Opus 4.8) · **Verdict:** ✅ **APPROVE WITH SUGGESTIONS** (non-blocking)

This gates merge. No BLOCKING findings. The crate is additive, isolated to `slideforge-docx`, CI is fully green (20+ checks pass on develop base, mergeState CLEAN), and every claim in the PR body matches the diff.

---

## What I verified

- **Every changed source file** in the diff: `lib.rs`, `document_body.rs`, `styles.rs`, `content_types.rs`, `zip_assembler.rs`, `error.rs`, `examples/demo_docx.rs`, the 1337-line `core_tests.rs`, three `.snap` snapshots, and demo evidence.
- **All 9 ACs trace to load-bearing tests** that exercise real production code paths (not tautologies). 28 tests; I spot-audited the high-risk ones below.
- **PR body accuracy** — claims match the diff (parts list, register routing, rId reservation, zip pin rationale, deferred item).
- **CI** — fmt, clippy, docs, doctest, msrv, snapshots, 4-platform test matrix, audit, supply-chain, semgrep, visual-regression: all green.
- **Upstream deps** — STORY-004/019/035 are on develop (verified via PR body + base branch state).

## Test adequacy (load-bearing, no paper-fixes)

The test suite is genuinely strong and resists the tautology trap:

- **AC-008 bold/italic (`..._inline_bold_italic_run_properties`)** — uses an *enclosure proof*: it extracts the exact `<w:r>…</w:r>` fragment around `<w:t>Bold</w:t>` and asserts `<w:b/>` is *within that run*, AND that the sibling `" and "` run does NOT contain `<w:b/>`. This kills the "property emitted anywhere in the paragraph" false-pass. Correctly de-tautologized (F-041-001).
- **AC-005/006 bleed tests** use the shared `BleedChecker` from slideforge-eval (cross-crate canonical method, not a local re-implementation that could drift).
- **F-041-007 rId collision** — parses every `Id="..."` out of the rels and asserts uniqueness + no reuse of reserved rId1–3. Real structural guard.
- **F-041-004 / F-DOCX-001 parse-back** — round-trips `document.xml`, `styles.xml`, `[Content_Types].xml` through `ooxmlsdk` deserializers. Proves well-formedness without a renderer (correct application of SID-1: no `#[ignore]` dodge).
- **F-041-008** — proves the `<w:p>` + `<w:p ` token count is precise and that the naive `<w:p` count is strictly larger, guarding against `<w:pPr`/`<w:pStyle` inflation.

The Red Gate exemption for `id()`/`extension()` (FRAMEWORK-WIRING) is legitimately classified.

## Deferred item — DEF-041-P5-001 (genuinely cross-story, not a dodge)

The exporter emits one `<w:p>` per `RegisteredContent` entry. Whether a single `report` field's internal `\n` splits into multiple entries is owned by the **STORY-035 evaluator output contract**, not this exporter. The exporter is correct given its input contract. EC-002's test (`..._ec002_multi_paragraph_report`) confirms multiple *entries* → multiple paragraphs, which is the exporter's actual responsibility. Deferral to the Wave 3 gate is appropriate and the routing is disclosed. I concur this is a legitimate cross-story integration concern, not a shortcut.

---

## Findings

| ID | Severity | Category | Location | Finding |
|----|----------|----------|----------|---------|
| S-1 | SUGGESTION | coherence/docs | `src/lib.rs` module doc (ZIP structure block) | The module doc lists `word/theme/theme1.xml` in the DOCX ZIP structure, but `build_docx` never emits a theme part. Doc overstates the output. Either drop the line or emit a minimal theme1.xml. Non-blocking: Word/LibreOffice open fine without a theme part. |
| S-2 | SUGGESTION | correctness | `src/lib.rs` `build_core_xml` | `core.xml` declares `xmlns:dcterms` and `xmlns:xsi` but emits only `<dc:language>`. The unused namespace decls are harmless but noise; conversely, `cp:coreProperties` conventionally also carries `dcterms:created`/`dcterms:modified` (with `xsi:type`). Not required for validity, but a stricter OOXML linter (Quality Bar: "custom OOXML linter") may flag the dangling decls later. Consider trimming to just the namespaces actually used. |
| S-3 | SUGGESTION | maintainability | `src/lib.rs` + `src/styles.rs` | `xml_attr_escape` is duplicated verbatim in both modules (and `xml_content_escape` in lib). Minor DRY violation. A shared `crate::xml_escape` helper would prevent the two copies drifting. Non-blocking. |
| N-1 | NIT | correctness | `src/styles.rs` | `Heading1`/`Heading2` are `basedOn` `Normal`, but `Normal` defines no `<w:name w:val="Normal"/>`-linked `w:default="1"` and there is no `docDefaults` block. Word tolerates this; a future "Microsoft Accessibility Checker" pass (per Quality Bar) is the right place to harden. No action this PR. |
| N-2 | NIT | style | `src/document_body.rs` `make_text` | Whitespace-preserve detection checks leading/trailing space and `\t` but not other significant whitespace (e.g. NBSP, newline). Given the deferred `\n` handling (DEF-041-P5-001), acceptable for now. |

None of these block merge.

## Notable strengths

- `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, `clippy::pedantic` all honored; zero `.unwrap()` in non-test code (the `let _ = writeln!` infallibility comment is correct and idiomatic).
- Deterministic ZIP via `BTreeMap` + epoch `DateTime::default()` is the right mechanism; determinism is tested at the byte level (`..._determinism_same_deck_twice`) AND at the assembler level (order + epoch timestamp).
- `next_rel_id` starting at 4 with explicit reserved-ID documentation is correct and tested.
- Register routing (notes excluded, report→Heading1+Normal, detail→Heading2 appendix after all slides) matches BC-4.02.001 exactly; one Heading2 per slide (F-041-003) is correctly hoisted out of the per-entry loop.
- `xmlns:r` on the document root (F-DOCX-001) — the prior CRIT regression — is now declared unconditionally and guarded by a parse-back test.

## Regression risk to develop

**Low.** Additive crate; no existing source modified except `Cargo.toml`/`Cargo.lock` (new member + workspace dep). The `zip =4.2.0 default-features=false` pin mirrors the proven slideforge-pptx pattern and resolves cleanly alongside zip 2.3.0 / 8.6.0 (no lzma link conflict). Full 4-platform CI matrix green confirms no cross-platform breakage.

---

**Verdict: APPROVE WITH SUGGESTIONS.** Merge authorized per STANDING MERGE AUTH. The five suggestions/nits are quality polish for future hardening passes (OOXML linter / accessibility checker phases) and do not need to be addressed in this PR.
