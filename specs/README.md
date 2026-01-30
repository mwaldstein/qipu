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
| [`telemetry.md`](telemetry.md) | Telemetry | DRAFT - usage analytics (not implemented) |

## Status Tracking

**Spec Status**: Is the specification complete and concrete enough to implement?
**Impl Status**: Is the implementation complete per the spec?
**Test Status**: Is test coverage adequate?

*Last audited: 2026-01-24*

| Spec | Spec | Impl | Tests | Notes |
| --- | --- | --- | --- | --- |
| `cli-tool.md` | ✅ | ✅ | ⚠️ | Missing performance tests for --help/--version, list, search |
| `knowledge-model.md` | ✅ | ✅ | ✅ | All note types, IDs, tags, typed links working; unknown types rejected (not coerced) |
| `storage-format.md` | ✅ | ✅ | ✅ | All features implemented with path traversal protection |
| `cli-interface.md` | ✅ | ⚠️ | ⚠️ | Exit code issue actually correct |
| `indexing-search.md` | ✅ | ✅ | ⚠️ | Field weights correct (2.0/1.5/1.0); search wraps query in quotes (phrase search) |
| `semantic-graph.md` | ✅ | ⚠️ | ⚠️ | `show --links` ignores `--no-semantic-inversion` flag; inversion tests sparse |
| `graph-traversal.md` | ✅ | ✅ | ✅ | Tree view correctly uses spanning_tree; hop limit is cost budget (spec ambiguity) |
| `similarity-ranking.md` | ✅ | ⚠️ | ⚠️ | Search uses additive boosts instead of multiplicative weights; wraps query in quotes |
| `records-output.md` | ✅ | ⚠️ | ⚠️ | `via` annotation missing in link JSON; missing truncation/S-prefix tests |
| `llm-context.md` | ✅ | ⚠️ | ⚠️ | Prime uses count-based limits (not character budget); `--max-tokens` flag out of scope |
| `llm-user-validation.md` | ✅ | ⚠️ | ⚠️ | Budget cost estimation inaccurate; budget warning doesn't enforce limits; events defined but not dispatched |
| `progressive-indexing.md` | ⚠️ | ❌ | New spec - not implemented |
| `provenance.md` | ✅ | ⚠️ | ⚠️ | Bibliography ignores `source` (singular), uses `sources[]` only; `source` vs `sources[]` ambiguous |
| `export.md` | ✅ | ✅ | ✅ | All features implemented; outline ordering uses wiki-links only (spec unclear on typed/markdown) |
| `compaction.md` | ✅ | ⚠️ | ✅ | Link JSON missing `via` annotation; truncation markers ARE present |
| `pack.md` | ✅ | ✅ | ✅ | Value/custom correctly preserved; merge-links restricted to newly loaded notes |
| `workspaces.md` | ✅ | ✅ | ⚠️ | Rename strategy link rewriting untested; metadata location per-workspace (spec ambiguous) |
| `structured-logging.md` | ✅ | ✅ | ⚠️ | Logs correctly route to stderr; missing TRACE tests; structured fields not validated |
| `operational-database.md` | ✅ | ⚠️ | ✅ | Consistency check result ignored (no auto-repair); no corruption detection/rebuild |
| `value-model.md` | ✅ | ✅ | ✅ | All features working; `ignore_value` default false (weighted by default) |
| `distribution.md` | ⚠️ | ⚠️ | ❌ | Install scripts work; release workflow disabled; SHA256SUMS format incorrect |
| `custom-metadata.md` | ✅ | ✅ | ✅ | Custom metadata fully implemented + tested |
| `telemetry.md` | DRAFT | ❌ | ❌ | Explicitly marked "DO NOT IMPLEMENT" |

## Legend

- ✅ Complete / Ready
- ⚠️ Partial / Has gaps
- ❌ Not implemented / No coverage

## Remaining Gaps

### P1: Correctness Bugs

