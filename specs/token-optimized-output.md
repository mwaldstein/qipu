# Token-Optimized Output

Status: Draft  
Last updated: 2026-01-12

## Motivation
LLM tools have constrained context windows. Even when the right notes are retrieved, the *formatting overhead* of the output can consume a meaningful fraction of the available budget.

Qipu should provide token-optimized output modes intended for LLM ingestion:
- smaller than full markdown bundles
- stable/deterministic
- still human-debuggable

This is complementary to `--json` output:
- `--json` is for programmatic tool integration
- token-optimized output is for LLM context injection

## Goals
- Provide a **token-optimized** output format for:
  - `qipu prime`
  - `qipu context`
  - `qipu link tree` / `qipu link path` / `qipu link list`
- Keep output **deterministic** (stable ordering, stable formatting).
- Support **budgets** (`--max-chars`, `--max-tokens` approximate).
- Support **progressive disclosure**:
  - emit a compact “index” view first
  - allow fetching full note bodies only when needed

## Non-goals
- Exact token counting for every model/tokenizer (provide estimates).
- Automatic summarization that requires calling an LLM API.

## Concept: output profiles
Many qipu commands already have a human format and `--json`.

Introduce a third output profile:
- **token**: optimized for LLM context

Where applicable, qipu should support:
- default human-readable output
- `--json` (stable machine output)
- `--token` (compact LLM-oriented output)

Notes:
- `--token` is intentionally allowed to evolve early-on as we observe how LLM tools use qipu.
- Token output should carry an explicit format version in the header line so downstream tooling can detect changes.
- `--token` should be mutually exclusive with `--json` (use `--json` for stable machine integration).

## Token format (proposed)
Token format should be:
- line-oriented
- low-ceremony
- easy for LLMs to parse

### Record types
Recommended record prefixes:
- `H` header line (bundle metadata)
- `N` note metadata line
- `S` summary line (optional)
- `E` edge line (optional)
- `B` body line(s) (optional; raw markdown)

The grammar does not need to be perfectly machine-parseable (that’s what `--json` is for), but it should be consistent.

### Example: token output for a traversal
```
H qipu=1 token=1 store=.qipu/ mode=link.tree root=qp-a1b2 direction=both max_depth=3 truncated=false
N qp-a1b2 permanent "Zettelkasten note types" tags=zettelkasten,qipu
E qp-a1b2 supports qp-3e7a typed
E qp-a1b2 related qp-f14c3 inline
N qp-3e7a literature "Paper: X" tags=paper
S qp-3e7a Key claim and why it matters.
```

### Example: token output for context
```
H qipu=1 token=1 store=.qipu/ mode=context notes=2 truncated=false
N qp-a1b2 permanent "Zettelkasten note types" tags=zettelkasten,qipu path=.qipu/notes/qp-a1b2-zettelkasten-note-types.md
S qp-a1b2 One-paragraph summary.
B qp-a1b2
<raw markdown body lines…>
```

## Summary extraction
Token-optimized output should prefer summaries over full bodies.

Summary extraction order:
1. Frontmatter `summary` field (if present)
2. A `## Summary` section (first paragraph under it)
3. First paragraph of the note body
4. Empty

This keeps the system usable without requiring summarization tooling.

## Budgets and truncation
- `--max-chars` is exact.
- `--max-tokens` is approximate and must be deterministic.

Recommended default token estimator:
- `estimated_tokens = ceil(chars / 4)` (simple, stable heuristic)

When a budget is exceeded:
- emit a final header line indicating truncation, or set `truncated=true` in the first header line
- do not emit partial records unless unavoidable

## Progressive disclosure workflow (LLM-friendly)
A recommended agent workflow:
1. Run traversal in token mode to get a compact graph neighborhood:
   - `qipu link tree <id> --max-depth 2 --token --max-chars 8000`
2. Select note IDs from that output.
3. Fetch full content only for selected notes:
   - `qipu context --note <id> --token --with-body --max-chars 16000`

This is analogous to “retrieve a small index, then expand” patterns used in RAG systems.

## Open questions
- Should `--token` allow selecting a format version (e.g., `--token-version 1`) for stability?
- Should token output include edges by default, or only with `--with-edges`?
- Should token output default to summaries only, requiring `--with-body` to include full content?
- Should qipu support a model-specific tokenizer option (e.g., `--tokenizer claude|openai|simple`), or keep a single stable heuristic?
