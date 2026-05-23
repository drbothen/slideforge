# Toolchain Preflight Report — 2026-05-23

## Verdict

**PASS-WITH-NOTES**

All workspace cargo commands pass cleanly. Two notes that require architect attention before Phase 1 spec work begins:

1. **MSRV conflict (HIGH):** `ooxmlsdk >= 0.4.0` declares `rust-version = "1.88"`. The workspace declares `rust-version = "1.85"` and the `rust-toolchain.toml` pins `channel = "stable"` (currently resolves to 1.95.0, so runtime is fine — but the `msrv` CI job pins `dtolnay/rust-toolchain@1.85` and will fail when `ooxmlsdk` is added as a real dependency). Architect must either bump MSRV to 1.88 or find a compatible older `ooxmlsdk` version before Phase 3 implementation begins.
2. **`notify` pre-release:** Latest stable is `8.2.0`; `9.0.0-rc.4` is the newest on crates.io but is a release candidate. Cargo.toml pins `notify = "6"` which resolves to `6.x` — stable, but two major versions behind. Decision needed: stay on 6.x (stable, fine), or adopt 8.x now.

---

## Toolchain

| Item | Value |
|------|-------|
| `rustc` version | 1.95.0 (59807616e 2026-04-14) |
| `cargo` version | 1.95.0 (f2d3ce0bd 2026-03-21) |
| `rust-toolchain.toml` channel | `stable` (no explicit version pin — resolves at install time) |
| `workspace.package.rust-version` (MSRV) | `1.85` |
| MSRV match (runtime toolchain) | YES — 1.95.0 >= 1.85 |
| MSRV match (CI msrv job) | YES — `dtolnay/rust-toolchain@1.85` used in `msrv` job |
| Edition | `2024` (Rust 2024 edition — requires 1.85+, confirmed compatible) |

---

## Workspace Health

| Check | Result |
|-------|--------|
| `cargo build --workspace` | PASS — 7 crates compiled, 0.20s (empty stubs, no deps resolved yet) |
| `cargo test --workspace` | PASS — 0 tests across 7 crates, 0 failures |
| `cargo fmt --check` | PASS — no formatting issues |
| `cargo clippy --workspace -- -D warnings` | PASS — 0 warnings, 0 errors |

All four workspace health checks are green on the current scaffolding.

---

## Required Dependencies (Resolved Versions)

| Crate | Required (Cargo.toml) | Latest on crates.io | Status | Notes |
|-------|----------------------|---------------------|--------|-------|
| `chumsky` | `"0.10"` | 0.13.0 | OK | Cargo `"0.10"` req resolves to latest 0.10.x (0.10.1). Consider bumping to `"0.13"` — confirmed in Context7 docs as current stable. MSRV: 1.65. |
| `ooxmlsdk` | `"0.6"` | 0.6.1 | **NOTE** | 0.6.1 exists and is not yanked. Context7 confirms `.pptx` support. **MSRV conflict: ooxmlsdk >= 0.4.0 requires rust-version 1.88; workspace declares 1.85. Must resolve before Phase 3.** |
| `serde` | `"1"` | 1.0.228 | OK | Stable, widely used. |
| `serde_derive` | (via `serde` `derive` feature) | — | OK | Included in `serde` feature flags. |
| `miette` | `"7"` (fancy feature) | 7.6.0 | OK | Diagnostics rendering. Chosen over `ariadne` (0.6.0 also available). |
| `ariadne` | Not in Cargo.toml (alternative) | 0.6.0 | AVAILABLE | Not selected; `miette` chosen. Either viable for diagnostics. |
| `clap` | `"4"` (derive feature) | 4.6.1 | OK | Stable. |
| `walkdir` | `"2"` | 2.5.0 | OK | Stable. |
| `notify` | `"6"` | 8.2.0 (stable); 9.0.0-rc.4 (pre-release) | NOTE | Pinned to `"6"` (6.x). Latest stable is 8.2.0, latest overall is 9.0.0-rc.4. Recommend upgrading to `"8"` before Phase 3 to avoid accumulating a two-major-version gap. |
| `tracing` | `"0.1"` | 0.1.44 | OK | Stable. |
| `tracing-subscriber` | `"0.3"` | — | OK | Standard companion crate. |
| `thiserror` | `"1"` | 2.0.18 | NOTE | `"1"` req resolves to 1.x. Latest is 2.x (breaking). Consider intentionally bumping to `"2"` for new project — it is API-compatible for typical use. |
| `anyhow` | `"1"` | — | OK | Standard error handling. |
| `insta` | `"1"` | 1.47.2 | OK | Snapshot testing. Stable. |
| `comemo` | Not in Cargo.toml | 0.5.1 | AVAILABLE | Typst's memoization crate — needed for Phase 4 incremental compilation. Should be added to `[workspace.dependencies]` when Phase 4 story is scoped. |

