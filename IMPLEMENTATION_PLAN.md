# Qipu Implementation Plan

## **Status: Active Development**

Core P0/P1 features are substantially complete. Detailed audit (2026-01-17) identified gaps requiring attention.

---

## **Priority Gaps (Audit Findings)**

### **P1: Correctness Issues**

#### Workspace Merge Bugs
- [ ] **`overwrite` strategy broken**: Calls `create_note_with_content` generating new ID instead of replacing existing note ([merge.rs#L44-L53](file:///home/micah/dev/qipu/src/commands/workspace/merge.rs#L44-L53))
- [ ] **`merge-links` merges tags, not links**: Spec requires union of links array, implementation unions tags ([merge.rs#L54-L65](file:///home/micah/dev/qipu/src/commands/workspace/merge.rs#L54-L65))
- [ ] **`--force` flag ignored in delete**: Always deletes without checking unmerged changes ([delete.rs#L9](file:///home/micah/dev/qipu/src/commands/workspace/delete.rs#L9))

#### Pack Load Missing Features
- [ ] **`--strategy` flag missing**: CLI has no `--strategy` arg for `qipu load`; always overwrites ([cli/mod.rs#L410-L413](file:///home/micah/dev/qipu/src/cli/mod.rs#L410-L413))
- [ ] **Conflict resolution not implemented**: `load_notes` doesn't check if note exists before writing ([load/mod.rs#L121-L187](file:///home/micah/dev/qipu/src/commands/load/mod.rs#L121-L187))
- [ ] **`store_version` header missing**: Not in PackHeader, no compatibility check ([model.rs#L20-L28](file:///home/micah/dev/qipu/src/commands/dump/model.rs#L20-L28))

#### LLM User Validation
- [ ] **No actual LLM invocation**: `OpenCodeAdapter.execute_task()` runs hardcoded commands, never calls any LLM CLI ([adapter.rs#L32-L149](file:///home/micah/dev/qipu/tests/llm/adapter.rs#L32-L149))

### **P2: Missing Test Coverage**

- [ ] **Workspace commands**: No tests for new/list/delete/merge
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
