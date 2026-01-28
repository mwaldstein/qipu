# Custom Ontology

Status: Reference Documentation  
Last updated: 2026-01-28

## Overview

Qipu provides a built-in ontology for organizing notes and links, with standard note types (`fleeting`, `literature`, `permanent`, `moc`) and standard link types (`related`, `supports`, `contradicts`, `part-of`, etc.).

Custom ontology lets you extend or replace these standard types with domain-specific types that better match your workflow, terminology, and research needs.

## When to Use Custom Ontology

Custom ontology is useful when:
- Standard note/link types don't match your domain terminology
- You need domain-specific link semantics (e.g., `depends-on`, `contradicts`, `applies-to`)
- You want LLM agents to use your domain's vocabulary when working with qipu
- You're building domain-specific tools on top of qipu

## Configuration

Custom ontology is configured in `.qipu/config.toml`:

```toml
[ontology]
mode = "extended"  # default, extended, or replacement

[ontology.note_types.<type-name>]
description = "Human-readable description"
usage = "Usage guidance for LLMs"

[ontology.link_types.<link-name>]
description = "Human-readable description"
inverse = "inverse-link-name"
usage = "Usage guidance for LLMs"
cost = 1.0
```

### Resolution Modes

#### Default Mode
Uses only standard built-in types. No custom types are available.

```toml
[ontology]
mode = "default"  # or omit the mode field
```

This is the default behavior when no custom ontology is configured.

#### Extended Mode
Extends standard ontology with custom types. Both standard and custom types are available.

```toml
[ontology]
mode = "extended"

[ontology.note_types.task]
description = "A task or action item"
usage = "Use for tracking tasks and action items"

[ontology.link_types.depends-on]
description = "Task dependency relationship"
inverse = "required-by"
usage = "Use when task B cannot start until task A completes"
cost = 0.5
```

**Key behavior in extended mode:**
- Standard types (`fleeting`, `literature`, `permanent`, `moc`, `related`, `supports`, etc.) remain available
- Custom types are added alongside standard types
- Custom inverses can override standard inverses
- If you define a custom `supports` type with a different inverse, it overrides the standard

#### Replacement Mode
Replaces standard ontology with custom types only. Standard types are not available.

```toml
[ontology]
mode = "replacement"

[ontology.note_types.idea]
description = "An idea or concept"
usage = "Use for capturing ideas and concepts"

[ontology.link_types.improves]
description = "Improvement relationship"
inverse = "improved-by"
usage = "Use when one idea improves or refines another"
```

**Key behavior in replacement mode:**
- Only custom types you define are available
- Standard types are **not** available
- You must define all types you need
- Useful for complete domain-specific implementations

### Configuration Fields

#### Note Type Configuration
```toml
[ontology.note_types.<name>]
description = "Human-readable description (optional)"
usage = "Usage guidance for LLMs (optional)"
```

#### Link Type Configuration
```toml
[ontology.link_types.<name>]
description = "Human-readable description (optional)"
inverse = "inverse-link-name (optional)"
usage = "Usage guidance for LLMs (optional)"
cost = 1.0  # Hop cost for graph traversal (optional, default 1.0)
```

**Link type fields:**
- `inverse`: The inverse relationship. If not specified, qipu generates `inverse-<link-name>`.
- `cost`: Hop cost for graph traversal (default 1.0). Lower costs create stronger cohesion in the graph.
  - Structural types (e.g., `part-of`, `same-as`) typically use 0.5
  - Standard relationships (e.g., `supports`, `contradicts`) typically use 1.0

## Domain Examples

### Legal Research

For legal research, you might define case-related types and citation relationships:

```toml
[ontology]
mode = "extended"

[ontology.note_types.case]
description = "Legal case or court decision"
usage = "Use for court decisions, case law, and legal precedents"

[ontology.note_types.statute]
description = "Statute or legislation"
usage = "Use for laws, statutes, and legislative documents"

[ontology.note_types.argument]
description = "Legal argument or position"
usage = "Use for legal arguments and positions"

[ontology.link_types.cites]
description = "Citation relationship"
inverse = "cited-by"
usage = "Use when a note cites another note (case, statute, etc.)"
cost = 0.5

[ontology.link_types.overruled-by]
description = "Overruling relationship"
inverse = "overrules"
usage = "Use when a case is overruled by another case"

[ontology.link_tests.distinguishes]
description = "Distinguishing relationship"
inverse = "distinguished-by"
usage = "Use when a case distinguishes itself from another case"
```

**Usage with LLMs:**
```bash
qipu prime | grep -A 20 "## Ontology"
# Shows:
# case - Legal case or court decision
#   Usage: Use for court decisions, case law, and legal precedents
# cites -> cited-by (Citation relationship)
#   Usage: Use when a note cites another note (case, statute, etc.)
```

### Medical Research

