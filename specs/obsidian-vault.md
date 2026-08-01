# Obsidian Vault Interoperability

## Status

Approved feature specification. Not implemented.

## Job to be done

Allow Qipu to operate over an existing Obsidian vault so people can author and
browse knowledge in Obsidian while Qipu supplies stable identity, typed graph
semantics, deterministic queries, validation, slices, and agent-oriented
context.

Vault support is an optional content layout. It does not replace the managed
Qipu store, Qipu's domain model, or Qipu's operational database.

## Design principles

1. Vault Markdown is the source of truth.
2. Qipu's SQLite database remains private, derived, and rebuildable.
3. The managed `notes/` and `mocs/` layout remains the default.
4. Vault mode is explicit; the presence of `.obsidian/` never enables it.
5. Qipu remains local, offline, headless, and independent of the Obsidian app.
6. Existing vault files must never lose properties or content through Qipu.
7. Stable Qipu identity is opt-in for pre-existing notes.
8. Linked collection roots remain ontology-neutral; folders and literal `moc`
   types do not define the role.

This spec extends, rather than overrides:

- `specs/storage-format.md`
- `specs/operational-database.md`
- `specs/indexing-search.md`
- `specs/progressive-indexing.md`
- `specs/custom-metadata.md`
- `docs/adr/0001-markdown-source-of-truth-sqlite-derived-index.md`
- `docs/adr/0002-linked-collection-roots-are-ontology-neutral.md`
- `docs/adr/0003-store-discovery-stops-at-project-boundaries.md`

Research and alternatives are recorded in
`docs/research/obsidian-vault-interoperability.md`.

## Terminology

- **Managed layout**: the existing Qipu layout below `.qipu/` or `qipu/`, with
  content in `notes/` and `mocs/`.
- **Vault layout**: Markdown content distributed through an explicitly selected
  Obsidian vault, with Qipu operational files in `<vault>/.qipu/`.
- **Observed note**: an indexed vault Markdown file without a durable Qipu ID.
  It is queryable by path but read-only to Qipu.
- **Adopted note**: a vault Markdown file with a valid, unique Qipu ID. It
  participates in the full Qipu graph and may be mutated through Qipu.
- **Vault reference**: a normalized vault-relative path used to address an
  observed note. It is deterministic for the current path but is not durable
  across renames.

## Store profiles

Qipu supports two source layouts:

```toml
[source]
layout = "managed" # managed | vault
```

Managed layout behavior remains unchanged.

Vault layout stores operational state at `<vault>/.qipu/`:

```text
<vault>/
  .obsidian/            # owned by Obsidian; Qipu does not read its cache
  .qipu/
    config.toml
    qipu.db             # derived and gitignored
    templates/
  Projects/
  Notes/
  *.md
```

Vault configuration is explicit:

```toml
[source]
layout = "vault"
root = ".."                    # relative to .qipu/, resolved canonically
include = ["**/*.md"]
exclude = [
  ".obsidian/**",
  ".qipu/**",
  ".trash/**",
  "**/.*/**",
]
id_property = "qipu-id"
id_aliases = ["id"]
title_property = "title"
creation_folder = "Qipu"
filename_style = "title"       # title | id-title
```

Requirements:

- The configured vault root must resolve to an existing directory.
- Qipu must canonicalize it and prevent every discovered or mutated path from
  escaping it.
- Include and exclude patterns use forward-slash vault-relative paths on every
  platform. Exclusion wins over inclusion.
- Directory symlinks are not followed by default. A symlink whose resolved
  target leaves the vault is always rejected.
- Hidden directories are excluded by default. Users may explicitly include a
  hidden content directory other than `.obsidian/` or `.qipu/`.
- Qipu does not read Obsidian's IndexedDB metadata cache, application settings,
  plugin data, or another tool's search index.
- `.base`, `.canvas`, attachments, and non-Markdown files are not notes. Their
  content is not indexed. Links to them remain ordinary Markdown content.

## CLI surface

### Initialize vault support

```bash
qipu vault init [<vault-path>] [--include <glob>]... [--exclude <glob>]...
                [--id-property <name>] [--creation-folder <path>]
```

- `<vault-path>` defaults to the current directory.
- The command creates `<vault>/.qipu/`, writes `layout = "vault"`, creates the
  derived database, and indexes the vault.
- It must not add IDs, titles, types, or any other properties to content files.
- If `<vault>/.qipu/` already contains a managed store, the command fails with a
  data error and explains that conversion needs an explicit future migration.
