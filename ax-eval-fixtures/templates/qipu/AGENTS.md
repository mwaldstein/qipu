# Qipu Agent Guide

Qipu is a knowledge-graph CLI built for scripts and agents. Notes are typed, linkable, and
queryable, with a line-oriented `records` output format designed for LLM context injection.

## Setup
- A store is provisioned for you via the `QIPU_STORE` environment variable — operate on it directly.
- Do **not** unset `QIPU_STORE` or pass `--store`/`--root`. If no store exists yet, run `qipu init`
  (it honors `QIPU_STORE` and creates the provisioned store).

## Core workflow
1. `qipu create "Title" --body "..." --type literature -t tag1 -t tag2` — capture a note.
2. `qipu search "query" --format records` — find notes (AND semantics, BM25-ranked).
3. `qipu link add <from-id> <to-id> --type related` — connect notes.
4. `qipu link tree <id>` / `qipu link path <a> <b>` — traverse the graph.
5. `qipu context --query "..." --format records --max-chars 8000` — build an LLM context bundle.
6. `qipu compact apply <digest-id> -n <src-id>` — fold source notes into a digest.
7. `qipu prime` — session primer: note/link ontology + recent notes.
8. `qipu doctor` — check store consistency (`--fix` to auto-repair).
9. `qipu workspace new <name>` … `qipu workspace merge <name>` — isolate then merge work.

## Note types
`fleeting`, `literature`, `permanent`, `moc` (map of content).

## Standard link ontology
`related`, `derived-from`, `supports`, `contradicts`, `part-of`, `answers`, `refines`, `same-as`,
`alias-of`, `follows`. Each has a semantic inverse (shown by `qipu prime`).

## Output formats
Every command takes `--format`: `human` (default), `json`, `records`. Records use single-letter
prefixes (`H` header, `N` note, `E` edge, `L` link type, `T` note type) — ideal for feeding back
into a prompt.

## Tips
- Prefer `qipu context` over manual `search` + `show` when building LLM context.
- Use `records` whenever qipu output goes back into a prompt.
- Note IDs look like `qp-01HYX...`.
