# Qipu Implementation Plan

## **Status: Active Development**

The core P0/P1 engine is complete. Current focus is on **User-Defined Link Types** (extending the semantic graph) and enhancing LLM/Agent trust through provenance metadata.

---

## **Current Priority Items**

### **P1: Agent Trust & Semantic Enhancements (High Priority)**
- [x] **Git Automation (`--push`)**: Ensure `qipu sync --push` handles remote synchronization correctly, including branch protection workflows (see `specs/storage-format.md`).

### **P2: Advanced Knowledge Management & Scalability**
- [ ] **Pack/Unpack**: Implement single-file raw knowledge dump/load as described in `specs/pack.md`.

### **P3: Usability & Architecture Refinement**
- [ ] **Deterministic ID Schemes**: Implement `qp-<hash>` with adaptive length to ensure collision resistance under parallel creation (see `specs/knowledge-model.md`).
- [ ] **Note Templates & Defaults**: Allow users to override default note templates via `.qipu/templates/` directory (see `specs/storage-format.md`).
- [ ] **Wiki-link Canonicalization**: Add an opt-in feature to `qipu index` to rewrite wiki-links into standard markdown links (see `specs/storage-format.md`).
- [ ] **Interactive Pickers**: Add `dialoguer` or `inquire` based selection for IDs in CLI commands.
- [ ] **SQLite FTS Backend**: Evaluate/Implement optional SQLite FTS5-powered index for very large stores (>10k notes) to augment/replace ripgrep (see `specs/indexing-search.md`).

---

## **Completed Work**

### **✅ Core P0/P1 Items (Complete)**
- **Attachment Validation**: Integrated missing/orphaned attachment detection into `qipu doctor`.
- **Compaction Enhancements**: Refactored clustering logic into `CompactionContext::suggest` and implemented boundary edge ratio/staleness indicators.
- **Semantic Inversion & Provenance Metadata**: Enhanced graph semantics and AI trust with virtual edges and author fields.
- **User-Defined Link Types**: Support for custom link types and inverses in config.
- **Token-based Budgeting**: `tiktoken-rs` integration for context management.
- **Workspaces**: `qipu workspace` suite for managing scratchpads.
- **Similarity Engine**: BM25 weighted Cosine Similarity for duplicate detection.
- **Indexer "Race Condition"**: Investigated `tests/cli/compact.rs`; confirmed `sleep` is for timestamp resolution (staleness detection).
- **Search Ranking (BM25)**: Field boosting (Title x2.0, Tags x1.5).
- **CLI Interface**: `dump`/`load`, `link` navigation, and `inbox` implemented.
- **ID Scheme**: Adaptive length hash ID scheme implemented.
- **Compaction**: `apply`, `suggest`, and `report` implemented.
- **Export**: Support for bundle, outline, and bibliography modes.
- **Sync & Git**: Automated `qipu sync` with git commit support.
- **Search Optimization**: <1s for 10k notes target achieved using ripgrep.
- **LLM User Validation**: Test harness `tests/llm_validation.rs` implemented.
- **Determinism**: Removed runtime timestamps and fixed HashMap ordering in JSON output.
