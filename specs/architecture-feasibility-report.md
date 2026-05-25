---
document_type: architecture-feasibility-report
level: ops
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
phase: 1b
inputs:
  - .factory/specs/prd.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/prd-supplements/nfr-catalog.md
  - .factory/specs/prd-supplements/interface-definitions.md
  - .factory/specs/domain-spec/invariants.md
  - .factory/planning/spikes/S1-ooxmlsdk-pptx-coverage.md
  - .factory/planning/spikes/S2-pdf-backend-evaluation.md
  - .factory/planning/spikes/S3-wcag-tooling-choice.md
  - .factory/planning/spikes/S4-chumsky-indentation-parser.md
  - .factory/planning/spikes/S5-brand-synthesis-layout-taxonomy.md
  - .factory/planning/spikes/S6-multi-renderer-parity.md
  - .factory/planning/spikes/S14-mermaid-diagram-engine.md
input-hash: "[pending compute-input-hash]"
traces_to: .factory/specs/prd.md
prd_version: "1.0"
---

# Architecture Feasibility Report — slideforge v1.0

## Verdict: PASS-WITH-NOTES

---

## Executive Summary

The slideforge v1.0 PRD (99 BCs across 5 bounded contexts) is architecturally feasible.
Every technology choice has been validated by a resolved spike (S1–S6, S14). The proposed
19-crate workspace, Two-IR model, plugin-first design, and four-stage pipeline (Parse →
Evaluate → Layout → Export) can deliver all P0 BCs within the stated NFR targets. Three
architectural notes require design attention before Phase 3 begins: the experimental status
of krilla's tagged PDF layer, the need for a `slideforge-validate` crate to own compile-time
accessibility enforcement (the BCs split this ownership ambiguously), and a clarification on
how the `clrMapOvr` pattern interacts with the S6 BUG-001 warning about LibreOffice
cross-renderer parity.

---

## Constraint Mapping

