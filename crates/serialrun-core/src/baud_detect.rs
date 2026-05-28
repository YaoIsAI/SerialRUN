/// Baud rate auto-detection by trying common rates and scoring results.

use crate::config::{DataBits, Parity, StopBits};
use crate::port::PortResult;
use crate::SerialConfig;

/// Common baud rates to try during detection.
pub const COMMON_BAUD_RATES: &[u32] = &[
    300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
];

/// Result of a baud rate detection attempt.
#[derive(Debug, Clone)]
pub struct BaudDetectResult {
    pub baud_rate: u32,
    pub bytes_read: usize,
    pub confidence: f64,
}

/// Attempt to auto-detect the baud rate of a serial port.
///
/// Opens the port at each common baud rate, reads available bytes,
/// and scores each rate based on how many valid bytes were received.
pub fn detect_baud_rate(
    port_name: &str,
    data_bits: DataBits,
    parity: Parity,
    stop_bits: StopBits,
) -> PortResult<Vec<BaudDetectResult>> {
    let mut results = Vec::new();

    for &baud in COMMON_BAUD_RATES {
        let config = SerialConfig::new(port_name)
            .with_baud_rate(baud)
            .with_data_bits(data_bits.clone())
            .with_parity(parity.clone())
            .with_stop_bits(stop_bits.clone())
            .with_timeout(200);

        match score_baud_rate(&config) {
            Ok(result) => results.push(result),
            Err(_) => {
                results.push(BaudDetectResult {
                    baud_rate: baud,
                    bytes_read: 0,
                    confidence: 0.0,
                });
            }
        }
    }

    // Sort by confidence descending
    results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    Ok(results)
}

/// Score a single baud rate by opening the port and reading data.
fn score_baud_rate(config: &SerialConfig) -> PortResult<BaudDetectResult> {
    let mut port = crate::SerialPort::new(config.clone());
    port.connect()?;

    let mut buf = [0u8; 1024];
    let bytes_read = match port.read(&mut buf) {
        Ok(n) => n,
        Err(_) => 0,
    };

    let _ = port.disconnect();

    let confidence = compute_confidence(&buf[..bytes_read]);

    Ok(BaudDetectResult {
        baud_rate: config.baud_rate,
        bytes_read,
        confidence,
    })
}

/// Compute a confidence score (0.0 - 1.0) for the received data.
///
/// Higher scores indicate data that looks like valid serial communication:
/// printable ASCII, common control characters (CR, LF, TAB),
/// and reasonable byte distribution.
fn compute_confidence(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let total = data.len() as f64;
    let mut printable = 0u32;
    let mut control = 0u32;
    let mut null_count = 0u32;

    for &byte in data {
        match byte {
            0 => null_count += 1,
            0x01..=0x1F if byte == 0x0A || byte == 0x0D || byte == 0x09 => control += 1,
            0x20..=0x7E => printable += 1,
            _ => {}
        }
    }

    let printable_ratio = printable as f64 / total;
    let control_ratio = control as f64 / total;
    let null_ratio = null_count as f64 / total;

    // High printable ratio with some control chars is typical of text protocols
    let mut score = printable_ratio * 0.6 + control_ratio * 0.3;

    // Penalize too many nulls (likely garbage data from wrong baud rate)
    score -= null_ratio * 0.2;

    // Bonus for having some data at all
    if data.len() > 10 {
        score += 0.1;
    }

    score.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_baud_rates() {
        assert!(COMMON_BAUD_RATES.contains(&9600));
        assert!(COMMON_BAUD_RATES.contains(&115200));
        assert!(COMMON_BAUD_RATES.len() >= 10);
    }

    #[test]
    fn test_baud_detect_result_struct() {
        let result = BaudDetectResult {
            baud_rate: 115200,
            bytes_read: 50,
            confidence: 0.85,
        };
        assert_eq!(result.baud_rate, 115200);
        assert_eq!(result.bytes_read, 50);
        assert!((result.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_confidence_empty() {
        assert_eq!(compute_confidence(&[]), 0.0);
    }

    #[test]
    fn test_compute_confidence_printable() {
        let data = b"Hello, World! This is a test message.\r\n";
        let score = compute_confidence(data);
        assert!(score > 0.5);
    }

    #[test]
    fn test_compute_confidence_nulls() {
        let data = [0u8; 100];
        let score = compute_confidence(&data);
        assert!(score < 0.1);
    }

    #[test]
    fn test_compute_confidence_mixed() {
        let data = b"\r\nOK\r\n> ";
        let score = compute_confidence(data);
        assert!(score > 0.3);
    }
}
