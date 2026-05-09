# ADR 0004: Pack And Workspace Merges Share Conflict Policy

## Status

Accepted

## Context

Qipu has two ways to bring external or isolated knowledge back into a store:

- Loading a pack file.
- Merging a workspace.

Both operations face the same domain problem: incoming notes may collide with
existing note IDs, and incoming links or attachments may need to be preserved.
Keeping separate conflict behavior for packs and workspaces would make tests,
documentation, and user expectations diverge.

## Decision

Pack loading and workspace merging use the same conflict policy vocabulary.

The canonical strategies are:

- `skip`: keep the target note and ignore the incoming colliding note.
- `overwrite`: replace the target note with the incoming note.
- `merge-links`: keep target content and union typed links from the incoming
  note.

Additional strategies, such as rename/fork behavior, may be added later only if
their link-rewrite and attachment semantics are specified.

## Consequences

- A shared merge policy module is preferred over command-specific conflict
  implementations.
- Tests for conflict behavior should exercise the shared policy directly, with
  pack/workspace command tests covering adapter behavior.
- Graph integrity after merge/load is required; broken links should be rejected,
  repaired, or surfaced through doctor checks.
- Documentation for pack and workspace conflict handling must stay aligned.

## References

- `specs/pack.md`
- `specs/workspaces.md`
