---
document_type: behavioral-contract
level: L3
version: "1.8"
status: active
producer: product-owner
timestamp: 2026-05-29T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-023
lifecycle_status: active
introduced: v1.0.0
modified: ["v1.2 — adversary pass 1 adjudication: codified ShapeSpec position schema, hex color contract, shape_type closed vocabulary, off-canvas boundary semantics, gradient deferral, MissingAlt span, multi-error accumulation", "v1.3 — roundRect added to closed vocabulary per Q7 decision example", "v1.3.1 — STORY-TBD-shape-gradient-fills placeholder resolved to STORY-072", "v1.3.2 — VP propagation burst: assigned VP-037 through VP-042 to all VP-TBD entries", "v1.4 — adversary pass 2 adjudications M/N/O/P/Q/R/T: ArithmeticOverflow Result return, LayoutError::Multiple uniformity, LaidOutDeck warnings field, fill+text fields on ShapeSpec, canonical test vectors, uppercase normalization phrasing, shape frame order enforcement", "v1.4.1 — pass-7 drift fix (F-P7-HIGH-004): slide_index → source_slide_index in EC-001 and EC-003 per AC-BC-A9 canonical field name", "v1.4.2 — pass-8 fix (F-P8-MED-001): Deferred Surfaces section rewritten to be consistent with Postcondition 1 — FillSpec::Gradient is NOT in the v1.0 enum (code confirmed absent); removed contradictory claim that variant is defined in IR", "v1.4.3 — pass-9 fix (F-P9-HIGH-002): E-PAR-013 → E-PAR-015 (hex color invalid) and E-PAR-014 → E-PAR-016 (gradient unsupported) to resolve namespace collision with parser template codes; updated precondition 5, EC-008, EC-009, EC-011, canonical test vectors, and Deferred Surfaces section", "v1.5.0 — pass-18 spec adjudication (F-P18-HIGH-001): added Invariant 11 (alt-wins over decorative when both supplied); updated Precondition 3 wording from exclusive-OR to explicit precedence; added EC-018 and canonical test vector for alt+decorative conflict; updated slideforge-validate handoff note. WCAG canonical: explicit alt text supersedes implicit-decorative inference.", "v1.5.1 — pass-19 prose fix (F-P19-MED-003): Invariant 11 reworded to remove ambiguous 'slideforge-validate W-A11-001' phrase (W-A11-001 is a deprecated warning code, NOT a validator name). Now reads: the slideforge-validate alt-text validator MUST emit W-A11-002 (replacing deprecated W-A11-001 from BC-5.01.002 §3 prior to v1.2).", "v1.5.2 — F-P20-LOW-003 prose precision: Invariant 11 rewritten to clarify that ShapeSpec.decorative is NOT mutated; the alt-wins effect is achieved via the typed ShapeFrame.alt enum (AltText::Provided) at layout resolution time. EC-018 canonical test vector updated to assert on ShapeFrame.alt typed enum instead of ShapeSpec.decorative field mutation. Canonical test vector row for alt+decorative conflict updated to match. STORY-028 AC-BC-A10 updated.", "v1.6 — STORY-086 pass-5 adjudication (F-086-P5-CRIT-001): Invariant 11 domain-scope clarification. Invariant 11 (alt-first) applies ONLY to the shape DSL alt-resolution path (ShapeSpec → ShapeFrame via layout_shapes). It does NOT govern Stage-2b thread_fields_to_blocks::resolve_alt for ChartSpec/ImageSpec/DiagramSpec, which is governed by BC-1.16.001 PC-12 (decorative-first). The two BCs cover non-overlapping implementation domains. W-A11-002 for the shape path is emitted by slideforge-validate; W-A11-002 for Stage-2b is emitted by resolve_alt via tracing::warn!. Domain-scope note added to Invariant 11 body.", "v1.7 — STORY-072 adversary Pass-1 MED-002/OBS-072-P1-001: (MED-002) corrected EC-011 example from brand-token placeholder syntax to correct quoted-hex form (`fill gradient \"#FF0000\" to \"#0000FF\"`); rewrote Deferred Surfaces gradient-fill paragraph to use quoted-hex syntax and added lexer-comment-character note explaining `#` is a line-comment when unquoted; (OBS-001) added shape-pipeline-wiring deferral paragraph to Deferred Surfaces noting pre-existing gap (Stage-2b does not emit ContentBlock::Shape for any shape; STORY-072 scope is parser branch + structural FillSpec::Gradient + 5 exporter renderers verified via constructed-ShapeSpec tests; end-to-end DSL path requires FU-SHAPE-PIPELINE-WIRING).",
"v1.8 — STORY-072 adversary Pass-2 MED-001 + AC-004 HTML correction: No BC postcondition or contract text required change — the BC's Deferred Surfaces section correctly describes STORY-072's deliverables without overstating the ShapeNode→ShapeSpec decode. Version bumped to track that the parallel STORY-072 spec corrections (AC-002 decode-deferral honesty per TD-VSDD-059, and AC-004 HTML SVG-native linearGradient mechanism) were applied in the same burst. The HTML gradient output for STORY-072's exporter AC is SVG-native (<defs><linearGradient> + fill=url(#sf-grad-...) on <rect>) — not CSS background — as the slide canvas uses an SVG graphics layer."]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.04.001: shape: Block Declares Custom Shape with type/position/fill/text/alt

