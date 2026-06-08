//! Static HTML exporter for slideforge — WCAG AA compliant output via axe-core.
//!
//! This crate implements the `slideforge-html` crate: a static HTML exporter
//! producing one HTML file per slide (or a single multi-page HTML file) that
//! passes WCAG AA validation via `@axe-core/playwright` in CI.
//!
//! ## Design decisions (ADR-008 and BC-4.03.003)
//!
//! 1. **SVG-based canvas rendering** — not `<canvas>` elements. Each slide is
//!    rendered as an `<svg>` element within an `<article>` semantic landmark.
//! 2. All non-decorative images have non-empty `alt` attributes.
//! 3. All decorative images have `alt=""` and `role="presentation"`.
//! 4. Charts and diagrams embedded as `<svg>` with `role="img"` and `<title>`.
//! 5. `<html lang="...">` derived from deck `lang` field (never hardcoded).
//! 6. Heading hierarchy (h1 → h2 → ...) is correct and non-skipped.
//! 7. ARIA landmark: `<main>` wraps all slide articles. Per-slide `<nav>` navigation
//!    is deferred to STORY-047 (preview); no empty `<nav>` is emitted in v1.
//!
//! ## Plugin trait boundary (BC-5.02.002)
//!
//! [`HtmlExporter`] implements the `Exporter` trait from `slideforge-plugin-api`.
//! This crate MUST NOT import from `slideforge-pptx`, `slideforge-docx`, or
//! `slideforge-pdf`.
//!
//! ## URL scheme security (AC-010 / CWE-601)
//!
//! Before rendering any `Link` or `Xref` inline node to `<a href="...">`, the
//! exporter validates the URL scheme against an explicit allowlist
//! (`http`, `https`, `mailto`, `tel`). Disallowed schemes (`javascript:`,
//! `data:`, `vbscript:`, `blob:`, etc.) are dropped and a `tracing::warn!` is
//! emitted.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod exporter;
pub mod render;

pub use exporter::HtmlExporter;
