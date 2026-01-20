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

### Relationship to `verified`
The existing `verified` frontmatter field indicates human review status. While related, these are orthogonal concepts:
- `verified` = "a human reviewed this" (boolean)
- `value` = "how important/useful is this" (numeric)

A note can be verified but low-value (e.g., a reviewed-but-superseded draft), or unverified but high-value (e.g., a high-quality LLM-generated synthesis awaiting review).

### Relationship to Compaction
The `value` field complements the compaction system (see `specs/compaction.md`):
- **Compaction** hides notes structurally (replaced by digests).
- **Value** deprioritizes notes algorithmically (lower weight in traversal).

Notes with `value < 20` are strong candidates for compaction. Conversely, digest notes typically warrant high value (`≥80`) since they represent distilled knowledge.

## Impact on Traversal (Weighted Graphs)
The primary utility of `value` is to modify the **effective weight** (or "resistance") of edges leading *to* a note during traversal.

### 1. Cost Function (Inverse Value)
In a weighted traversal (e.g., Dijkstra's algorithm or cost-limited expansion), the cost to traverse an edge `Note A -> Note B` is derived from `Note B`'s value. We model this as "resistance": higher value means lower resistance.

Cost Formula:
> `EdgeCost(A->B) = LinkTypeCost(edge) * (1 + (100 - Value(B)) / 100)`

Where `LinkTypeCost(edge)` is the existing per-link-type cost from `get_link_type_cost()`. Currently this returns 1.0 for all types, but the function is designed for future per-link-type configuration (e.g., `part-of = 0.5`, `contradicts = 1.5`). The value multiplier preserves this extension point.

*Examples (assuming LinkTypeCost = 1.0):*
- Target Value **100** (Gem):
  `Cost = 1.0 * (1 + 0) = 1.0` (Minimum resistance)
- Target Value **50** (Standard):
  `Cost = 1.0 * (1 + 0.5) = 1.5`
- Target Value **0** (Junk):
  `Cost = 1.0 * (1 + 1.0) = 2.0` (Maximum resistance)

*Note: The value multiplier is bounded [1.0, 2.0], avoiding pathological costs. Combined with future link-type costs, this enables fine-grained traversal control (e.g., a `contradicts` edge to a low-value note would have high resistance).*

### 2. Priority Queue Traversal
The current BFS implementation (`src/lib/graph/bfs.rs`) uses a `VecDeque` (FIFO queue) and already tracks `accumulated_cost` via the `HopCost` type. However, it expands nodes in FIFO order rather than cost order.

With the value model, we support an opt-in **Priority Queue** traversal mode:
- Replace `VecDeque` with `BinaryHeap` (min-heap by accumulated cost).
- The queue is ordered by accumulated path cost.
- A high-value note at Depth 2 (Cost ~2.0) might be visited *before* a low-value note at Depth 1 (Cost ~2.0 if value is low).
- This ensures that a limited "context budget" (e.g., for an LLM) is spent on the highest-quality reachable nodes, rather than just the nearest ones.

### 3. De-duplication Strategy
This model directly addresses the "duplicate research" problem:
1.  **Ingest**: User captures 5 articles about "Raft Consensus".
2.  **Evaluate**: User identifies one as "canonical" (`value: 90`) and marks others as repetitive (`value: 30`).
3.  **Traverse**: When an agent walks the graph for "Raft Consensus", the cost function naturally funnels it to the canonical note. The low-value duplicates are either visited last or pruned entirely if the budget is exhausted.

## Filtering & CLI
The `value` attribute enables simple threshold filtering and traversal control.

### Weighted vs. Unweighted Traversal
By default, traversal commands (`link tree`, `link path`) use **weighted traversal** with the value-based cost function.

- **Default**: Weighted traversal (Dijkstra, cost based on target value).
  - Notes without explicit `value` default to 50, giving uniform cost (1.5).
  - Only notes with explicit values influence traversal order.
- **Flag**: `--ignore-value` (or `--unweighted`)
  - Disables the value-based cost function.
  - Treats all edges as cost 1.0.
  - Reverts to standard BFS (Breadth-First Search).
  - Useful for structural analysis where content quality is irrelevant.

### Thresholds
- `qipu context --min-value 80`
  *Only include "Gems" in the context window.*
- `qipu link tree --min-value 20`
  *Hide "Junk" notes from the visualization.*
- `qipu list --min-value 50`
  *List only notes at or above standard value.*
- `qipu search --sort value`
  *Rank results by explicit value rather than just text match score.*

## CLI Commands

### `qipu value set <id> <score>`
Set the value of a note explicitly.

- **Usage**: `qipu value set qp-a1b2 90`
- **Arguments**:
  - `id`: Note ID or path.
  - `score`: Integer 0-100.
- **Behavior**: Updates the frontmatter `value` field. If the field is missing, it is added.
- **Validation**: Rejects scores outside 0-100 range.

### `qipu value show <id>`
Display the current value of a note.

- **Usage**: `qipu value show qp-a1b2`
- **Output**: `qp-a1b2: 90` (or `qp-a1b2: 50 (default)` if unset)

## Design Decisions
- **Manual vs. Calculated**: Currently, `value` is a manual field. Future iterations could calculate a `derived_value` based on backlink count (PageRank-lite) or interaction frequency, but the explicit `value` field remains the manual override ("God mode").
- **Granularity**: 0-100 allows for nuance without being overwhelming. It maps easily to UI sliders or star ratings (0-5 stars -> 0, 25, 50, 75, 100).
- **Weighted by default**: The value-based cost function is always active. Since unset notes default to `value=50`, traversal behavior is unchanged when no values are set (all edges cost 1.5 uniformly). Users can opt out with `--ignore-value` for pure structural analysis.

## Technical Implementation Notes

### SQLite Schema
To support performant filtering and sorting (e.g., `qipu search --sort value`), the `value` field must be promoted to a first-class column in the derived SQL index (`notes` table).

```sql
ALTER TABLE notes ADD COLUMN value INTEGER DEFAULT 50;
CREATE INDEX idx_notes_value ON notes(value);
```

This requires a schema version bump (`CURRENT_SCHEMA_VERSION` in `src/lib/db/schema.rs`).

### Frontmatter Extension
Add to `NoteFrontmatter` in `src/lib/note/frontmatter.rs`:
```rust
/// Note importance/quality score (0-100, default 50)
#[serde(skip_serializing_if = "Option::is_none")]
pub value: Option<u8>,
```

### Algorithm: Dijkstra vs. BFS
The current implementation (`src/lib/graph/bfs.rs`) uses BFS with cost tracking but FIFO expansion.

**Unweighted (Default)**: Continue using `VecDeque` with FIFO semantics. All edges effectively cost 1.0 regardless of target value. This is optimal for "shortest hop path" queries.

**Weighted (Opt-in via `--weighted`)**: To strictly respect the cost function:
1. Replace `VecDeque` with `BinaryHeap` (min-heap).
2. Order queue items by `accumulated_cost` (lowest first).
3. Compute edge cost using the value-based formula.
4. This ensures a "cheap" 2-hop path (Cost 2.0) is expanded before an "expensive" 1-hop path (Cost 2.5).

### HopCost Integration
The existing `HopCost` type (`src/lib/graph/types.rs`) uses `f32` internally, supporting fractional costs. The `get_link_type_cost()` function is designed for future per-link-type costs but currently returns 1.0 for all types.

Extend or wrap this function to incorporate value:

```rust
pub fn get_edge_cost(link_type: &str, target_value: Option<u8>) -> HopCost {
    let base = get_link_type_cost(link_type);
    let value = target_value.unwrap_or(50) as f32;
    let multiplier = 1.0 + (100.0 - value) / 100.0;
    HopCost::new(base.value() * multiplier)
}
```

This preserves the link-type cost extension point while adding value-based weighting.

## Open Questions
- Should `value` influence search ranking by default, or only when explicitly sorted by value?
- Should digest notes automatically receive a value boost (e.g., `min(80, value)`) during indexing?
- Should `qipu compact suggest` factor in value when identifying compaction candidates (prefer low-value notes)?
