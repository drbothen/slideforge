//! Font file loading helpers and styled face resolution for `slideforge-pdf`.
//!
//! Font subsetting for text drawn via krilla's `Surface`/text API is handled
//! **internally** by krilla — no `subsetter::subset(...)` call is needed for
//! the normal text-rendering path (BC-4.03.002 AC-004, tech-validation RISK-2).
//!
//! This module provides helpers to load raw font bytes from disk so they can be
//! passed to krilla's font API. The loaded bytes are consumed by krilla's
//! internal subsetting mechanism; only the glyphs actually used by the deck are
//! embedded in the output PDF.
//!
//! ## ADR-023 — fontdb-backed styled face resolution
//!
//! [`ResolvedFontSet`] and [`resolve_font_set`] implement the ADR-023-approved
//! mechanism for resolving Bold/Italic/Mono faces via `fontdb` OS/2-table metadata
//! (weight class + fsSelection bits), NOT filename heuristics. This ensures
//! reliable cross-platform font resolution without silent fallback to the wrong face.
//!
//! ## Direct `subsetter` usage
//!
//! If a concrete gap in krilla's text API is discovered during the TDD green
//! phase that requires manual glyph embedding outside krilla's Surface (unlikely
//! for this story), `subsetter::{subset, GlyphRemapper}` (transitive dep of
//! krilla, no direct Cargo dep needed) may be used. See tech-validation RISK-2
//! before adding that path.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use slideforge_types::BrandFonts;

use crate::error::PdfExportError;

// ─── Font-load instrumentation ────────────────────────────────────────────────

/// Process-global count of [`load_system_fonts_counted`] invocations.
///
/// This counter is incremented **structurally** — only via
/// [`load_system_fonts_counted`], the sole wrapper that calls
/// `fontdb::Database::load_system_fonts()` in this crate. Calling
/// `db.load_system_fonts()` directly anywhere else in this crate is
/// prohibited; always use [`load_system_fonts_counted`] instead.
///
/// Used by observability metrics. Not conditionally compiled — zero overhead
/// in release builds (one `fetch_add(Relaxed)` per export call).
#[allow(dead_code)]
static LOAD_SYSTEM_FONTS_COUNT: AtomicUsize = AtomicUsize::new(0);

// Per-thread count of `load_system_fonts_counted` invocations.
// Each thread starts at 0. Unlike `LOAD_SYSTEM_FONTS_COUNT`, this counter is
// isolated per test thread, making it safe to use in parallel unit tests.
// Used by `test_obs_p09_001` to assert the single-load invariant without
// interference from other tests running concurrently on different threads.
#[cfg(test)]
thread_local! {
    static LOAD_SYSTEM_FONTS_THREAD_COUNT: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
}

/// Load system fonts into `db` and increment both instrumentation counters.
///
/// This is the **only** function in this crate that calls
/// `fontdb::Database::load_system_fonts()`. All call sites must use this
/// wrapper — never call `db.load_system_fonts()` directly — so the counter
/// invariant is structural, not a documentation promise.
///
/// ADV-P11-LOW-001 structural fix: wrapping the call here makes the
/// counter invariants true by construction.
fn load_system_fonts_counted(db: &mut fontdb::Database) {
    db.load_system_fonts();
    LOAD_SYSTEM_FONTS_COUNT.fetch_add(1, Ordering::Relaxed);
    #[cfg(test)]
    LOAD_SYSTEM_FONTS_THREAD_COUNT.with(|c| c.set(c.get() + 1));
}

/// Return the number of times [`load_system_fonts_counted`] has been invoked
/// on the **current thread** since the thread started.
///
/// Thread-isolated: parallel tests on other threads do not affect this count.
/// Use this in unit tests instead of the process-global counter to avoid
/// false positives from concurrent test execution (OBS-P09-001 fix).
#[cfg(test)]
#[must_use]
pub(crate) fn load_system_fonts_thread_call_count() -> usize {
    LOAD_SYSTEM_FONTS_THREAD_COUNT.with(std::cell::Cell::get)
}

// `ttf-parser` is used by `measure_text_width_pt` to compute per-glyph
// horizontal advances (cmap → glyph_id → hmtx) for the horizontal cursor in
// `draw_inline_spans_at_y` (ADV-P04-CRIT-001 fix).
// Import is at the use-site via the crate name (no re-export needed).
extern crate ttf_parser;

// ─── ResolvedFace ─────────────────────────────────────────────────────────────

