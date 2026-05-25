---
document_type: architecture-section
section: crate-architecture
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# Crate Architecture

## 20-Crate Workspace

The workspace uses Cargo resolver 3, edition 2024. All crates enforce
`#![forbid(unsafe_code)]`. Production dependencies use `=` version pinning.

```
crates/
├── slideforge-plugin-api/   # [SS-14] 10 trait surfaces — pure types, no I/O
├── slideforge-types/        # [SS-15] Deck, LaidOutDeck, Brand, Value + 31 SlideType impls — pure types
├── slideforge-syntax/       # [SS-01] Lexer + chumsky parser → Typed AST — PURE CORE
├── slideforge-eval/         # [SS-02] Expression eval, type checking — PURE CORE
├── slideforge-validate/     # [SS-03] All compile-time validation — PURE CORE
├── slideforge-brand/        # [SS-04] BrandProvider impl, 31-layout synthesis — EFFECTFUL
├── slideforge-layout/       # [SS-05] Deck → LaidOutDeck — PURE CORE
├── slideforge-pptx/         # [SS-06] ooxmlsdk 0.6.1 serialization — EFFECTFUL
├── slideforge-docx/         # [SS-08] OOXML DOCX serialization — EFFECTFUL
├── slideforge-pdf/          # [SS-07] pdf-writer + krilla (Phase 4) — EFFECTFUL
├── slideforge-html/         # [SS-09 partial] HTML Exporter — EFFECTFUL (Phase 4)
├── slideforge-preview/      # [SS-09] axum + WebSocket live preview — EFFECTFUL
├── slideforge-charts/       # [SS-12] plotters SVG chart generation — PURE CORE
├── slideforge-diagrams/     # [SS-11] mermaid-rs-renderer + usvg — EFFECTFUL (font I/O)
├── slideforge-math/         # [SS-13] LaTeX → OMML/MathML/HTML — PURE CORE
├── slideforge-data/         # [SS-10] DataSource plugins — EFFECTFUL (all I/O)
├── slideforge-package/      # [SS-16] sf.lock, git-based package install — EFFECTFUL
├── slideforge-config/       # [SS-17] slideforge.toml + .sfconfig cascade — EFFECTFUL
├── slideforge/              # Root library — assembles PluginRegistry — EFFECTFUL
└── slideforge-cli/          # [SS-18] CLI binary, watch mode, lifecycle — EFFECTFUL
```

Phase 4 crates (`slideforge-pdf`, `slideforge-html`) are in `[workspace] exclude`
until Phase 3 gates pass.

## Purity Classification

| Classification | Rule | Crates |
|---------------|------|--------|
| Pure Core | Deterministic, no I/O, Kani-amenable | slideforge-syntax, slideforge-eval, slideforge-validate, slideforge-layout, slideforge-charts, slideforge-math, slideforge-plugin-api, slideforge-types |
| Effectful Shell | Has I/O, network, or filesystem | slideforge-brand, slideforge-pptx, slideforge-docx, slideforge-pdf, slideforge-preview, slideforge-data, slideforge-package, slideforge-config, slideforge-cli, slideforge, slideforge-diagrams |

`slideforge-diagrams` is classified Effectful because of the cold font DB scan
(fontdb memory-mapped font files), even though the rendering logic itself is pure.

## Dependency Direction Rules

1. Effectful crates MAY depend on pure-core crates. Never the reverse.
2. Bundled plugin crates MUST depend only on `slideforge-plugin-api`, not on each
   other's internal types (DI-008 dog-fooding guarantee).
3. `slideforge-types` and `slideforge-plugin-api` have zero production dependencies
   on other workspace crates.
4. The dependency graph is a DAG — no cycles (enforced by cargo).

## Validation Crate Ownership (Feasibility Note 2)

ALL compile-time content validation belongs to `slideforge-validate` (SS-03):
- Alt text enforcement (BC-5.01.001 through BC-5.01.005)
- Color-coded label enforcement (DI-002)
- Canvas overflow detection (BC-3.03.001)
- Strict mode no-output gate (DI-017, BC-3.03.002)
- Zero-slide deck error (BC-3.03.004)
- Brand color contrast check (S3: `BrandValidator::check_theme_pairs()`)

The validation stage runs after evaluation (all variables resolved) and before
layout (all coordinates computed). This is the only pipeline position where
full semantic information is available for all checks.

`slideforge-eval` handles type errors during expression evaluation; it calls
`slideforge-validate`'s validators on the completed Deck IR before passing to layout.

## Currently Scaffolded Crates (Phase 1)

Seven crates exist in `workspace.members` as stubs:
`slideforge`, `slideforge-cli`, `slideforge-syntax`, `slideforge-eval`,
`slideforge-layout`, `slideforge-pptx`, `slideforge-validate`.

Remaining 13 crates are created during Phase 3 story delivery.
