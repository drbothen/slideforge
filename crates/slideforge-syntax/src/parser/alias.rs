//! Alias declaration parser and expansion registry.
//!
//! Implements `alias_decl()` which parses `alias <name> = <base_type>: <fields>`
//! declarations into [`AliasNode`]s, and an [`AliasRegistry`] that eagerly
//! expands alias references in `slide` blocks.
//!
//! # Grammar
//!
//! ```text
//! alias_decl ::= "alias" IDENT "=" IDENT ":" NEWLINE INDENT field_line+ DEDENT
//! field_line ::= IDENT value NEWLINE
//! ```
//!
//! # Expansion Semantics
//!
//! After parsing an alias declaration:
//! 1. The alias name and base type + preset fields are stored in the registry.
//! 2. When a `slide <alias_name>:` is encountered, the registry looks up the
//!    alias and produces a `SlideNode` with:
//!    - `kind` = base type (not the alias name)
//!    - `fields` = preset fields (from alias) merged with explicit fields
//!      (explicit fields override preset fields when both name the same field)
//!
//! Aliases do NOT appear in the final `DeckNode`. They are a parse-time
//! syntactic sugar only.
//!
//! # Field Validation
//!
//! Alias bodies are validated against the [`known_fields()`](crate::known_fields())
//! registry. A field name not in the base type's known fields produces E-PAR-011.
//!
//! # Error Codes
//!
//! | Code | Condition |
//! |------|-----------|
//! | E-PAR-006 | Alias name collides with a built-in slide type |
//! | E-PAR-011 | Alias body contains a field not valid for the base type |

use std::collections::HashMap;

use chumsky::{input::ValueInput, prelude::*};

use crate::{
    ast::{AliasNode, FieldNode, FieldValue, SlideNode},
    keywords::classify_keyword,
    known_fields::{all_slide_types, known_fields},
    span::{Span, Spanned},
    template::TemplateChunk,
    token::Token,
};

use super::template::template_value;

// ─── Type aliases ─────────────────────────────────────────────────────────────

/// The span type used by chumsky in the token-stream parser.
type TSpan = SimpleSpan;

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Convert a chumsky `SimpleSpan` + `file_id` into a project [`Span`].
fn to_span(ss: SimpleSpan, file_id: u32) -> Span {
    Span::new(file_id, ss.start, ss.end)
}

/// Zero-length span for error-recovery nodes.
fn zero_span(file_id: u32) -> Span {
    Span::new(file_id, 0, 0)
}

/// Match any identifier token and return its string value and span.
fn any_ident<'src, I>()
-> impl Parser<'src, I, (String, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    select! { Token::Ident(s) = e => (s.to_string(), e.span()) }
}

/// Match an identifier token whose string value equals `kw`.
fn keyword<'src, I>(
    kw: &'static str,
) -> impl Parser<'src, I, TSpan, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    select! { Token::Ident(s) = e if s.as_ref() == kw => e.span() }
}

// ─── Field value parser ───────────────────────────────────────────────────────

/// Parse a field value inside an alias body (same as slide field values).
fn alias_field_value<'src, I>()
-> impl Parser<'src, I, (FieldValue, TSpan), extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    let template_val = template_value().validate(
        move |(chunks, errs): (Vec<TemplateChunk>, Vec<crate::parser::template::TemplateError>), info, emitter| {
            for err in errs {
                emitter.emit(Rich::custom(info.span(), err.into_message()));
            }
            (FieldValue::Template(chunks), info.span())
        },
    );
    let other_val = select! {
        Token::IntLit(n) = e => (FieldValue::Num(n), e.span()),
        Token::FloatLit(f) = e => (FieldValue::Float(f), e.span()),
        Token::BoolLit(b) = e => (FieldValue::Bool(b), e.span()),
        Token::Ident(s) = e => (FieldValue::Ident(s.to_string()), e.span()),
    };
    template_val.or(other_val)
}

// ─── Alias declaration parser ─────────────────────────────────────────────────

