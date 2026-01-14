# Qipu Implementation Plan (Remaining Work)

- [P0] Implement `qipu dump` and `qipu load <pack>` per `specs/pack.md`: add CLI surface (`dump`, `load`), define a pack encoding with an explicit version marker, support selectors (`--note/--tag/--moc/--query`) plus traversal flags (`--direction/--max-hops/--type/--typed-only/--inline-only`), include attachments by default with `--no-attachments`, and ensure load round-trips notes + attachments without lossy transforms.
- [P0] Add pack test coverage: extend `tests/cli_tests.rs` for dump/load behavior (including selectors/traversal + `--no-attachments`), add/update goldens where determinism matters, and update `tests/golden/help.txt` once new commands exist.

- [P0] Implement the `specs/llm-user-validation.md` “LLM primary user” validation harness: add an E2E test runner with pluggable tool adapters (OpenCode first), transcript capture to `tests/transcripts/<tool>/<timestamp>/`, and programmatic validation of resulting store state (notes created, links created, and retrieval works).
- [P0] Add `.gitignore` coverage for volatile transcript artifacts described in `specs/llm-user-validation.md` (at minimum ignore `tests/transcripts/`).

- [P0] Fix determinism violations in command outputs:
  - Remove or make stable any runtime timestamps in outputs expected to be deterministic (e.g., `generated_at: Utc::now()` in `qipu context` and `qipu prime` JSON, and corresponding human output).
  - Ensure backend selection / progress text does not leak to stderr in non-verbose modes (e.g., `Using ripgrep search` / `Using embedded search`).

- [P0] Fix CLI parse-time JSON behavior to match `specs/cli-tool.md`:
  - Detect JSON requests in argv for both `--format json` and `--format=json`.
  - Ensure `qipu --help` and `qipu --version` always exit `0` and print normal help/version output even when `--format json` is present (avoid wrapping clap help/version in JSON error paths).
  - Ensure usage/parse errors emit a structured JSON envelope with correct exit code (`2` usage vs `3` store/data) when JSON output is requested.

- [P0] Normalize “usage error” exit code (`2`) for invalid flag values and missing required inputs, and ensure the JSON error envelope is used consistently when JSON output is requested (examples: invalid `qipu list --since` date, invalid `qipu link --direction`, `qipu context` with no selection criteria, invalid `qipu export --mode`, and `qipu export` with no selection criteria).

- [P1] Bring compaction resolution in line with `specs/compaction.md` across command surfaces:
  - `qipu show`: default to resolved view (if an ID is compacted, redirect to `canon(id)`), and only show raw compacted notes when `--no-resolve-compaction` is set.
  - `qipu show --links`: apply contracted-graph semantics (canonicalize nodes/edges; hide compacted sources by default).
  - `qipu context`: in resolved view, canonicalize + deduplicate selected IDs (including `--query` hits), hide compacted source notes by default, and preserve a `via`/breadcrumb mechanism for query-driven selection.
  - `qipu export`: apply resolved/raw toggle and decide deterministic ordering behavior when MOCs link to compacted notes (export canonical digest vs raw source, and how to annotate).
  - `qipu link` list/tree/path: aggregate edges for a canonical digest across the full compaction equivalence class (transitive chains), not just direct `compacts`.

- [P1] Align `qipu context --moc <id>` behavior with `specs/cli-interface.md`: include the MOC itself (not just its linked notes), and ensure ordering/dedup remain deterministic.

- [P1] Reconcile store discovery requirements across specs: `specs/cli-tool.md` currently describes discovering `.qipu/` only, while `specs/storage-format.md`, `specs/cli-interface.md`, and the implementation support both `.qipu/` and `qipu/`; decide the intended behavior and update spec(s) accordingly.

- [P1] Fix records header provenance (`store=...`) to always report the actual opened store root (not `cli.store` or a hardcoded `.qipu` fallback), including at least `qipu doctor --format records` and `qipu index --format records`.

- [P1] Make search output fully deterministic:
  - Add stable tiebreakers in result sorting (e.g., when relevance ties, sort by `(relevance desc, id asc)`), and ensure canonicalization/dedup does not introduce unstable ordering.

- [P1] Meet `specs/records-output.md` budgeting requirements:
  - Ensure `--max-chars` is exact where supported (not “best effort”), and ensure truncation is indicated consistently (`truncated=true` and/or an explicit truncation record).
  - Add budget support (`--max-chars`) to unbounded records outputs that can be large but currently can’t be bounded (notably `qipu link list --format records`), or explicitly document/justify why a given records output is intentionally unbudgeted.

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
