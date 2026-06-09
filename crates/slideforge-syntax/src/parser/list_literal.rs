//! Shared list-literal element validator used by ALL four value-position parsers.
//!
//! This module provides [`list_literal_elements`] and [`list_literal_elements_cf`],
//! two thin wrappers over a single internal combinator that produces the element
//! list for a `[item, item, ...]` list literal.
//!
//! # Motivation
//!
//! Four value-position parsers in the slideforge DSL accept list literals:
//!
//! 1. `deck.rs::value_parser` — `vars:` block and `@var` deck-level assignments
//! 2. `control_flow.rs::field_line_cf` — slide-body field assignments (production path)
//! 3. `deck.rs::set_rule_value_parser` — `set <type>: <field> [...]` rules
//! 4. `variants.rs::variant_value` — `variants: <name>: vars: <key>: [...]` entries
//!
//! Previously each site duplicated the list-item validation logic, causing
//! message inconsistencies (F-088-P5-MED-001 and F-088-P5-MED-002): sites 3
//! and 4 used a generic message without the per-type `got <type>` substitution
//! required by error-taxonomy v2.28 §E-PAR-024, and site 2 produced a cryptic
//! `ExpectedFound(LBracket)` for nested-list attempts instead of E-PAR-024
//! "nested list".
//!
//! This single shared combinator eliminates the quad-duplication. All four
//! sites route through it, guaranteeing identical E-PAR-024 emission (including
//! `got <type>` and `got nested list`) for the same malformed input in any
//! value-position context.
//!
//! # Grammar
//!
//! ```text
//! list_literal ::= "[" (list_item ("," list_item)*)? ","? "]"
//! list_item    ::= template_string   -- valid: FieldValue::Template
//!                | INT               -- invalid: E-PAR-024 "got integer"
//!                | FLOAT             -- invalid: E-PAR-024 "got decimal number"
//!                | BOOL              -- invalid: E-PAR-024 "got boolean"
//!                | IDENT             -- invalid: E-PAR-024 "got bare word"
//!                | "[" ...           -- invalid: E-PAR-024 "got nested list"
//! ```
//!
//! # Error Contract (BC-1.15.001 error accumulation)
//!
//! - Non-string items emit E-PAR-024 with `got <type>` and continue parsing.
//! - Nested list items `[` emit E-PAR-024 with `got nested list`; the inner
//!   list is consumed iteratively (O(1) call-stack depth) to resume after `]`.
//! - Parsing never fails on the first error — all items are inspected.
//!
//! # `DoS` Hardening (F-088-P6-CRIT-001)
//!
//! The spec mandates flat lists only — nested `[...]` is always an error. The
//! nested-list handler does NOT recurse into the inner content; instead it uses
//! an iterative depth-tracking loop (chumsky `custom` combinator) that consumes
//! tokens until the matching `]` with O(1) call-stack depth regardless of how
//! deeply the brackets are nested. This mirrors the E-PAR-021 guard in
//! `template.rs` (F-088-P6-CRIT-001 fix).

use chumsky::{input::ValueInput, prelude::*};

use crate::{ast::FieldValue, template::TemplateChunk, token::Token};

use super::template::template_value;

// ─── Type alias ───────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
type TSpan = SimpleSpan;

// ─── Internal implementation ──────────────────────────────────────────────────

