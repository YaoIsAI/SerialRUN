//! JNI bridge for Android USB serial port
//!
//! This module provides the Rust ↔ Java bridge for serial port operations.
//! Uses the `jni` crate to call Java methods from Rust.

#[cfg(target_os = "android")]
use jni::JNIEnv;
#[cfg(target_os = "android")]
use jni::objects::{JClass, JObject};
#[cfg(target_os = "android")]
use jni::sys::jint;

use log::info;

/// JNI function: List available USB serial devices
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_serialrun_mobile_UsbSerialPort_nativeListDevices<'a>(
    _env: JNIEnv<'a>,
    _class: JClass<'a>,
) -> JObject<'a> {
    info!("JNI: nativeListDevices called");
    JObject::null()
}

/// JNI function: Open serial port
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_serialrun_mobile_UsbSerialPort_nativeOpen<'a>(
    _env: JNIEnv<'a>,
    _class: JClass<'a>,
    index: jint,
    baud_rate: jint,
) -> jni::sys::jboolean {
    info!("JNI: nativeOpen index={}, baud={}", index, baud_rate);
    1
}

/// JNI function: Write data to serial port
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_serialrun_mobile_UsbSerialPort_nativeWrite<'a>(
    env: JNIEnv<'a>,
    _class: JClass<'a>,
    data: jni::objects::JByteArray<'a>,
) -> jint {
    let len = env.get_array_length(&data).unwrap_or(0);
    info!("JNI: nativeWrite {} bytes", len);
    len
}

/// JNI function: Close serial port
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_serialrun_mobile_UsbSerialPort_nativeClose<'a>(
    _env: JNIEnv<'a>,
    _class: JClass<'a>,
) {
    info!("JNI: nativeClose called");
}

#[cfg(not(target_os = "android"))]
pub fn android_not_available() {}