## Description

A `shape:` block inside a slide declaration adds a custom visual shape to that slide.
The shape DSL supports `type` (a closed v1.0 vocabulary of 6 keywords), `position`
(x/y/width/height in inches or em), `fill` (solid hex color — gradient deferred),
`text` (inline content), and required `alt` (accessibility description). The shape
is rendered as an explicit OOXML `<p:sp>` element positioned using integer EMU
coordinates. ALL errors from a slide's shape set are accumulated before returning
(multi-error, per DI-018).

## Preconditions

1. The `shape:` block is nested inside a `slide <type>:` block.
2. The `shape:` block declares `type:` with a keyword from the closed v1.0 vocabulary:
   `rect`, `ellipse`, `arrow`, `line`, `star`, `roundRect`. Any other keyword
   produces E-PAR-012 (unknown shape type; hard error).
3. The `shape:` block declares `alt "..."` OR `decorative: true` (or both — see Invariant 11 for
   conflict resolution). A shape with neither is a hard error (E-A11-001 / `LayoutError::MissingAlt`).
4. `position:` specifies x, y, width, height as numbers with units (in or em).
5. Hex fill color, if supplied, is exactly 6 uppercase or lowercase hexadecimal
   digits preceded by `#` (i.e., `#RRGGBB` or `#rrggbb`). Case-insensitive;
   both forms accepted. Short form `#RGB` and alpha form `#RRGGBBAA` are rejected.

## Postconditions

1. The shape is added to the slide's `Deck` IR node via a `ShapeSpec` that carries:
   ```rust
   pub struct ShapeSpec {
       pub shape_type: ShapeType,   // resolved enum variant — NOT Arc<str> (see Item P note)
       pub position: ShapePosition, // x/y/width/height in ShapeUnit (milliinches or milliem)
       pub fill: FillSpec,          // SolidColor(Rgb) or None; Gradient deferred to STORY-072
       pub text: Option<Vec<InlineNode>>,
       pub alt: Option<AltText>,
       pub decorative: bool,
       pub span: SourceSpan,
   }
   ```
   with supporting types:
   ```rust
   pub struct ShapePosition {
       pub x: ShapeUnit,
       pub y: ShapeUnit,
       pub width: ShapeUnit,
       pub height: ShapeUnit,
   }

   pub enum ShapeUnit {
       /// inches × 1000 stored as i64 (avoids f64)
       /// e.g., 0.5in → Inches(500)
       Inches(i64),
       /// em × 1000 stored as i64
       /// e.g., 2em → Em(2000)
       Em(i64),
   }

   pub enum ShapeType {
       Rect,
       Ellipse,
       Arrow,
       Line,
       Star,
       RoundRect,
       // NOTE: no Custom variant in v1.0 — unknown keyword is a parse error
   }

   // Bridge function required until the DSL parser is implemented (future story).
   // All ShapeSpec construction sites (tests and parser) MUST call this to lift
   // Arc<str> keywords to the resolved enum.  The IR MUST NOT store Arc<str>.
   // See interface-definitions.md §9 for the full contract.
   impl ShapeType {
       pub fn from_keyword(kw: &str) -> Result<ShapeType, ParseError>; // returns E-PAR-012 on unknown keyword
   }

   pub enum FillSpec {
       SolidColor(Rgb),
       None,
       // Gradient variant: DEFERRED to STORY-072 (see Deferred Surfaces section)
   }
   ```
   All types derive `Debug + Clone + PartialEq + Eq + Hash` for comemo compatibility
   (DI-010, CLAUDE.md hash+eq+clone rule).