For medical research, you might define evidence-based types and clinical relationships:

```toml
[ontology]
mode = "extended"

[ontology.note_types.study]
description = "Research study or clinical trial"
usage = "Use for research studies, clinical trials, and medical research"

[ontology.note_types.guideline]
description = "Clinical guideline or protocol"
usage = "Use for clinical guidelines, protocols, and standards of care"

[ontology.note_types.patient]
description = "Patient case or observation"
usage = "Use for patient cases, observations, and clinical findings"

[ontology.link_types.supports-evidence]
description = "Evidence support relationship"
inverse = "supported-by-evidence"
usage = "Use when a note provides evidence supporting another note"

[ontology.link_types.contradicts-evidence]
description = "Evidence contradiction relationship"
inverse = "contradicted-by-evidence"
usage = "Use when evidence contradicts a claim or finding"

[ontology.link_types.applies-to]
description = "Application relationship"
inverse = "applied-in"
usage = "Use when a guideline or study applies to a patient or condition"
```

**Example workflow:**
```bash
# Create a study note
qipu create "Aspirin reduces cardiovascular risk" --type study --tag cardiology

# Create a guideline note
qipu create "Cardiovascular disease prevention" --type guideline --tag prevention

# Link the study to the guideline
qipu link add <study-id> <guideline-id> --type supports-evidence
```

### Software Architecture

For software architecture, you might define component relationships and design patterns:

```toml
[ontology]
mode = "extended"

[ontology.note_types.component]
description = "Software component or module"
usage = "Use for software components, modules, and architectural elements"

[ontology.note_types.pattern]
description = "Design pattern or architectural pattern"
usage = "Use for design patterns, architectural patterns, and best practices"

[ontology.note_types.decision]
description = "Architectural decision record"
usage = "Use for architectural decisions, trade-offs, and rationales"

[ontology.link_types.depends-on]
description = "Dependency relationship"
inverse = "required-by"
usage = "Use when one component depends on another"
cost = 0.5

[ontology.link_types.implements]
description = "Implementation relationship"
inverse = "implemented-by"
usage = "Use when a component implements a pattern or interface"

[ontology.link_types.constrained-by]
description = "Constraint relationship"
inverse = "constrains"
usage = "Use when a design decision constrains a component or pattern"
```

**Example workflow:**
```bash
# Create component notes
qipu create "User authentication service" --type component --tag auth
qipu create "OAuth 2.0 provider" --type component --tag auth

# Create a pattern note
qipu create "Authentication broker pattern" --type pattern --tag architecture

# Link the pattern to the components
qipu link add <pattern-id> <auth-service-id> --type implements
qipu link add <pattern-id> <oauth-provider-id> --type implements

# Link components with dependencies
qipu link add <oauth-provider-id> <auth-service-id> --type depends-on
```

## Migration Workflows

### Starting with Standard Ontology

1. **Begin with default mode** to explore standard types:
   ```toml
   [ontology]
   mode = "default"
   ```

2. **Switch to extended mode** when you identify gaps:
   ```toml
   [ontology]
   mode = "extended"
   
   [ontology.note_types.custom-type]
   description = "Custom note type"
   ```

3. **Gradually replace** standard types if needed:
   ```toml
   [ontology]
   mode = "replacement"
   
   [ontology.note_types.idea]
   description = "An idea or concept"
   ```

### Migrating from Backward-Compatible `graph.types`

Qipu previously supported custom link types via `graph.types`. This is still supported for backward compatibility, but new configurations should use `ontology.link_types`.

**Old configuration (backward compatible but deprecated):**
```toml
[graph.types.custom-link]
description = "Custom link type"
inverse = "inverse-custom"
cost = 0.5
```

**New configuration (recommended):**
```toml
[ontology]
mode = "extended"

[ontology.link_types.custom-link]
description = "Custom link type"
inverse = "inverse-custom"
cost = 0.5
```

**Migration path:**
1. Both configurations work simultaneously
2. `graph.types` is merged with `ontology.link_types`
3. Migrate `graph.types` definitions to `ontology.link_types` when convenient
4. Remove `graph.types` once fully migrated

### Changing Modes

Switching modes affects type availability:

**Extended → Default:**
- Custom types become invalid
- Existing notes with custom types remain valid but won't be accepted in new captures
- Consider renaming custom types to standard types or switch back to extended mode

**Extended → Replacement:**
- Standard types become invalid
- Only custom types are available
- Ensure you've defined all necessary custom types before switching

**Default → Extended:**
- No issues; custom types become available
- Standard types remain available

**Default → Replacement:**
- Standard types become invalid
- Only custom types are available
- Ensure you've defined all necessary custom types before switching

**Recommendation:** Test mode changes in a separate store or branch before applying to production.

## LLM Guidance Best Practices

### Providing Clear Usage Instructions

