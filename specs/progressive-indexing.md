# Progressive Indexing and Re-Indexing

## Purpose

For large knowledge bases (10k+ notes), full indexing can take minutes or longer. Since qipu stores raw notes in git, full re-indexing from raw files is a common operation (e.g., after cloning a repo or after significant changes).

This spec defines strategies for progressive indexing, user feedback during indexing, and selective indexing options to handle the cost of indexing large knowledge bases.

## Goals

- Enable efficient indexing of large knowledge bases (10k+ notes)
- Provide clear progress feedback during indexing operations
- Support selective/critical-only indexing when full index isn't needed
- Allow interruption and resumption of long-running indexing
- Minimize the cost of common re-indexing scenarios

## Indexing Strategies

### 1. Incremental Indexing (mtime-based)

Only re-index notes that have been modified since the last index update.

**Implementation:**
- Store `mtime` of each indexed note in database
- Before indexing, compare file `mtime` vs database `mtime`
- Skip unchanged notes, only process modified/new files
- Use this for `qipu index` command (default behavior)

**Use Cases:**
- After pulling git updates (only changed notes need re-indexing)
- After adding/editing notes locally
- Periodic sync with remote stores

**Cost:**
- O(changed notes) vs O(total notes)
- Git mtime comparison is fast; only file reads for changed notes

### 2. Background Indexing with Progress

Run indexing as a background process with real-time progress reporting.

**Implementation:**
- Display progress: "Indexed N / Total notes (XX%)" or "Indexed N notes (X notes/sec)"
- Update progress every N notes (e.g., every 100)
- Show estimated time remaining based on current rate
- Allow cancellation via Ctrl+C (graceful shutdown with partial index saved)

**Progress Output:**
```
Indexing 12,450 notes...
  ████████████░░░░░░░░░ 42% (5,239 / 12,450) ~2m 15s remaining
```

**Use Cases:**
- Initial index of large existing knowledge base
- Full re-index after significant schema changes
- Recovery from corrupted index

### 3. Batched Indexing with Checkpointing

Process notes in batches, saving progress after each batch.

**Implementation:**
- Define batch size (e.g., 1000 notes per batch)
- After each batch: commit changes, save checkpoint
- If interrupted, can resume from last checkpoint
- Track checkpoints in database: `indexing_checkpoint {batch_number, last_note_id}`

**Use Cases:**
- Very large knowledge bases (50k+ notes)
- Unstable environments where interruptions are likely
- Limited resource environments

### 4. Selective/Critical-Only Indexing

Index only a subset of notes based on criteria.

**Selectors:**
- `--tag <tag>`: Index only notes with specified tag
- `--type <type>`: Index only notes of specified type (e.g., `--type permanent`)
- `--moc <id>`: Index MOC and its linked notes
- `--recent <n>`: Index N most recently updated notes
- `--quick`: Index only "critical" notes (MOCs + recent 100)
- `--modified-since <date>`: Index notes modified since timestamp

**Examples:**
```bash
# Quick index: MOCs + 100 recent notes (fast, useful for exploration)
qipu index --quick

# Index only research-related notes
qipu index --tag research

# Index only permanent notes (skip ephemeral)
qipu index --type permanent

# Index notes modified in last 24 hours
qipu index --modified-since "24 hours ago"
```

