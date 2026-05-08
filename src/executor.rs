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
pub fn spawn<F, T>(fut: F) -> Task<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let ptr = crate::hal::EXECUTOR.load(std::sync::atomic::Ordering::SeqCst);
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
