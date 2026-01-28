# Provenance & Source Metadata

## Motivation
To support "Enterprise-grade" trust and auditability in a local knowledge graph, Qipu needs a standard way to track where information came from. This allows agents to distinguish between "human-verified facts" and "LLM-generated speculation," and enables users to trace a note back to its original source URL or generation prompt.

This spec defines a lightweight adaptation of the W3C PROV-O standard for Qipu's markdown frontmatter.

## Standard Metadata Fields
These fields are optional but recommended for notes created by automated tools or derived from external sources.

| Field | Type | Description | PROV-O Equivalent |
| :--- | :--- | :--- | :--- |
| `source` | string (URL/Path) | The original source of the information (lightweight). | `wasDerivedFrom` |
| `sources` | array of objects | Structured sources with metadata (url, title, accessed). | `wasDerivedFrom` |
| `author` | string | Name of the human or agent who created the note. | `wasAttributedTo` |
| `generated_by` | string | Name of the LLM model (e.g., `gpt-4o`, `claude-3-5-sonnet`). | `wasGeneratedBy` |
| `prompt_hash` | string | Hash or ID of the prompt used to generate the content. | `activity` |
| `verified` | boolean | Flag indicating if a human has manually reviewed the content. | (Validation) |

### Source vs sources[] Semantics

- **`source` (singular string)**: Lightweight provenance for simple use cases. Just stores a URL or path. Ideal for quick capture, manual notes, or when source metadata is unavailable.

- **`sources` (array of objects)**: Rich structured sources for bibliography generation and scholarly use. Each source includes:
  - `url`: The source URL (required)
  - `title`: Source title (optional, for citation formatting)
  - `accessed`: Date accessed in ISO format `YYYY-MM-DD` (optional, for web sources)

**Relationship**: Both fields can coexist. Bibliography export reads from both fields, treating `source` as a simple URL with no additional metadata. Use `sources` when you need citation-ready metadata; use `source` for lightweight tracking.

## Example Frontmatter

### Lightweight (using `source`)
```yaml
---
id: qp-a1b2
title: "GraphRAG Architecture"
type: permanent
tags: [ai, architecture]
source: "https://arxiv.org/abs/2402.xxxx"
author: "ResearchAgent-v2"
generated_by: "gpt-4o"
verified: false
---
```

### Structured (using `sources`)
```yaml
---
id: qp-a1b2
title: "GraphRAG Architecture"
type: permanent
tags: [ai, architecture]
sources:
  - url: "https://arxiv.org/abs/2402.xxxx"
    title: "GraphRAG: Knowledge-Augmented Generation"
    accessed: "2024-01-15"
  - url: "https://example.com/tutorial"
    title: "RAG Tutorial"
    accessed: "2024-01-16"
author: "ResearchAgent-v2"
generated_by: "gpt-4o"
verified: false
---
```

## Usage Patterns

### 1. Web Capture (Lightweight)
When capturing a webpage quickly:
- Set `source` to the URL.
- Set `author` to the user's name (if manual) or "Qipu Clipper" (if automated).

### 2. Literature Notes (Structured)
When creating literature notes with bibliography intent:
- Use `sources` array with structured metadata.
- Include `title` and `accessed` dates for proper citation.
- Set `type: literature`.

### 3. LLM Generation
When an agent generates a note:
- Set `generated_by` to the model name.
- Set `prompt_hash` to track the prompt version.
- Set `verified: false` by default.

### 4. Human Review
When a user reviews an AI-generated note:
- They can flip `verified: true`.
- This signal can be used by `qipu context` to prioritize verified notes.

## Future Extensions
- **Commit Linking**: We rely on Git for the history of *changes* (`wasRevisionOf`), so we don't need to store revision history in frontmatter.
- **Detailed Activity**: For complex pipelines, `prompt_hash` could link to a separate "Activity Note" that describes the full generation process.
