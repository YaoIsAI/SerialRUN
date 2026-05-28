use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransferError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Transfer failed: {0}")]
    TransferFailed(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("Checksum mismatch")]
    ChecksumMismatch,
}

pub type TransferResult<T> = Result<T, TransferError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransferProtocol {
    Xmodem,
    XmodemCrc,
    Ymodem,
    Zmodem,
}

#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed_bps: f64,
    pub elapsed: Duration,
    pub eta: Option<Duration>,
}

impl TransferProgress {
    pub fn percentage(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.bytes_transferred as f32 / self.total_bytes as f32) * 100.0
    }
}

pub struct FileTransfer {
    protocol: TransferProtocol,
    chunk_size: usize,
    timeout: Duration,
}

impl FileTransfer {
    pub fn new(protocol: TransferProtocol) -> Self {
        let chunk_size = match protocol {
            TransferProtocol::Xmodem | TransferProtocol::XmodemCrc => 128,
            TransferProtocol::Ymodem => 1024,
            TransferProtocol::Zmodem => 1024,
        };

        Self {
            protocol,
            chunk_size,
            timeout: Duration::from_secs(10),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn send_file<F>(
        &self,
        file_path: &Path,
        mut write_fn: F,
        mut read_fn: impl FnMut() -> TransferResult<u8>,
        mut progress_fn: impl FnMut(TransferProgress),
    ) -> TransferResult<()>
    where
        F: FnMut(&[u8]) -> TransferResult<()>,
    {
        let mut file = File::open(file_path)?;
        let file_size = file.metadata()?.len();

        let mut buffer = vec![0u8; self.chunk_size];
        let mut bytes_sent = 0u64;
        let start_time = std::time::Instant::now();

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];

            match self.protocol {
                TransferProtocol::Xmodem => {
                    self.send_xmodem_chunk(chunk, bytes_sent as u16, &mut write_fn, &mut read_fn)?;
                }
                TransferProtocol::XmodemCrc => {
                    self.send_xmodem_crc_chunk(chunk, bytes_sent as u16, &mut write_fn, &mut read_fn)?;
                }
                TransferProtocol::Ymodem => {
                    self.send_ymodem_chunk(chunk, bytes_sent, file_size, &mut write_fn, &mut read_fn)?;
                }
                TransferProtocol::Zmodem => {
                    self.send_zmodem_chunk(chunk, bytes_sent, file_size, &mut write_fn, &mut read_fn)?;
                }
            }

            bytes_sent += bytes_read as u64;
            let elapsed = start_time.elapsed();
            let speed = if elapsed.as_secs() > 0 {
                bytes_sent as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };

            let eta = if speed > 0.0 {
                let remaining = file_size - bytes_sent;
                Some(Duration::from_secs_f64(remaining as f64 / speed))
            } else {
                None
            };

            progress_fn(TransferProgress {
                bytes_transferred: bytes_sent,
                total_bytes: file_size,
                speed_bps: speed,
                elapsed,
                eta,
            });
        }

        Ok(())
    }

    pub fn receive_file<F, G>(
        &self,
        file_path: &Path,
        mut write_fn: F,
        mut read_fn: G,
        mut progress_fn: impl FnMut(TransferProgress),
    ) -> TransferResult<()>
    where
        F: FnMut(&[u8]) -> TransferResult<()>,
        G: FnMut() -> TransferResult<u8>,
    {
        let mut file = File::create(file_path)?;
        let mut buffer: Vec<u8> = Vec::new();
        let mut bytes_received = 0u64;
        let start_time = std::time::Instant::now();

        loop {
            let chunk = match self.protocol {
                TransferProtocol::Xmodem => {
                    self.receive_xmodem_chunk(&mut read_fn)?
                }
                TransferProtocol::XmodemCrc => {
                    self.receive_xmodem_crc_chunk(&mut read_fn)?
                }
                TransferProtocol::Ymodem => {
                    self.receive_ymodem_chunk(&mut read_fn)?
                }
                TransferProtocol::Zmodem => {
                    self.receive_zmodem_chunk(&mut read_fn)?
                }
            };

            if chunk.is_empty() {
                break;
            }

            file.write_all(&chunk)?;
            bytes_received += chunk.len() as u64;

            let elapsed = start_time.elapsed();
            let speed = if elapsed.as_secs() > 0 {
                bytes_received as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };

            progress_fn(TransferProgress {
                bytes_transferred: bytes_received,
                total_bytes: 0,
                speed_bps: speed,
                elapsed,
                eta: None,
            });
        }

        Ok(())
    }

