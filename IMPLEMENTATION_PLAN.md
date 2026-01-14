# Qipu Implementation Plan (Remaining Work)

- [P0] Implement `qipu dump` and `qipu load <pack>` per `specs/pack.md`: add CLI surface (`dump`, `load`), define a pack encoding with an explicit version marker, support selectors (`--note/--tag/--moc/--query`) plus traversal flags (`--direction/--max-hops/--type/--typed-only/--inline-only`), include attachments by default with `--no-attachments`, and ensure load round-trips notes + attachments without lossy transforms.
- [P0] Add pack test coverage: extend `tests/cli_tests.rs` for dump/load behavior (including selectors/traversal + `--no-attachments`), add/update goldens where determinism matters, and update `tests/golden/help.txt` once new commands exist.

- [P0] Implement the `specs/llm-user-validation.md` “LLM primary user” validation harness: add an E2E test runner with pluggable tool adapters (OpenCode first), transcript capture to `tests/transcripts/<tool>/<timestamp>/`, and programmatic validation of resulting store state (notes created, links created, and retrieval works).
- [P0] Add `.gitignore` coverage for volatile transcript artifacts described in `specs/llm-user-validation.md` (at minimum ignore `tests/transcripts/`).

- [P1] Bring compaction resolution in line with `specs/compaction.md` across command surfaces:
  - `qipu show`: default to resolved view (if an ID is compacted, redirect to `canon(id)`), and only show raw compacted notes when `--no-resolve-compaction` is set.
  - `qipu show --links`: apply the same contracted-graph semantics as `qipu link list` when compaction is resolved.
  - `qipu context`: in resolved view, canonicalize + deduplicate selected IDs (including `--query` hits) and hide compacted source notes by default; preserve a `via`/breadcrumb mechanism for query-driven selection.
  - `qipu export`: apply resolved/raw toggle and decide deterministic ordering behavior when MOCs link to compacted notes (export canonical digest vs raw source, and how to annotate).
  - `qipu link` list/tree/path: aggregate edges for a canonical digest across the full compaction equivalence class (transitive chains), not just direct `compacts`.

- [P1] Reconcile store discovery requirements across specs: `specs/cli-tool.md` currently describes discovering `.qipu/` only, while `specs/storage-format.md`, `specs/cli-interface.md`, and the implementation support both `.qipu/` and `qipu/`; decide the intended behavior and update spec(s) accordingly.

- [P1] Fix CLI parse-time JSON behavior to match `specs/cli-tool.md`:
  - Detect JSON requests in argv for both `--format json` and `--format=json`.
  - Ensure `qipu --help` and `qipu --version` always exit `0` and print normal help/version output even when `--format json` is present (avoid treating clap help/version as an error path).
  - Ensure usage/parse errors emit a structured JSON envelope with correct exit code (`2` usage vs `3` store/data) when JSON output is requested.

- [P1] Normalize “usage error” exit code (`2`) for invalid flag values and missing required inputs, and ensure the JSON error envelope is used consistently when JSON output is requested (examples: invalid `qipu list --since` date, invalid `qipu link --direction`, and `qipu context` with no selection criteria).

- [P1] Align `qipu doctor` store discovery with `specs/cli-tool.md` walk-up rules even in the unchecked-open fallback path (if discovery fails due to validation errors, still walk up and open unchecked at the discovered store root).
- [P1] Fix `qipu doctor --format records` header provenance: set `store=...` to the actual opened store root (not `cli.store` / a hardcoded `.qipu` fallback).

- [P1] Add budget support (`--max-chars` exact) to unbounded records outputs that can be large but currently can’t be bounded (notably `qipu link list --format records`), or explicitly document/justify why a given records output is intentionally unbudgeted.

- [P2] Meet the `specs/cli-tool.md` search performance target: profile and optimize `qipu search`, then tighten `tests/performance_tests.rs` from “no regression” to spec-level budgets (notably ~10k-note search < 1s when indexes are available).
- [P2] Remove non-verbose debug noise on stderr (e.g., the unconditional `Using ripgrep search` / `Using embedded search` prints) so commands remain scriptable and quiet by default.
- [P2] Make `--verbose` timing output consistent across commands (avoid resetting the timing epoch inside individual command handlers such as `sync`; keep stable phase keys/labels).
- [P2] Make golden output path normalization portable (avoid hardcoding a Linux `/tmp/.tmpXXXXXX` pattern in goldens).

- [P3] Optional: decide whether `qipu link add/remove` should require `--type` (as proposed in `specs/cli-interface.md`) or allow a default of `related`, and update spec/implementation accordingly.
- [P3] Optional: implement git automation in `qipu sync` when `store.config().branch` is set (switch branch, commit changes, optional push), guarded behind explicit flags.
- [P3] Optional: implement export attachment copying (e.g., `qipu export --with-attachments`) as documented as a “future enhancement” in `docs/attachments.md`.
- [P3] Optional: remove `qipu sync` placeholder output values by refactoring `doctor` to return structured results in addition to printing them (so sync can report consistent totals in JSON/records modes when `--validate` is used).