- Initialization is idempotent when the existing vault configuration matches.
- `qipu init --visible` inside a vault remains supported as a managed-subtree
  compatibility option; it is not equivalent to `qipu vault init`.

After initialization, ordinary discovery finds `<vault>/.qipu/` according to
ADR 0003. `--store <vault>/.qipu` remains the explicit override.

### Inspect compatibility

```bash
qipu vault status [--format human|json|records]
```

Status reports at least:

- vault root and source configuration;
- included, excluded, observed, and adopted note counts;
- invalid or duplicate IDs;
- invalid YAML/frontmatter files;
- resolved, unresolved, and ambiguous inline link counts;
- files that would require adoption for mutation;
- current index consistency.

Status is read-only and deterministic.

### Adopt existing notes

```bash
qipu vault adopt <path>... [--apply]
qipu vault adopt --all [--apply]
```

- Exactly one of one-or-more paths or `--all` is required.
- Paths are vault-relative files, directories, or configured glob patterns.
- Without `--apply`, the command prints a deterministic plan and changes
  nothing. Human output must say explicitly that it is a dry run.
- With `--apply`, Qipu adds the configured ID property to notes that do not have
  one, then reindexes them.
- Existing valid and unique values under the configured ID property are preserved.
- When `id_property` differs from `id`, a valid existing `id` may be adopted
  without copying only if configuration explicitly enables `id` as an alias.
- Missing title does not require a write. Qipu derives it as specified below.
- The entire apply operation validates all targets before the first write.
- Duplicate IDs, path escapes, parse failures, or concurrent modifications make
  the apply fail before mutation.
- Multi-file writes are best-effort transactional: Qipu stages all replacements
  first and restores originals if a later replacement fails.

There is no automatic or implicit adoption. Removing an ID is not part of this
feature because it can break durable references.

### Move an adopted note

```bash
qipu vault move <id-or-path> <new-vault-relative-path> [--apply]
```

- Without `--apply`, the command reports the file move and every inbound link
  rewrite it would perform.
- Only adopted notes may be moved through Qipu.
- With `--apply`, Qipu updates all resolvable inbound wikilinks and Markdown
  links while preserving label and heading/block suffixes.
- Ambiguous inbound references block the move.
- The target must remain within the vault and must not already exist.
- The move and link rewrites use the same staged, conflict-checked mutation
  protocol as adoption.

In vault mode, `qipu update --title` updates the title property but does not
rename or move the file. File movement is explicit through `qipu vault move`.

## Identity model

### Adopted notes

The configured `id_property` defaults to `qipu-id` to avoid claiming a generic
property in an existing vault:

```yaml
---
qipu-id: qp-a1b2
title: Raft consensus
tags:
  - distributed-systems
---
```

IDs use the configured Qipu ID scheme and the same validation and cross-branch
collision rules as managed notes. The property name is configuration; the
domain concept remains Qipu note ID.

### Observed notes

Observed notes are indexed for reading and discovery but do not pretend to have
stable IDs.

- Human output identifies them with their vault-relative path and the marker
  `observed`.
- JSON output uses `"id": null`, includes `"path"`, and includes
  `"identity": "observed"`.
- Adopted note JSON uses its ID and `"identity": "adopted"`.
- Records output uses `id=-`, `path=<encoded-path>`, and `identity=observed`.
- CLI arguments accept an observed note's exact vault-relative path wherever a
  read-only command accepts `<id-or-path>`.
- A path is not accepted where a command contract specifically requires a
  durable note ID.
- Rename or deletion may change or remove an observed note's identity. Output
  and documentation must not describe the path as stable.

Observed notes may participate in an in-memory/SQLite path-keyed inline-link
graph for read-only search, backlinks, traversal, and context. Typed links,
packs, workspace merges, compaction registration, and Qipu mutations require
adopted IDs.

## Title, type, tags, and properties

### Title

Title resolution is deterministic, in this order:

1. Non-empty configured title property.
2. First level-one Markdown heading.
3. Filename stem.

Qipu does not add a derived title to the file during indexing or adoption.

### Note type

- An ontology-valid `type` property is used when present.
- Missing type uses the configured default in derived query output.
- Invalid types are reported by `doctor` and `vault status`.
- Folder placement never implies type.
- A literal `moc` type is not required for linked collection root selection.

### Tags

- Qipu indexes both the YAML `tags` property and Obsidian inline `#tags`.
- Duplicate normalized tags are collapsed in derived output.
- Mutating tags through Qipu changes only the YAML `tags` property. Qipu never
  rewrites inline hashtags as metadata.

### Arbitrary properties

