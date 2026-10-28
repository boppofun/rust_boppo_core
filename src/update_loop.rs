//! Provides the [`Updatable`] trait for calling a function at a fixed interval.
use crate::MainFramebuffer;

use std::sync::atomic::{AtomicBool, Ordering};

use embassy_time::{Duration, Instant, Timer};

static REPORT_FPS: AtomicBool = AtomicBool::new(false);

/// Turns on or off logging the frame rate and update times of every [`run`] loop, every 5
/// seconds. Off by default.
///
/// Useful to check whether updates fit in the frame: if the max update time is longer than a
/// frame, the frame rate drops below what was asked for.
pub fn set_report_fps(enabled: bool) {
    REPORT_FPS.store(enabled, Ordering::Relaxed);
}

/// Represents control flow for Update loops in the `run` function.
#[derive(PartialEq, Eq, Debug)]
pub enum LoopControlFlow {
    /// Updatable should be called again after the next time delta.
    Continue,
    /// Updatable is complete and should not be called again.
    Break,
}

/// An `Updatable` expects to have `update` called at a desired fixed (but not guaranteed) "frame" rate.
///
/// Use `run` to have `update` called until `LoopControlFlow::Break` is returned.
pub trait Updatable: Send {
    /// Called each interval (e.g. frame) to update the object.
    ///
    /// - `delta_time` is the actual time that has passed since the previous update was called.
    ///   It might be longer than the frame rate if, for example, if the CPU has been very busy
    ///   or if updates have been paused.
    fn update(&mut self, delta_time: std::time::Duration) -> LoopControlFlow;

    /// Run this updatable
    fn run(&mut self, frames_per_second: u16) -> impl Future<Output = ()>
    where
        Self: Sized,
    {
        run(self, frames_per_second)
    }
}

/// Minimum time [`run`] sleeps between updates, even if an update takes longer than a frame.
const MIN_SLEEP: Duration = Duration::from_millis(1);

/// Call `update` on `updatable` in a loop at a rate of `frames_per_second` until
/// `LoopControlFlow::Break` is returned by `updatable`.
///
/// If an update takes longer than a frame, the next update still waits at least 1 ms so other
/// tasks and the host get to run, and the frame rate drops.
///
/// To check how long updates take, turn on [`set_report_fps`].
pub async fn run(updatable: &mut dyn Updatable, frames_per_second: u16) {
    // compute once the normal matching delay for `frames_per_second`
    let delay = embassy_time::Duration::from_millis((1000.0 / frames_per_second as f64) as u64);

    // Update work start time. On the first frame, since there is no previous frame, use the current instant
    // before the first frame delay (the first delta_time will be shorter)
    let mut update_start = Instant::now();

    let mut stats: Option<FrameStats> = None;

    // Actual update cycle loop
    'update_loop: loop {
        let now = Instant::now();
        let elapsed = now.duration_since(update_start);
        // Always sleep, even when the previous update overran the frame. Otherwise the loop never
        // yields and the host can't deliver events (button presses, exit, audio finished) or run
        // other tasks.
        let sleep_time = delay
            .checked_sub(elapsed)
            .unwrap_or_default()
            .max(MIN_SLEEP);
        Timer::after(sleep_time).await;

        let flush_guard = MainFramebuffer::get().pause_auto_flush();

        // Immediately update the update_start instant to start tracking the next update cycle time
        let now = Instant::now();
        // Compute precise delta time for the upcoming update work
        let delta_time = now.duration_since(update_start);
        update_start = now;

        // Update the updatable and stop the cycle if the call returns a Break value
        let control_flow = updatable.update(delta_time.into());
        drop(flush_guard);
        if REPORT_FPS.load(Ordering::Relaxed) {
            stats
                .get_or_insert_with(|| FrameStats::new(update_start))
                .add(update_start, Instant::now(), frames_per_second);
        } else {
            stats = None;
        }
        if control_flow == LoopControlFlow::Break {
            break 'update_loop;
        }
    }
}

/// Frame rate and update times collected by [`run`] when [`set_report_fps`] is on.
struct FrameStats {
    since: Instant,
    frames: u32,
    min: Duration,
    max: Duration,
    total: Duration,
}

impl FrameStats {
    const REPORT_EVERY: Duration = Duration::from_secs(5);

    fn new(since: Instant) -> FrameStats {
        FrameStats {
            since,
            frames: 0,
            min: Duration::MAX,
            max: Duration::MIN,
            total: Duration::MIN,
        }
    }