/// Build the core list-literal elements parser.
///
/// `exclude_in`: when `true`, the bare-word alternative rejects the literal
/// identifier string `"in"` (for the `control_flow` slide-body path where `in`
/// is a structural keyword in `@for x in coll:` expressions).
///
/// # Stack depth
///
/// This parser is intentionally non-recursive. Nested-list detection is handled
/// by an iterative depth-tracking loop inside a `custom` combinator, so the
/// call-stack depth is O(1) regardless of how deeply `[` tokens are nested in
/// the input (`DoS` hardening per F-088-P6-CRIT-001).
fn list_literal_elements_impl<'src, I>(
    exclude_in: bool,
) -> impl Parser<'src, I, Vec<(FieldValue, TSpan)>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    // ── Valid item: template string (quoted literal or {{ expr }}) ──────────────
    let template_item = template_value().validate(
        move |(chunks, errs): (
            Vec<TemplateChunk>,
            Vec<crate::parser::template::TemplateError>,
        ),
              info,
              emitter| {
            for err in errs {
                emitter.emit(Rich::custom(info.span(), err.into_routing_message()));
            }
            (FieldValue::Template(chunks), info.span())
        },
    );

    // ── Invalid: nested list `[...]` ────────────────────────────────────────────
    //
    // The spec defines list literals as flat-only: a `[` at list-item position
    // is always an error. Rather than recursing into the nested content (which
    // would give O(depth) stack usage and allow adversarial stack exhaustion),
    // this combinator:
    //
    //   1. Uses `just(Token::LBracket)` to detect `[` at item position (chumsky
    //      handles the "not a `[`" failure path cleanly — other alternatives are
    //      tried without consuming input).
    //   2. Uses a `custom` parser to consume the nested tokens iteratively with a
    //      depth counter: `LBracket` → depth+1, `RBracket` → depth-1, stop at 0.
    //      Stack depth is O(1) for ANY nesting depth in the input.
    //   3. Emits a canonical E-PAR-024 "got nested list" diagnostic via `validate`.
    //
    // This replaces the previous `recursive(single_item_ref)` approach that gave
    // unbounded O(depth) stack growth (F-088-P6-CRIT-001 fix).
    let nested_item = just(Token::LBracket)
        .ignore_then(
            // Iterative balanced-bracket skip.  The opening `[` is already
            // consumed by `just(Token::LBracket)` above; depth starts at 1.
            // Each additional `[` increments depth; each `]` decrements it.
            // The loop stops as soon as depth reaches 0 (matching close `]`)
            // or at EOF (truncated/malformed input — stops gracefully).
            custom::<_, I, (), extra::Err<Rich<'src, Token, TSpan>>>(|inp| {
                let mut depth = 1_u32;
                while depth > 0 {
                    match inp.next() {
                        Some(Token::LBracket) => {
                            depth = depth.saturating_add(1);
                        },
                        Some(Token::RBracket) => {
                            depth -= 1;
                        },
                        Some(_) => {},
                        None => break, // EOF inside nested list — stop gracefully
                    }
                }
                Ok(())
            }),
        )
        .map_with(|(), e| (FieldValue::Error, e.span()))
        .validate(|(_, item_span): (FieldValue, TSpan), _info, emitter| {
            emitter.emit(Rich::custom(
                item_span,
                "E-PAR-024: non-string list item. \
                 List items must be quoted string literals; got nested list. \
                 Wrap the value in quotes to use it as a string."
                    .to_string(),
            ));
            (FieldValue::Error, item_span)
        });

    // ── Invalid: non-string primitives (Int/Float/Bool/Ident) ────────────────────
    //
    // The ident variant optionally excludes `"in"` for the control_flow path.
    // The validate() emits E-PAR-024 with the per-type `got <type>` substitution
    // (error-taxonomy v2.28 §E-PAR-024 binding format).
    let primitive_item = select! {
        Token::IntLit(n) = e => (FieldValue::Num(n), e.span()),
        Token::FloatLit(f) = e => (FieldValue::Float(f), e.span()),
        Token::BoolLit(b) = e => (FieldValue::Bool(b), e.span()),
        Token::Ident(s) = e => (FieldValue::Ident(s.to_string()), e.span()),
    }
    // For the control_flow path, exclude the `"in"` identifier so that `@for`
    // structural keywords are not consumed as bare-word list elements.
    .filter(move |(item, _span): &(FieldValue, TSpan)| {
        if exclude_in {
            !matches!(item, FieldValue::Ident(s) if s == "in")
        } else {
            true
        }
    })
    .validate(
        move |(item, item_span): (FieldValue, TSpan), _info, emitter| {
            let type_name = match &item {
                FieldValue::Num(_) => "integer",
                FieldValue::Float(_) => "decimal number",
                FieldValue::Bool(_) => "boolean",
                FieldValue::Ident(_) => "bare word",
                FieldValue::Shape(_) => "shape block",
                // Error sentinel — already reported.
                FieldValue::Error => "error sentinel",
                // Defense-in-depth for direct AST construction.
                FieldValue::List(_) => "nested list",
                // Template is matched above.
                FieldValue::Template(_) => unreachable!(
                    "Template is handled by template_item — should not reach here"
                ),
            };
            emitter.emit(Rich::custom(
                item_span,
                format!(
                    "E-PAR-024: non-string list item. \
                     List items must be quoted string literals; got {type_name}. \
                     Wrap the value in quotes to use it as a string."
                ),
            ));
            (FieldValue::Error, item_span)
        },
    );

    // Priority: template (valid) → nested list detection (bounded skip + E-PAR-024)
    // → primitive rejection.  No recursive combinator; O(1) call-stack depth.
    let single_item = template_item.or(nested_item).or(primitive_item);

    // Wrap the single-item parser in `[` ... `]` with comma separation and
    // optional trailing comma. Token-stream idiom: just(Token::LBracket) etc.
    // allow_trailing() accepts `["A", "B",]` (EC-003 compliance).
    single_item
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Build a list-literal elements parser for `vars:`, `@var`, `set_rule`, and
/// `variant_value` contexts (positions 1, 3, 4).
///
/// All identifier tokens are treated as bare words. No keyword exclusion.
///
/// Returns a parser matching `"[" (item ("," item)*)? ","? "]"` and producing
/// `Vec<(FieldValue, TSpan)>`. Callers wrap into `FieldValue::List` or
/// `SetRuleValue::List` as appropriate.
///
/// # Example
///
/// ```ignore
/// let list_val = list_literal_elements()
///     .map_with(|items, e| {
///         let items_fv = items.into_iter().map(|(fv, _)| fv).collect();
///         (FieldValue::List(items_fv), e.span())
///     });
/// ```
#[must_use]
pub fn list_literal_elements<'src, I>()
-> impl Parser<'src, I, Vec<(FieldValue, TSpan)>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    list_literal_elements_impl(false)
}

