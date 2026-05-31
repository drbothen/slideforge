//! `BleedChecker` — post-serialization no-bleed invariant test utility.
//!
//! # Purpose
//!
//! `BleedChecker` is the final defense against register-content bleed bugs. It
//! works at the byte/ZIP level (post-serialization), scanning actual PPTX and
//! DOCX output for sentinel strings that must never appear in the wrong output
//! section.
//!
//! This utility is gated behind the `test-utils` Cargo feature and is intended
//! exclusively for use in exporter test suites. It is **not a production code
//! path** — no production function calls into this module.
//!
//! # Architecture
//!
//! - PPTX and DOCX files are ZIP archives. `BleedChecker` opens the ZIP in
//!   memory and scans specific member files for sentinel strings.
//! - PPTX slide bodies live in `ppt/slides/slide*.xml`.
//! - PPTX speaker notes live in `ppt/notesSlides/notesSlide*.xml`.
//! - DOCX body lives in `word/document.xml`.
//!
//! # EC-003 / EC-004 compliance
//!
//! - EC-003: The sentinel must not be detected as bleed if it appears in a
//!   slide field that the checker does NOT scan (e.g., a `notesSlides` path or
//!   `ppt/theme/theme1.xml`). The methods are scoped to specific ZIP paths; a
//!   sentinel in a path not scanned does not trigger a panic.
//! - EC-004: `BleedChecker` works on **decoded XML text**. Before searching for
//!   the sentinel, each ZIP member's raw UTF-8 bytes are XML-entity-decoded via
//!   a **per-token tolerant decoder** (see [`decode_xml_entities`]). This means
//!   callers pass the **human-readable** (unescaped) register string as the
//!   sentinel — the checker finds it regardless of how the exporter XML-escapes
//!   it in the output. For example, a register string `R&D roadmap` will be
//!   detected even when the exporter writes `R&amp;D roadmap` in the XML.
//!
//! # Cross-crate availability
//!
//! Because this module is gated on `#[cfg(feature = "test-utils")]` (not
//! `#[cfg(test)]`), it is compiled and available when exporter crates enable
//! `slideforge-eval/test-utils` in their `[dev-dependencies]`. Items gated on
//! `#[cfg(test)]` are not visible to external crates even when that feature is
//! enabled — hence the feature-gate approach mandated by STORY-036.
//!
//! # Usage
//!
//! ```rust,ignore
//! // In exporter test:
//! use slideforge_eval::BleedChecker;
//!
//! let pptx_bytes: Vec<u8> = build_pptx_somehow();
//! // Pass the human-readable sentinel — entity decoding is handled internally.
//! BleedChecker::assert_absent_from_pptx_slides(&pptx_bytes, "R&D roadmap");
//! BleedChecker::assert_absent_from_pptx_all(&pptx_bytes, "DETAIL_SENTINEL");
//! ```

// This entire module is only compiled when the `test-utils` feature is active.
// This prevents the `zip` and `quick-xml` crates from being hard production dependencies.
#![cfg(feature = "test-utils")]

// zip and quick-xml are optional deps gated on `test-utils`; always available here.
use std::io::Read;

use zip::ZipArchive;

/// Test utility for verifying that register content does not bleed across
/// format boundaries.
///
/// All methods are static (no instance state needed) and panic with a
/// descriptive message when the invariant is violated. The panic message
/// includes the sentinel string and the first matching file path, making
/// failures easy to diagnose.
///
/// See the [module-level documentation](self) for architecture details.
pub struct BleedChecker;

