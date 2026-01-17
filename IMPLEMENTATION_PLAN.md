# Qipu Implementation Plan

## **Status: Active Development**

Core P0/P1 features are substantially complete. Detailed audit (2026-01-17) identified gaps requiring attention.

**Current Priority**: LLM User Validation Testing Harness (see P0 section)

---

## **P0: LLM User Validation Testing Harness**

The core goal of qipu is LLM usability. Current scaffold runs hardcoded commands instead of real LLM invocations. This must be fixed before other work continues.

### Architecture Decision

**The test harness is a separate binary (`llm-tool-test`), NOT part of qipu.**

Rationale:
- Test infrastructure shouldn't ship in the distributed binary
- Black-box testing validates the actual user experience
- Harness could be reused for testing other LLM-facing CLI tools
- Keeps qipu's dependencies minimal

### Phase 1: Separate Crate Setup
- [x] **Create `llm-tool-test` crate**: New workspace member
  - `crates/llm-tool-test/Cargo.toml`
  - `crates/llm-tool-test/src/main.rs`
  - Add to workspace in root `Cargo.toml`
- [x] **Remove existing scaffold**: Delete `tests/llm/` module
  - Current code runs hardcoded commands, not useful
  - Clean slate for proper implementation
- [x] **Define CLI interface**: Using clap
  - `run`, `list`, `show`, `compare`, `clean` subcommands
  - `--scenario`, `--tags`, `--tool`, `--max-usd`, `--dry-run` flags

### Phase 2: Test Fixtures
- [x] **Create fixture directory**: `crates/llm-tool-test/fixtures/qipu/`
  - `AGENTS.md` - Qipu usage instructions for LLM
  - `README.md` - Project context
  - Sample scenarios in `scenarios/`
- [x] **AGENTS.md for qipu tests**: Representative documentation
  - Core commands with examples
  - Common workflows (create, link, search)
  - Output format guidance
  - Error handling patterns
- [x] **Scenario definitions**: YAML files
  - `capture_basic.yaml` - Create notes from content
  - `link_navigation.yaml` - Navigate linked notes
  - `context_retrieval.yaml` - Use qipu context
  - `search_workflow.yaml` - Find existing notes

### Phase 3: Core Harness Infrastructure
- [x] **Scenario loader**: Parse YAML scenario files
  - `Scenario` struct matching spec schema
  - Support `fixture`, `task.prompt`, `tool_matrix`
  - Support `evaluation.gates` and `evaluation.judge`
- [x] **Environment setup**: Create isolated test directory
  - Copy fixture files (AGENTS.md, README.md, seed data)
  - Initialize qipu store if needed
  - Set working directory for LLM tool
- [x] **PTY session capture**: Real LLM CLI invocation
  - Add `portable-pty` dependency
  - `SessionRunner` for PTY-based command execution
  - Fallback to piped stdout/stderr
  - Capture timing, exit codes, raw output stream
- [x] **Transcript bundle writer**: Create reviewable artifacts
  - `transcript.raw.txt` - Complete session output
  - `events.jsonl` - Structured event log
  - `run.json` - Metadata (scenario, tool, duration, cost)
  - `store_snapshot/` - Post-run qipu export
  - `report.md` - Human-readable summary

### Phase 4: Tool Adapters
- [x] **Amp adapter**: Real `amp` CLI invocation
  - Build context from fixture AGENTS.md
  - Prompt file creation and invocation
  - Transcript capture via PTY
- [x] **OpenCode adapter**: Real `opencode` CLI invocation
  - Same interface as Amp adapter
  - Implemented via `opencode run`
- [x] **Availability checks**: Verify tool installed + authenticated
  - `ToolAdapter::check_availability()` method
  - Graceful skip/error if tool unavailable

### Phase 5: Evaluation System
- [x] **Structural gates**: Cheap, deterministic checks
  - Run qipu commands to verify store state
  - Note count minimum, link count minimum
  - Retrieval query checks (`qipu search` returns results)
  - Doctor validation passes
- [x] **Metrics computation**: Scored evaluation
  - `EvaluationMetrics` struct with counts and checks
  - Return metric vector, not just pass/fail
- [ ] **LLM-as-judge** (optional): Qualitative evaluation
  - Rubric loading from YAML
  - Judge prompt construction
  - Score parsing from structured JSON response

### Phase 6: Results & Reporting
- [ ] **Results database**: Append-only run records
  - `results/results.jsonl`
  - Record: scenario, tool, metrics, scores, outcome, transcript path
- [ ] **Regression detection**: Compare against baselines
  - Score degradation warnings
  - Gate failure alerts
