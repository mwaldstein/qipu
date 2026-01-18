# Qipu Implementation Plan

## **Status: Active Development**

Core P0/P1 features are substantially complete. Detailed audit conducted on 2026-01-17 identified gaps requiring attention.

---

## **Current Priority: Code Refactoring **

---

## **P0: Large File Refactoring** ⚠️ IN PROGRESS

**Priority**: Top priority unless test failures exist (test failures always take precedence)

Several files have grown overly large and need refactoring to improve maintainability:

### **Highest Priority Refactoring Candidates:**

1. **`tests/cli/link.rs` (1,438 lines)** ✅ COMPLETE - Broken into separate modules:
   - `tests/cli/link/list.rs` - link list command tests ✅
   - `tests/cli/link/tree.rs` - link tree command tests ✅
   - `tests/cli/link/path.rs` - link path command tests ✅
   - `tests/cli/link/add_remove.rs` - basic link operations ✅
   - `tests/cli/link/compaction.rs` - compaction-related link tests ✅
   - `tests/cli/link/mod.rs` - module declarations ✅

2. **`tests/cli/context.rs` (1,059 lines)** ✅ COMPLETE - Broken into separate modules:
   - `tests/cli/context/basic.rs` - basic context selection tests ✅
   - `tests/cli/context/budget.rs` - budget and truncation tests ✅
   - `tests/cli/context/compaction.rs` - compaction expansion tests ✅
   - `tests/cli/context/formats.rs` - output format tests ✅
   - `tests/cli/context/mod.rs` - module declarations ✅

3. **`tests/cli/compact.rs` (896 lines)** ✅ COMPLETE - Broken into separate modules:
   - `tests/cli/compact/commands.rs` - compact report/suggest/apply tests ✅
   - `tests/cli/compact/annotations.rs` - compaction visibility tests ✅
   - `tests/cli/compact/mod.rs` - module declarations ✅

### **Medium Priority Production Code:**

4. **`src/lib/compaction.rs` (656 lines)** - Core business logic with multiple responsibilities
5. **`src/cli/mod.rs` (634 lines)** - CLI argument definitions for all commands
6. **`src/lib/store/mod.rs` (608 lines)** - Mixed store concerns
7. **`src/commands/export/emit.rs` (601 lines)** - Multiple export formats
8. **`src/commands/link/tree.rs` (586 lines)** - Algorithm + formatting mixing

### **Function-Level Refactoring:**

9. **`src/commands/dispatch.rs`** - Large match statement (500+ lines) needs delegation
10. **Large functions across codebase** - Break down overly complex functions for maintainability

**Recommended approach**: Start with test files (easier to refactor), then focus on `src/lib/compaction.rs` as core business logic, followed by function-level decomposition.

---

## **P1: LLM User Validation Testing Harness** ✅ COMPLETE
All 6 phases completed:
- Phase 1: Separate Crate Setup ✅
- Phase 2: Test Fixtures ✅
- Phase 3: Core Harness Infrastructure ✅
- Phase 4: Tool Adapters ✅
- Phase 5: Evaluation System ✅
- Phase 6: Results & Reporting ✅

---

## **P1.5: Structured Logging Infrastructure** ❌ NOT STARTED

**Priority**: High - Improves observability and debugging capabilities

Replace primitive boolean logging with structured logging framework:

### **Current State**
- Basic boolean verbosity flag (`src/lib/logging.rs`)
- Ad-hoc `eprintln!` statements throughout codebase
- No structured log levels, categories, or machine-readable output

### **Implementation Tasks**
1. **Add tracing dependencies**: `tracing`, `tracing-subscriber` to `Cargo.toml`
2. **Extend CLI arguments**: Add `--log-level` and `--log-json` flags
3. **Initialize logging system**: Set up structured logging in `main.rs`
4. **Instrument core operations**: Add spans to store, search, index, graph operations
5. **Replace eprintln! statements**: Convert to structured logging with appropriate levels
6. **Add performance tracing**: Timing spans for major operations
7. **Enhance error context**: Structured error information with operation traces
8. **Update tests**: Handle new logging output in test assertions

**Spec**: `specs/structured-logging.md` ✅ COMPLETE  
**Target**: Zero performance impact when disabled, granular control when enabled

---

## **P1.6: Human Testing Guide** ❌ NOT STARTED

**Priority**: Medium - Improves release confidence and onboarding

Author a human-run testing guide in the docs folder to validate core user flows end-to-end.

### **Implementation Tasks**
1. **Create doc**: `docs/human-testing.md`
2. **Cover core flows**: init/store creation, create/capture, list/show/search, links, context/prime, export
3. **Include fixtures**: sample commands and expected outputs (human format)
4. **Cross-platform notes**: macOS/Linux differences (paths, editor, clipboard)
5. **Define "done"**: minimal smoke test vs full regression checklist

---

## **P2: Correctness Issues** ⚠️ MOSTLY COMPLETE
- Workspace merge bugs: 3/3 fixed ✅
- Pack load missing features: 3/3 fixed (test was flaky, not actual bug) ✅
- List performance: Fixed O(n²) compaction_pct calculation by caching note_map ✅

---

## **P2.5: Code Quality and Safety Audit** ❌ NOT STARTED

**Priority**: Medium-High - Improves code robustness and error handling

### **Unwrap/Expect Usage Audit**
Current codebase contains 262 instances of `.unwrap()` and `.expect()` calls outside of tests, creating potential panic risks:

**Audit Tasks:**
1. **Catalog all unwrap/expect locations**: Systematically review each instance
2. **Categorize by safety level**: 
   - Safe (guaranteed by invariants)
   - Defensive (has fallback with `unwrap_or`)  
   - Risky (could panic in normal operation)