impl BleedChecker {
    /// Assert that `sentinel` does NOT appear in any `ppt/slides/slide*.xml`
    /// file within the PPTX ZIP.
    ///
    /// This checks the slide **body** only — not speaker notes, masters, or
    /// layouts. A finding here means notes/report/detail content has bled into
    /// the visual slide body (a P0 bleed defect per BC-1.14.004).
    ///
    /// # Sentinel contract (EC-004)
    ///
    /// Pass the **human-readable** register string as `sentinel`. `BleedChecker`
    /// XML-entity-decodes each member before searching using a per-token tolerant
    /// decoder, so `R&D roadmap` is detected even when the exporter writes
    /// `R&amp;D roadmap` in the XML. Only the decoded text is searched; the raw
    /// XML bytes are never searched (see F-P4-001 for why the raw-search branch
    /// was removed — it caused false positives for sentinels that are substrings
    /// of XML escape machinery, e.g., sentinel `"amp"` matching inside `&amp;`).
    ///
    /// # EC-003 compliance
    ///
    /// Only members whose names start with `ppt/slides/slide` and end with
    /// `.xml` are scanned. A sentinel in any other path (theme, notesSlides,
    /// masters, layouts, content types, etc.) does NOT trigger a panic.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `pptx_bytes` is not a valid ZIP archive.
    /// - No `ppt/slides/slide*.xml` members are found — indicates a malformed
    ///   or empty PPTX that would make an absence check vacuously true.
    /// - `sentinel` is found (after XML-entity decoding) in any
    ///   `ppt/slides/slide*.xml` file.
    pub fn assert_absent_from_pptx_slides(pptx_bytes: &[u8], sentinel: &str) {
        let mut archive = Self::open_zip(pptx_bytes, "PPTX");

        // F-003: Fail closed — a valid PPTX always contains ≥1 slide body.
        // Collect slide member count first (by index, F-036-004) to enforce the guard.
        let slide_count = (0..archive.len())
            .filter(|&i| {
                let name = archive
                    .by_index(i)
                    .unwrap_or_else(|e| {
                        panic!("BleedChecker: failed to index PPTX ZIP entry {i}: {e}")
                    })
                    .name()
                    .to_owned();
                Self::is_pptx_slide_path(&name)
            })
            .count();
        assert!(
            slide_count > 0,
            "BleedChecker: no ppt/slides/slide*.xml members found — not a valid PPTX.\n\
             (A valid PPTX must contain at least one slide body. Absence check would be \
             vacuously true on an empty/malformed archive — refusing to give a false green.)"
        );

        // Scan each physical entry by index (F-036-004 — handles duplicate names).
        // EC-003: only ppt/slides/slide*.xml paths are scanned.
        for i in 0..archive.len() {
            if let Some(name) =
                Self::member_contains(&mut archive, i, sentinel, Self::is_pptx_slide_path)
            {
                panic!(
                    "BleedChecker: register-content bleed detected in PPTX slide body.\n\
                     sentinel : {sentinel:?}\n\
                     found in : {name:?}\n\
                     (BC-1.14.004 invariant 1: register content must not appear in PPTX slide body)"
                );
            }
        }
    }

    /// Assert that `sentinel` does NOT appear in ANY file within the PPTX ZIP.
    ///
    /// Used for `detail` register content which must be excluded from the
    /// entire PPTX output — not just the slide body, but also notes slides,
    /// masters, layouts, and all other parts.
    ///
    /// # Sentinel contract (EC-004)
    ///
    /// Pass the **human-readable** register string as `sentinel`. `BleedChecker`
    /// XML-entity-decodes each member before searching using a per-token tolerant
    /// decoder, so `R&D roadmap` is detected even when the exporter writes
    /// `R&amp;D roadmap` in the XML. Only the decoded text is searched; the raw
    /// XML bytes are never searched (see F-P4-001).
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `pptx_bytes` is not a valid ZIP archive.
    /// - The archive contains zero members — indicates a malformed or empty
    ///   PPTX that would make an absence check vacuously true.
    /// - `sentinel` is found (after XML-entity decoding) in any file within
    ///   the ZIP.
    pub fn assert_absent_from_pptx_all(pptx_bytes: &[u8], sentinel: &str) {
        let mut archive = Self::open_zip(pptx_bytes, "PPTX");

        // F-003: Fail closed — an empty archive means zero members were scanned,
        // making any absence check vacuously true and silently hiding that the
        // PPTX was never written.
        assert!(
            !archive.is_empty(),
            "BleedChecker: PPTX archive contains zero members — not a valid PPTX.\n\
             (Absence check would be vacuously true on an empty archive — refusing to \
             give a false green.)"
        );

        // Scan every physical entry by index (F-036-004 — handles duplicate names).
        for i in 0..archive.len() {
            if let Some(name) = Self::member_contains(&mut archive, i, sentinel, |_| true) {
                panic!(
                    "BleedChecker: register-content bleed detected in PPTX archive.\n\
                     sentinel : {sentinel:?}\n\
                     found in : {name:?}\n\
                     (BC-1.14.004 postcondition 6: detail content must not appear anywhere in PPTX)"
                );
            }
        }
    }

    /// Assert that `sentinel` IS present in `word/document.xml` within the
    /// DOCX ZIP.
    ///
    /// This is a **positive** test: it verifies that `report` register content
    /// actually made it into the DOCX body, confirming the routing is active
    /// (not just checking for the absence of bleed).
    ///
    /// # Sentinel contract (EC-004)
    ///
    /// Pass the **human-readable** register string as `sentinel`. `BleedChecker`
    /// XML-entity-decodes the member before searching using a per-token tolerant
    /// decoder. Only the decoded text is searched (see F-P4-001).
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `docx_bytes` is not a valid ZIP archive.
    /// - `word/document.xml` is not present in the ZIP.
    /// - `sentinel` is NOT found (after XML-entity decoding) in
    ///   `word/document.xml`.
    pub fn assert_present_in_docx_body(docx_bytes: &[u8], sentinel: &str) {
        let mut archive = Self::open_zip(docx_bytes, "DOCX");
        let decoded = Self::read_docx_member_decoded(&mut archive, "word/document.xml");
        assert!(
            decoded.contains(sentinel),
            "BleedChecker: expected register content NOT found in DOCX body.\n\
             sentinel     : {sentinel:?}\n\
             searched in  : \"word/document.xml\"\n\
             (BC-1.14.004 postcondition 3: report content must be present in DOCX body)"
        );
    }

