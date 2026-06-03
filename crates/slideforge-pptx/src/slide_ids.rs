//! Slide ID assignment for PPTX export.
//!
//! [`SlideIdAssigner`] produces the deterministic slide ID sequence required
//! by BC-4.01.005: IDs start at exactly 256 and increment by 1 per slide.
//!
//! [`MASTER_ID`] is the canonical slide master ID constant (2^31).
//!
//! ## BC-4.01.005 invariants covered
//!
//! - Invariant 1: slide IDs start at exactly 256.
//! - Invariant 2: master ID is exactly 2,147,483,648 (2^31).
//! - Invariant 4: ID assignment happens in the PPTX exporter, not the layout engine.

/// The minimum slide ID required by ECMA-376 (BC-4.01.005 invariant 1 / Spike S6 BUG-006).
///
/// All generated `<p:sldId id="...">` attributes must use this as the starting
/// value. IDs are sequential: 256, 257, 258, …
pub const SLIDE_ID_START: u32 = 256;

/// The slide master ID required by several PPTX renderers (BC-4.01.005 invariant 2).
///
/// `<p:sldMasterId id="2147483648">` must appear in `presentation.xml`.
/// This value is `2^31`.
pub const MASTER_ID: u32 = 2_147_483_648;

/// Assigns deterministic slide IDs for a deck.
///
/// Encapsulates BC-4.01.005 invariant 1: IDs start at exactly [`SLIDE_ID_START`]
/// (256) and increment by 1. This is the single authoritative code path for slide
/// ID assignment — no ad-hoc ID generation elsewhere in the exporter.
///
/// ## Example
///
/// ```rust
/// use slideforge_pptx::slide_ids::SlideIdAssigner;
/// let ids = SlideIdAssigner::assign(3);
/// assert_eq!(ids, vec![256, 257, 258]);
/// ```
pub struct SlideIdAssigner;

impl SlideIdAssigner {
    /// Return the ordered slide ID sequence for `count` slides.
    ///
    /// The sequence is `[SLIDE_ID_START, SLIDE_ID_START+1, ..., SLIDE_ID_START+count-1]`.
    /// For an empty deck, returns an empty `Vec`.
    ///
    /// # Panics
    ///
    /// Panics if `count > u32::MAX as usize - SLIDE_ID_START as usize`. In practice
    /// a deck cannot have 4 billion slides, so this is a documented infallible path.
    #[must_use]
    pub fn assign(count: usize) -> Vec<u32> {
        (0..count)
            .map(|i| {
                SLIDE_ID_START
                    + u32::try_from(i).expect(
                        "slide count must fit in u32: \
                         a deck cannot have more than ~4 billion slides",
                    )
            })
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items, clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_slide_id_assigner_zero_slides_returns_empty() {
        assert_eq!(SlideIdAssigner::assign(0), Vec::<u32>::new());
    }

    #[test]
    fn test_slide_id_assigner_one_slide_returns_256() {
        assert_eq!(SlideIdAssigner::assign(1), vec![256]);
    }

    #[test]
    fn test_slide_id_assigner_three_slides_returns_256_257_258() {
        assert_eq!(SlideIdAssigner::assign(3), vec![256, 257, 258]);
    }

    #[test]
    fn test_master_id_constant_is_2_to_31() {
        assert_eq!(MASTER_ID, 2_u32.pow(31));
    }
}