| PRD Section | BC(s) | Requirement | Architecture Constraint | Feasibility | Resolution |
|-------------|-------|-------------|------------------------|-------------|-----------|
| 1 — Authoring | BC-1.01.001 | Parse valid .sf to typed AST with error accumulation | chumsky 0.10 + hand-written indentation lexer (S4 hybrid architecture) | Feasible | S4 resolved: 14/14 tests pass, < 1ms for 200 slides. Caveat: slide-level recovery must be added in production (S4 note). |
| 1 — Authoring | BC-1.01.002, BC-1.01.003 | Reject tab indentation and indentation inconsistency with file:line:col span | Indentation lexer must produce byte-accurate spans; chumsky Stream loses byte spans (S4 friction point 1) | Feasible-with-caveat | Span table lookup pattern from S4 is the production path. ADR-009 may adopt BorrowInput newtype for cleaner spans. |
| 1 — Authoring | BC-1.02.003 | No implicit type coercion (DI-004) | Evaluator type system must enforce strict type checking; `| int`, `| float`, `| bool` are the only coercion paths | Feasible | S4 lexer already preserves `"1.10"` as string token, not float. Enforced in `slideforge-eval` type checker. |
| 1 — Authoring | BC-1.04.003 | Reject any non-terminating construct (DI-005) | Grammar must structurally exclude @while, @fn. Bounded collection iteration only. Kani proof of termination required (Phase 6). | Feasible | chumsky grammar defines what is allowed. Reserved keyword rejection is a grammar rule. Kani proof is O(N×M) bounded model checking — feasible for the computation model. |
| 1 — Authoring | BC-1.10.003 | Math renders to OMML (PPTX/DOCX), MathML (HTML), paths (PDF) | Three distinct math output paths. OMML generation is non-trivial for complex LaTeX. | Feasible-with-complexity-note | `pulldown-latex` or a LaTeX-to-OMML transpiler must be selected (not yet spiked as S7–S14 range). This is a P1 BC; can be deferred to Wave 2. Risk: LaTeX-to-OMML has limited Rust library support. Recommend a research spike before Phase 3 Wave 2 begins. |
| 1 — Authoring | BC-1.12.001, BC-1.12.003 | Mermaid renders via mermaid-rs-renderer; SVG normalized via usvg | S14 resolved: 23 diagram types, < 3ms warm, PPTX-safe SVG by default | Feasible | mermaid-rs-renderer v0.2.2 confirmed. usvg normalization pipeline confirmed. |
| 2 — Branding | BC-2.01.002, BC-2.01.005 | Synthesize brand template from brand.toml with all 31 layouts | 31 layouts (11 standard + 20 custom) per S5 taxonomy. Dark layouts (CL-01, CL-11) require clrMapOvr + explicit white runs. | Feasible | S5 prototype: 27 KB .pptx with 16 layouts generated, extraction round-trip confirmed. Full 31-layout synthesis is an extension of the proven pattern. |
| 2 — Branding | BC-2.01.005 + DI-016 | clrMapOvr for dark layouts | S6 BUG-001: LibreOffice clrMapOvr support is incomplete. S5 R4 mitigation: write explicit white text colors on dark-layout runs in addition to clrMapOvr. | Feasible-with-cross-renderer-note | Both clrMapOvr AND explicit white runs must be written. This creates a slight tension with the clrMapOvr approach. The mitigation is well-defined (S5 §6.1, S6 §4.2) and must be encoded as an invariant in the `slideforge-brand` crate. |
| 3 — Layout | BC-3.01.001 | SlideType trait enforces required fields for all 31 types | Plugin trait API is the validation path; validated in `slideforge-eval` after interpolation | Feasible | Interface-definitions.md defines `SlideType::required_fields()`. The pattern is clean: all 31 types go through the same validation path (DI-008). |
| 3 — Layout | BC-3.04.001 | shape: block produces PPTX <p:sp> with integer EMU | Requires float-to-EMU conversion at parse time; shape DSL parsed in `slideforge-layout` | Feasible | S1 confirmed: ShapeTreeChoice::PSp with EMU positioning is available in ooxmlsdk 0.6.1. |
| 4 — Export | BC-4.01.001 | Serialize LaidOutDeck to valid .pptx with correct placeholder inheritance | ooxmlsdk 0.6.1 with two workarounds (W1: table XML, W2: Content_Types Default entries) | Feasible | S1 resolved: 55/57 capability checks pass. W1 and W2 are bounded workarounds. |
| 4 — Export | BC-4.01.002 | PPTX passes SSIM ≥ 0.99 AND PSNR ≥ 35dB vs reference | S6 threshold calibration: SSIM 0.99 (raised from 0.97) + PSNR 35dB dual gate | Feasible | S6 confirmed the dual-gate approach. LibreOffice CI pipeline (PPTX → PDF → PNG at 300 DPI) is specified. Cross-renderer divergences (font substitution) are catalogued and acceptable. |
| 4 — Export | BC-4.03.001 | PDF passes veraPDF --flavour ua1 with zero violations | pdf-writer + krilla (experimental tagged PDF layer) + custom SlideTagEngine | Feasible-with-risk | S2 risk R1: krilla tagged PDF is experimental. Fallback path is direct pdf-writer structure tree construction (keeping krilla for drawing only). This risk must be tracked into Phase 4. |
| 4 — Export | BC-4.03.002 | PDF produced via pdf-writer + krilla (no Chrome) | S2 resolved: Chrome headless rejected (150–250 MB binary, PDF/UA-1 unreliable for absolute-positioned layouts) | Feasible | Constraint is architecturally locked. |
| 4 — Export | BC-4.03.005 | EMU to PDF coordinate mapping with Y-axis flip | 1pt = 12700 EMU; IR top-left origin → PDF bottom-left origin. S2 code sample confirmed. | Feasible | The coordinate mapping is a pure function. Kani proof is specified in BC-4.03.005 and is feasible (bounded integer arithmetic). |
| 4 — Export | BC-4.03.004 | Web preview via axum + WebSocket + SVG canvas | axum is in the planned crate list (`slideforge-preview`); SVG-based rendering required by S3 (WCAG 1.1.1 — no bare canvas) | Feasible | Node.js only for `@axe-core/playwright` test harness, not for the production server. |
| 5 — Cross-cutting | BC-5.01.001 | Missing alt is a compile error (DI-001) | Alt enforcement must occur at the validation stage (after parse, before layout/export) | Feasible | S3 confirmed chumsky `validate()` accumulates alt errors without stopping the parse. The production architecture places this in `slideforge-validate` (or `slideforge-eval`). See Note 2 below. |
| 5 — Cross-cutting | BC-5.02.002 | No bundled plugin bypasses the trait API (DI-008) | Requires `slideforge-plugin-api` crate with all 10 trait surfaces publicly visible. Bundled plugins depend only on `slideforge-plugin-api`, not on each other's internals. | Feasible | 10 trait signatures are fully defined in interface-definitions.md §6. The dependency graph direction must be enforced: bundled plugins → plugin-api, never bundled-plugin-A → bundled-plugin-B internals. |
| 5 — Cross-cutting | BC-5.03.001 | Package install writes sha256 to sf.lock | Git-based package distribution, sha256 integrity verification | Feasible | sha256 hashing is trivial in Rust. The lock file format is TOML-based (analogous to Cargo.lock). No dependency on a central registry. |
| 5 — Cross-cutting | BC-5.05.001–005 | Watch mode: re-evaluate on .sf or HTTP data source change | Requires a file watcher (`notify` crate), HTTP polling loop, axum WebSocket push | Feasible | This is infrastructure work, not algorithmic work. `notify` crate is the standard Rust file watcher. The incremental build (NFR-002, < 50ms) is the real constraint: full re-parse is 71µs for 25 slides (S4), leaving 429ms budget for eval + layout + export delta. |

