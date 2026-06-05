//! Built-in section type implementations.
//!
//! This module provides concrete implementations of the
//! [`SectionType`](crate::traits::SectionType) plugin trait for all 7 bundled
//! section types defined in `CANONICAL_MANUAL_SECTION_TYPES`.
//!
//! ## Module structure
//!
//! | Module | Types | Auto-generated? |
//! |--------|-------|-----------------|
//! | [`executive_summary`] | [`ExecutiveSummarySectionType`] | Yes — scans for `takeaway` fields |
//! | [`risk_register`] | [`RiskRegisterSectionType`] | Yes — scans for `severity_cards` slides |
//! | [`manual_types`] | [`MethodologySectionType`], [`ScopeSectionType`], [`ApprovalSectionType`], [`AppendixSectionType`], [`GlossarySectionType`] | No — always `vec![]` |
//!
//! ## Adding a new section type
//!
//! 1. Create `crates/slideforge-plugin-api/src/section_types/<name>.rs`
//! 2. Implement `SectionType` for the new struct
//! 3. Add `pub mod <name>;` in this file
//! 4. Re-export the struct below
//! 5. Register it in `PluginRegistryBuilder` (STORY-049)
//! 6. Add its name to `CANONICAL_MANUAL_SECTION_TYPES` in `slideforge-types`

pub mod executive_summary;
pub mod manual_types;
pub mod risk_register;

pub use executive_summary::ExecutiveSummarySectionType;
pub use manual_types::{
    AppendixSectionType, ApprovalSectionType, GlossarySectionType, MethodologySectionType,
    ScopeSectionType,
};
pub use risk_register::RiskRegisterSectionType;
