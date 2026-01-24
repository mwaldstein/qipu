# Progressive Indexing and Re-Indexing

## Purpose

For large knowledge bases (10k+ notes), full text indexing can take minutes or longer. Since qipu stores raw notes in git, full re-indexing from raw files is a common operation (e.g., after cloning a repo or after significant changes).

This spec defines a **two-level indexing approach**:
1. **Basic indexing** (fast): Index metadata for ALL notes - title, type, links, tags, sources
2. **Full-text indexing** (slower): Index note body content for comprehensive search

This strategy ensures immediate queryability across all notes while deferring expensive full-text indexing.

## Indexing Levels

### Level 1: Basic Indexing (Metadata-Only)

Index lightweight metadata for all notes, enabling queries without full text search.

**Indexed Fields:**
- Note ID
- Title
- Note type (permanent, ephemeral, moc, spec)
- Tags
- Creation/updated timestamps
- Links (inbound/outbound with types)
- Sources (URLs, titles)
- Value score
- Custom metadata keys (optional, for filtering)

**Not Indexed:**
- Note body text
- Note summary

**Performance:**
- O(total notes) but very fast: ~100-200 notes/sec
- Minimal I/O: reads frontmatter only
- 10k notes: ~1-2 minutes
- 50k notes: ~5-10 minutes

**Query Capabilities with Basic Index:**
- Search by title, type, tags, sources
- Traverse graph by links (all link types)
- Filter by value, custom metadata
- List notes with metadata
- Show note with metadata

**Query Limitations without Full-Text Index:**
- Cannot search body content
- Cannot search for specific text phrases within notes
- Similarity search not available
- Search relevance is based on metadata matches only

### Level 2: Full-Text Indexing

Index complete note content (body + summary) for comprehensive search.

**Indexed Fields (in addition to Level 1):**
- Note body text (FTS5 full-text search)
- Note summary text

**Performance:**
- O(total notes) but slower: ~50-100 notes/sec
- Full I/O: reads entire note files
- 10k notes: ~2-4 minutes
- 50k notes: ~10-20 minutes

**Query Capabilities with Full-Text Index:**
- All Level 1 capabilities
- Search body content
- Phrase search, BM25 ranking
- Similarity search
- Context-aware search (semantic matching)

**Query Graceful Degradation:**
- Search works with Level 1 (basic index) even if full-text isn't indexed
- Results limited to metadata-only matches when full-text unavailable
- Clear indication when full-text search is limited (e.g., "Basic index only: search limited to metadata")

## Goals

- Enable efficient indexing of large knowledge bases (10k+ notes)
- Provide clear progress feedback during indexing operations
- Support selective/critical-only indexing when full index isn't needed
- Allow interruption and resumption of long-running indexing
- Minimize the cost of common re-indexing scenarios

## Indexing Strategies

### 0. Auto-Indexing on Store Open

When opening a store (e.g., `qipu init`, `qipu search`, `qipu list`), automatically index notes if needed, using intelligent two-level approach to avoid unexpected delays.

**Two-Level Approach:**
1. Always run **Basic Indexing** (metadata-only) for all notes - fast, enables immediate queries
2. Conditionally run **Full-Text Indexing** based on note count and config
   - Small repos (<1k notes): Full-text immediately (fast enough)
   - Medium repos (1k-10k notes): Full-text immediately or prompt user
   - Large repos (10k+ notes): Basic only initially; defer full-text via explicit command or background

**Basic Indexing (Always Runs):**
- Index metadata for ALL notes (title, type, links, tags, sources, timestamps)
- Skips note body content
- Very fast (~100-200 notes/sec)
- Enables: search by metadata, graph traversal, listing, filtering

**Full-Text Indexing (Conditional):**
- Index note body and summary text (FTS5)
- Slower (~50-100 notes/sec)
- Enables: body content search, phrase search, BM25 ranking, similarity

**Auto-Indexing Decision Table (Basic + Full-Text):**

| Note Count | Basic Index | Full-Text Index | Rationale |
|-----------|--------------|-------------------|-----------|
| <1k | ✓ Always | ✓ Always | Fast enough to complete in <1s total |
| 1k-10k | ✓ Always | ✓ Always | Acceptable startup time (<30s) |
| 10k-50k | ✓ Always | ✗ Prompt/Optional | Basic index provides immediate value; user chooses full-text |
| >50k | ✓ Always | ✗ Optional | Basic index sufficient for exploration; full-text on-demand |

