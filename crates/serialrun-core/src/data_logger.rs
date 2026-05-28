/// CSV data logger for recording serial data with timestamps and scaling.

use chrono::Utc;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoggerError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Logger not started")]
    NotStarted,
}

pub type LoggerResult<T> = Result<T, LoggerError>;

/// A single log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub source: String,
    pub raw_value: String,
    pub scaled_value: String,
    pub unit: String,
}

/// CSV data logger with buffered writes and auto-flush.
pub struct DataLogger {
    path: PathBuf,
    writer: Option<BufWriter<File>>,
    buffer: Vec<LogEntry>,
    auto_flush_threshold: usize,
    is_started: bool,
}

impl Default for DataLogger {
    fn default() -> Self {
        Self {
            path: PathBuf::from("data_log.csv"),
            writer: None,
            buffer: Vec::new(),
            auto_flush_threshold: 100,
            is_started: false,
        }
    }
}

impl Drop for DataLogger {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl DataLogger {
    /// Create a new logger that will write to the given path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            writer: None,
            buffer: Vec::new(),
            auto_flush_threshold: 100,
            is_started: false,
        }
    }

    /// Set the auto-flush threshold (number of entries before auto-flush).
    pub fn with_auto_flush_threshold(mut self, threshold: usize) -> Self {
        self.auto_flush_threshold = threshold;
        self
    }

    /// Start the logger, creating the CSV file and writing the header.
    pub fn start(&mut self) -> LoggerResult<()> {
        if self.is_started {
            return Ok(());
        }

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let is_new = file.metadata()?.len() == 0;
        let mut writer = BufWriter::new(file);

        // Write CSV header if file is empty
        if is_new {
            writeln!(writer, "timestamp,source,raw_value,scaled_value,unit")?;
        }

        self.writer = Some(writer);
        self.is_started = true;
        Ok(())
    }

    /// Log a single entry.
    pub fn log(&mut self, entry: LogEntry) -> LoggerResult<()> {
        if !self.is_started {
            return Err(LoggerError::NotStarted);
        }

        self.buffer.push(entry);

        if self.buffer.len() >= self.auto_flush_threshold {
            self.flush()?;
        }

        Ok(())
    }

    /// Log a simple value pair (raw and scaled).
    pub fn log_value(
        &mut self,
        source: &str,
        raw: &str,
        scaled: &str,
        unit: &str,
    ) -> LoggerResult<()> {
        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            source: source.to_string(),
            raw_value: raw.to_string(),
            scaled_value: scaled.to_string(),
            unit: unit.to_string(),
        };
        self.log(entry)
    }

    /// Flush all buffered entries to disk.
    pub fn flush(&mut self) -> LoggerResult<()> {
        if let Some(writer) = self.writer.as_mut() {
            for entry in self.buffer.drain(..) {
                writeln!(
                    writer,
                    "{},{},{},{},{}",
                    entry.timestamp,
                    entry.source,
                    entry.raw_value,
                    entry.scaled_value,
                    entry.unit,
                )?;
            }
            writer.flush()?;
        }
        Ok(())
    }

    /// Stop the logger and flush remaining data.
    pub fn stop(&mut self) -> LoggerResult<()> {
        if !self.is_started {
            return Ok(());
        }

        self.flush()?;
        self.writer = None;
        self.is_started = false;
        Ok(())
    }

    /// Check if the logger is currently started.
    pub fn is_started(&self) -> bool {
        self.is_started
    }

    /// Get the number of buffered (unflushed) entries.
    pub fn buffered_count(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        dir.join(format!("serialrun_test_{}_{}.csv", name, std::process::id()))
    }

    #[test]
    fn test_default() {
        let logger = DataLogger::default();
        assert!(!logger.is_started());
        assert_eq!(logger.buffered_count(), 0);
    }

    #[test]
    fn test_start_and_stop() {
        let path = temp_path("start_stop");
        let mut logger = DataLogger::new(&path);
        logger.start().unwrap();
        assert!(logger.is_started());
        logger.stop().unwrap();
        assert!(!logger.is_started());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_log_and_flush() {
        let path = temp_path("log_flush");
        let mut logger = DataLogger::new(&path);
        logger.start().unwrap();

        logger
            .log_value("test", "100", "25.5", "C")
            .unwrap();
        logger.flush().unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("timestamp,source,raw_value,scaled_value,unit"));
        assert!(content.contains("100,25.5,C"));

        logger.stop().unwrap();
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_auto_flush() {
        let path = temp_path("auto_flush");
        let mut logger = DataLogger::new(&path)
            .with_auto_flush_threshold(3);

        logger.start().unwrap();

        // First two entries should be buffered
        logger.log_value("s", "1", "1.0", "V").unwrap();
        logger.log_value("s", "2", "2.0", "V").unwrap();
        assert_eq!(logger.buffered_count(), 2);

        // Third entry should trigger auto-flush
        logger.log_value("s", "3", "3.0", "V").unwrap();
        assert_eq!(logger.buffered_count(), 0);

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("1.0"));
        assert!(content.contains("3.0"));

        logger.stop().unwrap();
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_drop_flushes() {
        let path = temp_path("drop_flush");
        {
            let mut logger = DataLogger::new(&path);
            logger.start().unwrap();
            logger.log_value("s", "x", "y", "z").unwrap();
            // logger dropped here
        }

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("y"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_log_when_not_started() {
        let path = temp_path("not_started");
        let mut logger = DataLogger::new(&path);
        let result = logger.log_value("s", "1", "1", "V");
        assert!(result.is_err());
    }
}