---

## Subsystem Grouping Assessment

| L2 Bounded Context | PRD Grouping Valid? | NFR Profile Coherent? | Notes |
|-------------------|--------------------|-----------------------|-------|
| Authoring (1.01–1.15) | Yes | Yes | DSL parsing, evaluation, iteration, composition, math, diagrams all belong together. The only cross-boundary concern is alt enforcement (see Note 2). |
| Branding (2.01–2.02) | Yes | Yes | Brand loading, synthesis, and extraction are a coherent cluster. The BrandProvider plugin trait correctly encapsulates this subsystem. |
| Layout (3.01–3.05) | Yes | Yes | The SlideType trait + LayoutEngine belong in one subsystem. Shape DSL and rich inline formatting are additive extensions to the layout surface. |
| Export (4.01–4.03) | Yes | Mostly — see note | PPTX, DOCX, PDF, HTML/Preview have distinct toolchain requirements but share the LaidOutDeck IR input. Grouping them as "Export Bounded Context" is correct. The NFR profile for PPTX (visual parity, < 200ms serialization) is different from PDF (PDF/UA-1 compliance) but the shared IR and the plugin architecture justify the unified grouping. |
| Cross-Cutting (5.01–5.05) | Yes | Yes | Accessibility validation, plugin architecture, package management, workspace config, and watch mode are genuinely cross-cutting. The cluster is internally coherent — all five subsections produce infrastructure that every other bounded context depends on. |

### Proposed Restructuring

No restructuring of bounded contexts is required. The L2 domain structure maps cleanly to the proposed architecture.

One clarification is needed at the module level (not BC level): the `slideforge-validate` crate must be explicitly introduced as the owner of compile-time validation rules (BC-5.01.001 through BC-5.01.005, BC-3.03.001 through BC-3.03.004). The PRD's architecture anchors reference `slideforge-eval` and `slideforge-validate` interchangeably for validation logic. The architect must assign ownership unambiguously to `slideforge-validate` during full architecture production (Phase 1b).

