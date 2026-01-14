# Qipu Implementation Plan (Remaining Work)

- [P0] Implement `qipu dump` and `qipu load` per `specs/pack.md` (no CLI surface exists today): define a pack file encoding + version marker, support selectors (`--note/--tag/--moc/--query`), support traversal-based selection flags, include attachments by default with `--no-attachments`, and ensure round-trip load restores notes + attachments without lossy transforms.
- [P0] Add test coverage for pack: extend `tests/cli_tests.rs` for dump/load behavior, add/update goldens where determinism matters, and update `tests/golden/help.txt` once new commands exist.

- [P0] Implement the `specs/llm-user-validation.md` “LLM primary user” validation harness (currently only a spec): add an E2E test runner with pluggable tool adapters (OpenCode first), transcript capture to `tests/transcripts/<tool>/<timestamp>/`, and programmatic validation of resulting store state.
- [P0] Add `.gitignore` coverage for volatile transcript artifacts described in `specs/llm-user-validation.md`.

- [P1] Fix CLI parse-time JSON behavior to match `specs/cli-tool.md`: ensure usage/parse errors produce structured JSON when the user requests JSON via either `--format json` or `--format=json`.
- [P1] Ensure `qipu --help` and `qipu --version` always exit `0` and print normal help/version output even when `--format json` is present (avoid treating clap help/version as an error path).

- [P1] Align `qipu doctor` store discovery with `specs/cli-tool.md` walk-up rules even in the “unchecked-open” fallback path (currently only checks the immediate `--root` for `.qipu/` or `qipu/`).
- [P1] Fix `qipu doctor --format records` header provenance: use the actual opened store root in `store=...` (avoid reporting `cli.store` or a hardcoded `.qipu` fallback).

- [P1] Remove `qipu sync --validate` placeholder validation counts by refactoring `doctor` to expose a structured, non-printing API (errors/warnings/fixed) and consuming it from `sync` for consistent human/json/records output.

- [P2] Meet the `specs/cli-tool.md` search performance target: profile and optimize `qipu search` and tighten `tests/performance_tests.rs` assertions from “no regression” to spec-level budgets.
- [P2] Make `--verbose` timing output consistent across commands (avoid resetting the timing epoch inside individual command handlers such as `sync`).
- [P2] Make golden output path normalization portable (avoid hardcoding a Linux `/tmp/.tmpXXXXXX` pattern in goldens).

- [P3] Optional: implement git automation in `qipu sync` when `store.config().branch` is set (switch branch, commit changes, optional push), guarded behind explicit flags.
- [P3] Optional: implement export attachment copying (e.g., `qipu export --with-attachments`) as documented as a “future enhancement” in `docs/attachments.md`.
