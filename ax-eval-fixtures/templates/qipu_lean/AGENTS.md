# Qipu Agent Guide

Qipu is a knowledge-graph CLI for agents: typed notes, typed links, and a line-oriented `records`
output format for feeding results back into a prompt.

## Setup
A store is provisioned via `QIPU_STORE` — operate on it directly; don't unset it or pass
`--store`/`--root`. If no store exists yet, run `qipu init`. Run `qipu prime` for the full
note/link ontology and recent notes.

## Core loop
- `qipu create "Title" --body "..." --type <type> -t <tag>` — capture a note.
- `qipu link add <from-id> <to-id> --type <rel>` — connect two notes.
- `qipu search "query" --format records` — find notes (BM25, AND semantics).
- `qipu context --query "..." --format records` — assemble a context bundle for a topic.
  Note: context also pulls in tag-similar notes by default; pass `--related 0` for exactly what
  you selected.
- `qipu doctor` — check store consistency.

Note types: `fleeting`, `literature`, `permanent`, `moc` (map of content).

## Grouping notes with a MOC
When several notes share a topic, create a MOC as their shared entry point:
1. `qipu create "Topic" --type moc`
2. Link each member **from the MOC**: `qipu link add <moc-id> <member-id> --type has-part`.
   The MOC must be the link *source* — otherwise the member won't appear in `qipu context --moc`.
3. `qipu doctor` flags an empty MOC.

Use `--format records` whenever qipu output goes back into a prompt.
