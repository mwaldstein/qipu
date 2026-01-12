# Qipu

Qipu is a local-first CLI for capturing and navigating research/knowledge so it stays available to humans and LLM coding agents.

It is intentionally inspired by beads/bd (git-backed, agent-optimized, graph-first), but focused on research/insight capture rather than task execution.

## Problem
LLM coding agents (and the humans driving them) are great at searching the current codebase, but often struggle to:

- Preserve external research (docs, blog posts, papers, issue threads) in a durable place
- Keep intermediate insights discoverable across sessions and across related tasks
- Avoid repeating the same research on future work
- Distinguish “useful knowledge” from “committed decisions” (specs) and “planned work” (tickets)

## Goals
- **Capture quickly**: record research/insights in seconds (stdin, editor, templates).
- **Structure knowledge**: Zettelkasten-inspired notes, links, tags, and maps of content.
- **Navigate easily**: list/search notes, follow backlinks, traverse topic maps.
- **LLM-friendly outputs**: deterministic “context bundles” suitable for prompt injection.
- **Git-backed**: knowledge should live in the repo as reviewable diffs.
- **Local/offline-first**: no required network access; no hosted service dependency.
- **Agent-optimized ergonomics**: stable IDs, `--json` output, and predictable commands.

## Non-goals
- **Not a replacement for code search**: qipu does not replace `rg`, `git grep`, or IDE search.
- **Not a ticket system**: qipu does not manage execution/work; use beads/Jira/GitHub issues.
- **Not an LLM runtime**: qipu should not require calling LLM APIs to be useful.
- **Not a secrets vault**: qipu should avoid storing secrets and help users keep them out.

## Target users
- Developers using agentic coding tools (opencode, claudecode) who need durable “project memory”.
- Teams doing research-heavy work and wanting a lightweight, reviewable knowledge base.
- Writers authoring white papers/design docs and needing structured research notes.

## Relationship to beads (task tracking)
Beads focuses on capturing and ordering work (“what to do”) as a git-backed graph of issues. Qipu focuses on capturing and linking knowledge (“what we learned”) as a git-backed graph of notes.

Typical flow:
1. Research happens (docs, experiments, conversations)
2. Insights are captured into qipu notes
3. When an insight becomes actionable, it is promoted into a spec/ticket/bead

Qipu may optionally export formats that help create tasks, but it should not become a task tracker.

## Success criteria
- A new note can be created and captured in under 10 seconds.
- A past insight can be found (tag/search/MOC) in under 30 seconds.
- `qipu prime` produces a small, stable primer suitable for agent session start.
- `qipu context …` produces stable, copy/pasteable output usable by LLM tools.
- The system remains usable with thousands of notes.

## Specs
Implementable specs live in `specs/` (each should map to buildable work):
- `specs/README.md`

## Open questions
- Default store location: hidden `.qipu/` (beads-aligned) vs visible `qipu/`?
- Note ID scheme: hash-based `qp-xxxx` (beads-aligned) vs ULID vs timestamp?
- Should qipu support a protected-branch workflow (commit qipu data to a separate branch)?
- Should qipu ship a `setup` command with recipes for common agent tools (AGENTS.md, Cursor rules, Claude hooks)?
- Should there be a global (cross-repo) store option?
