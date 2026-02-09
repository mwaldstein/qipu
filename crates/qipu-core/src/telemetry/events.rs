//! Telemetry event types for anonymous usage analytics

use serde::{Deserialize, Serialize};

/// Duration buckets for command execution time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DurationBucket {
    LessThan100ms,
    Ms100To500,
    Ms500To1s,
    S1To5,
    MoreThan5s,
}

impl DurationBucket {
    pub fn from_millis(ms: u128) -> Self {
        if ms < 100 {
            Self::LessThan100ms
        } else if ms < 500 {
            Self::Ms100To500
        } else if ms < 1000 {
            Self::Ms500To1s
        } else if ms < 5000 {
            Self::S1To5
        } else {
            Self::MoreThan5s
        }
    }
}

/// Workspace count buckets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceCountBucket {
    Zero,
    One,
    TwoToFive,
    SixToTwenty,
    MoreThanTwenty,
}

impl WorkspaceCountBucket {
    pub fn from_count(count: usize) -> Self {
        match count {
            0 => Self::Zero,
            1 => Self::One,
            2..=5 => Self::TwoToFive,
            6..=20 => Self::SixToTwenty,
            _ => Self::MoreThanTwenty,
        }
    }
}

/// Note count buckets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoteCountBucket {
    LessThan10,
    TenTo100,
    HundredTo1000,
    ThousandTo10000,
    MoreThan10000,
}

impl NoteCountBucket {
    pub fn from_count(count: usize) -> Self {
        match count {
            0..=9 => Self::LessThan10,
            10..=100 => Self::TenTo100,
            101..=1000 => Self::HundredTo1000,
            1001..=10000 => Self::ThousandTo10000,
            _ => Self::MoreThan10000,
        }
    }
}

/// Command names (enum variants only, no args)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandName {
    Init,
    Create,
    New,
    List,
    Show,
    Inbox,
    Capture,
    Index,
    Search,
    Edit,
    Update,
    Context,
    Dump,
    Export,
    Load,
    Prime,
    Quickstart,
    Verify,
    Value,
    Tags,
    Custom,
    Link,
    Onboard,
    Setup,
    Doctor,
    Sync,
    Compact,
    Workspace,
    Merge,
    Store,
    Ontology,
    Telemetry,
    Hooks,
}

impl CommandName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Create => "create",
            Self::New => "new",
            Self::List => "list",
            Self::Show => "show",
            Self::Inbox => "inbox",
            Self::Capture => "capture",
            Self::Index => "index",
            Self::Search => "search",
            Self::Edit => "edit",
            Self::Update => "update",
            Self::Context => "context",
            Self::Dump => "dump",
            Self::Export => "export",
            Self::Load => "load",
            Self::Prime => "prime",
            Self::Quickstart => "quickstart",
            Self::Verify => "verify",
            Self::Value => "value",
            Self::Tags => "tags",
            Self::Custom => "custom",
            Self::Link => "link",
            Self::Onboard => "onboard",
            Self::Setup => "setup",
            Self::Doctor => "doctor",
            Self::Sync => "sync",
            Self::Compact => "compact",
            Self::Workspace => "workspace",
            Self::Merge => "merge",
            Self::Store => "store",
            Self::Ontology => "ontology",
            Self::Telemetry => "telemetry",
            Self::Hooks => "hooks",
        }
    }
}

/// Error type variants only (no messages or paths)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorType {
    UsageError,
    StoreError,
    ConfigError,
    IOError,
    DatabaseError,
    ParseError,
    ValidateError,
    Other,
}

/// Query types for database operation tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QueryType {
    Search,
    GetNote,
    ListNotes,
    GetBacklinks,
    GetOutboundEdges,
    GetTagFrequencies,
    GetNoteMetadata,
    ListNoteIds,
    Traversal,
}

impl QueryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::GetNote => "get_note",
            Self::ListNotes => "list_notes",
            Self::GetBacklinks => "get_backlinks",
            Self::GetOutboundEdges => "get_outbound_edges",
            Self::GetTagFrequencies => "get_tag_frequencies",
            Self::GetNoteMetadata => "get_note_metadata",
            Self::ListNoteIds => "list_note_ids",
            Self::Traversal => "traversal",
        }
    }
}

/// Result count buckets for query statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResultCountBucket {
    Zero,
    One,
    TwoToFive,
    SixToTwenty,
    TwentyOneTo100,
    MoreThan100,
}

impl ResultCountBucket {
    pub fn from_count(count: usize) -> Self {
        match count {
            0 => Self::Zero,
            1 => Self::One,
            2..=5 => Self::TwoToFive,
            6..=20 => Self::SixToTwenty,
            21..=100 => Self::TwentyOneTo100,
            _ => Self::MoreThan100,
        }
    }
}

