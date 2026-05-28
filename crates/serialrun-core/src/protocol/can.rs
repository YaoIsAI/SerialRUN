/// CAN bus analyzer for parsing and analyzing CAN frames.

use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanError {
    #[error("Invalid frame format: {0}")]
    InvalidFrame(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type CanResult<T> = Result<T, CanError>;

/// A CAN bus frame.
#[derive(Debug, Clone)]
pub struct CanFrame {
    pub id: u32,
    pub is_extended: bool,
    pub is_rtr: bool,
    pub dlc: u8,
    pub data: Vec<u8>,
    pub timestamp: Option<Instant>,
    pub channel: u8,
}

impl CanFrame {
    /// Create a new standard (11-bit) CAN frame.
    pub fn new_standard(id: u32, data: Vec<u8>) -> Self {
        let dlc = data.len().min(8) as u8;
        let mut truncated_data = data;
        truncated_data.truncate(8);

        Self {
            id: id & 0x7FF,
            is_extended: false,
            is_rtr: false,
            dlc,
            data: truncated_data,
            timestamp: Some(Instant::now()),
            channel: 0,
        }
    }

    /// Create a new extended (29-bit) CAN frame.
    pub fn new_extended(id: u32, data: Vec<u8>) -> Self {
        let dlc = data.len().min(8) as u8;
        let mut truncated_data = data;
        truncated_data.truncate(8);

        Self {
            id: id & 0x1FFFFFFF,
            is_extended: true,
            is_rtr: false,
            dlc,
            data: truncated_data,
            timestamp: Some(Instant::now()),
            channel: 0,
        }
    }

    /// Create a new RTR (Remote Transmission Request) frame.
    pub fn new_rtr(id: u32, is_extended: bool, dlc: u8) -> Self {
        Self {
            id: if is_extended { id & 0x1FFFFFFF } else { id & 0x7FF },
            is_extended,
            is_rtr: true,
            dlc: dlc.min(8),
            data: Vec::new(),
            timestamp: Some(Instant::now()),
            channel: 0,
        }
    }

    /// Get the data bytes as a hex string.
    pub fn data_hex(&self) -> String {
        self.data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get the ID as a formatted string.
    pub fn id_str(&self) -> String {
        if self.is_extended {
            format!("0x{:08X}", self.id)
        } else {
            format!("0x{:03X}", self.id)
        }
    }
}

/// Filter for accepting/rejecting CAN frames.
#[derive(Debug, Clone)]
pub struct CanFilter {
    pub min_id: u32,
    pub max_id: u32,
    pub extended_only: bool,
    pub standard_only: bool,
    pub exclude_rtr: bool,
}

impl Default for CanFilter {
    fn default() -> Self {
        Self {
            min_id: 0,
            max_id: 0x1FFFFFFF,
            extended_only: false,
            standard_only: false,
            exclude_rtr: false,
        }
    }
}

impl CanFilter {
    /// Check if a frame passes this filter.
    pub fn accept(&self, frame: &CanFrame) -> bool {
        if frame.id < self.min_id || frame.id > self.max_id {
            return false;
        }

        if self.extended_only && !frame.is_extended {
            return false;
        }

        if self.standard_only && frame.is_extended {
            return false;
        }

        if self.exclude_rtr && frame.is_rtr {
            return false;
        }

        true
    }
}

/// Statistics for a specific CAN ID.
#[derive(Debug, Clone)]
pub struct CanIdStats {
    pub count: usize,
    pub last_data: Vec<u8>,
    pub last_seen: Option<Instant>,
    pub avg_interval_ms: f64,
}

impl Default for CanIdStats {
    fn default() -> Self {
        Self {
            count: 0,
            last_data: Vec::new(),
            last_seen: None,
            avg_interval_ms: 0.0,
        }
    }
}

/// CAN bus analyzer that collects frames and maintains statistics.
pub struct CanAnalyzer {
    frames: Vec<CanFrame>,
    stats: HashMap<u32, CanIdStats>,
    filter: CanFilter,
    last_timestamps: HashMap<u32, Instant>,
}

impl Default for CanAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CanAnalyzer {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            stats: HashMap::new(),
            filter: CanFilter::default(),
            last_timestamps: HashMap::new(),
        }
    }

    /// Set the filter for this analyzer.
    pub fn set_filter(&mut self, filter: CanFilter) {
        self.filter = filter;
    }

    /// Parse an SLCAN-format line (e.g., "T00108DEADBEEF01020304\r").
    /// Returns the parsed frame or an error.
    pub fn parse_slcan(&self, line: &str) -> CanResult<CanFrame> {
        let line = line.trim();

        if line.is_empty() {
            return Err(CanError::ParseError("Empty line".to_string()));
        }

        let first_char = line.chars().next().ok_or_else(|| {
            CanError::ParseError("Empty line".to_string())
        })?;

        match first_char {
            't' => {
                // Standard frame: t<id><dlc><data>
                if line.len() < 5 {
                    return Err(CanError::ParseError(
                        "Standard frame too short".to_string(),
                    ));
                }

                let id = u32::from_str_radix(&line[1..4], 16)
                    .map_err(|e| CanError::ParseError(format!("Invalid ID: {}", e)))?;

                let dlc = line[4..5]
                    .parse::<usize>()
                    .map_err(|e| CanError::ParseError(format!("Invalid DLC: {}", e)))?;

                let data_str = &line[5..];
                let mut data = Vec::new();
                for i in (0..data_str.len()).step_by(2) {
                    if i + 2 <= data_str.len() {
                        let byte = u8::from_str_radix(&data_str[i..i + 2], 16)
                            .map_err(|e| CanError::ParseError(format!("Invalid data byte: {}", e)))?;
                        data.push(byte);
                    }
                }

                data.truncate(dlc);

                Ok(CanFrame::new_standard(id, data))
            }
            'T' => {
                // Extended frame: T<id8><dlc><data>
                if line.len() < 10 {
                    return Err(CanError::ParseError(
                        "Extended frame too short".to_string(),
                    ));
                }

                let id = u32::from_str_radix(&line[1..9], 16)
                    .map_err(|e| CanError::ParseError(format!("Invalid ID: {}", e)))?;

                let dlc = line[9..10]
                    .parse::<usize>()
                    .map_err(|e| CanError::ParseError(format!("Invalid DLC: {}", e)))?;

                let data_str = &line[10..];
                let mut data = Vec::new();
                for i in (0..data_str.len()).step_by(2) {
                    if i + 2 <= data_str.len() {
                        let byte = u8::from_str_radix(&data_str[i..i + 2], 16)
                            .map_err(|e| CanError::ParseError(format!("Invalid data byte: {}", e)))?;
                        data.push(byte);
                    }
                }

                data.truncate(dlc);

                Ok(CanFrame::new_extended(id, data))
            }
            'r' => {
                // Standard RTR: r<id><dlc>
                if line.len() < 5 {
                    return Err(CanError::ParseError(
                        "RTR frame too short".to_string(),
                    ));
                }

                let id = u32::from_str_radix(&line[1..4], 16)
                    .map_err(|e| CanError::ParseError(format!("Invalid ID: {}", e)))?;

                let dlc = line[4..5]
                    .parse::<u8>()
                    .map_err(|e| CanError::ParseError(format!("Invalid DLC: {}", e)))?;

                Ok(CanFrame::new_rtr(id, false, dlc))
            }
            'R' => {
                // Extended RTR: R<id8><dlc>
                if line.len() < 10 {
                    return Err(CanError::ParseError(
                        "Extended RTR frame too short".to_string(),
                    ));
                }

                let id = u32::from_str_radix(&line[1..9], 16)
                    .map_err(|e| CanError::ParseError(format!("Invalid ID: {}", e)))?;

                let dlc = line[9..10]
                    .parse::<u8>()
                    .map_err(|e| CanError::ParseError(format!("Invalid DLC: {}", e)))?;

                Ok(CanFrame::new_rtr(id, true, dlc))
            }
            _ => Err(CanError::ParseError(format!(
                "Unknown frame type: {}",
                first_char
            ))),
        }
    }

    /// Parse a raw CAN frame from bytes (CAN ID + data).
    /// Format: [id_high, id_low, dlc, data...]
    pub fn parse_raw(&self, data: &[u8]) -> CanResult<CanFrame> {
        if data.len() < 3 {
            return Err(CanError::ParseError(
                "Raw frame too short".to_string(),
            ));
        }

        let id = ((data[0] as u32) << 8) | (data[1] as u32);
        let dlc = data[2].min(8);
        let frame_data = if data.len() > 3 {
            data[3..].to_vec()
        } else {
            Vec::new()
        };

        let mut frame = CanFrame::new_standard(id, frame_data);
        frame.dlc = dlc;
        Ok(frame)
    }

    /// Add a CAN frame to the analyzer and update statistics.
    pub fn add_frame(&mut self, frame: CanFrame) {
        // Update stats
        let stats = self.stats.entry(frame.id).or_default();
        stats.count += 1;
        stats.last_data = frame.data.clone();
        stats.last_seen = frame.timestamp;

        // Calculate average interval
        if let Some(last_ts) = self.last_timestamps.get(&frame.id) {
            if let Some(current_ts) = frame.timestamp {
                let interval = current_ts.duration_since(*last_ts).as_secs_f64() * 1000.0;
                if stats.count > 1 {
                    stats.avg_interval_ms =
                        (stats.avg_interval_ms * (stats.count as f64 - 1.0) + interval)
                            / stats.count as f64;
                } else {
                    stats.avg_interval_ms = interval;
                }
            }
        }

        if let Some(ts) = frame.timestamp {
            self.last_timestamps.insert(frame.id, ts);
        }

        self.frames.push(frame);
    }

    /// Get frames that pass the current filter.
    pub fn filtered_frames(&self) -> Vec<&CanFrame> {
        self.frames
            .iter()
            .filter(|f| self.filter.accept(f))
            .collect()
    }

    /// Get statistics for a specific CAN ID.
    pub fn get_stats(&self, id: u32) -> Option<&CanIdStats> {
        self.stats.get(&id)
    }

    /// Get all ID statistics.
    pub fn all_stats(&self) -> &HashMap<u32, CanIdStats> {
        &self.stats
    }

    /// Format a CAN frame for display.
    pub fn format_frame(frame: &CanFrame) -> String {
        let rtr = if frame.is_rtr { " RTR" } else { "" };
        let ext = if frame.is_extended { " EXT" } else { "" };

        format!(
            "CH{} {:>5}{}{} [{}] {}",
            frame.channel,
            frame.id_str(),
            rtr,
            ext,
            frame.dlc,
            frame.data_hex(),
        )
    }

    /// Clear all stored frames and statistics.
    pub fn clear(&mut self) {
        self.frames.clear();
        self.stats.clear();
        self.last_timestamps.clear();
    }

    /// Get total number of stored frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Get unique CAN IDs seen.
    pub fn unique_ids(&self) -> Vec<u32> {
        self.stats.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_frame_standard() {
        let frame = CanFrame::new_standard(0x123, vec![0x01, 0x02, 0x03]);
        assert_eq!(frame.id, 0x123);
        assert!(!frame.is_extended);
        assert!(!frame.is_rtr);
        assert_eq!(frame.dlc, 3);
        assert_eq!(frame.data, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_can_frame_extended() {
        let frame = CanFrame::new_extended(0x12345678, vec![0xAA, 0xBB]);
        assert!(frame.is_extended);
        assert_eq!(frame.id, 0x12345678);
        assert_eq!(frame.dlc, 2);
    }

    #[test]
    fn test_can_frame_truncate() {
        let frame = CanFrame::new_standard(0x100, vec![0; 10]);
        assert_eq!(frame.dlc, 8);
        assert_eq!(frame.data.len(), 8);
    }

    #[test]
    fn test_can_frame_rtr() {
        let frame = CanFrame::new_rtr(0x200, false, 4);
        assert!(frame.is_rtr);
        assert_eq!(frame.dlc, 4);
        assert!(frame.data.is_empty());
    }

    #[test]
    fn test_can_frame_hex() {
        let frame = CanFrame::new_standard(0x100, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(frame.data_hex(), "DE AD BE EF");
    }

    #[test]
    fn test_can_frame_id_str() {
        let std = CanFrame::new_standard(0x123, vec![0]);
        assert_eq!(std.id_str(), "0x123");

        let ext = CanFrame::new_extended(0x12345678, vec![0]);
        assert_eq!(ext.id_str(), "0x12345678");
    }

    #[test]
    fn test_can_filter_default() {
        let filter = CanFilter::default();
        let frame = CanFrame::new_standard(0x100, vec![1]);
        assert!(filter.accept(&frame));
    }

    #[test]
    fn test_can_filter_id_range() {
        let filter = CanFilter {
            min_id: 0x100,
            max_id: 0x200,
            ..Default::default()
        };

        let frame_low = CanFrame::new_standard(0x050, vec![1]);
        let frame_in = CanFrame::new_standard(0x150, vec![1]);
        let frame_high = CanFrame::new_standard(0x300, vec![1]);

        assert!(!filter.accept(&frame_low));
        assert!(filter.accept(&frame_in));
        assert!(!filter.accept(&frame_high));
    }

    #[test]
    fn test_can_filter_extended_only() {
        let filter = CanFilter {
            extended_only: true,
            ..Default::default()
        };

        let std = CanFrame::new_standard(0x100, vec![1]);
        let ext = CanFrame::new_extended(0x100, vec![1]);

        assert!(!filter.accept(&std));
        assert!(filter.accept(&ext));
    }

    #[test]
    fn test_can_filter_standard_only() {
        let filter = CanFilter {
            standard_only: true,
            ..Default::default()
        };

        let std = CanFrame::new_standard(0x100, vec![1]);
        let ext = CanFrame::new_extended(0x100, vec![1]);

        assert!(filter.accept(&std));
        assert!(!filter.accept(&ext));
    }

    #[test]
    fn test_can_filter_exclude_rtr() {
        let filter = CanFilter {
            exclude_rtr: true,
            ..Default::default()
        };

        let normal = CanFrame::new_standard(0x100, vec![1]);
        let rtr = CanFrame::new_rtr(0x100, false, 0);

        assert!(filter.accept(&normal));
        assert!(!filter.accept(&rtr));
    }

    #[test]
    fn test_parse_slcan_standard() {
        let analyzer = CanAnalyzer::new();
        let frame = analyzer.parse_slcan("t1232AABB").unwrap();
        assert_eq!(frame.id, 0x123);
        assert_eq!(frame.dlc, 2);
        assert_eq!(frame.data, vec![0xAA, 0xBB]);
    }

    #[test]
    fn test_parse_slcan_extended() {
        let analyzer = CanAnalyzer::new();
        let frame = analyzer.parse_slcan("T000001232AABBCCDD").unwrap();
        assert_eq!(frame.id, 0x123);
        assert_eq!(frame.dlc, 2);
        assert_eq!(frame.data, vec![0xAA, 0xBB]);
    }

    #[test]
    fn test_parse_slcan_rtr() {
        let analyzer = CanAnalyzer::new();
        let frame = analyzer.parse_slcan("r1234").unwrap();
        assert!(frame.is_rtr);
        assert_eq!(frame.id, 0x123);
        assert_eq!(frame.dlc, 4);
    }

    #[test]
    fn test_parse_slcan_empty() {
        let analyzer = CanAnalyzer::new();
        assert!(analyzer.parse_slcan("").is_err());
    }

    #[test]
    fn test_parse_raw() {
        let analyzer = CanAnalyzer::new();
        let frame = analyzer.parse_raw(&[0x01, 0x23, 0x03, 0xAA, 0xBB, 0xCC]).unwrap();
        assert_eq!(frame.id, 0x0123);
        assert_eq!(frame.dlc, 3);
        assert_eq!(frame.data, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_parse_raw_too_short() {
        let analyzer = CanAnalyzer::new();
        assert!(analyzer.parse_raw(&[0x01]).is_err());
    }

    #[test]
    fn test_add_frame_and_stats() {
        let mut analyzer = CanAnalyzer::new();
        let frame = CanFrame::new_standard(0x100, vec![1, 2, 3]);
        analyzer.add_frame(frame);

        assert_eq!(analyzer.frame_count(), 1);
        let stats = analyzer.get_stats(0x100).unwrap();
        assert_eq!(stats.count, 1);
        assert_eq!(stats.last_data, vec![1, 2, 3]);
    }

    #[test]
    fn test_filtered_frames() {
        let mut analyzer = CanAnalyzer::new();
        analyzer.set_filter(CanFilter {
            min_id: 0x100,
            max_id: 0x200,
            ..Default::default()
        });

        analyzer.add_frame(CanFrame::new_standard(0x050, vec![1]));
        analyzer.add_frame(CanFrame::new_standard(0x150, vec![2]));
        analyzer.add_frame(CanFrame::new_standard(0x300, vec![3]));

        let filtered = analyzer.filtered_frames();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 0x150);
    }

    #[test]
    fn test_format_frame() {
        let frame = CanFrame::new_standard(0x123, vec![0xDE, 0xAD]);
        let formatted = CanAnalyzer::format_frame(&frame);
        assert!(formatted.contains("0x123"));
        assert!(formatted.contains("DE AD"));
    }

    #[test]
    fn test_clear() {
        let mut analyzer = CanAnalyzer::new();
        analyzer.add_frame(CanFrame::new_standard(0x100, vec![1]));
        assert_eq!(analyzer.frame_count(), 1);

        analyzer.clear();
        assert_eq!(analyzer.frame_count(), 0);
        assert!(analyzer.unique_ids().is_empty());
    }

    #[test]
    fn test_unique_ids() {
        let mut analyzer = CanAnalyzer::new();
        analyzer.add_frame(CanFrame::new_standard(0x100, vec![1]));
        analyzer.add_frame(CanFrame::new_standard(0x200, vec![2]));
        analyzer.add_frame(CanFrame::new_standard(0x100, vec![3]));

        let mut ids = analyzer.unique_ids();
        ids.sort();
        assert_eq!(ids, vec![0x100, 0x200]);
    }
}
