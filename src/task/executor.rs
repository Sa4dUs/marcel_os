use super::{Task, TaskId};
use alloc::task::Wake;
use alloc::{collections::BTreeMap, sync::Arc};
use core::task::Waker;
use core::task::{Context, Poll};
use crossbeam_queue::ArrayQueue;

/// A struct representing the Executor, which manages and runs tasks in a cooperative multitasking system.
pub struct Executor {
    /// A collection of tasks managed by the executor, indexed by their unique task IDs.
    tasks: BTreeMap<TaskId, Task>,
    /// A queue holding the IDs of tasks ready to be executed.
    task_queue: Arc<ArrayQueue<TaskId>>,
    /// A cache of `Waker` objects, indexed by task IDs, to wake tasks when needed.
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    /// Creates a new `Executor` instance with empty task collection, task queue, and waker cache.
    ///
    /// # Returns
    /// A new `Executor` instance.
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    /// Spawns a new task by adding it to the executor's task collection and queue.
    ///
    /// # Arguments
    /// * `task` - The task to be spawned.
    ///
    /// # Panics
    /// Panics if a task with the same ID is already present in the executor.
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    /// Runs tasks that are ready to be executed. This method processes tasks in the task queue.
    fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        // Process all tasks that are ready to run.
        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    /// Starts the executor and runs tasks indefinitely, yielding to the CPU if idle.
    ///
    /// # Returns
    /// This function never returns, running indefinitely.
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    /// Puts the executor to sleep if no tasks are available to run.
    ///
    /// The executor checks if the task queue is empty and halts the processor if no tasks are pending.
    fn sleep_if_idle(&self) {
        if self.task_queue.is_empty() {
            use x86_64::instructions::interrupts::{self, enable_and_hlt};

            interrupts::disable();
            if self.task_queue.is_empty() {
                enable_and_hlt();
            } else {
                interrupts::enable();
            }
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

/// A struct representing a `Waker` for a specific task, allowing it to be awakened from a blocking state.
struct TaskWaker {
    /// The unique identifier of the task that this `Waker` is responsible for.
    task_id: TaskId,
    /// The queue used to schedule tasks for execution.
    task_queue: Arc<ArrayQueue<TaskId>>,
}

#[allow(clippy::new_ret_no_self)]
impl TaskWaker {
    /// Creates a new `Waker` for the specified task.
    ///
    /// # Arguments
    /// * `task_id` - The ID of the task to wake.
    /// * `task_queue` - The queue to push the task ID into when waking it.
    ///
    /// # Returns
    /// A `Waker` instance for the specified task.
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    /// Wakes the task associated with this `TaskWaker` by pushing its ID onto the task queue.
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker {
    /// Wakes the task by calling the `wake_task` method on the `TaskWaker`.
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    /// Wakes the task by calling the `wake_task` method on the `TaskWaker`, keeping the reference intact.
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
