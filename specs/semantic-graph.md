# Semantic Graph & Link Ontology

Status: Draft  
Last updated: 2026-01-17

## Motivation
To enable powerful graph traversal for LLM agents, links must be more than simple "goto" pointers. They must carry semantic weight.

This spec defines:
1.  The **Standard Ontology** of link types.
2.  **Semantic Inversion** (how `supports` implies `supported-by`).
3.  **Traversal Logic** (how types affect graph walking).
4.  **Extensibility** (how users define custom types).

## 1. Standard Ontology
Qipu ships with a minimal, "batteries-included" set of link types. These cover two primary dimensions: **Structure** (organizing notes) and **Argumentation** (reasoning about notes).

### A. Structural Types (The Skeleton)
These types define the hierarchy and composition of the knowledge base.

| Forward Type | Inverse Type | Description |
| :--- | :--- | :--- |
| `part-of` | `has-part` | Meronymy. Note A is a component of Note B (e.g., Chapter -> Book). |
| `follows` | `precedes` | Sequence. Note A comes after Note B (e.g., Step 2 -> Step 1). |
| `related` | `related` | Symmetric association. General relevance (default for inline links). |

### B. Argumentative Types (The Reasoning - IBIS-lite)
These types capture the "why" and "so what," enabling agents to trace lines of reasoning.

| Forward Type | Inverse Type | Description |
| :--- | :--- | :--- |
| `supports` | `supported-by` | Evidential support. Note A provides evidence for Note B. |
| `contradicts`| `contradicted-by`| Evidential conflict. Note A argues against Note B. |
| `answers` | `answered-by` | Solution. Note A answers the question posed in Note B. |
| `refines` | `refined-by` | Iteration. Note A is a newer/better version of Note B. |

### C. Identity Types (The Unifiers)
These types handle synonyms, duplicates, and preferred terms, enabling "Identity Resolution".

| Forward Type | Inverse Type | Description |
| :--- | :--- | :--- |
| `same-as` | `same-as` | Strong identity. Note A and Note B represent the exact same concept (synonym). |
| `alias-of` | `has-alias` | Redirection. Note A is an alternative name for Note B (the canonical note). |

## 2. Semantic Inversion (Virtual Edges)
In a raw graph, edges are directional. If Note A has `links: [{ to: B, type: supports }]`, traversing starting at B would typically require a "backlink" search.

Qipu traversal treats **inverse relationships as first-class virtual edges**.

### The Rule
When traversing from Note B, if the index finds a link `A -> B` of type `T`, the traverser presents a virtual edge `B -> A` of type `Inverse(T)`.

*   **Raw Data**: Note A: `type: supports`, target: Note B.
*   **Traversal View at B**:
    *   Edge to A: `type: supported-by` (virtual).

### Benefits
*   **Cognitive Load**: The consumer (human or LLM) doesn't need to manually check "incoming" vs "outgoing" lists and mentally reverse verbs.
*   **Unified Querying**: `qipu link tree --type supported-by` works naturally, even though `supported-by` is never stored on disk.

## 3. Traversal Implications
Link types influence *how* the graph is traversed, not just what is displayed.

### A. Transitivity & Hop Cost
Not all hops are equal.
*   **Standard Hop**: Cost = 1. (Default).
*   **Transitive/Cohesive Hops**: Some relationships imply strong cohesion (e.g., `part-of`).
    *   *Proposal*: Allow specific types to have reduced cost (e.g., 0.5) or be "free" up to a limit, effectively expanding the "context window" for cohesive units.
    *   *Implementation*: For v1, we stick to **Cost = 1** for simplicity, but the traversal engine should be designed to support variable costs.

### B. Default Inclusion
*   `related` (and inline links) are high-volume/low-signal.
*   Typed links are high-signal.
*   **Traversal Default**: Include ALL types.
*   **Context Default**: When generating `qipu context`, strongly prefer typed links (especially `part-of` and `supports`) over generic `related` links if token budget is tight.

## 4. Extensibility & Defaults
Qipu supports custom knowledge graphs while enforcing a "common dialect" for interoperability.

### The "Standard Library" (Hard-coded Defaults)
The types listed in Section 1 are **hard-coded defaults** in the Qipu binary.
*   They are always available.
*   They have defined inverses.
*   LLM prompts are optimized to understand them.

### User Customization (Config-based)
Users can define additional types in `.qipu/config.toml`:

```toml
[graph.types.embodies]
inverse = "embodied-by"
description = "Relationship for abstract concepts and concrete examples"
```

*   **Unknown Types**: If a note uses a type not defined in defaults or config, Qipu treats it as a raw string.
    *   Inverse: `inverse-<type>` (fallback).
    *   Traversal: Works normally, just lacks semantic sugar.

## 5. Implementation Notes
*   **Index**: The search index must store the forward link. Inversion happens at *query time* (or `link tree` time) to keep the index simple.
*   **Validation**: `qipu doctor` should warn if standard types are used incorrectly (e.g., `part-of` pointing to a non-existent note), but `qipu` is generally permissive.
