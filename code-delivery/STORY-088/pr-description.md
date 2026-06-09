## feat(syntax,eval): STORY-088 bullets list-literal field-value DSL syntax

### Summary

Adds `[...]` list-literal parsing to the `slideforge-syntax` field-value grammar so that
`bullets: ["A","B","C"]` and `@var items = ["A","B","C"]` both parse correctly, producing
`FieldValue::List(Vec<FieldValue>)` in the AST. A single shared `list_literal_elements`
combinator covers all four value-position surfaces (field-value, `@var`/vars-block,
`set`-rule defaults, and variant `vars:` overrides). New error code `E-PAR-024` reports
non-string and nested-list elements with a `"got <type>"` substitution and a source span;
a non-recursive O(1) bracket-depth tracker guards against unbounded-recursion DoS.

Closes the STORY-086 AC-007 parser-gap note. After this PR merges, STORY-086 AC-007's
`#[ignore]`'d E2E fixture can be un-ignored.

---

### Architecture Changes

```mermaid
graph TD
    DSL[".sf source"] --> Parser["value_parser()\nlist_literal_elements combinator"]
    Parser --> FVL["FieldValue::List(Vec<FieldValue>)"]
    FVL --> Eval["slideforge-eval\nFieldValue::List → Value::List"]
    Eval --> Threading["thread_fields_to_blocks\nBC-1.16.001 PC-7"]
    Threading --> CBB["ContentBlock::Bullets"]
    CBB --> Layout["Layout pass\n(unchanged)"]
    Layout --> PPTX["slideforge-pptx\n<a:r> text runs\n(unchanged)"]

    Parser2["set_rule_value_parser()"] --> FVL
    Parser3["variant_value parser"] --> FVL
    Parser4["vars-block value parser"] --> FVL

    subgraph "Shared combinator (STORY-088)"
        Parser
        Parser2
        Parser3
        Parser4
    end
```

**Files changed:**

| File | Change |
|------|--------|
| `crates/slideforge-syntax/src/ast.rs` | Add `List(Vec<FieldValue>)` variant to `FieldValue` enum; update all match arms |
| `crates/slideforge-syntax/src/parser/deck.rs` | Add `list_literal_elements` shared combinator; extend `value_parser()`, `set_rule_value_parser()`, vars-block parser; `E-PAR-024` error recovery; O(1) DoS guard |
| `crates/slideforge-syntax/src/parser/variants.rs` | Extend `variant_value` parser with the shared list-literal arm |
| `crates/slideforge-eval/src/` | Add `FieldValue::List` eval arm → `Value::List`; confirm `FieldValue::Ident` lookup resolves `Value::List` correctly |
| `tests/integration/` | Un-ignore STORY-086 AC-007 E2E fixture; add AC-007 E2E test for direct list-literal bullets |
| `docs/demo-evidence/STORY-088/` | 4 recordings + evidence-report.md |

No changes to `slideforge-layout`, `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, or `slideforge-html`.

---

### Story Dependencies

```mermaid
graph LR
    S006["STORY-006\nParser Core\n(merged)"] --> S088["STORY-088\nBullets list-literal\n(THIS PR)"]
    S086["STORY-086\nStage 2b threading\nValue::List→Bullets\n(merged)"] --> S088
    S088 -.->|"unblocks E2E"| S086_AC7["STORY-086 AC-007\nE2E fixture\n(can un-ignore after merge)"]
