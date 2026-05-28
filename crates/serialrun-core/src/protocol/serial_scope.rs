/// Serial oscilloscope for visualizing serial data as waveforms.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScopeError {
    #[error("Scope error: {0}")]
    ScopeError(String),
}

pub type ScopeResult<T> = Result<T, ScopeError>;

/// A single sample point in the waveform.
#[derive(Debug, Clone)]
pub struct SamplePoint {
    /// Time offset in microseconds from the start of capture.
    pub time_us: f64,
    /// Logic level (true = high, false = low).
    pub level: bool,
}

/// Detected bit period information.
#[derive(Debug, Clone)]
pub struct BitPeriod {
    /// Period in microseconds.
    pub period_us: f64,
    /// Confidence (0.0 - 1.0).
    pub confidence: f64,
}

/// A captured byte with timing information.
#[derive(Debug, Clone)]
pub struct CapturedByte {
    /// The decoded byte value.
    pub value: u8,
    /// Start time in microseconds.
    pub start_us: f64,
    /// End time in microseconds.
    pub end_us: f64,
    /// Bit period used for decoding.
    pub bit_period_us: f64,
}

/// Configuration for the serial scope.
#[derive(Debug, Clone)]
pub struct ScopeConfig {
    /// Expected baud rate (used to calculate bit period).
    pub baud_rate: u32,
    /// Number of data bits (5-8).
    pub data_bits: u8,
    /// Number of stop bits (1 or 2).
    pub stop_bits: f32,
    /// Whether parity is enabled.
    pub parity: bool,
    /// Sample rate multiplier (1 = one sample per bit).
    pub sample_multiplier: u32,
}

impl Default for ScopeConfig {
    fn default() -> Self {
        Self {
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: 1.0,
            parity: false,
            sample_multiplier: 1,
        }
    }
}

/// Scope statistics for the captured data.
#[derive(Debug, Clone, Default)]
pub struct ScopeStats {
    /// Total number of bytes captured.
    pub total_bytes: usize,
    /// Average bit period in microseconds.
    pub avg_bit_period_us: f64,
    /// Minimum bit period in microseconds.
    pub min_bit_period_us: f64,
    /// Maximum bit period in microseconds.
    pub max_bit_period_us: f64,
    /// Number of framing errors detected.
    pub framing_errors: usize,
    /// Total capture duration in microseconds.
    pub total_duration_us: f64,
}

/// Serial oscilloscope for analyzing serial data waveforms.
pub struct SerialScope {
    config: ScopeConfig,
    waveform: Vec<SamplePoint>,
    captured_bytes: Vec<CapturedByte>,
    bit_periods: Vec<BitPeriod>,
}

impl Default for SerialScope {
    fn default() -> Self {
        Self::new()
    }
}

impl SerialScope {
    pub fn new() -> Self {
        Self {
            config: ScopeConfig::default(),
            waveform: Vec::new(),
            captured_bytes: Vec::new(),
            bit_periods: Vec::new(),
        }
    }

    /// Create a scope with custom configuration.
    pub fn with_config(config: ScopeConfig) -> Self {
        Self {
            config,
            waveform: Vec::new(),
            captured_bytes: Vec::new(),
            bit_periods: Vec::new(),
        }
    }

    /// Calculate the bit period in microseconds from the baud rate.
    pub fn bit_period_us(&self) -> f64 {
        1_000_000.0 / self.config.baud_rate as f64
    }

    /// Process raw serial data and convert to waveform sample points.
    ///
    /// Each byte is decoded into start bit + data bits + parity + stop bits.
    pub fn process_raw_data(&mut self, data: &[u8], start_time_us: f64) {
        let bit_period = self.bit_period_us();
        let mut current_time = start_time_us;

        for &byte in data {
            let byte_start = current_time;

            // Start bit (low)
            self.waveform.push(SamplePoint {
                time_us: current_time,
                level: false,
            });
            current_time += bit_period;

            // Data bits (LSB first)
            for i in 0..self.config.data_bits {
                let level = (byte >> i) & 1 == 1;
                self.waveform.push(SamplePoint {
                    time_us: current_time,
                    level,
                });
                current_time += bit_period;
            }

            // Parity bit (if enabled)
            if self.config.parity {
                let ones = byte.count_ones() as u8;
                let parity_bit = ones % 2 == 1; // Odd parity
                self.waveform.push(SamplePoint {
                    time_us: current_time,
                    level: parity_bit,
                });
                current_time += bit_period;
            }

            // Stop bits (high)
            let stop_bits = self.config.stop_bits as u32;
            for _ in 0..stop_bits {
                self.waveform.push(SamplePoint {
                    time_us: current_time,
                    level: true,
                });
                current_time += bit_period;
            }

            let byte_end = current_time;

            self.captured_bytes.push(CapturedByte {
                value: byte,
                start_us: byte_start,
                end_us: byte_end,
                bit_period_us: bit_period,
            });
        }
    }

    /// Convert the internal waveform to a simple list of (time, level) points.
    pub fn to_waveform_points(&self) -> Vec<(f64, bool)> {
        self.waveform
            .iter()
            .map(|p| (p.time_us, p.level))
            .collect()
    }

