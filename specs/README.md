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
| `cli-tool.md` | ✅ | ⚠️ | ⚠️ | `--format=json` parse envelope bug; 10k perf budget unverified |
| `knowledge-model.md` | ✅ | ⚠️ | ⚠️ | Context MOC ordering not preserved; quality/duplicate enforcement missing |
| `storage-format.md` | ✅ | ⚠️ | ⚠️ | Config store root + rewrite + collision guards missing |
| `cli-interface.md` | ✅ | ⚠️ | ⚠️ | `create` output missing path; tests missing for create alias/open/id, list tag/since/records, search opts |
| `indexing-search.md` | ✅ | ⚠️ | ⚠️ | Incremental indexing no-op; qp-link scope; related-notes missing |
| `semantic-graph.md` | ✅ | ⚠️ | ⚠️ | Context typed-link preference + doctor validation missing |
| `graph-traversal.md` | ✅ | ⚠️ | ⚠️ | `link path` weighted default; CSV flags missing; tests incomplete |
| `similarity-ranking.md` | ✅ | ⚠️ | ⚠️ | Default related threshold/opt-out missing; boosts hardcoded |
| `records-output.md` | ✅ | ⚠️ | ⚠️ | `prime` max-chars missing; link records omit path |
| `llm-context.md` | ✅ | ⚠️ | ⚠️ | Per-note truncation + prime budgeting missing; empty headers omitted |
| `llm-user-validation.md` | ✅ | ⚠️ | ⚠️ | Harness partial; guard/dry-run/gates/artifacts missing |
| `provenance.md` | ✅ | ⚠️ | ⚠️ | Verified defaults + web capture defaults missing |
| `export.md` | ✅ | ⚠️ | ⚠️ | Outline ordering + query cap; tests missing; optional formats |
| `compaction.md` | ✅ | ⚠️ | ⚠️ | Link outputs miss annotations; truncation flags ignored |
| `pack.md` | ✅ | ⚠️ | ⚠️ | Skip drops links; dump filters drop links; merge-links skips existing; path ignored |
| `workspaces.md` | ✅ | ⚠️ | ⚠️ | Rename strategy + graph-slice + post-merge checks missing |
| `structured-logging.md` | ✅ | ⚠️ | ⚠️ | Log-level gating/validation gaps; default warn |
| `operational-database.md` | ✅ | ⚠️ | ✅ | Startup repair/rebuild missing; DB not sole index |
| `value-model.md` | ✅ | ⚠️ | ⚠️ | Min-value validation missing; Dijkstra ordering bug; tests incomplete |
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
| `cli-tool.md` | JSON error envelope misses `--format=json` | `src/main.rs:82-93` |
| `operational-database.md` | DB-only index + repair + auto-rebuild gaps | `src/lib/store/query.rs:14-52`, `src/lib/db/mod.rs:84-85`, `src/lib/db/schema.rs:94-112` |
| `indexing-search.md` | `index --rebuild` no-op | `src/commands/index.rs:14-19` |
| `graph-traversal.md` | `link path` weighted default; missing CSV types flags | `src/commands/link/path.rs:71-95`, `src/cli/link.rs:73-79` |
| `pack.md` | `load --strategy skip` drops all links | `src/commands/load/mod.rs:77-84` |
| `pack.md` | `dump` filters can drop links between included notes | `src/commands/dump/mod.rs:225-246` |
| `records-output.md` | Link records omit `path=` | `src/commands/link/records.rs:65-71`, `src/commands/link/tree.rs:293-299` |
| `value-model.md` | Missing `--min-value` validation; Dijkstra ordering bug | `src/cli/commands.rs:136-141`, `src/lib/graph/bfs.rs:340-346` |
| `cli-interface.md` | `create` output missing path | `src/commands/create.rs:88-95` |
| `knowledge-model.md` | Context traversal ignores MOC ordering | `src/commands/context/select.rs:21-38` |
| `structured-logging.md` | Log-level gated by `--verbose` | `src/commands/dispatch/notes.rs:23-26` |
| `llm-user-validation.md` | Guard + dry-run missing | `crates/llm-tool-test/src/main.rs:1-53`, `crates/llm-tool-test/src/run.rs:90-93` |

### P2/P3: Missing Coverage or Features

| Spec | Gap | Notes |
| --- | --- | --- |
| `cli-tool.md` | Test coverage | Visible-store discovery + `--format=json` parse errors + more goldens |
| `cli-tool.md` | Test coverage | Performance budget coverage missing for 10k-note search |
| `cli-interface.md` | Test coverage | Create/list/search/compact flag variants missing |
| `indexing-search.md` | Feature gaps | qp-link scope + related-notes missing |
| `storage-format.md` | Feature gaps | Config store root + rewrite + collision guards missing |
| `semantic-graph.md` | Feature gaps | Typed-link preference + doctor validation missing |
| `graph-traversal.md` | Feature gaps/tests | CSV flags missing; inversion + truncation tests missing |
| `records-output.md` | Feature gaps/tests | `prime` max-chars missing; tree/path trunc tests missing |
| `llm-context.md` | Feature gaps | Per-note truncation + prime budgeting missing; empty headers omitted |
| `llm-user-validation.md` | Feature gaps | Schema/gates/budget/artifacts/tool-trait gaps |
| `provenance.md` | Feature gaps | Verified defaults + web capture defaults missing |
| `export.md` | Feature gaps/tests | Outline ordering + query cap; tests missing; optional formats |
| `compaction.md` | Feature gaps | Link annotations + truncation flags missing |
| `pack.md` | Feature gaps/tests | Merge-links + path gaps; selector/attachment tests missing |
| `workspaces.md` | Feature gaps/tests | Rename + graph-slice + post-merge checks missing |
| `structured-logging.md` | Feature gaps | Log-level validation + default warn |
| `similarity-ranking.md` | Feature gaps/tests | Default threshold + stemming opt-out; boosts hardcoded |
| `custom-metadata.md` | Unimplemented | Custom frontmatter + DB + CLI missing |
| `distribution.md` | Unimplemented | Release automation and install scripts missing |

### Not Applicable

| Spec | Reason |
| --- | --- |
| `telemetry.md` | DRAFT spec explicitly prohibits implementation |
| `knowledge-model.md` tag aliases | Marked as optional in spec |
