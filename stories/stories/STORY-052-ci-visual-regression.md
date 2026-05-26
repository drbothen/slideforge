---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-052
title: "CI: Visual Regression (LibreOffice + SSIM/PSNR)"
epic: EPIC-19
wave: 1
points: 5
priority: P0
tdd_mode: facade
status: draft
producer: story-writer
phase: 2
target_module: "(infrastructure — .github/workflows/ + scripts/ + tests/fixtures/)"
subsystems: []
behavioral_contracts:
  - BC-4.01.002
# BC-4.01.002: PPTX visual fidelity — rendered output matches reference within
# SSIM ≥ 0.99 and PSNR ≥ 35dB (visual parity contract).
nfr_refs:
  - NFR-007  # SSIM ≥ 0.99 per slide (LibreOffice rendering vs reference)
  - NFR-008  # PSNR ≥ 35dB per slide
  - NFR-009  # slide count exact match
  - NFR-010  # cross-renderer divergence (informational, LibreOffice vs PowerPoint)
  - NFR-011  # shape position drift ≤ 4pt (≤ 17px at 300 DPI)
verification_properties: []
depends_on:
  - STORY-051  # CI base workflow must exist before visual regression job is valid
blocks:
  - STORY-037  # PPTX core serialization activates this gate
  - STORY-040  # PPTX notes/sections activates slide-level fixture coverage
estimated_days: 2
---

# STORY-052: CI: Visual Regression (LibreOffice + SSIM/PSNR)

## Summary

Implement the visual regression CI pipeline that validates PPTX output fidelity
against committed reference PNGs. The pipeline uses LibreOffice Still headless
to convert PPTX to PNG at 300 DPI, then computes SSIM and PSNR per slide against
reference images using a Python script backed by scikit-image. Any slide below
SSIM ≥ 0.99 or PSNR ≥ 35dB fails the gate.

This story establishes the workflow, the `scripts/visual-diff.py` utility, the
fixture management protocol, and the divergence-log mechanism for informational
LibreOffice-vs-PowerPoint delta tracking. The gate is skipped until Phase 3 PPTX
fixtures exist (guarded by a file-presence check in the workflow), but the
workflow job is defined from Wave 1 so branch protection can reference it
immediately.

## Narrative

As a devops engineer, I want a CI job that renders every slideforge PPTX fixture
through LibreOffice headless and computes SSIM/PSNR metrics per slide against
committed reference PNGs, so that visual regressions in the PPTX exporter are
caught automatically before they reach `develop`.

## Behavioral Contracts

| BC-ID | Clause | How Story Implements It |
|-------|--------|------------------------|
| BC-4.01.002 | PPTX output renders correctly in LibreOffice with SSIM ≥ 0.99 per slide | `visual-regression` job in CI; `scripts/visual-diff.py` SSIM gate |

## Acceptance Criteria

### AC-001: Visual Regression Job in ci.yml (traces to BC-4.01.002 postcondition 1)

The `visual-regression` job exists in `.github/workflows/ci.yml` and is listed in
the `needs:` array of the `all-checks-pass` synthetic job. It runs on
`ubuntu-latest` with a timeout of 30 minutes.

### AC-002: Fixture-Presence Guard (traces to BC-4.01.002 precondition 1)

The `visual-regression` job begins with a `Check for PPTX test fixture` step that
sets `has_fixtures=true/false` in `$GITHUB_OUTPUT`. All subsequent steps use
`if: steps.fixture-check.outputs.has_fixtures == 'true'`. When no fixtures are
found, the job exits with status `success` (not `failure`) and prints a clear
message: "Visual regression gate skipped — activates when Phase 3 PPTX exporter
ships test fixtures."

This guard is removed (or switched to an unconditional fail) when STORY-037
merges and ships a `tests/fixtures/test-fixture.pptx`.

### AC-003: LibreOffice Installation Step (traces to BC-4.01.002 postcondition 1)

When `has_fixtures == 'true'`, the workflow installs via `apt-get`:
`libreoffice-still`, `imagemagick`, `python3-pip`. Then installs Python packages:
`scikit-image`, `numpy`. The LibreOffice version installed is Still channel
(as of authorship: 25.8.x; exact version determined at runner image time).
The step is idempotent and uses `apt-get install -y` with `update -y` prefix.

