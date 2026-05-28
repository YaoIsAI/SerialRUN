use std::sync::mpsc;
use std::time::Duration;

/// A handle for async operations that run in background threads
pub struct AsyncHandle<T> {
    receiver: Option<mpsc::Receiver<T>>,
    result: Option<T>,
}

impl<T> AsyncHandle<T> {
    pub fn new() -> Self {
        Self { receiver: None, result: None }
    }

    /// Start an async operation
    pub fn start<F>(&mut self, f: F)
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = f();
            let _ = tx.send(result);
        });
        self.receiver = Some(rx);
        self.result = None;
    }

    /// Check if the operation is complete and return the result
    pub fn poll(&mut self) -> Option<T> {
        if let Some(result) = self.result.take() {
            return Some(result);
        }
        if let Some(ref rx) = self.receiver {
            if let Ok(result) = rx.try_recv() {
                self.receiver = None;
                self.result = Some(result);
                return self.result.take();
            }
        }
        None
    }

    /// Check if an operation is in progress
    pub fn is_running(&self) -> bool {
        self.receiver.is_some()
    }
}

/// Run a blocking operation with a timeout on a background thread
pub fn run_with_timeout<F, T>(f: F, timeout: Duration) -> Option<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = f();
        let _ = tx.send(result);
    });
    rx.recv_timeout(timeout).ok()
}

/// Helper for serial port operations that need wait+read pattern
pub fn serial_read_with_wait<F, R>(
    write_fn: F,
    wait_ms: u64,
    read_fn: impl FnOnce() -> R,
) -> (Option<R>, String)
where
    F: FnOnce() -> Result<(), String>,
    R: Send + 'static,
{
    let write_result = write_fn();
    if let Err(e) = write_result {
        return (None, e);
    }
    std::thread::sleep(Duration::from_millis(wait_ms));
    let result = read_fn();
    (Some(result), String::new())
}
