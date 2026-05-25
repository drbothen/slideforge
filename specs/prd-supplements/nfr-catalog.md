---
document_type: prd-supplement
supplement_type: nfr-catalog
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
traces_to: .factory/specs/prd.md
primary_consumers: [architect, performance-engineer]
---

# NFR Catalog — slideforge v1.0

> Non-functional requirements with numerical targets and validation methods.
> NFRs are cross-cutting concerns that do not map to a single BC.
> All targets are binding for v1.0 release per CLAUDE.md quality bar.

---

## Performance NFRs

| NFR-ID | Category | Requirement | Numerical Target | Validation Method | Risk Source |
|--------|---------|-------------|-----------------|------------------|------------|
| NFR-001 | Performance | Cold build time for a 25-slide deck with brand.toml synthesis | < 500ms wall-clock | CI criterion benchmark on Linux x86_64 GitHub Actions runner | R-005, ASM-011 |
| NFR-002 | Performance | Incremental rebuild in watch mode (single .sf file change, no data re-fetch) | < 50ms wall-clock | CI criterion benchmark | R-005 |
| NFR-003 | Performance | Diagram cold font DB scan (mermaid-rs-renderer first call per process) | < 200ms wall-clock | Unit benchmark | S14 finding: 124ms on M-series |
| NFR-004 | Performance | Warm diagram render per diagram (mermaid-rs-renderer, font DB cached) | < 10ms per diagram | Unit benchmark (Criterion) | S14 finding: < 3ms typical |
| NFR-005 | Performance | PPTX/DOCX serialization for a 25-slide deck | < 200ms wall-clock | CI criterion benchmark | R-005 |
| NFR-006 | Performance | Memory usage peak for a 25-slide deck with @data sources | < 256MB resident | CI memory profiling (heaptrack or valgrind) | R-005 |

**Validation workflow for NFR-001/002:**
1. Implement the criterion benchmark in `crates/slideforge-cli/benches/build_bench.rs`
2. The benchmark fixture: 25 slides using at least 10 slide types, brand.toml synthesis, 2 @data sources (file-based)
3. CI uses `cargo bench` and compares vs threshold; fails PR if threshold exceeded
4. Gate: **blocking** — PR cannot merge if NFR-001 is violated

---

## Visual Parity NFRs

| NFR-ID | Category | Requirement | Numerical Target | Validation Method | Risk Source |
|--------|---------|-------------|-----------------|------------------|------------|
| NFR-007 | Visual Parity | PPTX rendered by LibreOffice Still 25.8.7 vs reference PNG — SSIM | SSIM ≥ 0.99 per slide | CI visual regression job (scikit-image) | R-001, S6 finding |
| NFR-008 | Visual Parity | PPTX rendered by LibreOffice Still 25.8.7 vs reference PNG — PSNR | PSNR ≥ 35dB per slide | CI visual regression job (scikit-image) | R-001, S6 finding |
| NFR-009 | Visual Parity | PPTX slide count matches expected count | Exact match | CI: PNG file count vs expected | R-001 |
| NFR-010 | Visual Parity | Cross-renderer divergence: LibreOffice vs PowerPoint (informational, not gated) | SSIM ≥ 0.90 per slide | Manual phase-gate review; recorded in divergence-log.md | R-001, R-016 |
| NFR-011 | Visual Parity | Shape position drift | ≤ 4pt (≤ 17px at 300 DPI) | Visual regression diff artifacts | S6 finding |

**Visual regression CI pipeline (per S6 spike):**
1. `cargo test` generates `tests/fixtures/test-fixture.pptx` (all 31 slide types)
2. LibreOffice headless: PPTX → PDF → PNG at 300 DPI via ImageMagick
3. `visual-diff.py`: compute SSIM + PSNR per slide vs committed reference PNGs
4. Gate: fail if any slide fails SSIM < 0.99 OR PSNR < 35dB
5. Reference PNG updates require human review + explicit commit message

---

## Accessibility NFRs

| NFR-ID | Category | Requirement | Numerical Target | Validation Method | Risk Source |
|--------|---------|-------------|-----------------|------------------|------------|
| NFR-012 | Accessibility | PDF output — PDF/UA-1 compliance via veraPDF | 0 veraPDF violations | CI: `verapdf --flavour ua1` gate | S2, R-003, DI-014 |
| NFR-013 | Accessibility | HTML web preview — WCAG AA via axe-core | 0 axe-core critical violations | CI: @axe-core/playwright on every PR touching HTML/preview | ASM-013 |
| NFR-014 | Accessibility | PPTX — alt text present on all non-decorative visual elements | 100% coverage | Compile-time enforcement + snapshot test inspection | DI-001 |
| NFR-015 | Accessibility | PPTX — PDF/UA-1-equivalent: accessibility metadata present | All required fields set | PPTX linter test (custom) | DI-001, DI-003 |

---

## Security NFRs

