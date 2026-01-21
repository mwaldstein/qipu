# Qipu Future Work

This document tracks features and capabilities mentioned in specs that are explicitly marked as future work, optional extensions, or not yet in the implementation plan. Unlike [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md), these items are exploratory and do not require immediate implementation readiness.

For concrete bugs and implementation tasks, see [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md).  
For specification status, see [`specs/README.md`](specs/README.md).

**Last Updated:** 2026-01-20

---

## Future Features by Category

### CLI Interface & User Experience

#### Interactive Pickers
- **Source:** `specs/cli-interface.md` (Open Questions, line 189)
- **Description:** Optional fzf-style interactive pickers for improved UX
- **Status:** Open question - should qipu support this as optional sugar?

#### qipu capture Default Type
- **Source:** `specs/cli-interface.md` (Open Questions, line 190)
- **Description:** Should `qipu capture` default to `--type fleeting`?
- **Status:** Open question - needs user feedback

#### qipu sync Git Integration
- **Source:** `specs/cli-interface.md` (Open Questions, line 191)
- **Description:** Should `qipu sync` manage git commits/pushes, or stay index/validate-only?
- **Status:** Open question - team workflow feature

#### Verbose Timing Keys
- **Source:** `specs/README.md` (line 89)
- **Description:** Only `discover_store` is currently instrumented for `--verbose` timing. Need to add timing for other major phases (load indexes, execute command, etc.)
- **Status:** Low priority enhancement

### Knowledge Model

#### Tag Aliases
- **Source:** `specs/knowledge-model.md` (line 53), `specs/README.md` (line 96)
- **Description:** Support tag aliases for flexible tagging
- **Status:** Optional feature, not planned for implementation
- **Note:** Marked as optional in spec; simple on-disk representation preferred

#### Duplicate Detection & Merge
- **Source:** `specs/knowledge-model.md` (Open Questions, line 101)
- **Description:** Should qipu support duplicate/near-duplicate detection and merge (beads analog: `bd duplicates`/`bd merge`)?
- **Status:** Open question - similarity ranking foundation exists, command surface needed

#### Typed Link Set
- **Source:** `specs/knowledge-model.md` (Open Questions, line 100)
- **Description:** Which typed link set is the minimal useful set?
- **Status:** Open question - current set working well, may expand based on usage

### Graph & Traversal

#### Inline Link Materialization
- **Source:** `specs/graph-traversal.md` (line 33)
- **Description:** Optional rewriting/materialization of extracted inline links into `links[]` frontmatter (opt-in)
- **Status:** Optional implementation to reduce traversal work

#### Additional Traversal Queries
- **Source:** `specs/graph-traversal.md` (Open Questions, line 217)
- **Description:** Additional first-class traversal queries beyond `tree` and `path` (e.g., `neighbors`, `subgraph`, `cycles`)
- **Status:** Future extension - tree and path cover primary use cases

#### Context Walk Command
- **Source:** `specs/graph-traversal.md` (line 209)
- **Description:** Add `qipu context --walk <id> --max-hops <n> ...` to perform traversal-and-bundle in one command
- **Status:** Future-friendly extension - currently requires two commands

#### Variable Hop Costs
- **Source:** `specs/semantic-graph.md` (line 72-77)
- **Description:** Allow specific link types to have reduced cost (e.g., 0.5) or be "free" up to a limit for transitive/cohesive relationships like `part-of`
- **Status:** Future enhancement - v1 uses Cost = 1 for simplicity, engine should support variable costs

#### Custom Link Types Ecosystem
- **Source:** `specs/semantic-graph.md` (line 94-106)
- **Description:** Rich ecosystem of user-defined custom link types beyond the standard ontology
- **Status:** Extensibility mechanism exists, adoption depends on community usage patterns

### Compaction

