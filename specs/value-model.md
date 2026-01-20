# Value Model & Weighted Traversal

## Motivation
As a knowledge base grows, the signal-to-noise ratio often decreases. Research produces duplicate information, superseded drafts, and varying levels of source quality. For both human browsing and LLM retrieval, treating all notes as "equal nodes" in the graph is inefficient. We need a way to represent the **quality** or **importance** of a note to guide traversal algorithms toward higher-quality information.

This concept aligns with established Graph Knowledge Management principles:
- **Node Centrality/PageRank**: The idea that some nodes are more "authoritative" than others.
- **Activation Energy**: In spreading activation networks, some nodes require less energy to activate (high value) than others.
- **Forgetting Curves/Spaced Repetition**: While distinct from time-based decay, explicit 'value' allows simulating "importance" manually, similar to marking a card as "leech" or "essential" in SRS systems.

## The `value` Attribute
We introduce an explicit `value` field to the note frontmatter.

- **Field**: `value`
- **Type**: Integer, `0` to `100`
- **Default**: `50` (neutral)

### Semantics
The scale is designed to be intuitive for humans but mathematically useful for weighting algorithms.

- **0-20 (Deprioritized/Junk)**:
  - Examples: Superseded drafts, raw logs known to be noisy, duplicate sources, active misinformation/refuted claims.
  - Behavior: Traversal algorithms should actively avoid these unless specifically requested.
- **21-80 (Standard)**:
  - Examples: General research, literature notes, work-in-progress.
  - Behavior: Treated with standard weight.
- **81-100 (High-Value/Gem)**:
  - Examples: Distilled insights, canonical definitions, high-quality MOCs, "landmark" notes.
  - Behavior: These act as "gravity wells" in the graph, pulling traversal paths toward them.

Example frontmatter:
```yaml
---
id: qp-a1b2
title: "The core thesis of Qipu"
type: permanent
value: 95
---
```

## Impact on Traversal (Weighted Graphs)
The primary utility of `value` is to modify the **effective weight** (or "resistance") of edges leading *to* a note during traversal.

### 1. Cost Function (Inverse Value)
In a weighted traversal (e.g., Dijkstra's algorithm or cost-limited expansion), the cost to traverse an edge `Note A -> Note B` is derived from `Note B`'s value. We model this as "resistance": higher value means lower resistance.

Proposed Cost Formula:
> `EdgeCost(A->B) = BaseCost * (1 + (100 - Value(B)) / 100)`

*Examples (assuming BaseCost = 1.0):*
- Target Value **100** (Gem):
  `Cost = 1.0 * (1 + 0) = 1.0` (Minimum resistance)
- Target Value **50** (Standard):
  `Cost = 1.0 * (1 + 0.5) = 1.5`
- Target Value **0** (Junk):
  `Cost = 1.0 * (1 + 1.0) = 2.0` (Maximum resistance)

*Note: We avoid value=0 causing infinite cost or cost=0 causing infinite loops. The cost effectively doubles for the lowest quality notes compared to the highest.*

### 2. Priority Queue Traversal
Existing commands like `qipu link tree` generally use BFS (Breadth-First Search), which expands strictly by hop count (Depth 1, then Depth 2).

With the value model, we can support a **Priority Queue** traversal mode:
- The queue is ordered by accumulated path cost.
- A high-value note at Depth 2 (Cost ~2.0) might be visited *before* a low-value note at Depth 1 (Cost ~2.0 if value is low, or if the edge weight is naturally high).
- This ensures that a limited "context budget" (e.g., for an LLM) is spent on the highest-quality reachable nodes, rather than just the nearest ones.

### 3. De-duplication Strategy
This model directly addresses the "duplicate research" problem:
1.  **Ingest**: User captures 5 articles about "Raft Consensus".
2.  **Evaluate**: User identifies one as "canonical" (`value: 90`) and marks others as repetitive (`value: 30`).
3.  **Traverse**: When an agent walks the graph for "Raft Consensus", the cost function naturally funnels it to the canonical note. The low-value duplicates are either visited last or pruned entirely if the budget is exhausted.

## Filtering & CLI
The `value` attribute enables simple threshold filtering:

- `qipu context --min-value 80`
  *Only include "Gems" in the context window.*
- `qipu link tree --min-value 20`
  *Hide "Junk" notes from the visualization.*
- `qipu search --sort value`
  *Rank results by explicit value rather than just text match score.*

## Design Decisions
- **Manual vs. Calculated**: Currently, `value` is a manual field. Future iterations could calculate a `derived_value` based on backlink count (PageRank-lite) or interaction frequency, but the explicit `value` field remains the manual override ("God mode").
- **Granularity**: 0-100 allows for nuance without being overwhelming. It maps easily to UI sliders or star ratings (0-5 stars -> 0, 25, 50, 75, 100).

## Technical Implementation Notes

### SQLite Schema
To support performant filtering and sorting (e.g., `qipu search --sort value`), the `value` field must be promoted to a first-class column in the derived SQL index (`notes` table).
- Column: `value INTEGER DEFAULT 50`
- Index: `CREATE INDEX idx_notes_value ON notes(value)`

### Algorithm: Dijkstra vs. BFS
Qipu's current traversal uses a standard Breadth-First Search (BFS) via a FIFO queue.
- **Unweighted (Standard)**: BFS is sufficient and optimal for finding "shortest path" in terms of hops.
- **Weighted (Value-based)**: To strictly respect the cost function, the traversal must use **Dijkstra's Algorithm**.
  - Implementation: Replace `VecDeque` with `BinaryHeap` (Priority Queue).
  - Ordering: Queue items are ordered by `accumulated_cost` (lowest first).
  - This ensures that a "cheap" 2-hop path (Cost 2.0) is expanded before an "expensive" 1-hop path (Cost 2.5).
