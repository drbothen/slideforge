//! Built-in slide type implementations and the [`SlideTypeRegistry`].
//!
//! This module provides:
//!
//! 1. **[`SlideTypeRegistry`]** — runtime dispatch from DSL keyword to
//!    [`SlideType`] implementation, with typo suggestions.
//! 2. **[`validate_fields`]** — accumulates all field-validation diagnostics
//!    for a slide against its declared type schema.
//! 3. **Individual slide type structs** — one per built-in type keyword.
//!    Currently 4 of 31 are implemented (Red Gate state); the implementer
//!    adds the remaining 27 as part of STORY-003.
//! 4. **[`SLIDE_TYPE_REGISTRY`]** — a process-wide lazy singleton holding the
//!    default registry for use by the evaluator and layout engine.
//!
//! ## Adding a new slide type
//!
//! 1. Create `crates/slideforge-types/src/slide_types/<name>.rs`
//! 2. Implement `SlideType` for the new struct
//! 3. `pub mod <name>;` in this file
//! 4. Add `r.register(Box::new(<Name>SlideType::new()))` in
//!    [`SlideTypeRegistry::default`]
//! 5. Update the assertion in `test_bc_1_03_017_all_keywords_len_equals_31`

use std::sync::LazyLock;

pub mod blank;
pub mod content;
pub mod registry;
pub mod stat_callout;
pub mod title;
// NOTE: The implementer will add the remaining 27 type modules here.
// Do NOT add todo!()-body modules — add them only when the implementation
// is complete and tests pass.

pub use registry::{SlideTypeRegistry, validate_fields};

/// Process-wide lazy singleton of the default slide type registry.
///
/// The evaluator and layout engine should use this singleton rather than
/// constructing their own registry. The registry is initialized once and
/// then immutable for the process lifetime.
///
/// # Thread safety
///
/// `LazyLock` guarantees that the registry is initialized exactly once.
/// The contained `SlideTypeRegistry` is not itself `Sync`, but access is
/// read-only after initialization (no mutation after construction).
///
/// For mutable registries (plugin testing, extension points), construct an
/// independent `SlideTypeRegistry` instance.
pub static SLIDE_TYPE_REGISTRY: LazyLock<SlideTypeRegistry> =
    LazyLock::new(SlideTypeRegistry::default);
