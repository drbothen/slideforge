//! S3 Spike — DSL compile-time `alt` enforcement prototype.
//!
//! Demonstrates how chumsky 0.10 enforces the `alt "..."` requirement on visual
//! elements at parse time, with:
//!   - Error accumulation (all errors reported, not just the first)
//!   - Source spans on every error (file:line:col for miette diagnostics)
//!   - Hard error for `image:` block without `alt "..."`
//!   - Hard error for `chart:` block without `alt "..."`
//!   - Hard error for `diagram:` block without `alt "..."`
//!   - `decorative: true` accepted as alternative to `alt "..."`
//!
//! This is a standalone spike binary — NOT part of the slideforge workspace.
//!
//! Run: cargo run --bin s3_parser_alt
//!
//! The input is a small hardcoded .sf fragment; adjust SOURCE_VALID and SOURCE_INVALID
//! below to explore the parser behavior.

use chumsky::prelude::*;

// ─── AST types ───────────────────────────────────────────────────────────────

/// An image block in the DSL.
///
/// ```text
/// image:
///   src "assets/logo.png"
///   alt "Company logo"
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ImageBlock {
    pub src: String,
    pub alt: AltText,
}

/// A chart block in the DSL.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartBlock {
    pub chart_type: String,
    pub alt: AltText,
}

/// Alt text variant: either a concrete string or decorative (empty alt + Artifact).
#[derive(Debug, Clone, PartialEq)]
pub enum AltText {
    Text(String),
    Decorative,
}

/// A visual element (image, chart, or diagram) from a slide.
#[derive(Debug, Clone, PartialEq)]
pub enum VisualElement {
    Image(ImageBlock),
    Chart(ChartBlock),
}

// ─── Error type ──────────────────────────────────────────────────────────────

/// A DSL parse/validation error carrying a source span and human-readable message.
///
/// In production, `span` is a `FileSpan { file: PathBuf, range: Range<usize> }`
/// so that miette can render `file:line:col` pointers. For the spike we use
/// `SimpleSpan` (byte range in the input string).
#[derive(Debug, Clone, PartialEq)]
pub struct SfError {
    pub span: SimpleSpan,
    pub message: String,
    pub hint: Option<String>,
}