2. The layout stage converts `ShapeSpec.position` fields to integer EMU at layout
   time using the following exact constants:

   | User Unit | Conversion Formula | Example |
   |-----------|-------------------|---------|
   | `1in` | `inches_milliinches * 914_400 / 1_000` | `0.5in → Emu(457_200)` |
   | `1em` | `em_milliems * brand_font_size_emu / 1_000` | `1em → Emu(457_200)` (default 36pt) |
   | `1cm` | `cm_millicm * 360_000 / 1_000` | `1cm → Emu(360_000)` |
   | `1mm` | `mm_millimm * 36_000 / 1_000` | `1mm → Emu(36_000)` |
   | `1pt` | `pt_millipt * 12_700 / 1_000` | `1pt → Emu(12_700)` |
   | `1px` | `px_millipx * 9_525 / 1_000` | `1px → Emu(9_525)` (96 dpi screen) |

   Note: `cm`, `mm`, `pt`, `px` are v1.0 supported units. The division is integer
   division (no `f64`). All intermediate values are `i64`.

3. The shape is produced in the `LaidOutDeck` as a `Frame` with
   `FrameContent::Shape(ShapeFrame { ... })` appended after all placeholder
   frames, in source order. Enforceable invariant: the 0-based index of the
   first `FrameContent::Shape` in `slide.frames` MUST be greater than or equal
   to the slide's placeholder region count (`region_count`). The canonical test
   vector for this: a `title_content` slide with 1 region and 1 `shape:` block
   must produce `frames[0]` = title placeholder, `frames[1]` = shape frame
   (i.e., `shape_frame_index >= 1`). See EC-013.
4. The shape is rendered into the .pptx as `<p:sp>` with
   `<p:cNvPr name="<alt text>">` (semantic name from alt text per accessibility rule).
5. The shape appears in all output formats that support positioned shapes
   (PPTX, PDF, HTML).
6. ALL `MissingAlt` errors for a slide are accumulated in a `Vec<LayoutError>`
   before returning — the function does NOT bail on the first missing-alt shape.
   The caller receives all errors at once (per DI-018).

   **LayoutError::Multiple uniformity (Item N):** `LayoutError::Multiple` is the
   ALWAYS-used wrapper for accumulated errors — even when only one error is
   collected. A single-error case MUST return `Multiple { inner: vec![one_error] }`,
   NOT the unwrapped inner variant. Callers pattern-match exclusively on `Multiple`
   for the multi-error path; the asymmetric "single error = unwrapped, multiple =
   wrapped" API is explicitly FORBIDDEN. The smart constructor
   `LayoutError::multiple(errors: Vec<Self>) -> Self` MUST be used at all call
   sites (see interface-definitions.md §9 and EC-010). The constructor MUST:
   - panic (debug_assert!) if `errors` is empty
   - flatten nested `Multiple` variants (no `Multiple { inner: [Multiple { ... }] }`)

7. Layout warnings (`XrefTargetNotFound` from `run_inline_validation` and
   `OffCanvas` from `layout_shapes`) are accumulated in `LaidOutDeck.warnings:
   Vec<LayoutWarning>` (Item O). The layout runner (`layout::run`) MUST NOT drop
   either warning type. The `LaidOutDeck` type MUST include the `warnings` field
   as part of its public schema. Exporters that consume `LaidOutDeck` MAY choose
   to surface or suppress individual warning types; the field is always populated
   by the layout runner.

## Invariants

1. `alt` or `decorative: true` is REQUIRED on every shape. A shape without either is a
   compile error (E-A11-001 at parse/validation; `LayoutError::MissingAlt` at layout
   as a defensive check). (DI-001)
2. Position values are converted to integer EMU at layout time; no `f64` coordinates
   anywhere in the IR. (DI-010, CLAUDE.md)
3. `raw` keyword is NOT available in user .sf files — the shape DSL is the only way
   to add custom shapes. (DI-021, BC-3.04.002)
