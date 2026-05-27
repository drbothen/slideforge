//! [`BrandLoadContext`] — runtime context passed to [`crate::loader::BrandLoader`].

use std::path::PathBuf;

use slideforge_types::SourceSpan;

/// Runtime context for a brand loading operation.
///
/// Passed to [`crate::loader::BrandLoader::load_template`] alongside the template path.
/// Controls font availability checking and provides the root directory for
/// resolving relative paths.
#[derive(Debug, Clone)]
pub struct BrandLoadContext {
    /// When `true`, the loader checks whether each declared font is installed on
    /// the build host and emits [`crate::error::BrandError::FontUnavailable`]
    /// warnings for any that are not found (BC-2.01.006).
    ///
    /// Set to `false` in unit tests that do not exercise font availability.
    pub check_font_availability: bool,

    /// The root directory of the slideforge workspace.
    ///
    /// Used to resolve relative paths in `brand "..."` declarations.
    pub root_dir: PathBuf,

    /// Source location of the `brand "..."` declaration in the `.sf` file.
    pub span: SourceSpan,
}

impl BrandLoadContext {
    /// Create a context suitable for testing.
    ///
    /// `check_font_availability` is `false`; `root_dir` is the current directory.
    #[must_use]
    pub fn for_test() -> Self {
        Self {
            check_font_availability: false,
            root_dir: PathBuf::from("."),
            span: SourceSpan::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc_2_01_001_brand_load_context_for_test() {
        let ctx = BrandLoadContext::for_test();
        assert!(!ctx.check_font_availability);
    }

    #[test]
    fn test_bc_2_01_006_brand_load_context_font_check_flag() {
        let ctx = BrandLoadContext {
            check_font_availability: true,
            root_dir: PathBuf::from("/tmp"),
            span: SourceSpan::default(),
        };
        assert!(ctx.check_font_availability);
    }
}