**User Feedback:**

For large repos (10k+ notes) where full-text is deferred:

```
qipu search "topic"
Detected 12,450 notes.
Basic index: Complete (12,450 notes) ✓
Full-text index: Not available (deferred)
Search results: Limited to metadata (title, type, tags, links).
Tip: Run `qipu index --full` for comprehensive text search, or `qipu index --quick` for MOCs + recent.
```

For medium repos (1k-10k notes) where both indexes are available:

```
qipu init
Detected 4,500 notes.
Basic index: Complete (4,500 notes) ✓
Full-text index: Complete (4,500 notes) ✓
Store ready for full search.
```

**Use Cases:**
- `qipu init` on cloned repo with existing knowledge base
- Opening store after pulling git changes
- Store auto-opened by first command

When opening a store (e.g., `qipu init`, `qipu search`, `qipu list`), automatically index notes if needed, but use intelligent selection based on note count to avoid unexpected delays.

**Implementation:**
- Check if index exists and is valid
- If no index or corrupted: determine best indexing strategy based on note count
- For small repos (<1k notes): Full index immediately (fast)
- For medium repos (1k-10k notes): Incremental index (mtime-based) or Quick index
- For large repos (10k+ notes): Quick index only (MOCs + 100 recent), with background full index option

**Auto-Indexing Decision Table:**

| Note Count | Index Strategy | Rationale |
|-----------|----------------|-----------|
| <1k | Full index | Fast enough to complete in <5s |
| 1k-10k | Incremental (mtime-based) | Only changed notes; typically <15s |
| 10k-50k | Quick index (MOCs + 100 recent) | Fast startup (<3s); offers critical notes for exploration |
| >50k | Quick index + prompt for background | Immediate usability; user can choose to run full index |

**Use Cases:**
- `qipu init` on cloned repo with existing knowledge base
- Opening store after pulling git changes
- Store auto-opened by first command

**User Feedback:**
```
qipu search "topic"
Detected 12,450 notes (no index). Quick-indexing MOCs + recent notes...
Indexed 120 notes in 1.2s (MOCs + 100 recent)
Search results from quick index. For full search, run: qipu index
```

or for large repos:
```
qipu init
Detected 45,000 notes. Quick-indexing critical notes...
Indexed 120 notes in 1.8s (MOCs + 100 recent)
Store ready with quick index.
Tip: For full search across all notes, run: qipu index
```

### 1. Incremental Indexing (mtime-based)

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

### 5. Lazy/On-Demand Indexing (OPTIONAL - for future consideration only)

Defer indexing until a note is actually accessed, then build index gradually.

**Note:** This approach is NOT recommended for primary use due to complexity. The recommended approach is auto-indexing with adaptive strategy on store open.

**Implementation (for future consideration):**
- Maintain "indexed" flag per note (default false)
- When note is accessed (via search, show, context):
  - If not indexed, index it immediately
  - Set indexed flag

**Use Cases:**
- Very large knowledge bases where only subset is regularly used
- Development/exploration workspaces
- Cold starts where full index isn't immediately needed

## Configuration Options

### Auto-Indexing Behavior

Add configuration for automatic indexing on store open:

```toml
[auto_index]
enabled = true                # Enable/disable auto-indexing (default: true)
strategy = "adaptive"          # "full", "incremental", "quick", "adaptive" (default: "adaptive")
adaptive_threshold = 10000      # Note count threshold for adaptive strategy (default: 10000)
quick_notes = 100            # Notes for --quick mode (default: 100)
```

**Strategies:**
- `full`: Always do full index on open (fast for small repos)
- `incremental`: Always do incremental index on open (skip unchanged)
- `quick`: Always do quick index on open (MOCs + recent)
- `adaptive`: Choose based on note count (see Auto-Indexing on Store Open section)

**Environment Variable:**
- `QIPU_AUTO_INDEX=0`: Disable auto-indexing
- `QIPU_AUTO_INDEX_STRATEGY=quick`: Force specific strategy

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

### Background Mode (NON-GOAL - see Non-Goals section)

```bash
qipu index --background
```
- Runs in background process
- Writes PID file for monitoring/cancellation
- Logs progress to file or stderr
- Returns immediately with PID

