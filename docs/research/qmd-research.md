# QMD Research Document

## Overview

QMD is a CLI search engine for markdown knowledge bases that combines full-text search, vector embeddings, and LLM re-ranking - all running locally via node-llama-cpp with GGUF models.

**Repository**: https://github.com/tobi/qmd  
**Stars**: 4.7k  
**Language**: TypeScript (73%), Python (26%)  
**Runtime**: Bun

---

## Core Features

### 1. Hybrid Search System
Combines three complementary approaches:

| Search Type | Command | Use Case | Technology |
|-------------|----------|-----------|------------|
| **BM25 Full-Text** | `qmd search` | Exact keyword matches | SQLite FTS5 |
| **Vector Semantic** | `qmd vsearch` | Conceptual similarity | Cosine distance on embeddings |
| **Hybrid + Rerank** | `qmd query` | Highest quality results | RRF fusion + LLM reranking |

### 2. Query Expansion
- Uses fine-tuned LLM (1.7B GGUF) to generate query variations
- Generates `lex` (keyword), `vec` (semantic), and `hyde` (hypothetical document) expansions
- Original query weighted 2x in fusion to preserve exact matches

### 3. MCP Server Integration
Exposes tools via Model Context Protocol:
- `qmd_search` - BM25 keyword search
- `qmd_vsearch` - Semantic vector search  
- `qmd_query` - Hybrid search with reranking
- `qmd_get` - Retrieve single document
- `qmd_multi_get` - Batch retrieve by glob pattern
- `qmd_status` - Index health info

Resources accessible via `qmd://` URI scheme.

### 4. Collection & Context Management
- **Collections**: Named groups of markdown files with glob patterns (YAML-configured)
- **Contexts**: Hierarchical metadata describing content types
  - Global context applies to all collections
  - Path-specific contexts inherit from parent directories
  - Example: "/talks" → "Conference presentations and keynotes"

---

## Architecture

### Data Model

```
┌─────────────────────────────────────────────────────────┐
│                    SQLite Database                  │
├─────────────────────────────────────────────────────────┤
│  collections (YAML)                                │
│  ├── name, path, pattern, context map               │
│  └── update commands (optional bash scripts)          │
├─────────────────────────────────────────────────────────┤
│  content (content-addressable storage)               │
│  ├── hash (PK)                                    │
│  ├── doc (markdown content)                         │
│  └── created_at                                   │
├─────────────────────────────────────────────────────────┤
│  documents (filesystem mapping)                      │
│  ├── id, collection, path, title                   │
│  ├── hash (FK → content)                           │
│  ├── active (soft delete flag)                      │
│  └── created_at, modified_at                       │
├─────────────────────────────────────────────────────────┤
│  documents_fts (FTS5 virtual table)               │
│  ├── filepath, title, body                          │
│  └── tokenizers: porter + unicode61                 │
├─────────────────────────────────────────────────────────┤
│  content_vectors (embeddings)                        │
│  ├── hash, seq, pos (chunk sequence/position)       │
│  ├── model, embedded_at                            │
│  └── PK: (hash, seq)                             │
├─────────────────────────────────────────────────────────┤
│  vectors_vec (sqlite-vec extension)                 │
│  ├── hash_seq TEXT PK                               │
│  ├── embedding float[dimensions]                     │
│  └── distance_metric=cosine                         │
├─────────────────────────────────────────────────────────┤
│  llm_cache (response caching)                       │
│  ├── hash (PK), result, created_at                  │
└─────────────────────────────────────────────────────────┘
```

### Storage Design Patterns

**Content-Addressable Storage**: Content stored by SHA-256 hash, documents reference hashes. This enables deduplication - if multiple files have identical content, only one copy is stored.

**Soft Deletes**: Documents marked `active=0` instead of deletion. Allows history tracking and easier cleanup operations.

**Vector Index**: Uses sqlite-vec for cosine similarity search. Primary key is `hash_seq` combining content hash and chunk sequence number.

---

## Search Pipeline

### Hybrid Query Flow

