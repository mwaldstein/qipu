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
| `cli-tool.md` | ✅ | ⚠️ | ⚠️ | `--root` behavior untested (`src/cli/mod.rs:29-33`); verbose timing keys incomplete (`src/main.rs:64-66`) |
| `knowledge-model.md` | ✅ | ⚠️ | ✅ | Note type is a closed enum (`src/lib/note/types.rs:6-19`); tag aliases not implemented |
| `storage-format.md` | ✅ | ⚠️ | ⚠️ | Markdown links only resolved if `qp-...` appears (`src/lib/index/links.rs:57-107`); `qipu.db` not implemented |
| `cli-interface.md` | ✅ | ⚠️ | ⚠️ | Some post-parse arg errors exit 1 not 2 (`src/commands/dispatch.rs:300-306`); `load --format records` header diverges (`src/commands/load/mod.rs:113-136`) |
| `indexing-search.md` | ✅ | ⚠️ | ⚠️ | SQLite FTS5 not yet implemented (currently uses ripgrep/embedded search); see `operational-database.md` for migration plan |
| `semantic-graph.md` | ✅ | ⚠️ | ⚠️ | Custom type config schema differs from spec (`src/lib/config.rs:40-69`); "prefer typed links under budget" not implemented in context (`src/commands/context/select.rs:14-32`) |
| `graph-traversal.md` | ✅ | ⚠️ | ⚠️ | "(seen)" refs not rendered; `max_hops` doesn't set `truncated=true` (`src/lib/graph/traversal.rs:87-90`); type filters + direction=in/both missing tests |
| `similarity-ranking.md` | ✅ | ⚠️ | ⚠️ | Stop words + stemming missing; similarity uses BM25-weighted vectors with `tf=1` (`src/lib/similarity/mod.rs:33-37`) |
| `records-output.md` | ✅ | ⚠️ | ✅ | Schema drift: extra prefixes (`W/D/C/M`) + `B-END`; header field ordering differs across commands (`src/commands/context/output.rs:445-449`) |
| `llm-context.md` | ✅ | ⚠️ | ⚠️ | Human/JSON budgeting can violate `--max-chars` due to summary estimate vs full body output (`src/commands/context/budget.rs:97-103`); JSON lacks safety banner |
| `llm-user-validation.md` | ✅ | ⚠️ | ⚠️ | Harness exists but many spec features missing; tool default mismatch (`crates/llm-tool-test/src/cli.rs:22-25`); rubric schema mismatch (`crates/llm-tool-test/src/judge.rs:5-17`) |
| `provenance.md` | ✅ | ⚠️ | ⚠️ | `create/capture/context --format json` omit provenance fields (`src/commands/create.rs:52-63`, `src/commands/context/output.rs:18-42`) |
| `export.md` | ✅ | ⚠️ | ⚠️ | MOC bundle ordering not honored (`src/commands/export/mod.rs:101-103`); anchor rewriting likely broken (`src/commands/export/emit/links.rs:16-18`); attachments copied without link rewrite (`src/commands/export/mod.rs:164-167`) |
| `compaction.md` | ✅ | ⚠️ | ⚠️ | JSON outputs omit compaction truncation flag (`src/commands/list.rs:88-97`); `--expand-compaction` drops truncation reporting (`src/commands/context/output.rs:72-110`) |
| `pack.md` | ✅ | ❌ | ⚠️ | `merge-links` semantics wrong (`src/commands/load/mod.rs:198`); dump filters inverted (`src/commands/dump/mod.rs:36-41`); pack encoding depends on `--format` (`src/commands/dump/mod.rs:52-62`) |
| `workspaces.md` | ✅ | ⚠️ | ⚠️ | `--dry-run` lacks conflict report (`src/commands/workspace/merge.rs:82-84`); `--empty` ignored (`src/commands/workspace/new.rs:13-14`); overwrite can leave duplicate note files (`src/commands/workspace/merge.rs:89-107`) |
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
- **llm-context.md**: budgeting can be violated due to summary estimates while output emits full bodies (`src/commands/context/budget.rs:97-103`)
- **indexing-search.md**: SQLite FTS5 not yet implemented; ripgrep/embedded search still in use (see `operational-database.md`)
- **cli-interface.md**: some invalid-arg errors return exit code 1 instead of 2 (`src/commands/dispatch.rs:300-306`)

### P2: Missing Test Coverage
- `cli-tool.md`: `--root` behavior
- `graph-traversal.md`: type filters, typed-only/inline-only, direction=in/both
- `provenance.md`: `prompt_hash` via CLI create/capture
- `export.md`: MOC bundle ordering, anchor existence, attachment link validity
- `structured-logging.md`: runtime logging behaviors

### P1.5: SQLite Operational Database
- `operational-database.md`: SQLite as performance layer, JSON remains source of truth

### P3: Future/Optional Items
- Stemming (similarity-ranking)
- Backlinks-in-context (llm-context)

### P4: Spec ambiguity / drift
- semantic-graph custom type config schema differs from spec
- records-output has extra record prefixes + `B-END` terminator
