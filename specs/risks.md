---
title: Slideforge Risks Register (Brief-Level)
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.0
created: 2026-05-23
status: SEED-EXTRACT
---

# Slideforge Risks Register (Brief-Level)

> Source: §9 (Risks and Mitigations) from PROJECT-SEED.md. Verbatim extraction.
> See also: `.factory/planning/market-intel-2026-05-23.md` §5 (8 additional risks) and
> `.factory/preflight/preflight-2026-05-23.md` (7 preflight risk entries).
> The L2 domain spec will produce the canonical R-NNN risk register with full traceability.

---

## Section 9: Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| `ooxmlsdk` API changes during development | Medium | Low | Pin to a specific minor version; vendor a fork if needed |
| OOXML edge cases break brand template loading | High | Medium | Snapshot-test against real brand template; manual QA in PowerPoint and Keynote |
| DSL syntax churn frustrates early users | Medium | Medium | Mark v0.x as pre-stable; freeze syntax at v1.0; offer `slideforge fmt` to migrate |
| chumsky 0.10 has rough edges (it's a recent rewrite) | Medium | Low | Use chumsky 0.10+ patterns from Tao and other reference projects; fall back to nom if blockers emerge |
| Visual output doesn't match Python tool | High | Low | Snapshot-test rendered XML; manual visual review at each phase |
| Incremental compilation produces stale output | High | Low | Use `comemo` only after Phase 4; test thoroughly; provide `--no-incremental` escape hatch |
| Brand template is a moving target | Medium | Medium | Template is a separate input file; users update independently of slideforge |
| FFI bindings (Python, Node) drift from Rust API | Low | Medium | Generate bindings from a shared spec where possible; CI tests for each binding |
