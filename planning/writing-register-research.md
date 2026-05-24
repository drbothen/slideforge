---
title: Writing Register Research -- Talk Track vs. Report Prose
date: 2026-05-23
analyst: research-agent
status: foundation-research
audience: product-owner + architect + human-reviewer
prompted_by: "User observation -- talk tracks and reports use fundamentally different writing registers"
---

# Writing Register Research

## Executive Summary

Talk tracks and report prose are **linguistically irreconcilable registers** -- they differ in person, tense, tone, sentence structure, citation style, audience assumptions, and rhetorical strategy. Every professional domain studied (consulting, military, incident response, audit) treats them as separate writing acts, not derivations of each other. No mainstream tool successfully auto-converts between registers without quality loss.

**Recommended approach: Option E (Dual-Track with Shared Data) as the v1 default, with Option C (Structured Content Blocks) as the data-heavy escape hatch.** The author writes `notes` (conversational, presenter-focused) and `report` (formal, standalone) as separate fields, both interpolating shared structured data via `{{ }}`. For data-heavy slides (metrics dashboards, finding tables), structured content blocks auto-render both outputs without prose authoring. The `report` field is optional -- omitting it falls back to structured-data-driven templates.

This avoids the "write everything twice" problem by sharing facts through data interpolation while preserving explicit human control over tone, framing, and register. It requires no LLM at build time, produces deterministic output, and works in air-gapped/regulated environments.

---

## Register Analysis

### Linguistic Differences

The following comparison is grounded in cybersecurity incident reporting (the primary slideforge domain), cross-validated against consulting and military conventions.

| Dimension | Speaker Notes / Talk Track | Formal Written Report |
|-----------|---------------------------|----------------------|
| **Person** | 1st person plural ("we detected..."), occasional 2nd person ("as you'll see on slide 4...") | 3rd person institutional ("the security team detected..."), no "you" |
| **Tense** | Present for status ("we're monitoring..."), past for events, future for actions ("we'll rotate credentials by EOD") | Past for events ("the attacker gained access via..."), present for enduring truths, future only in recommendations |
| **Tone** | Conversational, direct ("we had an outage", contractions, idioms) | Formal, neutral, impersonal ("service availability was degraded for 47 minutes") |
| **Sentence structure** | Short, spoken-style, bulleted prompts, fragments: "02:13 alert -- unusual outbound traffic" | Complex subordination: "Following detection of anomalous outbound traffic at 02:13 UTC, the SOC initiated containment measures..." |
| **Hedging** | Everyday qualifiers ("we're pretty confident", "it looks like") | Formal epistemic markers ("available evidence indicates", "cannot be ruled out", "consistent with") |
| **Citation** | Verbal attributions, slide references ("see slide 7") | Formal references ("see Appendix B", "per NIST SP 800-61", "Table 2 summarizes...") |
| **Visual assumption** | Assumes visuals visible: "the red peaks show exfiltration attempts" | Standalone: "Figure 2 (p. 14) shows a 300% increase in outbound traffic from the web tier..." |
| **Rhetorical cues** | Signposting: "let me walk you through...", "[pause]", "any questions?" | Written discourse markers: "Furthermore", "However", "In conclusion" |

Sources: Ethos3 speaker notes guide [1], Graylog IT security incident report best practices [2], NIST SP 800-61r3 [3], SANS incident handler templates [4].

### Concrete Example: Same Incident, Two Registers

**Incident facts:** On May 12, 2026 at 02:13 UTC, the SOC detected anomalous outbound traffic from the public web portal. Within 9 minutes, the IR team isolated the web tier (47-minute portal outage). Root cause: weak admin password, no MFA on legacy account. Impact: ~3,024 failed customer logins, no confirmed data theft.

**Talk track (speaker notes for executive briefing):**

> We detected unusual outbound traffic from our public web portal at about 2:13 this morning. You can see that spike on the left side of slide 3. Within nine minutes, we'd isolated the web tier, which took the customer portal offline for roughly 47 minutes.
>
> Right now, based on our logs and the forensics we've done so far, we don't see any evidence that production data was accessed. The attacker took advantage of a weak admin password on a legacy account that didn't have MFA.
>
> I'll close by asking for your approval on two items: funding a privileged access management solution and extending our log retention from 90 days to one year.

**Report prose (formal incident report executive summary):**

