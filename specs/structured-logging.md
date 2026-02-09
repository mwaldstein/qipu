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
- **Default behavior**: Warn level by default (shows errors and warnings, hides info/debug/trace)
- **Output format**: Human-readable by default, JSON option for machine consumption

### CLI Flags

```
--verbose           Enable verbose output (shortcut for debug level)
--log-level LEVEL   Set log level (error, warn, info, debug, trace)
--log-json          Output logs in JSON format
```

### Environment Variable

`QIPU_LOG` - Set log level via environment (e.g., `QIPU_LOG=debug`)

## Instrumentation Plan

### Tier 1: Operations (Always Instrumented)
These functions are annotated with `#[tracing::instrument]` and provide entry point visibility:

**Store Lifecycle:**
- `Store::discover()` - Store discovery with path context
- `Store::init()` - Store initialization with migration tracking
- `NoteStore::create_fleeting_note()` - Note creation with title/type/id fields
- `NoteStore::create_literature_note()` - Literature note creation
- `NoteStore::update_note()` - Note update operations

**Indexing Operations:**
- `IndexBuilder::build()` - Full index build with store root
- `index_note_links()` - Per-note link indexing with note_id
- `Indexer::rebuild()` / `resume()` - Database rebuild/resume with checkpoint tracking

**Graph Operations:**
- `find_path_bfs()` - Path finding with from/to/direction/max_hops
- `bfs_traversal()` / `dijkstra_traversal()` - Graph traversal with root/direction/cost parameters

**Search Operations:**
- `process_search_results()` - Result processing with results_count and sort parameters

**Parsing:**
- `Note::from_bytes()` - Note parsing with path context
- `parse_note()` - Low-level parse with path context

**Query Operations:**
- `Store::get_note()` - Note retrieval with note_id

**Doctor/Validation:**
- `DoctorCommand::execute()` - Doctor execution with store_root and check parameters
- `AutoFixer::execute()` - Auto-fix operations
- `DbValidator::validate()` - Database validation

### Tier 2: Timing Points (Debug/Trace Level)
Functions instrumented with manual timing logs at strategic points:

**Resource Metrics (via `log_resource_metrics!` macro):**
- Search result processing - memory/cache statistics

**Timing Spans (via `trace_time!` macro or manual `tracing::debug!`):**
- CLI argument parsing
- Index loading operations
- Context command completion
- Export/dump operations
- Link operations (path, tree, list)
- Store operations with timing context

### Tier 3: Event Logging (Info/Warn/Error Level)
Structured logging for operational events:

**Info Level:**
- Database rebuild start/completion
- Migration events (JSON cache → SQLite)
- Checkpoint commits during indexing
- Telemetry events (enable/disable/pending/upload)

**Warn Level:**
- Failed note parsing (with path and error)
- Database inconsistencies triggering repair
- Compaction validation errors
- Workspace deletion with unmerged changes

**Error Level:**
- Database validation failures
- Doctor check failures
- Critical operational errors

### Tier 4: Internal Tracing (Trace Level)
Fine-grained tracing for deep debugging:
- Individual cache operations
- Memory allocation patterns
- Individual traversal steps
- Parse token-level details

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
- **Default behavior**: Warn level by default (shows errors and warnings)
- **Error output**: Error messages continue to stderr, with optional structured enhancement

## Success Criteria

- [ ] Structured logging integrated with zero performance impact when disabled
- [ ] All major operations instrumented with appropriate spans and context
- [ ] CLI provides granular control over logging verbosity and format
- [ ] Error messages enhanced with structured context while maintaining readability
- [ ] Performance tracing available for debugging without impacting normal operation
