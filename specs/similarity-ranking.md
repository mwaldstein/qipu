# Content Similarity & Ranking (Bag-of-Words)

## Motivation
While Qipu does not use vector embeddings (to avoid complexity/dependencies), it still needs a way to find "similar" notes beyond direct links. This enables:
- "See Also" suggestions.
- Duplicate detection.
- Clustering for MOC generation.

## Approach: BM25 / TF-IDF
We will implement a classic, robust "Bag-of-Words" approach.

### 1. Term Frequency (TF)
- **Tokenization**: Simple whitespace/punctuation splitting.
- **Stop Words**: Remove common English stop words (the, a, is, etc.).
- **Stemming**: Optional (Porter stemmer) to match `graph` and `graphs`.
- **Fields**:
    - `title`: Weight 2.0
    - `tags`: Weight 1.5
    - `body`: Weight 1.0

### 2. Inverse Document Frequency (IDF)
- Calculate how "rare" a term is across the entire store.
- Rare terms (e.g., "zettelkasten", "ontology") carry more weight than common terms (e.g., "note", "system").

### 3. Similarity Score
- Compute the **Cosine Similarity** between the TF-IDF vectors of two notes.
- Result: A score from 0.0 (no overlap) to 1.0 (identical content).

## Use Cases

### A. "Related Notes" (Context Expansion)
When building an LLM context bundle:
1.  Fetch directly linked notes (Graph).
2.  Fetch high-similarity unlinked notes (Similarity).
    *   *Threshold*: Score > 0.3 (tunable).

### B. Duplicate Detection (`qipu doctor`)
- Find pairs of notes with Score > 0.85.
- Flag as potential duplicates.
- Suggest merging or adding a `same-as` link.

### C. Search Ranking
- Use BM25 (Best Matching 25) for full-text search ranking.
- This is the industry standard for "keyword search" and works extremely well without neural networks.

## Implementation Plan (Future)
- This logic belongs in the `qipu index` or `qipu-server` component.
- It can be computed lazily or incrementally updated.