    /// Assert that `sentinel` does NOT appear in `word/document.xml` within
    /// the DOCX ZIP.
    ///
    /// Used to verify that `notes` register content was not written to the
    /// DOCX report body (where it would constitute a bleed defect).
    ///
    /// # Sentinel contract (EC-004)
    ///
    /// Pass the **human-readable** register string as `sentinel`. `BleedChecker`
    /// XML-entity-decodes the member before searching using a per-token tolerant
    /// decoder. Only the decoded text is searched (see F-P4-001).
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `docx_bytes` is not a valid ZIP archive.
    /// - `word/document.xml` is not present in the ZIP.
    /// - `sentinel` IS found (after XML-entity decoding) in
    ///   `word/document.xml`.
    pub fn assert_absent_from_docx_body(docx_bytes: &[u8], sentinel: &str) {
        let mut archive = Self::open_zip(docx_bytes, "DOCX");
        let decoded = Self::read_docx_member_decoded(&mut archive, "word/document.xml");
        assert!(
            !decoded.contains(sentinel),
            "BleedChecker: register-content bleed detected in DOCX body.\n\
             sentinel    : {sentinel:?}\n\
             found in    : \"word/document.xml\"\n\
             (BC-1.14.004 postcondition 2: notes content must not appear in DOCX body)"
        );
    }

    // ─── Internal helpers ─────────────────────────────────────────────────────

    /// Returns `true` if `name` matches the PPTX slide body path pattern:
    /// `ppt/slides/slide<N>.xml` (where N is one or more digits).
    ///
    /// This implements EC-003 path scoping: only slide body files are scanned
    /// by `assert_absent_from_pptx_slides`. Notes slides, masters, layouts,
    /// and any other paths are excluded.
    fn is_pptx_slide_path(name: &str) -> bool {
        // Must start with the slides directory prefix and end with .xml.
        // The filename portion must be "slide" followed by at least one digit.
        let Some(filename) = name.strip_prefix("ppt/slides/") else {
            return false;
        };
        let Some(stem) = filename.strip_suffix(".xml") else {
            return false;
        };
        let Some(digits) = stem.strip_prefix("slide") else {
            return false;
        };
        !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
    }

