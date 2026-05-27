//! Validation mode and pipeline configuration (STORY-016).
//!
//! [`ValidationMode`] controls whether validator diagnostics cause hard
//! failures (`Strict`) or are demoted to warnings (`WarnOnly`). This maps
//! to the `slideforge build` vs `slideforge build --warn-only` CLI flags.
//!
//! [`ValidationConfig`] bundles mode with per-validator knobs such as
//! [`ValidationConfig::strict_overflow`] for the [`crate::CanvasOverflowValidator`].
//!
//! ## Error codes affected
//!
//! | Code | Default severity | With `ValidationMode::WarnOnly` |
//! |------|------------------|---------------------------------|
//! | `E-LAY-001` | Error (when `strict_overflow: true`) / Warning | Warning |
//! | `E-LAY-002` | Error | Error (never demoted) |

/// Whether validation failures are hard errors or demoted to warnings.
///
/// The default mode is [`ValidationMode::Strict`]. Pass `--warn-only` on the
/// `slideforge build` CLI to activate `WarnOnly` mode during authoring
/// iteration.
///
/// In `Strict` mode, any `Error`-severity diagnostic produced by a registered
/// [`crate::Validator`] causes the export pipeline to abort. In `WarnOnly`
/// mode, all diagnostics are reported but none block export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationMode {
    /// Hard-error mode (default): `Error`-severity diagnostics abort export.
    Strict,
    /// Warn-only mode: all diagnostics are reported but never abort export.
    WarnOnly,
}

/// Configuration for the validation pipeline.
///
/// `ValidationConfig` is constructed by the CLI and passed to the pipeline
/// dispatcher before any validator runs. Individual validators may inspect
/// `strict_overflow` to decide which severity to attach to layout diagnostics.
///
/// ## Default
///
/// ```
/// use slideforge_validate::ValidationConfig;
/// let cfg = ValidationConfig::default();
/// // mode: Strict, strict_overflow: false
/// ```
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Overall validation mode. Defaults to [`ValidationMode::Strict`].
    pub mode: ValidationMode,

    /// When `true`, canvas overflow (`E-LAY-001`) is emitted as `Error`
    /// instead of the default `Warning`. Authors opt in to this via
    /// `--strict-overflow` on the CLI.
    pub strict_overflow: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            mode: ValidationMode::Strict,
            strict_overflow: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ValidationConfig, ValidationMode};

    /// BC-5.03.016: default `ValidationConfig` has Strict mode and `strict_overflow` = false.
    #[test]
    fn test_default_config() {
        let cfg = ValidationConfig::default();
        assert_eq!(
            cfg.mode,
            ValidationMode::Strict,
            "default mode must be Strict"
        );
        assert!(
            !cfg.strict_overflow,
            "strict_overflow must default to false"
        );
    }

    #[test]
    fn test_validation_mode_debug() {
        let s = format!("{:?}", ValidationMode::Strict);
        assert!(s.contains("Strict"));
        let w = format!("{:?}", ValidationMode::WarnOnly);
        assert!(w.contains("WarnOnly"));
    }

    #[test]
    fn test_validation_mode_eq() {
        assert_eq!(ValidationMode::Strict, ValidationMode::Strict);
        assert_eq!(ValidationMode::WarnOnly, ValidationMode::WarnOnly);
        assert_ne!(ValidationMode::Strict, ValidationMode::WarnOnly);
    }

    #[test]
    fn test_validation_mode_copy() {
        let m = ValidationMode::Strict;
        let m2 = m; // Copy — no move
        let m3 = m; // Still usable
        assert_eq!(m2, ValidationMode::Strict);
        assert_eq!(m3, ValidationMode::Strict);
    }

    #[test]
    fn test_validation_config_clone() {
        let cfg = ValidationConfig {
            mode: ValidationMode::WarnOnly,
            strict_overflow: true,
        };
        let cfg2 = cfg.clone();
        assert_eq!(cfg2.mode, ValidationMode::WarnOnly);
        assert!(cfg2.strict_overflow);
    }

    #[test]
    fn test_validation_config_debug() {
        let cfg = ValidationConfig::default();
        let s = format!("{cfg:?}");
        assert!(s.contains("ValidationConfig"));
    }
}