4. The shape type vocabulary is closed in v1.0: `rect`, `ellipse`, `arrow`, `line`,
   `star`, `roundRect`. Unknown keywords produce E-PAR-012 (no silent
   `ShapeType::Custom(...)` fallback). (CLAUDE.md "no silent fallback" canonical rule)
5. Hex fill colors are case-insensitive at parse time (`#FF6F00` and `#ff6f00` both
   accepted). Both case forms are parsed and the canonical IR representation is the
   numeric `Rgb { r: u8, g: u8, b: u8 }` struct where each byte is the decoded
   channel value. "Uppercase" or "lowercase" is meaningless at the IR level because
   `Rgb` stores integers, not strings — there is NO string round-trip.
   Short-form `#RGB` and alpha form `#RRGGBBAA` are rejected with E-PAR-015.
6. `x + width == page_width` (or `y + height == page_height`) is ON-canvas (inclusive
   boundary). Only strict `> page_width` / `> page_height` triggers off-canvas.
7. `LayoutError::MissingAlt` carries a `span: SourceSpan` field pointing to the
   offending `shape:` block in the source file (per CLAUDE.md error-handling rule:
   all errors carry source spans).
8. **ArithmeticOverflow is a hard error (Item M).** `ShapeUnit::from_inches` and
   `ShapeUnit::from_em` MUST use `checked_mul` for the EMU conversion. On overflow,
   they MUST return `Err(LayoutError::ArithmeticOverflow { source_slide_index, span })`,
   which is then wrapped in `LayoutError::Multiple` at the accumulation site and
   reported as E-LAY-006. Silent saturation (`saturating_mul` without error return) is
   FORBIDDEN — it produces garbage coordinates that propagate to exporters undetected.
   (CLAUDE.md Rule 1: no silent fallback; Rule 4: no rationalized deferrals)
9. **`ShapeSpec.shape_type` stores `ShapeType` (resolved enum), NOT `Arc<str>`.** The
   DSL parser (future story) and all test construction sites MUST call
   `ShapeType::from_keyword(kw)` to lift string keywords to enum variants before
   constructing `ShapeSpec`. No code path may store a raw string keyword in
   `shape_type`. (BC-3.04.001 postcondition 1; source-of-truth precedence Rule 7)
10. **`ShapeSpec.fill` and `ShapeSpec.text` are required fields in v1.0.** The `fill`
    field is `FillSpec` (not `Option<FillSpec>`) — absence of a `fill:` directive means
    `FillSpec::None`. The `text` field is `Option<Vec<InlineNode>>` — absence of
    a `text:` directive means `None`. Both fields MUST be present in the struct
    definition. (CLAUDE.md Rule 1: "shape: type/position/fill/text/alt" is the v1.0
    shape contract; Item Q adjudication)
11. **Domain scope: This invariant governs the `shape:` block alt-resolution path ONLY
    (`layout_shapes` → `ShapeFrame` construction in `slideforge-layout`). It does NOT
    apply to Stage-2b `thread_fields_to_blocks::resolve_alt` for ChartSpec/ImageSpec/
    DiagramSpec, which is governed exclusively by BC-1.16.001 PC-12 (decorative-first).**
    The two BCs cover non-overlapping implementation domains and must NOT be cross-applied.
    **Within the shape DSL path:** When both `alt "..."` and `decorative: true` are supplied
    on the same `shape:` block, `alt` takes precedence. The resulting `ShapeFrame.alt`
    carries `AltText::Provided(s)` regardless of the input `ShapeSpec.decorative` value.
    The IR-level `ShapeSpec.decorative` field is preserved (not mutated); the alt-wins
    effect is achieved via the typed `ShapeFrame.alt` enum at layout resolution time.
    Rationale: explicit accessibility annotations always supersede implicit-decorative
    inference per WCAG AA in the shape DSL domain. An author supplying both has provided a
    textual description that MUST be preserved — silently discarding it would be an
    accessibility regression. The `slideforge-validate` `alt-text` validator MUST emit
    W-A11-002 — "shape has both alt and decorative: true; alt takes precedence, decorative
    flag ignored. Consider removing one." — to prompt the author to clean up the ambiguity,
    but MUST NOT suppress the alt text. (Note: for Stage-2b media blocks, W-A11-002 is
    emitted by `resolve_alt` via `tracing::warn!(code = "W-A11-002")` at resolution time,
    NOT by `slideforge-validate`, because `AltTextValidator::validate()` is Shape-only
    per ADR-018 v1.2 Decision-3.)
    (F-P18-HIGH-001 adjudication, 2026-05-29; F-P19-MED-003 prose fix, 2026-05-29; F-P20-LOW-003 precision fix, 2026-05-29; F-086-P5-CRIT-001 domain-scope clarification, 2026-06-06; WCAG 2.1 §1.1.1)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | shape: without alt and without decorative: true | `LayoutError::MissingAlt { source_slide_index, span }` (layout defensive check); E-A11-001 at validation |
