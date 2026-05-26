//! The `diagram` slide type — a diagram or flowchart slide.
//!
//! A `diagram` slide renders a diagram from Mermaid syntax (or another
//! `DiagramRenderer` plugin) and places it on the slide. The `alt` field
//! is required for WCAG AA compliance.
//!
//! Maps to the `"Title and Content"` PPTX layout.

use std::sync::Arc;

use slideforge_types::{Brand, LaidOutSlide, SLIDE_HEIGHT, SLIDE_WIDTH, Slide};

use crate::traits::{Canvas, FieldDef, LayoutError, SlideType};

/// The built-in `diagram` slide type.
///
/// Required fields: `title`, `diagram`, `alt`.
/// Optional fields: `notes`, `footer`, `logo`, `decorative`, `tags`.
///
/// Maps to the `"Title and Content"` OOXML layout.
#[derive(Debug)]
pub struct DiagramSlideType {
    required: Vec<FieldDef>,
    optional: Vec<FieldDef>,
}

impl DiagramSlideType {
    /// Construct a new `DiagramSlideType` with its canonical field definitions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: vec![
                FieldDef {
                    name: Arc::from("title"),
                    description: Arc::from("The slide title shown above the diagram."),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("diagram"),
                    description: Arc::from(
                        "The diagram source (Mermaid syntax or other supported format). \
                         Provided as a fenced code block in the slide body.",
                    ),
                    required: true,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("alt"),
                    description: Arc::from(
                        "Accessibility alt text describing the diagram and its meaning. \
                         Required for WCAG AA compliance.",
                    ),
                    required: true,
                    default_value: None,
                },
            ],
            optional: vec![
                FieldDef {
                    name: Arc::from("decorative"),
                    description: Arc::from(
                        "When true, marks the diagram as decorative (empty alt in output). \
                         Overrides the `alt` field.",
                    ),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("notes"),
                    description: Arc::from("Presenter notes for this slide."),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("footer"),
                    description: Arc::from("Override footer text for this slide."),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("logo"),
                    description: Arc::from("Override the brand logo for this slide."),
                    required: false,
                    default_value: None,
                },
                FieldDef {
                    name: Arc::from("tags"),
                    description: Arc::from("User-defined tags for filtering and grouping."),
                    required: false,
                    default_value: None,
                },
            ],
        }
    }
}

impl Default for DiagramSlideType {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideType for DiagramSlideType {
    fn id(&self) -> &'static str {
        "diagram"
    }

    fn required_fields(&self) -> &[FieldDef] {
        &self.required
    }

    fn optional_fields(&self) -> &[FieldDef] {
        &self.optional
    }

    fn layout_name(&self) -> &'static str {
        "Title and Content"
    }

    fn lay_out(
        &self,
        _slide: &Slide,
        _brand: &Brand,
        _canvas: Canvas,
    ) -> Result<LaidOutSlide, LayoutError> {
        // Stub: returns an empty LaidOutSlide. Full geometric layout in Phase 3.
        Ok(LaidOutSlide {
            width: SLIDE_WIDTH,
            height: SLIDE_HEIGHT,
            elements: vec![],
            slide_index: 0,
        })
    }
}
