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
- **Export issue state for git**: installed hooks run `bd hooks run pre-commit`
- **Commit issue state in Dolt**: `bd dolt commit` when auto-commit is off
- **Push issue state to Dolt remote**: `bd dolt push` when a remote is configured

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

This repo is on `bd` 1.0+ with the Dolt backend. There is no top-level `bd sync`
command. Complete issue work before the git commit:

1. Run `bd close <id>` for completed work.
2. Let the installed pre-commit hook export `.beads/issues.jsonl`.
3. If Dolt auto-commit is off and issue changes are pending, run `bd dolt commit`.
4. Include the resulting tracked `.beads/*` changes in the same git commit as the code.

Use `bd hooks list` to verify hooks. Refresh local hooks with
`bd hooks install --force`, or use `bd hooks install --beads` for Dolt-backed
hook storage under `.beads/hooks/`.
