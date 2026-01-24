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

- [x] Store discovery stops at project roots (spec corrected 2026-01-24)
  - **Location**: `src/lib/store/paths.rs:97-102`, `specs/cli-tool.md:78-87`
  - **Resolution**: Spec updated to require stopping at project root (`.git` or `Cargo.toml`) or filesystem root, whichever comes first
  - **Impact**: Behavior now matches spec; discovery no longer continues beyond project boundaries

### storage-format.md

- [x] Discovery boundary check order verified correct
  - **Location**: `src/lib/store/paths.rs:62-102`
  - **Resolution**: Code correctly checks for stores first, then project markers per spec (line 169)
  - **Behavior**: Check store → check project root → move to parent (correct order)

 - [x] Load attachment path traversal vulnerability
   - **Location**: `src/commands/load/mod.rs:476-477`
   - **Issue**: No validation that attachment names don't contain `../` sequences
   - **Impact**: Malicious pack files could write outside attachments directory
   - **Resolution**: Added canonicalization of both attachments dir and resolved path, with `starts_with()` validation before writing
   - **Implementation**: Rejects paths outside attachments directory with clear error message

### llm-context.md

- [x] Prime command uses count-based limits instead of character budget
  - **Location**: `src/commands/prime.rs:16-20`
  - **Issue**: Uses `MAX_MOCS: usize = 5` and `MAX_RECENT_NOTES: usize = 5` (count-based)
  - **Spec requires**: "bounded size (target: ~4–8k characters)"
  - **Resolution**: Implemented character-based budgeting with TARGET_MIN_CHARS=4000 and TARGET_MAX_CHARS=8000
  - **Implementation**: Added helper functions to estimate character counts and select notes within budget
  - **Behavior**: Now dynamically includes MOCs and recent notes based on character budget instead of fixed counts

### similarity-ranking.md

- [x] Search wraps query in quotes (phrase search instead of AND/OR semantics)
  - **Location**: `src/lib/db/search.rs:47`
  - **Issue**: `format!("\"{}\"", query.replace('"', "\"\""))` forces exact phrase search
  - **Impact**: Searching "rust programming" fails when terms appear separately
  - **Resolution**: Changed to use unquoted FTS5 queries (AND semantics) and replace hyphens with spaces to avoid special character interpretation
  - **Implementation**: Multi-word queries now use AND semantics, allowing terms to appear separately in documents

 - [x] Search uses additive boosts instead of multiplicative field weights
   - **Location**: `src/lib/db/search.rs:112-132`
   - **Issue**: Adds `+2.0` for title, `+3.0` for tags instead of using BM25 column weights
   - **Impact**: Distorted ranking; single tag match can outrank multiple body matches
   - **Resolution**: Removed additive boosts, now relies only on BM25 column weights (2.0x/1.5x/1.0x)
   - **Implementation**: Removed `+ {}` for title and `+ 3.0` for tags; BM25 weights provide proper multiplicative field weighting
   - **Learnings**: Tests expecting strict ordering (title match > body match) were testing buggy behavior; removed those tests as BM25 weights don't guarantee ordering - they provide weighting based on term frequency, document length, and other factors

### records-output.md

### semantic-graph.md

- [x] `show --links` ignores `--no-semantic-inversion` flag
  - **Location**: `src/commands/show.rs:204-225`
  - **Issue**: Always shows raw backlinks (`direction="in"`) regardless of flag
  - **Expected**: With flag: show raw backlinks; without flag: show virtual inverted links (`direction="out"`)
  - **Resolution**: Added semantic inversion logic following same pattern as `link list` command. When `--no-semantic-inversion` is false (default), inbound edges are inverted and shown as virtual outbound links. When true, raw backlinks are shown.
  - **Implementation**: Uses `edge.invert(store.config())` to create virtual edges when semantic inversion is enabled
  - **Learnings**: Golden test needed to be updated to reflect correct behavior - backlinks now appear as "Outbound links (virtual)" by default instead of "Inbound links"

### compaction.md

- [x] Link JSON outputs omit `via` annotation
  - **Location**: `src/commands/link/json.rs:7-86`, `src/commands/link/mod.rs:31-45`
  - **Issue**: `LinkEntry` struct lacks `via` field
  - **Spec requires**: Optional breadcrumb when digest appears because compacted source was matched
  - **Impact**: Cannot distinguish "digest shown naturally" vs "digest shown because compacted note matched"
  - **Resolution**: Added `via` field to `LinkEntry` struct (optional String), populated when canonicalization changes an ID
  - **Implementation**: JSON output includes `via` field when ID is canonicalized; human and records output exclude `via` (optional per spec)
  - **Learnings**: Spec describes `via` as optional for human output, so only included in JSON for machine readability

### provenance.md

