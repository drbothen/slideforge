# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Toolchain:** Rust stable (per `rust-toolchain.toml`), edition 2024, resolver 3. Components: rustfmt, clippy. Cross-compile targets (planned): aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl, x86_64-pc-windows-msvc. 20-crate workspace (7 scaffolded, 13 to be created during Phase 1-3).

---

## Source-of-Truth Precedence

When two artifacts disagree, the **LATER, MORE-SPECIFIC artifact wins**. Apply this rule when adversary, consistency-validator, or spec-reviewer surfaces a conflict between two project documents:

1. **Story spec** (under `.factory/stories/`) supersedes the BC it traces to, when the conflict is about implementation scope. The BC supersedes when the conflict is about contract semantics.
2. **ADR** (under `.factory/specs/architecture/adr/` or numbered `ADR-NNN-*.md`) supersedes earlier ADRs that address the same decision; superseded ADRs are marked with explicit `Supersedes: ADR-NNN` and `Superseded by: ADR-MMM` frontmatter back-refs.
3. **PRD supplements** (`interface-definitions`, `error-taxonomy`, `nfr-catalog`, `test-vectors`) supersede the PRD prose for the same surface area.
4. **VP files** (`.factory/specs/verification-properties/`) supersede the prose verification narrative in PRD/architecture for the property they cover.
5. **DSL design decisions** (`.factory/planning/q1-decision-final.md` through `q16-q25-decisions.md`) supersede the original seed spec (`product-brief.md`) for any topic they address. The 25 Q&A decisions are BINDING.
6. **Recent adversary pass reports** supersede earlier pass reports for the same finding ID.
7. **For code-vs-spec conflicts**: the SPEC wins (Standing Rule for VSDD). Code is brought into alignment via fix-burst or follow-up story, not the other way around. Only the human can authorize spec amendment to match code.

If two artifacts are at the same precedence level and disagree, surface to the orchestrator. The orchestrator routes to the artifact's owner-specialist for adjudication.

---

## Pipeline Authority

The orchestrator (`vsdd-factory:orchestrator` agent) coordinates all phases. Specialist agents do the writing. **The orchestrator does NOT write files itself** — it delegates via the `Agent` tool with `subagent_type` set to the specialist (see Agent Routing Table in the Companion Principle section below).

Phase sequence for slideforge (greenfield mode):

- Pre-Pipeline: Toolchain + LLM + MCP preflight (DONE 2026-05-23)
- Market Intelligence: GO with medium confidence (DONE 2026-05-23)
- Planning: 25 DSL design questions + 14 research threads (DONE 2026-05-24)
- **Phase 1: Spec Crystallization (NEXT)** — domain spec / PRD / architecture / ADRs / adversarial review
- Phase 2: Story Decomposition — stories, dependency graph, wave schedule
- Phase 3: TDD Implementation — per-story delivery through specialist subagents
- Phase 4: Holdout Evaluation (gated on per-wave readiness)
- Phase 5: Adversarial Refinement (post-implementation cascade)
- Phase 6: Formal Hardening (Kani + cargo-fuzz + cargo-mutants + semgrep)
- Phase 7: Convergence — 7-dimensional convergence assessment

Per-story Phase 3 sub-workflow: stubs → failing tests → TDD green → LOCAL adversary 3-CLEAN → demo-recorder per-AC → push → pr-manager 9-step PR cycle → squash-merge → state-manager post-merge burst.

---

## CANONICAL PRINCIPLE — Production-Grade Default

This principle binds every AI agent operating on this project. It overrides any default behavior in agent prompts, skills, or templates that conflicts with it. Declared by the human on 2026-05-23 and recorded in `.factory/STATE.md` Quality Bar section.

### Statement

**Default behavior is enterprise/production-grade correctness. Speed lives in feature *ordering*, not feature *completeness*.**

### Six rules

1. **No MVP-driven deferrals.** Phrases like "for now," "good enough," "we can fix later," "minimum viable," and "ship fast and iterate" are RATIONALIZATIONS, not engineering decisions. Treat them as defect-pattern smells. If a thing is worth doing in v1, it is worth doing correctly in v1.