    /// Open `bytes` as a ZIP archive, panicking with a context message on failure.
    ///
    /// # Panics
    ///
    /// Panics with a descriptive message (including the format name) if `bytes`
    /// is not a valid ZIP archive.
    fn open_zip<'a>(bytes: &'a [u8], format: &str) -> ZipArchive<std::io::Cursor<&'a [u8]>> {
        let cursor = std::io::Cursor::new(bytes);
        ZipArchive::new(cursor).unwrap_or_else(|e| {
            panic!(
                "BleedChecker: failed to open {format} bytes as a ZIP archive: {e}\n\
                 (Are you passing valid {format} bytes?)"
            )
        })
    }

    /// Read a ZIP member by physical index as a UTF-8 string, then XML-entity-decode it
    /// using the per-token tolerant decoder ([`decode_xml_entities`]).
    ///
    /// Returns the **decoded** text only. The raw text is NOT returned and is NOT
    /// searched (see F-P4-001: the raw-search branch was removed because it caused
    /// false positives for sentinels that are substrings of XML escape machinery).
    ///
    /// # Why index, not name (F-036-004)
    ///
    /// ZIP permits duplicate member names (two entries named `ppt/slides/slide1.xml`).
    /// `ZipArchive::by_name` returns the FIRST match, silently skipping any
    /// duplicate. By reading by index we visit every physical entry — including
    /// duplicates — so no bleed-bearing duplicate can hide behind an earlier entry.
    ///
    /// # EC-004 compliance (F-036-002, F-P4-001, F-P4-003)
    ///
    /// XML exporters escape special characters: `&` → `&amp;`, `<` → `&lt;`,
    /// `>` → `&gt;`, `"` → `&quot;`, `'` → `&apos;`. Numeric character
    /// references (`&#NN;`, `&#xHH;`) are also decoded. Without this step, a
    /// sentinel like `R&D roadmap` would not be found in a member containing
    /// `R&amp;D roadmap`, causing a false negative (missed bleed detection).
    ///
    /// Decoding uses a **per-token tolerant** strategy via [`decode_xml_entities`]:
    ///
    /// - The five predefined XML entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`)
    ///   are decoded to their corresponding characters.
    /// - Numeric character references (`&#NN;`, `&#xHH;`) that map to valid Unicode
    ///   scalar values are decoded to those characters.
    /// - Unknown named entities (e.g. `&copy;`, `&nbsp;`) decode to the Unicode
    ///   replacement character U+FFFD (`\u{FFFD}`).
    /// - Malformed numeric references (e.g. `&#xZZ;`) decode to U+FFFD rather than
    ///   aborting — one bad token never disables decoding for the rest of the member.
    ///
    ///   U+FFFD is a sentinel-neutral placeholder: it never forms a substring equal
    ///   to a well-formed ASCII/UTF-8 sentinel, so it cannot cause false-positive
    ///   joins (e.g. `A&copy;B` → `A\u{FFFD}B`, not `AB`) or false-green joins
    ///   (e.g. `R&copy;D` → `R\u{FFFD}D`, not `RD`).
    ///
    /// The sentinel is checked against the **decoded text only** (F-P4-001).
    /// The raw-search branch was removed because it matched escape-machinery
    /// substrings: a sentinel like `"amp"` would match inside `&amp;D` raw bytes,
    /// producing a spurious panic (false positive) or a spurious pass (false green)
    /// depending on whether the check is an absence or presence assertion.
    ///
    /// Non-UTF-8 bytes in the member are replaced with U+FFFD before entity decoding.
    ///
    /// # Panics
    ///
    /// Panics if the member index is out of range or the member cannot be read.
    fn read_member_by_index_decoded(
        archive: &mut ZipArchive<std::io::Cursor<&[u8]>>,
        index: usize,
    ) -> String {
        let mut entry = archive.by_index(index).unwrap_or_else(|e| {
            panic!(
                "BleedChecker: failed to read ZIP member at index {index}: {e}\n\
                 (ZIP archive may be malformed or truncated)"
            )
        });
        let name = entry.name().to_owned();
        let mut raw_bytes = Vec::new();
        entry.read_to_end(&mut raw_bytes).unwrap_or_else(|e| {
            panic!("BleedChecker: failed to read ZIP member {name:?} at index {index}: {e}")
        });
        let raw_utf8 = String::from_utf8_lossy(&raw_bytes);

        // Per-token tolerant entity decoding (EC-004, F-036-002, F-P4-001, F-P4-003):
        // decode_xml_entities processes each &...;  token individually and never
        // returns Err — one malformed token decodes to U+FFFD and processing
        // continues for the rest of the member. This eliminates the whole-member
        // Err→raw fallback that was the root cause of false negatives (F-P4-003).
        decode_xml_entities(&raw_utf8)
    }

    /// Check whether `sentinel` is present in the **decoded** text of a ZIP member.
    ///
    /// Returns `Some(name)` with the member name if a match is found, `None` otherwise.
    ///
    /// Only the decoded text is searched (F-P4-001: the raw-search branch was
    /// removed — see [`read_member_by_index_decoded`] for the full rationale).
    fn member_contains(
        archive: &mut ZipArchive<std::io::Cursor<&[u8]>>,
        index: usize,
        sentinel: &str,
        path_filter: impl Fn(&str) -> bool,
    ) -> Option<String> {
        // Peek at the name without reading bytes first.
        let name = {
            let entry = archive.by_index(index).unwrap_or_else(|e| {
                panic!(
                    "BleedChecker: failed to index ZIP entry {index}: {e}\n\
                     (ZIP archive may be malformed)"
                )
            });
            entry.name().to_owned()
        };

        if !path_filter(&name) {
            return None;
        }

        let decoded = Self::read_member_by_index_decoded(archive, index);
        if decoded.contains(sentinel) {
            Some(name)
        } else {
            None
        }
    }

    /// Read a named DOCX member (`word/document.xml`) for presence/absence checks.
    ///
    /// For DOCX checks, the member name is canonical and unique per spec; there is
    /// only ever one `word/document.xml`. We still use by-index scanning to remain
    /// consistent with the F-036-004 robustness requirement and to correctly handle
    /// any ZIP with duplicate-named entries.
    ///
    /// Returns the decoded text for the first member whose name equals `target_name`,
    /// or panics if none is found.
    fn read_docx_member_decoded(
        archive: &mut ZipArchive<std::io::Cursor<&[u8]>>,
        target_name: &str,
    ) -> String {
        for i in 0..archive.len() {
            let name = {
                let entry = archive
                    .by_index(i)
                    .unwrap_or_else(|e| panic!("BleedChecker: failed to index ZIP entry {i}: {e}"));
                entry.name().to_owned()
            };
            if name == target_name {
                return Self::read_member_by_index_decoded(archive, i);
            }
        }
        panic!(
            "BleedChecker: ZIP member {target_name:?} not found in archive.\n\
             (Is this a valid DOCX file with the expected structure?)"
        );
    }
}

