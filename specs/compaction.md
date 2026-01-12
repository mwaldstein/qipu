# Compaction (Digests and Lossless Knowledge Decay)

Status: Draft  
Last updated: 2026-01-12

## Motivation
As a knowledge store grows, the “working set” an LLM needs becomes too large to traverse or load directly.

Qipu’s answer is **lossless compaction**:
- Keep the underlying notes intact.
- Introduce **digest notes** that summarize sets of notes.
- Make default navigation **digest-first** so traversal/search/context stays small.

Important constraint: **qipu must not require calling an LLM API**. Compaction content is authored externally (by a human or LLM tool) and then registered in qipu.

## Goals
- **Lossless**: compaction never deletes or overwrites source notes.
- **Digest-first navigation**: compacted notes are hidden by default; digests are surfaced.
- **Chain-friendly**: compaction must support multi-level digests.
- **Deterministic**: compaction resolution and output ordering must be stable.
- **LLM-usable**: provide commands and guidance that let an LLM:
  - find good compaction candidates
  - register compaction relationships
  - validate that compaction is coherent and worthwhile
- **Encoding-agnostic**: the storage encoding (frontmatter keys vs typed links, etc.) is an implementation detail; the semantics are what matter.

## Non-goals
- Automatic summarization (inside qipu).
- Garbage collection / deletion of compacted notes.
- Perfect semantic clustering (compaction suggestion is heuristic).
- Forcing a specific note template or taxonomy for digests.

## Terminology
- **Note**: a normal qipu note.
- **Digest**: a note intended to stand in for a set of notes (usually shorter than the expansion).
- **Source note**: a note compacted by a digest.
- **Compaction edge**: a directed relationship `digest -> source` meaning “digest compacts source”.
- **Compactor**: the digest that compacts a given note.
- **Compaction chain**: a digest may itself be compacted by a higher-level digest.
- **Canonical note**: the “topmost” digest reached by following a compaction chain.
- **Resolved view**: the default view where references are canonicalized (digests shown, compacted notes hidden).
- **Raw view**: the view where canonicalization is disabled.
- **Contracted graph**: the effective knowledge graph used in the resolved view after canonicalization.

## Core semantics
### 1) Compaction is explicit
Qipu must represent compaction relationships explicitly (stored in the repo).

A digest must maintain a **direct source set**: the list of notes it compacts.

### 2) Lossless, hidden-by-default
If a note is compacted by any digest, it remains:
- present on disk
- searchable/inspectable on demand

…but it should be **hidden by default** in most commands.

### 3) Chaining is supported
Digests may compact other digests, producing multi-level structures.

Qipu must support queries such as:
- “What does this digest compact (directly)?”
- “What is the canonical digest for this note?”
- “Show the compaction tree under this digest (depth-limited).”

## Required invariants
To keep compaction resolution deterministic and useful for LLM tooling, qipu should enforce or at least diagnose these invariants (via `qipu doctor` and/or in `qipu compact apply`):

- **At most one active compactor per note**: a note must not be compacted by multiple digests simultaneously.
- **Acyclic compaction**: compaction edges must not form cycles.
- **No self-compaction**: a digest must not compact itself.
- **All referenced IDs resolve**: compaction edges pointing to unknown notes are data errors.

If invariants are violated, commands that rely on canonicalization should:
- produce deterministic behavior (never “pick arbitrarily”), and
- surface a clear data error suitable for tool/LLM repair.

## Canonicalization (resolved view)
### Canonical ID function
Define `canon(id)` as:
1. If `id` has no compactor, `canon(id) = id`.
2. If `id` is compacted by digest `d`, then `canon(id) = canon(d)`.

This follows compaction chains to the topmost digest.

Cycle safety:
- Canonicalization must be cycle-safe via a visited set.
- A detected cycle is a data error.

### Contracted graph
When operating in the resolved view, qipu should act as if the user’s note graph is **contracted**:
- Every node ID is mapped through `canon(id)`.
- Duplicate nodes are merged.
- Edges are mapped to `(canon(from), canon(to))`.
- Self-loops introduced by contraction are dropped.

This preserves “topic connectivity” while keeping compacted notes out of the default working set.

## Visibility rules
### Default visibility
In the resolved view, a note should be treated as **hidden** if it has a compactor.

The visible surface should therefore primarily consist of:
- notes with no compactor
- top-level digests

### Explicit access to sources
Users/LLMs must be able to access a compacted note on demand.

Introduce a compaction resolution toggle:
- `--no-resolve-compaction`: disables canonicalization and shows the raw note(s).

This flag should be supported anywhere that canonicalization would otherwise apply (e.g., show/search/context/link traversal).

## Output annotations (digest-first)
Whenever a digest is shown in the resolved view, qipu should annotate it with:

- **Direct compaction count**: `compacts=<N>` where `N` is the number of direct sources compacted by the digest.
- **Compaction percent**: `compaction=<P%>` indicating estimated savings versus expanding direct sources (see Metrics).

Optional but recommended:
- **Breadcrumb**: `via=<id>` when a digest appears specifically because some compacted note was encountered/matched.
  - For example: a search hit in a compacted note yields a result for the digest with `via=<matching-note-id>`.

Annotations should be available in human output, `--json`, and `--token` modes.

## Inspecting compaction contents
Compaction IDs should be available, but not part of the default output.

### Flags
- `--with-compaction-ids`: include direct compacted IDs for any digest entries shown.
  - This is equivalent to a compaction traversal depth of 1.