2. **Feature order is the only acceptable speed lever.** It is acceptable to defer an entire feature (e.g., a future story or wave) to a later cycle. It is NOT acceptable to ship the current story partially or with shortcuts that need later cleanup. Each shipped feature must be production-grade on the cycle it ships.

3. **Tech debt register (`.factory/tech-debt-register.md`) is for HUMAN-DIRECTED deferrals ONLY.** AI agents must NOT add entries to it as a default catchment for issues found during review. If an agent discovers a defect, the default action is to FIX it in-scope. Adding to the register requires ALL of:
   - Explicit human direction to defer, AND
   - A concrete future dependency that makes the deferral necessary, AND
   - Attachment to the specific future story or wave where it will be resolved.

4. **AI-built defects are the AI's responsibility to fix.** Every artifact in `.factory/` and most code in `crates/` was written by AI (with human approval). When an AI agent finds an issue in another AI agent's output, the default is to fix it in the current scope — even if that means expanding scope.

5. **`Suggest` is acceptable. `Default to cheap path` is not.** Agents may propose cheaper alternatives to the human, but the agent's DEFAULT action must be the correct path.

6. **"Pending architect review" / "TODO for architect" / "Placeholder for architect" in spec artifacts is forbidden when the question is answerable in current scope.**

### Self-Audit Checklist (every agent, before declaring work done)

- [ ] Did I rationalize any decision with "MVP," "for now," "good enough," or "we can fix later"?
- [ ] Did I add a new tech-debt-register entry without **all three** of: explicit human direction, concrete future dependency, and a specific future story/wave anchor?
- [ ] Did I leave any "pending architect review" or "TODO" in a spec artifact for a question I could have answered in scope?
- [ ] Did I find a bug or gap in another AI's output and surface it as a question/advisory instead of fixing it in scope?
- [ ] Did I default to the cheapest mechanism instead of the correct mechanism?

### Boundaries

- **It does not mean "do everything before shipping anything."** Phasing waves is correct. Within a wave, every shipped story must be production-grade.
- **It does not mean "no asks of the human."** Genuine human decisions should be surfaced. The principle forbids deferring WORK; it does not forbid surfacing DECISIONS.
- **It does not mean "infinite scope expansion."** If the fix requires new architecture decisions, surface it cleanly and request scope expansion.

### Quality Bar (Non-Negotiable Gates for v1.0 Release)

Declared 2026-05-23. v1.0 release is gated on ALL rows below — no "ship and polish later" tolerated.

| Dimension | Day-1 Gate |
|-----------|-----------|
| Spec convergence | 3 clean adversarial passes on PRD + architecture before Phase 2 starts |
| Tests | Every public API has unit tests; snapshot tests per slide type; integration tests for CLI; fuzz harness in CI |
| Implementation | `#![forbid(unsafe_code)]` (except FFI if added); zero `.unwrap()` outside tests; `clippy::pedantic` clean; `#![warn(missing_docs)]` enforced on public APIs |
| Verification | Kani proofs for pure-core functions in `slideforge-syntax` and `slideforge-eval`; `cargo-fuzz` harness; `cargo-mutants` mutation testing in CI with documented score budget |
| Visual parity | Snapshot tests against rendered XML; CI renders sample decks in headless LibreOffice + screenshots; visual diff against fixtures |
| Performance | < 500ms cold build for 25-slide deck enforced in CI as a benchmark gate (criterion + bench regression check); incremental rebuild < 50ms |
| Documentation | rustdoc on every public item; published to docs.rs on release; user-facing DSL reference book; every ADR signed off |
| Security | `cargo audit` + `cargo deny` in CI; signed release artifacts; SBOM generation per release; semgrep or CodeQL scan per PR; security-reviewer agent on every PR |
| Supply chain | All production-crate deps pinned with `=`; `Cargo.lock` committed; `rust-toolchain.toml` pinned; reproducible builds verified |
| Multi-platform | macOS arm64+x86_64, Linux x86_64+arm64, Windows x86_64 binaries from v1.0; cross-platform CI matrix |
| Multi-renderer parity | Synthesized .pptx must render correctly in PowerPoint (Office), Keynote, Google Slides, LibreOffice — verified via automated rendering + visual diff in CI |
| Observability | `tracing` instrumentation throughout the pipeline; structured logs; opentelemetry-compatible export hooks |
| Accessibility | Web preview audited against WCAG AA via `@axe-core/playwright` on every PR touching the preview; PDF passes `veraPDF` for PDF/UA-1; PPTX passes custom OOXML linter + Microsoft Accessibility Checker (manual per-release) |
| Convergence gate | Full 7-dimension convergence check (spec/tests/impl/verify/visual/perf/docs) before release |
| Holdout eval | Mean satisfaction ≥ 0.85, must-pass ≥ 0.6 (factory default — non-negotiable for v1.0) |