/// A single resolved font face — binds the drawable `krilla::text::Font`, the
/// raw font bytes for glyph-metric measurement, and the `face_index` into the
/// font collection together as an inseparable unit.
///
/// ## Why one struct (ADV-P05-MED-001 + OBS-P05-001 unifying fix)
///
/// The previous design stored `font: Option<krilla::text::Font>` and
/// `raw: Option<Arc<[u8]>>` as independent `Option`s, which allowed:
///
/// 1. **`face_index` discard (MED-001):** fontdb's `with_face_data` callback
///    provided the `face_index` for `.ttc` collections, but it was never stored
///    — the measurement path hardcoded `face_index=0`, yielding wrong advances
///    for any face in a collection other than the first.
///
/// 2. **`Some(font) + None raw` divergence (OBS-P05-001):** the public
///    `with_resolved_font_set` seam allowed a slot where the font was drawable
///    but the raw bytes were absent, producing silent zero-width cursor advance
///    and visual overprint.
///
/// `ResolvedFace` makes BOTH of these invalid states UNREPRESENTABLE:
/// - If the slot is `Some(ResolvedFace)`, all three fields are present.
/// - If resolution fails, the slot is `None` — the whole face is absent.
///
/// ## Measurement
///
/// Pass `raw.as_ref()` and `face_index` to [`measure_text_width_pt`] for
/// cursor width computation; the SAME `face_index` that was used to construct
/// the `Font` via `krilla::text::Font::new(data, face_index)`.
#[derive(Clone)]
pub struct ResolvedFace {
    /// The drawable krilla font for this face.
    ///
    /// Constructed via `krilla::text::Font::new(data, face_index)` at the
    /// fontdb-resolved or override `face_index`.
    pub font: krilla::text::Font,
    /// Raw font bytes for this face — for glyph-metric width measurement.
    ///
    /// Used by [`measure_text_width_pt`] to compute per-span horizontal
    /// advances (cmap → glyph ID → hmtx advance) via `ttf-parser`.
    ///
    /// These bytes are the same source that krilla consumed when constructing
    /// [`font`](Self::font); they represent the entire collection file, not
    /// just this face's bytes. [`face_index`](Self::face_index) selects the
    /// correct face within them.
    pub raw: Arc<[u8]>,
    /// Face index within the font collection file.
    ///
    /// For single-face `.otf`/`.ttf` files this is always `0`. For `.ttc`
    /// collection files this is the fontdb-resolved index of this face —
    /// may be non-zero (e.g., `1` for Helvetica-Bold in a
    /// `HelveticaNeueDeskInterface.ttc` collection on macOS).
    ///
    /// MUST be passed to `measure_text_width_pt` instead of a hardcoded `0`.
    pub face_index: u32,
}

// ─── ResolvedFontSet ──────────────────────────────────────────────────────────

/// A set of resolved font faces — one per style — for use in the PDF draw path.
///
/// Populated by [`resolve_font_set`] using `fontdb` OS/2-table metadata
/// (ADR-023). Each field carries a [`ResolvedFace`] for a specific style, or
/// `None` if the face could not be resolved on the current system.
///
/// ## Invariant (ADV-P05-MED-001 + OBS-P05-001 + ADV-P06-MED-001 unifying fixes)
///
/// Every `Some(ResolvedFace)` slot binds `font`, `raw`, and `face_index`
/// together, making font/raw/index divergence UNREPRESENTABLE. If a slot is
/// `Some`, the draw path can measure width and draw glyphs from the SAME
/// `ResolvedFace` without risking a mismatch between the drawn face index and
/// the measured face index (ADV-P05-MED-001).
///
/// Additionally, both the measurement path
/// (`exporter::compute_multi_span_x_positions`) and the draw path
/// (`exporter::font_for_span`) delegate slot selection to the SAME private
/// `exporter::face_for_span_kind` helper — making it **structurally impossible**
/// for the two paths to select different slots for the same `FontFaceKind`
/// (ADV-P06-MED-001).
///
/// ## Fallback semantics (non-silent degradation)
///
/// When a styled face is unavailable, [`resolve_font_set`] falls back to the
/// `regular` face AND emits `tracing::warn!` — never a silent wrong-face or
/// silent drop. Slot selection for each `FontFaceKind` is performed by
/// `exporter::face_for_span_kind` (see its dispatch table for per-variant chains).
///
/// ## Test seam
///
/// Tests may construct a `ResolvedFontSet` directly from known fixture font
/// bytes via [`ResolvedFontSet::from_faces`] to obtain a deterministic,
/// system-font-free fixture. See
/// [`crate::exporter::PdfExporter::with_resolved_font_set`] for the injection
/// point.
#[derive(Clone)]
pub struct ResolvedFontSet {
    /// Regular (weight-400) font face — the base face.
    ///
    /// `None` if no body font could be resolved (brand family not found on
    /// this system AND no override path was provided).
    pub regular: Option<ResolvedFace>,
    /// Bold (weight-700) font face.
    ///
    /// Falls back to [`regular`](Self::regular) at draw time if `None`.
    pub bold: Option<ResolvedFace>,
    /// Italic (oblique) font face.
    ///
    /// Falls back to [`regular`](Self::regular) at draw time if `None`.
    pub italic: Option<ResolvedFace>,
    /// Monospace font face — used for `InlineNode::Code`.
    ///
    /// Falls back to [`regular`](Self::regular) at draw time if `None`.
    pub mono: Option<ResolvedFace>,
}

impl ResolvedFontSet {
    /// Construct a `ResolvedFontSet` from individual [`ResolvedFace`] slots.
    ///
    /// Use this constructor in tests (and production callers with pre-resolved
    /// faces) to build a `ResolvedFontSet` without going through the fontdb
    /// system-font scan. Each slot is either `Some(ResolvedFace)` or `None`.
    ///
    /// This replaces the old struct-literal construction that exposed separate
    /// `raw_*` and `font` fields — those are now co-located in [`ResolvedFace`].
    #[must_use]
    pub fn from_faces(
        regular: Option<ResolvedFace>,
        bold: Option<ResolvedFace>,
        italic: Option<ResolvedFace>,
        mono: Option<ResolvedFace>,
    ) -> Self {
        Self {
            regular,
            bold,
            italic,
            mono,
        }
    }
}

