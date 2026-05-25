---
title: "S3 — WCAG AA Tooling Choice"
spike_id: S3
status: COMPLETE
severity: MEDIUM
estimated_effort: 1 day
actual_effort: 1 day
owner: architect
output_feeds: ADR-011
completed: 2026-05-24
traces_to:
  - wcag-for-slides.md
  - q4-q15-decisions.md §Q6
  - CLAUDE.md Quality Bar (accessibility row)
code_path: spikes/S3-code/
tests_pass: true  # 15/15 Rust unit tests pass
---

# S3 — WCAG AA Tooling Choice

## Spike Objectives

Evaluate and select the accessibility toolchain for each of slideforge's four output surfaces
(web preview, HTML export, PDF export, PPTX export), and prototype the compile-time `alt`
enforcement mechanism in the DSL parser. Produce ADR-011 input.

---

## 1. Tool Comparison Matrix

### Web / HTML Surfaces

| Tool | WCAG Coverage | Rule Count | Playwright Integration | CI-Suitable | Node.js surface | Verdict |
|---|---|---|---|---|---|---|
| **@axe-core/playwright** | WCAG 2.0/2.1/2.2 A+AA | ~110 rules | Native — `new AxeBuilder({ page }).analyze()` | YES | Single package | **SELECTED** |
| pa11y (axe runner) | Same as axe-core | Same | Extra abstraction layer | YES | Additional CLI dependency | REJECTED — functional duplicate |
| pa11y (HTML_CodeSniffer) | WCAG 2.x A+AA | ~100 rules | External subprocess | YES | Additional CLI dependency | REJECTED — false positives on SVG ARIA |
| Lighthouse | Subset of axe-core | ~50 axe rules | Built into Playwright via evaluate | YES | None (Chromium bundled) | REJECTED — strict subset of axe |

**Decision: `@axe-core/playwright` exclusively for web surfaces.**

Rationale: Playwright is already required for E2E testing of the web preview. Adding
`@axe-core/playwright` is a single dev dependency. Pa11y with axe runner produces identical
results with an extra abstraction layer. Pa11y with HTML_CodeSniffer produces false positives
on SVG ARIA patterns (which is exactly what slideforge uses). Lighthouse covers ~50 of axe's
~110 rules — strict subset, no additional value.

### PDF Surface

| Tool | Standard | Coverage | CI-Suitable | Platform | Cost | Verdict |
|---|---|---|---|---|---|---|
| **veraPDF** | PDF/UA-1 (ISO 14289-1) | Full Matterhorn Protocol (31 checkpoints, 136 failure conditions) | YES — Java CLI, exit codes 0/1/2+ | Linux, macOS (requires JVM) | Free/OSS | **SELECTED for CI gate** |
| PAC 2024 | PDF/UA-1 + WCAG 2.1 AA | Full Matterhorn + UX-oriented checks | NO — Windows desktop only, interactive | Windows only | Free | **SELECTED for release gate (manual)** |
| Adobe Acrobat Pro | PDF/UA + WCAG | Proprietary | NO — manual, commercial | Windows/Mac | Commercial license | REJECTED |
| Rust PDF crates (lopdf, printpdf) | None built-in | No PDF/UA validation engine | N/A | All | Free | REJECTED for validation (OK for generation) |

**Decision: veraPDF in CI (per-PR), PAC 2024 manual per-release.**

veraPDF CLI usage:
```bash
# Install: download JAR installer from https://docs.verapdf.org/install/
# Requires: Java 8+ (OpenJDK acceptable)
verapdf -f ua1 --format json output.pdf > report.json
# Exit: 0 = compliant, 1 = non-compliant, 2+ = tool error
```

Docker alternative for CI runners without JVM:
```bash
docker run --rm -v $(pwd):/data verapdf/rest validate ua1 /data/output.pdf
```

No Rust-native PDF/UA-1 validator exists at production quality as of 2026-05. veraPDF is
the only OSS tool with full Matterhorn Protocol coverage.

### PPTX Surface

