# Structured Logging

## Overview

Replace the primitive logging system with structured logging to improve observability, debugging capabilities, and operational visibility.

## Requirements

### Logging Levels
Support standard log levels: ERROR, WARN, INFO, DEBUG, TRACE

### Structured Data
Logs should support key-value pairs for searchable log data.

### Performance
Zero-cost when disabled, minimal overhead when enabled.

### Log Configuration

- **CLI integration**: Extend existing `--verbose` flag with level control
- **Environment variables**: Support `QIPU_LOG` environment variable
- **Default behavior**: Maintain current quiet-by-default behavior
- **Output format**: Human-readable by default, JSON option for machine consumption

### CLI Flags

```
--verbose           Enable verbose output (shortcut for debug level)
--log-level LEVEL   Set log level (error, warn, info, debug, trace)
--log-json          Output logs in JSON format
```

### Environment Variable

`QIPU_LOG` - Set log level via environment (e.g., `QIPU_LOG=debug`)

## Logging Categories

### Core Operations
- **Store operations**: Discovery, initialization, validation
- **Note operations**: Creation, parsing, indexing
- **Search operations**: Query processing, ranking, result filtering
- **Graph operations**: Traversal, link resolution, compaction

### Performance Tracing
- **Timing spans**: Major operation timing with structured context
- **Resource usage**: Memory allocation patterns, cache hit rates
- **Index operations**: Build time, cache operations, search performance

### Error Context
- **Error chains**: Structured error context with operation traces
- **Recovery actions**: Log automatic recovery attempts
- **Validation failures**: Detailed context for data validation errors

## Compatibility

- **Backward compatibility**: Existing `--verbose` flag behavior preserved
- **Default behavior**: Silent operation by default (same as current)
- **Error output**: Error messages continue to stderr, with optional structured enhancement

## Success Criteria

- [ ] Structured logging integrated with zero performance impact when disabled
- [ ] All major operations instrumented with appropriate spans and context
- [ ] CLI provides granular control over logging verbosity and format
- [ ] Error messages enhanced with structured context while maintaining readability
- [ ] Performance tracing available for debugging without impacting normal operation
