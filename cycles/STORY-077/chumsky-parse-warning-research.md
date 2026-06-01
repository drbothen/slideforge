# Research: chumsky Parse-Time Warning Capability — BC-3.02.002 Invariant 4 (DIR-077-001 Follow-up)

| Field | Value |
|-------|-------|
| Research type | general (technology / parser capability) |
| Date | 2026-06-01 |
| Trigger | DIR-077-001 §4 proposed amending BC-3.02.002 invariant 4 from "parse time" to "Evaluate stage" |
| chumsky version (verified) | **0.10.1** — `Cargo.toml:48` pins `chumsky = "=0.10.1"`; `Cargo.lock:355-357` confirms resolved `0.10.1` |
| Verdict | **Invariant 4's "parse time" placement is ACHIEVABLE and CORRECT for the sub-block KEY check. The architect CONFLATED two distinct checks. The amendment is UNNECESSARY for invariant 4 and should be WITHDRAWN (or narrowed to apply only to the TYPE check, which is not what invariant 4 governs).** |

---

## Executive Summary

The human's suspicion is **correct on all counts**, and is corroborated by three independent lines of evidence:

1. **chumsky 0.10.1 explicitly supports non-fatal diagnostic emission during parsing** via the `validate()` combinator + `Emitter::emit()`. The official docs state emitted errors are "non-terminal" — they "do not immediately halt parsing" and the parser can still return `Ok(output)`. This is exactly a recoverable warning channel.

2. **The slideforge parser ALREADY uses this exact pattern** to validate identifiers against fixed/known sets at parse time without aborting — e.g. E-PAR-007 (unknown slide type in `set` rules) and E-PAR-008 (var-name collision) both call `emitter.emit(Rich::custom(span, msg))` inside `.validate()` in `parser/deck.rs`. The infrastructure for a parse-time warning is not hypothetical; it is in production code today.

3. **The project already has a first-class `ParseSeverity::Warning`** (`error.rs:41-48`), a `DiagnosticSink::push_with_severity` warning channel (`sink.rs:142-156`), a `ParseResult::warnings: Vec<SyntaxError>` return path (`parser/mod.rs:60-65`), AND a working precedent for a **parse-time non-fatal warning**: the missing-`slideforge_version` advisory (`VersionError { is_fatal: false }`), which is emitted at parse time, carries `ParseSeverity::Warning`, and lets the parse succeed (`parser/mod.rs:293-305`).

The architect's rationale ("Moving the check to the parser requires the parser to know the section type registry — a coupling that makes the parser depend on plugin registration, violating the pure-core boundary") is **true for the section-TYPE check (invariant 3) but false for the sub-block-KEY check (invariant 4)**. The two checks were conflated. Per DIR-077-001 §5 itself, the recognized register sub-block vocabulary is the FIXED set `["report", "detail"]` — a compile-time constant, NOT plugin-registered. Validating against a fixed constant requires zero registry coupling and is the same shape as the already-shipped E-PAR-008 keyword check.

---

## Research Question 1 — Does chumsky 0.10+ support non-fatal diagnostics during parsing?

**Answer: YES. This is a first-class, documented feature.**

chumsky 0.10's `Parser::validate` combinator runs a closure with signature:

```rust
fn validate<U, F>(self, f: F) -> Validate<Self, O, F>
where F: Fn(O, &mut MapExtra<'src, '_, I, E>, &mut Emitter<E::Error>) -> U
```

(Source: docs.rs `chumsky::combinator` / `Parser::validate`, via Context7 `/websites/rs_chumsky_chumsky`.)

The `Emitter<E::Error>` is the recoverable diagnostic channel. The official docs are explicit about its non-fatal semantics:

> "Validate an output, producing **non-terminal errors** if it does not fulfill certain criteria. The errors **will not immediately halt parsing** on this path, but instead it will continue, potentially emitting one or more other errors, only failing after the pattern has otherwise successfully [parsed], or emitted another terminal error."
> — docs.rs `chumsky` `Parser::validate` (retrieved 2026-06-01 via Context7)

The canonical docs example demonstrates `emitter.emit(...)` while the parser still yields a value — note `parse("537")` returns `Ok(537)` even though the *same parser* would emit on smaller inputs:

```rust
let large_int = text::int::<_, extra::Err<Rich<char>>>(10)
    .from_str().unwrapped()
    .validate(|x: u32, e, emitter| {
        if x < 256 { emitter.emit(Rich::custom(e.span(), format!("{} must be 256 or higher.", x))) }
        x
    });
assert_eq!(large_int.parse("537").into_result(), Ok(537)); // emit did NOT abort
```

