# Structured Logging

**Status**: ❌ NOT STARTED  
**Spec**: ✅ COMPLETE  
**Impl**: ❌ NOT STARTED  
**Tests**: ❌ NOT STARTED  
**Last audited**: 2026-01-17

## Overview

Replace the current primitive logging system with a structured logging framework to improve observability, debugging capabilities, and operational visibility.

## Current State

The current logging infrastructure in `src/lib/logging.rs` is minimal:

```rust
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(enabled: bool) {
    VERBOSE.store(enabled, Ordering::Relaxed);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}
```

This provides only boolean verbosity control with ad-hoc `eprintln!` statements throughout the codebase.

## Requirements

### Logging Framework

- **Framework**: Use the `tracing` crate ecosystem for structured logging
- **Levels**: Support standard log levels: ERROR, WARN, INFO, DEBUG, TRACE
- **Structured data**: Support key-value pairs for searchable log data
- **Performance**: Zero-cost when disabled, minimal overhead when enabled

### Log Configuration

- **CLI integration**: Extend existing `--verbose` flag with level control
- **Environment variables**: Support `QIPU_LOG` environment variable
- **Default behavior**: Maintain current quiet-by-default behavior
- **Output format**: Human-readable by default, JSON option for machine consumption

### Logging Categories

#### Core Operations
- **Store operations**: Discovery, initialization, validation
- **Note operations**: Creation, parsing, indexing
- **Search operations**: Query processing, ranking, result filtering
- **Graph operations**: Traversal, link resolution, compaction

#### Performance Tracing
- **Timing spans**: Major operation timing with structured context
- **Resource usage**: Memory allocation patterns, cache hit rates
- **Index operations**: Build time, cache operations, search performance

#### Error Context
- **Error chains**: Structured error context with operation traces
- **Recovery actions**: Log automatic recovery attempts
- **Validation failures**: Detailed context for data validation errors

## Implementation Specification

### Dependencies

Add to `Cargo.toml`:

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"  # Optional: for file output
```

### CLI Integration

Extend existing verbosity options:

```rust
#[derive(Parser)]
pub struct GlobalCli {
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Set log level (error, warn, info, debug, trace)
    #[arg(long, value_name = "LEVEL")]
    pub log_level: Option<String>,
    
    /// Output logs in JSON format
    #[arg(long)]
    pub log_json: bool,
}
```

### Structured Spans

```rust
// Store operations
#[tracing::instrument(skip(path), fields(store_path = %path.display()))]
pub fn discover_store(path: &Path) -> Result<Store> {
    // Implementation
}

// Search operations
#[tracing::instrument(skip(index), fields(query = %query, limit = limit))]
pub fn search(index: &Index, query: &str, limit: usize) -> SearchResults {
    // Implementation
}

// Performance-critical operations
let _span = tracing::info_span!("index_build", 
    note_count = notes.len(),
    incremental = is_incremental
).entered();
```

### Error Context Enhancement

```rust
use tracing::{error, warn, info, debug};

// Enhanced error reporting
if let Err(e) = parse_note(&path) {
    error!(
        error = %e,
        note_path = %path.display(),
        "Failed to parse note frontmatter"
    );
    // Continue processing other notes
}
```

### Configuration System

```rust
pub fn init_logging(cli: &GlobalCli) -> Result<()> {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};
    
    let level = match (cli.verbose, &cli.log_level) {
        (true, None) => "qipu=debug",
        (false, None) => "qipu=warn", 
        (_, Some(level)) => &format!("qipu={}", level),
    };
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    let subscriber = tracing_subscriber::registry()
        .with(filter);
    
    if cli.log_json {
        subscriber.with(fmt::layer().json()).try_init()?;
    } else {
        subscriber.with(fmt::layer().compact()).try_init()?;
    }
    
    Ok(())
}
```

## Performance Requirements

- **Zero overhead**: No logging impact when disabled
- **Minimal allocation**: Structured logging should not significantly impact memory usage
- **Async-safe**: Compatible with future async operations

## Migration Strategy

### Phase 1: Infrastructure Setup
1. Add tracing dependencies
2. Initialize logging system in `main.rs`
3. Update CLI argument parsing

### Phase 2: Core Operations
1. Add spans to major operations (store, search, index)
2. Replace existing `eprintln!` statements with structured logging
3. Add performance tracing for timing-sensitive operations

### Phase 3: Enhanced Observability  
1. Add detailed error context
2. Implement operation tracing for complex workflows
3. Add optional file output and log rotation

### Phase 4: Testing and Validation
1. Update tests to handle new logging output
2. Verify performance impact is minimal
3. Test different log levels and output formats

## Compatibility

- **Backward compatibility**: Existing `--verbose` flag behavior preserved
- **Default behavior**: Silent operation by default (same as current)
- **Error output**: Error messages continue to stderr, with optional structured enhancement

## Success Criteria

- [ ] Structured logging framework integrated with zero performance impact when disabled
- [ ] All major operations instrumented with appropriate spans and context
- [ ] CLI provides granular control over logging verbosity and format
- [ ] Error messages enhanced with structured context while maintaining readability
- [ ] Performance tracing available for debugging without impacting normal operation
- [ ] Tests updated to handle structured logging output appropriately