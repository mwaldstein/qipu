//! Telemetry event collector and local storage

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use serde_json;

use super::events::{NoteCountBucket, TelemetryEvent, WorkspaceCountBucket};

const MAX_EVENTS: usize = 1000;
const MAX_RETENTION_DAYS: i64 = 7;

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub events_dir: PathBuf,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        let events_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from(".local/share"))
            .join("qipu/telemetry");

        fs::create_dir_all(&events_dir).ok();

        Self {
            enabled: std::env::var("QIPU_NO_TELEMETRY").is_err(),
            events_dir,
        }
    }
}

pub struct TelemetryCollector {
    config: TelemetryConfig,
    events: Arc<Mutex<Vec<TelemetryEvent>>>,
}

impl TelemetryCollector {
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn record_command(
        &self,
        command: super::events::CommandName,
        success: bool,
        duration_ms: u128,
        error_type: Option<super::events::ErrorType>,
    ) {
        if !self.is_enabled() {
            return;
        }

        let event = TelemetryEvent::CommandExecuted {
            timestamp: Utc::now().timestamp(),
            command,
            success,
            duration: super::events::DurationBucket::from_millis(duration_ms),
            error: error_type,
        };

        self.add_event(event);
    }

    pub fn record_session_stats(
        &self,
        workspace_count: usize,
        note_count: usize,
        app_version: &str,
    ) {
        if !self.is_enabled() {
            return;
        }

        let event = TelemetryEvent::SessionStats {
            timestamp: Utc::now().timestamp(),
            os_platform: std::env::consts::OS.to_string(),
            app_version: app_version.to_string(),
            workspace_count: WorkspaceCountBucket::from_count(workspace_count),
            note_count: NoteCountBucket::from_count(note_count),
        };

        self.add_event(event);
    }

    fn add_event(&self, event: TelemetryEvent) {
        let mut events = self.events.lock().unwrap();

        events.push(event);

        if events.len() > MAX_EVENTS {
            events.remove(0);
        }
    }

    pub fn get_pending_events(&self) -> Vec<TelemetryEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn persist_to_disk(&self) -> Result<(), std::io::Error> {
        if !self.is_enabled() {
            return Ok(());
        }

        let events = self.events.lock().unwrap();
        if events.is_empty() {
            return Ok(());
        }

        let events_file = self.config.events_dir.join("events.jsonl");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&events_file)?;

        for event in events.iter() {
            let line = serde_json::to_string(event)
                .map_err(|e| std::io::Error::other(format!("serialization error: {}", e)))?;
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    pub fn rotate_events(&self) -> Result<(), std::io::Error> {
        let events_file = self.config.events_dir.join("events.jsonl");
        if !events_file.exists() {
            return Ok(());
        }

        let cutoff = Utc::now().timestamp() - (MAX_RETENTION_DAYS * 24 * 60 * 60);

        let content = fs::read_to_string(&events_file)?;
        let filtered: Vec<&str> = content
            .lines()
            .filter(|line| {
                if let Ok(event) = serde_json::from_str::<TelemetryEvent>(line) {
                    event.timestamp() >= cutoff
                } else {
                    false
                }
            })
            .collect();

        let temp_file = self.config.events_dir.join("events.jsonl.tmp");
        let mut writer = BufWriter::new(File::create(&temp_file)?);
        for line in filtered {
            writeln!(writer, "{}", line)?;
        }
        writer.flush()?;

        fs::rename(temp_file, events_file)?;

        Ok(())
    }

    pub fn clear_events(&self) {
        self.events.lock().unwrap().clear();
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new(TelemetryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_max_events_limit() {
        let collector = TelemetryCollector::new(TelemetryConfig {
            enabled: true,
            events_dir: PathBuf::from("/tmp/test_telemetry"),
        });

        for i in 0..1100 {
            collector.add_event(TelemetryEvent::CommandExecuted {
                timestamp: i as i64,
                command: super::super::events::CommandName::List,
                success: true,
                duration: super::super::events::DurationBucket::LessThan100ms,
                error: None,
            });
        }

        let events = collector.get_pending_events();
        assert_eq!(events.len(), MAX_EVENTS);
    }

    #[test]
    fn test_collector_disabled_no_collection() {
        let collector = TelemetryCollector::new(TelemetryConfig {
            enabled: false,
            events_dir: PathBuf::from("/tmp/test_telemetry"),
        });

        collector.record_command(super::super::events::CommandName::List, true, 50, None);

        let events = collector.get_pending_events();
        assert!(events.is_empty());
    }

    #[test]
    fn test_rotate_events_removes_old() {
        let temp_dir = tempfile::tempdir().unwrap();
        let events_dir = temp_dir.path().join("telemetry");
        fs::create_dir_all(&events_dir).unwrap();

        let events_file = events_dir.join("events.jsonl");

        let old_timestamp = Utc::now().timestamp() - (10 * 24 * 60 * 60);
        let new_timestamp = Utc::now().timestamp();

        let old_event = TelemetryEvent::CommandExecuted {
            timestamp: old_timestamp,
            command: super::super::events::CommandName::List,
            success: true,
            duration: super::super::events::DurationBucket::LessThan100ms,
            error: None,
        };

        let new_event = TelemetryEvent::CommandExecuted {
            timestamp: new_timestamp,
            command: super::super::events::CommandName::List,
            success: true,
            duration: super::super::events::DurationBucket::LessThan100ms,
            error: None,
        };

        let mut file = File::create(&events_file).unwrap();
        writeln!(file, "{}", serde_json::to_string(&old_event).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&new_event).unwrap()).unwrap();

        let collector = TelemetryCollector::new(TelemetryConfig {
            enabled: true,
            events_dir,
        });

        collector.rotate_events().unwrap();

        let content = fs::read_to_string(&events_file).unwrap();
        assert!(!content.contains(&old_timestamp.to_string()));
        assert!(content.contains(&new_timestamp.to_string()));
    }
}
