//! Telemetry endpoint configuration and HTTP client
//!
//! This module provides the infrastructure for uploading telemetry data
//! to a remote endpoint. It includes:
//! - Endpoint URL configuration
//! - HTTP client with privacy-compliant headers
//! - Retry logic with exponential backoff
//! - Privacy-first request formatting

use std::time::Duration;

use super::aggregation::UploadBatch;
use super::uploader::UploadError;

/// Default endpoint URL (placeholder - to be configured when endpoint is available)
pub const DEFAULT_ENDPOINT_URL: &str = "";

/// Default timeout for upload requests
pub const DEFAULT_UPLOAD_TIMEOUT_SECONDS: u64 = 30;

/// Maximum number of retry attempts
pub const MAX_RETRY_ATTEMPTS: u32 = 3;

/// Configuration for telemetry endpoint
#[derive(Debug, Clone)]
pub struct EndpointConfig {
    /// URL of the telemetry endpoint (empty means disabled)
    pub url: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retry attempts for failed uploads
    pub max_retries: u32,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            url: Self::get_endpoint_url(),
            timeout_seconds: DEFAULT_UPLOAD_TIMEOUT_SECONDS,
            max_retries: MAX_RETRY_ATTEMPTS,
        }
    }
}

impl EndpointConfig {
    /// Get endpoint URL from environment or use default (empty = disabled)
    pub fn get_endpoint_url() -> String {
        std::env::var("QIPU_TELEMETRY_ENDPOINT")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_ENDPOINT_URL.to_string())
    }

    /// Check if endpoint is configured and enabled
    pub fn is_configured(&self) -> bool {
        !self.url.is_empty()
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(url) = std::env::var("QIPU_TELEMETRY_ENDPOINT") {
            if !url.is_empty() {
                config.url = url;
            }
        }

        if let Ok(timeout) = std::env::var("QIPU_TELEMETRY_TIMEOUT") {
            if let Ok(seconds) = timeout.parse::<u64>() {
                config.timeout_seconds = seconds.clamp(5, 300);
            }
        }

        if let Ok(retries) = std::env::var("QIPU_TELEMETRY_RETRIES") {
            if let Ok(count) = retries.parse::<u32>() {
                config.max_retries = count.clamp(0, 10);
            }
        }

        config
    }
}

/// HTTP client for telemetry uploads
pub struct EndpointClient {
    pub config: EndpointConfig,
    user_agent: String,
}

impl EndpointClient {
    /// Create a new endpoint client with the given configuration
    pub fn new(config: EndpointConfig) -> Self {
        let app_version = super::get_app_version();
        let os_platform = std::env::consts::OS;
        let user_agent = format!("qipu/{} ({})", app_version, os_platform);

        Self { config, user_agent }
    }

    /// Upload telemetry batch to the configured endpoint
    ///
    /// This method implements privacy-first upload:
    /// - No identifying headers (no session cookies, no device IDs)
    /// - Minimal User-Agent (only version and platform)
    /// - No retry on 4xx errors (avoids leaking data to wrong endpoints)
    /// - Aggregated data only (individual events never sent)
    pub fn upload_batch(&self, batch: &UploadBatch) -> Result<(), UploadError> {
        if !self.config.is_configured() {
            return Err(UploadError::EndpointUnavailable);
        }

        let json_payload = serde_json::to_string(batch)
            .map_err(|e| UploadError::SerializationError(e.to_string()))?;

        let timeout = Duration::from_secs(self.config.timeout_seconds);

        let response = ureq::post(&self.config.url)
            .set("Content-Type", "application/json")
            .set("User-Agent", &self.user_agent)
            .set("X-Qipu-Telemetry-Version", "1.0")
            .timeout(timeout)
            .send_string(&json_payload);

        match response {
            Ok(res) => {
                let status = res.status();
                if (200..300).contains(&status) {
                    Ok(())
                } else if (400..500).contains(&status) {
                    Err(UploadError::EndpointUnavailable)
                } else {
                    Err(UploadError::NetworkError(format!(
                        "Server returned status {}",
                        status
                    )))
                }
            }
            Err(ureq::Error::Transport(e)) => {
                Err(UploadError::NetworkError(format!("Transport error: {}", e)))
            }
            Err(ureq::Error::Status(code, _)) => {
                if (400..500).contains(&code) {
                    Err(UploadError::EndpointUnavailable)
                } else {
                    Err(UploadError::NetworkError(format!("HTTP {}", code)))
                }
            }
        }
    }