    /// Records an update (including flushing the lights) that ran from `update_start` to `now`,
    /// and logs and resets the stats every [`Self::REPORT_EVERY`].
    fn add(&mut self, update_start: Instant, now: Instant, frames_per_second: u16) {
        let update_time = now - update_start;
        self.frames += 1;
        self.min = self.min.min(update_time);
        self.max = self.max.max(update_time);
        self.total += update_time;
        let period = now - self.since;
        if period >= Self::REPORT_EVERY {
            log::info!(
                "update_loop: {:.1}/{} fps, update ms min/avg/max {:.1}/{:.1}/{:.1}",
                self.frames as f32 / (period.as_micros() as f32 / 1_000_000.0),
                frames_per_second,
                ms(self.min),
                ms(self.total) / self.frames as f32,
                ms(self.max),
            );
            *self = FrameStats::new(now);
        }
    }
}

fn ms(d: Duration) -> f32 {
    d.as_micros() as f32 / 1000.0
}

/// Run Updatables in series
pub struct InSeries {
    // Executes last to first
    updatables: Vec<Box<dyn Updatable>>,
}

impl InSeries {
    /// Run each updatable one after the other, moving to the next when `LoopControlFlow::Break` is returned.
    #[must_use]
    pub fn new(mut updatables: Vec<Box<dyn Updatable>>) -> InSeries {
        updatables.reverse();
        InSeries { updatables }
    }
}

impl Updatable for InSeries {
    fn update(&mut self, delta_time: std::time::Duration) -> LoopControlFlow {
        let Some(updatable) = &mut self.updatables.last_mut() else {
            return LoopControlFlow::Break;
        };
        let res = updatable.update(delta_time);
        match res {
            LoopControlFlow::Continue => (),
            LoopControlFlow::Break => {
                self.updatables.pop();
            }
        }
        LoopControlFlow::Continue
    }
}

/// Run Updatables in parallel
pub struct InParallel {
    updatables: Vec<Box<dyn Updatable>>,
}

impl InParallel {
    /// Run all updatables at the same time. For each frame they are run in order.
    /// An updatable is removed after it returns Break.
    #[must_use]
    pub fn new(updatables: Vec<Box<dyn Updatable>>) -> InParallel {
        InParallel { updatables }
    }
}

impl Updatable for InParallel {
    fn update(&mut self, delta_time: std::time::Duration) -> LoopControlFlow {
        self.updatables
            .retain_mut(|up| up.update(delta_time) == LoopControlFlow::Continue);
        if self.updatables.is_empty() {
            LoopControlFlow::Break
        } else {
            LoopControlFlow::Continue
        }
    }
}

/// Returns [`LoopControlFlow::Break`] after a specified time has elapsed when run via [`run`].
pub struct DoNothingFor {
    remaining: std::time::Duration,
}

impl DoNothingFor {
    /// Returns a [`DoNothingFor`] that will return [`LoopControlFlow::Break`] after at least
    /// `time` has elapsed.
    #[must_use]
    pub fn new(time: std::time::Duration) -> DoNothingFor {
        DoNothingFor { remaining: time }
    }
}

impl Updatable for DoNothingFor {
    fn update(&mut self, delta_time: std::time::Duration) -> LoopControlFlow {
        self.remaining = self.remaining.checked_sub(delta_time).unwrap_or_default();
        if self.remaining == std::time::Duration::from_millis(0) {
            LoopControlFlow::Break
        } else {
            LoopControlFlow::Continue
        }
    }
}

impl<F> Updatable for F
where
    F: FnMut(std::time::Duration) -> LoopControlFlow + Send,
{
    fn update(&mut self, delta_time: std::time::Duration) -> LoopControlFlow {
        self(delta_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::pin;
    use std::task::{Context, Waker};

    #[test]
    fn run_yields_after_overrunning_update() {
        let _ = crate::internal::LIGHTS.set(MainFramebuffer::new(|_| {}));
        let mut updates = 0;
        let mut updatable = |_| {
            updates += 1;
            // Take longer than the 10 ms frame
            std::thread::sleep(std::time::Duration::from_millis(20));
            LoopControlFlow::Continue
        };
        let mut cx = Context::from_waker(Waker::noop());
        {
            let mut fut = pin!(run(&mut updatable, 100));
            // Sleeps before the first update
            assert!(fut.as_mut().poll(&mut cx).is_pending());
            std::thread::sleep(std::time::Duration::from_millis(15));
            // Runs one update that overruns the frame, then must still yield
            assert!(fut.as_mut().poll(&mut cx).is_pending());
        }
        assert_eq!(updates, 1);
    }

    #[test]
    fn frame_stats_reset_after_report() {
        let start = Instant::from_millis(0);
        let mut stats = FrameStats::new(start);
        let mut t = 0;
        for update_ms in [10, 30, 20] {
            stats.add(
                Instant::from_millis(t),
                Instant::from_millis(t + update_ms),
                30,
            );
            t += 100;
        }
        assert_eq!(stats.frames, 3);
        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(30));
        assert_eq!(stats.total, Duration::from_millis(60));

        stats.add(Instant::from_millis(4_990), Instant::from_millis(5_000), 30);
        assert_eq!(stats.frames, 0);
        assert_eq!(stats.since, Instant::from_millis(5_000));
    }
}