---

## Subsystem-to-Module Mapping (Preliminary)

| L2 Bounded Context | Proposed Modules (Crates) | Pure/Effectful Split | Notes |
|-------------------|--------------------------|---------------------|-------|
| Authoring — DSL | `slideforge-syntax` | Pure core | Lexer + chumsky parser → typed AST. No I/O. Kani-amenable. |
| Authoring — Evaluation | `slideforge-eval` | Pure core | Expression evaluation, variable scoping, type checking, iteration expansion. No I/O. |
| Authoring — Validation | `slideforge-validate` | Pure core | Compile-time validation: alt enforcement, canvas overflow, color contrast, required-field checks. No I/O. |
| Authoring — Data | `slideforge-data` | Effectful shell | DataSource plugins: JSON/CSV/YAML/TOML file loading, HTTP fetch. All I/O is in this crate. |
| Authoring — Diagrams | `slideforge-diagrams` | Effectful shell | mermaid-rs-renderer integration, usvg normalization. Font DB scan = I/O. |
| Authoring — Charts | `slideforge-charts` | Pure core | `plotters` SVG chart generation from ChartSpec. Pure function: spec in, SVG out. |
| Authoring — Math | `slideforge-math` | Pure core | LaTeX → OMML/MathML/PDF paths. Pure transformation. |
| Branding | `slideforge-brand` | Effectful shell | BrandProvider implementation: reads brand.toml (file I/O), reads .pptx template (file I/O), synthesizes BrandTemplate in memory. Pure synthesis function is isolatable. |
| Layout | `slideforge-layout` | Pure core | Deck IR → LaidOutDeck IR. All 31 SlideType trait implementations. Pure: coordinates in, LaidOutDeck out. |
| Export — PPTX | `slideforge-pptx` | Effectful shell | ooxmlsdk 0.6.1 serialization to bytes. Exporter plugin. File I/O is at the outer boundary only; serialization itself is pure. |
| Export — DOCX | `slideforge-docx` | Effectful shell | OOXML DOCX serialization. Exporter plugin. |
| Export — PDF | `slideforge-pdf` | Effectful shell | pdf-writer + krilla + SlideTagEngine. Font file I/O is effectful. Structure tree construction is pure. |
| Export — HTML/Preview | `slideforge-preview` | Effectful shell | axum + WebSocket server. SVG-canvas HTML generation. All network I/O is here. |
| Cross-cutting — Plugin API | `slideforge-plugin-api` | Pure (types only) | All 10 trait surfaces. No implementation, no I/O. The dependency inversion point. |
| Cross-cutting — Types | `slideforge-types` | Pure (types only) | `Deck`, `LaidOutDeck`, `Brand`, shared value types. Hash + Eq + Clone on everything. |
| Cross-cutting — Package | `slideforge-package` | Effectful shell | Git-based package install, sf.lock management. File I/O + network. |
| Cross-cutting — Workspace | `slideforge-config` | Effectful shell | slideforge.toml + .sfconfig cascade parsing. File I/O only. |
| Cross-cutting — Watch | `slideforge-cli` | Effectful shell | CLI entry point, watch mode orchestration, axum server lifecycle. All I/O is here. |

---

## Notes (PASS-WITH-NOTES Justification)

### Note 1 — krilla tagged PDF layer is experimental (HIGH risk, Phase 4)

S2 explicitly flags krilla's tagged PDF support as "experimental" at v0.6.0. The fallback
is direct pdf-writer structure tree construction (krilla used only for drawing). The
`SlideTagEngine` must be designed with an interface that allows the tagging layer to be
swapped without rewriting the coordinate mapping or font embedding code. This is not a
PRD change — it is a Phase 4 architecture decision that must be made explicit in the
architecture documents (verification-architecture.md and the PDF subsystem design).