    /// Upload with retry logic using exponential backoff
    ///
    /// Retries only on 5xx errors and network failures, never on 4xx
    /// (to avoid sending data to wrong endpoints)
    pub fn upload_with_retry(&self, batch: &UploadBatch) -> Result<(), UploadError> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let backoff = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                std::thread::sleep(backoff);
            }

            match self.upload_batch(batch) {
                Ok(()) => return Ok(()),
                Err(e @ UploadError::EndpointUnavailable) => {
                    return Err(e);
                }
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or(UploadError::EndpointUnavailable))
    }
}

impl Default for EndpointClient {
    fn default() -> Self {
        Self::new(EndpointConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_config_default_disabled() {
        let config = EndpointConfig::default();
        assert!(config.url.is_empty());
        assert!(!config.is_configured());
        assert_eq!(config.timeout_seconds, DEFAULT_UPLOAD_TIMEOUT_SECONDS);
        assert_eq!(config.max_retries, MAX_RETRY_ATTEMPTS);
    }

    #[test]
    fn test_endpoint_config_from_env_url() {
        std::env::set_var("QIPU_TELEMETRY_ENDPOINT", "https://example.com/telemetry");
        let config = EndpointConfig::from_env();
        assert_eq!(config.url, "https://example.com/telemetry");
        assert!(config.is_configured());
        std::env::remove_var("QIPU_TELEMETRY_ENDPOINT");
    }

    #[test]
    fn test_endpoint_config_timeout_parsing() {
        std::env::set_var("QIPU_TELEMETRY_TIMEOUT", "60");
        let config = EndpointConfig::from_env();
        assert_eq!(config.timeout_seconds, 60);
        std::env::remove_var("QIPU_TELEMETRY_TIMEOUT");
    }

    #[test]
    fn test_endpoint_config_timeout_clamping() {
        std::env::set_var("QIPU_TELEMETRY_TIMEOUT", "1");
        let config = EndpointConfig::from_env();
        assert_eq!(config.timeout_seconds, 5);

        std::env::set_var("QIPU_TELEMETRY_TIMEOUT", "1000");
        let config = EndpointConfig::from_env();
        assert_eq!(config.timeout_seconds, 300);

        std::env::remove_var("QIPU_TELEMETRY_TIMEOUT");
    }

    #[test]
    fn test_endpoint_config_retry_parsing() {
        std::env::set_var("QIPU_TELEMETRY_RETRIES", "5");
        let config = EndpointConfig::from_env();
        assert_eq!(config.max_retries, 5);
        std::env::remove_var("QIPU_TELEMETRY_RETRIES");
    }

    #[test]
    fn test_endpoint_config_retry_clamping() {
        std::env::set_var("QIPU_TELEMETRY_RETRIES", "100");
        let config = EndpointConfig::from_env();
        assert_eq!(config.max_retries, 10);

        std::env::remove_var("QIPU_TELEMETRY_RETRIES");
    }

    #[test]
    fn test_endpoint_client_user_agent() {
        let config = EndpointConfig::default();
        let client = EndpointClient::new(config);
        assert!(client.user_agent.contains("qipu/"));
        assert!(!client.user_agent.contains("session"));
        assert!(!client.user_agent.contains("id"));
    }

    #[test]
    fn test_endpoint_client_upload_unconfigured() {
        let config = EndpointConfig::default();
        let client = EndpointClient::new(config);
        let batch = UploadBatch {
            sessions: vec![],
            metadata: super::super::aggregation::UploadMetadata {
                batch_id: "test".to_string(),
                batch_date: "2024-01-01".to_string(),
                session_count: 0,
            },
        };

        let result = client.upload_batch(&batch);
        assert!(matches!(result, Err(UploadError::EndpointUnavailable)));
    }
}
