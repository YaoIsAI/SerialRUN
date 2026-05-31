use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;

/// Persistent background reader for continuous serial capture.
/// Spawns a thread that loops reading from a serial port and sends
/// parsed results through a channel. Call `poll()` each frame.
pub struct PersistentReader<T> {
    stop: Arc<AtomicBool>,
    receiver: mpsc::Receiver<T>,
    handle: Option<JoinHandle<()>>,
}

impl<T: Send + 'static> PersistentReader<T> {
    /// Start a persistent reader. `read_fn` receives a stop flag and a sender;
    /// it should loop, checking the stop flag, and send parsed data via the sender.
    pub fn start<F>(read_fn: F) -> Self
    where
        F: FnOnce(Arc<AtomicBool>, mpsc::Sender<T>) + Send + 'static,
    {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || read_fn(stop_clone, tx));
        Self {
            stop,
            receiver: rx,
            handle: Some(handle),
        }
    }

    /// Poll for new data. Returns None if nothing available yet.
    pub fn poll(&self) -> Option<T> {
        self.receiver.try_recv().ok()
    }

    /// Signal the reader to stop. Does NOT block — the thread will exit on its own
    /// within ~50ms when it checks the stop flag (port.read has 50ms timeout).
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.handle.take();
    }
}

impl<T> Drop for PersistentReader<T> {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.handle.take();
    }
}