/// Maximum number of bytes to scan ahead from `&` when looking for the
/// closing `;` of an entity reference.
///
/// A valid entity body is at most 16 characters (`&#x` + 13 hex digits
/// would already exceed the Unicode scalar range, and well-known named
/// entities are much shorter). We allow the `&` byte itself plus 16 body
/// characters plus the `;`, giving a window of 18 bytes. Any `&` followed
/// by a `;` further than this limit is not a valid entity reference — the
/// `&` is emitted literally and we advance one byte.
const MAX_ENTITY_WINDOW: usize = 18;

/// Classify a potential entity body (the text between `&` and `;`) as either
/// a valid entity form or invalid.
///
/// Valid forms (F-P5-001):
/// - Named predefined: matched by `quick_xml::escape::resolve_predefined_entity`.
/// - Named well-formed (unknown to us): `[A-Za-z][A-Za-z0-9]*`, max 16 chars.
///   These decode to U+FFFD (non-joining placeholder).
/// - Decimal numeric: `#[0-9]+` — decoded via `char::from_u32`.
/// - Hex numeric (lowercase `x`): `#x[0-9A-Fa-f]+` — decoded via `char::from_u32`.
///
/// Everything else (contains spaces, punctuation, starts with digit, empty,
/// uppercase `X`, too long) is INVALID — the caller emits a literal `&`.
#[derive(Debug, PartialEq)]
enum EntityBodyKind {
    /// `&amp;`/`&lt;`/etc. — decode to the predefined character.
    Predefined(&'static str),
    /// Well-formed but unknown named entity — decode to U+FFFD.
    UnknownNamed,
    /// Decimal numeric reference `&#NNN;` with a parse-able code point.
    Decimal(u32),
    /// Hex numeric reference `&#xHHH;` with a parse-able code point.
    Hex(u32),
    /// Body does not match any valid entity form — emit literal `&`.
    Invalid,
}

/// Classify `body` (the text between `&` and `;`, not including delimiters).
fn classify_entity_body(body: &str) -> EntityBodyKind {
    // Empty body → invalid.
    if body.is_empty() {
        return EntityBodyKind::Invalid;
    }

    // Enforce overall body length limit (defensive; window already bounds this).
    if body.len() > 16 {
        return EntityBodyKind::Invalid;
    }

    // Predefined entities take priority.
    if let Some(ch) = quick_xml::escape::resolve_predefined_entity(body) {
        return EntityBodyKind::Predefined(ch);
    }

    // Numeric character references.
    if let Some(rest) = body.strip_prefix('#') {
        if rest.is_empty() {
            // `&#;` — invalid.
            return EntityBodyKind::Invalid;
        }
        if let Some(hex_digits) = rest.strip_prefix('x') {
            // Hex reference: `&#xHHH;` — `x` must be lowercase (XML spec).
            // All characters after `x` must be hex digits; empty digits invalid.
            if hex_digits.is_empty() || !hex_digits.chars().all(|c| c.is_ascii_hexdigit()) {
                return EntityBodyKind::Invalid;
            }
            let value = u32::from_str_radix(hex_digits, 16).unwrap_or(u32::MAX);
            return EntityBodyKind::Hex(value);
        }
        // Decimal reference: `&#NNN;` — all characters must be ASCII digits.
        if !rest.chars().all(|c| c.is_ascii_digit()) {
            return EntityBodyKind::Invalid;
        }
        let value: u32 = rest.parse().unwrap_or(u32::MAX);
        return EntityBodyKind::Decimal(value);
    }

    // Named entity: must start with an ASCII letter, followed only by ASCII
    // letters and digits (standard XML Name production, simplified).
    // Entities containing spaces, punctuation, or starting with a digit are
    // invalid (e.g. `&D summary;`, `&1foo;`, `& ;`).
    let mut chars = body.chars();
    let first = chars.next().expect("body non-empty checked above");
    if !first.is_ascii_alphabetic() {
        return EntityBodyKind::Invalid;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric()) {
        return EntityBodyKind::Invalid;
    }

    // Well-formed named entity but not one of the five predefined ones.
    EntityBodyKind::UnknownNamed
}

/// Decode XML entity references in `input` using a per-token tolerant strategy
/// with entity-body validation (F-P5-001).
///
/// This is the core EC-004 decoder for `BleedChecker`. It processes each
/// `&...;` token individually — a malformed or unknown entity never disables
/// decoding for the tokens that follow it (F-P4-003). Each candidate entity
/// reference is validated before decoding: if the body between `&` and `;`
/// does not match a recognised entity form, the `&` is emitted literally and
/// the scanner advances one byte, leaving the rest of the input intact.
///
/// # Decoding rules (F-P5-001)
///
/// | Token form | Body valid? | Decoding |
/// |---|---|---|
/// | `&amp;` / `&lt;` / `&gt;` / `&quot;` / `&apos;` | yes (predefined) | that char |
/// | `&#NNN;` (valid decimal Unicode scalar) | yes | that Unicode char |
/// | `&#xHHH;` (valid hex Unicode scalar, lowercase `x`) | yes | that Unicode char |
/// | `&#NNN;` / `&#xHHH;` valid form but non-scalar code point | yes (well-formed) | U+FFFD |
/// | unknown named (`&copy;`, `&nbsp;`, …) | yes (well-formed name) | U+FFFD |
/// | invalid body (space, punctuation, too long, empty, uppercase `X`, …) | no | literal `&` + advance 1 |
/// | `&` with no `;` within the bounded window | — | literal `&` + advance 1 |
/// | plain text (no `&`) | — | passed through unchanged |
///
/// Key invariant for bleed detection (F-P5-001): a member containing the raw
/// string `R&D summary; q3` decodes to `R&D summary; q3` — the `&D summary`
/// body is invalid (contains a space) so the `&` is emitted literally, and the
/// sentinel `R&D` is preserved in the decoded text.
///
/// U+FFFD is used only for *well-formed* but unrecognised tokens (unknown named
/// entities and numeric refs that map to non-scalar code points). It is a
/// sentinel-neutral placeholder: it never forms a substring equal to a
/// well-formed ASCII/UTF-8 sentinel, preventing both false-positive joins
/// (`A&copy;B` → `A\u{FFFD}B`, not `AB`) and false-green joins
/// (`R&copy;D` → `R\u{FFFD}D`, not `RD`).
///
/// # Why not `quick_xml::escape::unescape_with`?
///
/// `unescape_with` processes the whole string atomically: if any numeric
/// character reference is malformed the function returns `Err` for the ENTIRE
/// input. The previous code fell back to raw text on `Err`, meaning a malformed
/// entity anywhere in a ZIP member silently disabled predefined-entity decoding
/// for the whole member — causing false negatives for correctly-escaped
/// sentinels elsewhere in the same member (F-P4-003).
///
/// The per-token loop below avoids this: each token is processed independently.
///
/// `quick_xml::escape::resolve_predefined_entity` IS still used internally to
/// decode individual predefined entities within the loop (it operates on a
/// single entity name, not the whole string, so it never triggers the
/// whole-input Err problem).
fn decode_xml_entities(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(amp_pos) = remaining.find('&') {
        // Append everything before the `&`.
        output.push_str(&remaining[..amp_pos]);
        remaining = &remaining[amp_pos..];

        // Search for `;` within a bounded window to avoid consuming unrelated
        // semicolons far from the `&` (F-P5-001). The window is MAX_ENTITY_WINDOW
        // bytes from the start of `remaining` (which begins with `&`).
        let window = &remaining[..remaining.len().min(MAX_ENTITY_WINDOW)];
        let semi_pos_opt = window.find(';');

        if let Some(semi_pos) = semi_pos_opt {
            let entity_body = &remaining[1..semi_pos]; // between `&` and `;`

            match classify_entity_body(entity_body) {
                EntityBodyKind::Predefined(ch) => {
                    output.push_str(ch);
                    remaining = &remaining[semi_pos + 1..];
                },
                EntityBodyKind::UnknownNamed => {
                    // Well-formed name but not predefined → U+FFFD (non-joining).
                    output.push('\u{FFFD}');
                    remaining = &remaining[semi_pos + 1..];
                },
                EntityBodyKind::Decimal(code_point) | EntityBodyKind::Hex(code_point) => {
                    match char::from_u32(code_point) {
                        Some(ch) => output.push(ch),
                        None => {
                            // Valid numeric form but non-scalar code point
                            // (surrogate, out-of-range) → U+FFFD.
                            output.push('\u{FFFD}');
                        },
                    }
                    remaining = &remaining[semi_pos + 1..];
                },
                EntityBodyKind::Invalid => {
                    // Body does not match any valid entity form (contains
                    // spaces, punctuation, empty, too long, etc.).
                    // Emit the `&` literally and advance exactly one byte so
                    // the scanner resumes at the character after `&`. This
                    // preserves the rest of the input — including the `;`
                    // that would have been consumed by a greedy match.
                    output.push('&');
                    remaining = &remaining[1..];
                },
            }
        } else {
            // No `;` within the bounded window — bare `&` with no matching
            // entity end. Emit the `&` literally and advance past it.
            output.push('&');
            remaining = &remaining[1..];
        }
    }

    // Append any trailing text after the last entity reference.
    output.push_str(remaining);
    output
}

#[cfg(test)]
mod unit_tests {
    use super::decode_xml_entities;