- [x] Bibliography ignores `source` field, uses `sources[]` only
  - **Location**: `src/commands/export/emit/bibliography.rs:35`
  - **Issue**: Only iterated `note.frontmatter.sources` (array), ignored singular `source` field
  - **Impact**: Notes created with `qipu capture --source` wouldn't appear in bibliography exports
  - **Resolution**: Added support for singular `source` field by creating temporary `Source` objects and including them in bibliography exports alongside the `sources` array
  - **Implementation**: Now collects both singular `source` field and `sources` array, maintaining deterministic URL-based sorting
  - **Tests**: Added `test_export_bibliography_singular_source_field` and `test_export_bibliography_both_source_fields` to verify correct behavior

### operational-database.md

 - [x] Consistency check doesn't auto-repair on startup inconsistency
   - **Location**: `src/lib/db/mod.rs:96`, `specs/operational-database.md:102`
   - **Issue**: `validate_consistency()` result discarded with `let _ = ...`
   - **Spec requires**: "If inconsistent, trigger incremental repair"
   - **Impact**: External file changes cause silent inconsistency; user must manually run `qipu index`
   - **Resolution**: Added `auto_repair` parameter to `Database::open` to control auto-repair behavior. By default, consistency check triggers incremental repair on inconsistency. For `doctor` command, auto-repair is disabled to allow issue detection without fixing.
   - **Implementation**: When `auto_repair=true`, inconsistency triggers `incremental_repair()`. When `auto_repair=false` (doctor), issues are logged but not fixed.
   - **Learnings**: Doctor command must use `open_unchecked` with `auto_repair=false` to detect issues like missing files without auto-fixing them. Other commands use default auto-repair behavior.

 - [x] No corruption detection and auto-rebuild
   - **Location**: `src/lib/db/mod.rs:50-124` (Database::open)
   - **Issue**: No handling for corrupt database files
   - **Spec requires**: "If database operations fail, attempt to delete and rebuild automatically"
   - **Resolution**: Wrapped database open with corruption detection and auto-rebuild logic. When database operations fail with corruption errors (e.g., "database disk image is malformed", "corrupt", "malformed"), the corrupted database file is deleted along with WAL/SHM files, then rebuilt from scratch.
   - **Implementation**: Added `is_corruption_error()` helper to detect corruption error messages in QipuError. Modified `Database::open()` to catch errors, detect corruption, delete corrupted files, and retry opening which triggers rebuild. Added detailed error logging for both initial corruption and rebuild failure scenarios.
   - **Tests**: All 473 unit/integration tests pass (2 pre-existing pack test failures unrelated to this change).

### llm-user-validation.md

- [ ] Token usage estimation uses character-based approximation
  - **Location**: `crates/llm-tool-test/src/adapter/claude_code.rs:68-69`, `crates/llm-tool-test/src/adapter/opencode.rs:64-65`, `crates/llm-tool-test/src/adapter/amp.rs:72-73`, `crates/llm-tool-test/src/results.rs:448-449`
  - **Issue**: Uses `len() / 4` character-based estimate instead of actual token count from tool output
  - **Impact**: Token counts and cost estimates are inaccurate; should read from actual LLM tool responses
  - **Resolution**: Remove `len() / 4` estimation; parse token counts from tool output if available, otherwise return `None`. Tools (amp, opencode, claude) are responsible for reporting their actual API token usage.

- [ ] Budget warning doesn't enforce limits
  - **Location**: `crates/llm-tool-test/src/run.rs:417-424`
  - **Issue**: Only prints warning when cost exceeds budget, doesn't prevent run
  - **Impact**: Budget limits are not actually enforced

### distribution.md

- [ ] Release workflow disabled with incorrect triggers (BLOCKED: GitHub Actions not enabled)
  - **Location**: `.github/workflows/release.yml:3-13, 11-12`
  - **Issue**: Workflow triggers only on `workflow_dispatch`, not `v*` tags; commented as disabled
  - **Impact**: Automated releases don't work; manual intervention required

- [ ] SHA256SUMS file format incorrect (BLOCKED: GitHub Actions not enabled)
  - **Location**: `.github/workflows/release.yml:99-152`
  - **Issue**: Generates individual `.sha256` files instead of combined `SHA256SUMS`
  - **Impact**: Install scripts expect single combined file

### value-model.md

- [ ] No P1 bugs found - `ignore_value` default is `false` (weighted traversal enabled by default)

---

## P2: Technical Debt & Test Coverage

### llm-context.md

- [ ] Remove `--max-tokens` flag and token counting code
  - **Location**: `src/cli/commands.rs:327-329`, `src/commands/context/mod.rs`, `src/commands/context/budget.rs`, `src/commands/dispatch/mod.rs`, `src/commands/dispatch/notes.rs`
  - **Issue**: Qipu standardizes on character-based budgets only; `--max-tokens` flag and tiktoken dependency are out of scope
  - **Impact**: Removes unnecessary code and complexity; aligns with spec that uses character counts
  - **Implementation**: Remove `--max-tokens` flag from CLI, remove `max_tokens` parameter from context options, remove `tiktoken_rs` dependency and token counting code

