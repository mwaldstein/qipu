# Qipu Implementation Plan

## **Status: Active Development**

The core P0/P1 engine for note management, linking, and basic search is complete. The current focus is on enhancing LLM/Agent trust, improving search accuracy, and adding advanced knowledge management features.

---

## **Current Priority Items**

### **P1: Agent Trust & Search (High Priority)**
- [ ] **Provenance Metadata**: Update `NoteFrontmatter` in `src/lib/note/frontmatter.rs` and CLI `create`/`capture` to support AI-specific fields: `author`, `generated_by`, `prompt_hash`, and `verified` boolean (see `specs/provenance.md`).
- [ ] **Token-based Budgeting**: Implement `tiktoken`-based token counting for `qipu context` to complement existing `--max-chars` enforcement (see `specs/llm-context.md`).
- [ ] **Search Ranking (BM25)**: Replace the heuristic-based scoring in `src/lib/index/search.rs` with proper BM25 ranking for more accurate search results (see `specs/similarity-ranking.md`).
- [ ] **Context Safety Banner**: Implement the actual banner text injection logic for `--safety-banner` in `qipu context` (see `specs/llm-context.md`).
- [ ] **Deterministic Context Truncation**: Ensure truncation in `qipu context` is stable and predictable across sessions.

### **P2: Advanced Knowledge Management & Scalability**
- [ ] **Workspaces**: Implement `qipu workspace` suite for managing temporary/secondary stores (scratchpads) under `.qipu/workspaces/`. Includes `new`, `list`, `delete`, and `merge` commands (see `specs/workspaces.md`).
- [ ] **Similarity Engine**: Implement Cosine Similarity for `qipu doctor --duplicates` and finding unlinked "Related Notes" (see `specs/similarity-ranking.md`).
- [ ] **Merge Command**: Implement a dedicated `qipu merge <id1> <id2>` to combine notes and update all inbound links automatically.
- [ ] **Attachment Validation**: Update `qipu doctor` to validate missing or orphaned attachments.

### **P3: Usability & Architecture Refinement**
- [ ] **Interactive Pickers**: Add `dialoguer` or `inquire` based selection for IDs in CLI commands.
- [ ] **SQLite FTS Backend**: Evaluate/Implement optional SQLite FTS5-powered index for very large stores (>10k notes) to augment/replace ripgrep (see `specs/indexing-search.md`).
- [ ] **Indexer Race Condition**: Investigate and fix the underlying cause of the race condition requiring `sleep` in `tests/cli/compact.rs`.
- [ ] **Outline Export Refinement**: Ensure Outline mode in `qipu export` follows MOC ordering strictly.

---

## **Completed Work**

### **✅ Core P0/P1 Items (Complete)**
- **CLI Interface**: `dump`/`load`, `link` navigation (`--max-hops`), and `inbox` implemented.
- **ID Scheme**: Adaptive length hash ID scheme implemented in `src/lib/id.rs`.
- **Compaction**: `apply`, `suggest`, and `report` implemented in `src/commands/compact/`.
- **Export**: Support for bundle, outline, and bibliography modes.
- **Sync & Git**: Automated `qipu sync` with git commit/push support.
- **Search Optimization**: <1s for 10k notes target achieved using ripgrep.
- **Context Budgeting**: Character-based budgeting implemented.
- **LLM User Validation**: Test harness `tests/llm_validation.rs` implemented.
- **Determinism**: Removed runtime timestamps and fixed HashMap ordering in JSON output.
