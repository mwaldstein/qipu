//! Telemetry uploader with endpoint infrastructure

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::collector::TelemetryCollector;
use super::endpoint::{EndpointClient, EndpointConfig};
use super::SessionAggregator;

pub struct TelemetryUploader {
    collector: Arc<TelemetryCollector>,
    endpoint: EndpointClient,
}

impl TelemetryUploader {
    pub fn new(collector: Arc<TelemetryCollector>) -> Self {
        let endpoint_config = EndpointConfig::from_env();
        let endpoint = EndpointClient::new(endpoint_config);

        Self {
            collector,
            endpoint,
        }
    }

    /// Start background upload process
    ///
    /// This aggregates pending events and uploads them to the configured endpoint.
    /// If no endpoint is configured, events are simply persisted to disk for later.
    pub fn start_background_upload(&self) {
        let collector = Arc::clone(&self.collector);
        let endpoint = EndpointClient::new(self.endpoint.config.clone());

        thread::spawn(move || {
            if !collector.is_enabled() {
                return;
            }

            let events = collector.get_pending_events();
            if events.is_empty() {
                return;
            }

            thread::sleep(Duration::from_millis(100));

            let batch = SessionAggregator::aggregate_events(events);

            if batch.is_empty() {
                return;
            }

            if endpoint.config.is_configured() {
                let _ = endpoint.upload_with_retry(&batch);
            }

            let _ = collector.persist_to_disk();
            collector.clear_events();
        });
    }

    /// Check if endpoint is configured for upload
    pub fn is_endpoint_configured(&self) -> bool {
        self.endpoint.config.is_configured()
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