**Note:** Background indexing is NOT recommended for LLM tool usage. LLMs are primary users and need immediate, synchronous access to the knowledge base.

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Basic index (all notes, metadata-only) | <1s | <1k notes |
| Basic index (all notes, metadata-only) | <2s | 1k-5k notes |
| Basic index (all notes, metadata-only) | <4s | 5k-10k notes |
| Basic index (all notes, metadata-only) | <8s | 10k-50k notes |
| Incremental full-text index (100 changed notes) | <2s | Most common case |
| Quick index (MOCs + 100 recent, full-text) | <2s | Exploration mode |
| Selective index (1k notes, full-text) | <5s | Targeted work |
| Full index (1k notes, full-text) | <3s | Small repos |
| Full index (5k notes, full-text) | <10s | Medium repos |
| Full index (10k notes, full-text) | <25s | Common large repo |
| Full index (50k notes, full-text) | <2m | Very large repos |

**Note:** These targets assume SSD storage. Adjust expectations for HDD or network filesystems.

## Incremental Test Strategy

Start with 1k notes and build up to 10k notes to ensure early feedback on performance issues:

1. Run all 1k, 2k, 5k scenario tests first (fast, catch regressions early)
2. Once 1k tests pass, move to 10k scenario tests
3. This prevents spending multiple minutes on a 10k test only to find a 1k-level issue
4. CI pipeline should fail fast if 1k/2k tests exceed targets

## Performance Testing

**Scenario 1: Cold Start on Small Repo (1k notes)**
- Setup: Fresh qipu init on 1k notes (no index)
- Expected: Basic index <2s, then prompt for full-text
- Measure: Total time to interactive prompt

**Scenario 2: Cold Start on Small-Medium Repo (2k notes)**
- Setup: Fresh qipu init on 2k notes (no index)
- Expected: Basic index <3s, then prompt for full-text
- Measure: Total time to interactive prompt

**Scenario 3: Cold Start on Medium Repo (5k notes)**
- Setup: Fresh qipu init on 5k notes (no index)
- Expected: Basic index <5s, then prompt for full-text
- Measure: Total time to interactive prompt

**Scenario 4: Cold Start on Large Repo (10k notes)**
- Setup: Fresh qipu init on 10k notes (no index)
- Expected: Basic index <8s, then prompt for full-text
- Measure: Total time to interactive prompt

**Scenario 5: Cold Start on Large Repo (50k notes)**
- Setup: Fresh qipu init on 50k notes (no index)
- Expected: Basic index <20s, then prompt for full-text
- Measure: Total time to interactive prompt

**Scenario 6: Incremental Update on Medium Repo (2k notes)**
- Setup: 2k notes, modify 100 notes, run qipu search
- Expected: Basic index auto-runs <1s (unchanged notes skipped)
- Measure: Time to search result

**Scenario 7: Incremental Update on Large Repo (10k notes)**
- Setup: 10k notes, modify 100 notes, run qipu search
- Expected: Basic index auto-runs <3s (unchanged notes skipped)
- Measure: Time to search result

**Scenario 8: Quick Mode on Medium Repo (5k notes)**
- Setup: 5k notes, run `qipu index --quick`
- Expected: MOCs + 100 recent indexed <1s
- Measure: Index time, notes indexed

**Scenario 9: Quick Mode on Large Repo (10k notes)**
- Setup: 10k notes, run `qipu index --quick`
- Expected: MOCs + 100 recent indexed <2s
- Measure: Index time, notes indexed

**Scenario 10: Full Text Rebuild on Small Repo (1k notes)**
- Setup: 1k notes, run `qipu index --force`
- Expected: <3s for full-text indexing
- Measure: Total index time, memory usage

**Scenario 11: Full Text Rebuild on Medium Repo (5k notes)**
- Setup: 5k notes, run `qipu index --force`
- Expected: <10s for full-text indexing
- Measure: Total index time, memory usage

**Scenario 12: Full Text Rebuild on Large Repo (10k notes)**
- Setup: 10k notes, run `qipu index --force`
- Expected: <25s for full-text indexing
- Measure: Total index time, memory usage

### Test Infrastructure

**Benchmark Command:**
```bash
qipu benchmark index --size 10000 --iterations 5
```

Output format:
```
Indexing Benchmark (5 iterations, 10,000 notes)
---------------------------------------------
Basic index: 1.2s ±0.1s (min: 1.0s, max: 1.4s)
Full-text index: 4.5s ±0.3s (min: 4.1s, max: 5.0s)
Memory peak: 45MB
Disk reads: 12,450 (all notes)
Batch size: 1000 notes
```

