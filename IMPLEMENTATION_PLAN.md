# Qipu Implementation Plan

Last updated: 2026-01-12

This document tracks implementation progress against the specs in `specs/`.

---

## Current Status

This implementation plan is intentionally technology-agnostic and does not track completion.

Use it as a proposed work breakdown against `specs/`.

---

## Phase 0: Project Bootstrap

- [ ] Choose implementation language/toolchain aligned with `specs/cli-tool.md`.
- [ ] Establish project structure for a single native `qipu` executable.
- [ ] Set up testing harness for deterministic integration + golden tests.
- [ ] Set up CI for build + tests across platforms.

---

## Phase 1: Foundation

- [ ] Store discovery (walk up from cwd; support `--store` and `--root`).
- [ ] Store initialization and directory layout.
- [ ] Config parsing (TOML) and defaults.
- [ ] Note parsing (frontmatter + markdown body) and deterministic serialization.
- [ ] ID and slug generation per spec.
- [ ] CLI runtime skeleton: `--help`, `--version`, global flags, exit codes.
- [ ] Deterministic error formatting for `--json`.

---

## Phase 2: Core Commands

- [ ] `qipu init` (idempotent; `--stealth`, `--visible`, optional `--branch`).
- [ ] `qipu create` / `qipu new` (new note; tags/types; optional `--open` and templates).
- [ ] `qipu capture` (stdin to note; supports non-interactive workflows).
- [ ] `qipu list` (filters; deterministic ordering; `--json`).
- [ ] `qipu show <id-or-path>` (resolve; print; `--json`).

---

## Phase 3: Indexing & Navigation

- [ ] Build/update indexes per `specs/indexing-search.md`.
- [ ] Implement `qipu index` (incremental; `--rebuild`).
- [ ] Implement `qipu search <query>` (filters; ranking; `--json`).
- [ ] Implement `qipu inbox` (unprocessed note queue; `--json`).

---

## Phase 4: Link Management & Graph Traversal

- [ ] Implement `qipu link add/remove/list` per `specs/cli-interface.md`.
- [ ] Implement `qipu link tree/path` per `specs/graph-traversal.md`.
- [ ] Ensure `--json` and `--token` output shapes are stable.

---

## Phase 5: LLM Integration (P5)

- [ ] Implement token-optimized output per `specs/token-optimized-output.md`.
- [ ] Implement `qipu prime` per `specs/llm-context.md`.
- [ ] Implement `qipu context` per `specs/llm-context.md`.

---

## Phase 6: Export (P6)

- [ ] Implement `qipu export` per `specs/export.md`.

---

## Phase 7: Compaction (P7)

- [ ] Digest note type
- [ ] Compaction edges and canonicalization
- [ ] `qipu compact apply/show/status/report/suggest/guide` commands
- [ ] Global compaction flags for other commands

---

## Phase 8: Maintenance & Validation (P8)

- [ ] `qipu doctor` - validate store invariants
- [ ] `qipu doctor --fix` - attempt repairs
- [ ] `qipu sync` - convenience workflow command

---

## Phase 9: Setup & Integration (P9)

- [ ] `qipu setup --list/--print/<tool>/--check/--remove`
- [ ] AGENTS.md integration

---

## Open Questions (from specs)

See `specs/` for detailed open questions on:

- Storage format (MOC location, path structure, attachments)
- Knowledge model (type taxonomy, link types, deduplication)
- CLI interface (interactive pickers, default behaviors)
- Indexing (JSON vs SQLite, backlink embedding)
- Graph traversal (default depth, link materialization)
- Token output (versioning, default inclusions)
- Compaction (inactive edges, exclusions)
- LLM context (summarization, backlinks)
- Export (Pandoc integration, transitive links)

---

## Implementation Notes

### Dependency Graph (Phases)

```
Phase 0 (Bootstrap)
    |
    v
Phase 1 (Foundation)
    |
    v
Phase 2 (Core Commands)
    |
    v
Phase 3 (Indexing)
    |
    +-----> Phase 4 (Graph Traversal)
    |              |
    +--------------+-----> Phase 5 (LLM Integration)
    |
    v
Phase 6 (Export)
    |
    v
Phase 7 (Compaction)
    |
    v
Phase 8 (Maintenance)
    |
    v
Phase 9 (Setup)
```

### Testing Strategy

- Integration tests for CLI commands (temporary directory stores)
- Golden tests for deterministic outputs
- Property-based tests where useful (e.g. ID collision resistance)
