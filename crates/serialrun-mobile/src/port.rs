//! Android USB serial port implementation
//!
//! On Android, serial port access goes through USB Host API.
//! This module provides the bridge between Rust and Android's USB stack.

use serialrun_core::config::SerialConfig;
use serialrun_core::port::SerialPortInfo;
use log::{info, error};

/// Mobile serial port wrapper
pub struct MobileSerialPort {
    is_connected: bool,
    config: SerialConfig,
    device_index: Option<i32>,
}

impl MobileSerialPort {
    pub fn new(config: SerialConfig) -> Self {
        Self {
            is_connected: false,
            config,
            device_index: None,
        }
    }

    /// List available USB serial devices
    pub fn list_ports() -> Result<Vec<SerialPortInfo>, String> {
        #[cfg(target_os = "android")]
        {
            info!("Android: listing USB serial devices via JNI");
            // In real implementation, would call Java UsbSerialPort.listDevices()
            // via JNI and parse the result into SerialPortInfo structs
            //
            // Example:
            // let devices = jni_call("listDevices")?;
            // for device in devices {
            //     result.push(SerialPortInfo {
            //         name: device.name,
            //         description: Some(device.description),
            //         manufacturer: Some(device.manufacturer),
            //         vid: Some(device.vid),
            //         pid: Some(device.pid),
            //     });
            // }
            Ok(Vec::new())
        }

        #[cfg(not(target_os = "android"))]
        {
            serialrun_core::SerialPort::list_ports()
                .map_err(|e| e.to_string())
        }
    }

    /// Connect to a serial device by index
    pub fn connect(&mut self, device_index: i32) -> Result<(), String> {
        #[cfg(target_os = "android")]
        {
            info!("Android: connecting to USB device index={}", device_index);
            // In real implementation, would call Java UsbSerialPort.open()
            // via JNI:
            // let result = jni_call("open", device_index, baud_rate)?;
            // if result { self.is_connected = true; }
            self.device_index = Some(device_index);
            self.is_connected = true;
            Ok(())
        }

        #[cfg(not(target_os = "android"))]
        {
            let _ = device_index;
            Err("Android only".into())
        }
    }

    /// Write data to serial port
    pub fn write(&mut self, data: &[u8]) -> Result<usize, String> {
        if !self.is_connected {
            return Err("Not connected".into());
        }

        #[cfg(target_os = "android")]
        {
            info!("Android: writing {} bytes", data.len());
            // In real implementation, would call Java UsbSerialPort.write()
            // via JNI with the byte array
            Ok(data.len())
        }

        #[cfg(not(target_os = "android"))]
        {
            Ok(data.len())
        }
    }

    /// Disconnect from serial port
    pub fn disconnect(&mut self) -> Result<(), String> {
        #[cfg(target_os = "android")]
        {
            info!("Android: closing USB device");
            // In real implementation, would call Java UsbSerialPort.close()
        }

        self.is_connected = false;
        self.device_index = None;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }
}

impl Drop for MobileSerialPort {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}
