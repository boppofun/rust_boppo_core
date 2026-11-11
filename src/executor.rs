//! An async executor for activities
use std::{
    future::Future,
    time::{Duration, Instant},
};

pub use edge_executor::Task;

const MAX_NUM_TASKS: usize = 32;

/// The static thread-local executor used by Boppo
pub type Executor = edge_executor::LocalExecutor<'static, MAX_NUM_TASKS>;

/// Spawn a future on the default executor.
///
/// The future is cancelled when the returned [`Task`] is dropped unless the task is detached.
#[allow(
    clippy::missing_panics_doc,
    reason = "the device or framework always sets the executor before activities run"
)]
pub fn spawn<F, T>(fut: F) -> Task<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let ptr = crate::internal::EXECUTOR.load(std::sync::atomic::Ordering::SeqCst);
    assert!(
        !ptr.is_null(),
        "executor not set, call internal::set_executor first"
    );
    // SAFETY: `ptr` is non-null and was set from a `&'static Executor`, so it is
    // valid for the rest of the program.
    //
    // `LocalExecutor` is `!Sync`, but this may be called from any thread. That is
    // sound because its run queue and task state are thread-safe, and the only
    // thing `LocalExecutor::spawn` allows beyond the thread-safe
    // `edge_executor::Executor::spawn` is spawning `!Send` futures. The `Send`
    // bounds above rule that out, so the future is only ever polled by the thread
    // that runs the executor.
    let executor = unsafe { &*ptr };
    executor.spawn(fut)
}

/// Async sleep for at least `dur`.
pub async fn sleep(dur: Duration) {
    embassy_time::Timer::after(embassy_time::Duration::from_micros(dur.as_micros() as u64)).await;
}

/// Async sleep for at least `ms` milliseconds.
pub async fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

/// Async sleep until after `t` has passed.
pub async fn sleep_until(t: Instant) {
    sleep(t.saturating_duration_since(Instant::now())).await;
}