    fn send_xmodem_chunk<F, G>(
        &self,
        chunk: &[u8],
        block_num: u16,
        write_fn: &mut F,
        read_fn: &mut G,
    ) -> TransferResult<()>
    where
        F: FnMut(&[u8]) -> TransferResult<()>,
        G: FnMut() -> TransferResult<u8>,
    {
        let mut packet = Vec::new();
        packet.push(0x01); // SOH
        packet.push((block_num & 0xFF) as u8);
        packet.push((!(block_num & 0xFF) as u8));

        let mut padded = chunk.to_vec();
        padded.resize(self.chunk_size, 0x1A); // Pad with SUB
        packet.extend_from_slice(&padded);

        let checksum = Self::checksum_simple(&padded);
        packet.push(checksum);

        write_fn(&packet)?;

        let ack = read_fn()?;
        if ack != 0x06 {
            return Err(TransferError::ProtocolError(
                "Expected ACK".to_string(),
            ));
        }

        Ok(())
    }

    fn send_xmodem_crc_chunk<F, G>(
        &self,
        chunk: &[u8],
        block_num: u16,
        write_fn: &mut F,
        read_fn: &mut G,
    ) -> TransferResult<()>
    where
        F: FnMut(&[u8]) -> TransferResult<()>,
        G: FnMut() -> TransferResult<u8>,
    {
        let mut packet = Vec::new();
        packet.push(0x01); // SOH
        packet.push((block_num & 0xFF) as u8);
        packet.push((!(block_num & 0xFF) as u8));

        let mut padded = chunk.to_vec();
        padded.resize(self.chunk_size, 0x1A);
        packet.extend_from_slice(&padded);

        let crc = Self::crc16(&padded);
        packet.extend_from_slice(&crc.to_be_bytes());

        write_fn(&packet)?;

        let ack = read_fn()?;
        if ack != 0x06 {
            return Err(TransferError::ProtocolError(
                "Expected ACK".to_string(),
            ));
        }

        Ok(())
    }

    fn send_ymodem_chunk<F, G>(
        &self,
        chunk: &[u8],
        offset: u64,
        total_size: u64,
        write_fn: &mut F,
        read_fn: &mut G,
    ) -> TransferResult<()>
    where
        F: FnMut(&[u8]) -> TransferResult<()>,
        G: FnMut() -> TransferResult<u8>,
    {
        let mut packet = Vec::new();
        packet.push(0x02); // STX
        packet.push(((offset / 1024) & 0xFF) as u8);
        packet.push((!((offset / 1024) & 0xFF) as u8));

        let mut padded = chunk.to_vec();
        padded.resize(self.chunk_size, 0x1A);
        packet.extend_from_slice(&padded);

        let crc = Self::crc16(&padded);
        packet.extend_from_slice(&crc.to_be_bytes());

        write_fn(&packet)?;

        let ack = read_fn()?;
        if ack != 0x06 {
            return Err(TransferError::ProtocolError(
                "Expected ACK".to_string(),
            ));
        }

        Ok(())
    }

    fn send_zmodem_chunk<F, G>(
        &self,
        chunk: &[u8],
        offset: u64,
        total_size: u64,
        write_fn: &mut F,
        _read_fn: &mut G,
    ) -> TransferResult<()>
    where
        F: FnMut(&[u8]) -> TransferResult<()>,
        G: FnMut() -> TransferResult<u8>,
    {
        // Simplified ZMODEM implementation
        let mut packet = Vec::new();
        packet.extend_from_slice(b"**\x18\x01"); // ZPAD ZDLE ZDATA

        let mut header = Vec::new();
        header.push(0x00); // Frame type
        header.extend_from_slice(&offset.to_le_bytes());
        header.extend_from_slice(&total_size.to_le_bytes());

        let crc = Self::crc16(&header);
        packet.extend_from_slice(&header);
        packet.extend_from_slice(&crc.to_le_bytes());

        // Send data with transparency
        for &byte in chunk {
            if byte == 0x0D || byte == 0x10 || byte == 0x11 || byte == 0x13 || byte == 0x18 || byte == 0x1B {
                packet.push(0x10); // ZDLE
                packet.push(byte ^ 0x40);
            } else {
                packet.push(byte);
            }
        }

        packet.extend_from_slice(b"\x18\x04"); // ZCRCZ

        write_fn(&packet)?;

        Ok(())
    }