| EC-002 | shape: with position where x + width == page_width | ON-canvas (inclusive boundary). No off-canvas warning. Canonical test vector: x=8in, width=2in on 10in canvas → no warning. |
| EC-003 | shape: with position where x + width > page_width by 1 EMU | `LayoutWarning::OffCanvas { source_slide_index, shape_type, x_emu, y_emu }`. Shape frame still produced at declared position. |
| EC-004 | shape: with negative x or y | `LayoutWarning::OffCanvas`. Shape frame produced. |
| EC-005 | shape: with unknown type keyword (e.g., `type frobnicator`) | E-PAR-012: "Unknown shape type 'frobnicator' at `<file>:<line>:<col>`. Known types: [rect, ellipse, arrow, line, star, roundRect]" |
| EC-006 | shape: with fill "#FF6F00" (uppercase) | Accepted; normalized to `Rgb { r: 255, g: 111, b: 0 }` |
| EC-007 | shape: with fill "#ff6f00" (lowercase) | Accepted; normalized to same `Rgb { r: 255, g: 111, b: 0 }` |
| EC-008 | shape: with fill "#F60" (short-form #RGB) | E-PAR-015: "Invalid hex color '#F60' at `<file>:<line>:<col>`. Expected 6-digit hex (#RRGGBB). Short-form #RGB is not supported." |
| EC-009 | shape: with fill "#FF6F00FF" (alpha #RRGGBBAA) | E-PAR-015: "Invalid hex color '#FF6F00FF' at `<file>:<line>:<col>`. 8-digit alpha hex not supported. Use fill-opacity: attribute for transparency." |
| EC-010 | Slide with two shapes where both lack alt | Both `MissingAlt` errors accumulated; function returns `Err(LayoutError::Multiple { inner: vec![MissingAlt{...}, MissingAlt{...}] })`. ALSO: a slide with ONE shape lacking alt returns `Err(LayoutError::Multiple { inner: vec![MissingAlt{...}] })` — uniform wrapper even for single errors (Item N). |
| EC-011 | shape: with gradient fill before STORY-072 ships (e.g., `fill gradient "#FF0000" to "#0000FF"` — correct quoted syntax) | E-PAR-016: "Shape gradient fill is not supported in v1.0. Use a solid hex color or 'none'. Gradient fill is planned for a future release (STORY-072)." Note: hex values MUST be quoted strings; `#` is a line-comment character when unquoted. After STORY-072 ships, E-PAR-016 is removed and this syntax is accepted. |
| EC-012 | shape: in DOCX export | Shape rendered as floating `<w:drawing>` inline image (SVG rasterized at 150 DPI) |
| EC-013 | `title_content` slide with 1 placeholder region and 1 `shape:` block | `frames[0]` = placeholder frame, `frames[1]` = shape frame; `shape_frame_index (1) >= region_count (1)` — shape is appended AFTER all placeholder frames. |
| EC-014 | `ShapeUnit::Inches(i64::MAX)` passed to EMU conversion | `layout_shapes` returns `Err(LayoutError::Multiple { inner: [LayoutError::ArithmeticOverflow { source_slide_index, span }] })`. E-LAY-006 is reported with source span. Saturating-silent behavior is FORBIDDEN. |
| EC-015 | `ShapeUnit::Em(i64::MAX)` passed to EMU conversion | Same as EC-014: `LayoutError::ArithmeticOverflow`; E-LAY-006. |
| EC-016 | Slide with one `Xref("missing")` shape text field | `LaidOutDeck.warnings` contains `LayoutWarning::XrefTargetNotFound { target: "missing", source_slide_index: 0 }`; shape frame IS produced; exit 0 (warning, not error). |
| EC-017 | Shape positioned at x=11in on a 10in canvas | `LaidOutDeck.warnings` contains `LayoutWarning::OffCanvas { source_slide_index, shape_type, x_emu, y_emu }`; shape frame IS produced at declared position. |
| EC-018 | `shape: type rect alt "Blue rect" decorative: true fill "#003766"` (both alt AND decorative supplied) | `ShapeFrame { alt: AltText::Provided("Blue rect"), ... }` — alt wins via typed enum at layout resolution; `ShapeSpec.decorative` input field is preserved (not mutated). slideforge-validate emits W-A11-002 lint warning. PPTX `<p:sp>` carries `descr="Blue rect"`, NOT empty descr. Exit 0. (Invariant 11; F-P18-HIGH-001; F-P20-LOW-003) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `shape: type rect position x 0.5in y 1.0in width 2.0in height 1.0in fill "#003766" alt "Blue rectangle highlight"` | `BoundingBox { x: Emu(457_200), y: Emu(914_400), width: Emu(1_828_800), height: Emu(914_400) }` in PPTX `<p:sp>`; exit 0 | happy-path |
| `shape: type rect` (no alt, no decorative) | `Err(LayoutError::Multiple { inner: vec![LayoutError::MissingAlt { source_slide_index: 0, span: <span> }] })` — single error, uniform Multiple wrapper | error |
| `shape: type ellipse decorative: true fill "#FF6F00"` | `AltText::Decorative`; PPTX `<p:sp>` with empty `descr=""` and PDF Artifact tag; exit 0 | edge-case |
| `shape: type frobnicator alt "x"` | E-PAR-012 with known-types list; exit 1 | error |
| `shape: type rect position x 8.0in y 0in width 2.0in height 1.0in alt "edge"` (x+w == page_width == 10in) | ON-canvas; no `OffCanvas` warning | boundary |
| `shape: type rect fill "#ff6f00" alt "orange"` (lowercase hex) | Accepted; `ShapeSpec { fill: FillSpec::SolidColor(Rgb { r: 255, g: 111, b: 0 }), ... }` — numeric Rgb, no string preserved | case-insensitive |
| `shape: type rect fill "#F60" alt "x"` (short-form) | E-PAR-015; exit 1 | error |
| Slide with 2 shapes both missing alt | `Err(LayoutError::Multiple { inner: vec![MissingAlt{source_slide_index:0,...}, MissingAlt{source_slide_index:0,...}] })` — 2 entries | multi-error |
| `shape: type rect position x 1in y 1in width 2in height 1in fill "#003766" alt "Blue rect"` | `ShapeSpec { shape_type: ShapeType::Rect, position: ShapePosition { x: Inches(1000), y: Inches(1000), width: Inches(2000), height: Inches(1000) }, fill: FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 }), text: None, alt: Some(AltText::Provided("Blue rect")), decorative: false, span: <span> }` | fill/text contract (Item Q) |
| `shape: type rect position x 1in y 1in width 2in height 1in alt "box" text "Hello world"` | `ShapeSpec { ..., text: Some(vec![InlineNode::Plain(Arc::from("Hello world"))]), ... }` | text field contract (Item Q) |
| `title_content` slide with 1 placeholder + 1 shape block | `frames[0]` = placeholder frame (`region_count = 1`), `frames[1]` = shape frame; `frames[1].index (1) >= region_count (1)` — shape appended after placeholders | frame-order (Item T) |
| `ShapeUnit::Inches(i64::MAX)` in position | `Err(LayoutError::Multiple { inner: [LayoutError::ArithmeticOverflow { source_slide_index: 0, span: <span> }] })` | arithmetic overflow (Item M) |
| `shape: type rect alt "Blue rect" decorative: true fill "#003766"` | `ShapeFrame { alt: AltText::Provided("Blue rect"), fill: FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 }), ... }` — alt wins via typed enum at layout resolution; `ShapeSpec.decorative` input preserved (not mutated); W-A11-002 lint emitted; exit 0 | alt+decorative conflict (Invariant 11, EC-018; F-P20-LOW-003) |

## Deferred Surfaces

The following surfaces are explicitly deferred to future stories. They are NOT silently
missing — they are planned features with spec boundaries:

**Gradient fills** (`fill gradient "#RRGGBB" to "#RRGGBB"`): The `FillSpec::Gradient { from: Rgb, to: Rgb }`
variant is NOT in the v1.0 `FillSpec` enum (see Postcondition 1). The enum contains only
`SolidColor(Rgb)` and `None` in v1.0. Any attempt to use gradient syntax at the DSL level
produces E-PAR-016 ("Shape gradient fill is not supported in v1.0.").
Note on syntax: hex values MUST be quoted strings — the lexer treats `#` as a line-comment
character when unquoted, so `fill gradient #FF0000 to #0000FF` is NOT parseable; the correct
form is `fill gradient "#FF0000" to "#0000FF"`.
Target story: **STORY-072** (`STORY-072-shape-gradient-fills.md`).
When that story ships, it will: (1) add `FillSpec::Gradient { from: Rgb, to: Rgb }` to the
enum in `slideforge-types`, (2) add gradient parsing to the DSL parser with quoted-hex syntax,
(3) remove E-PAR-016, (4) implement `FillSpec::Gradient` handling in PPTX/PDF/HTML exporters.

**Shape-pipeline end-to-end wiring** (pre-existing gap, OBS-072-P1-001): Even after STORY-072
ships the parser branch, structural `FillSpec::Gradient`, and all 5 exporter renderers, the
full DSL→output path for ANY shape block (not just gradient) is not wired in production:
Stage-2b (`thread_fields_to_blocks`) does not yet emit `ContentBlock::Shape` (see BC-1.16.001
inv-4). STORY-072's exporter rendering is verified via directly-constructed `ShapeSpec` unit
tests. The end-to-end DSL→output pipeline for shapes requires a separate shape-pipeline-wiring
story (filed as FU-SHAPE-PIPELINE-WIRING; route to wave-gate).

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-037 | Shape position in PPTX XML matches EMU conversion of declared user-unit values | unit test + Kani (known input → known EMU assertion; pure integer arithmetic) |
| VP-038 | Shape without alt or decorative always produces LayoutError::MissingAlt | unit test |
| VP-039 | Unknown shape type keyword produces E-PAR-012 (no silent Custom fallback) | unit test |
| VP-040 | Hex color case-insensitive: lowercase = uppercase for all 6-digit forms; Rgb stores integer bytes, not strings | unit test + Kani |
| VP-041 | x + width == page_width is NOT off-canvas; x + width == page_width + 1 EMU IS off-canvas | unit test + Kani (boundary; pure integer comparison) |
| VP-042 | Slide with N shapes all missing alt returns Multiple with N MissingAlt entries (N=1 also returns Multiple) | unit test (multi-error accumulation + uniformity) |
| VP-048 | from_inches / from_em with i64::MAX input returns Err(LayoutError::ArithmeticOverflow) — NOT silent saturation | unit test + Kani (checked_mul path; pure integer arithmetic) |
| VP-049 | LaidOutDeck.warnings is populated with XrefTargetNotFound and OffCanvas warnings from layout::run | unit test (warnings field not dropped) |
| VP-050 | Shape frames in LaidOutDeck.frames appear at index >= region_count (after all placeholder frames) | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-023 ("Structured Shape DSL") per capabilities.md §CAP-023 |
| Capability Anchor Justification | CAP-023 ("Structured Shape DSL") per capabilities.md §CAP-023 — "shape: blocks with type, position, size, fill, text, and required alt text" is the verbatim description of CAP-023 |
| L2 Domain Invariants | DI-001 (alt text required on visual elements), DI-010 (integer EMU for all coordinates), DI-018 (error accumulation), DI-021 (raw keyword rejected) |
| Architecture Module | slideforge-layout crate — shape block parsing and EMU conversion; slideforge-pptx crate — shape to sp element |
| Stories | STORY-028, STORY-072, STORY-074 |
| Schema Delegation | `ShapeSpec` position fields (`ShapePosition`, `ShapeUnit`) are added to `slideforge-types/src/specs.rs` by data-engineer per orchestrator dispatch. BC defines the contract; implementation is in data-engineer scope. |

## Related BCs

- BC-3.04.002 — depends on (raw keyword rejection is the complementary contract to shape DSL availability)
- BC-5.01.001 — depends on (alt text enforcement applies to shape: blocks)

## Architecture Anchors

- `architecture/crate-architecture.md` — shape block parsing and EMU conversion

## Story Anchor

STORY-028

## VP Anchors

VP-037, VP-038, VP-039, VP-040, VP-041, VP-042, VP-048, VP-049, VP-050