### AC-004: PPTX Generation Step (traces to BC-4.01.002 precondition 2)

When `has_fixtures == 'true'`, the workflow runs:
```
cargo test --workspace --all-features -- --include-ignored generate_pptx_fixtures
```
This runs the `generate_pptx_fixtures` integration test (tagged `#[ignore]` in
normal test runs; activated by `--include-ignored`). The test must produce at
least `tests/fixtures/test-fixture.pptx`. The workflow step fails if the fixture
file does not exist after this step.

### AC-005: PPTX → PDF → PNG Conversion Pipeline (traces to BC-4.01.002 postcondition 1)

When `has_fixtures == 'true'`, the workflow runs a bash script that:
1. Creates `/tmp/visual-regression/png/` directory
2. For each `tests/fixtures/*.pptx`, runs:
   `libreoffice --headless --convert-to pdf "$pptx" --outdir /tmp/visual-regression/`
3. For each resulting PDF, runs:
   `convert -density 300 "$pdf" /tmp/visual-regression/png/"$(basename "$pdf" .pdf)"-%03d.png`
   (ImageMagick `convert` at 300 DPI, zero-padded 3-digit slide index)

### AC-006: Visual Diff Script (traces to BC-4.01.002 postcondition 1, invariant 1)

The file `scripts/visual-diff.py` exists and implements the following interface:

```
python3 scripts/visual-diff.py \
  --generated /tmp/visual-regression/png \
  --reference tests/fixtures/reference-pngs \
  --ssim-threshold 0.99 \
  --psnr-threshold 35.0 \
  --fail-on-violation
```

Behavior:
- Pairs each generated PNG with the corresponding reference PNG by filename
- Computes SSIM using `skimage.metrics.structural_similarity`
- Computes PSNR using `skimage.metrics.peak_signal_noise_ratio`
- Prints a per-slide table: `slide | SSIM | PSNR | PASS/FAIL`
- Exits with code 0 if all slides pass both thresholds; exits with code 1 if any
  slide fails either threshold (when `--fail-on-violation` is set)
- Writes a `divergence-log.md` summary in `/tmp/visual-regression/` with
  informational cross-renderer delta data (NFR-010)

### AC-007: SSIM and PSNR Thresholds (traces to BC-4.01.002 postcondition 2)

The gate uses exactly:
- SSIM threshold: ≥ 0.99 (per NFR-007)
- PSNR threshold: ≥ 35.0 dB (per NFR-008)

Both thresholds must pass per slide. A slide failing either metric causes the CI
job to exit non-zero. Threshold values are not configurable at runtime in CI (they
are hardcoded in the workflow step that invokes `visual-diff.py` via
`--ssim-threshold 0.99 --psnr-threshold 35.0`).

### AC-008: Reference PNG Management Protocol

`tests/fixtures/reference-pngs/` is a git-committed directory containing one PNG
per slide for each fixture PPTX. Reference PNG updates require:
1. A PR that includes only the PNG updates (no code changes)
2. A commit message with prefix `chore(fixtures): update reference PNGs — <reason>`
3. Human review and approval (not auto-merged by any CI rule)

The `scripts/visual-diff.py` script must detect missing reference PNGs (no
corresponding reference for a generated slide) and fail with a clear message:
"Missing reference PNG for <slide>. Run generate_reference_pngs.sh to create it."

### AC-009: Slide Count Validation (traces to BC-4.01.002 invariant 1)

After PNG generation, the workflow validates that the count of generated PNGs
equals the expected slide count (NFR-009). The count is stored in
`tests/fixtures/expected-slide-counts.json` as a map from fixture name to expected
count. If counts mismatch, the job fails before running SSIM/PSNR.

### AC-010: Divergence Log (informational — NFR-010)

`scripts/visual-diff.py` writes `/tmp/visual-regression/divergence-log.md` with:
- Date/time of the run
- Git commit SHA
- Per-slide SSIM and PSNR values
- A notation for slides where SSIM < 0.90 (potential PowerPoint divergence risk)

This file is uploaded as a CI artifact on every run (even passing runs) using
`actions/upload-artifact` with `retention-days: 7`. It is informational and does
not affect pass/fail status.

### AC-011: Position Drift Detection (traces to BC-4.01.002 invariant 2)

