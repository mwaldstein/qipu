# ADR 0003: Store Discovery Stops At Project Boundaries

## Status

Accepted

## Context

Qipu commands need a default store when the user does not pass `--store`.
Walking upward from the current directory is convenient, but unconstrained
upward discovery can accidentally select unrelated or malicious stores in
parent directories.

This risk is especially high for agents and scripts running from nested project
folders or temporary directories.

## Decision

When no explicit store is provided, qipu searches upward for `.qipu/` or
`qipu/`, but stops at a project boundary or filesystem root.

Project boundaries include common repository and language markers such as
`.git/`, `.hg/`, `.svn/`, `Cargo.toml`, `package.json`, `go.mod`, and
`pyproject.toml`.

If a user wants a store above a project boundary, they must pass it explicitly
with `--store`.

## Consequences

- Store discovery is predictable and safer for agents, tests, and scripts.
- Global or parent-directory personal stores are not implicitly used from inside
  a project.
- CLI behavior should keep "explicit over implicit" as the rule when discovery
  is ambiguous.
- Tests should cover project-boundary discovery rather than assuming a simple
  parent walk.

## References

- `specs/storage-format.md`
- `docs/building-on-qipu.md`