**Idiomatic mechanism in chumsky 0.10:** `parser.validate(|output, extra, emitter| { ... emitter.emit(Rich::custom(extra.span(), msg)); output })`. The emitted `Rich` carries a span. There is no separate "severity tag" on `Rich` itself — chumsky treats all emitted items uniformly as diagnostics and lets the host project assign severity downstream (which is exactly what slideforge does at the `Rich → SyntaxError` conversion boundary in `parser/mod.rs:191-282`, and via `DiagnosticSink::push_with_severity`).

`recover_with(...)` is a *different* mechanism (it recovers from a hard parse failure by skipping tokens). For a warning where the construct is syntactically well-formed but semantically dubious (the unrecognized-key case — it parses fine as `IDENT value` / `IDENT: block`, it's just an unknown key name), `validate()`+`emit` is the correct tool, not `recover_with`. The slideforge codebase uses both: `recover_with(skip_then_retry_until(...))` for malformed structure (`deck.rs:515-520`) and `validate`+`emit` for semantic checks on well-formed structure (`deck.rs:244-273`, `298-334`).

---

## Research Question 2 — Can a chumsky parser validate a parsed identifier against a FIXED known-set at parse time and attach a warning-severity span diagnostic without aborting?

**Answer: YES, and slideforge already does precisely this.**

The shipped precedents in `crates/slideforge-syntax/src/parser/deck.rs`:

- **E-PAR-007 (unknown slide type in `set` rule), `set_rule_parser`, lines 298-334.** The parsed type identifier is checked against the fixed/known field registry (`known_fields(&slide_type).is_none()`), and on mismatch it calls `emitter.emit(Rich::custom(info.span(), "E-PAR-007: Unknown slide type '...'. Did you mean '...'?"))`. This is a parse-time validation of an identifier against a known set, with a span and a correction hint — exactly the shape invariant 4 needs.

- **E-PAR-008 (var-name collision), `vars_block_parser`, lines 240-273.** Each `vars:` entry name is checked against `is_slide_type_keyword(&name)` / `classify_keyword(&name)` (fixed compile-time sets) and emits `Rich::custom` on collision while continuing the parse.

These confirm the mechanism is not just theoretically available in chumsky 0.10 — it is the established slideforge idiom for "well-formed token, semantically checked against a fixed vocabulary, diagnostic carries a span, parse continues."

The only delta for invariant 4 is **severity**: E-PAR-007/008 are fatal errors, whereas the unrecognized-sub-block-key diagnostic is a non-fatal *warning*. The project already supports this distinction end-to-end:

- `ParseSeverity::Warning` exists (`error.rs:41-48`).
- `SyntaxError::severity()` already returns `Warning` for the non-fatal `VersionError { is_fatal: false }` case (`error.rs:493-500`), proving a parse-produced `SyntaxError` can be a warning.
- `ParseResult::warnings` is the documented non-fatal return path (`parser/mod.rs:60-65`), and `parse()` already routes a parse-time warning through it (the missing-version advisory, `parser/mod.rs:293-305`).
- `DiagnosticSink::push_with_severity(err, ParseSeverity::Warning)` (`sink.rs:142-156`) is the explicit warning-push API, with a passing test `test_push_with_severity_stores_explicit_severity` (`sink.rs:583-599`).

So invariant 4's "parse time" is not only achievable — every piece of plumbing already exists. What's missing is just a new warning-carrying diagnostic variant (or reuse of the `Rich::custom` → warning conversion) wired into the STORY-078 `section.rs` sub-block parser.

---

## Research Question 3 — Is the distinction (TYPE check = eval/runtime; KEY check = parse time) sound?

**Answer: YES, the distinction is sound, and it is the crux the architect missed.**

### Section TYPE check (invariant 3) — genuinely runtime/eval

`methodology`, `scope`, `approval`, `appendix`, `glossary`, **plus any type registered via the `SectionType` plugin trait** (BC-3.02.002 precondition 2; DIR-077-001 §3.2). Because the valid set is open and plugin-extensible, a pure parser cannot know the full set without depending on the plugin registry. The architect's "pure-core boundary" argument is **correct here**.

> Caveat / inconclusive sub-point: DIR-077-001 §3.2 itself instructs the STORY-078 parser to validate the section TYPE at parse time against the *built-in* list and emit an E-PAR-007-class fatal error (and §7 AC-003 mandates it). That is internally inconsistent with §4's "parser must not know the registry" rationale: the parser can check built-ins at parse time but cannot see plugin-registered types, so a plugin-registered section type would be falsely rejected at parse time. **This TYPE-check placement is a separate open question** that this research flags but does not resolve — it is the architect's domain. The cleanest resolution is: TYPE validation belongs at eval (where the full registry is visible); the parser stores the type name verbatim. But that is about invariant 3, not invariant 4.

### Section sub-block KEY check (invariant 4) — correctly parse time

The recognized register sub-block keys are the **FIXED** set. Per DIR-077-001 §5 (the architect's own ruling), `notes` is removed, leaving `REGISTER_SUB_BLOCK_KEYS = ["report", "detail"]` — two compile-time string constants. This vocabulary:

- does NOT depend on the `SectionType` plugin registry,
- does NOT depend on which section type encloses the sub-block (`report`/`detail` are the universal document-mode registers; q1-decision-final.md),
- is identical in nature to the slide-type keyword set already checked at parse time for E-PAR-008.

Therefore checking a parsed sub-block key against `["report", "detail"]` at parse time introduces **zero** registry coupling and **zero** pure-core violation. The architect's stated rationale for moving invariant 4 to eval ("requires the parser to know the section type registry") **does not apply to invariant 4 at all** — it describes invariant 3. This is the conflation the human suspected.

**Conclusion:** TYPE check (invariant 3) → eval (or, at most, parse-time built-in pre-screen with eval as authority). KEY check (invariant 4) → parse time is correct, achievable, and consistent with existing slideforge parser conventions.

---

## Research Question 4 — Recommended pattern for threading a warning collector through the chumsky 0.10 parser

There are two viable, idiomatic chumsky-0.10 patterns. **Pattern A is recommended** because it matches the slideforge codebase's existing convention exactly and requires no change to the `extra` type of the whole grammar.

### Pattern A (RECOMMENDED) — `validate()` + `emitter.emit(Rich::custom(...))`, classified at the conversion boundary

This is the established slideforge idiom (E-PAR-007/008). In the new `section_block_parser` (STORY-078, `parser/section.rs`), parse each sub-block key as a normal `IDENT`, then in a `.validate()` closure:

```rust
// Pseudocode mirroring deck.rs:244-273 conventions
.validate(move |((key, key_span), value), info, emitter| {
    if !is_register_sub_block_key(&key) {          // fixed set ["report","detail"]
        if is_reserved_register_collision(&key) {  // EC-006 case
            emitter.emit(Rich::custom(info.span(),
                format!("E-PAR-???: Key '{key}' is a reserved register name — \
                         use '{key}:' register syntax or choose a different key.")));
        } else {
            // Invariant 4 — NON-FATAL warning, prefixed so the conversion
            // boundary maps it to ParseSeverity::Warning rather than Fatal.
            emitter.emit(Rich::custom(info.span(),
                format!("W-PAR-???: Unrecognized section sub-block key '{key}' — ignored")));
        }
    }
    FieldNode { name: Spanned::new(key, to_span(key_span, file_id)), value }
})
```

Then extend the `Rich → SyntaxError` conversion in `parser/mod.rs:191-282` to recognize the warning prefix (e.g. `W-PAR-` or a dedicated code) and construct a warning-severity `SyntaxError` (or push via `push_with_severity(.., ParseSeverity::Warning)`). The reserved-name collision (`EC-006`) stays fatal.

> **Required supporting change:** today the `parse()` gate treats ANY non-empty `errors` vec as failure (`parser/mod.rs:284-287`). To let a parse-time *warning* not fail the parse, the conversion must route warning-severity diagnostics into the `warnings` vec (like the missing-version advisory at `parser/mod.rs:293-305`) instead of the `errors` vec. The missing-version path is the exact template to copy. This is a small, well-precedented change — not new architecture.

Advantages: no change to the grammar-wide `extra::Err<Rich<...>>` type; reuses the existing emit→convert→sink pipeline; identical in shape to code that already ships and is tested.

### Pattern B (alternative) — custom `State` side-channel via `extra::Full<Rich, WarningCollector, ()>`

chumsky 0.10 supports a mutable `State` threaded through the whole parse, accessed in any `validate`/`map_with` closure via `MapExtra::state()`:

- `extra::Full<E, S, C>` declares `Error = E`, `State = S`, `Context = C` (docs.rs `chumsky::extra::Full` / `type.Context`, via Context7). `State` "provides stateful output of the parser ... [but] cannot influence the actual progress of the parser."
- `MapExtra::state(&mut self) -> &mut E::State` gives mutable access inside closures (docs.rs `chumsky::input::MapExtra`).
- Drive it with `parser().parse_with_state(input, &mut my_state)` (docs.rs `chumsky::extra::State`).

So one could thread a `&mut Vec<SectionWarning>` (or a `SimpleState<Vec<...>>`) and push warnings into it directly inside the section sub-block closure, then merge into `ParseResult::warnings` after the parse.

Disadvantage: changing the parser `extra` type from `extra::Err<Rich<...>>` to `extra::Full<Rich<...>, WarningCollector, ()>` is a grammar-wide ripple (every `impl Parser<..., extra::Err<...>>` signature in `parser/*.rs` would need updating) and the call site switches to `parse_with_state`. This is heavier than Pattern A for a single warning class and is **not** recommended unless a broader warning-collection refactor is independently desired.

**Recommendation:** Pattern A. It is the minimal, convention-consistent path and the entire emit→convert→warnings→sink pipeline already exists.

---

## Verdict & Recommendation

1. **BC-3.02.002 invariant 4's "parse time" placement is CORRECT and ACHIEVABLE.** chumsky 0.10.1's `validate()`/`Emitter` is purpose-built for non-fatal, span-carrying, parse-continuing diagnostics, and slideforge already uses this exact idiom (E-PAR-007/008) plus a parse-time *warning* precedent (missing-version). No new chumsky capability is required.

2. **The architect's DIR-077-001 §4 amendment is UNNECESSARY and rests on a conflation.** The "parser can't know the plugin registry" rationale describes the section-**TYPE** check (invariant 3), not the sub-block-**KEY** check (invariant 4). The KEY vocabulary is the fixed constant `["report", "detail"]` (per the architect's own §5), which carries no registry coupling. Recommend: **withdraw the §4 amendment**; keep invariant 4 at "parse time." Do NOT bump BC-3.02.002 to v1.3 for this reason. (If the product owner wants invariant 4 prose tightened, the correct edit is to *clarify* that the KEY vocabulary is a fixed parse-time constant distinct from the plugin-registered TYPE set — strengthening, not relocating, the invariant.)

3. **Ownership split (answering the explicit deliverable question):**
   - **STORY-078 parser SHOULD own the sub-block-KEY warning at parse time** (invariant 4 / EC-005) and the reserved-name collision error (EC-006). This also fixes DIR-077-001 §7 AC-004, which currently tells STORY-078 to merely store unrecognized keys and defer the warning to eval — that deferral is the consequence of the conflation and should be revised so STORY-078 emits the warning at parse time.
   - **Eval SHOULD own the section-TYPE validation** against the full plugin registry (invariant 3) — *if* plugin-registered section types must be supported. The parser may keep a built-in fast-path TYPE check only if a plugin-registered type would not be falsely rejected; otherwise TYPE validation belongs wholly at eval. (See the inconclusive flag below.)

4. **Implementation note for STORY-078:** route warning-severity diagnostics into `ParseResult::warnings` (not `errors`) so the parse still succeeds — copy the missing-version pattern at `parser/mod.rs:293-305`. The `DiagnosticSink::push_with_severity(.., ParseSeverity::Warning)` path then carries it to the CLI renderer with correct severity.

---

## Inconclusive / Flagged Areas

- **TYPE-check placement (invariant 3) is internally inconsistent in DIR-077-001.** §3.2 and §7 AC-003 mandate a parse-time fatal error for unrecognized section TYPES against the built-in list, while §4's rationale says the parser must not know the section registry. A plugin-registered section type would be falsely rejected at parse time under §3.2. This is the architect's domain to resolve and is **out of scope** for this parser-capability research; flagged for architect adjudication. It does not affect the invariant-4 verdict.
- **Exact diagnostic code for the new warning** (`W-PAR-NNN` vs reusing an `E-PAR` slot demoted to warning) is a product-owner/architect naming decision, not a capability question. The codebase has no existing `W-PAR-*` convention; the missing-version warning reuses `E-PAR-010` with `is_fatal:false`. Either approach works mechanically.
- I did not execute the parser (`Bash` is denied to the research agent). All code-behavior claims are sourced from reading the committed source and the committed passing tests, plus the official chumsky 0.10 docs. The conclusions are grounded in shipped, tested code paths (E-PAR-007/008, missing-version warning) rather than speculative new code.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Context7 query-docs | 2 | chumsky 0.10 `validate`/`Emitter::emit` non-fatal semantics; `extra::Full`/`State`/`MapExtra::state` threading |
| Context7 resolve-library-id | 1 | Resolve chumsky → `/websites/rs_chumsky_chumsky` (High reputation, 4721 snippets) |
| Read | 6 | BC-3.02.002, DIR-077-001, `sink.rs`, `error.rs`, `parser/mod.rs`, `parser/deck.rs`, `known_fields.rs` |
| Grep | 4 | chumsky version pin; severity/warning/Rich/recover patterns; section/REGISTER_SUB_BLOCK/notes keywords |
| Glob | 1 | Enumerate `slideforge-syntax/src` parser files |
| Training data | 0 areas | No reliance — chumsky claims verified against docs.rs via Context7; behavior claims verified against committed source + tests |

**Total MCP tool calls:** 3 (Context7). **Total local reads/searches:** 11.
**Training data reliance:** low — every chumsky capability claim is backed by a docs.rs citation retrieved 2026-06-01, and every slideforge-behavior claim cites a specific committed file:line (and, where relevant, a committed passing test).