#### Implications
1. **No "ship it, polish later" PRs.** Every merge goes through full per-story-delivery flow with adversarial review, security review, and demo evidence.
2. **Phase 6 formal hardening is non-optional** for v1.0 — Kani + fuzz + mutation testing must all green-light.
3. **CI/CD matrix is built in Phase 1**, before any feature stories start.
4. **Timeline expectation:** v1.0 takes real engineering time. The factory executes rigorously, not fast.

### Companion Principle — Correct Agent Routing

"Fix in scope" works ONLY when paired with correct agent routing. Agents own their domain. The orchestrator owns routing.

#### Agent Routing Table

| If the work is... | Route to agent ID |
|-------------------|-------------------|
| Product brief, PRD, behavioral contracts (BCs), holdout scenarios | `vsdd-factory:product-owner` |
| Market analysis, L2 domain spec, ubiquitous language | `vsdd-factory:business-analyst` |
| Architecture, ADRs, DTU assessment, dependency manifest | `vsdd-factory:architect` |
| UX spec, design system, wireframes, interaction design | `vsdd-factory:ux-designer` |
| Story decomposition, dependency graph, wave schedule | `vsdd-factory:story-writer` |
| Cross-document consistency (IDs, anchors, counts, naming) | `vsdd-factory:consistency-validator` |
| Adversarial fresh-context review (specs or implementation) | `vsdd-factory:adversary` |
| Constructive spec/story review (cognitive diversity) | `vsdd-factory:spec-reviewer` |
| PR diff code review (cognitive diversity) | `vsdd-factory:code-reviewer` |
| Deep codebase scanning, semantic analysis | `vsdd-factory:codebase-analyzer` |
| TDD test stubs and failing tests | `vsdd-factory:test-writer` |
| TDD implementation (failing test → minimum code → micro-commit) | `vsdd-factory:implementer` |
| E2E browser tests (Playwright/Cypress) | `vsdd-factory:e2e-tester` |
| Demo recordings (VHS terminal or Playwright browser) | `vsdd-factory:demo-recorder` |
| PR lifecycle (create, review dispatch, finding triage, merge) | `vsdd-factory:pr-manager` |
| Final fresh-eyes PR diff review before merge | `vsdd-factory:pr-reviewer` |
| Formal proofs (Kani), fuzzing, mutation testing, security scan | `vsdd-factory:formal-verifier` |
| Security review / triage (CWE/CVE, OWASP) | `vsdd-factory:security-reviewer` |
| Holdout scenario evaluation (strict info asymmetry) | `vsdd-factory:holdout-evaluator` |
| Repo setup, worktrees, CI/CD, release, Cargo workspace init | `vsdd-factory:devops-engineer` |
| Toolchain preflight, env setup, dependency installation | `vsdd-factory:dx-engineer` |
| `.factory/STATE.md` updates, `.factory/` commits, cycle bookkeeping | `vsdd-factory:state-manager` |
| Spec governance, versioning, traceability audit | `vsdd-factory:spec-steward` |
| Documentation generation from code/specs (current behavior only) | `vsdd-factory:technical-writer` |
| External research (Perplexity, Context7, Tavily MCP access) | `vsdd-factory:research-agent` |
| GitHub CLI operations on behalf of agents without shell access | `vsdd-factory:github-ops` |
| Performance benchmarks, regression detection | `vsdd-factory:performance-engineer` |
| Data schemas, migrations, pure-core / effectful-I/O boundary | `vsdd-factory:data-engineer` |
| WCAG AA/AAA accessibility audit | `vsdd-factory:accessibility-auditor` |
| Visual regression, mockup fidelity comparison | `vsdd-factory:visual-reviewer` |
| Post-pipeline analysis, lessons capture, improvement proposals | `vsdd-factory:session-reviewer` |

