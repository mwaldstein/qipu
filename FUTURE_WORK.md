# Qipu Future Work

Items in this file are NOT ready for immediate implementation. They require design decisions, spec clarification, or depend on unimplemented infrastructure.

For concrete bugs and implementation tasks, see [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md).  
For specification status, see [`specs/README.md`](specs/README.md).

*Last updated: 2026-01-23*

---

## Deferred by Spec

### telemetry.md (DRAFT - DO NOT IMPLEMENT)

The telemetry spec is explicitly marked as draft with "DO NOT IMPLEMENT" warning.

| Item | Reason | Spec Line |
|------|--------|-----------|
| Usage analytics collection | Awaiting finalization and privacy review | 1-61 |
| Anonymous metrics | Design not approved | 15-30 |
| Opt-in/opt-out mechanism | Success criteria unchecked | 57-61 |

---

## Needs Design Work

### similarity-ranking.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Merge/same-as link suggestions for duplicates | Spec mentions "suggest merging or adding a `same-as` link" but no design for UX/automation | 40 |
| Related notes via 2-hop neighborhoods | TF-IDF exists but no explicit 2-hop relatedness algorithm | 65 |
| MOC clustering from similarity | Spec mentions as use case but no design | 8 |

### llm-context.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Automatic summarization for long notes | Open question: "Should qipu support lightweight automatic summarization (without an LLM)?" | 119 |

### workspaces.md

| Item | Issue | Spec Line |
|------|--------|-----------|
| Git integration for temp workspaces | Auto-add to .gitignore | 142 |
| Merge creates git commit | Design needed for commit message format | 143 |

### llm-user-validation.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Multi-model statistical benchmarking | Marked out of scope in spec | N/A |
| Real-time cost tracking via provider APIs | Marked out of scope | N/A |
| Interactive test authoring UI | Marked out of scope | N/A |
| CI integration | Marked too expensive | N/A |
| SQLite results.db | Spec says "Optional SQLite for queries" - design needed | 400-401 |
| Transcript redaction for secrets | Security feature, needs design | 505-512 |
| Event log rich format | Only simple `execution` event; spec shows spawn/tool_call/etc | 303-310 |

### distribution.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Tag-triggered GitHub Release automation | Workflow exists but is disabled + manual-only; blocked on Actions enablement | 18-35 |
| Canonical repo slug for installers | Installers hardcode `mwaldstein/qipu` but Cargo metadata points elsewhere; decide canonical repo | 18-35 |
| Homebrew tap | Requires separate repository setup | 62-69 |
| crates.io publishing | Account setup and verification needed | 92 |
| AUR (Arch Linux) | Medium priority package manager | 71-81 |
| Nix (NixOS/macOS) | Medium priority package manager | 71-81 |
| winget/Scoop (Windows) | Low priority package managers | 71-81 |
| deb/rpm (Debian/RHEL) | Low priority package managers | 71-81 |
| Release signatures (GPG/sigstore) | Security enhancement | 116-118 |

### compaction.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Depth-aware compaction metrics | Spec marks as optional ("may optionally") | 176-177 |
| Compaction versioning/history | Open question: inactive compaction edges for history? | 272 |
| MOC treatment in compaction | Open question: exclude MOCs by default? | 273 |
| Leaf source vs intermediate digest | Open question: first-class concept? | 274 |
| Alternate size bases | Future flags beyond summary-sized estimates | 173 |

---

## Needs Spec Clarification

### provenance.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| `source` vs `sources` semantics | Spec defines `source` only, but implementation uses `sources[]` heavily (context/export); decide canonical field + migration story | 11-17 |
| Bibliography input field | Implementation reads only `sources[]` for bibliography export; spec does not define bibliography behavior | 58-61 |

### storage-format.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Wiki-link canonicalization to markdown links | Spec says "optional; opt-in" but no flag/config defined | 113-114 |
| Per-note attachment folders | Open question: `attachments/<id>/...`? | 147 |