#### Compaction Versioning/History
- **Source:** `specs/compaction.md` (Open Questions, line 272)
- **Description:** Should qipu support "inactive" compaction edges for history (versioning), or only one active mapping?
- **Status:** Open question - single active mapping working well

#### MOC Treatment in Compaction
- **Source:** `specs/compaction.md` (Open Questions, line 273)
- **Description:** Should compaction suggestions default to excluding MOCs/spec notes, or treat them like normal notes?
- **Status:** Open question - needs user feedback

#### Leaf Source vs Intermediate Digest
- **Source:** `specs/compaction.md` (Open Questions, line 274)
- **Description:** Should there be a first-class concept of "leaf source" vs "intermediate digest" in outputs?
- **Status:** Open question - current model working well

#### Depth-Aware Metrics
- **Source:** `specs/compaction.md` (line 176)
- **Description:** Optionally provide depth-aware compaction metrics (e.g., compaction percent at depth N)
- **Status:** Optional enhancement if `--compaction-depth` proves useful

#### Alternate Size Bases
- **Source:** `specs/compaction.md` (line 173)
- **Description:** Support alternate size bases beyond summary-sized estimates (e.g., body size)
- **Status:** Future flags, must keep defaults stable

### LLM Integration & Context

#### Beads Usage Audit
- **Source:** Research task
- **Description:** Audit how beads (`bd`) is used by LLM agents compared to qipu. Key questions:
  - How do agents discover and use `bd prime` vs `qipu prime`?
  - What patterns emerge for task tracking (beads) vs knowledge retrieval (qipu)?
  - Should qipu adopt any beads patterns (e.g., hooks, onboarding flow)?
  - How do agents combine beads + qipu in practice (task tracking + knowledge)?
- **Status:** Research needed - observe real agent workflows

#### Automatic Summarization (Without LLM)
- **Source:** `specs/llm-context.md` (Open Questions, line 120)
- **Description:** Should qipu support lightweight automatic summarization (without an LLM) for long notes?
- **Status:** Open question - extractive summaries might be useful

#### Backlinks in Context
- **Source:** `specs/llm-context.md` (Open Questions, line 121)
- **Description:** Should `context` support "include backlinks" as additional material?
- **Status:** Open question - `--backlinks` flag could be useful

### Search & Ranking

#### Related Notes Query
- **Source:** `specs/similarity-ranking.md` (line 31-35)
- **Description:** Fetch high-similarity unlinked notes for context expansion (threshold: Score > 0.3)
- **Status:** Foundation exists, needs command surface

#### Clustering for MOC Generation
- **Source:** `specs/similarity-ranking.md` (line 7)
- **Description:** Use similarity clustering to suggest MOC groupings
- **Status:** Future feature building on similarity foundation

#### See Also Suggestions
- **Source:** `specs/similarity-ranking.md` (line 5)
- **Description:** "See Also" suggestions based on content similarity
- **Status:** Future feature building on similarity foundation

### Storage & Indexing

#### Backlink Embedding
- **Source:** `specs/indexing-search.md` (Open Questions, line 73)
- **Description:** Should backlinks be embedded into notes (opt-in) or remain fully derived?
- **Status:** Open question - currently fully derived, works well

#### Attachment Content Search
- **Source:** `specs/operational-database.md` (Open Questions, line 173)
- **Description:** Should search include attachment content (PDFs, etc.)?
- **Status:** Open question - text extraction needed

#### Query Statistics
- **Source:** `specs/operational-database.md` (Open Questions, line 174)
- **Description:** Should we track query statistics for optimization?
- **Status:** Open question - observability enhancement

#### Database Size/Stats Reporting
- **Source:** `specs/operational-database.md` (Open Questions, line 175)
- **Description:** Should `qipu doctor` report database size/stats?
- **Status:** Open question - diagnostic enhancement

#### Per-Note Attachment Folders
- **Source:** `specs/storage-format.md` (Open Questions, line 147)
- **Description:** Should attachments be per-note folders (`attachments/<id>/...`)?
- **Status:** Open question - current flat structure working

