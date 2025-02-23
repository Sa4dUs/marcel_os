use alloc::boxed::Box;
use core::sync::atomic::{AtomicU64, Ordering};
use core::{future::Future, pin::Pin, task::Context, task::Poll};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

/// A struct representing a unique task identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    /// Creates a new, unique `TaskId` by atomically incrementing a counter.
    ///
    /// # Returns
    /// A new `TaskId` with a unique identifier.
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// A struct representing a task that can be executed in the system.
pub struct Task {
    /// The unique identifier for this task.
    id: TaskId,
    /// The future associated with the task, which will be polled to completion.
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// Creates a new `Task` instance with the provided future.
    ///
    /// # Arguments
    /// * `future` - The future that represents the task's work.
    ///
    /// # Returns
    /// A new `Task` instance containing the given future.
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    /// Polls the task's future to check if it is ready.
    ///
    /// # Arguments
    /// * `context` - The context for the task, which includes the waker to notify when the task can make progress.
    ///
    /// # Returns
    /// A `Poll` indicating whether the future is ready (`Poll::Ready`) or still pending (`Poll::Pending`).
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
