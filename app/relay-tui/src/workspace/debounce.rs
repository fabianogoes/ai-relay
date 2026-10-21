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
    on_idle: impl FnMut() -> bool,
) {
    debounce_with_receive(
        |within| raw.recv_timeout(within),
        out,
        quiescence,
        stop,
        on_idle,
    );
}

// Keep the wait boundary injectable so tests can place events inside a window
// without relying on the operating system to resume a sleeping test in time.
fn debounce_with_receive(
    mut receive: impl FnMut(Duration) -> Result<(), RecvTimeoutError>,
    out: Sender<WorkspaceEvent>,
    quiescence: Duration,
    stop: &AtomicBool,
    mut on_idle: impl FnMut() -> bool,
) {
    loop {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        match receive(IDLE_POLL) {
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
            match receive(quiescence) {
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

    use WorkspaceEvent::{Dirty, Settled};

    struct Rig {
        raw: Sender<()>,
        events: Receiver<WorkspaceEvent>,
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
    fn every_write_restarts_the_window_and_a_burst_yields_one_signal_pair() {
        let (out, events) = mpsc::channel();
        // Each receive says how long to wait, what has been emitted so far,
        // and whether another write arrives before that wait expires.
        let mut steps = [
            (IDLE_POLL, None, Ok(())),
            (Duration::from_millis(150), Some(Dirty), Ok(())),
            (Duration::from_millis(150), None, Ok(())),
            (Duration::from_millis(150), None, Ok(())),
            (Duration::from_millis(150), None, Ok(())),
            (
                Duration::from_millis(150),
                None,
                Err(RecvTimeoutError::Timeout),
            ),
            (
                IDLE_POLL,
                Some(Settled),
                Err(RecvTimeoutError::Disconnected),
            ),
        ]
        .into_iter();
        debounce_with_receive(
            |within| {
                let (expected_wait, emitted, result) = steps.next().expect("unexpected wait");
                assert_eq!(within, expected_wait);
                assert_eq!(events.try_recv().ok(), emitted);
                result
            },
            out,
            QUIESCENCE,
            &AtomicBool::new(false),
            || false,
        );
        assert!(steps.next().is_none(), "the burst was not fully processed");
        assert_eq!(events.try_recv(), Err(mpsc::TryRecvError::Disconnected));
    }

    #[test]
    fn two_bursts_apart_yield_two_pairs() {
        let rig = start(QUIESCENCE, || false);
        rig.raw.send(()).unwrap();
        assert_eq!(
            (next(&rig, LONG), next(&rig, LONG)),
            (Some(Dirty), Some(Settled))
        );
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
    fn the_stop_flag_ends_the_loop_before_another_idle_wait() {
        for initially_stopped in [false, true] {
            let stop = AtomicBool::new(initially_stopped);
            let (out, events) = mpsc::channel();
            let mut waited = false;
            debounce_with_receive(
                |within| {
                    assert!(!waited, "waited again after a stop request");
                    assert_eq!(within, Duration::from_millis(200));
                    waited = true;
                    Err(RecvTimeoutError::Timeout)
                },
                out,
                QUIESCENCE,
                &stop,
                || {
                    stop.store(true, Ordering::Relaxed);
                    false
                },
            );
            assert_eq!(waited, !initially_stopped);
            assert_eq!(events.try_recv(), Err(mpsc::TryRecvError::Disconnected));
        }
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
