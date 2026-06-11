//! Error-slide placeholder constructor (STORY-016).
//!
//! When the validation pipeline runs in watch mode and encounters errors, it
//! can inject an error-slide placeholder into the output deck in place of
//! (or alongside) the problematic slide. This gives authors immediate visual
//! feedback in the live preview without blocking the export entirely.
//!
//! The placeholder slide uses the internal slide type `"__error_placeholder__"`
//! which the layout engine and exporters recognise and render as a red-bordered
//! error card.
//!
//! ## Fields set on the error slide
//!
//! | Field | Description |
//! |-------|-------------|
//! | `title` | `"Error: <code>"` — machine-readable error card heading |
//! | `body` | Same as `error_message` — present so `thread_fields_to_blocks` threads the message into the `RegionRole::Body` frame, making it visible in ALL exporters (F-094-P10-001) |
//! | `error_code` | The diagnostic code (e.g., `"E-LAY-002"`) |
//! | `error_message` | Human-readable description of the error |
//! | `source_location` | Source file location (empty until span support is available) |
//! | `position` | 1-based slide position in the deck |
//!
//! ## Rendering mechanism (F-094-P10-001)
//!
//! The `title` and `body` fields are recognised by `thread_fields_to_blocks` and
//! threaded into `ContentBlock::Text` entries before layout runs.  Layout then fills:
//!
//! - `body` → fills the `RegionRole::Body` slot pre-allocated by the
//!   `__error_placeholder__` region map → `FrameContent::Body([error message])`
//! - `title` → no `RegionRole::Title` slot in the region map; falls through to the
//!   Phase-3 append path → `FrameContent::Title("Error: E-LAY-008")` with a clamped
//!   full-page bbox
//!
//! Both variants are rendered by all exporters (PPTX, HTML, PDF, DOCX).  The error
//! code and message therefore appear in every output format for warn-only builds.

use std::sync::Arc;

use slideforge_types::{FieldValue, OrderedMap, Slide, SourceSpan, Value};

/// The internal slide type identifier for error placeholder slides.
///
/// Re-exported from `slideforge_types::ERROR_PLACEHOLDER_SLIDE_TYPE`, which
/// is the single authoritative source. Both `slideforge-layout` (region map)
/// and `slideforge-validate` (placeholder constructor) depend on `slideforge-types`,
/// so placing the constant there avoids a circular-dependency between the two crates
/// (F-094-P9-001 / TD-VSDD-060).
pub use slideforge_types::ERROR_PLACEHOLDER_SLIDE_TYPE;

