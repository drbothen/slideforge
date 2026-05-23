---
title: Slideforge Sample DSL Snippets
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.0
created: 2026-05-23
status: SEED-EXTRACT
---

# Sample DSL Snippets

> Source: Appendix B (Sample DSL Files) from PROJECT-SEED.md. Verbatim extraction.
> These samples are authoritative DSL examples for the initial implementation.
> Full example files will live in `examples/` in the project source tree.

---

## Appendix B: Sample DSL Files

### Minimal valid input (`minimal.sf`)

```
metadata:
  title "Minimal Example"
  output "minimal.pptx"

slide title:
  color blue
  title "Hello, World"
  subtitle "A minimal slideforge example"

slide end:
  color blue
```

### Realistic content slide

```
slide content:
  title "Five Things Every MSSP Metric System Must Answer"
  bullets:
    - "Can we sell it?"
    - "Can we onboard it?"
    - "Can we detect accurately?"
    - "Can we operate within SLA?"
    - "Can we do all this profitably?"
  takeaway "If our metrics can't answer all five, we're measuring the wrong things."
  talk_track """
    Five questions an MSSP metric system has to answer. Can we sell it.
    Can we onboard it. Can we detect accurately. Can we operate it within
    SLA. And can we do all of that profitably and retain the client.
    Today our metric system answers question four well. The other four
    are partial or implicit. That's the gap.
  """
```

### Complex composite (formula slide)

```
slide formula:
  title "North Star Formula"
  result "Retained Protected Revenue at Target Margin"
  operator "×"
  accent orange
  stack true   # default
  term:
    name "ARR"
    color blue
    definition "Annual recurring revenue from retained MSS clients"
  term:
    name "Gross Margin %"
    color teal
    definition "Profitability of the service after delivery cost"
  term:
    name "Service Health Score"
    color purple
    definition "Composite of SLA, detection, onboarding, response, satisfaction, platform"
  takeaway "Revenue alone isn't health. Margin and delivery health both have to hold."
```
