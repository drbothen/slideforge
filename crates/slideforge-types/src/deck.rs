//! The semantic pre-layout IR — [`Deck`], [`DeckMetadata`], and [`SectionBlock`].
//!
//! The slideforge pipeline transforms a parsed `.sf` file into a [`Deck`] —
//! the semantic, pre-layout intermediate representation. Geometric layout
//! is performed by `slideforge-layout`, which produces `LaidOutDeck`.
//!
//! Exporters receive both IRs: the semantic `Deck` for placeholder resolution
//! and the geometric `LaidOutDeck` for rendering.

use std::sync::Arc;

use crate::block::Block;
use crate::ordered_map::OrderedMap;
use crate::register::Register;
use crate::slide::Slide;
use crate::span::SourceSpan;
use crate::value::Value;

/// A manually authored document section block from `section <type>:` DSL syntax.
///
/// `SectionBlock` captures a `section <type>: ...` declaration at the deck
/// level. It is stored on [`Deck::section_blocks`] and consumed by the layout
/// engine's section collection pass (BC-3.02.002).
///
/// ## Supported section types
///
/// `executive_summary`, `risk_register`, `methodology`, `scope`, `approval`,
/// `appendix`, `glossary`. An unrecognised type name produces
/// `LayoutError::UnknownSectionType` at layout time.
///
/// `executive_summary` and `risk_register` are special: when a manual block
/// with one of these names is present, the layout engine applies the
/// supersession rule (BC-3.02.001 EC-002 / BC-3.02.002 AC-006) and suppresses
/// the corresponding auto-generated section. The remaining five types
/// (`methodology`, `scope`, `approval`, `appendix`, `glossary`) are purely
/// manual sections with no auto-generated equivalent.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionBlock {
    /// The section type name (e.g., `"methodology"`, `"scope"`).
    pub name: Arc<str>,

    /// Section content fields declared inside the block.
    ///
    /// Keys are field names; values are the resolved field values. Uses
    /// [`OrderedMap`] to preserve insertion order (determinism requirement)
    /// and satisfy the `Hash` bound on all IR types.
    ///
    /// NOTE: body uses `Value` (not `FieldValue`) so it cannot represent rich
    /// inline formatting or nested `report:` sub-blocks today. BC-3.02.002
    /// postcondition 1 and EC-004 require these capabilities; full support is
    /// deferred to a future IR-extension story (cross-crate work needed to
    /// introduce `FieldValue::Inlines` and nested `Vec<Block>` in `SectionBlock`).
    pub body: OrderedMap<Arc<str>, Value>,

    /// Source location of the `section <type>:` declaration.
    pub span: SourceSpan,
}

/// Deck-level metadata.
///
/// Metadata lives in the deck frontmatter and applies to the entire document.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeckMetadata {
    /// The deck title (for the document title bar and DOCX cover page).
    ///
    /// `None` when the title is not specified in the frontmatter.
    pub title: Option<Arc<str>>,

    /// The slideforge DSL version used to compile this deck.
    pub slideforge_version: Arc<str>,

    /// The primary language of the deck content (BCP-47, e.g., `"en-US"`).
    ///
    /// `None` when no language tag is provided. The accessibility validator
    /// (STORY-003) enforces that `lang` is present for final export.
    pub lang: Option<Arc<str>>,

    /// The deck author (for PDF metadata and DOCX properties).
    pub author: Option<Arc<str>>,

    /// Explicit section ordering override (AC-007 / BC-3.02.002).
    ///
    /// When `Some`, the layout engine sorts the collected sections to match
    /// the declared order. Sections not listed appear at the end in default
    /// order. Section names must match [`SectionKind`] display names
    /// (`"executive_summary"`, `"risk_register"`) or the manual section type
    /// name (e.g., `"methodology"`).
    pub section_order: Option<Vec<Arc<str>>>,
}