**Action required (architect, Phase 1b):** The `SlideTagEngine` interface design must
decouple drawing (krilla) from tagging (potentially direct pdf-writer calls). Document
this in `architecture/export-subsystem.md` with explicit fallback trigger criteria.

### Note 2 — Compile-time alt enforcement module ownership is ambiguous in the PRD

BC-5.01.001 traces to `slideforge-eval or slideforge-validate crate (filled by architect)`.
BC-3.03.001 through BC-3.03.004 trace to `slideforge-layout`. Several validation rules
span multiple BCs and require a single owner to avoid duplicate enforcement logic.

The correct architecture assigns ALL compile-time content validation to `slideforge-validate`:
- Alt text enforcement (BC-5.01.001 through BC-5.01.005)
- Color-coded label enforcement (BC-5.01.003)
- Canvas overflow detection (BC-3.03.001)
- Strict mode no-output gate (BC-3.03.002)
- Zero-slide deck error (BC-3.03.004)
- Required field validation (BC-3.01.001 — via SlideType trait in `slideforge-eval`)

The validation stage runs after evaluation (variables resolved) and before layout. This
is the only position where all the information needed for these checks is available.

**Action required (architect, Phase 1b):** The full architecture must assign these BCs
to `slideforge-validate` in the Architecture Module column of each BC's traceability table.

### Note 3 — LaTeX-to-OMML conversion requires a pre-Phase-3 spike (MEDIUM risk, P1 BCs)

BC-1.10.003 requires LaTeX → OMML for PPTX/DOCX output. No mature Rust crate for LaTeX
→ OMML conversion was spiked (S7–S13 range covers other concerns). The available options are:
- `latex2mathml` crate (LaTeX → MathML, then MathML → OMML via XSLT — requires XSLT runtime)
- Custom transpiler from a subset of LaTeX command coverage
- Call into a Python/Java OMML generator (incompatible with single-binary constraint)

Since BC-1.10.003 is P1, this risk does not block Wave 1. However, a research spike must
precede Wave 2 (math rendering stories) to avoid a mid-sprint surprise.

**Action required (story-writer, Phase 2):** Place a research spike story (S-MATH-01) as
a dependency gate before any BC-1.10.x story enters Wave 2.

### Note 4 — clrMapOvr + explicit white runs is a dual-write pattern requiring test coverage

S5 §6.1 specifies the mitigation for BUG-001 (LibreOffice clrMapOvr incomplete support):
write BOTH `clrMapOvr` AND explicit `<a:solidFill><a:srgbClr val="FFFFFF"/>` on dark-layout
placeholder runs. This dual-write pattern means any future maintenance of dark layout
colors must update both paths.

**Action required (architect, Phase 1b):** Document this as an invariant in
`architecture/branding-subsystem.md`: "CL-01 and CL-11 layouts must always write both
clrMapOvr and explicit white text run colors. A test in `slideforge-brand` must verify
both are present in the synthesized XML."

### Note 5 — PRD BC count discrepancy (minor)

