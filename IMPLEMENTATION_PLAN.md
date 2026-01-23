# Qipu Implementation Plan

For exploratory future work, see [`FUTURE_WORK.md`](FUTURE_WORK.md).

## Status

- **Test baseline**: 791 tests pass
- **Schema version**: 6 (custom metadata column)
- **Last audited**: 2026-01-23
- **Last CI check added**: function complexity (>100 lines)

---

## TODO

### P1: Correctness

*No pending P1 items*

### P2: Technical Debt

| Task | Priority |
|------|----------|
| Refactor `src/commands/doctor/content.rs` (829 lines) | High |
| Refactor `src/commands/setup.rs` (780 lines) | Medium |
| Refactor `src/commands/doctor/database.rs` (722 lines) | Medium |
| Externalize model pricing to config | Low |
| Shared `OutputFormatter` trait | Low |

### Blocked

| Item | Blocker |
|------|---------|
| Release workflow | GitHub Actions enablement |
| `telemetry.md` | DRAFT spec; prohibits implementation |

---

## Completed (Summary)

**Revision 6** (2026-01-23): Refactored `src/lib/graph/bfs.rs` (842→820 lines). Extracted helper functions (`get_note_value`, `canonicalize_with_context`) to eliminate code duplication in neighbor processing loops across `bfs_search` and `dijkstra_search`.

**Revision 5** (2026-01-23): Added CI check for function complexity (>100 lines) with grandfathering for existing violations.

**Revision 4** (2026-01-23): Added CI check for file size (>500 lines) with grandfathering for existing violations.

**Revision 3** (2026-01-23): Spec audit - 14 correctness fixes, 4 test coverage items, 6 new features including `qipu context --walk`, `qipu store stats`, and custom metadata.

**Revision 2** (2026-01-23): Machine-readable output for `value`/`custom`, budget truncation, search breadcrumbs, major refactoring (bfs_find_path 400→59 lines, DoctorCheck trait).

**Revision 1** (2026-01-22): Core correctness fixes (Dijkstra ordering, JSON envelopes, value validation), 100+ new tests, custom metadata, export improvements.

See git history for full details.
