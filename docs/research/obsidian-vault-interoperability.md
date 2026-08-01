# Obsidian Vault Interoperability

**Research date:** 2026-08-01

**Issue:** `qipu-srpv`

## Executive conclusion

Qipu should support an Obsidian vault as an optional content layout and
authoring surface while retaining Qipu's ontology, graph semantics, validation,
query behavior, and SQLite operational index.

This is a strong architectural fit because an Obsidian vault and a Qipu store
share the same durable substrate: local Markdown files, YAML frontmatter, links,
and rebuildable derived indexes. The right integration is not to read
Obsidian's internal database or require Obsidian to be running. It is to add a
vault-aware file adapter to Qipu.

The recommended end state is:

- Markdown in the vault remains the source of truth.
- Qipu keeps its own rebuildable SQLite index and remains usable headlessly.
- A vault layout adapter recursively discovers selected Markdown files rather
  than requiring `notes/` and `mocs/`.
- Stable Qipu IDs and typed links remain Qipu semantics.
- Obsidian-style links and arbitrary properties are resolved and round-tripped
  without loss.
- An optional Obsidian plugin or CLI bridge may improve editor UX later, but is
  not part of the core persistence path.

An immediately usable, zero-code subset already exists: initialize a **visible**
`qipu/` store inside a vault. Obsidian can edit `qipu/notes/` and `qipu/mocs/`,
and Qipu can reconcile those external edits. This makes Obsidian a GUI for a
Qipu-managed subset, but does not make Qipu operate over an existing whole
vault.

## What an Obsidian vault actually is

