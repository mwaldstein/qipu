# Usage Patterns and Workflows

This document is early, non-authoritative guidance and examples.

It describes intended usage patterns, not requirements. CLI commands shown here may be aspirational and can drift from the current implementation.

Treat `specs/` as the source of truth for anything implementable.

Status: Draft  
Last updated: 2026-01-12

## Workflow: beads-style agent session
Beads encourages a consistent session shape (find ready work -> do work -> sync). Qipu should encourage an analogous workflow for knowledge:

1. Start session with a primer:
   - `qipu prime`
2. Identify the research/knowledge backlog:
   - `qipu inbox`
3. Capture external research as it happens:
   - `qipu capture --type literature --tag <topic>`
4. Distill raw research into durable notes:
   - `qipu create "<insight>" --type permanent --open`
5. Curate via a MOC (topic index / whitepaper outline):
   - `qipu create "<topic>" --type moc --open`
6. When working with an LLM tool, generate a working set:
   - `qipu context --moc <moc-id> --max-chars 20000`
7. End session by ensuring indexes/validations are current:
   - `qipu sync`

## Workflow: research-heavy implementation task
Goal: keep external research and intermediate insights available throughout a coding task.

1. Create a MOC for the task topic:
   - `qipu create "OAuth provider research" --type moc --tag auth --open`
2. Capture sources as literature notes:
   - `qipu create "Auth0 docs: PKCE flow" --type literature --tag auth --tag oauth --open`
3. Distill key takeaways into permanent notes:
   - `qipu create "When to use PKCE" --type permanent --tag oauth --open`
4. Link distilled notes into the MOC.
5. When working with an LLM tool, generate a bundle:
   - `qipu context --moc <moc-id> --max-chars 20000`

## Workflow: "insight that isn't a spec yet"
Goal: capture ideas before they become commitments.

- Use fleeting notes for early thoughts.
- Promote to permanent notes when the idea stabilizes.
- Only after review/consensus should it become a spec/ticket.

## Workflow: authoring a white paper
Goal: structure research so it can be turned into a publishable document.

1. Create a MOC representing the paper outline.
2. Capture sources as literature notes (each with URL/title/access date).
3. Distill claims into permanent notes, each referencing supporting literature notes.
4. Export a draft bundle:
   - `qipu export --moc <paper-moc-id> --format markdown > paper-notes.md`

## Workflow: team collaboration
- Commit `.qipu/notes/` and `.qipu/mocs/` to the repo.
- Review notes via PRs like any other docs.
- Avoid committing derived indexes/caches unless the team wants them.

## Workflow: personal/stealth usage
- Initialize in stealth mode so `.qipu/` is not committed:
  - `qipu init --stealth`

## Bridging to tasks (beads/tickets)
Qipu is intentionally not a task tracker, but it should support a clean boundary:

- Notes can contain "Next steps" sections.
- `qipu export` may support a task-extraction format that a task tool can ingest.

## Anti-patterns
- Putting large code excerpts into qipu notes instead of linking to source files.
- Treating qipu as a dumping ground with no links/tags/MOCs.
- Using deep folder hierarchies as the primary organization mechanism.

## Open questions
- Should qipu provide a first-class "promote" command (fleeting -> permanent)?
- Should qipu support per-repo and global stores simultaneously?
