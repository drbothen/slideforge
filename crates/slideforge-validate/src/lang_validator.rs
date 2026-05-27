//! Lang declaration validator and default injector (STORY-017).
//!
//! [`LangValidator`] checks `DeckMetadata.lang` for the presence of a
//! non-empty, non-whitespace BCP-47 language tag. If absent or blank, it:
//!
//! 1. Emits an `E-A11-003` cosmetic diagnostic (severity [`DiagnosticSeverity::Info`],
//!    never blocking — exit 0 even in strict mode).
//! 2. Injects the default language tag `"en"` into `deck.metadata.lang`.
//!
//! This is the ONLY place where the lang default is injected. After this
//! validator runs, all downstream code (layout engine, exporters) can treat
//! `metadata.lang` as `Some("...")` — it is never `None` post-validation.
//!
//! ## Architecture note
//!
//! The [`Validator`] trait takes `&Deck` (immutable). Lang default injection
//! requires `&mut Deck`. Therefore injection is implemented as a standalone
//! free function [`inject_lang_default`] that the pipeline dispatcher calls
//! before the trait dispatch loop. The trait method handles only diagnostic
//! emission; the free function handles the mutation.
//!
//! ## Error codes
//!
//! | Code | Severity | Meaning |
//! |------|----------|---------|
//! | `E-A11-003` | Info (cosmetic) | Missing or blank `lang` declaration; default "en" applied |

use std::sync::Arc;

use slideforge_plugin_api::{Diagnostic, DiagnosticSeverity, Validator, ValidatorOptions};
use slideforge_types::Deck;

use crate::utils::is_blank;

/// Cosmetic diagnostic code for a missing or blank `lang` declaration.
///
/// `Info` severity — never blocks export, even in strict mode.
/// Traces to BC-5.01.004.
// Used in tests via `super::E_A11_003`. Rust dead_code lint does not count
// cfg(test) usage, so we suppress it here.
#[allow(dead_code)]
pub(crate) const E_A11_003: &str = "E-A11-003";

/// Validates the deck's `lang` field and emits `E-A11-003` if absent or blank.
///
/// Does NOT inject the default (`"en"`) — that is done by [`inject_lang_default`]
/// before the validator dispatch loop. This separation keeps the `Validator`
/// trait immutable while still allowing the mutation to happen before layout.
///
/// Register with [`slideforge_plugin_api::PluginRegistry::register_validator`].
pub struct LangValidator;

impl Validator for LangValidator {
    fn id(&self) -> &'static str {
        "lang"
    }

    fn validate(&self, deck: &Deck, _opts: &ValidatorOptions) -> Vec<Diagnostic> {
        let lang_missing = match &deck.metadata.lang {
            None => true,
            Some(lang) => is_blank(lang.as_ref()),
        };

        if lang_missing {
            vec![Diagnostic {
                severity: DiagnosticSeverity::Info,
                code: Arc::from(E_A11_003),
                message: Arc::from(
                    "Missing lang declaration in deck metadata. Defaulting to \"en\". \
                     Screen readers may mispronounce non-English content. \
                     Add lang \"en-US\" (or appropriate BCP-47 tag).",
                ),
                span: slideforge_types::SourceSpan::default(),
                hint: Some(Arc::from(
                    "Add lang \"en-US\" at the top of your deck metadata block.",
                )),
            }]
        } else {
            vec![]
        }
    }
}

/// Inject the `"en"` default lang value into `deck.metadata.lang` when the
/// field is absent or blank.
///
/// This function is called by the validation pipeline dispatcher BEFORE the
/// trait-based validator loop runs. After it returns:
/// - If `lang` was `None` or blank, it is now `Some(Arc::from("en"))`.
/// - If `lang` was already set to a non-blank value, it is unchanged.
///
/// # Returns
///
/// `true` if the default was injected (i.e., `lang` was absent or blank),
/// `false` if `lang` was already valid.
pub fn inject_lang_default(deck: &mut Deck) -> bool {
    let needs_default = match &deck.metadata.lang {
        None => true,
        Some(lang) => is_blank(lang.as_ref()),
    };

    if needs_default {
        deck.metadata.lang = Some(Arc::from("en"));
        true
    } else {
        false
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)] // BC-traceability IDs use uppercase: test_BC_S_SS_NNN_xxx
mod tests {
    use std::sync::Arc;

    use slideforge_plugin_api::{DiagnosticSeverity, Validator, ValidatorOptions};
    use slideforge_types::{Deck, DeckMetadata, OrderedMap};

    use super::{E_A11_003, LangValidator, inject_lang_default};

    // ── Deck/metadata construction helpers ────────────────────────────────────

