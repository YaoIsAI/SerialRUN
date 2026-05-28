use anyhow::Result;
use clap::{Parser, Subcommand};
use serialrun_core::{SerialConfig, SerialPort};
use std::io::{self, Write};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "serialrun")]
#[command(about = "SerialRUN - Cross-platform serial port assistant for embedded developers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available serial ports
    List {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Connect to a serial port
    Connect {
        /// Port name (e.g., COM1, /dev/ttyUSB0)
        port: String,

        /// Baud rate
        #[arg(short, long, default_value = "115200")]
        baud: u32,

        /// Data bits (5, 6, 7, 8)
        #[arg(short = 'd', long, default_value = "8")]
        data_bits: u8,

        /// Stop bits (1, 2)
        #[arg(short = 's', long, default_value = "1")]
        stop_bits: u8,

        /// Parity (none, odd, even)
        #[arg(short = 'p', long, default_value = "none")]
        parity: String,

        /// Flow control (none, software, hardware)
        #[arg(short = 'f', long, default_value = "none")]
        flow_control: String,
    },

    /// Send data to a serial port
    Send {
        /// Port name
        port: String,

        /// Data to send
        data: String,

        /// Baud rate
        #[arg(short, long, default_value = "115200")]
        baud: u32,

        /// Send as hex
        #[arg(short = 'x', long)]
        hex: bool,
    },

    /// Monitor serial port data
    Monitor {
        /// Port name
        port: String,

        /// Baud rate
        #[arg(short, long, default_value = "115200")]
        baud: u32,

        /// Log file path
        #[arg(short, long)]
        log: Option<String>,

        /// Show timestamps
        #[arg(short = 't', long)]
        timestamp: bool,

        /// Hex mode
        #[arg(short = 'x', long)]
        hex: bool,
    },

    /// Replay a script
    Replay {
        /// Port name
        port: String,

        /// Script file path
        script: String,

        /// Baud rate
        #[arg(short, long, default_value = "115200")]
        baud: u32,
    },

    /// Record a script
    Record {
        /// Port name
        port: String,

        /// Output script file
        #[arg(short, long, default_value = "script.txt")]
        output: String,

        /// Baud rate
        #[arg(short, long, default_value = "115200")]
        baud: u32,
    },

    /// Agent mode (JSON output for automation)
    Agent {
        /// Port name (optional, lists ports if not provided)
        port: Option<String>,

        /// Action to perform
        #[command(subcommand)]
        action: AgentAction,
    },
}

#[derive(Subcommand)]
enum AgentAction {
    /// List all ports
    ListPorts,

    /// Get port info
    PortInfo {
        /// Port name
        name: String,
    },

    /// Send data
    Send {
        /// Data to send
        data: String,

        /// Baud rate
        #[arg(short, long, default_value = "115200")]
        baud: u32,
    },

    /// Read data with timeout
    Read {
        /// Timeout in milliseconds
        #[arg(short, long, default_value = "1000")]
        timeout: u64,

        /// Maximum bytes to read
        #[arg(short, long, default_value = "1024")]
        max_bytes: usize,
    },

    /// Run a script
    RunScript {
        /// Script file path
        script: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::List { format } => cmd_list(&format),
        Commands::Connect {
            port,
            baud,
            data_bits,
            stop_bits,
            parity,
            flow_control,
        } => cmd_connect(&port, baud, data_bits, stop_bits, &parity, &flow_control),
        Commands::Send {
            port,
            data,
            baud,
            hex,
        } => cmd_send(&port, &data, baud, hex),
        Commands::Monitor {
            port,
            baud,
            log,
            timestamp,
            hex,
        } => cmd_monitor(&port, baud, log.as_deref(), timestamp, hex),
        Commands::Replay {
            port,
            script,
            baud,
        } => cmd_replay(&port, &script, baud),
        Commands::Record {
            port,
            output,
            baud,
        } => cmd_record(&port, &output, baud),
        Commands::Agent { port, action } => cmd_agent(port.as_deref(), action),
    }
}

fn cmd_list(format: &str) -> Result<()> {
    let ports = SerialPort::list_ports()?;

    if ports.is_empty() {
        println!("No serial ports found");
        return Ok(());
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&ports)?;
            println!("{}", json);
        }
        _ => {
            println!("Available serial ports:");
            println!("{:<20} {:<30} {:<10}", "Port", "Description", "VID:PID");
            println!("{}", "-".repeat(60));
            for port in &ports {
                let vid_pid = match (port.vid, port.pid) {
                    (Some(vid), Some(pid)) => format!("{:04X}:{:04X}", vid, pid),
                    _ => "N/A".to_string(),
                };
                println!(
                    "{:<20} {:<30} {:<10}",
                    port.name,
                    port.description.as_deref().unwrap_or("N/A"),
                    vid_pid
                );
            }
        }
    }

    Ok(())
}

