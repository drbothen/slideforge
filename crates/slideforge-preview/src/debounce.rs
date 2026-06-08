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
//!
//! ## Implementation
//!
//! The debouncer spawns a background tokio task that receives trigger signals
//! via an `mpsc` channel. On each signal it restarts a 100ms sleep. When
//! the sleep completes without a new signal, the callback is invoked.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

/// Duration of the debounce window (100ms).
pub const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

/// A debouncer that coalesces rapid events into at most one callback invocation
/// per [`DEBOUNCE_DURATION`] window.
///
/// # AC-008 / BC-4.03.004 EC-005
///
/// 10+ saves per second must result in at most 1 evaluation trigger per 100ms.
pub struct Debouncer {
    /// Channel sender to signal file-change events to the background task.
    tx: mpsc::UnboundedSender<()>,
    /// Count of callback invocations — accessible for test assertions.
    invocation_count: Arc<AtomicUsize>,
}

impl Debouncer {
    /// Create a new `Debouncer`.
    ///
    /// The `callback` is invoked at most once per [`DEBOUNCE_DURATION`] window.
    /// Spawns a background tokio task to manage the debounce timer.
    #[must_use]
    pub fn new(callback: impl Fn() + Send + 'static) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<()>();
        let invocation_count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&invocation_count);

        tokio::spawn(async move {
            loop {
                // Wait for the first trigger.
                // None means the sender was dropped — shut down the task.
                if rx.recv().await.is_none() {
                    break;
                }

                // Drain additional triggers within the debounce window.
                // Reset the timer each time a new trigger arrives.
                loop {
                    match time::timeout(DEBOUNCE_DURATION, rx.recv()).await {
                        Ok(None) => {
                            // Channel closed while draining — fire once and exit.
                            callback();
                            count_clone.fetch_add(1, Ordering::Relaxed);
                            return;
                        },
                        Ok(Some(())) => {
                            // Another trigger arrived — reset timer (continue loop).
                        },
                        Err(_elapsed) => {
                            // Debounce window expired with no new events — fire.
                            callback();
                            count_clone.fetch_add(1, Ordering::Relaxed);
                            break;
                        },
                    }
                }
            }
        });

        Self {
            tx,
            invocation_count,
        }
    }

    /// Signal a file-change event.
    ///
    /// If called multiple times within [`DEBOUNCE_DURATION`], the callback is
    /// invoked only once — after the last call plus the debounce window.
    pub fn trigger(&self) {
        // Ignore send errors — if the task has exited, triggers are no-ops.
        let _ = self.tx.send(());
    }

    /// Returns the number of times the callback has been invoked since creation.
    ///
    /// Used in tests to verify debounce coalescing.
    #[must_use]
    pub fn invocation_count(&self) -> usize {
        self.invocation_count.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
#[allow(non_snake_case)] // BC naming convention: test_BC_S_SS_NNN_xxx (CLAUDE.md)
#[allow(clippy::doc_markdown)] // prose references to method names in test docs
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    /// AC-008 / BC-4.03.004 EC-005 — 10 events in 50ms coalesce to at most 1 callback.
    #[tokio::test]
    async fn test_BC_4_03_004_debounce_10_events_coalesce_to_1() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let debouncer = Debouncer::new(move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        // Fire 10 events within 50ms (every 5ms).
        for _ in 0..10 {
            debouncer.trigger();
            time::sleep(Duration::from_millis(5)).await;
        }

        // Wait 200ms for the debounce window to expire and callback to fire.
        time::sleep(Duration::from_millis(200)).await;

        let count = counter.load(Ordering::Relaxed);
        assert_eq!(
            count, 1,
            "expected exactly 1 callback invocation (burst must coalesce AND fire), got {count}"
        );
    }

    /// AC-008 — single event triggers exactly one callback after debounce window.
    #[tokio::test]
    async fn test_BC_4_03_004_debounce_single_event_triggers_callback() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let debouncer = Debouncer::new(move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        debouncer.trigger();

        // Wait 200ms for the debounce window to expire.
        time::sleep(Duration::from_millis(200)).await;

        let count = counter.load(Ordering::Relaxed);
        assert_eq!(
            count, 1,
            "expected exactly 1 callback invocation, got {count}"
        );
    }

    /// AC-008 — two events 200ms apart each trigger a separate callback.
    #[tokio::test]
    async fn test_BC_4_03_004_debounce_spaced_events_each_trigger() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let debouncer = Debouncer::new(move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        // First event at t=0.
        debouncer.trigger();

        // Wait 150ms — well past the 100ms window — for first callback to fire.
        time::sleep(Duration::from_millis(150)).await;

        // Second event at t=150ms.
        debouncer.trigger();

        // Wait 150ms — well past the 100ms window — for second callback to fire.
        time::sleep(Duration::from_millis(150)).await;

        let count = counter.load(Ordering::Relaxed);
        assert_eq!(
            count, 2,
            "expected 2 callback invocations for spaced events, got {count}"
        );
    }

    /// AC-008 — no event means no callback.
    #[tokio::test]
    async fn test_BC_4_03_004_debounce_no_event_no_callback() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let _debouncer = Debouncer::new(move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        // Wait 200ms without triggering.
        time::sleep(Duration::from_millis(200)).await;

        let count = counter.load(Ordering::Relaxed);
        assert_eq!(
            count, 0,
            "expected 0 callbacks when no events fired, got {count}"
        );
    }
}
