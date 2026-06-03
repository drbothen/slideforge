---
document_type: architecture-section
section: dependency-graph
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# Dependency Graph

## External Production Dependencies

All production crate deps use `=` version pinning (NFR-025). Dev/test deps use
compatible-version constraints. `Cargo.lock` is committed.

| Dep | Pinned Version | Category | License | Used In |
|-----|---------------|----------|---------|---------|
| ooxmlsdk | =0.6.1 | PPTX generation | MIT | slideforge-pptx, slideforge-docx |
| pdf-writer | =0.14.0 | PDF low-level | Apache-2.0 | slideforge-pdf |
| krilla | =0.6.0 | PDF drawing | Apache-2.0 | slideforge-pdf |
| usvg | =0.47.0 | SVG normalization | MIT/Apache-2.0 | slideforge-pdf, slideforge-diagrams |
| subsetter | =0.2.0 | Font subsetting | Apache-2.0 | slideforge-pdf |
| mermaid-rs-renderer | =0.2.2 | Diagram rendering | MIT | slideforge-diagrams |
| chumsky | =0.10.1 | Parser combinator | MIT | slideforge-syntax |
| axum | =0.8.1 | HTTP server | MIT | slideforge-preview |
| tokio | =1.38 | Async runtime | MIT | slideforge-preview, slideforge-data |
| plotters | =0.3.7 | SVG chart generation | MIT | slideforge-charts |
| serde | =1.0 | Serialization | MIT/Apache-2.0 | multiple |
| serde_json | =1.0 | JSON | MIT/Apache-2.0 | slideforge-data, slideforge-eval |
| toml | =0.8 | TOML parsing | MIT/Apache-2.0 | slideforge-config, slideforge-data |
| reqwest | =0.12 | HTTP client | MIT/Apache-2.0 | slideforge-data |
| notify | =6.1 | File watching | CC0-1.0 | slideforge-cli |
| thiserror | =1.0 | Error derives | MIT/Apache-2.0 | all error enums |
| miette | =7.2 | Error rendering | Apache-2.0 | slideforge-cli |
| tracing | =0.1 | Instrumentation | MIT | all crates |
| tracing-subscriber | =0.3 | Tracing setup | MIT | slideforge-cli |
| clap | =4.5 | CLI parsing | MIT/Apache-2.0 | slideforge-cli |
| zip | =2.1 | ZIP handling | MIT | slideforge-pptx (opc_postprocess) |
| sha2 | =0.10 | SHA-256 | MIT/Apache-2.0 | slideforge-package |
| calamine | =0.24 | Excel reading | MIT | slideforge-data |
| rusqlite | =0.31 | SQLite | MIT | slideforge-data |
| quick-xml | =0.36 | XML parsing | MIT | slideforge-brand (extraction + serialization) |

## cargo deny Policy

`cargo deny check all` runs on every PR (NFR-016, NFR-017):
- Deny: all licenses not in allowlist (MIT, Apache-2.0, ISC, BSD-2, BSD-3, CC0-1.0, Zlib)
- Deny: any crate with a high or critical CVE in `cargo audit` database
- Warn: crates with no version pinning in workspace.dependencies

## Internal Dependency Direction

Pure core crates can only depend on other pure core crates or external pure libraries.
Effectful crates may depend on pure core crates. Enforced by cargo (no cycles) and
by the dog-fooding rule (DI-008).

```
slideforge-cli
  └─ slideforge (registry + pipeline orchestration)
       ├─ slideforge-syntax    (pure core)
       ├─ slideforge-eval      (pure core)
       ├─ slideforge-validate  (pure core)
       ├─ slideforge-brand     (effectful)
       │    └─ slideforge-types
       │    └─ slideforge-plugin-api
       ├─ slideforge-layout    (pure core)
       │    └─ slideforge-types
       ├─ slideforge-pptx      (effectful)         ← ADR-015
       │    ├─ slideforge-brand   (layout/master/theme XML serialization)
       │    ├─ slideforge-layout  (LaidOutDeck IR types only — no layout::run calls)
       │    ├─ slideforge-types
       │    └─ slideforge-plugin-api
       ├─ slideforge-preview   (effectful)
       ├─ slideforge-data      (effectful)
       └─ slideforge-plugin-api (pure types, root of dep graph)
```

`slideforge-types` and `slideforge-plugin-api` are leaves — they have no workspace
crate dependencies.

### Dependency Notes (ADR-015)

**`slideforge-pptx → slideforge-brand`** (added 2026-06-03, ADR-015): The PPTX
exporter calls `slideforge_brand::layout_xml::serialize_layout_to_xml`,
`serialize_master_to_xml`, and `serialize_theme_to_xml` to obtain XML bytes for
brand parts. No cycle risk: `slideforge-brand` explicitly forbids depending on
`slideforge-pptx` (Architecture Compliance Rule 5, STORY-022).

**`slideforge-pptx → slideforge-layout`** (added 2026-06-03, ADR-015): `LaidOutDeck`
is defined in `slideforge-layout::types`, not `slideforge-types`. The exporter
receives `&LaidOutDeck` as input and may depend on `slideforge-layout` for IR type
access only. The prohibition on calling `slideforge_layout::layout::run` (or any
layout computation function) from within `slideforge-pptx` remains in full force.