fn cmd_connect(
    port_name: &str,
    baud: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: &str,
    flow_control: &str,
) -> Result<()> {
    let config = SerialConfig::new(port_name)
        .with_baud_rate(baud)
        .with_data_bits(match data_bits {
            5 => serialrun_core::config::DataBits::Five,
            6 => serialrun_core::config::DataBits::Six,
            7 => serialrun_core::config::DataBits::Seven,
            _ => serialrun_core::config::DataBits::Eight,
        })
        .with_stop_bits(match stop_bits {
            2 => serialrun_core::config::StopBits::Two,
            _ => serialrun_core::config::StopBits::One,
        })
        .with_parity(match parity.to_lowercase().as_str() {
            "odd" => serialrun_core::config::Parity::Odd,
            "even" => serialrun_core::config::Parity::Even,
            _ => serialrun_core::config::Parity::None,
        })
        .with_flow_control(match flow_control.to_lowercase().as_str() {
            "software" => serialrun_core::config::FlowControl::Software,
            "hardware" => serialrun_core::config::FlowControl::Hardware,
            _ => serialrun_core::config::FlowControl::None,
        });

    let mut port = SerialPort::new(config);
    port.connect()?;

    println!("Connected to {} at {} baud", port_name, baud);
    println!("Press Ctrl+C to disconnect");
    println!();

    let mut input = String::new();
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush()?;

        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        let data = input.trim();
        if data.is_empty() {
            continue;
        }

        port.write_string(data)?;
        port.write_string("\r\n")?;

        let mut buf = [0u8; 1024];
        match port.read(&mut buf) {
            Ok(n) => {
                let received = String::from_utf8_lossy(&buf[..n]);
                print!("{}", received);
                io::stdout().flush()?;
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
            }
        }
    }

    port.disconnect()?;
    println!("\nDisconnected");
    Ok(())
}

fn cmd_send(port_name: &str, data: &str, baud: u32, hex: bool) -> Result<()> {
    let config = SerialConfig::new(port_name).with_baud_rate(baud);
    let mut port = SerialPort::new(config);
    port.connect()?;

    let bytes = if hex {
        parse_hex(data)?
    } else {
        data.as_bytes().to_vec()
    };

    port.write(&bytes)?;
    println!("Sent {} bytes to {}", bytes.len(), port_name);

    let mut buf = [0u8; 1024];
    std::thread::sleep(Duration::from_millis(100));
    match port.read(&mut buf) {
        Ok(n) => {
            if n > 0 {
                let received = String::from_utf8_lossy(&buf[..n]);
                println!("Received: {}", received);
            }
        }
        Err(_) => {}
    }

    port.disconnect()?;
    Ok(())
}