```

Dependencies STORY-006 and STORY-086 are both merged to `develop`.

---

### Spec Traceability

```mermaid
flowchart LR
    BC1["BC-1.01.002\nParser Core:\nSlide Field Parsing\n(primary)"]
    BC2["BC-1.16.001\nPost-Eval Field-to-Block\nPC-7: Value::List→Bullets\n(secondary)"]
    BC3["BC-1.15.001\nError Accumulation\n(error path)"]

    AC1["AC-001\nbullets: list literal\n→ FieldValue::List"]
    AC2["AC-002\nempty list []"]
    AC3["AC-003\nsingle-item list"]
    AC4["AC-004\nindentation context"]
    AC5["AC-005\nE-PAR-024 non-string"]
    AC6["AC-006\n@var regression"]
    AC7["AC-007\nE2E ≥3 text runs"]
    AC12["AC-012\nset-rule default"]
    AC13["AC-013\nvariant vars override"]

    T1["test_bc_1_01_002_ac001_*"]
    T2["test_bc_1_01_002_ac002_*"]
    T3["test_bc_1_01_002_ac003_*"]
    T4["test_bc_1_01_002_ac004_*"]
    T5["test_bc_1_01_002_ac005_*\n+ MED001a/b/p3"]
    T6["test_bc_1_01_002_ac006_*"]
    T7["test_bc_1_01_002_ac007_*\n(E2E)"]
    T12["test_bc_1_01_002_ac012_*\n+ p5_med001"]
    T13["test_bc_1_01_002_ac013_*"]

    BC1 --> AC1 & AC2 & AC3 & AC4 & AC5 & AC6 & AC12 & AC13
    BC2 --> AC7
    BC3 --> AC5 & AC12 & AC13
    AC1 --> T1
    AC2 --> T2
    AC3 --> T3
    AC4 --> T4
    AC5 --> T5
    AC6 --> T6
    AC7 --> T7
    AC12 --> T12
    AC13 --> T13