---

## Conventions (Code-Level)

slideforge-specific coding patterns enforced by CI and/or adversarial review. These are non-negotiable under the production-grade default.

### Architecture: Plugin-First

All functionality goes through plugin traits defined in `slideforge-plugin-api`. There is no "built-in code" vs "plugin code" — only plugin code. The built-in plugins ARE the test suite for the plugin API.

10 extensibility surfaces:
1. `DataSource` — fetches external data (json, csv, yaml, toml, http, xlsx, sqlite)
2. `Exporter` — produces output format (pptx, docx, pdf, html, preview)
3. `ChartRenderer` — produces SVG from chart spec (plotters)
4. `DiagramRenderer` — produces SVG from diagram source (mermaid)
5. `Validator` — checks content for issues (overflow, wcag, contrast, alt-text)
6. `MathRenderer` — converts LaTeX to MathML/OMML/HTML (pulldown-latex + KaTeX)
7. `BrandProvider` — loads/synthesizes/extracts brand config (file-based)
8. `SlideType` — defines visual slide pattern + layout (31 built-in types)
9. `SectionType` — defines document section pattern (auto-generated + manual)
10. `InlineFormat` — defines inline formatting rule (bold, italic, code, etc.)

**If a bundled plugin needs to bypass the trait API, the API is wrong and must be fixed.**

### Two-IR Model

The pipeline produces two intermediate representations:
- `Deck` (semantic, pre-layout) — slide content with semantic structure
- `LaidOutDeck` (geometric, post-layout) — positioned shapes with coordinates

Exporters consume BOTH: PPTX needs semantic info for placeholders; PDF/HTML run off `LaidOutDeck`.

All IR types implement `Hash + Eq + Clone` from day 1 (comemo compatibility for future incremental compilation). Use `Arc<str>` + integer EMUs, not `f64`.

### DSL Parser (chumsky 0.10+)

- Mode-based parsing: text mode, math mode, raw mode
- `{{ var }}` interpolation in text mode; `@{var}` in math mode
- `$...$` / `$$...$$` delimiters toggle math mode
- Error accumulation (never fail-on-first) — collect ALL errors in one pass
- Every error carries file:line:col span + correction hint
- Indentation-significant (spaces only, reject tabs hard)
- No YAML-style implicit type coercion (`NO` stays string "NO", `1.10` stays "1.10")

### Strict Defaults

- **`#![forbid(unsafe_code)]`** on all crates (no exceptions in v1.0 — no FFI crate yet)
- **Zero `.unwrap()`** outside of tests and clearly-infallible paths
- **`clippy::pedantic`** enabled with documented exceptions only
- **`#![warn(missing_docs)]`** on all public modules and items
- **Strict mode is the default build.** `slideforge build` fails on validation errors. `--warn-only` for iteration.

### Error Handling

- Crate-level error enums using `thiserror`
- All parser/eval errors carry source spans (file, line, column, range)
- CLI errors render via `miette` with colored source pointers
- No `println!` in library crates — use `tracing::*!` with structured fields

### Testing

- Unit tests in `#[cfg(test)] mod tests` (next to the code)
- Snapshot tests via `insta` for parser AST output, IR, and rendered XML
- Integration tests in `tests/` for end-to-end CLI behavior
- Fixture tests for expected `.pptx` output comparison
- `cargo-fuzz` harness for parser (Phase 6)
- `cargo-mutants` mutation testing (Phase 6)
- Kani proofs for pure-core functions in `slideforge-syntax` and `slideforge-eval` (Phase 6)

