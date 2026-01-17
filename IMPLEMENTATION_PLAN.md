# Qipu Implementation Plan

## **Status: Active Development**

The core P0/P1 engine is complete. Current focus is on **User-Defined Link Types** (extending the semantic graph) and enhancing LLM/Agent trust through provenance metadata.

---

## **Current Priority Items**

### **P1: Agent Trust & Semantic Enhancements (High Priority)**
- [x] **Semantic Inversion & Provenance Metadata**: Combined effort to enhance graph semantics and AI trust.
    - [x] **Expand LinkType enum with full Standard Ontology**
    - [x] **Semantic Inversion (Virtual Edges)**: Update `src/lib/index/mod.rs` and `src/commands/link/` to support virtual inverse edges (e.g., `supports` $\implies$ `supported-by`) as first-class traversal entities (see `specs/semantic-graph.md`).
    - [x] **Expand Standard Ontology**: Add missing standard link types to `LinkType` enum: `follows/precedes`, `answers/answered-by`, `refines/refined-by`, `same-as`, `alias-of` (see `specs/semantic-graph.md`).
- [x] **Provenance Metadata**: Update `NoteFrontmatter` in `src/lib/note/frontmatter.rs` and CLI `create`/`capture` to support AI-specific fields: `author`, `generated_by`, `prompt_hash`, and `verified` boolean (see `specs/provenance.md`). Verified prioritization implemented in `qipu context`. Added `qipu verify` command.
- [x] **User-Defined Link Types**: Update `StoreConfig` and `LinkType` parsing to support custom link types and their inverses defined in `.qipu/config.toml` (see `specs/semantic-graph.md`).
    - *Note*: Transitioning from `enum LinkType` to `struct LinkType(String)` to support extensibility while maintaining the standard ontology as defaults. Updated `LinkConfig` with `descriptions`.
- [x] **Token-based Budgeting**: Implement `tiktoken`-based token counting for `qipu context` to complement existing `--max-chars` enforcement (see `specs/llm-context.md`).
    - *Learnings*: Used `tiktoken-rs` for counting; added `--max-tokens` and `--model` flags to the `context` command.
- [ ] **Next Steps**: Add more integration tests for user-defined link types and provenance.
- [ ] **Semantic Graph Library**: Extract graph traversal logic (tree, path) from commands in `src/commands/link/` into `src/lib/index/` or a new `src/lib/graph/` component (see `specs/graph-traversal.md`).
- [ ] **Git Automation (`--push`)**: Ensure `qipu sync --push` handles remote synchronization correctly, including branch protection workflows (see `specs/storage-format.md`).

### **P2: Advanced Knowledge Management & Scalability**
- [ ] **Workspaces**: Implement `qipu workspace` suite for managing temporary/secondary stores (scratchpads) under `.qipu/workspaces/`. Includes `new`, `list`, `delete`, and `merge` commands (see `specs/workspaces.md`).
- [ ] **Similarity Engine**: Implement Cosine Similarity for `qipu doctor --duplicates` and finding unlinked "Related Notes" (see `specs/similarity-ranking.md`).
- [ ] **Compaction Enhancements**: Implement boundary edge ratio and staleness indicators in `qipu compact report`. Improve `suggest` clustering beyond connected components (see `specs/compaction.md`).
- [ ] **Merge Command**: Implement a dedicated `qipu merge <id1> <id2>` to combine notes and update all inbound links automatically.
- [ ] **Attachment Validation**: Update `qipu doctor` to validate missing or orphaned attachments in `.qipu/attachments/`.
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
- **Indexer "Race Condition"**: Investigated `tests/cli/compact.rs`; confirmed `sleep` is for timestamp resolution (staleness detection) and not an indexer race.
- **Search Ranking (BM25)**: Implemented BM25 with word-based tokenization and field boosting (Title x2.0, Tags x1.5).
- **Semantic Graph**: Virtual inverse edges and standard ontology implemented.
- **Token-based Budgeting**: `tiktoken-rs` integration for context management.
- **CLI Interface**: `dump`/`load`, `link` navigation (`--max-hops`), and `inbox` implemented.
- **ID Scheme**: Adaptive length hash ID scheme implemented in `src/lib/id.rs`.
- **Compaction**: `apply`, `suggest`, and `report` implemented in `src/commands/compact/`.
- **Export**: Support for bundle, outline, and bibliography modes.
- **Sync & Git**: Automated `qipu sync` with git commit support.
- **Search Optimization**: <1s for 10k notes target achieved using ripgrep.
- **Context Budgeting**: Character-based budgeting implemented.
- **LLM User Validation**: Test harness `tests/llm_validation.rs` implemented.
- **Determinism**: Removed runtime timestamps and fixed HashMap ordering in JSON output.