| Spec | Gap | Notes |
| --- | --- | --- |
| `llm-context.md` | Prime uses count-based limits (5 notes) not character budget (~4-8k characters) | `src/commands/prime.rs:16-20` |
| `similarity-ranking.md` | Search wraps query in quotes (phrase search vs AND/OR semantics) | `src/lib/db/search.rs:47` |
| `similarity-ranking.md` | Search uses additive boosts instead of multiplicative field weights | `src/lib/db/search.rs:112-132` |
| `semantic-graph.md` | `show --links` ignores `--no-semantic-inversion` flag | `src/commands/show.rs:204-225` |
| `compaction.md` | Link JSON missing `via` annotation (breadcrumb for compacted sources) | `src/commands/link/json.rs:7-86`, `src/commands/link/mod.rs:31-45` |
| `provenance.md` | Bibliography ignores `source` field (singular), uses `sources[]` only | `src/commands/export/emit/bibliography.rs:35` |
| `operational-database.md` | Consistency check result ignored (no auto-repair) | `src/lib/db/mod.rs:96` |
| `operational-database.md` | No corruption detection and auto-rebuild | `src/lib/db/mod.rs:50-99` |
| `llm-user-validation.md` | Token usage uses char/4 approximation instead of parsing actual tool output | `crates/llm-tool-test/src/adapter/*.rs` |
| `llm-user-validation.md` | Budget warning doesn't enforce limits | `crates/llm-tool-test/src/run.rs:417-424` |
| `distribution.md` | Release workflow disabled with incorrect triggers (not `v*` tags) | `.github/workflows/release.yml:3-13` |
| `distribution.md` | SHA256SUMS file format incorrect (individual files instead of combined) | `.github/workflows/release.yml:99-152` |

### P2/P3: Missing Coverage or Features

| Spec | Gap | Notes |
| --- | --- | --- |
| `cli-tool.md` | Test coverage | Missing tests for duplicate `--format` detection; no performance tests |
| `storage-format.md` | Test coverage | Missing security tests for discovery boundary with parent store; malicious attachment paths |
| `cli-interface.md` | Test coverage | Missing tests asserting JSON schema compliance (required fields present) |
| `indexing-search.md` | Test coverage | Missing test for relative `.md` links cross-directory edge case; no direct 2-hop neighborhood CLI tests |
| `semantic-graph.md` | Test coverage | Missing tests for `show --links --no-semantic-inversion`; sparse inversion tests for context walk/dump |
| `graph-traversal.md` | Test coverage | Missing tests for max-fanout limit behavior; records format edge cases |
| `similarity-ranking.md` | Test coverage | Missing integration test for multi-word search queries; tests don't validate actual weight values |
| `records-output.md` | Test coverage | Missing tests for S prefix semantic distinction; truncation flags |
| `llm-context.md` | Test coverage | Missing tests for `qipu prime --format json/records`; missing-selection exit codes |
| `llm-context.md` | Out of scope | `--max-tokens` flag and tiktoken dependency should be removed (character-only budgets) |
| `pack.md` | Test coverage | Missing tests for `--tag`/`--moc`/`--query` selectors in dump; graph traversal options; link preservation |
| `workspaces.md` | Test coverage | Missing tests for rename strategy link rewriting; `--delete-source` flag |
| `structured-logging.md` | Test coverage | Missing TRACE level tests; structured field validation; span/trace relationship tests |
| `operational-database.md` | Test coverage | No FTS5 field weight tests; no performance benchmarks; no WAL concurrent read tests |
| `value-model.md` | Test coverage | Missing tests for compaction suggest + value; context `--min-value` edge cases; search sort-by-value defaults |
| `export.md` | Test coverage | Missing tests for outline with typed/markdown links; PDF edge cases; BibTeX/CSL-JSON edge cases |
| `compaction.md` | Test coverage | Missing `via` annotation tests for link commands; multi-level compaction chains |
| `provenance.md` | Test coverage | Missing bibliography test for `source` field; notes with both `source` and `sources[]` |
| `llm-user-validation.md` | Test coverage | Missing tests for transcript report; event logging; human review; CLI commands; LLM judge; link parsing |
| `distribution.md` | Test coverage | No install script tests; release workflow tests; checksum verification; version consistency; cross-platform binary tests |

### Not Applicable

| Spec | Reason |
| --- | --- |
| `telemetry.md` | DRAFT spec explicitly prohibits implementation |
| `knowledge-model.md` tag aliases | Marked as optional in spec |