    #[test]
    fn test_predefined_entities_decoded() {
        assert_eq!(decode_xml_entities("R&amp;D"), "R&D");
        assert_eq!(decode_xml_entities("&lt;item&gt;"), "<item>");
        assert_eq!(
            decode_xml_entities("say &quot;hello&quot;"),
            "say \"hello\""
        );
        assert_eq!(decode_xml_entities("it&apos;s"), "it's");
    }

    #[test]
    fn test_decimal_numeric_ref_decoded() {
        // &#65; = 'A'
        assert_eq!(decode_xml_entities("&#65;"), "A");
        // &#38; = '&'
        assert_eq!(decode_xml_entities("&#38;"), "&");
    }

    #[test]
    fn test_hex_numeric_ref_decoded() {
        // &#x41; = 'A'
        assert_eq!(decode_xml_entities("&#x41;"), "A");
        // &#x26; = '&'
        assert_eq!(decode_xml_entities("&#x26;"), "&");
    }

    #[test]
    fn test_unknown_named_entity_becomes_fffd() {
        let result = decode_xml_entities("&copy;");
        assert_eq!(result, "\u{FFFD}");
    }

    #[test]
    fn test_unknown_entity_does_not_join_neighbours() {
        // A&copy;B must NOT produce "AB" (which would be a false positive for sentinel "AB").
        let result = decode_xml_entities("A&copy;B");
        assert_eq!(result, "A\u{FFFD}B");
        assert!(!result.contains("AB"));
    }

