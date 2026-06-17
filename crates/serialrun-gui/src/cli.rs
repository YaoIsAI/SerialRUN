use clap::Parser;

#[derive(Parser)]
#[command(name = "serialrun", about = "SerialRUN — Serial Port Assistant")]
pub struct Cli {
    /// Subcommand
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// List available serial ports
    ListPorts,
    /// Connect to a serial port
    Connect {
        /// Port name (e.g. COM1, /dev/ttyUSB0)
        port: String,
        /// Baud rate
        #[arg(short, long, default_value = "115200")]
        baud: u32,
    },
    /// Send data to serial port
    Send {
        /// Port name
        port: String,
        /// Data to send
        data: String,
        /// Send as hex
        #[arg(short, long)]
        hex: bool,
    },
    /// Read data from serial port
    Read {
        /// Port name
        port: String,
        /// Read timeout in ms
        #[arg(short, long, default_value = "1000")]
        timeout: u64,
    },
}

pub fn run_cli(cli: Cli, _config: Option<&str>) {
    match cli.command {
        Some(Commands::ListPorts) => {
            let ports = serialrun_core::SerialPort::list_ports().unwrap_or_default();
            for p in &ports {
                println!("{} - {}", p.name, p.description.as_deref().unwrap_or(""));
            }
        }
        Some(Commands::Connect { port, baud }) => {
            println!("Connecting to {} at {} baud...", port, baud);
            let config = serialrun_core::config::SerialConfig {
                port_name: port,
                baud_rate: baud,
                ..Default::default()
            };
            let mut sp = serialrun_core::SerialPort::new(config);
            if sp.connect().is_ok() {
                println!("Connected. Press Ctrl+C to disconnect.");
                loop {
                    let mut buf = [0u8; 1024];
                    match sp.read(&mut buf) {
                        Ok(n) if n > 0 => {
                            print!("{}", String::from_utf8_lossy(&buf[..n as usize]));
                        }
                        _ => {}
                    }
                }
            } else {
                eprintln!("Failed to connect.");
            }
        }
        Some(Commands::Send { port, data, hex }) => {
            let config = serialrun_core::config::SerialConfig {
                port_name: port,
                ..Default::default()
            };
            let mut sp = serialrun_core::SerialPort::new(config);
            if sp.connect().is_ok() {
                let bytes = if hex {
                    data.split_whitespace()
                        .filter_map(|b| u8::from_str_radix(b, 16).ok())
                        .collect::<Vec<_>>()
                } else {
                    data.into_bytes()
                };
                match sp.write(&bytes) {
                    Ok(n) => println!("{{\"success\": true, \"bytes_written\": {}}}  ", n),
                    Err(e) => eprintln!("Send failed: {}", e),
                }
                let _ = sp.disconnect();
            }
        }
        Some(Commands::Read { port, timeout }) => {
            let config = serialrun_core::config::SerialConfig {
                port_name: port,
                timeout_ms: timeout,
                ..Default::default()
            };
            let mut sp = serialrun_core::SerialPort::new(config);
            if sp.connect().is_ok() {
                let mut buf = [0u8; 4096];
                match sp.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let data = &buf[..n as usize];
                        let hex: String = data.iter().map(|b| format!("{:02X}", b)).collect();
                        let text = String::from_utf8_lossy(data).to_string();
                        println!("{{\"success\": true, \"bytes_read\": {}, \"data_hex\": \"{}\", \"data_text\": \"{}\"}}  ", n, hex, text.replace('"', "\\\""));
                    }
                    _ => println!("{{\"success\": false, \"error\": \"No data received\"}}"),
                }
                let _ = sp.disconnect();
            }
        }
        None => {
            eprintln!("No command specified. Use --help for usage.");
        }
    }
}