### Accessibility (WCAG AA)

- `alt "..."` required on every visual element — compile error if absent
- `decorative: true` opts out (emits empty alt + PDF Artifact tag)
- `lang "en-US"` required at deck level
- Color-coded slide types require `label "..."` co-encoding meaning in text
- Web preview: SVG-based canvas with ARIA (not `<canvas>`)
- Validators: `@axe-core/playwright` (HTML), `veraPDF` (PDF), custom OOXML linter (PPTX)

### OOXML Generation (slideforge-pptx / slideforge-docx)

- Element ordering is schema-significant everywhere (R4 finding)
- Placeholder inheritance: layout→master by `type`, slide→layout by `idx` (R4 finding)
- `clrMapOvr` for dark-themed layouts (R2 finding)
- `notesMaster1.xml` + `handoutMaster1.xml` required even if empty (R2 finding)
- Slide IDs start at 256; master IDs start at 2^31 (R4 finding)
- `[Content_Types].xml` must register every part type comprehensively (R4 finding)
- 11 standard layouts + ~20 custom layouts generated per brand template (R2 finding)

### Forbidden Patterns

| Pattern | Reason |
|---------|--------|
| `unwrap()` / `expect()` on `Result` in non-test code | Error taxonomy rule; use `?` + structured error variants |
| `println!` in library crates | Use `tracing::*!` with structured fields |
| `f64` in IR coordinate/size fields | Use integer EMUs (914400 per inch); comemo Hash compatibility |
| Silent fallback on unknown color names | Must be a compile error (R1 finding: Python silently falls back to TEAL) |
| String-prefix-based bold (`"**header**"`) | Anti-pattern from Python reference; use structural `Inline::Bold` (R1 finding) |
| `{{ }}` interpolation inside `$...$` math blocks | Math mode disables text interpolation; use `@{var}` instead |
| `@include` with circular dependencies | Must produce compile error with cycle shown |
| Implicit type coercion (`NO` → bool, `1.10` → float) | All values stay as-is; strings are strings (R3 finding) |
| `---` as slide separator | Collides with YAML frontmatter + CommonMark thematic break (R3 finding) |
| `raw pptx:` or `raw html:` in user .sf files | Raw escape hatch is IR-internal only; users get shape DSL (R13 finding) |

---

## Build & Test

```bash
# Build the workspace (currently empty stubs — will populate during Phase 3)
cargo build --workspace

# Run all tests
cargo test --workspace --all-features --no-fail-fast

# Format check
cargo fmt --all -- --check

# Clippy (pedantic)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Snapshot tests (once insta fixtures exist)
cargo insta test --check --workspace

# Documentation build
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

### TDD Inner Loop Discipline

When iterating through a TDD fix-burst (closing multiple findings in sequence), use the cheapest verification that proves what you need. Match the tool to the question:

| Question | Command | Time (warm) |
|---|---|---|
| Did my single fix make its target test pass? | `cargo nextest run -p <crate> -E 'test(<test_name>)'` | < 1s after build |
| Did my fix break anything in this crate? | `cargo nextest run -p <crate> --no-fail-fast` | 10-30s |
| See ALL failing tests at once (don't stop at first) | `cargo nextest run -p <crate> --no-fail-fast` | 30-60s |
| Final pre-push gate (workspace canonical) | `just check` | 1min warm / 5-8min cold |

**Common anti-pattern:** running `just check` (full workspace) between every TDD fix in a multi-finding burst. For a 10-fix burst this burns 10-50 minutes. Reserve `just check` for ONCE at end of fix-burst before declaring done.

**Auto-iteration:** `cargo watch -x 'nextest run -p <crate> --no-fail-fast'` re-runs on save.

### Planned Build Commands (Phase 3+)

```bash
# TDD inner loop — single crate
cargo nextest run -p slideforge-syntax -E 'test(test_name)'

# Full pre-push gate
just check          # fmt + clippy + nextest + doctests + layout

