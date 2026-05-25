// bench.rs — Performance benchmark for Spike S4.
//
// Generates a synthetic 100-slide .sf document and measures parse time.
// Not a criterion benchmark (no external dep) — uses std::time::Instant.

mod ast;
mod lexer;
mod parser;
mod token;

fn generate_slides(n: usize) -> String {
    let mut out = String::new();
    out.push_str("metadata:\n");
    out.push_str("    title: \"Benchmark Deck\"\n");
    out.push_str("    author: \"Bench Bot\"\n\n");

    for i in 0..n {
        match i % 5 {
            0 => {
                // title slide
                out.push_str(&format!(
                    "slide title:\n    title: \"Slide {i}\"\n    subtitle: \"Subtitle {i}\"\n    color: blue\n\n"
                ));
            }
            1 => {
                // content with bullets
                out.push_str(&format!(
                    "slide content:\n    title: \"Content {i}\"\n    bullets:\n        - \"Bullet one for {i}\"\n        - \"Bullet two for {i}\"\n        - \"Bullet three for {i}\"\n    takeaway: \"Takeaway {i}\"\n\n"
                ));
            }
            2 => {
                // two_column
                out.push_str(&format!(
                    "slide two_column:\n    title: \"Two Column {i}\"\n    left:\n        header: \"Left {i}\"\n        body: \"Left body text {i}\"\n    right:\n        header: \"Right {i}\"\n        body: \"Right body text {i}\"\n\n"
                ));
            }
            3 => {
                // divider
                out.push_str(&format!(
                    "slide divider:\n    color: purple\n    title: \"Section {i}\"\n    subtitle: \"Subsection {i}\"\n\n"
                ));
            }
            _ => {
                // metric_tree
                out.push_str(&format!(
                    "slide metric_tree:\n    title: \"Metrics {i}\"\n    root:\n        label: \"North Star {i}\"\n        color: blue\n    footnote: \"Footnote {i}\"\n\n"
                ));
            }
        }
    }

    out.push_str("slide end:\n    color: blue\n");
    out
}

fn main() {
    let slide_counts = [10, 25, 50, 100, 200];

    println!("=== Spike S4: Parse performance benchmark ===");
    println!("(Using std::time::Instant — wall-clock, single-run each)");
    println!();
    println!("{:<15} {:>12} {:>12} {:>12} {:>15}",
        "Slides", "Tokens", "Lex (µs)", "Parse (µs)", "Total (µs)");
    println!("{}", "-".repeat(68));

    for &n in &slide_counts {
        let src = generate_slides(n);
        let src_bytes = src.len();

        // Warm up (parse once before measuring).
        let (warm_toks, _) = lexer::lex(&src);
        let _ = parser::parse(&warm_toks);

        // Measure lex.
        let lex_start = std::time::Instant::now();
        let iterations = 10;
        let mut final_toks = Vec::new();
        for _ in 0..iterations {
            let (toks, _) = lexer::lex(&src);
            final_toks = toks;
        }
        let lex_us = lex_start.elapsed().as_micros() / iterations;

        // Measure parse.
        let parse_start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = parser::parse(&final_toks);
        }
        let parse_us = parse_start.elapsed().as_micros() / iterations;

        println!("{:<15} {:>12} {:>12} {:>12} {:>15}",
            n,
            final_toks.len(),
            lex_us,
            parse_us,
            lex_us + parse_us,
        );
    }

    println!();
    println!("Notes:");
    println!("  - Averaged over {iterations} iterations to reduce jitter", iterations = 10);
    println!("  - Single-threaded, no criterion overhead");
    println!("  - Includes token allocation overhead");
    println!("  - chumsky 0.10.1 — error recovery paths not exercised in this benchmark");
    println!();

    // Show the generated source size for transparency.
    let src_100 = generate_slides(100);
    println!("100-slide source: {} bytes, {} lines",
        src_100.len(),
        src_100.lines().count()
    );
}