/// Compute the total horizontal advance of `text` rendered in the given font
/// at `font_size` points, using real font metrics (`ttf-parser` cmap + hmtx).
///
/// ## Measurement algorithm
///
/// For each character `ch` in `text`:
/// 1. Look up the glyph ID via `ttf_parser::Face::glyph_index(ch)`.
/// 2. Retrieve the horizontal advance via `Face::glyph_hor_advance(glyph_id)`.
/// 3. Scale: `advance_pt = (advance_units as f32 / units_per_em as f32) * font_size`.
///
/// Characters whose glyph is not found in the cmap contribute zero advance
/// (graceful — should not occur for ASCII text with typical fonts).
///
/// ## Why real metrics, not an estimate
///
/// Using a fixed per-character width estimate (e.g., `font_size * 0.6 * text.len()`)
/// would produce incorrect cursor positions for proportional fonts (kerned, variable
/// advance width). Real glyph advances are the only production-grade approach.
///
/// ## Why `ttf-parser` and not krilla
///
/// `krilla::text::Font` exposes `units_per_em()` but NOT raw glyph-advance
/// queries — the raw bytes are `pub(crate)` inside krilla's `Data` struct.
/// `ttf-parser` parses the same font bytes independently, exposing
/// `Face::glyph_index` (cmap) and `Face::glyph_hor_advance` (hmtx).
///
/// `ttf-parser =0.25.1` is already in `Cargo.lock` as a transitive dep of
/// `fontdb =0.23.0` (which is itself a transitive dep of `usvg =0.47.0`).
/// Adding it as a direct pinned dep does NOT expand the supply chain.
#[must_use]
pub fn measure_text_width_pt(
    font_bytes: &[u8],
    face_index: u32,
    font_size: f32,
    text: &str,
) -> f32 {
    let Ok(face_index_u16) = u16::try_from(face_index) else {
        tracing::debug!(
            face_index,
            "face_index exceeds u16::MAX — cannot parse with ttf-parser"
        );
        return 0.0;
    };
    let face = match ttf_parser::Face::parse(font_bytes, face_index_u16.into()) {
        Ok(f) => f,
        Err(e) => {
            tracing::debug!(error = ?e, "ttf-parser: failed to parse font for width measurement");
            return 0.0;
        },
    };

    let upem = face.units_per_em();
    if upem == 0 {
        return 0.0;
    }

    let mut total: f32 = 0.0;
    for ch in text.chars() {
        let Some(glyph_id) = face.glyph_index(ch) else {
            // Character not in cmap — skip (zero advance contribution).
            continue;
        };
        let Some(advance_units) = face.glyph_hor_advance(glyph_id) else {
            continue;
        };
        // Scale from font design units to points:
        //   advance_pt = (advance_units / units_per_em) * font_size
        // Use f64 intermediate to avoid f32 precision loss from the integer casts.
        #[allow(clippy::cast_precision_loss)]
        let advance_pt = (f64::from(advance_units) / f64::from(upem)) * f64::from(font_size);
        #[allow(clippy::cast_possible_truncation)]
        let advance_pt_f32 = advance_pt as f32;
        total += advance_pt_f32;
    }
    total
}

// ─── resolve_font_set ─────────────────────────────────────────────────────────

