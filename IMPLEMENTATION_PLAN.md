# Qipu Implementation Plan

## **Status: Active Development**

Core P0/P1 features are substantially complete. Detailed audit conducted on 2026-01-17 identified gaps requiring attention.

---

## **Current Priority: P2 Test Coverage and P3 Spec-Implementation Gaps**

---

## **P0: LLM User Validation Testing Harness** ✅ COMPLETE
All 6 phases completed:
- Phase 1: Separate Crate Setup ✅
- Phase 2: Test Fixtures ✅
- Phase 3: Core Harness Infrastructure ✅
- Phase 4: Tool Adapters ✅
- Phase 5: Evaluation System ✅
- Phase 6: Results & Reporting ✅

---

## **P1: Correctness Issues** ⚠️ MOSTLY COMPLETE
- Workspace merge bugs: 3/3 fixed ✅
- Pack load missing features: 3/4 fixed (1 failing test needs investigation) ⚠️

---

## **P2: Missing Test Coverage** ⚠️ PARTIAL (1/6 complete)
- Workspace commands: Partially tested ✅
- **5 areas remain untested**: Capture command, Graph traversal limits, Type filtering, Pack conflict strategies, Provenance fields, Token budgeting

---

## **P3: Spec-Implementation Gaps** ❌ NOT STARTED (10 items remain)

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

## **P2 Test Coverage Status** ⚠️ PARTIAL (1/6 complete)

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

## **P3 Spec-Implementation Gaps** ❌ NOT STARTED (10 items remain)

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
3. **Implement missing P3 features**: Focus on spec-implementation gaps, particularly similarity ranking improvements

---

*Last updated: 2026-01-18*