/// The semantic, pre-layout intermediate representation of a presentation.
///
/// A `Deck` is produced by the evaluator from the parsed AST. It carries the
/// resolved slide content with no geometric information.
///
/// ## Design invariant
///
/// All types reachable from `Deck` implement `Hash + Eq + Clone + Debug`.
/// This makes `Deck` suitable as a comemo query input in Phase 3+.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Deck {
    /// The ordered list of slides.
    pub slides: Vec<Slide>,

    /// Top-level variable bindings (`@var name = value`).
    pub vars: OrderedMap<Arc<str>, Value>,

    /// Deck-level metadata.
    pub metadata: DeckMetadata,

    /// Register-gated content blocks at the deck level (e.g., a shared
    /// appendix visible only in the document export).
    pub registers: OrderedMap<Register, Vec<Block>>,

    /// Manually authored section blocks from `section <type>:` DSL syntax
    /// (BC-3.02.002).
    ///
    /// In source order. The layout engine collects these into
    /// [`GeneratedSection`]s with [`SectionSource::ManuallyAuthored`]. An
    /// unrecognised type name returns `LayoutError::UnknownSectionType`.
    pub section_blocks: Vec<SectionBlock>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::Register;
    use std::sync::Arc;

    fn make_metadata() -> DeckMetadata {
        DeckMetadata {
            title: Some(Arc::from("Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        }
    }

    fn make_deck() -> Deck {
        Deck {
            slides: vec![],
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
            section_blocks: vec![],
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // AC-001 — Deck has correct fields, implements Hash+Eq+Clone+Debug
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_deck_fields_present() {
        let deck = make_deck();
        assert!(deck.slides.is_empty());
        assert!(deck.vars.is_empty());
        assert_eq!(deck.metadata.title.as_deref(), Some("Test Deck"));
        assert!(deck.registers.is_empty());
    }

    #[test]
    fn test_bc_1_01_001_deck_clone() {
        let deck = make_deck();
        let deck2 = deck.clone();
        assert_eq!(deck, deck2);
    }

    #[test]
    fn test_bc_1_01_001_deck_eq() {
        let a = make_deck();
        let b = make_deck();
        assert_eq!(a, b);
    }

    #[test]
    fn test_bc_1_01_001_deck_hash() {
        use std::collections::HashMap;
        let mut map: HashMap<Deck, &str> = HashMap::new();
        map.insert(make_deck(), "deck");
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_bc_1_01_001_deck_debug() {
        let deck = make_deck();
        let s = format!("{deck:?}");
        assert!(s.contains("Deck"));
    }

    // ──────────────────────────────────────────────────────────────────────────
    // AC-010 — DeckMetadata fields correct, implements Hash+Eq+Clone+Debug
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_010_deck_metadata_fields() {
        let meta = make_metadata();
        assert_eq!(meta.title.as_deref(), Some("Test Deck"));
        assert_eq!(meta.slideforge_version.as_ref(), "0.1.0");
        assert_eq!(meta.lang.as_deref(), Some("en-US"));
        assert!(meta.author.is_none());
    }

    #[test]
    fn test_bc_1_01_010_deck_metadata_with_author() {
        let meta = DeckMetadata {
            title: Some(Arc::from("My Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: Some(Arc::from("Jane Doe")),
            section_order: None,
        };
        assert_eq!(meta.author.as_deref(), Some("Jane Doe"));
    }

    #[test]
    fn test_bc_1_01_010_deck_metadata_none_title_lang() {
        // AC-010: title and lang are Option — None is valid
        let meta = DeckMetadata {
            title: None,
            slideforge_version: Arc::from("0.1.0"),
            lang: None,
            author: None,
            section_order: None,
        };
        assert!(meta.title.is_none());
        assert!(meta.lang.is_none());
    }

    #[test]
    fn test_bc_1_01_010_deck_metadata_hash() {
        use std::collections::HashSet;
        let meta = make_metadata();
        let mut set: HashSet<DeckMetadata> = HashSet::new();
        set.insert(meta);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_bc_1_01_010_deck_metadata_clone() {
        let meta = make_metadata();
        let meta2 = meta.clone();
        assert_eq!(meta, meta2);
    }

    #[test]
    fn test_bc_1_01_010_deck_metadata_debug() {
        let meta = make_metadata();
        let s = format!("{meta:?}");
        assert!(s.contains("DeckMetadata"));
    }

    #[test]
    fn test_bc_1_01_001_deck_vars_field() {
        let mut deck = make_deck();
        deck.vars
            .insert(Arc::from("company"), Value::Str(Arc::from("Acme Corp")));
        assert_eq!(
            deck.vars.get("company").and_then(Value::as_str),
            Some("Acme Corp")
        );
    }

    #[test]
    fn test_bc_1_01_001_deck_registers_field() {
        let mut deck = make_deck();
        deck.registers.insert(Register::Detail, vec![]);
        assert!(deck.registers.contains_key(&Register::Detail));
    }

    #[test]
    fn test_bc_1_01_001_deck_vars_is_ordered_map() {
        // Verify vars is an OrderedMap (not a plain HashMap).
        let mut deck = make_deck();
        deck.vars.insert(Arc::from("b"), Value::Int(2));
        deck.vars.insert(Arc::from("a"), Value::Int(1));
        // Insertion order must be preserved (b before a).
        let keys: Vec<&str> = deck.vars.keys().map(std::convert::AsRef::as_ref).collect();
        assert_eq!(keys, vec!["b", "a"]);
    }
}
