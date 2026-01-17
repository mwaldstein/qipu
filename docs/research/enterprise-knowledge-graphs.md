# Enterprise Knowledge Graph (EKG) Research & Qipu Adaptation

## 1. Executive Summary
Enterprise Knowledge Graphs (EKGs) are massive, formal semantic networks used by organizations to unify data, enable reasoning, and power AI agents. They are characterized by strict schemas, complex data pipelines, and multi-user governance.

**Vision for Qipu:** To adapt these "enterprise-scale" capabilities into a **local-first, git-backed** tool. Qipu aims to provide the *power* of an EKG (reasoning, context for agents) without the *overhead* (RDF triple stores, complex ETL).

## 2. Core Architecture: Enterprise vs. Qipu

| Component | Enterprise Standard | Qipu Adaptation (Local/Git) |
| :--- | :--- | :--- |
| **Storage** | Graph Database (Neo4j, Amazon Neptune) | Git Repo (Markdown + Frontmatter) |
| **Schema** | Formal Ontology (OWL, SHACL, RDF) | Lightweight Config (`.qipu/config.toml`) |
| **Ingestion** | ETL Pipelines (Spark, Kafka) | CLI Commands (`qipu capture`, `qipu link`) |
| **Querying** | SPARQL, Cypher, GQL | `qipu link tree`, `qipu search` |
| **Reasoning** | Inference Engines (RDFS, OWL RL) | Traversal-time Logic (Inversion, Transitivity) |
| **Vector** | Vector DB (Milvus, Pinecone) | *Future*: Local embeddings (sqlite-vss / lance) |

## 3. Key Concepts & Adaptations

### A. GraphRAG (Graph-based Retrieval-Augmented Generation)
**Enterprise:** Combines vector search (similarity) with graph traversal (structure) to ground LLM responses and reduce hallucinations.
**Qipu Choice:**
*   **Implement "Structural RAG"**: `qipu context` bundles the "neighborhood" of a note.
*   **Future**: Add local embeddings to support "Semantic Entry Points" (find the right starting note via vector search, then traverse the graph).

### B. Ontology & Schema
**Enterprise:** Strict, comprehensive ontologies (e.g., FIBO for finance) to ensure data interoperability across departments.
**Qipu Choice:** **"Just-in-Time" Ontology**.
*   Ship with a minimal "Standard Library" (Section 1 of `specs/semantic-graph.md`).
*   Allow "Folksonomy" (custom user tags/types).
*   *Anti-pattern to avoid*: Forcing users to define a full SHACL schema before writing their first note.

### C. Data Provenance (Lineage)
**Enterprise:** PROV-O standard. Tracks `Entity` -> `wasGeneratedBy` -> `Activity` -> `wasAssociatedWith` -> `Agent`. Crucial for audit trails.
**Qipu Choice:** **Git-backed Provenance**.
*   Git commit history *is* the provenance log for "who changed what".
*   Add metadata for "Source Origin":
    *   `source_url`: Where did this info come from?
    *   `generated_by`: Which LLM model wrote this?
    *   `prompt_id`: Which prompt generated this?

### D. Identity Resolution
**Enterprise:** sophisticated "Entity Resolution" pipelines to merge "John Smith" and "J. Smith".
**Qipu Choice:** **Manual + Assisted**.
*   Support `same-as` link type to explicitly link synonyms/duplicates without destroying data.
*   CLI tool `qipu doctor --duplicates` to suggest merges based on fuzzy title matching.

### E. Inference & Reasoning
**Enterprise:** Materializing billions of inferred triples (e.g., if `A parent-of B`, write `B child-of A` to DB).
**Qipu Choice:** **Query-time Inference**.
*   Do not pollute source files with inferred data.
*   Calculate inverses (`supported-by`) and transitivity (`part-of` chains) purely in memory during traversal.

## 4. Conscious Choices (The "Qipu Way")

1.  **Text over Triples**: We store data as **Markdown**, not RDF/Turtle. *Why?* Human readability is paramount for a "Personal" KG. Agents can read Markdown easier than Turtle.
2.  **Git over DB**: We rely on **Git** for versioning, sync, and history. *Why?* It fits the developer workflow and provides "Time Travel" for free.
3.  **Local over Cloud**: No server required. *Why?* Privacy, speed, and offline capability.
4.  **Agent-First Output**: While storage is human-readable, output (stdout) is optimized for **Agent Consumption** (JSON, deterministic ordering, stable IDs).