# Formal verification (Phase 6)
just kani-local     # Kani proofs
just fuzz-local slideforge-syntax parse_fuzz  # cargo-fuzz
just mutants        # mutation testing
```

### Formal Verification (Phase 6)

Verification properties will have Kani proofs in `crates/slideforge-syntax/src/proofs/` and `crates/slideforge-eval/src/proofs/`. The architecture must be Kani-amenable from day 1:

- Pure functions (no side effects) are provable; impure functions are not
- IR types implement `Hash + Eq + Clone` (required for comemo AND for Kani bounded-model checking)
- Integer EMUs (not `f64`) enable exact arithmetic proofs
- Parser error recovery is bounded (no infinite loops — provably terminating)

**Platform support:** Kani is **Linux/macOS only**. Windows contributors rely on concrete unit tests + CI's Linux/macOS proof job.

VP coverage layers (when Phase 6 activates):
- **Kani proof** (formal, exhaustive within bounds) — Linux/macOS only
- **Concrete unit tests** (specific points, deterministic) — all platforms
- **Fuzz targets** (random exploration) — Linux CI smoke + nightly long-run
- **Mutation testing** (cargo-mutants kill-rate budget) — all platforms

### Implementer Discipline: No-Ignored-Test Rationalization (SID-1)

When no failing test drives a spec-required behavior because integration tests are `#[ignore]`'d (e.g., requires LibreOffice headless, or requires a .pptx template fixture):

1. This is NOT justification to defer the behavior
2. The correct response: add a unit test in the production module's `#[cfg(test)] mod tests` block that drives the behavior WITHOUT the external dependency (mock or stub at the boundary)
3. The unit test must actually exercise the production code path
4. `#[ignore]`'d integration test must include a code comment citing the blocking dependency
5. "Deferred to non-ignored test" is ONLY valid if a SPECIFIC story ID and SPECIFIC test name are cited
6. Implementer must self-check this before declaring a Red Gate test pass

---

## Git Workflow

### Branch model
- **Default branch:** `main` (release branch)
- **Active development:** `develop` (PRs target `develop`)
- **Feature branches:** `feature/<story-id>` (e.g., `feature/S-1.01`)
- **Worktree pattern:** per-story worktrees in `.worktrees/<story-id>/`
- **Factory artifacts branch:** `factory-artifacts` (orphan branch mounted at `.factory/`)

### Commit conventions
- **Conventional Commits** format: `<type>(<scope>): <description>`
- Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `perf`, `ci`
- Scopes: `syntax`, `eval`, `layout`, `pptx`, `docx`, `html`, `pdf`, `cli`, `validate`, `brand`, `data`, `preview`, `plugin-api`, `types`, `math`, `diagrams`, `package`, `docs`, `ci`
- **No AI attribution in commits** — no `Co-Authored-By: Claude`, no robot emojis.

### Non-negotiable git rules
- **NEVER skip hooks** (`--no-verify`, `--no-gpg-sign`). If a hook fails, fix the underlying issue.
- **NEVER force-push to `main`.** Force-push to `develop` requires explicit human approval.
- **NEVER use destructive operations as a first-line response.** Prefer `git stash`, `git reset --soft`, worktree isolation.

