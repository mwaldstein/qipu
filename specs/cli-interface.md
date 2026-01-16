# CLI Interface

Status: Draft  
Last updated: 2026-01-15

## CLI principles (beads-aligned)
- **Scriptable by default**: commands should work non-interactively.
- **Deterministic output**: especially with `--format json`, `prime`, and `context`.
- **Composable**: stdin/stdout friendly.
- **Fast**: listing/search should be instant for typical repos.
- **Minimize cognitive overload**: prefer flags on existing commands; group related operations.

## Global flags (proposed)
- `--store <path>`: explicit store root path (relative paths resolve against `--root` or cwd)
- `--root <path>`: base directory for resolving the store (default: cwd)
- `--format <human|json|records>`: output format (default: `human`)
  - `json` is stable, machine-readable output (supported on all commands)
  - `records` is line-oriented, low-overhead output (see `specs/records-output.md`)
- `--quiet` / `--verbose`

## Store discovery and resolution
- Resolution order:
  1. If `--store` is provided, resolve relative to `--root` (or cwd).
  2. Otherwise walk up from `--root` (or cwd), checking `.qipu/` first, then `qipu/`.
- Missing store behavior:
  - Commands requiring an existing store exit with code `3`.
  - `qipu init` may create the store at the default location.

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

Default behavior:
- `type in {fleeting, literature}`
- `--exclude-linked`: exclude notes already linked into a MOC

### `qipu show <id-or-path>`
Print a note to stdout.

Common flags:
- `--links`: show inbound and outbound links for the note

### `qipu search <query>`
Search within qipu notes (titles + bodies).

Common flags:
- `--type <type>`: filter by note type
- `--tag <tag>`: filter by tag
- `--exclude-mocs`: exclude MOC notes from results

### `qipu link`
Manage and traverse note links (typed + inline).

Subcommands:
- `qipu link add <from> <to> --type <related|derived-from|supports|contradicts|part-of>`
- `qipu link remove <from> <to> --type <...>`
- `qipu link list <id-or-path> [--direction <out|in|both>] [--typed-only|--inline-only] [--type <t>] [--max-chars <n>]`
- `qipu link tree <id-or-path> [--direction <out|in|both>] [--max-hops <n>] [--typed-only|--inline-only] [--type <t>] [--max-chars <n>]`
- `qipu link path <from> <to> [--direction <out|in|both>] [--max-hops <n>] [--typed-only|--inline-only] [--type <t>] [--max-chars <n>]`

### `qipu compact`
Manage note compaction (digest-first navigation). See `specs/compaction.md` for details.

Subcommands:
- `qipu compact apply <digest_id> --note <id> [--note <id2> ...]`: mark one or more notes as compacted into a digest
- `qipu compact suggest`: find potential compaction candidates
- `qipu compact report`: show compaction statistics
- `qipu compact status <id>`: show compaction status of a note

### `qipu dump` / `qipu load`
Export and import raw knowledge packs. See `specs/pack.md` for details.

- `qipu dump <file>`: pack notes into a single file
- `qipu load <file>`: unpack notes from a pack file

Notes:
- Default traversal direction should be `both`.
- `--format json` output should be supported for list/tree/path (tool-friendly).
- `--format records` output should be supported for list/tree/path (low-overhead; see `specs/records-output.md`).

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
- `--max-chars <n>` (exact)

Output formats (proposed):
- default: markdown (`--format human`)
- `--format json`
- `--format records` (see `specs/records-output.md`)

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

With `--format json`, commands should emit either:
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
