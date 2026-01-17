# Semantic & Knowledge Graph Research for Qipu

## 1. Definitions & Context

### Semantic Graph
A semantic graph is a network of concepts where edges represent well-defined semantic relationships. Unlike a simple hyperlinked web (where links are just "goto" instructions), a semantic graph encodes *meaning* (e.g., "A *is a type of* B", "A *causes* B").

### Knowledge Graph (KG)
A knowledge graph is a semantic graph where:
1.  **Nodes** are entities (or in Qipu's case, atomic notes/ideas).
2.  **Edges** describe facts or relationships.
3.  **Schema/Ontology** typically constrains valid types and relationships.

Qipu operates as a **Personal Knowledge Graph (PKG)**. Unlike Google's KG (facts about the world), a PKG captures a user's *understanding* of the world.

## 2. Link Types (Ontology)

Standard ontologies exist for different domains. For a Zettelkasten/Research tool, two categories are relevant:

### A. Structural/Hierarchical (The "Skeleton")
These organize the notes.
*   `part-of` / `has-part`: Meronymy. (Qipu has `part-of`)
*   `is-a` / `instance-of`: Hyponymy/Taxonomy. (Qipu implies this via tags, but graph edges are stronger).
*   `next` / `previous` (or `follows`): Sequences/Chains (Folgezettel).

### B. Argumentative/Rhetorical (The "Meat")
These capture the reasoning process (IBIS - Issue-Based Information System is a common standard here).
*   `supports` / `contradicts`: Evidential relationships. (Qipu has these).
*   `answers` / `questions`: Problem-solution mapping.
*   `refines` / `alternatives`: Iteration on ideas.

### C. Associative
*   `related`: The generic "see also".
*   `analogy`: "A is like B".

## 3. Inference & Reasoning
"Smart" traversal often requires simple inference rules to avoid manual double-linking.

*   **Symmetry**: If A `related` B, then B `related` A. (Qipu inline links are implicitly bidirectional in traversal, but typed links might need this).
*   **Inversion**: If A `part-of` B, then B `has-part` A. If A `supports` B, B `supported-by` A.
*   **Transitivity**: If A `part-of` B and B `part-of` C, then A `part-of` C. (Crucial for "fetch all context for C").

## 4. Graph Analysis Metrics
Beyond just listing neighbors, graph algorithms identify "important" notes.

*   **Degree Centrality**: "Hubs" (lots of connections). Good for finding MOC candidates.
*   **Betweenness Centrality**: "Bridges" (connects two clusters). Good for finding interdisciplinary insights.
*   **PageRank**: "Authority". Good for finding the foundational notes in a mess of fleeting notes.

## 5. Visualizing vs. Traversing
*   **Traversing**: "Walk the path" (LLM context, reading order).
*   **Clustering**: "Group by community" (topic modeling).
