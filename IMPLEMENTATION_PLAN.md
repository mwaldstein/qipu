# Qipu Implementation Plan

For exploratory future work, see [`FUTURE_WORK.md`](FUTURE_WORK.md).

## Status

- **Test baseline**: 791 tests pass
- **Schema version**: 6 (custom metadata column)
- **Last audited**: 2026-01-24
- **Last CI check added**: function complexity (>100 lines)

---

## P1: Correctness Bugs

### cli-tool.md

- [ ] Store discovery stops at project roots instead of continuing to filesystem root
  - **Location**: `src/lib/store/paths.rs:97-102`
  - **Issue**: Implementation stops at project markers (`.git`, `Cargo.toml`) but spec requires continuing to filesystem root
  - **Impact**: Users cannot discover stores in parent directories beyond project boundaries

### storage-format.md

- [ ] Discovery boundary check order incorrect
  - **Location**: `src/lib/store/paths.rs:62-102`
  - **Issue**: Checks for `.qipu/` directories before checking project markers
  - **Impact**: May find parent store when at project root boundary without store

- [ ] Load attachment path traversal vulnerability
  - **Location**: `src/commands/load/mod.rs:476-477`
  - **Issue**: No validation that attachment names don't contain `../` sequences
  - **Impact**: Malicious pack files could write outside attachments directory
  - **Fix**: Add canonicalization and `starts_with()` validation before writing

### cli-interface.md

- [ ] Inbox JSON omits `path` field
  - **Location**: `src/commands/dispatch/notes.rs:160-181`
  - **Issue**: `path` field only included when `Some`, but spec requires as minimum field
  - **Impact**: Scripts expecting `path` in inbox JSON fail when absent

### llm-context.md

- [ ] Context JSON omits per-note `path` field
  - **Location**: `src/commands/context/json.rs:171-195`
  - **Issue**: JSON output builds note object without path field (human format includes it)
  - **Impact**: LLMs receive incomplete location information

- [ ] Prime command uses count-based limits instead of token budget
  - **Location**: `src/commands/prime.rs:16-20`
  - **Issue**: Uses `MAX_MOCS: usize = 5` and `MAX_RECENT_NOTES: usize = 5` (count-based)
  - **Spec requires**: "bounded size (target: ~1–2k tokens)"
  - **Fix**: Either implement token counting to ~1-2k tokens, or update spec

### similarity-ranking.md

- [ ] Search wraps query in quotes (phrase search instead of AND/OR semantics)
  - **Location**: `src/lib/db/search.rs:47`
  - **Issue**: `format!("\"{}\"", query.replace('"', "\"\""))` forces exact phrase search
  - **Impact**: Searching "rust programming" fails when terms appear separately

- [ ] Search uses additive boosts instead of multiplicative field weights
  - **Location**: `src/lib/db/search.rs:112-132`
  - **Issue**: Adds `+2.0` for title, `+3.0` for tags instead of using BM25 column weights
  - **Impact**: Distorted ranking; single tag match can outrank multiple body matches
  - **Fix**: Remove additive boosts, rely only on BM25 column weights (2.0x/1.5x/1.0x)

### records-output.md

- [ ] Link commands use store-relative paths instead of CWD-relative
  - **Location**: `src/commands/link/records.rs:103, 286`
  - **Issue**: Uses `meta.path` directly (relative to store) instead of CWD-relative
  - **Spec requires**: "path field should be relative to current working directory"
  - **Impact**: Path values incorrect when running from subdirectories

### semantic-graph.md

- [ ] `show --links` ignores `--no-semantic-inversion` flag
  - **Location**: `src/commands/show.rs:204-225`
  - **Issue**: Always shows raw backlinks (`direction="in"`) regardless of flag
  - **Expected**: With flag: show raw backlinks; without flag: show virtual inverted links (`direction="out"`)

### compaction.md

