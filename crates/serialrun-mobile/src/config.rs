//! Mobile-specific configuration

/// Mobile app configuration
#[derive(Debug, Clone)]
pub struct MobileConfig {
    /// USB vendor ID filter (None = show all)
    pub vendor_id_filter: Option<u16>,
    /// USB product ID filter (None = show all)
    pub product_id_filter: Option<u16>,
    /// Enable auto-reconnect on USB disconnect
    pub auto_reconnect: bool,
    /// Read timeout in milliseconds
    pub read_timeout_ms: u32,
}

impl Default for MobileConfig {
    fn default() -> Self {
        Self {
            vendor_id_filter: None,
            product_id_filter: None,
            auto_reconnect: true,
            read_timeout_ms: 1000,
        }
    }
}
