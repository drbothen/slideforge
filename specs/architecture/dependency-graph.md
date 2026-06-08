---
document_type: architecture-section
section: dependency-graph
version: "1.1"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
modified:
  - "2026-06-07: v1.1 — Wave-5 remove-uncertainty updates (ADR-021, ADR-022):
     tokio pin corrected to =1.52.3 with curated features; axum corrected to =0.8.9
     (0.8.2 yanked); reqwest updated to =0.13.4 with default-features=false/rustls-tls;
     sha2 unified on =0.11.0 (was split =0.10.9/=0.11.0); toml updated to =1.1.2;
     notify updated to =8.2.0; notify-debouncer-full =0.7.0 added; git2 =0.21.0 added;
     tar =0.4.46 + flate2 =1.1.9 added; crossterm =0.29.0 added; dirs =6.0.0 added;
     globset =0.4.18 added; hex =0.4.3 added; toml_edit =0.25.12 added;
     tokio-tungstenite =0.29.0 added (dev-only); thiserror corrected to =2.0.18;
     clap corrected to =4.6.1; miette corrected to =7.6.0;
     all loose version pins corrected to exact minor."
traces_to: ARCH-INDEX.md
---

# Dependency Graph

## External Production Dependencies

All production crate deps use `=` version pinning (NFR-025). Dev/test deps use
compatible-version constraints. `Cargo.lock` is committed. All entries below are
registry-verified (2026-06-07); MSRV ≤ 1.88 confirmed for all.

Every dep listed here MUST be declared in `[workspace.dependencies]` (ADR-022,
Decision 1). Member crate `Cargo.toml` files MUST reference via `{ workspace = true }`.

### Core Pipeline

| Dep | Pinned Version | Category | License | Used In | ADR |
|-----|---------------|----------|---------|---------|-----|
| ooxmlsdk | =0.6.1 | PPTX/DOCX generation | MIT | slideforge-pptx, slideforge-docx | ADR-001 |
| pdf-writer | =0.14.0 | PDF low-level (transitive via krilla) | Apache-2.0 | slideforge-pdf (via krilla only — NOT direct dep) | ADR-003 |
| krilla | =0.6.0 | PDF drawing + tagged PDF | Apache-2.0 | slideforge-pdf | ADR-003 |
| usvg | =0.47.0 | SVG normalization | MIT/Apache-2.0 | slideforge-pdf, slideforge-diagrams | — |
| subsetter | =0.2.0 | Font subsetting | Apache-2.0 | slideforge-pdf | — |
| mermaid-rs-renderer | =0.2.2 | Diagram rendering | MIT | slideforge-diagrams | ADR-014 |
| chumsky | =0.10.1 | Parser combinator | MIT | slideforge-syntax | ADR-009 |
| plotters | =0.3.7 | SVG chart generation | MIT | slideforge-charts | — |
| quick-xml | =0.36.0 | XML parsing/construction | MIT | slideforge-pptx, slideforge-brand | — |
| zip | =2.1.0 | ZIP handling | MIT | slideforge-pptx (opc_postprocess) | — |

### Async Runtime and Networking (ADR-021)

| Dep | Pinned Version | Features | Category | License | Used In |
|-----|---------------|----------|----------|---------|---------|
| tokio | =1.52.3 | rt-multi-thread, macros, net, time, sync, signal, fs, io-util | Async runtime | MIT | slideforge-preview, slideforge-data, slideforge-cli |
| axum | =0.8.9 | ws | HTTP server | MIT | slideforge-preview |
| reqwest | =0.13.4 | default-features=false; rustls-tls, charset, http2 | HTTP client | MIT/Apache-2.0 | slideforge-data |
| crossterm | =0.29.0 | event-stream | Terminal I/O | MIT | slideforge-cli |

Note: `pdf-writer` is listed above as "transitive via krilla only" — slideforge-pdf
MUST NOT declare pdf-writer as a direct dependency (version skew risk).

### Serialization and Configuration

| Dep | Pinned Version | Features | Category | License | Used In |
|-----|---------------|----------|----------|---------|---------|
| serde | =1.0.228 | derive | Serialization | MIT/Apache-2.0 | multiple |
| serde_json | =1.0.150 | — | JSON | MIT/Apache-2.0 | slideforge-data, slideforge-eval |
| toml | =1.1.2 | — | TOML parsing | MIT/Apache-2.0 | slideforge-config, slideforge-data, slideforge-brand, slideforge-math |
| toml_edit | =0.25.12 | — | Structure-preserving TOML edits | MIT/Apache-2.0 | slideforge-config, slideforge-package |
| calamine | =0.24.0 | — | Excel reading | MIT | slideforge-data |
| rusqlite | =0.31.0 | — | SQLite | MIT | slideforge-data |

### Error Handling and Observability

| Dep | Pinned Version | Features | Category | License | Used In |
|-----|---------------|----------|----------|---------|---------|
| thiserror | =2.0.18 | — | Error derives | MIT/Apache-2.0 | all error enums |
| miette | =7.6.0 | — | Error rendering | Apache-2.0 | slideforge-cli |
| tracing | =0.1.44 | — | Instrumentation | MIT | all crates |
| tracing-subscriber | =0.3.23 | env-filter | Tracing setup | MIT | slideforge-cli |

### CLI and File System

| Dep | Pinned Version | Features | Category | License | Used In |
|-----|---------------|----------|----------|---------|---------|
| clap | =4.6.1 | derive | CLI parsing | MIT/Apache-2.0 | slideforge-cli |
| notify | =8.2.0 | — | File watching | CC0-1.0 | slideforge-cli |
| notify-debouncer-full | =0.7.0 | — | Debounced file watching | CC0-1.0 | slideforge-cli |
| globset | =0.4.18 | — | Glob pattern matching | MIT/Apache-2.0 | slideforge-package, slideforge-config |
| dirs | =6.0.0 | — | Platform home dir | MIT/Apache-2.0 | slideforge-cli, slideforge-package |
| tempfile | =3.27.0 | — | Temp files/dirs | MIT/Apache-2.0 | slideforge-package, tests |

### Package Management (slideforge-package)

| Dep | Pinned Version | Features | Category | License | Used In |
|-----|---------------|----------|----------|---------|---------|
| git2 | =0.21.0 | default-features=false; vendored-libgit2, https | Git operations | MIT/Apache-2.0 | slideforge-package |
| tar | =0.4.46 | — | Archive creation | MIT/Apache-2.0 | slideforge-package |
| flate2 | =1.1.9 | — | Gzip compression | MIT/Apache-2.0 | slideforge-package |
| sha2 | =0.11.0 | — | SHA-256 | MIT/Apache-2.0 | slideforge-package, slideforge-pptx |
| hex | =0.4.3 | — | Hex encoding | MIT/Apache-2.0 | slideforge-package, slideforge-pptx |

Note: flate2 1.1.6 and 1.1.7 are YANKED. 1.1.9 is the safe minimum. sha2 is unified on
0.11.0 across the workspace — slideforge-pptx GUID derivation must use 0.11.0 (see ADR-022).

### Dev/Test Only (NOT production deps)

| Dep | Pinned Version | Category | Used In |
|-----|---------------|----------|---------|
| tokio-tungstenite | =0.29.0 | WebSocket test client | slideforge-preview (dev-dep) |

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
