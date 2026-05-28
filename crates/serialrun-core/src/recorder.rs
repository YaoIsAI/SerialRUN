use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecorderError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Script error: {0}")]
    ScriptError(String),
}

pub type RecorderResult<T> = Result<T, RecorderError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptCommand {
    pub delay_ms: u64,
    pub action: Action,
    pub data: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Send,
    Wait,
    Read,
    Comment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub name: String,
    pub description: String,
    pub commands: Vec<ScriptCommand>,
    pub created_at: DateTime<Utc>,
}

impl Script {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            commands: Vec::new(),
            created_at: Utc::now(),
        }
    }

    pub fn add_command(&mut self, command: ScriptCommand) {
        self.commands.push(command);
    }

    pub fn add_send(&mut self, data: &str, delay_ms: u64) {
        self.add_command(ScriptCommand {
            delay_ms,
            action: Action::Send,
            data: Some(data.to_string()),
            timestamp: Some(Utc::now()),
        });
    }

    pub fn add_wait(&mut self, duration_ms: u64) {
        self.add_command(ScriptCommand {
            delay_ms: duration_ms,
            action: Action::Wait,
            data: None,
            timestamp: Some(Utc::now()),
        });
    }

    pub fn add_comment(&mut self, comment: &str) {
        self.add_command(ScriptCommand {
            delay_ms: 0,
            action: Action::Comment,
            data: Some(comment.to_string()),
            timestamp: Some(Utc::now()),
        });
    }

    pub fn save(&self, path: &Path) -> RecorderResult<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load(path: &Path) -> RecorderResult<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let script: Script = serde_json::from_reader(reader)?;
        Ok(script)
    }

    pub fn save_text(&self, path: &Path) -> RecorderResult<()> {
        let mut file = File::create(path)?;
        writeln!(file, "# Script: {}", self.name)?;
        writeln!(file, "# Description: {}", self.description)?;
        writeln!(file, "# Created: {}", self.created_at)?;
        writeln!(file)?;

        for cmd in &self.commands {
            match cmd.action {
                Action::Send => {
                    writeln!(file, "SEND {} {}", cmd.delay_ms, cmd.data.as_deref().unwrap_or(""))?;
                }
                Action::Wait => {
                    writeln!(file, "WAIT {}", cmd.delay_ms)?;
                }
                Action::Read => {
                    writeln!(file, "READ {}", cmd.delay_ms)?;
                }
                Action::Comment => {
                    writeln!(file, "# {}", cmd.data.as_deref().unwrap_or(""))?;
                }
            }
        }

        Ok(())
    }

    pub fn load_text(path: &Path) -> RecorderResult<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut commands = Vec::new();
        let mut name = String::new();
        let mut description = String::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.starts_with("# ") {
                let content = &line[2..];
                if content.starts_with("Script: ") {
                    name = content[8..].to_string();
                } else if content.starts_with("Description: ") {
                    description = content[13..].to_string();
                }
                continue;
            }

            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() < 2 {
                continue;
            }

            let action = match parts[0].to_uppercase().as_str() {
                "SEND" => Action::Send,
                "WAIT" => Action::Wait,
                "READ" => Action::Read,
                _ => continue,
            };

            let delay_ms = parts[1].parse().unwrap_or(0);
            let data = if parts.len() > 2 {
                Some(parts[2].to_string())
            } else {
                None
            };

            commands.push(ScriptCommand {
                delay_ms,
                action,
                data,
                timestamp: Some(Utc::now()),
            });
        }

        Ok(Script {
            name,
            description,
            commands,
            created_at: Utc::now(),
        })
    }
}

pub struct ScriptRecorder {
    script: Script,
    is_recording: bool,
}

impl ScriptRecorder {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            script: Script::new(name, description),
            is_recording: false,
        }
    }

    pub fn start(&mut self) {
        self.is_recording = true;
    }

    pub fn stop(&mut self) {
        self.is_recording = false;
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn record_send(&mut self, data: &str) {
        if self.is_recording {
            self.script.add_send(data, 0);
        }
    }

    pub fn record_wait(&mut self, duration_ms: u64) {
        if self.is_recording {
            self.script.add_wait(duration_ms);
        }
    }

    pub fn record_comment(&mut self, comment: &str) {
        if self.is_recording {
            self.script.add_comment(comment);
        }
    }

    pub fn script(&self) -> &Script {
        &self.script
    }

    pub fn save(&self, path: &Path) -> RecorderResult<()> {
        self.script.save(path)
    }
}

pub struct ScriptReplayer {
    script: Script,
    current_index: usize,
    is_playing: bool,
}

impl ScriptReplayer {
    pub fn new(script: Script) -> Self {
        Self {
            script,
            current_index: 0,
            is_playing: false,
        }
    }

    pub fn load(path: &Path) -> RecorderResult<Self> {
        let script = if path.extension().map_or(false, |e| e == "json") {
            Script::load(path)?
        } else {
            Script::load_text(path)?
        };
        Ok(Self::new(script))
    }

    pub fn start(&mut self) {
        self.current_index = 0;
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn next_command(&mut self) -> Option<&ScriptCommand> {
        if !self.is_playing || self.current_index >= self.script.commands.len() {
            self.is_playing = false;
            return None;
        }

        let cmd = &self.script.commands[self.current_index];
        self.current_index += 1;
        Some(cmd)
    }

    pub fn get_delay(&self) -> Duration {
        if self.current_index > 0 && self.current_index <= self.script.commands.len() {
            let cmd = &self.script.commands[self.current_index - 1];
            Duration::from_millis(cmd.delay_ms)
        } else {
            Duration::from_millis(100)
        }
    }

    pub fn progress(&self) -> f32 {
        if self.script.commands.is_empty() {
            return 1.0;
        }
        self.current_index as f32 / self.script.commands.len() as f32
    }

    pub fn script(&self) -> &Script {
        &self.script
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_new() {
        let script = Script::new("test", "Test script");
        assert_eq!(script.name, "test");
        assert!(script.commands.is_empty());
    }

    #[test]
    fn test_script_add_commands() {
        let mut script = Script::new("test", "Test script");
        script.add_send("Hello\r\n", 100);
        script.add_wait(500);
        script.add_comment("Test comment");

        assert_eq!(script.commands.len(), 3);
        assert_eq!(script.commands[0].action, Action::Send);
        assert_eq!(script.commands[1].action, Action::Wait);
        assert_eq!(script.commands[2].action, Action::Comment);
    }

    #[test]
    fn test_recorder() {
        let mut recorder = ScriptRecorder::new("test", "Test recorder");
        recorder.start();
        recorder.record_send("AT+RST\r\n");
        recorder.record_wait(1000);
        recorder.record_send("AT+CWMODE=1\r\n");
        recorder.stop();

        assert!(!recorder.is_recording());
        assert_eq!(recorder.script().commands.len(), 3);
    }

    #[test]
    fn test_replayer() {
        let mut script = Script::new("test", "Test script");
        script.add_send("AT+RST\r\n", 100);
        script.add_wait(500);

        let mut replayer = ScriptReplayer::new(script);
        replayer.start();

        let cmd1 = replayer.next_command().unwrap();
        assert_eq!(cmd1.action, Action::Send);

        let cmd2 = replayer.next_command().unwrap();
        assert_eq!(cmd2.action, Action::Wait);

        assert!(replayer.next_command().is_none());
        assert!(!replayer.is_playing());
    }
}