| NFR-ID | Category | Requirement | Numerical Target | Validation Method | Risk Source |
|--------|---------|-------------|-----------------|------------------|------------|
| NFR-016 | Security | No high or critical CVEs in production dependencies | 0 high/critical CVEs | CI: `cargo audit` on every PR | R-011, R-012 |
| NFR-017 | Security | No denied licenses in production dependencies | 0 violations | CI: `cargo deny` on every PR | R-015 |
| NFR-018 | Security | Package SHA-256 integrity check | 100% packages verified at use time | Unit test: verify checksum before @import resolution | R-012 |
| NFR-019 | Security | HTTP data source URL allowlist enforced | No requests to non-allowlisted domains when allowlist configured | Unit + integration test | R-011 |
| NFR-020 | Security | SBOM generated on release | SBOM present in release artifacts | CI release job check | R-011, R-012 |

---

## Code Quality NFRs

| NFR-ID | Category | Requirement | Numerical Target | Validation Method | Risk Source |
|--------|---------|-------------|-----------------|------------------|------------|
| NFR-021 | Code Quality | `.unwrap()` usage in non-test code | 0 occurrences | CI: `cargo clippy -- -D clippy::unwrap_used` on non-test targets | CLAUDE.md quality bar |
| NFR-022 | Code Quality | `clippy::pedantic` violations | 0 undocumented suppressions | CI: `cargo clippy --all-targets -- -D warnings` | CLAUDE.md quality bar |
| NFR-023 | Code Quality | Missing documentation on public API items | 0 undocumented public items | CI: `RUSTDOCFLAGS="-D warnings" cargo doc` | CLAUDE.md quality bar |
| NFR-024 | Code Quality | `#![forbid(unsafe_code)]` enforcement | All crates except approved FFI boundary | CI: `cargo clippy` (lint is compile-level) | CLAUDE.md quality bar |
| NFR-025 | Code Quality | Supply chain: production dep version pinning | 100% of production crate deps use `=` version specifier | CI: grep Cargo.toml for unpinned production deps | CLAUDE.md quality bar, R-015 |

---

## Compatibility NFRs

| NFR-ID | Category | Requirement | Numerical Target | Validation Method | Risk Source |
|--------|---------|-------------|-----------------|------------------|------------|
| NFR-026 | Compatibility | macOS arm64 binary builds and passes tests | All CI steps pass | CI matrix: macos-14 (arm64) | CLAUDE.md quality bar |
| NFR-027 | Compatibility | macOS x86_64 binary builds and passes tests | All CI steps pass | CI matrix: macos-13 (x86_64) | CLAUDE.md quality bar |
| NFR-028 | Compatibility | Linux x86_64 binary builds and passes tests | All CI steps pass | CI matrix: ubuntu-latest (x86_64) | CLAUDE.md quality bar |
| NFR-029 | Compatibility | Linux arm64 binary builds and passes tests | All CI steps pass | CI matrix: ubuntu-24.04-arm | CLAUDE.md quality bar |
| NFR-030 | Compatibility | Windows x86_64 binary builds and passes tests | All CI steps pass | CI matrix: windows-latest (x86_64) | CLAUDE.md quality bar, R-016 |
| NFR-031 | Compatibility | Reproducible builds: same source + same toolchain → same binary | Binary hash equality between builds | CI reproducible-build job | CLAUDE.md quality bar |

---

## Observability NFRs

| NFR-ID | Category | Requirement | Numerical Target | Validation Method | Risk Source |
|--------|---------|-------------|-----------------|------------------|------------|
| NFR-032 | Observability | All 6 pipeline stages (Parse/Evaluate/Brand/Validate/Layout/Export) emit tracing spans | 6 spans present per build | tracing instrumentation audit (test helper: span_counter) | CLAUDE.md quality bar |
| NFR-033 | Observability | Structured diagnostic JSON output complete and valid | JSON parses without error; all fields present | Unit test: parse JSON output for every test fixture | CLAUDE.md quality bar |

---

## Holdout Evaluation NFRs

| NFR-ID | Category | Requirement | Numerical Target | Validation Method | Risk Source |
|--------|---------|-------------|-----------------|------------------|------------|
| NFR-034 | Holdout Eval | Phase 4 holdout mean satisfaction score | ≥ 0.85 | holdout-evaluator agent Phase 4 | ASM-001, R-009 |
| NFR-035 | Holdout Eval | Phase 4 holdout must-pass scenario pass rate | ≥ 0.60 | holdout-evaluator agent Phase 4 | ASM-001, R-009 |

---

## NFR Dependencies on Spikes

| NFR-ID | Spike | Dependency |
|--------|-------|-----------|
| NFR-012 | S2 | pdf-writer + krilla must produce PDF/UA-1. veraPDF CI gate specified in S2. |
| NFR-007, NFR-008 | S6 | SSIM ≥ 0.99 AND PSNR ≥ 35dB thresholds empirically validated in S6. |
| NFR-003, NFR-004 | S14 | mermaid-rs-renderer < 200ms cold, < 10ms warm measured in S14. |