/// Event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "kebab-case")]
pub enum TelemetryEvent {
    CommandExecuted {
        timestamp: i64,
        command: CommandName,
        success: bool,
        duration: DurationBucket,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<ErrorType>,
    },
    SessionStats {
        timestamp: i64,
        os_platform: String,
        app_version: String,
        workspace_count: WorkspaceCountBucket,
        note_count: NoteCountBucket,
    },
    QueryStats {
        timestamp: i64,
        query_type: QueryType,
        duration: DurationBucket,
        result_count: ResultCountBucket,
        success: bool,
    },
}

impl TelemetryEvent {
    pub fn timestamp(&self) -> i64 {
        match self {
            Self::CommandExecuted { timestamp, .. } => *timestamp,
            Self::SessionStats { timestamp, .. } => *timestamp,
            Self::QueryStats { timestamp, .. } => *timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_buckets() {
        assert_eq!(
            DurationBucket::from_millis(50),
            DurationBucket::LessThan100ms
        );
        assert_eq!(DurationBucket::from_millis(100), DurationBucket::Ms100To500);
        assert_eq!(DurationBucket::from_millis(500), DurationBucket::Ms500To1s);
        assert_eq!(DurationBucket::from_millis(1000), DurationBucket::S1To5);
        assert_eq!(
            DurationBucket::from_millis(10000),
            DurationBucket::MoreThan5s
        );
    }

    #[test]
    fn test_workspace_count_buckets() {
        assert_eq!(
            WorkspaceCountBucket::from_count(0),
            WorkspaceCountBucket::Zero
        );
        assert_eq!(
            WorkspaceCountBucket::from_count(1),
            WorkspaceCountBucket::One
        );
        assert_eq!(
            WorkspaceCountBucket::from_count(3),
            WorkspaceCountBucket::TwoToFive
        );
        assert_eq!(
            WorkspaceCountBucket::from_count(10),
            WorkspaceCountBucket::SixToTwenty
        );
        assert_eq!(
            WorkspaceCountBucket::from_count(100),
            WorkspaceCountBucket::MoreThanTwenty
        );
    }

    #[test]
    fn test_note_count_buckets() {
        assert_eq!(NoteCountBucket::from_count(5), NoteCountBucket::LessThan10);
        assert_eq!(NoteCountBucket::from_count(50), NoteCountBucket::TenTo100);
        assert_eq!(
            NoteCountBucket::from_count(500),
            NoteCountBucket::HundredTo1000
        );
        assert_eq!(
            NoteCountBucket::from_count(5000),
            NoteCountBucket::ThousandTo10000
        );
        assert_eq!(
            NoteCountBucket::from_count(50000),
            NoteCountBucket::MoreThan10000
        );
    }

    #[test]
    fn test_command_name_display() {
        assert_eq!(CommandName::Capture.as_str(), "capture");
        assert_eq!(CommandName::Search.as_str(), "search");
    }

    #[test]
    fn test_event_serialization_no_pii() {
        let event = TelemetryEvent::CommandExecuted {
            timestamp: 1234567890,
            command: CommandName::Capture,
            success: true,
            duration: DurationBucket::Ms100To500,
            error: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.contains("filename"));
        assert!(!json.contains("path"));
        assert!(!json.contains("user"));
        assert!(!json.contains("message"));
    }

    #[test]
    fn test_result_count_buckets() {
        assert_eq!(ResultCountBucket::from_count(0), ResultCountBucket::Zero);
        assert_eq!(ResultCountBucket::from_count(1), ResultCountBucket::One);
        assert_eq!(
            ResultCountBucket::from_count(3),
            ResultCountBucket::TwoToFive
        );
        assert_eq!(
            ResultCountBucket::from_count(10),
            ResultCountBucket::SixToTwenty
        );
        assert_eq!(
            ResultCountBucket::from_count(50),
            ResultCountBucket::TwentyOneTo100
        );
        assert_eq!(
            ResultCountBucket::from_count(200),
            ResultCountBucket::MoreThan100
        );
    }

    #[test]
    fn test_query_type_display() {
        assert_eq!(QueryType::Search.as_str(), "search");
        assert_eq!(QueryType::GetNote.as_str(), "get_note");
        assert_eq!(QueryType::ListNotes.as_str(), "list_notes");
    }

    #[test]
    fn test_query_stats_event_serialization() {
        let event = TelemetryEvent::QueryStats {
            timestamp: 1234567890,
            query_type: QueryType::Search,
            duration: DurationBucket::Ms100To500,
            result_count: ResultCountBucket::TwentyOneTo100,
            success: true,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("query-stats"));
        assert!(json.contains("search"));
        assert!(json.contains("1234567890"));
        assert!(json.contains("ms100-to500"));
        assert!(json.contains("twenty-one-to100"));
    }
}
