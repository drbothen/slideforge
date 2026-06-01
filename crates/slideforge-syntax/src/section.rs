//! Section-block DSL constants and helpers.
//!
//! Defines the compile-time set of recognised register sub-block keys for
//! `section <type>:` blocks, and exposes helper functions used by the parser
//! and (optionally) by the evaluator to classify section sub-block identifiers.
//!
//! # Register Sub-block Keys
//!
//! A section block may contain two register sub-blocks:
//!
//! - `report:` — reader-narrative register (document-mode readers)
//! - `detail:` — extended-analysis register (document-only)
//!
//! The `notes:` register is intentionally **excluded** — `notes` is the
//! presenter register for slides. Document sections have no slide canvas or
//! speaker view, so `notes:` has no semantic meaning inside a section block.
//! Per DIR-077-001 §5: a `notes:` key inside a section block is treated as an
//! *unrecognised sub-block key* and produces a parse-time non-fatal warning.
//!
//! # STORY-078
//!
//! Introduced in STORY-078 (Parser: section block syntax). The STORY-008 stub
//! for `SECTION_REGISTER_KEYS` incorrectly included `"notes"` as a recognised
//! key. The live constant (defined at module scope below) is corrected to
//! `["report", "detail"]` per DIR-077-001 §5, which excludes `"notes"` because
//! document sections carry no slide canvas or speaker view.

/// The recognised register sub-block keys for `section <type>:` blocks.
///
/// Any sub-block key NOT in this set is an unrecognised key, which produces a
/// non-fatal parse-time warning (`W-PAR-001`) emitted by `section_block_parser`.
///
/// `"notes"` is explicitly excluded — see module-level documentation.
pub const SECTION_REGISTER_KEYS: &[&str] = &["report", "detail"];

/// Return `true` if `key` is a recognised register sub-block key for section
/// blocks.
///
/// Returns `true` only for `"report"` and `"detail"`. Returns `false` for
/// `"notes"` (per DIR-077-001 §5) and for any other key.
///
/// # Examples
///
/// ```
/// use slideforge_syntax::section::is_register_sub_block_key;
///
/// assert!(is_register_sub_block_key("report"));
/// assert!(is_register_sub_block_key("detail"));
/// assert!(!is_register_sub_block_key("notes")); // excluded per DIR-077-001 §5
/// assert!(!is_register_sub_block_key("foo"));
/// ```
#[must_use]
pub fn is_register_sub_block_key(key: &str) -> bool {
    SECTION_REGISTER_KEYS.contains(&key)
}

/// Return `true` if `key` is a reserved register name that is syntactically
/// valid only with the register `:` suffix syntax.
///
/// This is used by `section_block_parser` to detect EC-006: a sub-block key
/// that matches a register name but is used WITHOUT the colon suffix (i.e.,
/// written as a plain scalar field `report "..."` instead of the multi-line
/// register sub-block `report:\n  "..."`). When this check fires, the parser
/// emits a FATAL error with a corrective hint.
///
/// The reserved register names are `["report", "detail", "notes"]` — note that
/// `notes` is included here even though it is not valid on sections, because a
/// user who writes `notes "..."` (without colon) on a section is almost
/// certainly confused and deserves a specific corrective message rather than a
/// generic parse error.
///
/// # Examples
///
/// ```
/// use slideforge_syntax::section::is_reserved_register_name;
///
/// assert!(is_reserved_register_name("report"));
/// assert!(is_reserved_register_name("detail"));
/// assert!(is_reserved_register_name("notes")); // reserved name (even though invalid on sections)
/// assert!(!is_reserved_register_name("foo"));
/// assert!(!is_reserved_register_name("title"));
/// ```
#[must_use]
pub fn is_reserved_register_name(key: &str) -> bool {
    matches!(key, "report" | "detail" | "notes")
}

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_register_keys_exclude_notes() {
        assert!(
            !is_register_sub_block_key("notes"),
            "'notes' must NOT be a register key"
        );
        assert!(
            is_register_sub_block_key("report"),
            "'report' must be a register key"
        );
        assert!(
            is_register_sub_block_key("detail"),
            "'detail' must be a register key"
        );
    }

    #[test]
    fn test_register_keys_unknown_returns_false() {
        assert!(!is_register_sub_block_key("foo"));
        assert!(!is_register_sub_block_key("body"));
        assert!(!is_register_sub_block_key("title"));
        assert!(!is_register_sub_block_key(""));
    }

    #[test]
    fn test_reserved_register_name_includes_notes() {
        assert!(is_reserved_register_name("report"));
        assert!(is_reserved_register_name("detail"));
        assert!(is_reserved_register_name("notes"));
        assert!(!is_reserved_register_name("foo"));
    }
}
