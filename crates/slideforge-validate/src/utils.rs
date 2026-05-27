//! Shared validation utility functions.

/// Returns `true` if the string is empty or contains only whitespace.
///
/// Used by validators to determine whether an alt text value is substantively
/// empty. Whitespace-only strings are treated as missing alt text.
///
/// # Examples
///
/// ```
/// use slideforge_validate::is_blank;
/// assert!(is_blank(""));
/// assert!(is_blank("   "));
/// assert!(!is_blank("a description"));
/// ```
#[must_use]
pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc_5_03_015_is_blank_empty() {
        assert!(is_blank(""));
    }

    #[test]
    fn test_bc_5_03_015_is_blank_spaces() {
        assert!(is_blank("   "));
    }

    #[test]
    fn test_bc_5_03_015_is_blank_tabs() {
        assert!(is_blank("\t"));
    }

    #[test]
    fn test_bc_5_03_015_is_blank_newline() {
        assert!(is_blank("\n"));
    }

    #[test]
    fn test_bc_5_03_015_is_blank_mixed_whitespace() {
        assert!(is_blank(" \t\n "));
    }

    #[test]
    fn test_bc_5_03_015_is_blank_content() {
        assert!(!is_blank("hello"));
    }

    #[test]
    fn test_bc_5_03_015_is_blank_mixed_content() {
        assert!(!is_blank(" a "));
    }

    #[test]
    fn test_bc_5_03_015_is_blank_single_char() {
        assert!(!is_blank("x"));
    }
}