BC-INDEX.md Section Summary shows 99 BCs total. PRD Section 2 states "Total: 99 BCs —
71 P0, 28 P1, 0 P2." However, the PRD Section 7 traceability matrix contains 99 rows.
These counts are consistent. The discrepancy with the prior reference to "101 behavioral
contracts" in the task prompt is not reflected in the actual BC-INDEX.md (which shows 99).
No action required — the PRD and BC-INDEX agree.

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| krilla experimental tagged PDF layer fails PDF/UA-1 validation in Phase 6 hardening | MEDIUM | HIGH | Design SlideTagEngine with decoupled draw/tag layers. Fallback: direct pdf-writer structure tree construction (documented in S2 §ADR-003 Option). If fallback also fails: Typst template approach (S2 Option A). |
| LaTeX-to-OMML has no production-ready Rust crate | MEDIUM | MEDIUM (P1 BCs only) | Research spike (S-MATH-01) before Wave 2. Fallback: ship math as MathML in all formats + SVG rasterization for PPTX (downgrade path). |
| mermaid-rs-renderer crate abandonment or insufficient diagram type coverage | LOW | MEDIUM | Version pinned to `=0.2.2`. Direct dependency on `selkie-rs` is the escape hatch (Option B from S14). The 23 covered diagram types cover all v1.0 use cases. |
| Performance: SlideTagEngine PDF structure tree construction exceeds 200ms budget for large decks | LOW | MEDIUM | NFR-005 targets < 200ms for PPTX/DOCX serialization. PDF is not explicitly bounded, but total cold build < 500ms (NFR-001) constrains all stages. Structure tree is O(elements per deck) — linear. Profile early in Phase 4. |
| clrMapOvr + explicit white runs produce conflicting colors if brand color slot changes | LOW | LOW | `slideforge-brand` unit test must verify both paths agree on all dark-layout color values. The invariant is testable in < 10 lines. |
| NFR-001 (< 500ms cold build) broken by font DB scan in mermaid-rs-renderer (S14: 124ms cold) | LOW | MEDIUM | Font DB scan is once per process, amortized. For a 25-slide deck with 5 diagrams: ~124ms + 5×3ms = ~139ms for diagrams alone. Remaining budget (~361ms) covers PPTX export which S1 benchmarks suggest is fast. Benchmark fixture must include 5 diagrams. |
| ooxmlsdk W2 workaround (Content_Types Default entries) not accepted by strict OOXML validators | LOW | LOW | `opc_postprocess.rs` post-processing is the fix. S1 confirmed modern renderers tolerate absence of Default entries. Open XML SDK conformance check will confirm. |

---

## Contradiction Detection

### No BC-to-BC contradictions found.

The following potential tensions were examined and resolved:

1. **BC-5.01.001 (alt is compile error, fatal) vs BC-3.03.003 (warn-only renders error-slide placeholders):**
   Not a contradiction. In `--warn-only` mode, E-A11-001 is promoted to a warning and the
   build continues. Strict mode (default) treats E-A11-001 as fatal. Both BCs consistently
   reference the strict/warn-only distinction. The behavior is consistent with the error
   taxonomy in PRD Section 5 (Accessibility category: "Fatal in strict; warning in warn-only").

2. **BC-2.01.005 (clrMapOvr on CL-01 and CL-11) vs S6 BUG-001 (clrMapOvr incomplete in LibreOffice):**
   Not a contradiction but a tension requiring dual-write mitigation (Note 4 above). The S5
   R4 mitigation (explicit white run colors) is the resolution. Both BC and spike are
   consistent — the BC specifies the behavior, the spike specifies the cross-renderer
   implementation requirement.

3. **BC-4.03.004 (web preview via axum + WebSocket) and BC-1.12.001 (no Node.js) — does the
   web preview require Node.js?** No contradiction. S3 specifies that Node.js is used ONLY
   for the `@axe-core/playwright` test harness under `crates/slideforge-preview/tests/`.
   The production binary (axum server + WebSocket + SVG canvas) has zero Node.js dependency.

4. **BC-1.14.002 (report register excluded from PPTX) vs BC-1.14.004 (no register content
   bleeds to wrong format):** BC-1.14.004 is a generalization of BC-1.14.002. Both are
   consistent — the underlying invariant is DI-012 (single source, consistent content per format).

---

## Missing Contract Analysis

### One gap identified — HTTP data source URL allowlist enforcement

NFR-019 specifies: "HTTP data source URL allowlist enforced — No requests to non-allowlisted
domains when allowlist configured." The `data.allowed_domains` configuration key is defined
in interface-definitions.md §4.1.

