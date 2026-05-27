//! Execution context for data source loading.
//!
//! [`DataSourceContext`] carries per-invocation metadata such as the base
//! directory for resolving relative file paths and security policy settings.

use std::path::PathBuf;

/// Per-invocation context passed to data source loaders.
///
/// This context is constructed by the evaluator when it encounters an
/// `@data` directive and passed through to the parser layer for security
/// policy enforcement (e.g., restricting file access to a project root).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSourceContext {
    /// The base directory for resolving relative file paths.
    ///
    /// All relative `@data` URIs are resolved against this path.
    /// Absolute paths that escape this root are rejected with
    /// [`crate::DataError::SsrfBlocked`].
    pub base_dir: PathBuf,

    /// Whether strict path containment is enforced.
    ///
    /// When `true` (the default), file paths must stay within `base_dir`.
    /// When `false`, any readable path is allowed (useful for tests).
    pub strict: bool,
}

impl DataSourceContext {
    /// Construct a new context rooted at `base_dir` with strict enforcement.
    #[must_use]
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        DataSourceContext {
            base_dir: base_dir.into(),
            strict: true,
        }
    }

    /// Construct a permissive context for tests (no path containment check).
    #[must_use]
    pub fn permissive(base_dir: impl Into<PathBuf>) -> Self {
        DataSourceContext {
            base_dir: base_dir.into(),
            strict: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc_5_03_context_new_is_strict() {
        let ctx = DataSourceContext::new("/tmp");
        assert!(ctx.strict);
        assert_eq!(ctx.base_dir, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_bc_5_03_context_permissive_not_strict() {
        let ctx = DataSourceContext::permissive("/tmp");
        assert!(!ctx.strict);
    }
}