    fn make_deck_with_lang(lang: Option<&str>) -> Deck {
        Deck {
            slides: vec![],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: Some(Arc::from("Test Deck")),
                slideforge_version: Arc::from("0.1.0"),
                lang: lang.map(Arc::from),
                author: None,
            },
            registers: OrderedMap::new(),
        }
    }

    fn default_opts() -> ValidatorOptions {
        ValidatorOptions::default()
    }

    // ── Validator ID ───────────────────────────────────────────────────────────

    /// Validator ID must be "lang".
    #[test]
    fn test_BC_5_01_004_validator_id() {
        assert_eq!(LangValidator.id(), "lang");
    }

    // ── Missing lang → E-A11-003 ───────────────────────────────────────────────

    /// BC-5.01.004 AC-009: Deck with no `lang` declaration → 1× E-A11-003 (Info).
    #[test]
    fn test_BC_5_01_004_missing_lang_produces_warning() {
        let deck = make_deck_with_lang(None);
        let diags = LangValidator.validate(&deck, &default_opts());
        assert_eq!(
            diags.len(),
            1,
            "missing lang must produce exactly 1 E-A11-003; got {diags:?}"
        );
        assert_eq!(
            diags[0].code.as_ref(),
            E_A11_003,
            "diagnostic code must be E-A11-003; got {}",
            diags[0].code
        );
        assert_eq!(
            diags[0].severity,
            DiagnosticSeverity::Info,
            "E-A11-003 must be Info (cosmetic) severity — never blocking"
        );
    }

    /// BC-5.01.004 AC-010: E-A11-003 is Info severity — never blocking, even in strict mode.
    #[test]
    fn test_BC_5_01_004_lang_not_blocking() {
        let deck = make_deck_with_lang(None);
        let diags = LangValidator.validate(&deck, &default_opts());
        for d in &diags {
            assert_ne!(
                d.severity,
                DiagnosticSeverity::Error,
                "E-A11-003 must never be Error severity; got {d:?}"
            );
        }
    }

    /// BC-5.01.004 AC-011 EC-007: `lang ""` (empty string) → E-A11-003.
    #[test]
    fn test_BC_5_01_004_lang_empty_string() {
        let deck = make_deck_with_lang(Some(""));
        let diags = LangValidator.validate(&deck, &default_opts());
        let lang_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_003)
            .collect();
        assert_eq!(
            lang_diags.len(),
            1,
            "empty string lang must produce 1 E-A11-003; got {diags:?}"
        );
    }

    /// BC-5.01.004: Whitespace-only lang produces E-A11-003.
    #[test]
    fn test_BC_5_01_004_lang_whitespace() {
        let deck = make_deck_with_lang(Some("   "));
        let diags = LangValidator.validate(&deck, &default_opts());
        let lang_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_003)
            .collect();
        assert_eq!(
            lang_diags.len(),
            1,
            "whitespace-only lang must produce 1 E-A11-003; got {diags:?}"
        );
    }

    // ── Present lang → no E-A11-003 ───────────────────────────────────────────

    /// BC-5.01.004 AC-009: `lang "en-US"` → 0 E-A11-003 diagnostics.
    #[test]
    fn test_BC_5_01_004_lang_present_no_warning() {
        let deck = make_deck_with_lang(Some("en-US"));
        let diags = LangValidator.validate(&deck, &default_opts());
        let lang_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_003)
            .collect();
        assert!(
            lang_diags.is_empty(),
            "valid lang must produce no E-A11-003; got {diags:?}"
        );
    }

    /// BC-5.01.005 AC-013 EC-006: `lang "zh-Hant-TW"` → no E-A11-003; value
    /// propagates unchanged to `DeckMetadata.lang`.
    #[test]
    fn test_BC_5_01_005_lang_zh_hant_tw_unchanged() {
        let deck = make_deck_with_lang(Some("zh-Hant-TW"));
        let diags = LangValidator.validate(&deck, &default_opts());
        let lang_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == E_A11_003)
            .collect();
        assert!(
            lang_diags.is_empty(),
            "zh-Hant-TW must produce no E-A11-003; got {diags:?}"
        );
        // Value must be unchanged after validation (no normalisation).
        assert_eq!(
            deck.metadata.lang.as_deref(),
            Some("zh-Hant-TW"),
            "lang value must be unchanged (no normalisation); got {:?}",
            deck.metadata.lang
        );
    }

    // ── inject_lang_default ────────────────────────────────────────────────────

    /// BC-5.01.004 AC-009: `inject_lang_default` on a deck with no lang sets
    /// `metadata.lang = Some("en")` and returns `true`.
    #[test]
    fn test_BC_5_01_004_inject_lang_default_sets_en_when_absent() {
        let mut deck = make_deck_with_lang(None);
        let injected = inject_lang_default(&mut deck);
        assert!(injected, "inject_lang_default must return true when lang was None");
        assert_eq!(
            deck.metadata.lang.as_deref(),
            Some("en"),
            "inject_lang_default must set lang to \"en\" when absent; got {:?}",
            deck.metadata.lang
        );
    }

    /// BC-5.01.004 AC-011: `inject_lang_default` on `lang ""` sets `"en"` and returns `true`.
    #[test]
    fn test_BC_5_01_004_inject_lang_default_sets_en_when_empty() {
        let mut deck = make_deck_with_lang(Some(""));
        let injected = inject_lang_default(&mut deck);
        assert!(injected, "inject_lang_default must return true when lang was empty");
        assert_eq!(
            deck.metadata.lang.as_deref(),
            Some("en"),
            "inject_lang_default must set lang to \"en\" when blank; got {:?}",
            deck.metadata.lang
        );
    }

    /// BC-5.01.005: `inject_lang_default` on a deck with `lang "en-US"` leaves
    /// the value unchanged and returns `false`.
    #[test]
    fn test_BC_5_01_005_inject_lang_default_noop_when_present() {
        let mut deck = make_deck_with_lang(Some("en-US"));
        let injected = inject_lang_default(&mut deck);
        assert!(!injected, "inject_lang_default must return false when lang already set");
        assert_eq!(
            deck.metadata.lang.as_deref(),
            Some("en-US"),
            "inject_lang_default must not modify a valid lang value; got {:?}",
            deck.metadata.lang
        );
    }

    /// BC-5.01.005 AC-013: `inject_lang_default` on `lang "zh-Hant-TW"` leaves
    /// the value unchanged.
    #[test]
    fn test_BC_5_01_005_inject_lang_default_noop_zh_hant_tw() {
        let mut deck = make_deck_with_lang(Some("zh-Hant-TW"));
        inject_lang_default(&mut deck);
        assert_eq!(
            deck.metadata.lang.as_deref(),
            Some("zh-Hant-TW"),
            "zh-Hant-TW must be preserved unchanged; got {:?}",
            deck.metadata.lang
        );
    }
}
