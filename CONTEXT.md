# Qipu Domain Context

## Glossary

### Linked Collection Root
A note used as the root of an ordered linked collection. The CLI still exposes
this through the legacy `--moc` flag and the neutral `--collection-root` alias.
The note does not need literal `type: moc`; custom ontologies may use
domain-specific outline, index, collection, project-map, or root note types.
`moc` is one standard ontology term for this role, not a privileged system
concept.

Selecting a linked collection root means selecting the root note plus the notes
it links to. Stored links are directional: the root links outward to its
children. Inbound traversal may present virtual inverse relationships, but the
collection definition is the root's outbound linked set.

### Slice
A subset of notes selected for export, dump, context, or workspace-style use.
A slice can be selected directly, by metadata, by a linked collection root, by
query, or by graph traversal.

### Link Direction
Typed links are stored as `from -> to`: the source note asserts the relationship
to the target note. A link type name should read naturally in that direction,
such as `child part-of parent`, `evidence supports claim`, or `task depends-on
dependency`. Inverses are virtual presentation/traversal names unless explicitly
stored as their own forward link.