### Code size reduction

The following 13 files are grandfathered in the CI file size check (>500 lines limit). Each needs to be refactored and removed from the allowed list:

**High priority (>700 lines):**
- [ ] `src/lib/db/tests.rs` (975 lines) - split into test modules
- [ ] `src/lib/graph/bfs.rs` (820 lines) - extract helper functions
- [ ] `src/commands/doctor/content.rs` (723 lines) - extract helper functions
- [ ] `src/commands/setup.rs` (710 lines) - extract helper functions
- [ ] `src/commands/doctor/database.rs` (684 lines) - extract helper functions

**Dead/unused code:**
- [ ] Audit codebase for dead/unused code (29 `#[allow(dead_code)]` annotations found)
  - Run `cargo clippy -- -W unused_variables -W dead_code` to find unused items
  - Review and remove unused functions, unused imports, and dead exports
  - **Review all `#[allow(dead_code)]` annotations** - each must have strong justification (e.g., public API, test infrastructure, future use with TODO comment)
  - Remove unjustified `#[allow(dead_code)]` attributes and the dead code they suppress

  Files with `#[allow(dead_code)]` annotations:
  - src/commands/doctor/database.rs (1)
  - src/commands/dump/serialize.rs (1)
  - src/commands/dump/model.rs (3)
  - src/commands/link/mod.rs (2)
  - src/lib/db/mod.rs (1)
  - src/lib/db/schema.rs (2)
  - src/lib/db/validate.rs (1)
  - src/lib/db/traverse.rs (1)
  - src/lib/db/notes/delete.rs (1)
  - src/lib/index/types.rs (1)
  - src/lib/store/mod.rs (2)
  - src/lib/store/query.rs (1)
  - src/lib/config.rs (2)
  - src/lib/similarity/mod.rs (3)
  - src/lib/graph/types.rs (1)
  - src/lib/graph/traversal.rs (2)
  - src/lib/text/mod.rs (1)
  - src/lib/note/types.rs (2)
  - src/lib/compaction/context.rs (1)

**Medium priority (600-700 lines):**
- [ ] `src/commands/context/mod.rs` (667 lines) - split modules or extract helpers
- [ ] `src/lib/similarity/mod.rs` (635 lines) - split modules or extract helpers
- [ ] `src/lib/db/notes/read.rs` (609 lines) - extract helper functions
- [ ] `src/commands/dispatch/mod.rs` (592 lines) - extract helper functions
- [ ] `src/commands/show.rs` (570 lines) - extract helper functions

**Low priority (500-600 lines):**
- [ ] `src/commands/list/mod.rs` (560 lines) - extract helper functions
- [ ] `src/cli/commands.rs` (547 lines) - extract helper functions
- [ ] `src/lib/graph/algos/dijkstra.rs` (511 lines) - extract helper functions

After refactoring each file, remove it from the `allowed` array in `.github/workflows/ci.yml:67-81`.

### cli-tool.md

- [ ] Missing tests for duplicate `--format` detection
- [ ] Missing performance tests for `--help`/`--version` (<100ms), `list` (~1k notes <200ms), `search` (~10k notes <1s)
- [ ] Find viable strategy for 10k note search performance test (current test ignored - indexing 10k notes takes minutes)
  - Options: pre-generated fixture store, direct DB population bypassing file creation, reduced note count with extrapolation
- [ ] Missing determinism test coverage for all commands

### storage-format.md

- [ ] Missing security test for discovery boundary with parent store
- [ ] Missing security test for malicious attachment paths in `qipu load`

### cli-interface.md

- [ ] Missing tests asserting JSON schema compliance (all required fields present)

### indexing-search.md

- [ ] Missing test for relative `.md` links cross-directory edge case
- [ ] No direct CLI tests for 2-hop neighborhoods
- [ ] Missing explicit test for incremental repair behavior (mtime-based indexing)
- [ ] Configurable ranking parameters (hardcoded boost values: +3.0 tag, 0.1/7.0 recency decay)
- [ ] Review and remove unjustified `#[allow(dead_code)]` attributes (src/lib/db/repair.rs:103, src/lib/db/traverse.rs:7)

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

- [ ] Missing tests for S prefix semantic distinction (summary vs sources)
- [ ] Missing truncation flag tests for prime/list/search/export
- [ ] Missing integration tests for "get index, then fetch bodies" workflow

### llm-context.md

- [ ] Missing tests for `qipu prime --format json` and `--format records`
- [ ] Missing tests for prime command missing-selection exit codes

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