> On 12 May 2026 at approximately 02:13 UTC, the security operations center (SOC) detected anomalous outbound network traffic originating from the public customer web portal. As detailed in Section 3.1, outbound connections to the external IP address 203.0.113.7 increased by approximately 300% over a ten-minute interval.
>
> Within nine minutes of detection, the incident response team isolated the affected web tier, resulting in a total customer portal outage of 47 minutes (02:22-03:09 UTC). Based on log analysis and host-based forensic imaging conducted between 02:30 and 10:00 UTC, there is currently no evidence of unauthorized access to production customer data. The attacker appears to have exploited weak administrative credentials on a legacy account that was not protected by multi-factor authentication (MFA).
>
> This report recommends implementing an enterprise privileged access management (PAM) solution and extending security log retention from 90 days to 365 days to improve future detection and investigation capabilities.

**Key observation:** The facts are identical. The framing, person, tense, hedging, and audience assumptions are completely different. This is not a formatting difference -- it is a **writing register** difference that cannot be resolved by changing fonts or layouts.

### Style Guide Survey

**McKinsey / Pyramid Principle (Barbara Minto, 1960s):**
- Slides follow the Pyramid Principle: answer first, supporting arguments second, evidence third [5].
- Action titles state conclusions, not topics. "Revenue declined 15% due to pricing pressure" not "Revenue Analysis" [6].
- One message per slide, max ~30 words on-screen [7].
- Written memos follow the same logical structure (SCQA: Situation, Complication, Question, Answer) but use full prose paragraphs, not bullet points [8].
- The conversion from slides to memo is a **separate writing pass** -- consultants draft the deck first (for client meetings), then write the recommendation memo separately (for the record). The memo references exhibit numbers but contains standalone prose [9].

**BCG / Bain:**
- Same SCQA opening structure as McKinsey [7].
- BCG's written deliverables ("blue books") use a structured report format: executive summary, findings organized by workstream, each finding as claim-evidence-recommendation, appendices with detailed data [10].
- The deck and the written report are parallel artifacts maintained by different team members (associates build slides, engagement managers write the report) [9].

**NIST SP 800-61r3 (Incident Response):**
- Mandates formal written documentation: detection timestamps, scope, all actions (who/when), evidence chain-of-custody, root cause determination [3].
- The written report structure: Executive Summary, Incident Overview, Timeline, Technical Analysis (IOCs/TTPs), Impact Analysis, Actions Taken, Evidence Appendix, Lessons Learned [3].
- No guidance on speaker notes or briefing slides -- the standard governs the **written** artifact only.

**SANS Incident Handler Templates:**
- Structured around the incident lifecycle: Preparation, Identification, Containment, Eradication, Recovery, Lessons Learned [4].
- Includes detailed technical fields: hardware identifiers, network addresses, incident type classification [4].
- War-room briefing slides are treated as operational ephemera; the written report is the compliance artifact [11].

**US Military (FM 6-0, ATP 5-0.1, CJCSM 3150.05F):**
- **SITREP (Situation Report):** "essentially a narrative report" (CJCSM 3150.05F) but formatted in structured fields (Own Situation, Operations, Intelligence, Critical Assets). Designed for rapid communication. "Brevity is paramount" [12].
- **AAR (After Action Report):** Uses the Observation-Discussion-Recommendation format per FM 6-0 Chapter 16 [13]. Full formal memorandum structure: cover page, preface signed by commander, table of contents, executive summary, detailed task organization, phase-by-phase analysis, conclusion, POC information, commander's signature [14].
- The SITREP is briefed (often read aloud from structured notes); the AAR is a standalone written document. The register transformation is explicit and doctrinal.

**IEEE/ACM Academic Papers:**
- Conference presentations: informal, audience-engaged, present tense, visual-dependent.
- Papers: third-person, past tense for methods/results, formal citations (numbered or author-date), reproducibility-focused.
- The gap is so well-understood that no one attempts to auto-generate one from the other.

### Industry-Specific Conventions

**MSSP/Cybersecurity Incident Reports:**
- Two-artifact pattern is universal: operational war-room slides for rapid coordination, formal written report for regulators/insurers [11][15].
- War-room slides: attack path diagrams, containment checklists, truncated IOC lists, status updates with color-coded severity [16].
- Written report: full forensic record, raw logs, chain-of-custody forms, MITRE ATT&CK mapping, detailed IOC tables [3][4].
- Practical mapping: Slide title/finding --> Executive Summary; bullet actions --> Containment section with timestamps/actors; attack path diagram --> Technical Analysis with evidence appendix; KPI slide --> Impact Analysis with regulatory consequences [15].

