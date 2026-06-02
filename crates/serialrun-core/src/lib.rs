pub mod baud_detect;
pub mod checksum;
pub mod config;
pub mod data_logger;
pub mod file_transfer;
pub mod plugin;
pub mod plugin_install;
pub mod plugin_registry;
pub mod port;
pub mod protocol;
pub mod recorder;

pub use config::SerialConfig;
pub use port::{SerialPort, SerialPortInfo};
pub use plugin::LoadedPlugin;
pub use recorder::{ScriptRecorder, ScriptReplayer};