/// Resolve a [`ResolvedFontSet`] from the brand font configuration using
/// `fontdb` OS/2-table metadata lookup (ADR-023).
///
/// ## Resolution priority per face
///
/// For each styled face (bold, italic, mono):
///
/// 1. If `font_override_path` is `Some` and the file loads successfully, use
///    that as the `regular` face (override path replaces brand family lookup for
///    the regular slot; styled faces still come from fontdb against the brand
///    family). This maintains the test seam from `PdfExporter::with_font_path`
///    while adding per-style fontdb resolution on top.
/// 2. Query a `fontdb::Database` (populated via `load_system_fonts()`) by
///    family name + weight/style metadata:
///    - **Bold**: `brand.fonts.body` at `Weight::BOLD`, `Style::Normal`.
///    - **Italic**: `brand.fonts.body` at `Weight::NORMAL`, `Style::Italic`.
///    - **Mono**: `brand.fonts.mono` at `Weight::NORMAL`, `Style::Normal`.
///    - **Regular**: `brand.fonts.body` (heading fallback) at `Weight::NORMAL`,
///      `Style::Normal`. OR the explicit `font_override_path` if provided.
/// 3. If fontdb finds no match for a styled face, fall back to `regular` at
///    draw time AND emit `tracing::warn!` here (non-silent degradation).
/// 4. If `regular` is also absent, draw functions skip that span entirely
///    (existing graceful-degradation behavior, already tested).
///
/// ## Supply chain
///
/// `fontdb = "=0.23.0"` is already in `Cargo.lock` as a transitive dep of
/// `usvg = "=0.47.0"`. Adding it as a direct pinned dep to `slideforge-pdf`
/// does NOT expand the supply chain (ADR-023).
///
/// ## Graceful degradation (no silent drop)
///
/// Every resolution failure emits a structured `tracing::warn!` before
/// returning `None` for the affected field. The draw path uses
/// `.bold.as_ref().or(regular.as_ref())` so a missing bold face falls back
/// visually to regular, but the user sees the warning in logs.
///
/// ## `face_index` preservation (ADV-P05-MED-001 fix)
///
/// Each resolved [`ResolvedFace`] carries the `face_index` reported by
/// fontdb's `with_face_data` callback. For `.ttc` collection files this may
/// be non-zero. The same index is used by both the draw path (via
/// `krilla::text::Font::new(data, face_index)`) and the measurement path
/// (via `measure_text_width_pt(raw, face_index, ...)`) — the divergence is
/// UNREPRESENTABLE because both are co-located in the same [`ResolvedFace`].
pub fn resolve_font_set(
    brand: &BrandFonts,
    font_override_path: Option<&std::path::Path>,
) -> ResolvedFontSet {
    // Populate a fontdb database for ALL face queries (regular, bold, italic, mono).
    //
    // ADR-023: fontdb reads OS/2 table usWeightClass (400=regular, 700=bold) and
    // fsSelection bits (bit 0=italic, bit 5=bold) — NOT filenames.
    // load_system_fonts() cold cost: ~50–300ms Linux, ~10–50ms macOS.
    //
    // OBS-P09-001 / OBS-P10-001 fix: a SINGLE fontdb::Database is created here
    // and passed to BOTH `resolve_regular_face` (regular slot) AND the styled-face
    // queries below.  The previous code created TWO databases — one inside
    // `resolve_regular_face` and one here — doubling the cold-path I/O cost
    // (~50–300ms wasted on every PDF export call).
    let mut db = fontdb::Database::new();
    // ADV-P11-LOW-001: use the counted wrapper — never call db.load_system_fonts() directly.
    load_system_fonts_counted(&mut db);

    // Resolve the regular face — passes the shared db to avoid a second load.
    let regular = resolve_regular_face(brand, font_override_path, &db);

    let bold = resolve_styled_face_via_fontdb(
        &db,
        brand.body.as_ref(),
        fontdb::Weight::BOLD,
        fontdb::Style::Normal,
        "bold",
    );
    if bold.is_none() {
        tracing::warn!(
            family = %brand.body,
            style = "bold",
            "styled font face not found on this system; \
             falling back to regular face — PDF bold text will render without correct weight"
        );
    }

    let italic = resolve_styled_face_via_fontdb(
        &db,
        brand.body.as_ref(),
        fontdb::Weight::NORMAL,
        fontdb::Style::Italic,
        "italic",
    );
    if italic.is_none() {
        tracing::warn!(
            family = %brand.body,
            style = "italic",
            "styled font face not found on this system; \
             falling back to regular face — PDF italic text will render without correct style"
        );
    }

    let mono = resolve_styled_face_via_fontdb(
        &db,
        brand.mono.as_ref(),
        fontdb::Weight::NORMAL,
        fontdb::Style::Normal,
        "mono",
    );
    if mono.is_none() {
        tracing::warn!(
            family = %brand.mono,
            style = "mono",
            "monospace font face not found on this system; \
             falling back to regular face — PDF code spans will render in body font"
        );
    }

    ResolvedFontSet {
        regular,
        bold,
        italic,
        mono,
    }
}

/// Resolve the regular (body/heading) font face.
///
/// Resolution order:
/// 1. `font_override_path` if provided (test seam / production override).
/// 2. `brand.heading` via the provided `db`.
/// 3. `brand.body` via the provided `db`.
///
/// ## OBS-P09-001 / OBS-P10-001 fix
///
/// The `db` parameter is the caller-owned `fontdb::Database` (already populated
/// by `load_system_fonts()` in [`resolve_font_set`]).  Accepting it by reference
/// eliminates the second redundant `load_system_fonts()` call that the previous
/// implementation performed here, halving the cold-path I/O cost.
///
/// Returns `None` if no font can be resolved; callers emit a warn before
/// returning the overall `ResolvedFontSet`.
///
/// Returns a [`ResolvedFace`] that bundles the krilla `Font`, raw bytes, and
/// `face_index` together (ADV-P05-MED-001 fix — `face_index` is always `0` for
/// the override-path case; fontdb-resolved faces carry the actual face index).
fn resolve_regular_face(
    brand: &BrandFonts,
    font_override_path: Option<&std::path::Path>,
    db: &fontdb::Database,
) -> Option<ResolvedFace> {
    // Step 1: explicit override path (test seam + production).
    // Override paths are always single-face files → face_index = 0.
    if let Some(path) = font_override_path {
        match load_font_data(path) {
            Ok(bytes) => {
                let raw: Arc<[u8]> = bytes.into();
                let data: krilla::Data = raw.to_vec().into();
                if let Some(font) = krilla::text::Font::new(data, 0) {
                    return Some(ResolvedFace {
                        font,
                        raw,
                        face_index: 0,
                    });
                }
                tracing::debug!(
                    path = %path.display(),
                    "krilla::text::Font::new returned None for override path; \
                     falling back to brand family resolution"
                );
            },
            Err(e) => {
                tracing::debug!(
                    path = %path.display(),
                    error = %e,
                    "font override path load failed; falling back to brand family resolution"
                );
            },
        }
    }

    // Step 2/3: fontdb metadata lookup for heading or body family.
    // fontdb reports the actual face_index (may be non-zero for .ttc collections).
    // `db` was loaded once by the caller — no additional I/O here.
    for family in [brand.heading.as_ref(), brand.body.as_ref()] {
        if let Some(face) = resolve_styled_face_via_fontdb(
            db,
            family,
            fontdb::Weight::NORMAL,
            fontdb::Style::Normal,
            "regular",
        ) {
            return Some(face);
        }
    }

    tracing::warn!(
        heading = %brand.heading,
        body = %brand.body,
        "brand font families not found on this system — text drawing will be skipped; \
         PDF will contain structural content but no visible text"
    );
    None
}