- [ ] Link JSON outputs omit `via` annotation
  - **Location**: `src/commands/link/json.rs:7-86`, `src/commands/link/mod.rs:31-45`
  - **Issue**: `LinkEntry` struct lacks `via` field
  - **Spec requires**: Optional breadcrumb when digest appears because compacted source was matched
  - **Impact**: Cannot distinguish "digest shown naturally" vs "digest shown because compacted note matched"

### provenance.md

- [ ] Bibliography ignores `source` field, uses `sources[]` only
  - **Location**: `src/commands/export/emit/bibliography.rs:35`
  - **Issue**: Only iterates `note.frontmatter.sources` (array), ignores singular `source` field
  - **Impact**: Notes created with `qipu capture --source` won't appear in bibliography exports
  - **Clarification needed**: Define `source` vs `sources[]` semantics

### operational-database.md

- [ ] Consistency check doesn't auto-repair on startup inconsistency
  - **Location**: `src/lib/db/mod.rs:96`
  - **Issue**: `validate_consistency()` result discarded with `let _ = ...`
  - **Spec requires**: "If inconsistent, trigger incremental repair"
  - **Impact**: External file changes cause silent inconsistency; user must manually run `qipu index`

- [ ] No corruption detection and auto-rebuild
  - **Location**: `src/lib/db/mod.rs:50-99` (Database::open)
  - **Issue**: No handling for corrupt database files
  - **Spec requires**: "If database operations fail, attempt to delete and rebuild automatically"
  - **Fix**: Wrap database operations with corruption detection and auto-rebuild

### llm-user-validation.md

- [ ] Budget cost estimation inaccurate
  - **Location**: `crates/llm-tool-test/src/run.rs:412`, `adapter/amp.rs:72-73`
  - **Issue**: Uses `len() / 4` character-based estimate instead of actual token count
  - **Impact**: Budget limits may be exceeded unexpectedly

- [ ] Budget warning doesn't enforce limits
  - **Location**: `crates/llm-tool-test/src/run.rs:417-424`
  - **Issue**: Only prints warning when cost exceeds budget, doesn't prevent run
  - **Impact**: Budget limits are not actually enforced

### distribution.md

- [ ] Release workflow disabled with incorrect triggers
  - **Location**: `.github/workflows/release.yml:3-13, 11-12`
  - **Issue**: Workflow triggers only on `workflow_dispatch`, not `v*` tags; commented as disabled
  - **Impact**: Automated releases don't work; manual intervention required

- [ ] SHA256SUMS file format incorrect
  - **Location**: `.github/workflows/release.yml:99-152`
  - **Issue**: Generates individual `.sha256` files instead of combined `SHA256SUMS`
  - **Impact**: Install scripts expect single combined file

### value-model.md

- [ ] No P1 bugs found - `ignore_value` default is `false` (weighted traversal enabled by default)

---

## P2: Technical Debt & Test Coverage

### cli-tool.md

- [ ] Missing tests for duplicate `--format` detection
- [ ] Missing performance tests for `--help`/`--version` (<100ms), `list` (~1k notes <200ms), `search` (~10k notes <1s)
- [ ] Missing determinism test coverage for all commands

### storage-format.md

- [ ] Missing security test for discovery boundary with parent store
- [ ] Missing security test for malicious attachment paths in `qipu load`

### cli-interface.md

- [ ] Missing tests asserting JSON schema compliance (all required fields present)
- [ ] Missing inbox JSON tests for notes without paths

### indexing-search.md

- [ ] Missing test for relative `.md` links cross-directory edge case
- [ ] No direct CLI tests for 2-hop neighborhoods

### semantic-graph.md

- [ ] Missing tests for `show --links --no-semantic-inversion`
- [ ] Sparse inversion tests for `context walk` and `dump` commands
- [ ] Missing integration tests for custom link costs affecting traversal

### graph-traversal.md

- [ ] Missing tests for max-fanout limit behavior
- [ ] Missing records format edge case tests (budget truncation, malformed output)

### similarity-ranking.md

- [ ] Missing integration test for multi-word search queries
- [ ] Tests don't validate actual weight values (2.0/1.5/1.0) in search ranking
- [ ] Missing tests for TF-IDF weights with real notes