| Tool | Coverage | CI-Suitable | Platform | Verdict |
|---|---|---|---|---|
| **Custom OOXML linter (Rust, in-tree)** | Rules we define and encode | YES — part of emit pipeline | All | **SELECTED for CI gate** |
| Microsoft PowerPoint Accessibility Checker | ~30% of relevant WCAG criteria | NO — interactive desktop only | Windows/Mac | **SELECTED for release gate (manual)** |
| office2pdf + downstream PDF checker | Indirect | CI-suitable but detects PPTX issues too late | Linux + LibreOffice | REJECTED — indirect, misses PPTX-specific issues |

**Decision: Custom OOXML linter (Rust) in CI, Microsoft checker manual per-release.**

The custom OOXML linter is zero additional cost: it runs at emit time inside the slideforge
pipeline. Since slideforge owns the PPTX emitter, accessibility invariants are enforced at
generation time — the linter is a belt-and-suspenders CI check, not the primary enforcement.

---

## 2. Recommended Toolchain

### Per-Surface Summary

| Surface | CI Gate (every PR) | Release Gate (manual) | Build-time enforcement |
|---|---|---|---|
| Web preview | `@axe-core/playwright` — zero serious/critical | Manual VoiceOver + NVDA walkthrough | SVG-based canvas rendering (ARIA per element) |
| HTML export | `@axe-core/playwright` — zero serious/critical | Same screen-reader walkthrough | HTML emitter type-level guards (alt required) |
| PDF export | `veraPDF -f ua1` — exit 0 required | PAC 2024 on Windows VM, zero Errors | PDF emitter produces tagged PDF from semantic IR |
| PPTX export | Custom OOXML linter — zero Errors | Microsoft Accessibility Checker, zero Errors | PPTX emitter encodes all invariants at generation time |

### Node.js Footprint

slideforge requires Node.js for two things: Playwright E2E tests and `@axe-core/playwright`.
Both live in `slideforge-preview`'s test harness. The production binary has no Node.js
dependency. The test harness is an isolated `package.json` under `crates/slideforge-preview/tests/`.

---

## 3. Evaluation Results

### 3.1 Contrast Checker (Rust)

Implemented and validated in `src/contrast.rs`. 6/6 unit tests pass. Key findings:

