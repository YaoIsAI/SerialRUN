/// CANalyst-II USB CAN analyzer driver.
///
/// Loads ControlCAN.dll (Windows) or libcontrolcan.so (Linux) at runtime
/// and provides a safe Rust wrapper around the VCI_* C API.

use std::sync::Arc;

#[cfg(target_os = "windows")]
const LIB_NAME: &str = "ControlCAN.dll";
#[cfg(target_os = "linux")]
const LIB_NAME: &str = "libcontrolcan.so";

/// Device type for CANalyst-II / USBCAN-2A / USBCAN-2C
pub const VCI_USBCAN2: u32 = 4;

// ── C-compatible structs ──────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Debug)]
pub struct VciBoardInfo {
    pub hw_version: u16,
    pub fw_version: u16,
    pub dr_version: u16,
    pub in_version: u16,
    pub irq_num: u16,
    pub can_num: u8,
    pub str_serial_num: [u8; 20],
    pub str_hw_type: [u8; 40],
    pub reserved: [u16; 4],
}

impl Default for VciBoardInfo {
    fn default() -> Self {
        Self {
            hw_version: 0, fw_version: 0, dr_version: 0, in_version: 0,
            irq_num: 0, can_num: 0,
            str_serial_num: [0u8; 20], str_hw_type: [0u8; 40],
            reserved: [0u16; 4],
        }
    }
}

