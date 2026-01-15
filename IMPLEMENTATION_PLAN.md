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
- **211 tests passing (62 unit + 131 CLI + 6 golden + 6 performance + 6 LLM validation)**
- **Git tag v0.0.90 created**

**NEXT PRIORITIES:**
- All P0 items are now complete!
- Move on to P1 items as needed

**NOTE:** Error code consistency fixes are complete and working - all usage validation now returns exit code 2 as specified. All major P0 CLI functionality is now complete with the dump/load implementation fully functional.

---

## Inconsistencies and Missing Spec Updates (COMPLETED)

- [P1] ~~Fix search recency boost to use `updated` timestamp instead of `created` per `specs/indexing-search.md`.~~ **COMPLETED**: Updated `src/lib/index/search.rs` to use `updated` (falling back to `created`) for recency boost calculation.
- [P2] ~~Update `specs/cli-interface.md` to include implemented but missing flags and commands:~~ **COMPLETED**:
  - Add `--links` to `qipu show`.
  - Add search filters (`--type`, `--tag`, `--exclude-mocs`) to `qipu search`.
  - Add `qipu compact`, `qipu dump`, and `qipu load` command summaries.
  - Document `--max-chars` for `qipu link` commands.
- [P2] ~~Clarify `qipu inbox --exclude-linked` flag in `specs/cli-interface.md`.~~ **COMPLETED**
- [P3] ~~Ensure search `exclude_mocs` is documented consistently.~~ **COMPLETED**
- [P1] ~~Fix golden test failure for version output (already done, but document it).~~ **COMPLETED**: Updated `tests/golden/version.txt` to match `Cargo.toml` version `0.0.89`.

---

## Code Quality Improvements (Completed 2026-01-15)

**Fixed error handling issues:**
- Fixed regex creation panic risk in doctor.rs (line 270) - added proper error message
- Fixed 5 instances of current_dir().unwrap() in compact.rs - added fallback to PathBuf::from(".")
- Pattern now matches the safe fallback used in main.rs:102

**Removed redundant #[allow(dead_code)] attributes:**
- src/lib/error.rs:33 - QipuError enum IS used throughout codebase
- src/lib/id.rs:51 - NoteId impl IS used throughout codebase

**Removed truly unused code:**
- NoteFrontmatter::with_tag() method (duplicate of with_tags())
- VALID_FORMATS constant and is_human()/is_json()/is_records() methods in format.rs
- Index::note_ids() method (trivial inline alternative exists)
- Index::get_all_edges() method (simple combination of get_outbound_edges/get_inbound_edges)

**All 207 tests passing (63 unit + 126 CLI + 6 golden + 6 performance + 6 LLM validation).**

---

## Issues Fixed 2026-01-15

**Fixed flaky test:**
- Fixed `test_incremental_index_updates_tags` in `src/lib/index.rs:1152` by increasing sleep from 1s to 2s. The test was failing when running in parallel due to insufficient filesystem mtime granularity. Test now passes reliably.

**Code quality improvements:**
- Added Default implementation for OpenCodeAdapter in tests/llm_validation.rs
- Removed empty string literal from writeln! in tests/llm_validation.rs
- Git tag v0.0.87 created