### records-output.md

- [ ] Missing tests for CWD-relative path handling from subdirectories
- [ ] Missing tests for S prefix semantic distinction (summary vs sources)
- [ ] Missing truncation flag tests for prime/list/search/export
- [ ] Missing integration tests for "get index, then fetch bodies" workflow

### llm-context.md

- [ ] Missing tests for `qipu prime --format json` and `--format records`
- [ ] Missing tests for prime command missing-selection exit codes
- [ ] Missing tests for JSON `path` field presence

### pack.md

- [ ] Missing tests for `--tag`/`--moc`/`--query` selectors in dump
- [ ] Missing tests for graph traversal options (direction, max-hops, type filters)
- [ ] Missing tests verifying typed links survive dump/load roundtrip

### workspaces.md

- [ ] Missing tests for rename strategy link rewriting
- [ ] Missing tests for `--delete-source` flag

### structured-logging.md

- [ ] No tests for TRACE level behavior
- [ ] No tests validating structured field content in logs
- [ ] No span/trace relationship tests
- [ ] Missing error chain propagation tests

### operational-database.md

- [ ] No tests for corrupt DB recovery (feature not implemented)
- [ ] No tests for auto-repair trigger (feature not implemented)
- [ ] No explicit tests for FTS5 field weighting (2.0/1.5/1.0)
- [ ] No performance benchmark tests (<50ms search, <10ms backlinks, <100ms traversal)
- [ ] No tests for WAL mode concurrent read behavior
- [ ] No tests for schema rollback (forward version mismatch)

### value-model.md

- [ ] Missing tests for compaction suggest + value interaction
- [ ] Limited test coverage for `--min-value` in context
- [ ] Missing tests for search sort-by-value edge cases (default value 50)

### export.md

- [ ] Missing test for outline mode with typed frontmatter links
- [ ] Missing test for outline mode with markdown links
- [ ] Missing PDF edge case tests (outline mode, attachments, anchor links)
- [ ] Missing BibTeX/CSL-JSON edge case tests (non-standard URLs, missing fields)

### compaction.md

- [ ] Missing `via` annotation tests for `qipu link list` and `qipu link path`
- [ ] Missing multi-level compaction tests (digest1 → digest2 chains)

### provenance.md

- [ ] Missing bibliography test for notes with `source` field (singular)
- [ ] No test for notes with both `source` and `sources[]`

### llm-user-validation.md

- [ ] Missing tests for transcript `write_report()`
- [ ] Missing tests for event logging (`log_spawn`, `log_output`, `log_complete`)
- [ ] Missing tests for human review workflow (`update_human_review`, `load_pending_review`)
- [ ] Missing tests for CLI commands (entirely untested)
- [ ] Missing tests for LLM judge (`run_judge`)
- [ ] Missing link parsing edge case tests in `store_analysis`

### distribution.md

- [ ] No install script tests (`install.sh`, `install.ps1`)
- [ ] No release workflow tests (artifact generation)
- [ ] No checksum verification tests
- [ ] No version consistency tests (`qipu --version` matches git tag/Cargo.toml)
- [ ] No cross-platform binary tests

---

## P3: Unimplemented but Ready

### indexing-search.md

- [ ] Attachment content search (PDF, etc.) - Open question: include in search?

### value-model.md

- [ ] Resolve: Should value influence search ranking by default? (Currently only with `--sort value`)
- [ ] Resolve: Should `qipu compact suggest` factor in value?
- [ ] Resolve: Should digest notes automatically receive value boost?

### records-output.md

- [ ] Add `--format version=1` selector (currently hardcoded)
- [ ] Resolve: Should edges be included by default?
- [ ] Resolve: Should summaries be included by default?

### compaction.md

- [ ] Add MOC/spec note filter to `qipu compact suggest` (spec line 272)

### structured-logging.md

- [ ] Resource usage logging (memory, cache hits) - Infrastructure not present
- [ ] Error chain trace logging - needs design

