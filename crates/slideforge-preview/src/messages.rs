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
    /// # Errors
    ///
    /// Returns `Err` if serialization fails — extremely unlikely given the field
    /// types are all JSON-safe, but the error is propagated rather than panicking.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
#[allow(clippy::doc_markdown)] // prose references to type names and BC IDs in test docs
mod tests {
    use super::*;

    /// BC-4.03.004 postcondition 3b — reload message serializes to correct JSON shape.
    #[test]
    fn test_bc_4_03_004_reload_message_json_shape() {
        let msg = WebSocketMessage::Reload {
            slides: vec!["<article>slide1</article>".to_string()],
        };
        let json = msg.to_json().expect("serialization should succeed");
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(v["type"], "reload");
        assert!(v["slides"].is_array());
        assert_eq!(v["slides"][0], "<article>slide1</article>");
    }

    /// BC-4.03.004 postcondition 3c — error message serializes to correct JSON shape.
    #[test]
    fn test_bc_4_03_004_error_message_json_shape() {
        let msg = WebSocketMessage::Error {
            errors: vec![DiagnosticMessage {
                file: "deck.sf".to_string(),
                line: 5,
                col: 3,
                message: "E-PAR-001".to_string(),
            }],
        };
        let json = msg.to_json().expect("serialization should succeed");
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(v["type"], "error");
        assert!(v["errors"].is_array());
        assert_eq!(v["errors"][0]["file"], "deck.sf");
    }

    /// BC-4.03.004 invariant 2 — type discriminator field is present and lowercase.
    #[test]
    fn test_bc_4_03_004_invariant_type_discriminator_present() {
        let reload = WebSocketMessage::Reload { slides: vec![] };
        let error = WebSocketMessage::Error { errors: vec![] };
        let reload_json: serde_json::Value =
            serde_json::from_str(&reload.to_json().expect("reload serialization ok"))
                .expect("valid json");
        let error_json: serde_json::Value =
            serde_json::from_str(&error.to_json().expect("error serialization ok"))
                .expect("valid json");
        assert_eq!(reload_json["type"], "reload");
        assert_eq!(error_json["type"], "error");
    }

    /// BC-4.03.004 — DiagnosticMessage serializes all four required fields.
    #[test]
    fn test_bc_4_03_004_diagnostic_message_all_fields_present() {
        let dm = DiagnosticMessage {
            file: "test.sf".to_string(),
            line: 10,
            col: 5,
            message: "E-PAR-001: unexpected token".to_string(),
        };
        let json = serde_json::to_string(&dm).expect("valid json");
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert!(v.get("file").is_some(), "file field missing");
        assert!(v.get("line").is_some(), "line field missing");
        assert!(v.get("col").is_some(), "col field missing");
        assert!(v.get("message").is_some(), "message field missing");
        assert_eq!(v["file"], "test.sf");
        assert_eq!(v["line"], 10);
        assert_eq!(v["col"], 5);
    }

    /// Round-trip: serialize then deserialize reload message produces identical value.
    #[test]
    fn test_bc_4_03_004_reload_message_roundtrip() {
        let original = WebSocketMessage::Reload {
            slides: vec![
                "<article>slide1</article>".to_string(),
                "<article>slide2</article>".to_string(),
            ],
        };
        let json = original.to_json().expect("serialization should succeed");
        let deserialized: WebSocketMessage = serde_json::from_str(&json).expect("valid json");
        assert_eq!(original, deserialized);
    }

    /// Round-trip: serialize then deserialize error message produces identical value.
    #[test]
    fn test_bc_4_03_004_error_message_roundtrip() {
        let original = WebSocketMessage::Error {
            errors: vec![DiagnosticMessage {
                file: "deck.sf".to_string(),
                line: 5,
                col: 3,
                message: "E-PAR-001: syntax error".to_string(),
            }],
        };
        let json = original.to_json().expect("serialization should succeed");
        let deserialized: WebSocketMessage = serde_json::from_str(&json).expect("valid json");
        assert_eq!(original, deserialized);
    }
}
