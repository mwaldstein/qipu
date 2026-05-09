# ADR 0002: Linked Collection Roots Are Ontology Neutral

## Status

Accepted

## Context

The standard ontology includes a `moc` note type. Historically, several CLI
flags used `--moc` to mean "select a map of content." Qipu now supports custom
and replacement ontologies where users may name that role `outline`, `index`,
`collection`, `project-map`, or something domain-specific.

If implementation code treats literal `type: moc` as the semantic requirement
for collection selection, custom ontology support becomes inconsistent and
future commands drift toward standard-ontology assumptions.

## Decision

A linked collection root is a role, not a privileged note type.

Linked collection root selection uses the supplied note as a root and follows
its outbound links to define the collection. The root does not need literal
`type: moc`.

The `moc` note type remains one standard ontology type. The `--moc` flag remains
as a legacy CLI alias, but new CLI and documentation should prefer
`--collection-root`.

## Consequences

- Slice selection, context, export, dump, and pack behavior should share one
  ontology-neutral linked collection root interpretation.
- Code should avoid asking `is_moc()` when the domain question is "is this the
  selected collection root?"
- Replacement ontologies can use domain-specific root note types without
  defining `moc`.
- Some storage or UI conventions may still mention MOCs for the standard
  ontology, but those conventions must not define the selection semantics.

## References

- `CONTEXT.md`
- `specs/knowledge-model.md`
- `specs/llm-context.md`
- `specs/export.md`
- `specs/pack.md`
- `docs/custom-ontology.md`