**Consulting Deliverables:**
- Slides: action titles + exhibits (charts/tables), ~30 words per slide, designed for live presentation [6].
- Written memo: SCQA structure, full prose paragraphs, exhibit references, implementation details, risk analysis [8][9].
- The conversion is never automated -- it is a distinct intellectual exercise performed by senior team members.

**Audit Reports (SOC 2, ISO 27001):**
- Extremely formal, passive voice, structured assertions: "Control X was tested and found to be operating effectively" [17].
- No conversational equivalent -- audit reports are written-only artifacts.
- Audit committee briefings use slides with a completely different register (active voice, simplified findings, risk heat maps).

---

## Professional Slide-to-Document Workflows

### Consulting (McKinsey/BCG)

The consulting industry's workflow is the closest analogue to slideforge's challenge:

1. **Analysis phase:** Associates build exhibits (data visualizations, tables) in Excel/PowerPoint.
2. **Deck assembly:** Exhibits are arranged into a Pyramid Principle structure with action titles. Speaker notes contain delivery guidance ("emphasize the 15% decline").
3. **Client presentation:** The deck is presented; speaker notes guide delivery.
4. **Written memo/report:** A separate writing pass converts the deck into a standalone recommendation memo. The memo follows SCQA structure but uses full prose. Exhibit numbers are cross-referenced but the text is self-contained [9].
5. **Appendix:** Detailed data tables, methodology notes, and additional analysis that did not fit on slides are written as standalone appendix sections.

**Key insight:** McKinsey/BCG do NOT auto-convert slides to memos. They treat them as **parallel artifacts sharing the same analytical foundation** but expressed in different registers. The Pyramid Principle governs the logical structure of both, but the linguistic realization is different.

This directly validates slideforge's Option E approach: shared data (the analytical foundation), separate prose (the register-specific expression).

### Incident Response (NIST/SANS)

The IR workflow mirrors consulting but with stronger compliance pressure:

1. **Detection:** Automated alerts trigger the incident record.
2. **War room (slides):** Concise slide pack with incident ID, status, attack path, containment actions, immediate next steps. Audience: operations/executives [16].
3. **Concurrent logging:** All actions logged with actor, system, timestamp in the incident management tool (TheHive, Sentinel, etc.) [18].
4. **Formal report (document):** Written after containment. Expands slide content into NIST/SANS sections. Adds raw forensic evidence, chain-of-custody, regulatory references [3][4].
5. **Lessons learned:** Written as part of the AAR section of the formal report.

**Key insight:** The written report is NOT a transformation of the slides. It is a **separate authoring act** that draws from the same incident data but adds formality, evidence, and regulatory framing. Many teams use incident management tools (TheHive, PagerDuty, incident.io) that export structured data into both slide templates and report templates [18].

This validates the Option C escape hatch: structured incident data can be auto-rendered into both formats for data-heavy sections.

### Military (SITREP to AAR)

The US military has the most formalized version of this workflow:

1. **SITREP (briefed):** Structured narrative using USMTF format. Covers own situation, operations, intelligence. Designed for "rapid communication" with "brevity paramount" [12].
2. **AAR (written):** Formal memorandum using Observation-Discussion-Recommendation format. Includes commander's preface, task organization, phase-by-phase analysis, conclusion, POC [13][14].
3. **The transformation:** Explicitly doctrinal -- FM 6-0 Chapter 16 provides the AAR template. The SITREP and AAR share the same facts but use completely different formats, structures, and registers.

**Key insight:** The military treats the briefed report and the written report as fundamentally different document types. They share data but never pretend to be derivations of each other.

### Enterprise Reporting (Workiva/Board/Domo)

Enterprise reporting tools increasingly use AI to bridge the gap between data and narrative:

- **Workiva AI:** Offers inline content generation in documents, presentations, and spreadsheets. Can "generate a simple narrative from your most complex data" by selecting a table. Supports tone adjustment (formal/casual) via AI companion [19].
- **Narrative BI:** No-code data storytelling platform that "automatically turns metrics into narrative insights" for non-technical users [20].
- **Domo/Tableau/Power BI:** Dashboard tools that support "dashboard to executive report" via scheduled exports, but the narrative layer is typically manual or uses AI-generated summaries.

**Key insight:** Enterprise tools are moving toward AI-generated narratives from structured data, but the quality is suitable for internal dashboards, not for compliance/audit reports or executive presentations. The formalization is a **draft**, not a final artifact.