```
Query: "authentication flow"

┌─────────────────────────────────────────────────────┐
│ Step 1: Query Expansion (LLM)                   │
├─────────────────────────────────────────────────────┤
│ Output:                                           │
│   lex: authentication                             │
│   lex: login flow                                │
│   vec: how do I authenticate                     │
│   hyde: To authenticate a user, you must...      │
│                                                   │
│ Original query: "authentication flow" (×2 weight)   │
└──────────────────┬────────────────────────────────┘
                   │
        ┌──────────┴──────────┐
        ▼                     ▼
┌──────────────┐      ┌──────────────┐
│   BM25 FTS   │      │   Vector     │
│   (parallel)  │      │   (parallel)  │
└──────┬───────┘      └──────┬───────┘
       │                     │
       └──────────┬──────────┘
                  ▼
┌────────────────────────────────────────┐
│ Step 2: RRF Fusion                 │
├────────────────────────────────────────┤
│ score = Σ(weight / (k + rank + 1)) │
│ k = 60, weight = 2 for original    │
│ Top-rank bonus: +0.05 (#1), +0.02 │
│ Keep top 30 candidates             │
└───────────────┬────────────────────┘
                │
                ▼
┌────────────────────────────────────────┐
│ Step 3: LLM Reranking             │
├────────────────────────────────────────┤
│ Cross-encoder scores relevance        │
│ (yes/no with logprob confidence)    │
│ Model: qwen3-reranker (0.6B)     │
└───────────────┬────────────────────┘
                │
                ▼
┌────────────────────────────────────────┐
│ Step 4: Position-Aware Blending    │
├────────────────────────────────────────┤
│ RRF rank 1-3:  75% retrieval      │
│ RRF rank 4-10: 60% retrieval     │
│ RRF rank 11+:  40% retrieval     │
│ (trust reranker more for lower     │
│  confidence retrieval results)       │
└───────────────┬────────────────────┘
                │
                ▼
        Final Ranked Results
```

### Reciprocal Rank Fusion (RRF)

Formula: `score = Σ(weight_i / (k + rank_i + 1))`

- `k = 60` (standard constant for damping)
- Original query weighted 2x, expansions weighted 1x
- Documents ranking #1 get +0.05 bonus, #2-3 get +0.02
- Preserves exact matches that might be diluted by query expansion

### Position-Aware Blending

Blends RRF retrieval score with LLM reranker score based on retrieval rank:

| RRF Rank | RRF Weight | Reranker Weight |
|-----------|-------------|-----------------|
| 1-3       | 75%         | 25%             |
| 4-10      | 60%         | 40%             |
| 11+       | 40%         | 60%             |

Rationale: High-confidence exact matches (top ranks) are preserved, while reranker influences lower-confidence results more.

---

## LLM Integration

### Models

| Model | Purpose | Size | Source |
|-------|---------|------|--------|
| embeddinggemma-300M-Q8_0 | Vector embeddings | ~300MB | ggml-org |
| qwen3-reranker-0.6b-q8_0 | Cross-encoder reranking | ~640MB | ggml-org |
| qmd-query-expansion-1.7B-q4_k_m | Query generation | ~1.1GB | tobil (fine-tuned) |

### Model Lifecycle

```
LlamaCpp Instance
├── Models (loaded once, kept warm)
│   ├── embedModel (LlamaModel)
│   ├── generateModel (LlamaModel)
│   └── rerankModel (LlamaModel)
└── Contexts (created per session, disposed on idle)
    ├── embedContext (LlamaEmbeddingContext)
    └── rerankContext (RankingContext)
```

**Inactivity Management**:
- Default timeout: 5 minutes
- Contexts disposed on inactivity (models remain loaded)
- Session tracking prevents disposal during active operations
- Optional model disposal available for aggressive memory reclaim

### Session Management

```typescript
await withLLMSession(async (session) => {
  const expanded = await session.expandQuery(query);
  const embeddings = await session.embedBatch(texts);
  const reranked = await session.rerank(query, docs);
  return reranked;
}, { maxDuration: 10 * 60 * 1000, name: 'querySearch' });
```

- Scoped access with automatic cleanup
- Abort signal support
- Max duration enforcement (default: 10 minutes)
- Reference counting for concurrent operations

---

## Document Processing

### Chunking Strategy

**Token-based chunking** (primary method):
- 800 tokens per chunk
- 120 tokens overlap (15%)
- ~4 characters per token approximation for sync fallback

**Break point priority**:
1. Paragraph break (`\n\n`)
2. Sentence end (`. `, `.\n`, `? `, `!\n`, `! `)
3. Line break
4. Word space

### Title Extraction

Format-specific extractors:
- `.md`: First heading (`#` or `##`) after optional "📝 Notes" or "Notes" header
- `.org`: `#+TITLE:` property or first `*` heading
- Fallback: Filename without extension

### Handelize (Path Normalization)

Converts paths to token-friendly format:
- `___` → `/` (folder separator)
- Non-word chars → `-`
- Preserves folder structure and file extension
- Example: `My__Docs/file (1).md` → `my-docs/file-1.md`

---

## Output Formats

All commands support multiple output formats:

