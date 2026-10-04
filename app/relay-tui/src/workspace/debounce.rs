//! Trailing debounce of raw file-system events into the two signals the view
//! needs: the files started changing (`Dirty`) and they stopped (`Settled`).
//!
//! Every event restarts the window; once it elapses with no new event, the
//! workspace is re-read as a whole. Over channels, so it is tested without a file system.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

/// The quiescence window: 150 ms with no event.
pub const QUIESCENCE: Duration = Duration::from_millis(150);

/// How often an idle loop looks up from waiting, to see whether it was asked
/// to stop and whether `on_idle` has anything new to report.
pub(super) const IDLE_POLL: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceEvent {
    /// The first write of a burst: the last snapshot is now stale.
    Dirty,
    /// `QUIESCENCE` passed with no write: read the workspace again.
    Settled,
}

/// Turns raw events into `Dirty` / `Settled`, until `raw` disconnects, `out`
/// is dropped or `stop` is set.
///
/// `on_idle` runs on every idle wake-up and after every burst; returning `true`
/// from an idle wake-up counts as an event, which is how a directory that
/// appeared after start-up is noticed without a file-system event.
pub(super) fn debounce(
    raw: Receiver<()>,
    out: Sender<WorkspaceEvent>,
    quiescence: Duration,
    stop: &AtomicBool,
    mut on_idle: impl FnMut() -> bool,
) {
    loop {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        match raw.recv_timeout(IDLE_POLL) {
            Ok(()) => {}
            Err(RecvTimeoutError::Timeout) => {
                if !on_idle() {
                    continue;
                }
            }
            Err(RecvTimeoutError::Disconnected) => return,
        }
        if out.send(WorkspaceEvent::Dirty).is_err() {
            return;
        }
        loop {
            match raw.recv_timeout(quiescence) {
                Ok(()) => {}
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        on_idle();
        if out.send(WorkspaceEvent::Settled).is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::mpsc;
    use std::thread::{self, JoinHandle};
    use std::time::Instant;

    use WorkspaceEvent::{Dirty, Settled};

    struct Rig {
        raw: Sender<()>,
        events: Receiver<WorkspaceEvent>,
        stop: Arc<AtomicBool>,
        thread: JoinHandle<()>,
    }

    fn start(quiescence: Duration, on_idle: impl FnMut() -> bool + Send + 'static) -> Rig {
        let (raw, raw_rx) = mpsc::channel();
        let (out, events) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let thread = thread::spawn(move || debounce(raw_rx, out, quiescence, &flag, on_idle));
        Rig {
            raw,
            events,
            stop,
            thread,
        }
    }

    fn next(rig: &Rig, within: Duration) -> Option<WorkspaceEvent> {
        rig.events.recv_timeout(within).ok()
    }

    const LONG: Duration = Duration::from_secs(2);

    #[test]
    fn the_quiescence_window_is_150_ms() {
        assert_eq!(QUIESCENCE, Duration::from_millis(150));
    }

    #[test]
    fn a_burst_of_writes_yields_one_dirty_and_one_settled_after_the_quiescence() {
        let rig = start(QUIESCENCE, || false);
        let mut last_write = Instant::now();
        for _ in 0..5 {
            rig.raw.send(()).unwrap();
            // The reference is the last write itself, not the end of the sleep
            // after it: on a loaded runner that sleep can overshoot by a lot.
            last_write = Instant::now();
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(next(&rig, LONG), Some(Dirty));
        assert_eq!(next(&rig, LONG), Some(Settled));
        // Not before the window after the last write (small slack for the clock).
        assert!(last_write.elapsed() >= QUIESCENCE - Duration::from_millis(15));
        // Exactly one signal pair for the whole burst.
        assert_eq!(next(&rig, Duration::from_millis(500)), None);
    }

    #[test]
    fn a_write_inside_the_window_postpones_the_settled_signal() {
        let rig = start(QUIESCENCE, || false);
        rig.raw.send(()).unwrap();
        assert_eq!(next(&rig, LONG), Some(Dirty));
        thread::sleep(Duration::from_millis(100));
        rig.raw.send(()).unwrap();
        let second_write = Instant::now();
        assert_eq!(next(&rig, LONG), Some(Settled));
        assert!(second_write.elapsed() >= QUIESCENCE - Duration::from_millis(15));
    }

    #[test]
    fn two_bursts_apart_yield_two_pairs() {
        let rig = start(QUIESCENCE, || false);
        rig.raw.send(()).unwrap();
        assert_eq!(
            (next(&rig, LONG), next(&rig, LONG)),
            (Some(Dirty), Some(Settled))
        );
        thread::sleep(Duration::from_millis(100));
        rig.raw.send(()).unwrap();
        assert_eq!(
            (next(&rig, LONG), next(&rig, LONG)),
            (Some(Dirty), Some(Settled))
        );
    }

    #[test]
    fn nothing_is_signalled_without_events() {
        let rig = start(QUIESCENCE, || false);
        assert_eq!(next(&rig, Duration::from_millis(500)), None);
    }

    #[test]
    fn dropping_the_event_source_ends_the_loop() {
        let rig = start(QUIESCENCE, || false);
        drop(rig.raw);
        rig.thread.join().unwrap();
    }

    #[test]
    fn the_stop_flag_ends_the_loop_within_an_idle_poll() {
        let rig = start(QUIESCENCE, || false);
        let asked = Instant::now();
        rig.stop.store(true, Ordering::Relaxed);
        rig.thread.join().unwrap();
        assert!(asked.elapsed() < IDLE_POLL * 3);
    }

    #[test]
    fn something_found_while_idle_counts_as_an_event() {
        let found = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&found);
        let rig = start(QUIESCENCE, move || flag.swap(false, Ordering::Relaxed));
        found.store(true, Ordering::Relaxed);
        assert_eq!(
            (next(&rig, LONG), next(&rig, LONG)),
            (Some(Dirty), Some(Settled))
        );
        assert_eq!(next(&rig, Duration::from_millis(500)), None);
    }
}