fn cmd_monitor(
    port_name: &str,
    baud: u32,
    log_file: Option<&str>,
    timestamp: bool,
    hex: bool,
) -> Result<()> {
    let config = SerialConfig::new(port_name).with_baud_rate(baud);
    let mut port = SerialPort::new(config);
    port.connect()?;

    println!("Monitoring {} at {} baud", port_name, baud);
    println!("Press Ctrl+C to stop");
    println!();

    let mut log_writer = if let Some(path) = log_file {
        Some(std::fs::File::create(path)?)
    } else {
        None
    };

    let mut buf = [0u8; 1024];
    loop {
        match port.read(&mut buf) {
            Ok(n) => {
                if n > 0 {
                    if timestamp {
                        print!("[{}] ", chrono::Local::now().format("%H:%M:%S%.3f"));
                    }

                    if hex {
                        let hex_str: Vec<String> = buf[..n].iter().map(|b| format!("{:02X}", b)).collect();
                        println!("{}", hex_str.join(" "));
                    } else {
                        print!("{}", String::from_utf8_lossy(&buf[..n]));
                    }
                    io::stdout().flush()?;

                    if let Some(ref mut writer) = log_writer {
                        writer.write_all(&buf[..n])?;
                    }
                }
            }
            Err(serialrun_core::port::PortError::ReadError(ref e)) if e.contains("TimedOut") => {
                continue;
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }

    port.disconnect()?;
    Ok(())
}

fn cmd_replay(port_name: &str, script_path: &str, baud: u32) -> Result<()> {
    let config = SerialConfig::new(port_name).with_baud_rate(baud);
    let mut port = SerialPort::new(config);
    port.connect()?;

    let mut replayer = serialrun_core::ScriptReplayer::load(std::path::Path::new(script_path))?;
    replayer.start();

    println!("Replaying script: {}", replayer.script().name);

    while let Some(cmd) = replayer.next_command() {
        match cmd.action {
            serialrun_core::recorder::Action::Send => {
                if let Some(ref data) = cmd.data {
                    println!("Sending: {}", data);
                    port.write_string(data)?;
                    port.write_string("\r\n")?;
                }
            }
            serialrun_core::recorder::Action::Wait => {
                std::thread::sleep(Duration::from_millis(cmd.delay_ms));
            }
            serialrun_core::recorder::Action::Read => {
                let mut buf = [0u8; 1024];
                if let Ok(n) = port.read(&mut buf) {
                    if n > 0 {
                        println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
                    }
                }
            }
            serialrun_core::recorder::Action::Comment => {
                if let Some(ref comment) = cmd.data {
                    println!("# {}", comment);
                }
            }
        }
        std::thread::sleep(replayer.get_delay());
    }

    port.disconnect()?;
    println!("Script replay complete");
    Ok(())
}

fn cmd_record(port_name: &str, output_path: &str, baud: u32) -> Result<()> {
    let config = SerialConfig::new(port_name).with_baud_rate(baud);
    let mut port = SerialPort::new(config);
    port.connect()?;

    let mut recorder = serialrun_core::ScriptRecorder::new("recorded", "Auto-recorded script");
    recorder.start();

    println!("Recording to {}", output_path);
    println!("Type commands to record, 'quit' to stop");
    println!();

    let mut input = String::new();
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush()?;

        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        let data = input.trim();
        if data == "quit" {
            break;
        }

        if !data.is_empty() {
            recorder.record_send(data);
            port.write_string(data)?;
            port.write_string("\r\n")?;
        }
    }

    recorder.stop();
    recorder.save(std::path::Path::new(output_path))?;
    port.disconnect()?;
    println!("Recording saved to {}", output_path);
    Ok(())
}

fn cmd_agent(port_name: Option<&str>, action: AgentAction) -> Result<()> {
    match action {
        AgentAction::ListPorts => {
            let ports = SerialPort::list_ports()?;
            let output = serde_json::json!({
                "success": true,
                "ports": ports
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        AgentAction::PortInfo { name } => {
            let ports = SerialPort::list_ports()?;
            let port = ports.iter().find(|p| p.name == name);
            let output = match port {
                Some(p) => serde_json::json!({
                    "success": true,
                    "port": p
                }),
                None => serde_json::json!({
                    "success": false,
                    "error": format!("Port '{}' not found", name)
                }),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        AgentAction::Send { data, baud } => {
            let port_name = port_name.ok_or_else(|| anyhow::anyhow!("Port name required"))?;
            let config = SerialConfig::new(port_name).with_baud_rate(baud);
            let mut port = SerialPort::new(config);
            port.connect()?;

            let bytes = data.as_bytes();
            let written = port.write(bytes)?;

            let output = serde_json::json!({
                "success": true,
                "bytes_written": written
            });
            println!("{}", serde_json::to_string_pretty(&output)?);

            port.disconnect()?;
        }
        AgentAction::Read { timeout, max_bytes } => {
            let port_name = port_name.ok_or_else(|| anyhow::anyhow!("Port name required"))?;
            let config = SerialConfig::new(port_name).with_timeout(timeout);
            let mut port = SerialPort::new(config);
            port.connect()?;

            let mut buf = vec![0u8; max_bytes];
            match port.read(&mut buf) {
                Ok(n) => {
                    let data = &buf[..n];
                    let hex_data: String = data.iter().map(|b| format!("{:02X}", b)).collect();
                    let output = serde_json::json!({
                        "success": true,
                        "bytes_read": n,
                        "data_hex": hex_data,
                        "data_text": String::from_utf8_lossy(data)
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                Err(e) => {
                    let output = serde_json::json!({
                        "success": false,
                        "error": e.to_string()
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
            }

            port.disconnect()?;
        }
        AgentAction::RunScript { script } => {
            let port_name = port_name.ok_or_else(|| anyhow::anyhow!("Port name required"))?;
            let config = SerialConfig::new(port_name);
            let mut port = SerialPort::new(config);
            port.connect()?;

            let mut replayer = serialrun_core::ScriptReplayer::load(std::path::Path::new(&script))?;
            replayer.start();

            let mut commands_executed = 0;
            while let Some(cmd) = replayer.next_command() {
                match cmd.action {
                    serialrun_core::recorder::Action::Send => {
                        if let Some(ref data) = cmd.data {
                            port.write_string(data)?;
                            port.write_string("\r\n")?;
                        }
                    }
                    serialrun_core::recorder::Action::Wait => {
                        std::thread::sleep(Duration::from_millis(cmd.delay_ms));
                    }
                    _ => {}
                }
                commands_executed += 1;
                std::thread::sleep(replayer.get_delay());
            }

            let output = serde_json::json!({
                "success": true,
                "commands_executed": commands_executed
            });
            println!("{}", serde_json::to_string_pretty(&output)?);

            port.disconnect()?;
        }
    }

    Ok(())
}

fn parse_hex(hex_str: &str) -> Result<Vec<u8>> {
    let hex_str = hex_str.replace(" ", "").replace("0x", "").replace("0X", "");
    let mut bytes = Vec::new();

    for i in (0..hex_str.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex_str[i..i + 2], 16)?;
        bytes.push(byte);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex() {
        let hex = "48 65 6C 6C 6F";
        let bytes = parse_hex(hex).unwrap();
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_parse_hex_no_spaces() {
        let hex = "48656C6C6F";
        let bytes = parse_hex(hex).unwrap();
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_parse_hex_0x_prefix() {
        let hex = "0x48 0x65";
        let bytes = parse_hex(hex).unwrap();
        assert_eq!(bytes, b"He");
    }
}