**Automated Tests:**
```rust
#[test]
fn bench_basic_indexing_1k_notes() {
    // Setup 1k notes
    // Measure basic indexing time
    // Assert <2s target
}

#[test]
fn bench_basic_indexing_2k_notes() {
    // Setup 2k notes
    // Measure basic indexing time
    // Assert <3s target
}

#[test]
fn bench_basic_indexing_5k_notes() {
    // Setup 5k notes
    // Measure basic indexing time
    // Assert <4s target
}

#[test]
fn bench_basic_indexing_10k_notes() {
    // Setup 10k notes
    // Measure basic indexing time
    // Assert <5s target
}

#[test]
fn bench_basic_indexing_50k_notes() {
    // Setup 50k notes
    // Measure basic indexing time
    // Assert <20s target
}

#[test]
fn bench_incremental_indexing_100_changed_1k_notes() {
    // Setup 1k notes, modify 10
    // Measure incremental indexing time
    // Assert <1s target for changed notes
}

#[test]
fn bench_incremental_indexing_100_changed_2k_notes() {
    // Setup 2k notes, modify 100
    // Measure incremental indexing time
    // Assert <1s target for changed notes
}

#[test]
fn bench_incremental_indexing_100_changed_10k_notes() {
    // Setup 10k notes, modify 100
    // Measure incremental indexing time
    // Assert <2s target for changed notes
}

#[test]
fn bench_full_text_indexing_1k_notes() {
    // Setup 1k notes with basic index
    // Measure full-text indexing time
    // Assert <3s target (includes basic + full-text)
}

#[test]
fn bench_full_text_indexing_5k_notes() {
    // Setup 5k notes with basic index
    // Measure full-text indexing time
    // Assert <8s target (includes basic + full-text)
}

#[test]
fn bench_full_text_indexing_10k_notes() {
    // Setup 10k notes with basic index
    // Measure full-text indexing time
    // Assert <25s target (includes basic + full-text)
}
```

### Performance Metrics to Track

1. **Indexing Time:**
   - Basic index time by note count buckets (1k, 10k, 50k)
   - Full-text index time by note count buckets
   - Incremental index time by changed note count (10, 100, 1000)

2. **Memory Usage:**
   - Peak memory during indexing
   - Memory per note (target: <1KB per note during indexing)

3. **I/O Operations:**
   - File reads (notes read, total bytes)
   - Database writes (transactions, rows inserted)
   - Checkpoint writes (if batched)

4. **Query Performance:**
   - Metadata-only search latency (p99 <10ms)
   - Full-text search latency (p99 <50ms)
   - Graph traversal time (100 hops <100ms)

### Performance Regression Testing

Add to CI pipeline:
```yaml
- name: Benchmark indexing performance
  run: |
    cargo test bench_basic_indexing_10k_notes --release -- --nocapture
    cargo test bench_full_text_indexing_10k --release -- --nocapture
    cargo test bench_incremental_indexing --release -- --nocapture
```

**Alert Thresholds:**
- Warning if performance degrades >30% from baseline
- Fail if any benchmark exceeds target by >50%
- Block merge if performance regresses significantly

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

### `qipu init` Command Updates

```bash
qipu init [OPTIONS]
```

**New Options:**
- `--no-index`: Skip automatic indexing on store initialization
- `--index-strategy <strategy>`: Override auto-indexing strategy (full, incremental, quick)

**Behavior:**
- Default: Auto-index using adaptive strategy (see Auto-Indexing on Store Open)
- `--no-index`: Create store without indexing; user must run `qipu index` manually
- `--index-strategy`: Force specific indexing strategy regardless of config

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

1. Should we detect if index is "stale" and prompt user?
   - Definition: More than N% of notes have mtime newer than index mtime
   - Threshold: e.g., 30% stale → suggest re-index

2. Should background indexing write logs to a specific file?
   - Default: `.qipu/indexing.log`
   - Configurable via config file

3. Should we provide `qipu index --watch` for continuous indexing?
   - Watches file changes and auto-indexes
   - Could be resource-intensive

## Implementation Phases

### Phase 0: Auto-Indexing on Store Open
- Implement intelligent auto-indexing on store open/init
- Add note count detection for strategy selection
- Implement adaptive strategy: full/incremental/quick based on thresholds
- Add `--no-index` and `--index-strategy` flags to `qipu init`
- Add config options for auto-indexing behavior
- Target: P1, prevents unexpected delays on large knowledge bases

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