/// Build a list-literal elements parser for the `control_flow` slide-body context
/// (position 2).
///
/// Identical to [`list_literal_elements`] but excludes the `"in"` identifier from
/// bare-word items to avoid consuming the `@for x in coll:` structural keyword as
/// a list element.
#[must_use]
pub fn list_literal_elements_cf<'src, I>()
-> impl Parser<'src, I, Vec<(FieldValue, TSpan)>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    list_literal_elements_impl(true)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use chumsky::prelude::*;

    use crate::{lexer::lex, parser::deck::deck_parser, span::SourceMap, token::Token};

    /// Helper: lex `src`, run the deck parser, and return the error reason strings.
    fn parse_get_errors(src: &str) -> Vec<String> {
        let file: Arc<str> = Arc::from("test.sf");
        let (tokens, _lex_errs) = lex(src, file.clone());
        let eoi = SimpleSpan::from(src.len()..src.len());
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
        let spanned_tokens: Vec<(Token, SimpleSpan)> = tokens
            .into_iter()
            .map(|(t, s)| (t, SimpleSpan::from(s)))
            .collect();
        let input = spanned_tokens
            .as_slice()
            .map(eoi, |(t, s): &(Token, SimpleSpan)| (t, s));
        let (_deck_opt, parse_errs) = deck_parser(file_id).parse(input).into_output_errors();
        parse_errs
            .iter()
            .map(|e| format!("{:?}", e.reason()))
            .collect()
    }

    // ── F-088-P6-CRIT-001 RED-GATE: deeply-nested list must NOT overflow the
    // call stack and must emit E-PAR-024.
    //
    // PRE-FIX behavior (the defect being fixed): the Pass-5 refactor introduced
    // `recursive(single_item_ref)` where `nested_item` consumed the contents of
    // `[...]` by recursing through `single_item_ref`.  For input with N levels of
    // `[`, this created N stack frames — adversarial input with N ≈ 5000 causes a
    // stack overflow (SIGSEGV/abort, NOT a graceful Err).
    //
    // POST-FIX behavior (what this test asserts): the iterative `custom`
    // depth-tracking loop uses O(1) call-stack depth regardless of N.  The test
    // must complete without panic/abort and must emit an E-PAR-024 diagnostic.
    //
    // To verify the RED-gate: on pre-fix code, running this test crashes the
    // process with a stack-overflow signal (observed via `cargo nextest run
    // -p slideforge-syntax -E 'test(deep_nested_list_no_stack_overflow)'` on the
    // pre-fix branch — the test process exits with signal rather than a clean
    // FAIL).

    /// F-088-P6-CRIT-001 (helper level): 5000-deep properly-matched nested
    /// `[...]` must NOT overflow the call stack and must produce an E-PAR-024
    /// diagnostic.
    ///
    /// Input structure: `bullets [` + 5000 `[` + `"item"` + 5001 `]`
    ///   - outer `delimited_by` consumes the outermost `[` / `]` pair
    ///   - `nested_item` detects the next `[` and consumes everything through
    ///     the matching close via the iterative depth-tracking loop
    ///
    /// Two load-bearing properties:
    /// 1. **No stack overflow** — reaching the assertions proves the depth-cap
    ///    fix fired (the iterative loop consumed all 5000 brackets without
    ///    recursing on the call stack).
    /// 2. **E-PAR-024 emitted** — the combinator correctly identifies the
    ///    nested-list construct and emits the canonical error code.
    ///
    /// PRE-FIX regression note: on the pre-fix branch (`recursive(single_item_ref)`),
    /// running this test causes a stack overflow — the test process exits with a
    /// signal (SIGSEGV or STATUS\_STACK\_OVERFLOW), not a clean `FAIL`. The fix uses
    /// an iterative `custom`-combinator depth tracker (O(1) call stack).
    #[test]
    fn test_f088_p6_crit001_deep_nested_list_no_stack_overflow() {
        // Build `bullets [` + 5000 `[` + `"item"` + 5001 `]` (properly matched).
        // The pre-fix recursive combinator attempts 5000 stack frames for this
        // input and overflows; the fixed iterative loop handles it in O(1) stack.
        const DEPTH: usize = 5000;
        let opens: String = "[".repeat(DEPTH);
        let closes: String = "]".repeat(DEPTH + 1); // +1 for the outer delimited_by close
        let src = format!("slide content:\n  bullets [{opens}\"item\"{closes}\n");

        // Must NOT panic — reaching here proves no stack overflow.
        let errors = parse_get_errors(&src);

        // Must emit ≥1 error (nested list is always invalid).
        assert!(
            !errors.is_empty(),
            "F-088-P6-CRIT-001: 5000-deep nested list must produce ≥1 parse error; got 0.\n\
             This likely means the input was silently accepted, which is wrong."
        );

        // At least one error must be E-PAR-024 "nested list" (not a generic
        // ExpectedFound or an unrelated parser rejection).
        let has_e_par_024_nested = errors
            .iter()
            .any(|msg| msg.contains("E-PAR-024") && msg.contains("nested list"));
        assert!(
            has_e_par_024_nested,
            "F-088-P6-CRIT-001: 5000-deep nested list must emit E-PAR-024 'nested list'; \
             got errors: {errors:?}"
        );
    }

    /// F-088-P6-CRIT-001 (integration, set-rule path): deep nesting through
    /// the `set_rule_value_parser` path (position 3) also uses the shared
    /// combinator and must not overflow.
    ///
    /// Verifies that the shared-combinator fix covers all 4 value-position paths
    /// (the combinator is the single chokepoint — one test at the helper level
    /// above is sufficient for depth proof; this test confirms path routing).
    #[test]
    fn test_f088_p6_crit001_set_rule_deep_nested_list_no_stack_overflow() {
        // 200-deep properly-matched nesting via the set-rule path.
        const DEPTH: usize = 200;
        let opens: String = "[".repeat(DEPTH);
        let closes: String = "]".repeat(DEPTH + 1);
        let src = format!(
            "set content: bullets [{opens}\"item\"{closes}\nslide content:\n  title \"T\"\n"
        );

        // Must NOT panic.
        let errors = parse_get_errors(&src);

        assert!(
            !errors.is_empty(),
            "F-088-P6-CRIT-001 set-rule: 200-deep nested list must produce ≥1 error; got 0."
        );

        let has_nested = errors
            .iter()
            .any(|msg| msg.contains("E-PAR-024") && msg.contains("nested list"));
        assert!(
            has_nested,
            "F-088-P6-CRIT-001 set-rule: must emit E-PAR-024 'nested list'; \
             got errors: {errors:?}"
        );
    }
}
