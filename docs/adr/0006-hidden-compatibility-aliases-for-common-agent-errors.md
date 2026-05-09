# ADR 0006: Hidden Compatibility Aliases For Common Agent Errors

## Status

Accepted

## Context

Qipu is used by humans, scripts, and LLM agents. Real agent transcripts show
that agents sometimes attempt plausible command shapes that are not the
canonical qipu API.

For example, agents tried to create notes with inline body text using shapes
such as:

- `qipu create "Title" --body "Body text"`
- `qipu create "Title" -c "Body text"`
- `qipu create --title "Title" --body "Body text"`

The first two are useful enough to become first-class documented API. The
`--title` form mirrors `capture --title`, but `create` already has a positional
title and documenting both forms would make the command harder to teach.

## Decision

Qipu may support hidden compatibility aliases when repeated real-world agent or
script error patterns show that accepting the shape improves successful tool
use without weakening the domain model.

Hidden compatibility aliases must not appear in command help, quickstarts, user
guides, or generated user-facing setup text. User-facing guidance should point
only to the intended API.

Compatibility behavior must be documented in ADRs or maintainer-facing notes so
future maintainers understand why an undocumented shape works.

Each hidden compatibility alias must be documented in three maintainer-facing
places:

- This ADR or a follow-on ADR, including the rationale and evidence pattern.
- The CLI argument or dispatch code, with a short comment pointing to the ADR.
- Focused regression tests whose names make the hidden compatibility behavior
  explicit.

Hidden compatibility aliases must not be documented in user-facing surfaces:

- command help
- README or quickstart material
- user guides
- generated setup text such as `AGENTS.md` or editor rules
- public integration guidance

For note creation:

- Intended inline-body API: `qipu create "Title" --body "Body text"`
- Intended stdin API: `echo "Body text" | qipu capture --title "Title"`
- Hidden compatibility API: `qipu create --title "Title" --body "Body text"`

When a hidden compatibility path succeeds and emits guidance, that guidance
should recommend only the intended API. It should not teach or advertise the
hidden alias.

## Consequences

- Agents can recover from common, harmless mistakes with fewer retries.
- Human-facing docs remain smaller and clearer.
- Hidden aliases require regression coverage because users may discover and
depend on them.
- Compatibility aliases should be removed only through an explicit deprecation
decision, not incidental cleanup.

## References

- `docs/adr/0005-qipu-is-llm-compatible-not-llm-powered.md`
- `specs/llm-user-validation.md`
- `llm-test-fixtures/create_smoke.yaml`
- `llm-test-fixtures/test_setup_scenario.yaml`