impl VciBoardInfo {
    pub fn serial_number(&self) -> String {
        let end = self.str_serial_num.iter().position(|&b| b == 0).unwrap_or(20);
        String::from_utf8_lossy(&self.str_serial_num[..end]).to_string()
    }
    pub fn hw_type(&self) -> String {
        let end = self.str_hw_type.iter().position(|&b| b == 0).unwrap_or(40);
        String::from_utf8_lossy(&self.str_hw_type[..end]).to_string()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VciCanObj {
    pub id: u32,
    pub timestamp: u32,
    pub time_flag: u8,
    pub send_type: u8,
    pub remote_flag: u8,
    pub extern_flag: u8,
    pub data_len: u8,
    pub data: [u8; 8],
    pub reserved: [u8; 3],
}

impl Default for VciCanObj {
    fn default() -> Self {
        Self {
            id: 0, timestamp: 0, time_flag: 0, send_type: 0,
            remote_flag: 0, extern_flag: 0, data_len: 0,
            data: [0u8; 8], reserved: [0u8; 3],
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct VciInitConfig {
    pub acc_code: u32,
    pub acc_mask: u32,
    pub reserved: u32,
    pub filter: u8,
    pub timing0: u8,
    pub timing1: u8,
    pub mode: u8,
}

// ── FFI function type aliases ─────────────────────────────────────────

type FnOpenDevice     = unsafe extern "C" fn(u32, u32, u32) -> u32;
type FnCloseDevice    = unsafe extern "C" fn(u32, u32) -> u32;
type FnInitCan        = unsafe extern "C" fn(u32, u32, u32, *const VciInitConfig) -> u32;
type FnReadBoardInfo  = unsafe extern "C" fn(u32, u32, *mut VciBoardInfo) -> u32;
type FnStartCan       = unsafe extern "C" fn(u32, u32, u32) -> u32;
type FnResetCan       = unsafe extern "C" fn(u32, u32, u32) -> u32;
type FnTransmit       = unsafe extern "C" fn(u32, u32, u32, *const VciCanObj, u32) -> u32;
type FnReceive        = unsafe extern "C" fn(u32, u32, u32, *mut VciCanObj, u32, i32) -> u32;
type FnClearBuffer    = unsafe extern "C" fn(u32, u32, u32) -> u32;
type FnGetReceiveNum  = unsafe extern "C" fn(u32, u32, u32) -> u32;
type FnFindUsbDevice2 = unsafe extern "C" fn(*mut VciBoardInfo) -> u32;
type FnUsbDeviceReset = unsafe extern "C" fn(u32, u32, u32) -> u32;

// ── Driver wrapper ────────────────────────────────────────────────────

pub struct CanalystDriver {
    _library: Option<libloading::Library>,
    fn_open_device: FnOpenDevice,
    fn_close_device: FnCloseDevice,
    fn_init_can: FnInitCan,
    fn_read_board_info: FnReadBoardInfo,
    fn_start_can: FnStartCan,
    fn_reset_can: FnResetCan,
    fn_transmit: FnTransmit,
    fn_receive: FnReceive,
    fn_clear_buffer: FnClearBuffer,
    fn_get_receive_num: FnGetReceiveNum,
    fn_find_usb_device2: FnFindUsbDevice2,
    fn_usb_device_reset: FnUsbDeviceReset,
}

// Safety: All VCI functions are thread-safe per the SDK documentation.
// The DLL handles internal synchronization.
unsafe impl Send for CanalystDriver {}
unsafe impl Sync for CanalystDriver {}

impl CanalystDriver {
    /// Load the ControlCAN library and resolve all symbols.
    pub fn new() -> Result<Self, String> {
        // Search order: exe directory, then system PATH
        let library = unsafe { libloading::Library::new(LIB_NAME) }
            .map_err(|e| format!("Failed to load {}: {}", LIB_NAME, e))?;

        unsafe fn get_sym<T: Copy>(
            lib: &libloading::Library, name: &[u8],
        ) -> Result<T, String> {
            lib.get(name)
                .map(|s: libloading::Symbol<T>| *s)
                .map_err(|e| format!("Symbol {:?} not found: {}", String::from_utf8_lossy(name), e))
        }

        unsafe {
            Ok(Self {
                fn_open_device:     get_sym(&library, b"VCI_OpenDevice")?,
                fn_close_device:    get_sym(&library, b"VCI_CloseDevice")?,
                fn_init_can:        get_sym(&library, b"VCI_InitCAN")?,
                fn_read_board_info: get_sym(&library, b"VCI_ReadBoardInfo")?,
                fn_start_can:       get_sym(&library, b"VCI_StartCAN")?,
                fn_reset_can:       get_sym(&library, b"VCI_ResetCAN")?,
                fn_transmit:        get_sym(&library, b"VCI_Transmit")?,
                fn_receive:         get_sym(&library, b"VCI_Receive")?,
                fn_clear_buffer:    get_sym(&library, b"VCI_ClearBuffer")?,
                fn_get_receive_num: get_sym(&library, b"VCI_GetReceiveNum")?,
                fn_find_usb_device2: get_sym(&library, b"VCI_FindUsbDevice2")?,
                fn_usb_device_reset: get_sym(&library, b"VCI_UsbDeviceReset")?,
                _library: Some(library),
            })
        }
    }

    /// Check if the library can be loaded (non-consuming probe).
    pub fn is_available() -> bool {
        unsafe { libloading::Library::new(LIB_NAME) }.is_ok()
    }

    // ── Device management ─────────────────────────────────────────────

    pub fn open_device(&self, dev_index: u32) -> Result<(), String> {
        let r = unsafe { (self.fn_open_device)(VCI_USBCAN2, dev_index, 0) };
        if r == 1 { Ok(()) } else { Err(format!("VCI_OpenDevice failed: {}", r)) }
    }

    pub fn close_device(&self, dev_index: u32) -> Result<(), String> {
        let r = unsafe { (self.fn_close_device)(VCI_USBCAN2, dev_index) };
        if r == 1 { Ok(()) } else { Err(format!("VCI_CloseDevice failed: {}", r)) }
    }

    pub fn read_board_info(&self, dev_index: u32) -> Result<VciBoardInfo, String> {
        let mut info = VciBoardInfo::default();
        let r = unsafe { (self.fn_read_board_info)(VCI_USBCAN2, dev_index, &mut info) };
        if r == 1 { Ok(info) } else { Err(format!("VCI_ReadBoardInfo failed: {}", r)) }
    }

    /// Find all connected USB-CAN devices. Returns board info for each.
    pub fn find_devices(&self) -> Vec<VciBoardInfo> {
        let mut infos = vec![VciBoardInfo::default(); 50];
        let count = unsafe { (self.fn_find_usb_device2)(infos.as_mut_ptr()) };
        infos.truncate(count as usize);
        infos
    }

    pub fn usb_reset(&self, dev_index: u32) -> Result<(), String> {
        let r = unsafe { (self.fn_usb_device_reset)(VCI_USBCAN2, dev_index, 0) };
        if r == 1 { Ok(()) } else { Err(format!("VCI_UsbDeviceReset failed: {}", r)) }
    }

    // ── CAN channel management ────────────────────────────────────────

    pub fn init_can(&self, dev_index: u32, can_index: u32, baud_rate: u32) -> Result<(), String> {
        let (t0, t1) = baud_to_timing(baud_rate)
            .ok_or_else(|| format!("Unsupported baud rate: {}", baud_rate))?;
        let config = VciInitConfig {
            acc_code: 0,
            acc_mask: 0xFFFFFFFF,
            reserved: 0,
            filter: 1, // receive all
            timing0: t0,
            timing1: t1,
            mode: 0, // normal
        };
        let r = unsafe { (self.fn_init_can)(VCI_USBCAN2, dev_index, can_index, &config) };
        if r == 1 { Ok(()) } else { Err(format!("VCI_InitCAN failed: {}", r)) }
    }

    pub fn init_can_with_mode(&self, dev_index: u32, can_index: u32, baud_rate: u32, mode: u8) -> Result<(), String> {
        let (t0, t1) = baud_to_timing(baud_rate)
            .ok_or_else(|| format!("Unsupported baud rate: {}", baud_rate))?;
        let config = VciInitConfig {
            acc_code: 0,
            acc_mask: 0xFFFFFFFF,
            reserved: 0,
            filter: 1,
            timing0: t0,
            timing1: t1,
            mode,
        };
        let r = unsafe { (self.fn_init_can)(VCI_USBCAN2, dev_index, can_index, &config) };
        if r == 1 { Ok(()) } else { Err(format!("VCI_InitCAN failed: {}", r)) }
    }

    pub fn start_can(&self, dev_index: u32, can_index: u32) -> Result<(), String> {
        let r = unsafe { (self.fn_start_can)(VCI_USBCAN2, dev_index, can_index) };
        if r == 1 { Ok(()) } else { Err(format!("VCI_StartCAN failed: {}", r)) }
    }

    pub fn reset_can(&self, dev_index: u32, can_index: u32) -> Result<(), String> {
        let r = unsafe { (self.fn_reset_can)(VCI_USBCAN2, dev_index, can_index) };
        if r == 1 { Ok(()) } else { Err(format!("VCI_ResetCAN failed: {}", r)) }
    }

    pub fn clear_buffer(&self, dev_index: u32, can_index: u32) -> Result<(), String> {
        let r = unsafe { (self.fn_clear_buffer)(VCI_USBCAN2, dev_index, can_index) };
        if r == 1 { Ok(()) } else { Err(format!("VCI_ClearBuffer failed: {}", r)) }
    }

    pub fn get_receive_num(&self, dev_index: u32, can_index: u32) -> u32 {
        unsafe { (self.fn_get_receive_num)(VCI_USBCAN2, dev_index, can_index) }
    }

    // ── Transmit / Receive ────────────────────────────────────────────

    /// Transmit CAN frames. Returns the number of frames actually sent.
    pub fn transmit(&self, dev_index: u32, can_index: u32, frames: &[VciCanObj]) -> Result<u32, String> {
        if frames.is_empty() { return Ok(0); }
        let r = unsafe {
            (self.fn_transmit)(VCI_USBCAN2, dev_index, can_index, frames.as_ptr(), frames.len() as u32)
        };
        if r == 0xFFFFFFFF {
            Err("VCI_Transmit: device not connected".into())
        } else {
            Ok(r)
        }
    }

    /// Receive CAN frames (non-blocking). Returns parsed frames.
    pub fn receive(&self, dev_index: u32, can_index: u32, max_frames: u32) -> Result<Vec<VciCanObj>, String> {
        let mut buf = vec![VciCanObj::default(); max_frames as usize];
        let r = unsafe {
            (self.fn_receive)(VCI_USBCAN2, dev_index, can_index, buf.as_mut_ptr(), max_frames, 0)
        };
        if r == 0xFFFFFFFF {
            Err("VCI_Receive: device not connected".into())
        } else {
            buf.truncate(r as usize);
            Ok(buf)
        }
    }

    /// Disconnect: reset channel + close device. Best-effort cleanup.
    pub fn disconnect(&self, dev_index: u32, can_index: u32) {
        let _ = self.reset_can(dev_index, can_index);
        let _ = self.close_device(dev_index);
    }
}

// ── Baud rate lookup ──────────────────────────────────────────────────

/// Convert baud rate to SJA1000 Timing0/Timing1 register values.
/// Returns None for unsupported rates.
pub fn baud_to_timing(baud_rate: u32) -> Option<(u8, u8)> {
    match baud_rate {
        10_000   => Some((0x31, 0x1C)),
        20_000   => Some((0x18, 0x1C)),
        33_330   => Some((0x09, 0x6F)),
        40_000   => Some((0x87, 0xFF)),
        50_000   => Some((0x09, 0x1C)),
        66_660   => Some((0x04, 0x6F)),
        80_000   => Some((0x83, 0xFF)),
        83_330   => Some((0x03, 0x6F)),
        100_000  => Some((0x04, 0x1C)),
        125_000  => Some((0x03, 0x1C)),
        200_000  => Some((0x81, 0xFA)),
        250_000  => Some((0x01, 0x1C)),
        400_000  => Some((0x80, 0xFA)),
        500_000  => Some((0x00, 0x1C)),
        666_000  => Some((0x80, 0xB6)),
        800_000  => Some((0x00, 0x16)),
        1000_000 => Some((0x00, 0x14)),
        _ => None,
    }
}

/// All supported baud rates for UI display.
pub const SUPPORTED_BAUD_RATES: &[u32] = &[
    10_000, 20_000, 33_330, 40_000, 50_000, 66_660, 80_000, 83_330,
    100_000, 125_000, 200_000, 250_000, 400_000, 500_000, 666_000,
    800_000, 1000_000,
];

// ── Shared driver singleton ───────────────────────────────────────────

use std::sync::Mutex;

static DRIVER: Mutex<Option<Arc<CanalystDriver>>> = Mutex::new(None);

/// Get or initialize the shared CanalystDriver instance.
/// Returns None if the DLL cannot be loaded.
pub fn get_driver() -> Option<Arc<CanalystDriver>> {
    let mut guard = DRIVER.lock().unwrap();
    if guard.is_none() {
        match CanalystDriver::new() {
            Ok(d) => { *guard = Some(Arc::new(d)); }
            Err(e) => { log::warn!("CANalyst-II DLL not available: {}", e); }
        }
    }
    guard.clone()
}

/// Check if CANalyst-II support is available on this platform.
#[cfg(any(target_os = "windows", target_os = "linux"))]
pub fn is_supported() -> bool { true }

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn is_supported() -> bool { false }