No BC explicitly covers the allowlist enforcement behavior:
- What happens when a domain is not on the allowlist? (Should emit E-DAT-003 or E-CFG-005)
- What is the behavior when `allowed_domains` is absent vs empty list vs non-empty list?
- Does `--offline` interact with the allowlist?

**Recommendation:** This is security-sensitive behavior (SSRF prevention). A BC-1.03.005
"HTTP data source domain is validated against allowed_domains allowlist" should be added
as P1. The story-writer should be aware of this gap when decomposing Section 5 (Watch Mode
stories that test HTTP error paths).

### One gap identified — DSL version forward-compatibility message

BC-1.13.001 covers the `slideforge_version` header and rejects forward-incompatible
versions. The BC specifies E-PAR-010 for version mismatches but the error-taxonomy
supplement (PRD Section 5) does not list E-PAR-010. This is a minor inconsistency.

**Recommendation:** Add E-PAR-010 to the error-taxonomy supplement: `"Unsupported DSL
version: source declares slideforge_version '{{N}}' but this binary supports up to version
'{{M}}'. Upgrade slideforge or downgrade your source."` This is a documentation fix, not
a BC change.

---

## Decision Log

| Decision | Alternatives Considered | Chosen | Rationale |
|----------|------------------------|--------|-----------|
| Accept ooxmlsdk 0.6.1 with two workarounds (W1, W2) | quick-xml raw XML generation, openxml (Rust, abandoned) | ooxmlsdk 0.6.1 + `opc_postprocess.rs` | 55/57 capabilities confirmed. W1 and W2 are bounded. quick-xml would require hand-maintained element ordering — high defect risk. |
| Accept pdf-writer + krilla as PDF backend with SlideTagEngine custom layer | printpdf (no UA-1), genpdf (architecture mismatch), Typst-as-library (IR mismatch), headless Chrome (binary size blocking) | pdf-writer + krilla + custom SlideTagEngine | Only path to PDF/UA-1 compliance within the single-binary constraint. Typst template approach is documented as fallback in ADR-003. |
| Accept chumsky 0.10 with hybrid architecture (hand-written indentation lexer) | winnow (worse error recovery ergonomics), pest (no built-in recovery), hand-written parser (~2x lines) | chumsky 0.10 + hand-written lexer | 14/14 spike tests pass. Error accumulation is chumsky's first-class design goal. Performance leaves ample headroom (89µs for 25 slides). |
| Accept mermaid-rs-renderer for DiagramRenderer | mermaid-cli (foreignObject in SVG, 1-4s per diagram, 200MB Chromium), WASM embedding (6-12 weeks R&D, no existing mermaid.wasm), Kroki HTTP service (offline-hostile) | mermaid-rs-renderer v0.2.2 | PPTX-safe SVG by default, < 3ms warm, single binary, 23 diagram types. All other options have BLOCKING issues for v1.0. |
| Raise SSIM threshold from 0.97 to 0.99 for CI visual regression gate | Keep 0.97 (too lenient per S6 empirical data), raise to 0.995 (too aggressive given font substitution) | SSIM ≥ 0.99 AND PSNR ≥ 35dB dual gate | S6 empirical finding: 4pt shape drift passes SSIM 0.97 but fails PSNR 35dB. Both gates together correctly discriminate all real regressions. |
| Assign all compile-time validation to `slideforge-validate` crate | Split between `slideforge-eval` (type errors) and `slideforge-layout` (overflow) | `slideforge-validate` owns all compile-time content validation | Prevents duplicated validation logic across crates. Validation runs after eval (all vars resolved) and before layout — a natural pipeline stage boundary. |

---

## Approval

| Role | Decision | Date |
|------|----------|------|
| Architect | APPROVED — PASS-WITH-NOTES — proceed to full architecture (Phase 1b) | 2026-05-24 |
| Product Owner | Acknowledged — BC-1.03.005 (HTTP allowlist) and E-PAR-010 additions recommended | |