/// Parse an `alias <name> = <base_type>: INDENT field_line+ DEDENT` declaration.
///
/// The returned `AliasNode` carries the raw declaration. Field validation and
/// alias registration are performed by the caller via [`AliasRegistry`].
///
/// Validation errors (E-PAR-006, E-PAR-011) are emitted via the chumsky
/// `validate()` mechanism so they are accumulated with other parse errors.
#[must_use]
pub fn alias_decl<'src, I>(
    file_id: u32,
) -> impl Parser<'src, I, Option<AliasNode>, extra::Err<Rich<'src, Token, TSpan>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = TSpan>,
{
    keyword("alias")
        .ignore_then(any_ident())
        .then_ignore(just(Token::Eq))
        .then(any_ident())
        .then_ignore(just(Token::Colon))
        .then_ignore(just(Token::Newline).or_not())
        .then_ignore(select! { Token::Indent(_) => () })
        .then(
            any_ident()
                .then(alias_field_value())
                .then_ignore(just(Token::Newline).or_not())
                .map(move |((name, name_span), (val, val_span))| FieldNode {
                    name: Spanned::new(name, to_span(name_span, file_id)),
                    value: Spanned::new(val, to_span(val_span, file_id)),
                })
                .recover_with(skip_then_retry_until(
                    any()
                        .filter(|t| !matches!(t, Token::Newline | Token::Dedent))
                        .ignored(),
                    just(Token::Newline).ignored(),
                ))
                .repeated()
                .at_least(1)
                .collect::<Vec<FieldNode>>(),
        )
        .then_ignore(just(Token::Dedent))
        .validate(
            move |(((alias_name, alias_span), (base_type, base_span)), preset_fields), info, emitter| {
                // E-PAR-006: alias name must not collide with a built-in type.
                let all_types = all_slide_types();
                if all_types.contains(&alias_name.as_str()) {
                    emitter.emit(Rich::custom(
                        info.span(),
                        format!(
                            "E-PAR-006: alias name '{alias_name}' collides with a built-in slide type; \
                             choose a different name"
                        ),
                    ));
                    return None;
                }

                // BC-1.09.001 / AC-013: alias name must not collide with a
                // reserved keyword. The keyword registry covers ~45 entries
                // including `raw*` variants (E-PAR-009) and future-reserved
                // identifiers like `component`, `extends`, `macro` (E-PAR-006).
                if let Some((code, desc)) = classify_keyword(&alias_name) {
                    emitter.emit(Rich::custom(
                        info.span(),
                        format!(
                            "{code}: alias name '{alias_name}' collides with reserved keyword \
                             ({desc}); choose a different name"
                        ),
                    ));
                    return None;
                }

                // E-PAR-011: each preset field must be a known field of base_type.
                if let Some(known) = known_fields(&base_type) {
                    for field in &preset_fields {
                        let field_name = field.name.value().as_str();
                        if !known.contains(&field_name) {
                            emitter.emit(Rich::custom(
                                info.span(),
                                format!(
                                    "E-PAR-011: '{field_name}' is not a valid field of slide type '{base_type}'. \
                                     Aliases can only preset existing fields."
                                ),
                            ));
                            // Continue to accumulate more E-PAR-011 errors.
                        }
                    }
                }

                // E-PAR-011 variant for alias of alias (EC-006).
                // If base_type is itself an alias name (not a real type), we reject it
                // at the caller level (AliasRegistry) since we don't have visibility here.

                Some(AliasNode {
                    name: Spanned::new(alias_name, to_span(alias_span, file_id)),
                    base_type: Spanned::new(base_type, to_span(base_span, file_id)),
                    preset_fields,
                })
            },
        )
}

// ─── AliasRegistry ────────────────────────────────────────────────────────────

/// Parse-local registry of alias declarations.
///
/// `AliasRegistry` is created once per deck parse and discarded afterwards.
/// It is NOT stored in `DeckNode` — aliases are expanded eagerly during parse.
///
/// # Usage
///
/// ```rust,ignore
/// let mut registry = AliasRegistry::new();
/// registry.register(alias_node);
/// // When a slide with an alias kind is encountered:
/// if let Some(slide) = registry.expand(&alias_name, explicit_fields, span) {
///     // use slide
/// }
/// ```
#[derive(Debug, Default)]
pub struct AliasRegistry {
    /// Maps alias name → (`base_type`, `preset_fields`).
    aliases: HashMap<String, (String, Vec<FieldNode>)>,
}

