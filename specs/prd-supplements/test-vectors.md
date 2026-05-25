---
document_type: prd-supplement
supplement_type: test-vectors
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
traces_to: .factory/specs/prd.md
primary_consumers: [test-writer, holdout-evaluator]
---

# Test Vectors — slideforge v1.0

> Canonical test inputs and expected outputs for key behavioral contracts.
> Used for regression testing and holdout evaluation.
> Primary consumers: test-writer, holdout-evaluator.

---

## TV-1: DSL Source Parsing (BC-1.01.001)

### TV-1.1: Minimal valid deck

**Input:**
```sf
slideforge_version "1"
lang "en-US"
brand "brand.toml"

slide title:
  title "Hello World"
  subtitle "A minimal deck"
  notes "Welcome to the presentation"
```

**Expected:** AST with 1 slide node, type=title, fields={title, subtitle, notes}.
No diagnostics. Exit 0.

---

### TV-1.2: Indentation error accumulation

**Input:**
```sf
slideforge_version "1"
lang "en-US"

slide content:
  title "Slide 1"
   bullets:          <- 3 spaces (wrong)
    - "Item A"
slide content:
  title "Slide 2"
    bullets:         <- 4 spaces (wrong)
    - "Item B"
```

**Expected:** 2 parse errors accumulated (E-PAR-001 for each bad indent).
Both errors reported. Exit 1. No output.

---

### TV-1.3: Reserved keyword rejection

**Input:**
```sf
slideforge_version "1"
lang "en-US"

vars:
  component: "my-component"
```

**Expected:** E-PAR-008: `Variable name 'component' collides with reserved keyword at <file>:<line>:<col>. Choose a different name.`
Exit 1.

---

## TV-2: Variable Interpolation (BC-1.02.001, BC-1.02.002, BC-1.02.003)

### TV-2.1: Basic arithmetic interpolation

**Input:**
```sf
vars:
  count: 42
  rate: 0.17

slide stat_callout:
  title "Summary"
  stat1: "{{ count }}"
  stat2: "{{ count * rate | round(2) }}"
```

**Expected:** stat1="42", stat2="7.14". No diagnostics.

---

### TV-2.2: Undefined variable error with scope

**Input:**
```sf
vars:
  name: "World"

slide content:
  title "Hello {{ greeting }}"   <- greeting not defined
```

**Expected:** E-EVL-001: `Undefined variable '{{ greeting }}' at deck.sf:6:16. Variables in scope: [name]`.
Exit 2.

---

### TV-2.3: No implicit type coercion — "NO" stays string

**Input:**
```sf
vars:
  flag: "NO"

slide content:
  title "Value: {{ flag }}"
```

**Expected:** title="Value: NO". The string "NO" is NOT coerced to boolean false.
No error. Consistent with DI-004.

---

### TV-2.4: ${{ currency }} in string — text mode, not math mode

**Input:**
```sf
vars:
  arr: 1200000

slide stat_callout:
  title "ARR: ${{ arr | currency }}"
```

**Expected:** title="ARR: $1,200,000". The `${{` sequence triggers text-mode
interpolation, NOT math mode. Per DEC-009.

---

## TV-3: Data Binding (BC-1.03.001)

### TV-3.1: JSON file binding

**Input fixture file (`metrics.json`):**
```json
{"q3_revenue": 450000, "yoy_growth": 0.23}
```

**Input deck:**
```sf
@data kpis from "metrics.json"

slide stat_callout:
  title "Q3 Results"
  stat1: "{{ kpis.q3_revenue | currency }}"
  stat2: "{{ (kpis.yoy_growth * 100) | round(1) }}% YoY"
```

**Expected:** stat1="$450,000", stat2="23.0% YoY". No diagnostics.

---

### TV-3.2: Missing field error

**Input:** Same deck above but `metrics.json` = `{"q3_revenue": 450000}` (no `yoy_growth`).

**Expected:** E-DAT-005: `Missing field 'kpis.yoy_growth' in data source 'kpis' at deck.sf:7:12`.
Exit 2. No output (strict mode).

---

