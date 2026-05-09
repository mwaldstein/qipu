# ADR 0005: Qipu Is LLM Compatible, Not LLM Powered

## Status

Accepted

## Context

Qipu is designed to work well with agentic and LLM-based tools. It provides
`prime`, `context`, JSON output, records output, deterministic selection, and
bounded context bundles.

That can be confused with making qipu itself call LLM APIs for summarization,
retrieval, compaction, or evaluation. Doing so would add provider dependencies,
network requirements, credentials, nondeterminism, and privacy concerns to a CLI
whose core value is local, git-backed knowledge.

## Decision

Qipu is LLM compatible, not LLM powered.

Qipu prepares clean, deterministic, bounded context for humans, scripts, and
LLM tools. It does not require an LLM API to perform core operations.

LLM-generated content may be authored outside qipu and then stored in qipu with
provenance metadata. Compaction content may be produced externally and registered
in qipu, but qipu itself does not depend on an LLM provider.

## Consequences

- Core commands must remain local and deterministic.
- Features that need summarization or judgment should expose import/apply
  workflows rather than embed provider calls.
- Integration affordances belong in output formats, context selection, and
  provenance, not provider-specific runtime dependencies.
- Product positioning should emphasize durable knowledge that works for both
  humans and agents.

## References

- `specs/llm-context.md`
- `specs/records-output.md`
- `specs/compaction.md`
- `specs/provenance.md`
- `docs/building-on-qipu.md`
- `docs/research/pre-release-review.md`