Qipu must parse and index arbitrary top-level YAML properties without treating
them as Qipu custom metadata. The existing nested `custom` namespace retains
its existing contract.

Vault properties are queryable with a separate filter:

```bash
qipu list --property-filter 'status=review'
qipu context --property-filter 'published>=2026-01-01'
```

`--property-filter` supports the equality, existence, numeric, and ISO date
operators defined for `--custom-filter`. Multiple filters use AND semantics.
Core and custom Qipu fields may also appear in the raw property map, but their
domain-specific CLI filters remain canonical.

Dataview inline fields such as `Key:: Value` are preserved as body content but
are not promoted to Qipu properties.

## Lossless frontmatter contract

Vault write support is prohibited until all of these hold:

- Unknown top-level YAML keys and values survive every Qipu mutation.
- Qipu changes only the requested Qipu-owned or explicitly targeted property.
- Markdown body bytes remain unchanged for metadata-only mutations.
- YAML comments, key order, scalar style, aliases, and formatting outside the
  changed property are preserved where the YAML representation permits it.
- A file with valid Obsidian YAML but an unsupported construct is read-only and
  receives a diagnostic rather than being normalized or partially rewritten.
- Files without frontmatter remain valid observed notes. Adoption inserts a new
  frontmatter block without changing their body.
- A UTF-8 BOM and original LF/CRLF newline style are preserved.

Implementation must use a property-aware patch representation or concrete
syntax tree. Deserializing into `NoteFrontmatter` and serializing the whole map
does not satisfy this contract.

## Obsidian link compatibility

Qipu parses these inline forms:

- `[[Note]]`, `[[Folder/Note]]`, and their `.md` variants;
- `[[Note|label]]`;
- `[[Note#Heading]]` and `[[Note#^block-id]]`;
- `![[Note]]` embeds;
- Markdown links and embeds with relative or vault-root paths;
- URL-encoded Markdown destinations.

For graph purposes, headings and blocks are subpaths of the target note, not
separate graph nodes. Embeds create inline `related` edges like ordinary links.
The original source representation is preserved.

Resolution order is:

1. Exact vault-relative path, with optional `.md`.
2. Exact path relative to the source note's directory.
3. Unique vault path suffix.
4. Unique filename stem.

Zero matches create an unresolved reference. Multiple matches at the first
matching resolution tier create an ambiguous reference. Qipu must not select a
target by iteration order. Aliases are indexed for search and may be offered in
diagnostics, but an alias alone does not override deterministic path resolution.

Typed frontmatter links continue to target stable Qipu IDs. If an inline and a
typed link connect the same adopted notes, the typed relationship takes
precedence under the existing edge-deduplication rules.

`qipu doctor` and `qipu vault status` distinguish unresolved from ambiguous
links and show the source path plus original target.

## Index and consistency

The vault profile uses `<vault>/.qipu/qipu.db`; no second operational index is
introduced.

The notes table must distinguish adopted ID keys from observed path keys while
preserving the public identity rules above. The index also stores enough raw
property data for `--property-filter` and enough normalized path data for link
resolution.

Consistency checks cover the complete configured file set:

- added, modified, renamed, and deleted Markdown files;
- include/exclude configuration changes;
- adopted ID changes and duplicates;
- title, tag, property, body, and link changes;
- files moving into or out of scope.

Count sampling alone is insufficient. Incremental reconciliation uses a stored
manifest of normalized path, size, mtime, and content hash. Hashing may be
deferred when size and high-resolution mtime prove unchanged, but correctness
must not depend on timestamps alone.

`qipu index --rebuild`, progressive indexing levels, interruption, batching,
and deterministic search behavior apply to vault mode. Rebuild must reproduce
the same index from vault files and configuration without Obsidian running.

## Mutation safety

Every mutation of a vault file must:

1. Resolve and validate every target beneath the canonical vault root.
2. Read and retain the original bytes and content hash.
3. Build the minimal property/body patch in memory.
4. Re-read before replacement and compare the content hash.
5. Fail with a conflict if the file changed; never overwrite silently.
6. Write a sibling temporary file, flush it, and atomically replace the target
   where supported.
7. Update SQLite only after filesystem success.
8. Preserve or restore originals if a staged multi-file operation fails.

Conflict is a data/store error (exit code `3`) and includes the affected path.
Qipu does not require Obsidian's Vault API, REST plugin, or official CLI to make
writes. Obsidian detects the resulting external file changes.

Only adopted notes may be changed by `update`, `custom`, `link add/remove`,
compaction apply/register, merge, or removal. Attempting to mutate an observed
note returns a data error with the exact adoption dry-run command.