## TV-4: Iteration (BC-1.04.001, BC-1.04.002)

### TV-4.1: @for over non-empty list

**Input fixture (`incidents.json`):**
```json
[
  {"id": "INC-001", "severity": "high", "title": "DB outage"},
  {"id": "INC-002", "severity": "medium", "title": "API latency"}
]
```

**Input deck:**
```sf
@data incidents from "incidents.json"

@for inc in incidents:
  slide severity_cards:
    title "{{ inc.id }}: {{ inc.title }}"
    cards:
      - severity: "{{ inc.severity }}"
        label: "Severity {{ inc.severity }}"
        title: "{{ inc.title }}"
        body: "Active incident"
        alt: "{{ inc.severity }} severity incident card"
```

**Expected:** 2 slides produced, one per incident. Slide 1 title="INC-001: DB outage",
Slide 2 title="INC-002: API latency". No diagnostics.

---

### TV-4.2: @for over empty list

**Input fixture (`incidents.json`):** `[]`

**Expected:** 0 slides from the @for block. If deck has other slides outside the @for,
those are still rendered. No error. Consistent with DEC-001.

---

## TV-5: Include and Cycle Detection (BC-1.01.004, BC-1.06.002)

### TV-5.1: Circular include detection

**Files:**
- `a.sf`: `@include "b.sf"`
- `b.sf`: `@include "c.sf"`
- `c.sf`: `@include "a.sf"`

**Entry:** `slideforge build a.sf`

**Expected:** E-PAR-004: `Include cycle detected: a.sf → b.sf → c.sf → a.sf`.
Exit 1. Full cycle path shown.

---

## TV-6: Variant Filtering (BC-1.07.001, BC-1.07.004)

### TV-6.1: Variant with exclude_tags

**Input:**
```sf
variants:
  exec:
    exclude_tags: [technical, appendix]

slide content:
  tags: [technical]
  title "Architecture Details"
  bullets:
    - "Deep technical content"

slide content:
  title "Executive Summary"
  bullets:
    - "High-level results"
```

**Build:** `slideforge build deck.sf --variant exec`

**Expected:** 1 slide (Executive Summary only). Technical slide excluded. No error.

---

### TV-6.2: Undefined variant

**Build:** `slideforge build deck.sf --variant nonexistent`

**Expected:** E-CFG-001: `Variant 'nonexistent' not defined. Defined variants: [exec]`.
Exit 4. Consistent with DEC-020.

---

## TV-7: Accessibility Enforcement (BC-5.01.001, BC-5.01.003)

### TV-7.1: Missing alt text — compile error

**Input:**
```sf
slide content:
  title "Market Analysis"
  image: "chart.png"   <- no alt field
```

**Expected:** E-A11-001: `Missing alt text on image 'chart.png' at deck.sf:3:3.
Add alt "..." or mark decorative: true`.
Exit 2. No output (strict mode).

---

### TV-7.2: Missing label on color-coded element

**Input:**
```sf
slide severity_cards:
  title "Incidents"
  cards:
    - severity: "high"
      title: "DB Outage"
      body: "Database unreachable"
      alt: "High severity incident"
      <- label field absent
```

**Expected:** E-A11-002: `Missing label on color-coded element 'severity_cards' card at deck.sf:4:5`.
Exit 2.

---

### TV-7.3: Decorative image — no error

**Input:**
```sf
slide content:
  title "Analysis"
  image: "background-texture.png"
    decorative: true
```

**Expected:** No accessibility error. Image emits empty alt attribute and PDF Artifact
tag. Consistent with DEC-008 and DI-001.

---

## TV-8: Brand Synthesis (BC-2.01.002, BC-2.01.004)

### TV-8.1: brand.toml with partial color slots

**Input (`brand.toml`):**
```toml
[colors]
acc1 = "3B82F6"
dk1  = "1F2937"
lt1  = "FFFFFF"
# dk2, lt2, acc2-acc6, hlink, folHlink absent
```

**Expected:** Brand synthesizer fills all 12 OOXML slots. Emits 9 E-BRD-003 warnings
(one per inferred slot) listing the derived hex values. Build continues. No exit error.
Consistent with DEC-016 and DI-015.

