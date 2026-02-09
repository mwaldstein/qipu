//! Session-level aggregation and bucketing for privacy

use std::collections::{HashMap, HashSet};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::events::{
    CommandName, DurationBucket, ErrorType, NoteCountBucket, TelemetryEvent, WorkspaceCountBucket,
};

const MAX_EVENTS_PER_BATCH: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedSession {
    pub date: String,
    pub os_platform: String,
    pub app_version: String,
    pub workspace_count: WorkspaceCountBucket,
    pub note_count: NoteCountBucket,
    pub command_counts: HashMap<CommandName, CommandStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandStats {
    pub success_count: u32,
    pub failure_count: u32,
    pub duration_buckets: HashMap<DurationBucket, u32>,
    pub error_types: HashSet<ErrorType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadBatch {
    pub sessions: Vec<AggregatedSession>,
    pub metadata: UploadMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMetadata {
    pub batch_id: String,
    pub batch_date: String,
    pub session_count: usize,
}

pub struct SessionAggregator {
    current_session: Option<AggregatedSession>,
    events_today: Vec<TelemetryEvent>,
}

impl SessionAggregator {
    pub fn new() -> Self {
        Self {
            current_session: None,
            events_today: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: TelemetryEvent) {
        let today = Self::get_today();

        if self.current_session.is_none() || self.current_session.as_ref().unwrap().date != today {
            self.start_new_session(&today);
        }

        self.events_today.push(event.clone());

        if let Some(session) = &mut self.current_session {
            match event {
                TelemetryEvent::CommandExecuted {
                    timestamp: _,
                    command,
                    success,
                    duration,
                    error,
                } => {
                    let stats = session
                        .command_counts
                        .entry(command)
                        .or_insert_with(CommandStats::default);

                    if success {
                        stats.success_count += 1;
                    } else {
                        stats.failure_count += 1;
                    }

                    *stats.duration_buckets.entry(duration).or_insert(0) += 1;

                    if let Some(err) = error {
                        stats.error_types.insert(err);
                    }
                }
                TelemetryEvent::SessionStats {
                    os_platform,
                    app_version,
                    workspace_count,
                    note_count,
                    ..
                } => {
                    session.os_platform = os_platform;
                    session.app_version = app_version;
                    session.workspace_count = workspace_count;
                    session.note_count = note_count;
                }
                TelemetryEvent::QueryStats { .. } => {
                    // Query statistics are for local observability only
                    // and not included in aggregated telemetry uploads
                }
            }
        }
    }

    pub fn finalize_session(&mut self) -> Option<AggregatedSession> {
        self.current_session.take()
    }

    pub fn create_upload_batch(&self, sessions: Vec<AggregatedSession>) -> UploadBatch {
        let batch_date = Utc::now().format("%Y-%m-%d").to_string();
        let batch_id = ulid::Ulid::new().to_string();
        let session_count = sessions.len();

        UploadBatch {
            sessions,
            metadata: UploadMetadata {
                batch_id,
                batch_date,
                session_count,
            },
        }
    }

    pub fn aggregate_events(events: Vec<TelemetryEvent>) -> UploadBatch {
        let mut aggregator = SessionAggregator::new();

        for event in events {
            aggregator.add_event(event);
        }

        let session = aggregator.finalize_session();
        let sessions = if let Some(s) = session {
            vec![s]
        } else {
            vec![]
        };

        aggregator.create_upload_batch(sessions)
    }

    fn start_new_session(&mut self, date: &str) {
        self.current_session = Some(AggregatedSession {
            date: date.to_string(),
            os_platform: std::env::consts::OS.to_string(),
            app_version: super::get_app_version().to_string(),
            workspace_count: WorkspaceCountBucket::Zero,
            note_count: super::events::NoteCountBucket::LessThan10,
            command_counts: HashMap::new(),
        });
    }

    fn get_today() -> String {
        Utc::now().format("%Y-%m-%d").to_string()
    }
}

impl Default for SessionAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl UploadBatch {
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn total_events(&self) -> usize {
        self.sessions
            .iter()
            .map(|s| {
                s.command_counts
                    .values()
                    .map(|c| c.success_count + c.failure_count)
                    .sum::<u32>() as usize
            })
            .sum()
    }

    pub fn should_upload(&self) -> bool {
        !self.is_empty() && self.total_events() >= MAX_EVENTS_PER_BATCH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_single_command() {
        let events = vec![TelemetryEvent::CommandExecuted {
            timestamp: Utc::now().timestamp(),
            command: CommandName::List,
            success: true,
            duration: DurationBucket::Ms100To500,
            error: None,
        }];

        let batch = SessionAggregator::aggregate_events(events);

        assert_eq!(batch.sessions.len(), 1);
        let session = &batch.sessions[0];
        assert_eq!(session.command_counts.len(), 1);
        assert_eq!(
            session
                .command_counts
                .get(&CommandName::List)
                .unwrap()
                .success_count,
            1
        );
    }

    #[test]
    fn test_aggregate_multiple_commands() {
        let now = Utc::now().timestamp();
        let events = vec![
            TelemetryEvent::CommandExecuted {
                timestamp: now,
                command: CommandName::List,
                success: true,
                duration: DurationBucket::LessThan100ms,
                error: None,
            },
            TelemetryEvent::CommandExecuted {
                timestamp: now + 1,
                command: CommandName::Capture,
                success: true,
                duration: DurationBucket::Ms500To1s,
                error: None,
            },
            TelemetryEvent::CommandExecuted {
                timestamp: now + 2,
                command: CommandName::List,
                success: false,
                duration: DurationBucket::MoreThan5s,
                error: Some(ErrorType::IOError),
            },
        ];

        let batch = SessionAggregator::aggregate_events(events);

        assert_eq!(batch.sessions.len(), 1);
        let session = &batch.sessions[0];
        assert_eq!(session.command_counts.len(), 2);

        let list_stats = session.command_counts.get(&CommandName::List).unwrap();
        assert_eq!(list_stats.success_count, 1);
        assert_eq!(list_stats.failure_count, 1);
        assert!(list_stats.error_types.contains(&ErrorType::IOError));

        let capture_stats = session.command_counts.get(&CommandName::Capture).unwrap();
        assert_eq!(capture_stats.success_count, 1);
        assert_eq!(capture_stats.failure_count, 0);
    }

    #[test]
    fn test_empty_events_creates_empty_batch() {
        let events = vec![];
        let batch = SessionAggregator::aggregate_events(events);

        assert!(batch.is_empty());
        assert_eq!(batch.sessions.len(), 0);
    }

    #[test]
    fn test_upload_batch_metadata() {
        let now = Utc::now().timestamp();
        let events = vec![TelemetryEvent::CommandExecuted {
            timestamp: now,
            command: CommandName::Show,
            success: true,
            duration: DurationBucket::LessThan100ms,
            error: None,
        }];

        let batch = SessionAggregator::aggregate_events(events);

        assert!(!batch.metadata.batch_id.is_empty());
        assert!(!batch.metadata.batch_date.is_empty());
        assert_eq!(batch.metadata.session_count, 1);
    }

    #[test]
    fn test_should_upload_threshold() {
        let now = Utc::now().timestamp();
        let mut events = Vec::new();

        for i in 0..99 {
            events.push(TelemetryEvent::CommandExecuted {
                timestamp: now + i as i64,
                command: CommandName::List,
                success: true,
                duration: DurationBucket::LessThan100ms,
                error: None,
            });
        }

        let batch = SessionAggregator::aggregate_events(events.clone());
        assert!(!batch.should_upload());

        events.push(TelemetryEvent::CommandExecuted {
            timestamp: now + 100,
            command: CommandName::List,
            success: true,
            duration: DurationBucket::LessThan100ms,
            error: None,
        });

        let batch = SessionAggregator::aggregate_events(events);
        assert!(batch.should_upload());
    }

    #[test]
    fn test_aggregation_preserves_anonymity() {
        let now = Utc::now().timestamp();
        let events = vec![
            TelemetryEvent::CommandExecuted {
                timestamp: now,
                command: CommandName::List,
                success: true,
                duration: DurationBucket::Ms100To500,
                error: None,
            },
            TelemetryEvent::SessionStats {
                timestamp: now,
                os_platform: "linux".to_string(),
                app_version: "0.1.0".to_string(),
                workspace_count: WorkspaceCountBucket::TwoToFive,
                note_count: NoteCountBucket::TenTo100,
            },
        ];

        let batch = SessionAggregator::aggregate_events(events);
        let json = serde_json::to_string(&batch).unwrap();

        assert!(!json.contains("session_id"));
        assert!(!json.contains("user_id"));
        assert!(!json.contains("sequence"));
    }
}
