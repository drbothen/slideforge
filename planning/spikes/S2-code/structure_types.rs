// S2 Spike — Structure Type Mapping Sample
//
// Demonstrates how slideforge SemanticRole maps to PDF standard structure element types.
// Throwaway code — for illustrative and proof-of-concept purposes only.
// Production implementation lives in crates/slideforge-pdf/src/tagging.rs (Phase 4).

/// slideforge semantic role for a laid-out element.
///
/// This type is determined during the Layout phase (LaidOutElement.semantic_role)
/// and consumed by the PDF exporter to construct the document structure tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticRole {
    /// Document heading at the given level (1–6).
    Heading { level: u8 },
    /// Body paragraph text.
    Paragraph,
    /// An item in a list (wraps label + body).
    ListItem,
    /// The label part of a list item (bullet, number).
    ListLabel,
    /// Visual figure: image, chart, or diagram. MUST have alt text.
    Figure,
    /// Table container element.
    Table,
    /// A row within a table.
    TableRow,
    /// A header cell within a table row.
    TableHeaderCell,
    /// A data cell within a table row.
    TableDataCell,
    /// Caption for a figure or table.
    Caption,
    /// Inline code span or code block.
    Code,
    /// Purely decorative element — excluded from structure tree, marked as Artifact.
    Decorative,
}

impl SemanticRole {
    /// Returns the PDF standard structure type name per ISO 32000-1 §14.8.4,
    /// or `None` if this role should be marked as an Artifact (not tagged).
    ///
    /// Structure type strings correspond to the `/S` entry in a `StructElem` dictionary.
    pub fn pdf_struct_type(&self) -> Option<&'static str> {
        match self {
            SemanticRole::Heading { level: 1 } => Some("H1"),
            SemanticRole::Heading { level: 2 } => Some("H2"),
            SemanticRole::Heading { level: 3 } => Some("H3"),
            SemanticRole::Heading { level: 4 } => Some("H4"),
            SemanticRole::Heading { level: 5 } => Some("H5"),
            SemanticRole::Heading { level: 6 } => Some("H6"),
            // For levels > 6 (non-standard), fall back to /H per PDF spec §14.8.4.2
            SemanticRole::Heading { .. } => Some("H"),
            SemanticRole::Paragraph => Some("P"),
            SemanticRole::ListItem => Some("LI"),
            SemanticRole::ListLabel => Some("Lbl"),
            SemanticRole::Figure => Some("Figure"),
            SemanticRole::Table => Some("Table"),
            SemanticRole::TableRow => Some("TR"),
            SemanticRole::TableHeaderCell => Some("TH"),
            SemanticRole::TableDataCell => Some("TD"),
            SemanticRole::Caption => Some("Caption"),
            SemanticRole::Code => Some("Code"),
            // Decorative elements are marked as Artifacts — not tagged
            SemanticRole::Decorative => None,
        }
    }

    /// Returns `true` if this element MUST have an `/Alt` entry per PDF/UA-1.
    ///
    /// PDF/UA-1 §7.3: "Actual content that is not text shall be tagged as an artifact
    /// or shall have an alternate text description."
    ///
    /// For slideforge, this applies to Figure elements (images, charts, diagrams).
    /// The DSL already enforces `alt "..."` at compile time (Q6 decision);
    /// this check is a defense-in-depth assertion in the PDF exporter.
    pub fn requires_alt_text(&self) -> bool {
        matches!(self, SemanticRole::Figure)
    }

    /// Returns `true` if this element should be excluded from the structure tree
    /// and instead marked as a PDF Artifact.
    ///
    /// Artifacts are content that does not convey information:
    /// page decorations, background shapes, watermarks, dividers.
    pub fn is_artifact(&self) -> bool {
        matches!(self, SemanticRole::Decorative)
    }
}

/// Validates that a slide's element list satisfies PDF/UA-1 structural requirements.
///
/// Returns a list of violations (empty = valid for structure purposes).
/// This does NOT replace veraPDF validation but provides early feedback.
pub fn validate_slide_structure<'a>(
    elements: &'a [(SemanticRole, Option<&'a str>)], // (role, alt_text)
) -> Vec<String> {
    let mut violations = Vec::new();

    for (role, alt) in elements {
        if role.requires_alt_text() && alt.is_none() {
            violations.push(format!(
                "Figure element missing required /Alt text (PDF/UA-1 §7.3)"
            ));
        }
        if role.requires_alt_text() {
            if let Some(text) = alt {
                if text.trim().is_empty() {
                    violations.push(format!(
                        "Figure element has empty /Alt text — must be meaningful description"
                    ));
                }
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_levels_map_correctly() {
        for level in 1u8..=6 {
            let expected = format!("H{level}");
            let role = SemanticRole::Heading { level };
            assert_eq!(
                role.pdf_struct_type(),
                Some(expected.as_str()),
                "H{level} should map to /H{level}"
            );
        }
    }

    #[test]
    fn heading_beyond_h6_maps_to_h() {
        let role = SemanticRole::Heading { level: 7 };
        assert_eq!(role.pdf_struct_type(), Some("H"));
    }

    #[test]
    fn decorative_is_not_tagged() {
        assert_eq!(SemanticRole::Decorative.pdf_struct_type(), None);
        assert!(SemanticRole::Decorative.is_artifact());
    }

    #[test]
    fn figure_requires_alt_text() {
        assert!(SemanticRole::Figure.requires_alt_text());
    }

    #[test]
    fn non_figure_roles_do_not_require_alt() {
        let non_figures = [
            SemanticRole::Paragraph,
            SemanticRole::Heading { level: 1 },
            SemanticRole::ListItem,
            SemanticRole::Table,
        ];
        for role in &non_figures {
            assert!(
                !role.requires_alt_text(),
                "{:?} should not require alt text",
                role
            );
        }
    }

    #[test]
    fn validate_catches_missing_alt_on_figure() {
        let elements = vec![
            (SemanticRole::Heading { level: 1 }, Some("Slide Title")),
            (SemanticRole::Figure, None), // no alt text — violation
        ];
        let violations = validate_slide_structure(&elements);
        assert!(
            !violations.is_empty(),
            "expected a violation for Figure with no alt text"
        );
    }

    #[test]
    fn validate_catches_empty_alt_on_figure() {
        let elements = vec![(SemanticRole::Figure, Some("  "))]; // whitespace only
        let violations = validate_slide_structure(&elements);
        assert!(
            !violations.is_empty(),
            "expected a violation for Figure with empty alt text"
        );
    }

    #[test]
    fn validate_passes_for_well_formed_slide() {
        let elements = vec![
            (SemanticRole::Heading { level: 1 }, Some("Revenue Growth")),
            (SemanticRole::Figure, Some("Bar chart showing 12% ARR growth YoY")),
            (SemanticRole::Paragraph, Some("Strong quarter with net retention at 115%")),
        ];
        let violations = validate_slide_structure(&elements);
        assert!(
            violations.is_empty(),
            "expected no violations, got: {:?}",
            violations
        );
    }
}