### llm-context.md

- [ ] Resolve: Automatic summarization for long notes (open question in spec)
- [ ] Resolve: Should backlinks be included in context by default? (open question)

### export.md

- [ ] Resolve: Should outline ordering include typed/markdown links or only wiki-links?

---

## Blocked

| Item | Blocker |
|------|---------|
| Release workflow | GitHub Actions enablement |
| `telemetry.md` | DRAFT spec; prohibits implementation |
| Homebrew tap | Requires separate repository setup |
| crates.io publishing | Account setup and verification needed |

---

 ## Completed (Summary)

 **Revision 13** (2026-01-24): Comprehensive spec audit across 19 specifications. Corrected status for knowledge-model.md (unknown types rejected, not coerced), operational-database.md (field weights correct at 2.0/1.5/1.0), pack.md (value/custom correctly preserved), value-model.md (ignore_value default false = weighted by default), structured-logging.md (logs correctly route to stderr). Identified 17 P1 correctness bugs, 60+ P2 test coverage gaps. Updated IMPLEMENTATION_PLAN.md with categorized items, updated FUTURE_WORK.md with design questions, updated specs/README.md status table and gaps.

 **Revision 12** (2026-01-24): Shared compaction formatting functions across `search` and `list` commands. Created shared functions in `src/lib/format.rs` for building compaction annotations (`build_compaction_annotations`), outputting compaction IDs (`output_compaction_ids`), and adding compaction to JSON (`add_compaction_to_json`, `add_compacted_ids_to_json`). Updated both commands' format modules (human, json, records) to use these shared functions, eliminating ~70 lines of duplicated code.

 **Revision 11** (2026-01-24): Externalized model pricing to config. Added `llm-tool-test-config.example.toml` template file with default pricing for Claude, OpenAI, and Amp models. Updated documentation to explain configuration process.

 **Revision 10** (2026-01-24): Aligned `qipu update` output format with `qipu create`. JSON now includes all fields (`created`, `updated`, `source`, `author`, `generated_by`, `prompt_hash`, `verified`). Human output simplified to just ID. Records output now includes header line.

**Revision 9** (2026-01-23): Refactored `src/commands/doctor/database.rs` (723→684 lines). Extracted helper functions (`get_note_path`, `report_semantic_link_misuse`, `check_self_referential_link`) to eliminate repeated patterns in `check_semantic_link_types`, `check_follows_cycles`, and `check_orphaned_notes`.

**Revision 8** (2026-01-23): Refactored `src/commands/setup.rs` (781→710 lines). Extracted test helper functions (`assert_unknown_tool_error`, `setup_agents_md`, `assert_install_success`, `assert_execute_ok`) to eliminate code duplication across 24 test functions.

**Revision 7** (2026-01-23): Refactored `src/commands/doctor/content.rs` (724→723 lines). Added `Note::id_string()` and `Note::path_display()` helper methods to eliminate repeated code patterns across multiple check functions.

**Revision 6** (2026-01-23): Refactored `src/lib/graph/bfs.rs` (842→820 lines). Extracted helper functions (`get_note_value`, `canonicalize_with_context`) to eliminate code duplication in neighbor processing loops across `bfs_search` and `dijkstra_search`.

**Revision 5** (2026-01-23): Added CI check for function complexity (>100 lines) with grandfathering for existing violations.

**Revision 4** (2026-01-23): Added CI check for file size (>500 lines) with grandfathering for existing violations.

**Revision 3** (2026-01-23): Spec audit - 14 correctness fixes, 4 test coverage items, 6 new features including `qipu context --walk`, `qipu store stats`, and custom metadata.

**Revision 2** (2026-01-23): Machine-readable output for `value`/`custom`, budget truncation, search breadcrumbs, major refactoring (bfs_find_path 400→59 lines, DoctorCheck trait).

**Revision 1** (2026-01-22): Core correctness fixes (Dijkstra ordering, JSON envelopes, value validation), 100+ new tests, custom metadata, export improvements.

See git history for full details.
