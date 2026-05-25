---
title: Slideforge Spikes Register
created: 2026-05-23
maintainer: business-analyst
status: ACTIVE
---

# Spikes Register — slideforge

Time-boxed research/investigation tasks that resolve uncertainty before committing to design or implementation. Each spike has a defined output (typically: an ADR input, an NFR validation, or a go/no-go decision).

## Phase 1 Spikes (Spec Crystallization)

| ID | Spike | Severity | Time-box | Owner | Output | Status |
|----|-------|----------|----------|-------|--------|--------|
| S1 | ooxmlsdk PPTX coverage validation | HIGH (blocking) | 2 days | architect | ADR-001 input | OPEN |
| S2 | PDF backend evaluation | HIGH | 2 days | architect | ADR-003 input | RESOLVED 2026-05-24 — see spikes/S2-pdf-backend-evaluation.md; ADR-003 input: pdf-writer+krilla |
| S3 | WCAG AA tooling choice | MEDIUM | 1 day | architect | ADR-011 input | RESOLVED 2026-05-24 — see spikes/S3-wcag-tooling-choice.md; ADR-011 input: axe-core/playwright + veraPDF (Docker) + custom OOXML linter + PAC 2024 (manual) |
| S4 | chumsky 0.10 indentation parser viability | MEDIUM | 2 days | architect | ADR-009 input | OPEN |
| S5 | Brand synthesis layout taxonomy | HIGH | 3 days | architect | ADR-001 input | RESOLVED 2026-05-24 — 31 layouts (11 standard + 20 custom), working .pptx synthesis + extraction prototypes in Rust, see spikes/S5-brand-synthesis-layout-taxonomy.md |
| S6 | Multi-renderer parity baseline + CI infra | HIGH | 3 days | architect + devops | ADR-002 input | RESOLVED 2026-05-24 — see spikes/S6-multi-renderer-parity.md; ADR-002 input: LO-Still 25.8.7 + PPTX→PDF→PNG pipeline + SSIM≥0.99/PSNR≥35dB dual gate + Git storage |

## Phase 2 Spikes (Story Decomposition)

| ID | Spike | Severity | Time-box | Owner | Output | Status |
|----|-------|----------|----------|-------|--------|--------|
| S7 | Canvas renderer for web preview | MEDIUM | 2 days | architect | ADR-008 input | OPEN |
| S8 | @include resolution semantics | LOW | 1 day | architect | ADR-004 input | OPEN |
| S9 | Performance baseline (cold + incremental) | MEDIUM | 2 days | architect | NFR validation | OPEN |

## Phase 3 Spikes (Implementation)

| ID | Spike | Severity | Time-box | Owner | Output | Status |
|----|-------|----------|----------|-------|--------|--------|
| S13 | MSSP incident-brief reference port to .sf | MEDIUM | 3 days | implementer | Flagship launch deck + Phase 4 holdout scenario seed | OPEN |
| S14 | Mermaid diagram rendering engine | HIGH (blocking for `slide diagram:`) | 2 days | architect | ADR-014 input | RESOLVED 2026-05-24 — see spikes/S14-mermaid-diagram-engine.md; ADR-014 input: mermaid-rs-renderer v0.2.2 (Option A) |

## Phase 4 Spikes (Polish)

| ID | Spike | Severity | Time-box | Owner | Output | Status |
|----|-------|----------|----------|-------|--------|--------|
| S10 | comemo incremental compilation integration | LOW | 2 days | architect | Phase-4 design | OPEN |

## Ongoing Spikes

| ID | Spike | Cadence | Owner | Output |
|----|-------|---------|-------|--------|
| S11 | Touying / Typst PPTX competitive watch | Quarterly | business-analyst | Market intel update |
| S12 | Mutation testing kill-rate budget tuning | Per-quarter post v1.0 | implementer + architect | Quality bar refinement |

## How Spikes Flow Through the Pipeline

1. Spikes assigned to Phase 1 MUST be completed before their owning ADR is signed off.
2. Spikes assigned to Phase 2 MUST be completed before stories that depend on them are written.
3. Phase 3 spikes are scheduled within their wave; if a wave depends on spike output, the wave gate blocks until the spike result is committed.
4. Phase 4 spikes can be deferred to actual Phase 4 work.
5. Ongoing spikes are run on a cadence by the competitive-monitoring or analytics skills.

## Updating This Register

When a spike completes:
1. Set status to RESOLVED with the resolution date
2. Cross-reference the ADR or NFR that consumed the spike result
3. Move resolved spikes to the bottom of the table (don't delete — keep audit trail)
4. New spikes discovered during a phase are appended to the appropriate phase table
