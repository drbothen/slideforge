---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-020
lifecycle_status: active
introduced: v1.0.0
modified: ["2026-06-04", "2026-06-12"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-5.01.005: lang Declaration Propagates to PPTX Core Properties, PPTX Run rPr, PDF /Lang, HTML lang Attr

## Description

The `lang "..."` declaration from the deck metadata (or the "en" default from BC-5.01.004)
must propagate to all output formats and all surfaces within those formats: PPTX
`docProps/core.xml` `<dc:language>`, PPTX `<a:rPr lang="..."/>` on every text run, PDF
`/Catalog` dictionary `/Lang` entry, DOCX `<w:lang w:val="..."/>` on every text run, and
HTML `<html lang="...">` attribute. This ensures screen readers use the correct
pronunciation engine regardless of which output format is consumed. The propagation is a
semantic requirement, not a cosmetic one.

## Preconditions

1. A `lang` value is available in the deck metadata (either explicit or defaulted to "en"
   per BC-5.01.004).
2. Export is requested for at least one format (PPTX, PDF, or HTML).

## Postconditions

1. PPTX (core properties): `docProps/core.xml` contains `<dc:language>LANG</dc:language>`
   where LANG is the exact BCP-47 tag from the deck declaration (or "en" by default).
2. PPTX (run level): every `<a:rPr>` element in every `slideN.xml` carries
   `lang="LANG"` — the same LANG value as postcondition 1.
3. PDF: The `/Catalog` dictionary includes `/Lang (LANG)` entry.
4. DOCX (run level): every `<w:rPr>` in `word/document.xml` carries
   `<w:lang w:val="LANG"/>` — the same LANG value as postconditions 1 and 2.
5. HTML (static and web preview): `<html lang="LANG">` attribute is set.
6. The LANG value is identical across ALL surfaces (PC-1 through PC-5) — no
   format-specific transformation (e.g., "en-US" declared → "en-US" everywhere;
   no lang declared → "en" everywhere, never "en-US").
7. veraPDF validates the PDF `/Lang` entry as part of the PDF/UA-1 check (BC-4.03.001).

## Invariants

1. Language propagation is lossless — no truncation, normalization, or case change.
   "zh-Hant-TW" stays "zh-Hant-TW" in all formats.
2. `deck.metadata.lang` (`DeckMetadata.lang: Option<Arc<str>>` in the semantic IR) is the
   single source of truth for the language tag. All exporters read lang from this field via
   the `deck: &Deck` parameter provided by the `Exporter` trait. No exporter may derive or
   override the lang value from any other source. (`LaidOutDeck` carries no `lang` field;
   threading lang through the geometric IR is unnecessary because every exporter already
   receives the semantic `Deck`.)
3. The propagation is unconditional — even when building only one format (e.g., `--format
   pptx`), the lang is embedded in that format.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | lang "en" (2-letter ISO) | PPTX `<dc:language>en</dc:language>`; PPTX `<a:rPr lang="en"/>`; PDF `/Lang (en)`; DOCX `<w:lang w:val="en"/>`; HTML `lang="en"` — NOT "en-US" on any surface |
| EC-002 | lang "zh-Hant-TW" (4-part BCP-47) | All formats and all surfaces embed "zh-Hant-TW" unchanged |
| EC-003 | lang not declared (default "en") | All formats and all surfaces embed "en" (from BC-5.01.004 default) — NOT "en-US" |
| EC-004 | PDF /Lang missing (regression) | veraPDF fails — BC-4.03.001 catches this as a CI gate |
| EC-005 | HTML preview: lang changes when variant is built with different lang | HTML `lang` attribute updated on WebSocket reload |
| EC-006 | PPTX run with no text content (empty rPr) | No `<a:rPr>` emitted; no error; no lang attribute injection into empty runs |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `lang "en-US"` → PPTX export | `docProps/core.xml`: `<dc:language>en-US</dc:language>`; all `<a:rPr lang="en-US"/>` | happy-path |
| `lang "fr-FR"` → PDF export | PDF `/Catalog` dict: `/Lang (fr-FR)` | happy-path |
| `lang "ja"` → HTML export | `<html lang="ja">` in output HTML | happy-path |
| `lang "en-US"` → DOCX export | all `<w:rPr><w:lang w:val="en-US"/></w:rPr>` | happy-path |
| No lang declared → PPTX export | `<dc:language>en</dc:language>` AND all `<a:rPr lang="en"/>` — NOT "en-US" | edge-case |
| No lang declared → any export | All formats and all surfaces embed "en" (default, not "en-US") | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | PPTX dc:language matches deck lang declaration exactly | unit test: parse PPTX core.xml |
| VP-TBD | PPTX a:rPr lang matches deck lang declaration exactly (including "en" default) | unit test: parse slideN.xml; assert every a:rPr has lang attr |
| VP-TBD | PDF /Lang entry present and matches deck lang | unit test: parse PDF catalog dict |
| VP-TBD | DOCX w:lang w:val matches deck lang declaration exactly | unit test: parse word/document.xml |
| VP-TBD | HTML html[lang] attribute matches deck lang declaration | unit test: parse HTML |
| VP-TBD | No lang declared → "en" (not "en-US") on ALL surfaces across all formats | unit test: build no-lang deck; assert all surfaces carry "en" |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 |
| Capability Anchor Justification | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 — language propagation to "PPTX Core Properties, PDF /Lang, HTML lang" is explicit in CAP-020 and DI-003 |
| L2 Domain Invariants | DI-003 (deck language propagates to all output formats) |
| Architecture Module | slideforge-pptx, slideforge-pdf, slideforge-html crates (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.01.004 — depends on (establishes the lang value, including default "en")
- BC-4.01.004 — composes with (PPTX dc:language embedding is part of PPTX accessibility metadata)
- BC-4.01.005 — composes with (PPTX a:rPr lang on runs is part of PPTX accessibility compliance; implemented by STORY-096)
- BC-4.02.001 — composes with (DOCX w:lang on runs is part of DOCX accessibility compliance; implemented by STORY-099)
- BC-4.03.001 — depends on (veraPDF validates PDF /Lang as part of PDF/UA-1)
- BC-4.03.003 — depends on (HTML lang attribute is part of WCAG AA compliance)

## Architecture Anchors

- `architecture/plugin-architecture.md` — language propagation pipeline
- `architecture/ir-design.md` — DeckMetadata.lang as single source of truth for language (semantic IR; LaidOutDeck carries no lang field)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-05-24 | product-owner | Initial draft |
| 1.1 | 2026-05-24 | product-owner | Added EC-004/EC-005, related BCs, architecture anchors |
| 1.2 | 2026-06-04 | product-owner | Invariant 2 corrected: lang source-of-truth is `deck.metadata.lang` (`DeckMetadata.lang: Option<Arc<str>>`), not `LaidOutDeck.lang` (which never existed). Architecture Anchors updated to match. Human-authorized spec amendment 2026-06-04 per Source-of-Truth rule 7; architect-recommended. No code change required — all exporters were already reading `deck.metadata.lang` correctly via the `Exporter` trait `deck: &Deck` parameter. |
| 1.3 | 2026-06-12 | product-owner | PO adjudication of F-096-002 (STORY-096 cascade finding): expanded scope to include PPTX `<a:rPr lang>` and DOCX `<w:lang>` run-level surfaces (formerly only documented for docProps/PDF/HTML). Ruling: canonical default is "en" (per BC-5.01.004), NOT "en-US", on ALL surfaces including run-level. STORY-096 AC-003 and STORY-099 AC-004 diverged to "en-US" — both corrected in same burst. PC-6 (cross-surface identity) updated to make the "en" default explicit. EC-003 and EC-006 updated. Test vectors expanded for run-level surfaces. |
