---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified: ["2026-06-04"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-5.01.005: lang Declaration Propagates to PPTX Core Properties, PDF /Lang, HTML lang Attr

## Description

The `lang "..."` declaration from the deck metadata (or the "en" default from BC-5.01.004)
must propagate to all output formats: PPTX `docProps/core.xml` `<dc:language>`, PDF
`/Catalog` dictionary `/Lang` entry, and HTML `<html lang="...">` attribute. This
ensures screen readers use the correct pronunciation engine regardless of which output
format is consumed. The propagation is a semantic requirement, not a cosmetic one.

## Preconditions

1. A `lang` value is available in the deck metadata (either explicit or defaulted to "en"
   per BC-5.01.004).
2. Export is requested for at least one format (PPTX, PDF, or HTML).

## Postconditions

1. PPTX: `docProps/core.xml` contains `<dc:language>LANG</dc:language>` where LANG is
   the exact BCP-47 tag from the deck declaration.
2. PDF: The `/Catalog` dictionary includes `/Lang (LANG)` entry.
3. HTML (static and web preview): `<html lang="LANG">` attribute is set.
4. The LANG value is the same in all formats — no format-specific transformation
   (e.g., "en-US" in PPTX becomes "en-US" in PDF, not "en").
5. veraPDF validates the PDF `/Lang` entry as part of the PDF/UA-1 check (BC-4.03.001).

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
| EC-001 | lang "en" (2-letter ISO) | PPTX `<dc:language>en</dc:language>`; PDF `/Lang (en)`; HTML `lang="en"` — not "en-US" |
| EC-002 | lang "zh-Hant-TW" (4-part BCP-47) | All formats embed "zh-Hant-TW" unchanged |
| EC-003 | lang not declared (default "en") | All formats embed "en" (from BC-5.01.004 default) |
| EC-004 | PDF /Lang missing (regression) | veraPDF fails — BC-4.03.001 catches this as a CI gate |
| EC-005 | HTML preview: lang changes when variant is built with different lang | HTML `lang` attribute updated on WebSocket reload |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `lang "en-US"` → PPTX export | `docProps/core.xml`: `<dc:language>en-US</dc:language>` | happy-path |
| `lang "fr-FR"` → PDF export | PDF `/Catalog` dict: `/Lang (fr-FR)` | happy-path |
| `lang "ja"` → HTML export | `<html lang="ja">` in output HTML | happy-path |
| No lang declared → any export | All formats embed "en" (default) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | PPTX dc:language matches deck lang declaration exactly | unit test: parse PPTX core.xml |
| VP-TBD | PDF /Lang entry present and matches deck lang | unit test: parse PDF catalog dict |
| VP-TBD | HTML html[lang] attribute matches deck lang declaration | unit test: parse HTML |

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
