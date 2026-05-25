// main.rs — Demo binary for Spike S4: chumsky 0.10 indentation parser.
//
// Demonstrates:
// 1. Indentation lexer (INDENT/DEDENT tokens, spaces-only enforcement)
// 2. Mini-DSL parser (chumsky 0.10 over spanned token stream)
// 3. Error recovery (collect ALL errors, not just the first)
// 4. Mode switching (text mode `{{ }}` vs math mode `$...$` + `@{var}`)

mod ast;
mod lexer;
mod parser;
mod token;


fn main() {
    println!("=== Spike S4: chumsky 0.10 indentation parser demo ===\n");

    // -----------------------------------------------------------------------
    // Demo 1: Successful parse of a valid slide file fragment.
    // -----------------------------------------------------------------------
    println!("--- Demo 1: Valid slide parse ---");
    let valid_src = r#"slide title:
    title: "Hello World"
    subtitle: "A {{ project }} demo"
    color: blue

slide content:
    title: "Five Things Every MSSP Metric System Must Answer"
    bullets:
        - "Can we sell it?"
        - "Can we onboard it?"
        - "Can we detect accurately?"
        - "Can we operate within SLA?"
        - "Can we do all this profitably?"
    takeaway: "If our metrics can't answer all five, we're measuring the wrong things."

slide metric_tree:
    title: "Executive Metric Tree"
    root:
        label: "Retained Protected Revenue at Target Margin"
        color: blue
    footnote: "Each leaf rolls up to one dimension."
"#;

    let (toks, lex_errors) = lexer::lex(valid_src);
    if lex_errors.is_empty() {
        println!("  Lexer: {} tokens, 0 errors", toks.len());
    } else {
        println!("  Lexer errors ({}):", lex_errors.len());
        for e in &lex_errors {
            println!("    {e}");
        }
    }

    let (ast, parse_errors) = parser::parse(&toks);
    if parse_errors.is_empty() {
        println!("  Parser: success");
    } else {
        println!("  Parser errors ({}):", parse_errors.len());
        for e in &parse_errors {
            println!("    {}", parser::format_parse_error(e, valid_src));
        }
    }
    if let Some(doc) = &ast {
        println!("  AST: {} top-level items", doc.items.len());
        for (item, span) in &doc.items {
            match item {
                ast::TopLevelItem::Slide(slide) => {
                    println!(
                        "    slide '{}' at bytes {}..{}, {} fields",
                        slide.slide_type.0,
                        span.start,
                        span.end,
                        slide.fields.len()
                    );
                }
                ast::TopLevelItem::Metadata(fields) => {
                    println!("    metadata: {} fields", fields.len());
                }
                ast::TopLevelItem::Include(path) => {
                    println!("    @include \"{path}\"");
                }
                ast::TopLevelItem::Error => {
                    println!("    <error node>");
                }
            }
        }
    }
    println!();

    // -----------------------------------------------------------------------
    // Demo 2: Error recovery — multiple errors, parser continues.
    // -----------------------------------------------------------------------
    println!("--- Demo 2: Error recovery (multiple errors collected) ---");
    let error_src = r#"slide title:
    title "Missing Colon Here"
    subtitle: "This line is OK"
    color blue
    unknown_key !!INVALID_VALUE
slide content:
    title: "Second slide — parser recovered"
"#;

    let (toks2, lex_errors2) = lexer::lex(error_src);
    println!("  Lexer: {} tokens, {} lex errors", toks2.len(), lex_errors2.len());
    for e in &lex_errors2 {
        println!("    lex: {e}");
    }

    let (ast2, parse_errors2) = parser::parse(&toks2);
    println!("  Parser: {} parse errors collected", parse_errors2.len());
    for e in &parse_errors2 {
        println!("    parse: {}", parser::format_parse_error(e, error_src));
    }
    if let Some(doc2) = &ast2 {
        println!("  AST (partial): {} top-level items recovered", doc2.items.len());
    } else {
        println!("  AST: None (recovery failed entirely)");
    }
    println!();

    // -----------------------------------------------------------------------
    // Demo 3: Tab detection — tabs produce errors, not silent acceptance.
    // -----------------------------------------------------------------------
    println!("--- Demo 3: Tab-in-indentation error ---");
    let tab_src = "slide title:\n\ttitle: \"Tab-indented — should error\"\n";
    let (toks3, lex_errors3) = lexer::lex(tab_src);
    println!("  Lexer: {} tokens, {} lex errors", toks3.len(), lex_errors3.len());
    for e in &lex_errors3 {
        println!("    {e}");
    }
    println!();

    // -----------------------------------------------------------------------
    // Demo 4: Mode switching — show token output for text and math modes.
    // -----------------------------------------------------------------------
    println!("--- Demo 4: Mode-switching token output ---");

    // Text mode: `{{ var }}` interpolation.
    let text_mode_src = "title: \"Hello {{ project }} world\"\n";
    println!("  Text mode input: {text_mode_src:?}");
    let (text_toks, _) = lexer::lex(text_mode_src);
    print!("  Tokens: ");
    for (tok, _) in &text_toks {
        if !matches!(tok, token::Token::Eof) {
            print!("{tok} ");
        }
    }
    println!();
    // NOTE: `{{ var }}` inside a double-quoted string is tokenized as a
    // SINGLE Str token by the lexer (the lexer scans to the closing `"`).
    // The interpolation is preserved verbatim inside the string content.
    // Interpolation expansion happens at parse/eval time when string values
    // are processed. This is the correct behavior: the lexer is mode-agnostic
    // for string contents.

    // Bare text with `{{ }}` outside a string (DSL field value interpolation).
    let bare_interp_src = "slide content:\n    title: {{ project_name }}\n";
    println!("  Bare interpolation input: {bare_interp_src:?}");
    let (bare_toks, _) = lexer::lex(bare_interp_src);
    print!("  Tokens: ");
    for (tok, _) in &bare_toks {
        if !matches!(tok, token::Token::Eof) {
            print!("[{tok}] ");
        }
    }
    println!();
    println!("  -> LBrace `{{{{` and RBrace `}}}}` are separate tokens outside strings");
    println!("  -> `$...$` tokens: Dollar, [content], Dollar");
    println!("  -> `$$...$$` tokens: DollarDollar, [content], DollarDollar");
    println!("  -> In math mode, `@{{var}}` uses AtBrace instead of LBrace");
    println!("  -> Rule: `{{{{ }}}}` disabled inside `$...$` math blocks (parser enforces this)");
    println!();

    // -----------------------------------------------------------------------
    // Demo 5: No implicit type coercion.
    // -----------------------------------------------------------------------
    println!("--- Demo 5: No implicit type coercion ---");
    let coercion_src = "flag: NO\nversion: 1.10\ncount: 42\n";
    let (coercion_toks, _) = lexer::lex(coercion_src);
    for (tok, span) in &coercion_toks {
        if !matches!(tok, token::Token::Newline | token::Token::Eof) {
            println!("  bytes {}..{}: {tok:?}", span.start, span.end);
        }
    }
    println!("  -> NO stays as Ident(\"NO\"), not Bool(false)");
    println!("  -> 1.10 stays as Float(\"1.10\"), not rounded to 1.1");
    println!("  -> 42 stays as Int(\"42\"), not i64");
    println!();

    // -----------------------------------------------------------------------
    // Demo 6: Span accuracy — every token has a valid byte-offset span.
    // -----------------------------------------------------------------------
    println!("--- Demo 6: Span accuracy ---");
    let span_src = "slide title:\n    color: blue\n";
    let (span_toks, _) = lexer::lex(span_src);
    let all_valid = span_toks.iter().all(|(_, s)| {
        s.start <= span_src.len() && s.end <= span_src.len() && s.start <= s.end
    });
    println!("  All {} token spans are valid byte offsets: {all_valid}", span_toks.len());
    for (tok, span) in &span_toks {
        if !matches!(tok, token::Token::Eof | token::Token::Newline | token::Token::Indent | token::Token::Dedent) {
            let slice = &span_src[span.start..span.end];
            println!("    [{tok}] bytes {}..{} = {:?}", span.start, span.end, slice);
        }
    }
    println!();

    println!("=== Demo complete — see S4-chumsky-indentation-parser.md for full assessment ===");
}