    #[test]
    fn test_malformed_hex_ref_invalid_body_emits_literal_ampersand() {
        // &#xZZ; — body `#xZZ` fails hex validation (Z is not a hex digit).
        // With F-P5-001 body validation: invalid body → literal `&` emitted,
        // scanner advances one byte, remaining `#xZZ;` passes through as-is.
        // Full decoded result is the original string unchanged.
        let result = decode_xml_entities("&#xZZ;");
        assert_eq!(result, "&#xZZ;");
    }

    #[test]
    fn test_malformed_numeric_ref_does_not_disable_rest_of_member() {
        // &#xZZ; (invalid body) must not prevent R&amp;D from decoding correctly.
        // With F-P5-001: &#xZZ; → literal `&` emitted + `#xZZ;` passes through.
        // R&amp;D → R&D.  Full result: "&#xZZ; R&D roadmap".
        // Sentinel detection invariant: the R&D sentinel IS present (must remain).
        let result = decode_xml_entities("&#xZZ; R&amp;D roadmap");
        assert!(result.contains("R&D roadmap"), "got: {result:?}");
        // The malformed token is rendered as a literal `&` + remainder of the
        // original token text (no U+FFFD for invalid bodies — F-P5-001).
        assert!(result.starts_with("&#xZZ;"), "got: {result:?}");
    }

    #[test]
    fn test_no_entity_passthrough() {
        let s = "plain text with no entities";
        assert_eq!(decode_xml_entities(s), s);
    }

    #[test]
    fn test_bare_ampersand_no_semicolon() {
        // A bare & with no closing ; must not panic — emit it literally.
        let result = decode_xml_entities("AT&T");
        assert_eq!(result, "AT&T");
    }

    #[test]
    fn test_escape_machinery_sentinel_amp_not_found_in_decoded() {
        // R&amp;D decoded is "R&D". Sentinel "amp" must NOT be in the decoded text.
        // (This is the F-P4-001 collision: raw bytes contain "amp" inside "&amp;"
        //  but decoded text does not.)
        let decoded = decode_xml_entities("R&amp;D project");
        assert_eq!(decoded, "R&D project");
        assert!(!decoded.contains("amp"));
    }

    #[test]
    fn test_surrogate_code_point_becomes_fffd() {
        // &#xD800; — body `#xD800` is valid hex form, but D800 is a surrogate.
        // char::from_u32(0xD800) returns None → U+FFFD.
        let result = decode_xml_entities("&#xD800;");
        assert_eq!(result, "\u{FFFD}");
    }

    #[test]
    fn test_out_of_range_code_point_becomes_fffd() {
        // &#x200000; — body `#x200000` is valid hex form, but 0x200000 > 0x10FFFF.
        // char::from_u32 returns None → U+FFFD.
        let result = decode_xml_entities("&#x200000;");
        assert_eq!(result, "\u{FFFD}");
    }

    // ─── F-P5-002: Direct unit tests for decode_xml_entities (body validation) ───

