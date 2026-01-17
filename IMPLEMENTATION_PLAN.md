# Qipu Implementation Plan

## **Status: Active Development**

While the core P0/P1 engine for note management, linking, and basic search is complete, several key features required for full agentic integration and advanced knowledge management are still in progress.

---

## **Current Priority Items**

### **P1: Agent Trust & Context (High Priority)**
- [ ] **Provenance Metadata**: Update `NoteFrontmatter` and CLI `create`/`capture` to support AI-specific fields: `author`, `generated_by`, `prompt_hash`, and `verified` boolean (see `specs/provenance.md`).
- [ ] **Token-based Budgeting**: Implement tiktoken-based (or approximate) token counting for `qipu context` to complement existing `--max-chars` enforcement (see `specs/llm-context.md`).
- [ ] **Safety Banner**: Add optional `--safety-banner` to `qipu context` to prepend "untrusted content" warnings for downstream LLMs.

### **P2: Advanced Knowledge Management**
- [ ] **Workspaces**: Implement `qipu workspace` suite for managing temporary/secondary stores (scratchpads) under `.qipu/workspaces/`. Includes `new`, `list`, `delete`, and `merge` commands (see `specs/workspaces.md`).
- [ ] **Similarity Ranking (BM25/TF-IDF)**: Upgrade the current heuristic-based search to use proper BM25 for ranking and TF-IDF for finding unlinked "Related Notes" (see `specs/similarity-ranking.md`).
- [ ] **Duplicate Detection**: Implement `qipu doctor --duplicates` using similarity scores to flag near-identical notes for merging.

### **P3: Usability & Scaling**
- [ ] **Interactive Pickers**: Add `fzf`-style selection for IDs in CLI commands.
- [ ] **SQLite Backend**: Optional FTS5-powered index for very large stores (>10k notes) to replace/augment ripgrep.
- [ ] **Merge Command**: Implement a dedicated `qipu merge <id1> <id2>` to combine notes and update all inbound links.

---

## **Completed Work**

### **✅ Core P0 Items (All 11/11 Complete)**
1. Fix CLI JSON behavior - --format flag parsing for equals syntax
2. Fix CLI JSON behavior - help/version exit codes
3. Fix CLI JSON behavior - error envelope exit codes
4. Eliminate nondeterminism - remove runtime timestamps
5. Eliminate nondeterminism - fix HashMap iteration order
6. Eliminate nondeterminism - add stable tie-breakers for sorting
7. Context budget enforcement - make exact across all formats (character-based)
8. Implement dump/load commands - `qipu dump` / `qipu load` (specs/pack.md)
9. Fix search/index determinism - Ripgrep JSON parsing and stable ordering
10. Implement LLM user validation harness - `tests/llm_validation.rs` (specs/llm-user-validation.md)
11. Optimize `qipu search` performance - <1s for 10k notes target

### **✅ P1 Items (Core Features)**
- **CLI Interface**: `dump`/`load`, `link` navigation (`--max-hops`), and `inbox` implemented.
- **Compaction**: `qipu compact` suite for managing note evolution.
- **Export**: Full support for bundle, outline, and bibliography modes.
- **Sync & Git**: Automated `qipu sync` with git commit/push support.
- **Quality**: Robust test suite with golden tests and performance benchmarks.