/// Construct an error-slide placeholder for the given diagnostic.
///
/// The returned [`Slide`] uses the internal type `"__error_placeholder__"`
/// and carries the error code, message, and 1-based slide position as
/// resolved field values.
///
/// # Arguments
///
/// * `code` — The diagnostic error code (e.g., `"E-LAY-002"`).
/// * `message` — A human-readable description of the error.
/// * `position` — The 1-based index of the slide in the deck where the
///   error occurred. For deck-level errors (e.g., zero slides), pass `0`.
#[must_use]
pub fn error_slide_placeholder(code: &str, message: &str, position: usize) -> Slide {
    let mut fields: OrderedMap<Arc<str>, FieldValue> = OrderedMap::new();

    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from(format!("Error: {code}")))),
    );
    // `body` mirrors `error_message` so that `thread_fields_to_blocks` threads
    // the error message text into the `RegionRole::Body` frame during re-layout.
    // Without this field the placeholder's region frame remains `FrameContent::Empty`
    // and all exporters render a blank slide instead of the diagnostic card
    // (F-094-P10-001 defect chain).  The `body` name is the canonical field name
    // recognised by the threading pass (ADR-019 §3 — body field maps to TextTag::Body).
    fields.insert(
        Arc::from("body"),
        FieldValue::Literal(Value::Str(Arc::from(message))),
    );
    fields.insert(
        Arc::from("error_code"),
        FieldValue::Literal(Value::Str(Arc::from(code))),
    );
    fields.insert(
        Arc::from("error_message"),
        FieldValue::Literal(Value::Str(Arc::from(message))),
    );
    // `source_location` is empty until full span support is available.
    fields.insert(
        Arc::from("source_location"),
        FieldValue::Literal(Value::Str(Arc::from(""))),
    );
    // Store position as i64 per Value::Int — usize fits within i64 on all
    // supported platforms (u64::MAX > i64::MAX is moot for slide positions).
    #[allow(clippy::cast_possible_wrap)]
    let position_i64 = position as i64;
    fields.insert(
        Arc::from("position"),
        FieldValue::Literal(Value::Int(position_i64)),
    );

    Slide {
        slide_type: Arc::from(ERROR_PLACEHOLDER_SLIDE_TYPE),
        fields,
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
        // Synthetic error-placeholder slides have no authored field spans.
        // field_spans is intentionally empty per Slide.field_spans doc invariant.
        field_spans: OrderedMap::new(),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use slideforge_types::{FieldValue, Value};

    use super::{ERROR_PLACEHOLDER_SLIDE_TYPE, error_slide_placeholder};

    // ── slide_type ─────────────────────────────────────────────────────────────

    /// The placeholder slide must use the internal error type identifier.
    #[test]
    fn test_error_slide_has_error_type() {
        let slide = error_slide_placeholder("E-LAY-002", "Deck contains zero slides", 0);
        assert_eq!(
            slide.slide_type.as_ref(),
            ERROR_PLACEHOLDER_SLIDE_TYPE,
            "error placeholder must have slide_type = '__error_placeholder__'"
        );
    }

    // ── Required fields ────────────────────────────────────────────────────────

    /// The placeholder slide must carry all required fields per the schema.
    #[test]
    fn test_error_slide_has_error_fields() {
        let slide = error_slide_placeholder("E-LAY-002", "Deck contains zero slides", 0);
        assert!(
            slide.fields.contains_key("title"),
            "error placeholder must have a 'title' field; fields: {:?}",
            slide.fields
        );
        assert!(
            slide.fields.contains_key("error_code"),
            "error placeholder must have an 'error_code' field; fields: {:?}",
            slide.fields
        );
        assert!(
            slide.fields.contains_key("error_message"),
            "error placeholder must have an 'error_message' field; fields: {:?}",
            slide.fields
        );
        assert!(
            slide.fields.contains_key("source_location"),
            "error placeholder must have a 'source_location' field; fields: {:?}",
            slide.fields
        );
        // F-094-P10-001: `body` field must be present so thread_fields_to_blocks
        // threads the error message into FrameContent::Body for all exporters.
        assert!(
            slide.fields.contains_key("body"),
            "error placeholder must have a 'body' field (F-094-P10-001: \
             thread_fields_to_blocks picks this up as FrameContent::Body so the \
             diagnostic renders in all exporters); fields: {:?}",
            slide.fields
        );
    }

    /// F-094-P10-001: `body` field value must equal the message passed in.
    #[test]
    fn test_f094_p10_001_error_slide_body_field_equals_message() {
        let message = "Deck contains zero slides";
        let slide = error_slide_placeholder("E-LAY-002", message, 0);
        let body_field = slide.fields.get("body");
        let body_value = match body_field {
            Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
            _ => None,
        };
        assert_eq!(
            body_value,
            Some(message),
            "body field must equal the message argument (F-094-P10-001); \
             got: {body_field:?}"
        );
    }

    /// The `error_code` field value must match the code passed in.
    #[test]
    fn test_error_slide_error_code_matches() {
        let slide = error_slide_placeholder("E-LAY-002", "Deck contains zero slides", 0);
        let code_field = slide.fields.get("error_code");
        let code_value = match code_field {
            Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
            _ => None,
        };
        assert_eq!(
            code_value,
            Some("E-LAY-002"),
            "error_code field must be 'E-LAY-002'; got: {code_field:?}"
        );
    }

    /// The `title` field must be `"Error: <code>"` (machine-readable heading).
    #[test]
    fn test_error_slide_title_is_error_code_format() {
        let slide = error_slide_placeholder("E-LAY-002", "Deck contains zero slides", 0);
        let title_field = slide.fields.get("title");
        let title_value = match title_field {
            Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
            _ => None,
        };
        assert_eq!(
            title_value,
            Some("Error: E-LAY-002"),
            "title field must be 'Error: E-LAY-002'; got: {title_field:?}"
        );
    }

    /// The `error_message` field must contain the human-readable error text.
    #[test]
    fn test_error_slide_error_message_contains_message() {
        let message = "Deck contains zero slides";
        let slide = error_slide_placeholder("E-LAY-002", message, 0);
        let msg_field = slide.fields.get("error_message");
        let msg_value = match msg_field {
            Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
            _ => None,
        };
        assert!(
            msg_value.is_some_and(|t| t.contains(message)),
            "error_message field must contain '{message}'; got: {msg_field:?}"
        );
    }

    /// The `source_location` field must be present (empty string until spans available).
    #[test]
    fn test_error_slide_source_location_field_present() {
        let slide = error_slide_placeholder("E-LAY-002", "Deck contains zero slides", 0);
        let loc_field = slide.fields.get("source_location");
        let loc_value = match loc_field {
            Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
            _ => None,
        };
        assert!(
            loc_value.is_some(),
            "source_location field must be present; got: {loc_field:?}"
        );
    }

    /// The placeholder slide must have no content blocks.
    #[test]
    fn test_error_slide_has_no_blocks() {
        let slide = error_slide_placeholder("E-LAY-002", "Deck contains zero slides", 0);
        assert!(
            slide.blocks.is_empty(),
            "error placeholder must have no content blocks; got: {:?}",
            slide.blocks
        );
    }

    /// The placeholder slide must have no register gating.
    #[test]
    fn test_error_slide_has_no_register() {
        let slide = error_slide_placeholder("E-LAY-002", "Deck contains zero slides", 0);
        assert!(
            slide.register.is_none(),
            "error placeholder must not be register-gated"
        );
    }

    // ── Position field ─────────────────────────────────────────────────────────

    /// The placeholder must record the 1-based slide position.
    #[test]
    fn test_error_slide_position_field() {
        let slide = error_slide_placeholder("E-LAY-001", "Overflow on slide 3", 3);
        let pos_field = slide.fields.get("position");
        let pos_value = match pos_field {
            Some(FieldValue::Literal(Value::Int(n))) => Some(*n),
            _ => None,
        };
        assert_eq!(
            pos_value,
            Some(3),
            "position field must be Int(3); got: {pos_field:?}"
        );
    }

    /// Position 0 (deck-level error) is valid and stored correctly.
    #[test]
    fn test_error_slide_position_zero() {
        let slide = error_slide_placeholder("E-LAY-002", "Zero slides", 0);
        let pos_field = slide.fields.get("position");
        let pos_value = match pos_field {
            Some(FieldValue::Literal(Value::Int(n))) => Some(*n),
            _ => None,
        };
        assert_eq!(
            pos_value,
            Some(0),
            "position field must be Int(0) for deck-level errors; got: {pos_field:?}"
        );
    }

    // ── Different error codes ──────────────────────────────────────────────────

    /// Works correctly with E-LAY-001 (overflow).
    #[test]
    fn test_error_slide_overflow_code() {
        let slide = error_slide_placeholder("E-LAY-001", "Content overflows placeholder", 5);
        assert_eq!(slide.slide_type.as_ref(), ERROR_PLACEHOLDER_SLIDE_TYPE);
        let code_field = slide.fields.get("error_code");
        let code_value = match code_field {
            Some(FieldValue::Literal(Value::Str(s))) => Some(s.as_ref()),
            _ => None,
        };
        assert_eq!(
            code_value,
            Some("E-LAY-001"),
            "error_code must be 'E-LAY-001'; got: {code_field:?}"
        );
    }
}