### Workspaces

#### Git Integration & Auto-Ignore
- **Source:** `specs/workspaces.md` (Open Questions, line 142)
- **Description:** Should temporary workspaces be added to `.gitignore` automatically? (Yes, if `--temp` is used)
- **Status:** Planned but not implemented

#### Workspace Merge History
- **Source:** `specs/workspaces.md` (Open Questions, line 143)
- **Description:** Does a merge create a git commit in the primary store? (Ideally yes, if configured)
- **Status:** Open question - automation vs manual control

#### Rename/Fork Strategy
- **Source:** `specs/workspaces.md` (line 113-116)
- **Description:** ID collision resolution via renaming (e.g., `qp-a1b2` -> `qp-a1b2-1`) with link rewriting
- **Status:** Complex to implement correctly; simpler versions might just error

### Records Output

#### Format Version Selection
- **Source:** `specs/records-output.md` (Open Questions, line 151)
- **Description:** Should records output allow selecting a format version (e.g., `records=1` in the header) for stability?
- **Status:** Open question - version in header already, selection mechanism TBD

#### Edge Inclusion Control
- **Source:** `specs/records-output.md` (Open Questions, line 152)
- **Description:** Should records output include edges by default, or only with `--with-edges`?
- **Status:** Open question - current behavior needs validation

#### Body Inclusion Control
- **Source:** `specs/records-output.md` (Open Questions, line 153)
- **Description:** Should records output default to summaries only, requiring `--with-body` to include full content?
- **Status:** Open question - progressive disclosure model

### Provenance

#### Commit Linking
- **Source:** `specs/provenance.md` (line 53)
- **Description:** Rely on Git for history of changes (`wasRevisionOf`)
- **Status:** Not needed in frontmatter, git handles this

#### Detailed Activity Tracking
- **Source:** `specs/provenance.md` (line 54)
- **Description:** For complex pipelines, `prompt_hash` could link to a separate "Activity Note" that describes the full generation process
- **Status:** Future extension for advanced workflows

### Distribution

**Status:** Entire spec is early draft, marked as unimplemented in `specs/README.md`

#### Package Manager Support
- **Source:** `specs/distribution.md` (line 71-81)
- **Description:** Future support for:
  - AUR (Arch Linux) - Medium priority
  - Nix (NixOS/macOS) - Medium priority  
  - winget (Windows) - Low priority
  - Scoop (Windows) - Low priority
  - deb/rpm (Debian/RHEL) - Low priority
- **Status:** Future distribution channels

#### Release Signatures
- **Source:** `specs/distribution.md` (line 116-118)
- **Description:** Consider GPG or sigstore signing for releases
- **Status:** Security enhancement for future

### Telemetry

**Status:** ENTIRE SPEC IS DRAFT - DO NOT IMPLEMENT (line 4)

- **Source:** `specs/telemetry.md`
- **Description:** Privacy-focused usage analytics and telemetry system
- **Status:** Explicitly marked "DO NOT IMPLEMENT" - discussion only
- **Success Criteria:** Not met - requires privacy review, opt-out mechanisms, and finalization

---

## Implementation Notes

These future items differ from `IMPLEMENTATION_PLAN.md` in that they:

1. **Are exploratory** - May or may not be implemented depending on user feedback and usage patterns
2. **Require discovery** - Need real-world usage data to determine best approach
3. **Are optional enhancements** - Core functionality works without them
4. **Depend on adoption** - Some features (like custom link types) depend on community usage

When an item moves from "future" to "planned", it should migrate to `IMPLEMENTATION_PLAN.md` with concrete implementation steps.

---

## Review Schedule

This document should be reviewed quarterly to:
- Promote items to `IMPLEMENTATION_PLAN.md` when they become concrete
- Archive items that prove unnecessary
- Add new future work identified during development