---

## TV-9: PPTX Structure (BC-4.01.005)

### TV-9.1: Slide and master ID ranges

**Assertion (unit test):** For any generated PPTX:
- All slide IDs ≥ 256 (ECMA-376 spec lower bound)
- All slide master IDs ≥ 2^31 (ECMA-376 spec)
- Total layouts = 31 (11 standard + 20 custom)
- `notesMaster1.xml` present in PPTX ZIP
- `handoutMaster1.xml` present in PPTX ZIP

---

## TV-10: PDF/UA-1 Compliance (BC-4.03.001)

### TV-10.1: Minimal deck passes veraPDF

**Input:** A 3-slide deck (title, content, severity_cards) with:
- `lang "en-US"` declared
- All visual elements have alt text or decorative: true
- All color-coded elements have label

**Expected:** `verapdf --flavour ua1 output.pdf` exits 0 with `isCompliant: true`.
Zero violations.

---

## TV-11: Writing Registers (BC-1.14.001 through BC-1.14.004)

### TV-11.1: notes register — PPTX only

**Input:**
```sf
slide content:
  title "Slide with Notes"
  bullets:
    - "Key point"
  notes "This is the presenter note — only in PPTX speaker notes"
```

**PPTX expected:** Presenter notes XML (`<p:notes>`) contains "This is the presenter note".
**DOCX expected:** Presenter notes text does NOT appear in DOCX body.
**HTML expected:** Notes appear in `<aside>` or `data-notes` attribute (not main content).
Consistent with DI-012 (no bleed across formats).

---

## TV-12: Diagram Rendering (BC-1.12.001)

### TV-12.1: Simple flowchart

**Input:**
```sf
slide diagram:
  title "System Architecture"
  lang: "mermaid"
  alt: "System architecture flowchart showing three components"
  source: |
    flowchart LR
      A[Client] --> B[API Gateway]
      B --> C[Database]
```

**Expected:**
- mermaid-rs-renderer produces SVG without `<foreignObject>`.
- usvg normalization: all dimensions absolute (px), no CSS class refs.
- SVG embedded in PPTX as `/ppt/media/diagram1.svg`.
- `<svg aria-label="System architecture flowchart...">` present.
- Warm render time < 10ms.

---

## TV-13: Error Accumulation (BC-1.15.002)

### TV-13.1: Multiple independent errors in one build

**Input:**
```sf
slideforge_version "1"
lang "en-US"

slide content:
  title "Hello {{ undefined_var }}"   <- E-EVL-001
  image: "photo.png"                  <- E-A11-001 (no alt)

slide content:
  title "{{ another_missing }}"       <- E-EVL-001
```

**Expected:** All 3 errors reported in a single build run:
1. E-EVL-001 for `undefined_var`
2. E-A11-001 for missing alt on photo.png
3. E-EVL-001 for `another_missing`

Build does NOT stop after the first error. All errors accumulated. Exit 2.

---

## TV-14: Real-World Corpus Scenarios

### TV-14.1: Known-Good Corpus — Well-maintained report deck

**Source:** The `.factory/seed/reference/` Python implementation's sample output,
translated to .sf syntax (25 slides, multiple types, brand.toml from the seed template).

**Expected:**
- Build exits 0 with no validation errors.
- PPTX visual regression: SSIM ≥ 0.99 vs reference PNG on all slides.
- PDF passes veraPDF.
- Total build time < 500ms.
- False positive rate: 0 unexpected errors on a correctly-authored deck.

### TV-14.2: Known-Problematic Corpus — Missing accessibility + data errors

**Source:** A 10-slide deck with:
- 3 images without alt text
- 1 color-coded slide without labels
- 1 undefined variable reference
- 1 empty @for collection (should produce no slides, not an error)

**Expected:**
- 3 E-A11-001 errors
- 1 E-A11-002 error
- 1 E-EVL-001 error
- 0 errors for the empty @for collection (DEC-001)
- Total: exactly 5 errors, 0 from the empty @for
- Exit 2 (validation error)
- False negative rate: 0 (all known errors detected)