    /// Get statistics about the captured data.
    pub fn stats(&self) -> ScopeStats {
        if self.captured_bytes.is_empty() {
            return ScopeStats::default();
        }

        let total_bytes = self.captured_bytes.len();
        let durations: Vec<f64> = self
            .captured_bytes
            .iter()
            .map(|b| b.end_us - b.start_us)
            .collect();

        let avg_duration = durations.iter().sum::<f64>() / durations.len() as f64;
        let min_duration = durations.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_duration = durations.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let total_duration_us = if let Some(last) = self.captured_bytes.last() {
            last.end_us
        } else {
            0.0
        };

        // Calculate average bit period from all captured bytes
        let avg_bit_period = self
            .captured_bytes
            .iter()
            .map(|b| b.bit_period_us)
            .sum::<f64>()
            / total_bytes as f64;

        let _ = avg_duration;
        let _ = min_duration;
        let _ = max_duration;

        ScopeStats {
            total_bytes,
            avg_bit_period_us: avg_bit_period,
            min_bit_period_us: min_duration,
            max_bit_period_us: max_duration,
            framing_errors: 0,
            total_duration_us,
        }
    }

    /// Clear all captured data.
    pub fn clear(&mut self) {
        self.waveform.clear();
        self.captured_bytes.clear();
        self.bit_periods.clear();
    }

    /// Get the current configuration.
    pub fn config(&self) -> &ScopeConfig {
        &self.config
    }

    /// Get the number of captured bytes.
    pub fn byte_count(&self) -> usize {
        self.captured_bytes.len()
    }

    /// Get the number of waveform sample points.
    pub fn sample_count(&self) -> usize {
        self.waveform.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_config_default() {
        let config = ScopeConfig::default();
        assert_eq!(config.baud_rate, 115200);
        assert_eq!(config.data_bits, 8);
        assert!((config.stop_bits - 1.0f32).abs() < f32::EPSILON);
        assert!(!config.parity);
    }

    #[test]
    fn test_serial_scope_new() {
        let scope = SerialScope::new();
        assert_eq!(scope.byte_count(), 0);
        assert_eq!(scope.sample_count(), 0);
    }

    #[test]
    fn test_bit_period_calculation() {
        let scope = SerialScope::new();
        // 115200 baud => ~8.68 us per bit
        let period = scope.bit_period_us();
        assert!((period - 8.68).abs() < 0.1);

        let scope2 = SerialScope::with_config(ScopeConfig {
            baud_rate: 9600,
            ..Default::default()
        });
        let period2 = scope2.bit_period_us();
        assert!((period2 - 104.17).abs() < 0.1);
    }

    #[test]
    fn test_process_raw_data_0x55() {
        let mut scope = SerialScope::new();
        // 0x55 = 01010101 in binary, LSB first = 1,0,1,0,1,0,1,0
        scope.process_raw_data(&[0x55], 0.0);

        assert_eq!(scope.byte_count(), 1);
        assert!(scope.sample_count() > 0);

        let points = scope.to_waveform_points();
        // Start bit (low), then 8 data bits, then stop bit (high)
        assert!(!points[0].1); // Start bit is low
        assert!(points[9].1); // Stop bit is high (index 1 = start, 2-9 = data bits, 10 = stop)
    }

    #[test]
    fn test_process_raw_data_multi_byte() {
        let mut scope = SerialScope::new();
        scope.process_raw_data(&[0xAA, 0x55], 0.0);
        assert_eq!(scope.byte_count(), 2);
    }

    #[test]
    fn test_process_raw_data_with_parity() {
        let config = ScopeConfig {
            parity: true,
            ..Default::default()
        };
        let mut scope = SerialScope::with_config(config);
        scope.process_raw_data(&[0x55], 0.0);
        assert_eq!(scope.byte_count(), 1);
    }

    #[test]
    fn test_to_waveform_points() {
        let mut scope = SerialScope::new();
        scope.process_raw_data(&[0x00], 0.0); // All zeros, LSB first = 0,0,0,0,0,0,0,0
        let points = scope.to_waveform_points();
        // Start bit (low) + 8 data bits (all low) + stop bit (high) = 10 points
        assert_eq!(points.len(), 10);
        // All data bits should be low
        for i in 1..=8 {
            assert!(!points[i].1);
        }
        // Stop bit should be high
        assert!(points[9].1);
    }

    #[test]
    fn test_stats() {
        let mut scope = SerialScope::new();
        scope.process_raw_data(&[0x01, 0x02, 0x03], 0.0);

        let stats = scope.stats();
        assert_eq!(stats.total_bytes, 3);
        assert!(stats.avg_bit_period_us > 0.0);
        assert!(stats.total_duration_us > 0.0);
    }

    #[test]
    fn test_stats_empty() {
        let scope = SerialScope::new();
        let stats = scope.stats();
        assert_eq!(stats.total_bytes, 0);
    }

    #[test]
    fn test_clear() {
        let mut scope = SerialScope::new();
        scope.process_raw_data(&[0x01], 0.0);
        assert_eq!(scope.byte_count(), 1);

        scope.clear();
        assert_eq!(scope.byte_count(), 0);
        assert_eq!(scope.sample_count(), 0);
    }

    #[test]
    fn test_config_access() {
        let config = ScopeConfig {
            baud_rate: 9600,
            data_bits: 7,
            stop_bits: 2.0,
            parity: true,
            sample_multiplier: 4,
        };
        let scope = SerialScope::with_config(config);
        assert_eq!(scope.config().baud_rate, 9600);
        assert_eq!(scope.config().data_bits, 7);
    }
}
