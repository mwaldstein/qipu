# Qipu Implementation Plan

This document tracks completed implementation work. For exploratory future work and open questions from specs, see [`FUTURE_WORK.md`](FUTURE_WORK.md).

## Status

- **Test baseline**: 757 tests pass (excludes performance tests)
- **Revision 2 complete**: 2026-01-23
- Related: [`specs/README.md`](specs/README.md) - Specification status tracking

---

## Revision 2 (2026-01-23)

### P1: Core Features

| Feature | Summary |
|---------|---------|
| **Machine-readable output** | `qipu value` and `qipu custom` support `--format json` and `--format records` |
| **show --format json** | Includes `value`; opt-in `--custom` flag includes custom metadata |
| **context selectors** | `--min-value` and `--custom-filter` work as standalone selectors |
| **custom-filter parsing** | Supports `key=value`, `key`, `!key`, `key>n`, `key>=n`, `key<n`, `key<=n`; multiple filters AND together |
| **Negative values** | `qipu custom set <id> <key> -100` works without `--` |
| **Clean JSON stdout** | Logging to stderr, ANSI disabled with `.with_ansi(false)` |
| **Budget truncation** | Notes truncated with `…[truncated]` marker instead of dropped entirely |
| **Search breadcrumbs** | `via=<id>` annotation when search hits compacted notes |

### P2: Code Quality & Refactoring

| Target | Result |
|--------|--------|
| `bfs_find_path` (400→59 lines) | Extracted BFS, Dijkstra, neighbor collection, path reconstruction helpers |
| `pricing.rs` module | Extracted model pricing from results.rs (-72 lines) |
| Gate evaluation | Created `GateEvaluator` trait; 10 validator functions (<17 lines each) |
| Output formatting | Created `src/commands/format/mod.rs` with shared helpers |
| Doctor command | `DoctorCheck` trait; mod.rs reduced to 116 lines (orchestration only) |
| Similarity engine | 82-line main module; field weights in `src/lib/index/weights.rs` |
| Test file splits | `export.rs` → 8 modules, `tree.rs` → 9 modules, `pack_tests.rs` → 4 modules |

### P2: LLM Tool Test Enhancements

| Feature | Summary |
|---------|---------|
| **Pre-existing test fixes** | Fixed 4 failing tests (MockAdapter CLI syntax, shlex parsing) |
| **Safety guard** | Requires `LLM_TOOL_TEST_ENABLED=1` to run |
| **Per-scenario timeout** | `run.timeout_secs` in YAML overrides CLI `--timeout` |
| **Store snapshot** | Copies `.qipu/` and `export.json` to transcript dir |

### P3: Optional Enhancements

| Feature | Summary |
|---------|---------|
| **qipu onboard** | Outputs minimal AGENTS.md snippet pointing to `qipu prime` |
| **Tag aliases** | `tag_aliases` config with bidirectional resolution; doctor warns on orphaned aliases |

---

## Revision 1 (2026-01-22)

### Correctness (P1)
- **CLI**: Fixed create output, JSON error envelopes
- **Database**: Removed filesystem fallbacks, auto-rebuild on schema mismatch
- **Graph**: Fixed Dijkstra min-heap ordering, default BFS for `link path`, CSV-style type filters
- **Pack**: Fixed `skip` strategy link handling, `merge-links` edge insertion
- **Value Model**: Added 0-100 validation, fixed default value handling
- **Workspaces**: Implemented `rename` strategy, graph-slice copy, post-merge validation

### Test Coverage (P1)
- Added 100+ new tests covering all CLI commands/flags per spec
- Golden determinism tests (context, search, inbox, show, link tree/path)

### Refactoring (P2)
- Split `bfs.rs` into `algos/` module
- Modularized similarity engine (tfidf, duplicates, tags)

### Features (P3)
- **Custom Metadata**: Full implementation with type detection, filtering, doctor checks
- **Distribution**: Release automation, cross-platform installers
- **Export**: BibTeX/CSL JSON, transitive traversal, pandoc/PDF integration
- **Semantic Graph**: Per-link-type hop costs
- **LLM Tool Test**: Report command, PTY fallback, structured event logging
- **Tags**: `qipu tags list` command with frequency counts

---

## Deferred Items

### Spec Clarification Needed

| Spec | Item |
|------|------|
| `indexing-search.md` | Backlink index storage - confirm stored vs derived |
| `workspaces.md` | `--temp` gitignore behavior |
| `telemetry.md` | DRAFT spec, explicitly prohibits implementation |

### Technical Debt

| Item | Priority |
|------|----------|
| File size monitoring | High - add CI check for files >500 lines |
| Function complexity monitoring | High - flag functions >100 lines |
| Model pricing externalization | Medium - move to config file |
| Output format consolidation | Medium - shared OutputFormatter trait |

---

## Notes

- **Schema version**: 6 (custom metadata column)
- **Store format version**: 1
