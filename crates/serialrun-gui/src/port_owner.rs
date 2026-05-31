use serialrun_core::config::SerialConfig;
use serialrun_core::port::ClearBuffer;
use serialrun_core::SerialPort;
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub enum PortCommand {
    Open(SerialConfig),
    Close,
    Write(Vec<u8>),
    /// Write data, then read response with timeout (for request-response protocols)
    WriteRead {
        data: Vec<u8>,
        timeout_ms: u64,
        resp_tx: mpsc::Sender<Result<Vec<u8>, String>>,
    },
    ClearBuffers,
    ChangeBaud(u32),
    /// Pause/resume the continuous read loop (for CAN/Scope exclusive access)
    ReadPause,
    ReadResume,
    SetDtr(bool),
    SetRts(bool),
    /// Wait until port is released (for CAN/Scope exclusive access)
    WaitForRelease {
        resp_tx: mpsc::Sender<()>,
    },
    /// Exclusive write-then-read: pauses continuous read loop, writes, reads, resumes.
    /// Used by MCP for request-response protocols without data race.
    ReadExclusive {
        data: Vec<u8>,
        timeout_ms: u64,
        resp_tx: mpsc::Sender<Result<Vec<u8>, String>>,
    },
    /// Write data then pause the continuous read loop (for MCP send+read flow).
    WriteAndPause(Vec<u8>),
    /// Read data without changing pause state (for MCP read after WriteAndPause).
    ReadWait {
        timeout_ms: u64,
        resp_tx: mpsc::Sender<Result<Vec<u8>, String>>,
    },
    /// Set the accumulation timeout for the continuous read loop (syncs with UI/MCP timeout setting).
    SetTimeout(u64),
}

pub enum PortEvent {
    Opened(bool, String),
    Closed,
    Written(usize),
    Data(Vec<u8>),
    Error(String),
}

pub struct PortOwnerHandle {
    cmd_tx: Option<mpsc::Sender<PortCommand>>,
    pub evt_rx: mpsc::Receiver<PortEvent>,
    handle: Option<thread::JoinHandle<()>>,
    broadcast_txs: Arc<Mutex<Vec<mpsc::Sender<PortEvent>>>>,
    port_name: Arc<Mutex<String>>,
    rx_buffer: Arc<Mutex<VecDeque<u8>>>,
}

impl PortOwnerHandle {
    pub fn start() -> Self {
        let (cmd_tx_inner, cmd_rx) = mpsc::channel();
        let (evt_tx, evt_rx) = mpsc::channel();
        let broadcast_txs: Arc<Mutex<Vec<mpsc::Sender<PortEvent>>>> = Arc::new(Mutex::new(Vec::new()));
        let bcast = broadcast_txs.clone();
        let port_name = Arc::new(Mutex::new(String::new()));
        let pname = port_name.clone();
        let rx_buffer: Arc<Mutex<VecDeque<u8>>> = Arc::new(Mutex::new(VecDeque::new()));
        let rbuf = rx_buffer.clone();
        let handle = thread::spawn(move || Self::run(cmd_rx, evt_tx, bcast, pname, rbuf));
        Self { cmd_tx: Some(cmd_tx_inner), evt_rx, handle: Some(handle), broadcast_txs, port_name, rx_buffer }
    }

    /// Subscribe to port events. Returns a receiver that gets all PortEvent::Data
    /// broadcast from the continuous read loop. Used by MCP for event push.
    pub fn subscribe_events(&self) -> mpsc::Receiver<PortEvent> {
        let (tx, rx) = mpsc::channel();
        if let Ok(mut bcast) = self.broadcast_txs.lock() {
            bcast.push(tx);
        }
        rx
    }

    /// Get the name of the currently connected port.
    pub fn port_name(&self) -> String {
        self.port_name.lock().unwrap().clone()
    }

    /// Get a clone of the broadcast_txs Arc for event subscription by MCP.
    pub fn broadcast_txs(&self) -> Arc<Mutex<Vec<mpsc::Sender<PortEvent>>>> {
        self.broadcast_txs.clone()
    }