| Format | Use Case |
|--------|----------|
| `cli` | Colored terminal output (default) |
| `json` | Structured data for scripting/LLMs |
| `csv` | Spreadsheet import |
| `md` | Markdown with headings |
| `xml` | XML structured data |
| `files` | Simple list (docid,score,filepath,context) |

### CLI Output Example

```
docs/guide.md:42 #a1b2c3
Title: Software Craftsmanship
Context: Work documentation
Score: 93%

This section covers the **craftsmanship** of building
quality software with attention to detail.
See also: engineering principles
```

Components:
- **Path**: Collection-relative path
- **Docid**: 6-char hash for quick reference (`#abc123`)
- **Title**: Extracted heading or filename
- **Context**: Path metadata from context system
- **Score**: Color-coded (green >70%, yellow >40%)
- **Snippet**: Contextual excerpt with query highlighting

---

## Configuration

### Collection Config (`~/.config/qmd/index.yml`)

```yaml
global_context: "Knowledge base for my projects"

collections:
  notes:
    path: ~/Documents/notes
    pattern: "**/*.md"
    context:
      "/": "Personal notes and ideas"
      "/work": "Work-related notes"
    update: "git pull"  # Optional bash command

  docs:
    path: ~/work/docs
    pattern: "**/*.md"
    context:
      "/api": "API reference documentation"
```

### Database Location

Default: `~/.cache/qmd/index.sqlite`

Override via `INDEX_PATH` environment variable or `--index` flag.

---

## Key Implementation Patterns

### 1. Content-Addressable Storage

Content stored once by hash, referenced by multiple documents:

```sql
INSERT INTO content (hash, doc, created_at)
VALUES (?, ?, ?);

INSERT INTO documents (collection, path, title, hash, ...)
VALUES (?, ?, ?, ?, ...);
```

Benefits:
- Deduplication across identical files
- Easy change detection (hash comparison)
- Efficient re-embedding (only changed hashes)

### 2. Triggers for FTS Sync

SQLite triggers keep full-text index in sync:

```sql
CREATE TRIGGER documents_ai AFTER INSERT ON documents
WHEN new.active = 1
BEGIN
  INSERT INTO documents_fts(rowid, filepath, title, body)
  SELECT new.id, new.collection || '/' || new.path, new.title,
         (SELECT doc FROM content WHERE hash = new.hash);
END;
```

Similar triggers for UPDATE and DELETE ensure consistency.

### 3. Virtual Path System

`qmd://collection/path.md` URIs abstract collection structure:

```typescript
// Parse virtual path
parseVirtualPath("qmd://notes/journal/2025-01.md")
// → { collectionName: "notes", path: "journal/2025-01.md" }

// Resolve to filesystem
resolveVirtualPath(db, "qmd://notes/journal/2025-01.md")
// → "/home/user/Documents/notes/journal/2025-01.md"
```

Benefits:
- Collection-agnostic addressing
- Easy URI scheme for MCP resources
- Portable references across different setups

### 4. Lazy Model Loading

Models loaded on first use, kept warm for session duration:

```typescript
private async ensureEmbedModel(): Promise<LlamaModel> {
  if (this.embedModel) return this.embedModel;
  if (this.embedModelLoadPromise) {
    return await this.embedModelLoadPromise;
  }
  
  this.embedModelLoadPromise = (async () => {
    const path = await this.resolveModel(this.embedModelUri);
    this.embedModel = await llama.loadModel({ modelPath: path });
    this.touchActivity();  // Reset inactivity timer
    return this.embedModel;
  })();
  
  return await this.embedModelLoadPromise;
}
```

### 5. Prompt Engineering for Query Expansion

Structured grammar forces predictable output:

```typescript
const grammar = await llama.createGrammar({
  grammar: `
    root ::= line+
    line ::= type ": " content "\\n"
    type ::= "lex" | "vec" | "hyde"
    content ::= [^\\n]+
  `
});
```

Ensures parseable output without regex post-processing.

---

## Dependencies

### Runtime Dependencies

| Package | Purpose |
|---------|---------|
| `node-llama-cpp` | Local LLM inference (GGUF models) |
| `sqlite-vec` | Vector similarity search extension |
| `bun:sqlite` | SQLite database (Bun built-in) |
| `yaml` | YAML configuration parsing |
| `zod` | Schema validation |
| `@modelcontextprotocol/sdk` | MCP server implementation |

### System Requirements

- **Bun** >= 1.0.0
- **macOS**: Homebrew SQLite (for extension support)

### Build/Install

```bash
# Global install
bun install -g github:tobi/qmd

# Development
git clone https://github.com/tobi/qmd
cd qmd
bun install
bun link
```

---

## CLI Commands

### Collection Management