### New Scope Dependencies (Feasibility Check)

| Category | Crates Investigated | Latest Version | Verdict |
|----------|---------------------|----------------|---------|
| PDF backend — Typst-as-library | `typst` | 0.14.2 | VIABLE — full document model, good Rust-native PDF output. Complex integration. Architect to decide via ADR-003. |
| PDF backend — direct Rust | `printpdf` | 0.9.1 | VIABLE — lower-level, no external deps. Less feature-rich than Typst. |
| PDF backend — direct Rust | `lopdf` | 0.40.0 | VIABLE — PDF read/write/modify. Lower-level than printpdf. |
| HTML templating | `maud` | 0.27.0 | VIABLE — compile-time HTML macros, zero-overhead. |
| HTML templating | `askama` | 0.16.0 | VIABLE — Jinja2-style templates, type-safe. Either would work for the HTML exporter. |
| Web preview server | `axum` | 0.8.9 | VIABLE — production-ready async HTTP. Good fit for embedded server. |
| Web preview WebSocket | `tokio-tungstenite` | 0.29.0 | VIABLE — standard WebSocket crate. Pairs naturally with axum. |
| WCAG AA accessibility (Rust) | `accesskit` | 0.24.0 | PARTIAL — UI accessibility tree, not WCAG HTML audit. |
| WCAG AA accessibility (Rust) | `wcag`, `axe-core` | NOT on crates.io | ABSENT — no Rust-native WCAG HTML linting library exists. |
| LibreOffice headless rendering | external binary (`soffice`) | N/A | See External Infrastructure below. |

**WCAG AA note:** No Rust-native WCAG AA HTML linting library exists on crates.io. The web preview accessibility requirement (Quality Bar row: "WCAG AA via accessibility-auditor on every PR") will require an external approach: either a headless browser + axe-core (Node.js), a pa11y/lighthouse CLI, or a Playwright MCP accessibility check. Architect should clarify the tooling path before Phase 1 accessibility story is written.

---

## Production-Grade Tooling (Presence Check — No Install)

| Tool | Installed? | Version | Required For |
|------|-----------|---------|--------------|
| `cargo audit` | YES | 0.22.1 | Supply chain security — CVE scanning in CI |
| `cargo deny` | YES | 0.19.0 | License + dependency policy enforcement in CI |
| `cargo fuzz` | YES | 0.13.1 | Fuzz harness for `slideforge-syntax` parser (Phase 6) |
| `cargo mutants` | YES | 27.0.0 | Mutation testing score gate (Phase 6) |
| `cargo bench` / criterion | Not standalone | — | Auto-installed via dev-dependency on first bench run; no separate install needed |
| `cargo insta` | Not checked as standalone | — | Installed via `cargo install cargo-insta` in ci.yml; needed for snapshot review workflow |
| `cargo kani` | NOT CHECKED | — | Kani formal verification (Phase 6 hardening) — devops-engineer to install in Phase 1 |
| `vhs` | NOT CHECKED | — | CLI demo recording — devops-engineer to install in Phase 1 |
| `semgrep` | NOT CHECKED | — | SAST scanning per PR — devops-engineer to install in Phase 1 |
| `docker` / `docker-compose` | NOT CHECKED | — | Container builds, reproducible-build verification |

All four cargo sub-commands required for quality bar (audit, deny, fuzz, mutants) are present locally. No install action needed for these.

---

## External Infrastructure

