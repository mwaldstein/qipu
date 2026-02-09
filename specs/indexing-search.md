# Indexing, Search, and Navigation

## Scope
This spec describes how qipu makes a note store navigable:

- Fast listing and filtering
- Full-text search across notes
- Backlinks and graph traversal

This is *only* for the qipu knowledge store. It is not intended to replace repo-wide grep.

## Indexes (derived data)
Qipu builds these derived views:

1. **Metadata index**: `id -> {title,type,tags,path,created,updated}`
2. **Tag index**: `tag -> [ids…]`
3. **Backlink index**: `id -> [ids that link to it…]`
4. **Graph**: adjacency list of links between notes (inline + typed)

The index is stored in `.qipu/qipu.db` (derived, gitignored).

## Incremental indexing
Indexing should be incremental where practical:
- Track file mtimes or content hashes
- Re-parse only changed notes

Provide `qipu index --rebuild` to drop and regenerate caches.

## Link extraction
Links can be discovered by parsing:
- wiki links: `[[<id>]]` / `[[<id>|label]]`
- markdown links pointing to qipu notes
- typed links in frontmatter (`links[]`)

Link extraction should:
- treat unknown IDs as "unresolved" (reported by `doctor`)
- ignore links outside the store by default

## Search behavior
### Full-text search
- Search note `title` + body by default.
- Provide type/tag filters.
- Provide an option to include/exclude MOCs.

See `operational-database.md` for search implementation requirements.

### Result ranking (initial heuristics)
- Title matches rank above body matches.
- Exact tag matches rank above plain text.
- Recently updated notes can receive a small boost.

Keep ranking simple and explainable.

## Navigation
### Backlinks
`qipu link list <id> --direction in` should render backlinks (direct inbound links).

For multi-hop traversal that includes backlinks, use `qipu link tree <id> --direction in|both`.

### Related notes
Relatedness can be approximated via:
- shared tags
- direct links
- typed link semantics
- 2-hop link neighborhoods

## Not replacing code search
Qipu must avoid scope creep into code search:
- qipu search is limited to the qipu store by default
- it should not attempt to index source code files

## Design decision: Derived backlinks

**Backlinks remain fully derived** (not embedded in note content). This decision:

- **Maintains single source of truth**: Links are stored in note frontmatter (`links[]`) and inline wiki links; backlinks are computed from these
- **Prevents synchronization issues**: Embedding backlinks would require keeping them in sync with actual links
- **Enables efficient queries**: The backlink index in `.qipu/qipu.db` supports fast reverse lookups
- **Supports semantic inversion**: Derived backlinks allow virtual edge transformation (see `graph-traversal.md`)

If users need backlink visibility in note content, use `qipu show <id>` or `qipu link list <id> --direction in`.
