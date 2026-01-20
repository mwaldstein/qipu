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
| [`value-model.md`](value-model.md) | Scoring | Note value (0-100), weighted traversal |
| [`distribution.md`](distribution.md) | Release | Binary builds, installers, crates.io |
| [`telemetry.md`](telemetry.md) | Telemetry | DRAFT - usage analytics (not implemented) |

## Status Tracking

**Spec Status**: Is the specification complete and concrete enough to implement?
**Impl Status**: Is the implementation complete per the spec?
**Test Status**: Is test coverage adequate?

*Last audited: 2026-01-20*

| Spec | Spec | Impl | Tests | Notes |
| --- | --- | --- | --- | --- |
| `cli-tool.md` | ✅ | ✅ | ✅ | All flags; timing keys; performance budgets |
| `knowledge-model.md` | ✅ | ✅ | ✅ | Closed enum; all fields; tag aliases optional |
| `storage-format.md` | ✅ | ✅ | ✅ | All directories; frontmatter; stealth mode |
| `cli-interface.md` | ✅ | ✅ | ✅ | All 16+ commands with correct exit codes |
| `indexing-search.md` | ✅ | ✅ | ✅ | FTS5; BM25; backlinks; incremental repair |
| `semantic-graph.md` | ✅ | ⚠️ | ✅ | Missing: context link type preference |
| `graph-traversal.md` | ✅ | ✅ | ✅ | All directions; type filters; truncation |
| `similarity-ranking.md` | ✅ | ✅ | ✅ | BM25; TF-IDF; duplicate detection |
| `records-output.md` | ✅ | ✅ | ⚠️ | All prefixes; missing: export records test |
| `llm-context.md` | ✅ | ✅ | ✅ | Budget; transitive; backlinks; safety banner |
| `llm-user-validation.md` | ✅ | ⚠️ | ⚠️ | Harness works; missing: tags, docs.prime, report |
| `provenance.md` | ✅ | ✅ | ✅ | All 5 fields; verify command; context priority |
| `export.md` | ✅ | ✅ | ⚠️ | Core complete; missing: bibliography, tag, query tests |
| `compaction.md` | ✅ | ✅ | ⚠️ | All commands; missing: guide command test |
| `pack.md` | ✅ | ✅ | ⚠️ | All strategies; missing: selector tests |
| `workspaces.md` | ✅ | ✅ | ⚠️ | Merge works; missing: strategy tests |
| `structured-logging.md` | ✅ | ⚠️ | ✅ | Tracing works; missing: some instrumentation |
| `operational-database.md` | ✅ | ✅ | ✅ | SQLite; FTS5; schema; incremental repair |
| `value-model.md` | ✅ | ⚠️ | ⚠️ | P1: tree/path use BFS not Dijkstra; missing: --ignore-value |
| `distribution.md` | ✅ | ⚠️ | ⚠️ | Cargo.toml ready; missing: release workflow, aarch64, installers |
| `telemetry.md` | DRAFT | ❌ | ❌ | Explicitly marked "DO NOT IMPLEMENT" |

## Legend

- ✅ Complete / Ready
- ⚠️ Partial / Has gaps
- ❌ Not implemented / No coverage

## Remaining Gaps

### P1: Correctness Bugs

| Spec | Issue | Reference |
| --- | --- | --- |
| `value-model.md` | `link tree`/`link path` use BFS instead of weighted Dijkstra | `src/commands/link/tree.rs:61`, `src/commands/link/path.rs:75` |
| `semantic-graph.md` | Context budget doesn't prefer typed links over `related` | `src/commands/context/budget.rs` |

### P2: Missing Test Coverage

| Spec | Gap | Reference |
| --- | --- | --- |
| `workspaces.md` | --dry-run, strategy tests | `tests/cli/workspace.rs` |
| `export.md` | bibliography, --tag, --query, --link-mode markdown | `tests/cli/export.rs` |
| `pack.md` | --tag, --moc, --query, --no-attachments | `tests/cli/pack.rs` |
| `compaction.md` | guide command test | `tests/cli/compact/commands.rs` |
| `value-model.md` | value set/show validation, doctor check | `tests/cli/` |

### P3: Optional / Low Priority

| Spec | Gap | Notes |
| --- | --- | --- |
| `value-model.md` | `--ignore-value` flag | Infrastructure exists, needs CLI exposure |
| `structured-logging.md` | Instrumentation gaps | Index/search/note ops need `#[tracing::instrument]` |
| `llm-user-validation.md` | Schema extensions | tags, docs.prime, report command |
| `distribution.md` | Release automation | Workflow, aarch64, installers |

### Not Applicable

| Spec | Reason |
| --- | --- |
| `telemetry.md` | DRAFT spec explicitly prohibits implementation |
| `knowledge-model.md` tag aliases | Marked as optional in spec |
