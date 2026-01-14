# Qipu Implementation Plan (Remaining Work)

## **🎉 P0 ISSUES COMPLETED!**

✅ **ALL P0 Items COMPLETED (10 of 10):**
1. Fix CLI JSON behavior - --format flag parsing for equals syntax
2. Fix CLI JSON behavior - help/version exit codes  
3. Fix CLI JSON behavior - error envelope exit codes
4. Eliminate nondeterminism - remove runtime timestamps
5. Eliminate nondeterminism - fix HashMap iteration order
6. Eliminate nondeterminism - add stable tie-breakers for sorting
7. Context budget enforcement - make exact across all formats
8. Implement dump/load commands - complete CLI definitions, serialization, and all functionality
9. ✅ Fix search/index determinism - Ripgrep JSON parsing and stable ordering
10. ✅ Implement LLM user validation harness per specs/llm-user-validation.md (tests/llm_validation.rs; transcripts ignored in .gitignore)

**SUMMARY:**
- **10/10 P0 items completed (100%)**
- **0/10 P0 items pending (0%)**

**NEXT PRIORITIES:**
- All P0 items are now complete!
- Move on to P1 items as needed

**NOTE:** Error code consistency fixes are complete and working - all usage validation now returns exit code 2 as specified. All major P0 CLI functionality is now complete with the dump/load implementation fully functional.

---

- [P0] ~~Implement `qipu dump` and `qipu load` per `specs/pack.md`~~ **COMPLETED**: Full implementation with CLI definitions, note selection with graph traversal, pack format serialization (JSON/records), deserialization, attachment handling, link management, and comprehensive error handling. All 198 tests passing.

- [P0] ~~Implement the `specs/llm-user-validation.md` “LLM primary user” validation harness (currently only a spec)~~ **COMPLETED:** Harness exists in `tests/llm_validation.rs` with transcript capture to `tests/transcripts/<tool>/<timestamp>/` and store validation.
- [P0] ~~Add `.gitignore` coverage for volatile transcript artifacts described in `specs/llm-user-validation.md`~~ **COMPLETED:** transcripts ignored in `.gitignore`.
- [P1] ~~Fix CLI parse-time JSON behavior to match `specs/cli-tool.md`~~ **COMPLETED:** JSON output now works correctly for both `--format json` and `--format=json` syntax variants.
- [P1] ~~Ensure `qipu --help` and `qipu --version` always exit `0` and print normal help/version output even when `--format json` is present~~ **COMPLETED:** Help and version commands now properly exit with code 0 regardless of format flag.

- [P1] ~~Bring compaction resolution in line with `specs/compaction.md` across command surfaces~~ **COMPLETED:** `show`/`context` now resolve to canonical digests by default with `--no-resolve-compaction` support, `show --links` applies contracted-graph semantics, `context` preserves `via` for query-driven selections, and `link` list/tree/path aggregate across full compaction equivalence classes (transitive chains).

- [P1] ~~Align `qipu context --moc <id>` behavior with `specs/cli-interface.md`~~ **COMPLETED:** MOC selection now includes the MOC itself alongside linked notes with deterministic ordering/dedup.

- [P1] Reconcile store discovery requirements across specs: `specs/cli-tool.md` currently describes discovering `.qipu/` only, while `specs/storage-format.md`, `specs/cli-interface.md`, and the implementation support both `.qipu/` and `qipu/`; decide the intended behavior and update spec(s) accordingly.

- [P1] ~~Fix records header provenance (`store=...`) to always report the actual opened store root (not `cli.store` or a hardcoded `.qipu` fallback), including at least `qipu doctor --format records` and `qipu index --format records`.~~ **COMPLETED:** records header now uses the actual opened store root for `qipu index --format records` and `qipu doctor --format records`.

- [P1] Make search output fully deterministic:
  - Add stable tiebreakers in result sorting (e.g., when relevance ties, sort by `(relevance desc, id asc)`), and ensure canonicalization/dedup does not introduce unstable ordering.

- [P1] ~~Meet `specs/records-output.md` budgeting requirements (including `qipu link list --format records` budget support).~~ **COMPLETED**

- [P2] Fix incremental indexing correctness and cache portability:
  - Ensure incremental indexing updates remove stale tag memberships when a note’s tags change (avoid accumulating outdated `tag -> ids[]`).
  - Decide whether `Index.metadata.path` should be store-relative (per `specs/indexing-search.md`) and ensure consumers remain correct (including ripgrep-assisted search matching).
  - Decide whether to keep a single `.cache/index.json` or split into multiple cache files as described in `specs/indexing-search.md`, and update spec or implementation accordingly.

- [P2] Remove non-verbose debug/progress noise on stderr so commands remain scriptable and quiet by default (e.g., unconditional parse warnings in note listing and progress/warnings during export), or gate them behind `--verbose`.
- [P2] Make `--verbose` timing output consistent across commands (avoid resetting the timing epoch inside individual command handlers such as `sync`; keep stable phase keys/labels).
- [P2] Make golden output path normalization portable (avoid hardcoding platform-specific temp path shapes in goldens).
- [P2] Ensure JSON outputs meet the minimum note schema described in `specs/cli-interface.md` (notably include both `created` and `updated` where applicable, e.g., `create`, `capture`, `inbox`).

- [P3] Optional: decide whether `qipu link add/remove` should require `--type` (as proposed in `specs/cli-interface.md`) or allow a default of `related`, and update spec/implementation accordingly.
- [P3] Optional: implement git automation in `qipu sync` when `store.config().branch` is set (switch branch, commit changes, optional push), guarded behind explicit flags.
- [P3] Optional: implement export attachment copying (e.g., `qipu export --with-attachments`) as documented as a “future enhancement” in `docs/attachments.md`.
- [P3] Optional: remove `qipu sync` placeholder output values by refactoring `doctor` to return structured results in addition to printing them (so sync can report consistent totals in JSON/records modes when `--validate` is used).
