# Qipu Future Work

Items in this file are NOT ready for immediate implementation. They require design decisions, spec clarification, or depend on unimplemented infrastructure.

*Last updated: 2026-01-20*

---

## Deferred by Spec

### telemetry.md (DRAFT - DO NOT IMPLEMENT)

The telemetry spec is explicitly marked as draft with "DO NOT IMPLEMENT" warning.

| Item | Reason | Spec Line |
|------|--------|-----------|
| Usage analytics collection | Awaiting finalization and privacy review | 1-61 |
| Anonymous metrics | Design not approved | 15-30 |
| Opt-in/opt-out mechanism | Success criteria unchecked | 57-61 |

### knowledge-model.md (Optional)

| Item | Reason | Spec Line |
|------|--------|-----------|
| Tag aliases | Marked as optional in spec | 53 |

---

## Needs Design Work

### similarity-ranking.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Merge/same-as link suggestions for duplicates | Spec mentions "suggest merging or adding a `same-as` link" but no design for UX/automation | 40 |

### llm-context.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Automatic summarization for long notes | Open question: "Should qipu support lightweight automatic summarization (without an LLM)?" | 119 |

### export.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Pandoc integration for PDF export | Open question, no design | 56 |
| Transitive links (depth-limited) | Open question, no design | 57 |
| BibTeX/CSL JSON for bibliography | Marked as "future" | 29 |

### workspaces.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| `rename` merge strategy | Spec marks as "Complexity warning" - fork IDs to avoid conflicts | 113-116 |
| Git integration for temp workspaces | Auto-add to .gitignore | 142 |
| Merge creates git commit | Design needed for commit message format | 143 |
| `--from-note` graph slice | Currently copies single note only; spec says "like dump -> load" | Comment in `src/commands/workspace/new.rs:77-78` |

### operational-database.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Auto-trigger incremental repair on inconsistency | `validate_consistency()` returns bool but result is ignored | `src/lib/db/mod.rs:84` |
| Delete and rebuild on corruption | Spec mentions automatic recovery, not implemented | N/A |

### llm-user-validation.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Multi-model statistical benchmarking | Marked out of scope in spec | N/A |
| Real-time cost tracking via provider APIs | Marked out of scope | N/A |
| Interactive test authoring UI | Marked out of scope | N/A |
| CI integration | Marked too expensive | N/A |
| SQLite results.db | Spec says "Optional SQLite for queries" - design needed | N/A |
| Transcript redaction for secrets | Security feature, needs design | N/A |
| `LLM_TOOL_TEST_ENABLED` safety check | Environment variable not enforced | N/A |
| `LLM_TOOL_TEST_BUDGET_USD` enforcement | Session budget not enforced | N/A |
| Per-scenario `run.timeout_secs` | Uses CLI timeout, not per-scenario | N/A |
| Per-scenario `run.max_turns` | Not implemented | N/A |
| Per-scenario `cost.max_usd` | Not implemented | N/A |
| `run.json` metadata artifact | Detailed run metadata file not generated | N/A |
| `store_snapshot/` artifact | Snapshot of .qipu/ after run not captured | N/A |
| `report.md` artifact | Human-readable summary not generated | N/A |

### distribution.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Homebrew tap | Requires separate repository setup | 57-75 |
| crates.io publishing | Account setup and verification needed | 25-32 |
| Repository URL alignment | Cargo.toml says `anomalyco/qipu`, spec says `mwaldstein/qipu` | 11 vs 36,49 |

---

## Needs Spec Clarification

### storage-format.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Wiki-link canonicalization to markdown links | Spec says "optional; opt-in" but no flag/config defined | 113-114 |

### indexing-search.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Backlinks embedded in notes | Open question: "Should backlinks be embedded into notes (opt-in) or remain fully derived?" | 72-73 |

### value-model.md

| Item | Issue | Spec Line |
|------|-------|-----------|
| Value in `compact suggest` | Open question: Should compaction suggestions factor in value? | 189 |

---

## Infrastructure Dependencies

### structured-logging.md

Full instrumentation requires deciding on:
- Which functions warrant timing spans
- Resource usage metrics (memory, cache hits) - infrastructure not present
- Error chain trace format - needs design

Current state: Core tracing works, ~60% of spec requirements met.

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
