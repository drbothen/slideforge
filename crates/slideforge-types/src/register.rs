//! Writing registers — the three linguistically distinct output registers.
//!
//! slideforge supports three writing registers that let authors annotate
//! content with its intended audience and output format:
//!
//! - [`Register::Notes`] — presenter notes (speaker guidance, not visible in handouts)
//! - [`Register::Report`] — reader-oriented prose (included in document exports)
//! - [`Register::Detail`] — deep-detail content (document-only; excluded from slide view)
//!
//! The ordering `Notes < Report < Detail` reflects increasing verbosity /
//! document-orientation. Exporters use this ordering to decide which content
//! to include at each output fidelity level.

/// A writing register identifies the intended audience and output fidelity
/// of a content block.
///
/// # Ordering
///
/// `Notes < Report < Detail` — increasing verbosity / document-orientation.
/// This ordering is used by exporters to filter content at each fidelity level.
///
/// # Examples
///
/// ```
/// use slideforge_types::Register;
/// assert!(Register::Notes < Register::Report);
/// assert!(Register::Report < Register::Detail);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Register {
    /// Presenter notes. Visible in speaker-view; excluded from handouts and
    /// most document exports.
    Notes,

    /// Reader-oriented prose. Included in document exports (DOCX, PDF) but
    /// may be condensed or excluded from slide view.
    Report,

    /// Deep-detail content. Included in document-only exports; always excluded
    /// from slide view and standard handouts.
    Detail,
}

impl Register {
    /// Return `true` if this register is [`Register::Notes`].
    #[must_use]
    pub fn is_notes(self) -> bool {
        matches!(self, Register::Notes)
    }

    /// Return `true` if this register is [`Register::Report`].
    #[must_use]
    pub fn is_report(self) -> bool {
        matches!(self, Register::Report)
    }

    /// Return `true` if this register is [`Register::Detail`].
    #[must_use]
    pub fn is_detail(self) -> bool {
        matches!(self, Register::Detail)
    }

    /// Return the canonical DSL keyword for this register.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::Register;
    /// assert_eq!(Register::Notes.as_keyword(), "notes");
    /// assert_eq!(Register::Report.as_keyword(), "report");
    /// assert_eq!(Register::Detail.as_keyword(), "detail");
    /// ```
    #[must_use]
    pub fn as_keyword(self) -> &'static str {
        match self {
            Register::Notes => "notes",
            Register::Report => "report",
            Register::Detail => "detail",
        }
    }
}

impl Default for Register {
    /// The default register is [`Register::Notes`] — the most conservative
    /// choice; content defaults to presenter-only unless explicitly promoted.
    fn default() -> Self {
        Register::Notes
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_keyword())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ──────────────────────────────────────────────────────────────────────────
    // AC-007 — Register has 3 variants, implements Hash+Eq+Clone+Copy+Debug+Ord
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_001_register_ordering() {
        assert!(Register::Notes < Register::Report);
        assert!(Register::Report < Register::Detail);
        assert!(Register::Notes < Register::Detail);
    }

    #[test]
    fn test_bc_1_01_001_register_equality() {
        assert_eq!(Register::Notes, Register::Notes);
        assert_ne!(Register::Notes, Register::Report);
    }

    #[test]
    fn test_bc_1_01_001_register_copy() {
        let r = Register::Report;
        let r2 = r; // Copy — no move
        let r3 = r; // Still usable (Copy)
        assert_eq!(r2, Register::Report);
        assert_eq!(r3, Register::Report);
    }

    #[test]
    fn test_bc_1_01_001_register_clone() {
        let r = Register::Detail;
        assert_eq!(r.clone(), Register::Detail);
    }

    #[test]
    fn test_bc_1_01_001_register_debug() {
        let s = format!("{:?}", Register::Notes);
        assert!(s.contains("Notes"));
    }

    #[test]
    fn test_bc_1_01_001_register_hash() {
        use std::collections::HashMap;
        let mut map: HashMap<Register, &str> = HashMap::new();
        map.insert(Register::Notes, "notes");
        map.insert(Register::Report, "report");
        map.insert(Register::Detail, "detail");
        assert_eq!(map.len(), 3);
        assert_eq!(map[&Register::Notes], "notes");
    }

    #[test]
    fn test_bc_1_01_001_register_keywords() {
        assert_eq!(Register::Notes.as_keyword(), "notes");
        assert_eq!(Register::Report.as_keyword(), "report");
        assert_eq!(Register::Detail.as_keyword(), "detail");
    }

    #[test]
    fn test_bc_1_01_001_register_predicates() {
        assert!(Register::Notes.is_notes());
        assert!(!Register::Notes.is_report());
        assert!(Register::Report.is_report());
        assert!(Register::Detail.is_detail());
    }

    #[test]
    fn test_bc_1_01_001_register_default() {
        assert_eq!(Register::default(), Register::Notes);
    }

    #[test]
    fn test_bc_1_01_001_register_display() {
        assert_eq!(Register::Notes.to_string(), "notes");
        assert_eq!(Register::Report.to_string(), "report");
        assert_eq!(Register::Detail.to_string(), "detail");
    }

    #[test]
    fn test_bc_1_01_001_register_sorted() {
        let mut registers = vec![Register::Detail, Register::Notes, Register::Report];
        registers.sort();
        assert_eq!(
            registers,
            vec![Register::Notes, Register::Report, Register::Detail]
        );
    }
}
