//! SerialRUN Mobile — Android USB serial port adapter
//!
//! This crate provides the mobile-specific serial port implementation
//! for Android, using JNI to bridge to Android's USB Host API.

pub mod port;
pub mod config;
#[cfg(target_os = "android")]
pub mod jni_bridge;

pub use port::MobileSerialPort;
pub use config::MobileConfig;
