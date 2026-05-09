# Issue Tracker: bd / Beads

Issues and PRDs for this repo live in bd / beads. Use the `bd` CLI for issue operations from the repo root.

## Conventions

- **Workflow context**: `bd prime`
- **Find ready work**: `bd ready`
- **Create an issue**: `bd create "Title" --type task --priority 2`
- **Read an issue**: `bd show <id>`
- **List issues**: `bd list`
- **Update an issue**: `bd update <id> ...`
- **Close an issue**: `bd close <id>`
- **Sync issue state**: `bd sync`

Issue IDs use the beads format already present in this repo, such as `qipu-xnlu`.

## When a skill says "publish to the issue tracker"

Create a bd issue. Use the issue type and priority that match the artifact:

- Use `--type task` for implementation work.
- Use `--type bug` for confirmed defects.
- Use `--type feature` for user-visible feature requests.
- Use `--type chore` for maintenance work.
- Use `--priority 1` for urgent work, `--priority 2` for normal work, and `--priority 3` for lower-priority work.

Prefer a heredoc or temporary file for long descriptions so shell quoting cannot corrupt markdown.

## When a skill says "fetch the relevant ticket"

Run `bd show <id>`. If the work queue is needed, run `bd ready` first and then inspect the chosen issue with `bd show <id>`.

## Session-End Sync

When completing issue work, close the issue and run `bd sync` before making a final commit so the beads state lands with the code changes.