/// Query `fontdb` for a face matching the given family + weight + style,
/// then load its bytes and construct a [`ResolvedFace`].
///
/// Returns `None` on any failure (family not found, file unreadable, invalid
/// font data). The `style_label` parameter is used only in diagnostic traces.
///
/// The returned [`ResolvedFace`] carries the krilla `Font`, raw bytes for
/// glyph-metric width measurement (ADV-P04-CRIT-001 fix), AND the resolved
/// `face_index` from fontdb's `with_face_data` callback (ADV-P05-MED-001 fix).
///
/// For `.ttc` collection files the `face_index` may be non-zero — it is
/// stored in the [`ResolvedFace`] and must be passed (not `0`) to
/// [`measure_text_width_pt`].
fn resolve_styled_face_via_fontdb(
    db: &fontdb::Database,
    family: &str,
    weight: fontdb::Weight,
    style: fontdb::Style,
    style_label: &str,
) -> Option<ResolvedFace> {
    let query = fontdb::Query {
        families: &[fontdb::Family::Name(family)],
        weight,
        stretch: fontdb::Stretch::Normal,
        style,
    };
    let face_id = db.query(&query)?;

    // Use fontdb's `with_face_data` to get raw font bytes + face index in one call.
    // This handles Binary / File / SharedFile source variants transparently,
    // including memory-mapped files (the fontdb default on most platforms).
    //
    // ADV-P05-MED-001 fix: capture `face_index` from the callback and store it
    // in `ResolvedFace` instead of discarding it. The same face_index is used
    // both to construct the krilla Font (draw path) and to call
    // `measure_text_width_pt` (measurement path) — making the draw/measure
    // pairing structurally guaranteed.
    db.with_face_data(face_id, |bytes, face_index| {
        let raw: Arc<[u8]> = bytes.into();
        let data: krilla::Data = raw.to_vec().into();
        let font_opt = krilla::text::Font::new(data, face_index);
        if font_opt.is_none() {
            tracing::debug!(
                family,
                style_label,
                face_index,
                "krilla::text::Font::new returned None for fontdb-resolved face"
            );
        }
        font_opt.map(|font| ResolvedFace {
            font,
            raw,
            face_index,
        })
    })?
}

/// Raw font bytes ready for use with krilla's font API.
///
/// This newtype wraps `Vec<u8>` so that font bytes are distinguishable from
/// arbitrary byte buffers at the type level. krilla's subsetting operates on
/// this raw data internally when drawing text via `Surface`.
#[derive(Debug, Clone)]
pub struct FontBytes(pub Vec<u8>);

impl FontBytes {
    /// Load a font file from `path` into memory.
    ///
    /// # Errors
    ///
    /// Returns [`PdfExportError::Io`] if the file cannot be read.
    pub fn load(path: &std::path::Path) -> Result<Self, PdfExportError> {
        std::fs::read(path).map(Self).map_err(|e| {
            // SEC-005: log the full path at debug level for diagnostics,
            // but keep the user-facing error generic (no internal path disclosure).
            tracing::debug!(path = %path.display(), error = %e, "failed to read font file");
            PdfExportError::Io {
                message: format!("failed to read font file: {e}"),
            }
        })
    }

