# CLI Interface

Status: Draft  
Last updated: 2026-01-12

## CLI principles (beads-aligned)
- **Scriptable by default**: commands should work non-interactively.
- **Deterministic output**: especially with `--json`, `prime`, and `context`.
- **Composable**: stdin/stdout friendly.
- **Fast**: listing/search should be instant for typical repos.
- **Minimize cognitive overload**: prefer flags on existing commands; group related operations.

## Global flags (proposed)
- `--store <path>`: path to store root (default: `.qipu/`)
- `--root <path>`: resolve store relative to this directory (default: cwd)
- `--json`: machine-readable output (should be supported on all commands)
- `--token`: token-optimized output for LLM context injection (mutually exclusive with `--json`; see `specs/token-optimized-output.md`)
- `--quiet` / `--verbose`

## Commands
### `qipu init`
Create a new store directory (and config/templates).

- Idempotent: safe to run multiple times.
- Should support a non-interactive mode for agents.

Common flags:
- `--stealth`: create a local-only store (gitignored)
- `--visible`: use `qipu/` instead of `.qipu/`
- `--branch <name>`: optional protected-branch workflow configuration

### `qipu create <title>` (alias: `qipu new`)
Create a new note and print its ID/path.

Common flags:
- `--type <fleeting|literature|permanent|moc>`
- `--tag <tag>` (repeatable)
- `--open` (open in `$EDITOR`)

### `qipu capture`
Create a new note from stdin.

Examples:
- `pbpaste | qipu capture --type fleeting --tag docs`
- `qipu capture --title "Thoughts on indexing" < notes.txt`

### `qipu list`
List notes.

Filters:
- `--tag <tag>`
- `--type <type>`
- `--since <date>`

### `qipu inbox`
List “unprocessed” notes (the knowledge processing queue).

Default behavior (proposed):
- `type in {fleeting, literature}`
- optionally exclude notes already linked into a MOC

This is the knowledge analog of `bd ready`.

### `qipu show <id-or-path>`
Print a note to stdout.

### `qipu search <query>`
Search within qipu notes (titles + bodies).

Notes:
- This is not intended to replace repo-wide code search.

### `qipu link`
Manage and traverse note links (typed + inline).

Subcommands (proposed):
- `qipu link add <from> <to> --type <related|derived-from|supports|contradicts|part-of>`
- `qipu link remove <from> <to> --type <...>`
- `qipu link list <id-or-path> [--direction <out|in|both>] [--typed-only|--inline-only] [--type <t>]`
- `qipu link tree <id-or-path> [--direction <out|in|both>] [--max-depth <n>] [--typed-only|--inline-only] [--type <t>]`
- `qipu link path <from> <to> [--direction <out|in|both>] [--max-depth <n>] [--typed-only|--inline-only] [--type <t>]`

Notes:
- Default traversal direction should be `both`.
- `--json` output should be supported for list/tree/path (tool-friendly).
- `--token` output should be supported for list/tree/path (LLM-friendly; see `specs/token-optimized-output.md`).

This is intentionally similar to beads’ `bd dep` commands, but for knowledge edges.

### `qipu index`
Build/refresh derived indexes (tags, backlinks, graph).

Flags:
- `--rebuild`

### `qipu context`
Build an LLM-friendly context bundle.

Selection options (one or more):
- `--note <id>` (repeatable)
- `--tag <tag>`
- `--moc <id>` (include that MOC and its linked notes)
- `--query <text>` (search-based selection)

Budgeting:
- `--max-chars <n>` or `--max-tokens <n>` (approx)

Output profiles (proposed):
- default: markdown
- `--json`
- `--token` (see `specs/token-optimized-output.md`)

### `qipu export`
Export notes into a single document (e.g., whitepaper notes).

### `qipu prime`
Emit a small, stable “primer” intended for agent session start.

Beads uses `bd prime` to inject workflow context for agents; qipu should provide the same pattern for knowledge:
- quick command reference
- store location
- top MOCs / recently updated notes (bounded output)

### `qipu setup`
Install qipu integration instructions for common agent tools.

This should follow beads’ recipe approach:
- `qipu setup --list`
- `qipu setup <tool>`
- `qipu setup --print`
- `qipu setup <tool> --check | --remove`

At minimum, support the AGENTS.md standard (cross-tool compatible).

### `qipu sync`
Optional convenience command for multi-agent workflows.

Proposed responsibilities:
- ensure derived indexes are up to date (`qipu index`)
- optionally run validations (`qipu doctor`)
- if branch/automation mode is configured, optionally commit/push qipu changes

### `qipu doctor`
Validate store invariants.

- Prefer repairs under `qipu doctor --fix` (beads principle) over adding many one-off repair commands.

## Output formats
Human output should be readable and concise.

With `--json`, commands should emit either:
- a single JSON object, or
- JSON lines (one object per note)

At minimum, each note entry includes:
- `id`, `title`, `type`, `tags`, `path`, `created`, `updated`

## Error handling and exit codes
- `0`: success
- `1`: generic failure
- `2`: usage error (bad flags/args)
- `3`: data error (invalid frontmatter, missing store, etc.)

## Open questions
- Should qipu support interactive pickers (fzf-style) as optional UX sugar?
- Should `qipu capture` default to `--type fleeting`?
- Should `qipu sync` manage git commits/pushes, or stay index/validate-only?
