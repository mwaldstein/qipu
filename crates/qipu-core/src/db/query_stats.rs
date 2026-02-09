//! Query statistics tracking for database operations
//!
//! Provides instrumentation for database queries to collect performance metrics.

use crate::telemetry::{QueryType, TelemetryCollector};
use std::sync::Arc;
use std::time::Instant;

/// Tracks timing and results for a database query
pub struct QueryTimer {
    start: Instant,
    query_type: QueryType,
    telemetry: Option<Arc<TelemetryCollector>>,
}

impl QueryTimer {
    /// Start timing a query with optional telemetry
    pub fn new(query_type: QueryType, telemetry: Option<Arc<TelemetryCollector>>) -> Self {
        Self {
            start: Instant::now(),
            query_type,
            telemetry,
        }
    }

    /// Record successful query completion with result count
    pub fn record_success(self, result_count: usize) {
        if let Some(telemetry) = self.telemetry {
            let duration_ms = self.start.elapsed().as_millis();
            telemetry.record_query(self.query_type, duration_ms, result_count, true);
        }
    }

    /// Record failed query execution
    pub fn record_failure(self) {
        if let Some(telemetry) = self.telemetry {
            let duration_ms = self.start.elapsed().as_millis();
            telemetry.record_query(self.query_type, duration_ms, 0, false);
        }
    }

    /// Get the elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    /// Get the query type
    pub fn query_type(&self) -> QueryType {
        self.query_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{DurationBucket, ResultCountBucket, TelemetryConfig};
    use std::path::PathBuf;
    use std::thread;

    #[test]
    fn test_query_timer_success() {
        let config = TelemetryConfig {
            enabled: true,
            events_dir: PathBuf::from("/tmp/test_telemetry"),
        };
        let telemetry = Arc::new(TelemetryCollector::new(config));

        let timer = QueryTimer::new(QueryType::Search, Some(telemetry.clone()));
        thread::sleep(std::time::Duration::from_millis(10));
        timer.record_success(5);

        let events = telemetry.get_pending_events();
        assert_eq!(events.len(), 1);

        match &events[0] {
            crate::telemetry::TelemetryEvent::QueryStats {
                query_type,
                result_count,
                success,
                ..
            } => {
                assert_eq!(*query_type, QueryType::Search);
                assert_eq!(*result_count, ResultCountBucket::TwoToFive);
                assert!(*success);
            }
            _ => panic!("Expected QueryStats event"),
        }
    }

    #[test]
    fn test_query_timer_failure() {
        let config = TelemetryConfig {
            enabled: true,
            events_dir: PathBuf::from("/tmp/test_telemetry"),
        };
        let telemetry = Arc::new(TelemetryCollector::new(config));

        let timer = QueryTimer::new(QueryType::GetNote, Some(telemetry.clone()));
        timer.record_failure();

        let events = telemetry.get_pending_events();
        assert_eq!(events.len(), 1);

        match &events[0] {
            crate::telemetry::TelemetryEvent::QueryStats {
                query_type,
                result_count,
                success,
                ..
            } => {
                assert_eq!(*query_type, QueryType::GetNote);
                assert_eq!(*result_count, ResultCountBucket::Zero);
                assert!(!*success);
            }
            _ => panic!("Expected QueryStats event"),
        }
    }

    #[test]
    fn test_query_timer_no_telemetry() {
        let timer = QueryTimer::new(QueryType::ListNotes, None);
        thread::sleep(std::time::Duration::from_millis(5));
        timer.record_success(10);
        // Should not panic or fail
    }
}
