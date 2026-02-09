# Qipu Specifications

This directory contains **implementable** Qipu specifications. Each file should describe a specific, buildable topic of concern and be concrete enough to implement and test.

Project-level vision/goals live in the repo root `README.md`. Non-spec guidance/examples live under `docs/`.

## Planning Documents

- Planning is tracked in beads. Use `bd list`, `bd ready`, and `bd search`.

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
| [`progressive-indexing.md`](progressive-indexing.md) | Indexing | Progressive re-indexing for large knowledge bases |
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
| [`telemetry.md`](telemetry.md) | Telemetry | Approved - local collection ready, endpoint pending |

## Status Tracking

**Spec Status**: Is the specification complete and concrete enough to implement?
**Impl Status**: Is the implementation complete per the spec?
**Test Status**: Is test coverage adequate?

*Last audited: 2026-02-09*

| Spec | Spec | Impl | Tests | Notes |
| --- | --- | --- | --- | --- |
| `cli-tool.md` | ✅ | ✅ | ✅ | All tests implemented including performance tests |
| `knowledge-model.md` | ✅ | ✅ | ⚠️ | All features working; missing MOC link validation (doctor should warn on empty MOCs) |
| `storage-format.md` | ✅ | ✅ | ✅ | All features implemented with path traversal protection |
| `cli-interface.md` | ✅ | ✅ | ✅ | Exit codes correct per spec |
| `indexing-search.md` | ✅ | ✅ | ✅ | Field weights correct (2.0/1.5/1.0); AND semantics working |
| `semantic-graph.md` | ✅ | ✅ | ✅ | `show --links` correctly handles `--no-semantic-inversion`; tests complete |
| `graph-traversal.md` | ✅ | ✅ | ✅ | Tree view correctly uses spanning_tree; hop limit is cost budget (spec ambiguity) |
| `similarity-ranking.md` | ✅ | ✅ | ✅ | BM25 multiplicative weights correct; AND semantics working |
| `records-output.md` | ✅ | ✅ | ⚠️ | `via` annotation present; missing truncation/S-prefix tests |
| `llm-context.md` | ✅ | ✅ | ✅ | Character budgeting implemented (4000-8000 chars); tests complete; `--max-tokens` intentionally removed |
| `llm-user-validation.md` | ✅ | ✅ | ✅ | **MOVED**: Implementation moved to standalone [llm-tool-test](https://github.com/mwaldstein/llm-tool-test) project |
| `progressive-indexing.md` | ✅ | ⚠️ | ✅ | Core features (quick, resume, progress, selective) implemented; missing --basic/--full flags, --modified-since |
| `provenance.md` | ✅ | ✅ | ✅ | Bibliography correctly handles both `source` (singular) and `sources[]` |
| `export.md` | ✅ | ✅ | ✅ | All features implemented; outline ordering uses wiki-links only (spec unclear on typed/markdown) |
| `compaction.md` | ✅ | ✅ | ✅ | Link JSON includes `via` annotation; truncation markers ARE present |
| `pack.md` | ✅ | ✅ | ✅ | Value/custom correctly preserved; merge-links restricted to newly loaded notes |
| `workspaces.md` | ✅ | ✅ | ✅ | All features implemented; link rewriting tested; file reference handling only gap |
| `structured-logging.md` | ✅ | ✅ | ✅ | Core logging complete; all 5 levels supported; resource metrics P4 feature |
| `operational-database.md` | ✅ | ✅ | ✅ | All features implemented; corruption detection + auto-rebuild working; auto-repair triggers incremental repair |
| `value-model.md` | ✅ | ✅ | ✅ | All features working; `ignore_value` default false (weighted by default) |
| `distribution.md` | ✅ | ⚠️ | ✅ | Release workflow + install scripts work; Homebrew formula outdated (v0.2.43 vs 0.3.0); needs tap repo creation |
| `custom-metadata.md` | ✅ | ✅ | ✅ | Custom metadata fully implemented + tested |
| `telemetry.md` | ✅ | ✅ | ✅ | Local collection complete; `telemetry show` implemented; remote endpoint stubbed pending infrastructure |

## Legend

- ✅ Complete / Ready
- ⚠️ Partial / Has gaps
- ❌ Not implemented / No coverage

## Remaining Gaps

### P1: Correctness Bugs

| Spec | Gap | Notes |
| --- | --- | --- |
| `similarity-ranking.md` | ✅ FIXED: Search correctly uses AND semantics (unquoted query) | Test: `tests/cli/search/basic.rs:407` |
| `similarity-ranking.md` | ✅ FIXED: Search correctly uses multiplicative field weights via BM25 | `crates/qipu-core/src/index/weights.rs` |
| `semantic-graph.md` | ✅ FIXED: `show --links` correctly respects `--no-semantic-inversion` flag | Tests: `tests/cli/show/semantic_inversion.rs` |
| `compaction.md` | ✅ FIXED: Link JSON correctly includes `via` annotation | Tests: `tests/cli/link/via_traversal.rs`, `tests/cli/link/path.rs` |
| `provenance.md` | ✅ FIXED: Bibliography now handles both `source` (singular) and `sources[]` | Tests: `tests/cli/export/bibliography.rs:325,359` |
| `operational-database.md` | ✅ FIXED: Consistency check result now triggers incremental repair when auto_repair is enabled | `crates/qipu-core/src/db/mod.rs:138-141` |
| `operational-database.md` | ✅ FIXED: Corruption detection and auto-rebuild implemented with tests | `crates/qipu-core/src/db/mod.rs:46-91` |
| `llm-user-validation.md` | Token usage uses char/4 approximation instead of parsing actual tool output | External: `llm-tool-test/src/adapter/*.rs` |
| `llm-user-validation.md` | Budget warning doesn't enforce limits | External: `llm-tool-test/src/run.rs:417-424` |
| `distribution.md` | Homebrew formula outdated (v0.2.43 vs current 0.3.0) | `distribution/homebrew-qipu/Formula/qipu.rb:4-5` |


### P2/P3: Missing Coverage or Features

| Spec | Gap | Notes |
| --- | --- | --- |
| `cli-tool.md` | ✅ Test coverage complete | Tests: `tests/cli/misc.rs`, `tests/performance_tests.rs` |
| `storage-format.md` | Test coverage | Missing security tests for discovery boundary with parent store; malicious attachment paths |
| `cli-interface.md` | Test coverage | Missing tests asserting JSON schema compliance (required fields present) |
| `indexing-search.md` | Test coverage | Missing test for relative `.md` links cross-directory edge case |
| `semantic-graph.md` | Test coverage | Sparse inversion tests for context walk/dump |
| `graph-traversal.md` | path command limits | --max-nodes, --max-edges, --max-fanout not wired in `dispatch/link.rs:114-128` |
| `similarity-ranking.md` | Test coverage | Missing integration test for multi-word search queries |
| `records-output.md` | ✅ Test coverage complete | S-prefix tests exist; via tests exist; truncation tests exist |
| `llm-context.md` | ✅ Test coverage complete | Tests for prime/context all formats, custom filters, budgets |
| `pack.md` | ✅ Test coverage complete | 39 tests covering all selectors, strategies, traversal |
| `workspaces.md` | ✅ Test coverage complete | 43 tests covering all commands and strategies |
| `structured-logging.md` | ✅ Test coverage complete | 17 CLI tests for all log levels and JSON output |
| `operational-database.md` | Test coverage | No FTS5 field weight tests; no WAL concurrent read tests |
| `value-model.md` | ✅ Test coverage complete | 55+ tests covering all value features |
| `export.md` | ✅ Test coverage complete | 19 test files, ~2878 lines covering all modes/formats |
| `compaction.md` | ✅ Test coverage complete | 14 test files covering all commands and annotations |
| `provenance.md` | ✅ Test coverage complete | 25+ tests across bibliography, formats, CLI |
| `custom-metadata.md` | ✅ Test coverage complete | 53 tests covering all commands and formats |
| `progressive-indexing.md` | Missing features | --basic/--full flags, --modified-since, --batch override |
| `distribution.md` | Infrastructure | Homebrew tap repository creation needed on GitHub |

### Not Applicable

| Spec | Reason |
| --- | --- |
| `knowledge-model.md` tag aliases | Optional per spec - implemented but not required |

### New Gaps Identified (2026-02-09 Audit)

| Spec | Gap | Priority | Notes |
| --- | --- | --- | --- |
| `knowledge-model.md` | MOC link validation | P3 | Doctor should warn when MOCs have zero links |
| `distribution.md` | Homebrew formula update | P3 | Formula at v0.2.43, needs v0.3.0 + SHA256 + tap repo creation |
| `progressive-indexing.md` | --basic/--full flags | P3 | Explicit two-level indexing not exposed in CLI |
| `progressive-indexing.md` | --modified-since flag | P3 | Selective indexing by modification time not implemented |
| `records-output.md` | S-prefix semantic tests | P2 | Tests exist but spec documentation gap on mode-specific semantics |
| `structured-logging.md` | Resource metrics | P4 | Memory/cache metrics not implemented (spec open question) |
| `workspaces.md` | File reference integrity | P3 | External file links in note body not rewritten on workspace operations |
| `graph-traversal.md` | path command limits | P3 | --max-nodes, --max-edges, --max-fanout not passed to path command |
