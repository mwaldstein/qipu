# Records Output

Status: Draft  
Last updated: 2026-01-12

## Motivation
LLM tools have constrained context windows. Even when the right notes are retrieved, the *formatting overhead* of the output can consume a meaningful fraction of the available budget.

Qipu should provide a records output mode intended for context injection (including, but not limited to, LLM ingestion):
- smaller than full markdown bundles
- stable/deterministic
- still human-debuggable

This is complementary to `--format json` output:
- `--format json` is for programmatic tool integration
- `--format records` is for low-overhead context injection

## Goals
- Provide a **records** output format for:
  - `qipu prime`
  - `qipu context`
  - `qipu link tree` / `qipu link path` / `qipu link list`
- Keep output **deterministic** (stable ordering, stable formatting).
- Support **budgets** (`--max-chars` exact).
- Support **progressive disclosure**:
  - emit a small “index” view first
  - allow fetching full note bodies only when needed

## Non-goals
- Exact token counting for every model/tokenizer.
- Automatic summarization that requires calling an LLM API.

## Concept: output formats
Many qipu commands already have a human format and `--format json`.

Introduce a third output format:
- `--format records`: minimized overhead for context injection

Where applicable, qipu should support:
- `--format human` (default)
- `--format json` (stable machine output)
- `--format records` (line-oriented, low-ceremony record output)

Notes:
- `records` is intentionally allowed to evolve early-on as we observe how tools consume qipu output.
- Records output should carry an explicit format version in the header line so downstream tooling can detect changes.

## Records format (proposed)
Records format should be:
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

The grammar does not need to be perfectly machine-parseable (that’s what `--format json` is for), but it should be consistent.

### Example: records output for a traversal
```
H qipu=1 records=1 store=.qipu/ mode=link.tree root=qp-a1b2 direction=both max_hops=3 truncated=false
N qp-a1b2 permanent "Zettelkasten note types" tags=zettelkasten,qipu
E qp-a1b2 supports qp-3e7a typed
E qp-a1b2 related qp-f14c3 inline
N qp-3e7a literature "Paper: X" tags=paper
S qp-3e7a Key claim and why it matters.
```

### Example: records output for context
```
H qipu=1 records=1 store=.qipu/ mode=context notes=2 truncated=false
N qp-a1b2 permanent "Zettelkasten note types" tags=zettelkasten,qipu path=.qipu/notes/qp-a1b2-zettelkasten-note-types.md
S qp-a1b2 One-paragraph summary.
B qp-a1b2
<raw markdown body lines…>
```

## Summary extraction
Records output should prefer summaries over full bodies.

Summary extraction order:
1. Frontmatter `summary` field (if present)
2. A `## Summary` section (first paragraph under it)
3. First paragraph of the note body
4. Empty

This keeps the system usable without requiring summarization tooling.

## Budgets and truncation
- `--max-chars` is exact.

When a budget is exceeded:
- emit a final header line indicating truncation, or set `truncated=true` in the first header line
- do not emit partial records unless unavoidable

## Progressive disclosure workflow (LLM-friendly)
A recommended agent workflow:
1. Run traversal in records mode to get a small graph neighborhood:
   - `qipu link tree <id> --max-hops 2 --format records --max-chars 8000`
2. Select note IDs from that output.
3. Fetch full content only for selected notes:
   - `qipu context --note <id> --format records --with-body --max-chars 16000`

This is analogous to “retrieve a small index, then expand” patterns used in RAG systems.

## Open questions
- Should records output allow selecting a format version (e.g., `records=1` in the header) for stability?
- Should records output include edges by default, or only with `--with-edges`?
- Should records output default to summaries only, requiring `--with-body` to include full content?