- [ ] **Caching**: Skip identical runs
  - Cache key: scenario hash + fixture hash + tool
  - `--no-cache` to force re-run

### Directory Structure

```
crates/llm-tool-test/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli.rs               # Clap definitions
│   ├── scenario.rs          # Scenario loading
│   ├── fixture.rs           # Test environment setup
│   ├── session.rs           # PTY session runner
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── amp.rs
│   │   └── opencode.rs
│   ├── evaluation.rs        # Gates + judge
│   ├── transcript.rs        # Artifact writing
│   └── results.rs           # Results database
├── fixtures/
│   └── qipu/
│       ├── AGENTS.md
│       ├── README.md
│       └── scenarios/
│           ├── capture_basic.yaml
│           ├── link_navigation.yaml
│           └── ...
└── results/                  # Gitignored, run artifacts
    ├── results.jsonl
    └── transcripts/
```

---

## **P1: Correctness Issues**

### Workspace Merge Bugs
- [x] **`overwrite` strategy broken**: Calls `create_note_with_content` generating new ID instead of replacing existing note ([merge.rs#L44-L53](file:///home/micah/dev/qipu/src/commands/workspace/merge.rs#L44-L53))
- [x] **`merge-links` merges tags, not links**: Spec requires union of links array, implementation unions tags ([merge.rs#L54-L65](file:///home/micah/dev/qipu/src/commands/workspace/merge.rs#L54-L65))
- [ ] **`--force` flag ignored in delete**: Always deletes without checking unmerged changes ([delete.rs#L9](file:///home/micah/dev/qipu/src/commands/workspace/delete.rs#L9))

### Pack Load Missing Features
- [ ] **`--strategy` flag missing**: CLI has no `--strategy` arg for `qipu load`; always overwrites ([cli/mod.rs#L410-L413](file:///home/micah/dev/qipu/src/cli/mod.rs#L410-L413))
- [ ] **Conflict resolution not implemented**: `load_notes` doesn't check if note exists before writing ([load/mod.rs#L121-L187](file:///home/micah/dev/qipu/src/commands/load/mod.rs#L121-L187))
- [ ] **`store_version` header missing**: Not in PackHeader, no compatibility check ([model.rs#L20-L28](file:///home/micah/dev/qipu/src/commands/dump/model.rs#L20-L28))

### **P2: Missing Test Coverage**

- [x] **Workspace commands**: Partial tests for merge/new (Added `tests/workspace_merge_test.rs`)
- [ ] **Capture command**: No dedicated tests
- [ ] **Graph traversal limits**: `--max-nodes`, `--max-edges`, `--max-fanout`, `--direction in` not tested
- [ ] **Type filtering**: `--type`, `--exclude-type`, `--typed-only`, `--inline-only` not tested
- [ ] **Pack conflict strategies**: No tests
- [ ] **`prompt_hash` provenance**: Not tested
- [ ] **Token budgeting**: `--max-tokens` not integration tested

### **P3: Spec-Implementation Gaps**

#### Similarity Ranking
- [ ] **Stop words removal**: Spec requires, not implemented
- [ ] **Stemming**: Spec mentions optional Porter stemmer, not implemented
- [ ] **Term frequency storage**: Index stores only unique terms (tf=1 assumed), reducing accuracy ([similarity/mod.rs#L136-L137](file:///home/micah/dev/qipu/src/lib/similarity/mod.rs#L136-L137) - has TODO)

#### Indexing/Search
- [ ] **SQLite FTS5 backend**: Spec mentions optional SQLite FTS, not implemented
- [ ] **Recency boost**: Spec says recent notes can receive boost, not in ranking

#### Workspace Additional
- [ ] **`--from-note` doesn't copy graph slice**: Only copies single note, not linked notes
- [ ] **`--from-query` uses simple substring match**: Doesn't use search index
- [ ] **`last_updated` missing from list output**
- [ ] **Dry-run shows no conflict report**
- [ ] **No `doctor` check after merge**

#### Provenance
- [ ] **JSON output missing provenance**: create command JSON output omits provenance fields

---

## **Future/Optional Items (Deferred)**

### From Specs
- [ ] Wiki-link canonicalization in `qipu index` (currently only in export)
- [ ] Interactive pickers (dialoguer/inquire)
- [ ] Per-note truncation with `[truncated]` marker
- [ ] Backlinks in context output (llm-context.md open question)
- [ ] Automatic summarization (llm-context.md open question)
- [ ] `qipu.db` SQLite acceleration
- [ ] `rename` merge strategy (complexity warning in spec)
- [ ] LLM-based meta-evaluation for validation tests

### Infrastructure
- [ ] **GitHub Actions**: Disabled, DO NOT enable unless confirmed

---

## **Completed Work**

### ✅ CLI Tool (cli-tool.md) - COMPLETE
- Global flags: `--root`, `--store`, `--format`, `--quiet`, `--verbose`
- Store discovery: walks up from cwd, checks `.qipu/` then `qipu/`
- Exit codes: 0=success, 1=failure, 2=usage, 3=data
- Determinism: stable output ordering throughout
- Full test coverage including golden tests

### ✅ Knowledge Model (knowledge-model.md) - COMPLETE
- Note types: Fleeting, Literature, Permanent, MOC
- ID format: `qp-<hash>` with adaptive length, ULID/timestamp alternatives
- Tags: Vec<String> in frontmatter with tag index
- Typed links: 10 types (5 from spec + 5 additional) with inverses

### ✅ Storage Format (storage-format.md) - COMPLETE
- Directory structure: `.qipu/`, `notes/`, `mocs/`, `attachments/`, `templates/`, `.cache/`
- Note frontmatter: all required + provenance fields
- Config: version, default_note_type, id_scheme, editor, branch, links
- Templates: 4 default templates created on init
- Branch protection: `--branch` flag with orphan branch creation
- Stealth mode: `--stealth` adds store to .gitignore

### ✅ CLI Interface (cli-interface.md) - COMPLETE
- All spec commands implemented + extras (workspace, merge, compact show/guide)
- Provenance flags on create/capture

### ✅ Indexing & Search (indexing-search.md) - SUBSTANTIALLY COMPLETE
- Index structure: metadata, tags, backlinks, graph
- Incremental indexing with mtime tracking
- BM25 with field boosting (title 2.0x, tags 1.5x)
- Ripgrep integration with embedded fallback

### ✅ Semantic Graph (semantic-graph.md) - COMPLETE
- 10 link types with proper inverses
- User-defined types in config
- Semantic inversion at query time
- Virtual edge marker (LinkSource::Virtual)

### ✅ Graph Traversal (graph-traversal.md) - COMPLETE
- `link tree` and `link path` fully implemented
- All direction/hop/limit options
- Deterministic ordering
- Cycle-safe with "(seen)" annotation
- Truncation reporting

### ✅ Records Output (records-output.md) - COMPLETE
- All record types: H, N, S, E, B (with B-END)
- All relevant commands support `--format records`
- Field encoding rules followed
- Summary extraction (frontmatter → ## Summary → first paragraph)

### ✅ LLM Context (llm-context.md) - COMPLETE
- `qipu prime`: deterministic, bounded, includes MOCs + recent notes
- `qipu context`: all selection options, formats, token budgeting
- tiktoken-rs integration with `--max-tokens` and `--model`
- Safety banners with `--safety-banner`

### ✅ Provenance (provenance.md) - SUBSTANTIALLY COMPLETE
- All 5 fields: source, author, generated_by, prompt_hash, verified
- `qipu verify` command with toggle
- Context prioritizes verified notes

### ✅ Export (export.md) - COMPLETE
- Modes: bundle, outline, bibliography
- Selection: --note, --tag, --moc, --query
- Attachments: --with-attachments
- Link modes: wiki, markdown, anchors
- Deterministic ordering

### ✅ Compaction (compaction.md) - COMPLETE
- All 6 subcommands: apply, show, status, report, suggest, guide
- `compacts` field in frontmatter
- Digest-first navigation with `--no-resolve-compaction`
- Virtual expansion at query time
- All invariants checked (single compactor, acyclic, no self-compact, IDs resolve)

### ⚠️ Pack (pack.md) - PARTIAL
- `qipu dump`: all selectors and traversal options ✅
- `qipu load`: basic functionality ✅
- Formats: JSON and records ✅
- **Missing**: `--strategy` flag, conflict resolution, `store_version` header

### ⚠️ Workspaces (workspaces.md) - PARTIAL
- Subcommands: new, list, delete, merge ✅
- Metadata: workspace.toml ✅
- Flags: --temp, --copy-primary, --from-tag ✅
- Global --workspace targeting ✅
- **Bugs**: overwrite/merge-links strategies broken, --force ignored
- **Missing**: rename strategy, graph slice seeding, no tests

### ⚠️ Similarity Ranking (similarity-ranking.md) - PARTIAL
- BM25 and cosine similarity ✅
- Field weighting ✅
- Duplicate detection ✅
- **Missing**: stop words, stemming, term frequency storage

### ⚠️ LLM User Validation (llm-user-validation.md) - SCAFFOLD ONLY
- Framework structure ✅
- Tool adapter pattern ✅
- Transcript recording ✅
- Store validation ✅
- **Critical**: No actual LLM invocation - runs hardcoded commands