    /// Construct `FontBytes` from a raw byte buffer.
    ///
    /// Used in tests to inject synthetic font data without hitting the
    /// filesystem.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Return the raw font bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Load raw font file bytes from `path`.
///
/// This is the AC-004 font loading function. It reads the font file bytes into
/// memory so they can be passed to krilla's font API. The loaded bytes are
/// consumed by krilla's internal subsetting mechanism when drawing text via
/// `Surface`.
///
/// # Errors
///
/// Returns [`PdfExportError::Io`] if the file cannot be read (e.g., it does
/// not exist, or the process lacks read permission). The full path is emitted
/// via `tracing::debug!` for diagnostics; the user-facing error message is
/// generic and does not expose internal paths (SEC-005).
pub fn load_font_data(path: &std::path::Path) -> Result<Vec<u8>, PdfExportError> {
    std::fs::read(path).map_err(|e| {
        // SEC-005: log the full path at debug level for diagnostics,
        // but keep the user-facing error generic (no internal path disclosure).
        tracing::debug!(path = %path.display(), error = %e, "failed to read font file");
        PdfExportError::Io {
            message: format!("failed to read font file: {e}"),
        }
    })
}

/// Best-effort lookup of a system font file by family name.
///
/// Scans the standard per-platform font directories for a `.ttf`, `.otf`, or
/// `.ttc` file whose file stem matches `family` after normalizing both sides
/// (lowercased, spaces and hyphens stripped). Returns the path of the first
/// match, or `None` if no match is found.
///
/// ## Platform font directories searched
///
/// - **macOS:** `/System/Library/Fonts`, `/Library/Fonts`,
///   `~/Library/Fonts`
/// - **Linux:** `/usr/share/fonts`, `/usr/local/share/fonts`, `~/.fonts`
/// - **Windows:** `%WINDIR%\Fonts` (falls back to `C:\Windows\Fonts` if the
///   env var is absent)
///
/// ## Important caveats
///
/// This is a **best-effort, filename-based** fallback. It does NOT do full
/// font family/subfamily matching, does NOT read font metadata, and does NOT
/// handle collection files (`.ttc`) that bundle multiple families in one file.
/// Real font resolution (with family name lookup via font metadata) happens in
/// STORY-044's draw pass. Use this only as a convenience fallback when a
/// specific font file path is not already known.
///
/// Returns `None` (without panicking) if:
/// - The font directory cannot be read.
/// - No file with a matching stem is found.
/// - The home directory cannot be determined.
#[must_use]
pub fn system_font_fallback(family: &str) -> Option<std::path::PathBuf> {
    let needle = normalize_font_name(family);
    if needle.is_empty() {
        return None;
    }

    let dirs = system_font_dirs();

    for dir in dirs {
        if let Some(found) = search_font_dir(&dir, &needle) {
            return Some(found);
        }
    }

    None
}

/// Normalize a font family name for comparison: lowercase, strip spaces and hyphens.
fn normalize_font_name(name: &str) -> String {
    name.chars()
        .filter(|c| *c != ' ' && *c != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

/// Returns the platform-specific list of directories to search for font files.
///
/// ## Platform font directories searched
///
/// - **macOS:** system fonts + user `~/Library/Fonts`
/// - **Windows:** system `%WINDIR%\Fonts` + per-user
///   `%USERPROFILE%\AppData\Local\Microsoft\Windows\Fonts`
/// - **Linux / other:** `/usr/share/fonts`, `/usr/local/share/fonts`, `~/.fonts`
fn system_font_dirs() -> Vec<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let mut dirs = vec![
            std::path::PathBuf::from("/System/Library/Fonts"),
            std::path::PathBuf::from("/Library/Fonts"),
        ];
        if let Some(home) = home_dir() {
            dirs.push(home.join("Library/Fonts"));
        }
        dirs
    }

    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| String::from("C:\\Windows"));
        let mut dirs = vec![std::path::PathBuf::from(windir).join("Fonts")];
        // Per-user font directory (Windows 10+): %USERPROFILE%\AppData\Local\Microsoft\Windows\Fonts
        if let Some(home) = home_dir() {
            dirs.push(
                home.join("AppData")
                    .join("Local")
                    .join("Microsoft")
                    .join("Windows")
                    .join("Fonts"),
            );
        }
        dirs
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Linux and other Unix-like systems.
        let mut dirs = vec![
            std::path::PathBuf::from("/usr/share/fonts"),
            std::path::PathBuf::from("/usr/local/share/fonts"),
        ];
        if let Some(home) = home_dir() {
            dirs.push(home.join(".fonts"));
        }
        dirs
    }
}

/// Attempt to determine the current user's home directory.
///
/// Uses the `HOME` env var (Unix/macOS) or `USERPROFILE` (Windows).
/// Returns `None` if neither is set.
fn home_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE")
            .ok()
            .map(std::path::PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok().map(std::path::PathBuf::from)
    }
}

/// Maximum recursion depth for [`search_font_dir`].
///
/// System font directories rarely exceed 3-4 levels deep. 32 is a generous
/// ceiling that prevents stack exhaustion from a symlink loop while allowing
/// any legitimate font tree layout.
///
/// **SEC-004 (CWE-61):** Symlinks to directories are skipped outright (not
/// followed); this depth cap is belt-and-suspenders for any case the symlink
/// check misses.
const MAX_FONT_DIR_DEPTH: usize = 32;

/// Search a single directory (recursively) for a font file matching `needle`.
///
/// Returns the **lexicographically-first** match found (sorted by path), so
/// results are deterministic regardless of the OS `read_dir` iteration order.
/// Returns `None` if the directory cannot be read or no matching file exists.
///
/// **SEC-004 (CWE-61) — symlink-loop safety:**
/// - Directory entries that are symlinks are **skipped** (not followed into).
///   `path.symlink_metadata().file_type().is_dir()` is used rather than
///   `path.is_dir()` (which follows symlinks) to detect real directories.
/// - A [`MAX_FONT_DIR_DEPTH`] cap prevents stack exhaustion in edge cases.
fn search_font_dir(dir: &std::path::Path, needle: &str) -> Option<std::path::PathBuf> {
    search_font_dir_inner(dir, needle, 0)
}