**Use Cases:**
- Exploratory work (don't need full index)
- Testing queries on subset of knowledge base
- Low-resource environments
- Quick recovery from index corruption

### 5. Lazy/On-Demand Indexing

Defer indexing until a note is actually accessed, then build index gradually.

**Implementation:**
- Maintain "indexed" flag per note (default false)
- When note is accessed (via search, show, context):
  - If not indexed, index it immediately
  - Set indexed flag
- Optional: `qipu index --lazy` to trigger background lazy indexing

**Use Cases:**
- Large knowledge bases where only subset is regularly used
- Development/exploration workspaces
- Cold starts where full index isn't immediately needed

## User Feedback and Status

### Indexing Progress Display

**TUI/Progress Bar:**
```
Indexing notes from .qipu/notes/...
  [████████████░░░░░░░░░░░░] 45% (5,602 / 12,450) 847 notes/sec
  ETA: 8s  Last: qp-x9a2b3 "OAuth 2.1 flow"
```

**Command Line Output:**
```
qipu index --verbose
Scanning for notes... found 12,450 notes
Indexing in progress...
  Batch 1: 1,000 notes (8%)    ✓ 1.2s
  Batch 2: 2,000 notes (16%)   ✓ 1.1s
  Batch 3: 3,000 notes (24%)   ✓ 1.3s
  Batch 4: 4,000 notes (33%)   ✓ 1.0s
  ...
  Batch 13: 12,450 notes (100%) ✓ 0.8s
Indexing complete: 12,450 notes in 14.3s (871 notes/sec)
```

### Index Status Command

Add `qipu index --status` to show current indexing state.

**Output:**
```
Index Status
-----------
Total notes: 12,450
Indexed notes: 12,450 (100%)
Last indexed: 2026-01-24 14:23:45
Database version: 6
Corruption check: Pass
Unindexed changes: 0
```

**Partial/In-Progress Status:**
```
Index Status (IN PROGRESS)
--------------------------
Total notes: 12,450
Indexed notes: 5,602 (45%)
Last indexed note: qp-x9a2b3 "OAuth 2.1 flow"
Indexing rate: 847 notes/sec
Started: 2026-01-24 14:23:15
Estimated completion: 2026-01-24 14:24:30 (~75s remaining)
Checkpoint: Batch 5 (5,000 notes)
```

**Corrupted/Missing Index Status:**
```
Index Status (NEEDS REBUILD)
----------------------------
Total notes: 12,450
Indexed notes: Unknown (corrupted)
Database version: 5 (expected 6)
Corruption check: Fail
Recommended action: Run `qipu index --force` to rebuild
```

## Indexing Modes

### Default Mode: Incremental

```bash
qipu index
```
- Checks mtime for all notes
- Only re-indexes modified notes
- Shows progress for changed notes only
- Fast for small changes

### Full Mode: Force Rebuild

```bash
qipu index --force
```
- Ignores mtime checks
- Re-indexes all notes
- Shows full progress
- Useful after schema upgrades or corruption

### Quick Mode: Critical-Only

```bash
qipu index --quick
```
- Indexes MOCs + 100 most recent notes
- Very fast for large knowledge bases
- Suitable for exploration/work
- Can be combined with selectors: `qipu index --quick --tag research`

### Selective Mode: Filtered

```bash
qipu index --tag research --type permanent --recent 500
```
- Index only notes matching criteria
- Useful for targeted work
- Multiple filters are AND-combined

### Background Mode: Detached

```bash
qipu index --background
```
- Runs in background process
- Writes PID file for monitoring/cancellation
- Logs progress to file or stderr
- Returns immediately with PID

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Incremental index (100 changed notes) | <1s | Most common case |
| Quick index (MOCs + 100 recent) | <1s | Exploration mode |
| Selective index (1k notes) | <2s | Targeted work |
| Full index (10k notes) | <30s | Common large repo |
| Full index (50k notes) | <2m | Very large repos |

**Note:** These targets assume SSD storage. Adjust expectations for HDD or network filesystems.

## Error Handling

### Interruption

- Handle Ctrl+C gracefully
- Save partial progress (checkpoint)
- Resumeable on next run
- Display "Index interrupted. Run `qipu index` to resume."

### Corruption

- Detect corruption via database validation
- On corruption: auto-rebuild (already implemented)
- Show clear error with recommendation

### Resource Exhaustion

- Monitor memory usage during indexing
- If memory pressure detected: reduce batch size
- Fall back to streaming (one note at a time) if needed

## Schema Additions

Add to `operational_database.md` schema:

```sql
-- Indexing checkpoints for batched indexing
CREATE TABLE IF NOT EXISTS indexing_checkpoints (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  batch_number INTEGER NOT NULL,
  last_note_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  completed_at TEXT
);

-- Track indexing metadata
CREATE TABLE IF NOT EXISTS index_metadata (
  key TEXT PRIMARY KEY,
  value TEXT
);

-- Metadata keys: 'total_notes', 'indexed_notes', 'last_indexed_at', 'indexing_status'
```

## CLI Interface

### `qipu index` Command

```bash
qipu index [OPTIONS]
```

**Options:**
- `--force`: Full re-index, ignore mtime
- `--quick`: Index only MOCs + 100 recent notes
- `--tag <tag>`: Index only notes with tag
- `--type <type>`: Index only notes of type
- `--moc <id>`: Index MOC and linked notes
- `--recent <n>`: Index N most recent notes
- `--modified-since <time>`: Index notes modified since timestamp
- `--batch <n>`: Set batch size (default: 1000)
- `--background`: Run in background
- `--verbose`: Show detailed progress
- `--status`: Show index status only (don't index)
- `--resume`: Resume from last checkpoint

**Exit Codes:**
- 0: Success
- 1: Error (corruption, file system error)
- 130: Interrupted (Ctrl+C) - partial index saved

## Open Questions

1. Should `qipu init` automatically run an initial index?
   - Pro: Ready to use immediately
   - Con: Adds delay to init; may surprise users

2. Should we detect if index is "stale" and prompt user?
   - Definition: More than N% of notes have mtime newer than index mtime
   - Threshold: e.g., 30% stale → suggest re-index

3. Should background indexing write logs to a specific file?
   - Default: `.qipu/indexing.log`
   - Configurable via config file

4. Should we provide `qipu index --watch` for continuous indexing?
   - Watches file changes and auto-indexes
   - Could be resource-intensive

## Implementation Phases

### Phase 1: Incremental Indexing (Core)
- Add mtime tracking to index_metadata table
- Modify `qipu index` to skip unchanged notes
- Add basic progress reporting
- Target: P1, required for usability

### Phase 2: Selective Indexing
- Add filter flags to `qipu index`
- Implement `--quick` mode
- Add `--status` command
- Target: P2, improves large knowledge base usability

### Phase 3: Batched/Progressive Indexing
- Add checkpointing infrastructure
- Implement batched indexing with resume
- Enhanced progress display with ETA
- Target: P2, handles very large repos

### Phase 4: Background/Lazy Indexing
- Add `--background` flag
- Implement lazy indexing on note access
- Target: P3, advanced use cases

## Non-Goals

- Automatic background indexing on file system changes (inotify/fsevents)
  - Too complex, platform-specific, resource-intensive
  - Users can run `qipu index` manually as needed
- Distributed indexing across multiple processes/workers
  - SQLite is single-writer; adds complexity
  - Single-threaded is sufficient for 50k notes at target rates
