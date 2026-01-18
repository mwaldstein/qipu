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

*Last audited: 2026-01-18*

| Spec | Spec | Impl | Tests | Notes |
| --- | --- | --- | --- | --- |
| `cli-tool.md` | ✅ | ⚠️ | ⚠️ | Determinism/perf/hygiene partially asserted; verbose timing keys incomplete (`src/main.rs:64-66`) |
| `knowledge-model.md` | ✅ | ⚠️ | ✅ | ID length growth is collision-driven (`src/lib/id.rs:79-87`); tag aliases not implemented |
| `storage-format.md` | ✅ | ⚠️ | ⚠️ | Markdown relative path links only resolved if `qp-...` appears (`src/lib/index/links.rs:57-107`); `qipu.db` not implemented |
| `cli-interface.md` | ✅ | ⚠️ | ⚠️ | Some post-parse arg errors exit 1 not 2 (`src/commands/dispatch.rs:300-306`); `--max-chars` for link cmds records-only |
| `indexing-search.md` | ✅ | ⚠️ | ⚠️ | Recency boost missing; ripgrep path can miss title-only matches (`src/lib/index/search.rs:53-110`) |
| `semantic-graph.md` | ✅ | ⚠️ | ⚠️ | Custom type config schema differs from spec (`src/lib/config.rs:40-69`); no direct `inverse()` unit tests |
| `graph-traversal.md` | ✅ | ⚠️ | ⚠️ | “(seen)” refs not rendered; type filters + direction=in/both missing tests |
| `similarity-ranking.md` | ✅ | ⚠️ | ⚠️ | Stop words + stemming missing; similarity uses BM25-ish weights + tf=1 (`src/lib/similarity/mod.rs:133-137`) |
| `records-output.md` | ✅ | ⚠️ | ✅ | Records schema has extra prefixes (`W/D/C/M`) and `B-END` terminator |
| `llm-context.md` | ✅ | ⚠️ | ⚠️ | Prime bounded by counts not tokens/chars (`src/commands/prime.rs:14-19`); backlinks-in-context not implemented |
| `llm-user-validation.md` | ✅ | ⚠️ | ⚠️ | Harness exists but many spec features missing; tool default mismatch (`crates/llm-tool-test/src/cli.rs:22-25`) |
| `provenance.md` | ✅ | ⚠️ | ⚠️ | `create/capture --format json` omit provenance fields; `prompt_hash` not covered via CLI provenance tests |
| `export.md` | ✅ | ⚠️ | ⚠️ | MOC bundle ordering not honored; anchor rewriting likely broken (`src/commands/export/emit/links.rs:16-18`) |
| `compaction.md` | ✅ | ⚠️ | ⚠️ | JSON outputs omit compaction truncation flag (`src/commands/list.rs:88-97`); `compact apply/show/status` lack direct tests |
| `pack.md` | ✅ | ❌ | ⚠️ | `merge-links` semantics wrong (`src/commands/load/mod.rs:198`); dump filters inverted (`src/commands/dump/mod.rs:36-41`) |
| `workspaces.md` | ✅ | ⚠️ | ⚠️ | `--dry-run` lacks conflict report (`src/commands/workspace/merge.rs:82-84`); seeding is shallow (`src/commands/workspace/new.rs:60-71`) |
| `structured-logging.md` | ✅ | ⚠️ | ❌ | Tracing init + flags exist, but no span/event instrumentation; still many `eprintln!` callsites |

## Legend

- ✅ Complete / Ready
- ⚠️ Partial / Has gaps
- ❌ Not implemented / No coverage

## Detailed Gap Summary

### P1: Correctness Issues
- **pack.md**: `load --strategy merge-links` and dump `--typed-only/--inline-only` filtering do not match spec (`src/commands/load/mod.rs:198`, `src/commands/dump/mod.rs:36-41`)
- **workspaces.md**: `workspace merge --dry-run` lacks conflict report and prints success-like message (`src/commands/workspace/merge.rs:82-84`)
- **export.md**: anchor link rewriting likely broken (rewrites to `#note-<id>` without emitting anchors) (`src/commands/export/emit/links.rs:16-18`)
- **indexing-search.md**: ripgrep-based search can miss title-only matches (`src/lib/index/search.rs:53-110`)
- **cli-interface.md**: some invalid-arg errors return exit code 1 instead of 2 (`src/commands/dispatch.rs:300-306`)

### P2: Missing Test Coverage
- `graph-traversal.md`: type filters, typed-only/inline-only, direction=in/both
- `provenance.md`: `prompt_hash` via CLI create/capture
- `export.md`: MOC bundle ordering, anchor existence
- `structured-logging.md`: runtime logging behaviors

### P3: Future/Optional Items
- SQLite FTS5 (indexing-search)
- Stemming (similarity-ranking)
- Backlinks-in-context (llm-context)

### P4: Spec ambiguity / drift
- semantic-graph custom type config schema differs from spec
- records-output has extra record prefixes + `B-END` terminator
