//! 100ms debounce for the file-watcher integration.
//!
//! `Debouncer` coalesces rapid file-change events into at most one evaluation
//! trigger per 100ms window (BC-4.03.004 AC-008 / EC-005).
//!
//! ## Design
//!
//! When a file-change event arrives:
//! 1. If no timer is pending, start a 100ms timer.
//! 2. If a timer is already pending, reset it to fire 100ms from *now*.
//! 3. When the timer fires, invoke the callback exactly once.
//!
//! This prevents evaluation queue explosion when the user saves rapidly
//! (e.g., auto-save on keystroke).

use std::time::Duration;

/// Duration of the debounce window (100ms).
pub const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

/// A debouncer that coalesces rapid events into at most one callback invocation
/// per [`DEBOUNCE_DURATION`] window.
///
/// # AC-008 / BC-4.03.004 EC-005
///
/// 10+ saves per second must result in at most 1 evaluation trigger per 100ms.
pub struct Debouncer {
    /// Internal state — implementation detail.
    _inner: DebouncerInner,
}

/// Implementation detail — hidden from public API.
struct DebouncerInner;

impl Debouncer {
    /// Create a new `Debouncer`.
    ///
    /// The `callback` is invoked at most once per [`DEBOUNCE_DURATION`] window.
    #[must_use]
    pub fn new(_callback: impl Fn() + Send + 'static) -> Self {
        todo!("implement Debouncer::new")
    }

    /// Signal a file-change event.
    ///
    /// If called multiple times within [`DEBOUNCE_DURATION`], the callback is
    /// invoked only once — after the last call plus the debounce window.
    pub fn trigger(&self) {
        todo!("implement Debouncer::trigger")
    }

    /// Returns the number of times the callback has been invoked since creation.
    ///
    /// Used in tests to verify debounce coalescing.
    pub fn invocation_count(&self) -> usize {
        todo!("implement Debouncer::invocation_count")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

    /// AC-008 / BC-4.03.004 EC-005 — 10 events in 50ms coalesce to at most 1 callback.
    #[tokio::test]
    async fn test_BC_4_03_004_debounce_10_events_coalesce_to_1() {
        todo!(
            "implement: signal 10 events within 50ms, wait 200ms, assert callback invoked <= 1 time"
        )
    }

    /// AC-008 — single event triggers exactly one callback after debounce window.
    #[tokio::test]
    async fn test_BC_4_03_004_debounce_single_event_triggers_callback() {
        todo!("implement: signal 1 event, wait 200ms, assert callback invoked exactly once")
    }

    /// AC-008 — two events 200ms apart each trigger a separate callback.
    #[tokio::test]
    async fn test_BC_4_03_004_debounce_spaced_events_each_trigger() {
        todo!(
            "implement: signal event at t=0, wait 150ms, signal again, wait 150ms, assert 2 callbacks"
        )
    }

    /// AC-008 — no event means no callback.
    #[tokio::test]
    async fn test_BC_4_03_004_debounce_no_event_no_callback() {
        todo!("implement: create debouncer, wait 200ms without triggering, assert 0 callbacks")
    }
}
