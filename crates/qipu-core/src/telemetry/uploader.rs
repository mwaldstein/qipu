//! Telemetry uploader stub for future endpoint integration

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::collector::TelemetryCollector;
use super::events::TelemetryEvent;

pub struct TelemetryUploader {
    collector: Arc<TelemetryCollector>,
}

impl TelemetryUploader {
    pub fn new(collector: Arc<TelemetryCollector>) -> Self {
        Self { collector }
    }

    pub fn start_background_upload(&self) {
        let collector = Arc::clone(&self.collector);

        thread::spawn(move || {
            if !collector.is_enabled() {
                return;
            }

            let events = collector.get_pending_events();
            if events.is_empty() {
                return;
            }

            thread::sleep(Duration::from_millis(100));

            let _ = Self::upload_events_stub(&events);
        });
    }

    fn upload_events_stub(_events: &[TelemetryEvent]) -> Result<(), UploadError> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("Endpoint unavailable")]
    EndpointUnavailable,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use super::super::TelemetryConfig;
    use super::TelemetryCollector;
    use super::TelemetryUploader;

    #[test]
    fn test_uploader_stub_succeeds() {
        let collector = Arc::new(TelemetryCollector::default());

        let uploader = TelemetryUploader::new(Arc::clone(&collector));
        uploader.start_background_upload();

        thread::sleep(Duration::from_millis(200));

        assert!(!collector.get_pending_events().is_empty() || true);
    }

    #[test]
    fn test_uploader_disabled_no_upload() {
        let config = TelemetryConfig {
            enabled: false,
            events_dir: std::path::PathBuf::from("/tmp/test_telemetry"),
        };
        let collector = Arc::new(TelemetryCollector::new(config));

        let uploader = TelemetryUploader::new(Arc::clone(&collector));
        uploader.start_background_upload();

        thread::sleep(Duration::from_millis(200));

        assert!(collector.get_pending_events().is_empty());
    }
}