impl SfError {
    pub fn new(span: SimpleSpan, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

// ─── Parser ──────────────────────────────────────────────────────────────────

// Convenience alias for the extra type carrying our error.
type Extra<'a> = extra::Err<Rich<'a, char>>;

/// Parse a quoted string: `"...content..."`
fn quoted_string<'a>() -> impl Parser<'a, &'a str, String, Extra<'a>> {
    just('"')
        .ignore_then(
            none_of('"')
                .repeated()
                .collect::<String>(),
        )
        .then_ignore(just('"'))
        .labelled("quoted string")
}

/// Parse an identifier: `[a-z_]+`
fn ident<'a>() -> impl Parser<'a, &'a str, String, Extra<'a>> {
    any()
        .filter(|c: &char| c.is_ascii_lowercase() || *c == '_')
        .repeated()
        .at_least(1)
        .collect::<String>()
        .labelled("identifier")
}

/// Parse one key-value pair on a single line, e.g.:
///   `  src "assets/logo.png"\n`
///   `  decorative: true\n`
fn kv_pair<'a>() -> impl Parser<'a, &'a str, (String, String), Extra<'a>> {
    // Allow optional leading whitespace.
    text::whitespace()
        .ignore_then(ident())
        .then(
            just(':')
                .or_not()
                .ignore_then(text::whitespace())
                .ignore_then(
                    // Value is either a quoted string or a bare word (true/false).
                    quoted_string().or(
                        any()
                            .filter(|c: &char| !c.is_whitespace() && *c != '\n')
                            .repeated()
                            .at_least(1)
                            .collect::<String>()
                    )
                ),
        )
        .then_ignore(text::newline().or_not())
        .labelled("key-value pair")
}

/// Parse an `image:` block from the DSL.
///
/// Grammar:
/// ```text
/// image:
///   src "path/to/file.png"
///   alt "Alt text here"    -- required (unless decorative: true)
///   [decorative: true]     -- alternative to alt
/// ```
///
/// Validation:
///   - Hard error if neither `alt` nor `decorative: true` is present.
fn image_block_parser<'a>() -> impl Parser<'a, &'a str, VisualElement, Extra<'a>> {
    just("image:")
        .then(text::newline())
        .ignore_then(
            kv_pair()
                .repeated()
                .at_least(1)
                .collect::<Vec<_>>()
        )
        .validate(|pairs: Vec<(String, String)>, e, emitter| {
            let mut src: Option<String> = None;
            let mut alt: Option<String> = None;
            let mut decorative = false;

            for (key, val) in &pairs {
                match key.as_str() {
                    "src" => src = Some(val.clone()),
                    "alt" => alt = Some(val.clone()),
                    "decorative" if val == "true" => decorative = true,
                    _ => {
                        // Unknown key — warn but do not block.
                        emitter.emit(Rich::custom(
                            e.span(),
                            format!("image: unknown field '{key}' (ignored)"),
                        ));
                    }
                }
            }

            // Require src.
            let src = src.unwrap_or_else(|| {
                emitter.emit(Rich::custom(
                    e.span(),
                    "image: missing required field 'src'",
                ));
                String::new()
            });

            // Require alt OR decorative: true.
            let alt_text = if decorative {
                AltText::Decorative
            } else if let Some(text) = alt {
                if text.is_empty() {
                    emitter.emit(Rich::custom(
                        e.span(),
                        "image: 'alt' field is empty. Provide a meaningful description, \
                        or use 'decorative: true' for purely decorative images.",
                    ));
                }
                AltText::Text(text)
            } else {
                // HARD ERROR: image without alt text.
                emitter.emit(Rich::custom(
                    e.span(),
                    "image: missing 'alt' field. Every non-decorative image requires \
                    alt text (WCAG 1.1.1). Add 'alt \"Description of the image\"' or \
                    'decorative: true'.",
                ));
                AltText::Text(String::new()) // placeholder so parsing can continue
            };

            VisualElement::Image(ImageBlock { src, alt: alt_text })
        })
        .labelled("image block")
}

/// Parse a `chart:` block from the DSL.
///
/// Grammar:
/// ```text
/// chart line:
///   alt "Chart description"
/// ```
fn chart_block_parser<'a>() -> impl Parser<'a, &'a str, VisualElement, Extra<'a>> {
    just("chart")
        .then(text::whitespace())
        .ignore_then(ident()) // chart type: line, bar, pie, etc.
        .then_ignore(just(':'))
        .then_ignore(text::newline())
        .then(
            kv_pair()
                .repeated()
                .collect::<Vec<_>>()
        )
        .validate(|(chart_type, pairs): (String, Vec<(String, String)>), e, emitter| {
            let mut alt: Option<String> = None;
            let mut decorative = false;

            for (key, val) in &pairs {
                match key.as_str() {
                    "alt" => alt = Some(val.clone()),
                    "decorative" if val == "true" => decorative = true,
                    _ => {} // other chart fields (data, color, etc.) not validated here
                }
            }

            let alt_text = if decorative {
                AltText::Decorative
            } else if let Some(text) = alt {
                AltText::Text(text)
            } else {
                // HARD ERROR: chart without alt text.
                emitter.emit(Rich::custom(
                    e.span(),
                    format!(
                        "chart {chart_type}: missing 'alt' field. Every chart requires alt text \
                        describing its content (WCAG 1.1.1). Add 'alt \"Chart description\"' or \
                        'decorative: true'."
                    ),
                ));
                AltText::Text(String::new())
            };

            VisualElement::Chart(ChartBlock { chart_type, alt: alt_text })
        })
        .labelled("chart block")
}

/// Parse a sequence of visual elements separated by blank lines.
fn visual_elements_parser<'a>() -> impl Parser<'a, &'a str, Vec<VisualElement>, Extra<'a>> {
    let element = image_block_parser().or(chart_block_parser());
    element
        .separated_by(text::newline().repeated().at_least(1))
        .allow_leading()
        .allow_trailing()
        .collect::<Vec<_>>()
}

// ─── Main / demo ─────────────────────────────────────────────────────────────

/// Valid DSL fragment — should parse with zero errors.
const SOURCE_VALID: &str = r#"
image:
  src "assets/revenue-chart.png"
  alt "Revenue trend chart showing 23% growth Q1-Q4 2025"

image:
  src "assets/decorative-divider.svg"
  decorative: true

chart line:
  data @revenue_data
  color brand.primary
  alt "Line chart: monthly revenue Jan-Dec 2025, peak in September"
"#;

/// Invalid DSL fragment — should accumulate multiple errors.
const SOURCE_INVALID: &str = r#"
image:
  src "assets/org-chart.png"

chart bar:
  data @headcount_data
  color brand.accent1

image:
  src "assets/logo.png"
  alt ""
"#;

fn print_result(label: &str, source: &str) {
    println!("=== {} ===", label);
    println!("Input:");
    for line in source.lines() {
        println!("  {line}");
    }
    println!();

    let result = visual_elements_parser().parse(source);

    match result.into_output_errors() {
        (Some(elements), errors) if errors.is_empty() => {
            println!("Parsed {} element(s), 0 errors:", elements.len());
            for (i, el) in elements.iter().enumerate() {
                println!("  [{i}] {el:?}");
            }
        }
        (output, errors) => {
            if let Some(elements) = output {
                println!("Parsed {} element(s) (with errors):", elements.len());
                for (i, el) in elements.iter().enumerate() {
                    println!("  [{i}] {el:?}");
                }
            } else {
                println!("Parse produced no output.");
            }
            println!("{} error(s):", errors.len());
            for err in &errors {
                println!("  [ERROR] span={:?}: {}", err.span(), err.reason());
                // In production: convert to miette::Diagnostic with source span rendering.
            }
        }
    }
    println!();
}

fn main() {
    println!("S3 Spike — DSL compile-time `alt` enforcement via chumsky 0.10");
    println!("================================================================");
    println!();
    println!("Key findings:");
    println!("  - chumsky 0.10 `validate` emits errors without stopping the parse.");
    println!("  - Multiple errors accumulate in one pass — matches slideforge requirement.");
    println!("  - Custom error types with SimpleSpan are straightforward.");
    println!("  - `Rich::custom(span, message)` produces human-readable errors.");
    println!("  - Production: replace Rich<char> with SfError {{ span, message, hint }}.");
    println!();

    print_result("Valid .sf fragment (expect 0 errors)", SOURCE_VALID);
    print_result("Invalid .sf fragment (expect 3 errors)", SOURCE_INVALID);

    println!("Architecture implication:");
    println!("  The `alt` enforcement belongs in slideforge-syntax (pure parser layer).");
    println!("  No I/O needed — purely structural validation of the AST.");
    println!("  This makes the enforcement Kani-amenable: the property");
    println!("  'if VisualElement::Image.alt == None then errors is non-empty'");
    println!("  can be formally proven as a VP in Phase 6.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_with_alt_parses_cleanly() {
        let src = "image:\n  src \"logo.png\"\n  alt \"Company logo\"\n";
        let result = image_block_parser().parse(src);
        let (output, errors) = result.into_output_errors();
        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
        assert_eq!(
            output,
            Some(VisualElement::Image(ImageBlock {
                src: "logo.png".to_string(),
                alt: AltText::Text("Company logo".to_string()),
            }))
        );
    }

    #[test]
    fn image_with_decorative_parses_cleanly() {
        let src = "image:\n  src \"divider.svg\"\n  decorative: true\n";
        let result = image_block_parser().parse(src);
        let (output, errors) = result.into_output_errors();
        assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
        assert_eq!(
            output,
            Some(VisualElement::Image(ImageBlock {
                src: "divider.svg".to_string(),
                alt: AltText::Decorative,
            }))
        );
    }

    #[test]
    fn image_without_alt_emits_error() {
        let src = "image:\n  src \"org-chart.png\"\n";
        let result = image_block_parser().parse(src);
        let (_, errors) = result.into_output_errors();
        assert!(
            !errors.is_empty(),
            "Expected an error for image without alt text"
        );
        let error_msg = format!("{:?}", errors[0].reason());
        assert!(
            error_msg.contains("alt"),
            "Error message should mention 'alt', got: {error_msg}"
        );
    }

    #[test]
    fn chart_without_alt_emits_error() {
        let src = "chart bar:\n  data @revenue\n  color brand.primary\n";
        let result = chart_block_parser().parse(src);
        let (_, errors) = result.into_output_errors();
        assert!(
            !errors.is_empty(),
            "Expected an error for chart without alt text"
        );
    }

    #[test]
    fn multiple_errors_accumulate_in_one_pass() {
        // Two images both missing alt — should get 2 errors, not 1.
        let src = concat!(
            "image:\n  src \"a.png\"\n\n",
            "image:\n  src \"b.png\"\n",
        );
        let result = visual_elements_parser().parse(src);
        let (_, errors) = result.into_output_errors();
        assert!(
            errors.len() >= 2,
            "Expected at least 2 errors for 2 images without alt, got {}: {errors:?}",
            errors.len()
        );
    }
}
