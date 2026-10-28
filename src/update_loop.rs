//! Provides the [`Updatable`] trait for calling a function at a fixed interval.
use crate::MainFramebuffer;

use embassy_time::{Instant, Timer};

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
const MIN_SLEEP: embassy_time::Duration = embassy_time::Duration::from_millis(1);

/// Call `update` on `updatable` in a loop at a rate of `frames_per_second` until
/// `LoopControlFlow::Break` is returned by `updatable`.
///
/// If an update takes longer than a frame, the next update still waits at least 1 ms so other
/// tasks and the host get to run, and the frame rate drops.
pub async fn run(updatable: &mut dyn Updatable, frames_per_second: u16) {
    // compute once the normal matching delay for `frames_per_second`
    let delay = embassy_time::Duration::from_millis((1000.0 / frames_per_second as f64) as u64);

    // Update work start time. On the first frame, since there is no previous frame, use the current instant
    // before the first frame delay (the first delta_time will be shorter)
    let mut update_start = Instant::now();

    const REPORT_FPS: bool = false;
    let mut elapseds = Vec::new();
    let mut last_fps_report = update_start;

    // Actual update cycle loop
    'update_loop: loop {
        let now = Instant::now();
        let elapsed = now.duration_since(update_start);
        if REPORT_FPS {
            elapseds.push(elapsed.as_millis());
            let time_since_last_report = now - last_fps_report;
            if time_since_last_report >= embassy_time::Duration::from_secs(5) {
                let fps = elapseds.len() as f32
                    / (time_since_last_report.as_micros() as f32 / 1_000_000.0);
                log::info!("fps: {}", fps);
                log::info!("processing times (ms): {:?}", elapseds);
                last_fps_report = now;
                elapseds.clear();
            }
        }
        // Always sleep, even when the previous update overran the frame. Otherwise the loop never
        // yields and the host can't deliver events (button presses, exit, audio finished) or run
        // other tasks.
        let sleep_time = delay
            .checked_sub(elapsed)
            .unwrap_or_default()
            .max(MIN_SLEEP);
        Timer::after(sleep_time).await;

        let _flush_guard = MainFramebuffer::get().pause_auto_flush();

        // Immediately update the update_start instant to start tracking the next update cycle time
        let now = Instant::now();
        // Compute precise delta time for the upcoming update work
        let delta_time = now.duration_since(update_start);
        update_start = now;

        // Update the updatable and stop the cycle if the call returns a Break value
        if updatable.update(delta_time.into()) == LoopControlFlow::Break {
            break 'update_loop;
        }
    }
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
}