    /// The critical bleed-detection correctness case (F-P5-001).
    ///
    /// `R&D summary; q3` contains a bare `&` whose "body" (`D summary`) fails
    /// the entity-body validator because it contains a space. Under the OLD
    /// greedy implementation this consumed everything up to the first `;`
    /// (producing `R\u{FFFD} q3`), silently erasing the sentinel `R&D`.
    ///
    /// With F-P5-001 body validation the `&` is emitted literally and the
    /// scanner advances one byte, leaving the full string intact.
    #[test]
    fn test_f_p5_001_invalid_body_with_space_preserves_sentinel() {
        let result = decode_xml_entities("R&D summary; q3");
        assert_eq!(
            result, "R&D summary; q3",
            "bare & followed by a body containing a space must emit literal & \
             and preserve the rest of the string unchanged"
        );
        // The sentinel substring is intact — the bleed checker will find it.
        assert!(
            result.contains("R&D"),
            "sentinel R&D must be present; got: {result:?}"
        );
    }

    /// Predefined entity decoding must still work after F-P5-001.
    #[test]
    fn test_f_p5_002_predefined_entity_still_decoded() {
        assert_eq!(decode_xml_entities("R&amp;D"), "R&D");
    }

    /// Unknown well-formed named entities still produce U+FFFD (non-joining).
    #[test]
    fn test_f_p5_002_unknown_named_entity_produces_fffd() {
        assert_eq!(decode_xml_entities("A&copy;B"), "A\u{FFFD}B");
    }

    /// Decimal numeric references are decoded to the corresponding character.
    #[test]
    fn test_f_p5_002_decimal_numeric_ref_decoded() {
        assert_eq!(decode_xml_entities("&#65;"), "A");
        assert_eq!(decode_xml_entities("&#x41;"), "A");
    }

    /// Invalid hex body (non-hex digit in body) → literal `&`, not U+FFFD.
    /// The rest of the string passes through unchanged.
    #[test]
    fn test_f_p5_002_invalid_hex_body_emits_literal_ampersand() {
        // `Z` is not a hex digit → body `#xZZ` is invalid.
        let result = decode_xml_entities("&#xZZ;");
        // Literal & emitted, scanner advances 1, rest passthrough.
        assert_eq!(result, "&#xZZ;");
    }

    /// Bare `&` with no `;` anywhere → literal `&`.
    #[test]
    fn test_f_p5_002_bare_ampersand_no_semicolon() {
        assert_eq!(decode_xml_entities("AT&T"), "AT&T");
    }

    /// Empty entity body `&;` is invalid → literal `&`.
    #[test]
    fn test_f_p5_002_empty_entity_body() {
        // `&;` → body is "" → invalid → literal & emitted, `;` passthrough.
        let result = decode_xml_entities("&;");
        assert_eq!(result, "&;");
    }

    /// `&amp` (no closing `;`) has no `;` within the window → literal `&`.
    #[test]
    fn test_f_p5_002_named_entity_no_semicolon() {
        let result = decode_xml_entities("&amp");
        // No `;` found → literal `&`, remaining `amp` passthrough.
        assert_eq!(result, "&amp");
    }

    /// `a & b ;` — the `&` is followed by a space, making the body ` b ` which
    /// is invalid (starts with space). Literal `&` emitted, rest unchanged.
    #[test]
    fn test_f_p5_002_spaced_ampersand_body() {
        let result = decode_xml_entities("a & b ;");
        // Body " b " has spaces → invalid → literal & emitted.
        assert_eq!(result, "a & b ;");
    }

    /// Over-long body (> 16 chars) → invalid → literal `&`.
    #[test]
    fn test_f_p5_002_overlength_body_emits_literal_ampersand() {
        // Body of 50 x's exceeds the 16-char limit → invalid.
        let long_body = format!("&{};", "x".repeat(50));
        let result = decode_xml_entities(&long_body);
        // Literal & emitted, scanner advances 1, rest passthrough.
        assert!(
            result.starts_with('&'),
            "over-long body must emit literal &; got: {result:?}"
        );
        assert!(
            !result.contains('\u{FFFD}'),
            "over-long body must not produce U+FFFD; got: {result:?}"
        );
    }

    /// Multi-byte UTF-8 character adjacent to an entity must not panic.
    /// (Byte vs char index safety — the `&` must be found at a char boundary.)
    #[test]
    fn test_f_p5_002_multibyte_char_adjacent_to_entity() {
        // 'é' is 2 bytes (U+00E9). The sentinel `é&amp;` must decode to `é&`.
        let result = decode_xml_entities("é&amp;");
        assert_eq!(result, "é&");

        // Multi-byte char after the entity must also be intact.
        let result2 = decode_xml_entities("&amp;é");
        assert_eq!(result2, "&é");
    }
}
