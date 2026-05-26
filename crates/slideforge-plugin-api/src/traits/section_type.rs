//! [`SectionType`] trait — generates document sections from a slide sequence.
//!
//! A `SectionType` plugin takes a sequence of [`Slide`]s from the semantic IR
//! and produces a list of [`SectionBlock`]s that represent the document-level
//! structure (table of contents, chapter markers, appendix headers, etc.).
//!
//! Built-in section types include auto-generated TOC and section dividers.
//! External plugins can register custom section types.

use std::sync::Arc;

use slideforge_types::Slide;

/// A generated document section produced by a [`SectionType`] plugin.
///
/// `SectionBlock` describes a structural document element (e.g., a TOC entry,
/// a chapter header, a divider page). The DOCX and HTML exporters consume these
/// to produce document-level navigation structures.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionBlock {
    /// The section title as it appears in the document outline.
    pub title: Arc<str>,

    /// Optional subtitle or description.
    pub subtitle: Option<Arc<str>>,

    /// The level of this section in the document hierarchy (1 = top-level
    /// chapter, 2 = subsection, 3 = sub-subsection, etc.).
    pub level: u8,

    /// The index of the first slide that belongs to this section, relative to
    /// the `slides` slice passed to [`SectionType::generate`].
    pub start_slide_index: usize,

    /// The index of the slide immediately after the last slide in this section.
    /// `None` means the section extends to the end of the slide sequence.
    pub end_slide_index: Option<usize>,

    /// Whether this section should be included in the table of contents.
    pub include_in_toc: bool,
}

/// A plugin that generates document section structure from a slide sequence.
///
/// The evaluator calls `generate` once per registered section type after all
/// slides are resolved. The returned [`SectionBlock`]s are stored in the
/// [`Deck`] and consumed by exporters to produce TOC, chapter markers, and
/// outline navigation.
///
/// Register implementations with [`crate::PluginRegistry::register_section_type`].
///
/// ## Thread safety
///
/// All implementations must be `Send + Sync`.
///
/// [`Deck`]: slideforge_types::Deck
pub trait SectionType: Send + Sync {
    /// A unique identifier for this section type plugin (e.g., `"toc"`,
    /// `"chapter"`, `"appendix"`).
    fn id(&self) -> &str;

    /// Generate section blocks from `slides`.
    ///
    /// Returning an empty `Vec` means this section type produces no structure
    /// from the given slide sequence (which is valid).
    fn generate(&self, slides: &[Slide]) -> Vec<SectionBlock>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn test_bc_5_02_009_section_type_trait_is_send_sync() {
        assert_send_sync::<dyn SectionType>();
    }

    #[test]
    fn test_bc_5_02_009_section_block_required_fields() {
        let block = SectionBlock {
            title: Arc::from("Introduction"),
            subtitle: None,
            level: 1,
            start_slide_index: 0,
            end_slide_index: Some(5),
            include_in_toc: true,
        };
        assert_eq!(block.title.as_ref(), "Introduction");
        assert_eq!(block.level, 1);
        assert!(block.include_in_toc);
    }

    #[test]
    fn test_bc_5_02_009_section_block_with_subtitle() {
        let block = SectionBlock {
            title: Arc::from("Appendix"),
            subtitle: Some(Arc::from("Supporting data")),
            level: 1,
            start_slide_index: 10,
            end_slide_index: None,
            include_in_toc: false,
        };
        assert!(block.subtitle.is_some());
        assert!(block.end_slide_index.is_none());
        assert!(!block.include_in_toc);
    }

    #[test]
    fn test_bc_5_02_009_section_block_implements_clone() {
        let block = SectionBlock {
            title: Arc::from("Section A"),
            subtitle: None,
            level: 2,
            start_slide_index: 3,
            end_slide_index: Some(7),
            include_in_toc: true,
        };
        let block2 = block.clone();
        assert_eq!(block, block2);
    }

    #[test]
    fn test_bc_5_02_009_section_block_implements_hash() {
        use std::collections::HashSet;
        let block = SectionBlock {
            title: Arc::from("TOC"),
            subtitle: None,
            level: 1,
            start_slide_index: 0,
            end_slide_index: Some(1),
            include_in_toc: true,
        };
        let mut set: HashSet<SectionBlock> = HashSet::new();
        set.insert(block);
        assert_eq!(set.len(), 1);
    }
}