Obsidian's durable store is a folder containing local files and subfolders.
Notes are Markdown plain text, external edits are detected, and `.obsidian/`
holds vault-specific application configuration. Obsidian also maintains an
IndexedDB metadata cache for graph, outline, and similar features; the cache can
drift and can be rebuilt. Therefore there is no supported, authoritative
"Obsidian database" for Qipu to reuse. The vault files are the integration
contract. See [How Obsidian stores data](https://obsidian.md/help/Files%2Band%2Bfolders/How%2BObsidian%2Bstores%2Bdata).

This is notably close to Qipu's accepted architecture in
[`ADR 0001`](../adr/0001-markdown-source-of-truth-sqlite-derived-index.md):
Markdown is authoritative and SQLite is a transparent, rebuildable operational
index. Sharing the source files while allowing each tool to keep its own cache
is an established pattern, not needless index duplication.

Obsidian properties are YAML frontmatter values. Obsidian supports scalar and
list property types, but its property UI does not support nested properties;
nested YAML remains available in source mode. This means Qipu's simple fields
fit well, while `links`, `sources`, and `custom` need careful UX treatment. See
[Properties](https://obsidian.md/help/properties).

Obsidian supports wikilinks and Markdown links, vault-root and relative paths,
headings, blocks, aliases, embeds, and shortest-unique-path generation. It can
update links when files are renamed. Some forms, especially block references,
are Obsidian-specific. See [Internal links](https://obsidian.md/help/Linking%2Bnotes%2Band%2Bfiles/Internal%2Blinks),
[Aliases](https://obsidian.md/help/aliases), and
[Files and links settings](https://obsidian.md/help/settings).

## Comparable tools and the patterns they validate

| Tool | What it does | Integration pattern | Lesson for Qipu |
|---|---|---|---|
| Obsidian metadata cache | Indexes file metadata for graph, outline, and navigation; rebuildable when stale | App-owned derived cache over vault files | Do not integrate with the cache; independently derive Qipu's index from the shared source |
| [Obsidian Bases](https://obsidian.md/help/bases) | Database-like table/list/card/map views over files and YAML properties; view definitions live in `.base` YAML | Views over file-backed data | File-backed structured metadata is a first-class Obsidian pattern; Qipu properties could be exposed in generated Base views |
| [Dataview](https://github.com/blacksmithgu/obsidian-dataview) | Indexes frontmatter, inline fields, tags, tasks, and file metadata for DQL/JavaScript queries | Plugin-local read-oriented index | A consumer can add rich query behavior without owning the source format |
| [Omnisearch](https://github.com/scambier/obsidian-omnisearch) | BM25-style ranked search over notes and optionally extracted PDF/OCR text; offers an optional local HTTP API | Specialized derived search index | Multiple rebuildable indexes can coexist; Qipu should keep its domain-specific search and graph behavior |
| [Smart Connections](https://github.com/brianpetro/obsidian-smart-connections) | Automatically embeds vault content and surfaces semantically related notes | Specialized local semantic index | Enrichment can remain local and derived; it need not change the vault's ownership model |
| [QMD](https://github.com/tobi/qmd) and [its Obsidian adapter](https://github.com/achekulaev/obsidian-qmd) | Points a standalone CLI collection at a directory/glob, builds keyword/vector indexes, and lets an Obsidian plugin trigger updates | Headless sidecar index plus optional thin editor adapter | Closest implementation precedent: standalone engine owns its index; the vault is selected by path and mask |
| [Khoj](https://github.com/khoj-ai/khoj) | Indexes Markdown and other personal documents for semantic search/agents and exposes the experience in Obsidian and other clients | External service plus editor client | Obsidian can be one interface among several; Qipu should not make its domain engine editor-dependent |
| [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) | Exposes authenticated CRUD, structured search, commands, and MCP through an Obsidian plugin | Bridge through a running Obsidian instance | Useful optional UX seam, but unsuitable as Qipu's required storage path |
| [Obsidian CLI](https://obsidian.md/help/cli) | Controls a running Obsidian app for search, CRUD, Bases, properties, commands, and automation | Official running-app bridge | Offers high semantic fidelity but compromises Qipu's standalone/headless contract if made mandatory |
| [Obsidian Headless Sync](https://obsidian.md/help/sync/headless) | Synchronizes vault files without the desktop app; currently open beta and requires Obsidian Sync | File synchronization, not a vault query engine | Could deliver vault files to a server where Qipu indexes them; it does not replace Qipu storage/query logic |

The strongest common pattern is **files as source, tool-specific index as
derived state**. Tools that expose Obsidian itself through REST, MCP, or the
official CLI solve a different problem: controlling the application and its
plugins. Qipu's core value is deterministic, local, agent-oriented graph logic,
so application control should remain optional.

## Fit with Qipu's current model

### Natural matches

| Concern | Obsidian | Qipu | Fit |
|---|---|---|---|
| Durable content | Markdown files | Markdown files | Excellent |
| Structured metadata | YAML properties | YAML frontmatter | Excellent for scalar/list fields |
| Source ownership | Files are editable by other tools | Notes are source of truth | Excellent |
| Derived navigation | Rebuildable metadata/search indexes | Rebuildable SQLite index | Excellent |
| Links | Wikilinks and Markdown links | Wikilinks, Markdown links, typed links | Strong substrate; different resolution rules |
| Folders | Arbitrary nested vault layout | Flat managed `notes/` and `mocs/` | Adapter required |
| Identity | Path/name-oriented links; aliases | Stable collision-resistant ID | Semantic mismatch that must be preserved, not removed |
| Graph semantics | Mostly untyped inline links | Inline plus ontology-defined typed links | Qipu adds value rather than duplicating Obsidian |
| Headless automation | Limited app CLI; Sync has a headless client | Standalone deterministic CLI | Qipu should remain independent |

Qipu's stable ID, typed-edge ontology, graph traversal, validation, slices,
linked collection roots, records output, and bounded agent context remain
distinct value. Vault support should not reduce Qipu to another search plugin.

### Current implementation gaps

The current implementation cannot safely point at an arbitrary existing vault:

1. **Discovery is layout-specific.** Rebuild and indexing scan only `notes/`
   and `mocs/` below the store root. A vault is recursively organized and may
   contain Markdown anywhere.

2. **Frontmatter is mandatory and lossy on write.** Qipu requires `id` and
   `title`. `NoteFrontmatter` does not flatten unknown top-level YAML fields, so
   parsing and saving an Obsidian note would discard properties owned by the
   user or other plugins. This is a hard blocker for write mode.

3. **Link identity differs.** Qipu's wikilink extractor treats the target as an
   ID. Obsidian commonly links by filename or vault path. Qipu also does not
   currently model Obsidian heading/block suffixes, aliases, embeds,
   shortest-unique-path lookup, URL-encoded Markdown paths, or extensionless
   Markdown targets.

4. **Mutation can race the editor.** Qipu reads and later writes whole files.
   Obsidian's own plugin guidance recommends process-style compare-and-update to
   avoid overwriting external changes. A vault adapter needs optimistic
   concurrency checks and atomic replacement. See the official
   [Vault API guidance](https://docs.obsidian.md/Plugins/Vault).

5. **Placement rules conflict.** Qipu creates files from type in `notes/` or
   `mocs/` and may rename them from ID/title. Vault users expect configurable
   folders, filenames, templates, and Obsidian-managed link updates.

6. **Visibility differs.** Obsidian normally ignores dot-prefixed folders.
   Therefore a default `.qipu/notes/` store is not a useful Obsidian authoring
   surface, while a visible `qipu/notes/` store is. In a future vault mode,
   `.qipu/` is a good location for Qipu's config/cache precisely because the
   vault content lives elsewhere.

7. **Exclusions can diverge.** Obsidian has configurable excluded files that
   affect search, graph, and suggestions. Qipu needs explicit include/exclude
   globs rather than silently depending on private Obsidian settings.

These are adapter problems. None require replacing the accepted Markdown plus
SQLite architecture.

## Architecture options

### 1. Visible Qipu subtree inside a vault

Initialize Qipu with its visible `qipu/` directory at the vault root. Obsidian
then edits the Qipu-managed notes as ordinary vault files.

**Advantages:** works now; no migration; preserves every Qipu invariant; no
Obsidian runtime dependency.

**Limits:** only the Qipu subtree participates in Qipu; existing vault notes are
not indexed; separate `notes/` and `mocs/` folders remain visible conventions.

**Assessment:** document and test this as the immediate compatibility path, but
do not present it as full vault support.

### 2. Import/export or mirrored notes

Copy vault notes into Qipu, or export Qipu notes into the vault.

**Advantages:** relatively small implementation and clear ownership during a
one-time migration.

**Limits:** continuous use creates two sources of truth, conflict policy,
duplicate attachments, link rewrites, and synchronization failure modes.

**Assessment:** useful for migration only. Do not use mirroring as the normal
integration architecture.

### 3. Native vault layout adapter

Keep Qipu's domain `Store`, `Note`, ontology, and `Database`, but place file
discovery, identity/property mapping, link resolution, and mutation behind a
layout/source adapter. Current managed-store behavior remains the default;
`vault` is an explicit alternative profile.

**Advantages:** full headless Qipu behavior over one source of truth; follows
the dominant ecosystem pattern; keeps Qipu useful from shells, agents, CI, and
servers whether or not Obsidian is installed.

**Limits:** meaningful engineering work, especially lossless YAML, identity
adoption, link resolution, and concurrent writes.

**Assessment:** recommended end state.

### 4. Delegate storage to Obsidian CLI, REST, or a plugin

Use the official CLI or a plugin API for all reads and writes.

**Advantages:** Obsidian performs its own link resolution, rename handling,
frontmatter edits, trash behavior, and plugin commands.

**Limits:** the official CLI requires the desktop app to run; REST/MCP requires
a community plugin and credentials; mobile and server behavior differ; latency
and availability become external concerns; Qipu would no longer be a standalone
local CLI.

**Assessment:** optional editor bridge only, never Qipu's core vault adapter.

### 5. Embed Qipu in an Obsidian plugin

Ship a thin plugin that invokes Qipu and renders commands/results in Obsidian.

**Advantages:** native discovery, command palette integration, visible doctor
results, and easy Base/view generation.

**Limits:** binary distribution, desktop-only process execution, duplicated UX,
and a much larger maintenance surface.

**Assessment:** defer until the headless vault adapter proves demand. The plugin
should remain thin and call the supported Qipu CLI.

## Recommended vault contract

### Configuration and layout

- Add an explicit `managed` versus `vault` source-layout setting.
- Keep the Qipu operational directory in `<vault>/.qipu/` in vault mode.
- Recursively scan configured content roots with explicit include/exclude globs.
- Default exclusions should include `.obsidian/`, `.qipu/`, `.trash/`, and other
  hidden operational folders, while remaining inspectable and overridable.
- Continue excluding attachment content from search per the current operational
  database spec.
- Never infer vault mode merely because `.obsidian/` exists; require an explicit
  initialization/adoption action, consistent with ADR 0003's explicitness rule.

### Identity and adoption

- Preserve stable Qipu IDs as the durable graph identity. Do not replace them
  with paths or filenames.
- Permit a configurable property name, with `id` for Qipu-native notes and a
  collision-resistant name such as `qipu-id` for minimally invasive adoption.
- For unmanaged notes, derive title from a configured property, first heading,
  or filename for read-only inspection.
- Provide `qipu vault adopt --dry-run` before any metadata mutation. Adoption
  assigns stable IDs, reports duplicate/ambiguous links, and makes all intended
  file changes reviewable.
- Do not silently assign path-hash IDs as durable identity: a rename would
  otherwise change identity. A provisional path identity may be used internally
  during a read-only scan but must not escape as a stable Qipu ID.

### Metadata

- Round-trip every unknown top-level YAML property byte-semantically or
  value-semantically without deletion.
- Map Qipu core fields explicitly and expose other top-level properties through
  custom filtering without forcing them under a nested `custom` object.
- Keep ontology-defined note types and typed links optional. Missing type uses
  the configured default; folder location must not imply `moc` or any other
  ontology term.
- Treat nested Qipu `links` and `sources` as valid source-mode YAML. An optional
  generated Obsidian Base may expose simpler Qipu fields, but Bases must not
  become authoritative.

### Links and graph

- Parse wikilinks, Markdown links, and embeds.
- Separate a link's file target from its heading/block subpath and display text.
- Resolve exact vault paths, relative paths, and filename-oriented wikilinks
  using a deterministic vault index; report ambiguity instead of guessing.
- Resolve the file to a stable Qipu ID before writing an edge to SQLite.
- Keep Obsidian inline links as `related` edges unless a Qipu typed link supplies
  stronger ontology semantics.
- Preserve unresolved and ambiguous references for `qipu doctor`.
- Keep linked collection roots ontology-neutral in accordance with ADR 0002.

### Mutations and consistency

- Start vault support read-only, then enable writes only after lossless YAML and
  link round-trip tests pass.
- Before writing, compare the current content hash/mtime with the version Qipu
  parsed. On mismatch, retry a frontmatter-aware merge or fail with a conflict;
  never overwrite silently.
- Write through a temporary file plus atomic replacement where supported.
- Reconcile external creates, edits, renames, and deletes incrementally on every
  invocation. A file watcher may improve long-running integrations later but is
  not required for CLI correctness.
- Keep SQLite private and rebuildable. Neither Obsidian nor plugins should read
  `qipu.db`, and Qipu should not read Obsidian's IndexedDB cache.

## Suggested delivery sequence

1. **Compatibility documentation and fixture:** document visible-subtree use and
   add a representative Obsidian vault fixture to prevent syntax regressions.
2. **Read-only vault index:** introduce the source-layout adapter, recursive
   discovery, exclusions, derived titles, and Obsidian link resolution. Search,
   list, context, graph, and doctor operate without changing files.
3. **Adoption workflow:** dry-run/report first, then opt-in stable ID assignment.
   No general write commands yet.
4. **Lossless metadata mutation:** preserve arbitrary YAML, add optimistic
   concurrency and atomic file updates, then enable update/link commands.
5. **Vault-aware creation and rename:** configurable folders/templates/naming;
   specify how link updates are handled before enabling rename behavior.
6. **Optional Obsidian surface:** generated `.base` views and/or a thin plugin
   that invokes Qipu. Keep this separate from the storage contract.

Each phase is independently useful. Read-only indexing proves compatibility and
retrieval value before Qipu is trusted to mutate existing vaults.

## Decision

Proceed toward a native vault layout adapter, starting read-only. Preserve the
current managed Qipu store as the default and preserve ADR 0001: Markdown remains
authoritative and Qipu's SQLite database remains its only operational index.

Do not:

- read or depend on Obsidian's IndexedDB metadata cache;
- make the Obsidian desktop app, CLI, Sync subscription, or a community plugin
  a prerequisite for Qipu;
- maintain a continuously mirrored second copy of notes;
- index arbitrary vault files and then allow writes before property round-trip,
  stable identity, link resolution, and concurrency are correct;
- treat a literal `moc` type or `mocs/` folder as the definition of a linked
  collection root.

This direction expands Qipu's addressable authoring ecosystem without giving up
the logic that makes Qipu more than an Obsidian search/index plugin.