| Item | Status | Notes |
|------|--------|-------|
| LibreOffice headless (`soffice` / `libreoffice`) | ABSENT — not installed locally | Required for multi-renderer snapshot tests (Quality Bar: "CI renders sample decks in headless LibreOffice + screenshots"). Available via `apt install libreoffice` on Ubuntu CI runners, `brew install --cask libreoffice` on macOS. Local install not required for development; CI matrix must include it. |
| `cargo-kani` | NOT CHECKED | Required for Phase 6 Kani proofs. Devops-engineer to provision in Phase 1. |
| `gh` CLI | NOT CHECKED | Required for PR automation in per-story-delivery flow. |

---

## CI Workflow Gap Analysis

### Current `ci.yml` Does

The existing `.github/workflows/ci.yml` is well-structured and covers more than a minimal scaffold:

1. `fmt` job — `cargo fmt --all -- --check` (ubuntu-latest)
2. `clippy` job — `cargo clippy --workspace --all-targets --all-features -- -D warnings` (ubuntu-latest, with `rust-cache`)
3. `test` job — `cargo test --workspace --all-features --no-fail-fast` on matrix: [ubuntu-latest, macos-latest, windows-latest]
4. `msrv` job — `cargo check --workspace --all-features` pinned to toolchain `1.85`
5. `docs` job — `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"`
6. `snapshots` job — `cargo insta test --check --workspace` (ubuntu-latest)

### Missing for Production-Grade Quality Bar

| Gap | Severity | Required By Quality Bar |
|-----|----------|------------------------|
| `cargo audit` in CI | HIGH | Supply chain: "cargo audit + cargo deny in CI" |
| `cargo deny` in CI | HIGH | Supply chain: "cargo deny in CI" |
| Benchmark regression gate (criterion) | HIGH | Performance: "< 500ms cold build... enforced in CI as benchmark gate" |
| Fuzz smoke test in CI | HIGH | Verification: "cargo-fuzz harness in CI" |
| Mutation testing (`cargo mutants`) score gate | HIGH | Verification: "cargo-mutants mutation testing in CI with documented score budget" |
| Kani proof job | HIGH | Verification: "Kani proofs for pure-core functions" |
| Cross-platform binary build matrix (arm64 targets) | MEDIUM | Multi-platform: "macOS arm64+x86_64, Linux x86_64+arm64, Windows x86_64" |
| LibreOffice headless render test job | MEDIUM | Visual parity: "CI renders sample decks in headless LibreOffice + screenshots; visual diff against fixtures" |
| Semgrep / CodeQL SAST scan | MEDIUM | Security: "semgrep or CodeQL scan per PR" |
| SBOM generation job | MEDIUM | Security: "SBOM generation per release" |
| WCAG AA accessibility audit (PR-level) | MEDIUM | Accessibility: "web preview audited against WCAG AA on every PR touching the preview" |
| Signed release artifact job | LOW (release-time) | Security: "signed release artifacts" |
| Reproducible build verification | LOW (release-time) | Supply chain: "reproducible builds verified" |
| `cargo-deny` license check | HIGH | Supply chain: license policy enforcement |
| Action SHA pinning | MEDIUM | Supply chain: "GitHub Actions: commit SHA (not version tags)" — current uses `@v4`, `@stable`, `@v2` tags |

### Recommended Phase 1 Stories for CI Matrix Expansion

1. **`phase-1-cicd-security-gates`** — Add `cargo audit`, `cargo deny` (license + ban policy), and Semgrep/CodeQL jobs. Add `deny.toml` policy file. Pin all GitHub Actions to commit SHAs.
2. **`phase-1-cicd-cross-platform-matrix`** — Expand test job to include arm64 targets; add cross-compile job using `cross` tool for Linux arm64.
3. **`phase-1-cicd-benchmark-gate`** — Add criterion benchmark baseline and regression check job; define 500ms cold-build / 50ms incremental-rebuild gate as CI-enforceable benchmark.
4. **`phase-1-cicd-fuzz-smoke`** — Add fuzz smoke job (short duration, 30s) against `slideforge-syntax` parser once fuzz harness stub is written.
5. **`phase-1-cicd-libreoffice-render`** — Add Ubuntu CI job with `apt install libreoffice`; render sample `.pptx` fixture in headless mode and capture screenshot for visual diff.
6. **`phase-1-cicd-kani`** — Add `cargo kani` job for pure-core proof functions once Kani harness is set up.

---

## Risks Identified

