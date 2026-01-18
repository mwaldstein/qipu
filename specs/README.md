# Qipu Specifications

This directory contains **implementable** Qipu specifications. Each file should describe a specific, buildable topic of concern and be concrete enough to implement and test.

Project-level vision/goals live in the repo root `README.md`. Non-spec guidance/examples live under `docs/`.

## Spec Index

| Spec | Domain | What it covers |
| --- | --- | --- |
| [`cli-tool.md`](cli-tool.md) | CLI runtime | Global flags, store discovery, determinism, exit codes |
| [`knowledge-model.md`](knowledge-model.md) | Domain model | Note types, IDs, tags, typed links |
| [`storage-format.md`](storage-format.md) | Storage | On-disk layout, note format, link encoding |
| [`cli-interface.md`](cli-interface.md) | Interface | Commands, flags, output formats, exit codes |
| [`indexing-search.md`](indexing-search.md) | Navigation | Indexes, backlinks/graph, search semantics |
| [`semantic-graph.md`](semantic-graph.md) | Links | User-defined link types, semantic inversion |
| [`graph-traversal.md`](graph-traversal.md) | Retrieval | Graph traversal (tree/path), ordering, JSON shape |
| [`similarity-ranking.md`](similarity-ranking.md) | Ranking | BM25, cosine similarity, duplicate detection |
| [`records-output.md`](records-output.md) | Output | Line-oriented records for context injection |
| [`llm-context.md`](llm-context.md) | LLM integration | `prime` + context bundles, budgeting, safety |
| [`llm-user-validation.md`](llm-user-validation.md) | LLM user test | Validate LLM can use qipu as primary user |
| [`provenance.md`](provenance.md) | Trust | Author/source/trust metadata for LLM content |
| [`export.md`](export.md) | Export | Bundling/outlines/bibliographies, deterministic ordering |
| [`compaction.md`](compaction.md) | Compaction | Digest-first navigation and lossless decay |
| [`pack.md`](pack.md) | Pack | Single-file dump/load for sharing raw knowledge |
| [`workspaces.md`](workspaces.md) | Workspaces | Temporary and secondary stores for agent tasks |
| [`structured-logging.md`](structured-logging.md) | Infrastructure | Structured logging framework with tracing support |

## Status Tracking

**Spec Status**: Is the specification complete and concrete enough to implement?
**Impl Status**: Is the implementation complete per the spec?
**Test Status**: Is test coverage adequate?

*Last audited: 2026-01-17*

| Spec | Spec | Impl | Tests | Notes |
| --- | --- | --- | --- | --- |
| `cli-tool.md` | ✅ | ✅ | ✅ | Performance budgets not automated |
| `knowledge-model.md` | ✅ | ✅ | ✅ | |
| `storage-format.md` | ✅ | ✅ | ✅ | `qipu.db` not implemented (future) |
| `cli-interface.md` | ✅ | ✅ | ⚠️ | Missing tests: capture, workspace commands |
| `indexing-search.md` | ✅ | ⚠️ | ✅ | Missing: SQLite FTS5, recency boost |
| `semantic-graph.md` | ✅ | ✅ | ⚠️ | No unit tests for LinkType::inverse() |
| `graph-traversal.md` | ✅ | ✅ | ⚠️ | Missing tests: limits, direction=in, type filtering |
| `similarity-ranking.md` | ✅ | ⚠️ | ⚠️ | Missing: stop words, stemming, term freq storage |
| `records-output.md` | ✅ | ✅ | ⚠️ | Limited test coverage |
| `llm-context.md` | ✅ | ✅ | ✅ | Open: backlinks, summarization |
| `llm-user-validation.md` | ✅ | ❌ | ⚠️ | **Scaffold only** - no actual LLM invocation |
| `provenance.md` | ✅ | ✅ | ⚠️ | Missing: prompt_hash test, JSON output fields |
| `export.md` | ✅ | ✅ | ⚠️ | Missing tests: --tag, --query, bibliography mode |
| `compaction.md` | ✅ | ✅ | ✅ | |
| `pack.md` | ✅ | ⚠️ | ⚠️ | Missing: --strategy, store_version, conflict resolution |
| `workspaces.md` | ✅ | ⚠️ | ❌ | **Bugs**: merge strategies broken; **No tests** |
| `structured-logging.md` | ✅ | ❌ | ❌ | **Not started**: Infrastructure improvement |

## Legend

- ✅ Complete / Ready
- ⚠️ Partial / Has gaps
- ❌ Not implemented / No coverage

## Detailed Gap Summary

### P1: Correctness Issues
- **workspaces.md**: `overwrite` and `merge-links` strategies have bugs; `--force` ignored
- **pack.md**: `--strategy` not implemented; always overwrites on load
- **llm-user-validation.md**: Test framework exists but doesn't invoke actual LLM

### P2: Missing Test Coverage
- `capture` command (cli-interface)
- `workspace` commands (workspaces)
- Graph traversal limits (graph-traversal)
- Type filtering (graph-traversal)
- Pack conflict strategies (pack)
- `prompt_hash` (provenance)
- `--max-tokens` (llm-context)

### P3: Future/Optional Items
- SQLite FTS5 (indexing-search)
- Recency boost (indexing-search)
- Stop words, stemming (similarity-ranking)
- Term frequency storage (similarity-ranking)
- Wiki-link canonicalization in index (records-output)
- Backlinks in context (llm-context)
- LLM meta-evaluation (llm-user-validation)
- `rename` merge strategy (workspaces)
