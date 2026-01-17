# Indexing, Search, and Navigation

## Scope
This spec describes how qipu makes a note store navigable:

- Fast listing and filtering
- Full-text search across notes
- Backlinks and graph traversal

This is *only* for the qipu knowledge store. It is not intended to replace repo-wide grep.

## Indexes (derived data)
Qipu should be able to build these derived views:

1. **Metadata index**: `id -> {title,type,tags,path,created,updated}`
2. **Tag index**: `tag -> [ids…]`
3. **Backlink index**: `id -> [ids that link to it…]`
4. **Graph**: adjacency list of links between notes (inline + typed)

Proposed cache locations:
- `.qipu/.cache/*.json` (portable, tool-friendly)
- `.qipu/qipu.db` (optional, derived, fast)

This mirrors beads’ architecture: git-trackable source data plus a local acceleration layer.

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
- treat unknown IDs as “unresolved” (reported by `doctor`)
- ignore links outside the store by default

## Search behavior
### Full-text search
- Search note `title` + body by default.
- Provide type/tag filters.
- Provide an option to include/exclude MOCs.

Implementation options:
- A simple embedded matcher for smaller stores
- Optional integration with `ripgrep` if available
- Optional SQLite FTS if `.qipu/qipu.db` is present

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

## Open questions
- Should qipu standardize on JSON indexes, SQLite indexes, or support both?
- Should backlinks be embedded into notes (opt-in) or remain fully derived?