---

## Tool Landscape

### Quarto Dual-Output Experience

Quarto (built on Pandoc) is the closest existing tool to slideforge's multi-format challenge:

**Mechanism:** One `.qmd` source renders to multiple formats via YAML configuration:
```yaml
format:
  revealjs:
    slide-level: 2
  html:
    toc: true
  pdf:
    documentclass: article
```

**Speaker notes handling:**
- `::: notes` blocks appear as speaker notes in revealjs/PowerPoint/beamer [21].
- In non-slide formats (HTML article, PDF article), `::: notes` blocks are **silently dropped** by default [21][22].
- No built-in mechanism to promote notes to body text in article output -- requires custom Pandoc filter.

**Conditional content:**
- `::: {.content-visible when-format="revealjs"}` shows content only in slides [23].
- `::: {.content-visible when-format="pdf"}` shows content only in article PDF [23].
- Complex cases (e.g., "both revealjs and pptx but not html") become verbose and hard to debug.

**Known pain points (from Quarto GitHub discussions and community):**
1. Same headings must serve as both slide boundaries and section headings -- structural conflict [22].
2. Speaker notes vanish in article output with no clean migration path [22].
3. PDF output is either "slide-PDF" (one page per slide) or "article PDF" (reflowed) -- you often maintain both, defeating single-source.
4. Conditional content complexity grows quickly for non-trivial differences [24].
5. Code block sizing and table rendering differ between formats, requiring format-specific CSS/options.

**Verdict for slideforge:** Quarto demonstrates that the "same source, different format" approach works for **layout** differences (slides vs. sections) but fails for **register** differences. Quarto has no concept of writing register -- notes are either shown or dropped, never transformed.

### Pandoc Slide-to-Article

Pandoc's approach is simpler and more revealing:

- `--slide-level=N` controls which heading level starts slides in slide outputs; has no effect in article outputs [22].
- `::: notes` content is used in revealjs/beamer/pptx, and **generally stripped in non-slide outputs** unless a custom filter is applied [21].
- There is no built-in "beamer-to-article" semantic conversion -- most users render the same Markdown with different output targets [22].

**Key limitation:** Pandoc treats the register difference as a non-problem. Content is either "on the slide" or "in notes." The concept of a formal-prose version of the same content does not exist in Pandoc's model.

### Enterprise BI Report Generation

- **Workiva AI (2025-2026):** AI companion can generate narrative from data tables, adjust tone between formal/casual. But it is an interactive editor feature, not a deterministic build-time transformation [19].
- **Narrative BI:** Generates plain-English insights from metrics automatically. Output quality is suitable for team dashboards, not compliance reports [20].
- **Beautiful.ai, Jasper, ClickUp AI:** Various AI-assisted report generators that transform notes/data into polished reports. All require human review and are non-deterministic [20].

### AI-Assisted Register Transformation

Several tools now offer register/tone transformation:

- **Grammarly:** Tone adjustment (formal/casual/friendly) on existing text [20].
- **Evernote AI Rewrite:** Reword content in "corporate style" with formal/casual/engaging tone options [25].
- **Socialite AI:** Content conversion between styles including formal/casual, with text expansion and summarization [26].
- **ChatGPT/Claude/Gemini:** General-purpose LLMs that can transform conversational text to formal prose with high quality but non-deterministic output.

**Verdict:** AI-assisted register transformation exists and works reasonably well for low-stakes content. For **incident reports** (compliance-critical), **consulting memos** (client-facing), and **audit reports** (legally binding), the quality variability is unacceptable without human review. This rules out Option D (auto-formalization) as the primary build path.

---

## DSL Design Options for slideforge

### Option A: Three Separate Fields

```
slide severity_cards:
  # ... slide content ...
  talk_track """
    Let me walk you through the impact...
  """
  narrative """
    The impact assessment revealed that 5 systems
    were affected...
  """
  detail """
    ## Extended Analysis
    @for system in affected_systems:
    ...
  """
```

**Exporter mapping:**
- `talk_track` --> PPTX speaker notes
- `narrative` --> DOCX body text
- `detail` --> DOCX extended sections / appendix

**Pros:**
- Cleanest architectural separation
- Each register explicitly authored, maximum quality control
- No ambiguity about which text goes where
- Simple exporter logic (pick the right field)