`scripts/visual-diff.py` flags slides where the SSIM is ≥ 0.99 but the computed
centroid drift of key shapes exceeds 17px (≡ 4pt at 300 DPI per NFR-011). This
flag is written to `divergence-log.md` as a WARNING and logged to stdout. It does
not cause a CI failure — it is informational for the PPTX team to investigate.
Implementation: compute per-channel histogram shift and structural component
centroid via `skimage.measure`.

## Tasks

1. Audit the `visual-regression` job in `.github/workflows/ci.yml` against ACs 001-005
2. Create `scripts/visual-diff.py` implementing the interface in AC-006
3. Implement SSIM + PSNR per-slide computation in `scripts/visual-diff.py` (AC-007)
4. Create `tests/fixtures/` directory and `expected-slide-counts.json` stub (AC-009)
5. Create `tests/fixtures/reference-pngs/` directory with `.gitkeep` (AC-008)
6. Implement slide count validation in `scripts/visual-diff.py` (AC-009)
7. Implement divergence log writing in `scripts/visual-diff.py` (AC-010)
8. Implement position drift detection flag in `scripts/visual-diff.py` (AC-011)
9. Add `actions/upload-artifact` step for divergence log to `ci.yml` (AC-010)
10. Document the reference PNG update protocol in `scripts/README.md` (AC-008)
11. Verify the job is listed in `all-checks-pass` `needs:` array (AC-001)

## File List

| File | Action | Purpose |
|------|--------|---------|
| `.github/workflows/ci.yml` | Update | Audit/complete `visual-regression` job |
| `scripts/visual-diff.py` | Create | SSIM/PSNR per-slide comparison tool |
| `scripts/README.md` | Create | Reference PNG update protocol documentation |
| `tests/fixtures/reference-pngs/.gitkeep` | Create | Placeholder until STORY-037 ships PNGs |
| `tests/fixtures/expected-slide-counts.json` | Create | Map of fixture name → expected slide count |

## Test Strategy

Facade mode validation — quality gate is mutation testing at wave gate, not Red Gate:

1. **Unit test `scripts/visual-diff.py`:** Run the script against a pair of
   identical PNGs; verify SSIM = 1.0, PSNR = infinity (or large value), exit 0.
2. **Unit test failure case:** Run the script against a generated PNG and a
   deliberately different reference PNG; verify SSIM < 0.99 triggers exit 1.
3. **Unit test missing reference:** Remove a reference PNG and confirm the script
   fails with the exact message specified in AC-008.
4. **CI live run:** Push a PR with only the `scripts/` and `tests/fixtures/` changes.
   The `visual-regression` job must log "Visual regression gate skipped" and exit
   with `success` (fixture check returns `has_fixtures=false`).
5. **Wave gate mutation test:** `cargo-mutants` on the PPTX exporter code in Wave 4
   will confirm that visual regression failures are detected for mutations that affect
   slide geometry.

## Previous Story Intelligence

N/A — first story in EPIC-19. This story's fixture management protocol and
`scripts/visual-diff.py` will be referenced by STORY-037 (PPTX core serialization),
which must ship the `generate_pptx_fixtures` test and populate `expected-slide-counts.json`.

## Architecture Compliance Rules

1. **No Chrome/headless dependency.** PDF export (STORY-043) uses pdf-writer + krilla,
   not Chrome. Visual regression uses LibreOffice only. Do not introduce Playwright
   or Puppeteer into the visual regression pipeline.
2. **300 DPI is the canonical rendering density.** Lower DPI underestimates fidelity
   issues; higher DPI is unnecessary for the 0.99 SSIM threshold. Do not change.
3. **scikit-image is the SSIM/PSNR library.** The S6 spike validated scikit-image
   metrics against the visual parity contract thresholds. Do not substitute with
   pillow-based metrics (different formula).
4. **Reference PNGs are committed to git, not stored in an artifact cache.** Cached
   references can drift silently. git-committed references are audited on every PR.
5. **`generate_pptx_fixtures` test is `#[ignore]`d in normal test runs.** Activated
   only by the CI visual regression step. Do not make it a default test; it requires
   a built PPTX exporter.

## Library and Framework Requirements