| Risk | Severity | Mitigation |
|------|----------|-----------|
| **MSRV conflict: `ooxmlsdk >= 0.4.0` requires rust-version 1.88; workspace declares 1.85** | HIGH | Architect must decide: bump MSRV to 1.88 (recommended — 1.88 is stable as of 2025-06), or pin `ooxmlsdk` to `"0.3"` (MSRV 1.73, but missing features). Decision required before any crate that depends on `ooxmlsdk` is implemented. ADR required. |
| **`ooxmlsdk` PPTX coverage depth unknown** | HIGH | Context7 confirms `.pptx` support exists, but the crate's primary development focus appears to be `.docx`/`.xlsx` (ported from .NET Open XML SDK). Full PPTX feature coverage (slide masters, layouts, themes, notes, charts) needs hands-on validation in Phase 1. If gaps exist, fallback is raw OOXML via `zip` + `quick-xml` (more work, full control). |
| **No Rust-native WCAG AA HTML linting library** | MEDIUM | WCAG AA accessibility requirement for web preview cannot be satisfied by a pure-Rust crate. External tooling (pa11y, axe-core via Playwright, or lighthouse CLI) is required. Adds Node.js toolchain dependency to CI. Architect must clarify approach for ADR or Quality Bar footnote before Phase 3. |
| **`notify` pinned to v6; latest stable is v8** | LOW | Two major versions behind. New API in v8 includes better debouncing and inotify improvements. Safe to stay on 6.x through Phase 3; recommend upgrading before Phase 4 (watch mode). |
| **`chumsky` 0.10 vs 0.13 API differences** | LOW | Cargo req `"0.10"` resolves to 0.10.x. The 0.13.x release may have breaking changes vs 0.10. Since this is a new project, recommend bumping the Cargo.toml req to `"0.13"` during Phase 1 dependency lock-in. Context7 confirms 0.13 is current. |
| **GitHub Actions using tag refs (`@v4`, `@stable`) not SHA-pinned** | MEDIUM | Quality bar requires SHA pinning for supply chain security. `ci.yml` currently uses `@v4`, `@v2`, `@stable`. Phase 1 security story must add Dependabot and pin all actions to commit SHAs. |
| **LibreOffice absent locally** | LOW | Local absence does not block development. CI renders on Ubuntu runners. Devs who want local visual parity testing need a separate install step (not automated). |

---

## Recommendations for Phase 1

1. **Resolve MSRV conflict immediately (BLOCKING):** Bump `workspace.package.rust-version` from `"1.85"` to `"1.88"` and update the `msrv` CI job from `dtolnay/rust-toolchain@1.85` to `@1.88`. This is the correct move — 1.88 is stable, and all other crates in scope are compatible. Update `rust-toolchain.toml` comment accordingly. This is a prerequisite for any `ooxmlsdk` integration.

2. **Pin `chumsky` to `"0.13"` in Cargo.toml:** New project, no migration cost. Locks to current stable API. Context7 confirms the 0.13 API as the canonical reference for docs.

3. **Build the full CI matrix as first Phase 1 deliverable** (before feature stories): `cargo audit`, `cargo deny`, benchmark gate, fuzz smoke, SAST scan, LibreOffice render test, SHA-pinned actions, and cross-platform build matrix. The Quality Bar is non-negotiable — starting feature work before the CI guardrails are in place creates debt that compounds.

4. **Spike `ooxmlsdk` PPTX coverage in Phase 1:** Before committing to `ooxmlsdk` as the PPTX serialization backend, implement a small spike: generate a minimal single-slide `.pptx` with a slide master + one layout. Validate that Keynote, Google Slides, and LibreOffice open it correctly. If gaps appear, evaluate raw OOXML construction as fallback (ADR-001 input).

5. **Add `comemo` to `[workspace.dependencies]`** now (even if not yet used in any crate): pins the version early and signals the Phase 4 incremental compilation dependency clearly in the manifest.

6. **Clarify WCAG AA tooling strategy for ADR or Quality Bar footnote:** The accessibility requirement for the web preview requires a non-Rust tool in CI (pa11y, playwright+axe, or lighthouse). Decide the approach before Phase 3 web preview story is decomposed, so the CI job can be designed correctly.

---

*Report generated by dx-engineer preflight agent. Workspace: `/Users/jmagady/Dev/slideforge`. No files were modified. No tools were installed.*
