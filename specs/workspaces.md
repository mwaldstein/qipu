# Workspaces and Temporary Stores

Status: Draft  
Last updated: 2026-01-17

## Motivation

Qipu acts as the long-term memory for an agentic system. However, agents often need "scratchpads" or "sandboxes" for:

- **Exploration**: Traversing a potential line of inquiry without polluting the main graph.
- **Aggregation**: Gathering a specific set of knowledge for a sub-agent to process.
- **Drafting**: Creating new structures that should only be committed if successful.

Workspaces allow multiple isolated qipu stores to exist within a single project, managed by a central registry.

## Principles

- **Primary is King**: There is always one "primary" workspace (the main `.qipu` store).
- **Isolation**: Workspaces are physically separate stores. Changes in one do not affect others until explicitly merged.
- **Addressable**: Workspaces have names.
- **Composable**: Workspaces can be created from existing graph slices and merged back.

## Workspace Model

A workspace is a fully valid qipu store (has `config.toml`, `notes/`, `mocs/`, etc.).

### Storage Layout

The primary workspace lives at `.qipu/`.
Secondary workspaces live at `.qipu/workspaces/<name>/`.

This ensures:
1. They are contained within the project's qipu footprint.
2. They are discoverable by inspecting the directory structure.
3. They can be git-ignored or committed based on policy.

### Metadata

Each secondary workspace may contain a `.qipu/workspace.toml` file:

```toml
[workspace]
name = "scratch-1"
created_at = "2026-01-17T10:00:00Z"
temporary = true
parent_id = "..." # Optional: ID of the workspace it was forked from
```

## Commands

### `qipu workspace list`
List all available workspaces.

Output should include:
- Name (`(primary)` for the root store)
- Status (Temporary/Persistent)
- Note count
- Last updated

### `qipu workspace new <name>`
Create a new workspace.

Flags:
- `--temp`: Mark as temporary (hint for UIs/cleanup tools).
- `--empty`: Start with a fresh, empty store (default).
- `--copy-primary`: Fork the *entire* primary store (heavy).
- `--from-query <q>` / `--from-tag <t>` / `--from-note <id>`: Initialize with a slice of the primary graph (like `dump` -> `load`).

Example:
`qipu workspace new exploration-1 --from-tag "machine-learning" --temp`

### `qipu workspace delete <name>`
Delete a workspace and all its contents.

- Requires `--force` if the workspace contains unmerged changes (future optimization: track "dirty" state).

### `qipu workspace merge <source> <target>`
Merge contents of one workspace into another.

Arguments:
- `<source>`: Workspace name (or `.` for current/primary).
- `<target>`: Workspace name (or `.` for current/primary).

Common Flags:
- `--dry-run`: Show what would happen (conflict report).
- `--strategy <strategy>`: Resolution strategy for ID collisions.
- `--delete-source`: Delete the source workspace after a successful merge.

### Global targeting
Any qipu command can target a workspace:

`qipu list --workspace exploration-1`
`qipu search "metrics" --workspace scratchpad`

This is syntactic sugar for finding the path to the workspace and passing it to `--store`.

## Merge and Conflict Resolution

When merging (or `loading` a pack), incoming notes may have IDs that already exist in the target store.

### Strategies

1. **`skip`** (default for safety?):
   - If ID exists in target, keep target. Ignore incoming.
   - Good for "fill in missing gaps" without touching established notes.

2. **`overwrite`**:
   - If ID exists, replace target with incoming.
   - Good for "sub-agent is the authority" workflows.

3. **`merge-links`** (Union):
   - Keep target *content* (title/body).
   - Union the `links` arrays (add new typed links from incoming).
   - Good for "enrichment" tasks where an agent discovers new connections.

4. **`rename`** (Fork):
   - If ID exists, generate a new ID for the incoming note (e.g. `qp-a1b2` -> `qp-a1b2-1`).
   - Rewrite incoming links to point to the new ID if they were internal to the merge set.
   - *Complexity warning*: This is complex to implement correctly; simpler versions might just error.

### Graph Integrity
After a merge, the target store must remain valid.
- `qipu doctor` should be run (or implicitly checked) to ensure no broken links were introduced (e.g. merging a note that links to a non-existent ID).

## Usage Patterns

### 1. The Sub-Agent Mission
1. User asks Agent to "research X".
2. Agent runs `qipu workspace new research-x --from-tag "X" --temp`.
3. Agent uses `qipu capture --workspace research-x` to add findings.
4. Agent distills findings into a summary note in `research-x`.
5. Agent runs `qipu workspace merge research-x primary --strategy merge-links`.
6. Agent runs `qipu workspace delete research-x`.

### 2. The "What If" Refactor
1. Human wants to radically reorganize MOCs.
2. `qipu workspace new refactor-1 --copy-primary`.
3. Human edits `refactor-1` aggressively.
4. Human reviews: `qipu list --workspace refactor-1`.
5. Human commits: `qipu workspace merge refactor-1 primary --strategy overwrite`.
6. Human deletes `refactor-1`.

## Open Questions

- **Git Integration**: Should temporary workspaces be added to `.gitignore` automatically? (Yes, if `--temp` is used).
- **History**: Does a merge create a git commit in the primary store? (Ideally yes, if configured).
