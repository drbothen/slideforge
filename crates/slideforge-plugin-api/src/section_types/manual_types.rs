//! Manual-only bundled section type implementations.
//!
//! The five types in this module — `methodology`, `scope`, `approval`,
//! `appendix`, and `glossary` — are **purely manual** section constructs.
//! Their content is authored by the DSL user inside `section <type>:` blocks
//! and is not derivable from slide data. Therefore, each implementation's
//! `generate()` method always returns an empty `Vec<SectionBlock>`.
//!
//! This contrasts with the two auto-generated types (`executive_summary` and
//! `risk_register`) in their respective sibling modules, which scan slide
//! content to produce structural output.
//!
//! All five types are registered by the `PluginRegistryBuilder` (STORY-049).

use slideforge_types::Slide;

use crate::traits::section_type::{SectionBlock, SectionType};

// ─────────────────────────────────────────────────────────────────────────────
// MethodologySectionType
// ─────────────────────────────────────────────────────────────────────────────

/// A [`SectionType`] plugin for the `methodology` section.
///
/// Content is authored manually via `section methodology:` DSL blocks.
/// `generate()` always returns `vec![]` because methodology sections carry no
/// structure derivable from slide data.
///
/// `id()` returns `"methodology"`.
#[derive(Debug, Default)]
pub struct MethodologySectionType;

