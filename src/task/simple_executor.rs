use super::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// A simple executor that manages a queue of tasks to run.
pub struct SimpleExecutor {
    /// The queue holding tasks that are ready to be executed.
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    /// Creates a new `SimpleExecutor` with an empty task queue.
    ///
    /// # Returns
    /// A new `SimpleExecutor` instance.
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    /// Spawns a new task and adds it to the task queue.
    ///
    /// # Arguments
    /// * `task` - The task to be added to the queue.
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }

    /// Runs tasks from the task queue until the queue is empty.
    ///
    /// The tasks are polled in the order they were added to the queue.
    /// If a task is not ready to complete, it is placed back into the queue to be polled again later.
    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {}
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}

impl Default for SimpleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a dummy `Waker` for polling tasks. This is necessary for providing a waker
/// to the `Context` used when polling tasks, though no actual waking behavior is implemented.
fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

/// Creates a dummy `RawWaker` for use in the dummy `Waker`. This `RawWaker` has no-op functions
/// since the tasks in this executor do not actually require waking to make progress.
fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(core::ptr::null::<()>(), vtable)
}