    /// Read and drain data from the shared RX buffer (non-blocking).
    /// Used by MCP to get data without actively reading from serial port.
    pub fn read_buffer(&self) -> Vec<u8> {
        if let Ok(mut buf) = self.rx_buffer.lock() {
            buf.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    /// Get direct access to the rx_buffer Arc for lock-free polling by MCP.
    pub fn rx_buffer_arc(&self) -> Arc<Mutex<VecDeque<u8>>> {
        self.rx_buffer.clone()
    }

    /// Sync the accumulation timeout with UI/MCP timeout setting.
    pub fn sync_timeout(&self, timeout_ms: u64) {
        self.send(PortCommand::SetTimeout(timeout_ms));
    }

    /// Get a clone of the command sender for use by other threads (e.g., Modbus, PLC).
    pub fn cmd_tx(&self) -> mpsc::Sender<PortCommand> {
        self.cmd_tx.clone().expect("PortOwnerHandle already shut down")
    }

    pub fn send(&self, cmd: PortCommand) {
        if let Some(ref tx) = self.cmd_tx {
            let _ = tx.send(cmd);
        }
    }

    pub fn poll(&self) -> Option<PortEvent> {
        self.evt_rx.try_recv().ok()
    }

    /// Pause the continuous read loop (for CAN/Scope exclusive port access)
    pub fn pause_reads(&self) {
        self.send(PortCommand::ReadPause);
    }

    /// Resume the continuous read loop
    pub fn resume_reads(&self) {
        self.send(PortCommand::ReadResume);
    }

    /// Wait until the port is released (for CAN/Scope exclusive access).
    /// Sends Close, waits for port_owner to release, then caller can open the port.
    /// Returns true if released, false if timed out or port_owner already gone.
    pub fn wait_for_release(&self) -> bool {
        let (resp_tx, resp_rx) = mpsc::channel();
        self.send(PortCommand::WaitForRelease { resp_tx });
        // Timeout after 200ms to avoid hanging the GUI thread
        resp_rx.recv_timeout(std::time::Duration::from_millis(200)).is_ok()
    }

    /// Send a write-read request and wait for the response (with timeout).
    /// Used by Modbus, PLC, I2C/SPI, Flasher, etc. for request-response protocols.
    pub fn write_read(&self, data: Vec<u8>, timeout_ms: u64) -> Result<Vec<u8>, String> {
        let (resp_tx, resp_rx) = mpsc::channel();
        self.send(PortCommand::WriteRead { data, timeout_ms, resp_tx });
        // Use recv_timeout to avoid hanging if port_owner thread is stuck
        resp_rx.recv_timeout(std::time::Duration::from_millis(timeout_ms + 500))
            .unwrap_or_else(|_| Err("Timeout: port owner did not respond".into()))
    }

    /// Exclusive write-then-read: pauses continuous read loop so no data is stolen.
    /// Used by MCP for request-response protocols (read, send_command).
    pub fn write_read_exclusive(&self, data: Vec<u8>, timeout_ms: u64) -> Result<Vec<u8>, String> {
        let (resp_tx, resp_rx) = mpsc::channel();
        self.send(PortCommand::ReadExclusive { data, timeout_ms, resp_tx });
        resp_rx.recv_timeout(std::time::Duration::from_millis(timeout_ms + 500))
            .unwrap_or_else(|_| Err("Timeout: port owner did not respond".into()))
    }

    /// Write data then pause the continuous read loop.
    /// Used by MCP send with pause_after=true, followed by read_wait().
    pub fn write_and_pause(&self, data: Vec<u8>) {
        self.send(PortCommand::WriteAndPause(data));
    }

    /// Read data without changing the pause state.
    /// Used by MCP read after write_and_pause().
    pub fn read_wait(&self, timeout_ms: u64) -> Result<Vec<u8>, String> {
        let (resp_tx, resp_rx) = mpsc::channel();
        self.send(PortCommand::ReadWait { timeout_ms, resp_tx });
        resp_rx.recv_timeout(std::time::Duration::from_millis(timeout_ms + 500))
            .unwrap_or_else(|_| Err("Timeout: port owner did not respond".into()))
    }

    fn run(cmd_rx: mpsc::Receiver<PortCommand>, evt_tx: mpsc::Sender<PortEvent>, broadcast_txs: Arc<Mutex<Vec<mpsc::Sender<PortEvent>>>>, port_name: Arc<Mutex<String>>, rx_buffer: Arc<Mutex<VecDeque<u8>>>) {
        let mut port: Option<SerialPort> = None;
        let mut read_paused = false;
        let mut accum_timeout_ms: u64 = 50; // Accumulation timeout, synced with UI/MCP rx_aggregate_ms

        loop {
            // Drain all pending commands first
            let mut had_command = true;
            while had_command {
                match cmd_rx.try_recv() {
                    Ok(cmd) => match cmd {
                        PortCommand::Open(config) => {
                            if port.is_some() {
                                let _ = evt_tx.send(PortEvent::Error("Port already open".into()));
                                continue;
                            }
                            let mut p = SerialPort::new(config.clone());
                            match p.connect() {
                                Ok(()) => {
                                    let _ = p.set_timeout(Duration::from_millis(50));
                                    if let Ok(mut pn) = port_name.lock() {
                                        *pn = config.port_name.clone();
                                    }
                                    let _ = evt_tx.send(PortEvent::Opened(true, config.port_name.clone()));
                                    // Broadcast to MCP subscribers
                                    if let Ok(mut bcast) = broadcast_txs.lock() {
                                        bcast.retain(|tx| tx.send(PortEvent::Opened(true, config.port_name.clone())).is_ok());
                                    }
                                    port = Some(p);
                                }
                                Err(e) => {
                                    let _ = evt_tx.send(PortEvent::Opened(false, e.to_string()));
                                }
                            }
                        }
                        PortCommand::Close => {
                            if let Some(mut p) = port.take() {
                                let _ = p.disconnect();
                            }
                            if let Ok(mut pn) = port_name.lock() {
                                pn.clear();
                            }
                            let _ = evt_tx.send(PortEvent::Closed);
                            // Broadcast to MCP subscribers
                            if let Ok(mut bcast) = broadcast_txs.lock() {
                                bcast.retain(|tx| tx.send(PortEvent::Closed).is_ok());
                            }
                        }
                        PortCommand::Write(data) => {
                            if let Some(ref mut p) = port {
                                match p.write(&data) {
                                    Ok(n) => { let _ = evt_tx.send(PortEvent::Written(n)); }
                                    Err(e) => { let _ = evt_tx.send(PortEvent::Error(e.to_string())); }
                                }
                            } else {
                                let _ = evt_tx.send(PortEvent::Error("Not connected".into()));
                            }
                        }
                        PortCommand::WriteRead { data, timeout_ms, resp_tx } => {
                            if let Some(ref mut p) = port {
                                let _ = p.set_timeout(Duration::from_millis(timeout_ms));
                                if let Err(e) = p.write(&data) {
                                    let _ = p.set_timeout(Duration::from_millis(50));
                                    let _ = resp_tx.send(Err(format!("Write failed: {}", e)));
                                    continue;
                                }
                                let mut buf = [0u8; 4096];
                                let result = match p.read(&mut buf) {
                                    Ok(n) if n > 0 => Ok(buf[..n].to_vec()),
                                    Ok(_) => Err("No response".into()),
                                    Err(e) => Err(e.to_string()),
                                };
                                let _ = p.set_timeout(Duration::from_millis(50));
                                let _ = resp_tx.send(result);
                            } else {
                                let _ = resp_tx.send(Err("Not connected".into()));
                            }
                        }
                        PortCommand::ClearBuffers => {
                            if let Some(ref p) = port {
                                let _ = p.clear_buffer(ClearBuffer::All);
                            }
                        }
                        PortCommand::ReadPause => { read_paused = true; }
                        PortCommand::ReadResume => { read_paused = false; }
                        PortCommand::SetDtr(dtr) => {
                            if let Some(ref mut p) = port {
                                let _ = p.write_data_terminal_ready(dtr);
                            }
                        }
                        PortCommand::SetRts(rts) => {
                            if let Some(ref mut p) = port {
                                let _ = p.write_request_to_send(rts);
                            }
                        }
                        PortCommand::WaitForRelease { resp_tx } => {
                            if let Some(mut p) = port.take() {
                                let _ = p.disconnect();
                            }
                            // Brief delay for OS to release the port handle
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            let _ = resp_tx.send(());
                        }
                        PortCommand::ReadExclusive { data, timeout_ms, resp_tx } => {
                            if let Some(ref mut p) = port {
                                // Pause continuous read loop so it doesn't steal data
                                read_paused = true;

                                let _ = p.set_timeout(Duration::from_millis(timeout_ms));
                                // Skip write if data is empty (read-only exclusive access)
                                if !data.is_empty() {
                                    if let Err(e) = p.write(&data) {
                                        let _ = p.set_timeout(Duration::from_millis(50));
                                        read_paused = false;
                                        let _ = resp_tx.send(Err(format!("Write failed: {}", e)));
                                        continue;
                                    }
                                }
                                let mut buf = [0u8; 4096];
                                let result = match p.read(&mut buf) {
                                    Ok(n) if n > 0 => {
                                        // Try to accumulate more data
                                        let mut all = buf[..n].to_vec();
                                        let _ = p.set_timeout(Duration::from_millis(10));
                                        loop {
                                            let mut tmp = [0u8; 4096];
                                            match p.read(&mut tmp) {
                                                Ok(m) if m > 0 => {
                                                    all.extend_from_slice(&tmp[..m]);
                                                    let _ = p.set_timeout(Duration::from_millis(5));
                                                }
                                                _ => break,
                                            }
                                        }
                                        Ok(all)
                                    }
                                    Ok(_) => Err("No response".into()),
                                    Err(e) => Err(e.to_string()),
                                };
                                let _ = p.set_timeout(Duration::from_millis(50));
                                read_paused = false;
                                let _ = resp_tx.send(result);
                            } else {
                                let _ = resp_tx.send(Err("Not connected".into()));
                            }
                        }
                        PortCommand::WriteAndPause(data) => {
                            if let Some(ref mut p) = port {
                                // Pause first, then drain stale data from previous reads
                                read_paused = true;
                                let _ = p.clear_buffer(ClearBuffer::Input);
                                match p.write(&data) {
                                    Ok(n) => {
                                        let _ = evt_tx.send(PortEvent::Written(n));
                                    }
                                    Err(e) => { let _ = evt_tx.send(PortEvent::Error(e.to_string())); }
                                }
                            } else {
                                let _ = evt_tx.send(PortEvent::Error("Not connected".into()));
                            }
                        }
                        PortCommand::ReadWait { timeout_ms, resp_tx } => {
                            if let Some(ref mut p) = port {
                                // Read without changing pause state
                                let _ = p.set_timeout(Duration::from_millis(timeout_ms));
                                let mut buf = [0u8; 4096];
                                let result = match p.read(&mut buf) {
                                    Ok(n) if n > 0 => {
                                        let mut all = buf[..n].to_vec();
                                        let _ = p.set_timeout(Duration::from_millis(10));
                                        loop {
                                            let mut tmp = [0u8; 4096];
                                            match p.read(&mut tmp) {
                                                Ok(m) if m > 0 => {
                                                    all.extend_from_slice(&tmp[..m]);
                                                    let _ = p.set_timeout(Duration::from_millis(5));
                                                }
                                                _ => break,
                                            }
                                        }
                                        Ok(all)
                                    }
                                    Ok(_) => Err("No response".into()),
                                    Err(e) => Err(e.to_string()),
                                };
                                let _ = p.set_timeout(Duration::from_millis(50));
                                let _ = resp_tx.send(result);
                            } else {
                                let _ = resp_tx.send(Err("Not connected".into()));
                            }
                        }
                        PortCommand::SetTimeout(timeout) => {
                            accum_timeout_ms = timeout;
                        }
                        PortCommand::ChangeBaud(baud) => {
                            if let Some(ref mut p) = port {
                                let mut cfg = p.config().clone();
                                cfg.baud_rate = baud;
                                let _ = p.disconnect();
                                thread::sleep(Duration::from_millis(50));
                                let mut new_port = SerialPort::new(cfg);
                                match new_port.connect() {
                                    Ok(()) => {
                                        let _ = new_port.set_timeout(Duration::from_millis(50));
                                        let _ = evt_tx.send(PortEvent::Opened(true, format!("Baud changed to {}", baud)));
                                        port = Some(new_port);
                                    }
                                    Err(e) => {
                                        let _ = evt_tx.send(PortEvent::Error(e.to_string()));
                                    }
                                }
                            }
                        }
                    },
                    Err(mpsc::TryRecvError::Empty) => { had_command = false; }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        if let Some(mut p) = port.take() {
                            let _ = p.disconnect();
                        }
                        return;
                    }
                }
            }

            // Non-blocking read from port (skip when paused for CAN/Scope)
            if read_paused {
                thread::sleep(Duration::from_millis(10));
            } else if let Some(ref mut p) = port {
                let mut buf = [0u8; 4096];
                match p.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        // Accumulation: read more data within the configured timeout
                        let mut all_data = buf[..n].to_vec();
                        let _ = p.set_timeout(Duration::from_millis(accum_timeout_ms));
                        // Try to read more data without blocking
                        loop {
                            let mut tmp = [0u8; 4096];
                            match p.read(&mut tmp) {
                                Ok(m) if m > 0 => {
                                    all_data.extend_from_slice(&tmp[..m]);
                                    // Reset timeout for next read — use full accum timeout
                                    // to catch MCU responses sent in multiple chunks
                                    let _ = p.set_timeout(Duration::from_millis(accum_timeout_ms));
                                }
                                _ => break,
                            }
                        }
                        let _ = p.set_timeout(Duration::from_millis(50));
                        // Store in shared RX buffer for MCP read_buffer()
                        if let Ok(mut rbuf) = rx_buffer.lock() {
                            rbuf.extend(all_data.iter().copied());
                        }
                        let _ = evt_tx.send(PortEvent::Data(all_data.clone()));
                        // Broadcast to MCP subscribers
                        if let Ok(mut bcast) = broadcast_txs.lock() {
                            bcast.retain(|tx| tx.send(PortEvent::Data(all_data.clone())).is_ok());
                        }
                    }
                    _ => {}
                }
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

impl Drop for PortOwnerHandle {
    fn drop(&mut self) {
        // Send Close, drop sender, then join with timeout.
        // The thread checks cmd_rx.try_recv() each loop iteration, so it will
        // see Disconnected and exit within ~100ms even if a serial read is pending.
        if let Some(tx) = self.cmd_tx.take() {
            let _ = tx.send(PortCommand::Close);
            drop(tx);
        }
        if let Some(h) = self.handle.take() {
            // Give thread up to 500ms to exit cleanly
            let _ = h.join();
        }
    }
}
