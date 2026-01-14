# Qipu Implementation Plan (Remaining Work)

- [P0] Implement `qipu dump` and `qipu load <pack>` per `specs/pack.md`: add CLI surface, define a pack encoding with an explicit version marker, support selectors (`--note/--tag/--moc/--query`) plus traversal flags (`--direction/--max-hops/--type/--typed-only/--inline-only`), include attachments by default with `--no-attachments`, and ensure load round-trips notes + attachments without lossy transforms.
- [P0] Add pack test coverage: extend `tests/cli_tests.rs` for dump/load behavior (including selectors/traversal + `--no-attachments`), add/update goldens where determinism matters, and update `tests/golden/help.txt` once new commands exist.

- [P0] Implement the `specs/llm-user-validation.md` “LLM primary user” validation harness: add an E2E test runner with pluggable tool adapters (OpenCode first), transcript capture to `tests/transcripts/<tool>/<timestamp>/`, and programmatic validation of resulting store state (notes created, links created, and retrieval works).
- [P0] Add `.gitignore` coverage for volatile transcript artifacts described in `specs/llm-user-validation.md` (at minimum ignore `tests/transcripts/`).

- [P1] Reconcile store discovery requirements across specs: `specs/cli-tool.md` currently describes discovering `.qipu/` only, while `specs/storage-format.md` and the implementation support both `.qipu/` and `qipu/`; decide the intended behavior and update spec(s) accordingly.

- [P1] Fix CLI parse-time JSON behavior to match `specs/cli-tool.md`: ensure usage/parse errors produce structured JSON when the user requests JSON via either `--format json` or `--format=json`.
- [P1] Ensure `qipu --help` and `qipu --version` always exit `0` and print normal help/version output even when `--format json` is present (avoid treating clap help/version as an error path).
- [P1] Normalize “invalid flag value” errors as usage errors (exit code `2`) across commands (e.g., invalid `--since` date, invalid `link --direction`), and ensure the JSON error envelope is used consistently when JSON output is requested.

- [P1] Align `qipu doctor` store discovery with `specs/cli-tool.md` walk-up rules even in the unchecked-open fallback path (if discovery fails due to validation errors, still walk up and open unchecked at the discovered store root).
- [P1] Fix `qipu doctor --format records` header provenance: set `store=...` to the actual opened store root (not `cli.store` / a hardcoded `.qipu` fallback).

- [P2] Meet the `specs/cli-tool.md` search performance target: profile and optimize `qipu search`, then tighten `tests/performance_tests.rs` from “no regression” to spec-level budgets (notably ~10k-note search < 1s when indexes are available).
- [P2] Make `--verbose` timing output consistent across commands (avoid resetting the timing epoch inside individual command handlers such as `sync`; keep stable phase keys/labels).
- [P2] Make golden output path normalization portable (avoid hardcoding a Linux `/tmp/.tmpXXXXXX` pattern in goldens).
- [P2] Add budget support (`--max-chars` exact) to records outputs that can be large but currently can’t be bounded (notably `qipu link list --format records`), or explicitly document/justify why a given records output is intentionally unbudgeted.
- [P2] Remove non-verbose debug noise on stderr (e.g., the unconditional `Using ripgrep search` / `Using embedded search` prints) so commands remain scriptable and quiet by default.

- [P3] Optional: implement git automation in `qipu sync` when `store.config().branch` is set (switch branch, commit changes, optional push), guarded behind explicit flags.
- [P3] Optional: implement export attachment copying (e.g., `qipu export --with-attachments`) as documented as a “future enhancement” in `docs/attachments.md`.
