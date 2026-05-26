//! The two-IR model — [`Deck`] (semantic) and [`LaidOutDeck`] (geometric).
//!
//! The slideforge pipeline transforms a parsed `.sf` file into:
//!
//! 1. A [`Deck`] — semantic, pre-layout. Content with structure; no positions.
//! 2. A [`LaidOutDeck`] — geometric, post-layout. Positioned shapes with EMU coords.
//!
//! Exporters consume both: PPTX uses the semantic IR for placeholder injection;
//! PDF/HTML operate primarily from the laid-out IR.

use std::sync::Arc;

use crate::block::Block;
use crate::emu::Emu;
use crate::inline::InlineNode;
use crate::ordered_map::OrderedMap;
use crate::register::Register;
use crate::slide::Slide;
use crate::value::Value;

/// Deck-level metadata.
///
/// Metadata lives in the deck frontmatter and applies to the entire document.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeckMetadata {
    /// The deck title (for the document title bar and DOCX cover page).
    pub title: Arc<str>,

    /// The slideforge DSL version used to compile this deck.
    pub slideforge_version: Arc<str>,

    /// The primary language of the deck content (BCP-47, e.g., `"en-US"`).
    /// Required at deck level (accessibility rule).
    pub lang: Arc<str>,

    /// The deck author (for PDF metadata and DOCX properties).
    pub author: Option<Arc<str>>,
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
}

/// The semantic role of a laid-out element within its parent slide.
///
/// Layout engines and exporters use `SemanticRole` to decide which PPTX
/// placeholder type (`ph type`) or PDF tag to apply.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticRole {
    /// The primary slide title.
    Title,
    /// A subtitle or deck title on a title-slide.
    Subtitle,
    /// Body content placeholder.
    Body,
    /// A speaker notes placeholder (presenter view).
    Notes,
    /// A media element (image, chart, diagram).
    Media,
    /// A custom / uncategorized element.
    Custom(Arc<str>),
}

/// The content of a positioned layout element.
///
/// `LaidOutContent` pairs inline or block content with its laid-out position.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LaidOutContent {
    /// A sequence of inline nodes (text run).
    Inlines(Vec<InlineNode>),
    /// A reference to a block in the semantic IR (chart, diagram, image, etc.).
    BlockRef(usize),
    /// An empty placeholder (present in the layout but no content yet assigned).
    Empty,
}

/// A positioned element within a laid-out slide.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LaidOutElement {
    /// Horizontal position from slide left edge.
    pub x: Emu,
    /// Vertical position from slide top edge.
    pub y: Emu,
    /// Element width.
    pub width: Emu,
    /// Element height.
    pub height: Emu,
    /// The semantic role of this element.
    pub role: SemanticRole,
    /// The content to render in this element.
    pub content: LaidOutContent,
}

/// A single slide in the geometric, post-layout IR.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LaidOutSlide {
    /// Slide width (typically [`crate::emu::SLIDE_WIDTH`]).
    pub width: Emu,
    /// Slide height (typically [`crate::emu::SLIDE_HEIGHT`]).
    pub height: Emu,
    /// The positioned elements on this slide.
    pub elements: Vec<LaidOutElement>,
    /// A back-reference to the semantic slide index in [`Deck::slides`].
    pub slide_index: usize,
}

/// The geometric, post-layout intermediate representation of a presentation.
///
/// `LaidOutDeck` is produced by the layout engine from a [`Deck`]. It carries
/// precise EMU coordinates for every element on every slide.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LaidOutDeck {
    /// The laid-out slides, one per entry in [`Deck::slides`].
    pub slides: Vec<LaidOutSlide>,
    /// Back-reference to the semantic IR.
    pub semantic: Deck,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emu::{SLIDE_HEIGHT, SLIDE_WIDTH};
    use crate::register::Register;
    use std::sync::Arc;

    fn make_metadata() -> DeckMetadata {
        DeckMetadata {
            title: Arc::from("Test Deck"),
            slideforge_version: Arc::from("0.1.0"),
            lang: Arc::from("en-US"),
            author: None,
        }
    }

    fn make_deck() -> Deck {
        Deck {
            slides: vec![],
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
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
        assert_eq!(deck.metadata.title.as_ref(), "Test Deck");
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
        assert_eq!(meta.title.as_ref(), "Test Deck");
        assert_eq!(meta.slideforge_version.as_ref(), "0.1.0");
        assert_eq!(meta.lang.as_ref(), "en-US");
        assert!(meta.author.is_none());
    }

    #[test]
    fn test_bc_1_01_010_deck_metadata_with_author() {
        let meta = DeckMetadata {
            title: Arc::from("My Deck"),
            slideforge_version: Arc::from("0.1.0"),
            lang: Arc::from("en-US"),
            author: Some(Arc::from("Jane Doe")),
        };
        assert_eq!(meta.author.as_deref(), Some("Jane Doe"));
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

    // LaidOutDeck tests

    #[test]
    fn test_bc_1_01_001_laid_out_deck_fields() {
        let deck = make_deck();
        let lod = LaidOutDeck {
            slides: vec![LaidOutSlide {
                width: SLIDE_WIDTH,
                height: SLIDE_HEIGHT,
                elements: vec![],
                slide_index: 0,
            }],
            semantic: deck,
        };
        assert_eq!(lod.slides.len(), 1);
        assert_eq!(lod.slides[0].width, SLIDE_WIDTH);
    }

    #[test]
    fn test_bc_1_01_001_laid_out_element_fields() {
        let elem = LaidOutElement {
            x: Emu(0),
            y: Emu(0),
            width: SLIDE_WIDTH,
            height: Emu(914_400), // 1 inch
            role: SemanticRole::Title,
            content: LaidOutContent::Empty,
        };
        assert_eq!(elem.role, SemanticRole::Title);
        assert_eq!(elem.content, LaidOutContent::Empty);
    }

    #[test]
    fn test_bc_1_01_001_semantic_role_custom() {
        let role = SemanticRole::Custom(Arc::from("logo"));
        match &role {
            SemanticRole::Custom(s) => assert_eq!(s.as_ref(), "logo"),
            _ => panic!("expected Custom"),
        }
    }

    #[test]
    fn test_bc_1_01_001_deck_vars_field() {
        let mut deck = make_deck();
        deck.vars.insert(Arc::from("company"), Value::Str(Arc::from("Acme Corp")));
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