impl AliasRegistry {
    /// Construct an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }

    /// Register an alias from an `AliasNode`.
    ///
    /// If the alias name collides with an existing alias, the new definition
    /// overwrites the old one (last-wins semantics — duplicate detection is a
    /// validation concern for STORY-016).
    pub fn register(&mut self, node: AliasNode) {
        self.aliases.insert(
            node.name.value().clone(),
            (node.base_type.value().clone(), node.preset_fields),
        );
    }

    /// Return `true` if `name` is a registered alias.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.aliases.contains_key(name)
    }

    /// Expand a `slide <alias_name>:` into a `SlideNode` with the base type.
    ///
    /// Merges the alias's preset fields with the explicit fields from the
    /// slide body. Explicit fields override preset fields when both name the
    /// same field (BC-1.09.001 postcondition 2, 5).
    ///
    /// Returns `None` if `alias_name` is not in the registry.
    #[must_use]
    pub fn expand(
        &self,
        alias_name: &str,
        explicit_fields: Vec<FieldNode>,
        kind_span: Span,
        _file_id: u32,
    ) -> Option<SlideNode> {
        let (base_type, preset_fields) = self.aliases.get(alias_name)?;

        // Merge: start with preset fields, then override with explicit fields.
        // "explicit wins" means: for fields that appear in both, the explicit
        // value is used.
        let explicit_names: std::collections::HashSet<String> = explicit_fields
            .iter()
            .map(|f| f.name.value().clone())
            .collect();

        let merged_fields: Vec<FieldNode> = preset_fields
            .iter()
            .filter(|f| !explicit_names.contains(f.name.value()))
            .cloned()
            .chain(explicit_fields)
            .collect();

        Some(SlideNode {
            kind: Spanned::new(base_type.clone(), kind_span),
            tags: vec![],
            fields: merged_fields,
            inline_items: vec![],
        })
    }

    /// Validate that `alias_name` is not itself an alias base type for another
    /// alias (EC-006: transitive aliasing is not supported in v1.0).
    ///
    /// Returns `Some(error_message)` if the base type is an alias name.
    #[must_use]
    pub fn check_alias_of_alias(&self, base_type: &str) -> Option<String> {
        if self.aliases.contains_key(base_type) {
            Some(format!(
                "E-PAR-011: alias cannot reference another alias '{base_type}'; \
                 use the base type directly"
            ))
        } else {
            None
        }
    }

    /// Return the zero-length span for error recovery.
    #[must_use]
    pub fn zero_span(file_id: u32) -> Span {
        zero_span(file_id)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ast::{BlockItem, DeckNode, FieldValue};
    use crate::span::SourceMap;
    use crate::template::TemplateChunk;
    use std::sync::Arc;

    fn parse_deck(src: &str) -> Result<DeckNode, Vec<crate::error::SyntaxError>> {
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
        crate::parser::parse(src, file_id, &sm).map(|pr| pr.deck)
    }

    fn dummy_span() -> Span {
        Span::new(0, 0, 0)
    }

    // ── AC-011: alias expansion ───────────────────────────────────────────────

    #[test]
    fn test_bc_1_09_001_alias_expansion() {
        // alias exec_title = title: footer "Internal"
        // slide exec_title: title "Hello"
        // → slide title: footer "Internal", title "Hello"
        let src = concat!(
            "alias exec_title = title:\n",
            "  footer \"Internal\"\n",
            "slide exec_title:\n",
            "  title \"Hello\"\n",
        );
        let result = parse_deck(src);
        let deck = result.expect("alias expansion must succeed");
        assert_eq!(
            deck.items.len(),
            1,
            "must have 1 slide item after expansion"
        );
        let BlockItem::Slide(slide_s) = &deck.items[0] else {
            panic!("expected Slide item after alias expansion");
        };
        let slide = slide_s.value();
        // The slide kind must be the base type, not the alias name.
        assert_eq!(
            slide.kind.value(),
            "title",
            "expanded slide must have base type 'title'"
        );
        // Must have both footer (preset) and title (explicit).
        let footer = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "footer")
            .expect("footer field must be preset from alias");
        assert_eq!(
            footer.value.value(),
            &FieldValue::Template(vec![TemplateChunk::Literal("Internal".to_string())])
        );
        let title = slide
            .fields
            .iter()
            .find(|f| f.name.value() == "title")
            .expect("title field must be explicit");
        assert_eq!(
            title.value.value(),
            &FieldValue::Template(vec![TemplateChunk::Literal("Hello".to_string())])
        );
    }

    // ── AC-012: alias with unknown field → E-PAR-011 ─────────────────────────

    #[test]
    fn test_bc_1_09_002_alias_unknown_field_rejected() {
        let src = concat!(
            "alias bad_alias = title:\n",
            "  new_field \"value\"\n",
            "slide title:\n",
            "  title \"Test\"\n",
        );
        let result = parse_deck(src);
        assert!(
            result.is_err(),
            "alias with unknown field must produce E-PAR-011"
        );
        let errors = result.unwrap_err();
        let has_par011 = errors.iter().any(|e| {
            let msg = e.to_string();
            msg.contains("E-PAR-011") || msg.contains("new_field") || msg.contains("valid field")
        });
        assert!(
            has_par011,
            "error must reference E-PAR-011 or 'new_field'; got: {errors:?}"
        );
    }

    // ── AC-013: alias name collides with built-in → E-PAR-006 ────────────────

    #[test]
    fn test_bc_1_09_001_alias_name_collides_with_builtin() {
        let src = concat!(
            "alias title = title:\n",
            "  footer \"x\"\n",
            "slide content:\n",
            "  title \"Test\"\n",
        );
        let result = parse_deck(src);
        assert!(
            result.is_err(),
            "alias name collision with built-in must produce E-PAR-006"
        );
        let errors = result.unwrap_err();
        let has_par006 = errors.iter().any(|e| {
            let msg = e.to_string();
            msg.contains("E-PAR-006") || msg.contains("collides") || msg.contains("built-in")
        });
        assert!(
            has_par006,
            "error must reference E-PAR-006 or 'collides'; got: {errors:?}"
        );
    }

    // ── BC-1.09.001: alias name collides with reserved keyword ─────────────

    #[test]
    fn test_bc_1_09_001_alias_raw_name_rejected() {
        // `alias raw = title:` must be rejected — `raw` is a reserved keyword.
        let src = concat!(
            "slideforge_version \"1\"\n",
            "alias raw = title:\n",
            "  footer \"x\"\n",
            "slide title:\n",
            "  title \"Test\"\n",
        );
        let result = parse_deck(src);
        assert!(
            result.is_err(),
            "alias name 'raw' must be rejected as reserved keyword"
        );
        let errors = result.unwrap_err();
        let has_keyword_err = errors.iter().any(|e| {
            let msg = e.to_string();
            msg.contains("E-PAR-009") || msg.contains("reserved keyword")
        });
        assert!(
            has_keyword_err,
            "error must reference reserved keyword collision; got: {errors:?}"
        );
    }

    #[test]
    fn test_bc_1_09_001_alias_component_name_rejected() {
        // `alias component = title:` must be rejected — `component` is reserved (v2+).
        let src = concat!(
            "slideforge_version \"1\"\n",
            "alias component = title:\n",
            "  footer \"x\"\n",
            "slide title:\n",
            "  title \"Test\"\n",
        );
        let result = parse_deck(src);
        assert!(
            result.is_err(),
            "alias name 'component' must be rejected as reserved keyword"
        );
        let errors = result.unwrap_err();
        let has_keyword_err = errors.iter().any(|e| {
            let msg = e.to_string();
            msg.contains("E-PAR-006") || msg.contains("reserved keyword")
        });
        assert!(
            has_keyword_err,
            "error must reference E-PAR-006 for reserved keyword; got: {errors:?}"
        );
    }

    // ── AliasRegistry unit tests ──────────────────────────────────────────────

    #[test]
    fn test_alias_registry_expand() {
        let mut reg = AliasRegistry::new();
        let alias = AliasNode {
            name: Spanned::new("exec_title".to_string(), dummy_span()),
            base_type: Spanned::new("title".to_string(), dummy_span()),
            preset_fields: vec![FieldNode {
                name: Spanned::new("footer".to_string(), dummy_span()),
                value: Spanned::new(
                    FieldValue::Template(vec![TemplateChunk::Literal("Internal".to_string())]),
                    dummy_span(),
                ),
            }],
        };
        reg.register(alias);

        // Expand with an explicit `title` field.
        let explicit_fields = vec![FieldNode {
            name: Spanned::new("title".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Literal("Hello".to_string())]),
                dummy_span(),
            ),
        }];
        let slide = reg
            .expand("exec_title", explicit_fields, dummy_span(), 0)
            .expect("expand must succeed for registered alias");
        assert_eq!(slide.kind.value(), "title");
        assert_eq!(slide.fields.len(), 2);
        // footer is preset; title is explicit.
        let has_footer = slide.fields.iter().any(|f| f.name.value() == "footer");
        let has_title = slide.fields.iter().any(|f| f.name.value() == "title");
        assert!(has_footer, "expanded slide must have preset 'footer'");
        assert!(has_title, "expanded slide must have explicit 'title'");
    }

    #[test]
    fn test_alias_registry_explicit_overrides_preset() {
        let mut reg = AliasRegistry::new();
        let alias = AliasNode {
            name: Spanned::new("exec_title".to_string(), dummy_span()),
            base_type: Spanned::new("title".to_string(), dummy_span()),
            preset_fields: vec![FieldNode {
                name: Spanned::new("footer".to_string(), dummy_span()),
                value: Spanned::new(
                    FieldValue::Template(vec![TemplateChunk::Literal("Preset".to_string())]),
                    dummy_span(),
                ),
            }],
        };
        reg.register(alias);

        // Explicit `footer` overrides the preset.
        let explicit_fields = vec![FieldNode {
            name: Spanned::new("footer".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Literal("Override".to_string())]),
                dummy_span(),
            ),
        }];
        let slide = reg
            .expand("exec_title", explicit_fields, dummy_span(), 0)
            .expect("expand must succeed");
        // Only one footer field — the explicit override.
        let footer_fields: Vec<_> = slide
            .fields
            .iter()
            .filter(|f| f.name.value() == "footer")
            .collect();
        assert_eq!(
            footer_fields.len(),
            1,
            "must have exactly one footer field (explicit override)"
        );
        assert_eq!(
            footer_fields[0].value.value(),
            &FieldValue::Template(vec![TemplateChunk::Literal("Override".to_string())])
        );
    }

    #[test]
    fn test_alias_registry_unknown_name_returns_none() {
        let reg = AliasRegistry::new();
        let result = reg.expand("nonexistent", vec![], dummy_span(), 0);
        assert!(result.is_none(), "unknown alias must return None");
    }

    #[test]
    fn test_alias_registry_contains() {
        let mut reg = AliasRegistry::new();
        assert!(!reg.contains("exec_title"));
        let alias = AliasNode {
            name: Spanned::new("exec_title".to_string(), dummy_span()),
            base_type: Spanned::new("title".to_string(), dummy_span()),
            preset_fields: vec![],
        };
        reg.register(alias);
        assert!(reg.contains("exec_title"));
        assert!(!reg.contains("other"));
    }

    #[test]
    fn test_alias_of_alias_detection() {
        let mut reg = AliasRegistry::new();
        let alias = AliasNode {
            name: Spanned::new("my_alias".to_string(), dummy_span()),
            base_type: Spanned::new("title".to_string(), dummy_span()),
            preset_fields: vec![],
        };
        reg.register(alias);

        // Attempting to use "my_alias" as a base type for another alias
        // should be detected.
        let err = reg.check_alias_of_alias("my_alias");
        assert!(err.is_some(), "alias-of-alias must be detected; got None");
        let err_msg = err.unwrap();
        assert!(
            err_msg.contains("E-PAR-011") || err_msg.contains("alias"),
            "message must reference E-PAR-011 or 'alias'; got: {err_msg}"
        );

        // A non-alias base type is fine.
        let ok = reg.check_alias_of_alias("title");
        assert!(
            ok.is_none(),
            "real type must not trigger alias-of-alias check"
        );
    }
}
