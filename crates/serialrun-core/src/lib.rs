pub mod baud_detect;
pub mod checksum;
pub mod config;
pub mod data_logger;
pub mod file_transfer;
pub mod plugin;
pub mod port;
pub mod protocol;
pub mod recorder;

pub use config::SerialConfig;
pub use port::{SerialPort, SerialPortInfo};
pub use plugin::{LoadedPlugin, PluginManager};
pub use recorder::{ScriptRecorder, ScriptReplayer};
