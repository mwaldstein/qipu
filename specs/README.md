# Qipu Specifications

This directory contains **implementable** Qipu specifications. Each file should describe a specific, buildable topic of concern and be concrete enough to implement and test.

Project-level vision/goals live in the repo root `README.md`. Non-spec guidance/examples live under `docs/`.

## Planning Documents

- **[`../IMPLEMENTATION_PLAN.md`](../IMPLEMENTATION_PLAN.md)** - Concrete implementation tasks, bugs, and missing test coverage that need to be addressed
- **[`../FUTURE_PLAN.md`](../FUTURE_PLAN.md)** - Future work, exploratory features, and open questions from specs not yet in the implementation plan

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
| [`value-model.md`](value-model.md) | Ranking | Note importance/quality scores and weighted traversal |
| [`distribution.md`](distribution.md) | Distribution | Installation methods and release automation |
| [`custom-metadata.md`](custom-metadata.md) | Metadata | Application-specific metadata in frontmatter |
| [`telemetry.md`](telemetry.md) | Telemetry | DRAFT - usage analytics (not implemented) |

## Status Tracking

**Spec Status**: Is the specification complete and concrete enough to implement?
**Impl Status**: Is the implementation complete per the spec?
**Test Status**: Is test coverage adequate?

*Last audited: 2026-01-20*

| Spec | Spec | Impl | Tests | Notes |
| --- | --- | --- | --- | --- |
| `cli-tool.md` | ✅ | ✅ | ⚠️ | Missing tests for visible store discovery, broader determinism, perf/no-network |
| `knowledge-model.md` | ✅ | ✅ | ⚠️ | ID scheme + backlink behaviors lack integration coverage; tag aliases optional |
| `storage-format.md` | ✅ | ⚠️ | ⚠️ | Missing wiki-link rewrite, config store root, flat-notes enforcement |
| `cli-interface.md` | ✅ | ✅ | ⚠️ | Missing tests for create alias/open/id, list tag/since/records, search opts |
| `indexing-search.md` | ✅ | ⚠️ | ✅ | Incremental indexing not exposed; backlink index derived on demand |
| `semantic-graph.md` | ✅ | ⚠️ | ⚠️ | CLI help lists subset of types; limited tests for other types/custom inverses |
| `graph-traversal.md` | ✅ | ⚠️ | ⚠️ | Missing CSV flags + context walk; tests missing inversion + truncation limits |
| `similarity-ranking.md` | ✅ | ⚠️ | ⚠️ | No clustering/see-also; tests missing weight/stop-word/default thresholds |
| `records-output.md` | ✅ | ⚠️ | ⚠️ | `prime` lacks max-chars; link tree/path truncation tests missing |
| `llm-context.md` | ✅ | ⚠️ | ⚠️ | Per-note truncation + prime budget missing; store path formatting differs |
| `llm-user-validation.md` | ✅ | ❌ | ❌ | No harness implementation under `src/` or tests (only transcripts) |
| `provenance.md` | ✅ | ⚠️ | ⚠️ | Verified defaults/automation not enforced or tested |
| `export.md` | ✅ | ⚠️ | ⚠️ | Tag/query tests missing; bibliography/link modes untested; no transitive export |
| `compaction.md` | ✅ | ⚠️ | ⚠️ | `compact show` ignores max nodes; `compact guide` tests missing |
| `pack.md` | ✅ | ⚠️ | ⚠️ | Link filtering vs spec; attachment paths flattened; selector/attachment tests missing |
| `workspaces.md` | ✅ | ⚠️ | ⚠️ | Last updated missing; rename strategy absent; graph-slice copy + parent_id gaps |
| `structured-logging.md` | ✅ | ⚠️ | ⚠️ | Log-level gating/validation gaps; note ops missing instrumentation |
| `operational-database.md` | ✅ | ⚠️ | ✅ | Startup repair unused; DB not sole index; rebuild always; weighting mismatch |
| `value-model.md` | ✅ | ✅ | ⚠️ | Tests missing for value set/show, search sort, list/context min-value |
| `distribution.md` | ⚠️ | ❌ | ❌ | Release automation + install scripts missing |
| `custom-metadata.md` | ✅ | ❌ | ❌ | No custom frontmatter/DB/CLI support |
| `telemetry.md` | DRAFT | ❌ | ❌ | Explicitly marked "DO NOT IMPLEMENT" |

## Legend

- ✅ Complete / Ready
- ⚠️ Partial / Has gaps
- ❌ Not implemented / No coverage

## Remaining Gaps

### P1: Correctness Bugs

| Spec | Gap | Notes |
| --- | --- | --- |
| `operational-database.md` | Repair + rebuild semantics | `src/lib/db/mod.rs:83-85`, `src/commands/index.rs:14-19` |
| `operational-database.md` | DB-only index + ranking weights | `src/lib/store/query.rs:13-52`, `src/lib/db/search.rs:20-105` |
| `storage-format.md` | Missing rewrite/config/flat enforcement | `src/lib/index/links.rs:35-137`, `src/lib/config.rs:14-115` |
| `graph-traversal.md` | CSV flags + context walk | `src/cli/link.rs:73-79`, `src/cli/commands.rs:226-279` |
| `records-output.md` | `prime` budgeting | `src/commands/prime.rs:184-196` |
| `llm-context.md` | Per-note truncation + prime budget | `src/commands/context/budget.rs:55-81`, `src/commands/prime.rs:15-70` |
| `workspaces.md` | List/merge/metadata gaps | `src/commands/workspace/list.rs:70-100`, `src/commands/workspace/merge.rs:20-91` |
| `pack.md` | Link filtering + attachment path flattening | `src/commands/dump/mod.rs:215-255`, `src/commands/load/mod.rs:70-90` |
| `semantic-graph.md` | Help lists subset of types | `src/cli/link.rs:17-56` |
| `structured-logging.md` | Log-level gating/validation + note ops spans | `src/commands/search.rs:36-54`, `src/lib/db/notes/create.rs:1-75` |
| `provenance.md` | Verified defaults missing | `src/commands/create.rs:49-61` |
| `compaction.md` | `compact show` truncation missing | `src/commands/compact/show.rs:46-105` |

### P2/P3: Missing Coverage or Features

| Spec | Gap | Notes |
| --- | --- | --- |
| `cli-tool.md` | Test coverage | Need visible-store discovery and broader golden determinism tests |
| `cli-interface.md` | Test coverage | Missing tests for create/list/search/compact flag variants |
| `value-model.md` | Test coverage | Missing tests for value set/show, search sort, min-value filters |
| `export.md` | Coverage + optional | Tag/query/bibliography/link-mode tests; optional BibTeX/transitive export |
| `graph-traversal.md` | Test coverage | Missing inversion + max_nodes/edges/fanout tests |
| `records-output.md` | Test coverage | Missing truncation tests for tree/path records |
| `workspaces.md` | Test coverage | Need `workspace merge --dry-run` coverage |
| `similarity-ranking.md` | Coverage + optional | Missing weight/stop-word/default threshold tests; no clustering feature |
| `pack.md` | Test coverage | Missing selector, attachment, compatibility tests |
| `custom-metadata.md` | Unimplemented | Custom frontmatter + DB + CLI missing |
| `distribution.md` | Unimplemented | Release automation and install scripts missing |
| `llm-user-validation.md` | Unimplemented | No harness implementation in `src/` or tests |

### Not Applicable

| Spec | Reason |
| --- | --- |
| `telemetry.md` | DRAFT spec explicitly prohibits implementation |
| `knowledge-model.md` tag aliases | Marked as optional in spec |
