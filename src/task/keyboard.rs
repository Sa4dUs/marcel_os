use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

/// A static queue used to store scancodes from the keyboard input.
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
/// A static waker used to wake up the task when new scancodes are available.
static WAKER: AtomicWaker = AtomicWaker::new();

/// Adds a scancode to the scancode queue and wakes up any waiting tasks.
///
/// # Arguments
/// * `scancode` - The scancode to add to the queue.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

/// A stream representing the scancode input from the keyboard.
pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    /// Creates a new `ScancodeStream` and initializes the scancode queue.
    ///
    /// # Returns
    /// A new `ScancodeStream` instance.
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Default for ScancodeStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    /// Polls the `ScancodeStream` to get the next scancode. If no scancode is available,
    /// it will register the waker to be notified when a scancode is pushed into the queue.
    ///
    /// # Arguments
    /// * `cx` - The context containing the waker for this task.
    ///
    /// # Returns
    /// A `Poll` indicating whether a scancode is ready (`Poll::Ready(Some(scancode))`) or if it's pending (`Poll::Pending`).
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // If there's a scancode available, return it immediately.
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        // Register the waker if no scancode is available.
        WAKER.register(cx.waker());
        match queue.pop() {
            // If a scancode is available after the task is woken, return it.
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            // No scancode is available yet, return `Poll::Pending`.
            None => Poll::Pending,
        }
    }
}

/// Asynchronously prints the characters or raw key codes from the keyboard input stream.
pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    // Process each scancode from the stream.
    while let Some(scancode) = scancodes.next().await {
        // Add the scancode to the keyboard input and process it.
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                // Match on the key and print the corresponding character or key code.
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
