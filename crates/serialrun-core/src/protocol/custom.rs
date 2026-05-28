use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type ProtocolResult<T> = Result<T, ProtocolError>;

#[derive(Debug, Clone)]
pub struct ProtocolFrame {
    pub timestamp: i64,
    pub direction: Direction,
    pub data: Vec<u8>,
    pub parsed: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Rx,
    Tx,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Rx => write!(f, "RX"),
            Direction::Tx => write!(f, "TX"),
        }
    }
}

pub struct ProtocolPattern {
    pub name: String,
    pub pattern: Regex,
    pub description: String,
    pub parser: Option<ParserFn>,
}

pub type ParserFn = Box<dyn Fn(&[u8]) -> Option<String> + Send + Sync>;

pub struct ProtocolParser {
    patterns: Vec<ProtocolPattern>,
}

impl ProtocolParser {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    pub fn add_pattern(
        &mut self,
        name: &str,
        pattern: &str,
        description: &str,
    ) -> ProtocolResult<()> {
        let regex = Regex::new(pattern).map_err(|e| {
            ProtocolError::InvalidPattern(format!("{}: {}", pattern, e))
        })?;

        self.patterns.push(ProtocolPattern {
            name: name.to_string(),
            pattern: regex,
            description: description.to_string(),
            parser: None,
        });

        Ok(())
    }

    pub fn add_pattern_with_parser(
        &mut self,
        name: &str,
        pattern: &str,
        description: &str,
        parser: ParserFn,
    ) -> ProtocolResult<()> {
        let regex = Regex::new(pattern).map_err(|e| {
            ProtocolError::InvalidPattern(format!("{}: {}", pattern, e))
        })?;

        self.patterns.push(ProtocolPattern {
            name: name.to_string(),
            pattern: regex,
            description: description.to_string(),
            parser: Some(parser),
        });

        Ok(())
    }

    pub fn parse(&self, data: &[u8]) -> Option<ProtocolFrame> {
        let text = String::from_utf8_lossy(data);

        for pattern in &self.patterns {
            if pattern.pattern.is_match(&text) {
                let parsed = if let Some(parser) = &pattern.parser {
                    parser(data)
                } else {
                    Some(format!("[{}] {}", pattern.name, text))
                };

                return Some(ProtocolFrame {
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    direction: Direction::Rx,
                    data: data.to_vec(),
                    parsed,
                });
            }
        }

        None
    }

    pub fn patterns(&self) -> &[ProtocolPattern] {
        &self.patterns
    }

    pub fn clear(&mut self) {
        self.patterns.clear();
    }
}

impl Default for ProtocolParser {
    fn default() -> Self {
        let mut parser = Self::new();

        let _ = parser.add_pattern("AT Command", r"^(AT|AT\+)", "AT command pattern");
        let _ = parser.add_pattern("JSON", r"^\{.*\}$", "JSON object");
        let _ = parser.add_pattern("Hex Data", r"^[0-9A-Fa-f\s]+$", "Hexadecimal data");
        let _ = parser.add_pattern("Error", r"(?i)error|fail|err", "Error message");

        parser
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_parser_default() {
        let parser = ProtocolParser::default();
        assert!(!parser.patterns().is_empty());
    }

    #[test]
    fn test_add_pattern() {
        let mut parser = ProtocolParser::new();
        assert!(parser.add_pattern("Test", r"^TEST", "Test pattern").is_ok());
        assert_eq!(parser.patterns().len(), 1);
    }

    #[test]
    fn test_parse_at_command() {
        let parser = ProtocolParser::default();
        let data = b"AT+RST\r\n";
        let frame = parser.parse(data);
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert!(frame.parsed.unwrap().contains("AT Command"));
    }

    #[test]
    fn test_parse_json() {
        let parser = ProtocolParser::default();
        let data = b"{\"key\": \"value\"}";
        let frame = parser.parse(data);
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert!(frame.parsed.unwrap().contains("JSON"));
    }

    #[test]
    fn test_parse_no_match() {
        let parser = ProtocolParser::default();
        let data = b"hello world";
        let frame = parser.parse(data);
        assert!(frame.is_none());
    }

    #[test]
    fn test_custom_parser() {
        let mut parser = ProtocolParser::new();
        let _ = parser.add_pattern_with_parser(
            "Custom",
            r"^CUSTOM",
            "Custom protocol",
            Box::new(|data| {
                let text = String::from_utf8_lossy(data);
                Some(format!("Parsed: {}", text.trim()))
            }),
        );

        let data = b"CUSTOM:123";
        let frame = parser.parse(data);
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert!(frame.parsed.unwrap().contains("Parsed: CUSTOM:123"));
    }

    #[test]
    fn test_invalid_pattern() {
        let mut parser = ProtocolParser::new();
        let result = parser.add_pattern("Invalid", r"[invalid", "Invalid regex");
        assert!(result.is_err());
    }
}
