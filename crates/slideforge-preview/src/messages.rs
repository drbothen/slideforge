//! `WebSocketMessage` — JSON message types pushed to connected browser clients.
//!
//! The preview server pushes two message types to all connected WebSocket clients:
//!
//! - [`WebSocketMessage::Reload`] — sent when deck evaluation succeeds; carries
//!   the re-rendered slide HTML strings.
//! - [`WebSocketMessage::Error`] — sent when deck evaluation fails; carries
//!   the diagnostic messages.
//!
//! ## Wire format (BC-4.03.004 postcondition 3)
//!
//! ```json
//! {"type": "reload", "slides": ["<svg>...</svg>", ...]}
//! {"type": "error", "errors": [{"file": "...", "line": N, "col": N, "message": "..."}]}
//! ```
//!
//! The `type` discriminator ensures the message format is extensible within v1.x
//! without breaking existing browser clients (BC-4.03.004 invariant 2).

use serde::{Deserialize, Serialize};

/// A single rendered slide's HTML string.
///
/// This is the `<article class="sf-slide">...</article>` string produced by
/// `slideforge_html::render::render_slide_to_html()` for each slide in the deck.
pub type SlideHtml = String;

/// A diagnostic message carried in [`WebSocketMessage::Error`] payloads.
///
/// Serializes to `{"file": "...", "line": N, "col": N, "message": "..."}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticMessage {
    /// Source file path (relative to workspace root).
    pub file: String,

    /// 1-based line number.
    pub line: u32,

    /// 1-based column number.
    pub col: u32,

    /// Human-readable diagnostic message.
    pub message: String,
}

/// Messages pushed to browser clients over the WebSocket connection.
///
/// Serialized as `{"type": "<kind>", ...fields}` using `#[serde(tag = "type")]`
/// with `rename_all = "lowercase"` so the discriminator is lowercase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WebSocketMessage {
    /// Deck re-evaluation succeeded; browser should update slides.
    ///
    /// Wire format: `{"type": "reload", "slides": ["<article>...</article>", ...]}`
    Reload {
        /// The rendered slide HTML strings, one per slide in source order.
        slides: Vec<SlideHtml>,
    },

    /// Deck re-evaluation failed; browser should show error overlay.
    ///
    /// Wire format: `{"type": "error", "errors": [...]}`
    Error {
        /// The diagnostic messages from the failed evaluation.
        errors: Vec<DiagnosticMessage>,
    },
}

impl WebSocketMessage {
    /// Serialize this message to a JSON string for transmission over WebSocket.
    ///
    /// # Panics
    ///
    /// Panics only if the message contains non-serializable data — which is
    /// impossible given the field types. `serde_json::to_string` on this type
    /// never fails in practice.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("WebSocketMessage is always serializable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BC-4.03.004 postcondition 3b — reload message serializes to correct JSON shape.
    #[test]
    fn test_BC_4_03_004_reload_message_json_shape() {
        todo!("implement: verify reload message serializes to {{\"type\":\"reload\",\"slides\":[...]}}")
    }

    /// BC-4.03.004 postcondition 3c — error message serializes to correct JSON shape.
    #[test]
    fn test_BC_4_03_004_error_message_json_shape() {
        todo!("implement: verify error message serializes to {{\"type\":\"error\",\"errors\":[...]}}")
    }

    /// BC-4.03.004 invariant 2 — type discriminator field is present and lowercase.
    #[test]
    fn test_BC_4_03_004_invariant_type_discriminator_present() {
        todo!("implement: verify 'type' field present in both reload and error JSON")
    }

    /// BC-4.03.004 — DiagnosticMessage serializes all four required fields.
    #[test]
    fn test_BC_4_03_004_diagnostic_message_all_fields_present() {
        todo!(
            "implement: verify DiagnosticMessage serializes file/line/col/message fields"
        )
    }

    /// Round-trip: serialize then deserialize reload message produces identical value.
    #[test]
    fn test_BC_4_03_004_reload_message_roundtrip() {
        todo!("implement: serialize reload message, deserialize back, assert eq")
    }

    /// Round-trip: serialize then deserialize error message produces identical value.
    #[test]
    fn test_BC_4_03_004_error_message_roundtrip() {
        todo!("implement: serialize error message, deserialize back, assert eq")
    }
}