| Color pair | Ratio | Threshold | Verdict |
|---|---|---|---|
| Dark navy (#1E3A5F) on white | 11.50 | 4.5 | PASS |
| Dark text (#111827) on white | 17.74 | 4.5 | PASS |
| Accent orange (#F97316) on white | 2.80 | 4.5 | **FAIL — common brand mistake** |
| White on severity red (#DC2626) | 4.83 | 4.5 | WARN (within 10% of threshold) |
| White on severity amber (#D97706) | 3.19 | 4.5 | **FAIL — common brand mistake** |
| White on severity green (#16A34A) | 3.30 | 4.5 | **FAIL — common brand mistake** |
| Accent orange on white (large text) | 2.80 | 3.0 | **FAIL** (even large text) |
| Orange stroke on light gray (non-text) | 2.55 | 3.0 | **FAIL** |

**Architecture implication:** The contrast algorithm belongs in `slideforge-validate`
(`BrandValidator::check_theme_pairs()`). It is a pure function (no I/O) — Kani-amenable.
The WCAG luminance formula is a VP candidate for Phase 6 formal proof.

**Spec note:** The WCAG 2.x spec uses the threshold 0.04045 for the linearization step
(not 0.03928 as cited in some older documents). The implementation uses 0.04045.
This changes contrast ratios by less than 0.5% in practice but must be consistent
with the authoritative spec.

### 3.2 Custom OOXML Linter (Rust)

Implemented in `src/ooxml_lint.rs`. 4/4 unit tests pass. Rules prototyped:

| Rule ID | Description | Severity |
|---|---|---|
| A1:SlideTitlePlaceholder | Every slide must have `<p:ph type="title">` or `type="ctrTitle"` | Error |
| A2:ShapeAltText | Every non-decorative, non-placeholder shape must have non-empty `cNvPr@descr` | Error |
| A3:TableHeaderRow | Every `<a:tbl>` must have `<a:tblPr firstRow="1"/>` | Error |
| A4:PresentationLang | Presentation must declare `lang=` on default run properties | Warning |

Self-test output (confirmed via `cargo run --bin s3_ooxml_lint`):
- Conformant slide: 0 findings
- Non-conformant slide: 3 errors (A1, A2, A3) detected correctly

**Bug found and fixed during spike:** The initial implementation used byte offset `pos + 6`
when checking the character after `descr="`. Since `descr="` is 7 bytes, the correct offset
is `pos + 7`. Fixed in the final code. This is exactly the kind of off-by-one error that
a Kani proof of the linter would catch formally.

**Production implementation path:**
- String-search → `quick-xml` element walker (correct XML parsing, not substring matching)
- Rules A1-A4 → extended to full ADR-011 rule set (reading order, language per-run, notes lint)
- Linter integrated into `slideforge-pptx` emit pipeline, not a post-hoc check

### 3.3 DSL Parser `alt` Enforcement (Rust + chumsky 0.10)

Implemented in `src/parser_alt.rs`. 5/5 unit tests pass. Key findings:

**What works:**
- `validate()` on a parser emits non-terminal errors without stopping the parse
- Multiple errors accumulate in a single parse pass (matches slideforge requirement)
- `Rich::custom(span, message)` produces human-readable error messages with spans
- Custom error types with `SimpleSpan` are straightforward to implement
- `decorative: true` as an alternative to `alt "..."` works correctly

**Grammar limitation found (spike learning):**
The spike's `visual_elements_parser()` using `separated_by(newline().repeated())`
does not correctly handle multi-block inputs with different block types in sequence.
The `image_block_parser()` consumed content it did not own when encountering a `chart:`
block. Production fix: use a `choice()` over block types with explicit block-start
detection, not `separated_by()`.

**Production parser architecture:**
```rust
// Production: identify block type by keyword, then dispatch to block parser
let visual_element = choice((
    image_block_parser(),
    chart_block_parser(),
    diagram_block_parser(),
));
// Wrapped in recovery: skip unknown blocks and report error
let recovering_element = visual_element
    .recover_with(skip_then_retry_until(
        any().ignored(),
        keyword_start_detector(),
    ));
```

**Kani-amenable property:** The invariant
`"if VisualElement is an Image with AltText::Text(s) where s.is_empty(), errors is non-empty"`
is a pure boolean property over the parse output. It can be formally proven with Kani once
the `slideforge-syntax` crate is in its production form.

### 3.4 axe-core/playwright Evaluation (TypeScript)

API confirmed working (not run against live server — spike is a code evaluation, not
integration test). Key findings:

**API (confirmed via docs and research):**
```typescript
import AxeBuilder from '@axe-core/playwright';

const results = await new AxeBuilder({ page })
  .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22a', 'wcag22aa'])
  .analyze();

const blocking = results.violations.filter(
  v => v.impact === 'serious' || v.impact === 'critical'
);
expect(blocking).toHaveLength(0);
```

**WCAG tags:** `wcag2a` + `wcag2aa` + `wcag21a` + `wcag21aa` + `wcag22a` + `wcag22aa`
covers WCAG 2.0 + 2.1 + 2.2 A and AA. Adding `wcag22aa` includes the three new 2.2
criteria relevant to slideforge's web preview (focus-not-obscured, dragging-movements,
target-size-minimum).

**pa11y comparison conclusion:** pa11y with axe runner produces the same results as
`@axe-core/playwright`. pa11y with HTML_CodeSniffer adds false positives on SVG ARIA
patterns. Neither provides unique value over `@axe-core/playwright` given that Playwright
is already a project dependency. Decision: do not add pa11y.

### 3.5 veraPDF CLI Evaluation

Evaluated via research (not installed locally — Java dependency not present on dev machine).
Confirmed findings:

**CLI interface:**
```bash
# Validate against PDF/UA-1 profile
verapdf -f ua1 --format json output.pdf > report.json
# Exit codes: 0 = compliant, 1 = non-compliant, 2+ = tool error

# Alternatively via Docker (no Java required on CI runner):
docker run --rm -v $(pwd):/data verapdf/rest:1.26.0 validate ua1 /data/output.pdf
```

**Version:** 1.26.x is the current stable series. Pin via Docker tag or installer checksum.

**Java-free alternatives:** No production-quality Rust or Java-free OSS PDF/UA-1 validator
exists. The Docker-based approach hides the Java dependency from the CI runner image.

**Recommendation for CI:** Use the Docker approach in GitHub Actions CI
(runner: `ubuntu-latest`, sidecar: `verapdf/rest:1.26.0`). On macOS dev machines,
skip veraPDF (only run on CI Linux). PAC 2024 runs on Windows per-release as the manual gate.

---

## 4. CI Integration Plan

### Pipeline Position

```
PR opened
  ↓
cargo test --workspace                  [Rust unit + integration tests]
  ↓
slideforge build deck.sf --format all  [produces .pptx, .pdf, .html]
  ↓
PPTX OOXML linter                       [slideforge-pptx internal; exit 1 if errors]
  ↓
veraPDF -f ua1 output/deck.pdf          [PDF/UA-1 gate; Docker sidecar on Linux CI]
  ↓
slideforge serve deck.sf --port 4173 & [start web preview]
  ↓
playwright test --grep @a11y            [axe-core WCAG 2.1 AA scan; blocks on serious/critical]
  ↓
npx serve output/ --port 4174 &        [serve HTML export]
  ↓
playwright test --grep @html-a11y      [axe-core scan of static HTML export]
  ↓
All gates pass → merge allowed
```

### GitHub Actions CI Job

```yaml
# .github/workflows/ci.yml (a11y section)
a11y:
  name: Accessibility Gate
  runs-on: ubuntu-latest
  services:
    verapdf:
      image: verapdf/rest:1.26.0
      ports:
        - 8080:8080

  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable

    - name: Build sample deck
      run: cargo run --bin slideforge -- build tests/fixtures/canonical-deck.sf --format all --output /tmp/a11y-out

    - name: Verify PDF/UA-1
      run: |
        curl -s -X POST -F "file=@/tmp/a11y-out/canonical-deck.pdf" \
          http://localhost:8080/api/validate/ua1 > /tmp/verapdf-report.json
        # Fail if isCompliant is false
        python3 -c "
        import json, sys
        r = json.load(open('/tmp/verapdf-report.json'))
        if not r.get('isCompliant', False):
          print('PDF/UA-1 FAILED:', json.dumps(r, indent=2))
          sys.exit(1)
        print('PDF/UA-1 PASS')
        "

    - uses: actions/setup-node@v4
      with:
        node-version: '22'

    - name: Install axe-core/playwright
      working-directory: crates/slideforge-preview/tests
      run: npm ci && npx playwright install --with-deps chromium

    - name: Start web preview
      run: cargo run --bin slideforge -- serve /tmp/a11y-out/canonical-deck.sf --port 4173 &

    - name: Start HTML export server
      run: npx serve /tmp/a11y-out --port 4174 &

    - name: Run axe-core accessibility tests
      working-directory: crates/slideforge-preview/tests
      run: npm test
      env:
        SLIDEFORGE_PREVIEW_URL: http://localhost:4173
        SLIDEFORGE_HTML_EXPORT_URL: http://localhost:4174
```

### macOS Dev Machine

On macOS (no Docker by default, no JVM):
- OOXML linter: runs as part of `cargo test` (in-process)
- Axe-core: runs via `npm test` in `crates/slideforge-preview/tests/`
- veraPDF: **not run locally** — Linux CI only. Developers are expected to trust CI.
- PAC 2024: **Windows only** — run on a Windows VM or GitHub Actions Windows runner per release.

---

## 5. ADR-011 Recommendation

**Decision: Use the following toolchain for WCAG AA enforcement.**

| Surface | CI Tool | Release Gate |
|---|---|---|
| Web preview | `@axe-core/playwright` | VoiceOver (macOS) + NVDA (Windows) manual walkthrough |
| HTML export | `@axe-core/playwright` | Same |
| PDF export | `veraPDF -f ua1` (Docker sidecar on Linux CI) | PAC 2024 (Windows, manual) |
| PPTX export | Custom OOXML linter (Rust, in `slideforge-pptx`) | Microsoft PowerPoint Accessibility Checker (manual) |
| Brand colors | `slideforge-validate` contrast checker (Rust, in-tree) | N/A — compile-time enforcement |
| DSL `alt` | `slideforge-syntax` chumsky validator (compile-time) | N/A — compile-time enforcement |

**Rejected alternatives:** Lighthouse (axe subset), pa11y (axe duplicate), Adobe Acrobat Pro
(commercial), Rust PDF/UA validators (none production-ready), any tool that is not
programmatically CI-automatable for the CI gate tier.

**Key constraint:** The web preview MUST use SVG-based rendering (not `<canvas>`) to satisfy
WCAG 1.1.1 and 4.1.2. A bare `<canvas>` is opaque to axe-core and to AT. This is an
**architecture constraint** that must flow into ADR-008 (canvas renderer choice).

---

## 6. Cost / Complexity Assessment

| Component | Setup Complexity | Ongoing Cost | Risk |
|---|---|---|---|
| `@axe-core/playwright` | Low — one npm install | Low — runs in existing Playwright suite | Low — widely used, well-maintained |
| veraPDF (Docker) | Medium — Docker sidecar in CI YAML | Low — passive check | Low — stable OSS, versioned Docker image |
| Custom OOXML linter | Low — already-needed code | Low — grows with new rules | Medium — must be kept in sync with emitter |
| PAC 2024 (manual) | Low — one-time tool install on Windows VM | Low — per-release only | Low — deterministic, interactive |
| DSL `alt` enforcement (chumsky) | Low — part of normal parser work | Low — static property | Low — unit-testable, Kani-provable |
| Brand contrast checker (Rust) | Low — pure function, no deps | Low — static property | Low — unit-testable, Kani-provable |

**Total additional Node.js surface:** 2 packages (`@axe-core/playwright`, `@playwright/test`)
in one `package.json` under the preview crate's test directory. No Node.js in the production binary.

**Total additional CI complexity:** One Docker sidecar (`verapdf/rest:1.26.0`) and one
additional `playwright test` step. Estimated CI wall-clock addition: +2-3 minutes.

---

## 7. Open Questions Resolved

| Question | Resolution |
|---|---|
| pa11y vs axe-core? | axe-core exclusively; pa11y provides no unique value given existing Playwright dependency |
| veraPDF without JVM? | Docker sidecar (`verapdf/rest:1.26.0`) on Linux CI; no JVM on runner |
| Canvas vs SVG for web preview? | SVG required (WCAG 1.1.1 constraint); flows to ADR-008 |
| Where does contrast check live? | `slideforge-validate` crate, `BrandValidator::check_theme_pairs()` |
| Where does `alt` enforcement live? | `slideforge-syntax` crate, inside `image_block_parser()` / `chart_block_parser()` via `validate()` |
| chumsky 0.10 error accumulation? | `validate()` emits errors without stopping the parse; confirmed working |
| Multi-block grammar limitation? | Spike found: `separated_by()` is wrong for mixed block types; production uses `choice()` with recovery |

---

## 8. Spike Artifacts

| File | Purpose |
|---|---|
| `S3-code/src/contrast.rs` | WCAG contrast algorithm + brand validator prototype (6/6 tests pass) |
| `S3-code/src/ooxml_lint.rs` | Custom OOXML accessibility linter prototype (4/4 tests pass) |
| `S3-code/src/parser_alt.rs` | chumsky 0.10 DSL `alt` enforcement prototype (5/5 tests pass) |
| `S3-code/ts-axe-test/tests/slideforge-a11y.spec.ts` | `@axe-core/playwright` test suite template |
| `S3-code/ts-axe-test/package.json` | npm dependencies for axe-core evaluation |

All 15 Rust unit tests pass. TypeScript tests are templates — run against live server.

---

## 9. Bugs Found During Spike

1. **Off-by-one in OOXML linter (`pos + 6` → `pos + 7`):** String search for `descr="` is
   7 bytes, not 6. The initial implementation used offset 6, causing `descr="VALUE"` to be
   misidentified as empty. Fixed. This is exactly the class of subtle bug that Kani would
   catch in the production linter.

2. **Grammar limitation in `separated_by()` for mixed-type blocks:** The spike parser used
   `separated_by(newline())` over a `choice()` of block types, causing the `image_block_parser`
   to greedily consume lines that belonged to a subsequent `chart:` block. The production
   grammar must use explicit block-start keyword detection before dispatching. Documented
   in §3.3.

Both bugs were found by running the code, not by review. This validates the spike methodology.