3. **Replace risky unwraps**: Convert to proper error handling with `?` operator
4. **Add context to expect calls**: Ensure all `.expect()` calls have descriptive messages
5. **Document safety invariants**: Add comments explaining why remaining unwraps are safe

**Examples found:**
- `current_dir().unwrap_or_else(|_| PathBuf::from("."))` ✅ Safe (has fallback)
- `.unwrap_or(std::cmp::Ordering::Equal)` ✅ Safe (has fallback)
- Other `.unwrap()` calls need review for safety

**Target**: Reduce panic risks while maintaining code clarity

---

## **P3: Missing Test Coverage** ⚠️ PARTIAL (1/6 complete)
- Workspace commands: Partially tested ✅
- **5 areas remain untested**: Capture command, Graph traversal limits, Type filtering, Pack conflict strategies, Provenance fields, Token budgeting

---

## **P4: Spec-Implementation Gaps** ❌ NOT STARTED (10 items remain)

#### Similarity Ranking Issues
- **Stop words removal**: Required by spec, not implemented
- **Stemming**: Optional Porter stemmer mentioned in spec, not implemented
- **Term frequency storage**: Index stores only unique terms (tf=1 assumed), reducing accuracy ([similarity/mod.rs#L136-L137])

#### Indexing/Search Issues
- **SQLite FTS5 backend**: Optional SQLite FTS mentioned in spec, not implemented
- **Recency boost**: Spec says recent notes can receive boost, not included in ranking

#### Workspace Additional Issues
- **`--from-note` doesn't copy graph slice**: Only copies single note, not linked notes
- **`--from-query` uses simple substring match**: Doesn't leverage search index
- **`last_updated` missing from list output**
- **Dry-run shows no conflict report**
- **No `doctor` check after merge**

#### Provenance Issue
- **JSON output missing provenance**: The create command's JSON output omits provenance fields

---

## **P3 Test Coverage Status** ⚠️ PARTIAL (1/6 complete)

**Completed:**
1. Workspace commands: Partially tested

**Remaining (5 areas):**
1. Capture command: No dedicated tests exist
2. Graph traversal limits: `--max-nodes`, `--max-edges`, `--max-fanout`, `--direction in` flags are not tested
3. Type filtering: `--type`, `--exclude-type`, `--typed-only`, `--inline-only` flags are not tested
4. Pack conflict strategies: No tests exist (skip, overwrite, merge-links)
5. Provenance fields: `prompt_hash` not tested
6. Token budgeting: `--max-tokens` flag has no integration tests

---

## **P4 Spec-Implementation Gaps** ❌ NOT STARTED (10 items remain)

### Similarity Ranking (3 items)
- Stop words removal (required by spec)
- Stemming (optional Porter stemmer)
- Term frequency storage (TODO: store unique terms, tf=1 assumption)

### Indexing/Search (3 items)
- SQLite FTS5 backend (optional)
- Recency boost (recent notes can receive boost)

### Workspace Additional (4 items)
- `--from-note` doesn't copy graph slice (only copies single note)
- `--from-query` uses simple substring match (doesn't leverage search index)
- `last_updated` missing from list output
- Dry-run shows no conflict report
- No `doctor` check after merge

### Provenance (1 item)
- JSON output missing provenance (create command omits fields)

---

## **Completed Work Summary**

### Fully Complete (11 items) ✅
1. CLI Tool (cli-tool.md)
2. Knowledge Model (knowledge-model.md)
3. Storage Format (storage-format.md)
4. CLI Interface (cli-interface.md)
5. Semantic Graph (semantic-graph.md)
6. Graph Traversal (graph-traversal.md)
7. Records Output (records-output.md)
8. LLM Context (llm-context.md)
9. Export (export.md)
10. Compaction (compaction.md)
11. LLM User Validation (llm-user-validation.md)

### Substantially Complete (4 items) ⚠️
1. Indexing & Search (with noted gaps)
2. Provenance (substantially complete)
3. Pack (marked complete with some gaps)
4. Similarity Ranking (partial - missing stop words, stemming, term frequency storage)
5. Workspaces (partial - missing rename strategy, graph slice seeding, more tests)

### Partial (2 items) ⚠️
1. Semantic Graph (missing some tests)
2. Workspaces (partial)

---

## **Infrastructure**

### GitHub Actions
Currently disabled (`on: {}` in ci.yml). **DO NOT enable until Actions is activated in GitHub repo settings.**

---

## **Key Learnings**

1. **`--id` flag works**: The implementation allows creating notes with specific IDs for testing and advanced use cases. This enables proper testing of pack load strategies.

2. **Pack load merge-links works**: Links are correctly added to existing notes when using merge-links strategy.

3. **Complex test isolation needed**: Current pack tests have state pollution issues between runs. Tests need proper isolation (unique temp directories) to ensure deterministic results.

4. **Spec-compliant but complex**: The application is mostly spec-compliant with well-structured code. However, pack load test coverage is complex due to state management requirements.

---

## **Next Steps (Prioritized)**

1. **Fix pack load test isolation**: Ensure tests don't pollute each other's state
2. **Add missing pack load tests**: Create proper test coverage for skip, overwrite, and merge-links strategies
3. **Complete large file refactoring**: Break down oversized files and large functions for better maintainability
4. **Implement structured logging**: Replace primitive logging with tracing framework for better observability
5. **Audit unwrap/expect usage**: Review and improve error handling to reduce panic risks
6. **Implement missing P4 features**: Focus on spec-implementation gaps, particularly similarity ranking improvements

---

*Last updated: 2026-01-17*