```

---

### Test Evidence

| Metric | Value |
|--------|-------|
| `cargo nextest` — workspace total | 3916 pass / 20 skip / 0 fail |
| `slideforge-syntax` suite (including new AC-001..007, AC-012, AC-013 tests) | 460 pass / 1 skip / 0 fail |
| AC-007 E2E (direct list-literal → ≥3 `<a:r>` PPTX runs) | 1 pass |
| `cargo fmt --all -- --check` | CLEAN |
| `cargo clippy` (pedantic + unwrap_used) | CLEAN |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | CLEAN |
| Mutation testing (Phase 6) | N/A — evaluated at Phase 6 |
| Kani proofs (Phase 6) | N/A — evaluated at Phase 6 |

---

### Demo Evidence

4 recordings covering all 9 ACs (AC-001 through AC-007, AC-012, AC-013). Located in
`docs/demo-evidence/STORY-088/` on branch `feature/STORY-088`.

| Recording | ACs Covered |
|-----------|-------------|
| `AC-001-004-list-literal-parse.gif` | AC-001 (primary positive), AC-002 (empty list), AC-003 (single-item), AC-004 (indentation context) |
| `AC-005-006-error-paths-regression.gif` | AC-005 (integer + boolean + nested-list E-PAR-024), AC-006 (@var regression) |
| `AC-007-e2e-pptx-runs.gif` | AC-007 (E2E: direct list-literal → PPTX ≥3 `<a:r>` text runs) |
| `AC-012-013-set-rule-variant-vars.gif` | AC-012 (set-rule positive + E-PAR-024), AC-013 (variant vars positive + E-PAR-024 + nested-list rejection) |

**Coverage map:** All 9 ACs have at least 1 demo recording. AC-005 has 4 sub-cases
demonstrated (integer, boolean, mixed, nested-list). AC-012 and AC-013 each demonstrate
3 sub-cases (positive, error, edge).

---

### Holdout Evaluation

N/A — evaluated at wave gate.

---

### Adversarial Review

LOCAL adversarial cascade: **CONVERGED — 3/3 strict-CLEAN** (passes 8, 9, 10 out of 10-pass cascade).

| Pass | Findings | Severity | Fixed | Status |
|------|----------|----------|-------|--------|
| 1 | E-PAR-024 anchor inconsistency; colon-form + @var parser coverage gaps | MED (3) | 3 | FIXED |
| 2 | `"got <type>"` substitution missing; Error-sentinel substitution gap | MED (2) | 2 | FIXED |
| 3 | Nested-list rejection incomplete; float coverage gap; stale comments | MED (2), OBS (1) | 3 | FIXED |
| 4 (scope expansion) | AC-012/AC-013 scope approved by human; quad-duplication identified | SCOPE | 0 findings | AUTHORIZED |
| 5 | Quad-duplication in 4 value-position parsers; shared combinator missing | MED (1) | 1 | FIXED |
| 6 | CRITICAL: unbounded-recursion DoS in nested-list guard (introduced in Pass-5) | CRIT (1) | 1 | FIXED |
| 7 | OBS: boundary + EOF branch coverage gaps in depth tracker | OBS (1) | 1 | FIXED |
| 8 | **CLEAN (strict): yes** | 0 | — | STREAK 1/3 |
| 9 | **CLEAN (strict): yes** | 0 | — | STREAK 2/3 |
| 10 | **CLEAN (strict): yes** | 0 | — | STREAK 3/3 — CONVERGED |

Notable findings caught and fixed:
- **Pass-6 CRITICAL (DoS guard):** A non-recursive nested-list guard introduced in Pass-5 introduced
  an unbounded recursion path when the depth counter was implemented with a recursive combinator.
  Fixed via a non-recursive O(1) bracket-depth tracker that inspects tokens without recursing.
- **Pass-5 MED (quad-duplication):** Identical list-literal logic in all 4 value-position parsers
  was consolidated into a single `list_literal_elements` shared combinator, eliminating the
  duplication vector for future divergence.
- **Pass-2 MED (message consistency):** `E-PAR-024` message template used inconsistent `got <type>`
  substitution patterns. Fixed by a single canonical format string across all positions.

---

### Security Review

Pending orchestrator dispatch of `vsdd-factory:security-reviewer`.

---

### Risk Assessment

| Dimension | Assessment |
|-----------|-----------|
| Blast radius | `slideforge-syntax` + `slideforge-eval` only; no layout/exporter changes |
| Backwards compatibility | Additive: new `FieldValue::List` variant; no existing variant removed or renamed |
| DoS surface | Mitigated: non-recursive O(1) bracket-depth tracker prevents nested-list recursion DoS (Pass-6 finding, fixed) |
| Performance | Parser adds a `list_literal_elements` alternative; no hot-path change; single-line lists only in v1.0 |
| Error message contract | `E-PAR-024` is a NEW error code; no existing error messages changed |

---

### AI Pipeline Metadata

| Dimension | Value |
|-----------|-------|
| Pipeline mode | Greenfield, Phase 3 (TDD per-story delivery) |
| Story points | 8 (scope-expanded from 5 during adversary cascade, human-authorized) |
| Adversary passes | 10 passes, 3/3 strict-CLEAN convergence (passes 8–10) |
| Specialist agents | test-writer, implementer, adversary (×10), demo-recorder |
| Model | claude-sonnet-4-6 |

---

### Pre-Merge Checklist

- [x] PR description matches actual diff (architecture diagram verified against commit log)
- [x] All 9 ACs covered by demo evidence (4 recordings, evidence-report.md confirmed)
- [x] Traceability chain complete: BC-1.01.002 → AC-001..007, AC-012, AC-013 → tests → demo
- [x] 3/3 strict-CLEAN LOCAL adversary convergence
- [x] Exit gate green: fmt PASS, pedantic clippy PASS, nextest 3916/20 pass/skip, rustdoc PASS
- [x] Dependencies merged: STORY-006 (merged), STORY-086 (merged)
- [x] Branch rebased onto develop @ c60cca36, HEAD d76f0ada
- [ ] Security review (orchestrator dispatches security-reviewer)
- [ ] PR-level adversarial review (orchestrator dispatches pr-reviewer)
- [ ] CI checks pass (triggered on PR creation)
- [ ] Squash merge authorized by orchestrator
