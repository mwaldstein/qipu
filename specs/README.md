# Qipu Specifications

This directory contains **implementable** Qipu specifications. Each file should describe a specific, buildable topic of concern and be concrete enough to implement and test.

Project-level vision/goals live in the repo root `README.md`. Non-spec guidance/examples live under `docs/`.

## Planning Documents

- **[`../IMPLEMENTATION_PLAN.md`](../IMPLEMENTATION_PLAN.md)** - Concrete implementation tasks, bugs, and missing test coverage that need to be addressed
- **[`../FUTURE_WORK.md`](../FUTURE_WORK.md)** - Future work, exploratory features, and open questions from specs not yet in the implementation plan

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

*Last audited: 2026-01-23*

| Spec | Spec | Impl | Tests | Notes |
| --- | --- | --- | --- | --- |
| `cli-tool.md` | ✅ | ⚠️ | ⚠️ | Discovery stops at project roots; `--format json --help` behavior unclear |
| `knowledge-model.md` | ✅ | ⚠️ | ⚠️ | DB unknown note types coerced to `fleeting` |
| `storage-format.md` | ✅ | ⚠️ | ⚠️ | Discovery boundary vs spec; load attachment path guards missing |
| `cli-interface.md` | ✅ | ⚠️ | ⚠️ | Search/inbox JSON missing spec-minimum fields; context missing-selection exit code |
| `indexing-search.md` | ✅ | ⚠️ | ⚠️ | DB edges miss some relative `.md` links; related notes only via context |
| `semantic-graph.md` | ✅ | ⚠️ | ⚠️ | Inversion works; `show --links` ignores inversion flag; direct tests sparse |
| `graph-traversal.md` | ✅ | ⚠️ | ⚠️ | Tree view semantics differ; hop limit is a cost budget |
| `similarity-ranking.md` | ✅ | ⚠️ | ⚠️ | Field weights mismatch vs spec; search wraps query in quotes |
| `records-output.md` | ✅ | ⚠️ | ⚠️ | Link record headers not CWD-relative; prefix overlaps (`C`, `S`) |
| `llm-context.md` | ✅ | ⚠️ | ⚠️ | Context JSON missing per-note `path`; prime is count-based (not token budget) |
| `llm-user-validation.md` | ✅ | ⚠️ | ⚠️ | Events/budgets/dry-run behavior partial |
| `provenance.md` | ✅ | ⚠️ | ⚠️ | Spec lacks `sources[]`; bibliography ignores `source` |
| `export.md` | ✅ | ⚠️ | ✅ | Outline ordering uses wiki-links only |
| `compaction.md` | ✅ | ⚠️ | ✅ | Link JSON missing compaction annotations/truncation markers |
| `pack.md` | ✅ | ⚠️ | ⚠️ | Dump/load lossy (value/custom); merge-links semantics restricted |
| `workspaces.md` | ✅ | ⚠️ | ⚠️ | Metadata file location mismatch; rename strategy untested |
| `structured-logging.md` | ✅ | ⚠️ | ⚠️ | Logs appear on stdout; instrumentation not comprehensive |
| `operational-database.md` | ✅ | ⚠️ | ✅ | Consistency check doesn’t auto-repair; no corruption rebuild |
| `value-model.md` | ✅ | ⚠️ | ✅ | `link path` defaults unweighted; traversal hop is a cost budget |
| `distribution.md` | ⚠️ | ⚠️ | ❌ | Install scripts exist; release workflow disabled/manual; repo mismatch |
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
| `cli-tool.md` | Store discovery stops at project root markers | `src/lib/store/paths.rs:29-38`, `src/lib/store/paths.rs:98-120` |
| `cli-tool.md` | `--format json --help/--version` likely treated as error | `src/main.rs:32-41` |
| `structured-logging.md` | Logs show up on stdout (breaks machine output expectations) | `tests/cli/logging.rs:19-25`, `src/lib/logging.rs:33-40` |
| `cli-interface.md` | Search JSON omits spec-minimum note fields (`path/created/updated`) | `src/commands/search/format/json.rs:20-29` |
| `cli-interface.md` | Inbox JSON omits `path` | `src/commands/dispatch/notes.rs:160-177` |
| `cli-interface.md` | `context` missing-selection returns exit 1 (not usage exit 2) | `src/commands/context/mod.rs:443-446`, `src/lib/error.rs:95-101` |
| `knowledge-model.md` | DB reads coerce unknown `type` to `fleeting` | `src/lib/db/notes/read.rs:248-249`, `src/lib/db/search.rs:206-207` |
| `indexing-search.md` | DB edge insertion misses relative `.md` links (empty `path_to_id`) | `src/lib/db/edges.rs:13-22` |
| `graph-traversal.md` | Tree human view expands from `result.links` (not `spanning_tree`) | `src/commands/link/tree.rs:171-293` |
| `value-model.md` | `link path` defaults to `--ignore-value` despite spec “weighted by default” | `src/cli/link.rs:154-155` |
| `records-output.md` | Link records headers use store-root path (not CWD-relative) | `src/commands/link/records.rs:176-186` |
| `llm-context.md` | `context --format json` omits per-note `path` | `src/commands/context/json.rs:171-195` |
| `compaction.md` | Link JSON outputs omit compaction annotations/truncation markers | `src/commands/link/json.rs:7-86` |
| `pack.md` | Dump/load lossy (value/custom dropped; merge-links restricted) | `src/commands/dump/serialize.rs:107-148`, `src/commands/load/mod.rs:95-104`, `src/commands/load/mod.rs:245-246` |
| `distribution.md` | Release automation disabled + installers repo mismatch | `.github/workflows/release.yml:3-13`, `scripts/install.sh:13-16`, `Cargo.toml:4-12` |

### P2/P3: Missing Coverage or Features

| Spec | Gap | Notes |
| --- | --- | --- |
| `semantic-graph.md` | Test coverage | Add direct CLI tests for inversion + type filtering + `source=virtual` |
| `custom-metadata.md` | Test coverage | Add tests for `qipu list --custom ...` + doctor custom checks |
| `workspaces.md` | Test coverage | Add tests for `workspace merge --strategy rename` |
| `llm-user-validation.md` | Feature gap | Enforce per-run budgets (`LLM_TOOL_TEST_BUDGET_USD`, `cost.max_usd`) preflight |
| `export.md` | Spec alignment | Decide whether outline ordering must include typed/markdown links (impl uses wiki-links only) (`src/commands/export/emit/outline.rs:92-121`) |
| `provenance.md` | Spec clarification | Define `source` vs `sources[]` and bibliography behavior |
| `structured-logging.md` | Design/test gap | Decide stdout/stderr routing guarantees for machine output; add tests |
| `distribution.md` | External dependency | Enable Actions + tag-triggered releases; decide canonical repo slug |

### Not Applicable

| Spec | Reason |
| --- | --- |
| `telemetry.md` | DRAFT spec explicitly prohibits implementation |
| `knowledge-model.md` tag aliases | Marked as optional in spec |
