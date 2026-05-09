# ADR 0007: Command Failures Include Short Intended Usage Guidance

## Status

Accepted

## Context

Qipu is used interactively and through agents. When a command fails because the
user or agent chose the wrong shape, a terse parser error often leads to
follow-up `--help` calls or trial-and-error retries.

Full help output is too large for many failure paths. It also mixes common,
advanced, and compatibility details. Short failure guidance can make recovery
faster while keeping canonical docs and command help focused.

## Decision

Command failures caused by usage errors should include short, command-specific
guidance when there is an obvious intended recovery path.

Guidance should:

- show one intended example for the specific command that failed
- prefer copy-pasteable commands
- stay below full help length
- avoid documenting hidden compatibility aliases
- avoid redirecting to a different command unless no same-command recovery exists
- mention other basic flags not shown in the example in one short sentence when
  useful
- end with a short hint to run command help for full and advanced details
- keep advanced options out unless the failing command is advanced

For note creation, failures should point to:

```text
qipu create "Title" --body "Body text"
Other basic flags: --type, --tag.
Run `qipu create --help` for full and advanced details.
```

Hidden compatibility behavior remains governed by ADR 0006. If a hidden alias
fails or succeeds with a warning, the guidance must still recommend only the
intended API.

## Consequences

- Agents should need fewer help calls and retries after common mistakes.
- Error messages become part of the user-facing command contract and need tests.
- Commands with many modes should prefer narrow recovery examples over full
help output.
- Hidden compatibility aliases stay discoverable to maintainers through ADRs
and tests, not through user-facing errors.

## References

- `docs/adr/0006-hidden-compatibility-aliases-for-common-agent-errors.md`
- `specs/cli-interface.md`
- `specs/llm-user-validation.md`
