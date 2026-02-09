//! Telemetry collection and upload for anonymous usage analytics

mod aggregation;
mod collector;
mod endpoint;
mod events;
mod privacy_manifest;
mod uploader;

pub use aggregation::{
    AggregatedSession, CommandStats, SessionAggregator, UploadBatch, UploadMetadata,
};
pub use collector::{TelemetryCollector, TelemetryConfig};
pub use endpoint::{EndpointClient, EndpointConfig};
pub use events::{
    CommandName, DurationBucket, ErrorType, NoteCountBucket, QueryType, ResultCountBucket,
    TelemetryEvent, WorkspaceCountBucket,
};
pub use privacy_manifest::{PrivacyManifest, PRIVACY_MANIFEST};
pub use uploader::{TelemetryUploader, UploadError};

use std::sync::Arc;
use std::time::Instant;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init_telemetry() -> Arc<TelemetryCollector> {
    let config = TelemetryConfig::default();
    let collector = Arc::new(TelemetryCollector::new(config.clone()));

    if config.enabled {
        collector.rotate_events().ok();
    }

    collector
}

pub fn record_command_execution(
    collector: &Arc<TelemetryCollector>,
    command: CommandName,
    result: &Result<(), crate::error::QipuError>,
    start: Instant,
) {
    let duration_ms = start.elapsed().as_millis();
    let success = result.is_ok();
    let error_type = result.as_ref().err().map(error_to_type);

    collector.record_command(command, success, duration_ms, error_type);

    let uploader = TelemetryUploader::new(Arc::clone(collector));
    uploader.start_background_upload();
}

fn error_to_type(error: &crate::error::QipuError) -> ErrorType {
    use crate::error::QipuError;

    match error {
        QipuError::UsageError(_) | QipuError::UnknownFormat(_) | QipuError::DuplicateFormat => {
            ErrorType::UsageError
        }
        QipuError::StoreNotFound { .. }
        | QipuError::InvalidStore { .. }
        | QipuError::NoteNotFound { .. } => ErrorType::StoreError,
        QipuError::InvalidFrontmatter { .. } => ErrorType::ParseError,
        QipuError::Io(_) => ErrorType::IOError,
        QipuError::Yaml(_) | QipuError::Json(_) | QipuError::Toml(_) => ErrorType::ParseError,
        _ => ErrorType::Other,
    }
}

pub fn get_app_version() -> &'static str {
    APP_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_telemetry_disabled() {
        std::env::set_var("QIPU_NO_TELEMETRY", "1");
        let collector = init_telemetry();
        assert!(!collector.is_enabled());
        std::env::remove_var("QIPU_NO_TELEMETRY");
    }

    #[test]
    fn test_error_to_type() {
        use crate::error::QipuError;

        let err = QipuError::UsageError("test".to_string());
        assert_eq!(error_to_type(&err), ErrorType::UsageError);

        let err = QipuError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert_eq!(error_to_type(&err), ErrorType::IOError);
    }

    #[test]
    fn test_get_app_version() {
        let version = get_app_version();
        assert!(!version.is_empty());
    }
}
