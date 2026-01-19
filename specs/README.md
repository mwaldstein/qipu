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
| [`operational-database.md`](operational-database.md) | Database | SQLite as operational layer, FTS5, schema |
| [`telemetry.md`](telemetry.md) | Telemetry | DRAFT - usage analytics (not implemented) |

## Status Tracking

**Spec Status**: Is the specification complete and concrete enough to implement?
**Impl Status**: Is the implementation complete per the spec?
**Test Status**: Is test coverage adequate?

*Last audited: 2026-01-19*

| Spec | Spec | Impl | Tests | Notes |
| --- | --- | --- | --- | --- |
| `cli-tool.md` | ✅ | ✅ | ✅ | All flags implemented; `--root` tested; verbose timing has `discover_store` key |
| `knowledge-model.md` | ✅ | ✅ | ✅ | Closed enum; all fields implemented; tag aliases optional/not implemented |
| `storage-format.md` | ✅ | ✅ | ✅ | All directories; frontmatter fields; `qipu.db` implemented |
| `cli-interface.md` | ✅ | ✅ | ✅ | All commands implemented with correct exit codes |
| `indexing-search.md` | ✅ | ✅ | ✅ | SQLite FTS5 complete; ripgrep removed; BM25 ranking |
| `semantic-graph.md` | ✅ | ✅ | ✅ | Config schema aligned; semantic inversion works; virtual edges |
| `graph-traversal.md` | ✅ | ✅ | ✅ | All directions; type filters; "(seen)" in human output; truncation flags |
| `similarity-ranking.md` | ✅ | ✅ | ✅ | BM25; cosine similarity; Porter stemming; stop words; duplicate detection |
| `records-output.md` | ✅ | ✅ | ✅ | All prefixes documented (H/N/S/E/B/W/D/C/M/L/A + B-END) |
| `llm-context.md` | ✅ | ✅ | ✅ | Budget enforcement; --transitive; --backlinks; --related; safety banner |
| `llm-user-validation.md` | ✅ | ⚠️ | ⚠️ | Harness works; missing: tool default, some scenario fields |
| `provenance.md` | ✅ | ✅ | ✅ | All 5 fields; JSON output; CLI support; context prioritization |
| `export.md` | ✅ | ✅ | ✅ | MOC ordering; anchor rewriting; attachment link rewriting |
| `compaction.md` | ✅ | ✅ | ✅ | All commands; all flags; truncation indicators |
| `pack.md` | ✅ | ✅ | ✅ | All strategies work; merge-links preserves content; filters work |
| `workspaces.md` | ✅ | ✅ | ⚠️ | Merge strategies work; --dry-run implemented; tests needed for --dry-run/--empty |
| `structured-logging.md` | ✅ | ⚠️ | ✅ | Tracing init works; tests pass; 16 eprintln! remain |
| `operational-database.md` | ✅ | ✅ | ✅ | SQLite complete; FTS5; schema version; incremental repair |
| `telemetry.md` | DRAFT | ❌ | ❌ | Explicitly marked "DO NOT IMPLEMENT" |

## Legend

- ✅ Complete / Ready
- ⚠️ Partial / Has gaps
- ❌ Not implemented / No coverage

## Remaining Gaps

### P3: Optional / Low Priority

| Spec | Gap | Notes |
| --- | --- | --- |
| `cli-tool.md` | Verbose timing keys | Only `discover_store` instrumented; `load_indexes`/`execute_command` missing |
| `structured-logging.md` | eprintln! cleanup | 16 callsites remain; should use tracing |
| `llm-user-validation.md` | Tool default | Defaults to "opencode", spec says "amp" |
| `llm-user-validation.md` | Scenario schema | Missing id, tags, docs.prime, setup, etc. |
| `workspaces.md` | Test coverage | Need --dry-run and --empty tests |
| `operational-database.md` | validate_consistency() | Method exists but not called on startup |

### Not Applicable

| Spec | Reason |
| --- | --- |
| `telemetry.md` | DRAFT spec explicitly prohibits implementation |
| `knowledge-model.md` tag aliases | Marked as optional in spec |