### Creation

In vault mode, `create` and `capture`:

- create adopted notes in `creation_folder` unless an explicit supported path
  selector is provided;
- add the configured ID property plus Qipu core properties;
- use the configured filename style;
- for `title`, use the slugged title and append `-<short-id>` on collision;
- for `id-title`, use the managed-store filename convention;
- never create a separate `mocs/` directory based on type.

### Deletion

When an existing Qipu operation removes a vault note, including merge source
removal, it moves the adopted note to `<vault>/.trash/` while preserving its
relative path. Vault mode has no permanent note deletion until that behavior is
separately specified. Qipu does not infer Obsidian's private trash setting.

## Query and output behavior

These read commands operate on both observed and adopted notes in vault mode:

- `list`, `search`, `show`;
- `context` and read-only slice selection;
- inline-link backlinks and traversal;
- `doctor`, `index`, and store statistics.

Commands whose portable output requires stable identity must either exclude
observed notes with an explicit count/warning or fail when the user directly
selects one. This includes pack/dump payloads intended for later merge,
workspaces, typed-link mutations, and compaction registration.

All output ordering remains deterministic. Path ordering compares normalized
forward-slash vault-relative paths bytewise after ID ordering rules have been
applied.

## Compatibility and non-goals

Required compatibility:

- Obsidian may be running or absent.
- External edits are reconciled on the next Qipu invocation.
- Existing Obsidian properties, links, embeds, headings, blocks, aliases,
  Dataview fields, and plugin-owned Markdown remain intact.
- Vaults stored in Git, Dropbox, iCloud, OneDrive, or Obsidian Sync are ordinary
  local files from Qipu's perspective. Sync conflict handling remains owned by
  the sync tool.

Non-goals:

- Reading Obsidian's IndexedDB metadata cache.
- Reimplementing Obsidian Bases, Dataview, Canvas, or plugin execution.
- Indexing attachment contents, PDFs, or OCR.
- Requiring Obsidian Sync, Obsidian CLI, Local REST API, or MCP.
- Continuous two-way mirroring between a vault and a managed Qipu store.
- Giving observed path references the guarantees of stable Qipu IDs.
- Treating folder hierarchy as note type or linked collection membership.

## Delivery order

Implementation may land in vertical slices, but every slice must preserve the
complete contract above:

1. Visible managed-subtree documentation and an Obsidian compatibility fixture.
2. Vault profile, recursive discovery, observed-note index, properties, and
   read-only queries.
3. Obsidian link resolution, graph queries, and diagnostics.
4. Adoption dry run and stable ID application.
5. Lossless single-note metadata mutations with conflict protection.
6. Staged multi-file link mutation and `vault move`.
7. Vault-aware create, capture, recoverable removal, pack/workspace eligibility, and full
   command parity for adopted notes.
8. Optional generated Bases or thin Obsidian integration, specified separately.

Read-only delivery is a safety sequence, not a reduced final product.

## Acceptance criteria

The feature is complete when automated tests demonstrate:

1. Managed-layout behavior and output remain backward compatible.
2. Vault initialization is explicit, idempotent, and non-mutating to notes.
3. Recursive include/exclude discovery is deterministic and path-safe on Unix and Windows path forms.
4. Observed notes without frontmatter are searchable, readable by path, and represented without a fake stable ID.
5. Adoption dry runs are deterministic; apply adds unique IDs without changing body content or unrelated YAML.
6. Unknown properties, YAML comments/order/style, BOM, and newline convention survive representative metadata mutations.
7. Concurrent external edits produce conflicts rather than lost updates.
8. Wikilinks, aliases-as-labels, relative and vault-root Markdown links, headings, blocks, embeds, URL encoding, duplicates, unresolved targets, and ambiguous targets have fixture coverage.
9. Adopted links resolve to stable IDs; observed links work for read-only graph operations and remain path-identified.
10. Arbitrary YAML properties and inline tags are indexed and filterable without changing the `custom` namespace contract.
11. Index rebuild and incremental reconciliation handle create, edit, rename,
    delete, scope changes, and ID changes without Obsidian running.
12. Mutation path guards reject traversal and symlink escapes.
13. Multi-file adoption and move operations roll back or preserve recoverable
    originals on injected failures.
14. Query output is deterministic in human, JSON, and records formats.
15. Adopted-note create, update, link, context, pack/workspace, move, and recoverable removal
    behavior works across supported platforms.
16. No test or production path reads Obsidian's internal cache or requires an
    Obsidian process, subscription, or plugin.
