use std::sync::atomic::{AtomicU64, Ordering};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Resource metrics for structured logging.
///
/// Tracks memory allocation and cache hit/miss statistics for performance analysis.
/// All operations are atomic and lock-free for minimal overhead.
#[derive(Debug, Default)]
pub struct ResourceMetrics {
    /// Total memory allocated in bytes (approximate)
    memory_allocated: AtomicU64,
    /// Cache hit count
    cache_hits: AtomicU64,
    /// Cache miss count
    cache_misses: AtomicU64,
}

impl ResourceMetrics {
    /// Create a new ResourceMetrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a memory allocation
    pub fn record_allocation(&self, bytes: u64) {
        self.memory_allocated.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current memory allocated in bytes
    pub fn memory_allocated(&self) -> u64 {
        self.memory_allocated.load(Ordering::Relaxed)
    }

    /// Get total cache lookups (hits + misses)
    pub fn total_cache_lookups(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed) + self.cache_misses.load(Ordering::Relaxed)
    }

    /// Get cache hit count
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    /// Get cache miss count
    pub fn cache_misses(&self) -> u64 {
        self.cache_misses.load(Ordering::Relaxed)
    }

    /// Get cache hit rate as a percentage (0.0-100.0)
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.memory_allocated.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
    }
}

/// Log resource metrics at debug level.
///
/// Usage:
/// ```rust,ignore
/// let metrics = ResourceMetrics::new();
/// // ... record some metrics ...
/// log_resource_metrics!(&metrics, "operation_name");
/// ```
#[macro_export]
macro_rules! log_resource_metrics {
    ($metrics:expr, $name:expr) => {
        tracing::debug!(
            operation = $name,
            memory_allocated = $metrics.memory_allocated(),
            cache_hits = $metrics.cache_hits(),
            cache_misses = $metrics.cache_misses(),
            cache_hit_rate = $metrics.cache_hit_rate(),
            "resource_metrics"
        );
    };
}

/// Helper macro for logging elapsed time at trace level.
///
/// Usage:
/// ```rust,ignore
/// let start = Instant::now();
/// // ... some work ...
/// trace_time!(start, "operation_name");
/// // Or with additional fields:
/// trace_time!(start, "operation_name", note_id = note.id());
/// ```
#[macro_export]
macro_rules! trace_time {
    ($start:expr, $name:expr) => {
        tracing::trace!(elapsed = ?$start.elapsed(), $name);
    };
    ($start:expr, $name:expr $(, $field:ident = $value:expr)*) => {
        tracing::trace!(elapsed = ?$start.elapsed(), $($field = $value),*, $name);
    };
}

/// Initialize structured logging based on CLI arguments
pub fn init_tracing(
    verbose: bool,
    log_level: Option<&str>,
    log_json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine log level from CLI arguments
    let level = match (verbose, log_level) {
        (true, None) => "qipu=debug",
        (false, None) => "qipu=warn",
        (_, Some(level)) => return init_with_level(level, log_json),
    };

    init_with_level(level, log_json)
}

fn init_with_level(level: &str, log_json: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Support QIPU_LOG environment variable override
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_from_env("QIPU_LOG"))
        .unwrap_or_else(|_| {
            EnvFilter::new(if level.contains('=') {
                level.to_string()
            } else {
                format!("qipu={}", level)
            })
        });

    let registry = tracing_subscriber::registry().with(filter);

    if log_json {
        registry
            .with(
                fmt::layer()
                    .json()
                    .with_writer(std::io::stderr)
                    .with_ansi(false)
                    .with_span_events(
                        tracing_subscriber::fmt::format::FmtSpan::NEW
                            | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
                    ),
            )
            .try_init()?;
    } else {
        registry
            .with(
                fmt::layer()
                    .compact()
                    .with_target(false)
                    .with_writer(std::io::stderr)
                    .with_ansi(false),
            )
            .try_init()?;
    }

    Ok(())
}