- `--compaction-depth <n>`: follow compaction edges up to `n` steps.
  - `1` = direct
  - `2+` = include compactions of compacted digests, etc.

When `--with-compaction-ids` is absent, `--compaction-depth` should have no effect.

### Determinism and bounds
Compaction expansion must be:
- deterministic (stable ordering, recommended sort: by note id)
- bounded (recommended: `--compaction-max-nodes <n>` optional)
- cycle-safe

If limits are hit, outputs must indicate truncation.

### Optional: expand to bodies
For commands that return note bodies (notably `qipu context`), provide an opt-in expansion mode:
- `--expand-compaction`: include compacted source notes in the output/bundle, depth-limited by `--compaction-depth`.

In other words:
- `--with-compaction-ids` shows “what’s inside” (IDs/metadata)
- `--expand-compaction` shows “the actual contents”

## Metrics
### Compaction percent
Compaction percent should help decide whether it’s worth drilling into compacted items.

Define:
- `digest_size = size(digest)`
- `expanded_size = sum(size(source_i))` across direct sources
- `compaction_pct = 100 * (1 - digest_size / expanded_size)`

Notes:
- If `expanded_size = 0`, compaction percent is undefined; treat as `0%`.
- The metric should be deterministic.

### Size() estimation
The default `size()` should be aligned with LLM retrieval:
- Prefer “summary-sized” estimates rather than full bodies.

Recommended default size basis (deterministic):
- extract a note summary using the same rules as token-optimized summary extraction
- measure size in estimated tokens using a stable heuristic (e.g., `ceil(chars/4)`)

Qipu may support alternate size bases as future flags (e.g., body size), but must keep defaults stable.

### Depth interaction
If `--compaction-depth` is used for compaction inspection/expansion, qipu may optionally provide depth-aware metrics (e.g., compaction percent at depth N). If implemented, it must be clearly labeled.

## Command surface (proposed)
This spec does not require a specific CLI encoding, but qipu should provide a coherent command surface for LLM-driven compaction.

### `qipu compact apply <digest-id> --note <id>...`
Register that `<digest-id>` compacts the given notes.

Requirements:
- must validate invariants (no cycles, no multi-compactor conflicts)
- idempotent (re-applying the same set should not create duplicates)
- deterministic ordering in stored representation and outputs

Input ergonomics (recommended):
- allow reading IDs from stdin (`--from-stdin`) or a file (`--notes-file`)

### `qipu compact show <digest-id>`
Show the direct compaction set:
- the direct compacted IDs
- the digest’s `compacts=<N>` and `compaction=<P%>` metrics

With `--compaction-depth <n>`, show a depth-limited compaction tree under the digest.

### `qipu compact status <id>`
Show compaction relationships for any note:
- canonical digest (`canon(id)`)
- direct compactor (if any)
- direct compacted set (if the note is a digest)

### `qipu compact report <digest-id>`
Provide mechanical checks an LLM can use to evaluate whether the compaction grouping is “good”.

Recommended outputs:
- `compacts_direct_count`
- `compaction_pct`
- boundary edge ratio (how many links from sources point outside the compaction set)
- “staleness” indicator (sources updated after digest)
- conflicts/cycles (if present)

### `qipu compact suggest`
Suggest candidate groups that may benefit from compaction.

Constraints:
- must be deterministic for the same underlying graph
- must be explainable via emitted stats

Recommended approach:
- use graph methods to find dense, relatively self-contained clumps (community/clump detection)
- rank candidates by a combination of:
  - estimated total size (token/chars)
  - node count
  - cohesion (internal edges)
  - boundary edges (external connectivity)

Recommended outputs (especially for `--json`):
- list of candidates, each with:
  - `ids[]`
  - node/edge counts
  - estimated size
  - boundary edge ratio
  - a suggested next command skeleton (e.g., `qipu compact apply <new-digest> --note ...`)

### `qipu compact guide`
Print stable, copy/pasteable guidance intended for LLM tools (and humans) to perform compaction safely.

Guidance should include:
1. How to choose a candidate (`qipu compact suggest`).
2. How to review candidate summaries (`qipu context --token` in summaries-first mode).
3. How to author a digest note (externally) with a “high signal, low tokens” structure.
4. How to register compaction (`qipu compact apply`).
5. How to validate (`qipu compact report`, plus a resolved traversal/search sanity check).

The guide may include a prompt template such as:
- “Create a digest that replaces these notes in day-to-day retrieval. Include a one-paragraph Summary, key claims, and a small section explaining when to expand into sources. Keep it short; include IDs for traceability.”

## Search and traversal behavior
### Search
In the resolved view (default):
- qipu may match against compacted source note content, but results must surface the **canonical digest**.
- if a match occurred in a compacted note, annotate the digest result with `via=<matching-source-id>` (or equivalent in JSON/token output).

With `--no-resolve-compaction`:
- qipu should return the raw matching notes (including compacted notes) without redirecting.

### Traversal
In the resolved view (default):
- traversals operate on the contracted graph (canonicalized nodes)
- compacted notes do not appear as nodes

With `--with-compaction-ids`:
- traversal outputs may include direct compaction IDs for digest nodes, without expanding note bodies.

With `--expand-compaction`:
- traversal/context outputs may include compacted source notes, depth-limited by `--compaction-depth`.

## Open questions
- Should qipu support “inactive” compaction edges for history (versioning), or only one active mapping?
- Should compaction suggestions default to excluding MOCs/spec notes, or treat them like normal notes?
- Should there be a first-class concept of “leaf source” vs “intermediate digest” in outputs?