| Tool | Version / Pinning | Usage |
|------|------------------|-------|
| LibreOffice Still | 25.8.x (ubuntu-latest apt-get) | PPTX → PDF conversion |
| ImageMagick | system package (ubuntu-latest) | PDF → PNG at 300 DPI |
| scikit-image | latest stable via pip3 | SSIM + PSNR computation |
| numpy | latest stable via pip3 | Array operations for scikit-image |
| Python | 3.x (ubuntu-latest system) | Visual diff script runtime |
| `actions/upload-artifact` | SHA `ea165f8d65b6e75b540449e92b4886f43607fa02` (v4) | Divergence log upload |

## File Structure Requirements

```
scripts/
  visual-diff.py          ← SSIM/PSNR per-slide comparison tool (AC-006)
  README.md               ← reference PNG update protocol (AC-008)
tests/
  fixtures/
    reference-pngs/
      .gitkeep            ← placeholder; PNGs committed here by STORY-037 team
    expected-slide-counts.json  ← {"test-fixture": 31, ...} stub
.github/
  workflows/
    ci.yml                ← visual-regression job (AC-001 through AC-010)
```

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~2,200 |
| `.github/workflows/ci.yml` visual-regression job section | ~400 |
| `scripts/visual-diff.py` (new file) | ~600 |
| NFR catalog (NFR-007 through NFR-011) | ~250 |
| BC-4.01.002 (behavioral contract) | ~150 |
| **Total** | **~3,600** |

Within 20% context budget. No splitting required.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | LibreOffice renders PPTX with slightly different font metrics than PowerPoint | SSIM ≥ 0.99 threshold is set to accommodate LibreOffice-specific differences, not PowerPoint differences. Divergence-log captures the delta informally (NFR-010). |
| EC-002 | Generated PNG count ≠ expected count (LibreOffice split or merged slides) | AC-009 slide count check fails CI with clear message before SSIM runs. |
| EC-003 | Reference PNG directory is empty (first time after STORY-037) | Script detects missing references and fails with message per AC-008. Team must run `generate_reference_pngs.sh` and commit the results. |
| EC-004 | `--include-ignored` flag runs too many ignored tests | The `generate_pptx_fixtures` test should use a unique `#[ignore]` tag that can be targeted via `--test-threads=1 generate_pptx_fixtures` rather than the blanket `--include-ignored`. Update AC-004 if needed. |
| EC-005 | ImageMagick `convert` conflicts with system `convert` binary on some runners | Use fully qualified `magick convert` (ImageMagick 7) or install `imagemagick` specifically. Add a version check step. |
| EC-006 | scikit-image SSIM returns NaN for uniform slides (all one color) | Both NaN and Inf SSIM values should be treated as PASS (uniform vs uniform = identical). Add special case in `visual-diff.py`. |
| EC-007 | PPTX fixture has 31 slides but one produces a blank PDF page | Blank-page detection: if PNG file size < 1KB, flag as "suspicious slide" in divergence log and continue. Do not silently pass. |

## Dependencies

- **Depends on:** STORY-051 (CI base workflow must exist)
- **Blocks:**
  - STORY-037 (PPTX core serialization — must ship `generate_pptx_fixtures` test
    and initial reference PNGs to activate this gate)
  - STORY-040 (PPTX notes/sections — visual regression must pass for all slide types
    before STORY-040 can be considered complete)

## Implementation Notes

The `visual-regression` job in the existing `ci.yml` already implements the basic
structure (fixture check, LibreOffice install, PNG conversion, visual diff). This
story's role is to:

1. Fill the `scripts/visual-diff.py` script (currently referenced but not yet
   created in the repo)
2. Create the `tests/fixtures/` directory structure
3. Create the `expected-slide-counts.json` stub with `{"test-fixture": 31}` as the
   expected final value (31 slide types, one per fixture slide)
4. Document the reference PNG update protocol

When STORY-037 ships, the STORY-037 team must:
- Implement `generate_pptx_fixtures` as an `#[ignore]`d integration test
- Add the generated PNGs to `tests/fixtures/reference-pngs/` and commit them
- Update `expected-slide-counts.json` with the actual count

**Forbidden dependencies:**
- Do not use `Pillow` for SSIM/PSNR — use `scikit-image` exclusively (S6 spike
  validation was done with scikit-image metrics)
- Do not use `playwright` or `puppeteer` in the visual regression pipeline — LibreOffice
  is the only headless renderer permitted here
- Do not introduce `f64` into Rust code from this story — visual regression is
  Python-side only
