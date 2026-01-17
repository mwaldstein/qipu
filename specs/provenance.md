# Provenance & Source Metadata

## Motivation
To support "Enterprise-grade" trust and auditability in a local knowledge graph, Qipu needs a standard way to track where information came from. This allows agents to distinguish between "human-verified facts" and "LLM-generated speculation," and enables users to trace a note back to its original source URL or generation prompt.

This spec defines a lightweight adaptation of the W3C PROV-O standard for Qipu's markdown frontmatter.

## Standard Metadata Fields
These fields are optional but recommended for notes created by automated tools or derived from external sources.

| Field | Type | Description | PROV-O Equivalent |
| :--- | :--- | :--- | :--- |
| `source` | string (URL/Path) | The original source of the information. | `wasDerivedFrom` |
| `author` | string | Name of the human or agent who created the note. | `wasAttributedTo` |
| `generated_by` | string | Name of the LLM model (e.g., `gpt-4o`, `claude-3-5-sonnet`). | `wasGeneratedBy` |
| `prompt_hash` | string | Hash or ID of the prompt used to generate the content. | `activity` |
| `verified` | boolean | Flag indicating if a human has manually reviewed the content. | (Validation) |

## Example Frontmatter

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

## Usage Patterns

### 1. Web Capture
When capturing a webpage:
- Set `source` to the URL.
- Set `author` to the user's name (if manual) or "Qipu Clipper" (if automated).

### 2. LLM Generation
When an agent generates a note:
- Set `generated_by` to the model name.
- Set `prompt_hash` to track the prompt version.
- Set `verified: false` by default.

### 3. Human Review
When a user reviews an AI-generated note:
- They can flip `verified: true`.
- This signal can be used by `qipu context` to prioritize verified notes.

## Future Extensions
- **Commit Linking**: We rely on Git for the history of *changes* (`wasRevisionOf`), so we don't need to store revision history in frontmatter.
- **Detailed Activity**: For complex pipelines, `prompt_hash` could link to a separate "Activity Note" that describes the full generation process.