```bash
qmd collection add . --name myproject
qmd collection add ~/Documents/notes --name notes --mask "**/*.md"
qmd collection list
qmd collection remove myproject
qmd collection rename myproject my-project
```

### Context Management

```bash
qmd context add qmd://notes "Personal notes and ideas"
qmd context add "Work-related notes"  # Uses current directory
qmd context add / "Global context for all collections"
qmd context list
qmd context rm qmd://notes/old
```

### Indexing

```bash
qmd update                    # Re-index all collections
qmd update --pull             # Re-index with git pull first
qmd embed                    # Generate embeddings (800 tokens/chunk, 15% overlap)
qmd embed -f                 # Force re-embed everything
qmd status                   # Show index health
qmd cleanup                  # Clean up orphaned data
```

### Search

```bash
qmd search "authentication flow"                    # BM25 only
qmd search "API" -c notes -n 10 --min-score 0.3   # Filter options

qmd vsearch "how to login"                          # Vector only
qmd vsearch "deployment" -c docs --json               # Output format

qmd query "user authentication"                       # Hybrid + rerank
qmd query "quarterly planning" -n 10 --full          # Full docs
```

### Document Retrieval

```bash
qmd get "docs/api-reference.md"
qmd get "#abc123"                           # By docid
qmd get "meeting.md:50" -l 100               # From line 50, max 100 lines

qmd multi-get "journals/2025-05*.md"       # Glob pattern
qmd multi-get "doc1.md, doc2.md"            # Comma-separated list
qmd multi-get "docs/*.md" --max-bytes 20480  # Skip large files
```

### MCP Server

```bash
qmd mcp
```

Claude Desktop configuration:
```json
{
  "mcpServers": {
    "qmd": {
      "command": "qmd",
      "args": ["mcp"]
    }
  }
}
```

---

## Testing

QMD includes comprehensive test coverage:

| Test File | Purpose |
|-----------|---------|
| `cli.test.ts` | CLI command integration tests (33KB) |
| `llm.test.ts` | LLM abstraction layer tests (20KB) |
| `mcp.test.ts` | MCP server protocol tests (33KB) |
| `store.test.ts` | Database operations tests (90KB) |
| `store-paths.test.ts` | Path resolution tests (15KB) |
| `eval.test.ts` | Search quality evaluation (16KB) |

---

## Design Principles

1. **Local-First**: All processing runs on-device using local GGUF models
2. **Privacy**: No external API calls, no cloud dependencies
3. **Composability**: Hybrid search combines complementary approaches
4. **Performance**: Lazy loading, caching, and inactivity management
5. **Extensibility**: YAML configuration, MCP integration, multiple output formats
6. **Type Safety**: TypeScript with Zod validation for schemas

---

## Comparison to Traditional Search

| Aspect | Traditional Search | QMD |
|--------|-------------------|------|
| **Search Backend** | Elasticsearch, Solr | SQLite FTS5 + sqlite-vec |
| **Deployment** | Server/cluster | Single CLI binary |
| **Privacy** | Cloud or on-prem | 100% local |
| **Semantic Understanding** | Optional (via plugins) | Built-in (vector + LLM) |
| **Query Understanding** | Keywords only | Expansion + reranking |
| **Resource Usage** | Heavy (GB+) | Light (MBs of models) |
| **Setup Complexity** | High (servers, config) | Low (bun install) |

---

## Potential Applications for Qipu

### Direct Feature Parity
- Hybrid search (BM25 + vector + LLM reranking)
- Query expansion with LLM
- MCP server integration
- Virtual path system (qipu:// vs qmd://)
- Collection management with YAML config

### Architectural Inspiration
- Content-addressable storage with hash-based deduplication
- Session-managed LLM operations
- Inactivity-based model lifecycle
- RRF fusion with position-aware blending

### Adaptation Considerations
- **Note types**: Qipu has specific note types (fleeting, literature, permanent, moc) - could inform context metadata
- **Link ontology**: Qipu's 9 link types could be incorporated into search ranking
- **ULID IDs**: Qipu uses ULID-based IDs vs QMD's 6-char hash prefixes
- **Language**: Qipu is Rust vs QMD's TypeScript - trade-offs in ecosystem vs performance

---

## References

- **Repository**: https://github.com/tobi/qmd
- **Node-LLM-CPP**: https://github.com/withcatai/node-llama-cpp
- **SQLite-VEC**: https://github.com/asg017/sqlite-vec
- **MCP Spec**: https://modelcontextprotocol.io/
- **BM25 Paper**: Robertson & Zaragoza (2009)
- **RRF Paper**: Cormack et al. (2009)
