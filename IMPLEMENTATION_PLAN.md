# Qipu Implementation Plan

## **Status: Active Development**

While the core P0/P1 engine for note management, linking, and basic search is complete, several key features required for full agentic integration and advanced knowledge management are still in progress.

---

## **Current Priority Items**

### **P1: Agent Trust & Context (High Priority)**
- [ ] **Provenance Metadata**: Update `NoteFrontmatter` in `src/lib/note/frontmatter.rs` and CLI `create`/`capture` to support AI-specific fields: `author`, `generated_by`, `prompt_hash`, and `verified` boolean (see `specs/provenance.md`).
- [ ] **Token-based Budgeting**: Implement tiktoken-based (or approximate) token counting for `qipu context` to complement existing `--max-chars` enforcement (see `specs/llm-context.md`).
- [ ] **Search Ranking (BM25)**: Replace the heuristic-based scoring in `src/lib/index/search.rs` with proper BM25 ranking for more accurate search results (see `specs/similarity-ranking.md`).
- [ ] **Safety Banner Defaults**: Refine `--safety-banner` in `qipu context` to include standard "Untrusted AI Content" warning text by default.

### **P2: Advanced Knowledge Management & Scalability**
- [ ] **Workspaces**: Implement `qipu workspace` suite for managing temporary/secondary stores (scratchpads) under `.qipu/workspaces/`. Includes `new`, `list`, `delete`, and `merge` commands (see `specs/workspaces.md`).
- [ ] **Similarity Engine**: Implement Cosine Similarity for `qipu doctor --duplicates` and finding unlinked "Related Notes" (see `specs/similarity-ranking.md`).
- [ ] **Compaction UX**: Implement `qipu compact suggest` and `qipu compact report` to assist users in identifying candidates for consolidation (see `specs/compaction.md`).
- [ ] **Fix Indexer Race Condition**: Address potential race condition in incremental indexing identified in `tests/golden_tests.rs` (currently mitigated by `sleep`).

### **P3: Usability & Refinement**
- [ ] **Interactive Pickers**: Add `fzf`-style selection for IDs in CLI commands for better human ergonomics.
- [ ] **Merge Command**: Implement a dedicated `qipu merge <id1> <id2>` to combine notes and update all inbound links automatically.
- [ ] **SQLite Backend**: Evaluate optional FTS5-powered index for very large stores (>10k notes) to replace/augment ripgrep.

---

## **Completed Work**

### **✅ Core P0 Items (Complete)**
1. Fix CLI JSON behavior - --format flag parsing for equals syntax
2. Fix CLI JSON behavior - help/version exit codes
3. Fix CLI JSON behavior - error envelope exit codes
4. Eliminate nondeterminism - remove runtime timestamps and fix HashMap ordering
5. Context budget enforcement - character-based exact budgeting
6. Implement dump/load commands - `qipu dump` / `qipu load` (specs/pack.md)
7. Implement LLM user validation harness - `tests/llm_validation.rs` (specs/llm-user-validation.md)
8. Optimize `qipu search` performance - <1s for 10k notes target

### **✅ P1 Items (Core Features)**
- **CLI Interface**: `dump`/`load`, `link` navigation (`--max-hops`), and `inbox` implemented.
- **Compaction**: Basic `qipu compact apply` logic and resolution engine.
- **Export**: Full support for bundle, outline, and bibliography modes.
- **Sync & Git**: Automated `qipu sync` with git commit/push support.
- **Quality**: Robust test suite with golden tests and performance benchmarks.