impl SectionType for MethodologySectionType {
    /// Returns `"methodology"`.
    fn id(&self) -> &'static str {
        "methodology"
    }

    /// Always returns an empty `Vec` — methodology content is author-supplied,
    /// not derived from slides.
    fn generate(&self, _slides: &[Slide]) -> Vec<SectionBlock> {
        vec![]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ScopeSectionType
// ─────────────────────────────────────────────────────────────────────────────

/// A [`SectionType`] plugin for the `scope` section.
///
/// Content is authored manually via `section scope:` DSL blocks.
/// `generate()` always returns `vec![]` because scope sections carry no
/// structure derivable from slide data.
///
/// `id()` returns `"scope"`.
#[derive(Debug, Default)]
pub struct ScopeSectionType;

impl SectionType for ScopeSectionType {
    /// Returns `"scope"`.
    fn id(&self) -> &'static str {
        "scope"
    }

    /// Always returns an empty `Vec` — scope content is author-supplied,
    /// not derived from slides.
    fn generate(&self, _slides: &[Slide]) -> Vec<SectionBlock> {
        vec![]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ApprovalSectionType
// ─────────────────────────────────────────────────────────────────────────────

/// A [`SectionType`] plugin for the `approval` section.
///
/// Content is authored manually via `section approval:` DSL blocks.
/// `generate()` always returns `vec![]` because approval sections carry no
/// structure derivable from slide data.
///
/// `id()` returns `"approval"`.
#[derive(Debug, Default)]
pub struct ApprovalSectionType;

impl SectionType for ApprovalSectionType {
    /// Returns `"approval"`.
    fn id(&self) -> &'static str {
        "approval"
    }

    /// Always returns an empty `Vec` — approval content is author-supplied,
    /// not derived from slides.
    fn generate(&self, _slides: &[Slide]) -> Vec<SectionBlock> {
        vec![]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AppendixSectionType
// ─────────────────────────────────────────────────────────────────────────────

/// A [`SectionType`] plugin for the `appendix` section.
///
/// Content is authored manually via `section appendix:` DSL blocks.
/// `generate()` always returns `vec![]` because appendix sections carry no
/// structure derivable from slide data.
///
/// `id()` returns `"appendix"`.
#[derive(Debug, Default)]
pub struct AppendixSectionType;

impl SectionType for AppendixSectionType {
    /// Returns `"appendix"`.
    fn id(&self) -> &'static str {
        "appendix"
    }

    /// Always returns an empty `Vec` — appendix content is author-supplied,
    /// not derived from slides.
    fn generate(&self, _slides: &[Slide]) -> Vec<SectionBlock> {
        vec![]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GlossarySectionType
// ─────────────────────────────────────────────────────────────────────────────

/// A [`SectionType`] plugin for the `glossary` section.
///
/// Content is authored manually via `section glossary:` DSL blocks.
/// `generate()` always returns `vec![]` because glossary sections carry no
/// structure derivable from slide data.
///
/// `id()` returns `"glossary"`.
#[derive(Debug, Default)]
pub struct GlossarySectionType;

impl SectionType for GlossarySectionType {
    /// Returns `"glossary"`.
    fn id(&self) -> &'static str {
        "glossary"
    }

    /// Always returns an empty `Vec` — glossary content is author-supplied,
    /// not derived from slides.
    fn generate(&self, _slides: &[Slide]) -> Vec<SectionBlock> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slideforge_types::{OrderedMap, SourceSpan};
    use std::sync::Arc;

    /// Build a minimal slide with no fields.
    fn blank_slide() -> Slide {
        Slide {
            slide_type: Arc::from("bullets"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-5.02.001: AC-004 — All 5 manual-only types return vec![]
    // ─────────────────────────────────────────────────────────────────────────

    /// `MethodologySectionType::id()` returns `"methodology"`.
    #[test]
    fn test_bc_5_02_001_methodology_id() {
        assert_eq!(MethodologySectionType.id(), "methodology");
    }

    /// `MethodologySectionType::generate()` always returns empty Vec.
    #[test]
    fn test_bc_5_02_001_methodology_generate_always_empty() {
        let slides = vec![blank_slide(), blank_slide()];
        assert!(MethodologySectionType.generate(&slides).is_empty());
    }

    /// `MethodologySectionType::generate()` is empty for an empty slice.
    #[test]
    fn test_bc_5_02_001_methodology_generate_empty_slice() {
        assert!(MethodologySectionType.generate(&[]).is_empty());
    }

    /// `ScopeSectionType::id()` returns `"scope"`.
    #[test]
    fn test_bc_5_02_001_scope_id() {
        assert_eq!(ScopeSectionType.id(), "scope");
    }

    /// `ScopeSectionType::generate()` always returns empty Vec.
    #[test]
    fn test_bc_5_02_001_scope_generate_always_empty() {
        let slides = vec![blank_slide()];
        assert!(ScopeSectionType.generate(&slides).is_empty());
    }

    /// `ScopeSectionType::generate()` is empty for an empty slice.
    #[test]
    fn test_bc_5_02_001_scope_generate_empty_slice() {
        assert!(ScopeSectionType.generate(&[]).is_empty());
    }

    /// `ApprovalSectionType::id()` returns `"approval"`.
    #[test]
    fn test_bc_5_02_001_approval_id() {
        assert_eq!(ApprovalSectionType.id(), "approval");
    }

    /// `ApprovalSectionType::generate()` always returns empty Vec.
    #[test]
    fn test_bc_5_02_001_approval_generate_always_empty() {
        let slides = vec![blank_slide(), blank_slide(), blank_slide()];
        assert!(ApprovalSectionType.generate(&slides).is_empty());
    }

    /// `ApprovalSectionType::generate()` is empty for an empty slice.
    #[test]
    fn test_bc_5_02_001_approval_generate_empty_slice() {
        assert!(ApprovalSectionType.generate(&[]).is_empty());
    }

    /// `AppendixSectionType::id()` returns `"appendix"`.
    #[test]
    fn test_bc_5_02_001_appendix_id() {
        assert_eq!(AppendixSectionType.id(), "appendix");
    }

    /// `AppendixSectionType::generate()` always returns empty Vec.
    #[test]
    fn test_bc_5_02_001_appendix_generate_always_empty() {
        let slides = vec![blank_slide()];
        assert!(AppendixSectionType.generate(&slides).is_empty());
    }

    /// `AppendixSectionType::generate()` is empty for an empty slice.
    #[test]
    fn test_bc_5_02_001_appendix_generate_empty_slice() {
        assert!(AppendixSectionType.generate(&[]).is_empty());
    }

    /// `GlossarySectionType::id()` returns `"glossary"`.
    #[test]
    fn test_bc_5_02_001_glossary_id() {
        assert_eq!(GlossarySectionType.id(), "glossary");
    }

    /// `GlossarySectionType::generate()` always returns empty Vec.
    #[test]
    fn test_bc_5_02_001_glossary_generate_always_empty() {
        let slides = vec![blank_slide(), blank_slide()];
        assert!(GlossarySectionType.generate(&slides).is_empty());
    }

    /// `GlossarySectionType::generate()` is empty for an empty slice.
    #[test]
    fn test_bc_5_02_001_glossary_generate_empty_slice() {
        assert!(GlossarySectionType.generate(&[]).is_empty());
    }
}