/// Inner recursive implementation with explicit `depth` counter.
fn search_font_dir_inner(
    dir: &std::path::Path,
    needle: &str,
    depth: usize,
) -> Option<std::path::PathBuf> {
    if depth > MAX_FONT_DIR_DEPTH {
        tracing::warn!(
            depth,
            max = MAX_FONT_DIR_DEPTH,
            dir = %dir.display(),
            "font dir recursion depth cap exceeded — stopping traversal"
        );
        return None;
    }

    // Collect all readable entries and sort them so iteration is deterministic.
    // Non-readable entries are silently skipped (graceful handling for
    // permission-restricted system font directories).
    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();

    for path in entries {
        // SEC-004: use symlink_metadata() so we inspect the symlink itself,
        // not its target. If the entry IS a symlink, skip it (do not follow
        // into symlinked directories — prevents infinite loops).
        let Ok(meta) = path.symlink_metadata() else {
            continue;
        };
        let file_type = meta.file_type();

        if file_type.is_symlink() {
            // Skip symlinks entirely — do not follow into symlinked directories.
            tracing::debug!(path = %path.display(), "skipping symlink during font dir scan (SEC-004)");
            continue;
        }

        if file_type.is_dir() {
            // Real directory (not a symlink) — recurse with incremented depth.
            if let Some(found) = search_font_dir_inner(&path, needle, depth + 1) {
                return Some(found);
            }
        } else if file_type.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_lowercase);
            if let Some("ttf" | "otf" | "ttc") = ext.as_deref() {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(normalize_font_name)
                    .unwrap_or_default();
                if stem == needle {
                    return Some(path);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// BC-4.03.002 AC-004: `FontBytes::load` reads a font file from disk.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_font_bytes_load_reads_file() {
        // Create a temp file with fake font bytes to avoid filesystem dependency.
        let mut tmp = NamedTempFile::new().expect("failed to create temp file");
        tmp.write_all(b"FAKE_FONT_DATA_FOR_TEST")
            .expect("failed to write temp font data");
        let path = tmp.path();

        let result = FontBytes::load(path);
        assert!(
            result.is_ok(),
            "FontBytes::load must succeed for a valid file"
        );
        let bytes = result.unwrap();
        assert_eq!(
            bytes.as_bytes(),
            b"FAKE_FONT_DATA_FOR_TEST",
            "FontBytes must contain exact file contents"
        );
    }

    /// `FontBytes::from_bytes` constructs from raw bytes without touching disk.
    ///
    /// This test PASSES immediately — it exercises the non-stub path.
    #[test]
    fn test_font_bytes_from_bytes_round_trips() {
        let data = b"TTF_FONT_DATA".to_vec();
        let fb = FontBytes::from_bytes(data.clone());
        assert_eq!(fb.as_bytes(), data.as_slice());
    }

    /// BC-4.03.002 AC-004: `load_font_data` reads a real font file's bytes.
    ///
    /// Uses a `tempfile` to avoid filesystem coupling; verifies the exact bytes
    /// round-trip correctly.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_bc_4_03_002_load_font_data_reads_file() {
        let mut tmp = NamedTempFile::new().expect("failed to create temp file");
        tmp.write_all(b"FAKE_FONT_BYTES_FOR_LOAD_FONT_DATA")
            .expect("failed to write temp font data");
        let path = tmp.path();

        let result = load_font_data(path);
        assert!(
            result.is_ok(),
            "load_font_data must succeed for a valid path"
        );
        assert_eq!(
            result.unwrap(),
            b"FAKE_FONT_BYTES_FOR_LOAD_FONT_DATA",
            "load_font_data must return exact file contents"
        );
    }

    /// BC-4.03.002 AC-004: `load_font_data` returns `Err(PdfExportError::Io)`
    /// for a path that does not exist.
    ///
    /// SEC-005: The user-facing error message must NOT contain the internal
    /// file path (no path disclosure). The path is emitted at `tracing::debug!`
    /// level for diagnostics but is not exposed in the `PdfExportError::Io`
    /// message field.
    #[test]
    fn test_bc_4_03_002_load_font_data_errors_on_missing_path() {
        let missing = std::path::Path::new("/tmp/__nonexistent_font_for_slideforge_test__.ttf");
        let result = load_font_data(missing);
        assert!(
            result.is_err(),
            "load_font_data must return Err for a non-existent path"
        );
        // Verify the error is an Io variant (not a panic).
        match result {
            Err(PdfExportError::Io { message }) => {
                // SEC-005: message must NOT contain the internal path.
                assert!(
                    !message.contains("__nonexistent_font_for_slideforge_test__"),
                    "Io error message must NOT expose the internal file path (SEC-005): {message}"
                );
                // The message must still describe the failure generically.
                assert!(
                    message.contains("failed to read font file"),
                    "Io error message must describe the failure: {message}"
                );
            },
            Err(other) => panic!("expected PdfExportError::Io, got {other:?}"),
            Ok(_) => panic!("expected Err, got Ok"),
        }
    }

    /// BC-4.03.002 AC-004: `system_font_fallback` returns `None` for a
    /// guaranteed-absent family name without panicking.
    ///
    /// This is the CI-safe assertion: `NoSuchFontFamily_ZZZ_Slideforge_Test`
    /// will never exist in any system font directory.
    #[test]
    fn test_bc_4_03_002_system_font_fallback_none_for_absent_family() {
        let result = system_font_fallback("NoSuchFontFamily_ZZZ_Slideforge_Test");
        assert!(
            result.is_none(),
            "system_font_fallback must return None for a non-existent font family"
        );
    }

    /// `normalize_font_name` strips spaces and hyphens and lowercases.
    #[test]
    fn test_normalize_font_name() {
        assert_eq!(normalize_font_name("Helvetica Neue"), "helveticaneue");
        assert_eq!(normalize_font_name("Open-Sans"), "opensans");
        assert_eq!(normalize_font_name("Arial"), "arial");
        assert_eq!(normalize_font_name(""), "");
    }

    /// SEC-004: `search_font_dir` skips symlinks and applies a depth cap so
    /// a self-referential symlink loop in a font directory does not cause
    /// infinite recursion or a stack overflow.
    ///
    /// This test creates a temp directory with a self-referential symlink
    /// (`link -> .`) and verifies that `search_font_dir` returns without
    /// hanging. It is `#[cfg(unix)]` because symlink creation requires Unix
    /// semantics.
    #[cfg(unix)]
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_sec_004_symlink_loop_does_not_recurse_infinitely() {
        use std::fs;
        use std::os::unix::fs as unix_fs;

        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let dir_path = dir.path();

        // Create a self-referential symlink: link -> . (points back to parent).
        let link_path = dir_path.join("loop_link");
        unix_fs::symlink(dir_path, &link_path).expect("failed to create symlink");

        // Place a real font file so there's something to find.
        let font_path = dir_path.join("TestFont.ttf");
        fs::write(&font_path, b"FAKE_FONT").expect("write font file");

        // The symlink loop must NOT cause infinite recursion or panic.
        // The real font file must still be found (symlink is skipped, not the real dir).
        let result = search_font_dir(dir_path, "testfont");
        assert_eq!(
            result,
            Some(font_path),
            "search_font_dir must find the real font file even when a symlink loop exists"
        );
    }

    /// `search_font_dir` returns the **lexicographically-first** match when
    /// multiple files share the same normalized stem (determinism invariant).
    ///
    /// Two temp `.ttf` files whose stems both normalize to "arial" are placed in
    /// a controlled temp directory. The function must always return the
    /// lexicographically-first path, regardless of the OS-level `read_dir`
    /// order. We verify this by running the search multiple times and confirming
    /// the result is identical and equal to the expected first path.
    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_search_font_dir_is_deterministic_on_matching_stem() {
        use std::fs;

        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let dir_path = dir.path();

        // Create two files with the same normalized stem ("arial"):
        //   "Arial-Bold.ttf"  → normalize_font_name → "arialbold"   (no match)
        //   "Arial.ttf"       → normalize_font_name → "arial"        (match)
        //   "Arial_v2.ttf"    → normalize_font_name → "arial_v2"     (no match)
        // We want two DISTINCT files that BOTH normalize to "arial" so we can
        // test which one wins. Use a subdirectory trick: place "AArial.ttf" and
        // "BArial.ttf" — both normalize to "aarial" / "barial" — they don't
        // share a stem. Instead, use TWO files with identical stems but
        // different filenames is not possible with the current normalizer since
        // it strips only spaces and hyphens. So we test the simpler invariant:
        // given one match and one non-match, the match is returned and repeated
        // calls produce the identical path.
        let font_a = dir_path.join("AArial.ttf"); // normalizes to "aarial"
        let font_b = dir_path.join("BArial.ttf"); // normalizes to "barial"
        fs::write(&font_a, b"FAKE").expect("write font_a");
        fs::write(&font_b, b"FAKE").expect("write font_b");

        // Search for "aarial" — only font_a matches.
        let result1 = search_font_dir(dir_path, "aarial");
        let result2 = search_font_dir(dir_path, "aarial");
        assert_eq!(
            result1,
            Some(font_a.clone()),
            "search_font_dir must return the matching file"
        );
        assert_eq!(
            result1, result2,
            "search_font_dir must return the same result on repeated calls (determinism)"
        );

        // Now test that when two files share the same normalized stem, the
        // lexicographically-first path wins. We achieve identical stems by
        // writing two files with the same name in two subdirectories.
        let sub_a = dir_path.join("a_subdir");
        let sub_b = dir_path.join("b_subdir");
        fs::create_dir(&sub_a).expect("create sub_a");
        fs::create_dir(&sub_b).expect("create sub_b");

        let match_in_a = sub_a.join("CommonFont.ttf"); // normalizes to "commonfont"
        let match_in_b = sub_b.join("CommonFont.ttf"); // normalizes to "commonfont"
        fs::write(&match_in_a, b"FONT_A").expect("write match_in_a");
        fs::write(&match_in_b, b"FONT_B").expect("write match_in_b");

        // sub_a sorts before sub_b lexicographically, so match_in_a must win.
        let result = search_font_dir(dir_path, "commonfont");
        assert_eq!(
            result,
            Some(match_in_a),
            "search_font_dir must return the lexicographically-first path \
             when multiple files share the same normalized stem"
        );
    }

    /// OBS-P09-001 / OBS-P10-001: `resolve_font_set` must call
    /// `load_system_fonts()` EXACTLY ONCE per invocation, not twice.
    ///
    /// The previous implementation created two independent `fontdb::Database`
    /// instances — one inside `resolve_regular_face` and one in
    /// `resolve_font_set` for styled faces — each calling `load_system_fonts()`.
    /// That doubled the cold-path I/O cost (~50–300ms wasted per export call).
    ///
    /// This test asserts the single-load invariant via the per-thread counter:
    /// one `resolve_font_set` call must produce exactly one `load_system_fonts`
    /// invocation on the calling thread.
    ///
    /// Note: uses [`load_system_fonts_thread_call_count`] (thread-local) rather
    /// than the process-global counter so parallel tests on other threads do not
    /// pollute the delta measurement.  The delta approach tolerates this test
    /// running on a thread that has already made prior `resolve_font_set` calls
    /// (e.g. in the same binary's setup code).
    #[test]
    fn test_obs_p09_001_resolve_font_set_loads_system_fonts_exactly_once() {
        use slideforge_types::BrandFonts;
        use std::sync::Arc;

        let brand = BrandFonts {
            heading: Arc::from("Helvetica"),
            body: Arc::from("Helvetica"),
            mono: Arc::from("Courier"),
            font_size_emu: 457_200,
        };
        let count_before = load_system_fonts_thread_call_count();
        let _font_set = resolve_font_set(&brand, None);
        let count_after = load_system_fonts_thread_call_count();
        let delta = count_after - count_before;
        assert_eq!(
            delta, 1,
            "resolve_font_set must call load_system_fonts() exactly once; \
             got {delta} calls (OBS-P09-001 / OBS-P10-001). \
             Check for multiple fontdb::Database::new() + load_system_fonts() in the call path."
        );
    }
}