The `usage` field is consumed by LLM agents when primed with `qipu prime`. Write clear, concise guidance:

**Good:**
```toml
[ontology.link_types.supports-evidence]
usage = "Use when a note provides evidence supporting another note (e.g., a study supports a guideline)"
```

**Less clear:**
```toml
[ontology.link_types.supports-evidence]
usage = "For evidence"
```

### Describing Domain Semantics

Help LLMs understand the semantic meaning of your types:

```toml
[ontology.note_types.case]
description = "Legal case or court decision"
usage = "Use for court decisions, case law, and legal precedents. Include jurisdiction, date, and key holding."

[ontology.link_types.overruled-by]
description = "Overruling relationship"
usage = "Use when a case is overruled by another case. The overruling case supersedes the overruled case."
```

### Balancing Specificity and Generality

Avoid overly specific types that won't be reused:

**Too specific:**
```toml
[ontology.link_types.cites-from-federal-court]
description = "Citation from federal court"
```

**Better (more general):**
```toml
[ontology.link_types.cites]
description = "Citation relationship"
```

Use tags or metadata to capture jurisdiction-level specificity instead.

### Documenting Inverse Relationships

Always define inverse relationships for bidirectional links:

```toml
[ontology.link_types.depends-on]
description = "Dependency relationship"
inverse = "required-by"
usage = "Use when task B cannot start until task A completes"

[ontology.link_types.required-by]
description = "Required by relationship"
inverse = "depends-on"
usage = "Use when task A is required for task B to start"
```

You can define both inverses, or just one (qipu will generate the other).

## Commands

### Show Active Ontology

```bash
# Show current ontology (human-readable)
qipu ontology show

# Show as JSON (for tools)
qipu ontology show --format json

# Show as records (for parsing)
qipu ontology show --format records
```

Example output (human format):
```
Ontology mode: extended

Note types:
  fleeting
  literature
  permanent
  moc
  task - A task or action item
    Usage: Use for tracking tasks and action items

Link types:
  related
  supports
  contradicts
  part-of
  depends-on -> required-by (Dependency relationship)
    Usage: Use when task B cannot start until task A completes
```

## Validation

Qipu validates note and link types against the active ontology:

```bash
# Valid link type
qipu link add <id1> <id2> --type supports

# Invalid link type (fails with error)
qipu link add <id1> <id2> --type invalid-link
# Error: Invalid link type: 'invalid-link'
```

Validation is performed by:
- `link add` commands
- `link edit` commands
- `capture` commands (for note types)
- `create` commands (for note types)

## Backward Compatibility

### `graph.types` (Deprecated but Supported)

For backward compatibility, custom link types defined in `graph.types` are merged with `ontology.link_types`:

```toml
# Both work together
[graph.types.custom-link]
inverse = "inverse-custom"

[ontology]
mode = "extended"

[ontology.link_types.another-link]
inverse = "inverse-another"
```

**Migration plan:**
1. Keep `graph.types` working during transition
2. Migrate definitions to `ontology.link_types`
3. Remove `graph.types` once migration is complete

### Future Deprecation

The `graph.types` field will be deprecated in a future version. Migrate to `ontology.link_types` at your convenience.

## Best Practices

1. **Start with extended mode** unless you have a specific reason to use replacement mode
2. **Define clear descriptions** for both human readers and LLMs
3. **Set appropriate costs** for link types based on your graph traversal needs
4. **Test mode changes** in a separate store before applying to production
5. **Use tags** for categorization instead of creating many similar note types
6. **Provide usage guidance** in the `usage` field for LLM integration
7. **Document your ontology** for team members and future maintainers

## Common Pitfalls

### Forgetting to Switch Mode

Configuring custom types without setting the mode has no effect:

```toml
# This won't work: mode is still "default"
[ontology.note_types.custom]
description = "Custom type"

# Correct: set the mode
[ontology]
mode = "extended"

[ontology.note_types.custom]
description = "Custom type"
```

### Too Many Similar Types

Creating many similar types makes the ontology hard to navigate:

```toml
# Too specific
[ontology.note_types.case-federal]
[ontology.note_types.case-state]
[ontology.note_types.case-appellate]

# Better: use one type with tags
[ontology.note_types.case]
description = "Legal case or court decision"
```

Use tags or metadata to capture specificity.

### Missing Inverse Definitions

Not defining inverses makes graph traversal less intuitive:

```toml
# Less clear
[ontology.link_types.depends-on]
# (no inverse defined, qipu generates "inverse-depends-on")

# Better: define a meaningful inverse
[ontology.link_types.depends-on]
inverse = "required-by"
```

## See Also

- `qipu ontology show` - Display active ontology
- `qipu prime` - Show ontology in session primer
- `specs/semantic-graph.md` - Graph traversal and link type costs
- `docs/llm-context.md` - LLM integration with custom ontology