### Operational tips
- **Heredoc workaround:** large commit-message heredocs are sometimes blocked by hook payload limits. When `git commit -m "$(cat <<'EOF' ... EOF)"` fails, write the message to `/tmp/<file>` and use `git commit -F /tmp/<file>`.
- **Soft reset for recovery, never `--hard`.** `git -C .factory reset --soft HEAD~N` preserves the working tree state; re-author as a single combined commit.
- **`git stash` for in-progress work** when context-switching between worktrees.
- **Factory-artifacts branch is local-only by default** — orchestrator does NOT push factory-artifacts to remote without explicit user authorization (exception: during planning/bootstrap phases where we've been pushing).

---

## Operational Discipline TDs

These project-wide operational rules layer onto the canonical principle. Enforced by the factory-dispatcher hook chain:

- **TD-VSDD-053 — Single-commit-per-burst.** Each logical burst → ONE commit in `.factory/`. Multi-commit chains (HEAD and HEAD^ both containing "backfill" / "Stage 1" / "Stage 2") trigger `MULTI_COMMIT_CHAIN_NOT_ALLOWED`. Recovery: `git -C .factory reset --soft HEAD~N` then re-author as single commit.

- **TD-VSDD-059 — Paper-fix detection.** State-manager and adversary must verify every claimed closure has a load-bearing test or assertion, not just a doc-comment or rename. Implementer self-disclosure of risk severity is NOT authoritative — adversary independently verifies.

- **TD-VSDD-060 — Sibling-site sweep on value changes.** When changing a function signature, constant, or canonical identifier, grep for ALL callsites in the same crate (and adjacent crates if `pub`) before committing.

- **BC-5.39.001 — 3-CLEAN convergence protocol.** Adversarial cascades require three consecutive clean passes for convergence; any finding resets the streak to 0/3. Applies to both LOCAL and PR-LEVEL cascades.

  **CLEAN (strict)** = ZERO findings of ANY severity. Required for streak advancement.
  **CLEAN (PR-merge)** = ZERO findings of CRIT + HIGH + MED (LOW/OBS non-blocking). PR-merge gate only; does NOT advance streak.

  Adversary CLEAN reports MUST specify both criteria explicitly:
  ```
  CLEAN (strict): yes/no
  CLEAN (PR-merge): yes/no
  ```

---

## Factory Hook Diagnostics

When `Agent` tool dispatches fail with errors like:

```
PreToolUse:Agent hook error: [...factory-dispatcher]: factory-dispatcher trace=<UUID> event=PreToolUse tool=Agent host_abi=1 matched_tiers=N plugins_run=N total_ms=N block_intent=true exit_code=2
```

— the factory-dispatcher hook chain blocked the dispatch. The error message carries NO human-readable reason — only a trace UUID. To diagnose:

### Step 1 — Locate the dispatcher log

```
.factory/logs/dispatcher-internal-YYYY-MM-DD.jsonl
```

### Step 2 — Find the block reason

```bash
grep '<TRACE-UUID>' .factory/logs/dispatcher-internal-$(date +%Y-%m-%d).jsonl
```

Look for `plugin.log` entries with `level: warn` — those carry the human-readable block reason.

### Step 3 — Common blockers and recovery

| Blocker | Detection | Recovery |
|---------|-----------|----------|
| **Multi-commit chain (TD-VSDD-053)** | HEAD and HEAD^ both have "backfill" / "Stage 1" / "Stage 2" | `git -C .factory reset --soft HEAD~N`; re-author as one commit; `--force-with-lease` push (requires human approval) |
| **SHA drift** | STATE.md cites a develop SHA that doesn't match `git rev-parse origin/develop` | Update via state-manager dispatch |
| **In-progress narrative** | STATE.md decision log has an open phase without closure | Add closure row via state-manager |
| **factory-artifacts dirty** | `git -C .factory status --porcelain` is non-empty | Commit/discard pending changes via state-manager |

### Step 4 — Going-forward discipline

- **Bundle backfills.** Stage all files THEN commit ONCE. Never two state-manager dispatches in a row both producing "backfill" commits.
- **Single-commit-per-burst.** Each logical burst → one commit in `.factory/`.
- **Soft-reset for recovery, never `--hard`.**
- **Force-push always needs user approval.**

### Hook source locations (read-only reference)

- Dispatcher binary: `~/.claude/plugins/cache/claude-mp/vsdd-factory/<version>/hooks/dispatcher/bin/<platform>/factory-dispatcher`
- Hook registry: `~/.claude/plugins/cache/claude-mp/vsdd-factory/<version>/hooks-registry.toml`
- Hook plugins (WASM): `~/.claude/plugins/cache/claude-mp/vsdd-factory/<version>/hook-plugins/*.wasm`

---

## Project References

| Path | Description |
|------|-------------|
| `.factory/STATE.md` | Live pipeline state — resumption brief for any new session |
| `.factory/specs/product-brief.md` | Slim 608-word brief (sharded modular specs below) |
| `.factory/specs/decisions-applied.md` | All 25 DSL decisions with canonical doc references |
| `.factory/specs/visual-parity-contract.md` | Binding tolerance spec for snapshot tests |
| `.factory/specs/architecture-overview.md` | Pipeline architecture (parse → eval → layout → export) |
| `.factory/specs/dsl-spec.md` | DSL syntax, keywords, color vocabulary, validation |
| `.factory/specs/slide-types-catalog.md` | 23 original slide types (from seed reference) |
| `.factory/specs/conventions.md` | Code style, error handling, testing, docs, release |
| `.factory/planning/q1-decision-final.md` | Q1: computation, formats, registers, roadmap (~615 lines) |
| `.factory/planning/q2-decision-final.md` | Q2: 31 types, aliases, DSL syntax per type (~340 lines) |
| `.factory/planning/q3-decision-final.md` | Q3: plugin-first, 10 surfaces, trait signatures (~220 lines) |
| `.factory/planning/q4-q15-decisions.md` | Q4-Q15: template, a11y, shapes, variants, etc. (~229 lines) |
| `.factory/planning/q16-q25-decisions.md` | Q16-Q25: packages, workspace, errors, merge (~162 lines) |
| `.factory/planning/spikes-register.md` | 14 spikes (S1-S14) with severity and status |
| `.factory/seed/` | Original seed bundle (PROJECT-SEED.md + DSL grammar + Python reference) |
| `.factory/seed/reference/` | Python reference implementation (IMMUTABLE source-of-truth for visual behavior) |
| `crates/` | Rust workspace (7 scaffolded, 12 more planned — see q1/q3 decision docs) |
| `rust-toolchain.toml` | Pinned Rust toolchain (stable channel) |
| `.github/workflows/ci.yml` | CI workflow (fmt + clippy + test + msrv + docs + snapshots) |

---

## Key Decisions (Quick Reference)

These are the highest-impact decisions from the 25 Q&A session. Full details in the canonical planning docs.

| Decision | Summary |
|----------|---------|
| **Computation** | Data-reactive declarative (rungs 1-9): vars, @data, @for, @if/@elif/@else, expressions, ~15 built-ins. No user functions until v2. |
| **Output formats** | PPTX + DOCX + PDF + HTML + web preview — all from one .sf file |
| **Writing registers** | `notes` (presenter), `report` (reader), `detail` (document-only) — linguistically distinct |
| **Slide types** | 31 built-in (23 seed + chart/toc/agenda/quote/grid/bio/diagram/team) |
| **Architecture** | Plugin-first: 10 extensibility surfaces, 20 crates, dog-food everything |
| **Math** | `$...$` / `$$...$$` LaTeX with mode-based parsing + `@{var}` math interpolation |
| **Charts** | SVG via `plotters` crate (pure Rust, all formats) |
| **Brand** | Bidirectional bridge for BOTH .pptx and .docx + full synthesis from .toml |
| **Packages** | Full Level 4: git-based, `sf.lock`, `@import`, unified with plugin system |
| **Workspace** | Cargo-style `[workspace]` + `.sfconfig` cascade + `slideforge config explain` |
| **Accessibility** | `alt` required (compile error), WCAG AA enforced, `lang` required |
| **Error recovery** | Accumulate all errors, spans + hints, error-slide placeholders in watch mode |
| **Merge semantics** | 11-level precedence chain, last-wins scalars, replace lists, deep-merge maps |
| **Shape DSL** | `shape:` block ships v1.0 (type/position/fill/text/alt) — no raw XML to users |
| **Mermaid** | Ships v1.0 via DiagramRenderer plugin (S14 spike determines engine) |

---

## Version Roadmap (Summary)

- **v1.0** — Data-reactive branded document platform. 31 types, 5 formats, plugin-first, production-grade.
- **v1.x** — Polish: more charts, DOCX Level 2, comemo incremental, slideforge fmt.
- **v2** — Platform: user-defined functions, SmartArt, animations, plugin connectors, mixins, LaTeX output.
- **v3** — Vision: morph transitions, video export, multi-document pipeline, marketplace, collaboration.

Full roadmap: `.factory/planning/q1-decision-final.md` Section 13.