### indexing-search.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Backlinks embedded in notes | Open question: "Should backlinks be embedded into notes (opt-in) or remain fully derived?" | 72-73 |
| Field weights discrepancy | Spec says 2.0x/1.5x/1.0x; impl uses 5.0x/8.0x/0.0x additive boosts | 48-49 |
| Attachment content search | Open question: include PDFs, etc.? | 173 |

### value-model.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Value in `compact suggest` | Open question: Should compaction suggestions factor in value? | 189 |

### records-output.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| `S` prefix semantic overlap | In pack format `S` means sources; elsewhere it means summary | 55, 96, 162-167 |
| Format version selection | Open question: `records=1` for stability? | 151 |
| Edge inclusion control | Open question: default edges? | 152 |
| Body inclusion control | Open question: summaries only by default? | 153 |

---

## Open Questions from Specs

### CLI Interface & User Experience

| Item | Source | Status |
|------|--------|--------|
| Interactive fzf-style pickers | `specs/cli-interface.md` line 189 | Open question - optional UX sugar |
| `qipu capture` default type | `specs/cli-interface.md` line 190 | Open question - default to `--type fleeting`? |
| `qipu sync` git integration | `specs/cli-interface.md` line 191 | Open question - manage git commits? |
| Verbose timing keys | `specs/README.md` line 89 | Low priority - only `discover_store` instrumented |

### Knowledge Model

| Item | Source | Status |
|------|--------|--------|
| Duplicate detection & merge | `specs/knowledge-model.md` line 101 | Similarity foundation exists, needs command surface |
| Typed link set | `specs/knowledge-model.md` line 100 | Current set working, may expand based on usage |

### Graph & Traversal

| Item | Source | Status |
|------|--------|--------|
| Inline link materialization | `specs/graph-traversal.md` line 33 | Optional: rewrite inline to frontmatter |
| Additional traversal queries | `specs/graph-traversal.md` line 217 | Future: `neighbors`, `subgraph`, `cycles` |
| Context walk command | `specs/graph-traversal.md` line 209 | Future: `qipu context --walk` |
| Custom link types ecosystem | `specs/semantic-graph.md` line 94-106 | Extensibility exists, adoption depends on usage |

### LLM Integration

| Item | Source | Status |
|------|--------|--------|
| Beads usage audit | Research task | Observe agent workflows with `bd` vs `qipu` |
| Backlinks in context | `specs/llm-context.md` line 121 | Open question: `--backlinks` flag |

### Storage & Database

| Item | Source | Status |
|------|--------|--------|
| Query statistics | `specs/operational-database.md` line 174 | Observability enhancement |
| Database size/stats reporting | `specs/operational-database.md` line 175 | Diagnostic enhancement |

### Provenance

| Item | Source | Status |
|------|--------|--------|
| Commit linking | `specs/provenance.md` line 53 | Not needed; git handles this |
| Detailed activity tracking | `specs/provenance.md` line 54 | Future: `prompt_hash` → Activity Note |

---

## Infrastructure Dependencies

### structured-logging.md

Full instrumentation requires deciding on:
- Which functions warrant timing spans
- Resource usage metrics (memory, cache hits) - infrastructure not present
- Error chain trace format - needs design

Current state: Core tracing works, ~70% of spec requirements met.

### graph-traversal.md

| Item | Issue |
|------|-------|
| `--types <csv>` variant | Spec mentions CSV format as alternative to repeatable `--type`. Repeatable flag works, CSV not needed. |

---

## Not Planned

These items are explicitly out of scope or rejected:

| Item | Reason |
|------|--------|
| Ripgrep-based search | Removed in favor of SQLite FTS5 |
| Date-partitioned note paths | Spec explicitly uses flat structure |
| External code indexing | Store limited to qipu notes only |

---

## Review Schedule

This document should be reviewed quarterly to:
- Promote items to `IMPLEMENTATION_PLAN.md` when they become concrete
- Archive items that prove unnecessary
- Add new future work identified during development

When an item moves from "future" to "planned", it should migrate to `IMPLEMENTATION_PLAN.md` with concrete implementation steps.