    fn receive_xmodem_chunk<G>(&self, read_fn: &mut G) -> TransferResult<Vec<u8>>
    where
        G: FnMut() -> TransferResult<u8>,
    {
        let header = read_fn()?;
        match header {
            0x01 => {
                let _block_num = read_fn()?;
                let _block_inv = read_fn()?;

                let mut data = Vec::new();
                for _ in 0..self.chunk_size {
                    data.push(read_fn()?);
                }

                let _checksum = read_fn()?;
                let _expected = Self::checksum_simple(&data);

                // In a real implementation, send ACK/NAK based on checksum
                Ok(data)
            }
            0x04 => Ok(Vec::new()), // EOT
            _ => Err(TransferError::ProtocolError(
                "Unexpected header".to_string(),
            )),
        }
    }

    fn receive_xmodem_crc_chunk<G>(&self, read_fn: &mut G) -> TransferResult<Vec<u8>>
    where
        G: FnMut() -> TransferResult<u8>,
    {
        let header = read_fn()?;
        match header {
            0x01 => {
                let _block_num = read_fn()?;
                let _block_inv = read_fn()?;

                let mut data = Vec::new();
                for _ in 0..self.chunk_size {
                    data.push(read_fn()?);
                }

                let crc_hi = read_fn()? as u16;
                let crc_lo = read_fn()? as u16;
                let _received_crc = (crc_hi << 8) | crc_lo;
                let _expected_crc = Self::crc16(&data);

                // In a real implementation, send ACK/NAK based on CRC
                Ok(data)
            }
            0x04 => Ok(Vec::new()),
            _ => Err(TransferError::ProtocolError(
                "Unexpected header".to_string(),
            )),
        }
    }

    fn receive_ymodem_chunk<G>(&self, read_fn: &mut G) -> TransferResult<Vec<u8>>
    where
        G: FnMut() -> TransferResult<u8>,
    {
        self.receive_xmodem_crc_chunk(read_fn)
    }

    fn receive_zmodem_chunk<G>(&self, read_fn: &mut G) -> TransferResult<Vec<u8>>
    where
        G: FnMut() -> TransferResult<u8>,
    {
        // Simplified ZMODEM receive
        let mut buffer = Vec::new();
        loop {
            let byte = read_fn()?;
            if byte == 0x04 {
                break;
            }
            buffer.push(byte);
        }
        Ok(buffer)
    }

    fn checksum_simple(data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
    }

    fn crc16(data: &[u8]) -> u16 {
        let mut crc: u16 = 0;
        for &byte in data {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }
}

fn write_fn_simple(_read_fn: &mut impl FnMut() -> TransferResult<u8>, _data: &[u8]) -> TransferResult<()> {
    // This is a simplified implementation
    // In real code, you'd need a proper write function
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let checksum = FileTransfer::checksum_simple(&data);
        assert_eq!(checksum, 10);
    }

    #[test]
    fn test_crc16() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let crc = FileTransfer::crc16(&data);
        assert!(crc != 0);
    }

    #[test]
    fn test_transfer_progress() {
        let progress = TransferProgress {
            bytes_transferred: 50,
            total_bytes: 100,
            speed_bps: 1024.0,
            elapsed: Duration::from_secs(1),
            eta: Some(Duration::from_secs(1)),
        };

        assert_eq!(progress.percentage(), 50.0);
    }

    #[test]
    fn test_file_transfer_new() {
        let transfer = FileTransfer::new(TransferProtocol::Xmodem);
        assert_eq!(transfer.chunk_size, 128);

        let transfer = FileTransfer::new(TransferProtocol::Ymodem);
        assert_eq!(transfer.chunk_size, 1024);
    }
}