**Cons:**
- Highest authoring burden -- 3x writing per content unit
- Realistic outcome: author fills one field well, others become stale or empty
- No data deduplication -- facts repeated across fields will drift
- Misaligned with primary persona (Python analyst who doesn't write reports today)

**User effort:** High. Every meaningful slide requires 3 writing acts.

**Verdict:** Architecturally clean but practically hostile to the target user.

### Option B: Single Prose with Register Hints

```
slide severity_cards:
  prose:
    register: formal
    """
    The impact assessment revealed...
    """
  detail:
    """
    ## Extended Analysis
    ...
    """
```

**Exporter mapping:**
- PPTX speaker notes: prose as-is, or auto-converted to conversational
- DOCX body: prose as-written

**Pros:**
- Low authoring burden (write once)
- Clear canonical source of truth

**Cons:**
- One register must be "real," the other derived -- derived quality suffers
- If canonical = formal, speaker notes sound stiff and unnatural
- If canonical = conversational, report sounds unprofessional
- Auto-conversion between registers is the hard problem (see Tool Landscape above)
- Requires either rule-based transformation (low quality) or LLM (non-deterministic)

**User effort:** Low for authoring, but hidden cost in reviewing derived output.

**Verdict:** Theoretically elegant, practically broken. The derived register will always be the weak link.

### Option C: Structured Content Blocks

```
slide severity_cards:
  content:
    summary "5 systems impacted, all restored"
    context """
      The incident affected CI/CD pipelines across 5 teams.
      Detection occurred within SLA.
    """
    evidence:
      @for system in affected_systems:
        finding:
          system {{ system.name }}
          impact {{ system.impact_level }}
          metric_before {{ system.avail_before | pct }}
          metric_after {{ system.avail_after | pct }}
    conclusion "All systems restored. No data loss."
```

**Exporter mapping:**
- PPTX: summary --> takeaway, context --> speaker notes (auto-conversational), evidence --> visual cards
- DOCX: summary --> executive callout, context --> intro paragraph, evidence --> detailed table, conclusion --> section close

**Pros:**
- Write content ONCE; register is the exporter's job
- Plays perfectly to data-first analyst persona
- Deterministic, no LLM dependency
- Structured data ensures factual consistency across outputs
- Works extremely well for data-heavy/tabular content

**Cons:**
- Complex DSL surface -- effectively a domain-specific schema language
- Templated prose sounds templated -- "The incident affected N systems" is adequate but not polished
- Limits human expressiveness -- cannot craft nuanced framing or emphasis
- Exporter templates become the bottleneck for quality
- Schema changes ripple through templates

**User effort:** Low for data-heavy content, but no path to high-quality narrative without escape hatches.

**Verdict:** Excellent for data-driven content. Insufficient alone for narrative-heavy sections (executive summaries, lessons learned, recommendations).

### Option D: Talk Track + Auto-Formalization

```
slide severity_cards:
  talk_track """
    Let me walk you through the impact. We had 5 systems
    affected. The big one was CI/CD -- 2,400 users were
    down for 4 hours. But here's the good news: we caught
    it in 23 minutes and everything's back up now.
  """
```

**Exporter mapping:**
- PPTX: talk_track as-is --> speaker notes
- DOCX: talk_track auto-transformed to formal register at export time (LLM or rule-based)

**Pros:**
- Lowest authoring burden -- write naturally, once
- Author writes in their most comfortable register (conversational)
- Captures domain expertise without requiring report-writing skill

**Cons:**
- **Hard LLM dependency at build time** -- the biggest architectural concern
- Non-deterministic output: same input may produce different report text across builds
- Quality risk: LLM may hallucinate details, misstate impact levels, or use inappropriate hedging
- Unacceptable for compliance/audit reports where every word has legal weight
- Cannot work in air-gapped/classified environments
- Caching/versioning of LLM output adds significant complexity
- Review bottleneck: someone must verify every auto-generated report

**User effort:** Very low for authoring. High for quality assurance.

**Verdict:** Good for drafting assistance (editor-side helper), bad as the canonical build path for sensitive documents. Appropriate as an optional v2+ feature, not as the core architecture.

### Option E: Dual-Track with Shared Data (RECOMMENDED)

```
slide severity_cards:
  # ... slide content (data-driven, from @data) ...

  notes """
    Walk through each system's impact. Emphasize
    the 23-minute detection time -- that's our SLA win.
    Expect questions about the CI/CD outage duration.
  """

  report """
    The impact assessment identified {{ affected_systems | len }}
    systems with confirmed exposure. The CI/CD Pipeline
    (serving {{ ci_cd.users }} users) experienced a
    {{ ci_cd.outage_hours }}-hour service disruption,
    the most significant impact of the incident.

    Detection latency of {{ detection_minutes }} minutes
    was within the {{ sla_minutes }}-minute SLA threshold
    (IRP-2025, S4.2).
  """
```

**Exporter mapping:**
- `notes` --> PPTX speaker notes (conversational, presenter-focused)
- `report` --> DOCX body text (formal, standalone, with data interpolation)
- Slide visual content --> data-driven from `@data` (shared)
- `detail` --> DOCX-only extended sections (optional)

**Pros:**
- Each register explicitly authored in its natural voice
- Shared data via `{{ }}` interpolation prevents fact drift
- No LLM dependency -- deterministic builds
- `report` is optional -- graceful degradation when only slides are needed
- `notes` can include delivery guidance ("expect questions about...") that has no report equivalent
- Matches how professionals actually work (McKinsey, NIST, military all maintain parallel artifacts sharing an analytical foundation)
- Both fields can use `@for`/`@if` for structured iteration

**Cons:**
- Author writes two texts for narrative-heavy slides (but NOT for data-heavy slides, which use structured content)
- The "same content, two ways" complaint is real but mitigated by data interpolation
- Requires the author (or their manager) to write report-quality prose

**User effort:** Medium. Shared data handles fact consistency; the author focuses on framing and register per output.

**Fallback behavior (critical for adoption):**
1. If `report` is present --> use for DOCX
2. If `report` is absent but structured content exists --> auto-render from templates
3. If neither --> omit section from DOCX (with build warning)

This means the Python analyst can write slides + notes (their current skill) and get a PPTX immediately. The DOCX is either auto-generated from data or manually authored when the manager adds `report` fields.

**Verdict:** Best fit for slideforge's users, architecture, and constraints.

---

## Recommendation

### Primary: Option E (Dual-Track with Shared Data)

**v1.0 scope:**
- `notes` field on any slide --> PPTX speaker notes
- `report` field on any slide --> DOCX body text
- Both support `{{ }}` interpolation and `@for`/`@if` from shared data
- `report` is optional -- omitting it produces a data-template-driven DOCX section or a build warning
- `detail` field for DOCX-only extended content (appendix material, deep-dive analysis)

**v1.0 keyword additions:**
```
notes    --> PPTX speaker notes (replaces talk_track)
report   --> DOCX body text
detail   --> DOCX extended sections (below report)
```

**Rationale for `notes` over `talk_track`:**
- `notes` is shorter, matches Pandoc/Quarto convention (`::: notes`), and is the standard term in PowerPoint/Keynote
- `talk_track` remains a valid alias (parser accepts both, canonical form is `notes`)

### Secondary: Option C (Structured Content) as escape hatch for v1.0

For data-heavy slides where writing prose in two registers is wasteful:

```
slide severity_cards:
  content:
    summary "5 systems impacted, all restored"
    evidence:
      @for system in affected_systems:
        row:
          system {{ system.name }}
          impact {{ system.impact_level }}
    conclusion "All systems restored. SLA met for detection."
```

The exporter auto-renders structured content into format-appropriate layouts:
- PPTX: visual cards, bullet summaries
- DOCX: formal tables, templated prose paragraphs

This is NOT a separate mode -- it coexists with Option E. A slide can have both structured content AND `notes`/`report` fields:

```
slide severity_cards:
  content:
    summary "5 systems impacted, all restored"
    evidence:
      @for system in affected_systems:
        row: ...
  notes """
    Walk through each system. Emphasize detection time.
  """
  report """
    The impact assessment identified {{ affected_systems | len }}
    systems with confirmed exposure...
  """
```

### v2.0 considerations (not in v1):
- **AI-assisted drafting:** Editor-side "draft report from notes" button using LLM, producing a `report` field for human review. Not in the build path.
- **Register linting:** Build-time check that `notes` uses conversational markers and `report` uses formal markers. Warning, not error.
- **Cross-reference generation:** Auto-generate "see Section 3.1" and "see Appendix B" references in `report` fields.

### Why This Matches the Industry

| Domain | Slide Artifact | Written Artifact | Shared Foundation | slideforge Mapping |
|--------|---------------|------------------|-------------------|--------------------|
| McKinsey | Deck with action titles | Recommendation memo (SCQA) | Analytical work, exhibits | `slide content` + `notes` --> PPTX; `report` --> DOCX |
| NIST IR | War-room slides | Formal incident report | Incident data, timeline | Structured `content` --> both; `notes`/`report` for narrative |
| Military | SITREP (briefed) | AAR (written memo) | Operational facts | `notes` = briefing text; `report` = formal memo |
| Audit | Committee briefing slides | SOC 2 / audit report | Control testing results | Structured data --> both; `report` for formal assertions |

---

## DSL Design Implications

### New Keywords (v1.0)

| Keyword | Scope | Purpose | Required? |
|---------|-------|---------|-----------|
| `notes` | Per-slide | PPTX speaker notes (conversational register) | Optional |
| `report` | Per-slide | DOCX body text (formal register) | Optional |
| `detail` | Per-slide | DOCX-only extended content | Optional |
| `content` | Per-slide | Structured semantic content block | Optional (alternative to `report` for data-heavy slides) |

### Exporter Decision Matrix

| Source Field | PPTX | DOCX | PDF | HTML |
|-------------|------|------|-----|------|
| Slide visual content | Rendered on slide | Rendered as figure/exhibit | Same as DOCX | Same as DOCX |
| `notes` | Speaker notes pane | Omitted (or footnote, configurable) | Omitted | Omitted |
| `report` | Omitted | Body text | Body text | Body text |
| `detail` | Omitted | Extended section (after report) | Extended section | Collapsible section |
| `content` (structured) | Auto-rendered per type | Auto-rendered as table/prose | Same as DOCX | Same as DOCX |

### Fallback Chain for DOCX Body Text

```
if slide.report exists:
    use slide.report  # explicitly authored formal prose
elif slide.content exists:
    auto-render from content templates  # structured data -> templated prose
elif slide.notes exists:
    emit build WARNING("slide X has notes but no report -- DOCX section will be minimal")
    use minimal template with slide title + data summary
else:
    emit build WARNING("slide X has no narrative content for DOCX")
    use slide title only
```

### Interaction with Existing DSL Features

- `{{ }}` interpolation works in `notes`, `report`, and `detail` fields identically to other string contexts.
- `@for`/`@if` directives work inside `notes`, `report`, and `detail` for structured iteration.
- `talk_track` (from current seed spec) becomes an alias for `notes` -- parser accepts both, emits deprecation notice for `talk_track`.
- `content` structured blocks use the same `@data` binding as slide visual content.

### Minimum Viable Dual-Register Experience

For the Python analyst who writes python-pptx scripts today:

1. **Day 1:** Write slides + `notes`. Get PPTX immediately. DOCX is auto-generated from structured data with template prose. Quality: adequate for internal use.
2. **Week 2:** Manager adds `report` fields to key slides (executive summary, key findings, recommendations). DOCX quality jumps to client/compliance grade.
3. **Month 2:** Team develops house templates for `content` structured blocks. Most data-heavy slides auto-render both outputs with no prose authoring.
4. **Month 6:** AI-assisted "draft report from notes" reduces the manager's writing burden (v2 feature).

This progressive adoption path means no user is forced to write both registers on day one, but the architecture supports it cleanly when they're ready.

---

## Sources

[1] Ethos3, "The Difference Between Speaker's Notes and Scripts." https://ethos3.com/the-difference-between-speakers-notes-and-scripts/

[2] Graylog, "Best Practices for Writing an IT Security Incident Report." https://graylog.org/post/best-practices-for-writing-an-it-security-incident-report/

[3] NIST SP 800-61r3, "Computer Security Incident Handling Guide." https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-61r3.pdf

[4] SANS, "Incident Handler Forms and Templates." https://sans.org/media/security-training/mgt512/secinc_forms.pdf

[5] Winning Presentations, "The Pyramid Principle: How McKinsey Structures Every Slide Deck." https://winningpresentations.com/pyramid-principle-presentations/

[6] Deckary, "Consulting Slide Standards: Rules McKinsey, BCG & Bain Follow." https://deckary.com/blog/consulting-slide-standards

[7] Supernormal, "Inside McKinsey's Presentation Playbook: Real Examples." https://www.supernormal.com/blog/mckinsey-presentation-playbook

[8] ModelThinkers, "Minto Pyramid & SCQA." https://modelthinkers.com/mental-model/minto-pyramid-scqa

[9] Deckary, "Consulting Presentations: MBB Slide Design & Structure Guide." https://deckary.com/blog/pillar-consulting-presentations-guide

[10] The Analyst Academy, "BCG Example -- Consulting Slide Structure." https://www.theanalystacademy.com/consulting-slide-structure/

[11] Palo Alto Networks, "Incident Response Plan Template." https://paloaltonetworks.com/cyberpedia/incident-response-plan-template

[12] CJCSM 3150.05F, "Joint Reporting Structure." https://www.jcs.mil/Portals/36/Documents/Library/Manuals/CJCSM%203150.05F.pdf

[13] FM 6-0, "Commander and Staff Organization and Operations." https://www.bits.de/NRANEU/others/amd-us-archive/FM6-0(14).pdf

[14] DINFOS Pavilion, "How to Create an After Action Report." https://pavilion.dinfos.edu/How-To/Article/2471936/how-to-create-an-after-action-report/

[15] Hack The Box, "Writing Incident Response Report Template." https://hackthebox.com/blog/writing-incident-response-report-template

[16] Horizon3.ai, "From War Room to Board Room: Own the Narrative." https://horizon3.ai/downloads/whitepapers/from-war-room-to-board-room-own-the-narrative

[17] Sygnia, "Incident Response Policies." https://sygnia.co/blog/incident-response-policies

[18] incident.io, "Best Incident Management Tools for Compliance Auditing." https://incident.io/blog/best-incident-management-tools-for-compliance-auditing

[19] Workiva, "Intro to Workiva AI." https://support.workiva.com/hc/en-us/articles/19163862687764-Intro-to-Workiva-AI

[20] Stepsize, "10 Best AI Report Generators." https://stepsize.com/blog/best-ai-report-generators

[21] Pandoc Manual, "Speaker Notes." https://pandoc.org/demo/example33/10.5-speaker-notes.html

[22] Pandoc Manual (full). https://pandoc.org/MANUAL.html

[23] Quarto, "Conditional Content." https://quarto.org/docs/authoring/conditional.llms

[24] Quarto GitHub Discussions, "Multi-format rendering pain points." https://github.com/orgs/quarto-dev/discussions/7018

[25] Evernote, "Rewrite Report in Corporate Style with AI." https://evernote.com/ai-rewrite/rewrite-report-in-corporate-style-with-ai

[26] Socialite AI, "AI Conversion Tool." https://dang.ai/tool/ai-conversion-tool-socialiteai

[27] Barbara Minto, "The Pyramid Principle" (Slideshare summary). https://www.slideshare.net/raquelcrespo/the-pyramid-principle-by-barbara-minto

[28] Slideworks, "The Pyramid Principle -- Consulting Toolbox." https://slideworks.io/resources/the-pyramid-principle-mckinsey-toolbox-with-examples

[29] DoD, "After Action Report Template." https://media.defense.gov/2021/Jan/15/2002565598/-1/-1/1/PAVILION%20After%20Action%20Report%20Template.DOCX

[30] US Army, "Training and Evaluation Outline -- AAR Preparation." https://rdl.train.army.mil/catalog-ws/view/100.ATSC/4CBAEFE6-5518-415F-B6B3-3D254EA4443B-1658770816640/report.pdf

[31] Quarto, "Presentations Overview." https://quarto.org/docs/presentations/

[32] Quarto Revealjs Tips. https://remlapmot.github.io/post/2025/quarto-revealjs-tips/

[33] TicNote, "10 AI Report Generator Tools for 2026." https://ticnote.com/en/blog/ai-report-generator-tools

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 2 | Deep research on linguistic register differences across domains; tool landscape for slide-to-document transformation |
| Perplexity perplexity_ask | 2 | Precise linguistic comparison table for incident reports; Quarto/Pandoc dual-output mechanisms |
| Perplexity perplexity_reason | 1 | DSL design option synthesis and hybrid recommendation analysis |
| Perplexity perplexity_search | 3 | Military SITREP vs AAR format differences; McKinsey/BCG pyramid principle methodology; AI register transformation tools |
| Context7 | 2 resolve + 2 query | Quarto conditional content/notes handling; Pandoc speaker notes in slide-to-document conversion |
| Tavily tavily_research | 1 | McKinsey/BCG/NIST methodology documentation cross-validation |
| Training data | 2 areas | General knowledge of writing register theory; awareness of audit report conventions (flagged as lower confidence) |

**Total MCP tool calls:** 13
**Training data reliance:** low -- all major claims verified against web sources; audit report conventions and IEEE/ACM conventions draw partially on model knowledge but are well-established in published literature.
