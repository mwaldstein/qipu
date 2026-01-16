# Qipu Implementation Plan (Complete)

## **🎉 PROJECT COMPLETE**

All P0 and P1 specifications have been implemented. The system is fully functional according to the design docs.

---

## **Completed Work**

### **✅ Core P0 Items (All 11/11 Complete)**
1. Fix CLI JSON behavior - --format flag parsing for equals syntax
2. Fix CLI JSON behavior - help/version exit codes
3. Fix CLI JSON behavior - error envelope exit codes
4. Eliminate nondeterminism - remove runtime timestamps
5. Eliminate nondeterminism - fix HashMap iteration order
6. Eliminate nondeterminism - add stable tie-breakers for sorting
7. Context budget enforcement - make exact across all formats
8. Implement dump/load commands - `qipu dump` / `qipu load` (specs/pack.md)
9. Fix search/index determinism - Ripgrep JSON parsing and stable ordering
10. Implement LLM user validation harness - `tests/llm_validation.rs` (specs/llm-user-validation.md)
11. Optimize `qipu search` performance - <1s for 10k notes target

### **✅ P1 Items (All Complete)**
- **CLI Interface**:
  - `qipu dump` / `qipu load` fully implemented.
  - `qipu link` flags: `--max-chars`, `--direction`, `--max-hops` implemented.
  - Search filters: `--type`, `--tag`, `--exclude-mocs` implemented.
  - `qipu inbox` command implemented.
  - `qipu setup` command implemented (AGENTS.md integration).
- **Compaction**:
  - `qipu compact` suite (`apply`, `suggest`, `report`, `status`, `guide`) implemented.
  - Compaction resolution in search/traversal implemented.
- **Export**:
  - `qipu export` implemented with bundle, outline, and bibliography modes.
  - Attachment export support.
- **Sync & Git**:
  - `qipu sync` implemented with index updates and git commit/push automation.
  - Protected branch workflow supported (`qipu init --branch`).
- **Quality**:
  - Error code consistency (exit code 2 for usage errors).
  - Test suite passing (unit, CLI, golden, performance, LLM validation).

---

## **Future / Deferred Work (P2+)**

These items are considered potential future enhancements but are not required for the v1.0 release.

### **Interactive Usability**
- [ ] **Interactive Pickers**: Add fzf-style interactive selection for notes/tags/types (mentioned in `specs/cli-interface.md`).
- [ ] **Interactive Capture**: Add a TUI for `qipu capture` rather than just stdin/editor.

### **Advanced Search & Indexing**
- [ ] **SQLite Backend**: Implement SQLite FTS for scaling beyond `ripgrep` for very large stores (mentioned in `specs/indexing-search.md`).
- [ ] **Semantic Search**: Add embedding-based search (requires external dependencies or optional feature).

### **Knowledge Management Tools**
- [ ] **Duplicate Detection**: Implement `qipu duplicates` to find potential near-duplicate notes.
- [ ] **Merge Command**: Implement `qipu merge` to combine notes (mentioned in `specs/knowledge-model.md`).

### **Extended Validation**
- [ ] **Multi-tool Validation**: Extend `tests/llm_validation.rs` to support tools beyond OpenCode (e.g. Claude CLI) for cross-agent validation.
