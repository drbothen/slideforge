---
title: Competing Slide DSL User-Pain Analysis
date: 2026-05-23
analyst: research-agent
status: foundation-research
audience: business-analyst + product-owner + architect
---

# Competing Slide DSLs — User-Pain Mining

## Executive Summary

After mining GitHub issues, discussions, blog posts, and comparison articles for the nine major slide-DSL competitors, **five user-pain patterns recur across the landscape**. The architect should design slideforge to dodge these:

1. **The "Markdown ceiling" — layout escape hatch is always HTML/CSS.** Every markdown-based tool (Marp, Slidev, Quarto-revealjs) requires raw HTML, `<style scoped>`, CSS Grid, or Vue components to do columns, complex tables, or branded layouts. Users complain that "I switched to Marp to escape PowerPoint, now I'm writing CSS Grid Layout." [Marp #62](https://github.com/orgs/marp-team/discussions/62), [Marp #132](https://github.com/marp-team/marp-core/issues/132)

2. **Output-format fidelity gap — what renders in HTML rarely renders in PPTX.** Pandoc's pptx writer corrupts files when math is in titles/tables ([#9465](https://github.com/jgm/pandoc/issues/9465)), drops images in two-column layouts ([#7595](https://github.com/jgm/pandoc/issues/7595)), and broke image embedding in v3.0 ([#8565](https://github.com/jgm/pandoc/issues/8565)). Quarto users have "no chance" of auto-animations cross-rendering to PowerPoint. [Posit forum](https://forum.posit.co/t/tips-and-tricks-for-presentations-that-are-both-reveal-js-and-powerpoint/174556)

3. **Default theme is shameful, custom theming is painful.** Slidev's maintainer publicly admits "even experienced developers shouldn't need to create custom themes for basic presentations" ([#2128](https://github.com/slidevjs/slidev/issues/2128)). Marp themes require CSS. python-pptx font substitution silently breaks line layout ([slideforge blog](https://slideforge.dev/blog/python-pptx-limitations-we-solved)).

4. **No validation, no preview-of-canvas-bounds, no overflow warnings until rendered.** Tables export with double-header bugs (Marp #330), python-pptx places text past the 13.33" right edge without warning, Quarto's auto-stretch silently fails on scrollable slides. The "looks fine in preview, broken in export" cycle dominates issue trackers.

5. **Heavy install footprint = abandonment for casual use.** "I am a teacher, currently using Marp, and I have hundreds of presentations. Slidev would require a 500MB standalone project for each presentation" ([Slidev #63](https://github.com/slidevjs/slidev/issues/63)). Node.js + npm + Vite + Vue + per-project node_modules is a recurring complaint. Marp's "no Node, just CLI binary" is repeatedly cited as why people *stay* with Marp despite limitations.

---

## Per-Tool Pain Analysis

### Marp

**Top complaints:**
- **Tables require enabling HTML and writing raw `<table>` + custom CSS** to lay out non-trivially. [#155 Tables Settings, 2020-03-18](https://github.com/marp-team/marp-core/issues/155)
- **Multi-column layouts are not first-class** — requires CSS columns or Flexbox via `<style>` blocks. [#132 Multi-column in single slide, 2019-11-18](https://github.com/marp-team/marp-core/issues/132)
- **Table export bug: header repeats when content overflows slide.** Upstream Chromium bug, no Marp-side workaround beyond manual font-shrinking. [marp-vscode #330](https://github.com/marp-team/marp-vscode/issues/330)
- **HTML allowlist confusion** — pre-v4, users repeatedly didn't realize they had to opt-in to `<style>` and `<table>` tags. Marp v4 changelog explicitly calls this out as "a very common pitfall." [Marp Core v4 Discussion #533, 2024-09-12](https://github.com/orgs/marp-team/discussions/533)
- **Centering surprises in v4** — old decks rendered differently after v4's `display: flex` → `display: block` change. [Discussion #533](https://github.com/orgs/marp-team/discussions/533)
- **`marp: true` directive is per-file but the CLI server renders *all* `.md` as slides**, cluttering output. [Discussion #301, 2022-05-03](https://github.com/orgs/marp-team/discussions/301)
- **Ctrl-+ zoom is broken** because Marp scales SVG to viewport — major accessibility failure for low-vision users. [marp-cli #636, 2025-01-20](https://github.com/marp-team/marp-cli/issues/636)
- **Fragmented repos** — newcomers get lost across marp-core, marpit, marp-cli, marp-vscode; people don't know where to file issues. [Discussion #209, 2021-11-08](https://github.com/orgs/marp-team/discussions/209)
- **Syntax highlighting languages limited** — "not all languages are highlighted and I didn't find how to add support for the missing languages." [Tonai blog 2022-01-18](https://tonai.github.io/blog/posts/slide-libraries/)
- **No Mermaid support out-of-the-box** — recurring complaint that drives users to Slidev. [sharayeh.com Marp alternative 2026-03-19](https://sharayeh.com/en/blog/marp-alternative-best-markdown-presentation-tools)

**Recurring feature requests:**
- First-class column/grid layout primitives (so users don't write CSS Grid by hand)
- Built-in TOC generation
- Mermaid/diagram inline support

**Where it hits ceilings:**
- 2D layouts ("I like to organize my presentation topics in columns and this helps pacing the presentation" — [Medium, 2021-11-10](https://medium.com/@lhcamilo/marp-is-nice-but-it-lags-behind-reveal-js-89d92891e5fb))
- Custom branding requiring per-slide layout variation
- Anything beyond bullets/code/full-bleed image: "Best for slides that are just bullet points, full-slide images, and code. Less good if you have a lot of images or need to do your own layout." [LibHunt comparison 2026-04-22](https://www.libhunt.com/compare-marp-vs-reveal.js)

---

### Slidev

**Top complaints:**
- **1,073 open issues** as of August 2025, including many "pending triage." [slidevjs/slidev/issues](https://github.com/slidevjs/slidev/issues)
- **Default theme has well-known typography problems** — "inconsistent typography, excessive top margins in headings, improper line spacing in lists, and many more" — admitted by maintainer. [#2128, 2025-03-22](https://github.com/slidevjs/slidev/issues/2128)
- **No global install** historically — required full per-project node_modules, painful for teachers/instructors with many decks. [#63, 2021-05-11](https://github.com/slidevjs/slidev/issues/63)
- **Math syntax breaks with `{{ }}` interpolation tokens** — "`{{` inside math causes unclear errors." [#2245 (Aug 2025)](https://github.com/slidevjs/slidev/issues) and [discussion thread](https://github.com/slidevjs/slidev/discussions)
- **`export --range` broken since v0.50.0** [#2227, 2025-07-14](https://github.com/slidevjs/slidev/issues)
- **Imported snippets do not hot-reload automatically** — `@include`-equivalent issue. [#2241, 2025-08-11](https://github.com/slidevjs/slidev/issues)
- **Built-in components stopped working in v51.x** for code-block highlighting/animation. [#2066, 2025-02-16](https://github.com/slidevjs/slidev/issues/2066)
- **No BiDi (right-to-left text) support.** Open issue, 2025. [slidevjs/slidev/issues](https://github.com/slidevjs/slidev/issues)
- **No i18n for presenter UI** — feature request for UI internationalization is open. [slidevjs/slidev/issues](https://github.com/slidevjs/slidev/issues)
- **No PPTX export** (only PDF/HTML) — repeated complaint vs. Marp. [sharayeh.com 2026-03-19](https://sharayeh.com/en/blog/marp-alternative-best-markdown-presentation-tools)

**Recurring feature requests:**
- Lighter footprint / global install / external slide file support [#63](https://github.com/slidevjs/slidev/issues/63)
- True PPTX export
- Better presenter mode that's distinct from viewer mode [Aug 26 2025](https://github.com/slidevjs/slidev/issues)

**Where it hits ceilings:**
- Files become bloated: "Slidev results in larger markdown files with all the extras and frontmatter I need to sprinkle in" [Slidev Discussion #86, 2021-05-11](https://github.com/slidevjs/slidev/discussions/86)
- Vue dependency raises the barrier: "Vue.js dependency. Developer-only tool." [sharayeh.com 2026-03-19](https://sharayeh.com/en/blog/marp-alternative-best-markdown-presentation-tools)

---

### Reveal.js

**Top complaints:**
- **774 open issues** as of August 2025. [hakimel/reveal.js/issues](https://github.com/hakimel/reveal.js/issues)
- **Math (KaTeX/MathJax) in external markdown does not render correctly** [#3786, 2025-04-03; #3796, 2025-05-06](https://github.com/hakimel/reveal.js/issues)
- **LaTeX spacing commands not rendered correctly** [#3813, 2025-08-18](https://github.com/hakimel/reveal.js/issues)
- **Screen-reader improvements still requested in 2025** [#issue, 2025-03-24](https://github.com/hakimel/reveal.js/issues); accessibility plugin is third-party ([marcysutton/reveal-a11y](https://github.com/marcysutton/reveal-a11y))
- **Scroll View URL hash not working** [#3815, 2025-08-22](https://github.com/hakimel/reveal.js/issues)
- **Speaker view jumps when used with r-stack and scroll view** [#3792, 2025-04-19](https://github.com/hakimel/reveal.js/issues)
- **No native external HTML import** (only external markdown) [#2441, 2019-06-22](https://github.com/hakimel/reveal.js/issues/2441) — open for 6+ years
- **Customization via HTML comments is fragile** — `vite-plugin-mdx` strips comments, breaking reveal.js attributes. [Tonai blog 2022-01-18](https://tonai.github.io/blog/posts/slide-libraries/)
- **No built-in TOC.** [Tonai blog](https://tonai.github.io/blog/posts/slide-libraries/)
- **Tailwind 4 breaks section transitions** [#3782, 2025-03-26](https://github.com/hakimel/reveal.js/issues)

**Recurring feature requests:**
- First-class screen-reader/a11y support
- Multi-user collaborative editing [#3812, 2025-08-14](https://github.com/hakimel/reveal.js/issues)
- Better LaTeX/math handling in markdown mode

**Where it hits ceilings:**
- Requires HTML literacy: "you are authoring a web presentation directly with HTML, CSS, and JavaScript" — [pkgpulse 2026-05-16](https://www.pkgpulse.com/guides/slidev-vs-marp-vs-revealjs-code-first-presentations-2026)
- Plugin sprawl: each animation/chart/diagram is a separate plugin to wire up.

---

### Pandoc (pptx output)

**Top complaints:**
- **pptx file corruption from math in headings/tables** — Microsoft PowerPoint demands "repair" on open. [#9465, 2024-02-15](https://github.com/jgm/pandoc/issues/9465)
- **Data-URL PNG images cause corruption when combined with reference-doc templates** [#9113, 2023-09-29](https://github.com/jgm/pandoc/issues/9113)
- **Document title block in RST output produces corrupt pptx** [#4181, 2017-12-20](https://github.com/jgm/pandoc/issues/4181)
- **Outline math in speaker notes corrupts files** [#6301, 2020-04-19](https://github.com/jgm/pandoc/issues/6301)
- **v3.0 silently dropped image embedding** — regression vs 2.19.2 [#8565, 2023-01-20](https://github.com/jgm/pandoc/issues/8565)
- **Images in two-column layouts go missing** [#7595, 2021-09-27](https://github.com/jgm/pandoc/issues/7595)
- **Template editing in LibreOffice removes pptx layouts** that pandoc needs (Section Header, etc.) [#10085, 2024-08-13](https://github.com/jgm/pandoc/issues/10085)
- **No pptx *input* support** — one-way conversion only [#4252, 2018-01-13](https://github.com/jgm/pandoc/issues/4252)
- **No template variable interpolation for pptx** — "pptx has no template. Use a reference doc to adjust the styles" — only `.docx`, `.odt`, `.pptx` get reference-doc, but no full pandoc-style template control. [Pandoc Manual](https://pandoc.org/MANUAL.html)
- **Background images, fonts, animations all need to be re-set per-output** for cross-rendering revealjs→pptx [Posit forum 2023-10-04](https://forum.posit.co/t/tips-and-tricks-for-presentations-that-are-both-reveal-js-and-powerpoint/174556)

**Recurring feature requests:**
- Native pptx reader [#4252](https://github.com/jgm/pandoc/issues/4252)
- Template/variable interpolation in pptx writer (parity with LaTeX)
- Better diagnostics for "this will corrupt" vs silent emit

**Where it hits ceilings:**
- Anything beyond bullets-in-content-placeholders: math, complex tables, custom layouts, data-URL images all risk corruption.

---

### Quarto

**Top complaints:**
- **Auto-stretch breaks scrollable slides** — images silently disappear. [Quarto docs](https://quarto.org/docs/presentations/revealjs/)
- **Multiple revealjs presentations in a website share one CSS file** — last render wins. [#8383, 2024-01-22](https://github.com/quarto-dev/quarto-cli/issues/8383)
- **revealjs→pptx cross-rendering breaks**: headings reshuffle, backgrounds vanish, fonts substitute, images disappear, "auto-animation: lol, no chance." [Posit forum 2023-10-04](https://forum.posit.co/t/tips-and-tricks-for-presentations-that-are-both-reveal-js-and-powerpoint/174556)
- **PowerPoint slide layouts don't match expectations** — quarto chooses layout based on whether a slide has "text followed by non-text" heuristic, surprising users. [#13885](https://github.com/quarto-dev/quarto-cli/issues/13885)
- **Self-contained reveal.js limitation** — Mermaid/external JS dependencies must be embedded as SVG/PNG; can't rely on JS at runtime. [Quarto docs](https://quarto.org/docs/presentations/revealjs/)
- **Code blocks capped at 500px height** before scrollbar appears. [Quarto docs](https://quarto.org/docs/presentations/revealjs/)
- **Container epic of revealjs issues** in quarto-cli — 20+ items tracked under [#4894, 2023-03-20](https://github.com/quarto-dev/quarto-cli/issues/4894): font sizing, MathJax3 support, multi-doc menu issues, PDF export columns misaligned, etc.
- **No native PowerPoint background images** via markdown syntax — must edit `.pptx` template manually. [Posit forum 2023-10-04](https://forum.posit.co/t/tips-and-tricks-for-presentations-that-are-both-reveal-js-and-powerpoint/174556)

**Recurring feature requests:**
- Genuine cross-format fidelity (revealjs ↔ pptx ↔ beamer)
- Better default themes that don't require Sass knowledge
- Per-format style overrides without duplicating content

**Where it hits ceilings:**
- "PPTX output is fundamentally limited by what Pandoc's pptx writer supports" — inherits all Pandoc pain points.

---

### Touying (Typst)

**Top complaints (low volume — younger project):**
- **`#pause` combined with `#uncover` and `#only` behaves unexpectedly** [#issue (open Mar 2025)](https://github.com/touying-typ/touying/issues)
- **Pagebreak not working with Theorion** [bug, open](https://github.com/touying-typ/touying/issues)
- **Cannot pass config to `#title-slide` in Metropolis theme** [#170, 2025-04-01](https://github.com/touying-typ/touying/issues/170)
- **Slides after table not rendering** [bug, open](https://github.com/touying-typ/touying/issues)
- **Wrong title/subtitle in `theme.simple`** [#70, 2024-09-05](https://github.com/touying-typ/touying/issues/70) — required workaround code
- **Layout did not converge under "weird circumstances"** leading to wrong page titles [bug, open](https://github.com/touying-typ/touying/issues)
- **Aspect ratio "16-10" not initially supported** — users had to discover this via forum [Typst forum 2026-03-12](https://forum.typst.app/t/touying-vertical-space-after-the-slide-title-line/8214)
- **Counter problems with ctheorems** [bug, upstream Typst issue](https://github.com/touying-typ/touying/issues)
- **Touying template uses outdated import** — `wontfix` label [open issue](https://github.com/touying-typ/touying/issues)

**Touying strengths the architect should study (it is the closest cousin to slideforge):**
- Touying compiles in milliseconds via Typst — fast feedback loop is a major selling point. [touying-typ.github.io](https://touying-typ.github.io)
- Built-in themes (Simple, University, Metropolis, Dewdrop, Aqua, Stargazer) provide zero-config presentations.
- Multi-output: PPTX *and* HTML export from same source.
- Animation as first-class (`#pause`, `#meanwhile`, `#uncover`, `#only`, `#alternatives`) — much more ergonomic than reveal.js fragments.

**Recurring feature requests:**
- Searchable API reference docs [#222](https://github.com/touying-typ/touying/issues)
- Navigation buttons enhancement [open](https://github.com/touying-typ/touying/issues)

**Where it hits ceilings:**
- Requires Typst installation — small but real barrier vs "single markdown file."
- Animation interaction edge cases (uncover×only×pause) regularly surprise users.

---

### python-pptx

**Top complaints (from 2026-04-20 production retrospective + GitHub issues):**
- **No validation: places text past canvas bounds without warning.** "There's no `overflow: hidden`, no canvas-bounds validator." Production teams write 46-rule heuristic linters externally. [slideforge blog 2026-04-20](https://slideforge.dev/blog/python-pptx-limitations-we-solved)
- **Merging decks silently loses embedded images** — Picture shapes reference rel IDs that don't exist in target deck. ~80 LOC custom helper required. [slideforge blog 2026-04-20](https://slideforge.dev/blog/python-pptx-limitations-we-solved)
- **Font substitution is silent** — `tf.font.name = "Helvetica Neue"` writes the name whether or not it exists; PowerPoint substitutes locally, changing line breaks. [slideforge blog 2026-04-20](https://slideforge.dev/blog/python-pptx-limitations-we-solved)
- **Performance O(N²) past ~500 slides** — `add_slide()` rescans all part names. [#644, 2020-09-01](https://github.com/scanny/python-pptx/issues/644)
- **No chart types beyond the basic six** — no waterfall, funnel, marimekko, radar, sunburst, treemap, bullet, heatmap, Gantt. [slideforge blog 2026-04-20](https://slideforge.dev/blog/python-pptx-limitations-we-solved)
- **Animation support effectively absent** [slideforge blog 2026-04-20](https://slideforge.dev/blog/python-pptx-limitations-we-solved)
- **Chart `replace_data()` is fragile** — shape must match existing chart exactly or exception. [slideforge blog 2026-04-20](https://slideforge.dev/blog/python-pptx-limitations-we-solved)
- **`add_picture()` requires local filesystem path** — no URLs. [slideforge blog 2026-04-20](https://slideforge.dev/blog/python-pptx-limitations-we-solved)
- **XML round-trips drop slide master properties** — SmartArt, custom XML parts get stripped on re-serialize. [slideforge blog 2026-04-20](https://slideforge.dev/blog/python-pptx-limitations-we-solved)
- **No built-in PDF export** [slideforge blog 2026-04-20](https://slideforge.dev/blog/python-pptx-limitations-we-solved)
- **No preview without saving and opening externally** [Stack Overflow recurring pattern](https://stackoverflow.com/questions/59561983/python-pptx-keyerror-no-placeholder-on-this-slide-with-idx-1)
- **Corruption from deepcopy and other innocent-looking operations** [#87, 2014-04-16](https://github.com/scanny/python-pptx/issues/87); [#941, 2024-01-26](https://github.com/scanny/python-pptx/issues/941)
- **Placeholder idx errors when copying templates** [SO #59561983](https://stackoverflow.com/questions/59561983/python-pptx-keyerror-no-placeholder-on-this-slide-with-idx-1)
- **Boilerplate-heavy API**: textboxes → paragraphs → runs, tables populated cell-by-cell. [washstat.org PDF 2019](https://www.washstat.org/presentations/20190923/Wilcox-Cook.pdf)
- **No error messages on schema mismatches** until you open in PowerPoint and click "Repair." [#87, #941]

**Recurring feature requests:**
- Higher-level "deck = compose(slides=[...])" API
- Native PDF export
- More chart types
- URL-based image insertion

**Where it hits ceilings:**
- The library is library-level low-level XML manipulation, not a presentation DSL. Anything resembling "describe this deck declaratively" must be built on top.

---

### MDX-deck / Spectacle

**Top complaints:**
- **mdx-deck is unmaintained** — archived/abandoned. "I think there are none which support mdx" with active maintenance. [#765, 2020-11-26](https://github.com/jxnblk/mdx-deck/issues/765)
- **Breaking changes between v1/v2/v3/v4** — multiple migration docs. Functional themes deprecated, multiple-file decks deprecated, swipe gestures removed, fixed aspect ratio removed. [MIGRATION.md](https://github.com/jxnblk/mdx-deck/blob/master/MIGRATION.md)
- **v2 broke slide-splitting** — `---` now requires empty newlines around it (CommonMark thematicBreak), silently breaking existing decks. [MIGRATION.md](https://github.com/jxnblk/mdx-deck/blob/master/MIGRATION.md)
- **Multiple MDX files can no longer be combined** as of v4 [MIGRATION.md](https://github.com/jxnblk/mdx-deck/blob/master/MIGRATION.md)
- **Spectacle no `imports/requires`** — "everything must come from the global namespace" in MDX mode. [jonathan-fielding/spectacle 2019](https://github.com/jonathan-fielding/spectacle)
- **Spectacle MDX requires webpack config** for the loader — high setup cost. [spectacle-mdx-loader](https://www.npmjs.com/package/spectacle-mdx-loader)

**Recurring feature requests:**
- Active maintenance (closed as wontfix; community migrated to Spectacle v10+ or ReMDX)

**Where it hits ceilings:**
- Whole class of tool is now niche / fading; React-based slide DSLs lost market share to Slidev/Marp.

---

## Pattern Extraction

### Top-5 recurring complaints across tools

1. **"Markdown's layout escape hatch is HTML/CSS, and I came to markdown to escape HTML/CSS."** Recurs in Marp, Slidev, Reveal.js, Quarto. Users want column/grid/two-col syntax as first-class.

2. **"PPTX export silently corrupts or silently drops content."** Pandoc, Quarto, python-pptx all surface this. Users can't trust the round-trip.

3. **"Default theme looks unprofessional; custom theming requires Sass/CSS/Vue knowledge."** Slidev maintainer admits it; Marp, Reveal.js, Quarto all share.

4. **"No validation, no overflow warnings, no canvas-bounds checks until I render and look."** Universal across all tools except Touying (which at least warns on layout non-convergence).

5. **"Heavy dependency footprint per project."** Slidev's node_modules, MDX-deck's webpack, Spectacle's React/babel pipeline. Marp's standalone binary is cited as the *reason people stay*.

### Top-5 recurring feature requests across tools

1. **First-class multi-column / grid layout primitives** (not requiring HTML/CSS escape hatch).
2. **Mermaid / diagrams as native syntax** (Marp users complain about absence; Slidev has it and users praise it).
3. **PPTX export with high fidelity** (Slidev users; revealjs cross-render users).
4. **Built-in TOC / outline generation** (Reveal.js, Marp, Quarto — all missing).
5. **Lightweight install / global CLI / single-file decks** (Slidev #63, MDX-deck abandonment).

### Top-5 ergonomics traps (DSL-syntax-level)

1. **CommonMark `---` slide separator collides with YAML frontmatter `---`.** MDX-deck v2 broke decks because users didn't surround `---` with empty newlines. Marp inherits the same risk. slideforge should make slide separators unambiguous (named slide types help here — `slide.cover`, not `---`).

2. **YAML/frontmatter type-coercion silently corrupts values.** `version: 1.10` → float `1.1`; `country: NO` → boolean `false`; `port: 0755` → octal 493. [dev.to YAML gotchas 2026-05-02](https://dev.to/snappy_tools/yaml-for-developers-syntax-gotchas-and-when-to-use-it-1364) slideforge's indentation-significant DSL must avoid YAML-style implicit typing — quote strings or make all values stringly-typed by default.

3. **Tab vs. space indentation surprises.** YAML rejects tabs; Python permits them with caveats. slideforge ships indentation-significant DSL — must lint hard for mixed indentation and reject tabs unambiguously up-front. [Red Hat YAML tips](https://www.redhat.com/en/blog/yaml-tips)

4. **Multiline strings: literal (`|`) vs folded (`>`) vs plain wrap.** Users get this wrong constantly. Code blocks, speaker notes, multi-paragraph body text all hit this. slideforge needs an obvious, single way to write multi-line text content per slide field.

5. **Inline interpolation collisions with content syntax.** Slidev's `{{ }}` Vue template syntax collides with LaTeX math `{...}`. [#issue Aug 2025](https://github.com/slidevjs/slidev/issues) "{{ inside math causes unclear errors". slideforge variable syntax (when added) must not collide with content tokens.

### Top-3 "graduation problems" (why users leave a tool)

1. **Outgrowing Marp → moving to Reveal.js or Slidev.** "I tried Marp for a few months on some significant projects (50-200 slides) and found it too limiting" — the 2D layout / column ceiling. [Slidev Discussion #86](https://github.com/slidevjs/slidev/discussions/86). Migration is painful: Marp's CSS theme model doesn't map to Slidev's Vue components or Reveal.js's plugin system.

2. **Outgrowing python-pptx → adopting an opinionated DSL on top.** Production teams hit the "no canvas bounds, no chart types, no animations, no PDF, font substitution" wall and build heuristic linters/wrappers. [slideforge blog 2026-04-20](https://slideforge.dev/blog/python-pptx-limitations-we-solved)

3. **Outgrowing MDX-deck → moving to Spectacle/Slidev because mdx-deck went unmaintained.** [#765](https://github.com/jxnblk/mdx-deck/issues/765) — abandonment is itself a graduation pressure. slideforge should bake stability/maintenance signals into its branding (architectural decisions, version-pinning policy).

---

## Implications for slideforge DSL design

(Cross-referenced to seed §5 / slide-types-catalog where possible.)

- **Columns and grids must be first-class slide types**, not CSS escape hatches. The seed's 23-type catalog should include `two-column`, `three-column`, `grid-2x2`, etc. (Counters: Marp #62, #132; Reveal.js no-TOC complaints.)

- **No HTML/CSS escape hatch in v1.** If users need raw HTML, slideforge has failed to provide a sufficient slide type. This is a forcing function on the slide-type catalog. (Counters: every Marp "use `<style scoped>`" workaround.)

- **PPTX export fidelity is a contract, not a best-effort.** slideforge must pre-validate that each DSL construct has a known-good PPTX rendering. If a construct can't render losslessly, it should fail at compile-time, not corrupt at render. (Counters: Pandoc #9465, #9113, #6301, #8565, #7595; Posit forum cross-render pain.)

- **Default theme must be presentation-ready.** Ship a theme that looks like it was designed by someone, not auto-generated. (Counters: Slidev #2128 admission.)

- **Canvas-bounds validator at compile time.** Every slide is rendered to a known canvas (13.33" × 7.5" for 16:9 PPTX). The DSL compiler must check overflow before emitting output, not after. (Counters: python-pptx slideforge blog; Marp `slide-content-overflow` diagnostic was added in marp-vscode.)

- **Indentation rules: spaces only, fixed width, hard-error on tabs.** No "either is fine" — pick spaces, document the width, reject everything else with a clear diagnostic pointing to the offending line. (Counters: YAML pain, Python whitespace pain.)

- **Strings are strings — no implicit type coercion.** A slide title `1.10` is the string "1.10", not the float 1.1. A bullet "NO" is the string "NO", not boolean false. (Counters: YAML 1.1 gotchas.)

- **Slide separator must be unambiguous.** Don't use `---` (collides with frontmatter, CommonMark thematic break, MDX-deck history). Use a named-section header or `=== slide.cover ===` style. (Counters: MDX-deck v2 migration pain.)

- **Variable interpolation syntax must not collide with math/code/LaTeX.** If slideforge adds variables, avoid `{{ }}` (Vue/Slidev collision), avoid `$...$` (LaTeX/Marp/markdown), avoid `<%...%>` (ERB collision). Suggest `@var:name` or `${var}` with strict scoping. (Counters: Slidev `{{` math bug.)

- **`@include` must hot-reload by default.** (Counters: Slidev #2241 "Imported snippets don't reload automatically.")

- **Comment syntax must be obvious and consistent.** Pick one — `#` (YAML/Python), `//` (C-style), or `--` (Haskell/SQL) — and enforce. Don't permit multiple comment styles. (Counters: every config-language confusion.)

- **Single-file decks must work without `npm init`.** A teacher with 200 lectures shouldn't need 200 node_modules folders. The CLI should ship as a single binary or a single Python wheel. (Counters: Slidev #63 teacher complaint.)

- **Mermaid / diagrams as native slide type.** Even if it costs install footprint, this is a top-3 cross-tool request. (Counters: Marp's absence drives users elsewhere; Slidev's presence is praised.)

- **Built-in TOC slide type.** Auto-generated from heading levels. (Counters: Reveal.js, Marp, Quarto all missing this.)

- **Animation primitives like Touying's `pause` / `uncover` / `only` / `alternatives`** — declarative incremental reveal, not jQuery-style event hooks. (Lessons from Touying; counters to Reveal.js fragment complexity.)

- **Clear error messages at compile-time pointing to line+column.** "Slide 4 overflows canvas at body bullet 7" beats "Repair? [Yes/No]" by orders of magnitude. (Counters: python-pptx silent corruption.)

---

## DSL features the seed grammar does NOT support that users keep asking for in competitors

Based on a survey of feature requests across all 9 competitors:

- **Variables / parameterization** — users want `${client_name}`, `${date}` interpolation. Quarto supports via YAML params; Marp/Slidev don't. **Verdict:** consider adding; very low cost, high value. Plan for it but defer to v2.

- **Loops / iteration** — "render one slide per row of this CSV/JSON." Quarto supports via R/Python embedded code; pure-markdown tools don't. **Verdict:** explicit non-goal for v1 — keep DSL declarative. Document as "use templating preprocessor outside slideforge" (Jinja, Mustache).

- **Conditional rendering** — "show this slide only for `audience=execs`." Vue/React-based tools (Slidev, Spectacle) support via `v-if`. Marp does not. **Verdict:** likely non-goal for v1 — tag-based slide filtering at compile time is a simpler win.

- **Data binding** — pulling tables/charts from external data files at compile time. python-pptx + pandas is the common workaround. **Verdict:** consider `@chart:from=data.csv` directive for v2.

- **Internationalization** — Slidev's open i18n issue; no major competitor handles this well. **Verdict:** non-goal for v1; document workaround (separate decks per locale).

- **Right-to-left text (BiDi)** — Slidev open issue; Marp/Reveal.js inconsistent. **Verdict:** flag as known gap; revisit when there's user demand.

- **Real-time collaborative editing** — Reveal.js #3812 feature request; Slidev claims it via Vue/Vite. **Verdict:** out of scope for v1 (a DSL-level tool can't easily provide this).

- **Speaker notes with rich formatting** — Pandoc supports plain text only; Reveal.js supports HTML notes; Marp supports HTML notes. **Verdict:** support markdown formatting in notes from day one.

- **Slide skip / "uncounted" slides** — Quarto has `visibility="uncounted"`; useful for backup slides. **Verdict:** include in v1 catalog.

- **Cross-slide references / "see slide 12"** — no tool handles this well. **Verdict:** v2 candidate via slide IDs.

- **PDF-print mode separate from screen-render mode** — Slidev #issue ignore v-clicks in PDF print. **Verdict:** must support from v1 (PDF is a deliverable format).

---

## Lock-in / portability lessons

| Tool | Source-file portability | Theme portability | Asset portability |
|------|-------------------------|-------------------|-------------------|
| Marp | Mostly portable (CommonMark + Marp directives in HTML comments). Themes are CSS, portable. | CSS themes are reusable in any web context. | Images relative paths; portable. |
| Slidev | Tied to Vue/Vite project structure. Cannot extract `slides.md` and run elsewhere — needs `package.json`, `node_modules`. | Vue components, not portable outside Slidev. | Tied to Vite asset pipeline. |
| Reveal.js | HTML + JS; portable but tightly coupled to reveal's API. | CSS portable. | HTML asset refs portable. |
| Pandoc | Markdown highly portable; pptx output partially editable in PowerPoint. | Reference doc is a `.pptx` — opaque. | Pandoc abstracts asset handling. |
| Quarto | `.qmd` is portable but requires Quarto CLI + Pandoc + (optionally) R/Python. | Themes are Sass; portable to other Quarto projects. | Asset resolution via project config. |
| Touying | `.typ` files are portable but require Typst install. | Theme is Typst module; portable within Typst ecosystem. | Typst handles paths. |
| python-pptx | Python code is portable but coupled to the library API. No DSL. | None — you write boilerplate. | URL fetch not supported, local paths only. |
| MDX-deck | `.mdx` portable in theory; tied to Gatsby v4. Abandoned. | Theme UI components; some portable. | Tied to webpack pipeline. |
| Spectacle | Tied to React + Spectacle component API. | React components only. | webpack-bundled. |

**Implications for slideforge:**
- Make the `.sf` (or chosen extension) file a self-describing artifact — front matter declares grammar version, target output, theme by name.
- `@include` should resolve via documented paths (relative to source file, then to `--include-path` flags).
- Theme should be a stand-alone artifact (own file/folder) — not "embedded CSS inside a comment block." A theme should be portable across decks.
- Document what slideforge will *not* do at file-read time (no shell-out, no network fetch) so users can trust the artifact.
- Provide an "eject to PPTX" or "eject to reveal.js HTML" path so users aren't locked in if slideforge stagnates.

---

## Accessibility lessons

Cross-tool accessibility complaints concentrate on a small set of WCAG criteria:

1. **WCAG 2.2.2 (Pause/Stop/Hide for moving content)** — Auto-advance slideshows must offer pause. [ICT Accessibility Testing PDF](https://www.ictaccessibilitytesting.org/wp-content/uploads/2020/10/The-Unbearable-Inaccessibility-of-Slideshows.pdf). slideforge: don't auto-advance by default; if added, require explicit opt-in and always render keyboard-accessible pause control.

2. **WCAG 2.1.1 (Keyboard accessible)** — Marp disables Ctrl-+ zoom because SVG viewport-scales. [Marp #636, 2025-01-20](https://github.com/marp-team/marp-cli/issues/636). Reveal.js shortcuts work behind overlays. [Reveal.js #3766, 2025-03-20](https://github.com/hakimel/reveal.js/issues/3766). slideforge: HTML output must support browser zoom (no fixed-viewport SVG); PDF/PPTX inherit OS-level zoom.

3. **WCAG 1.1.1 (Non-text content alternatives)** — Image alt text is rarely first-class in DSLs. Reveal.js a11y plugin uses `<figure>/<caption>` workaround. [Reveal.js #3637](https://github.com/hakimel/reveal.js/issues/3637). slideforge: alt text should be a *required* field on image slide types, not optional. Compile-time error if missing.

4. **WCAG 4.1.2 (Name, role, value) / aria-live for slide changes** — Reveal.js wraps each slide in `aria-live`, but image alt text and iframe titles get lost. [#3637](https://github.com/hakimel/reveal.js/issues/3637). slideforge: HTML output should add `aria-label` on each section, `aria-live="polite"` on the active slide container.

5. **WCAG 1.4.3 (Contrast)** — Default themes ship with low-contrast color combos. slideforge: ship a contrast-checker for theme authors; reject themes that fail AA at compile time.

6. **WCAG 2.4.3 (Focus order)** — Reveal.js fragments and overlays disrupt tab order. [#3766](https://github.com/hakimel/reveal.js/issues/3766). slideforge: keep slide-internal interactive elements in document order; speaker-only UI (notes, timer) excluded from focus.

7. **Screen reader landmark issues** — reveal-a11y plugin retrofits `aria-label="Slide N"` on `<section>` elements. [marcysutton/reveal-a11y](https://github.com/marcysutton/reveal-a11y). slideforge: emit landmarks natively.

8. **Right-to-left (BiDi) text** — Slidev open issue, no major tool handles it well. slideforge: at minimum, support `lang` attribute and CSS `direction` on slide content.

9. **Captions for audio/video** — almost no tool ships caption-track UI. Out of scope for v1 but plan the slot.

10. **Alternative content / "skip past slideshow"** — desktop presentation tools don't typically need this, but HTML output should include a skip-link to post-deck content (if embedded).

**Recommendation:** slideforge should ship an `--a11y-check` flag that runs WCAG 2.1 AA validation at compile time. This is differentiating — no major competitor does this.

---

## Sources

(All accessed 2026-05-23.)

### Marp
- https://github.com/orgs/marp-team/discussions/62 — Tables in Marp (2021-01-02)
- https://github.com/orgs/marp-team/discussions/533 — Marp Core v4 changes (2024-09-12)
- https://github.com/marp-team/marp-core/issues/155 — Tables Settings (2020-03-18)
- https://github.com/marp-team/marp-vscode/issues/330 — Table export bug
- https://github.com/marp-team/marp-core/issues/132 — Multi-column (2019-11-18)
- https://github.com/orgs/marp-team/discussions/301 — CLI server renders all md (2022-05-03)
- https://github.com/orgs/marp-team/discussions/209 — Marp architecture opacity (2021-11-08)
- https://github.com/marp-team/marp-cli/issues/636 — Ctrl-+ zoom accessibility (2025-01-20)
- https://github.com/marp-team/marp/blob/main/website/docs/guide/how-to-write-slides.md
- https://github.com/marp-team/marp-vscode — VS Code extension README
- https://medium.com/@lhcamilo/marp-is-nice-but-it-lags-behind-reveal-js-89d92891e5fb (2021-11-10)
- https://news.ycombinator.com/item?id=34501901 — Marp HN thread (2023-01-24)

### Slidev
- https://github.com/slidevjs/slidev/issues — Issues list (2025-08-26)
- https://github.com/slidevjs/slidev/discussions — Discussions (2025-09-07)
- https://github.com/slidevjs/slidev/discussions/86 — Slidev vs Marp (2021-05-11)
- https://github.com/slidevjs/slidev/issues/63 — Global install (2021-05-11)
- https://github.com/slidevjs/slidev/issues/1515 — Project todos (2024-04-11)
- https://github.com/slidevjs/slidev/issues/2128 — Default theme problems (2025-03-22)
- https://github.com/slidevjs/slidev/issues/2066 — Built-in components broken in 51.x (2025-02-16)
- https://github.com/slidevjs/slidev/issues/2060 — Title missing from Frontmatter type
- https://sli.dev/guide/layout — Slidev layouts docs
- https://www.nirtamir.com/articles/advanced-slides-with-slidev

### Reveal.js
- https://github.com/hakimel/reveal.js/issues (2025-08-22 snapshot)
- https://github.com/hakimel/reveal.js/issues/2441 — Section attribute external HTML (2019-06-22)
- https://github.com/hakimel/reveal.js/issues/3637 — Accessibility (2024-06-11)
- https://github.com/hakimel/reveal.js/issues/3766 — Shortcuts in overlay (2025-03-20)
- https://github.com/hakimel/reveal.js/discussions/categories/ideas — Ideas (2025-02-05)
- https://github.com/marcysutton/reveal-a11y — Accessibility plugin
- https://github.com/rstudio/revealjs/issues — RStudio revealjs

### Pandoc (pptx)
- https://github.com/jgm/pandoc/issues/4252 — PowerPoint reader (2018-01-13)
- https://github.com/jgm/pandoc/issues/9465 — pptx math wrapping (2024-02-15)
- https://github.com/jgm/pandoc/issues/9113 — Data URL PNG corruption (2023-09-29)
- https://github.com/jgm/pandoc/issues/4181 — RST title block corrupt pptx (2017-12-20)
- https://github.com/jgm/pandoc/issues/6301 — Outline math in notes (2020-04-19)
- https://github.com/jgm/pandoc/issues/8565 — v3.0 image regression (2023-01-20)
- https://github.com/jgm/pandoc/issues/7595 — pptx two-column missing images (2021-09-27)
- https://github.com/jgm/pandoc/issues/10085 — Template corruption via LibreOffice (2024-08-13)
- https://pandoc.org/MANUAL.html — Pandoc manual
- https://github.com/ickc/pandoc-pptx — Pandoc pptx feature support (2020-06-10)

### Quarto
- https://quarto.org/docs/presentations/revealjs/
- https://quarto.org/docs/presentations/revealjs/advanced.html
- https://quarto.org/docs/presentations/powerpoint.html
- https://quarto.org/docs/reference/formats/presentations/pptx.html
- https://github.com/quarto-dev/quarto-cli/issues/4894 — revealjs Bugs & Improvements epic (2023-03-20)
- https://github.com/quarto-dev/quarto-cli/issues/8383 — Multi-format CSS collision (2024-01-22)
- https://github.com/quarto-dev/quarto-cli/issues/13885 — PowerPoint slide layout issues
- https://forum.posit.co/t/tips-and-tricks-for-presentations-that-are-both-reveal-js-and-powerpoint/174556 (2023-10-04)

### Touying (Typst)
- https://github.com/touying-typ/touying/issues (2025-04-23)
- https://github.com/touying-typ/touying/issues/70 — Wrong title in theme.simple (2024-09-05)
- https://github.com/touying-typ/touying/issues/170 — Title-slide config in Metropolis (2025-04-01)
- https://github.com/touying-typ/touying — Touying README
- https://touying-typ.github.io — Documentation
- https://typst.app/universe/package/touying/ — Touying on Typst Universe
- https://forum.typst.app/t/touying-vertical-space-after-the-slide-title-line/8214 (2026-03-12)
- https://news.ycombinator.com/item?id=45694955 — HN discussion (2025-10-24)

### python-pptx
- https://github.com/scanny/python-pptx/issues/644 — Performance O(N²) (2020-09-01)
- https://github.com/scanny/python-pptx/issues/173 — Open existing pptx (2015-08-12)
- https://github.com/scanny/python-pptx/issues/87 — Corruption from deepcopy (2014-04-16)
- https://github.com/scanny/python-pptx/issues/941 — Generated presentation corrupt (2024-01-26)
- https://stackoverflow.com/questions/59561983/python-pptx-keyerror-no-placeholder-on-this-slide-with-idx-1
- https://slideforge.dev/blog/python-pptx-limitations-we-solved (2026-04-20)
- https://python-pptx.readthedocs.io/en/latest/api/exc.html
- https://www.washstat.org/presentations/20190923/Wilcox-Cook.pdf

### MDX-deck / Spectacle
- https://github.com/jxnblk/mdx-deck/issues/765 — Still maintained? (2020-11-26)
- https://github.com/jxnblk/mdx-deck — Repo
- https://github.com/jxnblk/mdx-deck/blob/master/CHANGELOG.md
- https://github.com/jxnblk/mdx-deck/blob/master/MIGRATION.md
- https://github.com/jonathan-fielding/spectacle — Spectacle (2019-06-23)
- https://www.npmjs.com/package/spectacle-mdx-loader
- https://www.npmjs.com/package/mdx-deck/v/3.0.1

### Comparisons / Cross-tool
- https://dasroot.net/posts/2026/04/markdown-presentation-tools-marp-slidev-reveal-js/ (2026-04-07)
- https://www.pkgpulse.com/guides/slidev-vs-marp-vs-revealjs-code-first-presentations-2026 (2026-05-16)
- https://www.libhunt.com/compare-marp-vs-reveal.js (2026-04-22)
- https://tonai.github.io/blog/posts/slide-libraries/ (2022-01-18)
- https://sharayeh.com/en/blog/marp-alternative-best-markdown-presentation-tools (2026-03-19)
- https://sharayeh.com/en/blog/convert-markdown-to-slides-developer-guide (2026-02-28)
- https://github.com/jgm/pandoc/issues/10483 — Integrate Slidev/Marp output (2024-12-21)

### Accessibility
- https://www.ictaccessibilitytesting.org/wp-content/uploads/2020/10/The-Unbearable-Inaccessibility-of-Slideshows.pdf
- https://equalizedigital.com/accessibility-checker/slider-is-present/ (2021-01-15)
- https://doit.illinois.gov/initiatives/accessibility/guides/captivate.html

### Ergonomics / DSL design
- https://utcc.utoronto.ca/~cks/space/blog/tech/YamlWhitespaceProblem
- https://news.ycombinator.com/item?id=19111313 — YAML whitespace HN (2019-02-08)
- https://www.redhat.com/en/blog/yaml-tips (2019-06-10)
- https://dev.to/snappy_tools/yaml-for-developers-syntax-gotchas-and-when-to-use-it-1364 (2026-05-02)
- https://alexharv074.github.io/2019/11/23/adventures-in-the-terraform-dsl-part-x-templates.html — Terraform template DSL evolution

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_search | 9 | Marp issues, Slidev issues, Reveal.js issues, Pandoc pptx, Quarto, Touying, python-pptx, MDX-deck/Spectacle, accessibility, ergonomics, variables/loops/conditionals, switching/graduation |
| Perplexity perplexity_ask | 0 | — |
| Perplexity perplexity_research | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | (Not used — research was about user pain on existing tools, not API verification) |
| Tavily tavily_search | 3 | Slidev frontmatter, Quarto pptx, python-pptx no-preview |
| Tavily tavily_research | 0 | — |
| Tavily tavily_extract | 0 | — |
| Tavily tavily_crawl | 0 | — |
| Tavily tavily_map | 0 | — |
| WebFetch | 0 | — (Perplexity surfaced full issue text in search results) |
| WebSearch | 0 | — |
| Training data | 1 area | General DSL design principles (variables, comments, multiline) — cross-referenced with verified sources |

**Total MCP tool calls:** 12
**Training data reliance:** low — every concrete claim about a specific tool's bugs, features, or user complaints is sourced to a URL with date. Training data only informed the cross-tool synthesis and DSL-design-principle framing.

**Distinct citations:** 70+ URLs across the Sources section.
