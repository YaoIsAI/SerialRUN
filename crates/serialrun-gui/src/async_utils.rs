use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;

/// One-shot async operation handle. Spawn a closure on a background thread,
/// poll for result each frame with `try_recv()`.
pub struct AsyncHandle<T> {
    receiver: Option<mpsc::Receiver<T>>,
}

impl<T: Send + 'static> AsyncHandle<T> {
    pub fn new() -> Self {
        Self { receiver: None }
    }

    /// Start an async operation. Returns immediately.
    pub fn start<F>(&mut self, f: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = f();
            let _ = tx.send(result);
        });
        self.receiver = Some(rx);
    }

    /// Poll for result. Returns Some(T) when complete, None if still running.
    pub fn poll(&mut self) -> Option<T> {
        if let Some(ref rx) = self.receiver {
            if let Ok(result) = rx.try_recv() {
                self.receiver = None;
                return Some(result);
            }
        }
        None
    }

    pub fn is_running(&self) -> bool {
        self.receiver.is_some()
    }
}

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

    /// Signal the reader to stop and wait for the thread to finish.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl<T> Drop for PersistentReader<T> {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Helper: run a serial write+read in a background thread.
/// Returns a receiver that yields (write_result, read_result).
pub fn spawn_serial_write_read(
    port_name: String,
    baud_rate: u32,
    data: Vec<u8>,
    wait_ms: u64,
) -> mpsc::Receiver<Result<Vec<u8>, String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let config = serialrun_core::config::SerialConfig {
            port_name,
            baud_rate,
            ..Default::default()
        };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() {
            let _ = tx.send(Err("Connect failed".into()));
            return;
        }
        if port.write(&data).is_err() {
            let _ = port.disconnect();
            let _ = tx.send(Err("Write failed".into()));
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(wait_ms));
        let mut buf = [0u8; 1024];
        let result = match port.read(&mut buf) {
            Ok(n) if n >= 4 => Ok(buf[..n].to_vec()),
            Ok(_) => Err("No response".into()),
            Err(e) => Err(e.to_string()),
        };
        let _ = port.disconnect();
        let _ = tx.send(result);
    });
    rx
}

/// Helper: run a serial write-only in a background thread.
pub fn spawn_serial_write(
    port_name: String,
    baud_rate: u32,
    data: Vec<u8>,
) -> mpsc::Receiver<Result<(), String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let config = serialrun_core::config::SerialConfig {
            port_name,
            baud_rate,
            ..Default::default()
        };
        let mut port = serialrun_core::SerialPort::new(config);
        if port.connect().is_err() {
            let _ = tx.send(Err("Connect failed".into()));
            return;
        }
        let result = port.write(&data).map(|_| ()).map_err(|e| e.to_string());
        let _ = port.disconnect();
        let _ = tx.send(result);
    });
    rx
}
