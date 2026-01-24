# Qipu Future Work

Items in this file are NOT ready for immediate implementation. They require design decisions, spec clarification, or depend on unimplemented infrastructure.

For concrete bugs and implementation tasks, see [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md).  
For specification status, see [`specs/README.md`](../specs/README.md).

*Last updated: 2026-01-24*

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

### llm-context.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Prime command size semantics | Spec says "~4-8k characters" - implemented correctly | 24 |
| Remove `--max-tokens` flag | Qipu standardizes on character-based budgets only | 38 |
| Automatic summarization for long notes | Open question: "Should qipu support lightweight automatic summarization (without an LLM)?" | 119 |
| Backlinks in context bundles | Open question: Should backlinks be included by default or via `--backlinks` flag? | 121 |

### graph-traversal.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Hop limit semantics | Spec says "max-hops <n>" (suggests integer) but implementation uses cost budget (f32 HopCost) | 86-92 |
| Semantic inversion type filtering | When filtering by `--type supported-by` with inversion, should match virtual inverted edges or original stored links? | 44-58 |
| Path result JSON shape | Spec mentions `notes`, `links`, `path_length` but implementation adds `from`, `to`, `direction`, `found` | 134-145 |

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

### structured-logging.md

| Item | Issue |
|------|-------|
| Full instrumentation | Deciding which functions warrant timing spans |
| Resource usage metrics | Memory, cache hits - infrastructure not present |
| Error chain trace format | Needs design for structured error context |

### llm-user-validation.md (Additional)

| Item | Issue | Spec Line |
|------|-------|-----------|
| Accurate cost estimation | Current implementation uses `len() / 4` character-based approximation; should parse actual token counts from tool output | 266-283 |
| Budget enforcement | Only warns when exceeded, doesn't prevent run | 417-424 |

---

## Needs Spec Clarification

### provenance.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| `source` vs `sources[]` semantics | Spec defines `source` as singular string; implementation also has `sources[]` array with Source{url,title,accessed} structure; bibliography export only reads `sources[]` | 11-17 |
| Bibliography input field | Implementation reads only `sources[]` for bibliography export; notes with singular `source` field (from `qipu capture --source`) are excluded | 58-61 |

### storage-format.md

| Item | Issue | Spec Line |
|------|-------|-----------|
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

### Value Model

| Item | Source | Status |
|------|--------|--------|
| Value in search ranking by default | `specs/value-model.md` line 187 | Currently only with `--sort value`; should it combine with relevance score? |
| Value in compaction suggestions | `specs/value-model.md` line 189 | Should low-value notes be prioritized for compaction? |
| Digest notes auto-value boost | `specs/value-model.md` line 188 | Should digest notes automatically receive higher value? |

### Knowledge Model

| Item | Source | Status |
|------|--------|--------|
| Duplicate detection & merge | `specs/knowledge-model.md` line 101 | Similarity foundation exists, needs command surface |
| Tag aliases | `specs/knowledge-model.md` line 56 | Marked as optional in spec |
| MOC validation | Missing validation that MOCs contain links | Not in spec, could be enhancement |

### Graph & Traversal

| Item | Source | Status |
|------|--------|--------|
| Inline link materialization | `specs/graph-traversal.md` line 33 | Optional: rewrite inline to frontmatter |
| Additional traversal queries | `specs/graph-traversal.md` line 217 | Future: `neighbors`, `subgraph`, `cycles` |

### Storage & Database

| Item | Source | Status |
|------|--------|--------|
| Query statistics | `specs/operational-database.md` line 174 | Observability enhancement |
| Performance targets validation | `specs/operational-database.md` line 161-166 | <50ms search, <10ms backlinks, <100ms traversal |

### CLI Interface & User Experience

| Item | Source | Status |
|------|--------|--------|
| Interactive fzf-style pickers | `specs/cli-interface.md` line 189 | Open question - optional UX sugar |
| `qipu sync` git integration | `specs/cli-interface.md` line 191 | Open question - manage git commits? |
| Verbose timing keys | `specs/README.md` line 89 | Low priority - only `discover_store` instrumented |

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

---

## Review Schedule

This document should be reviewed quarterly to:
- Promote items to `IMPLEMENTATION_PLAN.md` when they become concrete
- Archive items that prove unnecessary
- Add new future work identified during development

When an item moves from "future" to "planned", it should migrate to `IMPLEMENTATION_PLAN.md` with concrete implementation steps.
